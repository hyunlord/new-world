from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal
from collections.abc import Mapping

KINDS: Final = ("TendFire", "Craft", "Repair", "HearthCircle", "Explore", "Play", "SitByFire")
AXES: Final = ("ns", "ha", "rd", "p")
TICKS_PER_DAY: Final = 1440


class InputError(ValueError):
    def __init__(self, location: str, reason: str) -> None:
        self.location = location
        self.reason = reason
        super().__init__(f"{location}: {reason}")


@dataclass(frozen=True, slots=True)
class Person:
    agent_id: int
    temperament: tuple[int, ...]
    adult_ticks: int
    social_completed: int
    activity_minutes: tuple[int, ...]
    activities_started: tuple[int, ...]
    completed_activities: int
    completed_minutes: int
    no_place_redraws: int
    no_place_failures: int


@dataclass(frozen=True, slots=True)
class Hour:
    day: int
    minute: int
    states: tuple[tuple[int, ...], ...]
    food: int
    wood: int


@dataclass(frozen=True, slots=True)
class World:
    seed: int
    alive_end: int
    non_old_age_deaths: int
    old_age_deaths: int
    shelters_completed: int
    people: tuple[Person, ...]
    hours: tuple[Hour, ...]
    no_place_redraws: int
    no_place_failures: int
    all_activity_minutes: tuple[int, ...]
    all_activities_started: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class Cohort:
    scene: str
    days: int
    worlds: tuple[World, ...]
    source: Path


@dataclass(frozen=True, slots=True)
class CohortSpec:
    scene: str
    seeds: tuple[int, ...]
    days: int


Verdict = Literal["PASS", "FAIL", "ERROR", "PENDING"]


@dataclass(frozen=True, slots=True)
class Metric:
    value: float | None
    ci95: tuple[float, float] | None
    verdict: Verdict
    bar: str
    unit: str
    candidate: float | None = None
    baseline: float | None = None
    bootstrap_valid: int = 0
    bootstrap_replicates: int = 2000
    details: Mapping[str, int | float | str] | None = None
    error: str | None = None


@dataclass(frozen=True, slots=True)
class Criterion:
    lower: float | None
    upper: float | None
    bar: str
    unit: str
    positive_interval: bool = False


@dataclass(frozen=True, slots=True)
class StageMetrics:
    metrics: Mapping[str, Metric]
    verdicts: Mapping[str, Verdict]
