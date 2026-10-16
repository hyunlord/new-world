"""WM-3 post-hoc comparisons (NOT pre-registered; reported in WM3.md as analysis, never as a verdict): the same paired
world bootstrap as the V rules (`wm2.effects.compare`) on the committed test predictions.

- WM-3 `wm3_full` (three seeds) against WM-2's M1b-act (the three seeds it started from, seed-averaged errors) on the
  12 V1 cells (30 d) and the 12 V2 cells (7 d): did the fine-tuning move the effect error at all?
- The `_s0` ablation of V5 repeated on the 7-day cells (V5 itself is on the 30-day cells).
- The per-outcome split of V2: each of the four outcomes on the three W3 arms at 7 d, `wm3_full` vs `an5_arm`.

usage: python3 tools/worldmodel/wm3/posthoc.py [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm3  # noqa: F401
from wm1 import common as c
from wm2 import effects as ef
from wm2 import v2
from wm3 import judge3 as j3


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", default=str(j3.RUNS))
    ap.add_argument("--wm2", default=str(j3.WM2))
    ap.add_argument("--out", default=str(j3.RUNS / "posthoc"))
    a = ap.parse_args(argv)
    runs, wm2 = Path(a.runs), Path(a.wm2)
    base = json.loads((wm2 / "effects_base" / "effects_base.json").read_text())
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    preds = dict(base["test_predictions"])
    for r in j3.WM2_BACT:
        preds[r] = json.loads((wm2 / f"{r}_effects" / "effects.json").read_text())["predictions"]
    for r in j3.ALL:
        p = runs / f"{r}_effects" / "effects.json"
        if p.exists():
            preds[r] = json.loads(p.read_text())["predictions"]
    errs = {n: ef.cell_errors(cases, p, stds) for n, p in preds.items()}
    full = [r for r in j3.FULL if r in errs]
    wm2_avg = ef.average_errors([errs[r] for r in j3.WM2_BACT])
    out = {"note": "post hoc, not pre-registered (METRICS.md has no such rule); same bootstrap as the V rules",
           "full_vs_wm2": {}, "ablation_7d": {}, "v2_by_outcome": {}}
    for name, cells in (("30d", j3.V1_CELLS), ("7d", j3.V2_CELLS)):
        out["full_vs_wm2"][name] = ef.compare([errs[r] for r in full], wm2_avg, cells, worlds) | {
            "baseline_member": "wm2_m1bact_v19 (seed mean)", "runs": full}
    for name, x, y in (("full_vs_ctrl", "wm3_full_s0", j3.CTRL), ("os_vs_ctrl", j3.OS, j3.CTRL),
                       ("full_vs_os", "wm3_full_s0", j3.OS), ("ctrl_vs_wm2_s0", j3.CTRL, "wm2_m1bact_v19_s0"),
                       ("full_s0_vs_wm2_s0", "wm3_full_s0", "wm2_m1bact_v19_s0")):
        if x in errs and y in errs:
            out["ablation_7d"][name] = ef.compare([errs[x]], errs[y], j3.V2_CELLS, worlds) | {"model_run": x,
                                                                                              "baseline_member": y}
    for o in v2.OUTCOMES:
        cells = [(arm, o, "7d") for arm in v2.W3_ARMS if stds[(arm, o, "7d")] is not None]
        out["v2_by_outcome"][o] = ef.compare([errs[r] for r in full], errs["an5_arm"], cells, worlds) | {
            "cells": ["|".join(x) for x in cells]}
    c.write_json(Path(a.out) / "posthoc.json", out)
    L = ["# WM-3 post-hoc comparisons (not pre-registered; `tools/worldmodel/wm3/posthoc.py`)", "",
         "| comparison | model (seeds) | baseline | Δ [95 % CI] | rule label |", "|---|---|---|---|---|"]
    row = lambda name, w: (f"| {name} | {j3.fmt(w['model'])} ({', '.join(j3.fmt(s) for s in w['seeds'])}) | "
                           f"`{w.get('baseline_member', '')}` {j3.fmt(w['baseline'])} | {j3.ci_text(w['ci_diff'])} | {w['verdict']} |")
    for k, w in out["full_vs_wm2"].items():
        L.append(row(f"`wm3_full` vs WM-2 M1b-act, {k} cells", w))
    for k, w in out["ablation_7d"].items():
        L.append(row(f"`{w['model_run']}` vs `{w['baseline_member']}`, 7 d cells", w))
    for k, w in out["v2_by_outcome"].items():
        L.append(row(f"`wm3_full` vs `an5_arm`, 7 d, {k} only", w | {"baseline_member": "an5_arm"}))
    (Path(a.out) / "posthoc.md").write_text("\n".join(L) + "\n")
    print("\n".join(L))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
