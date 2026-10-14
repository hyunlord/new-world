extends SceneTree

const Measure = preload("res://scripts/test/art/frame_measure_session.gd")
const Probe = preload("res://scripts/test/art/frame_cpu_probe.gd")
var failures: int = 0

class Backend extends RefCounted:
	var state: Dictionary = {"tick": 126994, "camera": [10, 20], "zoom": [1, 1], "population": 81, "alive_count": 3, "simulation_frozen": true, "selected_id": 1}
	var enabled: bool = true
	var restored: bool = false
	var frame: int = 0
	var corrupt_at: int = -1
	var zero_render: bool = false
	var broken_restore: bool = false
	func begin() -> Dictionary:
		return state.duplicate(true)
	func snapshot() -> Dictionary:
		return state.duplicate(true)
	func set_enabled(value: bool) -> void:
		enabled = value
	func next_frame() -> Dictionary:
		frame += 1
		if frame == corrupt_at:
			state["tick"] += 1
		return {"process_frame": frame, "paired": true, "process_ms": 2.0 if enabled else 1.5, "render_cpu_ms": 0.0 if zero_render else 1.0, "setup_cpu_ms": 9.0, "gpu_ms": 0.0, "gpu_status": "unavailable", "state": snapshot()}
	func restore() -> Dictionary:
		restored = true
		return {"vsync_restored": not broken_restore, "layers_restored": true, "camera_process_restored": true, "probes_removed": true}

func _initialize() -> void:
	call_deferred("run")

func check(condition: bool, label: String) -> void:
	if not condition:
		failures += 1
		push_error(label)

func run() -> void:
	var backend = Backend.new()
	var result: Dictionary = await Measure.collect(backend)
	check(result["valid"] and backend.restored, "valid collection restores before returning")
	check(result["blocks"].size() == 12 and result["discarded_transition_frames"].size() == 36, "twelve blocks and three discarded frames per toggle")
	var on: int = 0
	var off: int = 0
	for index: int in result["blocks"].size():
		var block: Dictionary = result["blocks"][index]
		check(block["layers_enabled"] == (index % 2 == 0) and block["samples"].size() == 50, "alternating fifty-frame blocks")
		if block["layers_enabled"]:
			on += block["samples"].size()
		else:
			off += block["samples"].size()
	check(on == 300 and off == 300, "equal counted samples")
	check(result["median_primary_cpu_ms"] == {"on": 3.0, "off": 2.5}, "primary excludes separately recorded setup CPU")
	check(Measure.accepts(11.4, 10.0) and Measure.accepts(1.8, 1.0) and not Measure.accepts(11.6, 10.0), "OR threshold covers relative and absolute alternatives")
	var corrupt = Backend.new()
	corrupt.corrupt_at = 20
	var invalid: Dictionary = await Measure.collect(corrupt)
	check(not invalid["valid"] and corrupt.restored, "state drift aborts and restores")
	var zero = Backend.new()
	zero.zero_render = true
	invalid = await Measure.collect(zero)
	check(not invalid["valid"] and zero.restored, "unavailable viewport CPU cannot pass")
	var broken = Backend.new()
	broken.broken_restore = true
	invalid = await Measure.collect(broken)
	check(not invalid["valid"], "restoration failure invalidates result")
	var probe = Probe.new()
	root.add_child(probe)
	check(probe.first.process_priority == -1000 and probe.last.process_priority == 1000, "actual process priority brackets")
	probe.clock_usec = func() -> int: return 1000
	probe.frame_number = func() -> int: return 42
	probe.begin_stamp()
	probe.clock_usec = func() -> int: return 3500
	probe.end_stamp()
	check(probe.readout()["paired"] and probe.readout()["process_ms"] == 2.5, "paired monotonic microsecond stamps")
	probe.frame_number = func() -> int: return 43
	probe.end_stamp()
	check(not probe.readout()["paired"], "cross-frame stamps rejected")
	probe.free()
	var live_probe = Probe.new()
	root.add_child(live_probe)
	var completed: Dictionary = await live_probe.frame_completed
	check(completed["paired"] and completed["process_ms"] >= 0.0, "real process callbacks publish a paired sample without waiting for drawable")
	var next: Dictionary = await live_probe.frame_completed
	check(int(next["process_frame"]) > int(completed["process_frame"]), "one completed sample per distinct Engine frame")
	root.remove_child(live_probe)
	live_probe.queue_free()
	print("test_frame_measure failures=", failures)
	quit(1 if failures else 0)
