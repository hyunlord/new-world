# Capture all four Style 2 near-tier game VATs at the same pose and scale.
# Bridge methods called: none.
extends SceneTree

const Vat = preload("res://scripts/world3d/vat3d.gd")
const People = preload("res://scripts/world3d/people3d.gd")


func _initialize() -> void:
	_run.call_deferred()


func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() != 2 or args[0] != "--style=s2" or not OS.has_environment("WORLDSIM_MUX2_LANE_FD") or DisplayServer.get_name() == "headless":
		_fail("expected a window, the GPU lock and --style=s2 OUTPUT")
		return
	var out: String = args[1]
	if DirAccess.make_dir_recursive_absolute(out) != OK:
		_fail("could not create the frame directory")
		return
	var stage: Node3D = Node3D.new()
	root.add_child(stage)
	var world_env: WorldEnvironment = WorldEnvironment.new()
	var environment: Environment = Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color("#4d5045")
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color("#eee0ca")
	environment.ambient_light_energy = 0.65
	world_env.environment = environment
	stage.add_child(world_env)
	var sun: DirectionalLight3D = DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-43.0, 32.0, 0.0)
	sun.light_color = Color("#fff0ce")
	sun.shadow_enabled = true
	stage.add_child(sun)

	var floor_node: MeshInstance3D = MeshInstance3D.new()
	var floor_mesh: PlaneMesh = PlaneMesh.new()
	floor_mesh.size = Vector2(20.0, 20.0)
	var floor_material: StandardMaterial3D = StandardMaterial3D.new()
	floor_material.albedo_color = Color("#605443")
	floor_mesh.material = floor_material
	floor_node.mesh = floor_mesh
	floor_node.position.y = -0.04
	stage.add_child(floor_node)

	var camera: Camera3D = Camera3D.new()
	camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	camera.size = 2.84
	stage.add_child(camera)
	camera.look_at_from_position(Vector3(0.0, 1.5, 5.0), Vector3(0.0, 0.89, 0.0))
	camera.make_current()
	RenderingServer.global_shader_parameter_set("w3d_night", 0.0)
	RenderingServer.global_shader_parameter_set("w3d_xray", 0.0)

	var clips: Dictionary = {}
	for i: int in People.STYLE2_SCALES.size():
		var model_id: String = "s2_p%d" % (i + 1)
		var vat: RefCounted = Vat.load_character(model_id)
		if not bool(vat.get("ok")):
			_fail("missing VAT for " + model_id)
			return
		var walk: Dictionary = (vat.get("meta") as Dictionary).get("clips", {}).get("walk", {})
		if int(walk.get("count", 0)) < 2 or not bool(walk.get("loop", false)):
			_fail("missing walk loop for " + model_id)
			return
		clips[model_id] = walk
		var model: MultiMeshInstance3D = MultiMeshInstance3D.new()
		var multi: MultiMesh = MultiMesh.new()
		multi.transform_format = MultiMesh.TRANSFORM_3D
		multi.use_colors = true
		multi.use_custom_data = true
		multi.mesh = vat.get("mesh")
		multi.instance_count = 1
		var size: float = People.STYLE2_SCALES[i]
		multi.set_instance_transform(0, Transform3D(Basis(Vector3.UP, deg_to_rad(42.0)).scaled(Vector3.ONE * size), Vector3((i - 1.5) * 1.13, 0.0, 0.0)))
		multi.set_instance_color(0, Color(1.0, 1.0, 1.0, 0.5))
		multi.set_instance_custom_data(0, Color(float(vat.call("clip_pack", "walk")), 0.0, 0.0, 0.0))
		model.multimesh = multi
		model.material_override = vat.call("with_findability", vat.call("material", Color.WHITE, true), true)
		stage.add_child(model)
	for frame: int in 36:
		RenderingServer.global_shader_parameter_set("w3d_time", frame / 12.0)
		await process_frame
		await RenderingServer.frame_post_draw
		var image: Image = root.get_texture().get_image()
		if image.get_size() != Vector2i(1920, 1080) or image.save_png(out.path_join("walk-%02d.png" % frame)) != OK:
			_fail("could not save frame %d" % frame)
			return
	print("S2KIT_WALK_DONE " + JSON.stringify({"frames": 36, "fps": 12, "clips": clips}))
	quit(0)


func _fail(message: String) -> void:
	push_error("S2KIT_WALK: " + message)
	quit(1)
