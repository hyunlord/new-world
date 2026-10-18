#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_scale_p19.py --input SCALE_RAW_DIR --out-md OUT.md --out-json OUT.json
"""Pass 19 Part 3: scale check of game_v24 (500 / 1,000 / 2,000 founders on a
64 x 64 map, 2,000 on 128 x 128; two days, fed; one process per size and
repeat). Medians over the repeats; per-system cost per tick and its log-log
slope over 500 -> 2,000 (x = mean alive at 06:00, as Passes 15-18); peak RSS
from /usr/bin/time -l; chronicle size; the live game's simulation CPU per
real second at 1x / 2x / 4x (30 ticks per second at 1x). S1-S4 verdicts are
reported, not acted on.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import statistics
import sys
from pathlib import Path
from typing import Any, Final

from perf_p15 import loglog

FIXED_TICKS_PER_SECOND: Final = 30  # FIXED_DT = 1/30 (world_node.rs)
MAX_ITERS_PER_FRAME: Final = 5
SPEEDS: Final = (1, 2, 4)
LABELS: Final = ("n500_side64", "n1000_side64", "n2000_side64", "n2000_side128")


def time_field(path: Path, label: str) -> int:
    match = re.search(r"(\d+)\s+" + label, path.read_text())
    if match is None:
        raise ValueError(f"no {label} in {path}")
    return int(match.group(1))


def rss_bytes(path: Path) -> int:
    return time_field(path, "maximum resident set size")


def load(run_dir: Path) -> dict[str, Any]:
    [path] = [p for p in run_dir.glob("stress_stress_v24_*.json")]
    run = json.loads(path.read_text())
    ticks = run["tick_summary"]["ticks"]
    systems = {s["name"]: s["total_ms"] / ticks for s in run["systems"]}
    return {"file": str(path), "mean_ms": run["tick_summary"]["mean_ms"], "p999_ms": run["tick_summary"]["p999_ms"],
        "max_ms": run["tick_summary"]["max_ms"], "alive": run["mean_alive_0600"], "alive_end": run["alive_end"],
        "founders": run["founders"], "settlements_end": run["settlements_end"],
        "settlement_sizes_end": run.get("settlement_sizes_end"), "chronicle_entries_end": run.get("chronicle_entries_end"),
        "chronicle_people_end": run.get("chronicle_people_end"), "chronicle_bytes_end": run.get("chronicle_approx_bytes_end"),
        "systems_ms_per_tick": systems, "rss_bytes": rss_bytes(run_dir / "time.log"),
        "footprint_bytes": time_field(run_dir / "time.log", "peak memory footprint"), "wall_seconds": run["wall_seconds"],
        "fingerprint": run["fingerprint"], "contended": run["contended"], "subprofile": run.get("subprofile")}


def median_of(rows: list[dict[str, Any]], key: str) -> float:
    return statistics.median(float(row[key]) for row in rows)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    repeats = sorted(p.name for p in args.input.glob("r*") if p.is_dir())
    runs = {label: [load(args.input / r / label) for r in repeats if (args.input / r / label).exists()] for label in LABELS}
    sizes: dict[str, Any] = {}
    for label, rows in runs.items():
        fingerprints = {row["fingerprint"] for row in rows}
        names = sorted(rows[0]["systems_ms_per_tick"])
        sizes[label] = {"repeats": len(rows), "fingerprints_equal": len(fingerprints) == 1,
            "mean_ms": median_of(rows, "mean_ms"), "p999_ms": median_of(rows, "p999_ms"), "max_ms": median_of(rows, "max_ms"),
            "alive": median_of(rows, "alive"), "alive_end": rows[0]["alive_end"], "rss_bytes": median_of(rows, "rss_bytes"),
            "rss_bytes_max": max(row["rss_bytes"] for row in rows),
            "footprint_bytes": median_of(rows, "footprint_bytes"), "settlements_end": rows[0]["settlements_end"],
            "settlement_sizes_end": rows[0]["settlement_sizes_end"], "chronicle_entries_end": rows[0]["chronicle_entries_end"],
            "chronicle_people_end": rows[0]["chronicle_people_end"], "chronicle_bytes_end": rows[0]["chronicle_bytes_end"],
            "systems_ms_per_tick": {name: statistics.median(row["systems_ms_per_tick"][name] for row in rows) for name in names},
            "contended": any(row["contended"] for row in rows),
            "per_repeat": [{k: row[k] for k in ("mean_ms", "p999_ms", "max_ms", "rss_bytes", "wall_seconds")} for row in rows]}
    # Per-system slopes over the three side-64 sizes (x = mean alive at 06:00).
    side64 = [sizes[label] for label in LABELS[:3]]
    xs = [row["alive"] for row in side64]
    slopes: dict[str, Any] = {}
    for name in side64[2]["systems_ms_per_tick"]:
        ys = [row["systems_ms_per_tick"][name] for row in side64]
        fit = loglog(xs, ys)
        share = side64[2]["systems_ms_per_tick"][name] / side64[2]["mean_ms"]
        endpoint = (math.log(ys[2] / ys[0]) / math.log(xs[2] / xs[0])) if ys[0] > 0 and ys[2] > 0 else None
        slopes[name] = {"ms_per_tick": ys, "share_at_2000": share, "slope_loglog": fit["b"] if fit else None, "slope_endpoints": endpoint}
    tick_fit = loglog(xs, [row["mean_ms"] for row in side64])
    # S1: systems with >= 5 % of the 2,000-person tick need slope < 1.7.
    heavy = {name: s for name, s in slopes.items() if s["share_at_2000"] >= 0.05}
    s1_findings = [name for name, s in heavy.items() if s["slope_loglog"] is None or s["slope_loglog"] >= 1.7]
    cpu = {label: {f"{speed}x": FIXED_TICKS_PER_SECOND * speed * sizes[label]["mean_ms"] / 1000 for speed in SPEEDS} for label in LABELS}
    caps = {label: {f"{speed}x": {"ticks_per_second": FIXED_TICKS_PER_SECOND * speed,
        "min_fps_before_iter_cap": FIXED_TICKS_PER_SECOND * speed / MAX_ITERS_PER_FRAME,
        "sim_ms_per_60fps_frame": FIXED_TICKS_PER_SECOND * speed / 60 * sizes[label]["mean_ms"]} for speed in SPEEDS} for label in LABELS}
    sub = None
    sub_dir = args.input / "subprofile" / "n2000_side64"
    if sub_dir.exists():
        full = load(sub_dir)
        sub = {k: full["subprofile"][k] for k in ("scopes", "slowest_ticks", "steady_ticks")}
        sub["mean_ms"] = full["mean_ms"]
        sub["decision_ms_per_tick"] = full["systems_ms_per_tick"].get("AgentDecisionSystem")
    verdicts = {"S1": "PASS" if not s1_findings else "FINDING", "S2": "PASS" if sizes["n1000_side64"]["mean_ms"] <= 4.0 else "FAIL",
        "S3": "PASS" if cpu["n1000_side64"]["2x"] <= 0.5 else "FAIL", "S4": "reported"}
    out = {"repeats": repeats, "sizes": sizes, "slopes": slopes, "tick_slope_loglog": tick_fit["b"] if tick_fit else None,
        "s1_heavy_systems": sorted(heavy), "s1_findings": s1_findings, "cpu_share_of_one_core": cpu, "iteration_cap": caps,
        "subprofile_n2000": sub, "verdicts": verdicts}
    args.out_json.write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    lines = ["# Pass 19 scale check (game_v24 stress worlds)", "",
        f"Repeats: {', '.join(repeats)} (medians). One camp of N founders around a single hearth (not villages of <= 50).", "",
        "| world | mean ms | p99.9 ms | max ms | mean alive 06:00 | peak RSS MB / peak footprint MB | settlements at end (sizes) | chronicle entries / people / KB | CPU 1x / 2x / 4x (cores) |",
        "|---|---|---|---|---|---|---|---|---|"]
    for label in LABELS:
        s = sizes[label]
        c = cpu[label]
        lines.append(f"| {label} | {s['mean_ms']:.3f} | {s['p999_ms']:.3f} | {s['max_ms']:.3f} | {s['alive']:.0f} | {s['rss_bytes'] / 2**20:.1f} / {s['footprint_bytes'] / 2**20:.1f} | "
            f"{s['settlements_end']} ({s['settlement_sizes_end']}) | {s['chronicle_entries_end']} / {s['chronicle_people_end']} / {s['chronicle_bytes_end'] / 1024:.1f} | "
            f"{c['1x']:.3f} / {c['2x']:.3f} / {c['4x']:.3f} |")
    lines += ["", f"Whole-tick slope (log-log, 500 -> 2,000 at side 64): {out['tick_slope_loglog']:.3f}", "",
        "| system | ms/tick 500 | 1,000 | 2,000 | share at 2,000 | slope (log-log) | slope (endpoints) |", "|---|---|---|---|---|---|---|"]
    for name, s in sorted(slopes.items(), key=lambda kv: -kv[1]["ms_per_tick"][2]):
        if s["ms_per_tick"][2] < 0.001:
            continue
        b = "n/a" if s["slope_loglog"] is None else f"{s['slope_loglog']:.3f}"
        e = "n/a" if s["slope_endpoints"] is None else f"{s['slope_endpoints']:.3f}"
        lines.append(f"| {name} | {s['ms_per_tick'][0]:.4f} | {s['ms_per_tick'][1]:.4f} | {s['ms_per_tick'][2]:.4f} | {s['share_at_2000']:.3f} | {b} | {e} |")
    lines += ["", "| rule | verdict | detail |", "|---|---|---|",
        f"| S1 | {verdicts['S1']} | systems >= 5 % at 2,000: {', '.join(out['s1_heavy_systems'])}; slope >= 1.7: {', '.join(s1_findings) or 'none'} |",
        f"| S2 | {verdicts['S2']} | mean tick at 1,000 = {sizes['n1000_side64']['mean_ms']:.3f} ms (<= 4.0) |",
        f"| S3 | {verdicts['S3']} | 1,000 people at 2x (60 ticks/s) = {cpu['n1000_side64']['2x']:.3f} of one core (<= 0.5); 4x {cpu['n1000_side64']['4x']:.3f} |",
        f"| S4 | reported | peak RSS at 2,000 = {sizes['n2000_side64']['rss_bytes'] / 2**20:.1f} MB (side 64), {sizes['n2000_side128']['rss_bytes'] / 2**20:.1f} MB (side 128) — flat across sizes (mapped pages dominate); peak memory footprint {sizes['n500_side64']['footprint_bytes'] / 2**20:.1f} / {sizes['n1000_side64']['footprint_bytes'] / 2**20:.1f} / {sizes['n2000_side64']['footprint_bytes'] / 2**20:.1f} MB at 500 / 1,000 / 2,000 (side 128: {sizes['n2000_side128']['footprint_bytes'] / 2**20:.1f} MB); chronicle {sizes['n2000_side64']['chronicle_entries_end']} entries / {sizes['n2000_side64']['chronicle_bytes_end'] / 1024:.1f} KB after 2 days |", ""]
    if sub is not None:
        lines += ["## AgentDecisionSystem scopes at 2,000 people (subprofile build, one run, side 64)", "",
            f"Whole tick {sub['mean_ms']:.3f} ms, decision system {sub['decision_ms_per_tick']:.3f} ms per tick (the subprofile build's own timing).", "",
            "| scope | mean µs per tick | mean calls per tick | max µs in a tick |", "|---|---|---|---|"]
        for name, scope in sorted(sub["scopes"].items(), key=lambda kv: -kv[1]["mean_us_per_tick"]):
            if scope["mean_us_per_tick"] >= 1.0:
                lines.append(f"| {name} | {scope['mean_us_per_tick']:.1f} | {scope['mean_calls_per_tick']:.1f} | {scope['max_us_per_tick']:.1f} |")
        lines += ["", "Slowest ticks: " + "; ".join(f"{t['clock']} day {t['display_day']} {t['total_ms']:.2f} ms" for t in sub["slowest_ticks"][:6]), ""]
    args.out_md.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
