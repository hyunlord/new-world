#!/usr/bin/env python3
"""Score an M4-U probe run (docs/v8/m4/M4_GOAL.md §10, bars F1–F9 and F11).

usage: m4_score.py SUMMARY_JSON [--out APPENDIX_JSON]

Reads the colony probe's summary.json (`worldsim_probe --scene colony --commands tools/v8/m4_u_script.json
--days 36`), whose per-seed `m4` block is written by rust/crates/sim-test/src/probe/knowledge.rs. A day d means the
end of day d (the `daily` row with "day": d). "By day N" means: reached at the end of some day ≤ N. Registered
before the first evaluation run (M4 DECISIONS D18):
- F1: at tick 0 at least 2 nodes are at risk (one holder) and the badge equals their count.
- F2: adult flint-knapping practitioners reach 6 by day 12.
- F3: the doorway apprenticeship order (the script's only order) closes done — its learner practises the doorway,
  by sessions or because it heard it at the fire first — before the end of day 8 (`teach_done_ticks`).
- F4: doorway practitioners reach 2 by day 10 and 3 by day 24.
- F5: at least one seeded entry fell below its threshold by day 36, and no once-known gate the colony had an order
  for stood without a living adult practitioner for more than 3 consecutive days.
- F6: stone grinding discovered by day 30.
- F7: clay digging, clay preparation, pot forming and pot firing all discovered and 2 or more clay pots fired by day
  36; the chain's stall point otherwise (the first missing step).
- F8: the top knapper at level 6 or more at day 36, and the blade quality mean of days 25–36 minus that of days 1–6
  is at least 8.
- F9: era progress > 0 at day 36 when F6 or F7 passed, and never above 643 ‰.
- F11: 0 ledger and reservation violation ticks, 0 deaths.
"""
import json
import sys

BARS = {"F1": 32, "F2": 28, "F3": 28, "F4": 24, "F5": 28, "F6": 20, "F7": 16, "F8": 24, "F9": 32, "F11": 32}
CHAIN = ["clay_digging", "clay_prep", "pot_forming", "pot_firing"]


def by_day(daily, n, f):
    return any(f(r) for r in daily if r["day"] <= n)


def at_day(daily, n):
    rows = [r for r in daily if r["day"] == n]
    return rows[0] if rows else None


def score_seed(s):
    m = s["m4"]
    names = m["node_names"]
    idx = {n: i for i, n in enumerate(names)}
    daily = m["daily"]
    day = 1440
    out = {"seed": s["seed"]}
    start = m["start"]
    out["F1"] = len(start["at_risk"]) >= 2 and start["badge"] == len(start["at_risk"])
    out["F1_detail"] = start["at_risk"]
    knap = [r["knappers"] for r in daily if r["day"] <= 12]
    out["F2"] = bool(knap) and max(knap) >= 6
    out["F2_detail"] = max(knap) if knap else None
    done = m.get("teach_done_ticks", [])
    out["F3"] = any(t < 8 * day for t in done)
    out["F3_detail"] = {"done_day": done[0] / day if done else None,
                        "by_sessions": [x[0] / day for x in m["learned"] if x[3] == "doorway"],
                        "sessions": m["teach"]["sessions"]}
    door = idx["doorway"]
    out["F4"] = by_day(daily, 10, lambda r: r["nodes"][door][0] >= 2) and by_day(daily, 24, lambda r: r["nodes"][door][0] >= 3)
    out["F4_detail"] = [r["nodes"][door][0] for r in daily if r["day"] in (10, 24)]
    last = daily[-1] if daily else {}
    out["F5"] = last.get("forgotten_seeded", 0) >= 1 and m["max_stall_days"] <= 3
    out["F5_detail"] = {"forgotten_seeded": last.get("forgotten_seeded"), "max_stall_days": m["max_stall_days"]}
    disc = m["discovered"]
    out["F6"] = "stone_grinding" in disc and disc["stone_grinding"] < 30 * day
    out["F6_detail"] = disc.get("stone_grinding", None) and disc["stone_grinding"] / day
    pots = last.get("clay_pots", 0)
    found = [c for c in CHAIN if c in disc and disc[c] < 36 * day]
    out["F7"] = len(found) == 4 and pots >= 2
    if out["F7"]:
        stall = "done"
    elif len(found) < 4:
        stall = next(c for c in CHAIN if c not in found)
    else:
        stall = "pots fired %d" % pots
    out["F7_detail"] = {"discovered": found, "pots": pots, "raw_pots": last.get("raw_pots"), "clay": last.get("clay"), "stall": stall}
    d6, d24, d36 = at_day(daily, 6), at_day(daily, 24), at_day(daily, 36)
    early = late = None
    if d6 and d6["blades"][0] > 0:
        early = d6["blades"][1] / d6["blades"][0]
    if d24 and d36 and d36["blades"][0] > d24["blades"][0]:
        late = (d36["blades"][1] - d24["blades"][1]) / (d36["blades"][0] - d24["blades"][0])
    top = d36["top_knap"] if d36 else 0
    out["F8"] = top >= 6 and early is not None and late is not None and late - early >= 8
    out["F8_detail"] = {"top_knap": top, "early": early, "late": late}
    progress = [r["progress"] for r in daily]
    out["F9"] = (max(progress) if progress else 0) <= 643 and (not (out["F6"] or out["F7"]) or last.get("progress", 0) > 0)
    out["F9_detail"] = {"end": last.get("progress"), "max": max(progress) if progress else None}
    deaths = sum(s["deaths"].values())
    out["F11"] = s["ledger_violation_ticks"] == 0 and s["reservation_violation_ticks"] == 0 and deaths == 0
    out["F11_detail"] = {"deaths": s["deaths"], "ledger": s["ledger_violation_ticks"], "reservations": s["reservation_violation_ticks"]}
    out["lost"] = m["lost"]
    out["bytes_per_person"] = m["bytes_per_person"]
    return out


def main(argv):
    if not argv:
        print(__doc__, file=sys.stderr)
        return 2
    seeds = json.load(open(argv[0]))["seeds"]
    rows = [score_seed(s) for s in seeds]
    n = len(rows)
    print(f"{n} seeds")
    for bar, need in BARS.items():
        got = sum(1 for r in rows if r[bar])
        scaled = need * n / 32
        verdict = "PASS" if got >= scaled - 1e-9 else "FAIL"
        print(f"{bar}: {got}/{n} (bar {need}/32 = {scaled:.1f}/{n}) {verdict}")
    stalls = {}
    for r in rows:
        stalls[r["F7_detail"]["stall"]] = stalls.get(r["F7_detail"]["stall"], 0) + 1
    print("F7 stall points:", stalls)
    if "--out" in argv:
        json.dump(rows, open(argv[argv.index("--out") + 1], "w"), indent=1, default=str)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
