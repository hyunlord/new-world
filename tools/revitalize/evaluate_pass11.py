#!/usr/bin/env python3
"""Evaluate Pass 11's J targets on schema-11 probe runs (PASS11_GOAL.md 7.3, 10). Stdlib only.

usage: evaluate_pass11.py --round10 DIR --round120 DIR --out DIR
                          [--v16-round10 DIR --v16-round120 DIR] [--v11-targets P6_V11_Y_TARGETS.json]
                          [--extra-round120 DIR ...] [--baseline-summary V16_10DAY.json]
                          [--baseline-summary-long V16_120DAY.json] [--variety DIR ...]
                          [--legacy-pin PASS|FAIL] [--v8-v16-repro PASS|FAIL] [--title TITLE]

DIR holds a schema-11 probe run (`summary.json` plus raw JSONL) of one v17
scene (`game_v17_nofission` or `game_v17`). Every per-seed quantity comes from
the per-seed fields of `summary.json` (`runs[].seed_info`, `runs[].pass11`),
never from its `rules` block (the first seed only).

- J1: determinism twins of both rounds plus the receipts `--legacy-pin` and
  `--v8-v16-repro`.
- J2 (no collapse, every 120-day seed): total alive >= max(8, 0.4 x starting
  agents) at every daily sample (08:00 checkpoints and 06:00 samples); alive
  on day 120 >= 0.6 x day 21; no settlement with 1-2 members for more than 48
  consecutive hours (the probe's hourly count).
- J3 (`game_v17` only; fission happens and varies): share of seeds with >= 1
  fission in [0.50, 0.90]; first-fission day (founding of the first daughter)
  median in [30, 90] and interquartile range >= 14 days; share of seeds with
  >= 2 settlements alive on day 120 in [0.30, 0.85]; no seed above
  `max_settlements` alive at any daily sample; cancellations <= 0.25 of the
  attempts (departed groups).
- J4 (`game_v17` only): share of daughters alive 28 days after founding
  (counted where day + 28 lies in the run) >= 0.60; the share dissolved or
  merged is reported.
- J5 (the cold can take anyone, judged on `game_v17`): over seed-winters with
  >= 1 exposure death, pooled over every 120-day round given (`--round120`
  plus `--extra-round120`): adult share in [0.15, 0.50], child share <= 0.70,
  elder share reported; median over settlement-nights with >= 1 exposure death
  of the most deaths in one hour <= 2; Pass 10's H3 on this round (share of
  seed-winters with a death in [0.15, 0.60], median <= 2, maximum <= 0.35 x
  members, mild < normal < harsh, starvation + dehydration).
- J6 (village rhythm kept; with `--v16-round*`, game_v16 on the same seeds):
  Pass 10's H5 with game_v16 as the reference (per-clause escape table
  `evaluate_pass10.ESCAPE_STAT`). Judged for `game_v17_nofission`, reported for
  `game_v17` (whose probe measures home distance and evening-at-the-hearth
  per agent against its own settlement's formation tile).
- J9 (`--variety`: 10-day v17 rounds): every hidden parameter's mean within
  +-10 % of its range centre (the eight of Pass 10 plus the three wild-region
  parameters); region placement failures and bush / deadwood shortfalls counted.
- J12: median ms per agent-tick <= 1.3 x the same-host game_v16 summary of the
  same duration.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import evaluate_pass10 as p10
from evaluate_pass4 import EvidenceError, Round, clause, quantiles, ratio, run_values, target
from evaluate_pass5 import load_round as load_round_schema

SCHEMA = 11
BASELINE_SCENE = "game_v16"
TIMING_FACTOR = 1.3
STAGES = ("child", "adult", "elder")
RANGES = {**p10.RANGES, "wild_regions": (1, 3), "region_bushes": (7, 11), "region_deadwood": (8, 14)}


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-11 round; every seed must have every day checkpoint."""
    return load_round_schema(path, duration, SCHEMA)


runs = p10.runs


def load_reference(path: Path, duration: int) -> Round:
    """Load a game_v16 reference round of schema 10 (a Pass 10 binary) or 11."""
    schema = json.loads((path / "summary.json").read_text()).get("schema_version")
    if schema not in (10, 11):
        raise EvidenceError(f"{path}: reference summary schema must be 10 or 11")
    return load_round_schema(path, duration, schema)


def j1(short: Round, long: Round, receipts: dict[str, bool]) -> dict:
    return p10.h1(short, long, receipts)


def j2(long: Round) -> tuple[dict, dict]:
    """No collapse; returns (target, per-seed rows)."""
    h2, per_seed = p10.h2(long)
    clauses = {k: v for k, v in h2["clauses"].items() if k != "one_settlement"}
    bad = {}
    for run in runs(long):
        hours = int(run["pass11"]["max_hours_with_1_to_2_members"])
        per_seed[str(run["seed"])]["max_hours_with_1_to_2_members"] = hours
        if hours > 48:
            bad[str(run["seed"])] = hours
    clauses["no_lingering_remnant"] = clause(not bad, {"failing": bad, "n": len(per_seed)},
                                             "no settlement with 1-2 members for more than 48 h, every seed")
    return target(clauses), per_seed


def fission_rows(long: Round) -> list[dict]:
    rows = []
    for run in runs(long):
        b = run["pass11"]
        alive = [int(a) for a in run["settlements_alive_daily"]]
        rows.append({"seed": run["seed"], "fissions": int(b["fissions"]), "attempts": int(b["fission_attempts"]),
                     "cancelled": int(b["fissions_cancelled"]), "blocked_no_site_days": int(b["fission_blocked_no_site"]),
                     "first_fission_day": b["first_fission_day"], "alive_last": alive[-1] if alive else None,
                     "max_alive_settlements": max(alive) if alive else None,
                     "max_settlements": (run["seed_info"].get("fission") or {}).get("max_settlements"),
                     "remnant_merges": int(b["remnant_merges"]), "exclusivity_moves": int(b["exclusivity_moves"]),
                     "alive_at_30_60_90_118": [alive[d - 1] if len(alive) >= d else None for d in (30, 60, 90, 118)]})
    return rows


def j3(long: Round) -> tuple[dict, list[dict]]:
    rows = fission_rows(long)
    n = len(rows)
    with_fission = ratio(sum(1 for r in rows if r["fissions"] >= 1), n) if n else None
    days = [float(r["first_fission_day"]) for r in rows if r["first_fission_day"] is not None]
    q = quantiles(days) if days else {"n": 0}
    iqr = (percentile(days, .75) - percentile(days, .25)) if days else None
    two = ratio(sum(1 for r in rows if (r["alive_last"] or 0) >= 2), n) if n else None
    over = [r["seed"] for r in rows if r["max_settlements"] is not None and r["max_alive_settlements"] > r["max_settlements"]]
    attempts = sum(r["attempts"] for r in rows)
    cancelled = sum(r["cancelled"] for r in rows)
    cancel_share = ratio(cancelled, attempts) if attempts else 0.0
    return target({
        "share_with_fission": clause(with_fission is not None and .50 <= with_fission <= .90, {"share": with_fission, "n": n},
                                     "share of 120-day seeds with >= 1 fission in [0.50, 0.90]"),
        "first_fission_median": clause(bool(days) and 30 <= q["median"] <= 90, q, "first-fission day median in [30, 90]"),
        "first_fission_iqr": clause(iqr is not None and iqr >= 14, {"iqr": iqr, "p25": percentile(days, .25) if days else None,
                                                                   "p75": percentile(days, .75) if days else None},
                                    "first-fission day interquartile range >= 14 days"),
        "two_alive_day_end": clause(two is not None and .30 <= two <= .85, {"share": two, "n": n},
                                    f"share of seeds with >= 2 settlements alive on day {long.duration} in [0.30, 0.85]"),
        "never_above_max": clause(not over, {"over": over}, "no seed above max_settlements alive at any daily sample"),
        "cancellations": clause(cancel_share <= .25, {"cancelled": cancelled, "attempts": attempts, "share": cancel_share},
                                "cancellations <= 0.25 of attempts"),
    }), rows


def percentile(values: list[float], p: float) -> float:
    """Linear-interpolated percentile (as the probe's)."""
    s = sorted(values)
    if not s:
        raise EvidenceError("percentile of an empty list")
    pos = p * (len(s) - 1)
    lo = int(pos)
    hi = min(lo + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (pos - lo)


def daughters(long: Round) -> list[dict]:
    out = []
    for run in runs(long):
        for s in run["pass11"]["settlements"]:
            if s["parent"] is not None:
                out.append({"seed": run["seed"], **s})
    return out


def j4(long: Round) -> tuple[dict, dict]:
    rows = daughters(long)
    judged = [d for d in rows if d["alive_28_days_after_founding"] is not None]
    alive = sum(1 for d in judged if d["alive_28_days_after_founding"])
    share = ratio(alive, len(judged)) if judged else None
    gone = sum(1 for d in rows if d["dissolved_tick"] is not None)
    detail = {"daughters": len(rows), "judged_28d": len(judged), "alive_28d": alive,
              "dissolved_or_merged": gone, "dissolved_or_merged_share": ratio(gone, len(rows)) if rows else None,
              "founders": quantiles([float(d["founders"]) for d in rows if d["founders"] is not None]) if rows else None}
    return target({"daughters_viable": clause(share is not None and share >= .60, {"share": share, **detail},
                                              "share of daughters alive 28 days after founding >= 0.60")}), detail


def stage_pool(rounds: list[Round]) -> dict:
    """Exposure deaths by stage over seed-winters with >= 1 exposure death, pooled over `rounds`."""
    totals = Counter()
    winters_with = 0
    elder_winters = Counter()
    for r in rounds:
        for run in runs(r):
            for w in run["pass11"]["exposure_by_winter"]:
                deaths = sum(int(w[s]) for s in STAGES)
                if deaths == 0:
                    continue
                winters_with += 1
                for s in STAGES:
                    totals[s] += int(w[s])
                if (w.get("elders_at_start") or 0) > 0:
                    elder_winters["deaths"] += deaths
                    elder_winters["elder"] += int(w["elder"])
                    elder_winters["n"] += 1
    total = sum(totals.values())
    return {"seed_winters_with_death": winters_with, "deaths": dict(totals), "total": total,
            "share": {s: ratio(totals[s], total) if total else None for s in STAGES},
            "elder_share_where_elders_present": ratio(elder_winters["elder"], elder_winters["deaths"]) if elder_winters["deaths"] else None,
            "seed_winters_with_elders": elder_winters["n"]}


def settlement_night_max(rounds: list[Round]) -> list[int]:
    return [int(n["max_in_one_hour"]) for r in rounds for run in runs(r) for n in run["pass11"]["exposure_settlement_nights"]]


def j5(short: Round, long: Round, pooled: list[Round]) -> tuple[dict, dict]:
    pool = stage_pool(pooled)
    hours = settlement_night_max(pooled)
    hq = quantiles([float(h) for h in hours]) if hours else {"n": 0, "median": 0.0}
    h3, h3_tables = p10.h3(short, long)
    clauses = {
        "adult_share": clause(pool["share"]["adult"] is not None and .15 <= pool["share"]["adult"] <= .50, pool,
                              "adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled)"),
        "child_share": clause(pool["share"]["child"] is not None and pool["share"]["child"] <= .70, pool["share"],
                              "child share of exposure deaths <= 0.70"),
        "max_per_hour": clause(not hours or hq["median"] <= 2, {**hq, "max": max(hours) if hours else 0},
                               "median over settlement-nights with a death of the most deaths in one hour <= 2"),
        **{f"H3_{k}": v for k, v in h3["clauses"].items()},
    }
    return target(clauses), {"stage_pool": pool, "h3": h3_tables, "settlement_night_max": dict(Counter(hours))}


def j6(short: Round, long: Round, v16_short: Round | None, v16_long: Round | None, v11: dict | None) -> tuple[dict, dict]:
    return p10.h5(short, long, v16_short, v16_long, v11)


def j9(rounds: list[Round]) -> tuple[dict, dict]:
    h6, detail = p10.h6(rounds, RANGES, p10.CLIMATE_PERMILLE)
    regions = [run["seed_info"].get("wild_regions") for r in rounds for run in runs(r)]
    missing = sum(1 for g in regions if g is None)
    failures = [{"requested": g["regions_requested"], "placed": g["regions_placed"]} for g in regions
                if g and g["regions_placed"] < g["regions_requested"]]
    bush_short = sum(int(g["region_bushes_requested"]) - int(g["region_bushes_placed"]) for g in regions if g)
    wood_short = sum(int(g["region_deadwood_requested"]) - int(g["region_deadwood_placed"]) for g in regions if g)
    water = [int(c["water_distance"]) for g in regions if g for c in g["centres"]]
    detail["regions"] = {"seeds": len(regions), "without_block": missing, "placement_failures": failures,
                         "bush_shortfall_total": bush_short,
                         "seeds_with_bush_shortfall": sum(1 for g in regions if g and g["region_bushes_placed"] < g["region_bushes_requested"]),
                         "deadwood_shortfall_total": wood_short,
                         "water_distance": quantiles([float(w) for w in water]) if water else None}
    clauses = dict(h6["clauses"])
    clauses["region_placement"] = clause(missing == 0 and not failures, detail["regions"],
                                         "region placement failures 0 (shortfalls counted)")
    return target(clauses), detail


def j12(short: Round, long: Round, base10: float | None, base_long: float | None) -> dict:
    clauses = {}
    for r, base in ((short, base10), (long, base_long or base10)):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = TIMING_FACTOR * base if base else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit,
                                             {**timing, "limit": limit, "ratio": timing["median"] / base if base else None},
                                             f"median ms/agent-tick <= {TIMING_FACTOR} x same-host {BASELINE_SCENE}")
    return target(clauses)


def baseline_ms(path: Path | None) -> float | None:
    if path is None:
        return None
    source = json.loads(path.read_text())
    if source["scene"] != BASELINE_SCENE:
        raise EvidenceError(f"timing baseline must be a {BASELINE_SCENE} summary")
    return float(source["timing"]["ms_per_agent_tick"]["median"])


def markdown(title: str, scene: str, targets: dict, extra: dict) -> str:
    lines = [f"# {title}", "", f"Scene `{scene}`.", "", "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
    for name, entry in targets.items():
        for key, item in entry["clauses"].items():
            observed = json.dumps(item["observed"], sort_keys=True)
            if len(observed) > 500:
                observed = observed[:500] + "…"
            lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {observed} |")
    if "fission" in extra:
        lines += ["", "## Fission per seed", "", "| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |",
                  "|---:|---:|---:|---:|---:|---:|---|---:|"]
        for r in extra["fission"]:
            lines.append(f"| {r['seed']} | {r['fissions']} | {r['attempts']} | {r['cancelled']} | {r['blocked_no_site_days']} | "
                         f"{r['first_fission_day']} | {r['alive_at_30_60_90_118']} | {r['remnant_merges']} |")
    if "stage_pool" in extra:
        lines += ["", f"Exposure deaths by stage (seed-winters with a death, pooled): {json.dumps(extra['stage_pool'], sort_keys=True)}"]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--extra-round120", type=Path, nargs="*", default=[], help="more 120-day rounds of the scene pooled into J5's stage shares")
    parser.add_argument("--v16-round10", type=Path)
    parser.add_argument("--v16-round120", type=Path)
    parser.add_argument("--v11-targets", type=Path)
    parser.add_argument("--baseline-summary", type=Path)
    parser.add_argument("--baseline-summary-long", type=Path)
    parser.add_argument("--variety", type=Path, nargs="*", default=[])
    parser.add_argument("--legacy-pin", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--v8-v16-repro", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--title", default="Pass 11 J targets")
    args = parser.parse_args(argv)
    try:
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        scene = long.summary.get("scene")
        if short.summary.get("scene") != scene:
            raise EvidenceError("both rounds must be the same scene")
        v16 = (load_reference(args.v16_round10, 10), load_reference(args.v16_round120, args.long_days)) \
            if args.v16_round10 else (None, None)
        if v16[0] is not None and (v16[0].summary.get("scene"), v16[1].summary.get("scene")) != (BASELINE_SCENE, BASELINE_SCENE):
            raise EvidenceError(f"--v16-round* must be {BASELINE_SCENE} runs")
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        extra_long = [load_round(p, args.long_days) for p in args.extra_round120]
        targets = {"J1": j1(short, long, {"legacy_pin": args.legacy_pin == "PASS", "v8_v16_repro": args.v8_v16_repro == "PASS"})}
        targets["J2"], j2_seeds = j2(long)
        extra: dict = {}
        if scene == "game_v17":
            targets["J3"], extra["fission"] = j3(long)
            targets["J4"], extra["daughters"] = j4(long)
        targets["J5"], j5_detail = j5(short, long, [long, *extra_long])
        extra["stage_pool"] = j5_detail["stage_pool"]
        targets["J6"], j6_reported = j6(short, long, v16[0], v16[1], v11)
        if args.variety:
            targets["J9"], extra["variety"] = j9([load_round(p, 10) for p in args.variety])
        if args.baseline_summary:
            targets["J12"] = j12(short, long, baseline_ms(args.baseline_summary), baseline_ms(args.baseline_summary_long))
        text = markdown(args.title, scene, targets, extra)
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for t in targets.values())
    report = {"schema_version": 1, "probe_schema": SCHEMA, "scene": scene, "passed": passed, "targets": targets,
              "j2_per_seed": j2_seeds, "j5": j5_detail, "j6_reported": j6_reported, "extra": extra,
              "seeds": {"round10": short.summary["seeds"], "round120": long.summary["seeds"]},
              "inputs": {k: (str(v) if isinstance(v, Path) else [str(p) for p in v] if isinstance(v, list) else v)
                         for k, v in vars(args).items() if k != "out"}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "j_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True, default=str) + "\n")
    (args.out / "j_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
