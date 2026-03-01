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
const _PATHFIND_BATCH_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_batch",
]
const _PATHFIND_BATCH_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_batch",
]

var _native_checked: bool = false
var _native_bridge: Object = null
var _pathfind_method_name: String = ""
var _pathfind_batch_method_name: String = ""


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
				_pathfind_batch_method_name = _pick_method(
					_PATHFIND_BATCH_METHOD_CANDIDATES, ""
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
				_pathfind_batch_method_name = _pick_method(
					_PATHFIND_BATCH_METHOD_CANDIDATES, ""
				)
				return _native_bridge

	return null


func _prefer_gpu() -> bool:
	if not Engine.has_singleton("ComputeBackend"):
		return false
	var backend: Object = Engine.get_singleton("ComputeBackend")
	if backend == null:
		return false
	if not backend.has_method("resolve_mode"):
		return false
	if str(backend.call("resolve_mode")) != "gpu":
		return false

	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return false
	if bridge.has_method("has_gpu_pathfinding"):
		return bool(bridge.call("has_gpu_pathfinding"))
	return bridge.has_method("pathfind_grid_gpu")


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
