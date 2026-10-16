extends RefCounted

## Capture-only clock. Automatic render-loop drawing must be disabled at launch.
var origin_frame: int = -1
var last_frame: int = -1
var steps: int = 0
var pre_draws: int = 0
var post_draws: int = 0
var valid: bool = true

func begin() -> void:
	valid = OS.get_cmdline_args().has("--disable-render-loop")
	RenderingServer.frame_pre_draw.connect(_pre)
	RenderingServer.frame_post_draw.connect(_post)

func schedule(frame: int) -> void:
	draw_frame.call_deferred(frame)

func draw_frame(frame: int) -> void:
	if origin_frame < 0:
		origin_frame = frame
		last_frame = frame - 1
	if frame <= last_frame:
		return
	if frame != last_frame + 1:
		valid = false
	last_frame = frame
	RenderingServer.force_draw(false, 1.0 / 60.0)
	steps += 1

func _pre() -> void:
	pre_draws += 1

func _post() -> void:
	post_draws += 1

func evidence() -> Dictionary:
	return {"automatic_loop_disabled": OS.get_cmdline_args().has("--disable-render-loop"),
		"origin_frame": origin_frame, "last_frame": last_frame, "manual_steps": steps,
		"step_seconds": 1.0 / 60.0, "pre_draws": pre_draws, "post_draws": post_draws,
		"engine_frames_drawn": Engine.get_frames_drawn(),
		"valid": valid and pre_draws == steps and post_draws == steps,
		"clock_source": "harness-owned positive render steps; no water masking"}
