extends SceneTree

# Copy unchanged into the detached merge baseline; never install runtime changes there.
const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const FixedDelta = preload("res://scripts/test/art/test_fixed_delta_p9.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const CASES: Dictionary = {"W0": [20261010, 40560, 0], "W200": [15100, 1440, 200], "W-multi": [19400, 121080, 0], "W500_FROZEN": [15100, 1440, 500], "W500_2X": [15100, 1440, 500]}
const START_FRAME: int = 120
const WARMUP_FRAMES: int = 120
const MARGIN_PX: float = 64.0
signal frame_settled
var _main: Node
var _world: Node
var _reads: Node
var _renderer: Node
var _samples: int = 0
var _pre_draws: int = 0
var _post_draws: int = 0

func _init() -> void:
	RenderingServer.frame_pre_draw.connect(_on_pre_draw)
	RenderingServer.frame_post_draw.connect(_on_post_draw)
	call_deferred("_run")

func _run() -> void:
	var label: String = OS.get_environment("P12_RUNNING_WORLD")
	var output: String = OS.get_environment("P12_RUNNING_OUT")
	var commit: String = OS.get_environment("P12_BUILD_COMMIT")
	var library_hash: String = OS.get_environment("P12_BUILD_SHA256")
	if not CASES.has(label) or output.is_empty() or commit.is_empty() or library_hash.is_empty():
		_fail("requires P12_RUNNING_WORLD=W0|W200|W500_FROZEN|W500_2X, P12_RUNNING_OUT, P12_BUILD_COMMIT and P12_BUILD_SHA256")
		return
	var actual_library_hash: String = FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib")
	if actual_library_hash != library_hash:
		_fail("loaded debug-path bridge SHA does not match supplied provenance")
		return
	if DirAccess.make_dir_recursive_absolute(output) != OK:
		_fail("cannot create output directory")
		return
	root.size = Vector2i(1920, 1080)
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_main.process_mode = Node.PROCESS_MODE_DISABLED
	_world = _main.get_node("WorldSim")
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	_reads = _main.get_node("SimReads")
	if not bool(_reads.call("owns_world", _world)):
		_fail("SimReads must own the captured world; direct mutation fallback is forbidden")
		return
	var spec: Array = CASES[label]
	var pan: bool = label.begins_with("W500")
	var frozen: bool = label == "W500_FROZEN"
	var capture_frames: int = 240 if pan else 600
	var advance_every: int = 0 if frozen else (1 if pan else 2)
	var reset_ok: bool
	if int(spec[2]) > 0:
		reset_ok = bool(ReadRoute.mutate(_world, "reset_world_stress", [int(spec[0]), int(spec[2])]))
	else:
		reset_ok = bool(ReadRoute.mutate(_world, "reset_world", [int(spec[0])]))
	if not reset_ok or int(ReadRoute.mutate(_world, "advance_ticks", [int(spec[1])])) != int(spec[1]):
		_fail("world reset or initial advance failed")
		return
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	camera.call("set_auto_track", false)
	camera.position_smoothing_enabled = false
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	var settlements: Dictionary = _reads.call("settlement_snapshot")
	var ids: PackedInt32Array = settlements["settlement_ids"]
	if ids.is_empty():
		_fail("requires a settlement for deterministic camera")
		return
	var first: int = 0
	for index: int in ids.size():
		if ids[index] < ids[first]:
			first = index
	camera.position = Vector2(448, 28) + Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	var hearth: Vector2 = camera.position
	if pan:
		camera.position = hearth - Vector2(1200.0, 0.0)
	var camera_start: Vector2 = camera.position
	camera.zoom = Vector2(4, 4)
	camera.reset_smoothing()
	camera.force_update_scroll()
	Selection.set_layers_enabled(self, true)
	_renderer = _main.get_node("AgentRenderer")
	# Keep scene process clocks stopped through startup; compositor clocks remain observed separately.
	if Engine.get_process_frames() > START_FRAME:
		_fail("startup passed scheduled render start frame")
		return
	await _until_frame(START_FRAME)
	_main.process_mode = Node.PROCESS_MODE_INHERIT
	await _until_frame(START_FRAME + WARMUP_FRAMES)
	var file := FileAccess.open(output.path_join(label + ".jsonl"), FileAccess.WRITE)
	if file == null:
		_fail("cannot open dump")
		return
	var process_frames: Array[int] = []
	var ticks: Array[int] = []
	var populations: Array[int] = []
	var advance_calls: int = 0
	var render_start: Dictionary = _render_counts()
	var render_counts: Array[Dictionary] = []
	for index: int in capture_frames:
		if advance_every > 0 and index % advance_every == 0:
			if int(ReadRoute.mutate(_world, "advance_ticks", [1])) != int(spec[1]) + advance_calls + 1:
				_fail("advance failed")
				return
			advance_calls += 1
		if pan:
			camera.position = camera_start + Vector2(10.0 * float(index), 0.0)
			camera.force_update_scroll()
		await _until_frame(START_FRAME + WARMUP_FRAMES + index + 1)
		var time: Dictionary = _reads.call("time_snapshot")
		var expected_tick: int = int(spec[1]) + advance_calls
		if int(time["tick"]) != expected_tick:
			_fail("tick schedule mismatch at frame %d" % index)
			return
		process_frames.append(Engine.get_process_frames())
		ticks.append(int(time["tick"]))
		var agents: Dictionary = _reads.call("agent_snapshot")
		var agent_ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
		populations.append(agent_ids.size())
		render_counts.append(_render_counts())
		file.store_line(JSON.stringify({"frame": index + 1, "process_frame": Engine.get_process_frames(), "tick": time["tick"], "population": populations.back(), "camera": [camera.position.x, camera.position.y], "agents": _agents()}, "", true, true))
	file.close()
	var manifest: Dictionary = {
		"world": label, "seed": spec[0], "start_tick": spec[1], "end_tick": ticks.back(),
		"frames": capture_frames, "fixed_fps": 60, "advance_every_frames": advance_every,
		"mutation_route": "SimReads", "advance_calls": advance_calls, "process_frames": process_frames,
		"ticks": ticks, "populations": populations, "scope": "viewport_plus_64px",
		"pan": pan, "camera_start": [camera_start.x, camera_start.y], "hearth": [hearth.x, hearth.y],
		"pan_step_world_px": 10 if pan else 0, "speed": 0 if frozen else (2 if pan else 1),
		"simulation_schedule": "manual SimReads advance; 30 TPS at fixed 60 FPS",
		"render_counts": render_counts, "render_counts_start": render_start, "render_counts_end": _render_counts(),
		"collector_sha256": FileAccess.get_sha256("res://scripts/test/art/capture_running_identity_p12.gd"), "render_start_frame": START_FRAME, "warmup_frames": WARMUP_FRAMES,
		"validated_delta_samples": _samples, "debug_build": OS.is_debug_build(),
		"build_commit": commit, "build_sha256": library_hash,
		"world_info": _reads.call("world_info"), "camera": [camera.position.x, camera.position.y],
		"zoom": 4, "dump_sha256": FileAccess.get_sha256(output.path_join(label + ".jsonl")),
	}
	var metadata := FileAccess.open(output.path_join(label + ".json"), FileAccess.WRITE)
	if metadata == null:
		_fail("cannot open metadata")
		return
	metadata.store_string(JSON.stringify(manifest, "\t", true, true) + "\n")
	metadata.close()
	quit(0)

func _on_pre_draw() -> void:
	_pre_draws += 1

func _on_post_draw() -> void:
	_post_draws += 1

func _render_counts() -> Dictionary:
	return {"pre_draw": _pre_draws, "post_draw": _post_draws, "engine_frames_drawn": Engine.get_frames_drawn()}

func _agents() -> Array:
	var sprites: Dictionary = _renderer.get("_sprites")
	var ids: Array = sprites.keys()
	ids.sort()
	var result: Array = []
	for id: int in ids:
		var sprite: Sprite2D = sprites[id]
		var screen_rect: Rect2 = sprite.get_global_transform_with_canvas() * sprite.get_rect()
		if not root.get_visible_rect().grow(MARGIN_PX).intersects(screen_rect, true):
			continue
		var region: Rect2 = sprite.region_rect
		var texture_path: String = ""
		if sprite.texture is AtlasTexture:
			region = (sprite.texture as AtlasTexture).region
			texture_path = (sprite.texture as AtlasTexture).atlas.resource_path
		elif sprite.texture != null:
			texture_path = sprite.texture.resource_path
		result.append({
			"id": id, "position": [sprite.position.x, sprite.position.y],
			"texture": texture_path, "region": [region.position.x, region.position.y, region.size.x, region.size.y],
			"region_enabled": sprite.region_enabled, "flip_h": sprite.flip_h,
			"visible": sprite.visible, "visible_in_tree": sprite.is_visible_in_tree(),
			"modulate": _color(sprite.modulate), "self_modulate": _color(sprite.self_modulate),
			"z_index": sprite.z_index, "z_as_relative": sprite.z_as_relative,
			"children": _children(sprite, sprite),
		})
	return result

func _children(node: Node, origin: Node) -> Array:
	var result: Array = []
	for child: Node in node.get_children():
		if child is CanvasItem:
			var item: CanvasItem = child as CanvasItem
			result.append({"path": str(origin.get_path_to(child)), "visible": item.visible, "visible_in_tree": item.is_visible_in_tree()})
		result.append_array(_children(child, origin))
	return result

func _color(color: Color) -> Array:
	return [color.r, color.g, color.b, color.a]

func _until_frame(target: int) -> void:
	while Engine.get_process_frames() < target:
		await process_frame
		call_deferred("_settled")
		await frame_settled

func _settled() -> void:
	frame_settled.emit()

func _process(delta: float) -> bool:
	_samples += 1
	if not FixedDelta.accepts_delta(delta):
		_fail("requires --fixed-fps 60; delta=%.15f" % delta)
	return false

func _fail(message: String) -> void:
	push_error("running-identity-p12: " + message)
	quit(1)
