#!/usr/bin/env python3
"""Evaluate the Pass 7 Z targets from raw probe JSONL. Stdlib only.

usage: evaluate_pass7.py --round10 DIR --round120 DIR --out DIR
                         [--v11-targets P6_V11_Y_TARGETS.json] [--v12-targets V12_Z_TARGETS.json]
                         [--severity-permille W0,W1,W2]
                         [--baseline-summary V12_10DAY.json] [--baseline-summary-long V12_120DAY.json]
                         [--cold-window START,END]
                         [--legacy-pin PASS|FAIL] [--v8-repro ...] [--v9-repro ...] [--v10-repro ...]
                         [--v11-repro ...] [--v12-repro ...] [--z6 PASS|FAIL] [--z7 PASS|FAIL]
                         [--recorder-seconds S]

DIR holds `<scene>_s<seed>.jsonl` raw samples and `summary.json` written by
worldsim_probe (schema 7): 64 x 10 and 32 x N with the eight Pass 6 clocks.

Z2 is Pass 6's Y2 and Y3, Z5 its Y4-Y8 and Y10-Y12, with Pass 6's thresholds
(computed by `evaluate_pass6.evaluate`; `--v11-targets` is Pass 6's v11
evaluation, `p6-1_v11_targets`, which sets Y5's and Y8's floors). Z3 and Z4
come from the Winters table, which extends Pass 6's by two columns: severity
(the `winter_severity` of the winter's first day checkpoint, 08:00) and the
re-target share (winter `wood_retargets` / winter wood trips). Night booking as
Pass 6: a 02:00 sample belongs to the evening of the previous display day.
`--v12-targets` is this script's evaluation of game_v12 (Part 1), whose
measures give Z4's floor (0.9 x v12's median winter wood gathered) and the
v12 re-target share. Z1, Z6, Z7 are receipts; Z8 compares the median ms per
agent-tick with same-host game_v12 summaries.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from evaluate_pass4 import EvidenceError, Round, clause, daily_delta, quantiles, ratio, run_values, target
from evaluate_pass5 import load_round as load_round_schema
from evaluate_pass6 import (WINTER, evaluate as evaluate_pass6, fmt, measures as measures_pass6, nights_at_0200,
                            seasons_on, system_day, winter_closings, winter_windows, wood_by_season, work_by_season)

SCHEMA = 7
RECEIPTS = ("legacy_pin", "v8_repro", "v9_repro", "v10_repro", "v11_repro", "v12_repro")
SEVERITY_NAMES = ("mild", "normal", "harsh")
DEFAULT_WEIGHTS = (300, 400, 300)


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-7 round; every seed must have every day checkpoint."""
    return load_round_schema(path, duration, SCHEMA)


def winters_table(r: Round) -> list[dict]:
    """One row per complete seed-winter: Pass 6's columns plus severity and re-targets."""
    dps = seasons_on(r)
    table = []
    for seed in r.seeds:
        closings = winter_closings(seed)
        nights = nights_at_0200(seed)
        for days in winter_windows(seed, r.duration, dps):
            first, year = days[0], (days[0] - 1) // (4 * dps)
            opening = seed.days[first - 1]
            evenings = [(evening, row) for evening, s, row in nights if s == WINTER and evening // (4 * dps) == year]
            out = [evening for evening, row in evenings if not row["fire_lit"]]
            closing = next((row for row in closings if system_day(row) // (4 * dps) == year), None)
            trips = sum(daily_delta(seed, d, "wood_trips_total") for d in days)
            retargets = sum(daily_delta(seed, d, "wood_retargets_total") for d in days)
            table.append({
                "seed": seed.seed, "year": year, "first_display_day": first - 1,
                "severity": int(opening["winter_severity"]),
                "wood_in_hand_at_start": float(opening["wood_in_hand"]),
                "nights": len(evenings), "nights_out": len(out),
                "first_night_out": (out[0] - (first - 1) + 1) if out else 0,
                "trip_days": sum(1 for d in days if daily_delta(seed, d, "wood_trips_total") > 0),
                "wood_trips": trips,
                "urgent_trips": sum(daily_delta(seed, d, "urgent_wood_trips_total") for d in days),
                "wood_gathered": sum(daily_delta(seed, d, "wood_gathered_total") for d in days),
                "wood_retargets": retargets,
                "retarget_share": ratio(retargets, trips),
                "cold_member_hours": sum(daily_delta(seed, d, "cold_member_hours_total") for d in days),
                "food_store_at_end": float(closing["stockpile_food"]) if closing else None,
            })
    return table


def measures(long: Round, cold_window: tuple[int, int]) -> dict:
    """Numbers Pass 7 compares across versions (written into the report)."""
    own = measures_pass6(long, cold_window)
    winters = winters_table(long)
    normal = [w["wood_gathered"] for w in winters if w["severity"] == 1]
    own.update({
        "winter_wood_gathered": quantiles([w["wood_gathered"] for w in winters]),
        "winter_wood_gathered_normal": quantiles(normal),
        "retarget_share": quantiles([w["retarget_share"] for w in winters]),
        "winter_wood_retargets": quantiles([w["wood_retargets"] for w in winters]),
    })
    return own


def z3_clauses(winters: list[dict], weights: tuple[int, int, int]) -> dict:
    n = len(winters)
    none = ratio(sum(1 for w in winters if w["nights_out"] == 0), n)
    many = ratio(sum(1 for w in winters if w["nights_out"] >= 2), n)
    firsts = [w["first_night_out"] for w in winters if w["nights_out"] > 0]
    distinct = len(set(firsts))
    seventh = ratio(sum(1 for f in firsts if f == 7), len(firsts))
    shares = [ratio(sum(1 for w in winters if w["severity"] == level), n) for level in range(3)]
    return {
        "no_dark_night": clause(n > 0 and .15 <= none <= .60, {"share": none, "n": n},
                                "share of seed-winters with no dark night in [0.15, 0.60]"),
        "two_or_more": clause(n > 0 and .10 <= many <= .50, {"share": many, "n": n},
                              "share of seed-winters with >= 2 dark nights in [0.10, 0.50]"),
        "first_night_spread": clause(bool(firsts) and distinct >= 3 and seventh <= .70,
                                     {"distinct_first_nights": distinct, "seventh_share": seventh, "n": len(firsts),
                                      "histogram": {str(k): firsts.count(k) for k in sorted(set(firsts))}},
                                     "among seed-winters with a dark night: first one on >= 3 distinct nights, <= 0.70 on the seventh"),
        "severity_shares": clause(n > 0 and all(abs(s - w / 1000) <= .10 for s, w in zip(shares, weights)),
                                  {"shares": dict(zip(SEVERITY_NAMES, shares)), "weights": dict(zip(SEVERITY_NAMES, weights))},
                                  "each severity's share of the seed-winters within +-0.10 of its weight"),
    }


def evaluate(short: Round, long: Round, receipts: dict[str, bool], v11: dict | None = None, v12: dict | None = None,
             weights: tuple[int, int, int] = DEFAULT_WEIGHTS, cold_window: tuple[int, int] = (1320, 360),
             baseline_ms: float | None = None, long_baseline_ms: float | None = None,
             z6: bool = False, z7: bool = False, recorder_seconds: float | None = None) -> dict:
    """Z1-Z8 and Y9 of PASS7_GOAL.md section 11. `v11` / `v12` are the `measures` of those evaluations."""
    pass6 = evaluate_pass6(short, long, None, {}, None, v11, cold_window)
    result = {}
    result["Z1"] = target({
        "twins_10": clause(short.summary["determinism_checked"] and len(short.seeds) == 64, len(short.seeds), "64 seeds, determinism checked"),
        "twins_120": clause(long.summary["determinism_checked"] and len(long.seeds) == 32, len(long.seeds), "32 seeds, determinism checked"),
        **{name: clause(receipts.get(name, False), receipts.get(name, False), f"{name} verified externally") for name in RECEIPTS},
    })
    result["Z2"] = target({f"Y2_{k}": c for k, c in pass6["Y2"]["clauses"].items()}
                          | {f"Y3_{k}": c for k, c in pass6["Y3"]["clauses"].items()})
    winters = winters_table(long)
    result["Z3"] = target(z3_clauses(winters, weights))
    share = quantiles([w["retarget_share"] for w in winters])
    normal = quantiles([w["wood_gathered"] for w in winters if w["severity"] == 1])
    floor = .9 * v12["winter_wood_gathered"]["median"] if v12 else None
    result["Z4"] = target({
        "retarget_share": clause(share["n"] > 0 and share["median"] <= .10,
                                 {**share, "v12": v12["retarget_share"] if v12 else None},
                                 "winter wood_retargets / winter wood trips, median over seed-winters <= 0.10"),
        "normal_gathered": clause(floor is not None and normal["n"] > 0 and normal["median"] >= floor, {**normal, "floor": floor},
                                  "median winter wood gathered over normal-severity seed-winters >= 0.9 x v12's median"),
    })
    clauses = {}
    for name in ("Y4", "Y5", "Y6", "Y7", "Y8", "Y10", "Y11", "Y12"):
        clauses.update({f"{name}_{k}": c for k, c in pass6[name]["clauses"].items()})
    result["Z5"] = target(clauses)
    result["Z6"] = target({"receipt": clause(z6, z6, "8.5 tests pass and check_z6.py verdicts all PASS (or applied 0, listed)")})
    result["Z7"] = target({"receipt": clause(z7, z7, "9.4 byte identity, loader round-trip, every meta field documented")})
    clauses = {}
    for r, base in ((short, baseline_ms), (long, long_baseline_ms or baseline_ms)):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = 1.2 * base if base else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit, {**timing, "limit": limit},
                                             "median ms/agent-tick <= 1.2 x same-host game_v12, recorder off")
    clauses["recorder"] = clause(recorder_seconds is not None, recorder_seconds,
                                 "the recorder's wall time per 120-day episode reported")
    result["Z8"] = target(clauses)
    result["Y9"] = pass6["Y9"]
    result["Y9"]["reported_only"] = True
    return result


def winters_by_severity(winters: list[dict]) -> list[dict]:
    rows = []
    for level, name in enumerate(SEVERITY_NAMES):
        group = [w for w in winters if w["severity"] == level]
        rows.append({"severity": name, "n": len(group),
                     **{key: quantiles([float(w[key]) for w in group if w[key] is not None])
                        for key in ("nights_out", "first_night_out", "wood_trips", "wood_gathered", "retarget_share",
                                    "cold_member_hours", "food_store_at_end")}})
    return rows


def markdown(targets: dict, long: Round, cold_end: int, title: str) -> str:
    lines = [f"# {title}", "", "Values: median / p10 / p90 / min / max. Y9 is reported only (Pass 7 section 3.2).", "",
             "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
    for name, entry in targets.items():
        for key, item in entry["clauses"].items():
            lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {json.dumps(item['observed'], sort_keys=True)} |")
    lines += ["", "## Work by season", "", "Per-seed medians of the clock samples (days >= 2) first, then quantiles across seeds.", "",
              "| Season | Adult work 12:00 | Idle 12:00 | Adult work 15:00 | Idle 15:00 |", "|---|---|---|---|---|"]
    for row in work_by_season(long):
        lines.append(f"| {row['season']} | {fmt(row['work_1200'])} | {fmt(row['idle_1200'])} | {fmt(row['work_1500'])} | {fmt(row['idle_1500'])} |")
    lines += ["", "## Wood by season", "",
              "Stock, target and food store at day checkpoints (08:00); per-day counters over seed-days; 02:00 lit share over nights booked to the evening's season (days >= 3).", "",
              "| Season | Wood stock | Wood target | Gathered/day | Wood trips/day | Urgent trips/day | Lit at 02:00 | Cold member-hours/day | Food store | Forage trips/day |",
              "|---|---|---|---|---|---|---:|---|---|---|"]
    for row in wood_by_season(long, cold_end):
        lines.append(f"| {row['season']} | {fmt(row['wood_stock'])} | {fmt(row['wood_target'])} | {fmt(row['wood_gathered'])} | {fmt(row['wood_trips'])} | "
                     f"{fmt(row['urgent_trips'])} | {row['lit_0200']:.3f} | {fmt(row['cold_member_hours'])} | {fmt(row['food_store'])} | {fmt(row['forage_trips'])} |")
    winters = winters_table(long)
    lines += ["", "## Winters", "",
              "One row per complete seed-winter. Severity = `winter_severity` at the first winter day's 08:00 checkpoint (0 mild, 1 normal, 2 harsh, -1 none); wood in hand at that checkpoint; nights out = winter nights (booked to their evening) with the fire out at 02:00; first night out = 1..7 (0 none); trip-days, trips, urgent trips, gathered, re-targets and cold member-hours over the winter's seven elapsed days; re-target share = re-targets / trips; food store at the winter's last periodic sample.", "",
              "| Seed | Year | Severity | Wood at start | Nights out | First night out | Trip-days | Wood trips | Urgent trips | Gathered | Re-targets | Re-target share | Cold member-hours | Food store at end |",
              "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for w in winters:
        food = "–" if w["food_store_at_end"] is None else f"{w['food_store_at_end']:.0f}"
        lines.append(f"| {w['seed']} | {w['year']} | {w['severity']} | {w['wood_in_hand_at_start']:.0f} | {w['nights_out']} | {w['first_night_out']} | "
                     f"{w['trip_days']} | {w['wood_trips']:.0f} | {w['urgent_trips']:.0f} | {w['wood_gathered']:.0f} | {w['wood_retargets']:.0f} | "
                     f"{w['retarget_share']:.3f} | {w['cold_member_hours']:.0f} | {food} |")
    if winters:
        lines += ["", "| Winter measure | Median / p10 / p90 / min / max |", "|---|---|"]
        for key in ("wood_in_hand_at_start", "nights_out", "first_night_out", "trip_days", "wood_trips", "urgent_trips",
                    "wood_gathered", "wood_retargets", "retarget_share", "cold_member_hours", "food_store_at_end"):
            values = [float(w[key]) for w in winters if w[key] is not None]
            lines.append(f"| {key} | {fmt(quantiles(values))} |")
        lines.append(f"| seed-winters with a night out | {sum(1 for w in winters if w['nights_out'])} of {len(winters)} |")
        lines += ["", "## Winters by severity", "", "| Severity | Seed-winters | Nights out | First night out | Wood trips | Gathered | Re-target share | Cold member-hours | Food store at end |",
                  "|---|---:|---|---|---|---|---|---|---|"]
        for row in winters_by_severity(winters):
            lines.append(f"| {row['severity']} | {row['n']} | {fmt(row['nights_out'])} | {fmt(row['first_night_out'])} | {fmt(row['wood_trips'])} | "
                         f"{fmt(row['wood_gathered'])} | {fmt(row['retarget_share'])} | {fmt(row['cold_member_hours'])} | {fmt(row['food_store_at_end'])} |")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--v11-targets", type=Path, help="y_targets.json of Pass 6's game_v11 evaluation (p6-1_v11_targets)")
    parser.add_argument("--v12-targets", type=Path, help="z_targets.json of this script's game_v12 evaluation (Part 1)")
    parser.add_argument("--severity-permille", default=",".join(map(str, DEFAULT_WEIGHTS)))
    parser.add_argument("--baseline-summary", type=Path, help="same-host game_v12 summary.json (10 days), recorder off")
    parser.add_argument("--baseline-summary-long", type=Path, help="same-host game_v12 summary of the long duration")
    parser.add_argument("--cold-window", default="1320,360")
    parser.add_argument("--recorder-seconds", type=float)
    parser.add_argument("--title", default="Pass 7 Z targets")
    for name in (*RECEIPTS, "z6", "z7"):
        parser.add_argument(f"--{name.replace('_', '-')}", choices=("PASS", "FAIL"), default="FAIL")
    args = parser.parse_args(argv)
    try:
        def baseline_of(path: Path | None) -> float | None:
            if path is None:
                return None
            source = json.loads(path.read_text())
            if source["scene"] != "game_v12":
                raise EvidenceError("timing baseline must be a game_v12 summary")
            return float(source["timing"]["ms_per_agent_tick"]["median"])
        weights = tuple(int(v) for v in args.severity_permille.split(","))
        if len(weights) != 3 or sum(weights) != 1000:
            raise ValueError("severity weights must be three thousandths summing to 1000")
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        v12 = json.loads(args.v12_targets.read_text())["measures"] if args.v12_targets else None
        start, end = (int(v) for v in args.cold_window.split(","))
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        receipts = {name: getattr(args, name) == "PASS" for name in RECEIPTS}
        baseline_ms, long_baseline_ms = baseline_of(args.baseline_summary), baseline_of(args.baseline_summary_long)
        targets = evaluate(short, long, receipts, v11, v12, weights, (start, end), baseline_ms, long_baseline_ms,
                           args.z6 == "PASS", args.z7 == "PASS", args.recorder_seconds)
        own = measures(long, (start, end))
        text = markdown(targets, long, end, args.title)
        winters = winters_table(long)
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for name, t in targets.items() if name != "Y9")
    report = {"schema_version": 1, "passed": passed, "targets": targets, "measures": own, "winters": winters,
              "winters_by_severity": winters_by_severity(winters),
              "work_by_season": work_by_season(long), "wood_by_season": wood_by_season(long, end),
              "baseline_ms": baseline_ms, "long_baseline_ms": long_baseline_ms, "severity_permille": list(weights),
              "cold_window": [start, end],
              "inputs": {"round10": str(args.round10), "round120": str(args.round120),
                         "v11_targets": str(args.v11_targets) if args.v11_targets else None,
                         "v12_targets": str(args.v12_targets) if args.v12_targets else None}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "z_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (args.out / "z_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
