# Frozen receipt controls: synthetic durations test attribution, never establish real bake performance.
# Bridge methods: none. The production FPS tool supplies the summary implementation under test.
extends SceneTree

const Fps = preload("res://tools/v8/mart2_fps.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BAKE_RECEIPT_%s %s" % ["PASS" if valid else "FAIL", label])


func _event(sequence: int, start: int, duration: int, kind: String, context: String, epoch: int, frame: int) -> Dictionary:
	return {"sequence": sequence, "start_usec": start, "end_usec": start + duration, "total_usec": duration,
		"compose_usec": duration, "upload_usec": 0, "strips_usec": 0, "compose_upload_usec": duration,
		"kind": kind, "context": context, "reset_epoch": epoch, "frame": frame,
		"chunk": [7, 3] if epoch == 0 else [1, 2], "factor": 1 if epoch == 0 else 2,
		"map_size": [192, 192] if epoch == 0 else [64, 64], "previous_bakes": 0 if kind == "fresh" else 1}


func _run() -> void:
	var old: Dictionary = _event(0, 10, 104496, "fresh", "bootstrap", 0, 1)
	var warm: Dictionary = _event(1, 110000, 4495, "rebake", "warmup", 1, 20)
	var earlier: Array[Dictionary] = [old, warm]
	var stale: Dictionary = Fps.summarize_bakes(earlier, 200000, 300000)
	_check("stale_last_value_not_a_measurement", stale["measured_rebake_status"] == "NOT_MEASURED")
	_check("bootstrap_max_retained", stale["groups"]["all"]["max_total_usec"] == 104496)
	_check("warmup_4495_retained_separately", stale["groups"]["context/warmup/rebake"]["max_total_usec"] == 4495)
	_check("no_measured_events_invented", not stale["groups"].has("measurement"))
	var slow: Dictionary = _event(2, 210000, 7000, "rebake", "measurement", 1, 60)
	var fast: Dictionary = _event(3, 220000, 2000, "rebake", "measurement", 1, 60)
	var events: Array[Dictionary] = [old, warm, slow, fast]
	var before: Array[Dictionary] = events.duplicate(true)
	var report: Dictionary = Fps.summarize_bakes(events, 200000, 300000)
	_check("multiple_same_frame_bakes_all_retained", report["groups"]["measurement"]["count"] == 2)
	_check("last_fast_bake_cannot_hide_earlier_slow_bake", report["groups"]["measurement"]["max_total_usec"] == 7000)
	_check("measured_slow_rebake_fails_unchanged_4000", report["measured_rebake_status"] == "FAIL" and report["measured_rebake_budget_usec"] == 4000)
	_check("raw_input_not_rewritten_or_reset", events == before and report["event_count"] == 4)
	_check("worst_measured_sequence_identifies_event", report["groups"]["measurement"]["worst_sequence"] == 2)
	_check("context_and_reset_epochs_remain_distinct", events[0]["reset_epoch"] == 0 and events[2]["reset_epoch"] == 1 and report["groups"].has("context/bootstrap/fresh"))
	var edge: Array[Dictionary] = [_event(0, 199000, 2000, "rebake", "warmup", 1, 40)]
	_check("start_boundary_overlap_is_measured", Fps.summarize_bakes(edge, 200000, 300000)["groups"]["measurement/rebake"]["count"] == 1)
	edge = [_event(0, 299000, 5000, "rebake", "measurement", 1, 70)]
	_check("end_boundary_spike_not_censored", Fps.summarize_bakes(edge, 200000, 300000)["measured_rebake_status"] == "FAIL")
	var cold: Array[Dictionary] = [_event(0, 210000, 9000, "fresh", "measurement", 1, 50)]
	var cold_report: Dictionary = Fps.summarize_bakes(cold, 200000, 300000)
	_check("measured_fresh_spike_retained", cold_report["groups"]["measurement/fresh"]["max_total_usec"] == 9000 and cold_report["groups"]["measurement"]["over_4000_total"] == 1)
	_check("fresh_only_does_not_claim_rebake_pass", cold_report["measured_rebake_status"] == "NOT_MEASURED")
	var strips: Dictionary = _event(0, 210000, 4500, "rebake", "measurement", 1, 50)
	strips["compose_usec"] = 3000
	strips["upload_usec"] = 1000
	strips["strips_usec"] = 500
	strips["compose_upload_usec"] = 4000
	var strip_events: Array[Dictionary] = [strips]
	_check("wall_strip_cost_not_excluded_to_pass", Fps.summarize_bakes(strip_events, 200000, 300000)["measured_rebake_status"] == "FAIL")
	var exact: Array[Dictionary] = [_event(0, 210000, 4000, "rebake", "measurement", 1, 50)]
	_check("exact_4000_boundary_allowed", Fps.summarize_bakes(exact, 200000, 300000)["measured_rebake_status"] == "PASS")
	var over: Array[Dictionary] = [_event(0, 210000, 4001, "rebake", "measurement", 1, 50)]
	_check("4001_fails", Fps.summarize_bakes(over, 200000, 300000)["measured_rebake_status"] == "FAIL")
	exact[0]["sequence"] = 5
	_check("missing_sequence_invalidates_receipt", Fps.summarize_bakes(exact, 200000, 300000)["measured_rebake_status"] == "INVALID")
	exact[0]["sequence"] = 0
	exact[0]["compose_usec"] = 3999
	_check("phase_sum_mismatch_invalidates_receipt", Fps.summarize_bakes(exact, 200000, 300000)["measured_rebake_status"] == "INVALID")
	_check("empty_window_is_invalid", not bool(Fps.summarize_bakes([], 200000, 200000)["valid_intervals"]))
	print("MUX2_BAKE_RECEIPT_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
