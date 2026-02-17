# T-2014-07: Save/Load Binary Format v5 — HEXACO Personality

## Objective
Update save_manager.gd to save/load HEXACO 24-facet personality data in binary format.
Bump SAVE_VERSION to 5, handle backward compatibility with v4 saves.

## Non-goals
- Do NOT modify entity_data.gd or any other files
- Do NOT change building, settlement, relationship, or resource map save/load
- Do NOT change the meta.json format

## Scope
Files to MODIFY:
- `scripts/core/save_manager.gd` — Entity save/load section only

## Current Binary Format (v4)
Entity personality section saves 5 floats in fixed order:
```
store_float(personality.get("openness", 0.5))
store_float(personality.get("agreeableness", 0.5))
store_float(personality.get("extraversion", 0.5))
store_float(personality.get("diligence", 0.5))
store_float(personality.get("emotional_stability", 0.5))
```

## New PersonalityData Format
`entity.personality` is now a PersonalityData object (RefCounted) with:
```gdscript
var facets: Dictionary = {}      # 24 keys, 0.0~1.0
var active_traits: Array = []    # ["honest", "fearful", ...]
const ALL_FACET_KEYS: Array = [
    "H_sincerity", "H_fairness", "H_greed_avoidance", "H_modesty",
    "E_fearfulness", "E_anxiety", "E_dependence", "E_sentimentality",
    "X_social_self_esteem", "X_social_boldness", "X_sociability", "X_liveliness",
    "A_forgiveness", "A_gentleness", "A_flexibility", "A_patience",
    "C_organization", "C_diligence", "C_perfectionism", "C_prudence",
    "O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality",
]
func recalculate_axes() -> void
func migrate_from_big_five(old: Dictionary) -> void  # for v4 migration
```

## Detailed Changes

### 1. Version bump
```gdscript
const SAVE_VERSION: int = 5
const MIN_LOAD_VERSION: int = 3  # Keep — still support v3+
```

### 2. Save entities — personality section
Replace the 5-float personality block with:
```gdscript
# Personality (v5: 24 facet floats in fixed order + trait count + trait strings)
var pd: RefCounted = e.personality
for fi in range(pd.ALL_FACET_KEYS.size()):
    f.store_float(pd.facets.get(pd.ALL_FACET_KEYS[fi], 0.5))
# Active traits
f.store_8(pd.active_traits.size())
for ti in range(pd.active_traits.size()):
    f.store_pascal_string(str(pd.active_traits[ti]))
```

### 3. Load entities — personality section
Replace the 5-float personality read with version-conditional:
```gdscript
var PersonalityDataScript = load("res://scripts/core/personality_data.gd")
var pd = PersonalityDataScript.new()
if _load_version >= 5:
    # v5+: read 24 facet floats + traits
    for fi in range(pd.ALL_FACET_KEYS.size()):
        pd.facets[pd.ALL_FACET_KEYS[fi]] = f.get_float()
    pd.recalculate_axes()
    var tc: int = f.get_8()
    for ti in range(tc):
        pd.active_traits.append(f.get_pascal_string())
else:
    # v3/v4: read old 5 Big Five floats, migrate
    var old_personality: Dictionary = {
        "openness": f.get_float(),
        "agreeableness": f.get_float(),
        "extraversion": f.get_float(),
        "diligence": f.get_float(),
        "emotional_stability": f.get_float(),
    }
    pd.migrate_from_big_five(old_personality)
e.personality = pd
```

### 4. Emotions block stays the same
The 5 emotion floats immediately follow personality. Make sure the read order is preserved:
```
[personality section] → [emotions: 5 floats] → [job + settlement] → ...
```

## IMPORTANT: Preserve everything else
Only modify:
1. `SAVE_VERSION` constant (line 8)
2. `_save_entities()` — personality section only (~lines 146-150)
3. `_load_entities()` — personality section only (~lines 215-221)

Do NOT change: buildings, relationships, settlements, resource map, stats, meta.json.

## Acceptance Criteria
- [ ] SAVE_VERSION = 5
- [ ] MIN_LOAD_VERSION still 3
- [ ] v5 save: writes 24 floats + trait count + trait strings
- [ ] v5 load: reads 24 floats + traits, recalculates axes
- [ ] v4 load: reads old 5 floats, calls migrate_from_big_five()
- [ ] v3 load: same as v4 (5 floats)
- [ ] Emotion floats still read correctly after personality section
- [ ] No changes to building/relationship/settlement/resource sections
