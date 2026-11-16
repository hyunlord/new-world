# Home framing: pure group translation, explicit exhaustion and paused native identity.
# Bridge methods: advance_ticks (setup only), get_agent_snapshot, get_time_snapshot,
# get_command_log, save_world; ordinary scene bootstrap calls set_sim_speed.
# No simulation commands. Run with --live for the scene block and external settings. M3D-SWITCH E2: the live block
# frames the 3D people (World3D/People bodies); each body is checked by an independent projection of a box around the
# figure (feet ± its radius, up to its height) through the World3D camera, not by the view's person_rect.
extends SceneTree
const Layout = preload("res://scripts/world/colony/marker_layout.gd")
var failures: int = 0
var checks: int = 0
var live_completed: bool = false
var animation_completed: bool = false
var layout: RefCounted = Layout.new()

func _init() -> void:
	call_deferred("_run")

func check(label: String, ok: bool) -> void:
	checks += 1
	print("CAMP_FRAMING %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1

func _run() -> void:
	if not layout.has_method("frame_translation"):
		check("group framing API exists", false)
		quit(1)
		return
	var area: Rect2 = Rect2(0, 0, 100, 100)
	var bodies: Array[Rect2] = [Rect2(10, 10, 10, 10), Rect2(70, 70, 10, 10)]
	var before: Array[Rect2] = bodies.duplicate()
	var obstacles: Array[Rect2] = [Rect2(40, 0, 20, 100)]
	var result: Dictionary = layout.call("frame_translation", bodies, obstacles, area, 0.0)
	check("sparse group fits despite merged bounding box", result["found"] and result["translation"] == Vector2.ZERO)
	check("input immutable and every body retained", bodies == before and result["body_count"] == 2)
	obstacles = [Rect2(0, 0, 30, 30)]
	result = layout.call("frame_translation", bodies, obstacles, area, 0.0)
	check("translation found", result["found"] and result["translation"] != Vector2.ZERO)
	var shift: Vector2 = result["translation"]
	for body: Rect2 in bodies:
		check("translated body stays in viewport and clears obstacle", area.encloses(Rect2(body.position + shift, body.size)) and not Rect2(body.position + shift, body.size).intersects(obstacles[0]))
	check("deterministic", result == layout.call("frame_translation", bodies, obstacles, area, 0.0))
	obstacles = [area]
	result = layout.call("frame_translation", bodies, obstacles, area, 0.0)
	check("no fit explicit zero fallback", not result["found"] and result["translation"] == Vector2.ZERO and result["body_count"] == 2)
	var fractional_bodies: Array[Rect2] = [Rect2(10.25, 10.25, 10.5, 10.5)]
	var fractional_obstacles: Array[Rect2] = [Rect2(10.0, 10.0, 15.0, 15.0)]
	var fractional: Dictionary = layout.call("frame_translation", fractional_bodies, fractional_obstacles, area)
	var rounded: Vector2 = (fractional["translation"] as Vector2).round()
	check("fractional solve survives safe pixel rounding", fractional["found"] and layout.call("frame_translation_clear", fractional_bodies, fractional_obstacles, area, rounded))
	check("unsafe achieved translation rejected", not layout.call("frame_translation_clear", fractional_bodies, fractional_obstacles, area, Vector2.ZERO))
	var hidden_panel: Panel = Panel.new()
	root.add_child(hidden_panel)
	hidden_panel.size = Vector2(100, 100)
	hidden_panel.hide()
	var visible_obstacles: Array[Rect2] = []
	Layout.ui_rects(hidden_panel, visible_obstacles)
	check("hidden panels contribute no footprint", visible_obstacles.is_empty())
	hidden_panel.queue_free()
	var empty: Array[Rect2] = []
	check("empty is not visibility success", not layout.call("frame_translation", empty, empty, area)["found"])
	var outside: Array[Rect2] = [Rect2(-1, 0, 10, 10)]
	check("invalid original set not silently filtered", not layout.call("frame_translation", outside, empty, area)["found"])
	if "--live" in OS.get_cmdline_user_args():
		await live()
		check("live block completed", live_completed)
	print("CAMP_FRAMING_DONE checks=%d failures=%d" % [checks, failures])
	quit(0 if failures == 0 else 1)

func settle() -> void:
	for _i: int in 30:
		await process_frame

func live() -> void:
	var directory: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not directory.is_absolute_path() or not DirAccess.dir_exists_absolute(directory) or directory.begins_with(ProjectSettings.globalize_path("res://")):
		check("external settings directory before instantiation", false)
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	world.call("advance_ticks", 60)
	var camera: Camera2D = main.get_node("Camera2D")
	camera.zoom = Vector2.ONE * 0.5
	await settle()
	var tick: Dictionary = world.call("get_time_snapshot")
	var log_before: Variant = world.call("get_command_log", 0)
	var first: String = directory.path_join("before.save")
	var last: String = directory.path_join("after.save")
	check("before save", bool(world.call("save_world", first)))
	camera.call("center_on_camp")
	await settle()
	var report: Dictionary = camera.call("camp_framing_report")
	print("CAMP_FRAMING_REPORT " + JSON.stringify(report))
	check("half zoom Home finds a clear view", report.get("status", "") == "placed")
	if report.get("status", "") == "placed":
		var shift: Vector2 = report["translation"]
		for body: Rect2 in report["bodies"]:
			var moved: Rect2 = Rect2(body.position + shift, body.size)
			check("original body retained on screen", camera.get_viewport_rect().encloses(moved))
			for obstacle: Rect2 in report["obstacles"]:
				check("original body clears actual HUD", not moved.intersects(obstacle))
		var w3d: Node = main.get_node("World3D")
		var people: Node = w3d.get_node("People")
		var current_obstacles: Array[Rect2] = []
		Layout.ui_rects(main.get_node("UI"), current_obstacles)
		var snapshot: Dictionary = world.call("get_agent_snapshot")
		var original_ids: Array[int] = []
		for bits: int in snapshot.get("ids", PackedInt64Array()):
			var body: Dictionary = people.call("body_of", bits)
			check("native resident has rendered person", not body.is_empty())
			if body.is_empty():
				continue
			var current: Rect2 = independent_body_bounds(w3d, body)
			check("native resident has nonempty body", current.has_area())
			var original: Rect2 = Rect2(current.position - shift, current.size)
			if camera.get_viewport_rect().encloses(original):
				original_ids.append(bits)
				check("fresh body projection retained", camera.get_viewport_rect().encloses(current))
				for obstacle: Rect2 in current_obstacles:
					check("fresh body projection clear", not current.intersects(obstacle))
		var reported_ids: Array = report["ids"].duplicate()
		reported_ids.sort()
		original_ids.sort()
		check("independent original native membership exact", not original_ids.is_empty() and reported_ids == original_ids)
	var home: Vector2 = camera.position
	print("CAMP_FRAMING_HOME_BEFORE " + JSON.stringify({"position": str(home), "report": camera.call("camp_framing_report")}))
	camera.call("center_on_camp")
	await settle()
	print("CAMP_FRAMING_HOME_AFTER " + JSON.stringify({"position": str(camera.position), "report": camera.call("camp_framing_report")}))
	check("Home idempotent", camera.position.is_equal_approx(home))
	camera.call("center_on_camp")
	camera.call("pan_screen", Vector2(16, 0))
	var panned: Vector2 = camera.position
	await settle()
	check("pan cancels delayed framing", camera.position.is_equal_approx(panned))
	camera.call("center_on_camp")
	camera.call("center_on_tile", Vector2i(32, 32))
	var explicit_position: Vector2 = camera.position
	await settle()
	check("explicit tile center cancels Home", camera.position.is_equal_approx(explicit_position))
	camera.call("center_on_camp")
	camera.call("step_zoom", 1)
	await settle()
	check("zoom cancels delayed framing", camera.call("camp_framing_report").get("status", "") == "cancelled")
	# Re-enter the half-zoom Home view, then watch a full real work-animation cycle.
	camera.zoom = Vector2.ONE * 0.5
	camera.call("center_on_camp")
	await settle()
	await animation_envelopes(main)
	check("animation block completed", animation_completed)
	check("no native tick changes", tick == world.call("get_time_snapshot"))
	check("no command changes", log_before == world.call("get_command_log", 0))
	check("after save", bool(world.call("save_world", last)))
	var bytes: PackedByteArray = FileAccess.get_file_as_bytes(first)
	check("nonempty identical save bytes", not bytes.is_empty() and bytes == FileAccess.get_file_as_bytes(last))
	live_completed = true
	main.queue_free()
	await process_frame


# Independent projection deliberately bypasses production's person_rect: the screen bounds of a box around the
# figure (feet ± HALF_WIDTH tiles, feet to head) through the World3D camera; Rect2() when behind the camera.
const HALF_WIDTH: float = 0.3


func independent_body_bounds(w3d: Node, body: Dictionary) -> Rect2:
	var cam: Camera3D = w3d.call("camera") as Camera3D
	if cam == null or body.is_empty():
		return Rect2()
	var feet: Vector3 = body["feet"]
	var out: Rect2 = Rect2()
	var first: bool = true
	for dy: float in [0.0, float(body["h"])]:
		for dx: float in [-HALF_WIDTH, HALF_WIDTH]:
			for dz: float in [-HALF_WIDTH, HALF_WIDTH]:
				var p: Vector3 = feet + Vector3(dx, dy, dz)
				if cam.is_position_behind(p):
					return Rect2()
				var s: Vector2 = cam.unproject_position(p)
				out = Rect2(s, Vector2.ZERO) if first else out.expand(s)
				first = false
	return Rect2(out.position.floor(), out.end.ceil() - out.position.floor())


## The paused Home view over a second of real rendering: every figure keeps its body (feet and height, what its
## screen rect is projected from) and its clip while the 3D clips play in the vertex shader. (The 2D test also checked
## a stable envelope around each sprite animation phase; a 3D figure's framing box does not depend on its pose.)
func animation_envelopes(main: Node) -> void:
	var w3d: Node = main.get_node("World3D")
	var people: Node = w3d.get_node("People")
	var bodies: Dictionary = {}
	var clips: Dictionary = {}
	for body: Dictionary in people.call("bodies", 0):
		var bits: int = int(body["id"])
		bodies[bits] = people.call("body_of", bits)
		clips[bits] = str((people.call("state_of", bits) as Dictionary).get("clip", ""))
	check("native fixture has drawn figures with clips", not bodies.is_empty() and not clips.values().has(""))
	# Real elapsed rendering time, not a frame-count assumption or a forced pose.
	var deadline: int = Time.get_ticks_msec() + 1100
	var body_stable: bool = true
	var clips_kept: bool = true
	while Time.get_ticks_msec() < deadline:
		await process_frame
		for bits: int in bodies:
			body_stable = body_stable and bodies[bits] == people.call("body_of", bits)
			clips_kept = clips_kept and clips[bits] == str((people.call("state_of", bits) as Dictionary).get("clip", ""))
	check("body unchanged across real animation cycle", body_stable)
	check("every figure keeps its clip", clips_kept)
	print("CAMP_FRAMING_ANIMATION " + JSON.stringify(clips.values()))
	animation_completed = true
