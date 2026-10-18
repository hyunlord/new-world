#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_pass21.py --v26 DIR --v26-chronicle DIR --v25 DIR --v25-chronicle DIR
#     --v24 DIR --v24-chronicle DIR --out-md OUT.md --out-json OUT.json [--seeds 19600..19632] [--no-u4]
"""Pass 21 U1-U4 and U7: game_v26 against game_v25 and game_v24 (METRICS.md `## Pass 21`).

Every cohort is a complete `worldsim_probe --agent-trace --chronicle-out` run
of the same seeds for 120 days (the v26 and v25 chronicle directories also
hold `relations_<seed>.json`, the measurement-only dispute log). A
*multi-village world* is one with >= 2 settlements alive at once in
game_v26 (`settlements_alive_daily`). Daughter villages are the settlements
of `DaughterFounded` entries; their non-old-age deaths and the daughters alive
at day 118 are Pass 20's T2a quantities (`evaluate_pass20.village_facts`).
U4's health rules (Pass 17 R2) and F1/F5 reuse Pass 17's code unchanged, as
Pass 20's T4. U5 (identity) and U6 (performance) are judged elsewhere; the
chronicle checks C1-C4 come from `evaluate_pass20_chronicle.py`.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any, Final

sys.path.insert(0, str(Path(__file__).resolve().parent))
from evaluate_pass17_data import CohortSpec, Criterion  # noqa: E402
from evaluate_pass17_family_read import read_family  # noqa: E402
from evaluate_pass17_read import load_cohort  # noqa: E402
from evaluate_pass17_stage_a import health_metrics, measure  # noqa: E402
from evaluate_pass17_stage_b import evaluate as evaluate_family, fraction_metric, split_exposure  # noqa: E402
from evaluate_pass20 import KIND, chronicle, living_ids, mean_difference, metric_json, runs, share, village_facts  # noqa: E402

DAYS: Final = 120
F5_BAR: Final = Criterion(None, None, "reference", "share")
CAP_DISPUTES: Final = 3
CAP_QUARRELS: Final = 1
# pass20.trips columns.
PORTER, FROM, TO, PKIND, CAUSE, WITHDRAWN, DELIVERED, RETURNED, LOST, FAILED, DEPART, END = range(12)
# pass20.days columns.
D_ESTRANGED, D_REACH, D_AID_FOOD, D_AID_WOOD, D_EXCHANGE, D_SENT = 4, 5, 14, 15, 16, 17


def relations_log(directory: Path, seed: int) -> list[list[int]]:
    path = directory / f"relations_{seed}.json"
    return json.loads(path.read_text())["dispute_days"] if path.exists() else []


def world(seed: int, r26: Mapping[str, Any], r25: Mapping[str, Any], r24: Mapping[str, Any],
          e26: Sequence[Sequence[Any]], e25: Sequence[Sequence[Any]], e24: Sequence[Sequence[Any]],
          log: Sequence[Sequence[int]], day_tick: int) -> dict[str, Any]:
    block = r26["pass20"]
    t = block["tallies"]
    p21 = block.get("pass21") or {}
    t21 = p21.get("tallies", {})
    alive = r26["settlements_alive_daily"]
    v26, v25, v24 = (village_facts(r, e, day_tick) for r, e in ((r26, e26), (r25, e25), (r24, e24)))
    trips = block["trips"]
    days = block["days"]
    kinds = Counter(e[1] for e in e26)
    delivered_entries = [e for e in e26 if e[1] == KIND["porter_delivered"]]
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
        # Pass 21 (6.3): every dispute is in the log; per pair-day the costed
        # disputes are min(n, cap), so costed + uncosted = disputes; quarrels
        # per pair-day never exceed the cap and their sum is the tally;
        # quarrels while estranged as the pass21 tally says.
        "disputes_logged": (t["forage_disputes"], sum(row[3] for row in log)),
        "costed_plus_uncosted": (t["forage_disputes"], sum(min(row[3], CAP_DISPUTES) for row in log) + t21.get("disputes_uncosted", 0)),
        "quarrels_logged": (t["quarrels"], sum(row[4] for row in log)),
        "pair_reconciliations": (t["villages_reconciled"], sum(len(p[4]) for p in p21.get("pairs", []))),
        "pair_estrangements": (t["villages_estranged"], sum(len(p[3]) for p in p21.get("pairs", []))),
    }
    over_cap = [row for row in log if row[4] > CAP_QUARRELS]
    ended = [r for r in trips if r[END] >= 0]
    unbalanced = [r for r in ended if r[WITHDRAWN] != r[DELIVERED] + r[RETURNED] + r[LOST]]
    in_flight = sum(r[WITHDRAWN] - r[DELIVERED] - r[RETURNED] - r[LOST] for r in trips if r[END] < 0)
    departures = Counter((e[0], e[2], e[5], e[6], e[7], e[8]) for e in e26 if e[1] == KIND["porter_departed"])
    records = Counter((r[DEPART], r[PORTER], r[FROM], r[TO], r[CAUSE], r[WITHDRAWN]) for r in trips)
    moves: dict[int, list[int]] = {}
    for e in e26:
        if e[1] in (KIND["joined"], KIND["left_settlement"], KIND["settled"]) and e[2] is not None:
            moves.setdefault(e[2], []).append(e[0])
    moved = [r for r in trips if any(r[DEPART] <= tick <= (r[END] if r[END] >= 0 else 10**18) for tick in moves.get(r[PORTER], []))]
    trip_keys = {(r[FROM], r[TO], r[DEPART]) for r in trips}
    delivered_keys = {(r[FROM], r[TO], r[DEPART]) for r in trips if r[DELIVERED] > 0}
    return {
        "seed": seed, "multi_village": bool(alive) and max(alive) >= 2, "max_alive": max(alive) if alive else 0,
        "tallies": t, "pass21": p21,
        "t7": {k: list(v) for k, v in t7.items()},
        "t7_mismatches": [k for k, (a, b) in t7.items() if a != b] + (["departure_entries"] if departures != records else [])
            + (["quarrel_cap"] if over_cap else []),
        "unbalanced_trips": len(unbalanced), "in_flight_units": in_flight, "in_flight_reported": block["in_flight_units"],
        "trips_with_village_change": len(moved),
        "trips": len(trip_keys), "delivered_trips": len(delivered_keys),
        "porters": len(trips), "failed": sum(r[FAILED] for r in trips), "lost_loads": sum(1 for r in trips if r[LOST] > 0),
        "food_moved": sum(r[DELIVERED] for r in trips if r[PKIND] == 0), "wood_moved": sum(r[DELIVERED] for r in trips if r[PKIND] == 1),
        "causes": dict(Counter(r[CAUSE] for r in trips)),
        "exchange_trips": len({(r[FROM], r[TO], r[DEPART]) for r in trips if r[CAUSE] == 2}),
        "exchange_pair_days": sum(d[D_EXCHANGE] for d in days),
        "exchange_pair_days_open": sum(d[D_EXCHANGE] for d in days if d[D_REACH] and not d[D_ESTRANGED]),
        "aid_food_pair_days_open": sum(d[D_AID_FOOD] for d in days if d[D_REACH] and not d[D_ESTRANGED]),
        "aid_food_pair_days_estranged": sum(d[D_AID_FOOD] for d in days if d[D_REACH] and d[D_ESTRANGED]),
        "pairs_end": block["pairs_end"],
        "pairs_invariant": {(p[0], p[1]) for p in block["pairs_end"]} == {
            (a, b) for a in living_ids(e26) for b in living_ids(e26) if a < b}
            and all(-1000 <= p[2] <= 1000 for p in block["pairs_end"]),
        "daughter_deaths": {"v26": v26["daughter_deaths"], "v25": v25["daughter_deaths"], "v24": v24["daughter_deaths"]},
        "alive_daughters": {"v26": v26["alive_daughters_at_day"], "v25": v25["alive_daughters_at_day"], "v24": v24["alive_daughters_at_day"]},
        "v25_tallies": r25["pass20"]["tallies"],
        "v25_delivered_trips": len({(r[FROM], r[TO], r[DEPART]) for r in r25["pass20"]["trips"] if r[DELIVERED] > 0}),
    }


def evaluate(args: argparse.Namespace) -> dict[str, Any]:
    c26, c25, c24 = runs(args.v26), runs(args.v25), runs(args.v24)
    start, end = (int(x) for x in args.seeds.split(".."))
    seeds = list(range(start, end))
    day_tick = 118 * 1440
    worlds = [world(s, c26[s], c25[s], c24[s], chronicle(args.v26_chronicle, s), chronicle(args.v25_chronicle, s),
                    chronicle(args.v24_chronicle, s), relations_log(args.v26_chronicle, s), day_tick) for s in seeds]
    multi = [w for w in worlds if w["multi_village"]]
    # U1.
    # METRICS: failed share = porters turned back / porters departed (tallies).
    porters = sum(w["tallies"]["porters_departed"] for w in worlds)
    failed = sum(w["tallies"]["porter_turnbacks"] for w in worlds)
    u1_share = share([w["delivered_trips"] > 0 for w in multi])
    fail_share = failed / porters if porters else None
    u1 = {"multi_village_worlds": len(multi), "delivering_worlds": sum(1 for w in multi if w["delivered_trips"] > 0),
        "share_with_delivery": u1_share, "failed_share": fail_share, "porters": porters, "failed": failed,
        "trips": sum(w["trips"] for w in worlds), "delivered_trips": sum(w["delivered_trips"] for w in worlds),
        "deliveries_per_world": {w["seed"]: w["delivered_trips"] for w in worlds},
        "food_moved": sum(w["food_moved"] for w in worlds), "wood_moved": sum(w["wood_moved"] for w in worlds),
        "lost_loads": sum(w["lost_loads"] for w in worlds),
        "causes": dict(sum((Counter({int(k): v for k, v in w["causes"].items()}) for w in worlds), Counter())),
        "aid_food_pair_days_open": sum(w["aid_food_pair_days_open"] for w in worlds),
        "aid_food_pair_days_estranged": sum(w["aid_food_pair_days_estranged"] for w in worlds),
        "v25_delivering_worlds": sum(1 for w in multi if w["v25_delivered_trips"] > 0)}
    u1["verdict"] = "PASS" if u1_share is not None and u1_share >= 0.6 and (fail_share is None or fail_share <= 0.1) else "FAIL"
    # U2.
    u2 = {"exchange_trips": sum(w["exchange_trips"] for w in worlds), "exchange_pair_days": sum(w["exchange_pair_days"] for w in worlds),
        "exchange_pair_days_open": sum(w["exchange_pair_days_open"] for w in worlds),
        "exchange_porters": sum(w["causes"].get(2, 0) + w["causes"].get(3, 0) for w in worlds)}
    u2["verdict"] = "PASS" if u2["exchange_trips"] >= 1 else "FAIL"
    # U3.
    dispute_share = share([w["tallies"]["forage_disputes"] > 0 for w in multi])
    quarrel_share = share([w["tallies"]["quarrels"] > 0 for w in multi])
    pairs_end = [p for w in worlds for p in w["pairs_end"]]
    estranged_share = (sum(p[3] for p in pairs_end) / len(pairs_end)) if pairs_end else None
    quarrels = sum(w["tallies"]["quarrels"] for w in worlds)
    q_est = sum(w["pass21"].get("tallies", {}).get("quarrels_while_estranged", 0) for w in worlds)
    q_est_share = q_est / quarrels if quarrels else None
    estrangements = sum(w["tallies"]["villages_estranged"] for w in worlds)
    reconciliations = sum(w["tallies"]["villages_reconciled"] for w in worlds)
    rec_share = reconciliations / estrangements if estrangements else None
    rec_ok = estrangements < 3 or (rec_share or 0) >= 0.5
    u3_ok = ((dispute_share or 0) >= 0.3 and (quarrel_share or 0) >= 0.15 and (estranged_share is None or estranged_share <= 0.25)
             and (q_est_share is None or q_est_share <= 0.2) and rec_ok)
    days_estranged = sum(p[2] for w in worlds for p in w["pass21"].get("pairs", []))
    u3 = {"share_with_dispute": dispute_share, "share_with_quarrel": quarrel_share, "pairs_at_end": len(pairs_end),
        "estranged_at_end": sum(p[3] for p in pairs_end), "estranged_share_at_end": estranged_share,
        "disputes": sum(w["tallies"]["forage_disputes"] for w in worlds),
        "disputes_uncosted": sum(w["pass21"].get("tallies", {}).get("disputes_uncosted", 0) for w in worlds),
        "quarrels": quarrels, "quarrels_capped": sum(w["pass21"].get("tallies", {}).get("quarrels_capped", 0) for w in worlds),
        "quarrels_while_estranged": q_est, "quarrels_while_estranged_share": q_est_share,
        "estrangements": estrangements, "reconciliations": reconciliations, "reconciled_share": rec_share,
        "reconciled_bar_applies": estrangements >= 3, "days_estranged": days_estranged,
        "relation_values_at_end": [p[2] for p in pairs_end],
        "v25": {"disputes": sum(w["v25_tallies"]["forage_disputes"] for w in worlds), "quarrels": sum(w["v25_tallies"]["quarrels"] for w in worlds),
            "estrangements": sum(w["v25_tallies"]["villages_estranged"] for w in worlds),
            "reconciliations": sum(w["v25_tallies"]["villages_reconciled"] for w in worlds)},
        "verdict": "PASS" if u3_ok else "FAIL"}
    # U4 deaths (the health half is added by u4()).
    deaths = {}
    for ref in ("v24", "v25"):
        pairs = [(float(w["daughter_deaths"]["v26"]), float(w["daughter_deaths"][ref])) for w in multi]
        metric = measure(pairs, mean_difference, Criterion(None, 0.0, f"mean daughter non-old-age deaths v26 - {ref} <= 0 (multi-village worlds)", "deaths/world"))
        deaths[ref] = {"metric": metric_json(metric), "v26": sum(p[0] for p in pairs), ref: sum(p[1] for p in pairs),
            "ok": metric.value is not None and metric.value <= 0}
    alive = {k: sum(w["alive_daughters"][k] for w in worlds) for k in ("v26", "v25", "v24")}
    deaths_ok = deaths["v24"]["ok"] and deaths["v25"]["ok"] and alive["v26"] >= alive["v24"]
    # U7.
    t7_bad = {w["seed"]: w["t7_mismatches"] for w in worlds if w["t7_mismatches"] or w["unbalanced_trips"]
        or w["in_flight_units"] != w["in_flight_reported"] or w["trips_with_village_change"]}
    u7 = {"worlds_with_mismatch": t7_bad, "unbalanced_trips": sum(w["unbalanced_trips"] for w in worlds),
        "trips_with_village_change": sum(w["trips_with_village_change"] for w in worlds),
        "in_flight_units_end": sum(w["in_flight_units"] for w in worlds),
        "pairs_invariant_worlds": sum(1 for w in worlds if w["pairs_invariant"]),
        "totals": {k: [sum(w["t7"][k][0] for w in worlds), sum(w["t7"][k][1] for w in worlds)] for k in worlds[0]["t7"]} if worlds else {},
        "verdict": "PASS" if not t7_bad else "FAIL"}
    return {"seeds": seeds, "worlds": worlds, "U1": u1, "U2": u2, "U3": u3,
            "U4_deaths": {"deaths": deaths, "alive_daughters_day118": alive, "ok": deaths_ok}, "U7": u7}


def u4_health(args: argparse.Namespace, seeds: Sequence[int]) -> dict[str, Any]:
    candidate = load_cohort(args.v26, CohortSpec("game_v26", tuple(seeds), DAYS))
    baseline = load_cohort(args.v24, CohortSpec("game_v24", tuple(seeds), DAYS))
    health = health_metrics(candidate, baseline)
    families = tuple(read_family(candidate, w) for w in candidate.worlds)
    base_families = tuple(read_family(baseline, w) for w in baseline.worlds)
    f1 = evaluate_family(candidate, baseline, families).stage.metrics["F1"]
    f1_base = evaluate_family(baseline, baseline, base_families).stage.metrics["F1"]
    f5 = fraction_metric([split_exposure(w) for w in families], F5_BAR)
    f5_base = fraction_metric([split_exposure(w) for w in base_families], F5_BAR)
    diffs = {name: (None if a.value is None or b.value is None else a.value - b.value)
        for name, a, b in (("F1", f1, f1_base), ("F5", f5, f5_base))}
    family_ok = all(d is not None and abs(d) <= 0.05 for d in diffs.values())
    return {"health": {k: metric_json(m) for k, m in health.items()}, "health_ok": all(m.verdict == "PASS" for m in health.values()),
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
    u1, u2, u3, u4d, u4h, u7 = report["U1"], report["U2"], report["U3"], report["U4_deaths"], report["U4_health"], report["U7"]
    d24, d25 = u4d["deaths"]["v24"], u4d["deaths"]["v25"]
    alive = u4d["alive_daughters_day118"]
    lines = ["# Pass 21 evaluation: game_v26 vs game_v25 and game_v24", "",
        f"Seeds {report['seeds'][0]}–{report['seeds'][-1]} ({len(report['seeds'])} worlds), {DAYS} days, paired by seed; "
        "Pass 17's definitions and bootstrap (2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 21`. "
        "Multi-village world: ≥ 2 settlements alive at once in game_v26.", "",
        "| # | rule | value | verdict |", "|---|---|---|---|",
        f"| U1 | ≥ 0.6 of multi-village worlds deliver; failed-trip share ≤ 0.1 | {num(u1['share_with_delivery'])} ({u1['delivering_worlds']} of {u1['multi_village_worlds']}); failed {num(u1['failed_share'])} ({u1['failed']}/{u1['porters']} porters) | {u1['verdict']} |",
        f"| U2 | ≥ 1 exchange trip over the {len(report['seeds'])} worlds | {u2['exchange_trips']} exchange trips ({u2['exchange_porters']} porters); exchange pair-days {u2['exchange_pair_days']} ({u2['exchange_pair_days_open']} in reach, not estranged) | {u2['verdict']} |",
        f"| U3 | disputes ≥ 0.3, quarrels ≥ 0.15 of multi-village worlds; estranged at d120 ≤ 0.25; quarrels while estranged ≤ 0.2; ≥ 3 estrangements → ≥ 0.5 reconciled | dispute {num(u3['share_with_dispute'])}, quarrel {num(u3['share_with_quarrel'])}, estranged {u3['estranged_at_end']} of {u3['pairs_at_end']} ({num(u3['estranged_share_at_end'])}), while estranged {u3['quarrels_while_estranged']} of {u3['quarrels']} ({num(u3['quarrels_while_estranged_share'])}), reconciled {u3['reconciliations']} of {u3['estrangements']} ({num(u3['reconciled_share'])}) | {u3['verdict']} |"]
    for key, metric in u4h.get("health", {}).items():
        lines.append(f"| U4 | {key}: {metric['bar']} (v26 vs v24) | {metric['candidate']:.3f} vs {metric['baseline']:.3f} (stat {fmt(metric)}) | {metric['verdict']} |")
    for name in ("F1", "F5"):
        d = u4h.get("differences", {}).get(name)
        if u4h.get(name):
            lines.append(f"| U4 | {name} within ± 0.05 of game_v24 | {fmt(u4h[name])} vs {fmt(u4h[name + '_game_v24'])} (diff {num(d, 4)}) | {'PASS' if d is not None and abs(d) <= 0.05 else 'FAIL'} |")
    lines += [
        f"| U4 | daughter non-old-age deaths per multi-village world v26 − v24 ≤ 0 | {fmt(d24['metric'])} ({d24['v26']:.0f} vs {d24['v24']:.0f}) | {'PASS' if d24['ok'] else 'FAIL'} |",
        f"| U4 | the same against game_v25 | {fmt(d25['metric'])} ({d25['v26']:.0f} vs {d25['v25']:.0f}) | {'PASS' if d25['ok'] else 'FAIL'} |",
        f"| U4 | daughters alive at day 118 ≥ game_v24 | {alive['v26']} vs {alive['v24']} (game_v25 {alive['v25']}) | {'PASS' if alive['v26'] >= alive['v24'] else 'FAIL'} |",
        f"| U7 | Pass 20's T7 on game_v26 + reconciliations = chronicle + costed + uncosted = disputes; C1–C4 (`evaluate_pass20_chronicle.py`, C4 against run B) | mismatched worlds {len(u7['worlds_with_mismatch'])}, unbalanced trips {u7['unbalanced_trips']}, village changes {u7['trips_with_village_change']}; C1–C4 {u7['chronicle']} | {u7['verdict']} |",
        "", f"U4 overall: {report['U4_verdict']}.", "",
        "## Porters (U1, U2)", "",
        f"Trips {u1['trips']} ({u1['delivered_trips']} delivered), porters {u1['porters']}, food moved {u1['food_moved']}, wood {u1['wood_moved']}, turned back {u1['failed']}, "
        f"loads lost {u1['lost_loads']}; departure causes {u1['causes']} (0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood). "
        f"In-reach food-aid pair-days: {u1['aid_food_pair_days_open']} open, {u1['aid_food_pair_days_estranged']} between estranged villages. "
        f"game_v25 delivering worlds on these seeds: {u1['v25_delivering_worlds']} of {u1['multi_village_worlds']}.", "",
        "## Conflict (U3)", "",
        f"Disputes {u3['disputes']} ({u3['disputes_uncosted']} past the daily dispute cap), quarrels {u3['quarrels']} ({u3['quarrels_capped']} disputes met the quarrel cap), "
        f"estrangements {u3['estrangements']}, reconciliations {u3['reconciliations']}, pair-days estranged {u3['days_estranged']}. "
        f"game_v25 on these seeds: disputes {u3['v25']['disputes']}, quarrels {u3['v25']['quarrels']}, estrangements {u3['v25']['estrangements']}, reconciliations {u3['v25']['reconciliations']}.", "",
        "| seed | villages | trips (delivered) | porters | food | wood | exchange trips | failed | disputes (uncosted) | quarrels (capped, while estranged) | estranged | reconciled | daughter deaths v26/v25/v24 | daughters alive d118 v26/v25/v24 | pairs at end [a, b, value, estranged, base] | v25 trips delivered |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for w in report["worlds"]:
        t, t21 = w["tallies"], w["pass21"].get("tallies", {})
        dd, al = w["daughter_deaths"], w["alive_daughters"]
        lines.append(f"| {w['seed']} | {w['max_alive']} | {w['trips']} ({w['delivered_trips']}) | {w['porters']} | {w['food_moved']} | {w['wood_moved']} | {w['exchange_trips']} | {w['failed']} | "
            f"{t['forage_disputes']} ({t21.get('disputes_uncosted', 0)}) | {t['quarrels']} ({t21.get('quarrels_capped', 0)}, {t21.get('quarrels_while_estranged', 0)}) | "
            f"{t['villages_estranged']} | {t['villages_reconciled']} | {dd['v26']}/{dd['v25']}/{dd['v24']} | {al['v26']}/{al['v25']}/{al['v24']} | {w['pairs_end']} | {w['v25_delivered_trips']} |")
    lines += ["", "## U7 totals (tally / ledger, log or chronicle)", ""]
    for key, (a, b) in u7["totals"].items():
        lines.append(f"- {key}: {a} / {b}")
    lines += [f"- units in flight at day 120: {u7['in_flight_units_end']}",
        f"- worlds whose relations are exactly the living pairs with values in range: {u7['pairs_invariant_worlds']} of {len(report['worlds'])}", "",
        "U5 (identity) and U6 (performance) are judged outside this script.", ""]
    return "\n".join(lines)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("v26", "v25", "v24"):
        parser.add_argument(f"--{name}", type=Path, required=True)
        parser.add_argument(f"--{name}-chronicle", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    parser.add_argument("--seeds", default="19600..19632")
    parser.add_argument("--no-u4", action="store_true", help="skip U4's health half (needs --agent-trace runs)")
    parser.add_argument("--chronicle-json", type=Path, help="evaluate_pass20_chronicle.py output (C1-C4) for U7")
    args = parser.parse_args(argv)
    report = evaluate(args)
    report["schema_version"] = 21
    # U7 needs C1-C4 as well; without the chronicle check it stays PENDING.
    u7 = report["U7"]
    if args.chronicle_json is None:
        u7["chronicle"] = "not evaluated"
        u7["verdict"] = "PENDING" if u7["verdict"] == "PASS" else u7["verdict"]
    else:
        verdict = json.loads(args.chronicle_json.read_text())["verdict"]
        u7["chronicle"] = ", ".join(f"{k} {v}" for k, v in sorted(verdict.items()))
        if any(verdict.get(k) != "PASS" for k in ("C1", "C2", "C3", "C4")):
            u7["verdict"] = "FAIL"
    report["U4_health"] = {} if args.no_u4 else u4_health(args, report["seeds"])
    health_ok = bool(report["U4_health"]) and report["U4_health"]["health_ok"] and report["U4_health"]["family_within_0_05"]
    report["U4_verdict"] = "PASS" if health_ok and report["U4_deaths"]["ok"] else ("SKIPPED" if args.no_u4 else "FAIL")
    args.out_json.write_text(json.dumps(report, indent=1, sort_keys=True, default=str) + "\n")
    text = markdown(report)
    args.out_md.write_text(text)
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
