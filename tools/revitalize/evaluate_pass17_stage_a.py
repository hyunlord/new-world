from __future__ import annotations

from collections.abc import Callable, Mapping, Sequence
from dataclasses import replace
from decimal import Decimal
from statistics import fmean
from typing import Literal, TypeVar, assert_never

from evaluate_pass17_data import Cohort, Criterion, Hour, InputError, Metric, Person, StageMetrics, Verdict, World
from evaluate_pass17_statistics import Pair, UndefinedStatistic, bootstrap, pooled_ratio, pooled_spearman

RoutineMetric = Literal["p", "ns", "rd"]
T = TypeVar("T")


def routine_pairs(people: Sequence[Person], metric: RoutineMetric) -> list[Pair]:
    pairs: list[Pair] = []
    for person in people:
        if person.adult_ticks < 10 * 1440:
            continue
        total = sum(person.activity_minutes)
        match metric:
            case "p":
                if person.completed_activities >= 3:
                    pairs.append((float(person.temperament[3]), person.completed_minutes / person.completed_activities))
            case "ns":
                pairs.append((float(person.temperament[0]), person.activity_minutes[4] / total if total else 0.0))
            case "rd":
                pairs.append((float(person.temperament[2]), person.activity_minutes[3] / total if total else 0.0))
            case unreachable:
                assert_never(unreachable)
    return pairs


def idle_exposure(hours: Sequence[Hour]) -> Pair:
    daytime = [hour for hour in hours if 480 <= hour.minute <= 1020]
    return float(sum(hour.states[0][0] for hour in daytime)), float(sum(sum(hour.states[0]) for hour in daytime))


def relative_change(pairs: Sequence[Pair]) -> float:
    pooled_ratio(pairs)
    return float(Decimal(str(sum(a for a, _ in pairs))) / Decimal(str(sum(b for _, b in pairs))) - 1)


def mean_difference(pairs: Sequence[Pair]) -> float:
    return fmean(candidate - baseline for candidate, baseline in pairs)


def measure(worlds: Sequence[T], statistic: Callable[[Sequence[T]], float], criterion: Criterion) -> Metric:
    try:
        estimate = bootstrap(worlds, statistic)
    except UndefinedStatistic as error:
        return Metric(None, None, "ERROR", criterion.bar, criterion.unit, error=error.reason)
    passed = (criterion.lower is None or estimate.value >= criterion.lower) and (criterion.upper is None or estimate.value <= criterion.upper)
    if criterion.positive_interval:
        passed = passed and estimate.interval[0] > 0.0
    return Metric(estimate.value, estimate.interval, "PASS" if passed else "FAIL", criterion.bar, criterion.unit,
        bootstrap_valid=estimate.valid_draws, bootstrap_replicates=estimate.replicates)


def group_verdict(verdicts: Sequence[Verdict]) -> Verdict:
    severities: tuple[Verdict, ...] = ("ERROR", "FAIL", "PENDING")
    for severity in severities:
        if severity in verdicts:
            return severity
    return "PASS"


def social_exposure(world: World) -> Pair:
    return float(sum(person.social_completed for person in world.people)), sum(person.adult_ticks for person in world.people) / 1440.0


def social_ratio(pairs: Sequence[tuple[World, World]]) -> float:
    candidate = pooled_ratio([social_exposure(pair[0]) for pair in pairs])
    baseline = pooled_ratio([social_exposure(pair[1]) for pair in pairs])
    return pooled_ratio([(candidate, baseline)])


def health_values(world: World) -> Mapping[str, float]:
    noon = [hour for hour in world.hours if hour.minute == 720]
    return {"alive": float(world.alive_end), "deaths": float(world.non_old_age_deaths),
        "shelters": float(world.shelters_completed), "food": fmean(hour.food for hour in noon),
        "wood": fmean(hour.wood for hour in noon)}


def health_metrics(candidate: Cohort, baseline: Cohort) -> Mapping[str, Metric]:
    values = [(health_values(a), health_values(b)) for a, b in zip(candidate.worlds, baseline.worlds, strict=True)]
    output: dict[str, Metric] = {}
    requirements = {
        "alive": (relative_change, Criterion(-0.10, 0.10, "relative change within +/-0.10", "relative change")),
        "deaths": (mean_difference, Criterion(None, 1.5, "non-old-age deaths difference <= +1.5/world", "deaths/world")),
        "shelters": (pooled_ratio, Criterion(0.9, None, "completed shelters ratio >= 0.90", "ratio")),
        "food": (relative_change, Criterion(-0.15, 0.15, "noon food relative change within +/-0.15", "relative change")),
        "wood": (relative_change, Criterion(-0.15, 0.15, "noon wood relative change within +/-0.15", "relative change")),
    }
    for key, (statistic, criterion) in requirements.items():
        pairs = [(a[key], b[key]) for a, b in values]
        output[f"R2-{key}"] = replace(measure(pairs, statistic, criterion),
            candidate=fmean(a for a, _ in pairs), baseline=fmean(b for _, b in pairs),
            details={"definition": "final minus initial complete Shelter count, including daughter-camp boot shelters"} if key == "shelters" else None)
    pairs = list(zip(candidate.worlds, baseline.worlds, strict=True))
    metric = measure(pairs, social_ratio, Criterion(0.9, None, "pooled adult social rate ratio >= 0.90", "ratio"))
    try:
        candidate_rate = pooled_ratio([social_exposure(world) for world in candidate.worlds])
        baseline_rate = pooled_ratio([social_exposure(world) for world in baseline.worlds])
        candidate_sensitivity = [pooled_ratio([social_exposure(world)]) for world in candidate.worlds if social_exposure(world)[1] > 0]
        baseline_sensitivity = [pooled_ratio([social_exposure(world)]) for world in baseline.worlds if social_exposure(world)[1] > 0]
        metric = replace(metric, candidate=candidate_rate, baseline=baseline_rate,
            details={"candidate_mean_world_rate": fmean(candidate_sensitivity),
                "baseline_mean_world_rate": fmean(baseline_sensitivity),
                "candidate_sensitivity_worlds": len(candidate_sensitivity), "baseline_sensitivity_worlds": len(baseline_sensitivity)})
    except UndefinedStatistic as error:
        metric = replace(metric, verdict="ERROR", error=error.reason)
    output["R2-social"] = metric
    return output


def evaluate(candidate: Cohort, baseline: Cohort) -> StageMetrics:
    if candidate.days != baseline.days or [w.seed for w in candidate.worlds] != [w.seed for w in baseline.worlds]:
        raise InputError("paired cohorts", "seed sets or duration differ")
    exposures = [idle_exposure(world.hours) for world in candidate.worlds]
    metrics: dict[str, Metric] = {"R1": measure(exposures, pooled_ratio, Criterion(0.10, 0.45, "0.10 <= pooled adult Idle share <= 0.45", "share"))}
    baseline_idle = measure([idle_exposure(world.hours) for world in baseline.worlds], pooled_ratio, Criterion(None, None, "reference only", "share"))
    metrics["R1"] = replace(metrics["R1"], candidate=metrics["R1"].value, baseline=baseline_idle.value)
    metrics["R1-baseline"] = baseline_idle
    metrics.update(health_metrics(candidate, baseline))
    routine_requirements: tuple[tuple[RoutineMetric, str, float], ...] = (("p", "P", 0.25), ("ns", "NS", 0.25), ("rd", "RD", 0.20))
    for metric, label, bar in routine_requirements:
        payloads = [routine_pairs(world.people, metric) for world in candidate.worlds]
        result = measure(payloads, pooled_spearman, Criterion(bar, None, f"rho >= {bar:.2f}; CI excludes zero", "Spearman rho", True))
        eligible = [person for world in candidate.worlds for person in world.people if person.adult_ticks >= 14400]
        metrics[f"R3-{label}"] = replace(result, details={"eligible_people": sum(len(p) for p in payloads),
            "adult_10day_people": len(eligible), "zero_activity_people": sum(sum(p.activity_minutes) == 0 for p in eligible),
            "no_place_redraws": sum(world.no_place_redraws for world in candidate.worlds)})
    verdicts: dict[str, Verdict] = {key: metrics[key].verdict for key in ("R1", "R3-P", "R3-NS", "R3-RD")}
    verdicts["R2"] = group_verdict([value.verdict for key, value in metrics.items() if key.startswith("R2-")])
    return StageMetrics(metrics, verdicts)
