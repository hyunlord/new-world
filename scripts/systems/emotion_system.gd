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

		# Step 1: Event -> emotion impulse (appraisal-based)
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

		# Step 11: Mental break check
		_check_mental_break(entity, dt_hours)

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

		# Appraisal -> 8 emotion impulses
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
# HEXACO -> Emotion Coupling
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
# Emotional Contagion
# ═══════════════════════════════════════════════════

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


# ═══════════════════════════════════════════════════
# Mental Break Detection
# ═══════════════════════════════════════════════════

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
			var start_behavior = BREAK_BEHAVIORS.get(break_type, {})
			ed.mental_break_remaining = float(start_behavior.get("duration_hours", 1.0))
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
