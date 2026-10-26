#!/usr/bin/env python3
"""CORE outcome comparison: exact BASE/mind worlds, exposed-only cold-snap arithmetic.

Adapted from the published MUX2 observer checks without changing that scorer.
This validates evaluation rows only, not shadow, why coverage, perf or the full gate.
Freeze chronology remains an external attestation; hashes prove byte identity only.
"""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 tools/v8/core_score.py --help
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from typing import Final, TypeAlias

import mux_safety_validate as validator
from mux_safety_validate import JSON

Data: TypeAlias = dict[str, JSON]
BASE_SHA: Final = "9855bd082ad88a4c49296410f206f148d98c4df9"
TARGETS: Final = {"stockpile_day1": 31, "walled_beds_day10": 31, "zero_cold_deaths": 31,
                 "alive_day48": 31, "wolf_deaths_at_most_one": 29, "exact_ledger": 32}


class ScoreError(ValueError):
    """An observation or receipt violates the CORE measurement contract."""


def require(condition: bool, message: str) -> None:
    """Reject invalid evidence before calculating any outcome."""
    if not condition:
        raise ScoreError(message)


def mapping(value: JSON) -> Data:
    """Narrow a JSON object at its boundary."""
    match value:
        case dict(): return value
        case _: raise ScoreError("Expected JSON object")


def sequence(value: JSON) -> list[JSON]:
    """Narrow a JSON array at its boundary."""
    match value:
        case list(): return value
        case _: raise ScoreError("Expected JSON array")


def number(value: JSON, name: str, minimum: int = 0) -> int:
    """Reject boolean, fractional and negative counters."""
    match value:
        case int() if type(value) is int and value >= minimum: return value
        case _: raise ScoreError(f"Invalid integer: {name}")


def boolean(value: JSON, name: str) -> bool:
    """Require a real boolean rather than a truthy value."""
    match value:
        case bool(): return value
        case _: raise ScoreError(f"Invalid boolean: {name}")


def digest(value: JSON, length: int) -> str:
    """Require a lowercase full source or content digest."""
    match value:
        case str() if re.fullmatch(r"[0-9a-f]{" + str(length) + "}", value): return value
        case _: raise ScoreError("Invalid SHA digest")


def first_tick(value: JSON, end: int) -> int | None:
    """Missing milestones fail their bar; malformed/future milestones are invalid."""
    if value is None:
        return None
    tick = number(value, "milestone tick", 1)
    require(tick <= end, "Future milestone")
    return tick


def world(row: Data, source: str, policy: str) -> Data:
    """Validate physical housing, complete coverage, fatal provenance and ledgers."""
    old, observed = mapping(row["mux"]), mapping(row["mux2"])
    require(number(row["days"], "days") == 48 and number(observed["schema_version"], "schema") == 1,
            "Invalid duration/schema")
    day = number(old["ticks_per_day"], "ticks per day", 1)
    end = 48 * day
    require(number(observed["start_tick"], "start") == 0
            and number(observed["end_tick"], "end") == end
            and number(observed["observed_ticks"], "coverage") == end, "Incomplete observer coverage")
    require(observed["observer_integrity_errors"] == [], "Observer integrity failure")
    inputs = mapping(observed["inputs"])
    require(all(inputs[key] is True for key in ("steward", "defence", "defence_enabled_end"))
            and inputs["player_script"] is False and inputs["story"] == inputs["neighbours"] == "on",
            "Non-equivalent run inputs")
    require(inputs["source_sha"] == source and inputs["policy_sha256"] == policy, "Source/policy mismatch")
    stock = first_tick(old["stockpile_first_tick"], end)
    house = first_tick(observed["walled_beds_for_all_first_tick"], end)
    winter = boolean(old["first_winter_observed"], "winter observed")
    complete = boolean(old["first_winter_complete"], "winter complete")
    require(not complete or winter, "Winter completion without observation")
    cold = number(old["first_winter_cold_deaths"], "cold deaths")
    snap = number(old["cold_snap_observed_ticks"], "snap coverage")
    zero = number(old["cold_snap_firewood_zero_ticks"], "zero fuel")
    require(zero <= snap <= end, "Invalid snap coverage")
    alive = number(old["alive_end"], "alive")
    alive48 = boolean(old["alive_at_day48"], "alive day48")
    require(alive48 == (alive > 0) and number(row["alive_end"], "row alive") == alive, "Alive disagreement")
    wolves = number(observed["wolf_deaths"], "wolf deaths")
    fatal = number(observed["fatal_animal_events"], "fatal events")
    combat = number(observed["combat_deaths"], "combat deaths")
    require(wolves <= fatal <= combat and number(mapping(row["deaths"])["combat"], "row combat") == combat,
            "Death count disagreement")
    events = sequence(observed["wolf_death_events"])
    require(len(events) == wolves, "Wolf provenance count differs")
    victims: set[int] = set()
    for value in events:
        event = mapping(value)
        require(number(event["tick"], "fatal tick") < end, "Fatal event after run")
        victim = number(event["victim"], "victim")
        require(victim not in victims and event["mechanic"] == "predator", "Wrong/duplicate fatal provenance")
        victims.add(victim)
        number(event["attacker"], "attacker")
        number(event["kind"], "species")
    ledger = number(row["ledger_violation_ticks"], "ledger violations")
    reserve = number(row["reservation_violation_ticks"], "reservation violations")
    require(ledger <= end and reserve <= end, "Invalid violation duration")
    return {"seed": number(row["seed"], "seed"), "ticks_per_day": day, "wolf_deaths": wolves,
            "cold_snap_observed": snap > 0, "first_winter_complete": complete,
            "bars": {"stockpile_day1": stock is not None and stock <= day,
                     "walled_beds_day10": house is not None and house <= 10 * day,
                     "zero_cold_deaths": winter and complete and cold == 0,
                     "firewood_during_snap": zero == 0 if snap > 0 else None,
                     "alive_day48": alive48, "wolf_deaths_at_most_one": wolves <= 1,
                     "exact_ledger": ledger == reserve == 0}}


def score(report: Data, receipt: Data, *, phase: str, brain: str) -> Data:
    """Score one complete arm without making a full milestone acceptance claim."""
    try:
        require(phase in ("calibration", "heldout") and brain in ("base", "mind"), "Invalid phase/brain")
        expected = list(range(8)) if phase == "calibration" else list(range(1232, 1264))
        require(receipt["phase"] == phase and receipt["brain"] == brain and receipt["defence"] is True,
                "Receipt phase/brain/defence mismatch")
        require([number(seed, "receipt seed") for seed in sequence(receipt["seeds"])] == expected
                and number(receipt["days"], "receipt days") == 48, "Receipt coverage mismatch")
        require(receipt["policy_frozen_before_run"] is True, "Missing pre-run freeze attestation")
        source, policy = digest(receipt["source_sha"], 40), digest(receipt["policy_sha256"], 64)
        binary, scorer_sha = digest(receipt["binary_sha256"], 64), digest(receipt["scorer_sha256"], 64)
        validator_sha = digest(receipt["validator_sha256"], 64)
        require(brain != "base" or source == BASE_SHA, "Old brain must be the exact CORE BASE source")
        if phase == "calibration":
            require(1 <= number(receipt["round"], "round", 1) <= 4, "Calibration exceeds four rounds")
        else:
            require(receipt["round"] is None, "Held-out must not be a tuning round")
        require(report["scene"] == "colony" and number(report["days"], "report days") == 48,
                "Expected colony exactly 48 days")
        rows = [mapping(value) for value in sequence(report["seeds"])]
        require(sorted(number(row["seed"], "seed") for row in rows) == expected, "Wrong/duplicate seed set")
        results = [world(row, source, policy) for row in sorted(rows, key=lambda row: number(row["seed"], "seed"))]
        bars: Data = {}
        for key, target in TARGETS.items():
            required = target if phase == "heldout" else (target * 8 + 31) // 32
            passed = sum(mapping(row["bars"])[key] is True for row in results)
            bars[key] = {"passed": passed, "required": required, "total": len(expected),
                         "status": "PASS" if passed >= required else "FAIL"}
        exposed = sum(row["cold_snap_observed"] is True for row in results)
        protected = sum(mapping(row["bars"])["firewood_during_snap"] is True for row in results)
        required = max(0, exposed - 1)
        bars["firewood_during_snap"] = {"passed": protected, "required": required, "total": exposed,
            "unexposed": len(expected) - exposed,
            "status": "NOT_EXPOSED" if exposed == 0 else ("PASS" if protected >= required else "FAIL")}
        return {"schema_version": 1, "status": "VALID", "phase": phase, "brain": brain,
                "source_sha": source, "policy_sha256": policy, "binary_sha256": binary,
                "scorer_sha256": scorer_sha, "validator_sha256": validator_sha, "acceptance_claim": False,
                "passed": all(mapping(bar)["status"] != "FAIL" for bar in bars.values()),
                "bars": bars, "seeds": results}
    except KeyError as error:
        raise ScoreError(f"Missing measurement field: {error}") from error


def score_pair(new_report: Data, new_receipt: Data, base_report: Data, base_receipt: Data) -> Data:
    """Compare matching held-out worlds; BASE outcomes are descriptive controls."""
    new = score(new_report, new_receipt, phase="heldout", brain="mind")
    base = score(base_report, base_receipt, phase="heldout", brain="base")
    require(new["scorer_sha256"] == base["scorer_sha256"], "Pair scorer mismatch")
    require(new["validator_sha256"] == base["validator_sha256"], "Pair validator mismatch")
    require([mapping(row)["ticks_per_day"] for row in sequence(new["seeds"])]
            == [mapping(row)["ticks_per_day"] for row in sequence(base["seeds"])], "Pair calendar mismatch")
    return {"schema_version": 1, "status": "VALID", "comparison_valid": True, "worlds": 64,
            "days": 48, "seeds": list(range(1232, 1264)), "evaluation_bars_passed": new["passed"],
            "acceptance_claim": False, "acceptance_scope": "outcome_rows_only_not_full_CORE_gate",
            "arms": {"mind": new, "base": base},
            "freeze_boundary": "Pre-run freeze is attested, not proved by hash equality."}


def reject_constant(value: str) -> JSON:
    """Non-finite JSON values are never valid evidence."""
    raise ScoreError(f"Non-finite JSON value: {value}")


def read_bound_run(summary: Path, receipt_path: Path) -> tuple[Data, Data, Data]:
    """Bind exact raw bytes and the current scorer before any score is returned."""
    raw, receipt_raw = summary.read_bytes(), receipt_path.read_bytes()
    receipt = mapping(json.loads(receipt_raw, object_pairs_hook=validator.unique_pairs, parse_constant=reject_constant))
    summary_sha = hashlib.sha256(raw).hexdigest()
    require(receipt.get("summary_sha256") == summary_sha, "Summary bytes not bound to receipt")
    require(receipt.get("scorer_sha256") == hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "Receipt does not bind this scorer")
    require(receipt.get("validator_sha256") == hashlib.sha256(Path(validator.__file__).read_bytes()).hexdigest(),
            "Receipt does not bind this validator")
    report = mapping(json.loads(raw, object_pairs_hook=validator.unique_pairs, parse_constant=reject_constant))
    return report, receipt, {"summary_sha256": summary_sha, "receipt_sha256": hashlib.sha256(receipt_raw).hexdigest()}


def main() -> int:
    """Write a new external score; malformed evidence exits 2 without output."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("summary", type=Path)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--phase", choices=("calibration", "heldout"), required=True)
    parser.add_argument("--base-summary", type=Path)
    parser.add_argument("--base-receipt", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        require(not args.output.exists() and not args.output.is_symlink(), "Output must be new")
        require(not args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]), "Output must be external")
        report, receipt, hashes = read_bound_run(args.summary, args.receipt)
        paired = args.base_summary is not None or args.base_receipt is not None
        require(not paired or (args.base_summary is not None and args.base_receipt is not None
                              and args.phase == "heldout"), "Pair requires heldout and both BASE paths")
        if paired:
            base, base_receipt, base_hashes = read_bound_run(args.base_summary, args.base_receipt)
            result = score_pair(report, receipt, base, base_receipt)
            result["input_hashes"] = {"mind": hashes, "base": base_hashes}
            passed = result["evaluation_bars_passed"] is True
        else:
            result = score(report, receipt, phase=args.phase, brain="mind")
            result["input_hashes"] = {"mind": hashes}
            passed = result["passed"] is True
        with args.output.open("x") as stream:
            stream.write(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
        print("CORE_OUTCOMES " + ("PASS" if passed else "FAIL") + "; full CORE acceptance not claimed")
        return 0 if passed else 1
    except (ValueError, OSError) as error:
        print(f"CORE_OUTCOMES INVALID: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
