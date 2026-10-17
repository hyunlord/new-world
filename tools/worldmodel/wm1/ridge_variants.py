"""Severity-free and true-severity ridge baselines (planner note after Pass 8; DECISIONS D7.2).

usage: python3 tools/worldmodel/wm1/ridge_variants.py --out RUN_DIR [--root WM_DATA_ROOT]

bench_v0's T2 ridge rollouts hold the start row's severity for the whole rollout, which feeds "no winter" into
winter rows (and the old winter into spring) whenever a rollout crosses a season boundary. This script adds:

- `ridge_nosev` / `ridge_action_nosev`: the bench's ridge recipe (same lags, hour, season, optional 24-hour action
  sums, same lambda grid chosen on the same validation roles) without any severity input — a fair baseline;
- `ridge_truesev` / `ridge_action_truesev`: the bench's fitted ridges rolled out with each row's *true* severity —
  an oracle reference (it reads the future winter draw), not a competitor.

All are evaluated through the harness on the same starts as the bench (T1, T2 24/168 h, T2-any, T3b rollouts).
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import common as c
from wm1 import evaluate as ev


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", type=Path, default=c.DEFAULT_ROOT)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args(argv)
    t0 = time.time()
    g = c.load_groups(["A_train", "A_val", "B_train", "B_val"] + list(c.TEST_SPLITS), args.root)
    tests = {n: g[n] for n in c.TEST_SPLITS}
    stds = c.split_stds(tests)
    bench = lambda names: [ep.bench for n in names for ep in g[n]]
    lam, ridge, _ = b.choose_lambda(bench(["A_train"]), bench(["A_val"]), False)
    lam_a, ridge_a, _ = b.choose_lambda(bench(["A_train", "B_train"]), bench(["A_val", "B_val"]), True)
    nosev, lam_n, sc_n = ev.fit_ridge_without_severity(g["A_train"], g["A_val"], False)
    nosev_a, lam_na, sc_na = ev.fit_ridge_without_severity(g["A_train"] + g["B_train"], g["A_val"] + g["B_val"], True)
    nosev.name, nosev_a.name = "ridge_nosev", "ridge_action_nosev"
    preds = [nosev, nosev_a, ev.RidgePredictor(ridge, False, "ridge_truesev", "true"),
             ev.RidgePredictor(ridge_a, True, "ridge_action_truesev", "true")]
    out = {"lambda": {"ridge": lam, "ridge_action": lam_a, "ridge_nosev": lam_n, "ridge_action_nosev": lam_na,
                      "val_scores_nosev": sc_n, "val_scores_action_nosev": sc_na},
           "reference_only": ["ridge_truesev", "ridge_action_truesev"],
           "T1": {}, "T2": {"24": {}, "168": {}}, "T2_any": {"24": {}, "168": {}}, "T3b_rollout": {}}
    for split, eps in tests.items():
        out["T1"][split] = {p.name: ev.t1(p, eps, stds[split]) for p in preds[:2]}
        for h in (24, 168):
            out["T2"][str(h)][split] = {p.name: ev.t2(p, eps, stds[split], h, breakdown=True) for p in preds}
            out["T2_any"][str(h)][split] = {p.name: ev.t2(p, eps, stds[split], h, any_hour=True, breakdown=True) for p in preds}
        out["T3b_rollout"][split] = {p.name: ev.t3b(p, eps) for p in preds}
        print(f"{split} {time.time() - t0:.0f}s", flush=True)
    out["code_commit"] = c.git_commit()
    out["seconds"] = time.time() - t0
    c.write_json(args.out / "ridge_variants.json", out)
    for h in ("24", "168"):
        print(h, {s: {k: round(v["mean"], 3) for k, v in d.items()} for s, d in out["T2"][h].items()})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
