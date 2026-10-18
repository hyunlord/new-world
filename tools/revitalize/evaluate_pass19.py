#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_pass19.py --candidate V24_DIR --baseline V23_DIR --out-md OUT.md --out-json OUT.json
"""Pass 19 H1-H4: game_v24 vs game_v23 (METRICS.md `## Pass 19`).

Both cohorts are complete `worldsim_probe --agent-trace` runs of seeds
19200-19231 for 120 days. H3's health rules (Pass 17 R2), H4's daytime idle
(Pass 17 R1's exposure) and F1/F5 reuse Pass 17's code unchanged; H1/H2 read
the routine trace (`<scene>_s<seed>_routine.jsonl`) and the people trace.
H5 (performance) and H6 (identity) are judged outside this script.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from collections.abc import Mapping, Sequence
from dataclasses import asdict
from pathlib import Path
from statistics import median
from typing import Any, Final

from evaluate_pass17_data import Cohort, CohortSpec, Criterion, Hour, Metric
from evaluate_pass17_family_read import read_family
from evaluate_pass17_read import load_cohort, read_jsonl
from evaluate_pass17_stage_a import health_metrics, idle_exposure, measure
from evaluate_pass17_stage_b import evaluate as evaluate_family, fraction_metric, split_exposure
from evaluate_pass17_statistics import Pair, UndefinedStatistic, pooled_ratio, pooled_spearman

SEEDS: Final = tuple(range(19200, 19232))
DAYS: Final = 120
WINDOW: Final = (960, 1200)  # [16:00, 20:00)
EVENING: Final = (1020, 1080, 1140)  # hourly samples 17:00, 18:00, 19:00
MIN_EXITS: Final = 5
F5_BAR: Final = Criterion(None, None, "reference", "share")


def metric_json(metric: Metric) -> dict[str, Any]:
    row = asdict(metric)
    row["ci95"] = list(metric.ci95) if metric.ci95 is not None else None
    return row


def routine_rows(cohort: Cohort, seed: int) -> list[Mapping[str, Any]]:
    return [row for row in read_jsonl(cohort.source / f"{cohort.scene}_s{seed}_routine.jsonl") if row["event"] == "ended"]


def clock_offset(cohort: Cohort) -> int:
    """The scene's clock offset from its first hourly trace row: display
    minute = (tick + offset) % 1440, display day = (tick + offset) // 1440 + 1."""
    seed = cohort.worlds[0].seed
    with (cohort.source / f"{cohort.scene}_s{seed}_hours.jsonl").open() as stream:
        row = json.loads(stream.readline())
    return (int(row["day"]) - 1) * 1440 + int(row["minute"]) - int(row["tick"])


def day_end_exit(row: Mapping[str, Any]) -> bool:
    return bool(row["completed"]) and row["until_tick"] is not None and row["tick"] < row["until_tick"]


def edge_maxima(rows: Sequence[Mapping[str, Any]], offset: int, days: int, keep=lambda row: True) -> tuple[list[int], list[int]]:
    """Per world-day (1..days): the largest number of selected ends in one
    tick inside the window (0 if none), and the minute of that tick (-1)."""
    per_tick: Counter = Counter()
    for row in rows:
        clock = row["tick"] + offset
        if WINDOW[0] <= clock % 1440 < WINDOW[1] and keep(row):
            per_tick[row["tick"]] += 1
    best: dict[int, tuple[int, int]] = {}
    for tick, count in per_tick.items():
        day = (tick + offset) // 1440 + 1
        if count > best.get(day, (0, -1))[0] or (count == best.get(day, (0, -1))[0] and tick < best.get(day, (0, 10**12))[1]):
            best[day] = (count, tick)
    maxima, minutes = [], []
    for day in range(1, days + 1):
        count, tick = best.get(day, (0, -1))
        maxima.append(count)
        minutes.append((tick + offset) % 1440 if tick >= 0 else -1)
    return maxima, minutes


def ratio_of_medians(pairs: Sequence[tuple[list[int], list[int]]]) -> float:
    candidate = median([value for pair in pairs for value in pair[0]])
    baseline = median([value for pair in pairs for value in pair[1]])
    if baseline <= 0:
        raise UndefinedStatistic("baseline median is zero")
    return candidate / baseline


def h1(candidate: Cohort, baseline: Cohort) -> dict[str, Any]:
    off_c, off_b = clock_offset(candidate), clock_offset(baseline)
    out: dict[str, Any] = {}
    selections = {"all_ends": lambda row: True, "day_end_exits": day_end_exit,
        "interrupted_ends": lambda row: not row["completed"]}
    for name, keep in selections.items():
        pairs, minutes_c, minutes_b = [], Counter(), Counter()
        for seed in (w.seed for w in candidate.worlds):
            rows_c, rows_b = routine_rows(candidate, seed), routine_rows(baseline, seed)
            maxima_c, at_c = edge_maxima(rows_c, off_c, candidate.days, keep)
            maxima_b, at_b = edge_maxima(rows_b, off_b, baseline.days, keep)
            pairs.append((maxima_c, maxima_b))
            minutes_c.update(m for m in at_c if m >= 0)
            minutes_b.update(m for m in at_b if m >= 0)
        criterion = Criterion(None, 0.15, "median per-world-day largest end tick in 16:00-20:00: v24 <= 0.15 x v23", "ratio")
        metric = measure(pairs, ratio_of_medians, criterion)
        out[name] = {"metric": metric_json(metric),
            "median_game_v24": median([v for pair in pairs for v in pair[0]]),
            "median_game_v23": median([v for pair in pairs for v in pair[1]]),
            "mean_game_v24": sum(v for pair in pairs for v in pair[0]) / sum(len(pair[0]) for pair in pairs),
            "mean_game_v23": sum(v for pair in pairs for v in pair[1]) / sum(len(pair[1]) for pair in pairs),
            "largest_tick_minute_game_v24": dict(sorted((f"{m // 60:02d}:{m % 60:02d}", n) for m, n in minutes_c.most_common(8))),
            "largest_tick_minute_game_v23": dict(sorted((f"{m // 60:02d}:{m % 60:02d}", n) for m, n in minutes_b.most_common(8)))}
    out["verdict"] = out["all_ends"]["metric"]["verdict"]
    return out


def people_p(cohort: Cohort, seed: int) -> dict[int, int]:
    rows = read_jsonl(cohort.source / f"{cohort.scene}_s{seed}_people.jsonl")
    return {int(row["id"]): int(row["temperament"]["p"]) for row in rows if row.get("temperament")}


def exit_pairs(cohort: Cohort, seed: int) -> tuple[list[Pair], list[int]]:
    offset = clock_offset(cohort)
    minutes: dict[int, list[int]] = defaultdict(list)
    all_minutes: list[int] = []
    for row in routine_rows(cohort, seed):
        if day_end_exit(row):
            minute = (row["tick"] + offset) % 1440
            minutes[int(row["id"])].append(minute)
            all_minutes.append(minute)
    p = people_p(cohort, seed)
    pairs = [(float(p[i]), sum(m) / len(m)) for i, m in sorted(minutes.items()) if len(m) >= MIN_EXITS and i in p]
    return pairs, all_minutes


def h2(candidate: Cohort, baseline: Cohort) -> dict[str, Any]:
    worlds, hist = [], Counter()
    for seed in (w.seed for w in candidate.worlds):
        pairs, minutes = exit_pairs(candidate, seed)
        worlds.append(pairs)
        hist.update(m // 30 * 30 for m in minutes)
    metric = measure(worlds, pooled_spearman, Criterion(0.5, None, "Spearman rho(P, mean day-end exit minute) >= 0.5", "rho"))
    base_hist = Counter()
    for seed in (w.seed for w in baseline.worlds):
        base_hist.update(m // 30 * 30 for m in exit_pairs(baseline, seed)[1])
    label = lambda m: f"{m // 60:02d}:{m % 60:02d}"
    return {"verdict": metric.verdict, "metric": metric_json(metric), "people": sum(len(w) for w in worlds),
        "exit_minutes_game_v24": {label(m): n for m, n in sorted(hist.items())},
        "exit_minutes_game_v23": {label(m): n for m, n in sorted(base_hist.items())}}


def evening_exposure(hours: Sequence[Hour]) -> Pair:
    evening = [hour for hour in hours if hour.minute in EVENING]
    return float(sum(hour.states[0][0] for hour in evening)), float(sum(sum(hour.states[0]) for hour in evening))


def h3(candidate: Cohort, baseline: Cohort) -> dict[str, Any]:
    health = health_metrics(candidate, baseline)
    families = tuple(read_family(candidate, world) for world in candidate.worlds)
    base_families = tuple(read_family(baseline, world) for world in baseline.worlds)
    f1 = evaluate_family(candidate, baseline, families).stage.metrics["F1"]
    f1_base = evaluate_family(baseline, baseline, base_families).stage.metrics["F1"]
    f5 = fraction_metric([split_exposure(world) for world in families], F5_BAR)
    f5_base = fraction_metric([split_exposure(world) for world in base_families], F5_BAR)
    diffs = {}
    for name, a, b in (("F1", f1, f1_base), ("F5", f5, f5_base)):
        diffs[name] = None if a.value is None or b.value is None else a.value - b.value
    family_ok = all(d is not None and abs(d) <= 0.05 for d in diffs.values())
    verdict = "PASS" if family_ok and all(m.verdict == "PASS" for m in health.values()) else "FAIL"
    return {"verdict": verdict, "health": {k: metric_json(m) for k, m in health.items()},
        "F1": metric_json(f1), "F1_game_v23": metric_json(f1_base), "F5": metric_json(f5), "F5_game_v23": metric_json(f5_base),
        "differences": diffs, "family_within_0_05": family_ok}


def h4(candidate: Cohort, baseline: Cohort) -> dict[str, Any]:
    ref = Criterion(None, None, "reference", "share")
    day_c = measure([idle_exposure(w.hours) for w in candidate.worlds], pooled_ratio, ref)
    day_b = measure([idle_exposure(w.hours) for w in baseline.worlds], pooled_ratio, ref)
    eve_c = measure([evening_exposure(w.hours) for w in candidate.worlds], pooled_ratio, ref)
    eve_b = measure([evening_exposure(w.hours) for w in baseline.worlds], pooled_ratio, ref)
    ok = day_c.value is not None and day_b.value is not None and day_c.value <= day_b.value + 0.02
    return {"verdict": "PASS" if ok else "FAIL", "daytime_game_v24": metric_json(day_c), "daytime_game_v23": metric_json(day_b),
        "difference": None if day_c.value is None or day_b.value is None else day_c.value - day_b.value,
        "evening_game_v24": metric_json(eve_c), "evening_game_v23": metric_json(eve_b)}


def fmt(metric: Mapping[str, Any], digits: int = 4) -> str:
    value, ci = metric["value"], metric["ci95"]
    if value is None:
        return f"ERROR ({metric.get('error')})"
    return f"{value:.{digits}f} [{ci[0]:.{digits}f}, {ci[1]:.{digits}f}]" if ci else f"{value:.{digits}f}"


def signed(value: float | None) -> str:
    return "n/a" if value is None else f"{value:+.4f}"


def markdown(report: Mapping[str, Any]) -> str:
    h = report["H"]
    a = h["H1"]["all_ends"]
    lines = ["# Pass 19 evaluation: game_v24 vs game_v23", "",
        f"Seeds {report['seeds'][0]}-{report['seeds'][-1]} ({len(report['seeds'])} worlds), {report['days']} days, paired by seed; Pass 17 definitions and bootstrap "
        "(2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 19`.", "",
        "| # | rule | game_v24 | game_v23 | verdict |", "|---|---|---|---|---|",
        f"| H1 | median per world-day of the largest end count in one tick, 16:00-20:00; ratio <= 0.15 | {a['median_game_v24']} (mean {a['mean_game_v24']:.2f}) | {a['median_game_v23']} (mean {a['mean_game_v23']:.2f}) | {h['H1']['verdict']} (ratio {fmt(a['metric'])}) |",
        f"| H2 | Spearman rho(P, mean day-end exit minute), >= {MIN_EXITS} exits, >= 0.5 | {fmt(h['H2']['metric'])} ({h['H2']['people']} people) | - | {h['H2']['verdict']} |"]
    for key, metric in h["H3"]["health"].items():
        lines.append(f"| H3 | {key}: {metric['bar']} | {metric['candidate']:.3f} | {metric['baseline']:.3f} | {metric['verdict']} (stat {fmt(metric)}) |")
    d = h["H3"]["differences"]
    lines.append(f"| H3 | F1 within +/-0.05 of game_v23 | {fmt(h['H3']['F1'])} | {fmt(h['H3']['F1_game_v23'])} | {'PASS' if d['F1'] is not None and abs(d['F1']) <= 0.05 else 'FAIL'} (diff {signed(d['F1'])}) |")
    lines.append(f"| H3 | F5 within +/-0.05 of game_v23 | {fmt(h['H3']['F5'])} | {fmt(h['H3']['F5_game_v23'])} | {'PASS' if d['F5'] is not None and abs(d['F5']) <= 0.05 else 'FAIL'} (diff {signed(d['F5'])}) |")
    lines.append(f"| H4 | adult daytime Idle share (08:00-17:00) <= v23 + 0.02 | {fmt(h['H4']['daytime_game_v24'])} | {fmt(h['H4']['daytime_game_v23'])} | {h['H4']['verdict']} (diff {signed(h['H4']['difference'])}) |")
    lines.append(f"| H4 | (reported) adult Idle share 17:00-19:00 | {fmt(h['H4']['evening_game_v24'])} | {fmt(h['H4']['evening_game_v23'])} | reported |")
    lines += ["", "## H1 diagnostics (reported, no verdict)", "", "| selection | median v24 | median v23 | mean v24 | mean v23 | ratio of medians |", "|---|---|---|---|---|---|"]
    for name in ("all_ends", "day_end_exits", "interrupted_ends"):
        row = h["H1"][name]
        lines.append(f"| {name} | {row['median_game_v24']} | {row['median_game_v23']} | {row['mean_game_v24']:.2f} | {row['mean_game_v23']:.2f} | {fmt(row['metric'])} |")
    lines += ["", "Clock minute of each world-day's largest tick (all ends), most common:", "",
        f"- game_v24: {h['H1']['all_ends']['largest_tick_minute_game_v24']}",
        f"- game_v23: {h['H1']['all_ends']['largest_tick_minute_game_v23']}", "",
        "## Day-end exit minutes (non-interrupted day-end exits, 30-minute bins)", "",
        f"- game_v24: {h['H2']['exit_minutes_game_v24']}", f"- game_v23: {h['H2']['exit_minutes_game_v23']}", "",
        "H5 (performance) and H6 (identity) are judged outside this script.", ""]
    return "\n".join(lines)


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    parser.add_argument("--seeds", default=f"{SEEDS[0]}..{SEEDS[-1] + 1}", help="A..B (default the registered 19200..19232)")
    parser.add_argument("--days", type=int, default=DAYS)
    args = parser.parse_args(argv)
    start, end = (int(x) for x in args.seeds.split(".."))
    seeds = tuple(range(start, end))
    candidate = load_cohort(args.candidate, CohortSpec("game_v24", seeds, args.days))
    baseline = load_cohort(args.baseline, CohortSpec("game_v23", seeds, args.days))
    report = {"schema_version": 19, "candidate": "game_v24", "baseline": "game_v23", "days": args.days, "seeds": list(seeds),
        "H": {"H1": h1(candidate, baseline), "H2": h2(candidate, baseline), "H3": h3(candidate, baseline), "H4": h4(candidate, baseline)}}
    args.out_json.write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
    text = markdown(report)
    args.out_md.write_text(text)
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
