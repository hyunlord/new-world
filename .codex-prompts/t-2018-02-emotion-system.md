# T-2018-02: EmotionSystem — Plutchik Emotion Update Engine

## Objective
Rewrite `scripts/systems/emotion_system.gd` to implement the full Plutchik 8-emotion update engine with 3-layer temporal dynamics, appraisal-based impulse generation, HEXACO personality coupling, opposite emotion inhibition, stress accumulation, and legacy writeback.

## Godot 4.6 Headless Compatibility (CRITICAL)
- **NO `class_name`** — this extends a RefCounted base script
- Use `extends "res://scripts/core/simulation_system.gd"` at top
- Use `preload("res://path.gd")` for cross-script references
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)` (inferred type fails)
- For `randfn()` which doesn't exist in Godot 4 — implement Box-Muller transform

## File to Modify

### `scripts/systems/emotion_system.gd` (FULL REWRITE — replace entire file)

## Dependencies (already exist)
- `scripts/core/emotion_data.gd` — EmotionData RefCounted (8 emotions × 3 layers, VA, Dyads)
- `scripts/core/personality_data.gd` — PersonalityData with HEXACO axes and `to_zscore()` method
- `data/emotions/event_presets.json` — Appraisal vectors for game events

## Current File (95 lines, to be replaced)
The current emotion_system.gd has 5 simple update functions (_update_happiness, _update_loneliness, etc.) operating on `entity.emotions` Dictionary with 0.0-1.0 range values. This entire file is replaced.

## Full Implementation

```gdscript
extends "res://scripts/core/simulation_system.gd"

## Plutchik 8-emotion update engine with 3-layer temporal dynamics.
## Fast (episodic decay) + Slow (mood/baseline, OU process) + Memory trace (long-term scars).
## Appraisal-based impulse from events. HEXACO personality coupling.
## References:
##   Plutchik (1980, 2001), Russell (1980), Lazarus (1991), Scherer (2009)
##   Verduyn & Brans (2012), Hatfield et al. (1993)

const EmotionDataScript = preload("res://scripts/core/emotion_data.gd")

var _entity_manager: RefCounted
var _event_presets: Dictionary = {}
var _pending_events: Dictionary = {}  # entity_id -> Array of event dicts


func _init() -> void:
	system_name = "emotions"
	priority = 32
	tick_interval = 12  # Once per game day


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_event_presets()


func _load_event_presets() -> void:
	var path: String = "res://data/emotions/event_presets.json"
	if not FileAccess.file_exists(path):
		push_warning("[EmotionSystem] Event presets not found: %s" % path)
		return
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return
	var json: JSON = JSON.new()
	if json.parse(f.get_as_text()) == OK:
		var data = json.data
		if data is Dictionary and data.has("events"):
			_event_presets = data["events"]


## Queue an emotional event for an entity (called by other systems via SimulationBus)
func queue_event(entity_id: int, event_key: String, overrides: Dictionary = {}) -> void:
	if not _pending_events.has(entity_id):
		_pending_events[entity_id] = []
	var preset = _event_presets.get(event_key, {})
	if preset.is_empty():
		push_warning("[EmotionSystem] Unknown event preset: %s" % event_key)
		return
	var event: Dictionary = preset.duplicate()
	# Apply overrides (e.g. custom intensity)
	for key in overrides:
		event[key] = overrides[key]
	_pending_events[entity_id].append(event)


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	var dt_hours: float = float(tick_interval) / 12.0  # 12 ticks = 1 game day = 24h, so 1 tick = 2h

	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		# Initialize emotion_data if missing
		if entity.emotion_data == null:
			entity.emotion_data = EmotionDataScript.new()
		var ed: RefCounted = entity.emotion_data
		var pd: RefCounted = entity.personality
		if pd == null:
			continue

		# Gather events for this entity
		var events: Array = _pending_events.get(entity.id, [])

		# Step 1: Event → emotion impulse (appraisal-based)
		var impulse: Dictionary = _calculate_event_impulse(events, pd, ed)

		# Step 2: Fast layer update (exponential decay + impulse)
		for emo in ed.fast:
			var hl: float = _get_adjusted_half_life(emo, pd, "fast")
			var k: float = 0.693147 / hl  # ln(2) / half_life
			ed.fast[emo] = ed.fast[emo] * exp(-k * dt_hours) + impulse.get(emo, 0.0)
			ed.fast[emo] = maxf(ed.fast[emo], 0.0)

		# Step 3: Slow layer update (Ornstein-Uhlenbeck mean-reverting process)
		for emo in ed.slow:
			var baseline: float = _get_baseline(emo, pd)
			var hl_slow: float = SLOW_HALF_LIFE.get(emo, 48.0)
			var k_slow: float = 0.693147 / hl_slow
			var sigma: float = 0.5  # mood fluctuation
			ed.slow[emo] = baseline + (ed.slow[emo] - baseline) * exp(-k_slow * dt_hours)
			ed.slow[emo] += sigma * sqrt(dt_hours) * _randfn()
			ed.slow[emo] = clampf(ed.slow[emo], 0.0, 30.0)

		# Step 4: Memory trace decay
		for emo in ed.memory_traces:
			var traces: Array = ed.memory_traces[emo]
			for j in range(traces.size() - 1, -1, -1):
				traces[j].intensity *= exp(-traces[j].decay_rate * dt_hours)
				if traces[j].intensity < 0.5:
					traces.remove_at(j)

		# Step 5: Opposite emotion inhibition
		for emo in OPPOSITE_PAIRS:
			var opp: String = OPPOSITE_PAIRS[emo]
			var opp_total: float = ed.get_emotion(opp)
			if opp_total > 0.0:
				ed.fast[emo] = maxf(0.0, ed.fast[emo] - INHIBITION_GAMMA * opp_total)

		# Step 6: Valence-Arousal recalculation
		ed.recalculate_va()

		# Step 7: Stress accumulation
		_update_stress(ed, dt_hours)

		# Step 8: Record habituation for processed events
		for j in range(events.size()):
			var cat = events[j].get("category", "generic")
			_record_habituation(ed, cat, tick)

		# Step 9: Write back legacy emotions for backward compatibility
		var legacy: Dictionary = ed.to_legacy_dict()
		for key in legacy:
			entity.emotions[key] = legacy[key]

	# Clear pending events
	_pending_events.clear()


# ═══════════════════════════════════════════════════
# Constants
# ═══════════════════════════════════════════════════

## Fast layer half-lives (hours) — Verduyn & Brans (2012)
const FAST_HALF_LIFE: Dictionary = {
	"joy": 0.75,         # 45 min
	"trust": 2.0,        # 2 hours
	"fear": 0.3,         # 18 min
	"surprise": 0.05,    # 3 min
	"sadness": 0.5,      # 30 min
	"disgust": 0.1,      # 6 min
	"anger": 0.4,        # 24 min
	"anticipation": 3.0  # 3 hours
}

## Slow layer half-lives (hours)
const SLOW_HALF_LIFE: Dictionary = {
	"joy": 48.0, "trust": 72.0, "fear": 24.0, "surprise": 6.0,
	"sadness": 120.0, "disgust": 12.0, "anger": 12.0, "anticipation": 36.0
}

## Opposite emotion pairs (mutual inhibition)
const OPPOSITE_PAIRS: Dictionary = {
	"joy": "sadness", "sadness": "joy",
	"trust": "disgust", "disgust": "trust",
	"fear": "anger", "anger": "fear",
	"surprise": "anticipation", "anticipation": "surprise"
}

## Inhibition coefficient
const INHIBITION_GAMMA: float = 0.3

## Minimum impulse for memory trace creation
const MEMORY_TRACE_THRESHOLD: float = 20.0

## Memory trace retention ratio (fraction of impulse stored long-term)
const MEMORY_TRACE_RATIO: float = 0.3


# ═══════════════════════════════════════════════════
# Appraisal-Based Impulse (Lazarus 1991, Scherer 2009)
# ═══════════════════════════════════════════════════

func _calculate_event_impulse(events: Array, pd: RefCounted, ed: RefCounted) -> Dictionary:
	var total: Dictionary = {}
	for emo in ed.fast:
		total[emo] = 0.0

	for j in range(events.size()):
		var event: Dictionary = events[j]
		var g: float = float(event.get("goal_congruence", 0.0))
		var n: float = float(event.get("novelty", 0.0))
		var c: float = float(event.get("controllability", 0.5))
		var a: float = float(event.get("agency", 0.0))
		var m: float = float(event.get("norm_violation", 0.0))
		var p: float = float(event.get("pathogen", 0.0))
		var b: float = float(event.get("social_bond", 0.0))
		var fr: float = float(event.get("future_relevance", 0.0))
		var base_intensity: float = float(event.get("intensity", 30.0))

		# Habituation factor
		var hab: float = _get_habituation(ed, event.get("category", "generic"))

		# Personality sensitivity
		var sens: Dictionary = _get_personality_sensitivity(pd)

		# Appraisal → 8 emotion impulses
		var impulse: Dictionary = {}
		impulse["joy"] = base_intensity * maxf(0.0, g) * (1.0 + 0.5 * n) * sens.get("joy", 1.0)
		impulse["sadness"] = base_intensity * maxf(0.0, -g) * (1.0 - c) * sens.get("sadness", 1.0)
		impulse["anger"] = base_intensity * maxf(0.0, -g) * c * maxf(0.0, -a + m) * sens.get("anger", 1.0)
		impulse["fear"] = base_intensity * maxf(0.0, -g) * (1.0 - c) * (0.5 + 0.5 * n) * sens.get("fear", 1.0)
		impulse["disgust"] = base_intensity * (p + 0.7 * m) * (0.5 + 0.5 * maxf(0.0, -g)) * sens.get("disgust", 1.0)
		impulse["surprise"] = base_intensity * n * sens.get("surprise", 1.0)
		impulse["trust"] = base_intensity * maxf(0.0, b) * (1.0 - p) * (1.0 - m) * sens.get("trust", 1.0)
		impulse["anticipation"] = base_intensity * fr * (0.5 + 0.5 * maxf(0.0, g)) * sens.get("anticipation", 1.0)

		# Apply habituation and accumulate
		for emo in impulse:
			impulse[emo] *= hab
			total[emo] += impulse[emo]

		# Create memory traces for strong impulses
		if base_intensity >= MEMORY_TRACE_THRESHOLD:
			_create_memory_trace(ed, impulse, event)

	return total


# ═══════════════════════════════════════════════════
# HEXACO → Emotion Coupling
# ═══════════════════════════════════════════════════

func _get_personality_sensitivity(pd: RefCounted) -> Dictionary:
	var z_E: float = pd.to_zscore(pd.axes.get("E", 0.5))
	var z_X: float = pd.to_zscore(pd.axes.get("X", 0.5))
	var z_A: float = pd.to_zscore(pd.axes.get("A", 0.5))
	var z_H: float = pd.to_zscore(pd.axes.get("H", 0.5))
	var z_C: float = pd.to_zscore(pd.axes.get("C", 0.5))
	var z_O: float = pd.to_zscore(pd.axes.get("O", 0.5))
	return {
		"fear": exp(0.4 * z_E),
		"sadness": exp(0.4 * z_E),
		"joy": exp(0.3 * z_X),
		"anger": exp(-0.35 * z_A),
		"disgust": exp(0.25 * z_H),
		"surprise": exp(0.2 * z_O),
		"anticipation": exp(0.2 * z_O + 0.15 * z_C),
		"trust": exp(0.2 * z_X + 0.15 * z_A),
	}


func _get_adjusted_half_life(emo: String, pd: RefCounted, layer: String) -> float:
	var base: float = FAST_HALF_LIFE.get(emo, 1.0) if layer == "fast" else SLOW_HALF_LIFE.get(emo, 48.0)
	var z_E: float = pd.to_zscore(pd.axes.get("E", 0.5))
	var z_X: float = pd.to_zscore(pd.axes.get("X", 0.5))
	var z_A: float = pd.to_zscore(pd.axes.get("A", 0.5))
	if emo == "fear" or emo == "sadness":
		return base * exp(0.3 * z_E)
	if emo == "joy":
		return base * exp(0.2 * z_X)
	if emo == "anger":
		return base * exp(-0.25 * z_A)
	return base


func _get_baseline(emo: String, pd: RefCounted) -> float:
	var z_X: float = pd.to_zscore(pd.axes.get("X", 0.5))
	var z_E: float = pd.to_zscore(pd.axes.get("E", 0.5))
	var z_A: float = pd.to_zscore(pd.axes.get("A", 0.5))
	match emo:
		"joy":
			return clampf(5.0 + 3.0 * z_X, 0.0, 15.0)
		"fear":
			return clampf(2.0 + 2.0 * z_E, 0.0, 10.0)
		"sadness":
			return clampf(2.0 + 1.5 * z_E, 0.0, 10.0)
		"anger":
			return clampf(2.0 - 2.0 * z_A, 0.0, 10.0)
		"trust":
			return clampf(5.0 + 1.5 * z_X, 0.0, 12.0)
		"anticipation":
			return clampf(5.0 + 1.0 * z_X, 0.0, 10.0)
		_:
			return 0.0


# ═══════════════════════════════════════════════════
# Stress Accumulation
# ═══════════════════════════════════════════════════

func _update_stress(ed: RefCounted, dt_hours: float) -> void:
	var tau_S: float = 48.0  # Stress half-life (hours)
	var decay: float = exp(-dt_hours / tau_S)
	var neg_input: float = (
		1.0 * ed.get_emotion("fear") +
		0.9 * ed.get_emotion("anger") +
		1.1 * ed.get_emotion("sadness") +
		0.6 * ed.get_emotion("disgust")
	)
	ed.stress = ed.stress * decay + neg_input * dt_hours


# ═══════════════════════════════════════════════════
# Habituation
# ═══════════════════════════════════════════════════

func _get_habituation(ed: RefCounted, category: String) -> float:
	if not ed.habituation.has(category):
		return 1.0
	var data = ed.habituation[category]
	var n_count: int = int(data.get("count", 0))
	var eta: float = 0.2
	return exp(-eta * float(n_count))


func _record_habituation(ed: RefCounted, category: String, current_tick: int) -> void:
	if not ed.habituation.has(category):
		ed.habituation[category] = {"count": 0, "last_tick": current_tick}
	ed.habituation[category]["count"] = int(ed.habituation[category].get("count", 0)) + 1
	ed.habituation[category]["last_tick"] = current_tick


# ═══════════════════════════════════════════════════
# Memory Trace Creation
# ═══════════════════════════════════════════════════

func _create_memory_trace(ed: RefCounted, impulse: Dictionary, event: Dictionary) -> void:
	var source: String = str(event.get("description", "unknown"))
	var is_trauma: bool = event.get("is_trauma", false)
	# Trauma: half-life 365 days (8760 hours), Normal: 30 days (720 hours)
	var base_decay: float = 0.693147 / (8760.0) if is_trauma else 0.693147 / (720.0)

	for emo in impulse:
		if impulse[emo] > MEMORY_TRACE_THRESHOLD:
			var traces: Array = ed.memory_traces.get(emo, [])
			# Cap at 20 traces per emotion to prevent unbounded growth
			if traces.size() >= 20:
				# Remove weakest trace
				var min_idx: int = 0
				var min_val: float = traces[0].intensity
				for k in range(1, traces.size()):
					if traces[k].intensity < min_val:
						min_val = traces[k].intensity
						min_idx = k
				traces.remove_at(min_idx)
			traces.append({
				"source": source,
				"intensity": impulse[emo] * MEMORY_TRACE_RATIO,
				"decay_rate": base_decay,
			})
			ed.memory_traces[emo] = traces


# ═══════════════════════════════════════════════════
# Box-Muller Transform (randfn replacement)
# ═══════════════════════════════════════════════════

var _spare_normal: float = 0.0
var _has_spare: bool = false


func _randfn() -> float:
	if _has_spare:
		_has_spare = false
		return _spare_normal
	var u: float = randf()
	var v: float = randf()
	# Prevent log(0)
	while u < 1e-10:
		u = randf()
	var mag: float = sqrt(-2.0 * log(u))
	_spare_normal = mag * sin(TAU * v)
	_has_spare = true
	return mag * cos(TAU * v)
```

## Non-goals
- Do NOT create EmotionData (that's T-2018-01, already done)
- Do NOT create event_presets.json (that's T-2018-03, already done)
- Do NOT modify entity_data.gd, save_manager.gd, or entity_detail_panel.gd
- Do NOT add emotional contagion (that's T-2018-04)
- Do NOT add mental break check/behavior (that's T-2018-05)
- Do NOT modify main.gd or SimulationBus signals
- Do NOT use `class_name` anywhere

## Acceptance Criteria
- [ ] `scripts/systems/emotion_system.gd` fully rewritten with all code above
- [ ] Extends `"res://scripts/core/simulation_system.gd"` (NOT SimulationSystem class_name)
- [ ] `init(entity_manager)` loads event presets from JSON
- [ ] `queue_event(entity_id, event_key, overrides)` public API for other systems
- [ ] `execute_tick()` processes all alive entities with 9 steps
- [ ] Fast layer: exponential decay with personality-adjusted half-lives
- [ ] Slow layer: OU process with personality-dependent baselines
- [ ] Memory traces: decay and pruning (< 0.5 intensity removed, max 20 per emotion)
- [ ] Opposite emotion inhibition (gamma = 0.3)
- [ ] Stress accumulation from negative emotions
- [ ] Legacy writeback: `entity.emotions` Dictionary updated each tick via `to_legacy_dict()`
- [ ] Box-Muller `_randfn()` implemented (no `randfn()` in Godot 4)
- [ ] No `class_name` declaration
- [ ] No GDScript parse errors
