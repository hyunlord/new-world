# world3d_shots.gd — M3D-1 side-by-side screens: one save, the same tiles and zooms, in the 2D or the 3D view.
# Purpose: `--save <wsave> --out <dir> --prefix <name> --shots "x,y,zoom[,yaw];…" [--hour H] [--frames N]`, plus
#   `--renderer=3d` (or 2d) in the user args, loads the save, stops the clock, frames each shot (Camera2D position and
#   zoom; in 3D also the yaw in 90° steps and a pinned hour) and saves `<prefix>_<i>_z<zoom>.png`. Prints
#   `W3D_SHOT <json>` per shot and `W3D_SHOTS_DONE n=<count>`; any failure quits 1.
#   `Godot --path . --rendering-driver opengl3 --resolution 1920x1080 --script res://tools/v8/world3d_shots.gd -- ...`
#   `--rooms 1` shows the room overlay (R); `--channel N` (3D) shows influence channel N (O pressed N+1 times);
#   `--advance T` runs the clock T ticks before the shots (both renderers replay the same world).
#   `--ui off` hides the HUD/panels CanvasLayer for clean world frames.
#   `--zones 1` toggles the automatic layout overlay with the player's Z key.
#   `--growth-check 1` proves saved growth in ko/en UI; `capture` also saves real panels (renderer/lock required).
#   (3D needs a real renderer: use the default Forward+ driver, not opengl3, for 3D shots.)
# Bridge methods called: load_world, advance_ticks, get_chronicle_since, get_building_cause (direct);
#   via SimReads get_time_snapshot/get_world_info/get_building_snapshot/get_colony_overlay;
#   OrderTool.area sends cmd_remove_zone_area; normal panels read their own Why data.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const GrowthText = preload("res://scripts/ui/chronicle_text.gd")
const OrderTool = preload("res://scripts/world/colony/order_tool.gd")

var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _settle: int = 60


func _init() -> void:
	call_deferred("_run")


func _arg(name: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else ""


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _fail(message: String) -> void:
	push_error("world3d_shots: " + message)
	quit(1)


func _run() -> void:
	var out: String = _arg("--out")
	var save: String = _arg("--save")
	if out == "" or save == "":
		_fail("--out and --save are required")
		return
	DirAccess.make_dir_recursive_absolute(out)
	if _arg("--frames") != "":
		_settle = int(_arg("--frames"))
	var prefix: String = _arg("--prefix") if _arg("--prefix") != "" else "shot"
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D")
	await _frames(10)
	if not bool(_world.call("load_world", save)):
		_fail("load_world failed: " + save)
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	await _frames(20)
	var w3d: Node = _main.get_node_or_null("World3D")
	if w3d != null and _arg("--hour") != "":
		w3d.get_node("Sky").call("set_hour_override", float(_arg("--hour")))
	# `--advance N` runs the clock N ticks first (e.g. into winter); the 2D and 3D runs replay the same world.
	if _arg("--advance") != "":
		_world.call("advance_ticks", int(_arg("--advance")))
		await _frames(10)
	if _arg("--growth-check") != "" and not await _growth_check(out, prefix):
		_fail("growth scene proof failed; see W3D_GROWTH evidence")
		return
	if _arg("--rooms") == "1":
		_main.get_node("RoomIndex").call("toggle")
	if w3d != null and _arg("--zones") == "1":
		var key: InputEventKey = InputEventKey.new()
		key.keycode = KEY_Z
		key.pressed = true
		w3d.get_node("Overlay").call("_unhandled_input", key)
	if w3d != null and _arg("--channel") != "":
		for _k: int in int(_arg("--channel")) + 1:
			w3d.get_node("Overlay").call("cycle_channel")
	if _arg("--ui") == "off":
		(_main.get_node("UI") as CanvasLayer).visible = false
	if _arg("--dump") != "":
		_dump()
	var n: int = 0
	for spec: String in _arg("--shots").split(";", false):
		var parts: PackedStringArray = spec.split(",")
		if parts.size() < 3:
			_fail("bad shot " + spec)
			return
		var tile: Vector2 = Vector2(float(parts[0]), float(parts[1]))
		var zoom: float = float(parts[2])
		var yaw_steps: int = int(parts[3]) if parts.size() > 3 else 0
		_camera.zoom = Vector2(zoom, zoom)
		_camera.position = tile * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
		_camera.call("_clamp_position")
		if w3d != null:
			var rig: Node = w3d.get_node("CameraRig")
			var current: float = float(rig.call("yaw"))
			var target: float = yaw_steps * PI * 0.5
			if absf(current - target) > 0.01:
				rig.call("rotate_view", roundi((target - current) / (PI * 0.5)))
			var terrain: Node = w3d.get_node("Terrain")
			terrain.call("build_all")
		await _frames(_settle)
		await RenderingServer.frame_post_draw
		var path: String = out.path_join("%s_%d_z%s.png" % [prefix, n, str(zoom).replace(".", "_")])
		root.get_viewport().get_texture().get_image().save_png(path)
		var note: Dictionary = {"file": path, "tile": [tile.x, tile.y], "zoom": zoom, "yaw": yaw_steps,
			"tick": int(SimReads.read(_world, "get_time_snapshot").get("tick", 0)), "renderer": "3d" if w3d != null else "2d"}
		print("W3D_SHOT %s" % JSON.stringify(note))
		n += 1
	print("W3D_SHOTS_DONE n=%d" % n)
	quit(0)


## Print where the built things are (template sites, workshops, rooms) to choose shot tiles.
func _dump() -> void:
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	print("W3D_LAYOUTS %s" % JSON.stringify(SimReads.read(_world, "get_colony_overlay").get("layout_zones", [])))
	for site: Variant in b.get("template_sites", []):
		var d: Dictionary = site
		print("W3D_YARD stage=%s props=%s" % [d.get("house_stage", 0), JSON.stringify(d.get("yard_props", []))])
		print("W3D_SITE %s %s at %d,%d %dx%d roofed=%s intact=%s" % [d.get("template_id"), d.get("status"), int(d.get("x")), int(d.get("y")), int(d.get("width")), int(d.get("height")), d.get("roofed"), d.get("is_intact")])
	print("W3D_BUILDINGS %s %s %s" % [b.get("kinds"), b.get("xs"), b.get("ys")])
	var w: Dictionary = SimReads.read(_world, "get_workshops")
	print("W3D_WORKSHOPS kinds=%s xs=%s ys=%s ws=%s hs=%s" % [w.get("kinds"), w.get("xs"), w.get("ys"), w.get("ws"), w.get("hs")])
	var r: Dictionary = SimReads.read(_world, "get_rooms")
	print("W3D_ROOMS ids=%s roles=%s roofed=%s cx=%s cy=%s tiles=%s" % [r.get("ids"), r.get("roles"), r.get("roofed"), r.get("centre_xs"), r.get("centre_ys"), r.get("tiles")])
	var a: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	print("W3D_AGENTS n=%d xs=%s ys=%s" % [(a.get("xs", PackedInt32Array()) as PackedInt32Array).size(), a.get("xs"), a.get("ys")])
	print("W3D_TIME %s" % JSON.stringify(SimReads.read(_world, "get_time_snapshot")))
	var an: Dictionary = SimReads.read(_world, "get_animals")
	print("W3D_ANIMALS names=%s xs=%s ys=%s" % [an.get("names"), an.get("xs"), an.get("ys")])
	var lv: Dictionary = SimReads.read(_world, "get_livestock")
	print("W3D_LIVESTOCK kinds=%s xs=%s ys=%s" % [lv.get("kinds"), lv.get("xs"), lv.get("ys")])
	var o: Dictionary = SimReads.read(_world, "get_outcrops")
	print("W3D_OUTCROPS %s %s %s" % [o.get("xs"), o.get("ys"), o.get("kinds")])
	var sp: Dictionary = SimReads.read(_world, "get_space")
	print("W3D_SPACE walls=%d styles=%s floors=%d" % [(sp.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size(), ",".join(Array(sp.get("wall_styles", PackedStringArray())).slice(0, 8)), (sp.get("floor_xs", PackedInt32Array()) as PackedInt32Array).size()])
	var cat: Dictionary = SimReads.read(_world, "get_content_catalog")
	for row: Variant in cat.get("outcrops", []):
		print("W3D_CAT_OUTCROP %s" % JSON.stringify(row))
	var shapes: Dictionary = {}
	for row: Variant in cat.get("items", []):
		shapes[str((row as Dictionary).get("art_shape", ""))] = str((row as Dictionary).get("id", ""))
	print("W3D_ITEM_SHAPES %s" % JSON.stringify(shapes))
	for row: Variant in cat.get("blueprints", []):
		print("W3D_BP %d %s %s %s" % [int(row.get("code", -1)), row.get("id"), row.get("construction_style"), row.get("completion_category")])


## Inspect real saved events and normal UI link actions; never manufacture a row or simulation fact.
func _growth_check(out: String, prefix: String) -> bool:
	var w3d: Node = _main.get_node_or_null("World3D")
	if w3d == null or (_arg("--growth-check") == "capture" and DisplayServer.get_name() == "headless"):
		return false
	var overlay: Node = w3d.get_node("Overlay")
	if bool(overlay.get("zones_shown")):
		return false
	var key: InputEventKey = InputEventKey.new()
	key.keycode = KEY_Z
	key.pressed = true
	for shown: bool in [true, false]:
		overlay.call("_unhandled_input", key)
		if bool(overlay.get("zones_shown")) != shown:
			return false
	var events: Dictionary = {}
	var cursor: int = 0
	while true:
		var page: Dictionary = _world.call("get_chronicle_since", cursor, 512)
		for row: Dictionary in GrowthText.rows(page):
			if int(row.get("kind", -1)) in [70, 71]:
				events[int(row["kind"])] = row
		var next: int = int(page.get("next", cursor))
		if next >= int(page.get("total", next)):
			break
		if next <= cursor:
			return false
		cursor = next
	var detail: Dictionary = (events.get(70, {}) as Dictionary).get("growth_evidence", {})
	if not events.has(71) or int(detail.get("zone_people", -1)) <= 0 or int(detail.get("zone_slots", -1)) < 0:
		return false
	var site: Dictionary = {}
	var cause: Dictionary = {}
	for candidate: Dictionary in SimReads.read(_world, "get_building_snapshot").get("template_sites", []):
		if not (candidate.get("yard_props", []) as Array).is_empty():
			var read: Dictionary = _world.call("get_building_cause", int(candidate["id"]), int(candidate["source_record"]))
			if bool(read.get("valid", false)) and read.has("zone_id") and read.get("cause_kind") == "steward":
				site = candidate
				cause = read
				break
	if site.is_empty():
		return false
	w3d.get_node("Terrain").call("build_all")
	await _frames(120)
	var yard_nodes: Array = w3d.get_node("Props").get("_yard_nodes")
	if yard_nodes.is_empty():
		return false
	var locale: Node = root.get_node("Locale")
	var original_language: String = str(locale.get("_current_lang"))
	var feed: Control = _main.get_node("UI/ChroniclePanel")
	var why: Control = _main.get_node("UI/WhyPanel")
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	for language: String in ["ko", "en"]:
		locale.call("set_language", language)
		if not feed.visible:
			feed.call("toggle_visible")
		feed.call("set_filter", 5)  # 마을 성장: growth lines however old
		await _frames(3)
		var stage_text: String = GrowthText.sentence(events[71], -1, locale, info)
		if not str(feed.call("plain_text")).contains(stage_text):
			print("W3D_GROWTH missing visible stage: ", stage_text)
			return false
		await _growth_capture(out.path_join(prefix + "_" + language + "_chronicle.png"))
		for kind: int in [70, 71]:
			feed.call("_follow_link", "why:%d" % int(events[kind]["index"]))
			var sentence: String = GrowthText.sentence(events[kind], -1, locale, info)
			var text: String = str(why.call("plain_text"))
			if not why.visible or not text.contains(sentence) or text.contains("CHRONICLE_") or text.contains("ZONE_CAUSE_"):
				return false
			print("W3D_GROWTH %s" % JSON.stringify({"locale": language, "kind": kind, "text": text, "evidence": events[kind].get("growth_evidence", {})}))
			await _growth_capture(out.path_join(prefix + "_" + language + "_why_%d.png" % kind))
		why.call("_follow_link", "building:%d:%d" % [int(site["id"]), int(site["source_record"])])
		var building_text: String = str(why.call("plain_text"))
		if not building_text.contains(GrowthText.zone_reason(cause, locale)) or not building_text.contains(GrowthText.loc(locale, "BUILD_CAUSE_STEWARD")):
			return false
		print("W3D_GROWTH_BUILDING %s" % JSON.stringify({"locale": language, "text": building_text, "cause": cause}))
		await _growth_capture(out.path_join(prefix + "_" + language + "_building_why.png"))
		why.call("close")
		feed.hide()
		var tool: Node = _main.get_node("ColonyView/OrderTool")
		var before: String = JSON.stringify(SimReads.read(_world, "get_colony_overlay").get("layout_zones", []))
		var tile: Vector2i = Vector2i(int(site["x"]), int(site["y"]))
		tool.call("set_mode", OrderTool.Mode.REMOVE_ZONE)
		var result: Dictionary = tool.call("area", tile, tile)
		var toast: Label = _main.get_node("UI/OrderBar").get("_toast")
		if not bool(result.get("automatic_zone_preserved", false)) or not toast.text.contains(str(locale.call("ltr", "MGROW_AUTOMATIC_ZONE_PRESERVED"))):
			return false
		if JSON.stringify(SimReads.read(_world, "get_colony_overlay").get("layout_zones", [])) != before:
			return false
		print("W3D_GROWTH_ZONE_PRESERVED %s" % JSON.stringify({"locale": language, "result": result, "text": toast.text}))
		await _growth_capture(out.path_join(prefix + "_" + language + "_zone_preserved.png"))
		tool.call("set_mode", OrderTool.Mode.NONE)
	feed.call("set_filter", 0)
	feed.hide()
	locale.call("set_language", original_language)
	print("W3D_GROWTH_PASS %s" % JSON.stringify({"save": _arg("--save"), "zone": detail, "site": site, "yard_nodes": yard_nodes.size(), "zones_default_off_toggle": true}))
	return true


func _growth_capture(path: String) -> void:
	if _arg("--growth-check") != "capture":
		return
	await _frames(5)
	await RenderingServer.frame_post_draw
	root.get_viewport().get_texture().get_image().save_png(path)
