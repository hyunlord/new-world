#!/usr/bin/env python3
# How to run: python3 tools/revitalize/attribution_p18.py RAW_DIR OUT.md OUT.json [--label TEXT]
"""Pass 18 section 6.2: tables from perf_attr_p18.sh's runs.

RAW_DIR holds plain/r{1,2,3} and sub/r{1,2,3}, each with one 500-agent fed
stress day per rule (seed 15100). Plain runs give the timings (median of the
three repeats); subprofile runs give per-scope costs (median of the repeats'
per-tick means) and the slowest ticks' scope breakdown; their ratio to the
plain runs is the feature's own overhead.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from statistics import median
from typing import Any

RULES = ("game_v20", "game_v21", "game_v22")


def load(raw: Path, build: str, rules: str) -> list[dict[str, Any]]:
    version = rules.removeprefix("game_v")
    return [json.loads((raw / build / f"r{r}" / f"stress_stress_v{version}_s15100_n500_side64_fed.json").read_text()) for r in (1, 2, 3)]


def decision(run: dict[str, Any]) -> dict[str, Any]:
    return next(row for row in run["systems"] if row["name"] == "AgentDecisionSystem")


def summarize(raw: Path) -> dict[str, Any]:
    out: dict[str, Any] = {"timing": {}, "overhead": {}, "scopes": {}, "slowest": {}, "fingerprints": {}}
    for rules in RULES:
        plain, sub = load(raw, "plain", rules), load(raw, "sub", rules)
        out["fingerprints"][rules] = sorted({run["fingerprint"] for run in plain + sub})
        out["timing"][rules] = {
            "mean_ms": median(run["tick_summary"]["mean_ms"] for run in plain),
            "p999_ms": median(run["tick_summary"]["p999_ms"] for run in plain),
            "max_ms": median(run["tick_summary"]["max_ms"] for run in plain),
            "decision_us_per_call": median(decision(run)["mean_us_per_call"] for run in plain),
            "decision_max_us": median(decision(run)["max_us"] for run in plain),
            "repeats_mean_ms": [run["tick_summary"]["mean_ms"] for run in plain],
            "repeats_p999_ms": [run["tick_summary"]["p999_ms"] for run in plain],
            "contended": [run["contended"] for run in plain + sub],
            "slowest_ticks_plain": [[{"tick": t["tick"], "clock": t["clock"], "ms": t["total_ms"]} for t in run["slowest_ticks"][:6]] for run in plain],
        }
        out["overhead"][rules] = {
            "sub_mean_ms": median(run["tick_summary"]["mean_ms"] for run in sub),
            "ratio": median(run["tick_summary"]["mean_ms"] for run in sub) / out["timing"][rules]["mean_ms"],
            "decision_ratio": median(decision(run)["mean_us_per_call"] for run in sub) / out["timing"][rules]["decision_us_per_call"],
        }
        names = sorted({name for run in sub for name in run["subprofile"]["scopes"]})
        out["scopes"][rules] = {name: {
            "us_per_tick": median(run["subprofile"]["scopes"].get(name, {}).get("mean_us_per_tick", 0.0) for run in sub),
            "calls_per_tick": median(run["subprofile"]["scopes"].get(name, {}).get("mean_calls_per_tick", 0.0) for run in sub),
            "max_us_per_tick": median(run["subprofile"]["scopes"].get(name, {}).get("max_us_per_tick", 0.0) for run in sub),
        } for name in names}
        out["slowest"][rules] = [[{"tick": t["tick"], "clock": t["clock"], "ms": t["total_ms"],
            "scopes": {k: [round(v["us"], 1), v["calls"]] for k, v in t["scopes"].items() if v["us"] >= 20 or v["calls"] and not v["us"]}}
            for t in run["subprofile"]["slowest_ticks"][:5]] for run in sub]
    return out


def markdown(data: dict[str, Any], label: str) -> str:
    t = data["timing"]
    lines = [f"# Pass 18 attribution ({label})", "",
        "500 agents, 64 × 64, fed, seed 15100, one day (1,380 measured ticks after 60 warm-up ticks); "
        "three interleaved repeats per rule and build under the quiet lock. Plain = default build "
        "(subprofile off); medians of the three repeats.", "",
        "| rules | mean tick ms | p99.9 ms | max ms | decision µs/call | repeats (mean / p99.9) |", "|---|---|---|---|---|---|"]
    for rules in RULES:
        row = t[rules]
        lines.append(f"| {rules} | {row['mean_ms']:.4f} | {row['p999_ms']:.3f} | {row['max_ms']:.3f} | {row['decision_us_per_call']:.1f} | "
            f"{', '.join(f'{a:.3f}' for a in row['repeats_mean_ms'])} / {', '.join(f'{a:.3f}' for a in row['repeats_p999_ms'])} |")
    base = t["game_v20"]
    lines += ["", "| ratio | mean | p99.9 | decision µs/call |", "|---|---|---|---|"]
    for rules in RULES[1:]:
        lines.append(f"| {rules} / game_v20 | {t[rules]['mean_ms'] / base['mean_ms']:.3f} | {t[rules]['p999_ms'] / base['p999_ms']:.3f} | "
            f"{t[rules]['decision_us_per_call'] - base['decision_us_per_call']:+.1f} µs |")
    lines += ["", "## The feature's own overhead (subprofile build / plain build, medians)", "",
        "| rules | plain mean ms | subprofile mean ms | tick ratio | decision ratio |", "|---|---|---|---|---|"]
    for rules in RULES:
        o = data["overhead"][rules]
        lines.append(f"| {rules} | {t[rules]['mean_ms']:.4f} | {o['sub_mean_ms']:.4f} | {o['ratio']:.3f} | {o['decision_ratio']:.3f} |")
    names = sorted({name for rules in RULES for name in data["scopes"][rules]})
    lines += ["", "## Scopes (subprofile build; median of the repeats' per-tick means)", "",
        "| scope | v20 µs/tick | v21 µs/tick | v22 µs/tick | v21 − v20 | v21 calls/tick | v21 max µs/tick |", "|---|---|---|---|---|---|---|"]
    for name in names:
        get = lambda rules, key: data["scopes"][rules].get(name, {}).get(key, 0.0)
        lines.append(f"| `{name}` | {get('game_v20', 'us_per_tick'):.2f} | {get('game_v21', 'us_per_tick'):.2f} | {get('game_v22', 'us_per_tick'):.2f} | "
            f"{get('game_v21', 'us_per_tick') - get('game_v20', 'us_per_tick'):+.2f} | {get('game_v21', 'calls_per_tick'):.2f} | {get('game_v21', 'max_us_per_tick'):.1f} |")
    lines += ["", "Scopes nest: the exit test and start arm are inside `decision_round_main`; `routine_places_collect` is inside `routine_new_maps` (Pass 17 code) or inside the first `place()` (Pass 18 code); counters (no time) are listed with 0 µs.", ""]
    for rules in RULES[1:]:
        lines += [f"## Slowest ticks, {rules} (subprofile runs; scopes ≥ 20 µs as [µs, calls])", ""]
        for index, ticks in enumerate(data["slowest"][rules], 1):
            lines.append(f"- repeat {index}: " + "; ".join(f"tick {x['tick']} {x['clock']} {x['ms']:.3f} ms {json.dumps(x['scopes'])}" for x in ticks[:3]))
        lines += ["", f"Plain-build slowest ticks, {rules}: " + " | ".join(
            ", ".join(f"{x['clock']} {x['ms']:.3f}" for x in repeat[:4]) for repeat in t[rules]["slowest_ticks_plain"]), ""]
    lines += ["Fingerprints (plain and subprofile, all repeats): " + "; ".join(f"{k}: {', '.join(v)}" for k, v in data["fingerprints"].items()), ""]
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    if len(argv) not in (4, 6):
        print(__doc__, file=sys.stderr)
        return 2
    label = argv[5] if len(argv) == 6 and argv[4] == "--label" else "Part 1"
    data = summarize(Path(argv[1]))
    Path(argv[3]).write_text(json.dumps(data, indent=2) + "\n")
    Path(argv[2]).write_text(markdown(data, label))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
