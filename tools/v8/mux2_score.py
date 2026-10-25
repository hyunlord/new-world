#!/usr/bin/env python3
"""M-UX2 scorer: exact input coverage, physical-house oracle and fatal-predator provenance."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import re

TARGETS = {"stockpile_day1": 32, "walled_beds_day10": 30, "zero_cold_deaths": 28,
           "firewood_during_snap": 24, "alive_day48": 30, "wolf_deaths_at_most_one": 28,
           "exact_ledger": 32}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def integer(value, name: str, minimum: int = 0) -> int:
    require(type(value) is int and value >= minimum, f"Invalid {name}")
    return value


def boolean(value, name: str) -> bool:
    require(type(value) is bool, f"Invalid {name}")
    return value


def sha(value, length: int, name: str) -> str:
    require(type(value) is str and re.fullmatch(r"[0-9a-f]{" + str(length) + "}", value) is not None,
            f"Invalid {name}")
    return value


def first_tick(value, end: int, name: str):
    if value is not None:
        integer(value, name, 1)
        require(value <= end, f"Future {name}")
    return value


def score(report: dict, receipt: dict, *, phase: str, defence: bool) -> dict:
    require(phase in ("calibration", "heldout"), "Invalid phase")
    expected = list(range(8)) if phase == "calibration" else list(range(1200, 1232))
    require(receipt["phase"] == phase and receipt["defence"] is defence, "Receipt arm/phase mismatch")
    require([integer(seed, "receipt seed") for seed in receipt["seeds"]] == expected and integer(receipt["days"], "receipt days") == 48, "Receipt coverage mismatch")
    require(receipt["policy_frozen_before_run"] is True, "Missing pre-run policy freeze")
    source = sha(receipt["source_sha"], 40, "source SHA")
    policy = sha(receipt["policy_sha256"], 64, "policy SHA")
    binary = sha(receipt["binary_sha256"], 64, "binary SHA")
    if phase == "calibration":
        require(1 <= integer(receipt["round"], "calibration round", 1) <= 4, "Calibration exceeds four rounds")
    else:
        require(receipt.get("round") is None, "Held-out must not be a tuning round")
    require(report["scene"] == "colony" and report["days"] == 48, "Expected colony exactly48days")
    rows = report["seeds"]
    require(type(rows) is list and len(rows) == len(expected), "Missing/extra worlds")
    actual = [integer(row["seed"], "seed") for row in rows]
    require(sorted(actual) == expected, "Wrong/duplicate seed set")
    results = []
    for row in sorted(rows, key=lambda entry: entry["seed"]):
        old, observed = row["mux"], row["mux2"]
        require(integer(row["days"], "days") == 48 and integer(observed["schema_version"], "schema") == 1, "Invalid duration/schema")
        day = integer(old["ticks_per_day"], "ticks_per_day", 1)
        end = 48 * day
        require(integer(observed["start_tick"], "start tick") == 0 and integer(observed["end_tick"], "end tick") == end
                and integer(observed["observed_ticks"], "observed ticks") == end, "Incomplete/censored observer coverage")
        require(observed["observer_integrity_errors"] == [], "Observer integrity failure")
        inputs = observed["inputs"]
        require(inputs["steward"] is True and inputs["defence"] is defence
                and inputs["defence_enabled_end"] is defence and inputs["player_script"] is False
                and inputs["story"] == "on" and inputs["neighbours"] == "on", "Non-equivalent run inputs")
        require(inputs["source_sha"] == source and inputs["policy_sha256"] == policy,
                "Run source/policy differs from receipt")
        stock = first_tick(old["stockpile_first_tick"], end, "stockpile tick")
        house = first_tick(observed["walled_beds_for_all_first_tick"], end, "walled beds tick")
        winter_started = boolean(old["first_winter_observed"], "winter observed")
        winter_complete = boolean(old["first_winter_complete"], "winter complete")
        require(not winter_complete or winter_started, "Winter completion without observation")
        cold = integer(old["first_winter_cold_deaths"], "cold deaths")
        snap = integer(old["cold_snap_observed_ticks"], "snap ticks")
        zero = integer(old["cold_snap_firewood_zero_ticks"], "zero-fuel ticks")
        require(zero <= snap <= end, "Invalid snap coverage")
        alive = integer(old["alive_end"], "alive_end")
        alive48 = boolean(old["alive_at_day48"], "alive_at_day48")
        require(alive48 == (alive > 0) and integer(row["alive_end"], "row alive") == alive, "Alive measurements disagree")
        wolves = integer(observed["wolf_deaths"], "wolf deaths")
        fatal = integer(observed["fatal_animal_events"], "fatal animal events")
        combat = integer(observed["combat_deaths"], "combat deaths")
        require(wolves <= fatal <= combat and integer(row["deaths"]["combat"], "row combat deaths") == combat, "Death counts disagree")
        events = observed["wolf_death_events"]
        require(type(events) is list and len(events) == wolves, "Wolf provenance count differs")
        victims = set()
        for event in events:
            require(integer(event["tick"], "fatal event tick") < end, "Fatal event after run")
            victim = integer(event["victim"], "victim")
            require(victim not in victims and event["mechanic"] == "predator", "Wrong/duplicate fatal provenance")
            victims.add(victim)
            integer(event["attacker"], "attacker")
            integer(event["kind"], "species code")
        ledger = integer(row["ledger_violation_ticks"], "ledger violations")
        reserve = integer(row["reservation_violation_ticks"], "reservation violations")
        flags = {"stockpile_day1": stock is not None and stock <= day,
                 "walled_beds_day10": house is not None and house <= 10 * day,
                 "zero_cold_deaths": winter_started and winter_complete and cold == 0,
                 "firewood_during_snap": snap > 0 and zero == 0,
                 "alive_day48": alive48,
                 "wolf_deaths_at_most_one": wolves <= 1,
                 "exact_ledger": ledger == 0 and reserve == 0}
        results.append({"seed": row["seed"], "bars": flags, "wolf_deaths": wolves,
                        "cold_snap_observed": snap > 0, "first_winter_complete": winter_complete})
    thresholds = TARGETS if phase == "heldout" else {k: (v * 8 + 31) // 32 for k, v in TARGETS.items()}
    bars = {key: {"passed": sum(row["bars"][key] for row in results), "required": minimum,
                  "total": len(expected)} for key, minimum in thresholds.items()}
    return {"schema_version": 1, "status": "VALID", "phase": phase, "defence": defence,
            "acceptance_claim": False, "acceptance_scope": "single_arm_only",
            "source_sha": source, "policy_sha256": policy, "binary_sha256": binary,
            "bars": bars, "passed": all(v["passed"] >= v["required"] for v in bars.values()), "seeds": results}


def score_pair(on_report: dict, on_receipt: dict, off_report: dict, off_receipt: dict) -> dict:
    """Validate all64 held-out worlds; only defence-ON thresholds gate acceptance."""
    on = score(on_report, on_receipt, phase="heldout", defence=True)
    off = score(off_report, off_receipt, phase="heldout", defence=False)
    for key in ("source_sha", "policy_sha256", "binary_sha256"):
        require(on[key] == off[key], f"Pair {key} mismatch")
    # Each arm already requires exact seed membership, 48days and continuous coverage.
    on_days = {row["seed"]: row["mux"]["ticks_per_day"] for row in on_report["seeds"]}
    off_days = {row["seed"]: row["mux"]["ticks_per_day"] for row in off_report["seeds"]}
    require(on_days == off_days, "Pair tick duration mismatch")
    return {"schema_version": 1, "status": "VALID", "phase": "heldout",
            "comparison_valid": True, "worlds": 64, "days": 48,
            "seeds": list(range(1200, 1232)), "source_sha": on["source_sha"],
            "policy_sha256": on["policy_sha256"], "binary_sha256": on["binary_sha256"],
            "passed": on["passed"], "acceptance_claim": on["passed"],
            "acceptance_scope": "matched_pair_defence_on_bars_only",
            "arms": {"on": on, "off": off},
            "freeze_boundary": "Root attests policy/source/binary/scorer frozen before held-out access; hash equality does not prove chronology."}


def read_bound_run(summary: Path, receipt_path: Path) -> tuple[dict, dict, dict]:
    """Bind each parsed run to exact raw summary and receipt bytes."""
    raw = summary.read_bytes()
    receipt_raw = receipt_path.read_bytes()
    receipt = json.loads(receipt_raw)
    summary_sha = hashlib.sha256(raw).hexdigest()
    require(summary_sha == receipt["summary_sha256"], "Summary bytes not bound to receipt")
    return json.loads(raw), receipt, {"summary_sha256": summary_sha,
                                    "receipt_sha256": hashlib.sha256(receipt_raw).hexdigest()}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("summary", type=Path)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--phase", required=True, choices=("calibration", "heldout"))
    parser.add_argument("--defence", required=True, choices=("on", "off"))
    parser.add_argument("--off-summary", type=Path, help="Validate final pair; positional summary must be ON")
    parser.add_argument("--off-receipt", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    paired = args.off_summary is not None or args.off_receipt is not None
    require(not paired or (args.off_summary is not None and args.off_receipt is not None
                           and args.phase == "heldout" and args.defence == "on"),
            "Pair mode requires both OFF paths and heldout/ON positional input")
    report, receipt, hashes = read_bound_run(args.summary, args.receipt)
    if paired:
        off_report, off_receipt, off_hashes = read_bound_run(args.off_summary, args.off_receipt)
        result = score_pair(report, receipt, off_report, off_receipt)
        result["input_hashes"] = {"on": hashes, "off": off_hashes}
    else:
        result = score(report, receipt, phase=args.phase, defence=args.defence == "on")
        result.update(hashes)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    if not paired and args.defence == "off":
        print("MUX2_CONTROL VALID (outcome bars descriptive; no acceptance claim)")
        return 0
    print("MUX2_PAIR " if paired else "MUX2_ARM ", "PASS" if result["passed"] else "FAIL", sep="")
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
