#!/usr/bin/env python3
"""Score M5-U probe runs (docs/v8/m5/M5_GOAL.md §11, bars F1–F10; F11 is the benchmark's).

usage: m5_score.py PRESSURE_SUMMARY [--control CONTROL_SUMMARY] [--nodraft NODRAFT_SUMMARY]
                   [--long LONG_SUMMARY] [--out APPENDIX_JSON]
       m5_score.py --story SUMMARY     the gate's `story` fixture (step 29): every seed of a storyteller run has
                                       F1's schedule, 0 ledger and 0 reservation violation ticks; STORY PASS / FAIL

The runs (each `worldsim_probe --scene colony --days 48`, one summary.json per run, per-seed `m5` block written
by rust/crates/sim-test/src/probe/m5.rs):
- PRESSURE: `--commands tools/v8/m5_u_script.json` (the storyteller on; the scripted draft at every wolf pack);
- CONTROL: the same script with `--story off` (no pressure: F6a, F9a);
- NODRAFT: `--commands tools/v8/m5_u_nodraft_script.json` (the storyteller on, nobody drafted: F3b);
- LONG: no script, `--story off`, `--days 72` (the 3-year no-orders control: F9b).

A day d means the end of day d (the `daily` row with "day": d). Registered before the first held-out run
(M5 DECISIONS D13):
- F1: every world sees ≥ 1 cold snap and ≥ 1 beasts; no drought or beasts before day 12, no illness before day
  18, no cold snap on the first winter's first two days (18, 19), majors ≥ 6 days apart, beasts ≥ 6 days apart.
- F2a: some cold snap's lowest hourly usable firewood (settlement wood + feedable firewood) fell below one day's
  need at the snap's burn. F2b: exposure deaths ≤ 1 over the run. Both bars must hold.
- F3a (PRESSURE): every pack that came is beaten — no wolf of it left on the map, at least half of it killed — and
  no one died while any pack was on the map. F3b (NODRAFT): some pack bit someone (≥ 1 injury).
- F4: by day 48 ≥ 6 deer killed (each a hide), ≥ 12 meat eaten (put in the larder).
- F5: ≥ 4 people wear a hide wrap at the end of day 30, and the clothed people's winter-night cold rate (cold
  person-hours / person-hours) is at most half the unclothed people's (both 0 passes; unclothed 0 with clothed > 0
  fails).
- F6a (CONTROL): breakdowns ≤ 1. F6b (PRESSURE): at least 1 world in 4 has a breakdown.
- F7: ≥ 4 quarrels in the 48 days (2 a world-year), every one with a resolvable "why?" chain (≥ 2 links), fights
  ≤ quarrels / 4, and no combat death outside the wolf packs (no quarrel death).
- F8: knives broken ≥ 10 by day 48, the top knapper at level ≥ 6 at day 48, and the blade quality mean of days
  37–48 minus that of days 1–12 ≥ 8.
- F9a (CONTROL): births per woman-year in 0.20–0.30, pooled over the worlds (Σ births / Σ woman-years of the
  demography blocks; a world has 0–4 births in two years, so a per-world rate is noise — as M1.5 measured it).
  F9b (LONG): no exposure death in ≥ 30/32 worlds.
- F10: 0 ledger and 0 reservation violation ticks in every run given.
"""
import json
import sys

BARS = {"F1": 32, "F2a": 24, "F2b": 28, "F3a": 24, "F3b": 24, "F4": 28, "F5": 24, "F6a": 28, "F6b": 8,
        "F7": 28, "F8": 24, "F9a": 32, "F9b": 30, "F10": 32}


def at_day(daily, n):
    rows = [r for r in daily if r["day"] == n]
    return rows[0] if rows else None


def f1(m):
    p = m["pressures"]
    kinds = [x["kind"] for x in p]
    ok = "cold_snap" in kinds and "beasts" in kinds
    majors = sorted(x["start_day"] for x in p if x["kind"] != "illness")
    beasts = sorted(x["start_day"] for x in p if x["kind"] == "beasts")
    ok &= all(b - a >= 6 for a, b in zip(majors, majors[1:]))
    ok &= all(b - a >= 6 for a, b in zip(beasts, beasts[1:]))
    for x in p:
        d = x["start_day"]
        if x["kind"] in ("drought", "beasts") and d < 12:
            ok = False
        if x["kind"] == "illness" and d < 18:
            ok = False
        if x["kind"] == "cold_snap" and d in (18, 19):
            ok = False
    return ok, {"kinds": kinds, "days": [x["start_day"] for x in p]}


def score_pressure(s):
    m = s["m5"]
    out = {"seed": s["seed"]}
    out["F1"], out["F1_detail"] = f1(m)
    snaps = m["snaps"]
    out["F2a"] = any(x["min_firewood"] < x["day_need"] for x in snaps)
    out["F2a_detail"] = snaps
    out["F2b"] = m["cold_deaths"] <= 1
    out["F2b_detail"] = m["cold_deaths"]
    packs = m["beasts"]
    beaten = all(x["alive"] == 0 and 2 * x["killed"] >= x["wolves"] for x in packs)
    out["F3a"] = bool(packs) and beaten and all(x["deaths"] == 0 for x in packs)
    out["F3a_detail"] = packs
    daily = m["daily"]
    d48 = at_day(daily, 48) or (daily[-1] if daily else {})
    w = m["wildlife"]
    out["F4"] = w["deer_killed"] >= 6 and w["meat_eaten"] >= 12
    out["F4_detail"] = {"deer_killed": w["deer_killed"], "meat_eaten": w["meat_eaten"], "meat_spoiled": w["meat_spoiled"]}
    a = m["apparel"]
    rate = lambda c, e: c / e if e else 0.0
    unclothed, clothed = rate(a["cold_hours"][0], a["exposure_hours"][0]), rate(a["cold_hours"][1], a["exposure_hours"][1])
    out["F5"] = (a["wraps_day30"] or 0) >= 4 and clothed <= unclothed / 2 + 1e-12
    out["F5_detail"] = {"wraps_day30": a["wraps_day30"], "clothed_rate": clothed, "unclothed_rate": unclothed}
    mood = m["mood"]
    out["breakdowns"] = mood["breakdowns"]
    q = mood["quarrel_list"]
    pack_deaths = sum(x["deaths"] for x in packs)
    out["F7"] = (len(q) >= 4 and all(x["resolvable"] for x in q) and mood["fights"] * 4 <= len(q)
                 and m["deaths"]["combat"] <= pack_deaths)
    out["F7_detail"] = {"quarrels": len(q), "fights": mood["fights"], "causes": [x["cause"] for x in q],
                        "combat_deaths": m["deaths"]["combat"], "pack_deaths": pack_deaths}
    d12, d36 = at_day(daily, 12), at_day(daily, 36)
    early = late = None
    if d12 and d12["blades"][0] > 0:
        early = d12["blades"][1] / d12["blades"][0]
    if d36 and d48 and d48["blades"][0] > d36["blades"][0]:
        late = (d48["blades"][1] - d36["blades"][1]) / (d48["blades"][0] - d36["blades"][0])
    out["F8"] = (d48.get("knives_broken", 0) >= 10 and d48.get("top_knap", 0) >= 6 and early is not None
                 and late is not None and late - early >= 8)
    out["F8_detail"] = {"knives_broken": d48.get("knives_broken"), "knife_uses": d48.get("knife_uses"),
                        "top_knap": d48.get("top_knap"), "early": early, "late": late}
    out["F10"] = s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0
    return out


def by_seed(path):
    return {s["seed"]: s for s in json.load(open(path))["seeds"]}


def story_fixture(path):
    seeds = json.load(open(path))["seeds"]
    bad = [] if seeds else ["no seeds"]
    for s in seeds:
        ok, detail = f1(s["m5"])
        clean = s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0
        print(f"seed {s['seed']}: schedule {'ok' if ok else 'BAD'} {detail['kinds']} days {detail['days']} "
              f"ledger {s['ledger_violation_ticks']} reservations {s['reservation_violation_ticks']}")
        if not ok:
            bad.append(f"seed {s['seed']}: F1 schedule")
        if not clean:
            bad.append(f"seed {s['seed']}: violations {s['first_violation']}")
    for b in bad:
        print("STORY FAIL", b)
    if not bad:
        print(f"STORY PASS ({len(seeds)} seeds: F1 schedule, ledger and reservations exact)")
    return 1 if bad else 0


def main(argv):
    if len(argv) == 2 and argv[0] == "--story":
        return story_fixture(argv[1])
    if not argv or argv[0].startswith("--"):
        print(__doc__, file=sys.stderr)
        return 2
    opt = lambda name: argv[argv.index(name) + 1] if name in argv else None
    pressure = by_seed(argv[0])
    rows = {seed: score_pressure(s) for seed, s in sorted(pressure.items())}
    n = len(rows)
    pooled = [0, 0.0]
    for name, key in (("--control", "control"), ("--nodraft", "nodraft"), ("--long", "long")):
        path = opt(name)
        if not path:
            continue
        for seed, s in by_seed(path).items():
            r = rows.get(seed)
            if r is None:
                continue
            m = s["m5"]
            ok10 = s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0
            r["F10"] = r["F10"] and ok10
            if key == "control":
                r["F6a"] = m["mood"]["breakdowns"] <= 1
                d = s.get("demography", {})
                pooled[0] += d.get("births", 0)
                pooled[1] += d.get("woman_years", 0.0)
            elif key == "nodraft":
                r["F3b"] = any(x["bitten"] >= 1 for x in m["beasts"])
                r["F3b_detail"] = m["beasts"]
            else:
                r["F9b"] = m["cold_deaths"] == 0
    if opt("--control"):
        rate = pooled[0] / pooled[1] if pooled[1] else 0.0
        print(f"F9a pooled births/woman-year {rate:.3f} ({pooled[0]} births / {pooled[1]:.1f} woman-years)")
        for r in rows.values():
            r["F9a"] = 0.20 <= rate <= 0.30
    with_breakdown = sum(1 for r in rows.values() if r["breakdowns"] >= 1)
    print(f"{n} seeds")
    for bar, need in BARS.items():
        if bar == "F6b":
            scaled = need * n / 32
            verdict = "PASS" if with_breakdown >= scaled - 1e-9 else "FAIL"
            print(f"{bar}: {with_breakdown}/{n} worlds with a breakdown (bar {need}/32 = {scaled:.1f}/{n}) {verdict}")
            continue
        scored = [r for r in rows.values() if bar in r]
        if not scored:
            print(f"{bar}: not scored (run missing)")
            continue
        got = sum(1 for r in scored if r[bar])
        scaled = need * n / 32
        verdict = "PASS" if got >= scaled - 1e-9 else "FAIL"
        print(f"{bar}: {got}/{n} (bar {need}/32 = {scaled:.1f}/{n}) {verdict}")
    if opt("--out"):
        json.dump(list(rows.values()), open(opt("--out"), "w"), indent=1, default=str)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
