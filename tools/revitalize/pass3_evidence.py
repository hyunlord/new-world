# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Strict evidence boundary for the Pass 3 evaluator (stdlib by goal constraint)."""
from __future__ import annotations

import json
import math
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final, TypeAlias

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
FLOAT_KEYS: Final = set("day hunger_mean hunger_p90 thirst_mean thirst_p90 fatigue_mean loneliness_mean home_distance_mean food_supply_per_day food_need_per_day".split())
TEXT_KEYS: Final = {"scene", "git_sha", "fingerprint", "sample_kind"}
KEYS: Final = set("""schema_version scene seed git_sha tick day alive births deaths_starvation deaths_dehydration deaths_combat hunger_mean hunger_p90 thirst_mean thirst_p90 fatigue_mean loneliness_mean state_idle state_seeking state_consuming idle_with_breach settlements settled_agents home_distance_mean buildings shelters construction_sites shelters_completed stockpile_food carried_food food_tiles_total food_tiles_finite_total water_tiles_count idle_breach_actionable idle_breach_unavailable social_chasing social_completed fingerprint sample_kind minute_of_day adults children housing_capacity_owned food_supply_per_day food_need_per_day sustainable_adults food_harvested_total meals_eaten_total meals_without_food_total beds_assigned sleeping_in_shelter idle_adults idle_adults_near_hearth""".split())
CLOCKS: Final = {120: ("night_in_shelter_share", "sleeping_in_shelter", "alive"), 720: ("noon_idle_share", "state_idle", "alive"), 1170: ("evening_near_hearth_share", "idle_adults_near_hearth", "idle_adults")}

@dataclass(frozen=True, slots=True)
class EvidenceError(ValueError):
    """Invalid or incomplete evidence; never interpreted as target success."""
    detail: str

    def __str__(self) -> str:
        return self.detail

def document(path: Path) -> Json:
    with path.open(encoding="utf-8") as stream:
        value: Json = json.load(stream)
    return value

def mapping(value: Json) -> dict[str, Json]:
    if not isinstance(value, dict):
        raise EvidenceError("expected JSON object")
    return value

def sequence(value: Json) -> list[Json]:
    if not isinstance(value, list):
        raise EvidenceError("expected JSON array")
    return value

def number(value: Json) -> float:
    if type(value) not in (int, float) or not math.isfinite(value):
        raise EvidenceError(f"expected finite number, got {value!r}")
    return float(value)

def integer(value: Json) -> int:
    if type(value) is not int or value < 0:
        raise EvidenceError(f"expected nonnegative integer, got {value!r}")
    return value

def boolean(value: Json) -> bool:
    if type(value) is not bool:
        raise EvidenceError("expected explicit boolean")
    return value

def quantiles(values: list[float]) -> dict[str, float]:
    if not values:
        raise EvidenceError("empty metric population")
    ordered = sorted(values)
    result: dict[str, float] = {}
    for key, fraction in (("min", 0.0), ("p10", .1), ("median", .5), ("p90", .9), ("max", 1.0)):
        index = fraction * (len(ordered) - 1)
        lo = math.floor(index)
        hi = math.ceil(index)
        result[key] = ordered[lo] + (ordered[hi] - ordered[lo]) * (index - lo)
    return result

@dataclass(frozen=True, slots=True)
class SeedEvidence:
    seed: int
    days: dict[int, dict[str, float]]
    minimum_alive: float
    clocks: dict[int, dict[int, dict[str, float]]]
    overflow: int
    social_rate: float
    timing: float
    maturation: dict[str, int]
    actionable: dict[int, float]

@dataclass(frozen=True, slots=True)
class RoundEvidence:
    path: Path
    duration: int
    runs: tuple[SeedEvidence, ...]
    deterministic: bool
    actionable_share: dict[int, dict[str, float]]
    revision: str

    def values(self, day: int, key: str) -> list[float]:
        return [run.days[day][key] for run in self.runs]

def sample(raw: Json, seed: int) -> tuple[str, int, dict[str, float], str, str]:
    row = mapping(raw)
    if set(row) != KEYS:
        raise EvidenceError(f"seed {seed}: expected exact 53 sample keys")
    if row["schema_version"] != 3 or type(row["schema_version"]) is not int or row["scene"] != "game_v9" or integer(row["seed"]) != seed:
        raise EvidenceError(f"seed {seed}: schema/scene/seed mismatch")
    for key in TEXT_KEYS:
        if not isinstance(row[key], str) or not row[key]:
            raise EvidenceError(f"seed {seed}: invalid {key}")
    revision, fingerprint = str(row["git_sha"]), str(row["fingerprint"])
    if not re.fullmatch("[0-9a-f]{40}", revision) or not re.fullmatch("[0-9a-f]{16}", fingerprint):
        raise EvidenceError(f"seed {seed}: expected full producer SHA and 16-digit fingerprint")
    kind = str(row["sample_kind"])
    if kind not in {"day", "clock", "periodic"}:
        raise EvidenceError(f"invalid sample kind {kind}")
    values = {key: number(value) for key, value in row.items() if key not in TEXT_KEYS}
    for key in KEYS - TEXT_KEYS - FLOAT_KEYS:
        integer(row[key])
    if any(value < 0 for value in values.values()):
        raise EvidenceError("negative sample metric")
    tick = integer(row["tick"])
    if values["day"] != tick / 1440 or values["adults"] + values["children"] != values["alive"]:
        raise EvidenceError("day/population partition mismatch")
    if values["minute_of_day"] >= 1440:
        raise EvidenceError("invalid minute_of_day")
    return kind, tick, values, revision, fingerprint

def raw_seed(path: Path, seed: int, plan: tuple[int, int]) -> tuple[dict[int, dict[str, float]], float, dict[int, dict[int, dict[str, float]]], dict[int, float], str]:
    duration, sample_every = plan
    days: dict[int, dict[str, float]] = {}
    periodic: dict[int, tuple[float, float]] = {}
    clocks: dict[int, dict[int, dict[str, float]]] = {}
    minimum = math.inf
    previous_tick = -1
    producer = ""
    previous_fingerprint = ""
    previous_values: dict[str, float] = {}
    kinds: set[str] = set()
    offset = -1
    with path.open(encoding="utf-8") as stream:
        for line in stream:
            raw: Json = json.loads(line)
            kind, tick, values, revision, fingerprint = sample(raw, seed)
            if producer and revision != producer:
                raise EvidenceError(f"{path}: mixed producer revisions")
            producer = revision
            if tick > duration * 1440 or tick < previous_tick:
                raise EvidenceError(f"{path}: out-of-range/unordered tick {tick}")
            if previous_tick < 0:
                if tick != 0:
                    raise EvidenceError(f"{path}: missing initial sample")
                offset = int(values["minute_of_day"])
            if tick == previous_tick:
                if kind in kinds or values != previous_values or fingerprint != previous_fingerprint:
                    raise EvidenceError(f"{path}: conflicting values/fingerprint or duplicate tick {tick}")
            else:
                kinds = set()
            kinds.add(kind)
            previous_tick, previous_values, previous_fingerprint = tick, values, fingerprint
            if values["minute_of_day"] != (tick + offset) % 1440:
                raise EvidenceError(f"{path}: inconsistent clock offset")
            minimum = min(minimum, values["alive"])
            if kind == "periodic":
                periodic[tick] = (values["alive"], values["idle_breach_actionable"])
            if kind == "day":
                if tick == 0 or tick % 1440:
                    raise EvidenceError(f"{path}: invalid day checkpoint")
                days[tick // 1440] = values
            if kind == "clock":
                day = (tick + offset) // 1440 + 1
                minute = int(values["minute_of_day"])
                clocks.setdefault(day, {})[minute] = values
    if set(days) != set(range(1, duration + 1)) or previous_tick != duration * 1440:
        raise EvidenceError(f"{path}: incomplete day samples")
    if set(periodic) != set(range(0, duration * 1440 + 1, sample_every)):
        raise EvidenceError(f"{path}: incomplete periodic samples at interval {sample_every}")
    actionable = {}
    for day in days:
        alive = sum(v[0] for tick, v in periodic.items() if tick <= day * 1440)
        breaches = sum(v[1] for tick, v in periodic.items() if tick <= day * 1440)
        actionable[day] = breaches / alive if alive else 0.
    return days, minimum, clocks, actionable, producer

def load_round(path: Path, duration: int, sample_every: int = 120) -> RoundEvidence:
    if sample_every < 1:
        raise EvidenceError("sample interval must be positive")
    summary = mapping(document(path / "summary.json"))
    expected = 64 if duration == 10 else 32
    if duration not in (10, 60) or summary["schema_version"] != 3 or summary["scene"] != "game_v9":
        raise EvidenceError("expected schema 3 game_v9 10/60-day round")
    bounds = mapping(summary["seeds"])
    if integer(bounds["start"]) != 0 or integer(bounds["end_exclusive"]) != expected:
        raise EvidenceError(f"expected seed range 0..{expected}")
    run_rows = [mapping(row) for row in sequence(summary["runs"])]
    if sorted(integer(row["seed"]) for row in run_rows) != list(range(expected)):
        raise EvidenceError("missing/duplicate run seed")
    source_days = mapping(summary["days"])
    if set(source_days) != {str(day) for day in range(1, duration + 1)}:
        raise EvidenceError("summary day coverage mismatch")
    actionable = {day: {key: number(value) for key, value in mapping(mapping(source_days[str(day)])["idle_breach_actionable_share"]).items()} for day in range(1, duration + 1)}
    runs: list[SeedEvidence] = []
    revisions: set[str] = set()
    for row in sorted(run_rows, key=lambda item: integer(item["seed"])):
        seed = integer(row["seed"])
        days, minimum, clocks, sampled_actionable, revision = raw_seed(path / f"game_v9_s{seed}.jsonl", seed, (duration, sample_every))
        revisions.add(revision)
        maturation = {key: integer(value) for key, value in mapping(row["maturation"]).items()}
        if set(maturation) != {"children_observed", "maturations_observed", "child_deaths", "children_still_alive", "late_children"}:
            raise EvidenceError("invalid maturation evidence")
        if maturation["children_observed"] != sum(maturation[k] for k in ("maturations_observed", "child_deaths", "children_still_alive")):
            raise EvidenceError("inconsistent maturation accounting")
        runs.append(SeedEvidence(seed, days, minimum, clocks, integer(row["max_owned_housing_overflow"]), number(row["social_completed_per_agent_day"]), number(row["ms_per_agent_tick"]), maturation, sampled_actionable))
        if runs[-1].timing <= 0 or runs[-1].social_rate < 0:
            raise EvidenceError("invalid run timing/social rate")
    if len(revisions) != 1:
        raise EvidenceError("mixed producer revisions across seeds")
    for day in range(1, duration + 1):
        for key in runs[0].days[day]:
            if key in {"schema_version", "seed", "tick", "day"}:
                continue
            wanted = quantiles([run.days[day][key] for run in runs])
            actual = mapping(mapping(source_days[str(day)])[key])
            if set(actual) != set(wanted) or any(not math.isclose(number(actual[k]), v, rel_tol=1e-12, abs_tol=1e-12) for k, v in wanted.items()):
                raise EvidenceError(f"summary/raw disagreement day={day} key={key}")
        wanted_actionable = quantiles([run.actionable[day] for run in runs])
        if any(not math.isclose(actionable[day][key], value, rel_tol=1e-12, abs_tol=1e-12) for key, value in wanted_actionable.items()):
            raise EvidenceError("summary/raw actionable share disagreement")
        if set(actionable[day]) != {"min", "p10", "median", "p90", "max"} or not all(0 <= v <= 1 for v in actionable[day].values()):
            raise EvidenceError("invalid actionable-share distribution")
    return RoundEvidence(path, duration, tuple(runs), boolean(summary["determinism_checked"]), actionable, revisions.pop())
