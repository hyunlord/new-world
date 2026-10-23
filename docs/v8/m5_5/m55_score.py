#!/usr/bin/env python3
"""Score M55-U probe runs (docs/v8/m5_5/M55_GOAL.md §10, bars F1–F10; F11 is the benchmark's).

usage: m55_score.py PRESSURE [--nodraft NODRAFT] [--storyoff STORYOFF] [--noneighbours NONEIGHBOURS]
                    [--quiet QUIET] [--out APPENDIX_JSON]
       m55_score.py --neighbours SUMMARY   the gate's `neighbours` fixture (step 30): every seed has two neighbours,
                                           waves 1 and 2 arrived in their windows and were accepted, at least one trade
                                           was made, and 0 ledger and 0 reservation violation ticks; NEIGHBOURS PASS/FAIL

Every run is `worldsim_probe --scene colony --days 48` on the same seeds (one summary.json each; per seed the `m5`
block of rust/crates/sim-test/src/probe/m5.rs and the `m55` block of probe/m55.rs):
- PRESSURE: `--commands tools/v8/m55_u_script.json` (storyteller and neighbours on; the scripted draft at every pack,
  every wave accepted, the blade trade at every caravan);
- NODRAFT: `--commands tools/v8/m55_u_nodraft_script.json` (the same, nobody drafted): F2b, F2c;
- STORYOFF: the M55-U script with `--story off`: F5b;
- NONEIGHBOURS: the M55-U script with `--neighbours off`: F8's comparison (discovery days), reported only;
- QUIET: the M55-U script with `--story off --neighbours off`: F10's demographics.

Registered before the first held-out run (M5.5 DECISIONS D22). A day d is the end of day d (the `daily` row "day": d).
- F1 (PRESSURE): (a) no minor pressure began during a major or within 2 days after one ended (`m55.overlap` empty),
  32/32; (b) deaths booked to pressures other than beasts ≤ 2 (Σ `deaths` of the non-beasts pressures), ≥ 30/32;
  (c) the world never lost half its people: total deaths × 2 < the most people at once, 32/32.
- F2: (a) PRESSURE: 0 deaths in combat while wolves were on the map (`m55.wolf_deaths`), ≥ 28/32; (b) NODRAFT: ≥ 1
  such death or someone below 50 hp while wolves were on the map (`m55.min_hp` < 50), ≥ 20/32; (c) pooled over the
  PRESSURE and NODRAFT worlds, flight episodes that moved away within 30 ticks / episodes ≥ 0.80 (`m55.flight`:
  an episode begins when an undrafted adult outside a roofed room comes within 8 of a hunting or biting wolf; it
  moved away when within 30 ticks they stand farther from the wolf's tile at its start than where they stood, or in
  a roofed room). All three.
- F3 (PRESSURE): ≥ 6 deer killed and ≥ 12 meat eaten by day 48, and meat spoiled off the stockpiles ≤ 10 % of the
  meat dropped (`meat_spoiled_afield` ≤ 0.10 × `meat_dropped`), ≥ 28/32.
- F4 (PRESSURE): ≥ 3 people wear a top (hide wrap or fur cloak) at the end of day 30 and ≥ 5 at day 48
  (`m55.apparel`), and the clothed people's winter-night cold rate ≤ 0.8 × the unclothed people's (both 0 passes;
  unclothed 0 with clothed > 0 fails), ≥ 24/32. Arithmetic: ~1 kill in 7 days early → ~4 wraps by day 30, ~6–7 by 48,
  plus migrants' wraps and traded ones; a wrap's 700 ‰ gives an expected ratio of 0.7.
- F5: (a) PRESSURE: ≥ 1 breakdown in ≥ 8/32 worlds and ≤ 3 breakdowns in every world; (b) STORYOFF: ≤ 1 breakdown
  in ≥ 28/32 worlds. Both.
- F6 (PRESSURE): (a) per world ≥ 4 quarrels in the 48 days, fights ≤ quarrels / 4, and no combat death outside the
  wolves (`deaths.combat` − `m55.wolf_deaths` = 0), ≥ 28/32; (b) pooled over the worlds, quarrels whose cause
  resolves (`m55.quarrels[].resolved`, `mood::check_cause` with a 2-day window) / quarrels ≥ 0.90. Both.
- F7 (PRESSURE): (a) waves 1 and 2 arrived (reached the hearth) on a day in 8–12 and 20–26, 32/32; (b) 16–40 people
  at the end of day 48, ≥ 30/32; (c) ≥ 1 accepted migrant brought a Neolithic node at or above its threshold
  (`waves[].neolithic_practitioners` of accepted waves), ≥ 20/32. All three.
- F8 (PRESSURE): (a) ≥ 6 caravans set out in the two years, ≥ 28/32; (b) ≥ 1 unknown artifact bought
  (`tallies.unknown_bought`: items taken whose producing node no colonist practised at the trade), ≥ 28/32. Both.
  Eureka discoveries are reported with the NONEIGHBOURS run's discovery days for comparison (no bar).
- F9 (PRESSURE): ≥ 5 blades crafted in days 37–48, the top knapper at level ≥ 6 at day 48, and the mean quality of the
  blades crafted in days 37–48 minus that of days 1–12 ≥ 8, ≥ 24/32. Arithmetic: quality 30 + 3L + 2.5 — level ~1
  early gives ~35.5, level ≥ 6 gives ≥ 50.5.
- F10: (a) QUIET: births per woman-year in 0.20–0.30, pooled (Σ births / Σ woman-years of the demography blocks);
  (b) 0 ledger (per kind, with traded in and out) and 0 reservation violation ticks in every run given. Both; the soak
  and story fixtures are gate steps.
"""
import json
import sys

BARS = {"F1a": 32, "F1b": 30, "F1c": 32, "F2a": 28, "F2b": 20, "F3": 28, "F4": 24, "F5b": 28, "F6a": 28,
        "F7a": 32, "F7b": 30, "F7c": 20, "F8a": 28, "F8b": 28, "F9": 24}


def at_day(daily, n):
    rows = [r for r in daily if r["day"] == n]
    return rows[0] if rows else None


def by_seed(path):
    return {s["seed"]: s for s in json.load(open(path))["seeds"]}


def score_pressure(s):
    m5, m = s["m5"], s["m55"]
    out = {"seed": s["seed"]}
    out["F1a"] = not m["overlap"]
    other = sum(p["deaths"] for p in m5["pressures"] if p["kind"] != "beasts")
    out["F1b"] = other <= 2
    deaths = sum(m5["deaths"].values())
    out["F1c"] = deaths * 2 < max(1, m["population"]["most"])
    out["F1_detail"] = {"overlap": m["overlap"], "non_wolf_pressure_deaths": other, "deaths": deaths,
                        "most": m["population"]["most"], "pressures": [(p["kind"], p["start_day"], p["end_day"])
                                                                         for p in m5["pressures"]]}
    out["F2a"] = m["wolf_deaths"] == 0
    out["flight"] = m["flight"]
    w = m5["wildlife"]
    out["F3"] = (w["deer_killed"] >= 6 and w["meat_eaten"] >= 12
                 and w.get("meat_spoiled_afield", 0) * 10 <= w["meat_dropped"])
    out["F3_detail"] = {"deer_killed": w["deer_killed"], "meat_eaten": w["meat_eaten"],
                        "meat_dropped": w["meat_dropped"], "spoiled_afield": w.get("meat_spoiled_afield"),
                        "meat_hauls": w.get("meat_hauls")}
    a = m5["apparel"]
    rate = lambda c, e: c / e if e else 0.0
    unclothed, clothed = rate(a["cold_hours"][0], a["exposure_hours"][0]), rate(a["cold_hours"][1], a["exposure_hours"][1])
    ap = m["apparel"]
    ratio_ok = clothed <= 0.8 * unclothed + 1e-12
    out["F4"] = (ap["wearers_day30"] or 0) >= 3 and (ap["wearers_end"] or 0) >= 5 and ratio_ok
    out["F4_detail"] = {"wearers_day30": ap["wearers_day30"], "wearers_end": ap["wearers_end"],
                        "clothed_rate": clothed, "unclothed_rate": unclothed}
    mood = m5["mood"]
    out["breakdowns"] = mood["breakdowns"]
    q = m["quarrels"]
    combat_other = m5["deaths"]["combat"] - m["wolf_deaths"]
    out["F6a"] = len(q) >= 4 and mood["fights"] * 4 <= len(q) and combat_other == 0
    out["quarrels"] = len(q)
    out["resolved"] = sum(1 for x in q if x["resolved"])
    out["F6_detail"] = {"quarrels": len(q), "resolved": out["resolved"], "fights": mood["fights"],
                        "combat_other": combat_other,
                        "unresolved": [x["reason"] for x in q if not x["resolved"]]}
    nb = m["neighbours"]
    waves = {wv["id"]: wv for wv in nb["waves"]}
    arrived_day = lambda wv: None if wv is None or wv["arrived"] is None else wv["arrived"] // 1440
    d1, d2 = arrived_day(waves.get(1)), arrived_day(waves.get(2))
    out["F7a"] = d1 is not None and 8 <= d1 <= 12 and d2 is not None and 20 <= d2 <= 26
    end = m["population"]["end"]
    out["F7b"] = 16 <= end <= 40
    prac = sum(wv["neolithic_practitioners"] for wv in nb["waves"] if wv["answer"] == "Accepted")
    out["F7c"] = prac >= 1
    out["F7_detail"] = {"wave_days": [(wv["id"], wv["day"], arrived_day(wv), wv["size"], wv["answer"])
                                      for wv in nb["waves"]], "population_end": end, "practitioner_nodes": prac}
    t = nb["tallies"]
    out["F8a"] = len(nb["caravans"]) >= 6
    out["F8b"] = t.get("unknown_bought", 0) >= 1
    out["F8_detail"] = {"caravans": len(nb["caravans"]), "trades": t["trades"], "unknown_bought": t.get("unknown_bought"),
                        "eurekas": len(nb["eurekas"]), "traded_in": nb["traded_in"], "traded_out": nb["traded_out"]}
    daily = m5["daily"]
    d12, d36, d48 = at_day(daily, 12), at_day(daily, 36), at_day(daily, 48) or (daily[-1] if daily else {})
    early = late = None
    late_n = 0
    if d12 and d12["blades"][0] > 0:
        early = d12["blades"][1] / d12["blades"][0]
    if d36 and d48:
        late_n = d48["blades"][0] - d36["blades"][0]
        if late_n > 0:
            late = (d48["blades"][1] - d36["blades"][1]) / late_n
    out["F9"] = (late_n >= 5 and d48.get("top_knap", 0) >= 6 and early is not None and late is not None
                 and late - early >= 8)
    out["F9_detail"] = {"blades_37_48": late_n, "top_knap": d48.get("top_knap"), "early": early, "late": late,
                        "knives_broken": d48.get("knives_broken")}
    out["F10"] = s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0
    out["discoveries"] = nb["discoveries"]
    return out


def neighbours_fixture(path):
    seeds = json.load(open(path))["seeds"]
    bad = [] if seeds else ["no seeds"]
    for s in seeds:
        nb = s["m55"]["neighbours"]
        waves = {wv["id"]: wv for wv in nb["waves"]}
        day = lambda i: None if waves.get(i) is None or waves[i]["arrived"] is None else waves[i]["arrived"] // 1440
        ok = (len(nb["list"]) == 2 and nb["list"][0]["edge"] != nb["list"][1]["edge"]
              and day(1) is not None and 8 <= day(1) <= 12 and day(2) is not None and 20 <= day(2) <= 26
              and all(waves[i]["answer"] == "Accepted" for i in (1, 2))
              and nb["tallies"]["trades"] >= 1)
        clean = s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0
        print(f"seed {s['seed']}: waves {[(i, day(i), waves[i]['answer']) for i in sorted(waves)]} trades "
              f"{nb['tallies']['trades']} traded out {nb['traded_out']} ledger {s['ledger_violation_ticks']} "
              f"reservations {s['reservation_violation_ticks']}")
        if not ok:
            bad.append(f"seed {s['seed']}: neighbours")
        if not clean:
            bad.append(f"seed {s['seed']}: violations {s['first_violation']}")
    for b in bad:
        print("NEIGHBOURS FAIL", b)
    if not bad:
        print(f"NEIGHBOURS PASS ({len(seeds)} seeds: two neighbours, waves 1 and 2 in their windows and accepted, "
              "a trade, ledger and reservations exact)")
    return 1 if bad else 0


def main(argv):
    if len(argv) == 2 and argv[0] == "--neighbours":
        return neighbours_fixture(argv[1])
    if not argv or argv[0].startswith("--"):
        print(__doc__, file=sys.stderr)
        return 2
    opt = lambda name: argv[argv.index(name) + 1] if name in argv else None
    rows = {seed: score_pressure(s) for seed, s in sorted(by_seed(argv[0]).items())}
    n = len(rows)
    flight = [0, 0]
    for r in rows.values():
        flight[0] += r["flight"]["moved_away"]
        flight[1] += r["flight"]["episodes"]
    births = [0, 0.0]
    discovery = {}
    for name in ("--nodraft", "--storyoff", "--noneighbours", "--quiet"):
        path = opt(name)
        if not path:
            continue
        for seed, s in by_seed(path).items():
            r = rows.get(seed)
            if r is None:
                continue
            m5, m = s["m5"], s["m55"]
            r["F10"] = r["F10"] and s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0
            if name == "--nodraft":
                r["F2b"] = m["wolf_deaths"] >= 1 or (m["min_hp"] is not None and m["min_hp"] < 50)
                r["F2b_detail"] = {"wolf_deaths": m["wolf_deaths"], "min_hp": m["min_hp"]}
                flight[0] += m["flight"]["moved_away"]
                flight[1] += m["flight"]["episodes"]
            elif name == "--storyoff":
                r["F5b"] = m5["mood"]["breakdowns"] <= 1
                r["breakdowns_storyoff"] = m5["mood"]["breakdowns"]
            elif name == "--noneighbours":
                discovery[seed] = m["neighbours"]["discoveries"]
            else:
                d = s.get("demography", {})
                births[0] += d.get("births", 0)
                births[1] += d.get("woman_years", 0.0)
    print(f"{n} seeds")
    ok = {}
    for bar, need in BARS.items():
        scored = [r for r in rows.values() if bar in r]
        if not scored:
            print(f"{bar}: not scored (run missing)")
            continue
        got = sum(1 for r in scored if r[bar])
        scaled = need * n / 32
        ok[bar] = got >= scaled - 1e-9
        print(f"{bar}: {got}/{n} (bar {need}/32 = {scaled:.1f}/{n}) {'PASS' if ok[bar] else 'FAIL'}")
    rate = flight[0] / flight[1] if flight[1] else 0.0
    ok["F2c"] = flight[1] > 0 and rate >= 0.80
    print(f"F2c: pooled {flight[0]}/{flight[1]} = {rate:.3f} moved away within 30 ticks (bar 0.80) "
          f"{'PASS' if ok['F2c'] else 'FAIL'}")
    with_bd = sum(1 for r in rows.values() if r["breakdowns"] >= 1)
    most_bd = max((r["breakdowns"] for r in rows.values()), default=0)
    ok["F5a"] = with_bd >= 8 * n / 32 - 1e-9 and most_bd <= 3
    print(f"F5a: {with_bd}/{n} worlds with a breakdown (bar 8/32 = {8 * n / 32:.1f}/{n}), most in a world {most_bd} "
          f"(bar ≤ 3) {'PASS' if ok['F5a'] else 'FAIL'}; mean {sum(r['breakdowns'] for r in rows.values()) / max(n, 1):.2f}")
    q = sum(r["quarrels"] for r in rows.values())
    res = sum(r["resolved"] for r in rows.values())
    ok["F6b"] = q > 0 and res / q >= 0.90
    print(f"F6b: pooled {res}/{q} quarrel causes resolve = {res / q if q else 0:.3f} (bar 0.90) "
          f"{'PASS' if ok['F6b'] else 'FAIL'}; quarrels per world-year {q / max(n, 1) / 2:.2f}")
    if opt("--quiet"):
        rate = births[0] / births[1] if births[1] else 0.0
        ok["F10a"] = 0.20 <= rate <= 0.30
        print(f"F10a: pooled births/woman-year {rate:.3f} ({births[0]} / {births[1]:.1f}) (bar 0.20–0.30) "
              f"{'PASS' if ok['F10a'] else 'FAIL'}")
    got10 = sum(1 for r in rows.values() if r["F10"])
    print(f"F10b: {got10}/{n} worlds with 0 ledger and 0 reservation violation ticks in every run "
          f"{'PASS' if got10 == n else 'FAIL'}")
    eur = sum(r["F8_detail"]["eurekas"] for r in rows.values())
    print(f"F8 eureka discoveries (no bar): {eur} over {n} worlds")
    verdict = {
        "F1": all(ok.get(b, False) for b in ("F1a", "F1b", "F1c")),
        "F2": all(ok.get(b, False) for b in ("F2a", "F2b", "F2c")),
        "F3": ok.get("F3", False),
        "F4": ok.get("F4", False),
        "F5": ok.get("F5a", False) and ok.get("F5b", False),
        "F6": ok.get("F6a", False) and ok.get("F6b", False),
        "F7": all(ok.get(b, False) for b in ("F7a", "F7b", "F7c")),
        "F8": ok.get("F8a", False) and ok.get("F8b", False),
        "F9": ok.get("F9", False),
        "F10": ok.get("F10a", False) and got10 == n,
    }
    for bar, v in verdict.items():
        print(f"{bar}: {'PASS' if v else 'FAIL'}")
    if opt("--out"):
        json.dump({"rows": list(rows.values()), "noneighbours_discoveries": discovery}, open(opt("--out"), "w"),
                  indent=1, default=str)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
