#!/usr/bin/env python3
"""Pass 12 Part 1 (PASS12_GOAL.md 6.2): the J6 bisection — which v16 -> v17_nofission difference took the afternoon.

For each 120-day probe round (tuning seeds 0..32, schema 11 or 12): Pass 6 Y9 `work_1200`, `work_1500`, `idle_1200`
exactly as `evaluate_pass6.py` computes them (median over seeds of the per-seed median of the 12:00 / 15:00 clock rows
in spring-autumn), the adult count at 15:00 and the share of adults by activity at 15:00 (schema-12 `adult_act_*`
keys; per-seed median of each share, then the median over seeds, plus the pooled share over every 15:00 row).
Reverse ablations (`abl_*`, v17_nofission with one difference reverted) report the share of the v16 - v17_nofission
work_1500 gap they restore; forward ablations (`fwd_*`, v16 with one difference added) the share they cause.

usage: bisect_p12.py --raw .harness/evidence/revitalize/probe --prefix p12-1_ --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from evaluate_pass4 import per_seed_median, quantiles  # noqa: E402
from evaluate_pass5 import AFTERNOON, NOON, load_round, with_unit_denominator  # noqa: E402

WORK = {0, 1, 2}
ACTS = ("forage", "wood", "build", "haul", "idle", "eat", "drink", "rest", "social", "founding")
REVERSE = ("abl_no_regions", "abl_v16_damage", "abl_no_hunger_weight", "abl_no_hardiness", "abl_no_regional_wood",
           "abl_no_rain_soak", "abl_v16_exposure", "abl_local_food", "abl_local_food2", "abl_local_food3")
FORWARD = ("fwd_regions", "fwd_regions_regional_wood", "fwd_v17_damage", "fwd_hunger_weight", "fwd_hardiness",
           "fwd_exposure_v2", "fwd_regions_bushes_only", "fwd_regions_wood_only")


def metrics(path: Path) -> dict:
    summary = json.loads((path / "summary.json").read_text())
    r = load_round(path, 120, int(summary["schema_version"]))
    with_unit_denominator(r)
    med = lambda minute, num, den: quantiles(per_seed_median(r, minute, num, den, WORK))["median"]
    out = {"scene": summary["scene"], "seeds": summary["seeds"], "determinism_checked": summary["determinism_checked"],
           "work_1200": med(NOON, "adult_work_share", "one"), "work_1500": med(AFTERNOON, "adult_work_share", "one"),
           "idle_1200": med(NOON, "idle_share", "one")}
    if int(summary["schema_version"]) >= 12:
        out["adults_1500"] = med(AFTERNOON, "adult_act_total", "one")
        out["share_1500_median"] = {a: med(AFTERNOON, f"adult_act_{a}", "adult_act_total") for a in ACTS}
        out["share_1500_median"]["walking"] = med(AFTERNOON, "adult_act_walking", "adult_act_total")
        tot = {a: 0.0 for a in (*ACTS, "walking")}
        n = 0.0
        for seed in r.seeds:
            for (day, minute), row in sorted(seed.clocks.items()):
                if minute == AFTERNOON and day >= 2 and row["season"] in WORK:
                    n += float(row["adult_act_total"])
                    for a in tot:
                        tot[a] += float(row[f"adult_act_{a}"])
        out["share_1500_pooled"] = {a: (v / n if n else None) for a, v in tot.items()}
    return out


def fmt(x) -> str:
    return "—" if x is None else f"{x:.3f}"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--raw", type=Path, required=True)
    ap.add_argument("--prefix", default="p12-1_")
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args(argv)
    names = ("game_v16", "game_v17_nofission", *REVERSE, *FORWARD)
    rows = {}
    for name in names:
        path = args.raw / f"{args.prefix}{name}_120day"
        if (path / "summary.json").exists():
            rows[name] = metrics(path)
    v16, v17n = rows["game_v16"]["work_1500"], rows["game_v17_nofission"]["work_1500"]
    gap = v16 - v17n
    for name, m in rows.items():
        if name in REVERSE:
            m["gap_share"] = (m["work_1500"] - v17n) / gap if gap else None
        elif name in FORWARD:
            m["gap_share"] = (v16 - m["work_1500"]) / gap if gap else None
    out = {"gap_work_1500": gap, "rows": rows}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "bisect.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    lines = ["# J6 bisection (Pass 12 Part 1)", "",
             f"Tuning seeds 0-31 x 120 days, Pass 11's flags, `probe_p1`. Gap in Pass 6 Y9 `work_1500` (game_v16 - "
             f"game_v17_nofission) on these seeds: {v16:.3f} - {v17n:.3f} = **{gap:.3f}**. `gap share`: for `abl_*` the share of "
             "the gap the single reversion restores, for `fwd_*` the share the single addition causes. `bisect_p12.py`.", "",
             "| Run | work_1200 | work_1500 | idle_1200 | gap share | adults at 15:00 |", "|---|---:|---:|---:|---:|---:|"]
    for name, m in rows.items():
        lines.append(f"| {name} | {fmt(m['work_1200'])} | {fmt(m['work_1500'])} | {fmt(m['idle_1200'])} | "
                     f"{fmt(m.get('gap_share'))} | {fmt(m.get('adults_1500'))} |")
    lines += ["", "## Adults by activity at 15:00 (spring-autumn; pooled share over every 15:00 row)", "",
              "| Run | " + " | ".join((*ACTS, "walking")) + " |", "|---|" + "---:|" * (len(ACTS) + 1)]
    for name, m in rows.items():
        if "share_1500_pooled" in m:
            lines.append(f"| {name} | " + " | ".join(fmt(m["share_1500_pooled"][a]) for a in (*ACTS, "walking")) + " |")
    (args.out / "bisect.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
