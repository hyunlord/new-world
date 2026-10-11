"""Counterfactual trajectory figure for ANALYSIS.md (GOAL.md 15.3): one C_cf test seed, rain vs no rain.

usage:
  python3 tools/worldmodel/wm1/plot_cf.py dump --run RUN_DIR --seed 2008 --variant rain --out traj.json   (DGX, torch)
  python3 tools/worldmodel/wm1/plot_cf.py svg --traj traj.json --out docs/worldmodel/wm1/fig_cf_rain.svg   (numpy only)

`dump` rolls the model out from the factual row at the intervention tick to day 29 07:00 twice (with and without the
intervention, exactly as T4 does) and stores the true factual and variant trajectories next to the predicted ones.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b

SHOW = ("wood_stock", "fire_lit", "cold_members", "food_stock")


def dump(args) -> int:
    from wm1 import common as c
    from wm1 import evaluate as ev
    from wm1 import m1_gru as g
    pred, config = g.load_predictor(Path(args.run))
    eps = c.load_groups(["C_test"], Path(args.root))["C_test"]
    group = ev.cf_groups(eps)[args.seed]
    factual, variant = group["none"], group[args.variant]
    (rec,) = variant.interventions
    start = int(rec["tick"]) // b.EVERY
    acts = np.zeros_like(factual.bench.actions)
    acts[start, b.KINDS.index(rec["kind"])] = float(rec["amount"])
    horizon = ev.CF_END_ROW - start
    on = pred.rollout(factual, np.array([start]), horizon, acts)
    off = pred.rollout(factual, np.array([start]), horizon, np.zeros_like(acts))
    rows = list(range(start - 48, ev.CF_END_ROW + 1))
    out = {"run": str(args.run), "seed": args.seed, "variant": args.variant, "intervention": rec, "start_row": start,
           "rows": rows, "hours": [(int(factual.bench.tick[r]) + b.OFFSET) / 60.0 for r in rows], "series": {}}
    for name in SHOW:
        i = b.VARS.index(name)
        pre = factual.bench.v[start - 48: start + 1, i].tolist()
        out["series"][name] = {"true_factual": factual.bench.v[rows, i].tolist(), "true_variant": variant.bench.v[rows, i].tolist(),
                               "pred_with": pre + on.v[0, :, i].tolist(), "pred_without": pre + off.v[0, :, i].tolist()}
    out["targets"] = {"true_effect": {k: b.c_cf_targets(variant.bench)[k] - b.c_cf_targets(factual.bench)[k] for k in b.T4_TARGETS},
                      "pred_effect": {k: ev.cf_branch_targets(pred, factual, start, acts)[k]
                                      - ev.cf_branch_targets(pred, factual, start, np.zeros_like(acts))[k] for k in b.T4_TARGETS}}
    Path(args.out).write_text(json.dumps(out, indent=1))
    return 0


def svg(args) -> int:
    d = json.loads(Path(args.traj).read_text())
    W, H, pad, gap = 900, 760, 60, 30
    panel_h = (H - 2 * pad - gap * (len(SHOW) - 1)) / len(SHOW)
    x = np.array(d["hours"]) / 24.0
    x0, x1 = x.min(), x.max()
    sx = lambda v: pad + (v - x0) / (x1 - x0) * (W - 2 * pad)
    colors = {"true_factual": "#6b7280", "true_variant": "#111827", "pred_without": "#60a5fa", "pred_with": "#dc2626"}
    dash = {"true_factual": "", "true_variant": "", "pred_without": "6,4", "pred_with": "6,4"}
    label = {"true_factual": "true, no rain", "true_variant": "true, rain", "pred_without": "M1, no rain",
             "pred_with": "M1, rain"}
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="sans-serif" font-size="12">',
             f'<rect width="{W}" height="{H}" fill="white"/>',
             f'<text x="{pad}" y="24" font-size="15" font-weight="bold">C_cf seed {d["seed"]}: {d["variant"]} '
             f'(amount {d["intervention"]["amount"]} at tick {d["intervention"]["tick"]}) — true vs M1 rollouts from the same row</text>']
    sx_int = sx((int(d["intervention"]["tick"]) + 480) / 60.0 / 24.0)
    for p, name in enumerate(SHOW):
        top = pad + p * (panel_h + gap)
        ser = d["series"][name]
        allv = np.concatenate([np.array(v, dtype=float) for v in ser.values()])
        lo, hi = float(allv.min()), float(allv.max())
        hi = hi if hi > lo else lo + 1
        sy = lambda v: top + panel_h - (v - lo) / (hi - lo) * panel_h
        parts.append(f'<rect x="{pad}" y="{top:.1f}" width="{W - 2 * pad}" height="{panel_h:.1f}" fill="none" stroke="#d1d5db"/>')
        parts.append(f'<text x="{pad + 4}" y="{top + 14:.1f}" font-weight="bold">{name}</text>')
        parts.append(f'<text x="{pad - 6}" y="{top + 10:.1f}" text-anchor="end">{hi:g}</text>')
        parts.append(f'<text x="{pad - 6}" y="{top + panel_h:.1f}" text-anchor="end">{lo:g}</text>')
        parts.append(f'<line x1="{sx_int:.1f}" y1="{top:.1f}" x2="{sx_int:.1f}" y2="{top + panel_h:.1f}" stroke="#f59e0b" stroke-width="1.5"/>')
        for key in ("true_factual", "true_variant", "pred_without", "pred_with"):
            pts = " ".join(f"{sx(a):.1f},{sy(v):.1f}" for a, v in zip(x, ser[key]))
            parts.append(f'<polyline points="{pts}" fill="none" stroke="{colors[key]}" stroke-width="1.6" stroke-dasharray="{dash[key]}"/>')
    for day in range(int(np.ceil(x0)), int(np.floor(x1)) + 1):
        parts.append(f'<text x="{sx(day):.1f}" y="{H - pad + 16}" text-anchor="middle">{day + 1}</text>')
    parts.append(f'<text x="{W / 2}" y="{H - 14}" text-anchor="middle">one-based day (orange line: the intervention)</text>')
    for i, key in enumerate(("true_factual", "true_variant", "pred_without", "pred_with")):
        lx = pad + 10 + i * 200
        parts.append(f'<line x1="{lx}" y1="44" x2="{lx + 30}" y2="44" stroke="{colors[key]}" stroke-width="2" stroke-dasharray="{dash[key]}"/>')
        parts.append(f'<text x="{lx + 36}" y="48">{label[key]}</text>')
    parts.append("</svg>")
    Path(args.out).write_text("\n".join(parts) + "\n")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    du = sub.add_parser("dump")
    du.add_argument("--run", required=True)
    du.add_argument("--seed", type=int, default=2008)
    du.add_argument("--variant", default="rain")
    du.add_argument("--out", required=True)
    du.add_argument("--root", default="/home/hyunlord/work/worldsim-wm-data")
    sv = sub.add_parser("svg")
    sv.add_argument("--traj", required=True)
    sv.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    return dump(args) if args.cmd == "dump" else svg(args)


if __name__ == "__main__":
    raise SystemExit(main())
