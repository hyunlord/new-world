#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy==2.4.3"]
# ///
# Usage: python3 f3_literal_audit.py tuning|evaluation RUN_ROOT CORRECTED_EVAL_JSON [--json]
"""Independent literal F3 reconstruction; no evaluator imports or workload launches."""

from __future__ import annotations

import hashlib
import json
import math
import os
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from enum import Enum, auto
from itertools import combinations
from pathlib import Path
from typing import Final, assert_never

os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")
os.environ.setdefault("OMP_NUM_THREADS", "1")
sys.dont_write_bytecode = True
import numpy as np
from numpy.typing import NDArray

import family_trace_recompute as raw

AXES: Final = ("ns", "ha", "rd", "p")
REPLICATES: Final = 2000
PINS: Final = {
    "family_trace_recompute.py": "4f6902c44ac7cdd64c6ab426887a5d894ad177168ffce39661f6020948939df9",
    "family_trace_audit_runner.py": "5faafff0b6c51381953e82bfbe25fe48b8065c47d52b978da1cf546f812f56cc",
}
type Vector = tuple[int, int, int, int]
type PairKey = tuple[int, int]
type Matrix = NDArray[np.float64]


@dataclass(frozen=True, slots=True)
class Person:
    identifier: int
    mother: int | None
    father: int | None
    axes: Vector


@dataclass(frozen=True, slots=True)
class Pair:
    ids: PairKey
    tick: int | None
    settlement: int | None
    stages: tuple[str, str] | None

    def json(self) -> raw.Json:
        return {"ids": list(self.ids), "tick": self.tick, "settlement": self.settlement,
                "stages": list(self.stages) if self.stages is not None else None}


@dataclass(frozen=True, slots=True)
class World:
    seed: int
    people: Mapping[int, Person]
    siblings: tuple[Pair, ...]
    unrelated: tuple[Pair, ...]
    excluded: tuple[Pair, ...]
    unassigned_samples: int
    unassigned_people: int


@dataclass(frozen=True, slots=True)
class Centered:
    count: Matrix
    mean_x: Matrix
    mean_y: Matrix
    xx: Matrix
    yy: Matrix
    xy: Matrix


class Kind(Enum):
    DIFFERENCE = auto()
    MEAN = auto()
    AXIS = auto()


@dataclass(frozen=True, slots=True)
class Statistic:
    name: str
    kind: Kind
    group: int = 0
    axis: int = 0


STATISTICS: Final = (
    Statistic("F3", Kind.DIFFERENCE),
    Statistic("F3-siblings", Kind.MEAN, 0), Statistic("F3-unrelated", Kind.MEAN, 1),
    *(Statistic(f"F3-{group}-{axis}", Kind.AXIS, index, coordinate)
      for index, group in enumerate(("siblings", "unrelated")) for coordinate, axis in enumerate(AXES)),
)


def require(condition: bool, where: str, message: str) -> None:
    if not condition:
        raise raw.InputError(where, message)


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reconstruct(seed: int, people_rows: Sequence[raw.Row], snapshots: Sequence[raw.Row]) -> World:
    people: dict[int, Person] = {}
    for row in people_rows:
        identifier = row.num("id")
        require(identifier not in people, row.where, "duplicate person")
        axes = raw.Row.parse(row.get("temperament"), row.where)
        values = tuple(axes.num(axis) for axis in AXES)
        require(max(values) <= 1000, row.where, "temperament exceeds 1000")
        people[identifier] = Person(identifier, row.optional("mother"), row.optional("father"),
                                    (values[0], values[1], values[2], values[3]))
    families: dict[PairKey, list[int]] = {}
    for person in people.values():
        if person.mother is not None and person.father is not None:
            families.setdefault((person.mother, person.father), []).append(person.identifier)
    siblings = {
        (first, second): Pair((first, second), None, None, None)
        for members in families.values() for first, second in combinations(sorted(members), 2)
    }
    unrelated: list[Pair] = []
    excluded: list[Pair] = []
    seen: set[PairKey] = set()
    unassigned: set[int] = set()
    unassigned_samples = 0
    previous_tick = -1
    for snapshot in snapshots:
        tick = snapshot.num("tick")
        require(tick > previous_tick, snapshot.where, "snapshots must be strictly chronological")
        previous_tick = tick
        settlements: dict[int, list[int]] = {}
        stages: dict[int, str] = {}
        for member in snapshot.rows("people"):
            identifier = member.num("id")
            require(identifier in people and identifier not in stages, member.where, "unknown or duplicate living person")
            stage = member.text("life_stage")
            require(stage in ("adult", "child", "elder"), member.where, "unknown life stage")
            stages[identifier] = stage
            settlement = member.optional("settlement")
            if settlement is None:
                unassigned.add(identifier)
                unassigned_samples += 1
            else:
                settlements.setdefault(settlement, []).append(identifier)
        for settlement, residents in sorted(settlements.items()):
            for first, second in combinations(sorted(residents), 2):
                key = first, second
                if key in seen:
                    continue
                seen.add(key)
                ordered_stages = sorted((stages[first], stages[second]))
                pair = Pair(key, tick, settlement, (ordered_stages[0], ordered_stages[1]))
                a, b = people[first], people[second]
                a_parents = {parent for parent in (a.mother, a.father) if parent is not None}
                b_parents = {parent for parent in (b.mother, b.father) if parent is not None}
                close_kin = second in a_parents or first in b_parents or bool(a_parents & b_parents)
                if key in siblings:
                    siblings[key] = pair
                elif close_kin:
                    excluded.append(pair)
                else:
                    unrelated.append(pair)
    return World(seed, people, tuple(siblings[key] for key in sorted(siblings)), tuple(unrelated),
                 tuple(excluded), unassigned_samples, len(unassigned))


def oriented(world: World, pairs: Sequence[Pair]) -> tuple[Matrix, Matrix]:
    x: list[Vector] = []
    y: list[Vector] = []
    for pair in pairs:
        first, second = (world.people[identifier].axes for identifier in pair.ids)
        x.extend((first, second))
        y.extend((second, first))
    return np.asarray(x, dtype=np.float64).reshape((-1, 4)), np.asarray(y, dtype=np.float64).reshape((-1, 4))


def centered(worlds: Sequence[World]) -> Centered:
    count = np.zeros((len(worlds), 2), dtype=np.float64)
    arrays = [np.zeros((len(worlds), 2, 4), dtype=np.float64) for _ in range(5)]
    mean_x, mean_y, xx, yy, xy = arrays
    for index, world in enumerate(worlds):
        for group, pairs in enumerate((world.siblings, world.unrelated)):
            x, y = oriented(world, pairs)
            count[index, group] = len(x)
            if len(x) == 0:
                continue
            mean_x[index, group] = x.mean(axis=0)
            mean_y[index, group] = y.mean(axis=0)
            dx, dy = x - x.mean(axis=0), y - y.mean(axis=0)
            xx[index, group] = np.sum(dx * dx, axis=0)
            yy[index, group] = np.sum(dy * dy, axis=0)
            xy[index, group] = np.sum(dx * dy, axis=0)
    return Centered(count, mean_x, mean_y, xx, yy, xy)


def correlations(blocks: Centered, weights: Matrix) -> Matrix:
    """Pool centered covariance blocks using within-world scatter plus mean offsets."""
    counts = np.einsum("rw,wg->rg", weights, blocks.count)
    weighted_n = weights[:, :, None] * blocks.count[None, :, :]
    with np.errstate(divide="ignore", invalid="ignore"):
        mx = np.einsum("rwg,wga->rga", weighted_n, blocks.mean_x) / counts[:, :, None]
        my = np.einsum("rwg,wga->rga", weighted_n, blocks.mean_y) / counts[:, :, None]
        dx = blocks.mean_x[None, :, :, :] - mx[:, None, :, :]
        dy = blocks.mean_y[None, :, :, :] - my[:, None, :, :]
        xx = np.sum(weights[:, :, None, None] * blocks.xx[None, :, :, :] + weighted_n[:, :, :, None] * dx * dx, axis=1)
        yy = np.sum(weights[:, :, None, None] * blocks.yy[None, :, :, :] + weighted_n[:, :, :, None] * dy * dy, axis=1)
        xy = np.sum(weights[:, :, None, None] * blocks.xy[None, :, :, :] + weighted_n[:, :, :, None] * dx * dy, axis=1)
        result = xy / np.sqrt(xx * yy)
    result[(counts < 3), :] = np.nan
    result[(xx <= 0) | (yy <= 0)] = np.nan
    return result


def select(values: Matrix, statistic: Statistic) -> Matrix:
    match statistic.kind:
        case Kind.DIFFERENCE:
            return values[:, 0, :].mean(axis=1) - values[:, 1, :].mean(axis=1)
        case Kind.MEAN:
            return values[:, statistic.group, :].mean(axis=1)
        case Kind.AXIS:
            return values[:, statistic.group, statistic.axis]
        case unreachable:
            assert_never(unreachable)


def explicit_point(worlds: Sequence[World]) -> Matrix:
    values = np.full((1, 2, 4), np.nan, dtype=np.float64)
    for group in range(2):
        vectors = [oriented(world, world.siblings if group == 0 else world.unrelated) for world in worlds]
        x = np.concatenate([pair[0] for pair in vectors], axis=0)
        y = np.concatenate([pair[1] for pair in vectors], axis=0)
        if len(x) < 3:
            continue
        dx, dy = x - x.mean(axis=0), y - y.mean(axis=0)
        with np.errstate(divide="ignore", invalid="ignore"):
            values[0, group] = np.sum(dx * dy, axis=0) / np.sqrt(np.sum(dx * dx, axis=0) * np.sum(dy * dy, axis=0))
    return values


def bootstrap_weights(worlds: int) -> Matrix:
    generator = np.random.default_rng(0)
    draws = [generator.integers(0, worlds, size=worlds) for _ in range(REPLICATES)]
    return np.asarray([np.bincount(draw, minlength=worlds) for draw in draws], dtype=np.float64)


def estimate(worlds: Sequence[World]) -> raw.Row:
    require(bool(worlds), "statistics", "at least one world required, including empty-pair worlds")
    blocks = centered(worlds)
    direct = explicit_point(worlds)
    pooled = correlations(blocks, np.ones((1, len(worlds)), dtype=np.float64))
    require(bool(np.allclose(direct, pooled, rtol=1e-11, atol=1e-12, equal_nan=True)),
            "statistics", "direct oriented Pearson disagrees with centered world pooling")
    result: dict[str, raw.Json] = {}
    for statistic in STATISTICS:
        point = float(select(direct, statistic)[0])
        if not math.isfinite(point):
            result[statistic.name] = {"value": None, "ci95": None, "bootstrap_valid": 0,
                                     "bootstrap_replicates": REPLICATES, "undefined": "insufficient oriented pairs or constant axis"}
            continue
        values = select(correlations(blocks, bootstrap_weights(len(worlds))), statistic)
        valid = values[np.isfinite(values)]
        interval: raw.Json = None
        if len(valid):
            lower, upper = np.percentile(valid, [2.5, 97.5], method="linear")
            interval = [float(lower), float(upper)]
        result[statistic.name] = {"value": point if len(valid) else None, "ci95": interval,
            "bootstrap_valid": len(valid), "bootstrap_replicates": REPLICATES,
            "undefined": None if len(valid) else "all bootstrap draws undefined"}
    return raw.Row(result, "independent statistics")


def pair_digest(pairs: Sequence[Pair]) -> str:
    encoded = json.dumps([pair.json() for pair in sorted(pairs, key=lambda pair: pair.ids)],
                         sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def compare_pairs(world: World, recorded: raw.Row, checks: raw.Checks) -> raw.Row:
    eligibility = raw.Row.parse(recorded.get("pair_eligibility"), recorded.where)
    details: dict[str, raw.Json] = {}
    for group, expected_pairs in (("siblings", world.siblings), ("unrelated", world.unrelated)):
        expected = {pair.ids: pair for pair in expected_pairs}
        actual: dict[PairKey, Pair] = {}
        for row in eligibility.rows(group):
            identifiers = row.ids("ids")
            require(len(identifiers) == 2, row.where, "pair requires two IDs")
            stages: tuple[str, str] | None = None
            if row.get("stages") is not None:
                stage_values = raw.array(row.get("stages"), row.where)
                require(len(stage_values) == 2, row.where, "observed context requires two stages")
                stage_row = raw.Row({"first": stage_values[0], "second": stage_values[1]}, row.where)
                stages = stage_row.text("first"), stage_row.text("second")
            pair = Pair((identifiers[0], identifiers[1]), row.optional("tick"), row.optional("settlement"), stages)
            context_present = (pair.tick is not None, pair.settlement is not None, pair.stages is not None)
            require(all(context_present) or not any(context_present), row.where, "snapshot context must be fully present or fully null")
            checks.require(pair.ids not in actual, row.where, "duplicate reported pair")
            checks.require(pair.ids[0] < pair.ids[1], row.where, "reported pair IDs must be unique and ascending")
            actual[pair.ids] = pair
        missing = sorted(expected.keys() - actual.keys())
        extra = sorted(actual.keys() - expected.keys())
        changed = sorted(key for key in expected.keys() & actual.keys() if expected[key] != actual[key])
        checks.require(not missing and not extra and not changed, f"seed={world.seed} {group}",
                       f"missing={len(missing)}, extra={len(extra)}, changed first-snapshot metadata={len(changed)}")
        details[group] = {"expected_pairs": len(expected), "reported_pairs": len(actual),
            "expected_pairs_without_shared_context": sum(pair.tick is None for pair in expected.values()),
            "reported_pairs_without_shared_context": sum(pair.tick is None for pair in actual.values()),
            "expected_pair_sha256": pair_digest(tuple(expected.values())),
            "reported_pair_sha256": pair_digest(tuple(actual.values())),
            "missing": [expected[key].json() for key in missing], "extra": [actual[key].json() for key in extra],
            "changed": [{"expected": expected[key].json(), "reported": actual[key].json()} for key in changed]}
        count_key = "sibling_unordered_pairs" if group == "siblings" else "unrelated_unordered_pairs"
        checks.equal(recorded.num(count_key), len(expected), f"seed={world.seed} {count_key}")
    checks.equal(recorded.num("excluded_close_kin_pairs"), len(world.excluded), f"seed={world.seed} excluded close kin")
    for key, expected_count in (("f3_unassigned_person_hours", world.unassigned_samples),
                                ("f3_unassigned_people", world.unassigned_people)):
        checks.equal(recorded.num(key), expected_count, f"seed={world.seed} {key}")
    return raw.Row({"seed": world.seed, "pairs": details, "excluded_close_kin_pairs": len(world.excluded),
        "unassigned_person_hours": world.unassigned_samples, "unassigned_people": world.unassigned_people}, "pair audit")


def number(value: raw.Json, where: str) -> float:
    match value:
        case bool() | str() | list() | dict() | None:
            raise raw.InputError(where, "expected finite number")
        case int() | float():
            result = float(value)
            require(math.isfinite(result), where, "nonfinite number")
            return result
        case unreachable:
            assert_never(unreachable)


def compare_number(actual: raw.Json, expected: raw.Json, where: str, checks: raw.Checks) -> None:
    if actual is None or expected is None:
        checks.equal(actual, expected, where)
    else:
        a, b = number(actual, where), number(expected, where)
        checks.require(math.isclose(a, b, rel_tol=1e-11, abs_tol=1e-12), where,
                       f"reported={a:.17g}; independent={b:.17g}; delta={a-b:.17g}")


def compare_statistics(independent: raw.Row, reported: raw.Row, worlds: Sequence[World], checks: raw.Checks) -> None:
    for statistic in STATISTICS:
        actual = raw.Row.parse(reported.get(statistic.name), statistic.name)
        expected = raw.Row.parse(independent.get(statistic.name), statistic.name)
        compare_number(actual.get("value"), expected.get("value"), f"{statistic.name}.value", checks)
        actual_ci, expected_ci = actual.get("ci95"), expected.get("ci95")
        if actual_ci is None or expected_ci is None:
            checks.equal(actual_ci, expected_ci, f"{statistic.name}.ci95")
        else:
            a, b = raw.array(actual_ci, actual.where), raw.array(expected_ci, expected.where)
            require(len(a) == 2 and len(b) == 2, statistic.name, "95% interval needs two endpoints")
            for index in range(2):
                compare_number(a[index], b[index], f"{statistic.name}.ci95[{index}]", checks)
        for key in ("bootstrap_valid", "bootstrap_replicates"):
            checks.equal(actual.num(key), expected.num(key), f"{statistic.name}.{key}")
    details = raw.Row.parse(raw.Row.parse(reported.get("F3"), "F3").get("details"), "F3.details")
    expected_details = {
        "sibling_unordered_pairs": sum(len(world.siblings) for world in worlds),
        "unrelated_unordered_pairs": sum(len(world.unrelated) for world in worlds),
        "sibling_worlds": sum(bool(world.siblings) for world in worlds),
        "unrelated_worlds": sum(bool(world.unrelated) for world in worlds),
        "zero_sibling_worlds": sum(not world.siblings for world in worlds),
        "zero_unrelated_worlds": sum(not world.unrelated for world in worlds),
        "excluded_close_kin_pairs": sum(len(world.excluded) for world in worlds), "orientations_per_pair": 2,
    }
    for key, expected_count in expected_details.items():
        checks.equal(details.num(key), expected_count, f"F3.details.{key}")


def terminal(root: Path) -> tuple[int, tuple[Path, ...]]:
    log_path = root / "RUNLOG.txt"
    rows = log_path.read_text(encoding="utf-8").splitlines()
    require(bool(rows), str(log_path), "empty log; original run has not completed")
    ending = rows[-1].rsplit(" workloads end exit=", 1)
    require(len(ending) == 2 and ending[1] in ("0", "1"), str(log_path), "original run lacks successful/metric-FAIL terminal marker")
    code = int(ending[1])
    exits = (root / "evaluate.exit", root / "game_v22.exit", root / "game_v21.exit")
    expected = (str(code), "0", "0")
    for path, value in zip(exits, expected, strict=True):
        require(path.read_text(encoding="utf-8").strip() == value, str(path), "terminal process exits disagree")
    return code, (log_path, *exits)


def contract(mode: str) -> tuple[tuple[int, ...], int]:
    match mode:
        case "tuning":
            return tuple(range(18100, 18116)), 60
        case "evaluation":
            return tuple(range(18000, 18032)), 120
        case invalid:
            raise raw.InputError("mode", f"unknown mode {invalid!r}")


def audit(mode: str, root: Path, report_path: Path) -> raw.Row:
    code, terminal_paths = terminal(root)
    seeds, days = contract(mode)
    paths = [*terminal_paths, report_path, root / "source_commit.txt", Path(__file__).resolve()]
    for filename, expected in PINS.items():
        path = Path(__file__).parent / filename
        require(digest(path) == expected, str(path), "previous family audit artifact changed")
        paths.append(path)
    hashes: dict[str, raw.Json] = {str(path): digest(path) for path in paths}
    source = (root / "source_commit.txt").read_text(encoding="utf-8").strip()
    reported = raw.read_json(report_path)
    require(reported.num("schema_version") == 17 and reported.text("stage") == "b" and reported.text("mode") == mode,
            str(report_path), "wrong corrected-report schema/stage/mode")
    candidate = raw.Row.parse(reported.get("candidate"), "candidate")
    require(candidate.text("scene") == "game_v22" and candidate.num("days") == days
            and candidate.num("worlds") == len(seeds) and candidate.ids("seeds") == seeds,
            candidate.where, "corrected report cohort does not match registered seeds/duration")
    family = raw.Row.parse(reported.get("family"), "family")
    recorded_worlds = {row.num("seed"): row for row in family.rows("worlds")}
    require(len(recorded_worlds) == len(family.rows("worlds")) and set(recorded_worlds) == set(seeds),
            family.where, "duplicate or incomplete corrected-report world inventory")
    reported_hashes = raw.Row.parse(family.get("inputs_sha256"), "family.inputs_sha256")
    checks = raw.Checks()
    worlds: list[World] = []
    pair_audits: list[raw.Json] = []
    for index, seed in enumerate(seeds, 1):
        stem = root / "game_v22" / f"game_v22_s{seed}"
        meta_path = Path(f"{stem}_family_meta.json")
        people_path = Path(f"{stem}_family_people.jsonl")
        hours_path = Path(f"{stem}_family_hours.jsonl")
        for path in (meta_path, people_path, hours_path):
            value = digest(path)
            hashes[str(path)] = value
            checks.equal(reported_hashes.get(str(path)), value, f"corrected report input hash {path.name}")
        meta = raw.read_json(meta_path)
        require(meta.num("seed") == seed and meta.text("scene") == "game_v22" and meta.num("end_tick") == days * 1440
                and meta.text("git_sha") == source and meta.num("observation_misses") == 0,
                str(meta_path), "raw world provenance, duration, or observation validity mismatch")
        snapshots = raw.read_lines(hours_path)
        require([row.num("tick") for row in snapshots] == list(range(0, days * 1440, 60)), str(hours_path), "hourly snapshot inventory incomplete")
        world = reconstruct(seed, raw.read_lines(people_path), snapshots)
        worlds.append(world)
        pair_audits.append(dict(compare_pairs(world, recorded_worlds[seed], checks).data))
        print(f"literal F3 {index}/{len(seeds)} seed={seed} siblings={len(world.siblings)} unrelated={len(world.unrelated)}", file=sys.stderr)
    independent = estimate(worlds)
    metrics = raw.Row.parse(reported.get("metrics"), "metrics")
    compare_statistics(independent, metrics, worlds, checks)
    for path, expected in tuple(hashes.items()):
        checks.equal(digest(Path(path)), expected, f"stable input {path}")
    return raw.Row({"schema_version": 1, "audit": "literal F3 raw pair/numeric comparison; not Part8 review",
        "audit_status": "CONSISTENT" if not checks.issues else "DISCREPANCIES", "acceptance_verdicts_assigned": False,
        "mode": mode, "worlds": len(worlds), "seeds": list(seeds), "days": days, "checks": checks.count,
        "original_run_exit": code, "producer_commit": source, "corrected_report": str(report_path),
        "imported_F3_verdict": raw.Row.parse(metrics.get("F3"), "F3").text("verdict"),
        "statistics": dict(independent.data), "pair_audits": pair_audits, "discrepancies": list(checks.issues),
        "numpy_version": np.__version__, "bootstrap": "2000 whole-world draws; fresh default_rng(0) per statistic; linear percentile 2.5/97.5; undefined draws excluded",
        "pair_definition": "Full siblings include every recorded pair with both known matching ordered mother/father, regardless of residence or coexistence. Context is the first shared non-null settlement hourly snapshot, or tick/settlement/stages all null when absent. Unrelated requires a shared non-null settlement snapshot, uses its first context, has no stage filter, and excludes direct parent-child and any shared known parent across either slot.",
        "numeric_method": "Explicit (a,b) and (b,a) observations; centered Pearson per axis; mean four correlations; siblings minus unrelated. Bootstrap pools independently centered world covariance blocks, retaining empty worlds.",
        "comparison_tolerance": {"relative": 1e-11, "absolute": 1e-12}, "input_sha256": hashes}, "audit")


def human(report: raw.Row) -> str:
    lines = [f"{report.text('audit_status')} — literal F3 raw audit; no acceptance verdict assigned",
        f"mode={report.text('mode')} worlds={report.num('worlds')} days={report.num('days')} checks={report.num('checks')}",
        f"producer={report.text('producer_commit')}; corrected report={report.text('corrected_report')}",
        f"Imported F3 verdict={report.text('imported_F3_verdict')}", report.text("pair_definition"), report.text("numeric_method"), report.text("bootstrap")]
    statistics = raw.Row.parse(report.get("statistics"), "statistics")
    for name, value in statistics.data.items():
        row = raw.Row.parse(value, name)
        lines.append(f"{name}: value={row.get('value')} ci95={row.get('ci95')} valid={row.num('bootstrap_valid')}/{row.num('bootstrap_replicates')}")
    for world in report.rows("pair_audits"):
        pairs = raw.Row.parse(world.get("pairs"), "pairs")
        parts = []
        for group in ("siblings", "unrelated"):
            row = raw.Row.parse(pairs.get(group), group)
            parts.append(f"{group}={row.num('expected_pairs')}/{row.num('reported_pairs')} without_shared_context={row.num('expected_pairs_without_shared_context')}/{row.num('reported_pairs_without_shared_context')} missing={len(row.rows('missing'))} extra={len(row.rows('extra'))} changed={len(row.rows('changed'))} raw_sha256={row.text('expected_pair_sha256')}")
        lines.append(f"seed={world.num('seed')} " + "; ".join(parts))
    for issue in raw.array(report.get("discrepancies"), "discrepancies"):
        lines.append(f"DISCREPANCY: {issue}")
    lines.append("Full missing/extra/changed pair records are preserved in the JSON report.")
    lines.append("Input SHA256:")
    for path, value in sorted(raw.Row.parse(report.get("input_sha256"), "hashes").data.items()):
        lines.append(f"  {value}  {path}")
    return "\n".join(lines)


def main(arguments: Sequence[str]) -> int:
    try:
        if len(arguments) == 2 and arguments[0] == "--render-json":
            report = raw.read_json(Path(arguments[1]))
            require(report.num("schema_version") == 1 and not report.flag("acceptance_verdicts_assigned"), report.where, "not a raw audit")
            print(human(report))
            return int(report.text("audit_status") != "CONSISTENT")
        values = [value for value in arguments if value != "--json"]
        if len(values) != 3:
            print("Usage: python3 f3_literal_audit.py tuning|evaluation RUN_ROOT CORRECTED_EVAL_JSON [--json]", file=sys.stderr)
            return 2
        report = audit(values[0], Path(values[1]).absolute(), Path(values[2]).absolute())
        print(json.dumps(report.data, indent=2, sort_keys=True, allow_nan=False) if "--json" in arguments else human(report))
        return int(report.text("audit_status") != "CONSISTENT")
    except (raw.InputError, OSError, json.JSONDecodeError) as error:
        print(f"INPUT ERROR; no literal F3 audit completed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
