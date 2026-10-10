#!/usr/bin/env python3
"""Evaluate the Pass 6 Y targets from raw probe JSONL. Stdlib only.

usage: evaluate_pass6.py --round10 DIR --round120 DIR --out DIR
                         [--baseline-summary V11_10DAY.json | --baseline-ms MS]
                         [--baseline-summary-long V11_120DAY.json]
                         [--v11-targets V11_Y_TARGETS.json]
                         [--winter-burn-per-day N] [--cold-window START,END]
                         [--legacy-pin PASS|FAIL] [--v8-repro PASS|FAIL] [--v9-repro PASS|FAIL]
                         [--v10-repro PASS|FAIL] [--v11-repro PASS|FAIL]

DIR holds `<scene>_s<seed>.jsonl` raw samples and `summary.json` written by
worldsim_probe (schema 6). The 10-day round is 64 x 10; the long round is
32 x N with clock samples at 02:00, 12:00, 15:00 and 19:30 at least. Per-seed
values are computed first and only then aggregated.

Night booking (every night measure): a sample taken after midnight and before
the end of the night belongs to the season of the previous display day, the
evening the night began. A 02:00 clock sample on display day D is the night of
the evening of day D - 1.

`--v11-targets` is the `y_targets.json` of the same evaluation run on game_v11
(Part 1): Y5's autumn noon work and Y8's winter food store compare with its
`measures`. `--winter-burn-per-day` (`burn_per_hour[3] x burn hours`) and
`--cold-window` come from `game_v12()`.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from evaluate_pass4 import (EvidenceError, Round, Seed, clause, daily_delta, longest_stagnation,
                            per_seed_median, quantiles, ratio, run_values, target, total_deaths, window_season)
from evaluate_pass5 import load_round as load_round_schema, minute_in_window, store_shares, with_unit_denominator

NIGHT, NOON, AFTERNOON, EVENING = 120, 720, 900, 1170
SPRING, SUMMER, AUTUMN, WINTER = range(4)
SEASON_NAMES = ("spring", "summer", "autumn", "winter")
SCHEMA = 6
TPD = 1440
RECEIPTS = ("legacy_pin", "v8_repro", "v9_repro", "v10_repro", "v11_repro")


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-6 round; every seed must have every day checkpoint."""
    return load_round_schema(path, duration, SCHEMA)


def day_index(row: dict) -> int:
    """Zero-based display day of a raw row (the simulation's `day_index`)."""
    tick = int(row["tick"])
    return (tick + (int(row["minute_of_day"]) - tick % TPD) % TPD) // TPD


def system_day(row: dict) -> int:
    """Zero-based display day the simulation systems saw when the row was taken:
    rows are written after a completed tick, so their `season` belongs to tick - 1."""
    tick = max(int(row["tick"]) - 1, 0)
    return (tick + (int(row["minute_of_day"]) - int(row["tick"]) % TPD) % TPD) // TPD


def season_of_day(day: int, days_per_season: int) -> int:
    """Season of zero-based display day `day` (0 spring .. 3 winter); spring without seasons."""
    return (day // days_per_season) % 4 if days_per_season > 0 else SPRING


def night_season(row: dict, night_end: int) -> int:
    """Season a night sample is booked to: before `night_end` the previous display
    day's (the evening the night began), otherwise the row's own day."""
    day = day_index(row)
    if int(row["minute_of_day"]) < night_end:
        day -= 1
    return season_of_day(max(day, 0), int(row["days_per_season"]))


def system_minute(row: dict) -> int:
    """Minute of day the simulation systems saw when the row was taken (`tick - 1`)."""
    return (int(row["minute_of_day"]) - 1) % TPD


def system_night_season(row: dict, night_end: int) -> int:
    """`night_season` of the system tick: a cold-window row reflects the hourly fire
    update of the tick before it, so its window and booking use that tick."""
    day = system_day(row)
    if system_minute(row) < night_end:
        day -= 1
    return season_of_day(max(day, 0), int(row["days_per_season"]))


def night_of(row: dict) -> int:
    """Zero-based display day of the evening a 02:00 sample's night began."""
    return day_index(row) - 1


def nights_at_0200(seed: Seed) -> list[tuple[int, int, dict]]:
    """(evening day, booked season, row) of every 02:00 clock sample from display day 3 (one-based)."""
    rows = []
    for (day, minute), row in sorted(seed.clocks.items()):
        if minute == NIGHT and day >= 3:
            rows.append((night_of(row), night_season(row, 720), row))
    return rows


def lit_shares(r: Round) -> tuple[list[float], list[float]]:
    """Per seed: share of winter nights lit at 02:00, and of all other nights."""
    winter, other = [], []
    for seed in r.seeds:
        nights = nights_at_0200(seed)
        w = [float(row["fire_lit"]) for _, s, row in nights if s == WINTER]
        o = [float(row["fire_lit"]) for _, s, row in nights if s != WINTER]
        if w:
            winter.append(sum(w) / len(w))
        if o:
            other.append(sum(o) / len(o))
    return winter, other


def seasons_on(r: Round) -> int:
    for seed in r.seeds:
        for row in seed.periodic:
            if row["days_per_season"]:
                return int(row["days_per_season"])
    return 0


def winter_windows(seed: Seed, last_day: int, dps: int) -> list[list[int]]:
    """Complete winters as lists of elapsed days (checkpoint d-1 to d) whose window season is winter."""
    if dps <= 0:
        return []
    groups: dict[int, list[int]] = {}
    for day in range(2, last_day + 1):
        if window_season(seed, day) == WINTER:
            groups.setdefault((day - 1) // (4 * dps), []).append(day)
    return [days for _, days in sorted(groups.items()) if len(days) == dps]


def winter_closings(seed: Seed) -> list[dict]:
    """The last periodic sample of each winter (season 3 followed by another season)."""
    rows = [row for row in seed.periodic if row["days_per_season"] > 0]
    return [row for index, row in enumerate(rows)
            if row["season"] == WINTER and index + 1 < len(rows) and rows[index + 1]["season"] != WINTER]


def winters_table(r: Round, cold_end: int) -> list[dict]:
    """One row per complete seed-winter (see the module docstring for booking)."""
    dps = seasons_on(r)
    table = []
    for seed in r.seeds:
        closings = winter_closings(seed)
        nights = nights_at_0200(seed)
        for days in winter_windows(seed, r.duration, dps):
            first, year = days[0], (days[0] - 1) // (4 * dps)
            # The opening checkpoint of the first winter day (display 08:00).
            opening = seed.days[first - 1]
            evenings = [(evening, row) for evening, s, row in nights if s == WINTER and evening // (4 * dps) == year]
            out = [evening for evening, row in evenings if not row["fire_lit"]]
            closing = next((row for row in closings if system_day(row) // (4 * dps) == year), None)
            table.append({
                "seed": seed.seed, "year": year, "first_display_day": first - 1,
                "wood_in_hand_at_start": float(opening["wood_in_hand"]),
                "nights": len(evenings), "nights_out": len(out),
                "first_night_out": (out[0] - (first - 1) + 1) if out else 0,
                "trip_days": sum(1 for d in days if daily_delta(seed, d, "wood_trips_total") > 0),
                "wood_trips": sum(daily_delta(seed, d, "wood_trips_total") for d in days),
                "urgent_trips": sum(daily_delta(seed, d, "urgent_wood_trips_total") for d in days),
                "wood_gathered": sum(daily_delta(seed, d, "wood_gathered_total") for d in days),
                "cold_member_hours": sum(daily_delta(seed, d, "cold_member_hours_total") for d in days),
                "food_store_at_end": float(closing["stockpile_food"]) if closing else None,
            })
    return table


def season_days(r: Round, season: int, key: str) -> list[float]:
    """Per-day counter differences over every seed-day whose window season is `season`."""
    return [daily_delta(seed, d, key) for seed in r.seeds for d in seed.days if window_season(seed, d) == season]


def season_checkpoints(r: Round, season: int, key: str) -> list[float]:
    return [float(seed.days[d][key]) for seed in r.seeds for d in seed.days if seed.days[d]["season"] == season]


def measures(long: Round, cold_window: tuple[int, int]) -> dict:
    """Numbers Pass 6 compares across versions (written into the report)."""
    with_unit_denominator(long)
    autumn_work = quantiles(per_seed_median(long, NOON, "adult_work_share", "one", {AUTUMN}))
    winter_food = [w["food_store_at_end"] for w in winters_table(long, cold_window[1]) if w["food_store_at_end"] is not None]
    return {"autumn_work_1200": autumn_work, "winter_food_store_at_end": quantiles(winter_food)}


def evaluate(short: Round, long: Round, baseline_ms: float | None, receipts: dict[str, bool],
             long_baseline_ms: float | None = None, v11: dict | None = None,
             cold_window: tuple[int, int] = (1320, 360)) -> dict:
    """Y1-Y13 of PASS6_GOAL.md section 11. `v11` is the `measures` of the v11 evaluation."""
    for r in (short, long):
        with_unit_denominator(r)
    result = {}
    result["Y1"] = target({
        "twins_10": clause(short.summary["determinism_checked"] and len(short.seeds) == 64, len(short.seeds), "64 seeds, determinism checked"),
        "twins_120": clause(long.summary["determinism_checked"] and len(long.seeds) == 32, len(long.seeds), "32 seeds, determinism checked"),
        **{name: clause(receipts.get(name, False), receipts.get(name, False), f"{name} verified externally") for name in RECEIPTS},
    })
    clauses = {}
    for key in ("deaths_starvation", "deaths_dehydration"):
        s, l = quantiles(short.day_values(10, key)), quantiles(long.day_values(long.duration, key))
        clauses[f"{key}_10"] = clause(s["max"] == 0, s, "zero in every seed at day 10")
        clauses[f"{key}_{long.duration}"] = clause(l["median"] == 0 and l["max"] <= 2, l, "total median 0 and max <= 2")
    result["Y2"] = target(clauses)

    last = long.duration
    settlements = quantiles([float(seed.days[d]["settlements"]) for r in (short, long) for seed in r.seeds for d in seed.days])
    birth4 = quantiles(short.day_values(4, "births"))
    alive10 = quantiles(short.day_values(10, "alive"))
    stagnation = quantiles([float(longest_stagnation(seed, last)) for seed in long.seeds])
    old30 = quantiles(long.day_values(30, "deaths_old_age"))
    margin = quantiles([(seed.days[last]["births"] - seed.days[29]["births"]) - (total_deaths(seed.days[last]) - total_deaths(seed.days[29])) for seed in long.seeds])
    band = quantiles([float(seed.days[d]["alive"]) for seed in long.seeds for d in seed.days if d >= 30])
    child_share = quantiles([ratio(seed.days[last]["children"], seed.days[last]["alive"]) for seed in long.seeds])
    elder_share = quantiles([ratio(seed.days[last]["elders"], seed.days[last]["alive"]) for seed in long.seeds])
    result["Y3"] = target({
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
    all_days, seeds_q, winter_q = quantiles(seed_day_shares), quantiles(per_seed), quantiles(winter)
    result["Y4"] = target({
        "store_share": clause(all_days["median"] >= .60, all_days, "store meals / all meals per seed-day from day 3, median >= 0.60"),
        "every_seed": clause(seeds_q["min"] >= .40, seeds_q, "each seed's store share from day 3 >= 0.40"),
        "winter": clause(winter_q["median"] >= .40, winter_q, "winter seed-day store share median >= 0.40"),
    })

    summer_wood = quantiles(season_days(long, SUMMER, "wood_gathered_total"))
    autumn_wood = quantiles(season_days(long, AUTUMN, "wood_gathered_total"))
    own = measures(long, cold_window)
    autumn_work = own["autumn_work_1200"]
    work_floor = v11["autumn_work_1200"]["median"] - .05 if v11 else None
    result["Y5"] = target({
        "autumn_wood": clause(autumn_wood["n"] > 0 and summer_wood["n"] > 0 and autumn_wood["median"] >= 1.5 * summer_wood["median"],
                              {"autumn": autumn_wood, "summer": summer_wood},
                              "autumn wood gathered per day median >= 1.5 x summer's (seed-days)"),
        "autumn_work": clause(work_floor is not None and autumn_work["median"] >= work_floor,
                              {**autumn_work, "floor": work_floor},
                              "autumn adult work share at 12:00 median >= v11's autumn value - 0.05"),
    })

    winters = winters_table(long, cold_window[1])
    trip_days = quantiles([float(w["trip_days"]) for w in winters])
    trips = quantiles([float(w["wood_trips"]) for w in winters])
    result["Y6"] = target({
        "trip_days": clause(trip_days["n"] > 0 and trip_days["median"] >= 4, trip_days, "days with wood trips per winter (of 7), median over seed-winters >= 4"),
        "trips": clause(trips["n"] > 0 and trips["median"] >= 20, trips, "wood trips per winter, median over seed-winters >= 20"),
    })

    winter_lit, other_lit = (quantiles(v) for v in lit_shares(long))
    result["Y7"] = target({
        "winter_lit": clause(winter_lit["n"] > 0 and .70 <= winter_lit["median"] <= .97 and winter_lit["min"] >= .50, winter_lit,
                             "per seed share of winter nights lit at 02:00: median in [0.70, 0.97], every seed >= 0.50"),
        "other_lit": clause(other_lit["n"] > 0 and other_lit["median"] >= .95 and other_lit["min"] >= .85, other_lit,
                            "outside winter, 02:00 lit share: median >= 0.95, every seed >= 0.85"),
    })

    cold = []
    for seed in long.seeds:
        # Rows are written after a completed tick: the 22:00 row shows the 21:00 fire
        # update (outside the window, nobody cold), the 06:00 row the 05:00 one. Use
        # the system tick for the window and the night booking (review round 1).
        rows = [row for row in seed.periodic if row["days_per_season"] > 0
                and minute_in_window(system_minute(row), *cold_window)
                and system_night_season(row, cold_window[1] if cold_window[0] > cold_window[1] else 0) == WINTER]
        if rows:
            cold.append(ratio(sum(r["cold_members"] for r in rows), sum(r["settled_agents"] for r in rows)))
    cold_q = quantiles(cold)
    food = own["winter_food_store_at_end"]
    food_floor = .8 * v11["winter_food_store_at_end"]["median"] if v11 else None
    result["Y8"] = target({
        "winter_cold": clause(cold_q["n"] > 0 and cold_q["median"] <= .25 and cold_q["max"] <= .50, cold_q,
                              "winter cold members / members in the cold window: median <= 0.25, every seed <= 0.50"),
        "winter_food": clause(food_floor is not None and food["n"] > 0 and food["median"] >= food_floor, {**food, "floor": food_floor},
                              "food store at each winter's last periodic sample, median >= 0.8 x v11's"),
    })

    work = {SPRING, SUMMER, AUTUMN}
    noon_work = quantiles(per_seed_median(long, NOON, "adult_work_share", "one", work))
    afternoon_work = quantiles(per_seed_median(long, AFTERNOON, "adult_work_share", "one", work))
    noon_idle = quantiles(per_seed_median(long, NOON, "idle_share", "one", work))
    result["Y9"] = target({
        "work_1200": clause(noon_work["median"] >= .35, noon_work, "adult work share at 12:00 median >= 0.35 (spring-autumn)"),
        "work_1500": clause(afternoon_work["median"] >= .25, afternoon_work, "adult work share at 15:00 median >= 0.25 (spring-autumn)"),
        "idle_1200": clause(noon_idle["median"] <= .60, noon_idle, "idle share of all residents at 12:00 median <= 0.60 (spring-autumn)"),
    })

    night = quantiles(per_seed_median(long, NIGHT, "sleeping_in_shelter", "alive"))
    evening = quantiles(per_seed_median(long, EVENING, "idle_adults_near_hearth", "idle_adults"))
    result["Y10"] = target({
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
    result["Y11"] = target(clauses)

    clauses = {}
    for r in (short, long):
        worst = max((row["gather_seekers_critical"] for seed in r.seeds for row in seed.periodic), default=0)
        clauses[f"day{r.duration}"] = clause(worst == 0, worst, "no gatherer with hunger or thirst >= 90 at any 120-tick sample")
    result["Y12"] = target(clauses)

    clauses = {}
    for r, base in ((short, baseline_ms), (long, long_baseline_ms or baseline_ms)):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = 1.2 * base if base else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit, {**timing, "limit": limit},
                                             "median ms/agent-tick <= 1.2 x same-host game_v11")
    result["Y13"] = target(clauses)
    return result


def work_by_season(long: Round) -> list[dict]:
    with_unit_denominator(long)
    rows = []
    for season, name in enumerate(SEASON_NAMES):
        entry = {"season": name}
        for minute, label in ((NOON, "1200"), (AFTERNOON, "1500")):
            entry[f"work_{label}"] = quantiles(per_seed_median(long, minute, "adult_work_share", "one", {season}))
            entry[f"idle_{label}"] = quantiles(per_seed_median(long, minute, "idle_share", "one", {season}))
        rows.append(entry)
    return rows


def wood_by_season(long: Round, cold_end: int) -> list[dict]:
    """Seasonal wood, fire, cold and food medians (seed-days; nights booked to their evening)."""
    rows = []
    nights = [(s, row) for seed in long.seeds for _, s, row in nights_at_0200(seed)]
    for season, name in enumerate(SEASON_NAMES):
        lit = [float(row["fire_lit"]) for s, row in nights if s == season]
        rows.append({
            "season": name,
            "wood_stock": quantiles(season_checkpoints(long, season, "wood_stock")),
            "wood_target": quantiles(season_checkpoints(long, season, "wood_target")),
            "wood_gathered": quantiles(season_days(long, season, "wood_gathered_total")),
            "wood_trips": quantiles(season_days(long, season, "wood_trips_total")),
            "urgent_trips": quantiles(season_days(long, season, "urgent_wood_trips_total")),
            "lit_0200": sum(lit) / len(lit) if lit else 0.0,
            "cold_member_hours": quantiles(season_days(long, season, "cold_member_hours_total")),
            "food_store": quantiles(season_checkpoints(long, season, "stockpile_food")),
            "forage_trips": quantiles(season_days(long, season, "forage_trips_total")),
        })
    return rows


def fmt(q: dict | float | None) -> str:
    if q is None:
        return "–"
    if isinstance(q, dict):
        return " / ".join(f"{q[k]:.3g}" for k in ("median", "p10", "p90", "min", "max"))
    return f"{q:.3g}"


def markdown(targets: dict, long: Round, cold_end: int, title: str) -> str:
    lines = [f"# {title}", "", "Values: median / p10 / p90 / min / max.", "",
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
    winters = winters_table(long, cold_end)
    lines += ["", "## Winters", "",
              "One row per complete seed-winter. Wood in hand at the opening checkpoint of the first winter day; nights out = winter nights (booked to their evening) with the fire out at 02:00; first night out = 1..7 (0 none); trip-days, trips, urgent trips, gathered and cold member-hours over the winter's seven elapsed days; food store at the winter's last periodic sample.", "",
              "| Seed | Year | Wood at start | Nights out | First night out | Trip-days | Wood trips | Urgent trips | Gathered | Cold member-hours | Food store at end |",
              "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for w in winters:
        food = "–" if w["food_store_at_end"] is None else f"{w['food_store_at_end']:.0f}"
        lines.append(f"| {w['seed']} | {w['year']} | {w['wood_in_hand_at_start']:.0f} | {w['nights_out']} | {w['first_night_out']} | {w['trip_days']} | "
                     f"{w['wood_trips']:.0f} | {w['urgent_trips']:.0f} | {w['wood_gathered']:.0f} | {w['cold_member_hours']:.0f} | {food} |")
    if winters:
        lines += ["", "| Winter measure | Median / p10 / p90 / min / max |", "|---|---|"]
        for key in ("wood_in_hand_at_start", "nights_out", "first_night_out", "trip_days", "wood_trips", "urgent_trips",
                    "wood_gathered", "cold_member_hours", "food_store_at_end"):
            values = [float(w[key]) for w in winters if w[key] is not None]
            lines.append(f"| {key} | {fmt(quantiles(values))} |")
        lines.append(f"| seed-winters with a night out | {sum(1 for w in winters if w['nights_out'])} of {len(winters)} |")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    baseline = parser.add_mutually_exclusive_group()
    baseline.add_argument("--baseline-summary", type=Path, help="same-host game_v11 summary.json (10 days)")
    baseline.add_argument("--baseline-ms", type=float)
    parser.add_argument("--baseline-summary-long", type=Path, help="same-host game_v11 summary of the long duration")
    parser.add_argument("--v11-targets", type=Path, help="y_targets.json of the game_v11 evaluation (Part 1)")
    parser.add_argument("--winter-burn-per-day", type=float, default=39.0)
    parser.add_argument("--cold-window", default="1320,360")
    parser.add_argument("--title", default="Pass 6 Y targets")
    for name in RECEIPTS:
        parser.add_argument(f"--{name.replace('_', '-')}", choices=("PASS", "FAIL"), default="FAIL")
    args = parser.parse_args(argv)
    try:
        def baseline_of(path: Path | None) -> float | None:
            if path is None:
                return None
            source = json.loads(path.read_text())
            if source["scene"] != "game_v11":
                raise EvidenceError("timing baseline must be a game_v11 summary")
            return float(source["timing"]["ms_per_agent_tick"]["median"])
        baseline_ms = args.baseline_ms if args.baseline_ms is not None else baseline_of(args.baseline_summary)
        long_baseline_ms = baseline_of(args.baseline_summary_long)
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        start, end = (int(v) for v in args.cold_window.split(","))
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        receipts = {name: getattr(args, name) == "PASS" for name in RECEIPTS}
        targets = evaluate(short, long, baseline_ms, receipts, long_baseline_ms, v11, (start, end))
        own = measures(long, (start, end))
        text = markdown(targets, long, end, args.title)
        winters = winters_table(long, end)
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for t in targets.values())
    report = {"schema_version": 1, "passed": passed, "targets": targets, "measures": own, "winters": winters,
              "work_by_season": work_by_season(long), "wood_by_season": wood_by_season(long, end),
              "baseline_ms": baseline_ms, "long_baseline_ms": long_baseline_ms,
              "winter_burn_per_day": args.winter_burn_per_day, "cold_window": [start, end],
              "inputs": {"round10": str(args.round10), "round120": str(args.round120),
                         "v11_targets": str(args.v11_targets) if args.v11_targets else None}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "y_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (args.out / "y_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
