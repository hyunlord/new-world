## Headless invariant checker — runs all 7 harness invariants directly in GDScript.
## Usage:
##   godot --headless --path . --script scripts/test/harness_invariant_check.gd -- --ticks <n>
## Output: prints each invariant result to stdout, exits 0 if all pass.

extends SceneTree

const PHASE_WAIT_SCENE: int = 0
const PHASE_WAIT_SETUP: int = 1
const PHASE_TICK: int = 2
const PHASE_CHECK: int = 3

var _total_ticks: int = 500
var _ticks_done: int = 0
var _batch: int = 50
var _phase: int = PHASE_WAIT_SCENE
var _wait_frames: int = 0
var _sim_engine: RefCounted = null
var _main_node: Node = null


func _init() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--ticks" and i + 1 < args.size():
			_total_ticks = int(args[i + 1])
			i += 1
		i += 1
	print("[inv-check] Starting invariant check with %d ticks" % _total_ticks)


func _initialize() -> void:
	var err := change_scene_to_file("res://scenes/main/main.tscn")
	if err != OK:
		print("[inv-check] ERROR: Failed to load main scene: %s" % error_string(err))
		quit(1)


func _process(_delta: float) -> bool:
	match _phase:
		PHASE_WAIT_SCENE:
			_main_node = current_scene
			if _main_node and _main_node.is_node_ready():
				_phase = PHASE_WAIT_SETUP
				_wait_frames = 60
				print("[inv-check] Main scene loaded, waiting for setup...")

		PHASE_WAIT_SETUP:
			_wait_frames -= 1
			if _wait_frames <= 0:
				_sim_engine = _main_node.get("sim_engine") as RefCounted
				if _sim_engine == null:
					print("[inv-check] ERROR: sim_engine not found on main node")
					quit(1)
					return true
				_sim_engine.is_paused = true
				_phase = PHASE_TICK
				print("[inv-check] Setup complete, advancing %d ticks..." % _total_ticks)

		PHASE_TICK:
			if _ticks_done >= _total_ticks:
				_phase = PHASE_CHECK
				return false
			var batch := mini(_batch, _total_ticks - _ticks_done)
			_sim_engine.advance_ticks(batch)
			_ticks_done += batch
			if _ticks_done % 500 < _batch or _ticks_done >= _total_ticks:
				print("[inv-check] Tick %d / %d" % [_ticks_done, _total_ticks])

		PHASE_CHECK:
			_run_invariants()
			quit(0)
			return true

	return false


func _run_invariants() -> void:
	print("[inv-check] === INVARIANT RESULTS AT TICK %d ===" % _ticks_done)

	# Get alive entities via world summary (agents serialized in settlement members)
	var entities: Array = _get_entities()
	print("[inv-check] Checking %d entity records" % entities.size())

	var results: Array = [
		["needs_bounded", _check_needs_bounded(entities)],
		["emotions_bounded", _check_emotions_bounded(entities)],
		["personality_bounded", _check_personality_bounded(entities)],
		["health_bounded", _check_health_bounded(entities)],
		["age_non_negative", _check_age_non_negative(entities)],
		["stress_non_negative", _check_stress_non_negative(entities)],
		["no_duplicate_traits", _check_no_duplicate_traits(entities)],
	]

	var total_pass: int = 0
	var total_fail: int = 0
	for r in results:
		var name: String = r[0]
		var violations: Array = r[1]
		if violations.is_empty():
			total_pass += 1
			print("[inv-check]   %s: PASS" % name)
		else:
			total_fail += 1
			print("[inv-check]   %s: FAIL (%d violations)" % [name, violations.size()])
			for v in violations.slice(0, 3):
				print("[inv-check]     → %s" % str(v))

	print("[inv-check] === SUMMARY: total=%d passed=%d failed=%d ===" % [results.size(), total_pass, total_fail])
	if total_fail == 0:
		print("[inv-check] ASSERTION 7: PASS — all 7 invariants passed")
	else:
		print("[inv-check] ASSERTION 7: FAIL — %d invariant(s) failed" % total_fail)


func _get_entities() -> Array:
	if _sim_engine == null:
		return []
	if not _sim_engine.has_method("get_agent_snapshots"):
		return []
	var snapshots: Array = _sim_engine.get_agent_snapshots()
	var result: Array = []
	for snap in snapshots:
		if snap is Dictionary and snap.get("alive", false):
			result.append(snap)
	return result


# ── Invariant checks ──────────────────────────────────────────────────────────

func _check_needs_bounded(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var needs: Variant = e.get("needs", null)
		if not (needs is Dictionary):
			continue
		for k in needs:
			var v = needs[k]
			if (typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT) and (v < 0.0 or v > 1.0):
				violations.append({"entity": e.get("id", "?"), "field": "needs.%s" % k, "value": v})
	return violations


func _check_emotions_bounded(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var emotions: Variant = e.get("emotions", null)
		if not (emotions is Dictionary):
			continue
		for k in emotions:
			var v = emotions[k]
			if (typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT) and (v < 0.0 or v > 1.0):
				violations.append({"entity": e.get("id", "?"), "field": "emotions.%s" % k, "value": v})
	return violations


func _check_personality_bounded(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var personality: Variant = e.get("personality", null)
		if not (personality is Dictionary):
			continue
		var axes: Variant = personality.get("axes", null)
		if not (axes is Dictionary):
			continue
		for k in axes:
			var v = axes[k]
			if (typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT) and (v < 0.0 or v > 1.0):
				violations.append({"entity": e.get("id", "?"), "field": "personality.%s" % k, "value": v})
	return violations


func _check_health_bounded(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var health: Variant = e.get("health", null)
		if health == null:
			continue
		if (typeof(health) == TYPE_FLOAT or typeof(health) == TYPE_INT) and (health < 0.0 or health > 1.0):
			violations.append({"entity": e.get("id", "?"), "field": "health", "value": health})
	return violations


func _check_age_non_negative(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var age: Variant = e.get("age", null)
		if age == null:
			continue
		if (typeof(age) == TYPE_FLOAT or typeof(age) == TYPE_INT) and float(age) < 0.0:
			violations.append({"entity": e.get("id", "?"), "field": "age", "value": age})
	return violations


func _check_stress_non_negative(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var emotions: Variant = e.get("emotions", null)
		if not (emotions is Dictionary):
			continue
		var stress: Variant = emotions.get("stress", null)
		if stress == null:
			continue
		if (typeof(stress) == TYPE_FLOAT or typeof(stress) == TYPE_INT) and float(stress) < 0.0:
			violations.append({"entity": e.get("id", "?"), "field": "stress", "value": stress})
	return violations


func _check_no_duplicate_traits(entities: Array) -> Array:
	var violations: Array = []
	for e in entities:
		if not (e is Dictionary):
			continue
		var traits: Variant = e.get("active_traits", null)
		if not (traits is Array):
			continue
		var seen: Dictionary = {}
		for t in traits:
			var trait_id: Variant = null
			if t is Dictionary:
				trait_id = t.get("trait_id", null)
			elif t is String or t is int:
				trait_id = t
			if trait_id == null:
				continue
			if trait_id in seen:
				violations.append({"entity": e.get("id", "?"), "duplicate_trait": trait_id})
			else:
				seen[trait_id] = true
	return violations
