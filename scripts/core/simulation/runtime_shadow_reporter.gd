extends RefCounted

var _report_path: String = "user://reports/rust_shadow/latest.json"
var _flush_interval_ticks: int = 500
var _allowed_max_tick_delta: int = 0
var _allowed_max_work_delta: int = 0
var _allowed_mismatch_ratio: float = 0.0
var _min_frames_for_cutover: int = 10000

var _frames: int = 0
var _mismatch_frames: int = 0
var _max_tick_delta: int = 0
var _sum_tick_delta: int = 0
var _max_work_delta: int = 0
var _sum_work_delta: int = 0
var _last_flushed_tick: int = 0
var _last_approved_for_cutover: bool = false
var _last_payload: Dictionary = {}


## Initializes shadow reporter output path and flush interval.
func setup(
	report_path: String,
	flush_interval_ticks: int,
	allowed_max_tick_delta: int = 0,
	allowed_max_work_delta: int = 0,
	allowed_mismatch_ratio: float = 0.0,
	min_frames_for_cutover: int = 10000
) -> void:
	_report_path = report_path
	_flush_interval_ticks = maxi(flush_interval_ticks, 1)
	_allowed_max_tick_delta = maxi(allowed_max_tick_delta, 0)
	_allowed_max_work_delta = maxi(allowed_max_work_delta, 0)
	_allowed_mismatch_ratio = clampf(allowed_mismatch_ratio, 0.0, 1.0)
	_min_frames_for_cutover = maxi(min_frames_for_cutover, 1)
	_last_approved_for_cutover = false
	_last_payload.clear()


## Records one frame of GD vs Rust shadow comparison.
## gd_work/rust_work currently use ticks_processed in each runtime path.
func record_frame(current_tick: int, gd_tick: int, rust_tick: int, gd_work: int, rust_work: int) -> void:
	_frames += 1
	var tick_delta: int = absi(gd_tick - rust_tick)
	var work_delta: int = absi(gd_work - rust_work)
	_sum_tick_delta += tick_delta
	_sum_work_delta += work_delta
	_max_tick_delta = maxi(_max_tick_delta, tick_delta)
	_max_work_delta = maxi(_max_work_delta, work_delta)
	if tick_delta > 0 or work_delta > 0:
		_mismatch_frames += 1
	if current_tick - _last_flushed_tick < _flush_interval_ticks:
		return
	_flush_report(current_tick)
	_last_flushed_tick = current_tick


## Writes current shadow comparison report to disk.
func flush_now(current_tick: int) -> void:
	_flush_report(current_tick)
	_last_flushed_tick = current_tick


## Returns current approval status for Rust primary cutover.
func is_approved_for_cutover() -> bool:
	return _last_approved_for_cutover


## Returns last flushed report snapshot.
func get_report_snapshot() -> Dictionary:
	return _last_payload.duplicate(true)


func _flush_report(current_tick: int) -> void:
	var avg_tick_delta: float = 0.0
	var avg_work_delta: float = 0.0
	var mismatch_ratio: float = 0.0
	var frames_ready: bool = false
	var approved_for_cutover: bool = false
	if _frames > 0:
		avg_tick_delta = float(_sum_tick_delta) / float(_frames)
		avg_work_delta = float(_sum_work_delta) / float(_frames)
		mismatch_ratio = float(_mismatch_frames) / float(_frames)
		frames_ready = _frames >= _min_frames_for_cutover
		approved_for_cutover = (
			frames_ready
			and
			_max_tick_delta <= _allowed_max_tick_delta
			and _max_work_delta <= _allowed_max_work_delta
			and mismatch_ratio <= _allowed_mismatch_ratio
		)
	_last_approved_for_cutover = approved_for_cutover

	var payload: Dictionary = {
		"current_tick": current_tick,
		"frames": _frames,
		"mismatch_frames": _mismatch_frames,
		"max_tick_delta": _max_tick_delta,
		"avg_tick_delta": avg_tick_delta,
		"max_work_delta": _max_work_delta,
		"avg_work_delta": avg_work_delta,
		# Backward compatible aliases for older readers.
		"max_event_delta": _max_work_delta,
		"avg_event_delta": avg_work_delta,
		"mismatch_ratio": mismatch_ratio,
		"allowed_max_tick_delta": _allowed_max_tick_delta,
		"allowed_max_work_delta": _allowed_max_work_delta,
		"allowed_max_event_delta": _allowed_max_work_delta,
		"allowed_mismatch_ratio": _allowed_mismatch_ratio,
		"min_frames_for_cutover": _min_frames_for_cutover,
		"frames_ready_for_cutover": frames_ready,
		"approved_for_cutover": approved_for_cutover,
		"generated_at_unix_ms": Time.get_ticks_msec(),
	}
	_last_payload = payload.duplicate(true)

	var abs_path: String = ProjectSettings.globalize_path(_report_path)
	var dir_path: String = abs_path.get_base_dir()
	DirAccess.make_dir_recursive_absolute(dir_path)
	var file: FileAccess = FileAccess.open(abs_path, FileAccess.WRITE)
	if file == null:
		return
	file.store_string(JSON.stringify(payload, "  "))
