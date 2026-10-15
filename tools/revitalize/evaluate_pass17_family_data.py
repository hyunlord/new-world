from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

LifeStage = Literal["child", "adult", "elder"]
StagePair = tuple[LifeStage, LifeStage]


@dataclass(frozen=True, slots=True)
class FamilyPerson:
    agent_id: int
    temperament: tuple[int, ...]
    female: bool
    mother: int | None
    father: int | None


@dataclass(frozen=True, slots=True)
class FamilyMember:
    agent_id: int
    settlement_id: int | None
    stage: LifeStage
    partner: int | None
    shelter: tuple[int, int] | None
    household: int
    pairing_tick: int | None


@dataclass(frozen=True, slots=True)
class FamilySnapshot:
    tick: int
    day: int
    minute: int
    members: tuple[FamilyMember, ...]


@dataclass(frozen=True, slots=True)
class FamilyBirth:
    tick: int
    day: int
    child: int
    mother: int
    father: int
    fallback: bool


@dataclass(frozen=True, slots=True)
class FamilyFission:
    tick: int
    parent_settlement: int
    couples: int
    split_couples: int
    households_kept: int
    households_split: int
    status: Literal["completed", "cancelled", "pending"] = "completed"


@dataclass(frozen=True, slots=True)
class FamilyWorld:
    seed: int
    people: tuple[FamilyPerson, ...]
    snapshots: tuple[FamilySnapshot, ...]
    day60: FamilySnapshot | None
    births: tuple[FamilyBirth, ...]
    fissions: tuple[FamilyFission, ...]


@dataclass(frozen=True, slots=True)
class FamilyPair:
    first: FamilyPerson
    second: FamilyPerson
    tick: int | None
    settlement_id: int | None
    stages: StagePair | None


@dataclass(frozen=True, slots=True)
class FamilyPairs:
    seed: int
    siblings: tuple[FamilyPair, ...]
    unrelated: tuple[FamilyPair, ...]
    categories: tuple[StagePair, ...]
    excluded_kin: int
