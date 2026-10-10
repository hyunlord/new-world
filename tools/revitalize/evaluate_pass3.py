#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Evaluate Pass 3 U targets. Stdlib only, as required by the goal.
Run: python3 tools/revitalize/evaluate_pass3.py --help
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Final, TypedDict

from pass3_evidence import CLOCKS, EvidenceError, Json, RoundEvidence, boolean, document, integer, load_round, mapping, number, quantiles

BASELINE_MS: Final = .0007874892241498525

@dataclass(frozen=True, slots=True)
class Receipts:
    legacy_verified: bool
    v8_verified: bool
    probe_10_exit_code: int
    probe_60_exit_code: int
    children_implemented: bool
    evidence: dict[str, str]
    expected_revision: str
    evidence_sha256: dict[str, str] = field(default_factory=dict)

    @classmethod
    def read(cls, path: Path) -> Receipts:
        row = mapping(document(path))
        evidence = mapping(row["evidence"])
        required = {"legacy", "v8", "probe_10", "probe_60"}
        if set(evidence) != required or any(not isinstance(v, str) or not v.strip() for v in evidence.values()):
            raise EvidenceError("receipt evidence must name four nonempty command/test log paths")
        expected_revision = str(row["expected_revision"])
        if not re.fullmatch("[0-9a-f]{40}", expected_revision):
            raise EvidenceError("receipt expected_revision must be a full producer SHA")
        paths: dict[str, str] = {}
        hashes: dict[str, str] = {}
        for key, name in evidence.items():
            source = Path(str(name))
            source = source if source.is_absolute() else path.parent / source
            data = source.read_bytes()
            if not data.strip():
                raise EvidenceError(f"empty receipt evidence: {source}")
            paths[key] = str(source.resolve())
            hashes[key] = hashlib.sha256(data).hexdigest()
        return cls(boolean(row["legacy_verified"]), boolean(row["v8_verified"]), integer(row["probe_10_exit_code"]), integer(row["probe_60_exit_code"]), boolean(row["children_implemented"]), paths, expected_revision, hashes)

class Clause(TypedDict):
    passed: bool
    observed: dict[str, float] | bool | int | float
    requirement: str

class Target(TypedDict):
    passed: bool
    clauses: dict[str, Clause]

def clause(passed: bool, observed: dict[str, float] | bool | int | float, requirement: str) -> Clause:
    return {"passed": passed, "observed": observed, "requirement": requirement}

def target(clauses: dict[str, Clause]) -> Target:
    return {"passed": all(c["passed"] for c in clauses.values()), "clauses": clauses}

def rhythm(round_data: RoundEvidence) -> tuple[dict[str, dict[str, float]], bool]:
    values: dict[str, list[float]] = {spec[0]: [] for spec in CLOCKS.values()}
    complete = True
    for run in round_data.runs:
        for minute, (key, numerator, denominator) in CLOCKS.items():
            observed = [run.clocks[day][minute] for day in range(2, round_data.duration + 1) if minute in run.clocks.get(day, {})]
            complete = complete and len(observed) == round_data.duration - 1
            ratios = [row[numerator] / row[denominator] if row[denominator] else 0. for row in observed]
            if ratios:
                values[key].append(quantiles(ratios)["median"])
    return {key: quantiles(rows) for key, rows in values.items() if rows}, complete

def evaluate(short: RoundEvidence, long: RoundEvidence, receipts: Receipts, baseline: float = BASELINE_MS) -> dict[str, Target]:
    """Paired raw-seed metrics, never arithmetic on marginal medians."""
    result: dict[str, Target] = {}
    result["U1"] = target({
        "revision": clause(short.revision == long.revision == receipts.expected_revision, short.revision == long.revision == receipts.expected_revision, "both raw rounds match receipt expected_revision"),
        "twins_10": clause(short.deterministic and receipts.probe_10_exit_code == 0, short.deterministic, "64 successful deterministic runs, exit 0"),
        "twins_60": clause(long.deterministic and receipts.probe_60_exit_code == 0, long.deterministic, "32 successful deterministic runs, exit 0"),
        "legacy": clause(receipts.legacy_verified, receipts.legacy_verified, "external legacy pin passed"),
        "v8": clause(receipts.v8_verified, receipts.v8_verified, "external v8 reproduction identical"),
    })
    for target_id, key, maximums in (("U2", "deaths_dehydration", (1, 2)), ("U3", "deaths_starvation", (0, 3))):
        clauses = {}
        for run, maximum in zip((short, long), maximums, strict=True):
            stats = quantiles(run.values(run.duration, key))
            clauses[f"day_{run.duration}"] = clause(stats["median"] == 0 and stats["max"] <= maximum, stats, f"median 0; max <= {maximum}")
        result[target_id] = target(clauses)
    settlements = [value["settlements"] for data in (short, long) for run in data.runs for value in run.days.values()]
    stats = quantiles(settlements)
    result["U4"] = target({"all_days": clause(stats["min"] == stats["max"] == 1, stats, "one settlement at every day sample")})
    overflow = quantiles([float(run.overflow) for run in long.runs])
    shelters = quantiles(short.values(10, "shelters_completed"))
    result["U5"] = target({"overflow": clause(overflow["max"] <= 2, overflow, "owned same-tick overflow max <= 2 from first formation"), "shelters": clause(shelters["median"] >= 2, shelters, "day 10 completions median >= 2")})
    pop10, pop60 = quantiles(short.values(10, "alive")), quantiles(long.values(60, "alive"))
    minimum = quantiles([run.minimum_alive for data in (short, long) for run in data.runs])
    stability = quantiles([abs(run.days[60]["alive"] - run.days[45]["alive"]) for run in long.runs])
    result["U6"] = target({"day10": clause(30 <= pop10["median"] <= 44, pop10, "median in [30,44]"), "day60": clause(38 <= pop60["median"] <= 48, pop60, "median in [38,48]"), "floor": clause(minimum["min"] >= 24, minimum, "every raw sample alive >= 24"), "stability": clause(stability["median"] <= 4, stability, "median per-seed |alive60-alive45| <= 4")})
    stock, need = quantiles(long.values(60, "stockpile_food")), quantiles(long.values(60, "food_need_per_day"))
    reserve = quantiles([run.days[60]["stockpile_food"] - .5 * run.days[60]["food_need_per_day"] for run in long.runs])
    result["U7"] = target({"reserve": clause(stock["median"] >= .5 * need["median"], {**{f"stock_{k}": v for k, v in stock.items()}, **{f"need_{k}": v for k, v in need.items()}, **{f"paired_margin_{k}": v for k, v in reserve.items()}}, "median stock >= 0.5 * median actual need")})
    clocks, complete = rhythm(long)
    clauses = {"coverage": clause(complete, complete, "59 snapshots per seed at each of 02:00,12:00,19:30")}
    for key, threshold in (("night_in_shelter_share", .70), ("evening_near_hearth_share", .50)):
        stats = clocks.get(key, {})
        clauses[key] = clause(complete and bool(stats) and stats["median"] >= threshold, stats, f"median over days per seed, then across seeds >= {threshold}")
    result["U8"] = target(clauses)
    shares = quantiles([run.days[10]["children"] / run.days[10]["alive"] if run.days[10]["alive"] else 0. for run in short.runs])
    observed = sum(run.maturation["children_observed"] for run in long.runs)
    matured = sum(run.maturation["maturations_observed"] for run in long.runs)
    late = sum(run.maturation["late_children"] for run in long.runs)
    result["U9"] = target({"implemented": clause(receipts.children_implemented, receipts.children_implemented, "children implemented; dropped scope is FAIL"), "child_share": clause(0 < shares["median"] <= .45, shares, "median per-seed children/alive at day10 in (0,.45]"), "maturation": clause(receipts.probe_60_exit_code == 0 and observed > 0 and matured > 0 and late == 0, {"children_observed": float(observed), "maturations_observed": float(matured), "late_children": float(late)}, "successful every-tick bound checks and nonzero observed maturation")})
    home = quantiles(short.values(10, "home_distance_mean"))
    clauses = {"home": clause(home["median"] <= 6, home, "day10 median home distance <= 6")}
    for data in (short, long):
        action = data.actionable_share[data.duration]
        social = quantiles([run.social_rate for run in data.runs])
        clauses[f"actionable_{data.duration}"] = clause(action["median"] <= .01, action, "median cumulative periodic actionable breach share <= .01")
        clauses[f"social_{data.duration}"] = clause(social["median"] >= .40 and social["min"] > 0, social, "median social pairs/agent-day >= .40, each seed > 0")
    result["U10"] = target(clauses)
    result["U11"] = target({f"day{data.duration}": clause((stats := quantiles([run.timing for run in data.runs]))["median"] <= 1.5 * baseline, stats, f"median ms/agent-tick <= {1.5 * baseline:.12g} (same-host baseline {baseline:.12g})") for data in (short, long)})
    return result

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--round10", required=True, type=Path)
    parser.add_argument("--round60", required=True, type=Path)
    parser.add_argument("--sample-every", type=int, default=120, help="expected raw periodic interval for both runs")
    parser.add_argument("--receipts", required=True, type=Path, help="trusted leader receipts; evidence paths relative to this file, must exist and be nonempty")
    parser.add_argument("--out", required=True, type=Path)
    baseline_args = parser.add_mutually_exclusive_group()
    baseline_args.add_argument("--baseline-ms", type=float, default=BASELINE_MS)
    baseline_args.add_argument("--baseline-summary", type=Path)
    parser.add_argument("--v8-context", type=Path, help="optional v8 summary; context only, never U gates")
    args = parser.parse_args()
    try:
        baseline = args.baseline_ms
        if args.baseline_summary:
            source = mapping(document(args.baseline_summary))
            if source["scene"] != "game":
                raise EvidenceError("timing baseline must be frozen game/v8 scene")
            baseline = number(mapping(mapping(source["timing"])["ms_per_agent_tick"])["median"])
        if not math.isfinite(baseline) or baseline <= 0:
            raise EvidenceError("baseline must be positive and finite")
        short, long = load_round(args.round10, 10, args.sample_every), load_round(args.round60, 60, args.sample_every)
        receipts = Receipts.read(args.receipts)
        if short.revision != long.revision or short.revision != receipts.expected_revision:
            raise EvidenceError("round producer revisions do not match receipt expected_revision")
        targets = evaluate(short, long, receipts, baseline)
        context: Json = None
        if args.v8_context:
            source = mapping(document(args.v8_context))
            if source["scene"] != "game":
                raise EvidenceError("v8 context must have scene game")
            context = source
        report = {"schema_version": 1, "status": "pass" if all(t["passed"] for t in targets.values()) else "fail", "targets": targets, "passed": all(t["passed"] for t in targets.values()), "inputs": {"round10": str(args.round10.resolve()), "round60": str(args.round60.resolve()), "receipts": str(args.receipts.resolve()), "producer_revision": short.revision, "sample_every": args.sample_every, "baseline_ms": baseline, "baseline_summary": str(args.baseline_summary) if args.baseline_summary else None}, "receipts": asdict(receipts), "rhythm_context": rhythm(long)[0], "v8_context": context, "notes": ["U7 compares marginal stock/need medians; paired margins also reported.", "U10 and U11 apply thresholds to both run durations.", "Timing excludes measured in-tick census work and all post-tick observation, I/O and twin validation.", "Raw clock day differs from elapsed checkpoint day. Missing clock evidence fails U8.", "Maturation covers observed living children; children still alive at run end are right-censored."]}
        args.out.mkdir(parents=True, exist_ok=True)
        (args.out / "u_targets.json").write_text(json.dumps(report, indent=2, allow_nan=False) + "\n", encoding="utf-8")
        lines = ["# Pass 3 U targets", "", "| Target | Clause | Verdict | Requirement | Observed |", "|---|---|---|---|---|"]
        for name, entry in targets.items():
            for key, item in entry["clauses"].items():
                lines.append(f"| {name} | {key} | {'PASS' if item['passed'] else 'FAIL'} | {item['requirement']} | {json.dumps(item['observed'], sort_keys=True)} |")
        lines.extend(["", "V8 evidence is comparison context only; U targets apply to v9.", "", *report["notes"]])
        (args.out / "u_targets.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
        print("PASS" if report["passed"] else "FAIL")
        return 0 if report["passed"] else 1
    except (EvidenceError, OSError, KeyError, json.JSONDecodeError, ValueError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        # Replace any prior success report when reusing an output directory.
        try:
            args.out.mkdir(parents=True, exist_ok=True)
            invalid = {"schema_version": 1, "status": "invalid", "passed": False, "error": str(exc)}
            (args.out / "u_targets.json").write_text(json.dumps(invalid, indent=2) + "\n", encoding="utf-8")
            (args.out / "u_targets.md").write_text(f"# Invalid evidence\n\n{exc}\n", encoding="utf-8")
        except OSError as output_error:
            print(f"Cannot write invalid-evidence report: {output_error}", file=sys.stderr)
        return 2

if __name__ == "__main__":
    raise SystemExit(main())
