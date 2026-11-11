# wall_shots.gd — before/after screenshots for W2 thin wall pieces.
# Usage: godot --path . --resolution 1920x1080 --script res://tools/v8/wall_shots.gd -- --out <dir> [--tag before|after]
# Boots seed 0, advances to tick ~7440 (day 6 noon, huts built), takes 1×–4× shots plus a ghost overlay.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const SiteGhost = preload("res://scripts/world/art/site_ghost.gd")
const WallBaker = preload("res://scripts/world/art/wall_baker.gd")
const TILE: int = 64

var _out: String = "/tmp/wall_shots"
var _tag: String = "after"


func _initialize() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size(): _out = args[i + 1]
		elif args[i] == "--tag" and i + 1 < args.size(): _tag = args[i + 1]
	call_deferred("_run")


func _wait(n: int) -> void:
	for _i: int in n:
		await process_frame


func _save(name: String) -> void:
	await RenderingServer.frame_post_draw
	var path: String = _out.path_join("%s_%s.png" % [_tag, name])
	var ok: bool = root.get_texture().get_image().save_png(path) == OK
	print("WALL_SHOT %s %s" % [path if ok else "FAILED", name])


func _run() -> void:
	DirAccess.make_dir_recursive_absolute(_out)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	await _wait(10)
	root.get_node("Locale").call("set_language", "ko")
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	# Reset to seed 0 and advance to day 6 noon (tick 7440).
	world.call("reset_world", 0)
	await _wait(5)
	world.call("advance_ticks", 7440)
	await _wait(30)
	# Find the hearth (settlement centre).
	var b: Dictionary = SimReads.read(world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var bxs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = b.get("ys", PackedInt32Array())
	var hearth: Vector2i = Vector2i(96, 96)
	for i: int in kinds.size():
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			hearth = Vector2i(bxs[i], bys[i])
			break
	print("WALL_SHOTS hearth=%s tag=%s" % [hearth, _tag])
	# Report bake timing.
	var layer: Node = main.get_node_or_null("ColonyView/GroundLayer")
	if layer != null:
		print("WALL_SHOTS_BAKE worst_bake=%s last_bake_usec=%d" % [layer.get("worst_bake"), int(layer.get("last_bake_usec"))])
	# Place a ghost hut (half-built outline) near the settlement.
	var world_node: Node = main.get_node_or_null("ColonyView/World")
	if world_node != null:
		var ring: Dictionary = {"size": [3, 3], "wall_style": "log", "door_style": "slab_door", "door": [1, 2], "floor_style": "packed_earth"}
		var ri: Dictionary = SiteGhost.ring_image(ring)
		if not ri.is_empty():
			var ghost_img: Image = SiteGhost.ghost_image(ri["image"])
			var gs: Sprite2D = Sprite2D.new()
			gs.texture = ImageTexture.create_from_image(ghost_img)
			gs.centered = false
			gs.position = Vector2((hearth.x + 4) * TILE, (hearth.y - 3) * TILE - WallBaker.CAP)
			world_node.add_child(gs)
			print("WALL_SHOTS ghost placed at %s" % gs.position)
	var ui: CanvasLayer = main.get_node("UI") as CanvasLayer
	ui.visible = false
	# Take 1×, 2× (overview) + 3×, 4× (close-up on a single hut with walls exposed).
	for zoom: float in [1.0, 2.0, 3.0, 4.0]:
		camera.zoom = Vector2.ONE * zoom
		camera.position = Vector2(hearth) * TILE + Vector2(TILE, TILE) * 0.5
		if camera.has_method("_clamp_position"):
			camera.call("_clamp_position")
		camera.reset_smoothing()
		camera.force_update_scroll()
		Input.warp_mouse(Vector2(4, 4))
		await _wait(40)
		await _save("settlement_%dx" % int(zoom))
	print("WALL_SHOTS_DONE")
	quit(0)
