#!/usr/bin/env python3
"""Pass 14 Part 0 item 3: the factual timeline of each seed and its cast times (PASS14_GOAL.md 5.3). Stdlib only.

usage: timeline_p14.py --summary SUMMARY.json --out DIR [--fork-tick 14280] [--days 120]

SUMMARY.json is a `worldsim_probe` schema-13 summary of `game_v19` (per-seed `runs[]` with `pass11`, `pass12`,
`pass13`). Per seed, from `pass12.fission_checks` = [tick, sid, members, capacity, food, g, q, s, h, draw, split,
site_free] (the eligible daily checks; a check's tick is its 06:00 tick):

- the food line of a check = max(1, floor(members x 2.4 x 4)) (adult need 0.05 x 1440 / 30 = 2.4 a day, 4 days) and
  s = clamp(1000 - food x 1000 // line, 0, 1000); `line_check` counts the checks whose logged s this reproduces;
- the home settlement = the lowest settlement id (1, founded at tick 0); its departures = `pass11.fission_log`
  records with that parent (cancelled ones included and counted), sorted by `depart_tick`;
- **T_low** = the tick of the first eligible home check with s > 0 at least 3 days (4,320 ticks) before the first
  departure; if none and the village departs, the first such check after that departure and at least 3 days before
  the next departure (no bound without one); a seed that never departs uses its first check with s > 0; none at
  all -> no T_low;
- **T_high** = the tick of the eligible home check with s = 0 and the largest food / line ratio (ties: earliest) at
  least 3 days before the first departure (no bound for a seed that never departs), used only if that ratio
  >= 1.25; else no T_high;
- **T_eve** = tick 29,400 (display day 21, 18:00), the same for every seed.

A cast at a check tick is applied before that tick's systems run, so it changes that same morning's s. Every time
must satisfy fork_tick <= T and T + 4 days < days x 1440 (the split dose's last row); a time that does not is
dropped and counted. Display days below are one-based ((tick + 480) // 1440 + 1); the probe's day fields are zero-based.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

TPD, OFFSET = 1440, 480
ADULT_NEED_PER_DAY = 0.05 * 1440 / 30  # 2.4 food a day (food_economy.rs)
SCARCITY_DAYS = 4
GAP = 3 * TPD
T_EVE = 29_400
SPLIT_SPAN = 4 * TPD
HIGH_MIN_RATIO = 1.25


def display_day(tick: int) -> int:
    """One-based display day of a tick."""
    return (tick + OFFSET) // TPD + 1


def clock(tick: int) -> str:
    minute = (tick + OFFSET) % TPD
    return f"{minute // 60:02d}:{minute % 60:02d}"


def food_line(members: int) -> int:
    return max(1, math.floor(members * ADULT_NEED_PER_DAY * SCARCITY_DAYS))


def scarcity(members: int, food: int) -> int:
    line = food_line(members)
    return min(1000, max(0, 1000 - food * 1000 // line))


def seed_timeline(run: dict, fork_tick: int, days: int) -> dict:
    """The timeline of one probe run (see the module docs)."""
    checks = run["pass12"]["fission_checks"]
    log = run["pass11"]["fission_log"]
    sids = sorted({c[1] for c in checks} | {f["parent"] for f in log} | {1})
    home = sids[0]
    reproduced = sum(1 for c in checks if scarcity(c[2], c[4]) == c[7])
    home_checks = [{"tick": c[0], "day": display_day(c[0]), "members": c[2], "food": c[4], "line": food_line(c[2]),
                    "ratio": c[4] / food_line(c[2]), "s": c[7], "split": c[10]} for c in checks if c[1] == home]
    departures = sorted(f["depart_tick"] for f in log if f["parent"] == home)
    cancelled = sum(1 for f in log if f["parent"] == home and f.get("cancelled_tick") is not None)
    first = departures[0] if departures else None
    end = days * TPD

    def usable(tick: int) -> bool:
        return fork_tick <= tick and tick + SPLIT_SPAN < end

    t_low, low_rule = None, None
    shortfall = [c for c in home_checks if c["s"] > 0]
    if first is None:
        cand = shortfall
        low_rule = "never departs: first s > 0"
    else:
        cand = [c for c in shortfall if c["tick"] <= first - GAP]
        low_rule = "s > 0, >= 3 d before the first departure"
        if not cand:
            nxt = departures[1] if len(departures) > 1 else None
            cand = [c for c in shortfall if c["tick"] > first and (nxt is None or c["tick"] <= nxt - GAP)]
            low_rule = "s > 0 after the first departure, >= 3 d before the next"
    dropped = []
    for c in cand:
        if usable(c["tick"]):
            t_low = c["tick"]
            break
        dropped.append(("T_low", c["tick"]))
    if t_low is None:
        low_rule = None
    t_high, high_ratio, best = None, None, None
    full = [c for c in home_checks if c["s"] == 0 and (first is None or c["tick"] <= first - GAP)]
    for c in full:
        if best is None or c["ratio"] > best["ratio"]:
            best = c
    if best is not None:
        high_ratio = best["ratio"]
        if best["ratio"] >= HIGH_MIN_RATIO:
            if usable(best["tick"]):
                t_high = best["tick"]
            else:
                dropped.append(("T_high", best["tick"]))
    return {
        "seed": run["seed"], "home": home, "checks": len(checks), "line_check": [reproduced, len(checks)],
        "home_checks_s_pos": [[c["day"], c["food"], c["line"], round(c["ratio"], 4), c["s"]] for c in home_checks if c["s"] > 0],
        "home_checks_s_zero": [[c["day"], c["food"], c["line"], round(c["ratio"], 4)] for c in home_checks if c["s"] == 0],
        "departures": departures, "departures_cancelled": cancelled,
        "first_departure_tick": first, "first_departure_day": None if first is None else display_day(first),
        "T_low": t_low, "T_low_rule": low_rule, "T_high": t_high, "T_high_ratio": high_ratio, "T_eve": T_EVE,
        "dropped_outside_run": dropped,
    }


def build(summary: dict, fork_tick: int, days: int) -> dict:
    seeds = [seed_timeline(r, fork_tick, days) for r in sorted(summary["runs"], key=lambda r: r["seed"])]
    n = len(seeds)
    share = lambda key: sum(1 for s in seeds if s[key] is not None) / n if n else 0.0
    ratios = sorted(s["T_high_ratio"] for s in seeds if s["T_high_ratio"] is not None)
    return {
        "scene": summary.get("scene"), "seeds": [s["seed"] for s in seeds], "fork_tick": fork_tick, "days": days,
        "rule": {"line": "max(1, floor(members x 2.4 x 4))", "gap_ticks": GAP, "T_eve": T_EVE, "high_min_ratio": HIGH_MIN_RATIO},
        "line_check": [sum(s["line_check"][0] for s in seeds), sum(s["line_check"][1] for s in seeds)],
        "share": {"T_low": share("T_low"), "T_high": share("T_high"), "T_eve": 1.0 if n else 0.0},
        "count": {"T_low": sum(1 for s in seeds if s["T_low"] is not None), "T_high": sum(1 for s in seeds if s["T_high"] is not None),
                  "T_eve": n, "seeds": n, "never_depart": sum(1 for s in seeds if s["first_departure_tick"] is None),
                  "low_after_first_departure": sum(1 for s in seeds if s["T_low_rule"] and "after" in s["T_low_rule"]),
                  "departures_cancelled": sum(s["departures_cancelled"] for s in seeds)},
        "max_s0_ratio": {"min": ratios[0] if ratios else None, "median": ratios[len(ratios) // 2] if ratios else None,
                         "max": ratios[-1] if ratios else None, "n": len(ratios)},
        "per_seed": seeds,
    }


def fmt_t(tick: int | None) -> str:
    return "—" if tick is None else f"{tick} (d{display_day(tick)} {clock(tick)})"


def markdown(t: dict) -> str:
    c = t["count"]
    lines = [f"# Pass 14 Part 0 — factual timeline (`{t['scene']}`, seeds {t['seeds'][0]}–{t['seeds'][-1]})", "",
             "Rules: PASS14_GOAL.md 5.3, implemented in `tools/revitalize/timeline_p14.py` (module docs). Display days are "
             "one-based (`(tick + 480) // 1440 + 1`; the probe's day fields are zero-based). A check's tick is its 06:00 tick; a cast "
             "at a check tick is applied before that tick's systems run (`export.rs` `apply_due` before `engine.tick()`), so it "
             "changes that same morning's s.", "",
             f"Food line check: the line max(1, floor(members × 2.4 × 4)) reproduces the logged s on {t['line_check'][0]} of "
             f"{t['line_check'][1]} eligible checks (all settlements, all seeds).", "",
             f"Seeds with T_low {c['T_low']} / {c['seeds']} ({t['share']['T_low']:.3f}; {c['low_after_first_departure']} from the "
             f"after-first-departure clause), T_high {c['T_high']} / {c['seeds']} ({t['share']['T_high']:.3f}), T_eve {c['T_eve']} / "
             f"{c['seeds']}. Seeds that never depart: {c['never_depart']}. Cancelled home departures: {c['departures_cancelled']}. "
             f"Largest s = 0 food/line ratio before the first departure (per seed with an s = 0 check): min "
             f"{t['max_s0_ratio']['min'] and round(t['max_s0_ratio']['min'], 3)}, median "
             f"{t['max_s0_ratio']['median'] and round(t['max_s0_ratio']['median'], 3)}, max "
             f"{t['max_s0_ratio']['max'] and round(t['max_s0_ratio']['max'], 3)} (n {t['max_s0_ratio']['n']}).", "",
             "| seed | eligible home checks s > 0 / s = 0 | first s > 0 day (food / line) | first departure | departures | T_low | T_high (ratio) | T_eve |",
             "|---:|---|---|---|---:|---|---|---|"]
    for s in t["per_seed"]:
        pos, zero = s["home_checks_s_pos"], s["home_checks_s_zero"]
        first_pos = "—" if not pos else f"d{pos[0][0]} ({pos[0][1]} / {pos[0][2]})"
        dep = "never" if s["first_departure_tick"] is None else fmt_t(s["first_departure_tick"])
        ratio = "" if s["T_high_ratio"] is None else f" ({s['T_high_ratio']:.3f})"
        lines.append(f"| {s['seed']} | {len(pos)} / {len(zero)} | {first_pos} | {dep} | {len(s['departures'])} | "
                     f"{fmt_t(s['T_low'])} | {fmt_t(s['T_high'])}{ratio} | {fmt_t(s['T_eve'])} |")
    lines += ["", "Per-seed lists of every eligible home check (display day, food, line, ratio, s) are in `timeline.json`.", "",
              "Rule notes (review round 1, L2): the after-first-departure clause also applies when shortfall checks exist but none "
              "lies at least 3 days before the first departure; a candidate time outside [fork tick, end − 4 d) is skipped for the next "
              "one; cancelled departures count as departures; a seed that never departs has no departure bound on T_high. The three "
              "names are store states, not a time order: T_low falls on day 12 (the first eligible check) or day 29 (the first spring "
              "check, after T_eve), and T_high often falls after T_low."]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--summary", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--fork-tick", type=int, default=14_280)
    ap.add_argument("--days", type=int, default=120)
    a = ap.parse_args(argv)
    summary = json.loads(Path(a.summary).read_text())
    t = build(summary, a.fork_tick, a.days)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "timeline.json").write_text(json.dumps(t, indent=1) + "\n")
    (out / "timeline.md").write_text(markdown(t))
    print(json.dumps({"line_check": t["line_check"], "count": t["count"], "share": t["share"], "max_s0_ratio": t["max_s0_ratio"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
