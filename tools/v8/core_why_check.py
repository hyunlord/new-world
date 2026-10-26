#!/usr/bin/env python3
"""Strict held-out CORE why coverage; synthetic tests never read real held-out data.

Hashes bind bytes, not chronology. The parent owns the freeze/source receipt.
This is a separate why gate, never a full milestone acceptance claim.
"""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import sys
from typing import Final

import core_score as core
from core_score import Data, ScoreError, digest, mapping, number, require, sequence
from mux_safety_validate import JSON

SEEDS: Final = list(range(1232, 1264))
TICKS: Final = 48 * 1440
COUNTERS: Final = (
    "observed_mind_ticks", "first_choices", "behaviour_switches", "node_only_choices",
    "same_behaviour_restarts", "expected_records", "covered_records", "behaviour_expected",
    "behaviour_covered", "node_expected", "node_covered", "critical_expected", "critical_covered",
    "critical_choice_expected", "critical_choice_covered", "failures",
)
PROBLEMS: Final = {"missing_dictionary", "missing_record", "invalid_record", "wrong_choice", "wrong_tick", "empty_facts"}


def ratio(covered: int, expected: int) -> float | None:
    """A zero denominator is explicitly unexposed, never 100 percent."""
    return covered / expected if expected else None


def check_ratio(value: JSON, covered: int, expected: int, name: str) -> None:
    """Reject nonfinite, boolean or fabricated reported ratios."""
    if expected == 0:
        require(value is None, f"{name}: zero denominator must be null")
        return
    match value:
        case int() | float() if type(value) is not bool:
            require(math.isfinite(value) and abs(value - covered / expected) <= 1e-12, f"{name}: inconsistent ratio")
        case _:
            raise ScoreError(f"{name}: invalid ratio")


def world(row: Data, source: str, policy: str) -> Data:
    """Check one complete observed world before deriving its coverage ratios."""
    seed = number(row["seed"], "seed")
    require(number(row["days"], "world days") == 48, "Wrong world duration")
    require(number(mapping(row["mux"])["ticks_per_day"], "ticks per day") == 1440, "Unexpected day length")
    observed = mapping(row["mux2"])
    require(number(observed["schema_version"], "observer schema") == 1, "Wrong observer schema")
    require(number(observed["start_tick"], "start") == 0
            and number(observed["end_tick"], "end") == TICKS
            and number(observed["observed_ticks"], "observed ticks") == TICKS, "Incomplete observed ticks")
    require(sequence(observed["observer_integrity_errors"]) == [], "Observer integrity errors")
    inputs = mapping(observed["inputs"])
    require(all(inputs[key] is True for key in ("steward", "defence", "defence_enabled_end"))
            and inputs["player_script"] is False and inputs["story"] == inputs["neighbours"] == "on", "Non-equivalent held-out inputs")
    require(inputs["source_sha"] == source and inputs["policy_sha256"] == policy, "World source/policy mismatch")
    coverage = mapping(row["mind_coverage"])
    c = {name: number(coverage[name], name) for name in COUNTERS}
    require(all(value <= 2**64 - 1 for value in c.values()), "Counter exceeds runtime u64")
    require(c["observed_mind_ticks"] == TICKS, "Incomplete mind observed ticks")
    require(c["first_choices"] > 0, "World has no first choices")
    require(c["behaviour_expected"] == c["first_choices"] + c["behaviour_switches"], "Behavior denominator mismatch")
    require(c["node_expected"] == c["node_only_choices"] + c["same_behaviour_restarts"], "Node denominator mismatch")
    require(c["expected_records"] == c["behaviour_expected"] + c["node_expected"], "Total denominator mismatch")
    require(c["covered_records"] == c["behaviour_covered"] + c["node_covered"], "Total covered mismatch")
    for prefix in ("behaviour", "node", "critical", "critical_choice"):
        require(c[f"{prefix}_covered"] <= c[f"{prefix}_expected"], f"{prefix}: covered exceeds expected")
    require(c["failures"] == c["expected_records"] - c["covered_records"], "Failure counter mismatch")
    require(c["critical_expected"] <= c["behaviour_expected"] and c["critical_covered"] <= c["behaviour_covered"], "Critical behavior subset mismatch")
    require(c["critical_expected"] <= c["critical_choice_expected"] <= c["expected_records"], "Critical choice denominator subset mismatch")
    require(c["critical_covered"] <= c["critical_choice_covered"] <= c["covered_records"], "Critical choice coverage subset mismatch")
    critical_missing = c["critical_expected"] - c["critical_covered"]
    choice_missing = c["critical_choice_expected"] - c["critical_choice_covered"]
    require(critical_missing <= choice_missing <= c["failures"], "Critical missing subset mismatch")
    require(critical_missing <= c["behaviour_expected"] - c["behaviour_covered"], "Critical behavior failures mismatch")
    require(c["critical_choice_expected"] - c["critical_expected"] <= c["node_expected"], "Critical node denominator mismatch")
    require(c["critical_choice_covered"] - c["critical_covered"] <= c["node_covered"], "Critical node covered mismatch")
    require(choice_missing - critical_missing <= c["node_expected"] - c["node_covered"], "Critical node failures mismatch")
    require(coverage["status"] == ("FAIL" if c["failures"] else "PASS"), "Observer status inconsistent")
    for name, prefix in (("coverage", "behaviour"), ("node_coverage", "node"), ("critical_coverage", "critical"), ("critical_choice_coverage", "critical_choice")):
        check_ratio(coverage[name], c[f"{prefix}_covered"], c[f"{prefix}_expected"], name)
    examples = sequence(coverage["examples"])
    require(len(examples) == min(c["failures"], 16), "Missing/excess diagnostic examples")
    for value in examples:
        example = mapping(value)
        require(number(example["tick"], "example tick") < TICKS, "Future example")
        number(example["agent"], "example agent")
        number(example["node"], "example node")
        require(type(example["problem"]) is str and example["problem"] in PROBLEMS, "Unknown example problem")
    require(type(coverage["definition"]) is str and bool(coverage["definition"]), "Missing denominator definition")
    return {"seed": seed, **c, "behaviour_coverage": ratio(c["behaviour_covered"], c["behaviour_expected"]),
            "node_coverage": ratio(c["node_covered"], c["node_expected"]),
            "critical_choice_coverage": ratio(c["critical_choice_covered"], c["critical_choice_expected"]),
            "critical_status": "N/A" if c["critical_choice_expected"] == 0 else ("PASS" if choice_missing == 0 else "FAIL")}


def check(report: Data, receipt: Data) -> Data:
    """Require all 32 frozen worlds and calculate integer-exact aggregate bars."""
    try:
        require(receipt["phase"] == "heldout" and receipt["brain"] == "mind" and receipt["round"] is None, "Wrong phase/brain/round")
        require(receipt["defence"] is True and receipt["policy_frozen_before_run"] is True, "Missing held-out attestation")
        require(number(receipt["days"], "receipt days") == 48
                and [number(seed, "receipt seed") for seed in sequence(receipt["seeds"])] == SEEDS, "Receipt coverage mismatch")
        source, policy = digest(receipt["source_sha"], 40), digest(receipt["policy_sha256"], 64)
        binary = digest(receipt["binary_sha256"], 64)
        require(report["scene"] == "colony" and number(report["days"], "days") == 48, "Wrong scene/days")
        rows = [mapping(value) for value in sequence(report["seeds"])]
        require(sorted(number(row["seed"], "seed") for row in rows) == SEEDS, "Missing/duplicate/wrong seed set")
        worlds = [world(row, source, policy) for row in sorted(rows, key=lambda row: number(row["seed"], "seed"))]
        total = {name: sum(number(row[name], name) for row in worlds) for name in COUNTERS}
        behaviour_pass = total["behaviour_covered"] * 100 >= total["behaviour_expected"] * 99
        critical_exposed = total["critical_choice_expected"] > 0
        critical_pass = critical_exposed and total["critical_choice_covered"] == total["critical_choice_expected"]
        return {"schema_version": 1, "status": "PASS" if behaviour_pass and critical_pass else "FAIL",
                "acceptance_claim": False, "phase": "heldout", "source_sha": source,
                "policy_sha256": policy, "binary_sha256": binary, "worlds": worlds,
                "aggregate": {**total, "behaviour_coverage": ratio(total["behaviour_covered"], total["behaviour_expected"]),
                              "critical_choice_coverage": ratio(total["critical_choice_covered"], total["critical_choice_expected"])},
                "bars": {"behaviour_at_least_99_percent": behaviour_pass,
                         "all_critical_choices_covered": critical_pass,
                         "critical_exposure": "OBSERVED" if critical_exposed else "N/A"}}
    except KeyError as error:
        raise ScoreError(f"Missing required field: {error}") from error


def read_bound(summary: Path, receipt_path: Path) -> tuple[Data, Data, Data]:
    """Bind summary, parent receipt, current checker and reused parser sources."""
    report, receipt, hashes = core.read_bound_run(summary, receipt_path)
    own_sha = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    require(receipt.get("why_checker_sha256") == own_sha, "Why checker hash mismatch")
    return report, receipt, {**hashes, "why_checker_sha256": own_sha}


def main() -> int:
    """Standalone checker; outputs a new external evidence artifact."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        require(not args.output.exists() and not args.output.is_symlink(), "Output must be new")
        require(not args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]), "Output must be external")
        report, receipt, hashes = read_bound(args.summary, args.receipt)
        result = check(report, receipt)
        result["input_hashes"] = hashes
        with args.output.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(result, indent=2, allow_nan=False) + "\n")
        print("CORE_WHY " + str(result["status"]) + "; full CORE acceptance not claimed")
        return 0 if result["status"] == "PASS" else 1
    except (ScoreError, OSError, json.JSONDecodeError, TypeError, ValueError) as error:
        print(f"CORE_WHY INVALID: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
