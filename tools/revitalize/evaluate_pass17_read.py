from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from math import isfinite
from pathlib import Path
from typing import Never, TypeAlias, assert_never

from evaluate_pass17_data import AXES, KINDS, Cohort, CohortSpec, Hour, InputError, Person, World

JsonValue: TypeAlias = str | int | float | bool | None | list["JsonValue"] | dict[str, "JsonValue"]


def mapping(value: JsonValue, location: str) -> Mapping[str, JsonValue]:
    match value:
        case dict():
            return value
        case str() | int() | float() | bool() | None | list():
            raise InputError(location, "expected a JSON object")
        case unreachable:
            assert_never(unreachable)


def array(value: JsonValue, location: str) -> Sequence[JsonValue]:
    match value:
        case list():
            return value
        case str() | int() | float() | bool() | None | dict():
            raise InputError(location, "expected an array")
        case unreachable:
            assert_never(unreachable)


def number(value: JsonValue, location: str) -> float:
    match value:
        case bool():
            raise InputError(location, "boolean is not a number")
        case int() | float():
            result = float(value)
            if isfinite(result):
                return result
            raise InputError(location, "number must be finite")
        case str() | None | dict() | list():
            raise InputError(location, "expected a number")
        case unreachable:
            assert_never(unreachable)


def integer(value: JsonValue, location: str) -> int:
    match value:
        case bool():
            raise InputError(location, "boolean is not an integer")
        case int():
            if value >= 0:
                return value
            raise InputError(location, "integer must be nonnegative")
        case float() | str() | None | dict() | list():
            raise InputError(location, "expected a nonnegative integer")
        case unreachable:
            assert_never(unreachable)


def required(row: Mapping[str, JsonValue], key: str) -> JsonValue:
    if key not in row:
        raise InputError(key, "required field is absent")
    return row[key]


def reject_constant(value: str) -> Never:
    raise InputError("JSON", f"nonfinite literal {value}")


def read_json(path: Path) -> JsonValue:
    try:
        with path.open() as stream:
            return json.load(stream, parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError) as error:
        raise InputError(str(path), str(error)) from error


def read_jsonl(path: Path) -> list[Mapping[str, JsonValue]]:
    try:
        with path.open() as stream:
            return [mapping(json.loads(line, parse_constant=reject_constant), f"{path}:{index}") for index, line in enumerate(stream, 1) if line.strip()]
    except (OSError, json.JSONDecodeError) as error:
        raise InputError(str(path), str(error)) from error


def counts(value: JsonValue, expected: int, location: str) -> tuple[int, ...]:
    result = tuple(integer(item, location) for item in array(value, location))
    if len(result) != expected:
        raise InputError(location, f"expected exactly {expected} counts")
    return result


def parse_person(row: Mapping[str, JsonValue]) -> Person:
    temperament = mapping(required(row, "temperament"), "temperament")
    axes = tuple(integer(required(temperament, axis), axis) for axis in AXES)
    if max(axes) > 1000:
        raise InputError("temperament", "axis exceeds 1000")
    routine = mapping(required(row, "routine"), "routine")
    adult_ticks = integer(required(row, "adult_ticks"), "adult_ticks")
    if abs(number(required(row, "adult_days"), "adult_days") - adult_ticks / 1440) > 1e-8:
        raise InputError("adult_days", "does not equal adult_ticks / 1440")
    minutes = counts(required(routine, "activity_minutes"), 7, "activity_minutes")
    started = counts(required(routine, "activities_started"), 7, "activities_started")
    completed = integer(required(routine, "completed_activities"), "completed_activities")
    duration = integer(required(routine, "completed_minutes"), "completed_minutes")
    if sum(minutes) > adult_ticks or duration > sum(minutes):
        raise InputError("routine", "activity exposure/counts are inconsistent")
    if completed == 0 and duration != 0:
        raise InputError("completed_minutes", "duration exists without completed activities")
    return Person(integer(required(row, "id"), "id"), axes, adult_ticks,
        integer(required(row, "social_completed"), "social_completed"), minutes, started, completed, duration,
        integer(required(routine, "no_place_redraws"), "no_place_redraws"),
        integer(required(routine, "no_place_failures"), "no_place_failures"))


def parse_hour(row: Mapping[str, JsonValue]) -> Hour:
    states = mapping(required(row, "states"), "states")
    minute = integer(required(row, "minute"), "minute")
    if minute >= 1440 or minute % 60:
        raise InputError("minute", "expected a clock hour in 0..1440")
    day = integer(required(row, "day"), "day")
    if day == 0:
        raise InputError("day", "display days are one-based")
    integer(required(row, "tick"), "tick")
    return Hour(day, minute,
        tuple(counts(required(states, stage), 5, stage) for stage in ("adult", "child", "elder")),
        integer(required(row, "stockpile_food"), "stockpile_food"),
        integer(required(row, "stockpile_wood"), "stockpile_wood"))


def read_world(root: Path, run: Mapping[str, JsonValue], spec: CohortSpec) -> World:
    seed = integer(required(run, "seed"), "seed")
    stem = root / f"{spec.scene}_s{seed}"
    people = tuple(parse_person(row) for row in read_jsonl(Path(f"{stem}_agents.jsonl")))
    if len({person.agent_id for person in people}) != len(people):
        raise InputError(str(stem), "duplicate person id")
    everyone = read_jsonl(Path(f"{stem}_people.jsonl"))
    all_ids = [integer(required(row, "id"), "id") for row in everyone]
    if len(set(all_ids)) != len(all_ids) or not {person.agent_id for person in people}.issubset(all_ids):
        raise InputError(str(stem), "all-person inventory is duplicated or excludes adults")
    if any(type(required(row, "ever_adult")) is not bool for row in everyone):
        raise InputError(str(stem), "ever_adult must be a boolean")
    expected_adults = {integer(required(row, "id"), "id") for row in everyone if row["ever_adult"] is True}
    if expected_adults != {person.agent_id for person in people} or any(not 0 < person.adult_ticks <= spec.days * 1440 for person in people):
        raise InputError(str(stem), "ever-adult inventory or adult exposure is incomplete")
    routines = [mapping(required(row, "routine"), "routine") for row in everyone]
    minute_rows = [counts(required(row, "activity_minutes"), 7, "activity_minutes") for row in routines]
    start_rows = [counts(required(row, "activities_started"), 7, "activities_started") for row in routines]
    all_minutes = tuple(sum(row[kind] for row in minute_rows) for kind in range(7))
    all_starts = tuple(sum(row[kind] for row in start_rows) for kind in range(7))
    hours = tuple(parse_hour(row) for row in read_jsonl(Path(f"{stem}_hours.jsonl")))
    slots = sorted((hour.day - 1) * 1440 + hour.minute for hour in hours)
    if len(slots) != spec.days * 24 or any(b - a != 60 for a, b in zip(slots, slots[1:])):
        raise InputError(str(stem), "hour inventory is missing, duplicated or discontinuous")
    if sum(hour.minute == 720 for hour in hours) != spec.days:
        raise InputError(str(stem), "noon inventory differs from requested days")
    meta = mapping(read_json(Path(f"{stem}_trace_meta.json")), "trace_meta")
    if integer(required(meta, "people"), "people") != len(everyone) or integer(required(meta, "traced_adults"), "traced_adults") != len(people):
        raise InputError(str(stem), "trace metadata person counts disagree with records")
    if integer(required(meta, "end_tick"), "end_tick") != spec.days * 1440:
        raise InputError(str(stem), "trace ended before requested duration")
    p17 = mapping(required(meta, "pass17"), "pass17")
    if integer(required(p17, "schema_version"), "schema_version") != 17:
        raise InputError(str(stem), "expected trace schema 17")
    if list(array(required(p17, "activity_kinds"), "activity_kinds")) != list(KINDS):
        raise InputError(str(stem), "activity kind order differs from the contract")
    if integer(required(p17, "observation_misses"), "observation_misses") != 0:
        raise InputError(str(stem), "routine observation misses invalidate metrics")
    alive = counts(required(run, "alive_daily"), spec.days, "alive_daily")
    census = mapping(required(run, "death_census"), "death_census")
    causes = mapping(required(census, "counts_by_cause"), "counts_by_cause")
    total = integer(required(census, "total"), "death_census.total")
    if sum(integer(value, cause) for cause, value in causes.items()) != total:
        raise InputError(str(stem), "death census total disagrees with causes")
    old_age = integer(causes.get("old_age", 0), "old_age")
    return World(seed, alive[-1], total - old_age, old_age,
        integer(required(p17, "shelters_completed"), "shelters_completed"), people, hours,
        integer(required(p17, "no_place_redraws"), "no_place_redraws"),
        integer(required(p17, "no_place_failures"), "no_place_failures"), all_minutes, all_starts)


def load_cohort(root: Path, spec: CohortSpec) -> Cohort:
    summary = mapping(read_json(root / "summary.json"), "summary")
    if required(summary, "scene") != spec.scene:
        raise InputError(str(root), f"expected scene {spec.scene}")
    days = mapping(required(summary, "days"), "summary.days")
    if set(days) != {str(day) for day in range(1, spec.days + 1)}:
        raise InputError(str(root), "summary day checkpoints are incomplete")
    if required(summary, "determinism_checked") is not True:
        raise InputError(str(root), "determinism was not checked")
    runs = [mapping(run, "runs") for run in array(required(summary, "runs"), "runs")]
    seeds = [integer(required(run, "seed"), "seed") for run in runs]
    if sorted(seeds) != sorted(spec.seeds):
        raise InputError(str(root), "missing, duplicate or unexpected seeds")
    declared = mapping(required(summary, "seeds"), "seeds")
    if (integer(required(declared, "start"), "start"), integer(required(declared, "end_exclusive"), "end_exclusive")) != (min(spec.seeds), max(spec.seeds) + 1):
        raise InputError(str(root), "declared seed range disagrees with cohort")
    worlds = tuple(sorted((read_world(root, run, spec) for run in runs), key=lambda world: world.seed))
    return Cohort(spec.scene, spec.days, worlds, root)
