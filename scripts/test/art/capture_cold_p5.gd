extends SceneTree

const Activity = preload("res://scripts/ui/agent_activity.gd")
const Cold = preload("res://scripts/ui/cold_effects.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
const Hash = preload("res://scripts/ui/art_hash.gd")
const SIZE: Vector2i = Vector2i(1024, 640)
var _rows: Array[Dictionary] = []
var _sprites: Array[Sprite2D] = []
var _scene: Node2D

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var output: String = ""
	var check_only: bool = false
	for argument: String in OS.get_cmdline_user_args():
		if argument.begins_with("--output="):
			output = argument.trim_prefix("--output=")
		elif argument == "--check-only":
			check_only = true
	if output.is_empty() and not check_only:
		push_error("Use -- --output=/absolute/path/cold_scene.png; controlled presentation evidence only")
		quit(2)
		return
	root.size = SIZE
	root.title = "Art Pass 5 controlled cold presentation test (not a natural simulation capture)"
	_scene = Node2D.new()
	root.add_child(_scene)
	var background: ColorRect = ColorRect.new()
	background.size = Vector2(SIZE)
	background.color = Color("17252d")
	_scene.add_child(background)
	var evidence_caption: Label = Label.new()
	evidence_caption.text = "CONTROLLED PRESENTATION TEST / NOT A NATURAL SIMULATION CAPTURE"
	evidence_caption.position = Vector2(64, 612)
	evidence_caption.add_theme_font_size_override("font_size", 16)
	evidence_caption.modulate = Color(0.72, 0.78, 0.82)
	_scene.add_child(evidence_caption)
	var locale: Node = root.get_node("Locale")
	for row: int in 2:
		var label: Label = Label.new()
		label.text = str(locale.ltr("HUD_COLD")) + (" 3" if row == 0 else " 0")
		label.position = Vector2(64, 24 + row * 304)
		label.add_theme_font_override("font", load("res://assets/fonts/galmuri/Galmuri11.ttf") as Font)
		label.add_theme_font_size_override("font_size", 24)
		_scene.add_child(label)
	var groups: Dictionary = Activity.build_frame_groups(Library.entries())
	for index: int in 6:
		var column: int = index % 3
		var cold: bool = index < 3
		var body: String = ["adult_a", "adult_b", "child"][column]
		var agent_id: int = column * 24
		var snapshot: Dictionary = {"agent_ids": PackedInt64Array([agent_id]), "colds": PackedByteArray([1 if cold else 0])}
		var marked: bool = Cold.snapshot_cold(snapshot, 0)
		var pose: String = Cold.pose("idle", marked, false)
		var choice: Dictionary = Activity.choose_frame(groups, body, pose, "S", 0.0, 0.0)
		var holder: Node2D = Node2D.new()
		holder.position = Vector2(200 + column * 312, 282 if cold else 586)
		holder.scale = Vector2(6, 6)
		_scene.add_child(holder)
		var sprite: Sprite2D = Sprite2D.new()
		holder.add_child(sprite)
		if choice.is_empty() or not Library.apply(sprite, str(choice["sprite_id"])):
			push_error("Controlled cold scene missing installed frame for " + body + "/" + pose)
			quit(1)
			return
		var look: Dictionary = Hash.look(agent_id)
		look["tone"] = 1
		look["hair"] = 1
		Library.apply_variant(sprite, look)
		_sprites.append(sprite)
		_rows.append({"body": body, "agent_id": agent_id, "colds": 1 if marked else 0, "pose": pose,
			"cold_pose": choice.get("activity", ""), "sprite_id": choice["sprite_id"], "scale": 6,
			"fallback_kind": choice.get("fallback_kind", ""), "snapshot_kind": "controlled_test_fixture"})
	var samples: Array[Dictionary] = []
	for seconds: float in [0.05, 0.22]:
		for index: int in _sprites.size():
			var sprite: Sprite2D = _sprites[index]
			sprite.position = Vector2.ZERO
			Cold.apply(sprite, int(_rows[index]["colds"]) == 1, false, int(_rows[index]["agent_id"]), seconds)
			_rows[index]["shiver_offset"] = sprite.get_meta("shiver_offset")
			_rows[index]["puff_alpha"] = sprite.get_meta("puff_alpha")
		var destination: String = output if seconds < 0.1 else output.get_basename() + ".shiver0.png"
		if not check_only:
			await process_frame
			await RenderingServer.frame_post_draw
			DirAccess.make_dir_recursive_absolute(destination.get_base_dir())
			var screenshot: Image = root.get_texture().get_image()
			if screenshot == null or screenshot.is_empty() or screenshot.save_png(destination) != OK:
				push_error("Controlled scene GPU screenshot failed: " + destination)
				quit(1)
				return
		samples.append({"seconds": seconds, "image": destination, "agents": _rows.duplicate(true)})
	if not check_only:
		var file: FileAccess = FileAccess.open(output.get_basename() + ".json", FileAccess.WRITE)
		if file == null:
			push_error("Controlled scene metadata write failed")
			quit(1)
			return
		file.store_string(JSON.stringify({"evidence_kind": "controlled_presentation_scene", "natural_simulation_capture": false,
			"simulation_instantiated": false, "viewport": [SIZE.x, SIZE.y], "source": "installed atlas and production presentation functions",
			"samples": samples}, "\t") + "\n")
	print("COLD_SCENE ", "CHECK_ONLY" if check_only else "GPU_CAPTURE", " agents=6 samples=2 natural=false simulation_instantiated=false")
	_scene.free()
	quit(0)
