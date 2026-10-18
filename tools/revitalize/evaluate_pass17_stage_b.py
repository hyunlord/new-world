from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass, replace

from evaluate_pass17_data import AXES, Cohort, Criterion, InputError, Metric, StageMetrics, Verdict
from evaluate_pass17_family_data import FamilyMember, FamilyPairs, FamilySnapshot, FamilyWorld
from evaluate_pass17_family_pairs import axis_correlation, collect_pairs, pair_moments, resemblance_difference, sibling_mean, unrelated_mean
from evaluate_pass17_read import JsonValue
from evaluate_pass17_stage_a import group_verdict, health_metrics, measure
from evaluate_pass17_statistics import Pair, pooled_ratio


@dataclass(frozen=True, slots=True)
class FamilyEvaluation:
    stage: StageMetrics
    pairs: tuple[FamilyPairs, ...]


def living_couples(snapshot: FamilySnapshot) -> tuple[tuple[FamilyMember, FamilyMember], ...]:
    members = {member.agent_id: member for member in snapshot.members}
    pairs = []
    for member in snapshot.members:
        if member.partner is not None and member.agent_id < member.partner and member.partner in members:
            partner = members[member.partner]
            if partner.partner == member.agent_id:
                pairs.append((member, partner))
    return tuple(pairs)


def partner_exposure(snapshot: FamilySnapshot) -> Pair:
    partnered = {member.agent_id for couple in living_couples(snapshot) for member in couple}
    adults = [member for member in snapshot.members if member.stage == "adult"]
    return float(sum(member.agent_id in partnered for member in adults)), float(len(adults))


def birth_exposure(world: FamilyWorld) -> Pair:
    births = [birth for birth in world.births if birth.day >= 20]
    return float(sum(not birth.fallback for birth in births)), float(len(births))


def bed_exposure(world: FamilyWorld) -> Pair:
    pairs = [couple for snapshot in world.snapshots if snapshot.minute == 1200
        for couple in living_couples(snapshot) if all(member.shelter is not None for member in couple)]
    return float(sum(a.shelter == b.shelter for a, b in pairs)), float(len(pairs))


def split_exposure(world: FamilyWorld) -> Pair:
    completed = [fission for fission in world.fissions if fission.status == "completed"]
    return float(sum(fission.split_couples for fission in completed)), float(sum(fission.couples for fission in completed))


def births_by_day(worlds: Sequence[FamilyWorld], days: int) -> list[JsonValue]:
    output: list[JsonValue] = []
    for day in range(1, days + 1):
        births = [birth for world in worlds for birth in world.births if birth.day == day]
        fallbacks = sum(birth.fallback for birth in births)
        output.append({"day": day, "births": len(births), "couple_births": len(births) - fallbacks, "fallback_births": fallbacks})
    return output


def fraction_metric(exposures: Sequence[Pair], criterion: Criterion) -> Metric:
    return replace(measure(exposures, pooled_ratio, criterion), details={
        "numerator": sum(a for a, _ in exposures), "denominator": sum(b for _, b in exposures),
        "worlds": len(exposures), "zero_denominator_worlds": sum(b == 0 for _, b in exposures)})


def evaluate(candidate: Cohort, baseline: Cohort, families: Sequence[FamilyWorld]) -> FamilyEvaluation:
    seeds = [world.seed for world in candidate.worlds]
    if candidate.days != baseline.days or seeds != [world.seed for world in baseline.worlds] or seeds != [world.seed for world in families]:
        raise InputError("paired cohorts", "family/candidate/baseline seed sets or durations differ")
    day60 = [world.day60 for world in families if world.day60 is not None]
    metrics = {"F1": fraction_metric([partner_exposure(snapshot) for snapshot in day60],
            Criterion(0.50, None, "pooled living mutually partnered adult share at elapsed day60 >= 0.50", "share")),
        "F2": fraction_metric([birth_exposure(world) for world in families],
            Criterion(0.90, None, "couple-parent births from elapsed simulation day20 inclusive >= 0.90", "share")),
        "F4": fraction_metric([bed_exposure(world) for world in families],
            Criterion(0.90, None, "same-shelter share of living mutual bedded couple-nights at20:00 >= 0.90", "share")),
        "F5": fraction_metric([split_exposure(world) for world in families],
            Criterion(None, 0.10, "split couples / parent couples at completed fission departures <= 0.10", "share"))}
    if len(day60) != len(families):
        metrics["F1"] = replace(metrics["F1"], value=None, ci95=None, verdict="ERROR",
            error="one or more worlds lack the exact elapsed day60 snapshot")
    pairs = tuple(collect_pairs(world) for world in families)
    moments = [pair_moments(world) for world in pairs]
    resemblance = measure(moments, resemblance_difference,
        Criterion(0.06, None, "mean four-axis sibling Pearson minus unrelated Pearson >= 0.06; lower CI > 0", "Pearson difference", True))
    metrics["F3"] = replace(resemblance, details={"sibling_unordered_pairs": sum(len(w.siblings) for w in pairs),
        "unrelated_unordered_pairs": sum(len(w.unrelated) for w in pairs),
        "sibling_worlds": sum(bool(w.siblings) for w in pairs), "unrelated_worlds": sum(bool(w.unrelated) for w in pairs),
        "zero_sibling_worlds": sum(not w.siblings for w in pairs), "zero_unrelated_worlds": sum(not w.unrelated for w in pairs),
        "excluded_close_kin_pairs": sum(w.excluded_kin for w in pairs), "orientations_per_pair": 2})
    reference = Criterion(None, None, "descriptive reference", "Pearson correlation")
    metrics["F3-siblings"] = measure(moments, sibling_mean, reference)
    metrics["F3-unrelated"] = measure(moments, unrelated_mean, reference)
    for axis, label in enumerate(AXES):
        metrics[f"F3-siblings-{label}"] = measure([world.siblings[axis] for world in moments], axis_correlation, reference)
        metrics[f"F3-unrelated-{label}"] = measure([world.unrelated[axis] for world in moments], axis_correlation, reference)
    metrics.update({key.replace("R2-", "F6-", 1): metric for key, metric in health_metrics(candidate, baseline).items()})
    verdicts: dict[str, Verdict] = {key: metrics[key].verdict for key in ("F1", "F2", "F3", "F4", "F5")}
    verdicts["F6"] = group_verdict([metric.verdict for key, metric in metrics.items() if key.startswith("F6-")])
    return FamilyEvaluation(StageMetrics(metrics, verdicts), pairs)
