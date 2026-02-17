# T-2014-01: PersonalityData Class + HEXACO Definition JSON

## Objective
Create the HEXACO 24-facet personality data class and JSON definition file.

## Non-goals
- Do NOT modify entity_data.gd, entity_manager.gd, or any existing files
- Do NOT create personality generation logic (that's T-2014-02)
- Do NOT create trait checking logic (that's T-2014-03)
- Do NOT add autoloads or scene tree dependencies

## Scope
Files to CREATE:
- `data/personality/hexaco_definition.json` — HEXACO axes + facets definition
- `scripts/core/personality_data.gd` — PersonalityData RefCounted class

## Critical Godot 4.6 Headless Constraints
- **DO NOT use `class_name`** on this script (it's RefCounted, fails in headless mode)
- Use `extends RefCounted`
- Other scripts will reference this via `preload("res://scripts/core/personality_data.gd")`
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)` (triggers inference error)

## Detailed Spec

### 1. `data/personality/hexaco_definition.json`

```json
{
    "axes": {
        "H": {
            "name": "Honesty-Humility",
            "name_kr": "정직-겸손",
            "facets": {
                "H_sincerity": { "name": "Sincerity", "name_kr": "진실성" },
                "H_fairness": { "name": "Fairness", "name_kr": "공정성" },
                "H_greed_avoidance": { "name": "Greed Avoidance", "name_kr": "탐욕 회피" },
                "H_modesty": { "name": "Modesty", "name_kr": "겸손" }
            }
        },
        "E": {
            "name": "Emotionality",
            "name_kr": "감정성",
            "facets": {
                "E_fearfulness": { "name": "Fearfulness", "name_kr": "두려움" },
                "E_anxiety": { "name": "Anxiety", "name_kr": "불안" },
                "E_dependence": { "name": "Dependence", "name_kr": "의존성" },
                "E_sentimentality": { "name": "Sentimentality", "name_kr": "감상성" }
            }
        },
        "X": {
            "name": "Extraversion",
            "name_kr": "외향성",
            "facets": {
                "X_social_self_esteem": { "name": "Social Self-Esteem", "name_kr": "사회적 자존감" },
                "X_social_boldness": { "name": "Social Boldness", "name_kr": "사회적 대담함" },
                "X_sociability": { "name": "Sociability", "name_kr": "사교성" },
                "X_liveliness": { "name": "Liveliness", "name_kr": "활기" }
            }
        },
        "A": {
            "name": "Agreeableness",
            "name_kr": "우호성",
            "facets": {
                "A_forgiveness": { "name": "Forgiveness", "name_kr": "용서" },
                "A_gentleness": { "name": "Gentleness", "name_kr": "온화" },
                "A_flexibility": { "name": "Flexibility", "name_kr": "유연성" },
                "A_patience": { "name": "Patience", "name_kr": "인내" }
            }
        },
        "C": {
            "name": "Conscientiousness",
            "name_kr": "성실성",
            "facets": {
                "C_organization": { "name": "Organization", "name_kr": "조직화" },
                "C_diligence": { "name": "Diligence", "name_kr": "근면" },
                "C_perfectionism": { "name": "Perfectionism", "name_kr": "완벽주의" },
                "C_prudence": { "name": "Prudence", "name_kr": "신중" }
            }
        },
        "O": {
            "name": "Openness to Experience",
            "name_kr": "경험 개방성",
            "facets": {
                "O_aesthetic": { "name": "Aesthetic Appreciation", "name_kr": "심미성" },
                "O_inquisitiveness": { "name": "Inquisitiveness", "name_kr": "호기심" },
                "O_creativity": { "name": "Creativity", "name_kr": "창의성" },
                "O_unconventionality": { "name": "Unconventionality", "name_kr": "비전통성" }
            }
        }
    },
    "interstitial": {
        "altruism": { "name": "Altruism", "name_kr": "이타성", "note": "Component between H and E" }
    }
}
```

### 2. `scripts/core/personality_data.gd`

```gdscript
extends RefCounted

## HEXACO 24-facet personality data container.
## No class_name — use preload("res://scripts/core/personality_data.gd") to reference.

## 24 facet values (0.0~1.0)
var facets: Dictionary = {}  # "H_sincerity": 0.5, "H_fairness": 0.6, ...

## 6 axis values (auto-calculated from facet averages)
var axes: Dictionary = {}    # "H": 0.55, "E": 0.42, ...

## Active discrete traits (emerged from extreme values)
var active_traits: Array = []  # ["honest", "fearful", ...]

## Ordered axis IDs
const AXIS_IDS: Array = ["H", "E", "X", "A", "C", "O"]

## Axis -> facet key mapping
const FACET_KEYS: Dictionary = {
    "H": ["H_sincerity", "H_fairness", "H_greed_avoidance", "H_modesty"],
    "E": ["E_fearfulness", "E_anxiety", "E_dependence", "E_sentimentality"],
    "X": ["X_social_self_esteem", "X_social_boldness", "X_sociability", "X_liveliness"],
    "A": ["A_forgiveness", "A_gentleness", "A_flexibility", "A_patience"],
    "C": ["C_organization", "C_diligence", "C_perfectionism", "C_prudence"],
    "O": ["O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality"],
}

## All 24 facet keys in fixed order (for serialization)
const ALL_FACET_KEYS: Array = [
    "H_sincerity", "H_fairness", "H_greed_avoidance", "H_modesty",
    "E_fearfulness", "E_anxiety", "E_dependence", "E_sentimentality",
    "X_social_self_esteem", "X_social_boldness", "X_sociability", "X_liveliness",
    "A_forgiveness", "A_gentleness", "A_flexibility", "A_patience",
    "C_organization", "C_diligence", "C_perfectionism", "C_prudence",
    "O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality",
]


## Get axis value (average of 4 facets)
func get_axis(axis_id: String) -> float:
    var sum: float = 0.0
    var count: int = 0
    var keys = FACET_KEYS.get(axis_id, [])
    for i in range(keys.size()):
        sum += facets.get(keys[i], 0.5)
        count += 1
    return sum / maxf(count, 1)


## Recalculate all 6 axis values from facets
func recalculate_axes() -> void:
    for i in range(AXIS_IDS.size()):
        axes[AXIS_IDS[i]] = get_axis(AXIS_IDS[i])


## Convert 0.0~1.0 trait value to z-score (mean=0.5, sd=0.15)
func to_zscore(trait01: float) -> float:
    return (trait01 - 0.5) / 0.15


## Convert z-score back to 0.0~1.0 (clamped to [0.05, 0.95])
func from_zscore(z: float) -> float:
    return clampf(0.5 + 0.15 * z, 0.05, 0.95)


## Get facet keys for a given axis
func get_facet_keys_for_axis(axis_id: String) -> Array:
    return FACET_KEYS.get(axis_id, [])


## Initialize with default values (all facets = 0.5)
func init_defaults() -> void:
    for i in range(ALL_FACET_KEYS.size()):
        facets[ALL_FACET_KEYS[i]] = 0.5
    recalculate_axes()


## Create from old Big Five personality Dictionary (migration)
## Maps: openness->O, extraversion->X, agreeableness->A+H(0.5), diligence->C,
## emotional_stability -> invert to E
func migrate_from_big_five(old_personality: Dictionary) -> void:
    var o_val: float = old_personality.get("openness", 0.5)
    var a_val: float = old_personality.get("agreeableness", 0.5)
    var x_val: float = old_personality.get("extraversion", 0.5)
    var c_val: float = old_personality.get("diligence", 0.5)
    var e_stab: float = old_personality.get("emotional_stability", 0.5)
    # Map emotional_stability (high=stable) to Emotionality (high=emotional)
    var e_val: float = 1.0 - e_stab
    # H has no Big Five equivalent -> default 0.5
    var h_val: float = 0.5

    for fk in FACET_KEYS["H"]:
        facets[fk] = h_val
    for fk in FACET_KEYS["E"]:
        facets[fk] = e_val
    for fk in FACET_KEYS["X"]:
        facets[fk] = x_val
    for fk in FACET_KEYS["A"]:
        facets[fk] = a_val
    for fk in FACET_KEYS["C"]:
        facets[fk] = c_val
    for fk in FACET_KEYS["O"]:
        facets[fk] = o_val
    recalculate_axes()


## Serialize to Dictionary for save/load
func to_dict() -> Dictionary:
    return {
        "facets": facets.duplicate(),
        "active_traits": active_traits.duplicate(),
    }


## Deserialize from Dictionary
static func from_dict(data: Dictionary) -> RefCounted:
    var script = load("res://scripts/core/personality_data.gd")
    var pd = script.new()
    var f_data = data.get("facets", {})
    if f_data.is_empty():
        # Migration: old format might have axes directly
        pd.init_defaults()
    else:
        for i in range(pd.ALL_FACET_KEYS.size()):
            var key = pd.ALL_FACET_KEYS[i]
            pd.facets[key] = float(f_data.get(key, 0.5))
    pd.recalculate_axes()
    pd.active_traits = []
    var traits_data = data.get("active_traits", [])
    for i in range(traits_data.size()):
        pd.active_traits.append(str(traits_data[i]))
    return pd
```

## Acceptance Criteria
- [ ] `data/personality/hexaco_definition.json` exists and is valid JSON
- [ ] `scripts/core/personality_data.gd` extends RefCounted, NO class_name
- [ ] Has all 24 facet keys in FACET_KEYS and ALL_FACET_KEYS
- [ ] `get_axis()` returns average of 4 facets
- [ ] `recalculate_axes()` updates all 6 axes
- [ ] `to_zscore()` / `from_zscore()` round-trip correctly
- [ ] `to_dict()` / `from_dict()` round-trip correctly
- [ ] `migrate_from_big_five()` maps old 5-key personality to 24 facets
- [ ] `init_defaults()` sets all 24 facets to 0.5
- [ ] No `var x :=` patterns (use untyped `var x =` for dict.get())
