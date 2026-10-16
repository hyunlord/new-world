#!/usr/bin/env python3
"""Evaluate the Pass 8 X targets from raw probe JSONL. Stdlib only.

usage: evaluate_pass8.py --round10 DIR --round120 DIR --out DIR
                         [--compare-to REFERENCE_X_TARGETS.json]
                         [--v11-targets P6_V11_Y_TARGETS.json] [--severity-permille W0,W1,W2]
                         [--baseline-summary V13_10DAY.json] [--baseline-summary-long V13_120DAY.json]
                         [--cold-window START,END] [--gathered-floor 40.5]
                         [--legacy-pin PASS|FAIL] [--v8-repro ...] ... [--v13-repro ...] [--x6 PASS|FAIL] [--x7 PASS|FAIL]

DIR holds `<scene>_s<seed>.jsonl` raw samples and `summary.json` written by
worldsim_probe (schema 8): 64 x 10 and 32 x N with the eight Pass 6 clocks.
The seeds may be any range (tuning 0-63 / 0-31, held-out 64-127 / 32-95).

Severity weights come from the run's `summary.json` `rules` block (PASS8_GOAL.md
6.2); `--severity-permille` overrides them and the override is recorded.

X2 (village health): Pass 6's Y2 with its thresholds, never in comparison mode;
every per-seed clause of Y3, Y7, Y8, Y10, Y11 and Y12 (a clause judged on its
minimum or maximum over seeds; for a clause that combines a median with an
every-seed bound, only the every-seed part, `..._every_seed`); and the
population clauses of Y3 (child and elder share medians). With
`--compare-to` (an evaluate_pass8 output of game_v12 on the same seeds) a
per-seed clause passes when this run's failing seeds number at most the
reference's + 1; both lists are reported. Without it a per-seed clause passes
when no seed fails. X3 judges the winters (section 9: severity-only R2, the
spread within normal and harsh winters, dark-night shares, first dark night),
X4 Pass 7's Z4, X5 Pass 6's Y4-Y6 and the population clauses of Y7, Y8, Y10
and Y11. X1, X6 and X7 are receipts; X8 compares the median ms per agent-tick
with same-host game_v13 summaries. Every per-seed value is stored in the JSON
(`per_seed`).
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

from evaluate_pass4 import (EvidenceError, Round, clause, clock_ratio, daily_delta, longest_stagnation, quantiles,
                            ratio, run_values, target, total_deaths)
from evaluate_pass5 import load_round as load_round_schema, minute_in_window, with_unit_denominator
from evaluate_pass6 import (EVENING, WINTER, evaluate as evaluate_pass6, fmt, nights_at_0200, seasons_on, system_day, system_minute, system_night_season,
                            winter_closings, winter_windows, wood_by_season, work_by_season)
from evaluate_pass7 import measures as measures_pass7

SCHEMA = 8
RECEIPTS = ("legacy_pin", "v8_repro", "v9_repro", "v10_repro", "v11_repro", "v12_repro", "v13_repro")
SEVERITY_NAMES = ("mild", "normal", "harsh")
GATHERED_FLOOR = 40.5


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-8 round; every seed must have every day checkpoint."""
    return load_round_schema(path, duration, SCHEMA)


def rules_weights(r: Round) -> tuple[int, int, int] | None:
    """The severity weights of the run's firewood rule (`summary.json` rules block)."""
    firewood = (r.summary.get("rules") or {}).get("firewood") or {}
    weights = firewood.get("winter_severity_permille")
    return tuple(int(w) for w in weights) if weights else None


def winters_table(r: Round) -> list[dict]:
    """Pass 7's Winters table plus the night burn and the day fall of each seed-winter.

    `night_burn_mean`: mean `night_burn_per_hour` over the winter's 02:00 samples
    (booked to their evening, as the dark nights); `fall_mean`: mean
    `winter_fall_today` over the winter's seven 08:00 day checkpoints."""
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
            burns = [float(row["night_burn_per_hour"]) for _, row in evenings]
            falls = [float(seed.days[d - 1]["winter_fall_today"]) for d in days]
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
                "night_burn_mean": sum(burns) / len(burns) if burns else 0.0,
                "fall_mean": sum(falls) / len(falls) if falls else 0.0,
            })
    return table


def r_squared(winters: list[dict]) -> float | None:
    """Share of the dark-night count's variance explained by severity alone:
    1 - sum_s sum_w (n_w - mean_s)^2 / sum_w (n_w - mean)^2; None when every
    winter has the same count."""
    counts = [w["nights_out"] for w in winters]
    if not counts:
        return None
    mean = sum(counts) / len(counts)
    total = sum((n - mean) ** 2 for n in counts)
    if total == 0:
        return None
    within = 0.0
    for level in {w["severity"] for w in winters}:
        group = [w["nights_out"] for w in winters if w["severity"] == level]
        mean_s = sum(group) / len(group)
        within += sum((n - mean_s) ** 2 for n in group)
    return 1 - within / total


def dark_nights_by_severity(winters: list[dict]) -> dict:
    """Histogram of the dark-night count per severity (`-1` = no severity)."""
    out = {}
    for level in sorted({w["severity"] for w in winters}):
        name = SEVERITY_NAMES[level] if 0 <= level < 3 else str(level)
        hist = Counter(w["nights_out"] for w in winters if w["severity"] == level)
        out[name] = {str(k): hist[k] for k in sorted(hist)}
    return out


def x3_clauses(winters: list[dict], weights: tuple[int, int, int] | None) -> dict:
    n = len(winters)
    r2 = r_squared(winters)
    spread = {}
    spread_ok = True
    for level in (1, 2):
        group = [w["nights_out"] for w in winters if w["severity"] == level]
        mode_share = ratio(max(Counter(group).values()), len(group)) if group else None
        spread[SEVERITY_NAMES[level]] = {"n": len(group), "mode_share": mode_share,
                                         "histogram": {str(k): v for k, v in sorted(Counter(group).items())}}
        spread_ok = spread_ok and mode_share is not None and mode_share <= .75
    none = ratio(sum(1 for w in winters if w["nights_out"] == 0), n)
    many = ratio(sum(1 for w in winters if w["nights_out"] >= 2), n)
    firsts = [w["first_night_out"] for w in winters if w["nights_out"] > 0]
    distinct = len(set(firsts))
    seventh = ratio(sum(1 for f in firsts if f == 7), len(firsts))
    shares = [ratio(sum(1 for w in winters if w["severity"] == level), n) for level in range(3)]
    return {
        "a_r2": clause(r2 is not None and r2 <= .75, {"r2": r2, "n": n},
                       "severity-only R2 of the dark-night count <= 0.75 (undefined fails)"),
        "b_spread_within_severity": clause(spread_ok, spread,
                                           "normal and harsh winters each: the most common dark-night count <= 0.75 of them"),
        "c_no_dark_night": clause(n > 0 and .15 <= none <= .60, {"share": none, "n": n},
                                  "share of seed-winters with no dark night in [0.15, 0.60]"),
        "d_two_or_more": clause(n > 0 and .10 <= many <= .50, {"share": many, "n": n},
                                "share of seed-winters with >= 2 dark nights in [0.10, 0.50]"),
        "e_first_night_spread": clause(bool(firsts) and distinct >= 4 and seventh <= .50,
                                       {"distinct_first_nights": distinct, "seventh_share": seventh, "n": len(firsts),
                                        "histogram": {str(k): firsts.count(k) for k in sorted(set(firsts))}},
                                       "first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh"),
        "f_severity_shares": clause(weights is not None and n > 0
                                    and all(abs(s - w / 1000) <= .10 for s, w in zip(shares, weights)),
                                    {"shares": dict(zip(SEVERITY_NAMES, shares)),
                                     "weights": dict(zip(SEVERITY_NAMES, weights)) if weights else None},
                                    "each severity's share of the seed-winters within +-0.10 of its weight"),
    }


def per_seed_values(short: Round, long: Round, cold_window: tuple[int, int]) -> dict[str, dict]:
    """Per-seed values and their bound for every per-seed clause of Y3, Y7, Y8, Y10, Y11 and Y12.

    Keys of `values` are seed labels: `<seed>` for a single round, `10:<seed>` /
    `<N>:<seed>` for a clause over both rounds. `ok(value)` is the clause's
    every-seed bound (Pass 6's thresholds)."""
    for r in (short, long):
        with_unit_denominator(r)
    last = long.duration
    out: dict[str, dict] = {}

    def add(name, values, ok, requirement):
        out[name] = {"values": values, "ok": ok, "requirement": requirement}

    add("Y3_one_settlement",
        {f"{r.duration}:{seed.seed}": sum(1 for d in seed.days if seed.days[d]["settlements"] != 1)
         for r in (short, long) for seed in r.seeds},
        lambda v: v == 0, "day samples with settlements != 1 (every seed: 0)")
    add("Y3_birth_by_day4", {str(s.seed): float(s.days[4]["births"]) for s in short.seeds},
        lambda v: v >= 1, "births by day 4 >= 1 (10-day round)")
    add("Y3_alive_day10", {str(s.seed): float(s.days[10]["alive"]) for s in short.seeds},
        lambda v: v >= 30, "day-10 alive >= 30")
    add("Y3_stagnation", {str(s.seed): float(longest_stagnation(s, last)) for s in long.seeds},
        lambda v: v <= 4, "longest stagnant run <= 4 days")
    add("Y3_old_age_by_day30", {str(s.seed): float(s.days[30]["deaths_old_age"]) for s in long.seeds},
        lambda v: v >= 1, ">= 1 old-age death by day 30")
    add("Y3_replacement",
        {str(s.seed): float((s.days[last]["births"] - s.days[29]["births"])
                            - (total_deaths(s.days[last]) - total_deaths(s.days[29]))) for s in long.seeds},
        lambda v: v >= -3, f"births - deaths over days 30..{last} >= -3")
    add("Y3_alive_band",
        {str(s.seed): [min(float(s.days[d]["alive"]) for d in s.days if d >= 30),
                       max(float(s.days[d]["alive"]) for d in s.days if d >= 30)] for s in long.seeds},
        lambda v: 32 <= v[0] and v[1] <= 44, "alive in [32, 44] at every day sample from day 30 ([min, max])")
    winter, other = {}, {}
    for seed in long.seeds:
        nights = nights_at_0200(seed)
        w = [float(row["fire_lit"]) for _, s, row in nights if s == WINTER]
        o = [float(row["fire_lit"]) for _, s, row in nights if s != WINTER]
        if w:
            winter[str(seed.seed)] = sum(w) / len(w)
        if o:
            other[str(seed.seed)] = sum(o) / len(o)
    add("Y7_winter_lit_every_seed", winter, lambda v: v >= .50, "share of winter nights lit at 02:00 >= 0.50")
    add("Y7_other_lit_every_seed", other, lambda v: v >= .85, "outside winter, 02:00 lit share >= 0.85")
    cold = {}
    for seed in long.seeds:
        rows = [row for row in seed.periodic if row["days_per_season"] > 0
                and minute_in_window(system_minute(row), *cold_window)
                and system_night_season(row, cold_window[1] if cold_window[0] > cold_window[1] else 0) == WINTER]
        if rows:
            cold[str(seed.seed)] = ratio(sum(r["cold_members"] for r in rows), sum(r["settled_agents"] for r in rows))
    add("Y8_winter_cold_every_seed", cold, lambda v: v <= .50, "winter cold members / members in the cold window <= 0.50")
    evening = {}
    for seed in long.seeds:
        values = clock_ratio(seed, EVENING, "idle_adults_near_hearth", "idle_adults")
        if values:
            evening[str(seed.seed)] = quantiles(values)["median"]
    add("Y10_evening_near_hearth_every_seed", evening, lambda v: v >= .35, "per-seed median evening near-hearth share >= 0.35")
    add("Y11_home_distance_every_seed", {str(s.seed): float(s.days[10]["home_distance_mean"]) for s in short.seeds},
        lambda v: v <= 13, "day-10 home distance <= 13")
    for r in (short, long):
        add(f"Y11_social_{r.duration}_every_seed",
            {str(run["seed"]): float(run["social_completed_per_agent_day"]) for run in r.summary["runs"]},
            lambda v: v > 0, "social completions per agent-day > 0")
        add(f"Y12_day{r.duration}",
            {str(s.seed): float(max((row["gather_seekers_critical"] for row in s.periodic), default=0)) for s in r.seeds},
            lambda v: v == 0, "no gatherer with hunger or thirst >= 90 at any 120-tick sample")
    for entry in out.values():
        entry["failing"] = sorted((k for k, v in entry["values"].items() if not entry["ok"](v)), key=seed_key)
    return out


def seed_key(label: str) -> tuple[int, ...]:
    return tuple(int(part) for part in label.split(":"))


def per_seed_clause(name: str, entry: dict, reference: dict | None) -> dict:
    failing = entry["failing"]
    observed = {"failing": {k: entry["values"][k] for k in failing}, "n_failing": len(failing), "n": len(entry["values"])}
    if reference is None:
        return clause(not failing, observed, f"every seed: {entry['requirement']}")
    ref = reference.get(name)
    if ref is None:
        return clause(False, {**observed, "reference": None}, f"comparison mode: reference lacks {name}")
    ref_failing = ref["failing"]
    observed["reference_failing"] = {k: ref["values"][k] for k in ref_failing}
    observed["reference_n_failing"] = len(ref_failing)
    return clause(len(failing) <= len(ref_failing) + 1, observed,
                  f"failing seeds <= reference's + 1 ({entry['requirement']})")


def population_clauses(pass6: dict) -> dict:
    """The median parts of Pass 6's mixed clauses and its pure-median clauses."""
    obs = lambda name, key: pass6[name]["clauses"][key]["observed"]
    wl, ol, wc = obs("Y7", "winter_lit"), obs("Y7", "other_lit"), obs("Y8", "winter_cold")
    ev, home = obs("Y10", "evening_near_hearth"), obs("Y11", "home_distance")
    out = {
        "Y7_winter_lit_median": clause(wl["n"] > 0 and .70 <= wl["median"] <= .97, wl, "per seed share of winter nights lit at 02:00: median in [0.70, 0.97]"),
        "Y7_other_lit_median": clause(ol["n"] > 0 and ol["median"] >= .95, ol, "outside winter, 02:00 lit share: median >= 0.95"),
        "Y8_winter_cold_median": clause(wc["n"] > 0 and wc["median"] <= .25, wc, "winter cold members / members in the cold window: median <= 0.25"),
        "Y8_winter_food": pass6["Y8"]["clauses"]["winter_food"],
        "Y10_night_in_shelter": pass6["Y10"]["clauses"]["night_in_shelter"],
        "Y10_evening_near_hearth_median": clause(ev["median"] >= .55, ev, "evening near-hearth share median >= 0.55"),
        "Y11_home_distance_median": clause(home["median"] <= 8.1, home, "day-10 home distance median <= 8.1"),
    }
    for key, item in pass6["Y11"]["clauses"].items():
        if key.startswith("t6a_"):
            out[f"Y11_{key}"] = item
        elif key.startswith("social_"):
            o = item["observed"]
            out[f"Y11_{key}_median"] = clause(o["median"] >= .40, o, "social completions per agent-day median >= 0.40")
    return out


def evaluate(short: Round, long: Round, receipts: dict[str, bool], v11: dict | None = None,
             weights: tuple[int, int, int] | None = None, cold_window: tuple[int, int] = (1320, 360),
             reference: dict | None = None, baseline_ms: float | None = None, long_baseline_ms: float | None = None,
             x6: bool = False, x7: bool = False, gathered_floor: float = GATHERED_FLOOR) -> tuple[dict, dict]:
    """X1-X8 and Y9 of PASS8_GOAL.md section 9. `reference` is the `per_seed` block of the
    comparison evaluation (game_v12 on the same seeds). Returns (targets, per_seed)."""
    pass6 = evaluate_pass6(short, long, None, {}, None, v11, cold_window)
    per_seed = per_seed_values(short, long, cold_window)
    result = {}
    def twins(r: Round, sizes: tuple[int, ...]) -> dict:
        declared = r.summary["seeds"]["end_exclusive"] - r.summary["seeds"]["start"]
        return clause(r.summary["determinism_checked"] and len(r.seeds) == declared and declared in sizes,
                      {"seeds": len(r.seeds), "range": r.summary["seeds"]},
                      f"every seed of the set ({' or '.join(map(str, sizes))}), determinism checked")
    result["X1"] = target({
        # Tuning sets are 64 x 10 and 32 x 120; held-out and fresh sets 64 x 10 and 64 x 120.
        "twins_10": twins(short, (64,)),
        f"twins_{long.duration}": twins(long, (32, 64)),
        **{name: clause(receipts.get(name, False), receipts.get(name, False), f"{name} verified externally") for name in RECEIPTS},
    })
    clauses = {f"Y2_{k}": c for k, c in pass6["Y2"]["clauses"].items()}
    clauses.update({name: per_seed_clause(name, entry, reference) for name, entry in per_seed.items()})
    clauses["Y3_child_share"] = pass6["Y3"]["clauses"]["child_share"]
    clauses["Y3_elder_share"] = pass6["Y3"]["clauses"]["elder_share"]
    result["X2"] = target(clauses)
    winters = winters_table(long)
    result["X3"] = target(x3_clauses(winters, weights))
    share = quantiles([w["retarget_share"] for w in winters])
    normal = quantiles([w["wood_gathered"] for w in winters if w["severity"] == 1])
    result["X4"] = target({
        "retarget_share": clause(share["n"] > 0 and share["median"] <= .10, share,
                                 "winter wood_retargets / winter wood trips, median over seed-winters <= 0.10"),
        "normal_gathered": clause(normal["n"] > 0 and normal["median"] >= gathered_floor, {**normal, "floor": gathered_floor},
                                  f"median winter wood gathered over normal-severity seed-winters >= {gathered_floor}"),
    })
    clauses = {}
    for name in ("Y4", "Y5", "Y6"):
        clauses.update({f"{name}_{k}": c for k, c in pass6[name]["clauses"].items()})
    clauses.update(population_clauses(pass6))
    result["X5"] = target(clauses)
    result["X6"] = target({"receipt": clause(x6, x6, "every Pass 7 intervention test passes and 8.1's Z6 for v14")})
    result["X7"] = target({"receipt": clause(x7, x7, "schema 2 byte identity, hidden columns match the probe, v0 sample loads and Part 0.4 reproduced it, dataset v1 manifest verified")})
    clauses = {}
    for r, base in ((short, baseline_ms), (long, long_baseline_ms or baseline_ms)):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = 1.2 * base if base else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit, {**timing, "limit": limit},
                                             "median ms/agent-tick <= 1.2 x same-host game_v13, recorder off")
    result["X8"] = target(clauses)
    result["Y9"] = pass6["Y9"]
    result["Y9"]["reported_only"] = True
    stored = {name: {"values": entry["values"], "failing": entry["failing"], "requirement": entry["requirement"]}
              for name, entry in per_seed.items()}
    return result, stored


def winters_by_severity(winters: list[dict]) -> list[dict]:
    rows = []
    for level, name in enumerate(SEVERITY_NAMES):
        group = [w for w in winters if w["severity"] == level]
        rows.append({"severity": name, "n": len(group),
                     "dark_nights": {str(k): v for k, v in sorted(Counter(w["nights_out"] for w in group).items())},
                     **{key: quantiles([float(w[key]) for w in group if w[key] is not None])
                        for key in ("nights_out", "first_night_out", "wood_trips", "wood_gathered", "retarget_share",
                                    "cold_member_hours", "food_store_at_end", "night_burn_mean", "fall_mean")}})
    return rows


def markdown(targets: dict, per_seed: dict, long: Round, cold_end: int, title: str, weights, weights_source: str) -> str:
    lines = [f"# {title}", "", f"Values: median / p10 / p90 / min / max. Y9 is reported only. Severity weights {weights} ({weights_source}).", "",
             "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
    for name, entry in targets.items():
        for key, item in entry["clauses"].items():
            lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {json.dumps(item['observed'], sort_keys=True)} |")
    lines += ["", "## Per-seed clauses", "", "| Clause | Seeds | Failing seeds (value) |", "|---|---:|---|"]
    for name, entry in per_seed.items():
        failing = ", ".join(f"{k} ({entry['values'][k]})" for k in entry["failing"]) or "none"
        lines.append(f"| {name} | {len(entry['values'])} | {failing} |")
    lines += ["", "## Work by season", "", "| Season | Adult work 12:00 | Idle 12:00 | Adult work 15:00 | Idle 15:00 |", "|---|---|---|---|---|"]
    for row in work_by_season(long):
        lines.append(f"| {row['season']} | {fmt(row['work_1200'])} | {fmt(row['idle_1200'])} | {fmt(row['work_1500'])} | {fmt(row['idle_1500'])} |")
    lines += ["", "## Wood by season", "",
              "| Season | Wood stock | Wood target | Gathered/day | Wood trips/day | Urgent trips/day | Lit at 02:00 | Cold member-hours/day | Food store | Forage trips/day |",
              "|---|---|---|---|---|---|---:|---|---|---|"]
    for row in wood_by_season(long, cold_end):
        lines.append(f"| {row['season']} | {fmt(row['wood_stock'])} | {fmt(row['wood_target'])} | {fmt(row['wood_gathered'])} | {fmt(row['wood_trips'])} | "
                     f"{fmt(row['urgent_trips'])} | {row['lit_0200']:.3f} | {fmt(row['cold_member_hours'])} | {fmt(row['food_store'])} | {fmt(row['forage_trips'])} |")
    winters = winters_table(long)
    lines += ["", "## Winters", "",
              "One row per complete seed-winter (Pass 7's columns). Night burn = mean `night_burn_per_hour` at the winter's 02:00 samples; fall = mean `winter_fall_today` at its seven 08:00 checkpoints.", "",
              "| Seed | Year | Severity | Wood at start | Nights out | First night out | Trip-days | Wood trips | Urgent trips | Gathered | Re-targets | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |",
              "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for w in winters:
        food = "–" if w["food_store_at_end"] is None else f"{w['food_store_at_end']:.0f}"
        lines.append(f"| {w['seed']} | {w['year']} | {w['severity']} | {w['wood_in_hand_at_start']:.0f} | {w['nights_out']} | {w['first_night_out']} | "
                     f"{w['trip_days']} | {w['wood_trips']:.0f} | {w['urgent_trips']:.0f} | {w['wood_gathered']:.0f} | {w['wood_retargets']:.0f} | "
                     f"{w['retarget_share']:.3f} | {w['cold_member_hours']:.0f} | {food} | {w['night_burn_mean']:.2f} | {w['fall_mean']:.2f} |")
    if winters:
        r2 = r_squared(winters)
        lines += ["", f"Severity-only R2 of the dark-night count: {'undefined' if r2 is None else f'{r2:.3f}'}.", "",
                  "## Winters by severity", "",
                  "| Severity | Seed-winters | Dark nights (count: winters) | Nights out | First night out | Wood trips | Gathered | Re-target share | Cold member-hours | Food store at end | Night burn | Fall |",
                  "|---|---:|---|---|---|---|---|---|---|---|---|---|"]
        for row in winters_by_severity(winters):
            lines.append(f"| {row['severity']} | {row['n']} | {json.dumps(row['dark_nights'])} | {fmt(row['nights_out'])} | {fmt(row['first_night_out'])} | "
                         f"{fmt(row['wood_trips'])} | {fmt(row['wood_gathered'])} | {fmt(row['retarget_share'])} | {fmt(row['cold_member_hours'])} | "
                         f"{fmt(row['food_store_at_end'])} | {fmt(row['night_burn_mean'])} | {fmt(row['fall_mean'])} |")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--compare-to", type=Path, help="x_targets.json of this script's game_v12 evaluation on the same seeds")
    parser.add_argument("--v11-targets", type=Path, help="y_targets.json of Pass 6's game_v11 evaluation (p6-1_v11_targets)")
    parser.add_argument("--severity-permille", help="override the run's rules block (recorded)")
    parser.add_argument("--baseline-summary", type=Path, help="same-host game_v13 summary.json (10 days), recorder off")
    parser.add_argument("--baseline-summary-long", type=Path, help="same-host game_v13 summary of the long duration")
    parser.add_argument("--cold-window", default="1320,360")
    parser.add_argument("--gathered-floor", type=float, default=GATHERED_FLOOR)
    parser.add_argument("--title", default="Pass 8 X targets")
    for name in (*RECEIPTS, "x6", "x7"):
        parser.add_argument(f"--{name.replace('_', '-')}", choices=("PASS", "FAIL"), default="FAIL")
    args = parser.parse_args(argv)
    try:
        def baseline_of(path: Path | None) -> float | None:
            if path is None:
                return None
            source = json.loads(path.read_text())
            if source["scene"] != "game_v13":
                raise EvidenceError("timing baseline must be a game_v13 summary")
            return float(source["timing"]["ms_per_agent_tick"]["median"])
        start, end = (int(v) for v in args.cold_window.split(","))
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        weights_source = "summary.json rules block"
        weights = rules_weights(long)
        if args.severity_permille:
            weights = tuple(int(v) for v in args.severity_permille.split(","))
            weights_source = "--severity-permille override"
            if len(weights) != 3 or sum(weights) != 1000:
                raise ValueError("severity weights must be three thousandths summing to 1000")
        reference = None
        if args.compare_to:
            ref = json.loads(args.compare_to.read_text())
            if "per_seed" not in ref:
                raise EvidenceError("--compare-to must be an evaluate_pass8 output (per_seed values)")
            mine = {"round10": short.summary["seeds"], "round120": long.summary["seeds"]}
            if ref.get("seeds") != mine:
                raise EvidenceError(f"--compare-to covers seeds {ref.get('seeds')}, this run {mine}")
            reference = ref["per_seed"]
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        receipts = {name: getattr(args, name) == "PASS" for name in RECEIPTS}
        targets, per_seed = evaluate(short, long, receipts, v11, weights, (start, end), reference,
                                     baseline_of(args.baseline_summary), baseline_of(args.baseline_summary_long),
                                     args.x6 == "PASS", args.x7 == "PASS", args.gathered_floor)
        own = measures_pass7(long, (start, end))
        winters = winters_table(long)
        text = markdown(targets, per_seed, long, end, args.title, weights, weights_source)
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for name, t in targets.items() if name != "Y9")
    report = {"schema_version": 1, "passed": passed, "targets": targets, "per_seed": per_seed, "measures": own,
              "winters": winters, "winters_by_severity": winters_by_severity(winters),
              "dark_nights_by_severity": dark_nights_by_severity(winters), "r2": r_squared(winters),
              "work_by_season": work_by_season(long), "wood_by_season": wood_by_season(long, end),
              "severity_permille": list(weights) if weights else None, "severity_source": weights_source,
              "rules": long.summary.get("rules"), "scene": long.summary.get("scene"),
              "seeds": {"round10": short.summary["seeds"], "round120": long.summary["seeds"]},
              "cold_window": [start, end],
              "inputs": {"round10": str(args.round10), "round120": str(args.round120),
                         "compare_to": str(args.compare_to) if args.compare_to else None,
                         "v11_targets": str(args.v11_targets) if args.v11_targets else None}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "x_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (args.out / "x_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
