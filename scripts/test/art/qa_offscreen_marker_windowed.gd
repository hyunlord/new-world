extends SceneTree
## Windowed real-society input check; parent runs through the window lock.
const Selection = preload("res://scripts/core/settlement_selection.gd")
const OUT_DEFAULT := "/Users/rexxa/work/art-production8-evidence/marker-input"
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var out: String = OS.get_environment("ART_MARKER_QA_OUT")
	if out.is_empty():
		out = OUT_DEFAULT
	DirAccess.make_dir_recursive_absolute(out)
	var main: Node = load("res://scenes/main.tscn").instantiate()
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	await process_frame
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	world.call("reset_world", 20261013)
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	var started: int = Time.get_ticks_msec()
	var batches: int = 0
	while int(world.call("get_time_snapshot")["tick"]) < 45991:
		var now: int = int(world.call("get_time_snapshot")["tick"])
		if batches >= 500 or Time.get_ticks_msec() - started > 900000:
			push_error("MARKER_WINDOWED bounded advance expired tick=%d batches=%d" % [now, batches])
			quit(1)
			return
		print("MARKER_WINDOWED batch start tick=", now, " elapsed_ms=", Time.get_ticks_msec() - started)
		world.call("advance_ticks", mini(1000, 45991 - now))
		var reached: int = int(world.call("get_time_snapshot")["tick"])
		print("MARKER_WINDOWED batch end tick=", reached, " elapsed_ms=", Time.get_ticks_msec() - started)
		if reached <= now:
			push_error("MARKER_WINDOWED advance made no progress")
			quit(1)
			return
		batches += 1
		await process_frame
	print("MARKER_WINDOWED reached walking frame")
	var camera := main.get_node("Camera2D") as Camera2D
	camera.position_smoothing_enabled = false
	camera.call("set_auto_track", false)
	camera.zoom = Vector2(4, 4)
	var groups: Node2D = main.get_node("AgentRenderer/FoundingGroups")
	var society: Dictionary = world.call("get_society_snapshot")
	Selection.observe(society, world.call("get_settlement_snapshot"))
	groups.call("refresh_snapshots", society, world.call("get_agent_snapshot"))
	var raw: Array = groups.get("_groups")
	if raw.is_empty():
		push_error("MARKER_WINDOWED no actual walking group at measured tick")
		quit(1)
		return
	camera.position = raw[0]["start"]
	camera.force_update_scroll()
	await _render_frame()
	var before: Dictionary = groups.call("capture_metadata")["founding_groups"][0]
	root.get_texture().get_image().save_png(out.path_join("before.png"))
	var passed: bool = bool(before.get("edge_visible", false)) and not bool(before.get("stake_visible", true))
	var edge: Array = before.get("edge_screen_position", [0, 0])
	var event := InputEventMouseButton.new()
	event.button_index = MOUSE_BUTTON_LEFT
	event.pressed = true
	event.position = Vector2(edge[0], edge[1])
	root.push_input(event, true)
	await _render_frame()
	var after: Dictionary = groups.call("capture_metadata")["founding_groups"][0]
	var target: Array = before["end"]
	passed = passed and camera.position.is_equal_approx(Vector2(target[0], target[1])) and bool(after.get("stake_visible", false)) and not bool(after.get("edge_visible", true))
	root.get_texture().get_image().save_png(out.path_join("after.png"))
	var founding_camera: Vector2 = camera.position
	var village: int = int(before["parent_id"])
	Selection.select(village)
	var labels: Node2D = main.get_node("SettlementLabels")
	var hearth: Vector2 = Vector2(448, 28) + Vector2(Selection.formation_for(village)) * 16.0 + Vector2(8, 8)
	camera.position = hearth + Vector2(400, 0)
	camera.force_update_scroll()
	labels.call("_update_labels")
	await _render_frame()
	var selected_before: Dictionary = labels.call("capture_metadata")["selected_marker"]
	root.get_texture().get_image().save_png(out.path_join("selected-before.png"))
	var selected_edge: Array = selected_before.get("edge_screen_position", [0, 0])
	var selected_event := InputEventMouseButton.new()
	selected_event.button_index = MOUSE_BUTTON_LEFT
	selected_event.pressed = true
	selected_event.position = Vector2(selected_edge[0], selected_edge[1])
	root.push_input(selected_event, true)
	await _render_frame()
	var selected_after: Dictionary = labels.call("capture_metadata")["selected_marker"]
	var selected_passed: bool = bool(selected_before.get("edge_visible", false)) and not selected_before.has("displayed_distance_tiles") and camera.position.is_equal_approx(hearth) and not bool(selected_after.get("edge_visible", true)) and Selection.selected_id == village
	passed = passed and selected_passed
	root.get_texture().get_image().save_png(out.path_join("selected-after.png"))
	var file := FileAccess.open(out.path_join("result.json"), FileAccess.WRITE)
	file.store_string(JSON.stringify({"passed": passed, "scenario": "SOCIETY SCENARIO seed 20261013", "tick": int(world.call("get_time_snapshot")["tick"]), "event": "Viewport.push_input InputEventMouseButton left pressed", "before": before, "after": after, "camera_after": [founding_camera.x, founding_camera.y], "selected_before": selected_before, "selected_after": selected_after, "selected_passed": selected_passed, "selected_camera_after": [camera.position.x, camera.position.y], "forced_draw": true}, "\t"))
	file.close()
	print("MARKER_WINDOWED passed=", passed, " out=", out)
	quit(0 if passed else 1)

func _render_frame() -> void:
	DisplayServer.window_move_to_foreground()
	await process_frame
	RenderingServer.force_draw(false)
