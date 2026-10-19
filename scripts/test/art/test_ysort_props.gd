extends SceneTree

var _failures: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		push_error(message)
		_failures += 1

func _run() -> void:
	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = packed.instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	await process_frame
	SimReads.mutate(world, "advance_ticks", [1])
	var renderer: Node = main.get_node("BuildingRenderer")
	var agents: Node = main.get_node("AgentRenderer")
	renderer.call("refresh_snapshot")
	agents.call("refresh_snapshot")
	await process_frame
	var objects: Node2D = main.get_node("WorldObjects") as Node2D
	var shadows: Node2D = main.get_node("Shadows") as Node2D
	_check(objects.y_sort_enabled and objects.z_index == 4, "upright objects require shared y-sort layer")
	_check(shadows.z_index == 3, "shadows must be behind upright objects")
	# Exercise the production atlas path before the forest renderer arrives in Part 4.
	var tree: Sprite2D = Sprite2D.new()
	objects.add_child(tree)
	ArtSpriteLibrary.apply(tree, "direction_14_young_tree")
	tree.position = Vector2(456, 43)
	tree.set_meta("art_kind", "tree")
	tree.set_meta("tile_x", 0)
	tree.set_meta("tile_y", 0)
	tree.set_meta("calibration_fixture", true)
	var counts: Dictionary = {}
	for child: Node in objects.get_children():
		if not child is Sprite2D:
			continue
		var sprite: Sprite2D = child as Sprite2D
		var kind: String = str(sprite.get_meta("art_kind", ""))
		counts[kind] = int(counts.get(kind, 0)) + 1
		_check(sprite.scale == Vector2.ONE, "world objects preserve native scale")
		_check(not sprite.centered, "atlas objects anchor from top-left")
		_check(sprite.z_index == 0, "objects must share their parent z-index")
		_check(sprite.offset == -Vector2(sprite.get_meta("anchor", Vector2.ZERO)), "atlas anchor matches offset")
		if kind in ["building", "site", "bush"]:
			var x: int = int(sprite.get_meta("tile_x"))
			var y: int = int(sprite.get_meta("tile_y"))
			_check(sprite.position == Vector2(448 + 16 * x + 8, 28 + 16 * y + 15), "building base matches tile bottom")
			_check(sprite.modulate.a == 1.0, "objects and construction frames stay opaque")
		if kind == "agent":
			var agent_id: int = int(sprite.get_meta("art_id"))
			var look: Dictionary = ArtHash.look(agent_id)
			var x: int = int(sprite.get_meta("tile_x"))
			var y: int = int(sprite.get_meta("tile_y"))
			var simulation_targets: Dictionary = agents.get("_curr_positions")
			_check(simulation_targets[agent_id] == Vector2(448 + 16 * x + 8 + int(look["jx"]), 28 + 16 * y + 14 + int(look["jy"])), "simulation feet retain snapshot tile and stable jitter")
			_check(sprite.position == agents.call("get_drawn_position", agent_id), "agent y-sort feet use the drawn position")
			var agent_shadows: Dictionary = agents.get("_shadows")
			var shadow: Sprite2D = agent_shadows[agent_id]
			_check(shadow.position == sprite.position, "agent shadow follows drawn feet in the same frame")
			_check(shadow.visible == sprite.visible, "agent shadow visibility follows its sprite")
			_check(sprite.modulate == Color.WHITE, "actual agent output has no state tint")
		if kind == "tree" and sprite.get_meta("calibration_fixture", false):
			_check(sprite.position.y == 43.0, "tree y-sort baseline is its tile base")
	for kind: String in ["building", "bush", "agent", "tree"]:
		_check(int(counts.get(kind, 0)) > 0, "real scene missing " + kind)
	for child: Node in shadows.get_children():
		if child is Sprite2D and str(child.get_meta("art_kind", "")) == "object_shadow":
			_check(is_equal_approx((child as Sprite2D).modulate.a, 0.4), "contact shadow opacity is shared")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	camera.zoom = Vector2.ONE
	renderer.call("_update_labels")
	var labels: Dictionary = renderer.get("_labels")
	for label: Label in labels.values():
		_check(not label.visible, "stockpile labels hidden at overview")
	camera.zoom = Vector2(4, 4)
	renderer.call("_update_labels")
	for label: Label in labels.values():
		_check(label.visible, "stockpile labels visible at village zoom")
		var height: float = label.get_minimum_size().y * label.scale.y * camera.zoom.y
		_check(height >= 10.0 and height <= 28.0, "stockpile label screen height remains readable")
	var original: Array[Vector2i] = renderer.get("_stockpile_positions")
	var positions: Array[Vector2i] = [Vector2i(1, 1), Vector2i(9, 9)]
	renderer.set("_stockpile_positions", positions)
	var totals: Dictionary = renderer.call("_food_totals_by_stockpile", {
		"formation_xs": PackedInt32Array([1, 2, 9]),
		"formation_ys": PackedInt32Array([1, 1, 9]),
		"food_stocks": PackedInt32Array([4, 7, 13]),
	})
	_check(int(totals.get(Vector2i(1, 1), 0)) == 11, "shared nearest stockpile sums settlements exactly once")
	_check(int(totals.get(Vector2i(9, 9), 0)) == 13, "far stockpile keeps its own total")
	renderer.set("_stockpile_positions", original)
	main.free()
	print("test_ysort_props: ", "PASS" if _failures == 0 else "FAIL")
	quit(0 if _failures == 0 else 1)
