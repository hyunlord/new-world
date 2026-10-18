#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_pass20.py --candidate V25_DIR --candidate-chronicle V25_CHRON
#     --baseline V24_DIR --baseline-chronicle V24_CHRON --out-md OUT.md --out-json OUT.json
"""Pass 20 T1-T4 and T7: game_v25 vs game_v24 (METRICS.md `## Pass 20`).

Both cohorts are complete `worldsim_probe --agent-trace --chronicle-out`
runs of seeds 19400-19431 for 120 days. A *multi-village world* is one with
>= 2 settlements alive at once in game_v25 (`settlements_alive_daily`). A
village's *region* is its fission site's wild region (the site is a region
centre), or "camp" for a settlement formed without a fission. Daughter
villages are the settlements of `DaughterFounded` entries. T4's health rules
(Pass 17 R2) and F1/F5 reuse Pass 17's code unchanged (as Pass 19's H3).
T5 (identity) and T6 (performance) are judged outside this script; C1-C4 on
game_v25 come from `evaluate_pass20_chronicle.py`.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from collections.abc import Mapping, Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any, Final

from evaluate_pass17_data import CohortSpec, Criterion, Metric
from evaluate_pass17_family_read import read_family
from evaluate_pass17_read import load_cohort
from evaluate_pass17_stage_a import health_metrics, measure
from evaluate_pass17_stage_b import evaluate as evaluate_family, fraction_metric, split_exposure
from evaluate_pass17_statistics import UndefinedStatistic

SEEDS: Final = tuple(range(19400, 19432))
DAYS: Final = 120
OLD_AGE: Final = 3
F5_BAR: Final = Criterion(None, None, "reference", "share")
KIND: Final = {"settlement_formed": 12, "daughter_founded": 14, "settlement_dissolved": 16, "died": 11,
    "joined": 8, "left_settlement": 9, "settled": 7, "porter_departed": 17, "porter_delivered": 18,
    "quarrel": 19, "villages_estranged": 20, "villages_reconciled": 21}
# pass20.trips columns.
PORTER, FROM, TO, PKIND, CAUSE, WITHDRAWN, DELIVERED, RETURNED, LOST, FAILED, DEPART, END = range(12)


def metric_json(metric: Metric) -> dict[str, Any]:
    row = asdict(metric)
    row["ci95"] = list(metric.ci95) if metric.ci95 is not None else None
    return row


def runs(probe: Path) -> dict[int, dict[str, Any]]:
    return {run["seed"]: run for run in json.loads((probe / "summary.json").read_text())["runs"]}


def chronicle(directory: Path, seed: int) -> list[list[Any]]:
    return json.loads((directory / f"chronicle_{seed}.json").read_text())["entries"]


def village_facts(run: Mapping[str, Any], entries: Sequence[Sequence[Any]], day_tick: int) -> dict[str, Any]:
    """Daughters, their regions, their non-old-age deaths, and alive at `day_tick`."""
    centres = [tuple(c["centre"]) for c in run.get("seed_info", {}).get("wild_regions", {}).get("centres", [])]
    log = run.get("pass11", {}).get("fission_log", [])
    region_of: dict[int, Any] = {}
    for record in log:
        if record.get("daughter") is not None:
            site = tuple(record["site"])
            nearest = min(range(len(centres)), key=lambda i: (max(abs(centres[i][0] - site[0]), abs(centres[i][1] - site[1])), i)) if centres else None
            region_of[record["daughter"]] = nearest
    daughters = {e[5] for e in entries if e[1] == KIND["daughter_founded"]}
    founded = {e[5]: e[0] for e in entries if e[1] == KIND["daughter_founded"]}
    dissolved = {e[5]: e[0] for e in entries if e[1] == KIND["settlement_dissolved"]}
    formed = {e[5] for e in entries if e[1] == KIND["settlement_formed"]}
    deaths_by_village: Counter = Counter()
    for e in entries:
        if e[1] == KIND["died"] and e[7] != OLD_AGE and e[5] is not None:
            deaths_by_village[e[5]] += 1
    alive_daughters = sum(1 for d in daughters if founded[d] <= day_tick and dissolved.get(d, 10**18) > day_tick)
    region = {sid: ("camp" if sid not in daughters else region_of.get(sid)) for sid in formed | daughters}
    return {"daughters": sorted(daughters), "region": region, "deaths": deaths_by_village,
        "daughter_deaths": sum(deaths_by_village[d] for d in daughters), "alive_daughters_at_day": alive_daughters}


def mean_difference(pairs: Sequence[tuple[float, float]]) -> float:
    if not pairs:
        raise UndefinedStatistic("no worlds")
    return sum(a - b for a, b in pairs) / len(pairs)


def share(values: Sequence[bool]) -> float | None:
    return sum(values) / len(values) if values else None


def evaluate(args: argparse.Namespace) -> dict[str, Any]:
    cand, base = runs(args.candidate), runs(args.baseline)
    seeds = [s for s in SEEDS if s in cand][: len(SEEDS)]
    if args.seeds:
        start, end = (int(x) for x in args.seeds.split(".."))
        seeds = list(range(start, end))
    day_tick = 118 * 1440
    worlds = []
    for seed in seeds:
        rc, rb = cand[seed], base[seed]
        ec, eb = chronicle(args.candidate_chronicle, seed), chronicle(args.baseline_chronicle, seed)
        block = rc["pass20"]
        t = block["tallies"]
        alive = rc["settlements_alive_daily"]
        vc, vb = village_facts(rc, ec, day_tick), village_facts(rb, eb, day_tick)
        trips = block["trips"]
        received = {row[TO] for row in trips if row[DELIVERED] > 0}
        # T7: tallies against chronicle entries and the trip ledger.
        kinds = Counter(e[1] for e in ec)
        delivered_entries = [e for e in ec if e[1] == KIND["porter_delivered"]]
        t7 = {
            "departed": (t["porters_departed"], kinds[KIND["porter_departed"]]),
            "delivered": (t["porter_deliveries"], sum(1 for e in delivered_entries if e[7] < 10)),
            "turned_back": (t["porter_turnbacks"], sum(1 for e in delivered_entries if e[7] >= 10)),
            "quarrels": (t["quarrels"], kinds[KIND["quarrel"]]),
            "estranged": (t["villages_estranged"], kinds[KIND["villages_estranged"]]),
            "reconciled": (t["villages_reconciled"], kinds[KIND["villages_reconciled"]]),
            "trip_records": (t["porters_departed"], len(trips)),
            "withdrawn": (t["porter_units_withdrawn"], sum(r[WITHDRAWN] for r in trips)),
            "delivered_units": (t["porter_food_delivered"] + t["porter_wood_delivered"], sum(r[DELIVERED] for r in trips)),
            "returned_units": (t["porter_units_returned"], sum(r[RETURNED] for r in trips)),
            "lost_units": (t["porter_units_lost"], sum(r[LOST] for r in trips)),
        }
        ended = [r for r in trips if r[END] >= 0]
        unbalanced = [r for r in ended if r[WITHDRAWN] != r[DELIVERED] + r[RETURNED] + r[LOST]]
        in_flight = sum(r[WITHDRAWN] - r[DELIVERED] - r[RETURNED] - r[LOST] for r in trips if r[END] < 0)
        # Each departure entry matches one trip record; each delivered entry one record.
        departures = Counter((e[0], e[2], e[5], e[6], e[7], e[8]) for e in ec if e[1] == KIND["porter_departed"])
        records = Counter((r[DEPART], r[PORTER], r[FROM], r[TO], r[CAUSE], r[WITHDRAWN]) for r in trips)
        # Members never change village during a trip.
        moves = defaultdict(list)
        for e in ec:
            if e[1] in (KIND["joined"], KIND["left_settlement"], KIND["settled"]) and e[2] is not None:
                moves[e[2]].append(e[0])
        moved = [r for r in trips if any(r[DEPART] <= tick <= (r[END] if r[END] >= 0 else 10**18) for tick in moves.get(r[PORTER], []))]
        worlds.append({
            "seed": seed, "multi_village": bool(alive) and max(alive) >= 2, "max_alive": max(alive) if alive else 0,
            "tallies": t, "t7": {k: list(v) for k, v in t7.items()},
            "t7_mismatches": [k for k, (a, b) in t7.items() if a != b] + (["departure_entries"] if departures != records else []),
            "unbalanced_trips": len(unbalanced), "in_flight_units": in_flight, "in_flight_reported": block["in_flight_units"],
            "trips_with_village_change": len(moved),
            "food_moved": sum(r[DELIVERED] for r in trips if r[PKIND] == 0), "wood_moved": sum(r[DELIVERED] for r in trips if r[PKIND] == 1),
            "trips": len(trips), "failed": sum(r[FAILED] for r in trips), "lost_loads": sum(1 for r in trips if r[LOST] > 0),
            "causes": dict(Counter(r[CAUSE] for r in trips)),
            "pairs_end": block["pairs_end"],
            "daughter_deaths_v25": vc["daughter_deaths"], "daughter_deaths_v24": vb["daughter_deaths"],
            "alive_daughters_v25": vc["alive_daughters_at_day"], "alive_daughters_v24": vb["alive_daughters_at_day"],
            "received": sorted(received), "regions_v25": {str(k): v for k, v in vc["region"].items()},
            "regions_v24": {str(k): v for k, v in vb["region"].items()},
            "deaths_v25": {str(k): v for k, v in vc["deaths"].items()}, "deaths_v24": {str(k): v for k, v in vb["deaths"].items()},
        })
    multi = [w for w in worlds if w["multi_village"]]
    # T1.
    porters = sum(w["tallies"]["porters_departed"] for w in worlds)
    failed = sum(w["tallies"]["porter_turnbacks"] for w in worlds)
    t1_share = share([w["tallies"]["porter_deliveries"] > 0 for w in multi])
    fail_share = failed / porters if porters else None
    t1 = {"multi_village_worlds": len(multi), "share_with_delivery": t1_share, "failed_share": fail_share,
        "porters": porters, "failed": failed,
        "deliveries_per_world": [w["tallies"]["porter_deliveries"] for w in worlds],
        "food_moved": sum(w["food_moved"] for w in worlds), "wood_moved": sum(w["wood_moved"] for w in worlds),
        "lost_loads": sum(w["lost_loads"] for w in worlds), "lost_units": sum(w["tallies"]["porter_units_lost"] for w in worlds),
        "trips": sum(w["tallies"]["porter_trips"] for w in worlds),
        "causes": dict(sum((Counter({int(k): v for k, v in w["causes"].items()}) for w in worlds), Counter()))}
    t1["verdict"] = "PASS" if t1_share is not None and t1_share >= 0.6 and (fail_share is None or fail_share <= 0.1) else "FAIL"
    # T2a.
    pairs = [(float(w["daughter_deaths_v25"]), float(w["daughter_deaths_v24"])) for w in multi]
    t2_metric = measure(pairs, mean_difference, Criterion(None, 0.0, "mean daughter non-old-age deaths v25 - v24 <= 0 (multi-village worlds)", "deaths/world"))
    alive25 = sum(w["alive_daughters_v25"] for w in worlds)
    alive24 = sum(w["alive_daughters_v24"] for w in worlds)
    t2a_ok = t2_metric.value is not None and t2_metric.value <= 0 and alive25 >= alive24
    # T2b: region-matched villages that received aid.
    matched, unmatched = [], 0
    for w in worlds:
        for sid in w["received"]:
            region = w["regions_v25"].get(str(sid))
            twins = [k for k, v in w["regions_v24"].items() if v == region]
            if region is None or not twins:
                unmatched += 1
                continue
            matched.append((float(w["deaths_v25"].get(str(sid), 0)), float(sum(w["deaths_v24"].get(k, 0) for k in twins))))
    t2b_metric = measure(matched, mean_difference, Criterion(None, None, "reported: aided village non-old-age deaths v25 - v24 (region-matched)", "deaths/village"))
    received_total = len(matched) + unmatched
    t2 = {"T2a": {"metric": metric_json(t2_metric), "worlds": len(multi),
            "daughter_deaths_v25": sum(p[0] for p in pairs), "daughter_deaths_v24": sum(p[1] for p in pairs),
            "alive_daughters_day118_v25": alive25, "alive_daughters_day118_v24": alive24, "verdict": "PASS" if t2a_ok else "FAIL"},
        "T2b": {"metric": metric_json(t2b_metric), "aided_villages": received_total, "matched": len(matched),
            "match_rate": len(matched) / received_total if received_total else None}}
    # T3.
    dispute_share = share([w["tallies"]["forage_disputes"] > 0 for w in multi])
    quarrel_share = share([w["tallies"]["quarrels"] > 0 for w in multi])
    pairs_end = [p for w in worlds for p in w["pairs_end"]]
    estranged_share = (sum(p[3] for p in pairs_end) / len(pairs_end)) if pairs_end else None
    t3_ok = (dispute_share or 0) >= 0.3 and (quarrel_share or 0) >= 0.15 and (estranged_share is None or estranged_share <= 0.5)
    t3 = {"share_with_dispute": dispute_share, "share_with_quarrel": quarrel_share, "pairs_at_end": len(pairs_end),
        "estranged_share_at_end": estranged_share, "disputes": sum(w["tallies"]["forage_disputes"] for w in worlds),
        "quarrels": sum(w["tallies"]["quarrels"] for w in worlds),
        "estrangements": sum(w["tallies"]["villages_estranged"] for w in worlds),
        "reconciliations": sum(w["tallies"]["villages_reconciled"] for w in worlds),
        "relation_values_at_end": [p[2] for p in pairs_end], "verdict": "PASS" if t3_ok else "FAIL"}
    # T7.
    t7_bad = {w["seed"]: w["t7_mismatches"] for w in worlds if w["t7_mismatches"] or w["unbalanced_trips"]
        or w["in_flight_units"] != w["in_flight_reported"] or w["trips_with_village_change"]}
    t7 = {"worlds_with_mismatch": t7_bad, "unbalanced_trips": sum(w["unbalanced_trips"] for w in worlds),
        "trips_with_village_change": sum(w["trips_with_village_change"] for w in worlds),
        "in_flight_units_end": sum(w["in_flight_units"] for w in worlds),
        "totals": {k: [sum(w["t7"][k][0] for w in worlds), sum(w["t7"][k][1] for w in worlds)] for k in worlds[0]["t7"]} if worlds else {},
        "verdict": "PASS" if not t7_bad else "FAIL"}
    return {"seeds": seeds, "worlds": worlds, "T1": t1, "T2": t2, "T3": t3, "T7": t7}


def t4(args: argparse.Namespace, seeds: Sequence[int]) -> dict[str, Any]:
    candidate = load_cohort(args.candidate, CohortSpec("game_v25", tuple(seeds), args.days))
    baseline = load_cohort(args.baseline, CohortSpec("game_v24", tuple(seeds), args.days))
    health = health_metrics(candidate, baseline)
    families = tuple(read_family(candidate, world) for world in candidate.worlds)
    base_families = tuple(read_family(baseline, world) for world in baseline.worlds)
    f1 = evaluate_family(candidate, baseline, families).stage.metrics["F1"]
    f1_base = evaluate_family(baseline, baseline, base_families).stage.metrics["F1"]
    f5 = fraction_metric([split_exposure(world) for world in families], F5_BAR)
    f5_base = fraction_metric([split_exposure(world) for world in base_families], F5_BAR)
    diffs = {name: (None if a.value is None or b.value is None else a.value - b.value)
        for name, a, b in (("F1", f1, f1_base), ("F5", f5, f5_base))}
    family_ok = all(d is not None and abs(d) <= 0.05 for d in diffs.values())
    verdict = "PASS" if family_ok and all(m.verdict == "PASS" for m in health.values()) else "FAIL"
    return {"verdict": verdict, "health": {k: metric_json(m) for k, m in health.items()},
        "F1": metric_json(f1), "F1_game_v24": metric_json(f1_base), "F5": metric_json(f5), "F5_game_v24": metric_json(f5_base),
        "differences": diffs, "family_within_0_05": family_ok}


def fmt(metric: Mapping[str, Any], digits: int = 3) -> str:
    value, ci = metric["value"], metric["ci95"]
    if value is None:
        return f"ERROR ({metric.get('error')})"
    return f"{value:.{digits}f} [{ci[0]:.{digits}f}, {ci[1]:.{digits}f}]" if ci else f"{value:.{digits}f}"


def num(value: float | None, digits: int = 3) -> str:
    return "n/a" if value is None else f"{value:.{digits}f}"


def markdown(report: Mapping[str, Any]) -> str:
    t1, t2, t3, t4r, t7 = report["T1"], report["T2"], report["T3"], report["T4"], report["T7"]
    lines = ["# Pass 20 evaluation: game_v25 vs game_v24", "",
        f"Seeds {report['seeds'][0]}–{report['seeds'][-1]} ({len(report['seeds'])} worlds), {report['days']} days, paired by seed; "
        "Pass 17's definitions and bootstrap (2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 20`.", "",
        "| # | rule | value | verdict |", "|---|---|---|---|",
        f"| T1 | ≥ 0.6 of multi-village worlds deliver; failed-trip share ≤ 0.1 | {num(t1['share_with_delivery'])} of {t1['multi_village_worlds']} worlds; failed {num(t1['failed_share'])} ({t1['failed']}/{t1['porters']} porters) | {t1['verdict']} |",
        f"| T2a | daughter non-old-age deaths v25 − v24 ≤ 0 per multi-village world; daughters alive at day 118 ≥ v24 | {fmt(t2['T2a']['metric'])} ({t2['T2a']['daughter_deaths_v25']:.0f} vs {t2['T2a']['daughter_deaths_v24']:.0f}); alive {t2['T2a']['alive_daughters_day118_v25']} vs {t2['T2a']['alive_daughters_day118_v24']} | {t2['T2a']['verdict']} |",
        f"| T2b | (reported) aided villages, region-matched, deaths v25 − v24 | {fmt(t2['T2b']['metric'])}; matched {t2['T2b']['matched']}/{t2['T2b']['aided_villages']} | reported |",
        f"| T3 | ≥ 0.3 dispute, ≥ 0.15 quarrel; estranged share at day 120 ≤ 0.5 | dispute {num(t3['share_with_dispute'])}, quarrel {num(t3['share_with_quarrel'])}, estranged {num(t3['estranged_share_at_end'])} of {t3['pairs_at_end']} pairs | {t3['verdict']} |"]
    for key, metric in t4r["health"].items():
        lines.append(f"| T4 | {key}: {metric['bar']} | {metric['candidate']:.3f} vs {metric['baseline']:.3f} (stat {fmt(metric)}) | {metric['verdict']} |")
    d = t4r["differences"]
    for name in ("F1", "F5"):
        ok = d[name] is not None and abs(d[name]) <= 0.05
        lines.append(f"| T4 | {name} within ± 0.05 of game_v24 | {fmt(t4r[name])} vs {fmt(t4r[name + '_game_v24'])} (diff {num(d[name], 4)}) | {'PASS' if ok else 'FAIL'} |")
    lines.append(f"| T7 | tallies = chronicle entries; withdrawn = delivered + returned + lost; no village change on a trip | mismatched worlds {len(t7['worlds_with_mismatch'])}, unbalanced trips {t7['unbalanced_trips']}, village changes {t7['trips_with_village_change']} | {t7['verdict']} (C1–C4: see chronicle.md) |")
    lines += ["", "## Porters (T1)", "",
        f"Trips {t1['trips']}, porters {t1['porters']}, delivered food {t1['food_moved']}, wood {t1['wood_moved']}, turned back {t1['failed']}, "
        f"loads lost {t1['lost_loads']} ({t1['lost_units']} units); departure causes {t1['causes']} (0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood).", "",
        "| seed | villages | trips | porters | delivered | turned back | food | wood | lost | disputes | quarrels | estranged | reconciled | daughter deaths v25/v24 | daughters alive d118 v25/v24 | pairs at end [a, b, value, estranged, base] |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for w in report["worlds"]:
        t = w["tallies"]
        lines.append(f"| {w['seed']} | {w['max_alive']} | {t['porter_trips']} | {t['porters_departed']} | {t['porter_deliveries']} | {t['porter_turnbacks']} | "
            f"{w['food_moved']} | {w['wood_moved']} | {w['lost_loads']} | {t['forage_disputes']} | {t['quarrels']} | {t['villages_estranged']} | {t['villages_reconciled']} | "
            f"{w['daughter_deaths_v25']}/{w['daughter_deaths_v24']} | {w['alive_daughters_v25']}/{w['alive_daughters_v24']} | {w['pairs_end']} |")
    lines += ["", "## T7 totals (tally, ledger or chronicle)", ""]
    for key, (a, b) in t7["totals"].items():
        lines.append(f"- {key}: {a} / {b}")
    lines += [f"- units in flight at day 120: {t7['in_flight_units_end']}", "",
        "T5 (identity) and T6 (performance) are judged outside this script.", ""]
    return "\n".join(lines)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--candidate-chronicle", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--baseline-chronicle", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    parser.add_argument("--seeds", default=f"{SEEDS[0]}..{SEEDS[-1] + 1}")
    parser.add_argument("--days", type=int, default=DAYS)
    parser.add_argument("--no-t4", action="store_true", help="skip T4 (needs --agent-trace runs)")
    args = parser.parse_args(argv)
    report = evaluate(args)
    report["days"] = args.days
    report["schema_version"] = 20
    report["T4"] = {"verdict": "SKIPPED", "health": {}, "differences": {"F1": None, "F5": None},
        "F1": None, "F1_game_v24": None, "F5": None, "F5_game_v24": None} if args.no_t4 else t4(args, report["seeds"])
    args.out_json.write_text(json.dumps(report, indent=1, sort_keys=True, default=str) + "\n")
    text = markdown(report) if not args.no_t4 else json.dumps({k: report[k] for k in ("T1", "T2", "T3", "T7")}, indent=1, default=str)
    args.out_md.write_text(text)
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
