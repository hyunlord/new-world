#!/usr/bin/env python3
"""Trace the first winter of two seeds from a schema-6 120-day round. Stdlib only.

usage: winter_trace.py --round120 DIR --out FILE.md [--winter-burn-per-day N] [--urgent-start HH:MM]

Picks the seed with the most winter nights out over the whole run and the one
with the fewest (ties: lower seed), then lists every 120-tick sample of its first
winter: display days 22-28 (one-based; zero-based 21-27) and the last winter
night, which ends at 07:00 on display day 29 (nights are booked to the evening
they began, as in `evaluate_pass6.py`). Columns: wood in hand, fire lit, cold
members, wood gatherers out, the urgent flag and the food store. Five plain
sentences per seed summarise what happened.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

from evaluate_pass4 import EvidenceError, Seed, quantiles
from evaluate_pass6 import day_index, load_round, winters_table

BURN_END = 420  # 07:00, the end of the burn window and of a night
NIGHT = 120  # 02:00, the night sample of Y7
DINNER = 1080  # 18:00
BREAKFAST_END = 480  # 08:00


def clock(row: dict) -> str:
    minute = int(row["minute_of_day"])
    return f"{minute // 60:02d}:{minute % 60:02d}"


def first_winter(seed: Seed, dps: int) -> list[dict]:
    """Periodic rows from 00:00 of the first winter day to 07:00 after its last evening."""
    first, last = 3 * dps, 4 * dps - 1
    rows = []
    for row in seed.periodic:
        day, minute = day_index(row), int(row["minute_of_day"])
        if first <= day <= last or (day == last + 1 and minute < BURN_END):
            rows.append(row)
    return rows


def pick_seeds(winters: list[dict]) -> tuple[int, int]:
    """(most winter nights out, fewest), ties to the lower seed."""
    totals: dict[int, int] = {}
    for w in winters:
        totals[w["seed"]] = totals.get(w["seed"], 0) + w["nights_out"]
    most = min(totals, key=lambda s: (-totals[s], s))
    fewest = min(totals, key=lambda s: (totals[s], s))
    return most, fewest


def night_drop(rows: list[dict], evening_day: int) -> float | None:
    """Food store at 18:00 of `evening_day` minus the store at 08:00 the next morning."""
    at = {(day_index(r), int(r["minute_of_day"])): float(r["stockpile_food"]) for r in rows}
    evening, morning = at.get((evening_day, DINNER)), at.get((evening_day + 1, BREAKFAST_END))
    return None if evening is None or morning is None else evening - morning


def story(seed: int, rows: list[dict], winter: dict, burn: float, dps: int, every: list[dict],
          urgent_start: str = "09:00") -> list[str]:
    """Five sentences about one seed's first winter; `every` is all of the seed's
    periodic rows (the morning after the last winter night lies past the table)."""
    first = 3 * dps
    start = rows[0]
    # Dark nights as in Y7: the fire is out at 02:00; the night belongs to the evening before.
    out_nights = sorted(day_index(r) - 1 for r in rows if int(r["minute_of_day"]) == NIGHT and not r["fire_lit"])
    sentences = [
        f"Seed {seed} began its first winter (display day {first + 1}) with {start['wood_in_hand']:.0f} wood in hand, "
        f"about {float(start['wood_in_hand']) / burn:.1f} nights of winter burn at {burn:.0f} a night, and {start['stockpile_food']:.0f} food in store.",
    ]
    if out_nights:
        evening = out_nights[0]
        night = [r for r in rows if (day_index(r) == evening and int(r["minute_of_day"]) >= DINNER)
                 or (day_index(r) == evening + 1 and int(r["minute_of_day"]) < BURN_END)]
        two = next(i for i, r in enumerate(night) if int(r["minute_of_day"]) == NIGHT)
        first_dark = two
        while first_dark > 0 and not night[first_dark - 1]["fire_lit"]:
            first_dark -= 1
        lit_before = any(r["fire_lit"] for r in night[:two])
        names = ", ".join(f"display day {d + 1}" for d in out_nights)
        when = f"the fire died at {clock(night[first_dark])}" if lit_before else "the fire was never lit that evening"
        sentences.append(f"The wood ran out on the night of display day {evening + 1}: {when} "
                         f"and {night[two]['cold_members']:.0f} members were cold at 02:00; dark nights (fire out at 02:00) fell on {names}.")
        morning = [r for r in every if day_index(r) == evening + 1 and BURN_END <= int(r["minute_of_day"]) <= 720]
        out_now = max((int(r["wood_gatherers_now"]) for r in morning), default=0)
        urgent = any(int(r["urgent_wood"]) for r in morning)
        sentences.append(f"The next morning {out_now} {'adult was' if out_now == 1 else 'adults were'} out for wood at once by noon"
                         f"{f' in urgent mode (its window opens at {urgent_start})' if urgent else ''}.")
    else:
        sentences.append("The fire stayed lit through every winter night.")
        busiest = max(rows, key=lambda r: int(r["wood_gatherers_now"]))
        sentences.append(f"At most {busiest['wood_gatherers_now']:.0f} {'adult was' if int(busiest['wood_gatherers_now']) == 1 else 'adults were'} out for wood at once "
                         f"(display day {day_index(busiest) + 1}, {clock(busiest)}).")
    sentences.append(f"Over the winter adults made {winter['wood_trips']:.0f} wood trips ({winter['urgent_trips']:.0f} urgent) "
                     f"and brought in {winter['wood_gathered']:.0f} units, a few branches a day.")
    drops = [d for d in (night_drop(every, e) for e in range(first, first + dps)) if d is not None]
    lit_drops = [d for e, d in ((e, night_drop(every, e)) for e in range(first, first + dps))
                 if d is not None and e not in out_nights]
    end = rows[-1]
    if out_nights and lit_drops:
        cold_drop = [d for d in (night_drop(every, e) for e in out_nights) if d is not None]
        cost = (quantiles(cold_drop)["median"] - quantiles(lit_drops)["median"]) if cold_drop else 0.0
        sentences.append(f"A cold night took {quantiles(cold_drop)['median']:.0f} food from the store between dinner and breakfast "
                         f"against {quantiles(lit_drops)['median']:.0f} on a lit winter night ({cost:+.0f}); "
                         f"the store ended the winter at {end['stockpile_food']:.0f}.")
    else:
        median = quantiles(drops)["median"] if drops else 0.0
        sentences.append(f"With no cold night the store fell by about {median:.0f} between dinner and breakfast "
                         f"and ended the winter at {end['stockpile_food']:.0f}.")
    return sentences


def table(rows: list[dict]) -> list[str]:
    lines = ["| Display day | Time | Wood in hand | Fire lit | Cold members | Wood gatherers | Urgent | Food store |",
             "|---:|---|---:|---:|---:|---:|---:|---:|"]
    for r in rows:
        lines.append(f"| {day_index(r) + 1} | {clock(r)} | {r['wood_in_hand']:.0f} | {int(r['fire_lit'])} | {int(r['cold_members'])} | "
                     f"{int(r['wood_gatherers_now'])} | {int(r['urgent_wood'])} | {r['stockpile_food']:.0f} |")
    return lines


def render(round_dir: Path, burn: float, days: int = 120, urgent_start: str = "09:00") -> str:
    r = load_round(round_dir, days)
    dps = next((int(row["days_per_season"]) for s in r.seeds for row in s.periodic if row["days_per_season"]), 0)
    if dps == 0:
        raise EvidenceError("winter trace needs seasons")
    winters = winters_table(r, 360)
    most, fewest = pick_seeds(winters)
    lines = ["# A winter in detail", "",
             f"Source: `{round_dir}` ({r.summary['scene']}, seeds {r.summary['seeds']['start']}..{r.summary['seeds']['end_exclusive']}, "
             f"{days} days). Two seeds: the one with the most winter nights out over the run (seed {most}) and the one with the "
             f"fewest (seed {fewest}); ties go to the lower seed. Each table lists the 120-tick samples of the first winter, "
             f"display days {3 * dps + 1}–{4 * dps} and the last winter night up to 07:00 of display day {4 * dps + 1} "
             "(nights belong to the evening they began). Wood in hand is the store plus wood carried by members; fire lit and cold "
             "members come from the hourly fire update; wood gatherers are members seeking or picking up deadwood; urgent is the "
             "settlement's urgent flag (tonight's burn not in hand).", ""]
    for label, seed_id in (("Most winter nights out", most), ("Fewest winter nights out", fewest)):
        seed = next(s for s in r.seeds if s.seed == seed_id)
        rows = first_winter(seed, dps)
        winter = next(w for w in winters if w["seed"] == seed_id and w["year"] == 0)
        total = sum(w["nights_out"] for w in winters if w["seed"] == seed_id)
        lines += [f"## Seed {seed_id} — {label.lower()} ({total} over the run, {winter['nights_out']} in the first winter)", ""]
        lines += [" ".join(story(seed_id, rows, winter, burn, dps, seed.periodic, urgent_start)), ""]
        lines += table(rows) + [""]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--round120", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--days", type=int, default=120)
    parser.add_argument("--winter-burn-per-day", type=float, default=39.0)
    parser.add_argument("--urgent-start", default="09:00", help="urgent_start_minute of game_v12() as HH:MM")
    args = parser.parse_args(argv)
    try:
        text = render(args.round120, args.winter_burn_per_day, args.days, args.urgent_start)
    except (EvidenceError, OSError, KeyError, ValueError, StopIteration) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(text)
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
