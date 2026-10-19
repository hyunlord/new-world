extends SceneTree
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
func _initialize() -> void:
	_run.call_deferred()
func _run() -> void:
	for settle_before: bool in [false, true, false]:
		var renderer = Renderer.new()
		renderer.call_deferred("_on_world_reset")
		if settle_before:
			await process_frame
			await process_frame
		renderer.set_perf_candidate_p10("seat-input-cache", false)
		var before: Dictionary = renderer.get_perf_candidate_p10()
		await process_frame
		await process_frame
		var after: Dictionary = renderer.get_perf_candidate_p10()
		print("settle_before=", settle_before, " before=",before," after=",after)
		assert(bool(after["candidate_enabled"]) == not settle_before)
		renderer.free()
	quit(0)
