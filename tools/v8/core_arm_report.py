#!/usr/bin/env python3
"""Strict external aggregation of complete CORE decision-arm observations."""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 tools/v8/core_arm_report.py --help
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from mux_safety_validate import FIXTURES, JSON, unique_pairs

TIMING: Final = "inclusive scopes overlap; do not sum nested timings"
DISCLAIMER: Final = (
    "Timings are diagnostic, possibly contended inclusive nanoseconds. "
    "Nested scope costs overlap and are never summed as system time. "
    "Calls are scope entries or counters, not unique decisions; evaluation is not choice. "
    "Command basenames validate run labels, not script bytes; bind scripts and binaries externally."
)


class ReportError(ValueError):
    """An input artifact does not satisfy its declared measurement contract."""

    def __init__(self, detail: str) -> None:
        super().__init__(detail)


@dataclass(frozen=True, slots=True)
class RunSpec:
    """Exact coverage and mode contract for one fixture."""

    name: str
    count: int
    days: int
    steward: bool
    commands: str | None = None
    story: str = "on"
    neighbours: str = "on"
    check_determinism: bool = False

    def inputs(self) -> JSON:
        """The feature emitter's required run inputs; defence remains default."""
        return {"steward": self.steward, "defence": None, "commands": self.commands,
                "story": self.story, "neighbours": self.neighbours,
                "check_determinism": self.check_determinism}


@dataclass(frozen=True, slots=True)
class Scope:
    """One scope's accumulated counts and inclusive time, never a system total."""

    name: str
    calls: int
    inclusive_ns: int


@dataclass(frozen=True, slots=True)
class Run:
    """Validated complete fixture with source-byte identity."""

    spec: RunSpec
    source: str
    sha256: str
    summary_sha256: str
    scopes: tuple[Scope, ...]


def reject_constant(value: str) -> JSON:
    """JSON NaN/Infinity are not measurement values."""
    raise ReportError(f"non-finite JSON constant {value}")


def read_json(path: Path) -> JSON:
    """Parse JSON without accepting duplicate keys or non-finite constants."""
    return json.loads(path.read_bytes(), object_pairs_hook=unique_pairs,
                      parse_constant=reject_constant)


def mapping(value: JSON, keys: set[str], label: str) -> dict[str, JSON]:
    """Require an exact object schema at the input boundary."""
    match value:
        case dict() if set(value) == keys:
            return value
        case _:
            raise ReportError(f"{label}: expected exact keys {sorted(keys)}")


def natural(value: JSON, label: str) -> int:
    """Parse a nonnegative integer without accepting booleans."""
    match value:
        case int() if type(value) is int and value >= 0:
            return value
        case _:
            raise ReportError(f"{label}: expected nonnegative integer, not bool")


def sequence(value: JSON, label: str) -> list[JSON]:
    """Require an array; missing data never acts like an empty observation."""
    match value:
        case list():
            return value
        case _:
            raise ReportError(f"{label}: expected array")


def read_registry(path: Path) -> tuple[str, ...]:
    """Read an explicit runtime-exported manifest; never infer names from Rust."""
    return registry_names(read_json(path))


def registry_names(value: JSON) -> tuple[str, ...]:
    """Validate the supplied runtime registry without reading source code."""
    data = mapping(value, {"schema_version", "scopes"}, "registry")
    if natural(data["schema_version"], "registry schema") != 1:
        raise ReportError("unsupported registry schema")
    names = []
    for name in sequence(data["scopes"], "registry scopes"):
        match name:
            case str() if name and not any(character.isspace() for character in name):
                names.append(name)
            case _:
                raise ReportError("registry scope names must be nonempty strings without whitespace")
    if not names or len(set(names)) != len(names):
        raise ReportError("registry must contain unique nonempty scopes")
    return tuple(sorted(names))


def summary_digest(path: Path, spec: RunSpec) -> str:
    """Require the completed summary and exact conservation-safe seed coverage."""
    raw = path.read_bytes()
    data = mapping(json.loads(raw, object_pairs_hook=unique_pairs, parse_constant=reject_constant),
                   {"scene", "days", "seeds"}, "summary")
    if data["scene"] != "colony" or natural(data["days"], "summary days") != spec.days:
        raise ReportError(f"{spec.name}: summary scene/duration mismatch")
    seeds = []
    for row in sequence(data["seeds"], "summary seeds"):
        match row:
            case {"seed": seed, "ledger_violation_ticks": ledger, "reservation_violation_ticks": reservations}:
                seeds.append(natural(seed, "summary seed"))
                if natural(ledger, "ledger violations") != 0 or natural(reservations, "reservation violations") != 0:
                    raise ReportError(f"{spec.name}: conservation violations")
            case _:
                raise ReportError(f"{spec.name}: missing summary seed/conservation fields")
    if sorted(seeds) != list(range(spec.count)):
        raise ReportError(f"{spec.name}: summary seed coverage mismatch")
    return hashlib.sha256(raw).hexdigest()


def read_run(path: Path, spec: RunSpec, registry: tuple[str, ...]) -> Run:
    """Validate every primary world's ticks, input mode and complete scope set."""
    raw = path.read_bytes()
    data = mapping(json.loads(raw, object_pairs_hook=unique_pairs, parse_constant=reject_constant),
                   {"scene", "days", "inputs", "seeds"}, spec.name)
    if data["scene"] != "colony" or natural(data["days"], "days") != spec.days:
        raise ReportError(f"{spec.name}: scene/duration mismatch")
    expected = spec.inputs()
    actual = data["inputs"]
    if actual != expected or json.dumps(actual, sort_keys=True) != json.dumps(expected, sort_keys=True):
        raise ReportError(f"{spec.name}: input contract mismatch")
    rows = sequence(data["seeds"], "seeds")
    if len(rows) != spec.count:
        raise ReportError(f"{spec.name}: seed coverage mismatch")
    totals = {name: [0, 0] for name in registry}
    seen = set()
    for value in rows:
        row = mapping(value, {"seed", "profile"}, "seed row")
        seed = natural(row["seed"], "seed")
        if seed in seen or seed not in range(spec.count):
            raise ReportError(f"{spec.name}: duplicate/wrong seed {seed}")
        seen.add(seed)
        profile = mapping(row["profile"], {"schema_version", "ticks", "timing", "scopes"}, "profile")
        if natural(profile["schema_version"], "profile schema") != 1:
            raise ReportError("unsupported profile schema")
        if natural(profile["ticks"], "ticks") != spec.days * 1440 or profile["timing"] != TIMING:
            raise ReportError(f"{spec.name}: incomplete ticks or incorrect timing semantics")
        scopes = set()
        for value in sequence(profile["scopes"], "scopes"):
            scope = mapping(value, {"name", "calls", "inclusive_ns"}, "scope")
            name = scope["name"]
            match name:
                case str() if name in totals and name not in scopes:
                    pass
                case _:
                    raise ReportError(f"{spec.name}: duplicate or unregistered scope {name}")
            scopes.add(name)
            calls = natural(scope["calls"], "calls")
            elapsed = natural(scope["inclusive_ns"], "inclusive_ns")
            if calls == 0 and elapsed != 0:
                raise ReportError(f"{name}: time without any scope calls")
            totals[name][0] += calls
            totals[name][1] += elapsed
        if scopes != set(registry):
            raise ReportError(f"{spec.name}: missing scopes including zero-hit arms")
    return Run(spec, str(path.resolve()), hashlib.sha256(raw).hexdigest(),
               summary_digest(path.with_name("summary.json"), spec),
               tuple(Scope(name, *totals[name]) for name in registry))


def specifications() -> tuple[RunSpec, ...]:
    """Import the nine OFF contracts and add both required steward groups."""
    return tuple(RunSpec(f.name, f.count, f.days, False, f.script, f.story, f.neighbours,
                         f.determinism == "yes") for f in FIXTURES) + (
        RunSpec("steward_on", 2, 24, True, check_determinism=True),
        RunSpec("inventory_on", 8, 48, True),
    )


def aggregate(runs: tuple[Run, ...]) -> dict[str, JSON]:
    """Accumulate each scope independently across the exact required fixture set."""
    expected = specifications()
    if tuple(run.spec for run in runs) != expected:
        raise ReportError("expected all nine OFF fixtures and both ON groups in contract order")
    names = tuple(scope.name for scope in runs[0].scopes)
    if any(tuple(scope.name for scope in run.scopes) != names for run in runs):
        raise ReportError("registry differs across runs")
    rows: list[JSON] = []
    for index, name in enumerate(names):
        by_run: JSON = {run.spec.name: asdict(run.scopes[index]) for run in runs}
        rows.append({"name": name, "calls": sum(run.scopes[index].calls for run in runs),
                     "inclusive_ns": sum(run.scopes[index].inclusive_ns for run in runs), "by_run": by_run})
    return {"schema_version": 1, "validation": "complete_inputs", "limitations": DISCLAIMER,
            "worlds": sum(run.spec.count for run in runs),
            "world_ticks": sum(run.spec.count * run.spec.days * 1440 for run in runs),
            "runs": [{"contract": asdict(run.spec), "source": run.source, "sha256": run.sha256,
                      "summary_sha256": run.summary_sha256} for run in runs],
            "scopes": rows}


def markdown(runs: tuple[Run, ...]) -> str:
    """Render scopes separately; never produce a sum of overlapping scope costs."""
    lines = ["# CORE arm observations", "", DISCLAIMER, "", "| Scope | Calls | Inclusive ns (diagnostic) |",
             "|---|---:|---:|"]
    for index, scope in enumerate(runs[0].scopes):
        name = scope.name.replace("|", "&#124;").replace("`", "&#96;")
        lines.append(f"| {name} | {sum(run.scopes[index].calls for run in runs)} | "
                     f"{sum(run.scopes[index].inclusive_ns for run in runs)} |")
    return "\n".join(lines) + "\n"


def main() -> int:
    """Validate all sources before writing either explicitly named report."""
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("registry", "off-root", "steward-dir", "inventory-dir", "json-out", "markdown-out"):
        parser.add_argument(f"--{option}", type=Path, required=True)
    args = parser.parse_args()
    try:
        registry_raw = args.registry.read_bytes()
        registry = registry_names(json.loads(registry_raw, object_pairs_hook=unique_pairs,
                                             parse_constant=reject_constant))
        specs = specifications()
        paths = [args.off_root / "colony" / spec.name / "perf_arms.json" for spec in specs[:-2]]
        paths.extend([args.steward_dir / "perf_arms.json", args.inventory_dir / "perf_arms.json"])
        runs = tuple(read_run(path, spec, registry) for path, spec in zip(paths, specs))
        result = aggregate(runs)
        result["registry_manifest"] = {"source": str(args.registry.resolve()),
            "sha256": hashlib.sha256(registry_raw).hexdigest()}
        if args.json_out.resolve() == args.markdown_out.resolve() or any(path.exists() for path in (args.json_out, args.markdown_out)):
            raise ReportError("report output paths must be distinct and new")
        if any(not path.parent.is_dir() for path in (args.json_out, args.markdown_out)):
            raise ReportError("both report output parents must already exist")
        with args.json_out.open("x", encoding="utf-8") as handle:
            handle.write(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
        with args.markdown_out.open("x", encoding="utf-8") as handle:
            handle.write(markdown(runs))
    except (OSError, UnicodeError, ValueError) as error:
        print(f"CORE_ARMS INVALID: {error}", file=sys.stderr)
        return 1
    print(f"CORE_ARMS VALID: {len(runs)} complete groups; {len(registry)} scopes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
