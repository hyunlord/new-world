#!/usr/bin/env python3
"""Evaluate Pass 10's H targets on schema-10 probe runs (PASS10_GOAL.md 6.3, 10). Stdlib only.

usage: evaluate_pass10.py --round10 DIR --round120 DIR --out DIR
                          [--v15-round10 DIR --v15-round120 DIR] [--v11-targets P6_V11_Y_TARGETS.json]
                          [--baseline-summary V15_10DAY.json] [--baseline-summary-long V15_120DAY.json]
                          [--variety DIR ...] [--climate-permille 333,334,333]
                          [--legacy-pin PASS|FAIL] [--v8-v15-repro PASS|FAIL] [--title TITLE]

DIR holds a schema-10 probe run (`summary.json` plus raw JSONL) of one v16
scene (`game_v16_neutral` or `game_v16`). Every per-seed quantity (starting
agents, severity weights, the hidden world parameters) comes from the per-seed
fields of `summary.json` (`runs[].seed_info`), never from its `rules` block,
which describes the first seed only.

- H1: determinism twins of both rounds (every seed of the set) plus the
  receipts `--legacy-pin` and `--v8-v15-repro`.
- H2 (no collapse, the 120-day round): one settlement at every day
  checkpoint; alive >= max(8, 0.4 x starting agents) at every daily sample
  (the 08:00 checkpoints and the 06:00 clock samples); alive on day 120 >=
  0.6 x alive on day 21. Every seed must pass.
- H3 (deaths exist and are bounded, the 120-day round; judged for game_v16):
  per seed-winter [day D 18:00, day D+7 07:00): the share with >= 1
  exposure death in [0.15, 0.60]; median exposure deaths <= 2; every
  seed-winter's deaths <= 0.35 x members at its start; mean deaths per
  seed-winter strictly increasing mild < normal < harsh; starvation +
  dehydration 0 in every seed at day 10, and at day 120 median <= 1, max <= 6.
- H4: every exposure death (system tick from the death census) lies on a
  FireSystem run (a multiple of 60) inside the cold window of a winter night
  (no probe run has cold snaps).
- H5 (village rhythm kept; with `--v15-round*`, game_v15 on the same seeds):
  `evaluate_pass8.evaluate` on both scenes. Per-seed clauses of Pass 6 Y10,
  Y11, Y12 pass when this run's failing seeds number at most v15's + 1;
  population clauses of Y4, Y5, Y6, Y9, Y10, Y11 and Pass 8 X3 (b)-(e), X4
  pass when they pass, or when v15 fails them too and v16 is no worse than
  v15 on the statistic the clause bounds, in its better direction
  (`ESCAPE_STAT`: e.g. Y4 `every_seed` compares minima, Y5 `autumn_wood`
  autumn - 1.5 x summer; X3 (c)-(e) are two-sided or compound and never
  escape; review rounds 1 L1 and 2 M1). Y2, Y3, Y7, Y8 and
  X3 (a), (f) are reported only (Y3's population clauses are replaced by H2;
  the reported Y3 births-minus-deaths margin uses Pass 4's `total_deaths`,
  which excludes exposure deaths).
- H6 (`--variety`: 10-day game_v16 runs, e.g. tuning + held-out): every
  parameter's mean within +-10 % of its range centre, each climate's share
  within +-0.10 of its weight; deadwood shortfalls and berry-repair swaps
  counted.
- H10: median ms per agent-tick <= 1.25 x the same-host game_v15 summary of
  the same duration.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import evaluate_pass8 as p8
from evaluate_pass4 import EvidenceError, Round, clause, quantiles, ratio, run_values, target
from evaluate_pass5 import load_round as load_round_schema

SCHEMA = 10
BASELINE_SCENE = "game_v15"
TIMING_FACTOR = 1.25
SEVERITY_NAMES = ("mild", "normal", "harsh")
# 7.1's ranges (Part 3 may move an end by one step; pass the frozen ranges with --ranges).
RANGES = {"starting_agents": (18, 30), "berry_bushes": (9, 15), "berry_capacity": (3, 5),
          "deadwood_spots": (14, 26), "deadwood_capacity": (5, 7), "forest_permille": (220, 380),
          "food_regen_permille": (850, 1150)}
CLIMATE_PERMILLE = (333, 334, 333)
H5_PER_SEED = ("Y10_evening_near_hearth_every_seed", "Y11_home_distance_every_seed", "Y11_social_10_every_seed",
               "Y11_social_120_every_seed", "Y12_day10", "Y12_day120")
H5_POPULATION_PREFIXES = ("Y4_", "Y5_", "Y6_", "Y10_night_in_shelter", "Y10_evening_near_hearth_median",
                          "Y11_home_distance_median", "Y11_t6a_", "Y11_social_")
H5_X3 = ("b_spread_within_severity", "c_no_dark_night", "d_two_or_more", "e_first_night_spread")


def load_round(path: Path, duration: int) -> Round:
    """Load a schema-10 round; every seed must have every day checkpoint."""
    return load_round_schema(path, duration, SCHEMA)


def runs(r: Round) -> list[dict]:
    return sorted(r.summary["runs"], key=lambda run: run["seed"])


def seed_weights(run: dict) -> tuple[int, int, int] | None:
    weights = (run.get("seed_info") or {}).get("winter_severity_permille")
    return tuple(int(w) for w in weights) if weights else None


def mean_weights(r: Round) -> tuple[int, int, int] | None:
    """Mean of the per-seed severity weights (the expected severity shares over the set)."""
    rows = [w for w in (seed_weights(run) for run in runs(r)) if w]
    if not rows:
        return None
    return tuple(round(sum(w[i] for w in rows) / len(rows)) for i in range(3))


def clock_offset(r: Round) -> int:
    """Display-clock offset of the run, from any raw row (minute_of_day - tick mod 1440)."""
    for seed in r.seeds:
        for row in seed.periodic:
            return (int(row["minute_of_day"]) - int(row["tick"])) % 1440
    raise EvidenceError("round has no periodic rows")


def h1(short: Round, long: Round, receipts: dict[str, bool]) -> dict:
    def twins(r: Round, sizes: tuple[int, ...]) -> dict:
        declared = r.summary["seeds"]["end_exclusive"] - r.summary["seeds"]["start"]
        return clause(r.summary["determinism_checked"] and len(r.seeds) == declared and declared in sizes,
                      {"seeds": len(r.seeds), "range": r.summary["seeds"]},
                      f"every seed of the set ({' or '.join(map(str, sizes))}), determinism checked")
    return target({"twins_10": twins(short, (64,)), f"twins_{long.duration}": twins(long, (32, 64)),
                   **{name: clause(ok, ok, f"{name} verified externally") for name, ok in receipts.items()}})


def h2(long: Round) -> tuple[dict, dict]:
    """No collapse over the long round; returns (target, per-seed rows)."""
    per_seed, bad_settle, bad_floor, bad_keep = {}, [], [], []
    for run in runs(long):
        seed = run["seed"]
        start = int(run["seed_info"]["starting_agents"])
        floor = max(8.0, 0.4 * start)
        alive = [int(a) for a in run["alive_daily"]]
        dawn = [int(row[3]) for row in run["dawn_health"]]
        settlements = [int(s) for s in run["settlements_daily"]]
        lowest = min(alive + dawn)
        day21 = alive[20] if len(alive) > 20 else None
        last = alive[-1]
        keep = None if day21 is None else last / day21 if day21 else 0.0
        per_seed[str(seed)] = {"starting_agents": start, "floor": floor, "min_alive": lowest, "alive_day21": day21,
                               "alive_last": last, "retention": keep,
                               "days_without_one_settlement": sum(1 for s in settlements if s != 1)}
        if any(s != 1 for s in settlements):
            bad_settle.append(seed)
        if lowest < floor:
            bad_floor.append(seed)
        if keep is None or keep < 0.6:
            bad_keep.append(seed)
    n = len(per_seed)
    return target({
        "one_settlement": clause(n > 0 and not bad_settle, {"failing": bad_settle, "n": n},
                                 "one settlement at every day checkpoint, every seed"),
        "alive_floor": clause(n > 0 and not bad_floor,
                              {"failing": {str(s): per_seed[str(s)]["min_alive"] for s in bad_floor}, "n": n,
                               "min_alive_over_floor": quantiles([per_seed[k]["min_alive"] - per_seed[k]["floor"] for k in per_seed])},
                              "alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed"),
        "retention": clause(n > 0 and not bad_keep,
                            {"failing": {str(s): per_seed[str(s)]["retention"] for s in bad_keep}, "n": n,
                             "retention": quantiles([v["retention"] for v in per_seed.values() if v["retention"] is not None])},
                            f"alive on day {long.duration} >= 0.6 x alive on day 21, every seed"),
    }), per_seed


def seed_winters(long: Round) -> list[dict]:
    out = []
    for run in runs(long):
        for w in run["seed_winters"]:
            out.append({"seed": run["seed"], **w})
    return out


def h3(short: Round, long: Round) -> tuple[dict, dict]:
    """Exposure deaths per seed-winter and starvation/dehydration bounds; returns (target, tables)."""
    winters = seed_winters(long)
    n = len(winters)
    deaths = [int(w["exposure_deaths"]) for w in winters]
    share = ratio(sum(1 for d in deaths if d >= 1), n) if n else None
    worst = [ratio(int(w["exposure_deaths"]), int(w["members_at_start"] or 0)) if w["members_at_start"] else
             (0.0 if int(w["exposure_deaths"]) == 0 else float("inf")) for w in winters]
    over = [{"seed": w["seed"], "first_day": w["first_day"], "deaths": w["exposure_deaths"], "members": w["members_at_start"]}
            for w, x in zip(winters, worst) if x > .35]
    by_sev = {}
    for level, name in enumerate(SEVERITY_NAMES):
        group = [int(w["exposure_deaths"]) for w in winters if int(w["severity"]) == level]
        by_sev[name] = {"n": len(group), "mean": sum(group) / len(group) if group else None,
                        "share_with_death": ratio(sum(1 for d in group if d >= 1), len(group)) if group else None,
                        "histogram": {str(k): v for k, v in sorted(Counter(group).items())},
                        "dark_nights_mean": (sum(int(w["dark_nights"]) for w in winters if int(w["severity"]) == level) / len(group)) if group else None}
    means = [by_sev[name]["mean"] for name in SEVERITY_NAMES]
    increasing = all(m is not None for m in means) and means[0] < means[1] < means[2]
    starve10 = {str(s.seed): float(s.days[10]["deaths_starvation"]) + float(s.days[10]["deaths_dehydration"]) for s in short.seeds}
    last = long.duration
    starve_long = {str(s.seed): float(s.days[last]["deaths_starvation"]) + float(s.days[last]["deaths_dehydration"]) for s in long.seeds}
    q = quantiles(list(starve_long.values()))
    deaths_q = quantiles([float(d) for d in deaths])
    by_dark = {}
    for w in winters:
        by_dark.setdefault(str(int(w["dark_nights"])), []).append(int(w["exposure_deaths"]))
    tables = {"by_severity": by_sev, "by_dark_nights": {k: {"n": len(v), "mean": sum(v) / len(v), "max": max(v)}
                                                         for k, v in sorted(by_dark.items(), key=lambda kv: int(kv[0]))},
              "deaths_histogram": {str(k): v for k, v in sorted(Counter(deaths).items())},
              "starvation_dehydration_120": starve_long, "starvation_dehydration_10": starve10}
    return target({
        "share_with_death": clause(share is not None and .15 <= share <= .60, {"share": share, "n": n},
                                   "share of seed-winters with >= 1 exposure death in [0.15, 0.60]"),
        "median_deaths": clause(n > 0 and deaths_q["median"] <= 2, deaths_q, "median exposure deaths per seed-winter <= 2"),
        "max_share_of_members": clause(n > 0 and not over, {"over": over, "max_share": max(worst) if worst else None},
                                       "every seed-winter: exposure deaths <= 0.35 x members at the winter's start"),
        "increasing_with_severity": clause(increasing, {name: by_sev[name]["mean"] for name in SEVERITY_NAMES},
                                           "mean exposure deaths per seed-winter strictly increasing mild < normal < harsh"),
        "starvation_10": clause(bool(starve10) and max(starve10.values()) == 0,
                                {"failing": {k: v for k, v in starve10.items() if v}},
                                "starvation + dehydration 0 in every seed at day 10"),
        "starvation_long": clause(q["n"] > 0 and q["median"] <= 1 and q["max"] <= 6, q,
                                  f"starvation + dehydration by day {last}: median <= 1 and max <= 6"),
    }), tables


def cold_night_tick(tick: int, offset: int, days_per_season: int, cold=(1320, 360)) -> tuple[bool, str]:
    """Whether a system tick is a FireSystem run inside the cold window of a winter night."""
    if tick % 60:
        return False, "not a FireSystem run"
    minute = (tick + offset) % 1440
    start, end = cold
    if not (minute >= start or minute < end):
        return False, f"minute {minute} outside the cold window"
    evening = (tick + offset) // 1440 - (1 if minute < end else 0)
    if (evening % (4 * days_per_season)) // days_per_season != 3:
        return False, f"evening day index {evening} is not a winter day"
    return True, ""


def h4(rounds: list[Round]) -> dict:
    total, bad = 0, []
    for r in rounds:
        offset = clock_offset(r)
        dps = next((int(row["days_per_season"]) for s in r.seeds for row in s.periodic), 7)
        for run in runs(r):
            for tick in run["exposure_death_ticks"]:
                total += 1
                ok, why = cold_night_tick(int(tick), offset, dps)
                if not ok:
                    bad.append({"round": r.duration, "seed": run["seed"], "tick": tick, "why": why})
    return target({"cold_window": clause(not bad, {"deaths": total, "outside": bad},
                                         "every exposure death on a FireSystem run in the cold window of a winter night "
                                         "(plus harness_exposure_only_from_cold, a test receipt)")})


def h5(short: Round, long: Round, v15_short: Round | None, v15_long: Round | None, v11: dict | None) -> tuple[dict, dict]:
    """Village rhythm against game_v15 on the same seeds; returns (target, reported-only block)."""
    weights = mean_weights(long)
    targets, per_seed = p8.evaluate(short, long, {}, v11, weights)
    ref_targets, ref_per_seed = (p8.evaluate(v15_short, v15_long, {}, v11, mean_weights(v15_long))
                                 if v15_short and v15_long else (None, None))
    if ref_targets is not None:
        mine = {"round10": short.summary["seeds"], "round120": long.summary["seeds"]}
        theirs = {"round10": v15_short.summary["seeds"], "round120": v15_long.summary["seeds"]}
        if mine != theirs:
            raise EvidenceError(f"v15 rounds cover {theirs}, this run {mine}")
    entries = p8.per_seed_values(short, long, (1320, 360))
    clauses = {}
    for name in H5_PER_SEED:
        clauses[name] = p8.per_seed_clause(name, entries[name], ref_per_seed)

    population = population_clause

    def ref_clause(group: str, key: str) -> dict | None:
        return ref_targets[group]["clauses"].get(key) if ref_targets else None

    for key, item in targets["X5"]["clauses"].items():
        if key.startswith(H5_POPULATION_PREFIXES):
            clauses[key] = population(key, item, ref_clause("X5", key))
    for key, item in targets["Y9"]["clauses"].items():
        clauses[f"Y9_{key}"] = population(f"Y9_{key}", item, ref_clause("Y9", key))
    for key in H5_X3:
        clauses[f"X3_{key}"] = population(f"X3_{key}", targets["X3"]["clauses"][key], ref_clause("X3", key))
    for key, item in targets["X4"]["clauses"].items():
        clauses[f"X4_{key}"] = population(f"X4_{key}", item, ref_clause("X4", key))
    reported = {"X2": targets["X2"], "X3_a_r2": targets["X3"]["clauses"]["a_r2"],
                "X3_f_severity_shares": targets["X3"]["clauses"]["f_severity_shares"],
                "Y7_Y8": {k: v for k, v in targets["X5"]["clauses"].items() if k.startswith(("Y7_", "Y8_"))},
                "v15": ({"X2": ref_targets["X2"]["passed"], "X3": ref_targets["X3"]["passed"], "X4": ref_targets["X4"]["passed"],
                         "X5": ref_targets["X5"]["passed"], "Y9": ref_targets["Y9"]["passed"]} if ref_targets else None),
                "per_seed": per_seed, "v15_per_seed": ref_per_seed,
                "winters": p8.winters_table(long), "r2": p8.r_squared(p8.winters_table(long)),
                "work_by_season": p8.work_by_season(long), "wood_by_season": p8.wood_by_season(long, 360)}
    return target(clauses), reported


# H5's "no worse than v15" test (review round 1, L1; round 2, M1): for every population
# clause, the statistic its bound is on and the better direction (+1 larger is better, -1
# smaller is better). None = the clause is two-sided or compound and never escapes. A
# population clause missing here is an error, so a new clause cannot inherit a guess.
ESCAPE_STAT = {
    "Y4_store_share": ("median", 1), "Y4_every_seed": ("min", 1), "Y4_winter": ("median", 1),
    "Y5_autumn_wood": ("autumn_minus_1.5_summer", 1), "Y5_autumn_work": ("median", 1),
    "Y6_trip_days": ("median", 1), "Y6_trips": ("median", 1),
    "Y9_work_1200": ("median", 1), "Y9_work_1500": ("median", 1), "Y9_idle_1200": ("median", -1),
    "Y10_night_in_shelter": ("median", 1), "Y10_evening_near_hearth_median": ("median", 1),
    "Y11_home_distance_median": ("median", -1), "Y11_t6a_10": ("median", -1), "Y11_t6a_120": ("median", -1),
    "Y11_social_10_median": ("median", 1), "Y11_social_120_median": ("median", 1),
    "X3_b_spread_within_severity": ("max_mode_share", -1),
    "X3_c_no_dark_night": None, "X3_d_two_or_more": None, "X3_e_first_night_spread": None,
    "X4_retarget_share": ("median", -1), "X4_normal_gathered": ("median", 1),
}


def escape_statistic(name: str, observed: dict) -> float | None:
    """The statistic `ESCAPE_STAT[name]` reads from a clause's observed block (None when absent)."""
    spec = ESCAPE_STAT[name]
    if spec is None:
        return None
    stat, _ = spec
    try:
        if stat in ("median", "min"):
            return float(observed[stat])
        if stat == "autumn_minus_1.5_summer":
            return float(observed["autumn"]["median"]) - 1.5 * float(observed["summer"]["median"])
        if stat == "max_mode_share":
            shares = [float(v["mode_share"]) for v in observed.values() if v.get("mode_share") is not None]
            return max(shares) if shares else None
    except (KeyError, TypeError, ValueError):
        return None
    raise EvidenceError(f"unknown escape statistic {stat}")


def no_worse(name: str, mine: dict, theirs: dict) -> bool:
    """Whether v16's failing clause `name` is at least as good as v15's failing clause."""
    if name not in ESCAPE_STAT:
        raise EvidenceError(f"H5 population clause {name} has no escape rule")
    spec = ESCAPE_STAT[name]
    a, b = escape_statistic(name, mine["observed"]), escape_statistic(name, theirs["observed"])
    if spec is None or a is None or b is None:
        return False
    return spec[1] * (a - b) >= 0


def population_clause(name: str, mine: dict, theirs: dict | None) -> dict:
    """H5's verdict on population clause `name`: it passes, or v15 fails it too and v16 is no worse."""
    if theirs is None:
        return clause(mine["passed"], {"v16": mine["observed"]}, f"{mine['requirement']} (no v15 reference)")
    escape = not mine["passed"] and not theirs["passed"] and no_worse(name, mine, theirs)
    return clause(mine["passed"] or escape,
                  {"v16": mine["observed"], "v16_passed": mine["passed"], "v15_passed": theirs["passed"],
                   "v15": theirs["observed"], "escaped_as_no_worse_than_v15": escape},
                  f"no new failure vs v15 (v16 no worse than v15 when both fail): {mine['requirement']}")


def h6(rounds: list[Round], ranges: dict, climate: tuple[int, int, int]) -> tuple[dict, dict]:
    rows = [run["seed_info"] for r in rounds for run in runs(r)]
    variety = [row["world_variety"] for row in rows if row.get("world_variety")]
    n = len(variety)
    params, bad = {}, []
    for name, (lo, hi) in ranges.items():
        values = [float(v[name]) for v in variety]
        centre = (lo + hi) / 2
        mean = sum(values) / n if n else None
        ok = mean is not None and abs(mean - centre) <= .10 * centre
        seen = sorted(set(int(v) for v in values))
        params[name] = {"range": [lo, hi], "centre": centre, "mean": mean, "min": min(values) if values else None,
                        "max": max(values) if values else None, "distinct": len(seen), "within_10pct": ok}
        if not ok:
            bad.append(name)
    shares = [ratio(sum(1 for v in variety if int(v["climate"]) == k), n) if n else None for k in range(3)]
    climate_ok = n > 0 and all(abs(s - w / 1000) <= .10 for s, w in zip(shares, climate))
    shortfall = [{"seed": r_run["seed"], "requested": r_run["seed_info"]["deadwood_spots_requested"],
                  "placed": r_run["seed_info"]["deadwood_spots_placed"]}
                 for r in rounds for r_run in runs(r)
                 if r_run["seed_info"]["deadwood_spots_placed"] < r_run["seed_info"]["deadwood_spots_requested"]]
    swaps = [{"seed": r_run["seed"], "swaps": r_run["seed_info"]["berry_repair_swaps"]}
             for r in rounds for r_run in runs(r) if r_run["seed_info"]["berry_repair_swaps"] > 0]
    detail = {"n": n, "parameters": params, "climate_shares": dict(zip(SEVERITY_NAMES, shares)),
              "climate_permille": list(climate), "deadwood_shortfalls": shortfall,
              "berry_repair_swaps": swaps}
    return target({
        "parameter_means": clause(n > 0 and not bad, {"outside": bad, "n": n},
                                  "every parameter's mean within +-10 % of its range centre"),
        "climate_shares": clause(climate_ok, {"shares": shares, "weights": list(climate), "n": n},
                                 "each climate's share within +-0.10 of its CLIMATE_PERMILLE weight"),
        "counted": clause(n > 0, {"deadwood_shortfall_seeds": len(shortfall), "berry_swap_seeds": len(swaps)},
                          "deadwood shortfalls and berry-repair swaps counted (explained in the report)"),
    }), detail


def baseline_ms(path: Path | None) -> float | None:
    if path is None:
        return None
    source = json.loads(path.read_text())
    if source["scene"] != BASELINE_SCENE:
        raise EvidenceError(f"timing baseline must be a {BASELINE_SCENE} summary")
    return float(source["timing"]["ms_per_agent_tick"]["median"])


def h10(short: Round, long: Round, base10: float | None, base_long: float | None) -> dict:
    clauses = {}
    for r, base in ((short, base10), (long, base_long or base10)):
        timing = quantiles(run_values(r, "ms_per_agent_tick"))
        limit = TIMING_FACTOR * base if base else None
        clauses[f"day{r.duration}"] = clause(limit is not None and timing["median"] <= limit,
                                             {**timing, "limit": limit, "ratio": timing["median"] / base if base else None},
                                             f"median ms/agent-tick <= {TIMING_FACTOR} x same-host {BASELINE_SCENE}")
    return target(clauses)


def markdown(title: str, scene: str, targets: dict, h3_tables: dict, h6_detail: dict | None) -> str:
    lines = [f"# {title}", "", f"Scene `{scene}`. Values: median / p10 / p90 / min / max where quantiles are shown.", "",
             "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
    for name, entry in targets.items():
        for key, item in entry["clauses"].items():
            observed = json.dumps(item["observed"], sort_keys=True)
            if len(observed) > 600:
                observed = observed[:600] + "…"
            lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {observed} |")
    lines += ["", "## Exposure deaths per seed-winter, by severity", "",
              "| Severity | Seed-winters | Mean deaths | Share with a death | Dark nights (mean) | Deaths (count: winters) |",
              "|---|---:|---:|---:|---:|---|"]
    for name, row in h3_tables["by_severity"].items():
        mean = "–" if row["mean"] is None else f"{row['mean']:.3f}"
        share = "–" if row["share_with_death"] is None else f"{row['share_with_death']:.3f}"
        dark = "–" if row["dark_nights_mean"] is None else f"{row['dark_nights_mean']:.2f}"
        lines.append(f"| {name} | {row['n']} | {mean} | {share} | {dark} | {json.dumps(row['histogram'])} |")
    lines += ["", "| Dark nights in the winter | Seed-winters | Mean deaths | Max deaths |", "|---:|---:|---:|---:|"]
    for k, row in h3_tables["by_dark_nights"].items():
        lines.append(f"| {k} | {row['n']} | {row['mean']:.3f} | {row['max']} |")
    if h6_detail:
        lines += ["", "## World variety (H6)", "", "| Parameter | Range | Centre | Mean | Min | Max | Distinct |", "|---|---|---:|---:|---:|---:|---:|"]
        for name, p in h6_detail["parameters"].items():
            lines.append(f"| {name} | {p['range']} | {p['centre']} | {p['mean']:.3f} | {p['min']:.0f} | {p['max']:.0f} | {p['distinct']} |")
        lines += ["", f"Climate shares {json.dumps(h6_detail['climate_shares'])} (weights {h6_detail['climate_permille']}); "
                  f"deadwood shortfalls in {len(h6_detail['deadwood_shortfalls'])} seeds; berry-repair swaps in "
                  f"{len(h6_detail['berry_repair_swaps'])} seeds."]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--long-days", type=int, default=120)
    parser.add_argument("--v15-round10", type=Path)
    parser.add_argument("--v15-round120", type=Path)
    parser.add_argument("--v11-targets", type=Path, help="y_targets.json of Pass 6's game_v11 evaluation (p6-1_v11_targets)")
    parser.add_argument("--baseline-summary", type=Path, help="same-host game_v15 summary.json (10 days)")
    parser.add_argument("--baseline-summary-long", type=Path, help="same-host game_v15 summary of the long duration")
    parser.add_argument("--variety", type=Path, nargs="*", default=[], help="10-day game_v16 rounds for H6")
    parser.add_argument("--climate-permille", default=",".join(map(str, CLIMATE_PERMILLE)))
    parser.add_argument("--ranges", help="JSON object overriding 7.1's ranges (frozen Part 3 values)")
    parser.add_argument("--legacy-pin", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--v8-v15-repro", choices=("PASS", "FAIL"), default="FAIL")
    parser.add_argument("--title", default="Pass 10 H targets")
    args = parser.parse_args(argv)
    try:
        short, long = load_round(args.round10, 10), load_round(args.round120, args.long_days)
        scene = long.summary.get("scene")
        if short.summary.get("scene") != scene:
            raise EvidenceError("both rounds must be the same scene")
        v15 = (load_round(args.v15_round10, 10), load_round(args.v15_round120, args.long_days)) if args.v15_round10 else (None, None)
        if v15[0] is not None and (v15[0].summary.get("scene"), v15[1].summary.get("scene")) != (BASELINE_SCENE, BASELINE_SCENE):
            raise EvidenceError(f"--v15-round* must be {BASELINE_SCENE} runs")
        v11 = json.loads(args.v11_targets.read_text())["measures"] if args.v11_targets else None
        climate = tuple(int(v) for v in args.climate_permille.split(","))
        ranges = {k: tuple(v) for k, v in json.loads(args.ranges).items()} if args.ranges else RANGES
        targets = {"H1": h1(short, long, {"legacy_pin": args.legacy_pin == "PASS", "v8_v15_repro": args.v8_v15_repro == "PASS"})}
        targets["H2"], h2_seeds = h2(long)
        targets["H3"], h3_tables = h3(short, long)
        targets["H4"] = h4([short, long])
        targets["H5"], h5_reported = h5(short, long, v15[0], v15[1], v11)
        h6_detail = None
        if args.variety:
            targets["H6"], h6_detail = h6([load_round(path, 10) for path in args.variety], ranges, climate)
        if args.baseline_summary:
            targets["H10"] = h10(short, long, baseline_ms(args.baseline_summary), baseline_ms(args.baseline_summary_long))
        text = markdown(args.title, scene, targets, h3_tables, h6_detail)
    except (EvidenceError, OSError, KeyError, ValueError, json.JSONDecodeError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    passed = all(t["passed"] for t in targets.values())
    report = {"schema_version": 1, "probe_schema": SCHEMA, "scene": scene, "passed": passed, "targets": targets,
              "h2_per_seed": h2_seeds, "h3": h3_tables, "h5_reported": h5_reported, "h6": h6_detail,
              "seed_winters": seed_winters(long),
              "seeds": {"round10": short.summary["seeds"], "round120": long.summary["seeds"]},
              "inputs": {k: (str(v) if isinstance(v, Path) else [str(p) for p in v] if isinstance(v, list) else v)
                         for k, v in vars(args).items() if k != "out"}}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "h_targets.json").write_text(json.dumps(report, indent=2, sort_keys=True, default=str) + "\n")
    (args.out / "h_targets.md").write_text(text)
    print("PASS" if passed else "FAIL")
    for name, entry in targets.items():
        print(f"{name}: {'PASS' if entry['passed'] else 'FAIL'} " + ", ".join(k for k, c in entry["clauses"].items() if not c["passed"]))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
