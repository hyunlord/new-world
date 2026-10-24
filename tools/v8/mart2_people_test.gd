# mart2_people_test.gd — headless tests of the M-ART2 people layer (goal §9).
# Purpose: boots scenes/main.tscn on colony seed 0 (reset_world(0), clock held, stepped to tick 1) and checks on the
#   real AgentLayer:
#   1. appearance = the bridge: every drawn person's look and variant equal the snapshot's `look_ids` / `variants`
#      (the Godot side of appearance agreement; the probe side is engine_appearance_ffi.rs), and the frames resolve
#      through manifest.json (fallback looks to their bun frames);
#   2. anchor placement in all four facings: the frame's lowest opaque row is within 2 px of its anchor y; the body is
#      offset so the anchor pixel (mirrored for W) lands on the person's feet, and the feet are the tile's base point
#      (tile_left + 32, tile_top + 43) on screen (the canvas transform of the base point);
#   3. the LOD switch at the thresholds: markers below `lod.people_below_zoom`, sprites at and above it;
#   4. a params reload changes what is drawn: a copy of render_params.json with `lod.people_below_zoom` 0.4 (in
#      user://) is reloaded through ArtData.reload_params (the F12 path) and ½× then shows sprites;
#   5. the variant tint is deterministic: the shader parameters of a person are a pure function of (look, variant).
#   Prints MART2_PEOPLE lines and MART2_PEOPLE_DONE failures=N; exits 1 on any failure.
# Bridge methods called: reset_world, cmd_set_steward, get_steward, get_time_snapshot, advance_ticks (direct).
# The historical seed-0 art fixture explicitly keeps the steward OFF; the layers read the rest.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")

var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool, detail: String) -> void:
	print("MART2_PEOPLE %s %s — %s" % ["PASS" if ok else "FAIL", name, detail])
	if not ok:
		_failures += 1


func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	await process_frame
	world.call("reset_world", 0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	var steward_command: Dictionary = world.call("cmd_set_steward", false, 0)
	var steward: Dictionary = world.call("get_steward")
	var clock: Dictionary = world.call("get_time_snapshot")
	var fixture_ready: bool = bool(steward_command.get("accepted", false)) and not bool(steward.get("enabled", true)) and int(clock.get("tick", -1)) == 0
	_check("fixture_steward_off", fixture_ready, "ordinary OFF command accepted, state OFF at tick zero")
	if not fixture_ready:
		quit(1)
		return
	world.call("advance_ticks", 1)
	for _i: int in 20:
		await process_frame
	var layer: Node = main.get_node("ColonyView/AgentLayer")
	var camera: Camera2D = main.get_node("Camera2D")
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	_check_appearance(layer, snap)
	_check_anchors(layer, snap)
	await _check_lod(layer, camera)
	await _check_reload(layer, camera)
	_check_tint(layer, snap)
	print("MART2_PEOPLE_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


func _check_appearance(layer: Node, snap: Dictionary) -> void:
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var looks: PackedInt32Array = snap.get("look_ids", PackedInt32Array())
	var variants: PackedByteArray = snap.get("variants", PackedByteArray())
	var bad: Array[String] = []
	var frames_ok: int = 0
	for i: int in bits.size():
		var p: Dictionary = layer.call("person_look", bits[i])
		if p.is_empty() or int(p["look"]) != looks[i] or int(p["variant"]) != variants[i]:
			bad.append("%d: drawn %s/%s, bridge %d/%d" % [bits[i], p.get("look"), p.get("variant"), looks[i], variants[i]])
		var row: Dictionary = ArtData.look(looks[i])
		if str(p.get("frames", "")) == str(row.get("frames", "?")) and ArtData.has("%s/S/idle" % row.get("frames", "")):
			frames_ok += 1
	_check("appearance_matches_bridge", bits.size() >= 8 and bad.is_empty() and looks.size() == bits.size(), "%d people, %d mismatches %s, frames resolved %d" % [bits.size(), bad.size(), bad.slice(0, 3), frames_ok])
	_check("frames_resolve", frames_ok == bits.size(), "%d of %d" % [frames_ok, bits.size()])


func _check_anchors(layer: Node, snap: Dictionary) -> void:
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var bad: Array[String] = []
	var checked: int = 0
	var canvas: Transform2D = root.get_viewport().get_canvas_transform()
	for i: int in mini(bits.size(), 6):
		for facing: String in ["S", "E", "W", "N"]:
			layer.call("face_for_test", bits[i], facing)
			var p: Dictionary = layer.call("person_look", bits[i])
			var anchor: Vector2i = p["anchor"]
			var offset: Vector2 = p["offset"]
			var flip: bool = p["flip"]
			var frame_id: String = "%s/%s/idle" % [p["frames"], "E" if facing == "E" or facing == "W" else "S"]
			var img: Image = ArtData.image(frame_id)
			var lowest: int = -1
			for y: int in range(img.get_height() - 1, -1, -1):
				for x: int in img.get_width():
					if img.get_pixel(x, y).a >= 0.5:
						lowest = y
						break
				if lowest >= 0:
					break
			var anchor_x_drawn: float = (img.get_width() - anchor.x) if flip else anchor.x
			var feet: Vector2 = p["feet"]
			var base: Vector2 = View.base_point(Vector2i(xs[i], ys[i]))
			var foot_screen: Vector2 = canvas * (feet + offset + Vector2(anchor_x_drawn, anchor.y))
			var want_screen: Vector2 = canvas * base
			var ok: bool = absi(lowest - anchor.y) <= 2 and offset.x == -anchor_x_drawn and offset.y == -anchor.y \
				and flip == (facing == "W") and feet.distance_to(base) < 0.01 and foot_screen.distance_to(want_screen) < 0.01
			checked += 1
			if not ok:
				bad.append("%d %s: lowest %d anchor %s offset %s flip %s feet %s base %s" % [bits[i], facing, lowest, anchor, offset, flip, feet, base])
	_check("anchor_placement_four_facings", checked >= 24 and bad.is_empty(), "%d person-facings, %d bad %s" % [checked, bad.size(), bad.slice(0, 2)])


func _check_lod(layer: Node, camera: Camera2D) -> void:
	var seen: Dictionary = {}
	for z: float in [1.0, 0.5, 0.25, 2.0]:
		camera.zoom = Vector2(z, z)
		await process_frame
		await process_frame
		seen[z] = bool(layer.call("lod_active"))
	var below: float = float(ArtData.param("lod.people_below_zoom", 1.0))
	_check("lod_switch", seen[1.0] == false and seen[2.0] == false and seen[0.5] == true and seen[0.25] == true, "threshold %.2f: %s" % [below, seen])


func _check_reload(layer: Node, camera: Camera2D) -> void:
	var copy: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(ArtData.ROOT + "render_params.json"))
	(copy["lod"] as Dictionary)["people_below_zoom"] = 0.4
	var path: String = "user://mart2_params_test.json"
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	f.store_string(JSON.stringify(copy))
	f.close()
	var version: int = ArtData.params_version()
	ArtData.params_path = path
	var ok: bool = ArtData.reload_params()
	camera.zoom = Vector2(0.5, 0.5)
	await process_frame
	await process_frame
	var lod_after: bool = bool(layer.call("lod_active"))
	ArtData.params_path = ArtData.ROOT + "render_params.json"
	ArtData.reload_params()
	await process_frame
	await process_frame
	var lod_back: bool = bool(layer.call("lod_active"))
	_check("params_reload", ok and ArtData.params_version() == version + 2 and not lod_after and lod_back, "reload ok %s, version %d → %d, at ½× with threshold 0.4 markers=%s, after restoring markers=%s" % [ok, version, ArtData.params_version(), lod_after, lod_back])


func _check_tint(layer: Node, snap: Dictionary) -> void:
	# Expected parameters straight from the imported record: (frames look, "v" + (variant + 1)) → skin_delta_lab.
	var records: Dictionary = {}
	var vt: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(ArtData.path("variant_transforms")))
	for r: Variant in vt.get("transforms", []):
		records["%s/%s" % [(r as Dictionary)["look"], (r as Dictionary)["variant"]]] = r
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var bad: Array[String] = []
	for b: int in bits:
		var p: Dictionary = layer.call("person_look", b)
		var params: Array = layer.call("tint_params", b)
		var rec: Dictionary = records.get("%s/v%d" % [p["frames"], int(p["variant"]) + 1], {})
		var want: Array = rec.get("skin_delta_lab", [])
		var got: Vector3 = params[0] if params.size() == 2 else Vector3(INF, INF, INF)
		if want.size() != 3 or got.distance_to(Vector3(want[0], want[1], want[2])) > 1e-4:
			bad.append("%d %s v%d: %s vs %s" % [b, p["frames"], int(p["variant"]) + 1, got, want])
		var again: Array = layer.call("tint_params", b)
		if again != params:
			bad.append("%d unstable" % b)
	_check("variant_tint_deterministic", bits.size() > 0 and bad.is_empty(), "%d people: skin shift = variant_transforms.json record, stable on re-read; bad %s" % [bits.size(), bad.slice(0, 2)])
