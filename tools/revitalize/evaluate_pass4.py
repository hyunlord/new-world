#!/usr/bin/env python3
"""Evaluate the Pass 4 W targets from raw probe JSONL. Stdlib only.

usage: evaluate_pass4.py --round10 DIR --round120 DIR --out DIR
                         [--baseline-summary V9_SUMMARY.json | --baseline-ms MS]
                         [--legacy-pin PASS|FAIL] [--v8-repro PASS|FAIL] [--v9-repro PASS|FAIL]

DIR holds `<scene>_s<seed>.jsonl` raw samples and `summary.json` written by
worldsim_probe (schema 4). Per-seed values are computed first and only then
aggregated; no target is computed from marginal medians of different seeds
unless the requirement itself is a median.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

NOON, NIGHT, EVENING = 720, 120, 1170
SPRING, SUMMER, AUTUMN, WINTER = range(4)


class EvidenceError(Exception):
    """Raw evidence is missing, inconsistent or of the wrong schema."""


def quantiles(values: list[float]) -> dict[str, float]:
    """Linear-interpolated median, p10, p90, min and max (all zero when empty)."""
    if not values:
        return {"median": 0.0, "p10": 0.0, "p90": 0.0, "min": 0.0, "max": 0.0, "n": 0}

    def at(fraction: float) -> float:
        ordered = sorted(values)
        position = fraction * (len(ordered) - 1)
        lo, hi = math.floor(position), math.ceil(position)
        return ordered[lo] + (ordered[hi] - ordered[lo]) * (position - lo)

    return {"median": at(.5), "p10": at(.1), "p90": at(.9), "min": at(0.), "max": at(1.), "n": len(values)}


def ratio(numerator: float, denominator: float) -> float:
    return numerator / denominator if denominator else 0.0


@dataclass
class Seed:
    """One seed's samples: day checkpoints, clock snapshots and periodic rows."""
    seed: int
    days: dict[int, dict] = field(default_factory=dict)
    clocks: dict[tuple[int, int], dict] = field(default_factory=dict)
    periodic: list[dict] = field(default_factory=list)


@dataclass
class Round:
    path: Path
    duration: int
    seeds: list[Seed]
    summary: dict

    def day_values(self, day: int, key: str) -> list[float]:
        return [float(s.days[day][key]) for s in self.seeds]


def display_day(row: dict, ticks_per_day: int) -> int:
    """One-based display day of a raw row, from its tick and minute of day."""
    tick, minute = int(row["tick"]), int(row["minute_of_day"])
    offset = (minute - tick % ticks_per_day) % ticks_per_day
    return (tick + offset) // ticks_per_day + 1


def load_round(path: Path, duration: int) -> Round:
    summary = json.loads((path / "summary.json").read_text())
    if summary.get("schema_version") != 4:
        raise EvidenceError(f"{path}: summary schema must be 4")
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
                if row.get("schema_version") != 4:
                    raise EvidenceError(f"{file}: raw rows must be schema 4")
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


def clause(passed: bool, observed, requirement: str) -> dict:
    return {"passed": bool(passed), "observed": observed, "requirement": requirement}


def target(clauses: dict) -> dict:
    return {"passed": all(c["passed"] for c in clauses.values()), "clauses": clauses}


def clock_ratio(seed: Seed, minute: int, numerator: str, denominator: str, seasons=None) -> list[float]:
    """Per-day ratios at a display minute, days 2..N, optionally restricted to seasons."""
    return [ratio(row[numerator], row[denominator]) for (day, m), row in sorted(seed.clocks.items())
            if m == minute and day >= 2 and (seasons is None or row["season"] in seasons)]


def per_seed_median(r: Round, minute: int, numerator: str, denominator: str, seasons=None) -> list[float]:
    medians = []
    for seed in r.seeds:
        values = clock_ratio(seed, minute, numerator, denominator, seasons)
        if values:
            medians.append(quantiles(values)["median"])
    return medians


def daily_delta(seed: Seed, day: int, key: str) -> float:
    before = seed.days[day - 1][key] if day > 1 else 0
    return float(seed.days[day][key]) - float(before)


def longest_stagnation(seed: Seed, last_day: int) -> int:
    """Longest run of days 2..N with members <= sustainable - 3 and no birth that day."""
    best = run = 0
    for day in range(2, last_day + 1):
        row = seed.days[day]
        stagnant = row["settled_agents"] <= row["sustainable_adults"] - 3 and daily_delta(seed, day, "births") == 0
        run = run + 1 if stagnant else 0
        best = max(best, run)
    return best


def autumn_closings(seed: Seed) -> list[dict]:
    """The last periodic sample of each autumn (season 2 followed by another season or the end)."""
    rows = [row for row in seed.periodic if row["days_per_season"] > 0]
    closings = []
    for index, row in enumerate(rows):
        following = rows[index + 1]["season"] if index + 1 < len(rows) else None
        if row["season"] == AUTUMN and following not in (None, AUTUMN):
            closings.append(row)
    return closings


def total_deaths(row: dict) -> float:
    return sum(float(row[k]) for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age"))


def run_values(r: Round, key: str) -> list[float]:
    return [float(run[key]) for run in r.summary["runs"]]


def evaluate(short: Round, long: Round, baseline_ms: float | None, receipts: dict[str, bool]) -> dict:
    result = {}
    result["W1"] = target({
        "twins_10": clause(short.summary["determinism_checked"] and len(short.seeds) == 64, len(short.seeds), "64 seeds, determinism checked"),
        "twins_120": clause(long.summary["determinism_checked"] and len(long.seeds) == 32, len(long.seeds), "32 seeds, determinism checked"),
        **{name: clause(receipts.get(name, False), receipts.get(name, False), f"{name} verified externally") for name in ("legacy_pin", "v8_repro", "v9_repro")},
    })
    for wid, key, (short_max, long_max) in (("W2", "deaths_dehydration", (1, 2)), ("W3", "deaths_starvation", (0, 3))):
        s, l = quantiles(short.day_values(10, key)), quantiles(long.day_values(long.duration, key))
        short_ok = s["max"] == 0 if wid == "W3" else (s["median"] == 0 and s["max"] <= short_max)
        result[wid] = target({
            "day_10": clause(short_ok, s, "zero in every seed" if wid == "W3" else f"median 0, max <= {short_max}"),
            f"day_{long.duration}": clause(l["median"] == 0 and l["max"] <= long_max, l, f"total median 0, max <= {long_max}"),
        })
    settlements = quantiles([float(seed.days[d]["settlements"]) for r in (short, long) for seed in r.seeds for d in seed.days])
    result["W4"] = target({"every_day": clause(settlements["min"] == settlements["max"] == 1, settlements, "one settlement at every day sample from day 1")})
    birth4 = quantiles(short.day_values(4, "births"))
    alive10 = quantiles(short.day_values(10, "alive"))
    stagnation = quantiles([float(longest_stagnation(seed, long.duration)) for seed in long.seeds])
    result["W5"] = target({
        "birth_by_day4": clause(birth4["min"] >= 1, birth4, "births >= 1 by the day-4 checkpoint in every seed (10-day round)"),
        "alive_day10": clause(alive10["min"] >= 30, alive10, "day-10 alive >= 30 in every seed"),
        "stagnation": clause(stagnation["max"] <= 4, stagnation, "longest run of days 2..N with members <= sustainable-3 and no birth <= 4 in every seed"),
    })
    old30 = quantiles(long.day_values(30, "deaths_old_age"))
    margin = quantiles([(seed.days[long.duration]["births"] - seed.days[30]["births"]) - (total_deaths(seed.days[long.duration]) - total_deaths(seed.days[30])) for seed in long.seeds])
    band = quantiles([float(seed.days[d]["alive"]) for seed in long.seeds for d in seed.days if d >= 30])
    last = long.duration
    child_share = quantiles([ratio(seed.days[last]["children"], seed.days[last]["alive"]) for seed in long.seeds])
    elder_share = quantiles([ratio(seed.days[last]["elders"], seed.days[last]["alive"]) for seed in long.seeds])
    result["W6"] = target({
        "old_age_by_day30": clause(old30["min"] >= 1, old30, ">= 1 old-age death by day 30 in every seed"),
        "replacement": clause(margin["min"] >= -3, margin, f"births - deaths over days 30..{last} >= -3 in every seed"),
        "alive_band": clause(32 <= band["min"] and band["max"] <= 44, band, "every day sample from day 30 has alive in [32, 44]"),
        "child_share": clause(.03 <= child_share["median"] <= .20, child_share, f"day-{last} child share median in [0.03, 0.20]"),
        "elder_share": clause(.08 <= elder_share["median"] <= .35, elder_share, f"day-{last} elder share median in [0.08, 0.35]"),
    })
    closings = [(row, row["days_per_season"] * max(0.0, row["food_need_per_day"] - row["food_supply_winter_per_day"])) for seed in long.seeds for row in autumn_closings(seed)]
    hoard = [max((row["stockpile_food"] for row in seed.periodic), default=0) - 3 * max((row["stock_target"] for row in seed.periodic), default=0) for seed in long.seeds]
    if closings:
        stock = quantiles([float(row["stockpile_food"]) for row, _ in closings])
        need = quantiles([.8 * shortfall for _, shortfall in closings])
        paired = quantiles([row["stockpile_food"] - .8 * shortfall for row, shortfall in closings])
        autumn = clause(stock["median"] >= need["median"], {"stock": stock, "required": need, "paired_margin": paired}, "median autumn-closing stock >= median 0.8 x days_per_season x max(0, need - winter supply)")
    else:
        autumn = clause(False, "no autumn observed (seasons off or run too short)", "autumn-closing store; not applicable without seasons")
    result["W7"] = target({"autumn_store": autumn, "hoard_cap": clause(max(hoard, default=0) <= 0, quantiles(hoard), "max stock - 3 x max target <= 0 in every seed")})
    seasons_on = any(row["days_per_season"] for seed in long.seeds for row in seed.periodic)
    work_seasons = {SPRING, SUMMER, AUTUMN} if seasons_on else None
    noon = quantiles(per_seed_median(long, NOON, "state_idle", "alive", work_seasons))
    trips = [daily_delta(seed, d, "forage_trips_total") for seed in long.seeds for d in seed.days
             if not seasons_on or seed.days[d]["season"] in (SUMMER, AUTUMN)]
    trips_stats = quantiles(trips)
    result["W8"] = target({
        "noon_idle": clause(noon["median"] <= .45, noon, "noon idle share median <= 0.45 over spring-autumn days (all days without seasons)"),
        "forage_trips": clause(trips_stats["median"] >= 10, trips_stats, "forage trips per day median >= 10 in summer and autumn (all days without seasons)"),
    })
    night = quantiles(per_seed_median(long, NIGHT, "sleeping_in_shelter", "alive"))
    evening = quantiles(per_seed_median(long, EVENING, "idle_adults_near_hearth", "idle_adults"))
    result["W9"] = target({
        "night_in_shelter": clause(night["median"] >= .70, night, "median >= 0.70"),
        "evening_near_hearth": clause(evening["median"] >= .50 and evening["min"] >= .30, evening, "median >= 0.50 and every seed >= 0.30"),
    })
    home = quantiles(short.day_values(10, "home_distance_mean"))
    clauses = {"home_distance": clause(home["median"] <= 7.0 and home["max"] <= 10, home, "day-10 median <= 7.0, every seed <= 10")}
    for r in (short, long):
        # The cumulative share is added to checkpoints after raw rows are written,
        # so it is read from the summary distribution (the requirement is a median).
        t6a = r.summary["days"][str(r.duration)]["idle_breach_actionable_share"]
        social = quantiles(run_values(r, "social_completed_per_agent_day"))
        clauses[f"t6a_{r.duration}"] = clause(t6a["median"] <= .01, t6a, "cumulative actionable idle-breach share median <= 1%")
        clauses[f"social_{r.duration}"] = clause(social["median"] >= .40 and social["min"] > 0, social, "median >= 0.40 and > 0 in every seed")
    result["W10"] = target(clauses)
    clauses = {}
    for r in (short, long):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = 1.5 * baseline_ms if baseline_ms else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit, {**timing, "limit": limit}, "median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9")
    result["W11"] = target(clauses)
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    baseline = parser.add_mutually_exclusive_group()
    baseline.add_argument("--baseline-summary", type=Path, help="same-host game_v9 summary.json")
    baseline.add_argument("--baseline-ms", type=float)
    for name in ("legacy-pin", "v8-repro", "v9-repro"):
        parser.add_argument(f"--{name}", choices=("PASS", "FAIL"), default="FAIL")
    args = parser.parse_args(argv)
    try:
        baseline_ms = args.baseline_ms
        if args.baseline_summary:
            source = json.loads(args.baseline_summary.read_text())
            if source["scene"] != "game_v9":
                raise EvidenceError("timing baseline must be a game_v9 summary")
            baseline_ms = float(source["timing"]["ms_per_agent_tick"]["median"])
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        receipts = {"legacy_pin": args.legacy_pin == "PASS", "v8_repro": args.v8_repro == "PASS", "v9_repro": args.v9_repro == "PASS"}
        targets = evaluate(short, long, baseline_ms, receipts)
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for t in targets.values())
    report = {"schema_version": 1, "passed": passed, "targets": targets, "baseline_ms": baseline_ms,
              "inputs": {"round10": str(args.round10), "round120": str(args.round120)}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "w_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    lines = ["# Pass 4 W targets", "", "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
    for name, entry in targets.items():
        for key, item in entry["clauses"].items():
            lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {json.dumps(item['observed'], sort_keys=True)} |")
    (args.out / "w_targets.md").write_text("\n".join(lines) + "\n")
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
