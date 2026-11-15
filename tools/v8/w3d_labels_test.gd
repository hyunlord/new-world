# w3d_labels_test.gd — headless test of the 3D map labels (World3D/LabelLayer/Labels, labels3d.gd; M3D-SWITCH E2).
# Purpose: on colony seed 0 at tick 1 (clock held, Korean), with a hut blueprint and a two-tile log-wall line placed
#   beside the hearth and the camera on the camp, the label layer draws exactly the texts the deleted 2D layers drew for
#   the same state (pinned below; each was derived from the 2D OverlayLayer / RoomLayer / AnimalLayer code at 1cdf9c98d):
#   1. the hut site's progress label without hover (UI_BLUEPRINT_LOGS in UI_BLUEPRINT_PROGRESS_FMT), and its bar;
#   2. a wall (space) site has no label until hovered, then its own progress label (UI_BLUEPRINT_MATERIALS);
#   3. a hovered resource node's "<name> · <item> N개 남음" (UI_NODE_LEFT_FMT), and an item stack's text when one lies
#      on the map;
#   4. a room's label at its centre while the room overlay is shown (RoomIndex.label_text; none while hidden);
#   5. a hovered wild animal's hover text (M5_ANIMAL_HOVER_FMT + C1_WILD_AGE) when an animal is on screen;
#   6. no label is drawn for an anchor behind the camera or off screen (every drawn anchor lies inside the viewport).
#   Prints `W3D_LABELS PASS|FAIL <name> <detail>` and `W3D_LABELS_DONE failures=N checks=M`; exits 1 on failure.
#   `godot --headless --path . --script res://tools/v8/w3d_labels_test.gd`
# Bridge methods called: via SimReads.mutate_raw reset_world, advance_ticks, cmd_place_blueprint, cmd_place_line;
#   via SimReads get_building_snapshot, get_colony_overlay, get_animals.
extends SceneTree

const Labels = preload("res://scripts/world3d/labels3d.gd")
const RoomIndex = preload("res://scripts/world/room_index.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")

var _failures: int = 0
var _checks: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool, detail: String = "") -> void:
	_checks += 1
	print("W3D_LABELS %s %s %s" % ["PASS" if ok else "FAIL", name, detail])
	if not ok:
		_failures += 1


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _texts(labels: Node, kind: String) -> Array[String]:
	var out: Array[String] = []
	for row: Dictionary in labels.call("drawn"):
		if str(row.get("kind", "")) == kind:
			out.append(str(row["text"]))
	return out


func _run() -> void:
	root.get_node("Locale").call("set_language", "ko")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	await _frames(5)
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	SimReads.mutate(world, "reset_world", [0])
	await _frames(3)
	SimReads.mutate(world, "advance_ticks", [1])
	var buildings: Dictionary = SimReads.read(world, "get_building_snapshot")
	var hearth: Vector2i = Vector2i(int((buildings.get("xs", PackedInt32Array()) as PackedInt32Array)[0]), int((buildings.get("ys", PackedInt32Array()) as PackedInt32Array)[0]))
	# The first free spots in rings around the hearth (deterministic order).
	var hut_at: Vector2i = Vector2i(-1, -1)
	var wall_at: Vector2i = Vector2i(-1, -1)
	var hut: Dictionary = {}
	var wall: Dictionary = {}
	for r: int in range(3, 9):
		for dy: int in range(-r, r + 1):
			for dx: int in range(-r, r + 1):
				var t: Vector2i = hearth + Vector2i(dx, dy)
				if hut_at.x < 0 and bool(world.call("can_place_blueprint", "hut", t.x, t.y)):
					hut = SimReads.mutate(world, "cmd_place_blueprint", ["hut", t.x, t.y])
					hut_at = t if bool(hut.get("accepted", false)) else hut_at
				elif wall_at.x < 0 and hut_at.x >= 0 and absi(dx) == r:
					var preview: Dictionary = world.call("get_place_preview", "log_wall", t.x, t.y, t.x + 1, t.y, 1)
					var ok: PackedByteArray = preview.get("ok", PackedByteArray())
					if ok.size() == 2 and ok[0] == 1 and ok[1] == 1:
						wall = SimReads.mutate(world, "cmd_place_line", ["log_wall", t.x, t.y, t.x + 1, t.y])
						wall_at = t if bool(wall.get("accepted", false)) else wall_at
	_check("fixture_orders_accepted", hut_at.x >= 0 and wall_at.x >= 0, "hut %s %s, wall %s %s" % [hut_at, hut, wall_at, wall])
	var camera: Camera2D = main.get_node("Camera2D")
	camera.call("center_on_tile", hearth)
	var w3d: Node = main.get_node("World3D")
	w3d.get_node("Terrain").call("build_all")
	var labels: Node = w3d.get_node("LabelLayer/Labels")
	labels.set("follow_mouse", false)
	labels.set("hover_tile", Vector2i(-1, -1))
	labels.set("hover_screen", Vector2(-1, -1))
	await _frames(40)
	var view: Rect2 = root.get_visible_rect()
	# 1 the hut site label (no hover).
	var locale: Node = root.get_node("Locale")
	var sites: Array[Dictionary] = Labels.site_rows(SimReads.read(world, "get_colony_overlay"))
	var hut_site: Dictionary = {}
	var wall_sites: Array[Dictionary] = []
	var other_sites: int = 0
	for site: Dictionary in sites:
		if EconomyText.is_space(int(site["kind"])):
			wall_sites.append(site)
		elif site["tile"] == hut_at:
			hut_site = site
		else:
			other_sites += 1
	# 2D: "%s · 작업 %d%%" with "통나무 %d/%d" (UI_BLUEPRINT_LOGS) for a hut (blueprint kind 0) — 0 delivered, 0 work.
	var hut_text: String = "통나무 0/%d · 작업 0%%" % int(hut_site.get("needed", -1))
	_check("hut_site_label", not hut_site.is_empty() and Labels.site_text(hut_site, locale) == hut_text and _texts(labels, "site").has(hut_text),
		"want %s; drawn %s" % [hut_text, _texts(labels, "site")])
	var unhovered: int = _texts(labels, "site").size()
	_check("space_site_no_label_unhovered", wall_sites.size() == 2 and unhovered == 1 + other_sites, "%d wall sites, %d other sites, drawn %s" % [wall_sites.size(), other_sites, _texts(labels, "site")])
	# 2 hover a wall site.
	if not wall_sites.is_empty():
		labels.set("hover_tile", wall_sites[0]["tile"])
		await _frames(2)
		# 2D: "재료 0/N · 작업 0%" (UI_BLUEPRINT_MATERIALS) for a non-hut site.
		var wall_text: String = str(locale.call("trf", "UI_BLUEPRINT_PROGRESS_FMT", {"logs": str(locale.call("trf", "UI_BLUEPRINT_MATERIALS", {"delivered": 0, "needed": int(wall_sites[0]["needed"])})), "percent": 0}))
		_check("space_site_label_on_hover", _texts(labels, "site").has(wall_text) and _texts(labels, "site").size() == unhovered + 1, "want %s; drawn %s" % [wall_text, _texts(labels, "site")])
	# 3 a resource node and an item stack under the mouse.
	var overlay: Dictionary = SimReads.read(world, "get_colony_overlay")
	var nxs: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var nleft: PackedInt32Array = overlay.get("node_left", PackedInt32Array())
	var node_index: int = _nearest(nxs, nys, hearth)
	if node_index >= 0:
		var t: Vector2i = Vector2i(nxs[node_index], nys[node_index])
		labels.set("hover_tile", t)
		await _frames(2)
		var row: Dictionary = ArtData.binding("nodes", nkinds[node_index])
		var node_text: String = EconomyText.trf(locale, "UI_NODE_LEFT_FMT", {"name": EconomyText.ltr(locale, str(row.get("name_key", "UI_UNKNOWN"))),
			"item": EconomyText.item(locale, int(row.get("yields", -1))), "left": nleft[node_index]})
		_check("node_hover_label", _texts(labels, "tile") == [node_text] and node_text.contains(str(nleft[node_index])), "tile %s want %s drawn %s" % [t, node_text, _texts(labels, "tile")])
	else:
		_check("node_hover_label", false, "no resource node on the map")
	var ixs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var item_index: int = _nearest(ixs, iys, hearth)
	if item_index >= 0:
		var t: Vector2i = Vector2i(ixs[item_index], iys[item_index])
		labels.set("hover_tile", t)
		await _frames(2)
		var kinds: PackedInt64Array = overlay.get("item_kinds", PackedInt64Array())
		var counts: PackedInt32Array = overlay.get("item_counts", PackedInt32Array())
		var qualities: PackedInt32Array = overlay.get("item_qualities", PackedInt32Array())
		var stack_text: String = EconomyText.stack(locale, kinds[item_index], "", counts[item_index], qualities[item_index]) if kinds[item_index] > 0 else str(counts[item_index])
		_check("stack_hover_label", _texts(labels, "tile").size() == 1 and _texts(labels, "tile")[0].begins_with(stack_text), "want %s… drawn %s" % [stack_text, _texts(labels, "tile")])
	else:
		print("W3D_LABELS INFO no item stack at tick 1; stack text covered by tile_text in the room check")
	labels.set("hover_tile", Vector2i(-1, -1))
	# 4 a room label while the overlay is shown (a room row injected into RoomIndex, as get_rooms would index it).
	var rooms: Node = main.get_node("RoomIndex")
	var info: Dictionary = {"id": 7, "role": 2, "overridden": false, "tiles": 12, "beds": 0, "roofed": true, "dug": false, "doors": 1, "centre": hearth + Vector2i(1, 1)}
	(rooms.call("rooms") as Dictionary)[7] = info
	(rooms.call("tiles") as Dictionary)[hearth + Vector2i(1, 1)] = 7
	await _frames(2)
	_check("room_label_hidden_without_overlay", _texts(labels, "room").is_empty(), str(_texts(labels, "room")))
	rooms.call("set_shown", true)
	await _frames(2)
	# 2D RoomLayer: RoomIndex.label_text → "창고 · 12칸" (UI_ROOM_LABEL_NOBED_FMT, role 2 storehouse).
	var room_text: String = RoomIndex.label_text(info, locale)
	_check("room_label_shown", _texts(labels, "room") == [room_text] and room_text.contains("12"), "want %s drawn %s" % [room_text, _texts(labels, "room")])
	# The 2D stack/room suffix on the room's tile: " · 방: 창고 7".
	var suffix: String = Labels.room_suffix(rooms, hearth + Vector2i(1, 1), true, locale)
	_check("room_suffix", suffix == " · " + EconomyText.trf(locale, "UI_INSPECT_ROOM_FMT", {"role": EconomyText.role(locale, 2), "id": 7}) + " · " + EconomyText.ltr(locale, "UI_ITEM_COVERED"), suffix)
	# The 2D stack text on that tile with a housing designation: stack + room suffix + " · 주거 구역" (OverlayLayer._draw).
	var room_tile: Vector2i = hearth + Vector2i(1, 1)
	var item_kind: int = -1
	for item: Variant in SimReads.read(world, "get_content_catalog").get("items", []):
		if item is Dictionary and int((item as Dictionary).get("code", 0)) > 0:
			item_kind = int(item["code"])
			break
	var stacks: Dictionary = {room_tile: {"kind": item_kind, "count": 3, "quality": 62, "covered": true}}
	var tile_text: String = Labels.tile_text(room_tile, stacks, {}, {room_tile: 4}, rooms, locale)
	var want_tile: String = EconomyText.stack(locale, item_kind, "", 3, 62) + suffix + " · " + EconomyText.ltr(locale, "ZONE_HOUSING")
	_check("stack_room_housing_text", tile_text == want_tile and tile_text.contains("62"), "%s want %s" % [tile_text, want_tile])
	_check("housing_alone_text", Labels.tile_text(room_tile + Vector2i(9, 9), {}, {}, {room_tile + Vector2i(9, 9): 4}, rooms, locale) == EconomyText.ltr(locale, "ZONE_HOUSING"))
	rooms.call("set_shown", false)
	# The 2D animal hover text on a synthetic get_animals row: head + " · " + C1_WILD_AGE.
	var fake: Dictionary = {"ids": PackedInt64Array([5]), "kinds": PackedInt64Array([0]), "states": PackedInt32Array([1]), "hps": PackedInt32Array([30]),
		"age_days": PackedInt64Array([3]), "female": PackedInt64Array([1])}
	var want_animal: String = _animal_head(locale, fake, 0) + " · " + StoryText.trf(locale, "C1_WILD_AGE", {"age": 3, "sex": StoryText.ltr(locale, "C1_SEX_FEMALE")})
	_check("animal_text", Labels.animal_text(fake, 0, locale) == want_animal and want_animal.contains("30"), "%s want %s" % [Labels.animal_text(fake, 0, locale), want_animal])
	# 5 a wild animal under the mouse.
	var animals: Node = w3d.get_node("Animals")
	var cam: Camera3D = w3d.call("camera") as Camera3D
	var picked: bool = false
	for body: Dictionary in animals.call("bodies", 0):
		var feet: Vector3 = body["feet"]
		var p: Vector2 = cam.unproject_position(feet + Vector3(0, float(body["h"]) * 0.5, 0))
		if cam.is_position_behind(feet) or not view.grow(-40.0).has_point(p):
			continue
		labels.set("hover_screen", p)
		await _frames(2)
		var read: Dictionary = SimReads.read(world, "get_animals")
		var i: int = (read.get("ids", PackedInt64Array()) as PackedInt64Array).find(int(body["id"]))
		var want: String = Labels.animal_text(read, i, locale)
		_check("animal_hover_label", _texts(labels, "animal").size() == 1 and want.begins_with(_animal_head(locale, read, i)), "want %s drawn %s" % [want, _texts(labels, "animal")])
		picked = true
		break
	if not picked:
		print("W3D_LABELS INFO no wild animal on screen at the camp; animal text not drawn")
	labels.set("hover_screen", Vector2(-1, -1))
	# 6 every drawn anchor is on screen.
	var outside: Array = []
	for row: Dictionary in labels.call("drawn"):
		if not view.has_point(row["at"]):
			outside.append(row)
	_check("anchors_on_screen", outside.is_empty(), str(outside))
	print("W3D_LABELS_DONE failures=%d checks=%d" % [_failures, _checks])
	quit(1 if _failures > 0 else 0)


## The 2D AnimalLayer's hover head: "<animal> · <state> · <hp>" (M5_ANIMAL_HOVER_FMT), before the age suffix.
static func _animal_head(locale: Node, read: Dictionary, i: int) -> String:
	var kinds: PackedInt64Array = read.get("kinds", PackedInt64Array())
	var states: PackedInt32Array = read.get("states", PackedInt32Array())
	var hps: PackedInt32Array = read.get("hps", PackedInt32Array())
	return StoryText.trf(locale, "M5_ANIMAL_HOVER_FMT", {"animal": StoryText.animal(locale, kinds[i]), "state": StoryText.animal_state(locale, states[i]), "hp": hps[i]})


static func _nearest(xs: PackedInt32Array, ys: PackedInt32Array, at: Vector2i) -> int:
	var best: int = -1
	var best_d: int = 1 << 30
	for i: int in mini(xs.size(), ys.size()):
		var d: int = absi(xs[i] - at.x) + absi(ys[i] - at.y)
		if d < best_d:
			best_d = d
			best = i
	return best
