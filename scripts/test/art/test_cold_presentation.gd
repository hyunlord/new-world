extends SceneTree
const Cold = preload("res://scripts/ui/cold_effects.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
var failures: int = 0

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	for pose: String in ["idle", "sit", "hop"]:
		check(Cold.pose(pose, true, false) == "crossed", "cold stationary eligible pose")
		check(Cold.pose(pose, false, false) == pose, "warm unchanged")
	for pose: String in ["eat", "asleep", "talk", "walk"]:
		check(Cold.pose(pose, true, false) == pose, "meal sleep talk and walking win")
	check(Cold.pose("walk", true, true) == "walk", "moving cold stays walking")
	for snapshot: Dictionary in [{}, {"colds": [1]}, {"colds": PackedByteArray()}, {"colds": PackedInt32Array([1])}]:
		check(not Cold.snapshot_cold(snapshot, 0), "absent or invalid optional cold marker unchanged")
	check(Cold.snapshot_cold({"colds": PackedByteArray([1, 0])}, 0), "typed marker")
	check(not Cold.snapshot_cold({"colds": PackedByteArray([1, 0])}, 1), "zero marker")
	check(Cold.shiver(true, false, 0, 0.0) == 1.0 and Cold.shiver(true, false, 0, 1.0 / 6.0) == 0.0, "shiver alternates each sixth second")
	check(Cold.shiver(true, true, 0, 0.0) == 0.0, "moving has no shiver")
	check(Cold.shiver(false, false, 0, 0.0) == 0.0, "warm has no shiver")
	check(is_equal_approx(float(Cold.breath(true, 0, 0.5)["alpha"]), 0.275), "breath fades")
	check(Cold.breath(true, 0, 1.1)["alpha"] == 0.0, "breath off after one second")
	check(Cold.breath(false, 0, 0.0)["alpha"] == 0.0, "warm never breathes")
	var entries: Dictionary = {
		"bad": {"body": "adult_b", "activity": "crossed", "view": "S", "qa_status": "accepted"},
		"sit": {"body": "adult_b", "activity": "sit", "view": "E", "qa_status": "accepted"},
		"idle": {"body": "adult_b", "activity": "idle", "view": "E", "qa_status": "accepted"}}
	var groups: Dictionary = Activity.build_frame_groups(entries)
	groups["adult_b/crossed/S"][0]["cold_audit_rejected"] = true
	check(Activity.choose_frame(groups, "adult_b", "crossed", "E", 0.0, 0.0)["sprite_id"] == "sit", "rejected crossed falls back to same view sit")
	groups["adult_b/crossed/S"][0]["cold_audit_rejected"] = false
	check(Activity.choose_frame(groups, "adult_b", "crossed", "E", 0.0, 0.0)["sprite_id"] == "bad", "usable south crossed precedes sit")
	var audit: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://scripts/ui/cold_audit.json"))
	check(Cold.audit_rejected("V_b3_16", str(audit["V_b3_16"])), "audited pixels rejected")
	check(not Cold.audit_rejected("V_b3_16", "replaced"), "changed pixels release audit rejection")
	var actual: Dictionary = Activity.build_frame_groups(Library.entries())
	for frame: Dictionary in actual.get("adult_b/crossed/S", []):
		if frame.get("sprite_id") == "V_b3_16" and bool(frame.get("cold_audit_rejected")):
			check(Activity.choose_frame(actual, "adult_b", "crossed", "S", 0.0, 0.0).get("sprite_id") != "V_b3_16", "runtime crop binding excludes audited crossed")
	var sprite: Sprite2D = Sprite2D.new()
	root.add_child(sprite)
	sprite.offset.y = -28.0
	Cold.apply(sprite, true, true, 0, 0.0)
	var puff: Sprite2D = sprite.get_node("ColdBreath") as Sprite2D
	check(puff.visible and puff.texture.get_size() == Vector2(3, 2), "moving cold gets runtime 3x2 puff")
	check(is_equal_approx(puff.modulate.a, 0.55) and puff.position.y == -31.0, "puff two pixels above top and white alpha")
	Cold.apply(sprite, false, false, 0, 0.0)
	check(not puff.visible and sprite.get_meta("puff_alpha") == 0.0, "warm removes puff and metadata")
	var renderer: Node2D = Renderer.new()
	var bridge := ClassDB.instantiate("WorldSimNode") as WorldSimNode
	renderer.world_sim = bridge
	renderer._sprites[0] = sprite
	var rows: Array[Dictionary] = [{"id": 0, "ipos": Vector2(200, 200), "tile": Vector2i(0, 0), "visible": true, "moving": false, "stationary_seconds": 1.0, "pose": "crossed", "meal": false}]
	for step: int in 60:
		renderer._apply_drawn_positions(rows, {"hearths": [], "seat_blocked": {}, "terrain": {}}, 1.0 / 60.0)
		Cold.apply(sprite, true, false, 0, float(step) / 60.0)
		check(renderer.get_drawn_position(0) == sprite.position, "cold picking follows shiver")
		check(sprite.position.x == 200.0 + Cold.shiver(true, false, 0, float(step) / 60.0), "shiver never feeds back into smoothed target")
	renderer.free()
	bridge.free()
	sprite.free()
	print("COLD_PRESENTATION failures=", failures)
	quit(1 if failures else 0)
