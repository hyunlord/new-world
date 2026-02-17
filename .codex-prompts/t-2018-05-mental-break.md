# T-2018-05: Mental Break System

## Objective
Add mental break detection and behavior to EmotionSystem — when stress exceeds a personality-dependent threshold, the entity enters a break state (panic/rage/shutdown/purge/outrage_violence) that overrides normal behavior for a duration.

## Godot 4.6 Headless Compatibility (CRITICAL)
- **NO `class_name`** — this modifies a RefCounted-extending script
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`

## File to Modify

### `scripts/systems/emotion_system.gd` (APPEND — add mental break methods + integrate into execute_tick)

## Dependencies (already exist)
- `scripts/systems/emotion_system.gd` — EmotionSystem with `execute_tick()`, stress accumulation
- `scripts/core/emotion_data.gd` — EmotionData with `stress: float`, `mental_break_type: String`, `mental_break_remaining: float`, `get_emotion()`, `get_dyad()`
- `scripts/core/personality_data.gd` — PersonalityData with `axes` and `to_zscore()`
- `scripts/core/simulation_system.gd` — base class with `emit_event()` helper

## Current EmotionSystem Structure

The existing `execute_tick()` has steps 1-10 (step 10 = contagion added by T-2018-04). Mental break check runs as step 11 after contagion.

## Changes Required

### 1. Add mental break constants (add after contagion constants)

```gdscript
# ═══════════════════════════════════════════════════
# Mental Break System
# ═══════════════════════════════════════════════════

## Break behavior definitions
const BREAK_BEHAVIORS: Dictionary = {
	"panic": {
		"description": "Panic — flee randomly",
		"duration_hours": 2.0,
		"energy_drain": 3.0,
	},
	"rage": {
		"description": "Rage — attack/destroy nearby",
		"duration_hours": 1.0,
		"energy_drain": 5.0,
	},
	"shutdown": {
		"description": "Shutdown — collapse, idle",
		"duration_hours": 6.0,
		"energy_drain": 0.5,
	},
	"purge": {
		"description": "Purge — expel contaminated",
		"duration_hours": 2.0,
		"energy_drain": 2.0,
	},
	"outrage_violence": {
		"description": "Outrage — immediate violence",
		"duration_hours": 0.5,
		"energy_drain": 4.0,
	},
}

## Sigmoid steepness for break probability
const BREAK_BETA: float = 60.0

## Per-tick break probability multiplier (keeps probability low per check)
const BREAK_TICK_PROB: float = 0.01
```

### 2. Add mental break methods

```gdscript
## Check if entity should enter a mental break state
func _check_mental_break(entity: RefCounted, dt_hours: float) -> void:
	var ed: RefCounted = entity.emotion_data
	if ed == null:
		return
	var pd: RefCounted = entity.personality
	if pd == null:
		return

	# If already in a break, count down and exit when done
	if ed.mental_break_type != "":
		ed.mental_break_remaining -= dt_hours
		# Apply energy drain during break
		var behavior = BREAK_BEHAVIORS.get(ed.mental_break_type, {})
		var drain: float = float(behavior.get("energy_drain", 1.0))
		entity.energy = maxf(0.0, entity.energy - drain * dt_hours / 24.0)
		if ed.mental_break_remaining <= 0.0:
			# Break ends — reduce stress to 50% (not 0, prevents immediate re-break)
			ed.stress *= 0.5
			var old_type: String = ed.mental_break_type
			ed.mental_break_type = ""
			ed.mental_break_remaining = 0.0
			emit_event("mental_break_ended", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"break_type": old_type,
			})
		return

	# C(Conscientiousness) ↑ → higher threshold (more self-control)
	var z_C: float = pd.to_zscore(pd.axes.get("C", 0.5))
	var threshold: float = 300.0 + 50.0 * z_C

	# Skip check if stress is well below threshold
	if ed.stress < threshold * 0.5:
		return

	# Sigmoid probability
	var p: float = 1.0 / (1.0 + exp(-(ed.stress - threshold) / BREAK_BETA))

	if randf() < p * BREAK_TICK_PROB:
		var break_type: String = _determine_break_type(ed)
		if break_type != "":
			ed.mental_break_type = break_type
			var behavior = BREAK_BEHAVIORS.get(break_type, {})
			ed.mental_break_remaining = float(behavior.get("duration_hours", 1.0))
			# Override entity action
			entity.current_action = "mental_break"
			entity.current_goal = break_type
			entity.action_timer = 0
			emit_event("mental_break_started", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"break_type": break_type,
				"stress": ed.stress,
				"threshold": threshold,
			})


## Determine break type based on dominant negative emotion
func _determine_break_type(ed: RefCounted) -> String:
	# Special check: Outrage (Surprise+Anger dyad) > 60 → outrage_violence
	var outrage: float = ed.get_dyad("outrage")
	if outrage > 60.0:
		return "outrage_violence"

	# Otherwise, dominant negative emotion determines type
	var candidates: Dictionary = {
		"panic": ed.get_emotion("fear"),
		"rage": ed.get_emotion("anger"),
		"shutdown": ed.get_emotion("sadness"),
		"purge": ed.get_emotion("disgust"),
	}

	var max_type: String = "shutdown"  # default fallback
	var max_val: float = 0.0
	for btype in candidates:
		if candidates[btype] > max_val:
			max_val = candidates[btype]
			max_type = btype
	return max_type
```

### 3. Integrate into execute_tick() — add mental break check

In the per-entity loop inside `execute_tick()`, **after step 9 (legacy writeback)** and **before the settlement grouping for contagion**, add:

```gdscript
		# Step 11: Mental break check
		_check_mental_break(entity, dt_hours)
```

This goes inside the `for i in range(alive.size()):` loop, after the `entity.emotions[key] = legacy[key]` writeback.

## Non-goals
- Do NOT implement the actual break behaviors (flee, attack, idle) — those are handled by BehaviorSystem checking `entity.current_action == "mental_break"` (future ticket)
- Do NOT modify EmotionData structure (T-2018-01)
- Do NOT modify entity_data.gd, save_manager.gd, or UI files
- Do NOT add SimulationBus signals — use `emit_event()` inherited from SimulationSystem base
- Do NOT modify steps 1-10 of execute_tick()
- Do NOT use `class_name`

## Acceptance Criteria
- [ ] `BREAK_BEHAVIORS` constant with 5 break types (panic, rage, shutdown, purge, outrage_violence)
- [ ] `_check_mental_break()` method with sigmoid probability
- [ ] Break threshold: `300.0 + 50.0 * z_C` (Conscientiousness raises threshold)
- [ ] Break countdown: `mental_break_remaining -= dt_hours`, ends when ≤ 0
- [ ] Post-break stress reduction: `stress *= 0.5` (50%, not 0)
- [ ] Energy drain during break based on break type
- [ ] Entity action set to "mental_break" with goal = break_type
- [ ] `_determine_break_type()` checks outrage dyad > 60, then dominant negative emotion
- [ ] Events emitted: `mental_break_started`, `mental_break_ended`
- [ ] Integrated as step 11 in the per-entity loop
- [ ] No changes to steps 1-10
- [ ] No GDScript parse errors
