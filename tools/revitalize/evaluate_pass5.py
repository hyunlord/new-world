#!/usr/bin/env python3
"""Evaluate the Pass 5 X targets from raw probe JSONL. Stdlib only.

usage: evaluate_pass5.py --round10 DIR --round120 DIR --out DIR
                         [--baseline-summary V10_10DAY.json | --baseline-ms MS]
                         [--baseline-summary-long V10_120DAY.json]
                         [--winter-burn-per-day N] [--cold-window START,END]
                         [--legacy-pin PASS|FAIL] [--v8-repro PASS|FAIL]
                         [--v9-repro PASS|FAIL] [--v10-repro PASS|FAIL]

DIR holds `<scene>_s<seed>.jsonl` raw samples and `summary.json` written by
worldsim_probe (schema 5). The 10-day round is `game_v11` 64 x 10; the long
round is `game_v11` 32 x N with clock samples at 02:00, 12:00, 15:00 and 19:30
at least. Per-seed values are computed first and only then aggregated.

`--winter-burn-per-day` is the configured winter burn (`burn_per_hour[3] x
burn hours`, 39 for the initial game_v11) and `--cold-window` the configured
cold window in minutes of day (1320,360); both come from `game_v11()` because
the raw samples carry the derived targets, not the rule.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from evaluate_pass4 import (EvidenceError, Round, Seed, autumn_closings, clause, daily_delta, display_day,
                            longest_stagnation, per_seed_median, quantiles, ratio, run_values, target,
                            total_deaths, window_season)

NIGHT, NOON, AFTERNOON, EVENING = 120, 720, 900, 1170
SPRING, SUMMER, AUTUMN, WINTER = range(4)
SCHEMA = 5


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-5 round; every seed must have every day checkpoint."""
    summary = json.loads((path / "summary.json").read_text())
    if summary.get("schema_version") != SCHEMA:
        raise EvidenceError(f"{path}: summary schema must be {SCHEMA}")
    if int(max(map(int, summary["days"]))) != duration:
        raise EvidenceError(f"{path}: summary covers {max(map(int, summary['days']))} days, expected {duration}")
    seeds = []
    pattern = re.compile(r"^(?P<scene>[a-z0-9_]+)_s(?P<seed>\d+)\.jsonl$")
    for file in sorted(path.iterdir()):
        match = pattern.match(file.name)
        if not match:
            continue
        seed = Seed(int(match["seed"]))
        with file.open() as stream:
            for line in stream:
                row = json.loads(line)
                if row.get("schema_version") != SCHEMA:
                    raise EvidenceError(f"{file}: raw rows must be schema {SCHEMA}")
                tpd = round(row["tick"] / row["day"]) if row["day"] else 1440
                kind = row["sample_kind"]
                if kind == "day":
                    seed.days[int(round(row["day"]))] = row
                elif kind == "clock":
                    seed.clocks[(display_day(row, tpd), int(row["minute_of_day"]))] = row
                else:
                    seed.periodic.append(row)
        if set(seed.days) != set(range(1, duration + 1)):
            raise EvidenceError(f"{file}: day checkpoints incomplete")
        seeds.append(seed)
    expected = summary["seeds"]["end_exclusive"] - summary["seeds"]["start"]
    if len(seeds) != expected:
        raise EvidenceError(f"{path}: {len(seeds)} raw seeds, summary declares {expected}")
    seeds.sort(key=lambda s: s.seed)
    return Round(path, duration, seeds, summary)


def minute_in_window(minute: int, start: int, end: int) -> bool:
    """Half-open, possibly midnight-wrapping minute window (as the simulation)."""
    if start < end:
        return start <= minute < end
    if start > end:
        return minute >= start or minute < end
    return False


def store_shares(seed: Seed, last_day: int, seasons=None) -> tuple[list[float], float, float]:
    """Per seed-day store-meal shares from day 3 (days with meals only), and the
    seed's store and total meal sums over the same days."""
    shares, store_sum, total_sum = [], 0.0, 0.0
    for day in range(3, last_day + 1):
        if seasons is not None and window_season(seed, day) not in seasons:
            continue
        store = daily_delta(seed, day, "meals_from_store_total")
        total = store + daily_delta(seed, day, "ground_meals_total")
        if total > 0:
            shares.append(store / total)
            store_sum += store
            total_sum += total
    return shares, store_sum, total_sum


def evaluate(short: Round, long: Round, baseline_ms: float | None, receipts: dict[str, bool],
             long_baseline_ms: float | None = None, winter_burn_per_day: float = 39.0,
             cold_window: tuple[int, int] = (1320, 360)) -> dict:
    """X1-X12 of PASS5_GOAL.md section 11."""
    for r in (short, long):
        with_unit_denominator(r)
    result = {}
    result["X1"] = target({
        "twins_10": clause(short.summary["determinism_checked"] and len(short.seeds) == 64, len(short.seeds), "64 seeds, determinism checked"),
        "twins_120": clause(long.summary["determinism_checked"] and len(long.seeds) == 32, len(long.seeds), "32 seeds, determinism checked"),
        **{name: clause(receipts.get(name, False), receipts.get(name, False), f"{name} verified externally")
           for name in ("legacy_pin", "v8_repro", "v9_repro", "v10_repro")},
    })
    clauses = {}
    for key in ("deaths_starvation", "deaths_dehydration"):
        s, l = quantiles(short.day_values(10, key)), quantiles(long.day_values(long.duration, key))
        clauses[f"{key}_10"] = clause(s["max"] == 0, s, "zero in every seed at day 10")
        clauses[f"{key}_{long.duration}"] = clause(l["median"] == 0 and l["max"] <= 2, l, "total median 0 and max <= 2")
    result["X2"] = target(clauses)

    settlements = quantiles([float(seed.days[d]["settlements"]) for r in (short, long) for seed in r.seeds for d in seed.days])
    birth4 = quantiles(short.day_values(4, "births"))
    alive10 = quantiles(short.day_values(10, "alive"))
    stagnation = quantiles([float(longest_stagnation(seed, long.duration)) for seed in long.seeds])
    last = long.duration
    old30 = quantiles(long.day_values(30, "deaths_old_age"))
    margin = quantiles([(seed.days[last]["births"] - seed.days[29]["births"]) - (total_deaths(seed.days[last]) - total_deaths(seed.days[29])) for seed in long.seeds])
    band = quantiles([float(seed.days[d]["alive"]) for seed in long.seeds for d in seed.days if d >= 30])
    child_share = quantiles([ratio(seed.days[last]["children"], seed.days[last]["alive"]) for seed in long.seeds])
    elder_share = quantiles([ratio(seed.days[last]["elders"], seed.days[last]["alive"]) for seed in long.seeds])
    result["X3"] = target({
        "one_settlement": clause(settlements["min"] == settlements["max"] == 1, settlements, "one settlement at every day sample"),
        "birth_by_day4": clause(birth4["min"] >= 1, birth4, "a birth by day 4 in every seed (10-day round)"),
        "alive_day10": clause(alive10["min"] >= 30, alive10, "day-10 alive >= 30 in every seed"),
        "stagnation": clause(stagnation["max"] <= 4, stagnation, "longest stagnant run <= 4 days in every seed"),
        "old_age_by_day30": clause(old30["min"] >= 1, old30, ">= 1 old-age death by day 30 in every seed"),
        "replacement": clause(margin["min"] >= -3, margin, f"births >= deaths - 3 over days 30..{last} in every seed"),
        "alive_band": clause(32 <= band["min"] and band["max"] <= 44, band, "alive in [32, 44] at every day sample from day 30"),
        "child_share": clause(.03 <= child_share["median"] <= .20, child_share, f"day-{last} child share median in [0.03, 0.20] (W6)"),
        "elder_share": clause(.08 <= elder_share["median"] <= .35, elder_share, f"day-{last} elder share median in [0.08, 0.35] (W6)"),
    })

    seed_day_shares, per_seed, winter = [], [], []
    for seed in long.seeds:
        shares, store, total = store_shares(seed, last)
        seed_day_shares.extend(shares)
        per_seed.append(ratio(store, total))
        winter.extend(store_shares(seed, last, {WINTER})[0])
    all_days = quantiles(seed_day_shares)
    seeds_q = quantiles(per_seed)
    winter_q = quantiles(winter)
    result["X4"] = target({
        "store_share": clause(all_days["median"] >= .60, all_days, "store meals / all meals per seed-day from day 3, median >= 0.60"),
        "every_seed": clause(seeds_q["min"] >= .40, seeds_q, "each seed's store share from day 3 >= 0.40"),
        "winter": clause(winter_q["median"] >= .40, winter_q, "winter seed-day store share median >= 0.40"),
    })

    clauses = {}
    for index, name in enumerate(("spring", "summer", "autumn", "winter")):
        trips = quantiles([daily_delta(seed, d, "forage_trips_total") for seed in long.seeds for d in seed.days
                           if window_season(seed, d) == index])
        clauses[name] = clause(trips["n"] > 0 and trips["median"] >= 8, trips, f"forage trips per day median >= 8 in {name}")
    result["X5"] = target(clauses)

    lit = []
    for seed in long.seeds:
        values = [float(row["fire_lit"]) for (day, minute), row in seed.clocks.items() if minute == NIGHT and day >= 3]
        if values:
            lit.append(sum(values) / len(values))
    lit_q = quantiles(lit)
    closings = [row for seed in long.seeds for row in autumn_closings(seed)]
    required = .8 * 7 * winter_burn_per_day
    if closings:
        stock = quantiles([float(row["wood_stock"]) for row in closings])
        autumn = clause(stock["median"] >= required, {"wood_stock": stock, "required": required},
                        f"median last-autumn wood stock >= 0.8 x 7 x {winter_burn_per_day}")
    else:
        autumn = clause(False, "no autumn observed", "last-autumn wood stock; needs seasons and >= 21 days")
    result["X6"] = target({
        "lit_at_0200": clause(lit_q["n"] > 0 and lit_q["median"] >= .90 and lit_q["min"] >= .75, lit_q,
                              "share of 02:00 samples from day 3 with the fire lit: median >= 0.90, every seed >= 0.75"),
        "autumn_wood": autumn,
    })

    cold = []
    for seed in long.seeds:
        rows = [row for row in seed.periodic if row["season"] == WINTER and row["days_per_season"] > 0
                and minute_in_window(int(row["minute_of_day"]), *cold_window)]
        if rows:
            cold.append(ratio(sum(r["cold_members"] for r in rows), sum(r["settled_agents"] for r in rows)))
    cold_q = quantiles(cold)
    result["X7"] = target({"winter_cold": clause(cold_q["n"] > 0 and cold_q["median"] <= .10 and cold_q["max"] <= .25, cold_q,
                                                 "winter cold members / members in the cold window: median <= 0.10, every seed <= 0.25")})

    work = {SPRING, SUMMER, AUTUMN}
    noon_work = quantiles(per_seed_median(long, NOON, "adult_work_share", "one", work))
    afternoon_work = quantiles(per_seed_median(long, AFTERNOON, "adult_work_share", "one", work))
    noon_idle = quantiles(per_seed_median(long, NOON, "idle_share", "one", work))
    result["X8"] = target({
        "work_1200": clause(noon_work["median"] >= .35, noon_work, "adult work share at 12:00 median >= 0.35 (spring-autumn)"),
        "work_1500": clause(afternoon_work["median"] >= .25, afternoon_work, "adult work share at 15:00 median >= 0.25 (spring-autumn)"),
        "idle_1200": clause(noon_idle["median"] <= .60, noon_idle, "idle share of all residents at 12:00 median <= 0.60 (spring-autumn)"),
    })

    night = quantiles(per_seed_median(long, NIGHT, "sleeping_in_shelter", "alive"))
    evening = quantiles(per_seed_median(long, EVENING, "idle_adults_near_hearth", "idle_adults"))
    result["X9"] = target({
        "night_in_shelter": clause(night["median"] >= .80, night, "median >= 0.80"),
        "evening_near_hearth": clause(evening["median"] >= .55 and evening["min"] >= .35, evening, "median >= 0.55 and every seed >= 0.35"),
    })

    home = quantiles(short.day_values(10, "home_distance_mean"))
    clauses = {"home_distance": clause(home["median"] <= 8.1 and home["max"] <= 13, home, "day-10 median <= 8.1, every seed <= 13")}
    for r in (short, long):
        t6a = r.summary["days"][str(r.duration)]["idle_breach_actionable_share"]
        social = quantiles(run_values(r, "social_completed_per_agent_day"))
        clauses[f"t6a_{r.duration}"] = clause(t6a["median"] <= .01, t6a, "cumulative actionable idle-breach share median <= 1%")
        clauses[f"social_{r.duration}"] = clause(social["median"] >= .40 and social["min"] > 0, social, "median >= 0.40 and > 0 in every seed")
    result["X10"] = target(clauses)

    clauses = {}
    for r in (short, long):
        worst = max((row["gather_seekers_critical"] for seed in r.seeds for row in seed.periodic), default=0)
        clauses[f"day{r.duration}"] = clause(worst == 0, worst, "no gatherer with hunger or thirst >= 90 at any 120-tick sample")
    result["X11"] = target(clauses)

    clauses = {}
    for r, base in ((short, baseline_ms), (long, long_baseline_ms or baseline_ms)):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = 1.5 * base if base else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit, {**timing, "limit": limit},
                                             "median ms/agent-tick <= 1.5 x same-host game_v10")
    result["X12"] = target(clauses)
    return result


def with_unit_denominator(r: Round) -> None:
    """Give every clock row a constant `one` so share keys can use per_seed_median."""
    for seed in r.seeds:
        for row in seed.clocks.values():
            row["one"] = 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    baseline = parser.add_mutually_exclusive_group()
    baseline.add_argument("--baseline-summary", type=Path, help="same-host game_v10 summary.json (10 days)")
    baseline.add_argument("--baseline-ms", type=float)
    parser.add_argument("--baseline-summary-long", type=Path, help="same-host game_v10 summary of the long duration")
    parser.add_argument("--winter-burn-per-day", type=float, default=39.0)
    parser.add_argument("--cold-window", default="1320,360")
    for name in ("legacy-pin", "v8-repro", "v9-repro", "v10-repro"):
        parser.add_argument(f"--{name}", choices=("PASS", "FAIL"), default="FAIL")
    args = parser.parse_args(argv)
    try:
        def baseline_of(path: Path | None) -> float | None:
            if path is None:
                return None
            source = json.loads(path.read_text())
            if source["scene"] != "game_v10":
                raise EvidenceError("timing baseline must be a game_v10 summary")
            return float(source["timing"]["ms_per_agent_tick"]["median"])
        baseline_ms = args.baseline_ms if args.baseline_ms is not None else baseline_of(args.baseline_summary)
        long_baseline_ms = baseline_of(args.baseline_summary_long)
        start, end = (int(v) for v in args.cold_window.split(","))
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        receipts = {key.replace("-", "_"): getattr(args, key.replace("-", "_")) == "PASS"
                    for key in ("legacy-pin", "v8-repro", "v9-repro", "v10-repro")}
        targets = evaluate(short, long, baseline_ms, receipts, long_baseline_ms, args.winter_burn_per_day, (start, end))
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for t in targets.values())
    report = {"schema_version": 1, "passed": passed, "targets": targets, "baseline_ms": baseline_ms,
              "long_baseline_ms": long_baseline_ms, "winter_burn_per_day": args.winter_burn_per_day,
              "inputs": {"round10": str(args.round10), "round120": str(args.round120)}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "x_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    lines = ["# Pass 5 X targets", "", "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
    for name, entry in targets.items():
        for key, item in entry["clauses"].items():
            lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {json.dumps(item['observed'], sort_keys=True)} |")
    (args.out / "x_targets.md").write_text("\n".join(lines) + "\n")
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
