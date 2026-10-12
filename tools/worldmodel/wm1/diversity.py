"""WM-1.1 Part 2: how different are the seeds of dataset v0? (GOAL_1_1.md 7). numpy only.

For each split group (A_train, A_val, A_test, B_test, OOD_v12, OOD_long), per variable of V over rows >= 24:
R^2 against CalAbs (wrapped past the train length) and CalYear (both from A_train; A_train itself is in-sample),
the seed-variance fraction eta^2, the divergence day of episode pairs, the between-seed spread at 06:00 of days 30,
60, 90, 118 over the split std, and the winter-severity sequences (18:00 of each winter's first day, a description
only) with eta^2 recomputed within groups of equal sequence. The math is in `wm1.calcore`.

usage (on the DGX): python3 tools/worldmodel/wm1/diversity.py --out RUN_DIR
"""
from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import calcore as cc
from wm1 import common as c

GROUPS = ("A_train", "A_val", "A_test", "B_test", "OOD_v12", "OOD_long")
LAST_WINTER_DAY = 218


def severity_sequence(ep) -> tuple[int, ...]:
    """The winter-severity draw of every winter (18:00 of days 22, 50, ..., up to day 218 where the episode has it)."""
    out = []
    for day in cc.winter_days(ep.rows, LAST_WINTER_DAY):
        r = cc.row_of(day, 1080)
        if int(ep.bench.season[r]) != 3 or int(ep.bench.severity[r]) < 0:
            raise ValueError(f"seed {ep.seed} day {day} 18:00: season {ep.bench.season[r]}, "
                             f"severity {ep.bench.severity[r]}")
        out.append(int(ep.bench.severity[r]))
    return tuple(out)


def stack(eps) -> np.ndarray:
    rows = {ep.rows for ep in eps}
    if len(rows) != 1:
        raise ValueError(f"episodes of a split differ in length: {sorted(rows)}")
    return np.stack([ep.bench.v for ep in eps]).astype(np.float64)


def run(groups: dict, m_abs: np.ndarray, m_year: np.ndarray) -> dict:
    out = {}
    for name, eps in groups.items():
        v = stack(eps)
        std = b.split_std([ep.bench for ep in eps])
        out[name] = cc.split_diagnostics(v, m_abs, m_year, std, [severity_sequence(ep) for ep in eps], b.VARS)
        out[name]["in_sample"] = name == "A_train"
        out[name]["seeds"] = [int(ep.seed) for ep in eps]
    return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    args = ap.parse_args(argv)
    t0 = time.time()
    g = c.load_groups(list(GROUPS), Path(args.root))
    m_abs, m_year = cc.calendar_means(stack(g["A_train"]))
    if not np.isfinite(m_year).all():
        raise SystemExit("m_year leaves a year position uncovered")
    out = {"dataset": "v0", "vars": list(b.VARS), "groups": run(g, m_abs, m_year), "code_commit": c.git_commit(),
           "manifest_sha256": c.manifest_sha(), "argv": sys.argv}
    out["seconds"] = time.time() - t0
    c.write_json(Path(args.out) / "diversity.json", out)
    for name, d in out["groups"].items():
        print(name, d["episodes"], "eta2", {k: round(x, 3) for k, x in d["eta2"].items() if x is not None},
              "median div day", d["divergence"]["median_day"], "never", d["divergence"]["never"], flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
