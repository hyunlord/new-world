from __future__ import annotations

from collections.abc import Sequence

from evaluate_pass17_data import Cohort
from evaluate_pass17_evidence import Evidence, digest
from evaluate_pass17_family_data import FamilyPair, FamilyPairs, FamilyWorld
from evaluate_pass17_read import JsonValue
from evaluate_pass17_report import JsonObject, cohort_json, evidence_json, metric_json
from evaluate_pass17_stage_a import group_verdict
from evaluate_pass17_stage_b import FamilyEvaluation, bed_exposure, birth_exposure, births_by_day, living_couples, partner_exposure, split_exposure


def pair_json(pair: FamilyPair) -> JsonObject:
    return {"ids": [pair.first.agent_id, pair.second.agent_id], "tick": pair.tick,
        "settlement": pair.settlement_id, "stages": list(pair.stages) if pair.stages is not None else None}


def world_json(world: FamilyWorld, pairs: FamilyPairs) -> JsonObject:
    nights = [snapshot for snapshot in world.snapshots if snapshot.minute == 1200]
    partnered, adults = partner_exposure(world.day60) if world.day60 is not None else (None, None)
    couple_births, births = birth_exposure(world)
    shared, bedded = bed_exposure(world)
    split, couples = split_exposure(world)
    return {"seed": world.seed, "day60_partnered_adults": partnered, "day60_adults": adults,
        "births_from_day20": births, "couple_births_from_day20": couple_births,
        "all_births": len(world.births), "all_fallback_births": sum(b.fallback for b in world.births),
        "same_shelter_couple_nights": shared, "bedded_couple_nights": bedded,
        "all_living_couple_nights": sum(len(living_couples(snapshot)) for snapshot in nights),
        "bedless_person_nights": sum(member.shelter is None for snapshot in nights for member in snapshot.members),
        "completed_fissions": sum(f.status == "completed" for f in world.fissions),
        "cancelled_fissions": sum(f.status == "cancelled" for f in world.fissions),
        "pending_fissions": sum(f.status == "pending" for f in world.fissions),
        "departure_couples": couples, "split_couples": split,
        "completed_departure_households_kept": sum(f.households_kept for f in world.fissions if f.status == "completed"),
        "completed_departure_households_split": sum(f.households_split for f in world.fissions if f.status == "completed"),
        "sibling_unordered_pairs": len(pairs.siblings), "unrelated_unordered_pairs": len(pairs.unrelated),
        "excluded_close_kin_pairs": pairs.excluded_kin,
        "stage_pair_categories": [list(pair) for pair in pairs.categories],
        "f3_unassigned_person_hours": sum(member.settlement_id is None for snapshot in world.snapshots for member in snapshot.members),
        "f3_unassigned_people": len({member.agent_id for snapshot in world.snapshots for member in snapshot.members if member.settlement_id is None}),
        "pair_eligibility": {"siblings": [pair_json(pair) for pair in pairs.siblings],
            "unrelated": [pair_json(pair) for pair in pairs.unrelated]}}


def build_family_report(candidate: Cohort, baseline: Cohort, measured: FamilyEvaluation,
                        families: Sequence[FamilyWorld], identity: Evidence, performance: Evidence, tuning: bool) -> JsonObject:
    rules = dict(measured.stage.verdicts)
    rules.update({"F7": identity.verdict, "F8": performance.verdict})
    files = [candidate.source / f"{candidate.scene}_s{world.seed}_family_{suffix}" for world in families
        for suffix in ("people.jsonl", "hours.jsonl", "events.jsonl", "fissions.jsonl", "meta.json")]
    return {"schema_version": 17, "stage": "b", "mode": "tuning" if tuning else "evaluation",
        "verdict": group_verdict(list(rules.values())), "rules": dict(sorted(rules.items())),
        "metrics": {key: metric_json(metric) for key, metric in sorted(measured.stage.metrics.items())},
        "candidate": cohort_json(candidate), "baseline": cohort_json(baseline),
        "identity": evidence_json(identity), "performance": evidence_json(performance),
        "family": {"births_by_day": births_by_day(families, candidate.days),
            "worlds": [world_json(world, pairs) for world, pairs in zip(families, measured.pairs, strict=True)],
            "inputs_sha256": {str(path): digest(path) for path in files}},
        "methods": {
            "bootstrap": "2,000 whole-world resamples; fresh numpy.default_rng(0) per statistic; percentile 95% CI; undefined draws excluded with valid count; F6 keeps seed pairs together",
            "F1": "Pooled living nonelder adults at exact elapsed day60 (after tick86399); partner must be alive and mutual; dead partner pointers do not count",
            "F2": "Pooled births with fallback=false from one-based elapsed simulation day20; simulation_day=tick//1440+1, tick>=27360; display-clock offset does not change inclusion; all daily births/fallbacks retained",
            "F3": "Pooled Pearson for each temperament axis, then mean four axes for siblings minus unrelated; each unique unordered pair contributes both orientations; bar>=0.06 and lower95%CI>0; exact per-world integer sufficient statistics preserve whole-world bootstrap",
            "F3_eligibility": "Full siblings are all recorded people with both known matching ordered mother/father, including fallback births, independent of residence or coexistence. Their first shared hourly snapshot in a non-null settlement supplies reference tick/settlement/stages when present; otherwise all three are null, without excluding the pair. Unrelated controls exclude direct parent-child or any shared known parent across either slot, and use their first shared hourly snapshot in the same non-null settlement regardless of life-stage categories. Sibling stage-pair categories describe observed shared context only and never filter either population. Unassigned settlement=null observations cannot form unrelated same-settlement pairs; their counts are reported. Unknown founder kinship is not inferred. Empty worlds remain in resampling; pair IDs/ticks/stages reported",
            "F4": "20:00 snapshots; each mutual living couple counted once if both have beds; same bed tile means shared shelter; bedless person-nights separately reported",
            "F5": "Only completed fissions; pooled split couples / mutual living couples in parent settlements at departure; raw membership and household partitions verify numerator; zero denominator ERROR",
            "F6": "Identical R2 definitions: equal-world alive/death/shelter means, noon settlement-summed stores averaged within worlds then across worlds; primary social rate pools adult completions/adult-days, mean world rate sensitivity",
            "shelters": "Final minus initial complete Shelter count, including daughter-camp boot shelters",
            "errors": "Missing/malformed observations or proof ERROR; undefined fractions/correlations ERROR, never artificial zero; measured bar misses FAIL with numeric estimates; absent F7/F8 tuning-only PENDING",
            "evidence": "F7 schema1 stageb requires I1-I5 plus v19/v20 pins and verifies every file hash; F8 schema1 stageb re-verifies complete raw performance evidence with unchanged timing bars"}}
