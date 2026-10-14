#!/usr/bin/env python3
"""Pass 13 Part 1 (PASS13_GOAL.md 6): the J6 remainder — which remaining v17/v18 difference keeps game_v18_nofission's
afternoon below game_v16's.

For each 120-day probe round (tuning seeds 0..32): `bisect_p12.metrics` (Pass 6 Y9 `work_1200`, `work_1500`,
`idle_1200` and the schema-12 adult activity shares at 15:00). Reverse ablations (`abl18_*`, game_v18_nofission with
one difference reverted to game_v16) report the share of the game_v16 - game_v18_nofission `work_1500` gap they
restore; the activity table shows which activity holds the missing afternoon share.

usage: bisect_p13.py --raw .harness/evidence/revitalize/probe --base-prefix p13-0_ --prefix p13-1_ --out DIR
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from bisect_p12 import ACTS, fmt, metrics

REVERSE = ("abl18_v16_damage", "abl18_no_hunger_weight", "abl18_no_hardiness", "abl18_v16_exposure", "abl18_no_regional_wood",
           "abl18_no_region_deadwood", "abl18_no_region_bushes", "abl18_no_regions", "abl18_no_wet_burn", "abl18_no_area_floor")
BASES = {"game_v16": "v16", "game_v18_nofission": "v18n"}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--raw", type=Path, required=True)
    ap.add_argument("--base-prefix", default="p13-0_")
    ap.add_argument("--prefix", default="p13-1_")
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args(argv)
    rows = {name: metrics(args.raw / f"{args.base_prefix}{short}_120day") for name, short in BASES.items()}
    for name in REVERSE:
        path = args.raw / f"{args.prefix}{name}_120day"
        if (path / "summary.json").exists():
            rows[name] = metrics(path)
    v16, v18n = rows["game_v16"]["work_1500"], rows["game_v18_nofission"]["work_1500"]
    gap = v16 - v18n
    for name in REVERSE:
        if name in rows:
            rows[name]["gap_share"] = (rows[name]["work_1500"] - v18n) / gap if gap else None
    a16, a18 = rows["game_v16"].get("share_1500_pooled", {}), rows["game_v18_nofission"].get("share_1500_pooled", {})
    moved = {a: a18[a] - a16[a] for a in a16 if a in a18}
    out = {"gap_work_1500": gap, "activity_shift_1500_v18n_minus_v16": moved, "rows": rows}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "bisect.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    lines = ["# J6 remainder (Pass 13 Part 1)", "",
             f"Tuning seeds 0-31 x 120 days, Pass 12's final flags, `probe_p1`. Gap in Pass 6 Y9 `work_1500` (game_v16 - "
             f"game_v18_nofission): {v16:.3f} - {v18n:.3f} = **{gap:.3f}**. `gap share`: the share of the gap the single "
             "reversion restores. `bisect_p13.py`.", "",
             "| Run | work_1200 | work_1500 | idle_1200 | gap share | adults at 15:00 |", "|---|---:|---:|---:|---:|---:|"]
    for name, m in rows.items():
        lines.append(f"| {name} | {fmt(m['work_1200'])} | {fmt(m['work_1500'])} | {fmt(m['idle_1200'])} | "
                     f"{fmt(m.get('gap_share'))} | {fmt(m.get('adults_1500'))} |")
    lines += ["", "## Adults by activity at 15:00 (spring-autumn; pooled share over every 15:00 row)", "",
              "| Run | " + " | ".join((*ACTS, "walking")) + " |", "|---|" + "---:|" * (len(ACTS) + 1)]
    for name, m in rows.items():
        if "share_1500_pooled" in m:
            lines.append(f"| {name} | " + " | ".join(fmt(m["share_1500_pooled"][a]) for a in (*ACTS, "walking")) + " |")
    lines += ["", "Shift v18_nofission − v16 of the pooled 15:00 shares: "
              + ", ".join(f"{a} {moved[a]:+.3f}" for a in (*ACTS, "walking") if a in moved)]
    (args.out / "bisect.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
