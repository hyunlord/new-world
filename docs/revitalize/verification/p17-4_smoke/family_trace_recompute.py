#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Usage: uv run --no-project family_trace_recompute.py OUTPUT_DIR [--json]
# Single external artifact is the assigned write boundary. # noqa: SIZE_OK
"""Recompute Pass 17 family evidence directly from probe JSONL, without evaluator imports.

Default output includes daily counts, each night and departure, and one concrete
family. --json preserves all counts, input hashes, checks, and selected raw rows.
No simulation, evaluator, bootstrap, threshold verdict, or filesystem write runs.
Exit 0: all implemented consistency checks pass; 1: discrepancies; 2: input error.
This is preparation evidence, not the independent Part 8 review.
"""

from __future__ import annotations

import hashlib
import json
import sys
from collections import Counter, defaultdict
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Final, assert_never

type Json = None | bool | int | float | str | list[Json] | dict[str, Json]
type Pair = tuple[int, int]

DAY: Final = 1440
LIMITATIONS: Final = (
    "No death/unpair events: partnership continuity and parent liveness between samples "
    "are not independently observable. Pair events, sampled reciprocity, pairing ticks, "
    "and final living reciprocity are checked.",
    "No per-shelter capacity inventory: housing_capacity is an input aggregate. "
    "Bedded people, bedless people, unused slots and couple-night denominators are recomputed.",
    "Night bed rows show assigned beds, not actual sleep state or nearest-tile sleeping. "
    "No claim about realized sleep follows from this checker.",
    "Departure household/couple arrays are raw snapshots; their counts and membership "
    "splits are recomputed. Unsampled departures cannot independently prove the "
    "full living settlement census or individual ranking decisions.",
    "This checker does not implement F3 resemblance, confidence intervals, health "
    "gates, cohort seed policy, or final acceptance verdicts.",
)


@dataclass(frozen=True, slots=True)
class InputError(Exception):
    """Malformed or missing evidence, distinguished from semantic discrepancies."""

    where: str
    reason: str

    def __str__(self) -> str:
        return f"{self.where}: {self.reason}"


def integer(value: Json, where: str) -> int:
    """Read unsigned integers without accepting JSON booleans as integers."""
    match value:
        case bool() | str() | float() | list() | dict() | None:
            raise InputError(where, "expected unsigned integer")
        case int() as number:
            if number < 0:
                raise InputError(where, "expected unsigned integer")
            return number
        case unreachable:
            assert_never(unreachable)


def array(value: Json, where: str) -> list[Json]:
    match value:
        case list() as values:
            return values
        case bool() | int() | str() | float() | dict() | None:
            raise InputError(where, "expected array")
        case unreachable:
            assert_never(unreachable)


@dataclass(frozen=True, slots=True)
class Row:
    """Typed, location-preserving boundary for a raw JSON object."""

    data: Mapping[str, Json]
    where: str

    @classmethod
    def parse(cls, value: Json, where: str) -> Row:
        match value:
            case dict() as values:
                return cls(values, where)
            case bool() | int() | str() | float() | list() | None:
                raise InputError(where, "expected JSON object")
            case unreachable:
                assert_never(unreachable)

    def get(self, key: str) -> Json:
        if key not in self.data:
            raise InputError(self.where, f"missing {key}")
        return self.data[key]

    def num(self, key: str) -> int:
        return integer(self.get(key), f"{self.where}.{key}")

    def optional(self, key: str) -> int | None:
        value = self.get(key)
        return None if value is None else integer(value, f"{self.where}.{key}")

    def flag(self, key: str) -> bool:
        match self.get(key):
            case bool() as value:
                return value
            case invalid:
                raise InputError(self.where, f"{key} must be a boolean, got {invalid!r}")

    def text(self, key: str) -> str:
        match self.get(key):
            case str() as value:
                return value
            case invalid:
                raise InputError(self.where, f"{key} must be a string, got {invalid!r}")

    def rows(self, key: str) -> tuple[Row, ...]:
        return tuple(Row.parse(value, f"{self.where}.{key}[{index}]")
                     for index, value in enumerate(array(self.get(key), self.where)))

    def ids(self, key: str) -> tuple[int, ...]:
        return tuple(integer(value, f"{self.where}.{key}")
                     for value in array(self.get(key), self.where))

    def groups(self, key: str) -> tuple[tuple[int, ...], ...]:
        return tuple(tuple(integer(value, f"{self.where}.{key}")
                           for value in array(group, self.where))
                     for group in array(self.get(key), self.where))

    def bed(self) -> Pair | None:
        value = self.get("bed")
        if value is None:
            return None
        values = tuple(integer(item, self.where) for item in array(value, self.where))
        if len(values) != 2:
            raise InputError(self.where, "bed requires two tile coordinates")
        return values[0], values[1]


@dataclass(slots=True)
class Checks:
    """Mutable accumulator intentionally keeps every discrepancy, not just the first."""

    count: int = 0
    issues: list[str] = field(default_factory=list)

    def require(self, condition: bool, where: str, detail: str) -> None:
        self.count += 1
        if not condition:
            self.issues.append(f"{where}: {detail}")

    def equal(self, actual: Json, expected: Json, where: str) -> None:
        self.require(actual == expected, where, f"actual={actual!r}; expected={expected!r}")


@dataclass(frozen=True, slots=True)
class Evidence:
    stem: str
    meta: Row
    people: tuple[Row, ...]
    hours: tuple[Row, ...]
    events: tuple[Row, ...]
    fissions: tuple[Row, ...]
    ordinary_people: tuple[Row, ...]
    ordinary_fissions: tuple[Row, ...]
    trace_meta: Row
    summary: Row
    hashes: Mapping[str, str]


def read_json(path: Path) -> Row:
    with path.open(encoding="utf-8") as stream:
        value: Json = json.load(stream)
    return Row.parse(value, str(path))


def read_lines(path: Path) -> tuple[Row, ...]:
    with path.open(encoding="utf-8") as stream:
        return tuple(Row.parse(json.loads(line), f"{path}:{number}")
                     for number, line in enumerate(stream, 1) if line.strip())


def load(meta_path: Path) -> Evidence:
    """Read one complete output family and hash the exact bytes consumed."""
    stem = str(meta_path).removesuffix("_family_meta.json")
    paths = [meta_path, *(Path(f"{stem}_{name}.jsonl") for name in (
        "family_people", "family_hours", "family_events", "family_fissions", "people", "fissions")),
        Path(f"{stem}_trace_meta.json"), meta_path.parent / "summary.json"]
    before = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    result = Evidence(stem, read_json(paths[0]), read_lines(paths[1]), read_lines(paths[2]),
                      read_lines(paths[3]), read_lines(paths[4]), read_lines(paths[5]),
                      read_lines(paths[6]), read_json(paths[7]), read_json(paths[8]), before)
    if any(hashlib.sha256(path.read_bytes()).hexdigest() != before[str(path)] for path in paths):
        raise InputError(stem, "input changed while being read; wait for the probe to finish")
    return result


def indexed(rows: Sequence[Row], key: str, checks: Checks) -> Mapping[int, Row]:
    result: dict[int, Row] = {}
    for row in rows:
        identifier = row.num(key)
        checks.require(identifier not in result, row.where, f"duplicate {key}={identifier}")
        result[identifier] = row
    return result


def clock(row: Row, end: int, offset: int, checks: Checks) -> None:
    tick = row.num("tick")
    checks.require(tick < end, row.where, f"tick {tick} outside completed duration {end}")
    checks.equal(row.num("simulation_day"), tick // DAY + 1, f"{row.where}.simulation_day")
    checks.equal(row.num("display_day"), (tick + offset) // DAY + 1, f"{row.where}.display_day")
    checks.equal(row.num("minute"), (tick + offset) % DAY, f"{row.where}.minute")


def axes(row: Row, checks: Checks) -> None:
    temperament = Row.parse(row.get("temperament"), f"{row.where}.temperament")
    for key in ("ns", "ha", "rd", "p"):
        checks.require(temperament.num(key) <= 1000, temperament.where, f"{key} exceeds 1000")


def identity(evidence: Evidence, checks: Checks) -> Mapping[int, Row]:
    people = indexed(evidence.people, "id", checks)
    ordinary = indexed(evidence.ordinary_people, "id", checks)
    checks.equal(sorted(people), sorted(ordinary), "family/ordinary lifetime IDs")
    for identifier, person in people.items():
        person.flag("female")
        person.flag("alive_end")
        person.num("lineage_key")
        person.num("household_key")
        person.optional("born_tick")
        axes(person, checks)
        for field_name in ("mother", "father", "partner"):
            related = person.optional(field_name)
            checks.require(related is None or related in people, person.where, f"unknown {field_name}={related}")
            checks.require(related != identifier, person.where, f"self {field_name}")
        source = ordinary.get(identifier)
        if source is not None:
            for key in ("lineage_key", "born_tick", "temperament", "alive_end"):
                checks.equal(person.get(key), source.get(key), f"{person.where} ordinary {key}")
            if person.optional("born_tick") is not None:
                checks.equal(source.get("parents"), [person.get("mother"), person.get("father")],
                             f"{person.where} ordered AgentBorn parents")
    return people


def births_and_pairs(evidence: Evidence, people: Mapping[int, Row], end: int,
                     offset: int, checks: Checks) -> tuple[Mapping[Pair, tuple[int, ...]], list[Json]]:
    """Reconstruct pair history and day bins from events before inspecting summary bins."""
    history: dict[Pair, list[int]] = defaultdict(list)
    births: dict[int, Row] = {}
    daily = [[0, 0] for _ in range((end + DAY - 1) // DAY)]
    previous_tick = -1
    for event in evidence.events:
        clock(event, end, offset, checks)
        tick = event.num("tick")
        checks.require(tick >= previous_tick, event.where, "event time moved backwards")
        previous_tick = tick
        match event.text("event"):
            case "paired":
                mother, father = event.num("mother"), event.num("father")
                pair = tuple(sorted((mother, father)))
                checks.require(mother != father, event.where, "self pairing")
                checks.require(tick not in history[pair], event.where, "duplicate pair event")
                history[pair].append(tick)
                checks.equal(event.num("minute"), 1200, f"{event.where} pairing clock")
                for identifier, female in ((mother, True), (father, False)):
                    person = people.get(identifier)
                    checks.require(person is not None, event.where, f"unknown paired person {identifier}")
                    if person is not None:
                        checks.equal(person.flag("female"), female, f"{event.where} sex of {identifier}")
                        born = person.optional("born_tick")
                        checks.require(born is None or born <= tick, event.where, "pair before birth")
                if mother in people and father in people:
                    first = {people[mother].optional(k) for k in ("mother", "father")} - {None}
                    second = {people[father].optional(k) for k in ("mother", "father")} - {None}
                    checks.require(not (first & second) and father not in first and mother not in second,
                                   event.where, "paired close kin recorded by Kin")
            case "born":
                child = event.num("child")
                checks.require(child not in births, event.where, f"duplicate birth for {child}")
                births[child] = event
                fallback = event.flag("fallback")
                if tick < end:
                    daily[tick // DAY][int(fallback)] += 1
                person = people.get(child)
                checks.require(person is not None, event.where, f"child {child} missing from lifetime")
                if person is not None:
                    checks.equal(person.optional("born_tick"), tick, f"{event.where} lifetime born_tick")
                    for key in ("female", "mother", "father", "lineage_key", "temperament"):
                        checks.equal(event.get(key), person.get(key), f"{event.where} lifetime {key}")
                for key in ("mother", "father"):
                    parent = event.optional(key)
                    checks.require(parent in people, event.where, f"missing lifetime {key}={parent}")
                    if parent in people:
                        born = people[parent].optional("born_tick")
                        checks.require(born is None or born < tick, event.where, f"{key} not born before child")
                if not fallback:
                    mother, father = event.num("mother"), event.num("father")
                    for identifier, female in ((mother, True), (father, False)):
                        if identifier in people:
                            checks.equal(people[identifier].flag("female"), female, f"{event.where} parent sex")
                    pair = tuple(sorted((mother, father)))
                    checks.require(any(time < tick for time in history[pair]), event.where,
                                   "nonfallback parents lack earlier pairing event")
            case "departed":
                checks.equal(event.num("depart_tick"), tick, f"{event.where} depart_tick")
            case unknown:
                raise InputError(event.where, f"unknown event {unknown!r}")
    expected_births = {identifier: person.optional("born_tick") for identifier, person in people.items()
                       if person.optional("born_tick") is not None}
    checks.require({identifier: event.num("tick") for identifier, event in births.items()} == expected_births,
                   evidence.stem, "birth events differ from nonfounder lifetime inventory")
    days: list[Json] = [{"day": index + 1, "births": couple + fallback,
                        "couple_births": couple, "fallback_births": fallback}
                       for index, (couple, fallback) in enumerate(daily)]
    checks.equal(evidence.meta.get("births_by_day"), days, f"{evidence.stem} births_by_day")
    for key in ("births_total", "source_births_total"):
        checks.equal(evidence.meta.num(key), len(births), f"{evidence.stem} {key}")
    return {pair: tuple(times) for pair, times in history.items()}, days


def partners(rows: Mapping[int, Row], people: Mapping[int, Row], history: Mapping[Pair, tuple[int, ...]],
             tick: int, checks: Checks) -> set[Pair]:
    """Find reciprocal edges, then collapse each edge into one unordered couple."""
    pairs: set[Pair] = set()
    for identifier, row in rows.items():
        partner = row.optional("partner")
        pairing_tick = row.optional("pairing_tick")
        checks.require(identifier in people, row.where, f"unknown person {identifier}")
        checks.require(partner is None or partner in people, row.where, f"unknown partner {partner}")
        checks.require(partner != identifier, row.where, "self partner")
        checks.require(pairing_tick is None or pairing_tick <= tick, row.where, "future pairing_tick")
        if partner is None:
            checks.require(pairing_tick is None, row.where, "unpaired person retains pairing_tick")
            continue
        pair = tuple(sorted((identifier, partner)))
        checks.require(pairing_tick in history.get(pair, ()), row.where,
                       f"partner {partner}, pairing_tick {pairing_tick} absent from pairing events")
        other = rows.get(partner)
        if other is not None:
            checks.equal(other.optional("partner"), identifier, f"{row.where} living reciprocal partner")
            checks.equal(other.optional("pairing_tick"), pairing_tick, f"{row.where} reciprocal pairing_tick")
            if other.optional("partner") == identifier and partner != identifier:
                pairs.add(pair)
    return pairs


def night_signature(row: Row) -> Json:
    return {key: row.get(key) for key in ("tick", "simulation_day", "display_day", "bed", "partner", "household_key")}


def samples(evidence: Evidence, people: Mapping[int, Row], history: Mapping[Pair, tuple[int, ...]],
            end: int, offset: int, checks: Checks) -> tuple[list[Json], list[Json]]:
    expected_ticks = list(range((-offset) % 60, end, 60))
    checks.equal([row.num("tick") for row in evidence.hours], expected_ticks, f"{evidence.stem} hourly inventory")
    lifetime_nights: dict[int, list[Json]] = defaultdict(list)
    nights: list[Json] = []
    all_hours: list[Json] = []
    by_tick: dict[int, Mapping[int, Row]] = {}
    for snapshot in evidence.hours:
        clock(snapshot, end, offset, checks)
        tick = snapshot.num("tick")
        checks.equal(snapshot.num("elapsed_ticks"), tick + 1, f"{snapshot.where} elapsed_ticks")
        members = indexed(snapshot.rows("people"), "id", checks)
        by_tick[tick] = members
        couples = partners(members, people, history, tick, checks)
        for identifier, member in members.items():
            checks.require(member.text("life_stage") in ("adult", "child", "elder"), member.where, "invalid life stage")
            member.optional("settlement")
            member.num("household_key")
            person = people.get(identifier)
            if person is not None:
                born = person.optional("born_tick")
                checks.require(born is None or born <= tick, member.where, "person sampled before birth")
        bedded = sum(member.bed() is not None for member in members.values())
        capacity = snapshot.num("housing_capacity")
        checks.require(bedded <= capacity, snapshot.where, f"{bedded} bedded exceeds capacity {capacity}")
        housing: dict[str, Json] = {"tick": tick, "simulation_day": tick // DAY + 1,
            "display_day": snapshot.num("display_day"), "minute": snapshot.num("minute"),
            "people": len(members), "housing_capacity": capacity, "people_with_bed": bedded,
            "bedless": len(members) - bedded, "unused_slots": max(0, capacity - bedded)}
        for key in ("people_with_bed", "bedless", "unused_slots"):
            checks.equal(snapshot.num(key), housing[key], f"{snapshot.where} {key}")
        all_hours.append(housing)
        if snapshot.num("minute") != 1200:
            continue
        counts = Counter[str]()
        for first, second in couples:
            beds = (members[first].bed(), members[second].bed())
            present = sum(bed is not None for bed in beds)
            counts[f"beds_{present}"] += 1
            if present == 2:
                counts["same" if beds[0] == beds[1] else "different"] += 1
        nights.append({**housing, "mutual_living_couples": len(couples),
            "both_bedded_denominator": counts["beds_2"], "same_bed_numerator": counts["same"],
            "different_bed": counts["different"], "one_bed_excluded": counts["beds_1"],
            "neither_bed_excluded": counts["beds_0"]})
        for identifier, member in members.items():
            lifetime_nights[identifier].append({
                "tick": tick, "simulation_day": snapshot.num("simulation_day"),
                "display_day": snapshot.num("display_day"), "bed": member.get("bed"),
                "partner": member.get("partner"), "household_key": member.get("household_key")})
    for identifier, person in people.items():
        actual = [night_signature(row) for row in person.rows("shelter_nights")]
        checks.equal(actual, lifetime_nights[identifier], f"{person.where} complete shelter_nights")
    for event in evidence.events:
        if event.text("event") != "paired":
            continue
        members = by_tick.get(event.num("tick"))
        if members is None:
            continue
        mother, father = event.num("mother"), event.num("father")
        for identifier, partner in ((mother, father), (father, mother)):
            member = members.get(identifier)
            if member is not None:
                checks.equal(member.optional("partner"), partner, f"{event.where} same-tick sample partner")
                checks.equal(member.optional("pairing_tick"), event.num("tick"), f"{event.where} same-tick pairing_tick")
    living = {identifier: person for identifier, person in people.items() if person.flag("alive_end")}
    partners(living, people, history, max(0, end - 1), checks)
    return nights, all_hours


def departures(evidence: Evidence, people: Mapping[int, Row], history: Mapping[Pair, tuple[int, ...]],
               end: int, checks: Checks) -> list[Json]:
    """Join departure events to both final-status inventories; count selected pair endpoints."""
    events = indexed(tuple(row for row in evidence.events if row.text("event") == "departed"), "fission_id", checks)
    final = indexed(evidence.fissions, "fission_id", checks)
    ordinary = indexed(evidence.ordinary_fissions, "fission_id", checks)
    checks.equal(sorted(events), sorted(final), "departure event/family fission IDs")
    checks.equal(sorted(events), sorted(ordinary), "departure event/ordinary fission IDs")
    output: list[Json] = []
    statuses = Counter[str]()
    for identifier, event in events.items():
        members = event.ids("members")
        selected = set(members)
        groups = event.groups("households")
        raw_pairs = event.groups("couples")
        checks.require(len(selected) == len(members), event.where, "duplicate departing person")
        checks.require(event.num("leader") in selected, event.where, "leader absent from departure members")
        residents = [person for group in groups for person in group]
        checks.require(all(groups) and len(set(residents)) == len(residents), event.where, "household partition empty or duplicated")
        checks.require(selected <= set(residents), event.where, "departure member missing from household partition")
        checks.require(set(residents) <= set(people), event.where, "household person missing from lifetime inventory")
        endpoints = [person for pair in raw_pairs for person in pair]
        checks.require(all(len(pair) == 2 for pair in raw_pairs) and len(set(endpoints)) == len(endpoints),
                       event.where, "couples are not distinct disjoint pairs")
        checks.require(set(endpoints) <= set(residents), event.where, "couple endpoint missing from household partition")
        couple_splits = 0
        for raw_pair in raw_pairs:
            if len(raw_pair) != 2:
                continue
            pair = (min(raw_pair), max(raw_pair))
            selected_count = len(selected.intersection(pair))
            couple_splits += int(selected_count == 1)
            checks.require(any(tick <= event.num("tick") for tick in history.get(pair, ())),
                           event.where, f"departure couple {pair} has no prior pair event")
        group_splits = sum(0 < len(selected.intersection(group)) < len(group) for group in groups)
        whole_groups = sum(bool(group) and len(selected.intersection(group)) == len(group) for group in groups)
        counts = {"couples_total": len(raw_pairs), "couples_split": couple_splits,
                  "households_total": len(groups), "households_split": group_splits,
                  "households_departed_whole": whole_groups}
        for key, value in counts.items():
            checks.equal(event.num(key), value, f"{event.where} raw {key}")
        record = final.get(identifier)
        source = ordinary.get(identifier)
        if record is None or source is None:
            continue
        for key, value in event.data.items():
            if key not in ("tick", "simulation_day", "display_day", "minute"):
                checks.equal(record.get(key), value, f"{record.where} departure event {key}")
        for key in ("fission_id", "parent", "depart_tick", "members", "status"):
            checks.equal(record.get(key), source.get(key), f"{record.where} ordinary {key}")
        founded, cancelled = record.optional("founded_tick"), record.optional("cancelled_tick")
        status = "cancelled" if cancelled is not None else ("completed" if founded is not None else "pending")
        checks.equal(record.text("status"), status, f"{record.where} terminal timestamps")
        for terminal in (founded, cancelled):
            checks.require(terminal is None or event.num("tick") <= terminal < end, record.where, "terminal tick outside departure/run")
        if founded is not None:
            checks.require(record.optional("daughter") is not None, record.where, "founded fission has no daughter")
        statuses[status] += 1
        output.append({"fission_id": identifier, "parent": event.num("parent"), "tick": event.num("tick"),
                       "status": status, "members": list(members), "couples": [list(pair) for pair in raw_pairs],
                       **counts, "founded_tick": founded, "cancelled_tick": cancelled})
    for status in ("completed", "cancelled", "pending"):
        checks.equal(evidence.trace_meta.num(f"fissions_{status}"), statuses[status], f"{evidence.stem} ordinary {status} census")
    for key in ("departures_total", "source_departures_total"):
        checks.equal(evidence.meta.num(key), len(events), f"{evidence.stem} {key}")
    return output


def family_example(evidence: Evidence) -> Row:
    """Select one observed pair maximizing linked birth, night and fission evidence."""
    candidates: list[tuple[int, int, int, list[Json]]] = []
    for pair_event in evidence.events:
        if pair_event.text("event") != "paired":
            continue
        mother, father = pair_event.num("mother"), pair_event.num("father")
        pair = {mother, father}
        linked_births = [event for event in evidence.events if event.text("event") == "born"
                         and not event.flag("fallback") and event.optional("mother") == mother
                         and event.optional("father") == father]
        linked_nights = []
        for sample in evidence.hours:
            if sample.num("minute") != 1200:
                continue
            members = {row.num("id"): row for row in sample.rows("people")}
            if pair <= members.keys() and members[mother].optional("partner") == father and members[father].optional("partner") == mother:
                linked_nights.append(sample)
        linked_fissions = [row for row in evidence.fissions if any(set(raw) == pair for raw in row.groups("couples"))]
        rows: list[Json] = [{"kind": "pair", "source": pair_event.where, "raw": dict(pair_event.data)}]
        for birth in linked_births[:1]:
            rows.append({"kind": "birth", "source": birth.where, "raw": dict(birth.data)})
        if linked_nights:
            first_birth = min((birth.num("tick") for birth in linked_births), default=0)
            night = next((sample for sample in linked_nights if sample.num("tick") > first_birth), linked_nights[0])
            family_ids = pair | {birth.num("child") for birth in linked_births}
            rows.append({"kind": "night", "source": night.where,
                         "raw": {**{key: night.get(key) for key in ("tick", "simulation_day", "display_day", "minute", "housing_capacity", "bedless", "unused_slots")},
                                 "people": [dict(row.data) for row in night.rows("people") if row.num("id") in family_ids]}})
        for fission in linked_fissions[:1]:
            rows.append({"kind": "fission", "source": fission.where, "raw": dict(fission.data),
                         "selected_pair_members": sorted(pair.intersection(fission.ids("members")))})
        score = 1 + bool(linked_births) + bool(linked_nights) + bool(linked_fissions)
        candidates.append((score, mother, father, rows))
    if not candidates:
        return Row({"coverage": 0, "message": "No observed pairing event; no concrete paired family can be selected.", "rows": []}, "example")
    score, mother, father, rows = min(candidates, key=lambda item: (-item[0], item[1], item[2]))
    return Row({"coverage": score, "mother": mother, "father": father,
                "missing": [kind for kind in ("pair", "birth", "night", "fission")
                            if kind not in {Row.parse(row, "example").text("kind") for row in rows}],
                "rows": rows}, "example")


def exposure(numerator: int, denominator: int) -> Json:
    return {"numerator": numerator, "denominator": denominator,
            "ratio": numerator / denominator if denominator else None}


def audit(evidence: Evidence) -> Row:
    checks = Checks()
    end = evidence.meta.num("end_tick")
    checks.require(end > 0, evidence.stem, "empty run")
    checks.equal(evidence.meta.num("schema_version"), 17, f"{evidence.stem} schema")
    checks.equal(evidence.meta.num("observation_misses"), 0, f"{evidence.stem} observation_misses")
    checks.equal(evidence.trace_meta.num("end_tick"), end, f"{evidence.stem} ordinary end_tick")
    for key in ("born_event_misses", "component_mismatches"):
        checks.equal(evidence.trace_meta.num(key), 0, f"{evidence.stem} {key}")
    if not evidence.hours:
        raise InputError(evidence.stem, "no hourly family samples; cannot establish clock offset")
    first = evidence.hours[0]
    offset = (first.num("display_day") - 1) * DAY + first.num("minute") - first.num("tick")
    checks.require(offset >= 0, evidence.stem, "negative clock offset")
    people = identity(evidence, checks)
    checks.equal(evidence.meta.num("people"), len(people), f"{evidence.stem} lifetime count")
    history, daily = births_and_pairs(evidence, people, end, offset, checks)
    nights, hours = samples(evidence, people, history, end, offset, checks)
    fissions = departures(evidence, people, history, end, checks)
    night_rows = [Row.parse(row, "night") for row in nights]
    birth_rows = [Row.parse(row, "day") for row in daily]
    fission_rows = [Row.parse(row, "fission") for row in fissions]
    late = [row for row in birth_rows if row.num("day") >= 20]
    by_status: dict[str, Json] = {}
    for status in ("completed", "cancelled", "pending"):
        selected = [row for row in fission_rows if row.text("status") == status]
        by_status[status] = {"fissions": len(selected),
            "couple_splits": sum(row.num("couples_split") for row in selected),
            "couples_at_departure": sum(row.num("couples_total") for row in selected)}
    completed = Row.parse(by_status["completed"], "completed")
    day60: Json = None
    raw_day60 = evidence.meta.get("day60")
    if end < 60 * DAY:
        checks.equal(raw_day60, None, f"{evidence.stem} premature day60")
    else:
        boundary = Row.parse(raw_day60, f"{evidence.stem} day60")
        clock(boundary, end, offset, checks)
        checks.equal(boundary.num("tick"), 60 * DAY - 1, f"{boundary.where} tick")
        checks.equal(boundary.num("elapsed_ticks"), 60 * DAY, f"{boundary.where} elapsed_ticks")
        members = indexed(boundary.rows("people"), "id", checks)
        paired = {identifier for pair in partners(members, people, history, boundary.num("tick"), checks) for identifier in pair}
        adults = {identifier for identifier, member in members.items() if member.text("life_stage") == "adult"}
        day60 = exposure(len(adults & paired), len(adults))
        runs = [row for row in evidence.summary.rows("runs") if row.num("seed") == evidence.meta.num("seed")]
        checks.equal(len(runs), 1, f"{boundary.where} summary run inventory")
        if len(runs) == 1:
            alive = runs[0].ids("alive_daily")
            checks.require(len(alive) >= 60, runs[0].where, "missing day60 daily population")
            if len(alive) >= 60:
                checks.equal(len(members), alive[59], f"{boundary.where} summary alive day60")
    return Row({"stem": evidence.stem, "seed": evidence.meta.num("seed"), "scene": evidence.meta.text("scene"),
        "git_sha": evidence.meta.text("git_sha"), "end_tick": end, "clock_offset_ticks": offset,
        "input_sha256": dict(evidence.hashes), "checks": checks.count, "issues": list(checks.issues),
        "status": "PASS" if not checks.issues else "FAIL", "lifetime_people": len(people),
        "female": sum(person.flag("female") for person in people.values()),
        "male": sum(not person.flag("female") for person in people.values()),
        "pair_events": sum(len(times) for times in history.values()),
        "partner_history": [{"pair": list(pair), "ticks": list(times)} for pair, times in sorted(history.items())],
        "births_by_day": daily, "housing_hours": hours, "nights": nights, "fissions": fissions,
        "fissions_by_status": by_status,
        "F1_raw_day60": day60,
        "F2_raw": exposure(sum(row.num("couple_births") for row in late), sum(row.num("births") for row in late)),
        "F4_raw": exposure(sum(row.num("same_bed_numerator") for row in night_rows), sum(row.num("both_bedded_denominator") for row in night_rows)),
        "F5_raw": exposure(completed.num("couple_splits"), completed.num("couples_at_departure")),
        "family_example": dict(family_example(evidence).data)}, "report")


def human(report: Row) -> str:
    lines = [f"{report.text('status')} — {report.text('stem')}",
             f"seed={report.num('seed')} git_sha={report.text('git_sha')} elapsed={report.num('end_tick')} checks={report.num('checks')}",
             f"lifetime people={report.num('lifetime_people')} female={report.num('female')} male={report.num('male')} pair events={report.num('pair_events')}"]
    for name in ("F2_raw", "F4_raw", "F5_raw"):
        row = Row.parse(report.get(name), name)
        lines.append(f"{name}: {row.num('numerator')}/{row.num('denominator')} ratio={row.get('ratio')}")
    lines.append(f"F1_raw_day60: {json.dumps(report.get('F1_raw_day60'), sort_keys=True)}")
    lines.append("Births by elapsed simulation day: day total couple fallback")
    for day in report.rows("births_by_day"):
        lines.append("  " + " ".join(str(day.num(key)) for key in ("day", "births", "couple_births", "fallback_births")))
    lines.append("20:00 nights: tick simday same/both-bedded living-couples one-bed neither-bed bedless unused")
    for night in report.rows("nights"):
        lines.append(f"  {night.num('tick')} {night.num('simulation_day')} {night.num('same_bed_numerator')}/{night.num('both_bedded_denominator')} "
                     f"{night.num('mutual_living_couples')} {night.num('one_bed_excluded')} {night.num('neither_bed_excluded')} "
                     f"{night.num('bedless')} {night.num('unused_slots')}")
    lines.append("Departures: id tick status split-couples/all-parent-couples split-households/all-households")
    for fission in report.rows("fissions"):
        lines.append(f"  {fission.num('fission_id')} {fission.num('tick')} {fission.text('status')} "
                     f"{fission.num('couples_split')}/{fission.num('couples_total')} "
                     f"{fission.num('households_split')}/{fission.num('households_total')}")
    example = Row.parse(report.get("family_example"), "example")
    lines.append(f"Concrete family evidence ({example.num('coverage')}/4 categories):")
    if example.num("coverage") == 0:
        lines.append(f"  {example.text('message')}")
    else:
        lines.append(f"  mother={example.num('mother')} father={example.num('father')} missing={example.get('missing')}")
    for row in example.rows("rows"):
        lines.append(f"  {row.text('kind')} source={row.text('source')} {json.dumps(row.get('raw'), sort_keys=True)}")
        if "selected_pair_members" in row.data:
            lines.append(f"    selected_pair_members={row.get('selected_pair_members')}")
    for issue in array(report.get("issues"), "issues"):
        lines.append(f"ERROR: {issue}")
    lines.append("Input SHA256:")
    for path, digest in sorted(Row.parse(report.get("input_sha256"), "input_sha256").data.items()):
        lines.append(f"  {digest}  {path}")
    return "\n".join(lines)


def main(arguments: Sequence[str]) -> int:
    """Read a completed output directory recursively, without creating any files."""
    json_output = "--json" in arguments
    paths = [value for value in arguments if value != "--json"]
    if len(paths) != 1 or paths[0].startswith("-"):
        print("Usage: uv run --no-project family_trace_recompute.py OUTPUT_DIR [--json]", file=sys.stderr)
        return 2
    try:
        root = Path(paths[0]).resolve(strict=True)
        metas = sorted(root.rglob("*_family_meta.json"))
        if not metas:
            raise InputError(str(root), "no *_family_meta.json files; no cohort verified")
        reports = [audit(load(path)) for path in metas]
        failed = any(report.text("status") == "FAIL" for report in reports)
        if json_output:
            print(json.dumps({"checker": "independent raw family trace recomputation", "worlds": len(reports),
                "status": "FAIL" if failed else "PASS", "limitations": LIMITATIONS,
                "reports": [dict(report.data) for report in reports]}, indent=2, sort_keys=True))
        else:
            print("\n\n".join(human(report) for report in reports))
            print("\nEvidence limits:\n" + "\n".join(f"- {text}" for text in LIMITATIONS))
            print(f"\n{'FAIL' if failed else 'PASS'} raw consistency, {len(reports)} worlds; acceptance gates not evaluated.")
        return int(failed)
    except (InputError, OSError, json.JSONDecodeError) as error:
        print(f"INPUT ERROR: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
