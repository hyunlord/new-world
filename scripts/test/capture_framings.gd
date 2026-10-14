extends SceneTree

const Metadata = preload("res://scripts/test/art/capture_metadata.gd")
const CapturePlans = preload("res://scripts/test/art/capture_plans.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const SocietyTrace = preload("res://scripts/test/art/capture_society_trace.gd")
const ColdTrace = preload("res://scripts/test/art/capture_cold_trace.gd")
const FrameMeasure = preload("res://scripts/test/art/frame_measure_session.gd")
const EventTrace = preload("res://scripts/test/art/capture_event_trace_p8.gd")

const ORIGIN: Vector2 = Vector2(448, 28)
const TILE_SIZE: float = 16.0
const BASELINE: Array[Array] = [[0, "F1"], [0, "F2"], [1440, "F1"], [1440, "F2"]]
const FINAL: Array[Array] = [
	[1, "12:00", "F1"], [1, "12:00", "F2"],
	[2, "12:00", "F1"], [2, "12:00", "F2"], [2, "12:00", "F3"],
	[2, "12:00", "F4"], [2, "12:00", "F2A"], [2, "22:00", "F2N"],
	[6, "12:00", "F1"], [6, "12:00", "F2"],
]

var _out: String = ""
var _plan: String = ""
var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _manifest: Array[Dictionary] = []
var _selected_seeking: bool = false
var _clock_offset: int = 0
var _failed: bool = false
var _motion_before: Dictionary = {}
var _motion_start_tick: int = -1
var _motion_end_tick: int = -1
var _casts: Array[Dictionary] = []
var _all_casts: Array[Dictionary] = []
var _world_epoch: int = 0
var _world_label: String = "default"
var _row_options: Dictionary = {}
var _need_provenance: Dictionary = {}
var _last_cast_usec: int = 0
var _shot_elapsed_s: float = 0.0
var _original_id: int = -1
var _daughter_id: int = -1
var _death_trace: RefCounted = SocietyTrace.new()
var _run_id: String = "%d-%d" % [OS.get_process_id(), Time.get_ticks_usec()]
var _seed: int = 20261010
var _target_name: String = ""
var _plan_row: Array = []
var _cold_trace: RefCounted = ColdTrace.new()
var _event_trace: RefCounted = EventTrace.new()


func _init() -> void:
	if _capture_fixture_mode():
		call_deferred("_run")
		return
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = 0
	while i < args.size():
		if args[i] in ["--out", "--plan"] and i + 1 < args.size():
			if args[i] == "--out":
				_out = args[i + 1]
			else:
				_plan = args[i + 1]
			i += 2
		else:
			push_error("capture: unknown or missing argument: " + args[i])
			quit(2)
			return
	if _out.is_empty() or _plan not in ["baseline", "final", "art", "art_long", "art_society", "art_god", "calibration"]:
		push_error("capture: requires --out DIRECTORY --plan baseline|final|art|art_long|art_society|art_god|calibration")
		quit(2)
		return
	call_deferred("_run")


func _run() -> void:
	if DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(_out)) != OK:
		_fail("cannot create capture directory")
		return
	var old_manifest: String = ProjectSettings.globalize_path(_out.path_join("manifest.json"))
	if FileAccess.file_exists(old_manifest) and DirAccess.remove_absolute(old_manifest) != OK:
		_fail("cannot invalidate previous capture manifest")
		return
	if _plan == "calibration":
		await _capture_calibration()
		_write_manifest()
		return
	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	if packed == null:
		_fail("main scene not loadable")
		return
	_main = packed.instantiate()
	_world = _main.get_node_or_null("WorldSim")
	_camera = _main.get_node_or_null("Camera2D") as Camera2D
	if _world == null or _camera == null:
		_main.free()
		_fail("missing WorldSim or Camera2D")
		return
	# Freeze before adding the scene: even the first process frame is tick zero.
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	var start: Dictionary = _world.call("get_time_snapshot")
	if int(start["tick"]) != 0:
		_fail("initial simulation tick is not zero")
		return
	var info: Dictionary = _world.call("get_world_info")
	_clock_offset = int(info.get("clock_offset_ticks", 0))
	_sample_event_truth(_world.call("get_agent_snapshot"))
	if int(info.get("ticks_per_day", 0)) != 1440:
		_fail("clock plan requires 1440 ticks per day")
		return
	if _plan == "art_society" and OS.get_environment("CAPTURE_SOCIETY_PERF_ONLY") == "1":
		await _measure_society_pair()
		_write_manifest()
		return
	var plan: Array[Array] = BASELINE if _plan == "baseline" else (CapturePlans.ART if _plan == "art" else FINAL)
	if _plan == "art_long":
		plan = CapturePlans.ART_LONG
	if _plan == "art_god":
		plan = CapturePlans.ART_GOD
	if _plan == "art_society":
		plan = CapturePlans.ART_SOCIETY
	var override_path: String = OS.get_environment("CAPTURE_PLAN_JSON")
	if not override_path.is_empty():
		var override_rows: Variant = JSON.parse_string(FileAccess.get_file_as_string(override_path))
		if not override_rows is Array:
			_fail("CAPTURE_PLAN_JSON must contain an array of plan rows")
			return
		plan = []
		for row: Variant in override_rows:
			if not row is Array:
				_fail("capture plan row must be an array")
				return
			plan.append(row)
	if plan.is_empty():
		_fail("capture plan requires measured dates; supply CAPTURE_PLAN_JSON")
		return
	for shot: Array in plan:
		_plan_row = shot.duplicate(true)
		if shot.size() == 2 and str(shot[0]) == "RESET":
			await _reset_capture_world(int(shot[1]))
			continue
		_row_options = shot[8].duplicate(true) if shot.size() > 8 and shot[8] is Dictionary else {}
		_world_label = str(_row_options.get("paired_world", "default"))
		_target_name = str(shot[7]) if _plan == "art_god" and shot.size() > 7 else (str(shot[3]) if _plan == "art_society" and shot.size() > 3 else "")
		var target: int = int(shot[0])
		var framing: String = str(shot[1])
		var stem: String = "%s_t%d_%s" % [_plan, target, framing]
		if _plan != "baseline":
			var clock: PackedStringArray = str(shot[1]).split(":")
			target = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1]) - _clock_offset
			framing = str(shot[2])
			var prefix: String = "a_" if _plan == "art" else ("l_" if _plan == "art_long" else ("soc_" if _plan == "art_society" else ""))
			stem = prefix + "d%d_%s%s_%s" % [int(shot[0]), clock[0], clock[1], framing]
		if _plan == "art_god":
			stem = "god_" + stem + "_" + str(shot[6])
		if not _target_name.is_empty():
			stem += "_" + _target_name
		if _plan == "art_society" and shot.size() > 4:
			stem += "_selected_" + str(shot[4])
		if _row_options.has("stem_override"):
			stem = "god_" + str(_row_options["stem_override"])
		print("capture start: %s tick=%d usec=%d" % [stem, target, Time.get_ticks_usec()])
		var before: Dictionary = _world.call("get_time_snapshot")
		var delta_ticks: int = target - int(before["tick"])
		if delta_ticks < 0 or not _advance_to(target):
			_fail("simulation did not reach requested tick")
			return
		Selection.clear()
		var selection_target: String = str(shot[4]) if _plan == "art_society" and shot.size() > 4 else (_target_name if _plan == "art_god" else "")
		if selection_target in ["orig", "daughter"]:
			var selected: int = _settlement_id(selection_target)
			if selected < 0 and not _is_paired():
				_fail("selected settlement unavailable: " + selection_target)
				return
			if selected >= 0:
				Selection.select(selected)
		var tile: Vector2 = Vector2(32, 32) if framing == "F1" else _village_tile()
		if _target_name in ["orig", "daughter"] or framing == "FS":
			tile = _settlement_tile("daughter" if framing == "FS" else _target_name)
		if _target_name == "leader" or framing == "FD":
			tile = _founder_tile()
		if _target_name == "all" or framing == "FV":
			tile = _settlement_bounds().get_center()
		if framing == "FT":
			tile = _founder_site_tile()
		if framing == "DE":
			tile = _recent_exposure_tile()
		if tile.x < 0 and _is_paired():
			tile = _village_tile()
		if tile.x < 0:
			_fail("framing %s target %s unavailable (FD requires a living founder leader)" % [framing, _target_name])
			return
		if framing == "F3":
			tile = _food_tile(tile)
			if tile.x < 0:
				_fail("F3 requires a food source; none exists at this tick")
				return
		if framing == "F5":
			tile = _deadwood_tile(tile)
			if tile.x < 0:
				_fail("F5 requires a deadwood spot; none exists at this tick")
				return
		if framing == "FC":
			tile = CapturePlans.wood_carrier_tile(_world.call("get_agent_snapshot"))
			if tile.x < 0:
				_fail("FC requires a walking wood carrier; none exists at this tick")
				return
		_selected_seeking = false
		_set_agent_layers(framing != "F2A")
		var inspector: Node = _main.get_node_or_null("UI/AgentInspectorPanel")
		if inspector != null:
			inspector.call("close")
		if framing == "F4" and not _open_inspector(inspector):
			_fail("inspector capture requires a living agent and inspector")
			return
		_camera.position = ORIGIN + tile * TILE_SIZE + Vector2(8, 8)
		_camera.zoom = Vector2.ONE if framing == "F1" else (Vector2(2, 2) if framing == "F2W" else (Vector2(6, 6) if framing == "F2Z" else Vector2(4, 4)))
		if framing == "FV" or _target_name == "all":
			_camera.zoom = Vector2.ONE * _all_villages_zoom()
		_camera.reset_smoothing()
		_camera.force_update_scroll()
		# Reconcile infrequent object renderers after synchronous tooling ticks.
		for node_name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
			var renderer: Node = _main.get_node_or_null(node_name)
			if renderer != null and renderer.has_method("refresh_snapshot"):
				renderer.call("refresh_snapshot")
		if _plan == "art_god":
			if not str(shot[3]).is_empty():
				# Reconcile the pre-cast resource snapshot, so a windfall increase is observed.
				await create_timer(1.15).timeout
				if not _perform_capture_cast(shot, target):
					return
			if framing == "CAST":
				continue
			if str(shot[6]) == "panel_log":
				var panel: Node = _main.get_node_or_null("UI/GodHandPanel")
				if panel != null and panel.has_method("set_log_open"):
					panel.call("set_log_open", true)
			var wait_seconds: float = float(shot[5])
			if str(shot[6]) == "windfall_1000ms":
				wait_seconds = maxf(0.0, 1.0 - float(Time.get_ticks_usec() - _last_cast_usec) / 1000000.0)
			if wait_seconds > 0.0:
				await create_timer(wait_seconds).timeout
		else:
			await create_timer(1.15).timeout
		for frame: int in range(0 if _plan == "art_god" else 5):
			await process_frame
		await _render_frame()
		var now: Dictionary = _world.call("get_time_snapshot")
		if int(now["tick"]) != target:
			_fail("wall-clock advanced a frozen capture")
			return
		_shot_elapsed_s = float(Time.get_ticks_usec() - _last_cast_usec) / 1000000.0 if _last_cast_usec > 0 else 0.0
		var image: Image = root.get_texture().get_image()
		if image == null or image.get_width() < 1920 or image.get_height() < 1080 or image.get_width() * 9 != image.get_height() * 16:
			_fail("capture framebuffer must be at least 1920x1080 with 16:9 aspect; no placeholder is written")
			return
		var filename: String = stem + ".png"
		if image.save_png(ProjectSettings.globalize_path(_out.path_join(filename))) != OK:
			_fail("cannot save " + filename)
			return
		if not _write_entities(stem, target, image.get_size()):
			return
		print("capture saved: %s tick=%d usec=%d" % [stem, target, Time.get_ticks_usec()])
		_manifest.append({
			"file": filename, "tick": target, "framing": framing,
			"scenario": _scenario(),
			"seed": _seed, "capture_plan_row": _plan_row.duplicate(true),
			"intervention_casts": _casts.duplicate(true),
			"world_epoch": _world_epoch, "world_label": _world_label, "capture_options": _row_options.duplicate(true), "run_id": _run_id,
			"tile_x": tile.x, "tile_y": tile.y, "zoom": _camera.zoom.x,
			"time_of_day": now["time_of_day"], "width": image.get_width(), "height": image.get_height(),
			"entities": stem + ".entities.json",
		})
		if _plan == "art" and stem == "a_d2_1200_F2":
			_write_forest_evidence()
		if _plan == "art" and stem == "a_d2_1200_F1":
			await _capture_strip("a_strip", true)
			_camera.zoom = Vector2.ONE
			_camera.force_update_scroll()
			await _measure_frame_times()
		if _plan == "art" and stem == "a_d2_1930_F2":
			await _capture_strip("a_evening", false)
		if _plan == "art_long" and framing in ["F2N", "F2Z", "DE"]:
			await _capture_cold_cycle(stem)
	if _plan == "art":
		_main.queue_free()
		await process_frame
		await _capture_calibration()
	_write_manifest()


func _write_manifest() -> void:
	if _failed:
		quit(1)
		return
	var manifest: FileAccess = FileAccess.open(_out.path_join("manifest.json"), FileAccess.WRITE)
	if manifest == null:
		_fail("cannot write manifest")
		return
	manifest.store_string(JSON.stringify({
		"schema_version": 3, "plan": _plan,
		"scenario": _scenario(), "seed": _seed, "run_id": _run_id, "performance_only": OS.get_environment("CAPTURE_SOCIETY_PERF_ONLY") == "1",
		"intervention_casts": _all_casts, "git_sha": OS.get_environment("WORLDSIM_GIT_SHA"),
		"captures": _manifest, "worktree_changed_paths": int(OS.get_environment("WORLDSIM_GIT_DIRTY")),
	}, "\t") + "\n")
	manifest.close()
	quit(0)


func _village_tile() -> Vector2:
	var snap: Dictionary = _world.call("get_settlement_snapshot")
	var ids: PackedInt32Array = snap["settlement_ids"]
	var counts: PackedInt32Array = snap["member_counts"]
	var xs: PackedInt32Array = snap["formation_xs"]
	var ys: PackedInt32Array = snap["formation_ys"]
	var best: int = -1
	for i: int in range(ids.size()):
		if best < 0 or counts[i] > counts[best] or (counts[i] == counts[best] and ids[i] < ids[best]):
			best = i
	return Vector2(32, 32) if best < 0 else Vector2(xs[best], ys[best])


func _food_tile(village: Vector2) -> Vector2:
	var snap: Dictionary = _world.call("get_resource_snapshot")
	var kinds: PackedInt32Array = snap["kinds"]
	var xs: PackedInt32Array = snap["xs"]
	var ys: PackedInt32Array = snap["ys"]
	var best: Vector2 = Vector2(-1, -1)
	var distance: float = INF
	for i: int in range(kinds.size()):
		if kinds[i] != 0:
			continue
		var tile: Vector2 = Vector2(xs[i], ys[i])
		var candidate: float = tile.distance_squared_to(village)
		if candidate < distance or (candidate == distance and (tile.x < best.x or (tile.x == best.x and tile.y < best.y))):
			best = tile
			distance = candidate
	return best


func _deadwood_tile(village: Vector2) -> Vector2:
	return Metadata.deadwood_tile(_world.call("get_resource_snapshot"), village)


func _open_inspector(inspector: Node) -> bool:
	if inspector == null:
		return false
	var snap: Dictionary = _world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap["ids"]
	var agent_ids: PackedInt64Array = snap["agent_ids"]
	if ids.is_empty():
		return false
	var best: int = 0
	for i: int in range(agent_ids.size()):
		if agent_ids[i] < agent_ids[best]:
			best = i
	var detail: Dictionary = _world.call("get_agent_detail", ids[best])
	if not bool(detail.get("found", false)):
		return false
	_selected_seeking = int(detail.get("state_tag", -1)) == 1
	detail["entity_bits"] = ids[best]
	inspector.call("display_agent", detail)
	return true


func _fail(message: String) -> void:
	_failed = true
	push_error("capture: " + message)
	quit(1)


func _advance_to(target: int) -> bool:
	var clock: Dictionary = _world.call("get_time_snapshot")
	var tick: int = int(clock["tick"])
	var agents: Node = _main.get_node_or_null("AgentRenderer")
	var trail: Node = _main.get_node_or_null("ActivityTrailRenderer")
	var status: Node = _main.get_node_or_null("UI/HudStatusPanel")
	var motion_start: int = maxi(tick, target - 8)
	while tick < target:
		if tick == motion_start:
			_motion_before = _world.call("get_agent_snapshot")
			_motion_start_tick = tick
		# Observe each stock delta and the exact 20:00 closing boundary.
		tick = int(_world.call("advance_ticks", 1))
		_death_trace.ingest(_world.call("get_recent_deaths"))
		_remember_settlements()
		if status != null and status.has_method("_ingest_snapshots"):
			status.call("_ingest_snapshots")
		var snapshot: Dictionary = _world.call("get_agent_snapshot")
		_sample_event_truth(snapshot)
		_cold_trace.ingest(snapshot, tick)
		if agents != null and agents.has_method("ingest_chill_snapshot"):
			agents.call("ingest_chill_snapshot", snapshot, tick)
		if tick == target:
			_motion_end_tick = tick
		if trail != null:
			trail.call("_process", 0.0)
	return tick == target


func _sample_event_truth(agents: Dictionary) -> void:
	_event_trace.ingest(_world.call("get_time_snapshot"), _world.call("get_settlement_snapshot"), _world.call("get_society_snapshot"), _world.call("get_construction_snapshot"), _world.call("get_tallies"), _world.call("get_intervention_state"), agents)


func _set_agent_layers(shown: bool) -> void:
	Metadata.show_agents(_main, shown)
	for node_name: String in ["SeekVizRenderer", "NeedBarRenderer", "ActivityTrailRenderer", "DeathVizRenderer"]:
		var layer: CanvasItem = _main.get_node_or_null(node_name) as CanvasItem
		if layer != null:
			layer.visible = shown


func _write_entities(stem: String, tick: int, image_size: Vector2i) -> bool:
	var viewport: Vector2 = Vector2(image_size)
	var pixel_scale: Vector2 = viewport / root.get_visible_rect().size
	var rows: Array[Dictionary] = []
	rows = Metadata.entities(_main, viewport, pixel_scale)
	var inspector: Node = _main.get_node("UI/AgentInspectorPanel")
	_death_trace.ingest(_world.call("get_recent_deaths"))
	var agent_snapshot: Dictionary = _world.call("get_agent_snapshot")
	var data: Dictionary = {
		"schema_version": 6, "camera_position": [_camera.position.x, _camera.position.y],
		"zoom": [_camera.zoom.x, _camera.zoom.y], "viewport_size": [image_size.x, image_size.y],
		"clock_offset_ticks": _clock_offset, "tick": tick, "entities": rows,
		"framebuffer_scale": pixel_scale.x, "labels": Metadata.labels(_main.get_node_or_null("UI"), pixel_scale),
		"agent_snapshot": _json_arrays(agent_snapshot),
		"settlement_snapshot": _json_arrays(_world.call("get_settlement_snapshot")),
		"resource_snapshot": _json_arrays(_world.call("get_resource_snapshot")),
		"life_stage_counts": Metadata.life_stage_counts(agent_snapshot),
		"time_snapshot": _json_arrays(_world.call("get_time_snapshot")),
		"tallies": _json_arrays(_world.call("get_tallies")),
		"recent_deaths": _json_arrays(_world.call("get_recent_deaths")),
		"head_dots": _draw_counter("SeekVizRenderer", "get_drawn_head_dot_count"),
		"need_icons": _draw_counter("NeedBarRenderer", "get_drawn_icon_count"),
		"need_icon_agent_ids": _draw_counter("NeedBarRenderer", "get_drawn_icon_agent_ids"),
		"selected_entity_bits": int(inspector.call("get_selected_entity_bits")), "selected_seeking": _selected_seeking,
		"seek_segments": int(_main.get_node("SeekVizRenderer").call("get_drawn_segment_count")),
		"trail_segments": int(_main.get_node("ActivityTrailRenderer").call("get_drawn_segment_count")),
		"installed_groups": _installed_groups(),
	}
	data["scenario"] = _scenario()
	data["seed"] = _seed
	data["capture_plan_row"] = _plan_row.duplicate(true)
	data["selected_settlement_id"] = Selection.selected_id
	data["measured_days_source"] = "docs/art/production8/MEASURED_DAYS.md"
	data["society_snapshot"] = _json_arrays(_world.call("get_society_snapshot")) if _world.has_method("get_society_snapshot") else {}
	data["death_truth_trace"] = {"source": "independent FFI recent_deaths census", "max_sample_interval_ticks": _death_trace.max_interval_ticks, "window_start_tick": 0, "window_end_tick": tick, "deaths": _death_trace.rows()}
	data["death_drawn"] = _draw_counter("DeathVizRenderer", "capture_metadata")
	for death: Dictionary in data["death_drawn"]:
		var death_world: Vector2 = ORIGIN + Vector2(death["x"], death["y"]) * TILE_SIZE + Vector2(8, 8)
		var death_screen: Vector2 = (root.get_canvas_transform() * death_world) * pixel_scale
		death["screen_position"] = [death_screen.x, death_screen.y]
		death["in_view"] = Rect2(Vector2.ZERO, viewport).has_point(death_screen)
	data["feed_log"] = _draw_counter("UI/HudStatusPanel", "get_feed_log")
	data["feed_visible"] = _draw_counter("UI/HudStatusPanel", "get_visible_feed")
	data["feed_audit"] = _draw_counter("UI/HudStatusPanel", "capture_feed_metadata")
	data["event_truth_trace"] = _event_trace.metadata()
	data["hud_death_total"] = _draw_counter("UI/HudTopbar", "get_death_total")
	var society_layers: Dictionary = {}
	for layer: Node in get_nodes_in_group("art_society_layers"):
		if layer.has_method("capture_metadata"):
			society_layers.merge(layer.call("capture_metadata"))
	data["society_presentation"] = society_layers
	_enrich_society_view(society_layers, rows, viewport, pixel_scale)
	var five_deaths: Dictionary = {}
	for key: String in ["deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]:
		five_deaths[key] = data["tallies"].get(key, 0)
	data["death_tallies"] = five_deaths
	data["intervention_casts"] = _casts.duplicate(true)
	data["run_id"] = _run_id
	data["git_sha"] = OS.get_environment("WORLDSIM_GIT_SHA")
	data["world_epoch"] = _world_epoch
	data["world_label"] = _world_label
	data["capture_options"] = _row_options.duplicate(true)
	data["original_settlement_observation"] = _original_observation()
	data["seconds_since_last_cast"] = _shot_elapsed_s
	data["intervention_effects"] = {}
	for effects: Node in get_nodes_in_group("art_god_hand_effects"):
		data["intervention_effects"] = _json_arrays(effects.call("metadata"))
	data["intervention_state"] = _json_arrays(_world.call("get_intervention_state")) if _world.has_method("get_intervention_state") else {}
	data["building_snapshot"] = _json_arrays(_world.call("get_building_snapshot"))
	data["chill_tint_strength"] = _tint_strength()
	data["cold_truth_trace"] = {"source": "FFI colds sampled independently of presentation", "max_sample_interval_ticks": 1, "agents": _cold_trace.rows()}
	if tick == _motion_end_tick and _motion_start_tick >= 0:
		data["agent_motion_evidence"] = {
			"provenance": "observed_tile_displacement_during_final_synchronous_advance",
			"start_tick": _motion_start_tick, "end_tick": _motion_end_tick,
			"before": _json_arrays(_motion_before),
			"after_source": "agent_snapshot",
		}
	var file: FileAccess = FileAccess.open(_out.path_join(stem + ".entities.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot write entity sidecar")
		return false
	file.store_string(JSON.stringify(data, "\t") + "\n")
	file.close()
	return true


func _installed_groups() -> Variant:
	var path: String = "res://docs/art/production5/harvest/installed_groups_final.json"
	if not FileAccess.file_exists(path):
		return []
	return JSON.parse_string(FileAccess.get_file_as_string(path))


func _capture_cold_cycle(stem: String) -> void:
	var samples: Array[Dictionary] = []
	var best_alpha: float = 0.0
	var images: Array[Image] = []
	var clip_start_usec: int = Time.get_ticks_usec()
	for index: int in range(51):
		await _render_frame()
		var sampled_usec: int = Time.get_ticks_usec()
		var image: Image = root.get_texture().get_image()
		var size: Vector2 = Vector2(image.get_size())
		image.resize(1920, 1080, Image.INTERPOLATE_NEAREST)
		images.append(image)
		var rows: Array[Dictionary] = Metadata.entities(_main, size, size / root.get_visible_rect().size)
		var agents: Array[Dictionary] = []
		var shelters: Array[Dictionary] = []
		var hearths: Array[Dictionary] = []
		for row: Dictionary in rows:
			if row.get("kind") == "agent" and row.get("visible", false):
				agents.append({"id": row["id"], "cold": row.get("cold", false), "pose": row.get("pose", ""), "puff_alpha": row.get("puff_alpha", 0.0), "shiver_offset": row.get("shiver_offset", 0.0)})
				var alpha: float = float(row.get("puff_alpha", 0.0))
				if bool(row.get("cold", false)) and alpha > best_alpha:
					best_alpha = alpha
					root.get_texture().get_image().save_png(_out.path_join(stem + ".cold_peak.png"))
			if bool(row.get("visible", false)) and row.has("cold_sleepers"):
				shelters.append(row)
				var doorway_alpha: float = float(row.get("doorway_puff_alpha", 0.0))
				if int(row.get("cold_sleepers", 0)) > 0 and doorway_alpha > best_alpha:
					best_alpha = doorway_alpha
					root.get_texture().get_image().save_png(_out.path_join(stem + ".cold_peak.png"))
			if bool(row.get("visible", false)) and row.has("cold_ash"):
				hearths.append(row)
		samples.append({"real_time_usec": sampled_usec, "file": "%s.cold_cycle_%02d.png" % [stem, index], "agents": agents,
			"shelters": shelters, "hearths": hearths, "chill_tint_strength": _tint_strength()})
		var remaining: float = float(clip_start_usec + (index + 1) * 100000 - Time.get_ticks_usec()) / 1000000.0
		if remaining > 0.0:
			await create_timer(remaining).timeout
	for index: int in images.size():
		if images[index].save_png(_out.path_join(str(samples[index]["file"]))) != OK:
			_fail("cannot save cold cycle visual frame")
			return
	images.clear()
	if not CapturePlans.write_json(_out.path_join(stem + ".cold_cycle.json"), {"simulation_frozen": true, "requested_interval_s": 0.1, "visual_size": [1920, 1080], "samples": samples}):
		_fail("cannot write observed cold cycle")


func _measure_frame_times(filename: String = "frame_times.json", frozen: bool = false) -> void:
	DisplayServer.window_move_to_foreground()
	if OS.get_name() == "macOS":
		var activate: String = 'tell application "System Events" to set frontmost of first process whose unix id is %d to true' % OS.get_process_id()
		OS.execute("/usr/bin/osascript", PackedStringArray(["-e", activate]))
	var load_status: Dictionary = _performance_load_status()
	var start_tick: int = int(_world.call("get_time_snapshot")["tick"])
	var settlements: Dictionary = _world.call("get_settlement_snapshot")
	var elapsed: Array[float] = []
	# Forced viewport draw avoids an unbounded post-draw wait when macOS obscures the window.
	await _render_frame()
	if not frozen:
		_world.process_mode = Node.PROCESS_MODE_INHERIT
		_world.call("set_sim_speed", 1.0)
	var previous: int = Time.get_ticks_usec()
	for frame: int in range(300):
		await _render_frame()
		var current: int = Time.get_ticks_usec()
		_death_trace.ingest(_world.call("get_recent_deaths"))
		elapsed.append(float(current - previous) / 1000.0)
		previous = current
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	var ordered: Array[float] = elapsed.duplicate()
	ordered.sort()
	if not CapturePlans.write_json(_out.path_join(filename), {
		"metric": "process_frame plus force_draw wall-clock pacing diagnostic; not CPU cost",
		"pacing_changed": true, "historical_metric": "frame_post_draw wall-clock pacing",
		"frames": 300, "zoom": _camera.zoom.x, "forced_draw": true, "run_id": _run_id,
		"seed": _seed, "tick": start_tick, "alive_count": settlements.get("settlement_ids", []).size(), "load_status": load_status, "median_ms": (ordered[149] + ordered[150]) / 2.0,
		"p95_ms": ordered[284], "samples_ms": elapsed, "simulation_frozen": frozen,
	}):
		_fail("cannot write frame timing evidence")


func _capture_strip(prefix: String, follow: bool) -> void:
	var followed: int = CapturePlans.moving_agent(_world) if follow else -1
	var frames: Array[Dictionary] = []
	var images: Array[Image] = []
	var clip_start_usec: int = Time.get_ticks_usec()
	var original_position: Vector2 = _camera.position
	_camera.zoom = Vector2(6, 6) if follow else Vector2(4, 4)
	_world.process_mode = Node.PROCESS_MODE_INHERIT
	_world.call("set_sim_speed", 1.0)
	for index: int in range(40):
		var start_usec: int = Time.get_ticks_usec()
		if followed >= 0:
			var position: Vector2 = CapturePlans.agent_position(_main, followed)
			if position != Vector2.INF:
				_camera.position = position
		_camera.force_update_scroll()
		await _render_frame()
		var now: Dictionary = _world.call("get_time_snapshot")
		var stem: String = "%s_%02d" % [prefix, index]
		var image: Image = root.get_texture().get_image()
		if image == null:
			_fail("cannot read animation framebuffer")
			return
		images.append(image)
		if not _write_entities(stem, int(now["tick"]), image.get_size()):
			return
		var entry: Dictionary = {"file": stem + ".png", "entities": stem + ".entities.json",
			"tick": int(now["tick"]), "framing": prefix, "zoom": _camera.zoom.x,
			"width": image.get_width(), "height": image.get_height(),
			"real_time_usec": start_usec, "followed_agent_id": followed, "scenario": _scenario(), "seed": _seed}
		frames.append(entry)
		_manifest.append(entry)
		var remaining: float = float(clip_start_usec + (index + 1) * 100000 - Time.get_ticks_usec()) / 1000000.0
		if remaining > 0:
			await create_timer(remaining).timeout
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_camera.position = original_position
	# Encode after freezing so PNG compression cannot change the sampling cadence.
	for index: int in images.size():
		if images[index].save_png(ProjectSettings.globalize_path(_out.path_join(str(frames[index]["file"])))) != OK:
			_fail("cannot save animation frame")
			return
	images.clear()
	if not CapturePlans.write_json(_out.path_join("strip.json" if follow else "evening.json"), {
		"frames": frames, "followed_agent_id": followed, "moving_agent_found": followed >= 0 if follow else null,
		"requested_interval_s": 0.1, "speed": 1.0,
	}):
		_fail("cannot write animation manifest")


func _capture_calibration() -> void:
	var path: String = "res://scenes/test/art_calibration.tscn"
	if not ResourceLoader.exists(path):
		_fail("missing required art calibration scene")
		return
	var packed: PackedScene = load(path) as PackedScene
	var scene: Node = packed.instantiate()
	root.add_child(scene)
	current_scene = scene
	var camera: Camera2D = scene.get_node_or_null("Camera2D") as Camera2D
	if camera == null:
		_fail("calibration scene requires Camera2D")
		return
	for capture_spec: Array in [[4, false], [6, false], [4, true], [2, true]]:
		var zoom_level: int = int(capture_spec[0])
		var lineup: bool = bool(capture_spec[1])
		scene.call("set_lineup", lineup)
		camera.zoom = Vector2(zoom_level, zoom_level)
		camera.force_update_scroll()
		for frame: int in range(5):
			await process_frame
		await _render_frame()
		var image: Image = root.get_texture().get_image()
		var stem: String = ("a_lineup_z%d" if lineup else "a_calib_z%d") % zoom_level
		if image == null or image.save_png(ProjectSettings.globalize_path(_out.path_join(stem + ".png"))) != OK:
			_fail("cannot save calibration framebuffer")
			return
		var viewport: Vector2 = Vector2(image.get_size())
		var pixel_scale: Vector2 = viewport / root.get_visible_rect().size
		if not CapturePlans.write_json(_out.path_join(stem + ".entities.json"), {
			"schema_version": 3, "scenario": "CALIBRATION", "entities": Metadata.entities(scene, viewport, pixel_scale),
			"zoom": [zoom_level, zoom_level], "viewport_size": [image.get_width(), image.get_height()],
			"framebuffer_scale": pixel_scale.x,
		}):
			_fail("cannot write calibration metadata")
			return
		_manifest.append({"file": stem + ".png", "entities": stem + ".entities.json", "framing": "a_calib", "scenario": "CALIBRATION",
			"zoom": zoom_level, "width": image.get_width(), "height": image.get_height()})
	scene.queue_free()


func _json_arrays(value: Variant) -> Variant:
	if value is Dictionary:
		var converted: Dictionary = {}
		for key: Variant in value:
			converted[key] = _json_arrays(value[key])
		return converted
	if value is PackedByteArray or value is PackedInt32Array or value is PackedInt64Array or value is PackedFloat32Array or value is PackedFloat64Array or value is PackedStringArray:
		return Array(value)
	return value


func _write_forest_evidence() -> void:
	var evidence: Dictionary = {}
	for domain: String in ["terrain", "building", "construction", "settlement", "resource"]:
		evidence[domain] = _json_arrays(_world.call("get_" + domain + "_snapshot"))
	var viewport: Vector2 = Vector2(root.get_texture().get_size())
	evidence["entities"] = Metadata.entities(_main, viewport, viewport / root.get_visible_rect().size)
	evidence["tick"] = _world.call("get_time_snapshot")["tick"]
	if not CapturePlans.write_json(_out.path_join("forest_evidence.json"), evidence):
		_fail("cannot write forest evidence")


func _draw_counter(node_name: String, method: String) -> Variant:
	var node: Node = _main.get_node_or_null(NodePath(node_name))
	return node.call(method) if node != null and node.has_method(method) else null


func _tint_strength() -> float:
	for node: Node in Metadata.tagged_nodes(_main):
		if node.has_meta("night_chill_strength"):
			return float(node.get_meta("night_chill_strength"))
		if node.has_meta("chill_tint_strength"):
			return float(node.get_meta("chill_tint_strength"))
	return 0.0


func _render_frame() -> void:
	# macOS can suppress automatic draws for obscured windows while processing continues.
	# Explicitly draw the real viewport instead of waiting indefinitely for that signal.
	DisplayServer.window_move_to_foreground()
	await process_frame
	RenderingServer.force_draw(false)


func _scenario() -> String:
	if _plan == "art_god":
		return "INTERVENTION SCENARIO"
	return "SOCIETY SCENARIO seed %d" % _seed if _plan == "art_society" else "NATURAL SCENARIO"


func _reset_capture_world(seed: int) -> void:
	_seed = seed
	_world_epoch += 1
	_world_label = "default"
	_original_id = -1
	_daughter_id = -1
	Selection.reset()
	_casts.clear()
	_last_cast_usec = 0
	_motion_before.clear()
	_motion_start_tick = -1
	_motion_end_tick = -1
	_cold_trace = ColdTrace.new()
	_event_trace.reset()
	_death_trace.reset()
	_world.call("reset_world", seed)
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_sample_event_truth(_world.call("get_agent_snapshot"))
	var info: Dictionary = _world.call("get_world_info")
	_clock_offset = int(info.get("clock_offset_ticks", 0))
	await process_frame
	await process_frame
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false


func _remember_settlements() -> void:
	var snap: Dictionary = _world.call("get_settlement_snapshot")
	var ordered: Array = Array(snap.get("settlement_ids", PackedInt32Array()))
	ordered.sort()
	for id: int in ordered:
		if _original_id < 0:
			_original_id = id
		elif _daughter_id < 0 and id != _original_id:
			_daughter_id = id


func _settlement_id(target: String) -> int:
	_remember_settlements()
	return _original_id if target == "orig" else _daughter_id


func _settlement_tile(target: String) -> Vector2:
	var snap: Dictionary = _world.call("get_settlement_snapshot")
	var ids: PackedInt32Array = snap.get("settlement_ids", PackedInt32Array())
	var index: int = ids.find(_settlement_id(target))
	if index < 0:
		return Vector2(-1, -1)
	return Vector2(snap["formation_xs"][index], snap["formation_ys"][index])


func _settlement_bounds() -> Rect2:
	var snap: Dictionary = _world.call("get_settlement_snapshot")
	var xs: PackedInt32Array = snap.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("formation_ys", PackedInt32Array())
	if xs.is_empty():
		return Rect2(Vector2(32, 32), Vector2.ZERO)
	var bounds: Rect2 = Rect2(Vector2(xs[0], ys[0]), Vector2.ZERO)
	for index: int in xs.size():
		bounds = bounds.expand(Vector2(xs[index], ys[index]))
	return bounds


func _all_villages_zoom() -> float:
	var extent: Vector2 = (_settlement_bounds().size + Vector2(12, 12)) * TILE_SIZE
	var viewport: Vector2 = root.get_visible_rect().size
	for zoom_step: float in [8.0, 6.0, 4.0, 3.0, 2.0, 1.0]:
		if extent.x * zoom_step <= viewport.x and extent.y * zoom_step <= viewport.y:
			return zoom_step
	return 1.0


func _founder_tile() -> Vector2:
	if not _world.has_method("get_society_snapshot"):
		return Vector2(-1, -1)
	var society: Dictionary = _world.call("get_society_snapshot")
	var leaders: PackedByteArray = society.get("founder_leader", PackedByteArray())
	var founders: PackedInt64Array = society.get("founder_agent_ids", PackedInt64Array())
	var agents: Dictionary = _world.call("get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	for index: int in founders.size():
		if index < leaders.size() and leaders[index] == 1:
			var agent_index: int = ids.find(founders[index])
			if agent_index >= 0:
				return Vector2(agents["xs"][agent_index], agents["ys"][agent_index])
	return Vector2(-1, -1)


func _founder_site_tile() -> Vector2:
	if not _world.has_method("get_society_snapshot"):
		return Vector2(-1, -1)
	var society: Dictionary = _world.call("get_society_snapshot")
	var xs: PackedInt32Array = society.get("founder_site_xs", PackedInt32Array())
	var ys: PackedInt32Array = society.get("founder_site_ys", PackedInt32Array())
	return Vector2(xs[0], ys[0]) if not xs.is_empty() else Vector2(-1, -1)


func _recent_exposure_tile() -> Vector2:
	var deaths: Dictionary = _world.call("get_recent_deaths")
	var reasons: PackedInt32Array = deaths.get("reasons", PackedInt32Array())
	var ticks: PackedInt64Array = deaths.get("ticks", PackedInt64Array())
	var best: int = -1
	for index: int in reasons.size():
		if reasons[index] == 4 and int(deaths["current_tick"]) - ticks[index] < 120 and (best < 0 or ticks[index] > ticks[best]):
			best = index
	return Vector2(deaths["xs"][best], deaths["ys"][best]) if best >= 0 else Vector2(-1, -1)


func _enrich_society_view(presentation: Dictionary, entities: Array[Dictionary], viewport: Vector2, scale_factor: Vector2) -> void:
	var transform: Transform2D = root.get_canvas_transform()
	var view: Rect2 = Rect2(Vector2.ZERO, viewport)
	for group: Dictionary in presentation.get("founding_groups", []):
		var start: Array = group.get("start", [0, 0])
		var finish: Array = group.get("end", [0, 0])
		var start_screen: Vector2 = (transform * Vector2(start[0], start[1])) * scale_factor
		var site_screen: Vector2 = (transform * Vector2(finish[0], finish[1])) * scale_factor
		group["leader_screen"] = [start_screen.x, start_screen.y]
		group["site_screen"] = [site_screen.x, site_screen.y]
		group["leader_in_view"] = view.has_point(start_screen)
		group["site_in_view"] = view.has_point(site_screen)
		var edge: Array = group.get("edge_screen_position", [0, 0])
		var edge_screen: Vector2 = Vector2(edge[0], edge[1]) * scale_factor
		group["edge_framebuffer_position"] = [edge_screen.x, edge_screen.y]
		group["marker_in_view"] = (bool(group.get("stake_visible", false)) and view.has_point(site_screen)) or (bool(group.get("edge_visible", false)) and view.has_point(edge_screen))
		for bundle: Dictionary in group.get("bundles", []):
			bundle["agent_in_view"] = false
			for entity: Dictionary in entities:
				if entity.get("kind") == "agent" and int(entity.get("id", -1)) == int(bundle.get("agent_id", -2)):
					bundle["agent_in_view"] = entity.get("visible", false)
					bundle["agent_screen_rect"] = entity.get("rect", [])


func _performance_load_status() -> Dictionary:
	var uptime_output: Array = []
	var process_output: Array = []
	var uptime_code: int = OS.execute("/usr/bin/uptime", PackedStringArray(), uptime_output, true)
	var process_code: int = OS.execute("/usr/bin/pgrep", PackedStringArray(["-fl", "cargo|worldsim_|rustc|Godot"]), process_output, true)
	return {"unix_time": Time.get_unix_time_from_system(), "uptime": uptime_output, "uptime_exit_code": uptime_code, "processes": process_output, "pgrep_exit_code": process_code}


func _measure_society_pair() -> void:
	await _reset_capture_world(20261013)
	if not _advance_to(126994):
		_fail("A30 did not reach the measured first three-living-settlement tick")
		return
	var settlements: Dictionary = _world.call("get_settlement_snapshot")
	var alive: int = 0
	for count: int in settlements.get("member_counts", []):
		alive += int(count > 0)
	if alive != 3:
		_fail("A30 expected exactly three living settlements at tick 126994")
		return
	var previous_selection: int = Selection.selected_id
	Selection.select(_original_id)
	_camera.position = ORIGIN + _settlement_bounds().get_center() * TILE_SIZE + Vector2(8, 8)
	_camera.zoom = Vector2.ONE * _all_villages_zoom()
	_camera.force_update_scroll()
	DisplayServer.window_move_to_foreground()
	await create_timer(1.15).timeout
	await _render_frame()
	var backend = FrameMeasure.Runtime.new()
	backend.tree = self
	backend.world = _world
	backend.camera = _camera
	var load_before: Dictionary = _performance_load_status()
	var result: Dictionary = await FrameMeasure.collect(backend)
	Selection.select(previous_selection)
	result["selection_restored"] = Selection.selected_id == previous_selection
	result["seed"] = 20261013
	result["run_id"] = _run_id
	result["git_sha"] = OS.get_environment("WORLDSIM_GIT_SHA")
	result["load_before"] = load_before
	result["load_after"] = _performance_load_status()
	if not CapturePlans.write_json(_out.path_join("frame_times_soc_layers.json"), result):
		_fail("cannot write A30 CPU timing evidence; runtime state already restored")
	elif not bool(result["valid"]):
		_fail("invalid A30 CPU measurement: " + str(result["errors"]))


func _is_paired() -> bool:
	return str(_row_options.get("case_id", "")) == "a33" and str(_row_options.get("paired_world", "")) in ["P", "G"]


func _perform_capture_cast(shot: Array, tick: int) -> bool:
	var kind: String = str(shot[3])
	var requested_target: int = _settlement_id(_target_name) if _target_name in ["orig", "daughter"] else Selection.resolved_target()
	var observation: Dictionary = _original_observation()
	if _is_paired() and not bool(observation["need_provenance"]["valid"]):
		_fail("paired scarcity provenance no longer matches Rust source")
		return false
	var cast: Dictionary = {"kind": kind, "amount": int(shot[4]), "settlement_id": requested_target, "seed": _seed, "tick": tick, "world_epoch": _world_epoch, "world_label": _world_label, "capture_options": _row_options.duplicate(true), "run_id": _run_id, "precast_original": observation}
	var snapshot: Dictionary = _world.call("get_settlement_snapshot")
	var ids: PackedInt32Array = snapshot.get("settlement_ids", PackedInt32Array())
	if _is_paired() and (requested_target < 0 or not ids.has(requested_target)):
		cast["reply"] = {"ok": false, "applied": 0, "reason": "paired target unavailable"}
		cast["input_surface"] = "not invoked: missing paired target"
		_casts.append(cast)
		_all_casts.append(cast.duplicate(true))
		return true
	var panel: Node = _main.get_node_or_null("UI/GodHandPanel")
	if panel == null:
		_fail("god scenario requires UI/GodHandPanel")
		return false
	var presets: Dictionary = panel.get_script().get_script_constant_map().get("PRESETS", {})
	if int(presets.get(kind, -1)) != int(shot[4]):
		_fail("god scenario preset differs from panel: " + kind)
		return false
	var buttons: Dictionary = panel.get("_buttons")
	var button: Button = buttons.get(kind) as Button
	if button == null or button.disabled:
		_fail("god scenario button missing or disabled: " + kind)
		return false
	button.emit_signal("pressed")
	var reply: Dictionary = (panel.get("_reply") as Dictionary).duplicate(true)
	_sample_event_truth(_world.call("get_agent_snapshot"))
	_last_cast_usec = Time.get_ticks_usec()
	cast["reply"] = reply
	cast["input_surface"] = "GodHandPanel button pressed"
	_casts.append(cast)
	_all_casts.append(cast.duplicate(true))
	if not bool(reply.get("ok", false)) and not _is_paired():
		_fail("god scenario cast failed: " + kind)
		return false
	return true


func _source_number(path: String, expression: String) -> float:
	var pattern := RegEx.new()
	if pattern.compile(expression) != OK:
		return -1.0
	var matched: RegExMatch = pattern.search(FileAccess.get_file_as_string(path))
	return float(matched.get_string(1)) if matched != null else -1.0


func _load_need_provenance() -> Dictionary:
	if not _need_provenance.is_empty():
		return _need_provenance
	var economy: String = "res://rust/crates/sim-systems/src/runtime/settlement/food_economy.rs"
	var decision: String = "res://rust/crates/sim-systems/src/runtime/decision/agent_decision.rs"
	var rules: String = "res://rust/crates/sim-core/src/rules.rs"
	var growth: float = _source_number(economy, "pub fn adult_need_per_day\\(ticks_per_day: u64\\) -> f64 \\{\\s*([0-9.]+) \\* ticks_per_day as f64 / f64::from\\(HUNGER_CONSUME_AMOUNT\\)")
	var consume: float = _source_number(decision, "pub const HUNGER_CONSUME_AMOUNT: f32 = ([0-9.]+);")
	var days: float = _source_number(rules, "(?s)pub const V19_HAZARD: FissionHazard = FissionHazard \\{[^}]*?scarcity_days: ([0-9]+),")
	var info: Dictionary = _world.call("get_world_info")
	var ticks_per_day: int = int(info.get("ticks_per_day", 0))
	var valid: bool = growth > 0.0 and consume > 0.0 and days > 0.0 and ticks_per_day > 0
	_need_provenance = {"valid": valid, "source": "Rust adult_need_per_day and V19_HAZARD source literals; live world ticks_per_day", "growth_per_tick": growth, "food_consume_amount": consume, "ticks_per_day": ticks_per_day, "scarcity_days": days, "adult_need_per_day": growth * float(ticks_per_day) / consume if valid else null, "source_files": []}
	for path: String in [economy, decision, rules]:
		_need_provenance["source_files"].append({"path": path, "sha256": FileAccess.get_sha256(path)})
	return _need_provenance


func _original_observation() -> Dictionary:
	var original: int = _settlement_id("orig")
	var settlement: Dictionary = _world.call("get_settlement_snapshot")
	var society: Dictionary = _world.call("get_society_snapshot")
	var ids: PackedInt32Array = settlement.get("settlement_ids", PackedInt32Array())
	var society_ids: PackedInt32Array = society.get("settlement_ids", PackedInt32Array())
	var row: int = ids.find(original)
	var society_row: int = society_ids.find(original)
	var need: Dictionary = _load_need_provenance()
	var result: Dictionary = {"settlement_id": original, "found": row >= 0, "need_provenance": need.duplicate(true), "time_snapshot": _json_arrays(_world.call("get_time_snapshot"))}
	if row < 0:
		return result
	var members: int = int(settlement["member_counts"][row])
	var food: int = int(settlement["food_stocks"][row])
	result["members"] = members
	result["food"] = food
	result["adult_need_per_day"] = need["adult_need_per_day"]
	result["scarcity_days"] = need["scarcity_days"]
	result["scarcity_line"] = maxi(1, int(floor(float(members) * float(need["adult_need_per_day"]) * float(need["scarcity_days"])))) if bool(need["valid"]) else null
	result["below_scarcity_line"] = food < int(result["scarcity_line"]) if bool(need["valid"]) else null
	result["pressure"] = int(society["fission_pressures"][society_row]) if society_row >= 0 else null
	result["threshold"] = int(society["fission_thresholds"][society_row]) if society_row >= 0 else null
	return result


func _capture_fixture_mode() -> bool:
	return false
