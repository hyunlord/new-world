# T-2014-04: PersonalityMaturation — Age-Based Personality Change

## Objective
Implement the OU (Ornstein-Uhlenbeck) process for age-based personality maturation.
Called once per game year on each entity's birthday.

## Non-goals
- Do NOT modify entity_data.gd, entity_manager.gd, or any existing files
- Do NOT hook this into age_system.gd (that's DIRECT wiring)
- Do NOT implement personality generation (that's T-2014-02)

## Scope
Files to CREATE:
- `scripts/systems/personality_maturation.gd` — OU maturation process

## Critical Godot 4.6 Constraints
- **DO NOT use `class_name`** (RefCounted, fails in headless mode)
- `randfn()` does NOT exist — implement Box-Muller transform
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`
- Reference PersonalityData via preload
- Reference TraitSystem via preload

## PersonalityData Interface (from T-2014-01)
```gdscript
var facets: Dictionary = {}    # 24 keys, 0.0~1.0
var axes: Dictionary = {}      # 6 keys
var active_traits: Array = []
const AXIS_IDS = ["H", "E", "X", "A", "C", "O"]
const FACET_KEYS: Dictionary = { "H": [...], "E": [...], ... }
func recalculate_axes() -> void
func to_zscore(trait01: float) -> float
func from_zscore(z: float) -> float
```

## Detailed Spec

```gdscript
extends RefCounted

## Age-based personality maturation using OU (Ornstein-Uhlenbeck) process.
## Called once per game year for each entity.
## Ashton & Lee (2016): H increases ~+1 SD from 18→60, E/X mild increase.
## No class_name — use preload("res://scripts/systems/personality_maturation.gd").

const PersonalityDataScript = preload("res://scripts/core/personality_data.gd")
const TraitSystem = preload("res://scripts/systems/trait_system.gd")

## OU process parameters
const THETA: float = 0.03   # Annual convergence rate (3%)
const SIGMA: float = 0.03   # Annual random drift SD

var _rng: RandomNumberGenerator


func init(rng: RandomNumberGenerator) -> void:
    _rng = rng


## Box-Muller normal random
func _randfn(mean: float, std: float) -> float:
    var u1: float = _rng.randf()
    var u2: float = _rng.randf()
    if u1 < 1e-10:
        u1 = 1e-10
    return mean + std * sqrt(-2.0 * log(u1)) * cos(2.0 * PI * u2)


## Apply one year of maturation to a PersonalityData.
## age: entity's current age in years (integer).
func apply_maturation(pd: RefCounted, age: int) -> void:
    var axis_ids: Array = PersonalityDataScript.AXIS_IDS

    for i in range(axis_ids.size()):
        var aid: String = axis_ids[i]
        var target: float = _get_maturation_target(aid, age)
        var fkeys: Array = PersonalityDataScript.FACET_KEYS[aid]

        for j in range(fkeys.size()):
            var fkey: String = fkeys[j]
            var current_z: float = pd.to_zscore(pd.facets.get(fkey, 0.5))
            # OU drift toward target + random noise
            # target is the shift from baseline, current deviation from mean is (current_z)
            var dz: float = THETA * (target - current_z) + _randfn(0.0, SIGMA)
            pd.facets[fkey] = pd.from_zscore(current_z + dz)

    pd.recalculate_axes()
    pd.active_traits = TraitSystem.check_traits(pd)


## Get maturation target z-shift for axis at given age.
## H: +1.0 SD from 18→60 (linear), E/X: +0.3 SD, A/C/O: stable.
## Ashton & Lee (2016).
func _get_maturation_target(axis_id: String, age: int) -> float:
    match axis_id:
        "H":
            return _linear_target(age, 1.0)
        "E":
            return _linear_target(age, 0.3)
        "X":
            return _linear_target(age, 0.3)
    # A, C, O are stable
    return 0.0


## Linear maturation target: 0 at age 18, max_shift at age 60, clamped.
func _linear_target(age: int, max_shift: float) -> float:
    if age < 18:
        return 0.0
    var t: float = clampf(float(age - 18) / 42.0, 0.0, 1.0)
    return max_shift * t
```

## Acceptance Criteria
- [ ] `scripts/systems/personality_maturation.gd` extends RefCounted, NO class_name
- [ ] `apply_maturation(pd, age)` modifies pd.facets in place
- [ ] H target: +1.0 SD from age 18 to 60 (linear)
- [ ] E, X target: +0.3 SD from age 18 to 60
- [ ] A, C, O target: 0 (stable)
- [ ] OU process: drift = θ * (target - current_z) + N(0, σ)
- [ ] θ = 0.03, σ = 0.03
- [ ] Age < 18: no maturation (target = 0)
- [ ] Recalculates axes and re-checks traits after maturation
- [ ] Box-Muller _randfn() correctly implemented
- [ ] Academic source comments on constants
