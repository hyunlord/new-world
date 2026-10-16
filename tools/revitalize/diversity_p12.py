#!/usr/bin/env python3
"""Pass 12: a diversity measure that tells levels from histories (PASS12_GOAL.md 5.2). numpy only.

For a scene's recordings (every episode), variable j and display day d in {30, 60, 90, 118}:

- y_e = mean of v_e[j] over the rows of display day d (rows r with (60 r + 480) // 1440 + 1 = d,
  `calcore_wm11.display_day`, one-based).
- features x_e = the episode's hidden world parameters (`meta.json` `world_variety` values: every slot except
  `climate`, plus a `climate` one-hot; `game_v15` has none) and m_e = mean of v_e[j] over display days 3..10.
  Features with zero std in the scene are dropped; the rest are standardized over the episodes in use.
- ridge (intercept unpenalized) with lambda chosen by leave-one-out MSE from LAMBDAS; LOO predictions yhat_e.
- R2_static(j, d) = 1 - sum (y - yhat)^2 / sum (y - ybar)^2.
- UBS(j, d) = sqrt(mean_e (y_e - yhat_e)^2) / S_j, S_j = `bench_v1.split_std` of the scene's episodes (std of v_j
  over rows >= 24 of every episode).

A linear ridge counts a nonlinear static effect as unexplained (UBS > 0 although the history adds nothing);
`test_diversity_p12.py` measures that on a synthetic quadratic map.

Paired bootstrap of delta UBS (scene B - scene A, same seeds): resample seeds with replacement 1,000 times
(`numpy.random.default_rng(0)`), refit both scenes' ridges at the lambda chosen on the full set, leaving out every
copy of a seed together (a duplicated seed never predicts itself), recompute S_j on the resample; report the 5 % and
95 % percentiles of the mean over days 60/90/118.

usage:
  diversity_p12.py score --root DIR --scenes a,b --seeds 22000..22112 --out DIR [--test 22096..22112]
                         [--bootstrap A:B ...] [--split-subsets SCENE] [--label TEXT] [--append]
"""
from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np

import diversity_p10 as d10
import diversity_p11 as d11

cc, bv, wm_data = d10.cc, d10.bv, d10.wm_data
VARS = ("food_stock", "wood_stock", "alive", "deadwood_total", "food_on_bushes_total", "adults_working")
DAYS = (30, 60, 90, 118)
MEAN_DAYS = (60, 90, 118)
LAMBDAS = (0.1, 1.0, 10.0, 100.0, 1000.0)
EARLY = (3, 10)
N_BOOT = 1000


def day_means(v: np.ndarray, days) -> np.ndarray:
    """v[T, J] -> [len(days), J]: the mean over the rows of each one-based display day."""
    dd = cc.display_day(np.arange(v.shape[0]))
    out = []
    for d in days:
        rows = dd == d
        if not rows.any():
            raise SystemExit(f"no rows of display day {d}")
        out.append(v[rows].mean(axis=0))
    return np.array(out)


def early_mean(v: np.ndarray) -> np.ndarray:
    dd = cc.display_day(np.arange(v.shape[0]))
    rows = (dd >= EARLY[0]) & (dd <= EARLY[1])
    return v[rows].mean(axis=0)


def variety_matrix(varieties: list) -> tuple[np.ndarray, list[str]]:
    """Hidden-parameter features: every slot except climate, plus a climate one-hot. [E, F]."""
    if not varieties or any(v is None for v in varieties):
        return np.zeros((len(varieties), 0)), []
    keys = sorted(k for k in varieties[0] if k != "climate")
    cols = [[float(v[k]) for k in keys] for v in varieties]
    names = list(keys)
    if "climate" in varieties[0]:
        for c in range(3):
            for row, v in zip(cols, varieties):
                row.append(1.0 if int(v["climate"]) == c else 0.0)
            names.append(f"climate_{c}")
    return np.array(cols, dtype=np.float64), names


def loo_predict(x: np.ndarray, y: np.ndarray, lam: float, groups: np.ndarray | None = None) -> np.ndarray:
    """Leave-one-out (or leave-one-group-out) ridge predictions, intercept unpenalized.

    Features are standardized over all rows in `x` (zero-std columns dropped); each fold solves
    (Xa'Xa - Xg'Xg + lam D) b = Xa'y - Xg'yg with Xa = [1, X] and D = diag(0, 1, ..., 1)."""
    n = len(y)
    std = x.std(axis=0)
    keep = std > 0
    z = (x[:, keep] - x[:, keep].mean(axis=0)) / std[keep] if keep.any() else np.zeros((n, 0))
    xa = np.hstack([np.ones((n, 1)), z])
    p = xa.shape[1]
    pen = lam * np.eye(p)
    pen[0, 0] = 0.0
    if groups is None:
        groups = np.arange(n)
    labels, inv = np.unique(groups, return_inverse=True)
    g = len(labels)
    xtx = xa.T @ xa
    xty = xa.T @ y
    # per-group sums of outer products and of x*y
    onehot = np.zeros((g, n))
    onehot[inv, np.arange(n)] = 1.0
    gxx = np.einsum("gn,ni,nj->gij", onehot, xa, xa)
    gxy = np.einsum("gn,ni,n->gi", onehot, xa, y)
    a = xtx[None] - gxx + pen[None]
    b = xty[None] - gxy
    coef = np.linalg.solve(a, b[..., None])[..., 0]  # [G, p]
    return np.einsum("ni,ni->n", xa, coef[inv])


def choose_lambda(x: np.ndarray, y: np.ndarray) -> tuple[float, np.ndarray]:
    best = None
    for lam in LAMBDAS:
        pred = loo_predict(x, y, lam)
        mse = float(np.mean((y - pred) ** 2))
        if best is None or mse < best[0] - 1e-15:
            best = (mse, lam, pred)
    return best[1], best[2]


class Scene:
    """Per-episode day means, early means, hidden features and the split std of one scene."""

    def __init__(self, eps: list):
        self.seeds = [int(e.seed) for e in eps]
        self.v = [e.v for e in eps]
        j = [bv.VARS.index(n) for n in VARS]
        self.j = j
        self.y = np.stack([day_means(e.v, DAYS)[:, j] for e in eps])  # [E, D, J]
        self.m = np.stack([early_mean(e.v)[j] for e in eps])  # [E, J]
        self.x_static, self.feature_names = variety_matrix([e.variety for e in eps])
        self.rows24 = [e.v[24:][:, j] for e in eps]

    def split_std(self, index: np.ndarray | None = None) -> np.ndarray:
        idx = range(len(self.v)) if index is None else index
        return np.vstack([self.rows24[i] for i in idx]).std(axis=0)

    def features(self, jj: int, index: np.ndarray | None = None) -> np.ndarray:
        idx = np.arange(len(self.v)) if index is None else index
        return np.hstack([self.x_static[idx], self.m[idx, jj:jj + 1]])


def score_scene(sc: Scene, index: np.ndarray | None = None) -> dict:
    """R2_static and UBS per variable and day (+ mean over MEAN_DAYS), lambdas chosen on these episodes."""
    idx = np.arange(len(sc.v)) if index is None else index
    s = sc.split_std(idx)
    out = {"n": int(len(idx)), "features": sc.feature_names + ["early_mean"], "split_std": {}, "lambda": {},
           "r2_static": {}, "ubs": {}, "r2_static_mean": {}, "ubs_mean": {}}
    for jj, name in enumerate(VARS):
        x = sc.features(jj, idx)
        out["split_std"][name] = float(s[jj])
        lam_d, r2_d, ubs_d = {}, {}, {}
        for di, d in enumerate(DAYS):
            y = sc.y[idx, di, jj]
            lam, pred = choose_lambda(x, y)
            sse = float(np.sum((y - pred) ** 2))
            sst = float(np.sum((y - y.mean()) ** 2))
            lam_d[str(d)] = lam
            r2_d[str(d)] = None if sst <= 0 else 1.0 - sse / sst
            ubs_d[str(d)] = None if s[jj] <= 0 else float(np.sqrt(sse / len(y)) / s[jj])
        out["lambda"][name], out["r2_static"][name], out["ubs"][name] = lam_d, r2_d, ubs_d
        mean = lambda dct: (None if any(dct[str(d)] is None for d in MEAN_DAYS)
                            else float(np.mean([dct[str(d)] for d in MEAN_DAYS])))
        out["r2_static_mean"][name] = mean(r2_d)
        out["ubs_mean"][name] = mean(ubs_d)
    return out


def ubs_resample(sc: Scene, block: dict, draw: np.ndarray) -> np.ndarray:
    """Mean UBS over MEAN_DAYS for each variable on the resampled episodes `draw` (indices, with repeats), at the
    lambdas in `block`, leaving out every copy of a seed together."""
    s = sc.split_std(draw)
    out = np.full(len(VARS), np.nan)
    for jj, name in enumerate(VARS):
        if s[jj] <= 0:
            continue
        x = sc.features(jj, draw)
        vals = []
        for d in MEAN_DAYS:
            y = sc.y[draw, DAYS.index(d), jj]
            pred = loo_predict(x, y, block["lambda"][name][str(d)], groups=draw)
            vals.append(np.sqrt(np.mean((y - pred) ** 2)) / s[jj])
        out[jj] = float(np.mean(vals))
    return out


def paired_bootstrap(a: Scene, block_a: dict, b: Scene, block_b: dict, n_boot: int = N_BOOT) -> dict:
    """90 % percentile interval of delta mean-UBS (b - a) over seeds resampled with replacement."""
    if a.seeds != b.seeds:
        raise SystemExit("paired bootstrap needs the same seeds in the same order")
    rng = np.random.default_rng(0)
    n = len(a.seeds)
    deltas = np.empty((n_boot, len(VARS)))
    for k in range(n_boot):
        draw = rng.integers(0, n, size=n)
        deltas[k] = ubs_resample(b, block_b, draw) - ubs_resample(a, block_a, draw)
    point = {name: (None if block_b["ubs_mean"][name] is None or block_a["ubs_mean"][name] is None
                    else block_b["ubs_mean"][name] - block_a["ubs_mean"][name]) for name in VARS}
    lo, hi = np.nanpercentile(deltas, 5, axis=0), np.nanpercentile(deltas, 95, axis=0)
    return {"n_boot": n_boot, "delta_ubs_mean": point,
            "ci90": {name: [float(lo[i]), float(hi[i])] for i, name in enumerate(VARS)},
            "above_zero": {name: bool(lo[i] > 0) for i, name in enumerate(VARS)}}


def ever_split(root: Path, scene: str, seed: int) -> bool:
    """True when two or more settlements held members at the same hourly row at any time."""
    ep = wm_data.load_episode(root / scene / f"{scene}_s{seed}_none", arrays=["global.npy", "settlements.npy"])
    if ep.arrays["settlements.npy"].shape[1] == 0:
        return False
    members = np.nan_to_num(ep.column("settlements.npy", "members"), nan=0.0)
    return bool(((members >= 1).sum(axis=1) >= 2).any())


def old_dyn_eta2(eps: list) -> dict:
    return d10.per(d11.dyn_eta2(d10.stack(eps)))


def fmt(x) -> str:
    return "—" if x is None else f"{x:.3f}"


def markdown(out: dict) -> str:
    lines = [f"# {out['label']}", "", f"Root `{out['root']}`, seeds {out['seeds']} (all episodes); dyn-eta2 "
             f"(Pass 11, continuity) on the test split {out['test']}. `diversity_p12.py`; lambda by LOO MSE from "
             f"{list(LAMBDAS)}.", "",
             "## Mean over days 60/90/118: R2_static / UBS (dyn-eta2)", "",
             "| Variable | " + " | ".join(out["scenes"]) + " |", "|---|" + "---|" * len(out["scenes"])]
    for name in VARS:
        cells = []
        for b in out["scenes"].values():
            cells.append(f"{fmt(b['r2_static_mean'][name])} / {fmt(b['ubs_mean'][name])} ({fmt(b['dyn_eta2_test'].get(name))})")
        lines.append(f"| {name} | " + " | ".join(cells) + " |")
    lines += ["", "## Per day: R2_static / UBS", ""]
    for scene, b in out["scenes"].items():
        lines += [f"### {scene} (n {b['n']}, features {len(b['features'])})", "",
                  "| Variable | " + " | ".join(f"day {d}" for d in DAYS) + " |", "|---|" + "---|" * len(DAYS)]
        for name in VARS:
            lines.append(f"| {name} | " + " | ".join(
                f"{fmt(b['r2_static'][name][str(d)])} / {fmt(b['ubs'][name][str(d)])}" for d in DAYS) + " |")
        lines.append("")
    for label, sub in out.get("subsets", {}).items():
        lines += [f"## {label}", "", "| Subset | n | " + " | ".join(VARS) + " |", "|---|---|" + "---|" * len(VARS)]
        for key, b in sub.items():
            lines.append(f"| {key} | {b['n']} | " + " | ".join(
                f"{fmt(b['r2_static_mean'][n])} / {fmt(b['ubs_mean'][n])}" for n in VARS) + " |")
        lines.append("")
    for label, bt in out.get("bootstrap", {}).items():
        lines += [f"## Paired bootstrap {label} (delta mean UBS, 90 % interval, {bt['n_boot']} resamples)", ""]
        for n in VARS:
            lo, hi = bt["ci90"][n]
            lines.append(f"- {n}: delta {fmt(bt['delta_ubs_mean'][n])}, [{lo:.3f}, {hi:.3f}]"
                         f"{' (above 0)' if bt['above_zero'][n] else ''}")
        lines.append("")
    return "\n".join(lines) + "\n"


def score(args) -> int:
    t0 = time.time()
    root = Path(args.root)
    seeds = d10.parse_range(args.seeds)
    test = d10.parse_range(args.test) if args.test else None
    target = Path(args.out)
    target.mkdir(parents=True, exist_ok=True)
    out = {"label": args.label, "root": str(root), "seeds": args.seeds, "test": args.test, "vars": list(VARS),
           "days": list(DAYS), "mean_days": list(MEAN_DAYS), "lambdas": list(LAMBDAS), "scenes": {}}
    scenes = {}
    for scene in args.scenes.split(","):
        eps = d10.load_dir(root, scene, seeds, "all")
        sc = Scene(eps)
        block = score_scene(sc)
        block["dyn_eta2_test"] = old_dyn_eta2([e for e in eps if test is None or e.seed in test]) if test else {}
        out["scenes"][scene] = block
        scenes[scene] = sc
        print(scene, "UBS mean", {n: fmt(block["ubs_mean"][n]) for n in VARS}, f"{time.time() - t0:.0f}s", flush=True)
    if args.split_subsets:
        scene = args.split_subsets
        sc = scenes[scene]
        flags = np.array([ever_split(root, scene, s) for s in sc.seeds])
        out["subsets"] = {f"{scene} by whether the world split (ridge refit per subset)": {
            "split": score_scene(sc, np.flatnonzero(flags)), "no_split": score_scene(sc, np.flatnonzero(~flags))}}
    out["bootstrap"] = {}
    for pair in args.bootstrap or []:
        a, b = pair.split(":")
        out["bootstrap"][f"{b} - {a}"] = paired_bootstrap(scenes[a], out["scenes"][a], scenes[b], out["scenes"][b])
        print("bootstrap", pair, out["bootstrap"][f"{b} - {a}"]["ci90"], f"{time.time() - t0:.0f}s", flush=True)
    out["seconds"] = time.time() - t0
    (target / "ubs.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    (target / "ubs.md").write_text(markdown(out))
    print(f"done {out['seconds']:.0f}s -> {target}")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("score")
    a.add_argument("--root", required=True)
    a.add_argument("--scenes", required=True)
    a.add_argument("--seeds", required=True)
    a.add_argument("--test")
    a.add_argument("--bootstrap", nargs="*")
    a.add_argument("--split-subsets")
    a.add_argument("--label", default="Pass 12 UBS")
    a.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    return score(args)


if __name__ == "__main__":
    raise SystemExit(main())
