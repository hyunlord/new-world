# T-2014-08: emotion_system.gd — HEXACO Personality Access Migration

## Objective
Update emotion_system.gd to read personality values from the new PersonalityData object
instead of the old Big Five dictionary keys.

## Non-goals
- Do NOT modify any other files
- Do NOT add new emotion mechanics
- Do NOT change emotion update logic beyond personality access

## Scope
Files to MODIFY:
- `scripts/systems/emotion_system.gd` — 2 lines that access personality

## Current Code (lines 64, 74)
```gdscript
# Line 64: in _update_stress()
var stability: float = entity.personality.get("emotional_stability", 0.5)

# Line 74: in _update_grief()
var stability: float = entity.personality.get("emotional_stability", 0.5)
```

## New PersonalityData Format
`entity.personality` is now a PersonalityData object (RefCounted):
```gdscript
var axes: Dictionary = {}  # "H", "E", "X", "A", "C", "O"
```

The old "emotional_stability" maps to INVERTED Emotionality (E):
- High emotional_stability (old) = Low E (new)
- stability = 1.0 - E_axis_value

## Changes

Replace both occurrences:
```gdscript
# OLD:
var stability: float = entity.personality.get("emotional_stability", 0.5)

# NEW:
var stability: float = 1.0 - entity.personality.axes.get("E", 0.5)
```

That's it. Two lines changed.

## Acceptance Criteria
- [ ] Both personality.get("emotional_stability") calls replaced
- [ ] Uses `1.0 - entity.personality.axes.get("E", 0.5)`
- [ ] No other changes to the file
- [ ] Stress and grief calculations still work correctly with inverted E
