"""World identification probe (GOAL_1_2.md 9.3, METRICS_1_2.md Z5).

For a trained M1b on a pilot dataset: the model's final layer-normed state (`M1TF.hidden`) at row s = 166 (06:00
of display day 8) after the window of rows s-167 .. s (row -1 masked, as the predictor's warm-up masks rows < 0),
for every train / val / test episode; a ridge from that state to each hidden world parameter of `meta.json`'s
`world_variety` (labels only, read here after training, never an input), lambda chosen on val (MSE), R^2 on test
(1 - SSE / SS around the test mean). `climate` (a 0-2 category): a one-vs-rest ridge (one-hot targets, lambda on
val one-hot MSE), accuracy of the argmax on test. Reference: the same ridges from the raw observable state row at s
(the 35 columns of the observable table). `starting_agents` equals `alive` at row 0, so the reference recovers it
almost trivially from the row-166 population.

GOAL_1_3.md 9.4 (Y6, game_v17): `PARAMS` gains the region parameters (`wild_regions`, `region_bushes`,
`region_deadwood`; a parameter absent from a dataset's `world_variety` is skipped, so p10 runs probe the seven of
WM-1.2), the train role keeps the dataset's `train_seeds`, and on a dataset whose table carries `g.fissions_total` and
`g.settlements_alive` three labels are read from those observable columns after training (labels only, never an
input of the probe window, which ends at row 166): fission by day 120 (`fissions_total` > 0 at the last row; argmax
of a one-vs-rest ridge, accuracy against the train majority class), `settlements_alive` at row 2,832 (ridge R^2), and
the first-fission display day among the worlds that split (ridge R^2, only when its test variance is above 0).

usage (DGX, venv): python3 tools/worldmodel/wm1/probe_world.py --run RUN_DIR --out OUT_DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import calcore as cc
from wm1 import common as c
from wm1 import data as d

PARAMS = ("starting_agents", "berry_bushes", "berry_capacity", "deadwood_spots", "deadwood_capacity",
          "forest_permille", "food_regen_permille", "wild_regions", "region_bushes", "region_deadwood")
CATEGORY = "climate"
CLIMATES = (0, 1, 2)
LAMBDAS = (0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0)
DAY, MINUTE = 8, 360
WINDOW = 168
SETTLEMENTS_ROW = 2832
FISSIONS, SETTLEMENTS = "g.fissions_total", "g.settlements_alive"


def probe_row() -> int:
    r = cc.row_of(DAY, MINUTE)
    assert r == 166, r
    return r


def r2(y: np.ndarray, pred: np.ndarray) -> float | None:
    sst = float(((y - y.mean()) ** 2).sum())
    return None if sst == 0 else 1.0 - float(((y - pred) ** 2).sum()) / sst


def fit_select(xtr, ytr, xva, yva):
    """Ridge (bench_v0.Ridge: standardized x, unpenalized intercept) with lambda from LAMBDAS by val MSE."""
    best, scores = None, {}
    for lam in LAMBDAS:
        m = b.Ridge().fit(xtr, ytr, lam)
        mse = float(((m.predict(xva) - yva) ** 2).mean())
        scores[str(lam)] = mse
        if best is None or mse < best[0]:
            best = (mse, lam, m)
    return best[2], best[1], scores


def classify(x: dict, y: dict, classes: tuple) -> dict:
    """One-vs-rest ridge on one-hot targets (lambda on val one-hot MSE); argmax accuracy on test and the accuracy of
    the train majority class."""
    hot = {r: b.onehot(y[r].astype(np.int64), list(classes)) for r in y}
    m, lam, scores = fit_select(x["train"], hot["train"], x["val"], hot["val"])
    pred = np.array(classes)[m.predict(x["test"]).argmax(axis=1)]
    truth = y["test"]
    majority = np.array(classes)[np.array([np.sum(y["train"] == k) for k in classes]).argmax()]
    return {"accuracy_test": float(np.mean(pred == truth)), "lambda": lam, "val_mse": scores,
            "majority_train_class_accuracy_test": float(np.mean(truth == majority)),
            "test_counts": [int(np.sum(truth == k)) for k in classes]}


def regress(x: dict, y: dict) -> dict:
    m, lam, scores = fit_select(x["train"], y["train"][:, None], x["val"], y["val"][:, None])
    return {"r2_test": r2(y["test"], m.predict(x["test"])[:, 0]), "lambda": lam, "val_mse": scores,
            "test_variance": float(np.var(y["test"]))}


def first_split_day(fissions: np.ndarray) -> float:
    """Display day of the first row with `fissions_total` > 0 (NaN when the world never splits)."""
    rows = np.nonzero(fissions > 0)[0]
    return float((rows[0] * b.EVERY + b.OFFSET) // b.TPD + 1) if len(rows) else float("nan")


def observable_labels(eps: list[d.EpisodeData]) -> dict[str, np.ndarray] | None:
    """Y6's labels from observable columns of the whole episode (read after training; labels only), or None when the
    table has no fission columns."""
    if not eps or FISSIONS not in eps[0].names or SETTLEMENTS not in eps[0].names:
        return None
    fis = [ep.table[:, ep.names.index(FISSIONS)] for ep in eps]
    return {"fission_by_day120": np.array([float(f[-1] > 0) for f in fis]),
            "settlements_alive_row2832": np.array([ep.table[SETTLEMENTS_ROW, ep.names.index(SETTLEMENTS)] for ep in eps]),
            "first_fission_day": np.array([first_split_day(f) for f in fis])}


def probe_observable(x: dict, labels: dict) -> dict:
    """The three Y6 labels: fission yes / no (classification), settlements alive (R^2), first-fission day (R^2 on
    the worlds that split, only when its test variance is above 0)."""
    out = {"fission_by_day120": classify(x, {r: labels[r]["fission_by_day120"] for r in x}, (0, 1)),
           "settlements_alive_row2832": regress(x, {r: labels[r]["settlements_alive_row2832"] for r in x})}
    keep = {r: np.isfinite(labels[r]["first_fission_day"]) for r in x}
    days = {r: labels[r]["first_fission_day"][keep[r]] for r in x}
    counts = {r: int(keep[r].sum()) for r in x}
    variance = float(np.var(days["test"])) if counts["test"] else 0.0
    if variance > 0 and min(counts.values()) >= 2:
        out["first_fission_day"] = regress({r: x[r][keep[r]] for r in x}, days) | {"worlds": counts}
        # METRICS_1_3.md amendment (review round 1, F1): also the R^2 on the test worlds whose first split lies inside
        # the train range of first-split days (a world splitting later than any train world is reported apart).
        hi = float(days["train"].max())
        inside = days["test"] <= hi
        m, lam, _ = fit_select(x["train"][keep["train"]], days["train"][:, None], x["val"][keep["val"]],
                               days["val"][:, None])
        pred = m.predict(x["test"][keep["test"]])[:, 0]
        out["first_fission_day"] |= {"train_range": [float(days["train"].min()), hi],
                                     "test_outside_train_range": sorted(float(v) for v in days["test"][~inside]),
                                     "r2_test_inside_train_range": r2(days["test"][inside], pred[inside])}
    else:
        out["first_fission_day"] = {"skipped": "test variance 0 (or too few worlds that split)", "worlds": counts,
                                    "test_variance": variance,
                                    "test_values": sorted({float(v) for v in days["test"]})}
    return out


def probe(x: dict[str, np.ndarray], y: dict[str, dict[str, np.ndarray]]) -> dict:
    """x[role] [E, D]; y[role][param] [E]. R^2 per parameter on test, climate accuracy on test (a parameter missing
    from the labels is skipped)."""
    out = {}
    for p in (p for p in PARAMS if p in y["train"]):
        m, lam, scores = fit_select(x["train"], y["train"][p][:, None], x["val"], y["val"][p][:, None])
        pred = m.predict(x["test"])[:, 0]
        out[p] = {"r2_test": r2(y["test"][p], pred), "lambda": lam, "val_mse": scores,
                  "train_values": sorted({float(v) for v in y["train"][p]})[:3] + ["..."]}
    hot = {r: b.onehot(y[r][CATEGORY].astype(np.int64), list(CLIMATES)) for r in y}
    m, lam, scores = fit_select(x["train"], hot["train"], x["val"], hot["val"])
    pred = np.array(CLIMATES)[m.predict(x["test"]).argmax(axis=1)]
    truth = y["test"][CATEGORY]
    out[CATEGORY] = {"accuracy_test": float(np.mean(pred == truth)), "lambda": lam, "val_mse": scores,
                     "majority_train_class_accuracy_test": float(np.mean(truth == np.bincount(
                         y["train"][CATEGORY].astype(np.int64), minlength=3).argmax())),
                     "test_counts": np.bincount(truth.astype(np.int64), minlength=3).tolist()}
    return out


def labels(root: Path, ds: str, role: str) -> tuple[list[int], dict[str, np.ndarray]]:
    """world_variety of every episode of a role (the train role keeps the dataset's `train_seeds`), in manifest
    order (meta.json read directly: labels only); the parameters every episode carries."""
    spec = c.dataset(ds)
    split, _ = spec.groups[role]
    keep = c.group_seeds(spec, role)
    seeds, rows = [], []
    for e in c.manifest(ds)["episodes"]:
        if e["split"] == split and e["role"] == role and (keep is None or int(e["seed"]) in keep):
            meta = json.loads((root / e["path"] / "meta.json").read_text())
            seeds.append(int(e["seed"]))
            rows.append(meta["world_variety"]["values"])
    names = [k for k in PARAMS + (CATEGORY,) if all(k in r for r in rows)]
    return seeds, {k: np.array([r[k] for r in rows], dtype=np.float64) for k in names}


def hidden_states(pred, eps: list[d.EpisodeData], s: int) -> np.ndarray:
    import torch
    out = []
    for ep in eps:
        prep = d.prepare(ep, pred.variant, pred.norm)
        rows = np.arange(s - (WINDOW - 1), s + 1)
        valid = rows >= 0
        r = np.clip(rows, 0, None)
        inp = np.concatenate([prep.x_state[r], prep.calendar[r], prep.actions[r], prep.sev[r]], axis=-1)
        tt = lambda a, dt=torch.float32: torch.tensor(np.asarray(a)[None], dtype=dt, device=pred.device)
        pred.model.eval()
        out.append(pred.model.hidden(tt(inp), tt(valid, torch.bool))[0].double().cpu().numpy())
    return np.stack(out)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    args = ap.parse_args(argv)
    from wm1 import m1_gru
    t0 = time.time()
    run, target = Path(args.run), Path(args.out) / "probe.json"
    if target.exists():
        raise SystemExit(f"{target} exists: test numbers are computed once")
    pred, config = m1_gru.load_predictor(run, args.device)
    ds = config["dataset"]
    if config.get("model") != "m1_tf":
        raise SystemExit("the probe reads M1TF.hidden")
    s = probe_row()
    g = c.load_groups(["train", "val", "test"], Path(args.root), ds)
    xs, xr, ys, obs = {}, {}, {}, {}
    for role in ("train", "val", "test"):
        seeds, ys[role] = labels(Path(args.root), ds, role)
        if seeds != [int(ep.seed) for ep in g[role]]:
            raise SystemExit(f"{role}: label order differs from the loaded episodes")
        xs[role] = hidden_states(pred, g[role], s)
        xr[role] = np.stack([ep.state(pred.variant)[s] for ep in g[role]])
        obs[role] = observable_labels(g[role])
    out = {"run": str(run), "dataset": ds, "row": s, "window": [s - WINDOW + 1, s],
           "hidden_dim": int(xs["train"].shape[1]), "reference_columns": list(pred.variant.state_names),
           "model": probe(xs, ys), "reference_raw_row": probe(xr, ys),
           "counts": {r: len(g[r]) for r in g}, "code_commit": c.git_commit(),
           "run_code_commit": config["code_commit"], "manifest_sha256": c.manifest_sha(ds), "argv": sys.argv,
           "seconds": time.time() - t0}
    if obs["train"] is not None:
        out["model_observable"] = probe_observable(xs, obs)
        out["reference_observable"] = probe_observable(xr, obs)
        out["observable_label_rows"] = {"settlements_alive": SETTLEMENTS_ROW, "fission_by_day120": "last row"}
        out["seconds"] = time.time() - t0
    c.write_json(target, out)
    print(json.dumps({p: (v.get("r2_test", v.get("accuracy_test"))) for p, v in out["model"].items()}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
