# T-2018-04: Emotional Contagion System

## Objective
Add emotional contagion to EmotionSystem — nearby entities' emotions spread within settlements, with distance decay, relationship strength multiplier, and personality-based susceptibility. Based on Hatfield et al. (1993) and Fan et al. (2016).

## Godot 4.6 Headless Compatibility (CRITICAL)
- **NO `class_name`** — this modifies a RefCounted-extending script
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`

## File to Modify

### `scripts/systems/emotion_system.gd` (APPEND — add contagion methods + integrate into execute_tick)

## Dependencies (already exist)
- `scripts/systems/emotion_system.gd` — EmotionSystem with `execute_tick()`, `_entity_manager`, personality coupling
- `scripts/core/emotion_data.gd` — EmotionData with `fast` Dictionary, `get_emotion()` method
- `scripts/core/personality_data.gd` — PersonalityData with `axes` Dictionary, `to_zscore()` method
- Entity has `settlement_id: int`, `position: Vector2i`, `emotion_data: RefCounted`, `personality: RefCounted`

## Current EmotionSystem Structure

The existing `execute_tick()` processes entities individually with 9 steps. Contagion runs **after** the per-entity loop as a settlement-scoped pass.

## Changes Required

### 1. Add contagion constants (add after existing constants section)

```gdscript
# ═══════════════════════════════════════════════════
# Emotional Contagion (Hatfield et al. 1993, Fan et al. 2016)
# ═══════════════════════════════════════════════════

## Contagion coefficients per emotion (anger > fear > joy)
## Fan et al. (2016): anger is more influential than joy in social transmission
const CONTAGION_KAPPA: Dictionary = {
	"anger": 0.12, "fear": 0.10, "joy": 0.08, "disgust": 0.06,
	"trust": 0.06, "sadness": 0.04, "surprise": 0.03, "anticipation": 0.03
}

## Distance decay scale (tiles)
const CONTAGION_DISTANCE_SCALE: float = 5.0

## Minimum emotion value for contagion source (weak emotions don't spread)
const CONTAGION_MIN_SOURCE: float = 10.0
```

### 2. Add contagion method

```gdscript
## Apply emotional contagion within a settlement
## Scope: settlement-only to avoid O(n²) global computation
func _apply_contagion_settlement(members: Array, dt_hours: float) -> void:
	if members.size() < 2:
		return

	# Pre-collect emotion snapshots to prevent order-dependent effects
	var snapshots: Array = []
	for i in range(members.size()):
		var entity: RefCounted = members[i]
		if entity.emotion_data == null:
			snapshots.append(null)
			continue
		var snap: Dictionary = {}
		for emo in CONTAGION_KAPPA:
			snap[emo] = entity.emotion_data.get_emotion(emo)
		snapshots.append(snap)

	for i in range(members.size()):
		var target: RefCounted = members[i]
		if target.emotion_data == null or target.personality == null:
			continue
		var ed: RefCounted = target.emotion_data

		# Susceptibility: E↑ and A↑ → more susceptible to emotional contagion
		var z_E: float = target.personality.to_zscore(target.personality.axes.get("E", 0.5))
		var z_A: float = target.personality.to_zscore(target.personality.axes.get("A", 0.5))
		var susceptibility: float = exp(0.2 * z_E + 0.1 * z_A)

		var delta: Dictionary = {}
		for emo in CONTAGION_KAPPA:
			delta[emo] = 0.0

		for j in range(members.size()):
			if i == j:
				continue
			var source: RefCounted = members[j]
			if snapshots[j] == null:
				continue

			# Distance decay
			var dx: float = float(target.position.x - source.position.x)
			var dy: float = float(target.position.y - source.position.y)
			var distance: float = sqrt(dx * dx + dy * dy)
			var distance_factor: float = exp(-distance / CONTAGION_DISTANCE_SCALE)

			# Skip if too far (optimization)
			if distance_factor < 0.01:
				continue

			# Relationship strength (default 0.5 if no relationship system)
			var relationship: float = 0.5

			for emo in CONTAGION_KAPPA:
				var source_val: float = snapshots[j][emo]
				if source_val > CONTAGION_MIN_SOURCE:
					delta[emo] += CONTAGION_KAPPA[emo] * source_val * distance_factor * relationship * susceptibility * dt_hours

		# Apply contagion deltas to fast layer
		for emo in delta:
			if delta[emo] > 0.0:
				ed.fast[emo] = ed.fast[emo] + delta[emo]
```

### 3. Integrate into execute_tick() — add contagion pass after per-entity loop

At the end of `execute_tick()`, **before** `_pending_events.clear()`, add:

```gdscript
	# Step 10: Emotional contagion (settlement-scoped)
	var settlement_groups: Dictionary = {}
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var sid: int = entity.settlement_id
		if not settlement_groups.has(sid):
			settlement_groups[sid] = []
		settlement_groups[sid].append(entity)
	for sid in settlement_groups:
		_apply_contagion_settlement(settlement_groups[sid], dt_hours)
```

## Non-goals
- Do NOT modify EmotionData (T-2018-01)
- Do NOT add relationship_manager integration (that's future work — use constant 0.5 for now)
- Do NOT modify entity_data.gd, save_manager.gd, or UI files
- Do NOT modify the existing 9 steps of execute_tick() — only APPEND step 10
- Do NOT add global (cross-settlement) contagion
- Do NOT use `class_name`

## Acceptance Criteria
- [ ] `CONTAGION_KAPPA` constants added with all 8 emotions
- [ ] `_apply_contagion_settlement()` method added
- [ ] Contagion uses pre-collected snapshots (order-independent)
- [ ] Distance decay: `exp(-distance / 5.0)`
- [ ] Susceptibility based on E and A personality axes
- [ ] Only emotions > 10.0 are contagious (CONTAGION_MIN_SOURCE)
- [ ] Settlement grouping in execute_tick() before contagion pass
- [ ] Contagion modifies `fast` layer only
- [ ] No changes to existing steps 1-9
- [ ] No GDScript parse errors
