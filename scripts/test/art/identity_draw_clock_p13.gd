extends RefCounted

## Capture-only clock. Automatic render-loop drawing must be disabled at launch.
var origin_frame: int = -1
var last_frame: int = -1
var steps: int = 0
var pre_draws: int = 0
var post_draws: int = 0
var valid: bool = true
var automatic_steps: int = 0
var initial_engine_frames_drawn: int = 0
var process_samples: Array[Dictionary] = []
var draw_events: Array[Dictionary] = []
var errors: Array[String] = []
var _samples_by_frame: Dictionary = {}
var _manual_draw: bool = false

func begin() -> void:
	initial_engine_frames_drawn = Engine.get_frames_drawn()
	if RenderingServer.is_render_loop_enabled() or initial_engine_frames_drawn != 0:
		_reject("render loop enabled or unobserved startup draws")
	RenderingServer.frame_pre_draw.connect(_pre)
	RenderingServer.frame_post_draw.connect(_post)

func schedule(frame: int, delta: float) -> void:
	var sample: Dictionary = {"frame": frame, "delta": delta, "time_scale": Engine.time_scale}
	if _samples_by_frame.has(frame) or frame != process_samples.size():
		_reject("process samples must start at zero and remain consecutive")
	if not is_finite(delta) or absf(delta - 1.0 / 60.0) > 0.000000001 or Engine.time_scale != 1.0 or RenderingServer.is_render_loop_enabled():
		_reject("process delta, time scale or render loop changed")
	process_samples.append(sample)
	_samples_by_frame[frame] = sample
	draw_frame.call_deferred(frame)

func draw_frame(frame: int) -> void:
	if origin_frame < 0:
		origin_frame = frame
		last_frame = frame - 1
	if frame <= last_frame:
		return
	if frame != last_frame + 1:
		_reject("manual draw schedule is discontinuous")
	if frame != Engine.get_process_frames() or not _samples_by_frame.has(frame):
		_reject("manual draw lacks matching actual process sample")
	last_frame = frame
	_manual_draw = true
	RenderingServer.force_draw(false, 1.0 / 60.0)
	_manual_draw = false
	steps += 1

func _pre() -> void:
	pre_draws += 1
	var frame: int = Engine.get_process_frames()
	var sample: Dictionary = _samples_by_frame.get(frame, {})
	if sample.is_empty() or RenderingServer.is_render_loop_enabled() or Engine.time_scale != 1.0:
		_reject("draw lacks process sample or loop/time scale changed")
	var step: float = 1.0 / 60.0 if _manual_draw else float(sample.get("delta", -1.0))
	draw_events.append({"kind": "manual" if _manual_draw else "automatic", "process_frame": frame, "step_seconds": step, "time_scale": Engine.time_scale})
	if not _manual_draw:
		automatic_steps += 1

func _post() -> void:
	post_draws += 1
	if post_draws > pre_draws:
		_reject("post draw without matching pre draw")

func _reject(reason: String) -> void:
	valid = false
	if not errors.has(reason):
		errors.append(reason)

func evidence() -> Dictionary:
	return {"schema_version": 2, "automatic_loop_disabled": not RenderingServer.is_render_loop_enabled(),
		"origin_frame": origin_frame, "last_frame": last_frame, "manual_steps": steps,
		"automatic_steps": automatic_steps, "initial_engine_frames_drawn": initial_engine_frames_drawn,
		"step_seconds": 1.0 / 60.0, "pre_draws": pre_draws, "post_draws": post_draws,
		"engine_frames_drawn": Engine.get_frames_drawn(),
		"process_samples": process_samples, "draw_events": draw_events, "errors": errors,
		"valid": valid and not RenderingServer.is_render_loop_enabled() and initial_engine_frames_drawn == 0 and pre_draws == steps + automatic_steps and post_draws == pre_draws and automatic_steps == Engine.get_frames_drawn(),
		"clock_source": "all manual and automatic render steps observed; no water masking"}
