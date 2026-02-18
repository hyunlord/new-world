"""
derive_composite_violation_stress.py
생성일: 2026-02-18
설계 의도: composite violation_stress 104개 자동 파생
학술 근거: Cognitive Dissonance Theory (Festinger, 1957)
  - composite는 facet의 조합이므로 위반 스트레스도 구성 facet에서 파생
게임 레퍼런스: CK3 stress system (수동 정의) → facet 기반 자동 파생으로 확장
대안으로 고려: 수동 정의 → 기각 (104개 수동 정의 시 facet과 일관성 보장 불가)

주의:
- dark tetrad (d_ prefix) violation_stress 절대 덮어쓰지 않음
- 이미 violation_stress 있는 composite 덮어쓰지 않음
- 원본 파일 보존, derived.json에만 저장
"""

from __future__ import annotations

import json
from collections import defaultdict
from copy import deepcopy
from pathlib import Path
from typing import Any

SOURCE_PATH = Path("data/personality/trait_definitions_fixed.json")
OUTPUT_PATH = Path("data/personality/trait_definitions_derived.json")

AXES = {"H", "E", "X", "A", "C", "O"}

AXIS_FACETS: dict[tuple[str, str], list[str]] = {
    ("H", "high"): ["f_sincere", "f_fair_minded", "f_frugal", "f_modest"],
    ("H", "low"): ["f_deceptive", "f_corrupt", "f_greedy", "f_self_important"],
    ("E", "high"): ["f_fearful", "f_anxious", "f_dependent", "f_sentimental"],
    ("E", "low"): ["f_fearless", "f_calm", "f_self_reliant", "f_tough_minded"],
    ("X", "high"): ["f_confident", "f_bold", "f_gregarious", "f_energetic"],
    ("X", "low"): ["f_insecure", "f_shy", "f_solitary", "f_reserved"],
    ("A", "high"): ["f_forgiving", "f_gentle", "f_flexible", "f_patient"],
    ("A", "low"): ["f_vengeful", "f_harsh", "f_stubborn", "f_hot_tempered"],
    ("C", "high"): ["f_organized", "f_industrious", "f_perfectionist", "f_prudent"],
    ("C", "low"): ["f_disorganized", "f_lazy", "f_careless", "f_reckless"],
    ("O", "high"): ["f_aesthetic", "f_curious", "f_creative", "f_nonconformist"],
    ("O", "low"): ["f_utilitarian", "f_apathetic", "f_conventional", "f_traditionalist"],
}

SUBFACET_FACETS: dict[tuple[str, str], list[str]] = {
    ("H_sincerity", "high"): ["f_sincere"],
    ("H_sincerity", "low"): ["f_deceptive"],
    ("H_fairness", "high"): ["f_fair_minded"],
    ("H_fairness", "low"): ["f_corrupt"],
    ("H_greed_avoidance", "high"): ["f_frugal"],
    ("H_greed_avoidance", "low"): ["f_greedy"],
    ("H_modesty", "high"): ["f_modest"],
    ("H_modesty", "low"): ["f_self_important"],
    ("E_fearfulness", "high"): ["f_fearful"],
    ("E_fearfulness", "low"): ["f_fearless"],
    ("E_anxiety", "high"): ["f_anxious"],
    ("E_anxiety", "low"): ["f_calm"],
    ("E_dependence", "high"): ["f_dependent"],
    ("E_dependence", "low"): ["f_self_reliant"],
    ("E_sentimentality", "high"): ["f_sentimental"],
    ("E_sentimentality", "low"): ["f_tough_minded"],
    ("X_social_self_esteem", "high"): ["f_confident"],
    ("X_social_self_esteem", "low"): ["f_insecure"],
    ("X_social_boldness", "high"): ["f_bold"],
    ("X_social_boldness", "low"): ["f_shy"],
    ("X_sociability", "high"): ["f_gregarious"],
    ("X_sociability", "low"): ["f_solitary"],
    ("X_liveliness", "high"): ["f_energetic"],
    ("X_liveliness", "low"): ["f_reserved"],
    ("A_forgiveness", "high"): ["f_forgiving"],
    ("A_forgiveness", "low"): ["f_vengeful"],
    ("A_gentleness", "high"): ["f_gentle"],
    ("A_gentleness", "low"): ["f_harsh"],
    ("A_flexibility", "high"): ["f_flexible"],
    ("A_flexibility", "low"): ["f_stubborn"],
    ("A_patience", "high"): ["f_patient"],
    ("A_patience", "low"): ["f_hot_tempered"],
    ("C_organization", "high"): ["f_organized"],
    ("C_organization", "low"): ["f_disorganized"],
    ("C_diligence", "high"): ["f_industrious"],
    ("C_diligence", "low"): ["f_lazy"],
    ("C_perfectionism", "high"): ["f_perfectionist"],
    ("C_perfectionism", "low"): ["f_careless"],
    ("C_prudence", "high"): ["f_prudent"],
    ("C_prudence", "low"): ["f_reckless"],
    ("O_aesthetic_appreciation", "high"): ["f_aesthetic"],
    ("O_aesthetic_appreciation", "low"): ["f_utilitarian"],
    ("O_inquisitiveness", "high"): ["f_curious"],
    ("O_inquisitiveness", "low"): ["f_apathetic"],
    ("O_creativity", "high"): ["f_creative"],
    ("O_creativity", "low"): ["f_conventional"],
    ("O_unconventionality", "high"): ["f_nonconformist"],
    ("O_unconventionality", "low"): ["f_traditionalist"],
}

TWO_AXIS_PREFIXES = (
    "c_he_",
    "c_hx_",
    "c_ha_",
    "c_hc_",
    "c_ho_",
    "c_ex_",
    "c_ea_",
    "c_ec_",
    "c_eo_",
    "c_xa_",
    "c_xc_",
    "c_xo_",
    "c_ac_",
    "c_ao_",
    "c_co_",
)


def has_violation_stress(trait: dict[str, Any]) -> bool:
    stress_modifiers = trait.get("effects", {}).get("stress_modifiers", {})
    return isinstance(stress_modifiers.get("violation_stress"), dict)


def resolve_facets_for_condition_item(item: dict[str, Any]) -> list[str]:
    facet = item.get("facet")
    direction = item.get("direction")
    if not isinstance(facet, str) or not isinstance(direction, str):
        return []

    key = (facet, direction)
    if key in AXIS_FACETS:
        return AXIS_FACETS[key]
    if key in SUBFACET_FACETS:
        return SUBFACET_FACETS[key]
    return []


def decay_factor_for_composite(composite: dict[str, Any]) -> float:
    comp_id = composite.get("id", "")
    if any(comp_id.startswith(prefix) for prefix in TWO_AXIS_PREFIXES):
        return 0.8

    all_conditions = composite.get("condition", {}).get("all", [])
    has_trait_reference = any("trait" in c for c in all_conditions)
    if has_trait_reference:
        return 0.6

    facet_tokens = [c.get("facet") for c in all_conditions if isinstance(c.get("facet"), str)]
    axis_only = len(facet_tokens) > 0 and all(token in AXES for token in facet_tokens)
    if axis_only and len(facet_tokens) >= 3:
        return 0.7

    return 0.6


def derive_violation_stress(
    composite: dict[str, Any],
    facet_violation_map: dict[str, dict[str, float]],
) -> dict[str, float]:
    factor = decay_factor_for_composite(composite)
    raw_sums: defaultdict[str, float] = defaultdict(float)

    for item in composite.get("condition", {}).get("all", []):
        for facet_id in resolve_facets_for_condition_item(item):
            violation = facet_violation_map.get(facet_id, {})
            for action, value in violation.items():
                raw_sums[action] += float(value)

    derived: dict[str, float] = {}
    for action, raw_value in raw_sums.items():
        value = round(raw_value * factor, 1)
        value = max(0.0, min(30.0, value))
        if value >= 1.0:
            derived[action] = value

    return derived


def main() -> None:
    with SOURCE_PATH.open("r", encoding="utf-8") as fp:
        traits: list[dict[str, Any]] = json.load(fp)

    facets = [t for t in traits if str(t.get("id", "")).startswith("f_")]
    composites = [t for t in traits if str(t.get("id", "")).startswith("c_")]
    dark_traits = [t for t in traits if str(t.get("id", "")).startswith("d_")]

    facet_violation_map: dict[str, dict[str, float]] = {}
    for facet in facets:
        violation = facet.get("effects", {}).get("stress_modifiers", {}).get("violation_stress")
        if isinstance(violation, dict):
            facet_violation_map[facet["id"]] = {k: float(v) for k, v in violation.items()}

    output_traits = deepcopy(traits)

    derived_count = 0
    skipped_already_set = 0
    skipped_dark_tetrad = len(dark_traits)
    sample_derived: list[tuple[str, dict[str, float]]] = []

    for trait in output_traits:
        trait_id = str(trait.get("id", ""))
        if not trait_id.startswith("c_"):
            continue

        if has_violation_stress(trait):
            skipped_already_set += 1
            continue

        derived = derive_violation_stress(trait, facet_violation_map)

        effects = trait.setdefault("effects", {})
        stress_modifiers = effects.setdefault("stress_modifiers", {})
        stress_modifiers["violation_stress"] = derived

        derived_count += 1
        if len(sample_derived) < 2:
            sample_derived.append((trait_id, derived))

    with OUTPUT_PATH.open("w", encoding="utf-8") as fp:
        json.dump(output_traits, fp, ensure_ascii=False, indent=2)
        fp.write("\n")

    all_values_in_range = True
    for trait in output_traits:
        violation = trait.get("effects", {}).get("stress_modifiers", {}).get("violation_stress")
        if not isinstance(violation, dict):
            continue
        for value in violation.values():
            v = float(value)
            if v < 0.0 or v > 30.0:
                all_values_in_range = False
                break
        if not all_values_in_range:
            break

    print(f"[DERIVE] Loaded {len(traits)} traits ({len(facets)} facets, {len(composites)} composites)")
    print(
        "[DERIVE] Derived violation_stress for "
        f"{derived_count} composites ({skipped_dark_tetrad} skipped: dark tetrad, "
        f"{skipped_already_set} skipped: already set)"
    )
    print("[DERIVE] Sample derived values:")
    if sample_derived:
        for trait_id, values in sample_derived:
            print(f"  {trait_id}: {json.dumps(values, ensure_ascii=False, sort_keys=True)}")
    else:
        print("  (none)")
    print(f"[DERIVE] Saved to {OUTPUT_PATH.as_posix()}")
    print(
        "[DERIVE] All derived values in range [0, 30]: "
        f"{'YES' if all_values_in_range else 'NO'}"
    )


if __name__ == "__main__":
    main()
