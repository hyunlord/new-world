#!/usr/bin/env python3
# How to run: python3 tools/revitalize/diag_p21.py --run LABEL=PROBE_DIR,CHRONICLE_DIR [--run ...]
#     --out-md OUT.md --out-json OUT.json [--model-seed 21]
"""Pass 21 diagnostic (brief 7.1) and the relation measures of the grid (7.2).

For every run (a `worldsim_probe --chronicle-out` run of a relations scene),
per world and village pair:

* disputes per pair-day before and while estranged — the real lambda of the
  planner's model (brief 3.3). A pair-day is one display day of an existing
  pair; it counts as estranged when the pair was estranged after that day's
  08:00 decision (`pass20.days`). Disputes come from the measurement-only log
  `relations_<seed>.json` (`[day, a, b, disputes, quarrels]` per display day);
* quarrels, estrangements, reconciliations and days estranged (chronicle);
* the relation at every 08:00 decision (the trajectory);
* the in-reach pair-days meeting the kin-aid and exchange conditions, and
  the trips;
* daughter villages' non-old-age deaths (as Pass 20's T2a).

The planner's model is then replayed with each pair's measured lambda
(Poisson disputes per day, the measured quarrel share, the rule's daily caps,
losses and drift, and contact while estranged scaled by the measured
while/before ratio) and its predicted estrangements, reconciled share and
estranged share at day 120 set beside the measured ones.
"""
from __future__ import annotations

import argparse
import json
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Final

DAY: Final = 1440
OFFSET: Final = 480
OLD_AGE: Final = 3
K_DIED, K_DAUGHTER, K_DISSOLVED, K_QUARREL, K_ESTRANGED, K_RECONCILED = 11, 14, 16, 19, 20, 21
# PairDay columns (sim-test/src/probe/pass20.rs).
TICK, FROM, TO, VALUE, ESTRANGED, REACH = 0, 1, 2, 3, 4, 5
F_DAYS, AID_FOOD, AID_WOOD, EXCHANGE, SENT_KIND = 6, 14, 15, 16, 17
# Trip columns.
T_FROM, T_TO, T_KIND, T_CAUSE, T_DELIVERED, T_FAILED = 1, 2, 3, 4, 6, 9


def display_day(tick: int) -> int:
    return (tick + OFFSET) // DAY


def read_run(probe: Path, chronicle: Path) -> dict[int, dict[str, Any]]:
    summary = json.loads((probe / "summary.json").read_text())
    out: dict[int, dict[str, Any]] = {}
    for run in summary["runs"]:
        seed = run["seed"]
        entries = json.loads((chronicle / f"chronicle_{seed}.json").read_text())["entries"]
        rel_path = chronicle / f"relations_{seed}.json"
        log = json.loads(rel_path.read_text())["dispute_days"] if rel_path.exists() else None
        out[seed] = {"run": run, "entries": entries, "log": log, "scene": summary.get("scene")}
    return out


def daughter_facts(entries: list[list[Any]]) -> tuple[int, int]:
    """(daughter non-old-age deaths, daughters alive at day 118)."""
    daughters = {e[5] for e in entries if e[1] == K_DAUGHTER}
    founded = {e[5]: e[0] for e in entries if e[1] == K_DAUGHTER}
    dissolved = {e[5]: e[0] for e in entries if e[1] == K_DISSOLVED}
    deaths = sum(1 for e in entries if e[1] == K_DIED and e[7] != OLD_AGE and e[5] in daughters)
    tick = 118 * DAY
    alive = sum(1 for d in daughters if founded[d] <= tick and dissolved.get(d, 10**18) > tick)
    return deaths, alive


def world(seed: int, data: dict[str, Any]) -> dict[str, Any]:
    run, entries, log = data["run"], data["entries"], data["log"]
    block = run.get("pass20") or {}
    days = block.get("days", [])
    t = block.get("tallies", {})
    alive = run.get("settlements_alive_daily", [])
    estranged_on: dict[tuple[int, int], dict[int, bool]] = defaultdict(dict)
    trajectory: dict[tuple[int, int], list[list[int]]] = defaultdict(list)
    for d in days:
        if d[FROM] < d[TO]:
            key = (d[FROM], d[TO])
            estranged_on[key][display_day(d[TICK])] = bool(d[ESTRANGED])
            trajectory[key].append([display_day(d[TICK]), d[VALUE]])
    disputes: dict[tuple[int, int], dict[int, tuple[int, int]]] = defaultdict(dict)
    for day, a, b, n, q in log or []:
        disputes[(a, b)][day] = (n, q)
    pairs = []
    for key in sorted(set(estranged_on) | set(disputes)):
        states = estranged_on.get(key, {})
        log_days = disputes.get(key, {})
        before = [day for day, est in states.items() if not est]
        while_ = [day for day, est in states.items() if est]
        n_before = sum(log_days.get(day, (0, 0))[0] for day in before)
        n_while = sum(log_days.get(day, (0, 0))[0] for day in while_)
        q_before = sum(log_days.get(day, (0, 0))[1] for day in before)
        q_while = sum(log_days.get(day, (0, 0))[1] for day in while_)
        unattributed = sum(n for day, (n, _) in log_days.items() if day not in states)
        est_ticks = [e[0] for e in entries if e[1] == K_ESTRANGED and (min(e[5], e[6]), max(e[5], e[6])) == key]
        rec_ticks = [e[0] for e in entries if e[1] == K_RECONCILED and (min(e[5], e[6]), max(e[5], e[6])) == key]
        quarrels = [e[0] for e in entries if e[1] == K_QUARREL and (min(e[5], e[6]), max(e[5], e[6])) == key]
        q_estranged = sum(1 for tick in quarrels if states.get(display_day(tick), False))
        traj = trajectory.get(key, [])
        pairs.append({
            "pair": list(key),
            "pair_days": len(states), "pair_days_before": len(before), "pair_days_while": len(while_),
            "disputes_before": n_before, "disputes_while": n_while, "disputes_unattributed": unattributed,
            "quarrels_before": q_before, "quarrels_while": q_while,
            "lambda_before": n_before / len(before) if before else None,
            "lambda_while": n_while / len(while_) if while_ else None,
            "quarrels": len(quarrels), "quarrels_while_estranged_chronicle": q_estranged,
            "estrangement_ticks": est_ticks, "reconciliation_ticks": rec_ticks,
            "days_estranged": len(while_),
            "estranged_at_end": bool(traj) and states.get(traj[-1][0], False),
            "alive_at_end": bool(traj) and traj[-1][0] >= display_day(run.get("ticks", 120 * DAY) - 1) - 1,
            "base": None,
            "trajectory": traj,
        })
    for row in block.get("pairs_end", []):
        for p in pairs:
            if p["pair"] == [row[0], row[1]]:
                p["base"] = row[4]
    trips = block.get("trips", [])
    reach = [d for d in days if d[REACH]]
    deaths, alive_d = daughter_facts(entries)
    # Pass 21's tallies live in the block's `pass21` key (capped rules only).
    t = {**t, **((block.get("pass21") or {}).get("tallies") or {})}
    return {
        "seed": seed,
        "max_alive": max(alive) if alive else 0,
        "multi_village": bool(alive) and max(alive) >= 2,
        "tallies": t, "pass21": block.get("pass21"),
        "pairs": pairs,
        "pairs_end": block.get("pairs_end", []),
        "aid_food_in_reach": sum(d[AID_FOOD] for d in reach),
        "aid_wood_in_reach": sum(d[AID_WOOD] for d in reach),
        "exchange_in_reach": sum(d[EXCHANGE] for d in reach),
        "aid_food_in_reach_not_estranged": sum(d[AID_FOOD] for d in reach if not d[ESTRANGED]),
        "exchange_in_reach_not_estranged": sum(d[EXCHANGE] for d in reach if not d[ESTRANGED]),
        "pair_days_sent": sum(1 for d in days if d[SENT_KIND] >= 0),
        "trips": len(trips),
        "delivered_trips": len({(r[T_FROM], r[T_TO], r[10]) for r in trips if r[T_DELIVERED] > 0}),
        "exchange_porters": sum(1 for r in trips if r[T_CAUSE] in (2, 3)),
        "failed_porters": sum(r[T_FAILED] for r in trips),
        "daughter_deaths": deaths, "daughters_alive_d118": alive_d,
    }


def poisson(rng: random.Random, lam: float) -> int:
    if lam <= 0:
        return 0
    limit, k, p = math.exp(-lam), 0, 1.0
    while True:
        p *= rng.random()
        if p <= limit:
            return k
        k += 1


def model(rule: dict[str, int], lam: float, contact: float, quarrel_share: float, base: int, days: int,
          rng: random.Random, trials: int = 400) -> dict[str, float]:
    """The planner's model of brief 3.3 with integer drift, caps and hysteresis."""
    estrangements = reconciled = estranged_end = 0
    for _ in range(trials):
        value, estranged = base, False
        for _day in range(days):
            value += int((base - value) * rule["drift"] / 1000)
            if not estranged and value <= rule["estrange_at"]:
                estranged, estrangements = True, estrangements + 1
            elif estranged and value >= rule["reconcile_at"]:
                estranged, reconciled = False, reconciled + 1
            n = poisson(rng, lam * (contact if estranged else 1.0))
            costed = n if rule["dispute_cap"] == 0 else min(n, rule["dispute_cap"])
            quarrels = sum(1 for _ in range(n) if rng.random() < quarrel_share)
            if rule["quarrel_cap"]:
                quarrels = min(quarrels, rule["quarrel_cap"])
            value = max(-1000, min(1000, value - costed * rule["dispute_loss"] - quarrels * rule["quarrel_loss"]))
        estranged_end += estranged
    return {"estrangements_per_pair": estrangements / trials,
            "reconciled_share": reconciled / estrangements if estrangements else None,
            "estranged_at_end": estranged_end / trials}


def rule_of(data: dict[int, dict[str, Any]], override: dict[str, int] | None) -> dict[str, int]:
    scene = next(iter(data.values()))["scene"]
    v25 = {"drift": 10, "dispute_loss": 10, "quarrel_loss": 100, "dispute_cap": 0, "quarrel_cap": 0,
           "estrange_at": -300, "reconcile_at": -150}
    v26 = {"drift": 30, "dispute_loss": 5, "quarrel_loss": 40, "dispute_cap": 3, "quarrel_cap": 1,
           "estrange_at": -300, "reconcile_at": -150}
    rule = dict(v25 if scene == "game_v25" else v26)
    rule.update(override or {})
    return rule


def summarize(label: str, data: dict[int, dict[str, Any]], rule: dict[str, int], rng: random.Random,
              days: int) -> dict[str, Any]:
    worlds = [world(seed, d) for seed, d in sorted(data.items())]
    multi = [w for w in worlds if w["multi_village"]]
    pairs = [p for w in worlds for p in w["pairs"]]
    lam_before = [p["lambda_before"] for p in pairs if p["lambda_before"] is not None and p["pair_days_before"] >= 3]
    n_before = sum(p["disputes_before"] for p in pairs)
    d_before = sum(p["pair_days_before"] for p in pairs)
    n_while = sum(p["disputes_while"] for p in pairs)
    d_while = sum(p["pair_days_while"] for p in pairs)
    disputes = sum(p["disputes_before"] + p["disputes_while"] for p in pairs)
    quarrels = sum(p["quarrels_before"] + p["quarrels_while"] for p in pairs)
    contact = (n_while / d_while) / (n_before / d_before) if d_while and d_before and n_before else None
    q_share = quarrels / disputes if disputes else 0.0
    end_pairs = [p for w in worlds for p in w["pairs_end"]]
    estr = sum(w["tallies"].get("villages_estranged", 0) for w in worlds)
    recs = sum(w["tallies"].get("villages_reconciled", 0) for w in worlds)
    q_all = sum(w["tallies"].get("quarrels", 0) for w in worlds)
    # Under a cap the pass21 tally; otherwise from the chronicle and the 08:00 states.
    q_est = sum(w["tallies"].get("quarrels_while_estranged", 0) for w in worlds) if all(
        "quarrels_while_estranged" in w["tallies"] for w in worlds) else sum(
        p["quarrels_while_estranged_chronicle"] for p in pairs)
    q_est_chronicle = sum(p["quarrels_while_estranged_chronicle"] for p in pairs)
    # The planner's model at each pair's measured lambda (pairs with >= 10
    # pair-days). Contact while estranged: the pair's own while/before ratio
    # when it was estranged on >= 5 pair-days and disputed before, else the
    # ratio pooled over the pairs that were ever estranged, else the
    # planner's 5 %.
    ever = [p for p in pairs if p["pair_days_while"] >= 5 and p["disputes_before"] > 0]
    pooled_contact = ((sum(p["disputes_while"] for p in ever) / sum(p["pair_days_while"] for p in ever))
                      / (sum(p["disputes_before"] for p in ever) / sum(p["pair_days_before"] for p in ever))) if ever else None
    predicted = []
    for w in worlds:
        for p in w["pairs"]:
            if p["pair_days"] < 10 or p["lambda_before"] is None:
                continue
            own = (p["lambda_while"] / p["lambda_before"]) if p in ever and p["lambda_while"] is not None else None
            pair_contact = own if own is not None else (pooled_contact if pooled_contact is not None else 0.05)
            m = model(rule, p["lambda_before"], pair_contact, q_share,
                      p["base"] if p["base"] is not None else 300, p["pair_days"], rng)
            m["contact"] = pair_contact
            predicted.append({"seed": w["seed"], "pair": p["pair"], "lambda_before": p["lambda_before"],
                "pair_days": p["pair_days"], "measured_estrangements": len(p["estrangement_ticks"]),
                "measured_reconciliations": len(p["reconciliation_ticks"]),
                "measured_estranged_at_end": p["estranged_at_end"], **{f"model_{k}": v for k, v in m.items()}})
    def mean(values: list[float]) -> float | None:
        return sum(values) / len(values) if values else None
    return {
        "label": label, "scene": next(iter(data.values()))["scene"], "rule": rule, "worlds": worlds,
        "multi_village_worlds": len(multi),
        "share_delivering": (sum(1 for w in multi if w["delivered_trips"] > 0) / len(multi)) if multi else None,
        "share_dispute": (sum(1 for w in multi if w["tallies"].get("forage_disputes", 0) > 0) / len(multi)) if multi else None,
        "share_quarrel": (sum(1 for w in multi if w["tallies"].get("quarrels", 0) > 0) / len(multi)) if multi else None,
        "pairs_at_end": len(end_pairs),
        "estranged_at_end": sum(p[3] for p in end_pairs),
        "estranged_share_at_end": (sum(p[3] for p in end_pairs) / len(end_pairs)) if end_pairs else None,
        "estrangements": estr, "reconciliations": recs,
        "reconciled_share": recs / estr if estr else None,
        "quarrels": q_all, "quarrels_while_estranged": q_est, "quarrels_while_estranged_by_day_state": q_est_chronicle,
        "quarrels_while_estranged_share": q_est / q_all if q_all else None,
        "disputes": sum(w["tallies"].get("forage_disputes", 0) for w in worlds),
        "disputes_uncosted": sum(w["tallies"].get("disputes_uncosted", 0) for w in worlds),
        "quarrels_capped": sum(w["tallies"].get("quarrels_capped", 0) for w in worlds),
        "lambda_before_pooled": n_before / d_before if d_before else None,
        "lambda_while_pooled": n_while / d_while if d_while else None,
        "contact_while_over_before": contact,
        "contact_while_over_before_estranged_pairs": pooled_contact,
        "quarrel_share": q_share,
        "lambda_before_by_pair": sorted(lam_before),
        "pair_days_before": d_before, "pair_days_while": d_while,
        "days_estranged": sum(p["days_estranged"] for p in pairs),
        "aid_food_in_reach": sum(w["aid_food_in_reach"] for w in worlds),
        "aid_food_in_reach_not_estranged": sum(w["aid_food_in_reach_not_estranged"] for w in worlds),
        "exchange_in_reach": sum(w["exchange_in_reach"] for w in worlds),
        "exchange_in_reach_not_estranged": sum(w["exchange_in_reach_not_estranged"] for w in worlds),
        "trips": sum(w["trips"] for w in worlds),
        "delivered_trips": sum(w["delivered_trips"] for w in worlds),
        "exchange_porters": sum(w["exchange_porters"] for w in worlds),
        "failed_porters": sum(w["failed_porters"] for w in worlds),
        "daughter_deaths": sum(w["daughter_deaths"] for w in worlds),
        "daughters_alive_d118": sum(w["daughters_alive_d118"] for w in worlds),
        "model": {"pairs": predicted,
            "mean_model_estrangements": mean([p["model_estrangements_per_pair"] for p in predicted]),
            "mean_measured_estrangements": mean([float(p["measured_estrangements"]) for p in predicted]),
            "mean_model_estranged_at_end": mean([p["model_estranged_at_end"] for p in predicted]),
            "measured_estranged_at_end": mean([float(p["measured_estranged_at_end"]) for p in predicted])},
        "days": days,
    }


def num(value: float | None, digits: int = 3) -> str:
    return "n/a" if value is None else f"{value:.{digits}f}"


def markdown(reports: list[dict[str, Any]]) -> str:
    lines = ["# Pass 21 diagnostic (7.1)", "",
        "Per run: seeds and days of the probe, multi-village worlds (≥ 2 villages alive at once in that run), "
        "disputes per pair-day before and while estranged (lambda, from the measurement-only dispute log; a pair-day is "
        "estranged when the pair was estranged after that day's 08:00 decision), quarrels, estrangements and reconciliations, "
        "aid / exchange conditions in reach, trips and daughter deaths. The model columns replay the planner's model "
        "(brief 3.3) at each pair's measured lambda with the run's rule.", "",
        "| run | multi-village | delivering share | dispute share | quarrel share | lambda before | lambda while | contact while/before | quarrel share of disputes | estrangements | reconciled | estranged at d120 | quarrels while estranged | uncosted disputes | capped quarrels | aid-food pair-days (not estranged) | exchange pair-days (not estranged) | trips (delivered) | exchange porters | daughter deaths | daughters alive d118 |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in reports:
        lines.append(f"| {r['label']} ({r['scene']}) | {r['multi_village_worlds']} of {len(r['worlds'])} | {num(r['share_delivering'])} | "
            f"{num(r['share_dispute'])} | {num(r['share_quarrel'])} | {num(r['lambda_before_pooled'])} ({r['pair_days_before']} pair-days) | "
            f"{num(r['lambda_while_pooled'])} ({r['pair_days_while']}) | {num(r['contact_while_over_before'])} (estranged pairs: {num(r['contact_while_over_before_estranged_pairs'])}) | {num(r['quarrel_share'])} | "
            f"{r['estrangements']} | {r['reconciliations']} ({num(r['reconciled_share'])}) | {r['estranged_at_end']} of {r['pairs_at_end']} ({num(r['estranged_share_at_end'])}) | "
            f"{r['quarrels_while_estranged']} of {r['quarrels']} ({num(r['quarrels_while_estranged_share'])}) | {r['disputes_uncosted']} of {r['disputes']} | {r['quarrels_capped']} | "
            f"{r['aid_food_in_reach']} ({r['aid_food_in_reach_not_estranged']}) | {r['exchange_in_reach']} ({r['exchange_in_reach_not_estranged']}) | "
            f"{r['trips']} ({r['delivered_trips']}) | {r['exchange_porters']} | {r['daughter_deaths']} | {r['daughters_alive_d118']} |")
    for r in reports:
        lines += ["", f"## {r['label']} ({r['scene']}): worlds and pairs", "",
            f"Lambda before estrangement by pair (≥ 3 pair-days, sorted): {[round(x, 2) for x in r['lambda_before_by_pair']]}", "",
            "| seed | villages | pair | base | pair-days (before / while) | disputes before / while | lambda before / while | quarrels (while estranged) | estranged at (day) | reconciled at (day) | estranged at end | relation every 10 days |",
            "|---|---|---|---|---|---|---|---|---|---|---|---|"]
        for w in r["worlds"]:
            if not w["pairs"]:
                lines.append(f"| {w['seed']} | {w['max_alive']} | – | | | | | | | | | |")
            for p in w["pairs"]:
                traj = " ".join(str(v) for d, v in p["trajectory"] if d % 10 == 0) or "–"
                lines.append(f"| {w['seed']} | {w['max_alive']} | {p['pair'][0]}–{p['pair'][1]} | {p['base']} | {p['pair_days_before']} / {p['pair_days_while']} | "
                    f"{p['disputes_before']} / {p['disputes_while']} | {num(p['lambda_before'], 2)} / {num(p['lambda_while'], 2)} | "
                    f"{p['quarrels']} ({p['quarrels_while_estranged_chronicle']}) | {[display_day(t) for t in p['estrangement_ticks']]} | "
                    f"{[display_day(t) for t in p['reconciliation_ticks']]} | {p['estranged_at_end']} | {traj} |")
        m = r["model"]
        lines += ["", f"Model check ({len(m['pairs'])} pairs with ≥ 10 pair-days; pairs without a dispute and without an estrangement are omitted from the table, the model predicts nothing for them either): estrangements per pair model {num(m['mean_model_estrangements'])} "
            f"vs measured {num(m['mean_measured_estrangements'])}; estranged at the end model {num(m['mean_model_estranged_at_end'])} vs measured "
            f"{num(m['measured_estranged_at_end'])}.", "",
            "| seed | pair | lambda before | contact while estranged (model input) | pair-days | estrangements measured / model | reconciled measured / model share | estranged at end measured / model |",
            "|---|---|---|---|---|---|---|---|"]
        for p in m["pairs"]:
            if p["lambda_before"] == 0 and p["measured_estrangements"] == 0:
                continue
            lines.append(f"| {p['seed']} | {p['pair'][0]}–{p['pair'][1]} | {p['lambda_before']:.2f} | {p['model_contact']:.2f} | {p['pair_days']} | {p['measured_estrangements']} / {p['model_estrangements_per_pair']:.2f} | "
                f"{p['measured_reconciliations']} / {num(p['model_reconciled_share'], 2)} | {int(p['measured_estranged_at_end'])} / {p['model_estranged_at_end']:.2f} |")
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="append", required=True, help="LABEL=PROBE_DIR,CHRONICLE_DIR[,quarrel_loss,drift]")
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    parser.add_argument("--model-seed", type=int, default=21)
    parser.add_argument("--days", type=int, default=120)
    args = parser.parse_args(argv)
    rng = random.Random(args.model_seed)
    reports = []
    for spec in args.run:
        label, paths = spec.split("=", 1)
        parts = paths.split(",")
        override = {"quarrel_loss": int(parts[2]), "drift": int(parts[3])} if len(parts) >= 4 else None
        data = read_run(Path(parts[0]), Path(parts[1]))
        reports.append(summarize(label, data, rule_of(data, override), rng, args.days))
    args.out_json.write_text(json.dumps({"runs": reports}, indent=1, sort_keys=True) + "\n")
    text = markdown(reports)
    args.out_md.write_text(text)
    print(text[:4000])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
