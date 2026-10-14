"""Validated stdlib models for schema-13 probe measurements."""
from __future__ import annotations

import json
from dataclasses import dataclass
from typing import TypeAlias

JsonValue: TypeAlias = None | bool | int | float | str | list['JsonValue'] | dict[str, 'JsonValue']


@dataclass(frozen=True, slots=True)
class SchemaError(ValueError):
    field: str
    expected: str


@dataclass(frozen=True, slots=True)
class Fields:
    values: dict[str, JsonValue]

    @classmethod
    def parse(cls, text: str) -> Fields:
        value: JsonValue = json.loads(text)
        match value:
            case dict():
                return cls(value)
            case _:
                raise SchemaError('root', 'object')

    def integer(self, key: str) -> int:
        value = self.values[key]
        if type(value) is not int:
            raise SchemaError(key, 'integer')
        return value

    def optional_integer(self, key: str) -> int | None:
        return None if self.values[key] is None else self.integer(key)

    def string(self, key: str) -> str:
        value = self.values[key]
        if not isinstance(value, str):
            raise SchemaError(key, 'string')
        return value

    def number(self, key: str) -> float:
        value = self.values[key]
        if not isinstance(value, (int, float)) or isinstance(value, bool):
            raise SchemaError(key, 'number')
        return float(value)

    def nested(self, key: str) -> Fields:
        return Fields.parse(json.dumps(self.values[key]))

    def records(self, key: str) -> tuple[Fields, ...]:
        value = self.values[key]
        if not isinstance(value, list):
            raise SchemaError(key, 'array')
        return tuple(Fields.parse(json.dumps(item)) for item in value)

    def integers(self, key: str) -> tuple[int, ...]:
        value = self.values[key]
        if not isinstance(value, list):
            raise SchemaError(key, 'integer array')
        result: list[int] = []
        for item in value:
            if type(item) is not int:
                raise SchemaError(key, 'integer array')
            result.append(item)
        return tuple(result)

    def matrix(self, key: str) -> tuple[tuple[int, ...], ...]:
        value = self.values[key]
        if not isinstance(value, list):
            raise SchemaError(key, 'integer matrix')
        return tuple(Fields({'row': row}).integers('row') for row in value)


@dataclass(frozen=True, slots=True)
class Sample:
    tick: int
    seed: int
    scene: str
    git_sha: str
    minute_of_day: int
    days_per_season: int
    cold_members: int
    cold_member_hours_total: int
    fire_lit: int
    winter_severity: int
    wood_gatherers_now: int
    wood_in_hand: int
    hauling_now: int
    stockpile_food: int
    settled_agents: int
    settlements_alive: int

    @classmethod
    def model_validate_json(cls, text: str) -> Sample:
        f = Fields.parse(text)
        return cls(f.integer('tick'), f.integer('seed'), f.string('scene'), f.string('git_sha'),
                   f.integer('minute_of_day'), f.integer('days_per_season'), f.integer('cold_members'),
                   f.integer('cold_member_hours_total'), f.integer('fire_lit'), f.integer('winter_severity'),
                   f.integer('wood_gatherers_now'), f.integer('wood_in_hand'), f.integer('hauling_now'),
                   f.integer('stockpile_food'), f.integer('settled_agents'), f.integer('settlements_alive'))


@dataclass(frozen=True, slots=True)
class Death:
    tick: int
    agent_id: int
    cause: str
    stage: str
    settlement_at_death: int | None

    @classmethod
    def model_validate_json(cls, text: str) -> Death:
        f = Fields.parse(text)
        return cls(f.integer('tick'), f.integer('agent_id'), f.string('cause'), f.string('stage'),
                   f.optional_integer('settlement_at_death'))


@dataclass(frozen=True, slots=True)
class Fission:
    fission_id: int
    parent: int
    site: tuple[int, ...]
    depart_tick: int
    group_size: int
    daughter: int | None
    founded_tick: int | None
    cancelled_tick: int | None

    @classmethod
    def parse(cls, f: Fields) -> Fission:
        site = f.integers('site')
        if len(site) != 2:
            raise SchemaError('site', 'coordinate pair')
        return cls(f.integer('fission_id'), f.integer('parent'), site, f.integer('depart_tick'),
                   f.integer('group_size'), f.optional_integer('daughter'),
                   f.optional_integer('founded_tick'), f.optional_integer('cancelled_tick'))


@dataclass(frozen=True, slots=True)
class Pass11:
    fission_log: tuple[Fission, ...]


@dataclass(frozen=True, slots=True)
class Pass12:
    fission_checks: tuple[tuple[int, ...], ...]


@dataclass(frozen=True, slots=True)
class Pass13:
    pressure_checks: tuple[tuple[int, ...], ...]


@dataclass(frozen=True, slots=True)
class Run:
    seed: int
    wall_seconds: float
    settlements_alive_daily: tuple[int, ...]
    pass11: Pass11
    pass12: Pass12
    pass13: Pass13

    @classmethod
    def parse(cls, f: Fields) -> Run:
        checks = f.nested('pass12').matrix('fission_checks')
        pressure = f.nested('pass13').matrix('pressure_checks')
        if any(len(row) != 12 for row in checks) or any(len(row) != 8 for row in pressure):
            raise SchemaError('fission_checks', 'schema 13 check widths')
        return cls(f.integer('seed'), f.number('wall_seconds'), f.integers('settlements_alive_daily'),
                   Pass11(tuple(Fission.parse(row) for row in f.nested('pass11').records('fission_log'))),
                   Pass12(checks), Pass13(pressure))


@dataclass(frozen=True, slots=True)
class Summary:
    schema_version: int
    scene: str
    runs: tuple[Run, ...]

    @classmethod
    def model_validate_json(cls, text: str) -> Summary:
        f = Fields.parse(text)
        if f.integer('schema_version') < 13:
            raise SchemaError('schema_version', '13 or greater')
        return cls(f.integer('schema_version'), f.string('scene'), tuple(Run.parse(row) for row in f.records('runs')))


@dataclass(frozen=True, slots=True)
class Clock:
    ticks_per_day: int
    offset: int

    def day(self, tick: int) -> int:
        return (tick + self.offset) // self.ticks_per_day + 1

    def label(self, tick: int) -> str:
        minute = (tick + self.offset) % self.ticks_per_day
        return f'd{self.day(tick)} {minute // 60:02d}:{minute % 60:02d}'


@dataclass(frozen=True, slots=True)
class CastDay:
    day: int
    tick: int
    members: int
    food: int
    scarcity_line: int
    pressure: int
    threshold: int
    hazard: int


@dataclass(frozen=True, slots=True)
class SeedSelection:
    seed: int
    qualifies: bool
    departure_days: tuple[int, ...]
    first_three_alive_tick: int | None
    first_three_alive_sample_tick: int | None
    alive_at_display_day_120: int
    alive_at_probe_day_120: int
    cast_days: tuple[CastDay, ...]
