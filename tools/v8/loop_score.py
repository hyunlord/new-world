#!/usr/bin/env python3
"""M-LOOP steward evaluation scorer (brief §3, owner decisions LOOP-D-steward and LOOP-D-teach).

usage: loop_score.py BASE_SUMMARY CAND_SUMMARY [--min-frac 30/32]

Both summaries come from the colony probe with --steward on --mux2-observe, run on the same seeds.
The M-UX2 rows are judged BASE-relative: the candidate's count is at least BASE − 1 world.
The log rows are judged against the candidate's own cap target. KnowledgeLost must be within
BASE + 1 in each world for the required fraction of worlds, and the total within BASE + 3.
Exits 0 when every bar passes and 1 otherwise. Prints one row per bar.
"""
import json
import sys

DAY = 1440


def seeds(path):
    return {s["seed"]: s for s in json.load(open(path))["seeds"]}


def rows(s):
    mux, mux2 = s.get("mux", {}), s.get("mux2", {})
    beds = mux2.get("walled_beds_for_all_first_tick", mux.get("walled_beds_for_all_first_tick"))
    caps = s.get("cap_target_by_day") or []
    logs = s.get("logs_ground_by_day") or []
    snap = mux.get("cold_snap_observed_ticks", 0) or 0
    return {
        "stockpile_day1": (mux.get("stockpile_first_tick") is not None and mux["stockpile_first_tick"] <= DAY),
        "walled_beds_day10": beds is not None and beds <= 10 * DAY,
        "zero_cold_deaths": s["deaths"].get("exposure", 0) == 0,
        "alive_day48": (mux.get("alive_at_day48") or s.get("alive_end", 0)) > 0,
        "wolf_deaths_le1": mux2.get("wolf_deaths", s["deaths"].get("combat", 0)) <= 1,
        "exact_ledger": s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0,
        "_snap_exposed": snap > 0,
        "_snap_ok": snap > 0 and (mux.get("cold_snap_firewood_zero_ticks", 0) or 0) == 0,
        "_logs5_ok": (len(caps) > 4 and s.get("loose_logs_day5", 0) <= 1.5 * caps[4]["target"]) if caps else None,
        "_logs24_ok": (len(caps) > 23 and len(logs) > 23 and logs[23] <= 1.5 * caps[23]["target"]) if caps else None,
        "_lost": len(s.get("m4", {}).get("lost") or []),
    }


def main(argv):
    base, cand = seeds(argv[1]), seeds(argv[2])
    num, den = (30, 32)
    if "--min-frac" in argv:
        num, den = map(int, argv[argv.index("--min-frac") + 1].split("/"))
    common = sorted(set(base) & set(cand))
    if not common:
        print("no common seeds")
        return 1
    n = len(common)
    need = -(-num * n // den)  # ceil(num/den * n)
    B = {k: rows(base[k]) for k in common}
    C = {k: rows(cand[k]) for k in common}
    ok = True
    print(f"seeds {common[0]}..{common[-1]} (n={n})")
    for bar in ("stockpile_day1", "walled_beds_day10", "zero_cold_deaths", "alive_day48", "wolf_deaths_le1", "exact_ledger"):
        b = sum(B[k][bar] for k in common)
        c = sum(C[k][bar] for k in common)
        p = c >= b - 1
        ok &= p
        print(f"  {bar:20s} BASE {b}/{n}  cand {c}/{n}  {'PASS' if p else 'FAIL'}")
    exposed = sum(C[k]["_snap_exposed"] for k in common)
    snap_ok = sum(C[k]["_snap_ok"] for k in common)
    p = snap_ok >= exposed - 1
    ok &= p
    print(f"  {'firewood_in_snaps':20s} exposed {exposed}  never-zero {snap_ok}  {'PASS' if p else 'FAIL'}")
    for bar, label in (("_logs5_ok", "day5_logs_le_1.5cap"), ("_logs24_ok", "day24_logs_le_1.5cap")):
        vals = [C[k][bar] for k in common]
        if any(v is None for v in vals):
            print(f"  {label:20s} no cap target in candidate output  FAIL")
            ok = False
            continue
        c = sum(vals)
        p = c >= -(-28 * n // 32)
        ok &= p
        print(f"  {label:20s} cand {c}/{n}  {'PASS' if p else 'FAIL'}")
    within = sum(C[k]["_lost"] <= B[k]["_lost"] + 1 for k in common)
    tb, tc = sum(B[k]["_lost"] for k in common), sum(C[k]["_lost"] for k in common)
    p = within >= need and tc <= tb + 3
    ok &= p
    print(f"  {'knowledge_lost':20s} within BASE+1 in {within}/{n} (need {need}), total BASE {tb} cand {tc}  {'PASS' if p else 'FAIL'}")
    print("LOOP_SCORE " + ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
