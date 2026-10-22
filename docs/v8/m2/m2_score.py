#!/usr/bin/env python3
"""M2 evaluation scorer (goal §10, E1–E7; E8–E10 come from the benchmark, identity and replay).

usage: m2_score.py <m2u_summary.json> <no_orders_summary.json> [--out appendix.json]

Reads two `worldsim_probe --scene colony` summaries: the M2-U script run (20 days) and the
no-orders run (3 colony years = 72 days). Per seed it computes every bar exactly as
registered in M2_EVALUATION.md §1, prints the bar table and writes a per-seed appendix.
"""
import json
import sys

DAY = 1440


def deaths(r, causes):
    return sum(r["deaths"].get(c, 0) for c in causes)


def score_seed(r):
    m = r["m2"]
    out = {"seed": r["seed"]}
    # E1: 6 blades made within 10 days.
    b6 = m.get("blades6_tick")
    out["e1_blades6_day"] = None if b6 is None else round(b6 / DAY, 2)
    out["E1"] = b6 is not None and b6 <= 10 * DAY
    # E2: 2 axes within 15 days, and at least one chop afterwards used an axe.
    a2 = m.get("axes2_tick")
    out["e2_axes2_day"] = None if a2 is None else round(a2 / DAY, 2)
    out["e2_chops_with_axe"] = m["chops_with_axe"]
    out["E2_axes"] = a2 is not None and a2 <= 15 * DAY
    out["E2"] = out["E2_axes"] and m["chops_with_axe"] >= 1
    # E3: firewood items in stock (on stockpile cells) >= 20 on every day from day 8.
    stock = m["firewood_stock_by_day"]  # index i = stock at the start of day i + 1
    from_day8 = stock[7:]
    out["e3_min_from_day8"] = min(from_day8) if from_day8 else None
    out["E3"] = bool(from_day8) and min(from_day8) >= 20
    # E4: the Chop = 0 adult took 0 chop jobs; the Craft = 1 adult made >= 50 % of the
    # crafts of work type Craft (knap blade, haft axe).
    allc = m["craft_crafts_all"]
    mine = m["craft1_craft_crafts"]
    out["e4_chop0_chops"] = m["chop0_chop_jobs"]
    out["e4_craft1_share"] = None if allc == 0 else round(mine / allc, 3)
    out["E4_chop0"] = m["chop0_agent"] is not None and m["chop0_chop_jobs"] == 0
    out["E4_craft1"] = allc > 0 and 2 * mine >= allc
    out["E4"] = out["E4_chop0"] and out["E4_craft1"]
    # E5: per-kind ledger exact every tick; reservations invariant.
    out["E5"] = r["ledger_violation_ticks"] == 0 and r["reservation_violation_ticks"] == 0
    # E6 (first half): no hunger, thirst or cold death in the 20-day M2-U run.
    out["e6_deaths_20d"] = deaths(r, ["starvation", "dehydration", "exposure"])
    out["E6_m2u"] = out["e6_deaths_20d"] == 0
    # E7: the Craft = 1 adult's blade quality at day 20 minus at day 3. The registered measure
    # is the mean of 200 keyed draws of the quality formula at the adult's Knap level on each
    # day (same draws, so the difference is 3 x the level difference; DECISIONS D18), plus the
    # actual blades for reference.
    snaps = {s["day"]: s for s in m["snapshots"]}
    l3 = snaps.get(3, {}).get("craft1_knap_level")
    l20 = snaps.get(20, {}).get("craft1_knap_level")
    out["e7_level_day3"] = l3
    out["e7_level_day20"] = l20
    out["e7_expected_gain"] = None if l3 is None or l20 is None else 3 * (l20 - l3)
    n3, s3 = snaps.get(3, {}).get("craft1_blades", 0), snaps.get(3, {}).get("craft1_blade_quality_sum", 0)
    n20, s20 = snaps.get(20, {}).get("craft1_blades", 0), snaps.get(20, {}).get("craft1_blade_quality_sum", 0)
    out["e7_blades_by_day3"] = n3
    out["e7_blades_days3_20"] = n20 - n3
    out["e7_actual_mean_by_day3"] = None if n3 == 0 else round(s3 / n3, 1)
    out["e7_actual_mean_days3_20"] = None if n20 == n3 else round((s20 - s3) / (n20 - n3), 1)
    out["E7"] = out["e7_expected_gain"] is not None and out["e7_expected_gain"] >= 8
    return out


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    m2u = json.load(open(argv[1]))["seeds"]
    no = {r["seed"]: r for r in json.load(open(argv[2]))["seeds"]}
    rows = [score_seed(r) for r in m2u]
    for row in rows:
        n = no.get(row["seed"])
        if n is None:
            row["E6_noorders"] = False
            continue
        row["e6_3y_cold_deaths"] = n["deaths"].get("exposure", 0)
        row["e6_3y_deaths"] = n["deaths"]
        row["e6_3y_alive"] = n["alive_end"]
        row["E6_noorders"] = row["e6_3y_cold_deaths"] == 0
    total = len(rows)

    def count(key):
        return sum(1 for r in rows if r.get(key))

    bars = [
        ("E1", "6 blades made within 10 days", count("E1"), 28),
        ("E2", "2 axes within 15 days and a chop with an axe afterwards", count("E2"), 28),
        ("E2a", "(part) 2 axes within 15 days", count("E2_axes"), None),
        ("E3", "firewood in stock >= 20 every day from day 8", count("E3"), 28),
        ("E4", "Chop=0 adult 0 chops and Craft=1 adult >= 50 % of crafts", count("E4"), total),
        ("E4a", "(part) Chop=0 adult took 0 chop jobs", count("E4_chop0"), None),
        ("E4b", "(part) Craft=1 adult >= 50 % of knap/haft crafts", count("E4_craft1"), None),
        ("E5", "per-kind ledger exact every tick; reservations", count("E5"), total),
        ("E6a", "no hunger/thirst/cold death in 20 days (M2-U)", count("E6_m2u"), total),
        ("E6b", "3 years without orders: 0 cold deaths", count("E6_noorders"), 30),
        ("E7", "Craft=1 adult's blade quality day 20 - day 3 >= 8", count("E7"), 24),
    ]
    print(f"| # | bar | result | pass at | verdict |\n|---|---|---|---|---|")
    verdicts = {}
    for key, text, got, need in bars:
        if need is None:
            print(f"| {key} | {text} | {got}/{total} | — | (part) |")
            continue
        ok = got >= need
        verdicts[key] = ok
        print(f"| {key} | {text} | {got}/{total} | ≥ {need}/{total} | {'PASS' if ok else 'FAIL'} |")
    out = {"bars": {k: v for k, v in verdicts.items()}, "seeds": rows}
    if "--out" in argv:
        path = argv[argv.index("--out") + 1]
        with open(path, "w") as f:
            json.dump(out, f, indent=1, sort_keys=True)
        print(f"appendix: {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
