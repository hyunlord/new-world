#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_perf_p20.py --input RAW_DIR --out-md OUT.md --out-json OUT.json
"""Pass 21 U6: game_v26 against game_v24, game_v25 reported beside
(PERF_BUDGET.md Pass 20/21, METRICS.md `## Pass 21`); evaluate_perf_p20.py
with the candidate moved to game_v26.

Input is a `perf_p21.sh` directory measured inside `quiet_lock_p21.sh`:
  - stress (500 founders, 1 day, fed), five interleaved repeats, medians:
    mean <= +15 %, p99.9 <= +35 %, mean <= 4.0 ms, decision-system slope
    (100/500/1,000, 2 days, log-log) < 1.7;
  - natural (`--days 120`) on four seeds, three interleaved repeats: per
    repeat the mean tick over the four worlds (sum of steady tick time /
    steady ticks) and the mean of their p99.9; ratio v26/v24 of the medians
    over repeats: mean <= 1.15, p99.9 <= 1.35; ms per agent-tick reported
    (sum of per-day tick time / sum of per-day ticks x alive at 06:00).
p99.9 is perf.rs's (second-largest of the measured ticks).
"""
from __future__ import annotations

import argparse
import json
import statistics
import sys
from decimal import Decimal
from pathlib import Path
from typing import Any

from perf_p15 import loglog


def stress_file(directory: Path, version: str, n: int = 500, mode: str = "stress") -> Path:
    return directory / f"{mode}_stress_v{version}_s15100_n{n}_side64_fed.json"


def summary(path: Path) -> dict[str, Any]:
    run = json.loads(path.read_text())
    decision = [s for s in run["systems"] if s["name"] == "AgentDecisionSystem"][0]
    relations = [s for s in run["systems"] if s["name"] == "RelationsSystem"]
    return {"file": str(path), "mean_ms": run["tick_summary"]["mean_ms"], "p999_ms": run["tick_summary"]["p999_ms"],
        "max_ms": run["tick_summary"]["max_ms"], "alive": run["mean_alive_0600"], "decision_us": decision["mean_us_per_call"],
        "relations_us": relations[0]["mean_us_per_call"] if relations else None,
        "fingerprint": run["fingerprint"], "contended": run["contended"], "git_sha": run["git_sha"]}


def rel(new: float, old: float) -> float:
    return float(Decimal(str(new)) / Decimal(str(old)) - 1)


def natural_world(path: Path) -> dict[str, Any]:
    run = json.loads(path.read_text())
    t = run["tick_summary"]
    agent_ticks = sum(day["ticks"] * (day["alive_0600"] or 0) for day in run["per_day"])
    tick_ms = sum(day["tick_ms"] for day in run["per_day"])
    relations = [s for s in run["systems"] if s["name"] == "RelationsSystem"]
    return {"file": path.name, "seed": run["seed"], "ticks": t["ticks"], "total_ms": t["mean_ms"] * t["ticks"],
        "mean_ms": t["mean_ms"], "p999_ms": t["p999_ms"], "max_ms": t["max_ms"], "alive_end": run["alive_end"],
        "settlements_end": run["settlements_end"], "ms_per_agent_tick": tick_ms / agent_ticks if agent_ticks else None,
        "relations_share": relations[0]["share_of_tick_time"] if relations else None,
        "fingerprint": run["fingerprint"], "contended": run["contended"]}


def natural(root: Path) -> dict[str, Any]:
    repeats = sorted(p.name for p in root.glob("nat_r[0-9]*") if p.is_dir())
    out: dict[str, Any] = {"repeats": {}, "per_repeat": []}
    for scene in ("game_v24", "game_v25", "game_v26"):
        rows = []
        for r in repeats:
            worlds = [natural_world(p) for p in sorted((root / r / scene).glob("natural_*.json"))]
            rows.append({"repeat": r, "worlds": worlds,
                "mean_ms": sum(w["total_ms"] for w in worlds) / sum(w["ticks"] for w in worlds),
                "p999_ms": statistics.fmean(w["p999_ms"] for w in worlds),
                "ms_per_agent_tick": statistics.fmean(w["ms_per_agent_tick"] for w in worlds if w["ms_per_agent_tick"])})
        out["repeats"][scene] = rows
    med = {scene: {k: statistics.median(r[k] for r in rows) for k in ("mean_ms", "p999_ms", "ms_per_agent_tick")}
        for scene, rows in out["repeats"].items()}
    out["medians"] = med
    for i, r in enumerate(repeats):
        a, b, c = out["repeats"]["game_v26"][i], out["repeats"]["game_v24"][i], out["repeats"]["game_v25"][i]
        out["per_repeat"].append({"repeat": r, "mean_ratio": a["mean_ms"] / b["mean_ms"], "p999_ratio": a["p999_ms"] / b["p999_ms"],
            "v25_mean_ratio": c["mean_ms"] / b["mean_ms"], "v25_p999_ratio": c["p999_ms"] / b["p999_ms"]})
    out["checks"] = {
        "natural_mean_ratio": {"value": med["game_v26"]["mean_ms"] / med["game_v24"]["mean_ms"], "limit": 1.15, "op": "<="},
        "natural_p999_ratio": {"value": med["game_v26"]["p999_ms"] / med["game_v24"]["p999_ms"], "limit": 1.35, "op": "<="},
    }
    out["beside_v25"] = {"natural_mean_ratio": med["game_v25"]["mean_ms"] / med["game_v24"]["mean_ms"],
        "natural_p999_ratio": med["game_v25"]["p999_ms"] / med["game_v24"]["p999_ms"]}
    fps = {scene: {tuple(w["fingerprint"] for w in r["worlds"]) for r in rows} for scene, rows in out["repeats"].items()}
    out["fingerprints_stable_across_repeats"] = {scene: len(v) == 1 for scene, v in fps.items()}
    return out


def evaluate(root: Path) -> dict[str, Any]:
    repeats = sorted(p.name for p in root.glob("r[0-9]*") if p.is_dir())
    rows = {rules: [summary(stress_file(root / r / rules, rules.removeprefix("game_v"))) for r in repeats]
        for rules in ("game_v24", "game_v25", "game_v26")}
    med = {rules: {k: statistics.median(x[k] for x in v) for k in ("mean_ms", "p999_ms")} for rules, v in rows.items()}
    slopes = {}
    for rules in ("game_v26", "game_v24"):
        points = [summary(stress_file(root / f"slope_{rules}", rules.removeprefix("game_v"), n)) for n in (100, 500, 1000)]
        fit = loglog([p["alive"] for p in points], [p["decision_us"] for p in points])
        slopes[rules] = {"b": fit["b"], "points": points}
    checks = {
        "mean_relative": {"value": rel(med["game_v26"]["mean_ms"], med["game_v24"]["mean_ms"]), "limit": 0.15, "op": "<="},
        "p999_relative": {"value": rel(med["game_v26"]["p999_ms"], med["game_v24"]["p999_ms"]), "limit": 0.35, "op": "<="},
        "mean_absolute_ms": {"value": med["game_v26"]["mean_ms"], "limit": 4.0, "op": "<="},
        "decision_slope": {"value": slopes["game_v26"]["b"], "limit": 1.7, "op": "<"},
    }
    nat = natural(root)
    checks.update(nat["checks"])
    for check in checks.values():
        check["passed"] = check["value"] < check["limit"] if check["op"] == "<" else check["value"] <= check["limit"]
    quick = {role: summary(root / f"quick_{role}" / "quick_stress_v19_s15100_n500_side64_fed.json") for role in ("base", "final")}
    per_repeat = [{"repeat": r, "mean_relative": rel(rows["game_v26"][i]["mean_ms"], rows["game_v24"][i]["mean_ms"]),
        "p999_relative": rel(rows["game_v26"][i]["p999_ms"], rows["game_v24"][i]["p999_ms"]),
        "v25_mean_relative": rel(rows["game_v25"][i]["mean_ms"], rows["game_v24"][i]["mean_ms"])} for i, r in enumerate(repeats)]
    beside = {"mean_relative": rel(med["game_v25"]["mean_ms"], med["game_v24"]["mean_ms"]),
        "p999_relative": rel(med["game_v25"]["p999_ms"], med["game_v24"]["p999_ms"])}
    return {"repeats": rows, "medians": med, "checks": checks, "per_repeat": per_repeat, "slopes": slopes, "beside_v25": beside,
        "quick": quick, "natural": nat, "verdict": "PASS" if all(c["passed"] for c in checks.values()) else "FAIL"}


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    provenance = json.loads((args.input / "provenance.json").read_text())
    result = evaluate(args.input)
    result["provenance"] = provenance
    args.out_json.write_text(json.dumps(result, indent=1, sort_keys=True) + "\n")
    m, c, n = result["medians"], result["checks"], result["natural"]
    lines = ["# Pass 21 U6: game_v26 vs game_v24 (game_v25 beside)", "",
        "## Stress (500 founders, one camp, 1 day, five interleaved repeats, medians)", "",
        "| rules | mean ms | p99.9 ms |", "|---|---|---|"]
    for rules in ("game_v24", "game_v25", "game_v26"):
        lines.append(f"| {rules} | {m[rules]['mean_ms']:.4f} | {m[rules]['p999_ms']:.3f} |")
    lines += ["", "Per repeat (v26 vs v24): " + "; ".join(f"{x['repeat']} mean {x['mean_relative']:+.1%} p99.9 {x['p999_relative']:+.1%}" for x in result["per_repeat"]),
        f"game_v25 beside (medians vs v24): mean {result['beside_v25']['mean_relative']:+.2%}, p99.9 {result['beside_v25']['p999_relative']:+.2%}.",
        f"Decision-system slope (100/500/1,000, 2 days): v26 {result['slopes']['game_v26']['b']:.3f}, v24 {result['slopes']['game_v24']['b']:.3f}. "
        f"quick: base {result['quick']['base']['mean_ms']:.4f} ms, final {result['quick']['final']['mean_ms']:.4f} ms (fingerprints equal: {result['quick']['base']['fingerprint'] == result['quick']['final']['fingerprint']}).",
        "", "## Natural (120 days, four multi-village seeds, three interleaved repeats)", "",
        "| rules | mean tick ms (median of repeats) | mean p99.9 ms | ms per agent-tick |", "|---|---|---|---|"]
    for scene in ("game_v24", "game_v25", "game_v26"):
        lines.append(f"| {scene} | {n['medians'][scene]['mean_ms']:.4f} | {n['medians'][scene]['p999_ms']:.3f} | {n['medians'][scene]['ms_per_agent_tick']:.6f} |")
    lines += ["", "Per repeat (v26 / v24): " + "; ".join(f"{x['repeat']} mean ×{x['mean_ratio']:.3f} p99.9 ×{x['p999_ratio']:.3f} (v25 ×{x['v25_mean_ratio']:.3f} / ×{x['v25_p999_ratio']:.3f})" for x in n["per_repeat"]),
        f"game_v25 beside (natural medians / v24): mean ×{n['beside_v25']['natural_mean_ratio']:.3f}, p99.9 ×{n['beside_v25']['natural_p999_ratio']:.3f}.",
        "", "| seed | v24 alive end / villages | v25 alive end / villages | v26 alive end / villages | v26 RelationsSystem share |", "|---|---|---|---|---|"]
    first = {s: {w["seed"]: w for w in n["repeats"][s][0]["worlds"]} for s in ("game_v24", "game_v25", "game_v26")}
    for seed in sorted(first["game_v26"]):
        a, c, b = first["game_v24"].get(seed), first["game_v25"].get(seed), first["game_v26"][seed]
        lines.append(f"| {seed} | {a['alive_end'] if a else '-'} / {a['settlements_end'] if a else '-'} | {c['alive_end'] if c else '-'} / {c['settlements_end'] if c else '-'} | {b['alive_end']} / {b['settlements_end']} | {b['relations_share']:.4%} |")
    lines += ["", "| check | value | limit | pass |", "|---|---|---|---|"]
    for name, check in c.items():
        lines.append(f"| {name} | {check['value']:.4f} | {check['op']} {check['limit']} | {check['passed']} |")
    lines += ["", f"Verdict: **{result['verdict']}**. Fingerprints stable across repeats: {n['fingerprints_stable_across_repeats']}. "
        f"Quiet hold started {provenance['quiet']['started']} (contended {provenance['contended']}).", ""]
    args.out_md.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
