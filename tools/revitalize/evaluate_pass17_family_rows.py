from __future__ import annotations

from collections.abc import Mapping, Sequence

from evaluate_pass17_data import AXES, InputError
from evaluate_pass17_evidence import boolean
from evaluate_pass17_family_data import FamilyBirth, FamilyFission, FamilyMember, FamilyPerson, LifeStage
from evaluate_pass17_read import JsonValue, array, counts, integer, mapping, required


def optional_id(value: JsonValue, location: str) -> int | None:
    return None if value is None else integer(value, location)


def parse_stage(value: JsonValue) -> LifeStage:
    match value:
        case "child":
            return "child"
        case "adult":
            return "adult"
        case "elder":
            return "elder"
        case _:
            raise InputError("life_stage", "requires child, adult or elder")


def parse_family_person(row: Mapping[str, JsonValue]) -> FamilyPerson:
    axes = mapping(required(row, "temperament"), "temperament")
    temperament = tuple(integer(required(axes, axis), axis) for axis in AXES)
    if max(temperament) > 1000:
        raise InputError("temperament", "axis exceeds 1000")
    person = FamilyPerson(integer(required(row, "id"), "id"), temperament,
        boolean(required(row, "female"), "female"), optional_id(required(row, "mother"), "mother"),
        optional_id(required(row, "father"), "father"))
    if person.agent_id in (person.mother, person.father):
        raise InputError("parents", "a person cannot be their own parent")
    integer(required(row, "lineage_key"), "lineage_key")
    optional_id(required(row, "born_tick"), "born_tick")
    boolean(required(row, "alive_end"), "alive_end")
    optional_id(required(row, "partner"), "partner")
    optional_id(required(row, "pairing_tick"), "pairing_tick")
    optional_id(required(row, "household_key"), "household_key")
    return person


def parse_members(value: JsonValue, tick: int) -> tuple[FamilyMember, ...]:
    output = []
    for value in array(value, "people"):
        row = mapping(value, "people")
        bed = required(row, "bed")
        tile = counts(bed, 2, "bed") if bed is not None else None
        pairing_tick = optional_id(required(row, "pairing_tick"), "pairing_tick")
        if pairing_tick is not None and pairing_tick > tick:
            raise InputError("pairing_tick", "future pairing tick")
        output.append(FamilyMember(integer(required(row, "id"), "id"),
            optional_id(required(row, "settlement"), "settlement"), parse_stage(required(row, "life_stage")),
            optional_id(required(row, "partner"), "partner"), (tile[0], tile[1]) if tile is not None else None,
            integer(required(row, "household_key"), "household_key"), pairing_tick))
    if len({person.agent_id for person in output}) != len(output):
        raise InputError("people", "duplicate living person at a snapshot")
    for member in output:
        if member.partner == member.agent_id:
            raise InputError("partner", "self partner")
    return tuple(output)


def event_tick(row: Mapping[str, JsonValue], end_tick: int) -> tuple[int, int]:
    tick = integer(required(row, "tick"), "tick")
    day = integer(required(row, "simulation_day"), "simulation_day")
    if tick >= end_tick or day != tick // 1440 + 1:
        raise InputError("simulation_day", "event day must equal tick//1440+1 inside cohort duration")
    return tick, day


def parse_births(rows: Sequence[Mapping[str, JsonValue]], end_tick: int) -> tuple[FamilyBirth, ...]:
    births = []
    for row in rows:
        tick, day = event_tick(row, end_tick)
        match required(row, "event"):
            case "born":
                births.append(FamilyBirth(tick, day, integer(required(row, "child"), "child"),
                    integer(required(row, "mother"), "mother"), integer(required(row, "father"), "father"),
                    boolean(required(row, "fallback"), "fallback")))
            case "paired" | "departed":
                continue
            case _:
                raise InputError("event", "unknown family event kind")
    if len({birth.child for birth in births}) != len(births):
        raise InputError("births", "duplicate child birth")
    return tuple(births)


def parse_fissions(rows: Sequence[Mapping[str, JsonValue]], end_tick: int) -> tuple[FamilyFission, ...]:
    output = []
    ids: set[int] = set()
    for row in rows:
        fission_id = integer(required(row, "fission_id"), "fission_id")
        if fission_id in ids:
            raise InputError("fissions", "duplicate fission id")
        ids.add(fission_id)
        tick = integer(required(row, "depart_tick"), "depart_tick")
        if tick >= end_tick:
            raise InputError("depart_tick", "departure outside cohort duration")
        members_list = tuple(integer(item, "members") for item in array(required(row, "members"), "members"))
        members = set(members_list)
        if len(members) != len(members_list):
            raise InputError("members", "duplicate departure member")
        couples = tuple(counts(pair, 2, "couples") for pair in array(required(row, "couples"), "couples"))
        if any(a == b for a, b in couples) or len({person for pair in couples for person in pair}) != 2 * len(couples):
            raise InputError("couples", "departure couples must be unique disjoint pairs")
        households = [tuple(integer(person, "households") for person in array(group, "households"))
            for group in array(required(row, "households"), "households")]
        residents = [person for group in households for person in group]
        if any(not group for group in households) or len(set(residents)) != len(residents) or not members.issubset(residents):
            raise InputError("households", "parent household partition is incomplete or duplicated")
        if not {person for pair in couples for person in pair}.issubset(residents):
            raise InputError("couples", "couples absent from parent households")
        split = sum((a in members) != (b in members) for a, b in couples)
        households_split = sum(bool(set(group) & members) and not set(group).issubset(members) for group in households)
        if (integer(required(row, "couples_total"), "couples_total"), integer(required(row, "couples_split"), "couples_split"),
                integer(required(row, "households_split"), "households_split")) != (len(couples), split, households_split):
            raise InputError("fissions", "departure counters disagree with raw members and households")
        status = required(row, "status")
        if status not in ("completed", "cancelled", "pending"):
            raise InputError("status", "unknown fission completion status")
        output.append(FamilyFission(tick, integer(required(row, "parent"), "parent"),
            len(couples), split, len(households) - households_split, households_split, status))
    return tuple(output)
