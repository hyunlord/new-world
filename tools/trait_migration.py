#!/usr/bin/env python3
"""Trait migration script for 2-level hybrid trait model outputs."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

SOURCE_PATH = Path("data/personality/trait_definitions_fixed.json")
TRAIT_V2_PATH = Path("data/personality/trait_defs_v2.json")
BEHAVIOR_PATH = Path("data/personality/behavior_mappings.json")
EMOTION_PATH = Path("data/personality/emotion_mappings.json")
VIOLATION_PATH = Path("data/personality/violation_mappings.json")
KO_TRAITS_PATH = Path("data/locales/ko/traits.json")
EN_TRAITS_PATH = Path("data/locales/en/traits.json")
KO_EVENTS_PATH = Path("data/locales/ko/traits_events.json")
EN_EVENTS_PATH = Path("data/locales/en/traits_events.json")

VIOLATION_ALPHA = 1.2

MUTEX_PAIRS = {
    "A_flexibility": ("f_flexible", "f_stubborn"),
    "A_forgiveness": ("f_forgiving", "f_vengeful"),
    "A_gentleness": ("f_gentle", "f_harsh"),
    "A_patience": ("f_patient", "f_hot_tempered"),
    "C_diligence": ("f_industrious", "f_lazy"),
    "C_organization": ("f_organized", "f_disorganized"),
    "C_perfectionism": ("f_perfectionist", "f_careless"),
    "C_prudence": ("f_prudent", "f_reckless"),
    "E_anxiety": ("f_anxious", "f_calm"),
    "E_dependence": ("f_dependent", "f_self_reliant"),
    "E_fearfulness": ("f_fearful", "f_fearless"),
    "E_sentimentality": ("f_sentimental", "f_tough_minded"),
    "H_fairness": ("f_fair_minded", "f_corrupt"),
    "H_greed_avoidance": ("f_frugal", "f_greedy"),
    "H_modesty": ("f_modest", "f_self_important"),
    "H_sincerity": ("f_sincere", "f_deceptive"),
    "O_aesthetic": ("f_aesthetic", "f_utilitarian"),
    "O_creativity": ("f_creative", "f_conventional"),
    "O_inquisitiveness": ("f_curious", "f_apathetic"),
    "O_unconventionality": ("f_nonconformist", "f_traditionalist"),
    "X_liveliness": ("f_energetic", "f_reserved"),
    "X_sociability": ("f_gregarious", "f_solitary"),
    "X_social_boldness": ("f_bold", "f_shy"),
    "X_social_self_esteem": ("f_confident", "f_insecure"),
}

MUTEX_BY_TRAIT_ID: dict[str, str] = {}
for facet_key, (trait_a, trait_b) in MUTEX_PAIRS.items():
    MUTEX_BY_TRAIT_ID[trait_a] = facet_key
    MUTEX_BY_TRAIT_ID[trait_b] = facet_key

EVENT_LOCALE_KO = {
    "CHRONICLE_TRAIT_DISPLAYED": "{name}의 두드러진 성격: {trait}",
    "CHRONICLE_TRAIT_STRENGTHENED": "{name}의 '{trait}' 성향이 더욱 강해졌다.",
    "CHRONICLE_TRAIT_WEAKENED": "{name}의 '{trait}' 성향이 옅어졌다.",
    "CHRONICLE_TRAIT_ARCHETYPE": "{name}은(는) 진정한 {trait}(으)로 불릴 만하다.",
    "UI_TRAIT_SALIENCE_BAR": "{name}  {bar} ({pct}%)",
    "UI_TRAIT_NO_DOMINANT": "두드러진 성격 없음",
}

EVENT_LOCALE_EN = {
    "CHRONICLE_TRAIT_DISPLAYED": "{name}'s dominant trait: {trait}",
    "CHRONICLE_TRAIT_STRENGTHENED": "{name}'s '{trait}' tendency has grown stronger.",
    "CHRONICLE_TRAIT_WEAKENED": "{name}'s '{trait}' tendency has faded.",
    "CHRONICLE_TRAIT_ARCHETYPE": "{name} can truly be called a {trait}.",
    "UI_TRAIT_SALIENCE_BAR": "{name}  {bar} ({pct}%)",
    "UI_TRAIT_NO_DOMINANT": "No dominant traits",
}


def clamp(value: float, lo: float, hi: float) -> float:
    return max(lo, min(hi, value))


def r3(value: float) -> float:
    return round(value + 1e-12, 3)


def load_traits(path: Path) -> list[dict[str, Any]]:
    with path.open("r", encoding="utf-8") as fp:
        data = json.load(fp)
    if not isinstance(data, list):
        raise ValueError("Trait source JSON must be a list")
    return data


def classify_trait(trait: dict[str, Any]) -> str:
    condition = trait.get("condition", {})
    if isinstance(condition, dict) and isinstance(condition.get("all"), list):
        return "composite"
    facet = condition.get("facet") if isinstance(condition, dict) else None
    if isinstance(facet, str) and "_" in facet:
        return "facet"
    return "unknown"


def transform_facet_trait(trait: dict[str, Any]) -> dict[str, Any]:
    condition = trait["condition"]
    facet = condition["facet"]
    direction = condition["direction"]
    threshold = float(condition["threshold"])
    mutex_group = MUTEX_BY_TRAIT_ID.get(trait["id"], facet)

    if direction == "high":
        t_on = threshold - 0.02
        t_off = threshold - 0.08
        sigmoid_s = clamp(0.012 + 0.25 * (1.0 - threshold), 0.015, 0.05)
        salience_center = threshold - 0.05
    elif direction == "low":
        t_on = threshold + 0.02
        t_off = threshold + 0.08
        sigmoid_s = clamp(0.012 + 0.25 * threshold, 0.015, 0.05)
        salience_center = threshold + 0.05
    else:
        raise ValueError(f"Unsupported direction for facet trait {trait['id']}: {direction}")

    return {
        "id": trait["id"],
        "name_key": f"TRAIT_{trait['id']}_NAME",
        "desc_key": f"TRAIT_{trait['id']}_DESC",
        "valence": trait.get("valence", "neutral"),
        "category": "facet",
        "facet": facet,
        "direction": direction,
        "threshold": r3(threshold),
        "t_on": r3(t_on),
        "t_off": r3(t_off),
        "sigmoid_s": r3(sigmoid_s),
        "salience_center": r3(salience_center),
        "salience_width": 0.12,
        "mutex_group": mutex_group,
        "axis": facet.split("_", 1)[0],
    }


def transform_composite_trait(trait: dict[str, Any]) -> dict[str, Any]:
    condition_items = trait["condition"]["all"]
    transformed_conditions: list[dict[str, Any]] = []

    for item in condition_items:
        facet = item.get("facet")
        direction = item.get("direction")
        threshold_raw = item.get("threshold")
        if not isinstance(facet, str) or direction not in ("high", "low") or threshold_raw is None:
            raise ValueError(f"Invalid composite condition in {trait['id']}: {item}")

        threshold = float(threshold_raw)
        if direction == "high":
            cond_center = threshold - 0.10
        else:
            cond_center = threshold + 0.10

        transformed_conditions.append(
            {
                "facet": facet,
                "direction": direction,
                "threshold": r3(threshold),
                "cond_center": r3(cond_center),
                "cond_width": 0.20,
            }
        )

    n = len(transformed_conditions)
    rarity_bonus = 1.0 + 0.1 * max(0, n - 2)

    is_dark = str(trait["id"]).startswith("d_")
    out: dict[str, Any] = {
        "id": trait["id"],
        "name_key": f"TRAIT_{trait['id']}_NAME",
        "desc_key": f"TRAIT_{trait['id']}_DESC",
        "valence": trait.get("valence", "neutral"),
        "category": "dark" if is_dark else "composite",
        "conditions": transformed_conditions,
        "rarity_bonus": r3(rarity_bonus),
    }
    if is_dark:
        out["violation_override"] = True

    return out


def build_trait_defs_v2(old_traits: list[dict[str, Any]]) -> list[dict[str, Any]]:
    new_traits: list[dict[str, Any]] = []
    for trait in old_traits:
        kind = classify_trait(trait)
        if kind == "facet":
            new_traits.append(transform_facet_trait(trait))
        elif kind == "composite":
            new_traits.append(transform_composite_trait(trait))
        else:
            raise ValueError(f"Unsupported trait condition shape for {trait.get('id')}")
    return new_traits


def build_behavior_map(old_traits: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    action_map: dict[str, list[dict[str, Any]]] = {}

    for trait in old_traits:
        behavior_weights = trait.get("effects", {}).get("behavior_weights", {})
        if not isinstance(behavior_weights, dict):
            continue

        kind = classify_trait(trait)
        for action, value in behavior_weights.items():
            if not isinstance(action, str):
                continue

            entry: dict[str, Any] = {
                "trait_id": trait["id"],
                "extreme_val": r3(float(value)),
            }

            if kind == "facet":
                cond = trait["condition"]
                entry.update(
                    {
                        "source": "facet",
                        "facet": cond["facet"],
                        "direction": cond["direction"],
                        "threshold": r3(float(cond["threshold"])),
                    }
                )
            else:
                entry["source"] = "composite"

            action_map.setdefault(action, []).append(entry)

    ordered: dict[str, list[dict[str, Any]]] = {}
    for action in sorted(action_map.keys()):
        ordered[action] = sorted(action_map[action], key=lambda item: item["trait_id"])
    return ordered


def parse_emotion_key(key: str) -> tuple[str, str]:
    stem, _, suffix = key.rpartition("_")
    if suffix in {"sensitivity", "baseline", "mult"} and stem:
        return stem, suffix
    return key, "mult"


def build_emotion_map(old_traits: list[dict[str, Any]]) -> dict[str, dict[str, list[dict[str, Any]]]]:
    out: dict[str, dict[str, list[dict[str, Any]]]] = {
        "sensitivity": {},
        "baseline": {},
        "mult": {},
    }

    for trait in old_traits:
        emotion_modifiers = trait.get("effects", {}).get("emotion_modifiers", {})
        if not isinstance(emotion_modifiers, dict):
            continue

        kind = classify_trait(trait)
        for raw_key, value in emotion_modifiers.items():
            if not isinstance(raw_key, str):
                continue

            metric, modifier_type = parse_emotion_key(raw_key)
            entry: dict[str, Any] = {
                "trait_id": trait["id"],
                "extreme_mult": r3(float(value)),
            }
            if kind == "facet":
                cond = trait["condition"]
                entry["facet"] = cond["facet"]
                entry["direction"] = cond["direction"]

            bucket = out[modifier_type].setdefault(metric, [])
            bucket.append(entry)

    for modifier_type in out:
        ordered_metrics: dict[str, list[dict[str, Any]]] = {}
        for metric in sorted(out[modifier_type].keys()):
            ordered_metrics[metric] = sorted(out[modifier_type][metric], key=lambda item: item["trait_id"])
        out[modifier_type] = ordered_metrics

    return out


def build_violation_map(old_traits: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    action_map: dict[str, list[dict[str, Any]]] = {}

    for trait in old_traits:
        stress_modifiers = trait.get("effects", {}).get("stress_modifiers", {})
        if not isinstance(stress_modifiers, dict):
            continue
        violation_stress = stress_modifiers.get("violation_stress", {})
        if not isinstance(violation_stress, dict):
            continue

        for action, value in violation_stress.items():
            if not isinstance(action, str):
                continue
            entry = {
                "trait_id": trait["id"],
                "base_stress": r3(float(value)),
                "alpha": VIOLATION_ALPHA,
            }
            action_map.setdefault(action, []).append(entry)

    ordered: dict[str, list[dict[str, Any]]] = {}
    for action in sorted(action_map.keys()):
        ordered[action] = sorted(action_map[action], key=lambda item: item["trait_id"])
    return ordered


def build_trait_locales(old_traits: list[dict[str, Any]]) -> tuple[dict[str, str], dict[str, str]]:
    ko: dict[str, str] = {}
    en: dict[str, str] = {}

    def safe_str(value: Any) -> str:
        if value is None:
            return ""
        return str(value)

    for trait in old_traits:
        trait_id = trait["id"]
        ko[f"TRAIT_{trait_id}_NAME"] = safe_str(trait.get("name_kr", ""))
        ko[f"TRAIT_{trait_id}_DESC"] = safe_str(trait.get("description_kr", ""))
        en[f"TRAIT_{trait_id}_NAME"] = safe_str(trait.get("name_en", ""))
        en[f"TRAIT_{trait_id}_DESC"] = safe_str(trait.get("description_en", ""))

    return ko, en


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fp:
        json.dump(data, fp, ensure_ascii=False, indent=2)
        fp.write("\n")


def find_new_trait(new_traits: list[dict[str, Any]], trait_id: str) -> dict[str, Any] | None:
    for trait in new_traits:
        if trait.get("id") == trait_id:
            return trait
    return None


def has_violation_entry(
    violation_map: dict[str, list[dict[str, Any]]], action: str, trait_id: str, base_stress: float
) -> bool:
    for item in violation_map.get(action, []):
        if item.get("trait_id") == trait_id and abs(float(item.get("base_stress", -9999.0)) - base_stress) <= 0.001:
            return True
    return False


def validate_migration(
    old_traits: list[dict[str, Any]],
    new_traits: list[dict[str, Any]],
    behavior_map: dict[str, list[dict[str, Any]]],
    emotion_map: dict[str, dict[str, list[dict[str, Any]]]],
    violation_map: dict[str, list[dict[str, Any]]],
    ko_locale: dict[str, str],
    en_locale: dict[str, str],
) -> list[str]:
    errors: list[str] = []

    if len(new_traits) != len(old_traits):
        errors.append(f"Count mismatch: {len(old_traits)} -> {len(new_traits)}")

    # violation_stress=0 보존
    zero_pairs: list[tuple[str, str]] = []
    for trait in old_traits:
        violation = trait.get("effects", {}).get("stress_modifiers", {}).get("violation_stress", {})
        if not isinstance(violation, dict):
            continue
        for action, value in violation.items():
            if abs(float(value)) <= 0.000001:
                zero_pairs.append((trait["id"], action))

    for trait_id, action in zero_pairs:
        if not has_violation_entry(violation_map, action, trait_id, 0.0):
            errors.append(f"Zero violation stress not preserved: {trait_id} / {action}")

    # mutex_group 24쌍 설정
    for facet_key, (trait_a, trait_b) in MUTEX_PAIRS.items():
        for trait_id in (trait_a, trait_b):
            migrated = find_new_trait(new_traits, trait_id)
            if migrated is None:
                errors.append(f"Missing mutex trait: {trait_id}")
                continue
            if migrated.get("category") != "facet":
                errors.append(f"Trait should be facet category: {trait_id}")
            if migrated.get("mutex_group") != facet_key:
                errors.append(
                    f"Wrong mutex_group for {trait_id}: {migrated.get('mutex_group')} (expected {facet_key})"
                )

    # f_sincere 기준값 검증
    sincere = find_new_trait(new_traits, "f_sincere")
    if sincere is None:
        errors.append("Missing f_sincere")
    else:
        if abs(float(sincere.get("t_on", -1.0)) - 0.90) > 0.001:
            errors.append(f"f_sincere t_on mismatch: {sincere.get('t_on')}")
        if abs(float(sincere.get("t_off", -1.0)) - 0.84) > 0.001:
            errors.append(f"f_sincere t_off mismatch: {sincere.get('t_off')}")
        if abs(float(sincere.get("salience_center", -1.0)) - 0.87) > 0.001:
            errors.append(f"f_sincere salience_center mismatch: {sincere.get('salience_center')}")

    deceptive = find_new_trait(new_traits, "f_deceptive")
    if deceptive is None:
        errors.append("Missing f_deceptive")
    else:
        if abs(float(deceptive.get("t_on", -1.0)) - 0.16) > 0.001:
            errors.append(f"f_deceptive t_on mismatch: {deceptive.get('t_on')}")
        if abs(float(deceptive.get("t_off", -1.0)) - 0.22) > 0.001:
            errors.append(f"f_deceptive t_off mismatch: {deceptive.get('t_off')}")

    # rarity_bonus n=3 -> 1.1
    found_three_condition = False
    for trait in new_traits:
        if trait.get("category") in {"composite", "dark"} and len(trait.get("conditions", [])) == 3:
            found_three_condition = True
            if abs(float(trait.get("rarity_bonus", -1.0)) - 1.1) > 0.001:
                errors.append(f"rarity_bonus mismatch for 3-condition trait {trait.get('id')}")
                break
    if not found_three_condition:
        errors.append("No 3-condition composite trait found to validate rarity_bonus=1.1")

    psych = find_new_trait(new_traits, "d_psychopath_primary")
    if psych is None:
        errors.append("Missing d_psychopath_primary")
    else:
        if psych.get("category") != "dark":
            errors.append("d_psychopath_primary category should be dark")
        if psych.get("violation_override") is not True:
            errors.append("d_psychopath_primary violation_override should be true")
        if abs(float(psych.get("rarity_bonus", -1.0)) - 1.2) > 0.001:
            errors.append(f"d_psychopath_primary rarity_bonus mismatch: {psych.get('rarity_bonus')}")

    if not has_violation_entry(violation_map, "lie", "f_sincere", 14.0):
        errors.append("violation_mappings missing lie -> f_sincere(14)")

    if not has_violation_entry(violation_map, "harm_innocent", "d_psychopath_primary", 0.0):
        errors.append("violation_mappings missing harm_innocent -> d_psychopath_primary(0)")

    if len(ko_locale) != 374:
        errors.append(f"ko locale key count mismatch: {len(ko_locale)}")
    if len(en_locale) != 374:
        errors.append(f"en locale key count mismatch: {len(en_locale)}")

    for trait in new_traits:
        if "name_kr" in trait or "name_en" in trait or "description_kr" in trait or "description_en" in trait:
            errors.append(f"forbidden localized field exists in trait_defs_v2: {trait.get('id')}")
            break

    if not behavior_map:
        errors.append("behavior_mappings is empty")
    if not emotion_map.get("sensitivity") and not emotion_map.get("baseline") and not emotion_map.get("mult"):
        errors.append("emotion_mappings is empty")

    return errors


def main() -> int:
    old_traits = load_traits(SOURCE_PATH)

    new_traits = build_trait_defs_v2(old_traits)
    behavior_map = build_behavior_map(old_traits)
    emotion_map = build_emotion_map(old_traits)
    violation_map = build_violation_map(old_traits)
    ko_locale, en_locale = build_trait_locales(old_traits)

    errors = validate_migration(
        old_traits,
        new_traits,
        behavior_map,
        emotion_map,
        violation_map,
        ko_locale,
        en_locale,
    )
    if errors:
        print("[FAIL] Migration validation failed:")
        for err in errors:
            print(f" - {err}")
        return 1

    write_json(TRAIT_V2_PATH, new_traits)
    write_json(BEHAVIOR_PATH, behavior_map)
    write_json(EMOTION_PATH, emotion_map)
    write_json(VIOLATION_PATH, violation_map)
    write_json(KO_TRAITS_PATH, ko_locale)
    write_json(EN_TRAITS_PATH, en_locale)
    write_json(KO_EVENTS_PATH, EVENT_LOCALE_KO)
    write_json(EN_EVENTS_PATH, EVENT_LOCALE_EN)

    print(f"[OK] Migrated {len(old_traits)} traits -> {TRAIT_V2_PATH}")
    print(f"[OK] behavior actions: {len(behavior_map)}")
    print(f"[OK] emotion buckets: sensitivity={len(emotion_map['sensitivity'])}, baseline={len(emotion_map['baseline'])}, mult={len(emotion_map['mult'])}")
    print(f"[OK] violation actions: {len(violation_map)}")
    print(f"[OK] locale keys: ko={len(ko_locale)}, en={len(en_locale)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
