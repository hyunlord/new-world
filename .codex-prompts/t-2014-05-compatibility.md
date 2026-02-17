# T-2014-05: HEXACO Personality Compatibility Function

## Objective
Create a standalone personality compatibility calculation using weighted axis similarity.
Replaces the existing `GameConfig.personality_compatibility()` function.

## Non-goals
- Do NOT modify game_config.gd (that's DIRECT wiring to remove old function)
- Do NOT modify social_event_system.gd (that's T-2014-09)
- Do NOT implement full relationship system changes

## Scope
Files to CREATE:
- `scripts/core/personality_system.gd` — Compatibility + future personality utility functions

## Critical Godot 4.6 Constraints
- **DO NOT use `class_name`** (RefCounted, fails in headless mode)
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`
- Use static functions

## PersonalityData Interface (from T-2014-01)
```gdscript
var axes: Dictionary = {}  # "H": 0.55, "E": 0.42, ... (6 keys, 0.0~1.0)
```

## Detailed Spec

```gdscript
extends RefCounted

## Personality utility functions: compatibility, axis queries.
## No class_name — use preload("res://scripts/core/personality_system.gd").

## Compatibility weights: trust/fairness (H) and conflict (A) matter most for relationships.
## H: 3.0, A: 2.0, C: 1.5, E: 1.0, X: 1.0, O: 0.8
const COMPAT_WEIGHTS: Dictionary = {
    "H": 3.0, "A": 2.0, "C": 1.5,
    "E": 1.0, "X": 1.0, "O": 0.8
}


## Calculate personality compatibility between two PersonalityData instances.
## Returns float in range [-1.0, +1.0].
## +1.0 = maximally compatible (identical on weighted axes)
## -1.0 = maximally incompatible (opposite extremes on weighted axes)
##  0.0 = average compatibility
static func personality_compatibility(pd_a: RefCounted, pd_b: RefCounted) -> float:
    var weighted_dist: float = 0.0
    var total_weight: float = 0.0
    var keys: Array = COMPAT_WEIGHTS.keys()
    for i in range(keys.size()):
        var axis_id: String = keys[i]
        var w: float = COMPAT_WEIGHTS[axis_id]
        var val_a: float = pd_a.axes.get(axis_id, 0.5)
        var val_b: float = pd_b.axes.get(axis_id, 0.5)
        var diff: float = absf(val_a - val_b)
        weighted_dist += w * diff
        total_weight += w
    var similarity: float = 1.0 - weighted_dist / total_weight
    return 2.0 * similarity - 1.0  # map [0,1] -> [-1,+1]


## Apply compatibility to affinity change speed.
## compat +1.0 → change * 1.5 (fast bonding)
## compat -1.0 → change * 0.5 (slow bonding)
## compat  0.0 → change * 1.0 (baseline)
static func apply_compatibility_modifier(base_change: float, compatibility: float) -> float:
    return base_change * (1.0 + 0.5 * compatibility)
```

## Acceptance Criteria
- [ ] `scripts/core/personality_system.gd` extends RefCounted, NO class_name
- [ ] `personality_compatibility()` returns [-1.0, +1.0]
- [ ] Weighted by H:3, A:2, C:1.5, E:1, X:1, O:0.8
- [ ] Identical personalities → +1.0
- [ ] Maximally different (all axis diffs = 1.0) → -1.0
- [ ] `apply_compatibility_modifier()` scales base_change by (1 + 0.5*compat)
- [ ] All functions are static
- [ ] Weight rationale in comments
