extends Node

signal frame_completed(sample: Dictionary)

class Stamp extends Node:
	var action: Callable
	func _process(_delta: float) -> void:
		action.call()

var clock_usec: Callable = Time.get_ticks_usec
var frame_number: Callable = Engine.get_process_frames
var first: Stamp
var last: Stamp
var _started_usec: int = 0
var _started_frame: int = -1
var _sample: Dictionary = {}

func _ready() -> void:
	first = Stamp.new()
	first.name = "ProcessStartMinus1000"
	first.process_priority = -1000
	first.action = begin_stamp
	add_child(first)
	last = Stamp.new()
	last.name = "ProcessEndPlus1000"
	last.process_priority = 1000
	last.action = end_stamp
	add_child(last)

func begin_stamp() -> void:
	_started_frame = int(frame_number.call())
	_started_usec = int(clock_usec.call())

func end_stamp() -> void:
	var frame: int = int(frame_number.call())
	var elapsed: int = int(clock_usec.call()) - _started_usec
	_sample = {"process_frame": frame, "process_ms": float(elapsed) / 1000.0, "paired": _started_frame == frame and elapsed >= 0}

	# Publish after callbacks drain so drawing and cleanup never run inside a stamp.
	_publish_sample.call_deferred(_sample.duplicate())

func _publish_sample(sample: Dictionary) -> void:
	frame_completed.emit(sample)

func readout() -> Dictionary:
	return _sample.duplicate()
