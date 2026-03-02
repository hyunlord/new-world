extends RefCounted

var _report_path: String = "user://reports/rust_shadow/latest.json"
var _flush_interval_ticks: int = 500

var _frames: int = 0
var _mismatch_frames: int = 0
var _max_tick_delta: int = 0
var _sum_tick_delta: int = 0
var _max_event_delta: int = 0
var _sum_event_delta: int = 0
var _last_flushed_tick: int = 0


## Initializes shadow reporter output path and flush interval.
func setup(report_path: String, flush_interval_ticks: int) -> void:
	_report_path = report_path
	_flush_interval_ticks = maxi(flush_interval_ticks, 1)


## Records one frame of GD vs Rust shadow comparison.
func record_frame(current_tick: int, gd_tick: int, rust_tick: int, gd_events: int, rust_events: int) -> void:
	_frames += 1
	var tick_delta: int = absi(gd_tick - rust_tick)
	var event_delta: int = absi(gd_events - rust_events)
	_sum_tick_delta += tick_delta
	_sum_event_delta += event_delta
	_max_tick_delta = maxi(_max_tick_delta, tick_delta)
	_max_event_delta = maxi(_max_event_delta, event_delta)
	if tick_delta > 0 or event_delta > 0:
		_mismatch_frames += 1
	if current_tick - _last_flushed_tick < _flush_interval_ticks:
		return
	_flush_report(current_tick)
	_last_flushed_tick = current_tick


## Writes current shadow comparison report to disk.
func flush_now(current_tick: int) -> void:
	_flush_report(current_tick)
	_last_flushed_tick = current_tick


func _flush_report(current_tick: int) -> void:
	var avg_tick_delta: float = 0.0
	var avg_event_delta: float = 0.0
	if _frames > 0:
		avg_tick_delta = float(_sum_tick_delta) / float(_frames)
		avg_event_delta = float(_sum_event_delta) / float(_frames)

	var payload: Dictionary = {
		"current_tick": current_tick,
		"frames": _frames,
		"mismatch_frames": _mismatch_frames,
		"max_tick_delta": _max_tick_delta,
		"avg_tick_delta": avg_tick_delta,
		"max_event_delta": _max_event_delta,
		"avg_event_delta": avg_event_delta,
		"generated_at_unix_ms": Time.get_ticks_msec(),
	}

	var abs_path: String = ProjectSettings.globalize_path(_report_path)
	var dir_path: String = abs_path.get_base_dir()
	DirAccess.make_dir_recursive_absolute(dir_path)
	var file: FileAccess = FileAccess.open(abs_path, FileAccess.WRITE)
	if file == null:
		return
	file.store_string(JSON.stringify(payload, "  "))
