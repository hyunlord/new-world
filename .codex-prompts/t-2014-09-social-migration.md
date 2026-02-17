# T-2014-09: social_event_system.gd — HEXACO Personality Access Migration

## Objective
Update social_event_system.gd to read personality values from the new PersonalityData object
and use the new compatibility function instead of GameConfig.personality_compatibility().

## Non-goals
- Do NOT modify any other files
- Do NOT add new social event types
- Do NOT change event logic beyond personality access

## Scope
Files to MODIFY:
- `scripts/systems/social_event_system.gd` — 3 personality access points

## Current Code
```gdscript
# Line 77: deep_talk check
if a.personality.get("extraversion", 0.5) > 0.4 and b.personality.get("extraversion", 0.5) > 0.4:

# Line 86: share_food check
var a_agree: float = a.personality.get("agreeableness", 0.5)

# Line 216: proposal acceptance
var compat: float = GameConfig.personality_compatibility(a.personality, b.personality)
```

## New PersonalityData Format
`entity.personality` is now a PersonalityData object (RefCounted):
```gdscript
var axes: Dictionary = {}  # "H", "E", "X", "A", "C", "O"
# X = Extraversion, A = Agreeableness
```

## New Compatibility Function
```gdscript
const PersonalitySystem = preload("res://scripts/core/personality_system.gd")
# PersonalitySystem.personality_compatibility(pd_a, pd_b) -> float [-1, +1]
```

## Changes

### 1. Add preload at top of file
```gdscript
const PersonalitySystem = preload("res://scripts/core/personality_system.gd")
```

### 2. Line 77: deep_talk — extraversion → X axis
```gdscript
# OLD:
if a.personality.get("extraversion", 0.5) > 0.4 and b.personality.get("extraversion", 0.5) > 0.4:
# NEW:
if a.personality.axes.get("X", 0.5) > 0.4 and b.personality.axes.get("X", 0.5) > 0.4:
```

### 3. Line 86: share_food — agreeableness → A axis
```gdscript
# OLD:
var a_agree: float = a.personality.get("agreeableness", 0.5)
# NEW:
var a_agree: float = a.personality.axes.get("A", 0.5)
```

### 4. Line 216: proposal — use new PersonalitySystem
```gdscript
# OLD:
var compat: float = GameConfig.personality_compatibility(a.personality, b.personality)
# NEW:
var compat: float = PersonalitySystem.personality_compatibility(a.personality, b.personality)
```

## Acceptance Criteria
- [ ] PersonalitySystem preloaded at top
- [ ] extraversion → .axes.get("X", 0.5)
- [ ] agreeableness → .axes.get("A", 0.5)
- [ ] GameConfig.personality_compatibility → PersonalitySystem.personality_compatibility
- [ ] No other changes to the file
- [ ] All 3 social event checks work with new PersonalityData
