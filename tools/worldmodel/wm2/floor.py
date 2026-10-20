"""The noise floor (GOAL.md 7.3) and the floor-matched scores of the predictors. numpy only (models enter as
`evaluate.Predictor`s).

A floor family is one test world forked at a start tick s with R = 4 factual replicates (`none_r0 .. none_r3`; replicate
0 = the unforked factual). For each start s, horizon h in {24, 168} hours and each of the 8 V variables j
(`bench_v0.VARS`): the replicate values y_r at row(s) + h; the floor error of replicate r is
|y_r - mean of the other replicates| x sqrt((R - 1) / R) (the uncorrected value is recorded too), divided by the test
split std of j (`bench_v0.split_std` of the factual test worlds). The **floor score** is the mean over worlds, starts,
variables and replicates (variables with a zero std are skipped, as `bench_v0.nmae` does).

This estimates the error of the conditional-*mean* predictor (the predictor that knows the mean over the noise after
s), exactly in expectation only for Gaussian noise (an approximation for integer columns such as `alive`). The
MAE-optimal predictor is the conditional median, so the true MAE floor is at or below this estimate and the headroom
(a predictor's score minus the floor) is a lower bound.

Every predictor is scored at exactly these starts and horizons on replicate 0 (the factual): the **floor-matched
score** |prediction - y_0| / std, averaged the same way (over worlds, starts and variables). Per-world scores (mean over
starts and variables) feed the world-level bootstrap.

usage (DGX):
  python3 tools/worldmodel/wm2/floor.py --scene game_v19 --dataset v2_v19_fact --selection CAL_VAL_JSON \
      [--run RUN ...] --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
import bench_v0 as b
import bench_v1 as bv1
from wm_data import load_family
from wm1 import bootstrap as bs
from wm1 import common as c
from wm1 import data as d
from wm1 import evaluate as ev
from wm2 import v2

ARRAYS = ["global.npy", "settlements.npy", "settlement_ids.npy"]
SCENE_DIR = {"game_v19": "v19", "game_v16": "v16"}


def floor_errors(y: np.ndarray, std: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """y [R, J] replicate values -> (corrected, uncorrected) floor errors [R, J] divided by std (NaN where std = 0)."""
    y = np.asarray(y, dtype=np.float64)
    r = y.shape[0]
    if r < 2:
        raise ValueError("the floor needs at least two replicates")
    others = (y.sum(axis=0, keepdims=True) - y) / (r - 1)
    raw = np.abs(y - others)
    with np.errstate(invalid="ignore", divide="ignore"):
        inv = np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), np.nan)
    return raw * np.sqrt((r - 1) / r) * inv, raw * inv


def floor_scores(values: dict, std: np.ndarray) -> dict:
    """`values[(seed, s, h)]` = [R, 8] replicate values -> per horizon: the floor (corrected and uncorrected), per
    variable, per start, and per world (mean over starts, variables, replicates)."""
    out = {}
    for h in sorted({k[2] for k in values}):
        keys = sorted(k for k in values if k[2] == h)
        corr, unc = zip(*[floor_errors(values[k], std) for k in keys])
        corr, unc = np.stack(corr), np.stack(unc)              # [n, R, 8]
        per_world: dict[str, list] = {}
        for k, e in zip(keys, corr):
            per_world.setdefault(str(k[0]), []).append(e)
        out[str(h)] = {"floor": float(np.nanmean(corr)), "floor_uncorrected": float(np.nanmean(unc)),
                       "per_var": np.nanmean(corr, axis=(0, 1)).tolist(),
                       "per_start": {str(s): float(np.nanmean(corr[[i for i, k in enumerate(keys) if k[1] == s]]))
                                     for s in sorted({k[1] for k in keys})},
                       "per_world": {w: float(np.nanmean(np.stack(v))) for w, v in per_world.items()},
                       "n": len(keys)}
    return out


def matched_scores(pred: ev.Predictor, eps: dict, values: dict, std: np.ndarray) -> dict:
    """A predictor's floor-matched scores: from row(s) of each world's replicate 0 (`eps[(seed, s)]`, an
    `EpisodeData`), |prediction at row(s) + h - y_0| / std, averaged over worlds, starts and variables."""
    out = {}
    for h in sorted({k[2] for k in values}):
        keys = sorted(k for k in values if k[2] == h)
        errs, per_world = [], {}
        for seed, s, _ in keys:
            ep = eps[(seed, s)]
            row = v2.row_of_tick(s)
            ro = ev.run_rollouts(pred, ep, np.array([row]), h)
            y0 = values[(seed, s, h)][0]
            if not np.array_equal(y0, ep.bench.v[row + h]):
                raise ValueError(f"seed {seed} s {s}: replicate 0 differs from the episode the predictor rolls")
            with np.errstate(invalid="ignore", divide="ignore"):
                e = np.abs(ro.v[0, -1] - y0) * np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), np.nan)
            errs.append(e)
            per_world.setdefault(str(seed), []).append(e)
        errs = np.stack(errs)
        out[str(h)] = {"score": float(np.nanmean(errs)), "per_var": np.nanmean(errs, axis=0).tolist(),
                       "per_world": {w: float(np.nanmean(np.stack(v))) for w, v in per_world.items()}, "n": len(keys)}
    return out


def load_floor(root: Path, scene: str, extra: tuple) -> tuple[dict, dict, dict]:
    """(values[(seed, s, h)] = [R, 8], eps[(seed, s)] = replicate 0 as EpisodeData, meta per family)."""
    values, eps, info = {}, {}, {}
    lo, hi = v2.ROLES[scene]["test"]
    for s in v2.FLOOR_STARTS:
        fam_root = Path(root) / "v2" / SCENE_DIR[scene] / f"floor_s{s}" / "fam"
        row = v2.row_of_tick(s)
        for seed in range(lo, hi):
            fam = load_family(fam_root / f"{scene}_s{seed}", verify=True, arrays=ARRAYS)
            if fam.fork_tick != s or fam.replicates != v2.FLOOR_R or fam.variants:
                raise ValueError(f"{fam.path}: not a factual R = {v2.FLOOR_R} family forked at {s}")
            v = [bv1.episode_arrays(fam.factual[k])["v"] for k in range(fam.replicates)]
            for h in v2.FLOOR_HORIZONS:
                values[(seed, s, h)] = np.stack([x[row + h] for x in v])
            eps[(seed, s)] = d.from_episode(fam.factual[0], f"{SCENE_DIR[scene]}_floor", "test", seed, "none", extra)
            info[f"{seed}@{s}"] = {"replicates": fam.replicates}
    return values, eps, info


def main(argv=None) -> int:
    from wm1 import climatology as cl
    from wm1 import m1_gru as g
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--scene", required=True, choices=sorted(SCENE_DIR))
    ap.add_argument("--dataset", required=True, help="the factual dataset (split std, calendar fit): v2_v19_fact / v2_v16")
    ap.add_argument("--selection", required=True, help="calendar_val.json of that dataset (cal_best per horizon)")
    ap.add_argument("--run", nargs="*", default=[], help="M1 run directories (best.pt)")
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    t0 = time.time()
    ds = c.dataset(a.dataset)
    g_ = c.load_groups(["train", "test"], Path(a.root), ds.name)
    std = b.split_std([ep.bench for ep in g_["test"]])
    values, eps, info = load_floor(Path(a.root), a.scene, ds.extra_state)
    print(f"floor families {len(info)} {time.time() - t0:.0f}s", flush=True)
    out = {"scene": a.scene, "dataset": ds.name, "test_std": std.tolist(), "floor": floor_scores(values, std),
           "families": len(info), "code_commit": c.git_commit(), "argv": sys.argv, "predictors": {}}
    fit = cl.CalendarFit(g_["train"], cl.T_TR, split=ds.groups["train"][0])
    sel = json.loads(Path(a.selection).read_text())["selection"]
    if json.loads(Path(a.selection).read_text()).get("dataset") != ds.name:
        raise SystemExit("the calendar selection is of another dataset")
    members = cl.predictors(fit)
    preds = {"persistence": ev.Persistence(), "an5": members["an5"]}
    chosen = {h: sel[f"T2any_{h}"]["cal_best"] for h in v2.FLOOR_HORIZONS}
    out["cal_best"] = {str(h): m for h, m in chosen.items()}
    for h, m in chosen.items():
        preds[f"cal_best@{h}"] = members[m]
    for run in a.run:
        p, config = g.load_predictor(Path(run), a.device)
        preds[Path(run).name] = p
    for name, p in preds.items():
        sc = matched_scores(p, eps, values, std)
        if name.startswith("cal_best@"):
            h = name.split("@")[1]
            out["predictors"].setdefault("cal_best", {})[h] = sc[h]
        else:
            out["predictors"][name] = sc
        print(f"{name} {time.time() - t0:.0f}s", flush=True)
    out["headroom"] = {}
    for h in map(str, v2.FLOOR_HORIZONS):
        cb, fl = out["predictors"]["cal_best"][h]["per_world"], out["floor"][h]["per_world"]
        out["headroom"][h] = bs.paired(cb, fl) | {"cal_best": out["predictors"]["cal_best"][h]["score"],
                                                 "floor": out["floor"][h]["floor"]}
    out["seconds"] = time.time() - t0
    c.write_json(Path(a.out) / "floor.json", out)
    print(json.dumps({"floor": {h: v["floor"] for h, v in out["floor"].items()}, "headroom": out["headroom"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
