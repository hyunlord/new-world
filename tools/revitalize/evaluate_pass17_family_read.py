from __future__ import annotations

from collections.abc import Mapping, Sequence
from pathlib import Path

from evaluate_pass17_data import Cohort, Hour, InputError, World
from evaluate_pass17_family_data import FamilyBirth, FamilyPerson, FamilySnapshot, FamilyWorld
from evaluate_pass17_family_rows import event_tick, optional_id, parse_births, parse_family_person, parse_fissions, parse_members
from evaluate_pass17_read import JsonValue, array, integer, mapping, read_json, read_jsonl, required
from evaluate_pass17_stage_b import births_by_day


def parse_snapshots(rows: Sequence[Mapping[str, JsonValue]], world: World) -> tuple[FamilySnapshot, ...]:
    expected = sorted(world.hours, key=lambda hour: (hour.day, hour.minute))
    if len(rows) != len(expected):
        raise InputError("family_hours", "hour inventory differs from routine hours")
    output = []
    for index, (row, hour) in enumerate(zip(rows, expected, strict=True)):
        tick, day = event_tick(row, len(expected) * 60)
        if tick != index * 60 or integer(required(row, "elapsed_ticks"), "elapsed_ticks") != tick + 1:
            raise InputError("family_hours", "expected ordered hourly ticks and elapsed_ticks=tick+1")
        if (integer(required(row, "display_day"), "display_day"), integer(required(row, "minute"), "minute")) != (hour.day, hour.minute):
            raise InputError("family_hours", "display clock differs from routine hours")
        members = parse_members(required(row, "people"), tick)
        stage_counts = tuple(sum(member.stage == stage for member in members) for stage in ("adult", "child", "elder"))
        if stage_counts != tuple(sum(counts) for counts in hour.states):
            raise InputError("family_hours", "living stage inventory differs from routine hourly counts")
        bedded = sum(member.shelter is not None for member in members)
        capacity = integer(required(row, "housing_capacity"), "housing_capacity")
        if (integer(required(row, "people_with_bed"), "people_with_bed"), integer(required(row, "bedless"), "bedless"),
                integer(required(row, "unused_slots"), "unused_slots")) != (bedded, len(members) - bedded, capacity - bedded):
            raise InputError("family_hours", "housing counters disagree with person beds/capacity")
        output.append(FamilySnapshot(tick, day, hour.minute, members))
    return tuple(output)


def parse_day60(meta: Mapping[str, JsonValue], days: int, first_hour: Hour) -> FamilySnapshot | None:
    raw = required(meta, "day60")
    if days < 60:
        if raw is not None:
            raise InputError("day60", "day60 snapshot precedes cohort completion")
        return None
    row = mapping(raw, "day60")
    tick = integer(required(row, "tick"), "tick")
    if tick != 86399 or integer(required(row, "elapsed_ticks"), "elapsed_ticks") != 86400:
        raise InputError("day60", "requires exact elapsed day60 boundary: tick86399, elapsed_ticks86400")
    return FamilySnapshot(tick, 60, (first_hour.minute + tick) % 1440, parse_members(required(row, "people"), tick))


def verify_births(people_rows: Sequence[Mapping[str, JsonValue]], births: Sequence[FamilyBirth]) -> None:
    expected = {integer(required(row, "id"), "id"): optional_id(required(row, "born_tick"), "born_tick")
        for row in people_rows if required(row, "born_tick") is not None}
    if {birth.child: birth.tick for birth in births} != expected:
        raise InputError("births", "born-event inventory disagrees with nonfounder people")
    rows = {integer(required(row, "id"), "id"): row for row in people_rows}
    for birth in births:
        if birth.mother not in rows or birth.father not in rows:
            raise InputError("births", "parent is absent from lifetime people inventory")
        row = rows[birth.child]
        if (required(row, "mother"), required(row, "father")) != (birth.mother, birth.father):
            raise InputError("births", "event parents disagree with recorded Kin")
        if not birth.fallback and (required(rows[birth.mother], "female") is not True or required(rows[birth.father], "female") is not False):
            raise InputError("births", "nonfallback parents do not have mother/father sex roles")


def verify_people(people: Sequence[FamilyPerson], snapshots: Sequence[FamilySnapshot]) -> None:
    ids = {person.agent_id for person in people}
    if len(ids) != len(people):
        raise InputError("family_people", "duplicate lifetime person")
    for person in people:
        if any(parent is not None and parent not in ids for parent in (person.mother, person.father)):
            raise InputError("family_people", "known parent missing from lifetime inventory")
    for snapshot in snapshots:
        members = {member.agent_id: member for member in snapshot.members}
        for member in snapshot.members:
            if member.agent_id not in ids or (member.partner is not None and member.partner not in ids):
                raise InputError("family_hours", "person/partner missing from lifetime inventory")
            if member.partner in members and members[member.partner].partner != member.agent_id:
                raise InputError("family_hours", "living partners are not mutual")


def read_family(cohort: Cohort, world: World) -> FamilyWorld:
    stem = cohort.source / f"{cohort.scene}_s{world.seed}"
    people_rows = read_jsonl(Path(f"{stem}_family_people.jsonl"))
    people = tuple(parse_family_person(row) for row in people_rows)
    ordinary_ids = {integer(required(row, "id"), "id") for row in read_jsonl(Path(f"{stem}_people.jsonl"))}
    if {person.agent_id for person in people} != ordinary_ids:
        raise InputError("family_people", "lifetime IDs differ from routine people inventory")
    snapshots = parse_snapshots(read_jsonl(Path(f"{stem}_family_hours.jsonl")), world)
    meta = mapping(read_json(Path(f"{stem}_family_meta.json")), "family_meta")
    if integer(required(meta, "schema_version"), "schema_version") != 17 or integer(required(meta, "end_tick"), "end_tick") != cohort.days * 1440:
        raise InputError("family_meta", "schema or completed duration differs from cohort")
    if integer(required(meta, "observation_misses"), "observation_misses") != 0:
        raise InputError("family_meta", "family observation misses invalidate metrics")
    day60 = parse_day60(meta, cohort.days, world.hours[0])
    verify_people(people, (*snapshots, day60) if day60 is not None else snapshots)
    if day60 is not None:
        summary = mapping(read_json(cohort.source / "summary.json"), "summary")
        runs = [mapping(row, "runs") for row in array(required(summary, "runs"), "runs")]
        run = next(row for row in runs if required(row, "seed") == world.seed)
        alive60 = integer(array(required(run, "alive_daily"), "alive_daily")[59], "alive_daily[59]")
        if len(day60.members) != alive60:
            raise InputError("day60", "snapshot population differs from summary day60 alive")
    births = parse_births(read_jsonl(Path(f"{stem}_family_events.jsonl")), cohort.days * 1440)
    verify_births(people_rows, births)
    fissions = parse_fissions(read_jsonl(Path(f"{stem}_family_fissions.jsonl")), cohort.days * 1440)
    for field, expected in (("people", len(people)), ("births_total", len(births)), ("source_births_total", len(births)),
            ("departures_total", len(fissions)), ("source_departures_total", len(fissions))):
        if integer(required(meta, field), field) != expected:
            raise InputError("family_meta", f"{field} disagrees with complete family records")
    routine_meta = mapping(read_json(Path(f"{stem}_trace_meta.json")), "trace_meta")
    for status in ("completed", "cancelled", "pending"):
        if integer(required(routine_meta, f"fissions_{status}"), f"fissions_{status}") != sum(f.status == status for f in fissions):
            raise InputError("family_fissions", "status counts differ from existing fission census")
    result = FamilyWorld(world.seed, people, snapshots, day60, births, fissions)
    daily = [{field: integer(required(mapping(row, "births_by_day"), field), field)
        for field in ("day", "births", "couple_births", "fallback_births")}
        for row in array(required(meta, "births_by_day"), "births_by_day")]
    if daily != births_by_day((result,), cohort.days):
        raise InputError("births_by_day", "daily birth/fallback counts disagree with complete raw birth events")
    return result


def load_families(cohort: Cohort) -> tuple[FamilyWorld, ...]:
    if cohort.scene != "game_v22":
        raise InputError("family cohort", "family metrics require game_v22")
    return tuple(read_family(cohort, world) for world in cohort.worlds)
