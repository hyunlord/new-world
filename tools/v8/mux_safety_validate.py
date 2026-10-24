"""Strict boundary checks for the nine deterministic MUX colony fixtures."""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 tools/v8/mux_safety_validate.py validate CAPTURE_DIR
from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final, TypeAlias

JSON: TypeAlias = str | int | float | bool | None | list["JSON"] | dict[str, "JSON"]


@dataclass(frozen=True, slots=True)
class Fixture:
    """The fixed seed, duration and input contract for one safety fixture."""

    name: str
    count: int
    days: int
    script: str
    story: str = "on"
    neighbours: str = "on"
    determinism: str = "no"


FIXTURES: Final = (
    Fixture("soak", 4, 20, "soak_script.json", "no-beasts", "on", "yes"),
    Fixture("story", 2, 48, "m5_u_script.json"),
    Fixture("neighbours", 2, 48, "m55_u_script.json"),
    Fixture("knowledge_f10", 8, 4, "m4_f10_script.json"),
    Fixture("m55_pressure", 8, 48, "m55_u_script.json"),
    Fixture("m55_nodraft", 8, 48, "m55_u_nodraft_script.json"),
    Fixture("m55_storyoff", 8, 48, "m55_u_script.json", "off"),
    Fixture("m55_noneighbours", 8, 48, "m55_u_script.json", "on", "off"),
    Fixture("m55_quiet", 8, 48, "m55_u_script.json", "off", "off"),
)


class SafetyError(ValueError):
    """A capture is missing or violates its registered fixture contract."""

    def __init__(self, path: Path, detail: str) -> None:
        self.path = path
        self.detail = detail
        super().__init__(f"{path}: {detail}")


def unique_pairs(pairs: list[tuple[str, JSON]]) -> JSON:
    """Reject duplicate JSON keys instead of silently retaining the last value."""
    result: dict[str, JSON] = {}
    for key, value in pairs:
        if key in result:
            raise SafetyError(Path("<JSON>"), f"duplicate key {key}")
        result[key] = value
    return result


def validate_fixture(root: Path, fixture: Fixture) -> None:
    """Parse exact seed coverage and conservation counters from a summary."""
    path = root / "colony" / fixture.name / "summary.json"
    try:
        document: JSON = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_pairs)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise SafetyError(path, str(error)) from error
    match document:
        case {"scene": "colony", "days": int(days), "seeds": list(rows)}:
            if type(days) is not int or days != fixture.days or len(document) != 3:
                raise SafetyError(path, "wrong days or top-level schema")
            if len(rows) != fixture.count:
                raise SafetyError(path, f"expected {fixture.count} seeds, got {len(rows)}")
            for expected_seed, row in enumerate(rows):
                match row:
                    case {"seed": int(seed), "ledger_violation_ticks": int(ledger),
                          "reservation_violation_ticks": int(reservations)}:
                        if any(type(value) is not int for value in (seed, ledger, reservations)):
                            raise SafetyError(path, "seed/counter must be an integer, not bool")
                        if seed != expected_seed or ledger != 0 or reservations != 0:
                            raise SafetyError(path, f"seed {expected_seed}: wrong seed/order or nonzero violations")
                    case _:
                        raise SafetyError(path, f"seed {expected_seed}: missing typed seed/invariant fields")
        case _:
            raise SafetyError(path, "expected colony scene, integer days and seed array")


def validate_colonies(root: Path) -> None:
    """Require every fixture; absent or malformed inputs never count as success."""
    for fixture in FIXTURES:
        validate_fixture(root, fixture)


def validate_identity(root: Path) -> None:
    """Reject truncated baseline captures before content comparators are called."""
    identity = root / "identity"
    samples = {f"game_v26_s{seed}{suffix}.jsonl" for seed in range(32) for suffix in ("", "_deaths")}
    chronicles = {f"{kind}_{seed}.json" for seed in range(32) for kind in ("chronicle", "relations")}
    if {path.name for path in (identity / "probe").glob("*.jsonl")} != samples:
        raise SafetyError(identity, "expected both raw sample files for each of 32 seeds")
    if {path.name for path in (identity / "chronicle").glob("*.json")} != chronicles:
        raise SafetyError(identity, "expected chronicle and relations for each of 32 seeds")
    path = identity / "probe" / "summary.json"
    try:
        document: JSON = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_pairs)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise SafetyError(path, str(error)) from error
    match document:
        case {"scene": "game_v26", "seeds": {"start": 0, "end_exclusive": 32},
              "days": dict(days), "runs": list(runs)}:
            if set(days) != {str(day) for day in range(1, 121)} or len(runs) != 32:
                raise SafetyError(path, "expected 120 days and 32 runs")
            for seed, row in enumerate(runs):
                match row:
                    case {"seed": int(actual)} if type(actual) is int and actual == seed:
                        continue
                    case _:
                        raise SafetyError(path, "baseline seed order/coverage differs")
        case _:
            raise SafetyError(path, "wrong baseline identity schema/seed range")


def main(arguments: list[str]) -> int:
    """Expose immutable fixture rows and capture validation to the shell runner."""
    match arguments:
        case ["list"]:
            for fixture in FIXTURES:
                print(f"{fixture.name}\t{fixture.count}\t{fixture.days}\t{fixture.script}"
                      f"\t{fixture.story}\t{fixture.neighbours}\t{fixture.determinism}")
        case ["validate", root]:
            try:
                validate_colonies(Path(root))
                validate_identity(Path(root))
            except SafetyError as error:
                print(f"MUX_SAFETY FAIL {error}", file=sys.stderr)
                return 1
            print("MUX_SAFETY coverage PASS: 9 fixtures, exact seed sets, zero invariant violations")
        case _:
            print("usage: mux_safety_validate.py list | validate CAPTURE_DIR", file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
