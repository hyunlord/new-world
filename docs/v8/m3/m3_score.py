#!/usr/bin/env python3
"""M3 evaluation scorer (goal §10, F1–F5, F7–F9; F6 and F10 come from nav-bench, the gate's bench,
identity and replay).

usage: m3_score.py <m3u_summary.json> <no_orders_summary.json> [--out appendix.json]

Reads two `worldsim_probe --scene colony` summaries: the M3-U script run (20 days) and the
no-orders run (3 colony years = 72 days). Per seed it computes every bar exactly as registered
in M3_DECISIONS.md D15, prints the bar table and writes a per-seed appendix.
"""
import json
import sys

DAY = 1440


def day_of(tick):
    return None if tick is None else round(tick / DAY, 2)


def score_seed(r):
    m = r["m3"]
    out = {"seed": r["seed"]}
    # F1: a roofed bedroom with >= 4 beds by day 8.
    out["f1_day"] = day_of(m["bedroom4_tick"])
    out["F1"] = m["bedroom4_tick"] is not None and m["bedroom4_tick"] < 8 * DAY
    # F2: a roofed storehouse by day 10 and >= 60 % of the stockpiled items in one at day 20.
    out["f2_day"] = day_of(m["storehouse_tick"])
    out["f2_share"] = round(m["stock_share_end"], 3)
    out["f2_stock"] = m["stock_total_end"]
    out["F2"] = (
        m["storehouse_tick"] is not None
        and m["storehouse_tick"] < 10 * DAY
        and m["stock_total_end"] > 0
        and m["stock_share_end"] >= 0.6
    )
    # F3 (only seeds whose hill_pocket found a pocket): dug by day 14 and a dug room by day 14.
    out["f3_has_hill"] = bool(m["dig_ordered"])
    out["f3_dug_day"] = day_of(m["dug_all_tick"])
    out["f3_room_day"] = day_of(m["dug_room_tick"])
    out["F3"] = (
        None
        if not m["dig_ordered"]
        else (
            m["dug_all_tick"] is not None
            and m["dug_all_tick"] < 14 * DAY
            and m["dug_room_tick"] is not None
            and m["dug_room_tick"] < 14 * DAY
        )
    )
    # F4: >= 3 distinct sleepers in a roofed bedroom on every night from day 9 (index 8 on).
    nights = m["bedroom_sleepers"][8:]
    out["f4_min_from_day9"] = min(nights) if nights else None
    out["F4"] = bool(nights) and min(nights) >= 3
    # F5: no Unreachable cancel into a room with a door; a door was walked through.
    out["f5_unreachable"] = m["unreachable"]
    out["f5_unreachable_room_door"] = m["unreachable_room_door"]
    out["f5_door_steps"] = m["door_steps"]
    out["F5"] = m["unreachable_room_door"] == 0 and m["door_steps"] >= 1
    # F7: top knapper level >= 5 at day 20, and 3 x (mean level days 15-20 - days 1-5) >= 8
    # (M2 D18's measure); the actual blades' mean qualities beside it.
    levels = m["knap_top_level_by_day"]  # index i = end of day i + 1
    early = levels[0:5]
    late = levels[14:20]
    out["f7_level_day20"] = levels[19] if len(levels) >= 20 else m["knap_top_level"]
    out["f7_quality_gain"] = (
        None
        if not early or not late
        else round(3 * (sum(late) / len(late) - sum(early) / len(early)), 2)
    )
    out["f7_blades_early"] = m["blades_early"]
    out["f7_blades_late"] = m["blades_late"]
    out["f7_blade_quality_early"] = m["blade_quality_early"]
    out["f7_blade_quality_late"] = m["blade_quality_late"]
    out["F7"] = (
        out["f7_level_day20"] is not None
        and out["f7_level_day20"] >= 5
        and out["f7_quality_gain"] is not None
        and out["f7_quality_gain"] >= 8
    )
    # F8: >= 10 chops with an axe after day 3.
    out["f8_axe_chops"] = m["axe_chops_after_day3"]
    out["F8"] = m["axe_chops_after_day3"] >= 10
    # F9 (20-day part): ledger and reservations exact every tick, no death.
    out["f9_deaths"] = sum(r["deaths"].values())
    out["F9_run"] = (
        r["ledger_violation_ticks"] == 0
        and r["reservation_violation_ticks"] == 0
        and out["f9_deaths"] == 0
    )
    out["rooms_end"] = m["rooms_end"]
    return out


def main():
    args = sys.argv[1:]
    if len(args) < 2:
        print(__doc__)
        return 2
    out_path = None
    if "--out" in args:
        out_path = args[args.index("--out") + 1]
    m3u = json.load(open(args[0]))["seeds"]
    no_orders = json.load(open(args[1]))["seeds"]
    rows = [score_seed(r) for r in m3u]
    n = len(rows)
    count = lambda k: sum(1 for r in rows if r[k])
    hill = [r for r in rows if r["F3"] is not None]
    f3 = sum(1 for r in hill if r["F3"])
    cold = {r["seed"]: r["deaths"].get("exposure", 0) for r in no_orders}
    f9_cold = sum(1 for v in cold.values() if v == 0)
    table = [
        ("F1 bedroom with 4 beds by day 8", count("F1"), n, ">= 28/32", count("F1") >= 28),
        ("F2 storehouse by day 10, >= 60 % stocked inside at day 20", count("F2"), n, ">= 28/32", count("F2") >= 28),
        ("F3 pocket dug and a dug room by day 14 (seeds with a hill)", f3, len(hill), ">= 90 %", len(hill) > 0 and f3 * 10 >= 9 * len(hill)),
        ("F4 >= 3 bedroom sleepers every night from day 9", count("F4"), n, ">= 28/32", count("F4") >= 28),
        ("F5 0 unreachable into door rooms, a door used", count("F5"), n, "32/32", count("F5") == n),
        ("F7 knap level >= 5 and quality gain >= 8", count("F7"), n, ">= 24/32", count("F7") >= 24),
        ("F8 >= 10 axe chops after day 3", count("F8"), n, ">= 28/32", count("F8") >= 28),
        ("F9 ledger, reservations, 0 deaths in 20 days", count("F9_run"), n, "32/32", count("F9_run") == n),
        ("F9 3-year no-orders: 0 exposure deaths", f9_cold, len(cold), ">= 30/32", f9_cold >= 30),
    ]
    print(f"{'bar':62} {'value':>8}  {'need':>9}  verdict")
    for name, got, of, need, ok in table:
        print(f"{name:62} {got:>3}/{of:<4}  {need:>9}  {'PASS' if ok else 'FAIL'}")
    if out_path:
        with open(out_path, "w") as f:
            json.dump({"bars": [list(t) for t in table], "seeds": rows, "no_orders_exposure": cold}, f, indent=1)
    return 0


if __name__ == "__main__":
    sys.exit(main())
