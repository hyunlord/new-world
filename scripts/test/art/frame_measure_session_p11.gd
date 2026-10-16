extends RefCounted

const ReaderCompat = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const Probe = preload("res://scripts/test/art/frame_cpu_probe_p11.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const TRANSITION_FRAMES: int = 3

class Runtime extends RefCounted:
	var tree: SceneTree
	var world: Node
	var camera: Camera2D
	var probe: Node
	var viewport: RID
	var prior_vsync: int
	var prior_camera_mode: int
	var prior_layers: bool
	# This exclusive capture viewport starts with measurement disabled; Godot exposes no getter.
	var prior_measure_render_time: bool = false
	var previous_usec: int
	var previous_process_frame: int = -1

	func begin() -> Dictionary:
		prior_vsync = DisplayServer.window_get_vsync_mode()
		prior_camera_mode = camera.process_mode
		prior_layers = Selection.layers_enabled
		camera.process_mode = Node.PROCESS_MODE_DISABLED
		viewport = tree.root.get_viewport_rid()
		DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
		RenderingServer.viewport_set_measure_render_time(viewport, true)
		probe = Probe.new()
		tree.root.add_child(probe)
		previous_usec = Time.get_ticks_usec()
		return snapshot()

	func snapshot() -> Dictionary:
		var settled: Dictionary = ReaderCompat.read(world, "get_settlement_snapshot", true)
		var alive: int = 0
		for count: int in settled.get("member_counts", []):
			alive += int(count > 0)
		return {"tick": int(ReaderCompat.read(world, "get_time_snapshot", true)["tick"]), "camera": [camera.position.x, camera.position.y], "zoom": [camera.zoom.x, camera.zoom.y], "population": ReaderCompat.read(world, "get_agent_snapshot", true)["agent_ids"].size(), "alive_count": alive, "selected_id": Selection.selected_id, "simulation_frozen": world.process_mode == Node.PROCESS_MODE_DISABLED}

	func set_enabled(enabled: bool) -> void:
		Selection.set_layers_enabled(tree, enabled)

	func next_frame() -> Dictionary:
		var sample: Dictionary = await probe.frame_completed
		# Obscured macOS windows may omit automatic draws; this is outside the CPU bracket.
		RenderingServer.force_draw(false)
		var now: int = Time.get_ticks_usec()
		var frame: int = int(sample.get("process_frame", -1))
		sample["paired"] = bool(sample.get("paired", false)) and frame > previous_process_frame
		previous_process_frame = frame
		sample["render_cpu_ms"] = RenderingServer.viewport_get_measured_render_time_cpu(viewport)
		sample["setup_cpu_ms"] = RenderingServer.get_frame_setup_time_cpu()
		sample["gpu_ms"] = RenderingServer.viewport_get_measured_render_time_gpu(viewport)
		sample["gpu_status"] = "measured" if float(sample["gpu_ms"]) > 0.0 else "unavailable"
		sample["pacing_ms"] = float(now - previous_usec) / 1000.0
		previous_usec = now
		sample["primary_cpu_ms"] = float(sample.get("process_ms", 0.0)) + float(sample["render_cpu_ms"])
		sample["total_with_setup_ms"] = float(sample["primary_cpu_ms"]) + float(sample["setup_cpu_ms"])
		sample["state"] = snapshot()
		return sample

	func restore() -> Dictionary:
		Selection.set_layers_enabled(tree, prior_layers)
		RenderingServer.viewport_set_measure_render_time(viewport, prior_measure_render_time)
		DisplayServer.window_set_vsync_mode(prior_vsync as DisplayServer.VSyncMode)
		camera.process_mode = prior_camera_mode as Node.ProcessMode
		# The collector resumes inside the completion signal; defer destruction safely.
		probe.get_parent().remove_child(probe)
		probe.queue_free()
		return {"vsync_restored": DisplayServer.window_get_vsync_mode() == prior_vsync, "layers_restored": Selection.layers_enabled == prior_layers, "camera_process_restored": camera.process_mode == prior_camera_mode, "render_measure_restored_to": prior_measure_render_time, "probes_removed": true}

static func collect(backend: RefCounted) -> Dictionary:
	var initial: Dictionary = backend.begin()
	var blocks: Array[Dictionary] = []
	var discarded: Array[Dictionary] = []
	var errors: Array[String] = []
	if not bool(initial.get("simulation_frozen", false)) or int(initial.get("alive_count", 0)) < 3:
		errors.append("requires frozen world with at least three living settlements")
	for block: int in range(12):
		if not errors.is_empty():
			break
		var enabled: bool = block % 2 == 0
		backend.set_enabled(enabled)
		for warmup: int in range(TRANSITION_FRAMES):
			var sample: Dictionary = await backend.next_frame()
			discarded.append({"block": block, "layers_enabled": enabled, "warmup_index": warmup, "sample": sample})
		var samples: Array[Dictionary] = []
		for index: int in range(50):
			var sample: Dictionary = await backend.next_frame()
			if not bool(sample.get("paired", false)) or sample.get("state", {}) != initial:
				errors.append("unpaired process stamps or changed tick/camera/population")
				break
			for key: String in ["process_ms", "render_cpu_ms", "setup_cpu_ms", "gpu_ms"]:
				if not is_finite(float(sample[key])) or float(sample[key]) < 0.0:
					errors.append("nonfinite or negative CPU/GPU measurement")
			sample["primary_cpu_ms"] = float(sample["process_ms"]) + float(sample["render_cpu_ms"])
			sample["total_with_setup_ms"] = float(sample["primary_cpu_ms"]) + float(sample["setup_cpu_ms"])
			sample["frame_index"] = index
			sample["block_index"] = block
			sample["layers_enabled"] = enabled
			samples.append(sample)
		blocks.append({"block": block, "layers_enabled": enabled, "tick": initial.get("tick", -1), "camera": initial.get("camera", []), "samples": samples})
	var final: Dictionary = backend.snapshot()
	var restored: Dictionary = backend.restore()
	for key: String in ["vsync_restored", "layers_restored", "camera_process_restored", "probes_removed"]:
		if not bool(restored.get(key, false)):
			errors.append("restoration failed: " + key)
	if final != initial:
		errors.append("world/camera state changed during measurement")
	var values_on: Array[float] = []
	var values_off: Array[float] = []
	var viewport_positive: int = 0
	for block: Dictionary in blocks:
		for sample: Dictionary in block["samples"]:
			var total: float = float(sample["process_ms"]) + float(sample["render_cpu_ms"])
			if bool(block["layers_enabled"]):
				values_on.append(total)
			else:
				values_off.append(total)
			viewport_positive += int(float(sample["render_cpu_ms"]) > 0.0)
	if values_on.size() != 300 or values_off.size() != 300:
		errors.append("requires exactly 300 counted frames in each condition")
	if viewport_positive == 0:
		errors.append("viewport render CPU unavailable: all measurements are zero")
	var on_median: float = median(values_on)
	var off_median: float = median(values_off)
	return {"blocks": blocks, "discarded_transition_frames": discarded, "transition_frames_per_block": TRANSITION_FRAMES, "before": initial, "after": final, "restoration": restored, "vsync_disabled": true, "vsync_restored": restored.get("vsync_restored", false), "forced_draw": true, "draw_trigger": "deferred end-process stamp; force_draw(false) outside measured process span", "process_priorities": [-1000, 1000], "simulation_frozen": initial.get("simulation_frozen", false), "tick": initial.get("tick", -1), "alive_count": initial.get("alive_count", 0), "population": initial.get("population", 0), "framing": "FV", "median_primary_cpu_ms": {"on": on_median, "off": off_median}, "threshold_pass": errors.is_empty() and accepts(on_median, off_median), "valid": errors.is_empty(), "errors": errors, "render_cpu_positive_samples": viewport_positive, "metric": "process_ms + render_cpu_ms; setup_cpu_ms separately reported, total_with_setup_ms diagnostic", "render_timestamp_alignment": "GLES readout may lag process stamps; 3 frames discarded after every toggle, counted samples remain within one condition"}

static func median(values: Array[float]) -> float:
	if values.is_empty():
		return 0.0
	var sorted: Array[float] = values.duplicate()
	sorted.sort()
	@warning_ignore("integer_division")
	var middle: int = sorted.size() / 2
	return (sorted[middle - 1] + sorted[middle]) / 2.0 if sorted.size() % 2 == 0 else sorted[middle]

static func accepts(enabled_ms: float, disabled_ms: float) -> bool:
	return enabled_ms <= 1.15 * disabled_ms or enabled_ms <= disabled_ms + 1.0
