# T-2014-03: TraitSystem + Trait Definitions JSON

## Objective
Create the discrete trait emergence system — when personality facet/axis values hit extreme thresholds
(top/bottom 10%), named traits activate with behavioral effect multipliers.

## Non-goals
- Do NOT modify any existing files
- Do NOT integrate with BehaviorSystem or Utility AI (future Phase C1)
- Do NOT add autoloads or modify project.godot
- Do NOT handle trait display in UI (that's T-2014-06)

## Scope
Files to CREATE:
- `data/personality/trait_definitions.json` — 14 trait definitions with conditions and effects
- `scripts/systems/trait_system.gd` — Static utility for checking traits and getting effects

## Critical Godot 4.6 Headless Constraints
- **DO NOT use `class_name`** on trait_system.gd (RefCounted, fails in headless)
- Use `static var` and `static func` for singleton-like behavior
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`

## PersonalityData Interface (created in T-2014-01)
TraitSystem receives a PersonalityData object with these fields:
```gdscript
# PersonalityData has:
var facets: Dictionary = {}  # "H_sincerity": 0.5, etc. (24 keys, 0.0~1.0)
var axes: Dictionary = {}    # "H": 0.55, etc. (6 keys, 0.0~1.0)
var active_traits: Array = []  # ["honest", "fearful", ...]
```

## Detailed Spec

### 1. `data/personality/trait_definitions.json`

```json
{
    "traits": [
        {
            "id": "deceitful",
            "name_kr": "사기꾼",
            "name_en": "Deceitful",
            "condition": { "axis": "H", "direction": "low", "threshold": 0.10 },
            "effects": { "steal_weight": 1.5, "bribe_weight": 1.3, "trust_penalty": -0.2 },
            "sentiment": "negative"
        },
        {
            "id": "honest",
            "name_kr": "청렴",
            "name_en": "Honest",
            "condition": { "axis": "H", "direction": "high", "threshold": 0.90 },
            "effects": { "share_weight": 1.3, "reputation_bonus": 0.1, "steal_weight": 0.1 },
            "sentiment": "positive"
        },
        {
            "id": "fearful",
            "name_kr": "겁이 많음",
            "name_en": "Fearful",
            "condition": { "facet": "E_fearfulness", "direction": "high", "threshold": 0.90 },
            "effects": { "flee_weight": 1.5, "explore_weight": 0.5, "combat_weight": 0.3 },
            "sentiment": "negative"
        },
        {
            "id": "bold",
            "name_kr": "대담함",
            "name_en": "Bold",
            "condition": { "facet": "E_fearfulness", "direction": "low", "threshold": 0.10 },
            "effects": { "explore_weight": 1.5, "combat_weight": 1.3, "flee_weight": 0.5 },
            "sentiment": "positive"
        },
        {
            "id": "empathic",
            "name_kr": "공감적",
            "name_en": "Empathic",
            "condition": { "facet": "E_sentimentality", "direction": "high", "threshold": 0.90 },
            "effects": { "care_weight": 1.5, "relationship_bonus": 0.1 },
            "sentiment": "positive"
        },
        {
            "id": "cold",
            "name_kr": "냉담",
            "name_en": "Cold",
            "condition": { "facet": "E_sentimentality", "direction": "low", "threshold": 0.10 },
            "effects": { "care_weight": 0.5, "relationship_penalty": -0.1 },
            "sentiment": "negative"
        },
        {
            "id": "charismatic",
            "name_kr": "카리스마",
            "name_en": "Charismatic",
            "condition": { "axis": "X", "direction": "high", "threshold": 0.90 },
            "effects": { "socialize_weight": 1.5, "lead_weight": 1.4, "relationship_speed": 1.3 },
            "sentiment": "positive"
        },
        {
            "id": "loner",
            "name_kr": "외톨이",
            "name_en": "Loner",
            "condition": { "axis": "X", "direction": "low", "threshold": 0.10 },
            "effects": { "socialize_weight": 0.4, "solitude_bonus": 1.3 },
            "sentiment": "neutral"
        },
        {
            "id": "vengeful",
            "name_kr": "복수심",
            "name_en": "Vengeful",
            "condition": { "axis": "A", "direction": "low", "threshold": 0.10 },
            "effects": { "revenge_weight": 1.5, "forgive_weight": 0.2, "anger_buildup": 1.4 },
            "sentiment": "negative"
        },
        {
            "id": "forgiving",
            "name_kr": "관대함",
            "name_en": "Forgiving",
            "condition": { "axis": "A", "direction": "high", "threshold": 0.90 },
            "effects": { "forgive_weight": 1.5, "anger_decay": 1.3 },
            "sentiment": "positive"
        },
        {
            "id": "industrious",
            "name_kr": "근면",
            "name_en": "Industrious",
            "condition": { "axis": "C", "direction": "high", "threshold": 0.90 },
            "effects": { "work_efficiency": 1.3, "task_persistence": 1.4, "build_quality": 1.2 },
            "sentiment": "positive"
        },
        {
            "id": "impulsive",
            "name_kr": "충동적",
            "name_en": "Impulsive",
            "condition": { "axis": "C", "direction": "low", "threshold": 0.10 },
            "effects": { "task_abandon_chance": 1.5, "work_efficiency": 0.7, "impulse_action_chance": 1.3 },
            "sentiment": "negative"
        },
        {
            "id": "innovator",
            "name_kr": "혁신가",
            "name_en": "Innovator",
            "condition": { "axis": "O", "direction": "high", "threshold": 0.90 },
            "effects": { "research_weight": 1.5, "explore_weight": 1.3, "new_food_accept": 1.4, "migration_willingness": 1.3 },
            "sentiment": "positive"
        },
        {
            "id": "traditionalist",
            "name_kr": "전통주의",
            "name_en": "Traditionalist",
            "condition": { "axis": "O", "direction": "low", "threshold": 0.10 },
            "effects": { "research_weight": 0.5, "explore_weight": 0.5, "routine_bonus": 1.3 },
            "sentiment": "neutral"
        }
    ]
}
```

### 2. `scripts/systems/trait_system.gd`

```gdscript
extends RefCounted

## Discrete trait emergence system.
## Checks personality extremes and returns active traits + combined effects.
## No class_name — use preload("res://scripts/systems/trait_system.gd").

static var _trait_definitions: Array = []
static var _loaded: bool = false


static func _ensure_loaded() -> void:
    if _loaded:
        return
    var file = FileAccess.open("res://data/personality/trait_definitions.json", FileAccess.READ)
    if file == null:
        push_warning("[TraitSystem] Cannot load trait_definitions.json")
        _loaded = true
        return
    var json = JSON.new()
    if json.parse(file.get_as_text()) != OK:
        push_warning("[TraitSystem] Invalid trait_definitions.json")
        _loaded = true
        return
    _trait_definitions = json.data.get("traits", [])
    _loaded = true


## Check which traits are active for a given PersonalityData.
## Returns Array of trait ID strings.
static func check_traits(pd: RefCounted) -> Array:
    _ensure_loaded()
    var traits: Array = []
    for i in range(_trait_definitions.size()):
        var tdef = _trait_definitions[i]
        var cond = tdef.get("condition", {})
        var value: float = 0.5
        if cond.has("facet"):
            value = pd.facets.get(cond.get("facet", ""), 0.5)
        elif cond.has("axis"):
            value = pd.axes.get(cond.get("axis", ""), 0.5)

        var threshold = float(cond.get("threshold", 0.5))
        var direction = cond.get("direction", "")
        if direction == "high" and value >= threshold:
            traits.append(tdef.get("id", ""))
        elif direction == "low" and value <= threshold:
            traits.append(tdef.get("id", ""))
    return traits


## Get combined effect multipliers from a list of active trait IDs.
## Effects are combined multiplicatively (multiple traits stack).
## Returns Dictionary of effect_key -> combined_multiplier.
static func get_trait_effects(trait_ids: Array) -> Dictionary:
    _ensure_loaded()
    var combined: Dictionary = {}
    for i in range(trait_ids.size()):
        var tid = trait_ids[i]
        for j in range(_trait_definitions.size()):
            var tdef = _trait_definitions[j]
            if tdef.get("id", "") == tid:
                var effects = tdef.get("effects", {})
                var effect_keys = effects.keys()
                for k in range(effect_keys.size()):
                    var ek = effect_keys[k]
                    var ev = float(effects[ek])
                    if combined.has(ek):
                        combined[ek] = combined[ek] * ev
                    else:
                        combined[ek] = ev
                break
    return combined


## Get trait definition by ID (for UI display).
## Returns Dictionary with id, name_kr, name_en, sentiment, etc.
static func get_trait_definition(trait_id: String) -> Dictionary:
    _ensure_loaded()
    for i in range(_trait_definitions.size()):
        if _trait_definitions[i].get("id", "") == trait_id:
            return _trait_definitions[i]
    return {}


## Get sentiment for a trait ("positive", "negative", "neutral").
static func get_trait_sentiment(trait_id: String) -> String:
    var tdef = get_trait_definition(trait_id)
    return tdef.get("sentiment", "neutral")
```

## Acceptance Criteria
- [ ] `data/personality/trait_definitions.json` exists and is valid JSON with 14 traits
- [ ] `scripts/systems/trait_system.gd` extends RefCounted, NO class_name
- [ ] `check_traits()` returns trait IDs for extreme personality values
- [ ] Threshold 0.10 (low) and 0.90 (high) correctly identify extremes
- [ ] `get_trait_effects()` combines effects multiplicatively
- [ ] `get_trait_definition()` returns full trait info for UI display
- [ ] `get_trait_sentiment()` returns "positive"/"negative"/"neutral"
- [ ] Lazy-loads JSON once (static var _loaded pattern)
- [ ] No `var x :=` patterns with dict.get()
