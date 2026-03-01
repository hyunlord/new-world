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


## Delegates full stress tick step (packed output) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_tick_step_packed(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	scalar_inputs: PackedFloat32Array,
	flags: PackedByteArray
):
	return _call_native_if_exists(
		"stat_stress_tick_step_packed",
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


## Delegates stress work efficiency curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_work_efficiency(
	stress: float,
	shaken_penalty: float
):
	return _call_native_if_exists(
		"stat_stress_work_efficiency",
		[
			stress,
			shaken_penalty
		]
	)


## Delegates stress personality scaling curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_personality_scale(
	values: PackedFloat32Array,
	weights: PackedFloat32Array,
	high_amplifies: PackedByteArray,
	trait_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_personality_scale",
		[
			values,
			weights,
			high_amplifies,
			trait_multipliers
		]
	)


## Delegates stress relationship scaling to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_relationship_scale(
	method: String,
	bond_strength: float,
	min_mult: float,
	max_mult: float
):
	return _call_native_if_exists(
		"stat_stress_relationship_scale",
		[
			method,
			bond_strength,
			min_mult,
			max_mult
		]
	)


## Delegates stress context scaling to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_context_scale(active_multipliers: PackedFloat32Array):
	return _call_native_if_exists(
		"stat_stress_context_scale",
		[
			active_multipliers
		]
	)


## Delegates stress emotion injection step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_emotion_inject_step(
	fast_current: PackedFloat32Array,
	slow_current: PackedFloat32Array,
	fast_inject: PackedFloat32Array,
	slow_inject: PackedFloat32Array,
	scale: float
):
	return _call_native_if_exists(
		"stat_stress_emotion_inject_step",
		[
			fast_current,
			slow_current,
			fast_inject,
			slow_inject,
			scale
		]
	)


## Delegates stress rebound queue step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_rebound_queue_step(
	amounts: PackedFloat32Array,
	delays: PackedInt32Array,
	decay_per_tick: float
):
	return _call_native_if_exists(
		"stat_stress_rebound_queue_step",
		[
			amounts,
			delays,
			decay_per_tick
		]
	)


## Delegates combined stress event scale step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_scale_step(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method: String,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_scale_step",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers
		]
	)


## Delegates combined stress event scale step (method code) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_scale_step_code(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method_code: int,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_scale_step_code",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method_code,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers
		]
	)


## Delegates combined stress event scale + emotion inject step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_inject_step(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method: String,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array,
	fast_current: PackedFloat32Array,
	slow_current: PackedFloat32Array,
	fast_inject: PackedFloat32Array,
	slow_inject: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_inject_step",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers,
			fast_current,
			slow_current,
			fast_inject,
			slow_inject
		]
	)


## Delegates combined stress event scale + emotion inject step (method code) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_inject_step_code(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method_code: int,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array,
	fast_current: PackedFloat32Array,
	slow_current: PackedFloat32Array,
	fast_inject: PackedFloat32Array,
	slow_inject: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_inject_step_code",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method_code,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers,
			fast_current,
			slow_current,
			fast_inject,
			slow_inject
		]
	)


## Delegates stress event final scaling to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_scaled(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	relationship_scale: float,
	context_scale: float,
	appraisal_scale: float
):
	return _call_native_if_exists(
		"stat_stress_event_scaled",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			relationship_scale,
			context_scale,
			appraisal_scale
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


## Delegates body age curve computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_compute_age_curve(axis: String, age_years: float):
	return _call_native_if_exists(
		"body_compute_age_curve",
		[
			axis,
			age_years
		]
	)


## Delegates batched body age curve computation to native bridge.
## Axis order: [str, agi, end, tou, rec, dr].
## Returns null when native bridge/method is unavailable.
func body_compute_age_curves(age_years: float):
	return _call_native_if_exists(
		"body_compute_age_curves",
		[
			age_years
		]
	)


## Delegates body training gain computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_calc_training_gain(
	potential: int,
	trainability: int,
	xp: float,
	training_ceiling: float,
	xp_for_full_progress: float
):
	return _call_native_if_exists(
		"body_calc_training_gain",
		[
			potential,
			trainability,
			xp,
			training_ceiling,
			xp_for_full_progress
		]
	)


## Delegates batched body training gain computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_calc_training_gains(
	potentials: PackedInt32Array,
	trainabilities: PackedInt32Array,
	xps: PackedFloat32Array,
	training_ceilings: PackedFloat32Array,
	xp_for_full_progress: float
):
	return _call_native_if_exists(
		"body_calc_training_gains",
		[
			potentials,
			trainabilities,
			xps,
			training_ceilings,
			xp_for_full_progress
		]
	)


## Delegates batched body realized value computation to native bridge.
## Axis order in output: [str, agi, end, tou, rec, dr].
## Returns null when native bridge/method is unavailable.
func body_calc_realized_values(
	potentials: PackedInt32Array,
	trainabilities: PackedInt32Array,
	xps: PackedFloat32Array,
	training_ceilings: PackedFloat32Array,
	age_years: float,
	xp_for_full_progress: float
):
	return _call_native_if_exists(
		"body_calc_realized_values",
		[
			potentials,
			trainabilities,
			xps,
			training_ceilings,
			age_years,
			xp_for_full_progress
		]
	)


## Delegates age-based body trainability modifier to native bridge.
## Returns null when native bridge/method is unavailable.
func body_age_trainability_modifier(axis: String, age_years: float):
	return _call_native_if_exists(
		"body_age_trainability_modifier",
		[
			axis,
			age_years
		]
	)


## Delegates batched age-based trainability modifiers to native bridge.
## Axis order: [str, agi, end, tou, rec].
## Returns null when native bridge/method is unavailable.
func body_age_trainability_modifiers(age_years: float):
	return _call_native_if_exists(
		"body_age_trainability_modifiers",
		[
			age_years
		]
	)


## Delegates body action energy cost computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_action_energy_cost(
	base_cost: float,
	end_norm: float,
	end_cost_reduction: float
):
	return _call_native_if_exists(
		"body_action_energy_cost",
		[
			base_cost,
			end_norm,
			end_cost_reduction
		]
	)


## Delegates body rest energy recovery computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_rest_energy_recovery(
	base_recovery: float,
	rec_norm: float,
	rec_recovery_bonus: float
):
	return _call_native_if_exists(
		"body_rest_energy_recovery",
		[
			base_recovery,
			rec_norm,
			rec_recovery_bonus
		]
	)


## Delegates thirst decay computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_thirst_decay(
	base_decay: float,
	tile_temp: float,
	temp_neutral: float
):
	return _call_native_if_exists(
		"body_thirst_decay",
		[
			base_decay,
			tile_temp,
			temp_neutral
		]
	)


## Delegates warmth decay computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_warmth_decay(
	base_decay: float,
	tile_temp: float,
	has_tile_temp: bool,
	temp_neutral: float,
	temp_freezing: float,
	temp_cold: float
):
	return _call_native_if_exists(
		"body_warmth_decay",
		[
			base_decay,
			tile_temp,
			has_tile_temp,
			temp_neutral,
			temp_freezing,
			temp_cold
		]
	)


## Delegates combined baseline need decay step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_base_decay_step_packed(
	scalar_inputs: PackedFloat32Array,
	flag_inputs: PackedByteArray
):
	return _call_native_if_exists(
		"body_needs_base_decay_step",
		[
			scalar_inputs,
			flag_inputs
		]
	)


## Delegates combined baseline need decay step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_base_decay_step(
	hunger_value: float,
	hunger_decay_rate: float,
	hunger_stage_mult: float,
	hunger_metabolic_min: float,
	hunger_metabolic_range: float,
	energy_decay_rate: float,
	social_decay_rate: float,
	safety_decay_rate: float,
	thirst_base_decay: float,
	warmth_base_decay: float,
	tile_temp: float,
	has_tile_temp: bool,
	temp_neutral: float,
	temp_freezing: float,
	temp_cold: float,
	needs_expansion_enabled: bool
):
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array()
	scalar_inputs.append(hunger_value)
	scalar_inputs.append(hunger_decay_rate)
	scalar_inputs.append(hunger_stage_mult)
	scalar_inputs.append(hunger_metabolic_min)
	scalar_inputs.append(hunger_metabolic_range)
	scalar_inputs.append(energy_decay_rate)
	scalar_inputs.append(social_decay_rate)
	scalar_inputs.append(safety_decay_rate)
	scalar_inputs.append(thirst_base_decay)
	scalar_inputs.append(warmth_base_decay)
	scalar_inputs.append(tile_temp)
	scalar_inputs.append(temp_neutral)
	scalar_inputs.append(temp_freezing)
	scalar_inputs.append(temp_cold)
	var flag_inputs: PackedByteArray = PackedByteArray()
	flag_inputs.append(1 if has_tile_temp else 0)
	flag_inputs.append(1 if needs_expansion_enabled else 0)
	return body_needs_base_decay_step_packed(scalar_inputs, flag_inputs)


## Delegates critical severity step for thirst/warmth/safety to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_critical_severity_step_packed(
	scalar_inputs: PackedFloat32Array
):
	return _call_native_if_exists(
		"body_needs_critical_severity_step_packed",
		[
			scalar_inputs
		]
	)


## Delegates critical severity step for thirst/warmth/safety to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_critical_severity_step(
	thirst: float,
	warmth: float,
	safety: float,
	thirst_critical: float,
	warmth_critical: float,
	safety_critical: float
):
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array()
	scalar_inputs.append(thirst)
	scalar_inputs.append(warmth)
	scalar_inputs.append(safety)
	scalar_inputs.append(thirst_critical)
	scalar_inputs.append(warmth_critical)
	scalar_inputs.append(safety_critical)
	return body_needs_critical_severity_step_packed(scalar_inputs)


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
