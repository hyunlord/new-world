extends RefCounted

const NAMES: Array[String] = ["reads_usec", "checksum_talk_usec", "chill_ingest_usec", "activity_context_usec", "tile_contexts_usec", "agent_setup_usec", "look_usec", "pose_usec", "frame_choice_usec", "variant_usec", "meta_usec", "children_usec", "draw_rows_usec", "blocked_usec", "seats_usec", "footprints_usec", "declump_usec", "easing_usec", "walker_offsets_usec", "walker_apply_usec", "post_loop_usec", "retire_usec", "selection_usec", "causal_usec", "other_usec"]
var _values := PackedInt64Array()
var _active: int = -1
var _started: int = 0
var _boundary: int = 0
var _frame: int = -1
var _complete: bool = false
var _reason: String = "not_started"
var _total: int = 0
var _clock_reads: int = 0
var agent_count: int = 0
var tile_context_misses: int = 0

func begin() -> void:
	_values.resize(NAMES.size())
	_values.fill(0)
	_frame = Engine.get_process_frames()
	_complete = false
	_reason = "callback_did_not_finish"
	agent_count = 0
	tile_context_misses = 0
	_clock_reads = 1
	_started = Time.get_ticks_usec()
	_boundary = _started
	_active = NAMES.size() - 1
	_total = 0

func mark(index: int) -> void:
	var now: int = Time.get_ticks_usec()
	_clock_reads += 1
	_values[_active] += now - _boundary
	_boundary = now
	_active = index

func finish(complete: bool, reason: String = "") -> void:
	var now: int = Time.get_ticks_usec()
	_clock_reads += 1
	_values[_active] += now - _boundary
	_total = now - _started
	_complete = complete
	_reason = reason
	_active = -1

func readout() -> Dictionary:
	var values: Dictionary = {}
	var sum_usec: int = 0
	for index: int in NAMES.size():
		var value: int = _values[index] if index < _values.size() else 0
		values[NAMES[index]] = value
		sum_usec += value
	var aligned: bool = _frame == Engine.get_process_frames()
	return {"schema_version": 1, "process_frame": _frame, "complete": _complete, "valid": _complete and aligned and sum_usec == _total, "error": _reason if not _complete else ("" if aligned else "stale_frame"), "instrumented": true, "unit": "usec", "agent_count": agent_count, "tile_context_misses": tile_context_misses, "total_usec": _total, "exclusive_usec": values, "remainder_usec": _total - sum_usec, "clock_reads": _clock_reads, "founding_groups_usec": null, "founding_groups_status": "unavailable_separate_callback"}
