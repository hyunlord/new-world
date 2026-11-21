# house_panel_capture.gd — INTEG1 house UI shots: loads a save, stops the clock, opens the inspector on a resident,
#   clicks its home line (RoomPanel.select_house) and saves the frame; then a lodger, a hut (house without rooms:
#   the house section alone) and a homeless person when the save has them. Prints `HOUSE_ROW <json>` per house, `HOUSE_SHOT <json>` per frame, `HOUSE_CAPTURE_DONE n=<count>`;
#   any failed check quits 1.
#   `Godot --path . --resolution 1920x1080 --script res://tools/v8/house_panel_capture.gd -- --save <wsave> --out <dir>
#    [--lang ko|en] [--house <id>] [--zoom 2] [--frames 60]` (3D needs the default Forward+ driver).
# Bridge methods called: load_world, get_agent_detail (via the inspector); via SimReads get_house_snapshot,
#   get_household_snapshot, get_agent_snapshot, get_time_snapshot.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const HouseText = preload("res://scripts/ui/panels/house_text.gd")

var _main: Node = null
var _world: Node = null
var _inspector: Control = null
var _rooms: Control = null
var _out: String = ""
var _lang: String = "ko"
var _settle: int = 60
var _shots: int = 0


func _init() -> void:
	call_deferred("_run")


func _arg(name: String, fallback: String = "") -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _fail(message: String) -> void:
	push_error("house_panel_capture: " + message)
	quit(1)


func _run() -> void:
	_out = _arg("--out")
	var save: String = _arg("--save")
	_lang = _arg("--lang", "ko")
	_settle = int(_arg("--frames", "60"))
	if _out == "" or save == "":
		_fail("--out and --save are required")
		return
	DirAccess.make_dir_recursive_absolute(_out)
	root.get_node("Locale").call("set_language", _lang)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	await _frames(10)
	if not bool(_world.call("load_world", save)):
		_fail("load_world failed: " + save)
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	await _frames(20)
	_inspector = _main.get_node("UI/AgentInspectorPanel") as Control
	_rooms = _main.get_node("UI/RoomPanel") as Control
	var houses: Array = HouseText.rows(_world)
	var resident: int = -1
	var lodger: int = -1
	var wanted: int = int(_arg("--house", "-1"))
	for row: Dictionary in houses:
		print("HOUSE_ROW %s" % JSON.stringify({"house": row["house_id"], "template": row["template_key"], "owner": row["owner_household"],
			"members": Array(row["household_member_ids"]), "residents": Array(row["resident_ids"]), "rooms": Array(row["room_ids"]),
			"capacity": row["capacity"], "first_completed_tick": row["first_completed_tick"], "status": row["status"]}))
		for id: int in row["resident_ids"] as PackedInt64Array:
			var family: bool = HouseText.is_family(_world, row, id)
			if family and resident < 0 and not (row["room_ids"] as PackedInt64Array).is_empty() and (wanted < 0 or int(row["house_id"]) == wanted):
				resident = id
			if not family and lodger < 0:
				lodger = id
	var homeless: int = -1
	for id: int in SimReads.read(_world, "get_agent_snapshot").get("agent_ids", PackedInt64Array()):
		if HouseText.home_of(_world, id).is_empty():
			homeless = id
			break
	print("HOUSE_CASES %s" % JSON.stringify({"houses": houses.size(), "resident": resident, "lodger": lodger, "homeless": homeless}))
	if resident < 0:
		_fail("no family resident in a house with rooms")
		return
	if not await _shot("resident", resident, true):
		return
	var hut_resident: int = -1
	for row: Dictionary in houses:
		if (row["room_ids"] as PackedInt64Array).is_empty() and not (row["resident_ids"] as PackedInt64Array).is_empty() \
				and (row["resident_ids"] as PackedInt64Array)[0] != lodger:
			hut_resident = (row["resident_ids"] as PackedInt64Array)[0]
			break
	if hut_resident >= 0 and not await _shot("hut", hut_resident, true):
		return
	if lodger >= 0 and not await _shot("lodger", lodger, true):
		return
	if homeless >= 0 and not await _shot("homeless", homeless, false):
		return
	print("HOUSE_CAPTURE_DONE n=%d" % _shots)
	quit(0)


## Inspect `agent`, click the home line when `housed`, check the panels and save `house_<lang>_<name>.png`.
func _shot(name: String, agent: int, housed: bool) -> bool:
	_rooms.call("close")
	_inspector.call("show_person", agent)
	await _frames(5)
	var link: LinkButton = _inspector.get("_house_link") as LinkButton
	var home: Dictionary = HouseText.home_of(_world, agent)
	var house: int = int((home.get("house", {}) as Dictionary).get("house_id", -1))
	if housed != (house >= 0) or link.disabled == housed:
		_fail("%s: home %d, link disabled %s" % [name, house, link.disabled])
		return false
	if housed:
		link.pressed.emit()
		await _frames(5)
		if int(_rooms.call("selected_house")) != house:
			_fail("%s: RoomPanel shows house %d, wanted %d" % [name, int(_rooms.call("selected_house")), house])
			return false
		_frame_house(HouseText.by_id(_world, house))
	await _frames(_settle)
	await RenderingServer.frame_post_draw
	var path: String = _out.path_join("house_%s_%s.png" % [_lang, name])
	root.get_viewport().get_texture().get_image().save_png(path)
	var house_text: String = ""
	if housed:
		house_text = ((_rooms.find_child("House", true, false) as Control).get_child(0) as Label).text
	print("HOUSE_SHOT %s" % JSON.stringify({"file": path, "agent": agent, "house": house, "line": link.text.replace("\u2060", ""),
		"title": house_text.replace("\u2060", ""), "tick": int(SimReads.read(_world, "get_time_snapshot").get("tick", 0))}))
	_shots += 1
	return true


## Centre the camera on house `house` (its first completed cell) at `--zoom` (default 2).
func _frame_house(house: Dictionary) -> void:
	var cells: PackedInt32Array = house.get("completed_cells", PackedInt32Array())
	if cells.size() < 2:
		return
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	var zoom: float = float(_arg("--zoom", "2"))
	camera.zoom = Vector2(zoom, zoom)
	var tile: Vector2 = Vector2(cells[0], cells[1])
	camera.position = tile * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
	camera.call("_clamp_position")
