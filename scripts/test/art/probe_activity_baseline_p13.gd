extends SceneTree
## Headless geometry and texture-data evidence, never framebuffer acceptance.
const Route = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const Icons = preload("res://scripts/ui/activity_icons_p11.gd")

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	if DisplayServer.get_name() != "headless":
		push_error("headless display required")
		quit(2)
		return
	var output: String = OS.get_environment("P13_BASELINE_OUT")
	if output.is_empty() or FileAccess.file_exists(output):
		push_error("requires unused P13_BASELINE_OUT JSON path")
		quit(2)
		return
	root.size = Vector2i(1920, 1080)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	current_scene = main
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	camera.position_smoothing_enabled = false
	camera.zoom = Vector2(4, 4)
	var renderer: Node = main.get_node("AgentRenderer")
	renderer.process_mode = Node.PROCESS_MODE_DISABLED
	var cases: Array[Dictionary] = []
	for spec: Array in [[20261010, 41880], [20261010, 42180], [20261018, 85080]]:
		Route.mutate(world, "reset_world", [spec[0]])
		var advanced: int = int(Route.mutate(world, "advance_ticks", [spec[1]]))
		if advanced != int(spec[1]):
			push_error("exact advance failed")
			quit(3)
			return
		var settlements: Dictionary = Route.read(world, "get_settlement_snapshot")
		if not settlements["settlement_ids"].is_empty():
			camera.position = Vector2(448, 28) + Vector2(settlements["formation_xs"][0], settlements["formation_ys"][0]) * 16.0 + Vector2(8, 8)
		camera.force_update_scroll()
		renderer.call("refresh_snapshot")
		var rows: Array[Dictionary] = []
		var sprites: Dictionary = renderer.get("_sprites")
		var icons: Dictionary = renderer.get("_activity_icons")
		for person: int in icons:
			var icon: Sprite2D = icons[person] as Sprite2D
			var sprite: Sprite2D = sprites.get(person) as Sprite2D
			if not is_instance_valid(icon) or not is_instance_valid(sprite) or icon.texture == null:
				continue
			var rect: Rect2 = icon.get_global_transform_with_canvas() * icon.get_rect()
			rows.append({"person": person, "entity_bits": sprite.get_meta("entity_bits", -1), "activity_kind": sprite.get_meta("activity_kind", -1), "arrived": sprite.get_meta("activity_place_known", false), "icon_id": icon.get_meta("sprite_id", ""), "visible_in_tree": icon.is_visible_in_tree(), "in_viewport": root.get_visible_rect().intersects(rect), "rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y], "texture_size": [icon.texture.get_width(), icon.texture.get_height()]})
		cases.append({"seed": spec[0], "tick": spec[1], "world_info": Route.read(world, "get_world_info"), "clock": Route.read(world, "get_time_snapshot"), "rows": rows})
	var textures: Array[Dictionary] = []
	for kind: int in 7:
		var texture: Texture2D = Icons.texture(kind)
		var pixels: Image = texture.get_image() if texture != null else null
		var row: Dictionary = {"kind": kind, "texture_available": texture != null, "image_available": pixels != null and not pixels.is_empty()}
		if texture != null:
			row["dimensions"] = [texture.get_width(), texture.get_height()]
		if pixels != null and not pixels.is_empty():
			var used: Rect2i = pixels.get_used_rect()
			row["opaque_bounds"] = [used.position.x, used.position.y, used.size.x, used.size.y]
			var colors: Dictionary = {}
			for y: int in pixels.get_height():
				for x: int in pixels.get_width():
					var color: Color = pixels.get_pixel(x, y)
					if color.a >= 0.5:
						var key: String = color.to_html(false)
						colors[key] = int(colors.get(key, 0)) + 1
			row["opaque_color_counts"] = colors
		textures.append(row)
	var file: FileAccess = FileAccess.open(output, FileAccess.WRITE)
	if file == null:
		push_error("cannot create output")
		quit(4)
		return
	file.store_string(JSON.stringify({"complete": true, "evidence_type": "headless_geometry_not_framebuffer", "display": DisplayServer.get_name(), "bridge_sha256": FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib"), "icon_source_sha256": FileAccess.get_sha256("res://scripts/ui/activity_icons_p11.gd"), "cases": cases, "textures": textures}, "\t") + "\n")
	file.close()
	print("P13_ACTIVITY_BASELINE_COMPLETE cases=3 textures=7")
	quit(0)
