# mux_minimap_test.gd — behavioral controls/minimap checks with a deterministic snapshot fixture.
# Bridge methods: none; parent also runs the widgets against the real scene in mux_ui_test.gd.
extends SceneTree

const Controls = preload("res://scripts/ui/mux/map_controls.gd")
const Minimap = preload("res://scripts/ui/mux/minimap.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Ground = preload("res://scripts/world/colony/ground_layer.gd")
const People = preload("res://scripts/world/colony/agent_layer.gd")

class TestCamera extends Camera2D:
	var speed: float = 1.0
	var paused: bool = false
	var camp_count: int = 0
	var target: Vector2i = Vector2i(-1, -1)
	func get_speed() -> float: return speed
	func is_paused() -> bool: return paused
	func set_speed(value: float) -> void: speed = value
	func set_paused(value: bool) -> void: paused = value
	func get_zoom_level() -> float: return zoom.x
	func step_zoom(direction: int) -> void: zoom *= pow(2.0, direction)
	func center_on_camp() -> void: camp_count += 1
	func center_on_tile(tile: Vector2i) -> void:
		target = tile
		position = Vector2(tile) * 64.0 + Vector2(32, 32)

class TestWorld extends Node:
	var snapshots: Dictionary = {}
	var reads: int = 0
	func get_colony_versions() -> Dictionary:
		reads += 1
		return {"rooms_version": 0, "overlay_version": 0}
	func get_terrain_snapshot() -> Dictionary: return snapshots["terrain"]
	func get_rooms() -> Dictionary: return snapshots["rooms"]
	func get_space() -> Dictionary: return snapshots["space"]
	func get_agent_snapshot() -> Dictionary: return snapshots["agents"]
	func get_animals() -> Dictionary: return snapshots["animals"]

class TestCatalog extends RefCounted:
	func tooltip(action: String) -> String: return "shortcut:" + action

var failures: int = 0
var checks: int = 0

func _init() -> void:
	call_deferred("_run")

func check(ok: bool, label: String) -> void:
	checks += 1
	if not ok:
		failures += 1
	print("MUX_MINIMAP %s %s" % ["PASS" if ok else "FAIL", label])

func _run() -> void:
	var camera: TestCamera = TestCamera.new()
	root.add_child(camera)
	var controls: Controls = Controls.new()
	root.add_child(controls)
	controls.setup(camera, TestCatalog.new())
	controls.button_for("pause").pressed.emit()
	check(camera.paused, "pause button")
	for speed: int in range(1, 5):
		controls.button_for("speed_%d" % speed).pressed.emit()
		check(camera.speed == speed and not camera.paused, "speed %d resumes" % speed)
		controls.refresh_state()
		check(controls.button_for("speed_%d" % speed).button_pressed, "speed active %d" % speed)
	controls.button_for("zoom_out").pressed.emit()
	check(camera.zoom.x == 0.5, "zoom out")
	check((controls.get("_zoom_label") as Label).text == "½×", "fractional zoom readout")
	controls.button_for("zoom_in").pressed.emit()
	check(camera.zoom.x == 1.0, "zoom in")
	check((controls.get("_zoom_label") as Label).text == "1×", "integer zoom readout uses supported formatter")
	controls.button_for("home").pressed.emit()
	check(camera.camp_count == 1, "camp button")
	check(controls.button_for("home").tooltip_text == "shortcut:home", "catalog tooltip")
	var minimap: Minimap = Minimap.new()
	root.add_child(minimap)
	minimap.setup(null, camera)
	minimap.set_process(false)
	_check_ground_palette(minimap)
	await process_frame
	minimap.refresh_snapshots(_fixture())
	for frame: int in range(120):
		minimap.process_pending()
		if not minimap.is_building(): break
	check(not minimap.is_building(), "incremental build completes")
	check(minimap.map_image().get_size() == Vector2i(8, 8), "map dimensions")
	var terrain_pixel: Color = _rgba8(minimap.terrain_colour(0))
	check(minimap.map_image().get_pixel(1, 1).to_rgba32() == terrain_pixel.to_rgba32(), "terrain palette")
	check(minimap.map_image().get_pixel(2, 2) != terrain_pixel, "room shown")
	check(minimap.map_image().get_pixel(3, 3) != terrain_pixel, "wall shown")
	var colours: Array = ArtData.param("lod.household_ring_rgb", [])
	check(minimap.map_image().get_pixel(4, 4).is_equal_approx(People._ring_colour(10, colours)), "household colour")
	var rgb: Array = ArtData.binding("animals", 0).get("lod_colour", [])
	check(minimap.map_image().get_pixel(6, 6).is_equal_approx(Color8(int(rgb[0]), int(rgb[1]), int(rgb[2]))), "animal registry colour")
	minimap.jump_to_fraction(Vector2(0.5, 0.25))
	check(camera.target == Vector2i(4, 2), "minimap target")
	minimap.jump_to_fraction(Vector2(2, -1))
	check(camera.target == Vector2i(7, 0), "minimap clamps target")
	var rect: Rect2 = minimap._map_rect()
	var press: InputEventMouseButton = InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	press.position = rect.position + rect.size * 0.25
	minimap.map_input(press)
	check(camera.target == Vector2i(2, 2), "pointer click moves camera")
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = rect.position + rect.size * 0.75
	minimap.map_input(motion)
	check(camera.target == Vector2i(6, 6), "pointer drag moves camera")
	press.pressed = false
	minimap.map_input(press)
	motion.position = rect.position
	minimap.map_input(motion)
	check(camera.target == Vector2i(6, 6), "release ends pointer drag")
	var changed: Dictionary = _fixture()
	changed["rooms"] = {"version": 1, "tile_xs": PackedInt32Array([1]), "tile_ys": PackedInt32Array([2])}
	minimap.refresh_snapshots(changed)
	for frame: int in range(120):
		minimap.process_pending()
		if not minimap.is_building(): break
	check(minimap.map_image().get_pixel(2, 2).to_rgba32() == terrain_pixel.to_rgba32(), "room version clears old room")
	check(minimap.map_image().get_pixel(1, 2).to_rgba32() != terrain_pixel.to_rgba32(), "room version paints new room")
	minimap.set_collapsed(true)
	check(minimap.is_collapsed(), "collapse")
	minimap.set_collapsed(false)
	check(not minimap.is_collapsed(), "expand")
	var world: TestWorld = TestWorld.new()
	world.snapshots = _fixture()
	root.add_child(world)
	minimap.setup(world, camera)
	minimap._process(0.0)
	for frame: int in range(120):
		minimap.process_pending()
		if not minimap.is_building(): break
	var before: int = world.reads
	for i: int in range(10): minimap._process(0.01)
	check(world.reads == before, "no refresh before interval")
	minimap._process(0.41)
	check(world.reads == before + 1, "refresh at two Hz")
	minimap.set_collapsed(true)
	before = world.reads
	minimap._process(1.0)
	check(world.reads == before, "collapsed skips bridge reads")
	print("MUX_MINIMAP_DONE checks=%d failures=%d" % [checks, failures])
	quit(1 if failures else 0)

func _fixture() -> Dictionary:
	var terrain: PackedByteArray = PackedByteArray()
	terrain.resize(64)
	return {
		"terrain": {"width": 8, "height": 8, "terrain": terrain},
		"rooms": {"tile_xs": PackedInt32Array([2]), "tile_ys": PackedInt32Array([2])},
		"space": {"wall_xs": PackedInt32Array([3]), "wall_ys": PackedInt32Array([3])},
		"agents": {"agent_ids": PackedInt64Array([10]), "xs": PackedInt32Array([4]), "ys": PackedInt32Array([4])},
		"animals": {"kinds": PackedInt32Array([0]), "xs": PackedInt32Array([6]), "ys": PackedInt32Array([6])}}

# Expected pixels must use Image's exact storage conversion, not Color.to_rgba32 rounding.
func _rgba8(colour: Color) -> Color:
	var reference: Image = Image.create(1, 1, false, Image.FORMAT_RGBA8)
	reference.set_pixel(0, 0, colour)
	return reference.get_pixel(0, 0)

func _check_ground_palette(minimap: Minimap) -> void:
	var ground: Ground = Ground.new()
	var rows: Array = ArtData.bindings().get("terrain", {}).get("codes", [])
	for row: Dictionary in rows:
		if bool(row.get("water", false)): continue
		ground.ground_ids.assign([str(row["ground"])])
		var expected: Color = ground._palette()[0]
		var actual: Color = minimap.terrain_colour(int(row["code"]))
		check(actual.is_equal_approx(expected), "terrain %d matches renderer D9 palette" % int(row["code"]))
	ground.free()
