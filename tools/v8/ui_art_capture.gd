# Real-main-scene Part U screenshots, using existing people, catalogues, workshops and natural cards.
# Bridge methods: load_world (optional), advance_ticks (tooling), get_agent_snapshot/get_agent_detail,
# get_tech, get_workshops, get_content_catalog, get_time_snapshot, get_colony_overlay. No invented simulation objects.
# --verify-only validates controls/resources headlessly; never captures pixels.
# Run windowed under the prescribed GPU lock wrapper, with -- --out DIR [--save FILE] [--capture-size WIDTHxHEIGHT] --renderer=3d.
extends SceneTree

const Content = preload("res://scripts/ui/content_context.gd")
const Art = preload("res://scripts/ui/generated_art.gd")
var _main: Node
var _world: Node
var _out: String
var _records: Array[Dictionary] = []
var _failures: int = 0
var _verify_only: bool = false
var _capture_size: Vector2i = Vector2i.ZERO


func _init() -> void:
	call_deferred("_run")


func _arg(key: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var index: int = args.find(key)
	return args[index + 1] if index >= 0 and index + 1 < args.size() else ""


## Reject malformed and unbounded capture requests before creating the main scene.
static func parse_capture_size(value: String) -> Vector2i:
	var dimensions: PackedStringArray = value.split("x")
	if dimensions.size() != 2 or not dimensions[0].is_valid_int() or not dimensions[1].is_valid_int():
		return Vector2i.ZERO
	var width: int = dimensions[0].to_int()
	var height: int = dimensions[1].to_int()
	if width < 1 or height < 1 or width > 4096 or height > 4096:
		return Vector2i.ZERO
	return Vector2i(width, height)


func _frames(count: int = 12) -> void:
	for _i: int in count:
		await process_frame


func _check(message: String, passed: bool) -> void:
	print("UI_ART %s %s" % ["PASS" if passed else "FAIL", message])
	if not passed:
		_failures += 1


func _panel(name: String) -> Control:
	return _main.find_child(name, true, false) as Control


func _hide_panels() -> void:
	for name: String in ["AgentInspectorPanel", "Goals", "TechPanel", "BuildPanel", "WorkshopPanel", "CardPanel", "ItemPanel", "ChroniclePanel", "MigrantPanel"]:
		var panel: Control = _panel(name)
		if panel != null:
			panel.hide()


func _paths(node: Node) -> PackedStringArray:
	var paths: PackedStringArray = []
	if node is TextureRect and node.is_visible_in_tree() and node.texture != null:
		var path: String = node.texture.resource_path
		if path.begins_with(Art.ROOT):
			paths.append(path)
	if node is Button and node.is_visible_in_tree() and node.icon != null:
		var path: String = node.icon.resource_path
		if path.begins_with(Art.ROOT):
			paths.append(path)
	if node is RichTextLabel and node.is_visible_in_tree():
		var images := RegEx.new()
		images.compile("\\[img=288x162\\](res://assets/ui/generated/[^\\[]+)\\[/img\\]")
		for found: RegExMatch in images.search_all(node.text):
			paths.append(found.get_string(1))
	for child: Node in node.get_children():
		paths.append_array(_paths(child))
	return paths


func _shot(name: String, panel: Control) -> void:
	await _frames()
	if not _verify_only:
		await RenderingServer.frame_post_draw
	var paths: PackedStringArray = _paths(panel)
	_check(name + " has visible generated art", not paths.is_empty())
	if name in ["recipes", "items"]:
		var item_images: int = 0
		for image_path: String in paths:
			if image_path.begins_with(Art.ROOT + "items/"):
				item_images += 1
		_check(name + " has generated item icons", item_images > 0)
	if name == "items" and "--u2" in OS.get_cmdline_user_args():
		var illustrated: bool = false
		for image_path: String in paths:
			illustrated = illustrated or image_path.begins_with(Art.ROOT + "item_illustrations/")
		_check("items has separate painterly illustration", illustrated)
	var path: String = ""
	var image_size: Vector2i = Vector2i.ZERO
	if _capture_size != Vector2i.ZERO:
		_check(name + " requested window size", root.size == _capture_size)
	if not _verify_only:
		path = _out.path_join(name + ".png")
		var screenshot: Image = root.get_texture().get_image()
		image_size = screenshot.get_size()
		if _capture_size != Vector2i.ZERO:
			_check(name + " captured pixel size", image_size == _capture_size)
		_check(name + " saved", screenshot.save_png(path) == OK)
	var record: Dictionary = {"surface": str(panel.get_path()), "images": paths,
		"tick": SimReads.read(_world, "get_time_snapshot").get("tick", -1),
		"requested_size": [_capture_size.x, _capture_size.y], "window_size": [root.size.x, root.size.y],
		"viewport_size": [root.get_visible_rect().size.x, root.get_visible_rect().size.y]}
	if not _verify_only:
		record["file"] = path
		record["image_size"] = [image_size.x, image_size.y]
	_records.append(record)
	print(("UI_ART_VERIFY " if _verify_only else "UI_ART_SHOT ") + JSON.stringify(record))


func _run() -> void:
	_out = _arg("--out")
	_verify_only = "--verify-only" in OS.get_cmdline_user_args()
	if "--capture-size" in OS.get_cmdline_user_args():
		_capture_size = parse_capture_size(_arg("--capture-size"))
		if _capture_size == Vector2i.ZERO:
			push_error("--capture-size requires WIDTHxHEIGHT, each dimension 1..4096")
			quit(1)
			return
	if _out.is_empty() or (DisplayServer.get_name() == "headless" and not _verify_only):
		push_error("ui_art_capture requires --out and a real window")
		quit(1)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	await _frames()
	if _capture_size != Vector2i.ZERO:
		# Camera _ready fits the play window; apply capture dimensions only after that finishes.
		root.mode = Window.MODE_WINDOWED
		root.size = _capture_size
		await _frames()
		_check("post-ready window size", root.size == _capture_size)
	_world.set_process(false)
	_world.call("set_sim_speed", 0.0)
	if not _arg("--save").is_empty():
		var loaded: bool = bool(SimReads.mutate(_world, "load_world", [_arg("--save")]))
		_check("real save loaded", loaded)
		if not loaded:
			quit(1)
			return
	else:
		SimReads.mutate(_world, "advance_ticks", [600])
	await _frames(30)
	var w3d: Node = _main.get_node_or_null("World3D")
	if w3d != null:
		w3d.get_node("Sky").call("set_hour_override", 10.0)
	var people: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = people.get("ids", PackedInt64Array())
	_check("live person exists", not ids.is_empty())
	if not ids.is_empty():
		_hide_panels()
		var inspector: Control = _panel("AgentInspectorPanel")
		inspector.call("display_agent", _world.call("get_agent_detail", ids[0]))
		await _shot("person", inspector)
	_hide_panels()
	var goals: Control = _panel("Goals")
	goals.call("toggle_panel")
	await _shot("chapters", goals)
	_hide_panels()
	var tech: Control = _panel("TechPanel")
	tech.call("open")
	for row: Dictionary in Content.rows("knowledge"):
		if Art.texture("tech", str(row.get("id", ""))) != null:
			tech.call("select_node", int(row.get("code", -1)))
			break
	await _shot("tech", tech)
	_hide_panels()
	var build: Control = _panel("BuildPanel")
	build.call("open")
	for row: Dictionary in Content.rows("buildings"):
		if Art.texture("buildings", str(row.get("id", ""))) != null:
			build.call("_select", str(row["id"]), -1)
			break
	await _shot("building", build)
	_hide_panels()
	var shops: Dictionary = SimReads.read(_world, "get_workshops")
	var shop_ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	_check("real workshop exists", not shop_ids.is_empty())
	if not shop_ids.is_empty():
		var workshop: Control = _panel("WorkshopPanel")
		workshop.call("display_workshop", shop_ids[0])
		await _shot("workshop", workshop)
		workshop.call("_toggle_recipes")
		await _shot("recipes", workshop)
	_hide_panels()
	var overlay: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var item_xs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var item_ys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	_check("real item stack exists", not item_xs.is_empty() and not item_ys.is_empty())
	if not item_xs.is_empty() and not item_ys.is_empty():
		_main.get_node("ColonyView/OrderTool").call("_show_item_tile", Vector2i(item_xs[0], item_ys[0]))
		await _shot("items", _panel("ItemPanel"))
	_hide_panels()
	var cards: Control = _panel("CardPanel")
	for _step: int in 48:
		if cards.call("has_active_cards"):
			break
		SimReads.mutate(_world, "advance_ticks", [60])
		await _frames(20)
	_check("natural decision card exists", bool(cards.call("has_active_cards")))
	cards.show()
	await _shot("cards", cards)
	if "--u2" in OS.get_cmdline_user_args():
		await _u2_surfaces()
	var file: FileAccess = FileAccess.open(_out.path_join("verification.json" if _verify_only else "screenshots.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify({"surfaces" if _verify_only else "shots": _records, "failures": _failures, "verify_only": _verify_only}, "\t"))
	else:
		_check("screenshot manifest opened", false)
	print(("UI_ART_VERIFY_DONE surfaces=%d failures=%d" if _verify_only else "UI_ART_CAPTURE_DONE shots=%d failures=%d") % [_records.size(), _failures])
	quit(0 if _failures == 0 else 1)


## Wait for real native visitor events, without injecting art-only simulation facts.
func _u2_surfaces() -> void:
	var migrants: Control = _panel("MigrantPanel")
	_check("migrant panel exists", migrants != null)
	if migrants == null:
		return
	for _day: int in 16:
		if int(migrants.call("shown_wave")) >= 0:
			break
		SimReads.mutate(_world, "advance_ticks", [1440])
		await _frames(24)
	var wave: int = int(migrants.call("shown_wave"))
	_check("natural migrant wave exists", wave >= 0)
	if wave >= 0:
		_hide_panels()
		migrants.call("open_wave", wave)
		await _shot("migrants", migrants)
	_hide_panels()
	var chronicle: Control = _panel("ChroniclePanel")
	chronicle.call("toggle_visible")
	await _shot("chronicle", chronicle)
