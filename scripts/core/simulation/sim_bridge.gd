extends Node

const _NATIVE_SINGLETON_CANDIDATES: Array[String] = [
	"WorldSimBridge",
	"SimBridgeNative",
	"RustBridge",
]
const _PATHFIND_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid",
	"pathfind",
]
const _PATHFIND_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu",
]
const _PATHFIND_XY_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_xy",
]
const _PATHFIND_XY_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_xy",
]
const _PATHFIND_BATCH_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_batch",
]
const _PATHFIND_BATCH_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_batch",
]
const _PATHFIND_BATCH_XY_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_batch_xy",
]
const _PATHFIND_BATCH_XY_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_batch_xy",
]
const _SET_PATHFIND_BACKEND_METHOD_CANDIDATES: Array[String] = [
	"set_pathfinding_backend",
]
const _GET_PATHFIND_BACKEND_METHOD_CANDIDATES: Array[String] = [
	"get_pathfinding_backend",
]
const _RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES: Array[String] = [
	"resolve_pathfinding_backend",
]

var _native_checked: bool = false
var _native_bridge: Object = null
var _pathfind_method_name: String = ""
var _pathfind_xy_method_name: String = ""
var _pathfind_batch_method_name: String = ""
var _pathfind_batch_xy_method_name: String = ""
var _set_pathfind_backend_method_name: String = ""
var _get_pathfind_backend_method_name: String = ""
var _resolve_pathfind_backend_method_name: String = ""
var _last_synced_pathfind_backend_mode: String = ""


## Delegates pathfinding to native bridge when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_x: int,
	from_y: int,
	to_x: int,
	to_y: int,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	var method_name: String = _pathfind_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_GPU_METHOD_CANDIDATES, _pathfind_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_x,
		from_y,
		to_x,
		to_y,
		max_steps
	)


## Delegates pathfinding with PackedInt32Array(x,y,...) output when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid_xy(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_x: int,
	from_y: int,
	to_x: int,
	to_y: int,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if _pathfind_xy_method_name == "":
		return null
	var method_name: String = _pathfind_xy_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_XY_GPU_METHOD_CANDIDATES, _pathfind_xy_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_x,
		from_y,
		to_x,
		to_y,
		max_steps
	)

## Delegates batch pathfinding to native bridge when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid_batch(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_points: PackedVector2Array,
	to_points: PackedVector2Array,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if _pathfind_batch_method_name == "":
		return null
	var method_name: String = _pathfind_batch_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_BATCH_GPU_METHOD_CANDIDATES, _pathfind_batch_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_points,
		to_points,
		max_steps
	)


## Delegates batch pathfinding with PackedInt32Array(x,y,...) points.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid_batch_xy(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_xy: PackedInt32Array,
	to_xy: PackedInt32Array,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if _pathfind_batch_xy_method_name == "":
		return null
	var method_name: String = _pathfind_batch_xy_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_BATCH_XY_GPU_METHOD_CANDIDATES, _pathfind_batch_xy_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_xy,
		to_xy,
		max_steps
	)


## Delegates LOG_DIMINISHING XP requirement curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_log_xp_required(
	level: int,
	base_xp: float,
	exponent: float,
	level_breakpoints: PackedInt32Array,
	breakpoint_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_log_xp_required",
		[
			level,
			base_xp,
			exponent,
			level_breakpoints,
			breakpoint_multipliers
		]
	)


## Delegates cumulative XP->level conversion to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_xp_to_level(
	xp: float,
	base_xp: float,
	exponent: float,
	level_breakpoints: PackedInt32Array,
	breakpoint_multipliers: PackedFloat32Array,
	max_level: int
):
	return _call_native_if_exists(
		"stat_xp_to_level",
		[
			xp,
			base_xp,
			exponent,
			level_breakpoints,
			breakpoint_multipliers,
			max_level
		]
	)


## Delegates skill XP progress computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_skill_xp_progress(
	level: int,
	xp: float,
	base_xp: float,
	exponent: float,
	level_breakpoints: PackedInt32Array,
	breakpoint_multipliers: PackedFloat32Array,
	max_level: int
):
	return _call_native_if_exists(
		"stat_skill_xp_progress",
		[
			level,
			xp,
			base_xp,
			exponent,
			level_breakpoints,
			breakpoint_multipliers,
			max_level
		]
	)


## Delegates SCURVE phase speed lookup to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_scurve_speed(
	current_value: int,
	phase_breakpoints: PackedInt32Array,
	phase_speeds: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_scurve_speed",
		[
			current_value,
			phase_breakpoints,
			phase_speeds
		]
	)


## Delegates natural need decay curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_need_decay(
	current: int,
	decay_per_year: int,
	ticks_elapsed: int,
	metabolic_mult: float,
	ticks_per_year: int
):
	return _call_native_if_exists(
		"stat_need_decay",
		[
			current,
			decay_per_year,
			ticks_elapsed,
			metabolic_mult,
			ticks_per_year
		]
	)


## Delegates continuous stress inputs from unmet needs to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_continuous_inputs(
	hunger: float,
	energy: float,
	social: float,
	appraisal_scale: float
):
	return _call_native_if_exists(
		"stat_stress_continuous_inputs",
		[
			hunger,
			energy,
			social,
			appraisal_scale
		]
	)


## Delegates Lazarus appraisal stress scale computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_appraisal_scale(
	hunger: float,
	energy: float,
	social: float,
	threat: float,
	conflict: float,
	support_score: float,
	extroversion: float,
	fear_value: float,
	trust_value: float,
	conscientiousness: float,
	openness: float,
	reserve_ratio: float
):
	return _call_native_if_exists(
		"stat_stress_appraisal_scale",
		[
			hunger,
			energy,
			social,
			threat,
			conflict,
			support_score,
			extroversion,
			fear_value,
			trust_value,
			conscientiousness,
			openness,
			reserve_ratio
		]
	)


## Delegates combined appraisal + continuous stress primary step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_primary_step(
	hunger: float,
	energy: float,
	social: float,
	threat: float,
	conflict: float,
	support_score: float,
	extroversion: float,
	fear_value: float,
	trust_value: float,
	conscientiousness: float,
	openness: float,
	reserve_ratio: float
):
	return _call_native_if_exists(
		"stat_stress_primary_step",
		[
			hunger,
			energy,
			social,
			threat,
			conflict,
			support_score,
			extroversion,
			fear_value,
			trust_value,
			conscientiousness,
			openness,
			reserve_ratio
		]
	)


## Delegates emotion-to-stress contribution computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_emotion_contribution(
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float
):
	return _call_native_if_exists(
		"stat_stress_emotion_contribution",
		[
			fear,
			anger,
			sadness,
			disgust,
			surprise,
			joy,
			trust,
			anticipation,
			valence,
			arousal
		]
	)


## Delegates stress recovery decay computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_recovery_value(
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool
):
	return _call_native_if_exists(
		"stat_stress_recovery_value",
		[
			stress,
			support_score,
			resilience,
			reserve,
			is_sleeping,
			is_safe
		]
	)


## Delegates emotion+recovery+delta(denial) combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_emotion_recovery_delta_step(
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float,
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	trace_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
):
	var emotion_inputs: PackedFloat32Array = PackedFloat32Array([
		fear,
		anger,
		sadness,
		disgust,
		surprise,
		joy,
		trust,
		anticipation,
		valence,
		arousal,
	])
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
		stress,
		support_score,
		resilience,
		reserve,
		continuous_input,
		trace_input,
		ace_stress_mult,
		trait_accum_mult,
		epsilon,
		denial_redirect_fraction,
		hidden_threat_accumulator,
		denial_max_accumulator,
	])
	var flags: PackedByteArray = PackedByteArray([
		1 if is_sleeping else 0,
		1 if is_safe else 0,
		1 if denial_active else 0,
	])
	return _call_native_if_exists(
		"stat_stress_emotion_recovery_delta_step",
		[
			emotion_inputs,
			scalar_inputs,
			flags
		]
	)


## Delegates combined trace+emotion+recovery+delta step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_trace_emotion_recovery_delta_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float,
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
):
	var emotion_inputs: PackedFloat32Array = PackedFloat32Array([
		fear,
		anger,
		sadness,
		disgust,
		surprise,
		joy,
		trust,
		anticipation,
		valence,
		arousal,
	])
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
		stress,
		support_score,
		resilience,
		reserve,
		continuous_input,
		ace_stress_mult,
		trait_accum_mult,
		epsilon,
		denial_redirect_fraction,
		hidden_threat_accumulator,
		denial_max_accumulator,
	])
	var flags: PackedByteArray = PackedByteArray([
		1 if is_sleeping else 0,
		1 if is_safe else 0,
		1 if denial_active else 0,
	])
	return _call_native_if_exists(
		"stat_stress_trace_emotion_recovery_delta_step",
		[
			per_tick,
			decay_rate,
			min_keep,
			emotion_inputs,
			scalar_inputs,
			flags
		]
	)


## Delegates full stress tick step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_tick_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	scalar_inputs: PackedFloat32Array,
	flags: PackedByteArray
):
	return _call_native_if_exists(
		"stat_stress_tick_step",
		[
			per_tick,
			decay_rate,
			min_keep,
			scalar_inputs,
			flags
		]
	)


## Delegates final stress delta step (including denial redirect) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_delta_step(
	continuous_input: float,
	trace_input: float,
	emotion_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	recovery: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
):
	return _call_native_if_exists(
		"stat_stress_delta_step",
		[
			continuous_input,
			trace_input,
			emotion_input,
			ace_stress_mult,
			trait_accum_mult,
			recovery,
			epsilon,
			denial_active,
			denial_redirect_fraction,
			hidden_threat_accumulator,
			denial_max_accumulator
		]
	)


## Delegates post-stress reserve/allostatic/state combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_post_update_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool,
	allostatic: float,
	avoidant_allostatic_mult: float
):
	return _call_native_if_exists(
		"stat_stress_post_update_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping,
			allostatic,
			avoidant_allostatic_mult
		]
	)


## Delegates post-update + resilience combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_post_update_resilience_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool,
	allostatic: float,
	avoidant_allostatic_mult: float,
	e_axis: float,
	c_axis: float,
	x_axis: float,
	o_axis: float,
	a_axis: float,
	h_axis: float,
	support_score: float,
	hunger: float,
	energy: float,
	scar_resilience_mod: float
):
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
		reserve,
		stress,
		resilience,
		stress_delta_last,
		float(gas_stage),
		allostatic,
		avoidant_allostatic_mult,
		e_axis,
		c_axis,
		x_axis,
		o_axis,
		a_axis,
		h_axis,
		support_score,
		hunger,
		energy,
		scar_resilience_mod,
	])
	var flags: PackedByteArray = PackedByteArray([
		1 if is_sleeping else 0,
	])
	return _call_native_if_exists(
		"stat_stress_post_update_resilience_step",
		[
			scalar_inputs,
			flags
		]
	)


## Delegates reserve + GAS stage transition step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_reserve_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool
):
	return _call_native_if_exists(
		"stat_stress_reserve_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping
		]
	)


## Delegates allostatic load update step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_allostatic_step(
	allostatic: float,
	stress: float,
	avoidant_allostatic_mult: float
):
	return _call_native_if_exists(
		"stat_stress_allostatic_step",
		[
			allostatic,
			stress,
			avoidant_allostatic_mult
		]
	)


## Delegates stress state + emotion-meta snapshot to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_state_snapshot(
	stress: float,
	allostatic: float
):
	return _call_native_if_exists(
		"stat_stress_state_snapshot",
		[
			stress,
			allostatic
		]
	)


## Delegates batch stress-trace decay/keep step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_trace_batch_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float
):
	return _call_native_if_exists(
		"stat_stress_trace_batch_step",
		[
			per_tick,
			decay_rate,
			min_keep
		]
	)


## Delegates stress resilience value computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_resilience_value(
	e_axis: float,
	c_axis: float,
	x_axis: float,
	o_axis: float,
	a_axis: float,
	h_axis: float,
	support_score: float,
	allostatic: float,
	hunger: float,
	energy: float,
	scar_resilience_mod: float
):
	return _call_native_if_exists(
		"stat_stress_resilience_value",
		[
			e_axis,
			c_axis,
			x_axis,
			o_axis,
			a_axis,
			h_axis,
			support_score,
			allostatic,
			hunger,
			energy,
			scar_resilience_mod
		]
	)


## Delegates SIGMOID_EXTREME influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_sigmoid_extreme(
	value: int,
	flat_zone_lo: int,
	flat_zone_hi: int,
	pole_multiplier: float
):
	return _call_native_if_exists(
		"stat_sigmoid_extreme",
		[
			value,
			flat_zone_lo,
			flat_zone_hi,
			pole_multiplier
		]
	)


## Delegates POWER influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_power_influence(value: int, exponent: float):
	return _call_native_if_exists("stat_power_influence", [value, exponent])


## Delegates THRESHOLD_POWER influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_threshold_power(
	value: int,
	threshold: int,
	exponent: float,
	max_output: float
):
	return _call_native_if_exists(
		"stat_threshold_power",
		[
			value,
			threshold,
			exponent,
			max_output
		]
	)


## Delegates LINEAR influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_linear_influence(value: int):
	return _call_native_if_exists("stat_linear_influence", [value])


## Delegates STEP influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_step_influence(
	value: int,
	threshold: int,
	above_value: float,
	below_value: float
):
	return _call_native_if_exists(
		"stat_step_influence",
		[
			value,
			threshold,
			above_value,
			below_value
		]
	)


## Delegates STEP_LINEAR influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_step_linear(
	value: int,
	below_thresholds: PackedInt32Array,
	multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_step_linear",
		[
			value,
			below_thresholds,
			multipliers
		]
	)


func _get_native_bridge() -> Object:
	if _native_checked:
		return _native_bridge
	_native_checked = true

	for i in range(_NATIVE_SINGLETON_CANDIDATES.size()):
		var singleton_name: String = _NATIVE_SINGLETON_CANDIDATES[i]
		if not Engine.has_singleton(singleton_name):
			continue
		var singleton_obj: Object = Engine.get_singleton(singleton_name)
		if singleton_obj == null:
			continue
		for j in range(_PATHFIND_METHOD_CANDIDATES.size()):
			var method_name: String = _PATHFIND_METHOD_CANDIDATES[j]
			if singleton_obj.has_method(method_name):
				_native_bridge = singleton_obj
				_pathfind_method_name = method_name
				_pathfind_xy_method_name = _pick_method(
					_PATHFIND_XY_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_method_name = _pick_method(
					_PATHFIND_BATCH_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_xy_method_name = _pick_method(
					_PATHFIND_BATCH_XY_METHOD_CANDIDATES, ""
				)
				_set_pathfind_backend_method_name = _pick_method(
					_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_get_pathfind_backend_method_name = _pick_method(
					_GET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_resolve_pathfind_backend_method_name = _pick_method(
					_RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				return _native_bridge

	for i in range(_NATIVE_SINGLETON_CANDIDATES.size()):
		var class_name: String = _NATIVE_SINGLETON_CANDIDATES[i]
		if not ClassDB.class_exists(class_name):
			continue
		var instance: Object = ClassDB.instantiate(class_name)
		if instance == null:
			continue
		for j in range(_PATHFIND_METHOD_CANDIDATES.size()):
			var method_name: String = _PATHFIND_METHOD_CANDIDATES[j]
			if instance.has_method(method_name):
				_native_bridge = instance
				_pathfind_method_name = method_name
				_pathfind_xy_method_name = _pick_method(
					_PATHFIND_XY_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_method_name = _pick_method(
					_PATHFIND_BATCH_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_xy_method_name = _pick_method(
					_PATHFIND_BATCH_XY_METHOD_CANDIDATES, ""
				)
				_set_pathfind_backend_method_name = _pick_method(
					_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_get_pathfind_backend_method_name = _pick_method(
					_GET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_resolve_pathfind_backend_method_name = _pick_method(
					_RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				return _native_bridge

	return null


func _prefer_gpu() -> bool:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return false

	_sync_pathfinding_backend_mode(bridge)
	var resolved_backend: String = _resolve_pathfinding_backend_mode(bridge)
	if resolved_backend != "":
		return resolved_backend == "gpu"

	if not Engine.has_singleton("ComputeBackend"):
		return false
	var backend: Object = Engine.get_singleton("ComputeBackend")
	if backend == null:
		return false
	if not backend.has_method("resolve_mode"):
		return false
	if str(backend.call("resolve_mode")) != "gpu":
		return false

	if bridge.has_method("has_gpu_pathfinding"):
		return bool(bridge.call("has_gpu_pathfinding"))
	return bridge.has_method("pathfind_grid_gpu")


func _resolve_desired_pathfinding_backend_mode() -> String:
	if not Engine.has_singleton("ComputeBackend"):
		return "auto"
	var backend: Object = Engine.get_singleton("ComputeBackend")
	if backend == null:
		return "auto"
	if not backend.has_method("get_mode"):
		return "auto"

	var mode: String = str(backend.call("get_mode"))
	if mode == "cpu":
		return "cpu"
	if mode == "gpu_force":
		return "gpu"
	return "auto"


func _sync_pathfinding_backend_mode(bridge: Object) -> void:
	var method_name: String = _set_pathfind_backend_method_name
	if method_name == "":
		method_name = _pick_method(_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, "")
		_set_pathfind_backend_method_name = method_name
	if method_name == "":
		return
	var desired_mode: String = _resolve_desired_pathfinding_backend_mode()
	if desired_mode == _last_synced_pathfind_backend_mode:
		return

	var applied: Variant = bridge.call(method_name, desired_mode)
	if applied is bool and not bool(applied):
		return
	_last_synced_pathfind_backend_mode = desired_mode


func _resolve_pathfinding_backend_mode(bridge: Object) -> String:
	var method_name: String = _resolve_pathfind_backend_method_name
	if method_name == "":
		method_name = _pick_method(_RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES, "")
		_resolve_pathfind_backend_method_name = method_name
	if method_name == "":
		return ""
	return str(bridge.call(method_name))


func _pick_method(candidates: Array[String], fallback: String) -> String:
	if _native_bridge == null:
		return fallback
	for i in range(candidates.size()):
		var name: String = candidates[i]
		if _native_bridge.has_method(name):
			return name
	return fallback


func _call_native_if_exists(method_name: String, args: Array):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if not bridge.has_method(method_name):
		return null
	return bridge.callv(method_name, args)
