"""W1: reproduce WM-Bench v0's baselines through the WM-1 harness, and compute the new tasks' baselines.

usage: python3 tools/worldmodel/wm1/repro_bench.py --out RUN_DIR [--root WM_DATA_ROOT]

Wraps `bench_v0`'s persistence, daily-naive, ridge and action-ridge as harness predictors (T1, T2), runs the rule
and logistic baselines through the harness's T3 rows and the zero / val-mean effects through its T4 scoring, and
compares every number with `docs/worldmodel/bench_v0.json` (tolerance 1e-3). Also writes the baselines of the new
tasks (T2-any, T3b, T4 normalized MAE) to `baselines.json`.
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
from wm1 import common as c
from wm1 import evaluate as ev

TOL = 1e-3


def fit_ridges(g) -> tuple[ev.RidgePredictor, ev.RidgePredictor, dict]:
    bench = lambda name: [ep.bench for ep in g[name]]
    lam, ridge, s1 = b.choose_lambda(bench("A_train"), bench("A_val"), False)
    lam_a, ridge_a, s2 = b.choose_lambda(bench("A_train") + bench("B_train"), bench("A_val") + bench("B_val"), True)
    return (ev.RidgePredictor(ridge, False, "ridge"), ev.RidgePredictor(ridge_a, True, "ridge_action"),
            {"ridge": lam, "ridge_action": lam_a, "val_scores": s1, "val_scores_action": s2})


def compare(rows: list, task: str, split: str, name: str, mine, ref) -> None:
    mine, ref = np.asarray(mine, dtype=float), np.asarray(ref, dtype=float)
    both_nan = np.isnan(mine) & np.isnan(ref)
    diff = np.where(both_nan, 0.0, np.abs(mine - ref))
    rows.append({"task": task, "split": split, "baseline": name, "max_abs_diff": float(np.nanmax(np.where(np.isnan(diff), np.inf, diff))),
                 "n_values": int(mine.size)})


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", type=Path, default=c.DEFAULT_ROOT)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args(argv)
    t0 = time.time()
    g = c.load_groups(list(c.GROUPS), args.root)
    tests = {n: g[n] for n in c.TEST_SPLITS}
    stds = c.split_stds(tests)
    ridge, ridge_a, lams = fit_ridges(g)
    preds = [ev.Persistence(), ev.DailyNaive(), ridge, ridge_a]
    ref = json.loads(c.BENCH_JSON.read_text())
    rows: list = []
    base: dict = {"lambda": lams, "T1": {}, "T2": {"24": {}, "168": {}}, "T2_any": {"24": {}, "168": {}}, "T3b_rollout": {}}
    for split, eps in tests.items():
        base["T1"][split] = {}
        for p in preds:
            r = ev.t1(p, eps, stds[split])
            base["T1"][split][p.name] = r
            compare(rows, "T1", split, p.name, r["per_var"], ref["T1"][split][p.name])
        for h in (24, 168):
            base["T2"][str(h)][split] = {}
            base["T2_any"][str(h)][split] = {}
            for p in preds:
                r = ev.t2(p, eps, stds[split], h, breakdown=True)
                base["T2"][str(h)][split][p.name] = r
                compare(rows, f"T2 {h} h", split, p.name, r["per_var"], ref["T2"][str(h)][split][p.name])
                base["T2_any"][str(h)][split][p.name] = ev.t2(p, eps, stds[split], h, any_hour=True, breakdown=True)
        base["T3b_rollout"][split] = {p.name: ev.t3b(p, eps) for p in (ridge, ridge_a)}
        print(f"{split} done {time.time() - t0:.0f}s", flush=True)
    t3 = ev.t3_baselines(g["A_train"], g["A_test"])
    base["T3"] = t3
    for name in ("rule", "logistic"):
        compare(rows, "T3", "A_test", name, [t3[name][k] for k in ("brier", "auc", "accuracy")],
                [ref["T3"][name][k] for k in ("brier", "auc", "accuracy")])
    base["T3b"] = ev.t3b_baselines(g["A_train"], g["A_val"], tests)
    true_test = b.c_cf_effects([ep.bench for ep in g["C_test"]])
    true_val = b.c_cf_effects([ep.bench for ep in g["C_val"]])
    base["T4"] = {}
    for name, eff in ev.t4_baseline_effects(true_test, true_val).items():
        sc = ev.t4_score(eff, true_test)
        base["T4"][name] = sc
        for variant in sc:
            compare(rows, "T4", f"C_test/{variant}", name,
                    [sc[variant][k][m] for k in b.T4_TARGETS for m in ("mae", "sign_accuracy")],
                    [ref["T4"][variant][name][k][m] for k in b.T4_TARGETS for m in ("mae", "sign_accuracy")])
    base["T4_true_mean_abs"] = {v: {k: float(np.mean([abs(e[k]) for e in s.values()])) for k in b.T4_TARGETS}
                                for v, s in true_test.items()}
    worst = max(r["max_abs_diff"] for r in rows)
    result = {"tolerance": TOL, "max_abs_diff": worst, "pass": bool(worst <= TOL), "rows": rows,
              "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(), "seconds": time.time() - t0}
    c.write_json(args.out / "w1_repro.json", result)
    c.write_json(args.out / "baselines.json", base)
    lines = ["# W1 — harness fidelity", "",
             f"Bench baselines recomputed through `tools/worldmodel/wm1/evaluate.py` and compared with "
             f"`docs/worldmodel/bench_v0.json` (code commit `{result['code_commit'][:8]}`, manifest sha256 "
             f"`{result['manifest_sha256'][:12]}`). Tolerance {TOL}; worst absolute difference "
             f"{worst:.2e}: **{'PASS' if result['pass'] else 'FAIL'}**.", "",
             "| Task | Split | Baseline | Values | Max abs diff |", "|---|---|---|---:|---:|"]
    lines += [f"| {r['task']} | {r['split']} | {r['baseline']} | {r['n_values']} | {r['max_abs_diff']:.2e} |" for r in rows]
    (args.out / "W1_REPRO.md").write_text("\n".join(lines) + "\n")
    print(f"W1 max_abs_diff {worst:.3e} pass {result['pass']} ({time.time() - t0:.0f}s)")
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
