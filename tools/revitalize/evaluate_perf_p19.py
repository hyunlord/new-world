#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_perf_p19.py --stage c6|h5 --input RAW_DIR --out-md OUT.md --out-json OUT.json
"""Pass 19 timing: C6 (chronicle write cost, base vs Part 1 build, game_v23,
three repeats) and H5 (game_v24 vs game_v20, five repeats, medians; slopes;
the 18:00-window ticks beside game_v20's and game_v23's).

Input is a `perf_p19.sh` directory measured inside `quiet_lock_p19.sh`.
p99.9 is perf.rs's (second-largest of the measured ticks). The 18:00 window
is every simulation tick whose clock minute is in [17:00, 19:00]
(`(tick + clock_offset_ticks) % 1440`), read from the `.tick_us.bin` series
(little-endian u32 microseconds from tick 0).
"""
from __future__ import annotations

import argparse
import json
import statistics
import struct
import sys
from decimal import Decimal
from pathlib import Path
from typing import Any

from perf_p15 import loglog

WINDOW = (1020, 1140)


def run_file(directory: Path, version: str, n: int = 500, mode: str = "stress") -> Path:
    return directory / f"{mode}_stress_v{version}_s15100_n{n}_side64_fed.json"


def window(path: Path) -> dict[str, Any]:
    run = json.loads(path.read_text())
    offset, tpd = run["clock_offset_ticks"], run["ticks_per_day"]
    raw = path.with_name(path.name.removesuffix(".json") + ".tick_us.bin").read_bytes()
    ticks = struct.unpack(f"<{len(raw) // 4}I", raw)
    warm = run["warmup"]["ticks"]
    rows = [(i, us) for i, us in enumerate(ticks) if i >= warm and WINDOW[0] <= (i + offset) % tpd <= WINDOW[1]]
    six = [us for i, us in rows if (i + offset) % tpd == 1080]
    return {"ticks": len(rows), "mean_ms": statistics.fmean(us for _, us in rows) / 1000,
        "max_ms": max(us for _, us in rows) / 1000, "tick_1800_ms": six[0] / 1000 if six else None,
        "top5_ms": sorted((us / 1000 for _, us in rows), reverse=True)[:5],
        "top5_minutes": [f"{((i + offset) % tpd) // 60:02d}:{((i + offset) % tpd) % 60:02d}" for i, _ in sorted(rows, key=lambda r: -r[1])[:5]]}


def summary(path: Path) -> dict[str, Any]:
    run = json.loads(path.read_text())
    decision = [s for s in run["systems"] if s["name"] == "AgentDecisionSystem"][0]
    return {"file": str(path), "mean_ms": run["tick_summary"]["mean_ms"], "p999_ms": run["tick_summary"]["p999_ms"],
        "max_ms": run["tick_summary"]["max_ms"], "alive": run["mean_alive_0600"], "decision_us": decision["mean_us_per_call"],
        "fingerprint": run["fingerprint"], "contended": run["contended"], "git_sha": run["git_sha"]}


def rel(new: float, old: float) -> float:
    return float(Decimal(str(new)) / Decimal(str(old)) - 1)


def c6(root: Path) -> dict[str, Any]:
    repeats = sorted(p.name for p in root.glob("r[0-9]*") if p.is_dir())
    rows = {role: [summary(run_file(root / r / role, "23")) for r in repeats] for role in ("base", "part1")}
    med = {role: {k: statistics.median(x[k] for x in v) for k in ("mean_ms", "p999_ms")} for role, v in rows.items()}
    fps = {role: {x["fingerprint"] for x in v} for role, v in rows.items()}
    return {"repeats": rows, "medians": med, "mean_relative": rel(med["part1"]["mean_ms"], med["base"]["mean_ms"]),
        "p999_relative": rel(med["part1"]["p999_ms"], med["base"]["p999_ms"]),
        "per_repeat_mean_relative": [rel(p["mean_ms"], b["mean_ms"]) for b, p in zip(rows["base"], rows["part1"])],
        "same_fingerprint": len(fps["base"] | fps["part1"]) == 1, "expected": "within +/-3 %"}


def h5(root: Path) -> dict[str, Any]:
    repeats = sorted(p.name for p in root.glob("r[0-9]*") if p.is_dir())
    rows, windows = {}, {}
    for rules in ("game_v20", "game_v23", "game_v24"):
        version = rules.removeprefix("game_v")
        rows[rules] = [summary(run_file(root / r / rules, version)) for r in repeats]
        windows[rules] = [window(run_file(root / r / rules, version)) for r in repeats]
    med = {rules: {k: statistics.median(x[k] for x in v) for k in ("mean_ms", "p999_ms")} for rules, v in rows.items()}
    win = {rules: {k: statistics.median(x[k] for x in v) for k in ("mean_ms", "max_ms", "tick_1800_ms")} for rules, v in windows.items()}
    slopes = {}
    for rules in ("game_v24", "game_v20"):
        points = [summary(run_file(root / f"slope_{rules}", rules.removeprefix("game_v"), n)) for n in (100, 500, 1000)]
        fit = loglog([p["alive"] for p in points], [p["decision_us"] for p in points])
        slopes[rules] = {"b": fit["b"], "points": points}
    checks = {
        "mean_relative": {"value": rel(med["game_v24"]["mean_ms"], med["game_v20"]["mean_ms"]), "limit": 0.15, "op": "<="},
        "p999_relative": {"value": rel(med["game_v24"]["p999_ms"], med["game_v20"]["p999_ms"]), "limit": 0.35, "op": "<="},
        "mean_absolute_ms": {"value": med["game_v24"]["mean_ms"], "limit": 4.0, "op": "<="},
        "decision_slope": {"value": slopes["game_v24"]["b"], "limit": 1.7, "op": "<"},
    }
    for check in checks.values():
        check["passed"] = check["value"] < check["limit"] if check["op"] == "<" else check["value"] <= check["limit"]
    quick = {role: summary(root / f"quick_{role}" / "quick_stress_v19_s15100_n500_side64_fed.json") for role in ("base", "final")}
    per_repeat = [{"repeat": r, "mean_relative": rel(rows["game_v24"][i]["mean_ms"], rows["game_v20"][i]["mean_ms"]),
        "p999_relative": rel(rows["game_v24"][i]["p999_ms"], rows["game_v20"][i]["p999_ms"])} for i, r in enumerate(repeats)]
    return {"repeats": rows, "medians": med, "checks": checks, "verdict": "PASS" if all(c["passed"] for c in checks.values()) else "FAIL",
        "per_repeat": per_repeat, "window_1700_1900": windows, "window_medians": win, "slopes": slopes, "quick": quick,
        "v24_vs_v23_mean_relative": rel(med["game_v24"]["mean_ms"], med["game_v23"]["mean_ms"]),
        "v24_vs_v23_p999_relative": rel(med["game_v24"]["p999_ms"], med["game_v23"]["p999_ms"]),
        "v23_vs_v20_mean_relative": rel(med["game_v23"]["mean_ms"], med["game_v20"]["mean_ms"]),
        "v23_vs_v20_p999_relative": rel(med["game_v23"]["p999_ms"], med["game_v20"]["p999_ms"])}


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", choices=("c6", "h5"), required=True)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    provenance = json.loads((args.input / "provenance.json").read_text())
    result = c6(args.input) if args.stage == "c6" else h5(args.input)
    result["provenance"] = provenance
    args.out_json.write_text(json.dumps(result, indent=1, sort_keys=True) + "\n")
    if args.stage == "c6":
        m = result["medians"]
        lines = ["# Pass 19 C6: chronicle write cost (game_v23 stress, 500 agents, 3 interleaved repeats, medians)", "",
            "| build | mean ms | p99.9 ms |", "|---|---|---|",
            f"| base 20a1d078 | {m['base']['mean_ms']:.4f} | {m['base']['p999_ms']:.3f} |",
            f"| Part 1 0ca95174 | {m['part1']['mean_ms']:.4f} | {m['part1']['p999_ms']:.3f} |", "",
            f"Mean {result['mean_relative']:+.2%} (expected within ±3 %), p99.9 {result['p999_relative']:+.2%}; per-repeat mean "
            + ", ".join(f"{x:+.2%}" for x in result["per_repeat_mean_relative"])
            + f"; identical fingerprints across builds: {result['same_fingerprint']}. Quiet hold: {provenance['quiet']['started']} (contended {provenance['contended']}).", ""]
    else:
        m, w, c = result["medians"], result["window_medians"], result["checks"]
        lines = ["# Pass 19 H5: game_v24 vs game_v20 (stress 500, five interleaved repeats, medians)", "",
            "| rules | mean ms | p99.9 ms | 17:00–19:00 mean ms | 17:00–19:00 max ms | 18:00 tick ms |", "|---|---|---|---|---|---|"]
        for rules in ("game_v20", "game_v23", "game_v24"):
            lines.append(f"| {rules} | {m[rules]['mean_ms']:.4f} | {m[rules]['p999_ms']:.3f} | {w[rules]['mean_ms']:.4f} | {w[rules]['max_ms']:.3f} | {w[rules]['tick_1800_ms']:.3f} |")
        lines += ["", "| check | value | limit | pass |", "|---|---|---|---|"]
        for name, check in c.items():
            lines.append(f"| {name} | {check['value']:.4f} | {check['op']} {check['limit']} | {check['passed']} |")
        lines += ["", f"Verdict: **{result['verdict']}**. Per repeat (v24 vs v20): " + "; ".join(
            f"{x['repeat']} mean {x['mean_relative']:+.1%} p99.9 {x['p999_relative']:+.1%}" for x in result["per_repeat"]),
            f"v24 vs v23: mean {result['v24_vs_v23_mean_relative']:+.2%}, p99.9 {result['v24_vs_v23_p999_relative']:+.2%}; "
            f"v23 vs v20: mean {result['v23_vs_v20_mean_relative']:+.2%}, p99.9 {result['v23_vs_v20_p999_relative']:+.2%}.",
            f"Decision-system slope (100/500/1,000, 2 days): v24 {result['slopes']['game_v24']['b']:.3f}, v20 {result['slopes']['game_v20']['b']:.3f}. "
            f"quick: base {result['quick']['base']['mean_ms']:.4f} ms, final {result['quick']['final']['mean_ms']:.4f} ms (fingerprints equal: {result['quick']['base']['fingerprint'] == result['quick']['final']['fingerprint']}).",
            f"Quiet hold started {provenance['quiet']['started']} (contended {provenance['contended']}).", ""]
    args.out_md.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
