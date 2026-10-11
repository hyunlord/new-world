extends SceneTree
const Chill = preload("res://scripts/ui/morning_chill.gd")
const Cold = preload("res://scripts/ui/cold_effects.gd")
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var memory = Chill.new()
	var cold: Dictionary = {"agent_ids": PackedInt64Array([1, 2]), "colds": PackedByteArray([1, 0])}
	var warm: Dictionary = {"agent_ids": PackedInt64Array([1, 2]), "colds": PackedByteArray([0, 0])}
	memory.ingest(cold, 359)
	check(memory.alpha(1, 359) == 0.0, "current cold is never morning chill")
	memory.ingest(warm, 360)
	check(memory.since_tick(1) == 360, "first clear tick starts window")
	check(memory.alpha(1, 390) == 1.0, "06:30 full chill")
	check(memory.alpha(1, 420) == 1.0, "07:00 fade starts")
	check(memory.alpha(1, 435) == 0.5, "07:15 half chill")
	check(memory.alpha(1, 449) > 0.0, "07:29 last visible minute")
	check(memory.alpha(1, 450) == 0.0 and memory.alpha(1, 451) == 0.0, "07:30 fade ends; gone07:31")
	check(memory.alpha(2, 390) == 0.0, "never cold has no chill")
	memory.ingest(warm, 430)
	check(memory.since_tick(1) == 360, "repeated warm snapshots do not extend window")
	var sprite := Sprite2D.new()
	root.add_child(sprite)
	sprite.set_meta("activity", "idle")
	sprite.set_meta("moving", false)
	Cold.apply(sprite, false, false, 1, 0.0)
	memory.apply(sprite, 1, 390, 0.0)
	check(sprite.get_meta("chill_alpha") == 1.0, "visible awake actor receives chill")
	check(not sprite.get_meta("cold") and sprite.get_meta("shiver_offset") == 0.0 and sprite.get_meta("puff_alpha") == 0.0, "chill never rewrites cold metadata")
	check(sprite.get_node("ChillBreath").visible, "chill breath shown separately")
	for pose: String in ["idle", "sit", "hop"]:
		check(Cold.pose(pose, memory.alpha(1, 390) > 0.0, false) == "crossed", "eligible chill pose crosses arms")
	for pose: String in ["eat", "walk", "gather", "build", "asleep"]:
		check(Cold.pose(pose, true, false) == pose, "meal work sleep pose wins")
	sprite.set_meta("activity", "asleep")
	memory.apply(sprite, 1, 390, 0.0)
	check(sprite.get_meta("chill_alpha") == 0.0 and not sprite.get_node("ChillBreath").visible, "sleep hides chill")
	sprite.set_meta("activity", "idle")
	sprite.visible = false
	memory.apply(sprite, 1, 390, 0.0)
	check(sprite.get_meta("chill_alpha") == 0.0, "hidden actors have no chill cues")
	memory.clear()
	check(memory.alpha(1, 390) == 0.0 and memory.since_tick(1) == -1, "clear removes history")
	var renderer = Renderer.new()
	renderer.ingest_chill_snapshot(cold, 359)
	renderer.ingest_chill_snapshot(warm, 360)
	check(renderer._morning_chill.alpha(1, 390) == 1.0, "capture facade ingests chunk history")
	renderer._on_world_reset()
	check(renderer._morning_chill.alpha(1, 390) == 0.0, "world reset clears renderer memory")
	renderer.free()
	sprite.free()
	print("MORNING_CHILL failures=", failures)
	quit(1 if failures else 0)
