# Purpose: capture the unchanged renderer from native-ASCII-verified saves at 64/32 pixels per tile.
# Bridge methods: load_world (via SimReads), get_time_snapshot, get_agent_snapshot, get_building_snapshot,
# get_colony_overlay, get_rooms, get_space, get_animals, get_activity_snapshot;
# inherited capture helpers read renderer positions. Loading is the only simulation lifecycle mutation.
extends "res://tools/v8/capture.gd"

const DATA: String = "/Users/rexxa/work/artenv1-data"
const SOURCE_SCENES: Array[String] = ["day6_noon", "day6_night", "winter_noon"]
const EXPECTED_TICKS: Dictionary = {"day6_noon": 7440, "day6_night": 6840, "winter_noon": 26160}
const HUD_HEIGHT: int = 112
const CAMP_SCREEN: Vector2 = Vector2(800, 506)


func _run() -> void:
	if _out.is_empty():
		push_error("baseline capture needs --out")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	root.content_scale_size = Vector2i(1600, 900)
	root.size = Vector2i(1600, 900)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	var reads: Node = main.get_node("SimReads")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	world.process_mode = Node.PROCESS_MODE_DISABLED
	for _i: int in 10:
		await process_frame
	var captures: Dictionary = {}
	for scene_id: String in SOURCE_SCENES:
		var save_path: String = DATA.path_join("scenes/rich_%s.save" % scene_id)
		var loaded: bool = bool(reads.call("mutate_raw", "load_world", [save_path]))
		if not loaded:
			push_error("baseline save load failed: " + save_path)
			quit(3)
			return
		for _i: int in 10:
			await process_frame
		var camp: Vector2i = _hearth(world)
		for pair: Array in [["1x", 4.0], ["half", 2.0]]:
			_frame(camera, Vector2(camp), float(pair[1]))
			camera.position.y -= (CAMP_SCREEN.y - 450.0) / float(pair[1])
			camera.reset_smoothing()
			camera.force_update_scroll()
			for _i: int in 30:
				await process_frame
			RenderingServer.force_draw(false)
			await RenderingServer.frame_post_draw
			var shot_name: String = "current_%s_%s.png" % [scene_id, str(pair[0])]
			var frame: Image = root.get_texture().get_image()
			if frame == null or frame.save_png(_out.path_join(shot_name)) != OK:
				push_error("baseline save image failed: " + shot_name)
				quit(4)
				return
			var clock: Dictionary = SimReads.read(world, "get_time_snapshot")
			if int(clock.get("tick", -1)) != int(EXPECTED_TICKS[scene_id]):
				push_error("baseline simulation advanced or loaded the wrong tick")
				quit(5)
				return
			var screen_camp: Vector2 = root.get_canvas_transform() * (Vector2(camp) * TILE + Vector2(8, 8))
			if screen_camp.distance_to(CAMP_SCREEN) > 0.01:
				push_error("baseline camp mapping differs from compositor contract")
				quit(6)
				return
			var hud: Image = frame.get_region(Rect2i(0, 0, 1600, HUD_HEIGHT))
			if hud.save_png(_out.path_join(shot_name.trim_suffix(".png") + "_hud.png")) != OK:
				push_error("baseline HUD save failed")
				quit(7)
				return
			var dump: Dictionary = _identified_dump(world)
			captures[shot_name] = {
				"scene_id": scene_id, "save_path": save_path,
				"clock": clock, "camp": [camp.x, camp.y],
				"camp_screen": [screen_camp.x, screen_camp.y], "hud_height": HUD_HEIGHT,
				"camera_zoom": float(pair[1]), "tile_pixels": 16.0 * float(pair[1]),
				"width": frame.get_width(), "height": frame.get_height(),
				"dump": dump, "rooms": SimReads.read(world, "get_rooms"),
				"space": SimReads.read(world, "get_space"), "animals": SimReads.read(world, "get_animals"),
				"overlay": SimReads.read(world, "get_colony_overlay"),
			}
			print("ENV1_BASELINE_SAVED ", shot_name)
	var output: FileAccess = FileAccess.open(_out.path_join("baseline_capture.json"), FileAccess.WRITE)
	if output == null:
		push_error("baseline manifest could not be opened")
		quit(8)
		return
	output.store_string(JSON.stringify(_json_value(captures), "\t"))
	print("ENV1_BASELINE_CAPTURE_DONE ", captures.size())
	quit(0)


func _identified_dump(world: Node) -> Dictionary:
	var dump: Dictionary = _dump(world)
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var activities: Dictionary = SimReads.read(world, "get_activity_snapshot")
	var activity_ids: PackedInt64Array = activities.get("ids", PackedInt64Array())
	var activity_kinds: PackedByteArray = activities.get("kinds", PackedByteArray())
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var entity_ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var carry_kinds: PackedInt32Array = snap.get("carry_kinds", PackedInt32Array())
	var tops: PackedByteArray = snap.get("tops", PackedByteArray())
	var people: Array = dump.get("people", [])
	for i: int in people.size():
		var person: Dictionary = people[i]
		var screen: Array = person.get("screen", [])
		var point: Vector2 = Vector2(screen[0], screen[1])
		person["agent_id"] = agent_ids[i]
		person["entity_bits"] = entity_ids[i]
		person["carry_kind"] = carry_kinds[i] if i < carry_kinds.size() else -1
		person["top"] = tops[i] if i < tops.size() else 0
		person["on_screen"] = Rect2(0, 0, 1600, 900).has_point(point)
		person["in_map_view"] = Rect2(0, HUD_HEIGHT, 1600, 900 - HUD_HEIGHT).has_point(point)
		var activity_row: int = activity_ids.find(entity_ids[i])
		person["activity_kind"] = activity_kinds[activity_row] if activity_row >= 0 else -1
	dump["people"] = people
	return dump


func _json_value(value: Variant) -> Variant:
	if value is Dictionary:
		var result: Dictionary = {}
		for key: Variant in value:
			result[str(key)] = _json_value(value[key])
		return result
	if value is Array or value is PackedByteArray or value is PackedInt32Array or value is PackedInt64Array or value is PackedStringArray or value is PackedFloat32Array or value is PackedFloat64Array:
		var result: Array = []
		for entry: Variant in value:
			result.append(_json_value(entry))
		return result
	return value
