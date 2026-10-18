#!/usr/bin/env python3
# How to run: python3 tools/revitalize/diag_p20.py --probe PROBE_DIR --out-md OUT.md --out-json OUT.json
"""Pass 20 diagnostic (brief 7.1): per world of a `game_v25` probe run, the
08:00 pair-days meeting the kin-aid and exchange conditions, the in-reach
pair-days, the trips, the cross-village forage disputes, the quarrels and the
relation trajectories, read from the summary's per-seed `pass20` block.

A pair-day is one ordered pair (sender, receiver) at one decision minute
(`pass20.days`; columns in `sim-test/src/probe/pass20.rs`). "Reachable"
summarises whether T1 (>= 0.6 of multi-village worlds deliver) and T3
(>= 0.3 dispute, >= 0.15 quarrel) are within reach of these 60-day worlds.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

# PairDay columns.
TICK, FROM, TO, VALUE, ESTRANGED, REACH = 0, 1, 2, 3, 4, 5
F_DAYS, F_FOOD, F_NIGHTS, F_WOOD, T_DAYS, T_FOOD, T_NIGHTS, T_WOOD = range(6, 14)
AID_FOOD, AID_WOOD, EXCHANGE, SENT_KIND, SENT_AMOUNT = 14, 15, 16, 17, 18


def world(run: dict[str, Any]) -> dict[str, Any]:
    block = run.get("pass20") or {}
    days = block.get("days", [])
    t = block.get("tallies", {})
    alive = run.get("settlements_alive_daily", [])
    pairs: dict[str, list[int]] = {}
    for d in days:
        if d[FROM] < d[TO]:
            pairs.setdefault(f"{d[FROM]}-{d[TO]}", []).append(d[VALUE])
    reach = [d for d in days if d[REACH]]
    return {
        "seed": run["seed"],
        "max_settlements_alive": max(alive) if alive else 0,
        "multi_village": bool(alive) and max(alive) >= 2,
        "pair_days": len(days),
        "in_reach": len(reach),
        "aid_food": sum(d[AID_FOOD] for d in days),
        "aid_wood": sum(d[AID_WOOD] for d in days),
        "exchange": sum(d[EXCHANGE] for d in days),
        "aid_food_in_reach": sum(d[AID_FOOD] for d in reach),
        "aid_wood_in_reach": sum(d[AID_WOOD] for d in reach),
        "exchange_in_reach": sum(d[EXCHANGE] for d in reach),
        "sent": sum(1 for d in days if d[SENT_KIND] >= 0),
        "giver_food_days_max": max((d[F_DAYS] for d in days), default=None),
        "receiver_food_days_min": min((d[T_DAYS] for d in days), default=None),
        "wood_nights_max": max((d[F_NIGHTS] for d in days), default=None),
        "wood_nights_min": min((d[F_NIGHTS] for d in days), default=None),
        "food_days_quantiles": quantiles([d[F_DAYS] for d in days]),
        "wood_nights_quantiles": quantiles([d[F_NIGHTS] for d in days]),
        "trips": t.get("porter_trips", 0),
        "porters": t.get("porters_departed", 0),
        "deliveries": t.get("porter_deliveries", 0),
        "turnbacks": t.get("porter_turnbacks", 0),
        "disputes": t.get("forage_disputes", 0),
        "quarrels": t.get("quarrels", 0),
        "estranged": t.get("villages_estranged", 0),
        "reconciled": t.get("villages_reconciled", 0),
        "trajectories": {k: [v[0], min(v), max(v), v[-1]] for k, v in pairs.items()},
    }


def quantiles(values: list[int]) -> list[int] | None:
    if not values:
        return None
    s = sorted(values)
    return [s[int((len(s) - 1) * q)] for q in (0.0, 0.1, 0.5, 0.9, 1.0)]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    summary = json.loads((args.probe / "summary.json").read_text())
    worlds = [world(run) for run in summary["runs"]]
    multi = [w for w in worlds if w["multi_village"]]
    share = lambda key: (sum(1 for w in multi if w[key] > 0) / len(multi)) if multi else None
    totals = {key: sum(w[key] for w in worlds) for key in ("pair_days", "in_reach", "aid_food", "aid_wood", "exchange",
        "aid_food_in_reach", "aid_wood_in_reach", "exchange_in_reach", "sent", "trips", "porters", "deliveries",
        "turnbacks", "disputes", "quarrels", "estranged", "reconciled")}
    reach = {"multi_village_worlds": len(multi), "worlds": len(worlds),
        "share_with_delivery": share("deliveries"), "share_with_dispute": share("disputes"), "share_with_quarrel": share("quarrels"),
        "T1_within_reach": (share("deliveries") or 0) >= 0.6, "T3_within_reach": (share("disputes") or 0) >= 0.3 and (share("quarrels") or 0) >= 0.15}
    days = max((len(run.get("settlements_alive_daily", [])) for run in summary["runs"]), default=0)
    out = {"scene": summary.get("scene"), "days": days, "worlds": worlds, "totals": totals, "reach": reach}
    args.out_json.write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    lines = [f"# Pass 20 diagnostic (7.1): {summary.get('scene')}, {len(worlds)} worlds, {days} days", "",
        "Pair-days are ordered (sender, receiver) pairs at the 08:00 decision. Conditions are counted before reach, "
        "estrangement and porter slots; `in reach` = within three quarters of the 08:00–15:00 window.", "",
        "| seed | villages (max alive) | pair-days | in reach | aid food | aid wood | exchange | sent | porters | delivered | turned back | disputes | quarrels | estranged | reconciled | relation (first, min, max, last) |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for w in worlds:
        traj = "; ".join(f"{k}: {v}" for k, v in w["trajectories"].items()) or "–"
        lines.append(f"| {w['seed']} | {w['max_settlements_alive']} | {w['pair_days']} | {w['in_reach']} | {w['aid_food']} | {w['aid_wood']} | "
            f"{w['exchange']} | {w['sent']} | {w['porters']} | {w['deliveries']} | {w['turnbacks']} | {w['disputes']} | {w['quarrels']} | "
            f"{w['estranged']} | {w['reconciled']} | {traj} |")
    lines += ["", "Totals: " + ", ".join(f"{k} {v}" for k, v in totals.items()), "",
        "Measures over all pair-days (sender side; quantiles 0/10/50/90/100 %): food days (permille) "
        + "; ".join(f"{w['seed']}: {w['food_days_quantiles']}" for w in worlds if w['food_days_quantiles'])
        + " — wood nights " + "; ".join(f"{w['seed']}: {w['wood_nights_quantiles']}" for w in worlds if w['wood_nights_quantiles']), "",
        f"Multi-village worlds: {len(multi)} of {len(worlds)}. Share with ≥ 1 delivery {reach['share_with_delivery']}, "
        f"with ≥ 1 dispute {reach['share_with_dispute']}, with ≥ 1 quarrel {reach['share_with_quarrel']}. "
        f"T1 within reach: {reach['T1_within_reach']}; T3 within reach: {reach['T3_within_reach']}.", ""]
    args.out_md.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
