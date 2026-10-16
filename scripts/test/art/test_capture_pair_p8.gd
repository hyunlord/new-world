extends "res://scripts/test/capture_framings.gd"

class FixtureCamera extends Camera2D:
	func set_auto_track(_enabled: bool) -> void: pass

class FixtureWorld extends Node:
	var present: bool = true
	func reset_world(_seed_value: int) -> void: present = false
	func set_sim_speed(_speed: float) -> void: pass
	func get_agent_snapshot() -> Dictionary: return {}
	func get_world_info() -> Dictionary: return {"ticks_per_day": 1440}
	func get_time_snapshot() -> Dictionary: return {"tick": 42000, "day": 30, "hour": 6}
	func get_settlement_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7]) if present else PackedInt32Array(), "member_counts": PackedInt32Array([35]), "food_stocks": PackedInt32Array([120])}
	func get_society_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7]), "fission_pressures": PackedInt64Array([700]), "fission_thresholds": PackedInt64Array([1200])}

func _capture_fixture_mode() -> bool: return true

func _sample_event_truth(_agents: Dictionary) -> void: pass

func _run() -> void:
	var fixture := FixtureWorld.new()
	_world = fixture
	_camera = FixtureCamera.new()
	root.add_child(_camera)
	_original_id = 7
	var row: Dictionary = _original_observation()
	assert(row["found"] and row["food"] == 120 and row["members"] == 35)
	assert(is_equal_approx(row["adult_need_per_day"], 2.4))
	assert(row["scarcity_line"] == 336 and row["below_scarcity_line"])
	assert(row["pressure"] == 700 and row["threshold"] == 1200)
	assert(row["need_provenance"]["source_files"].size() == 3)
	for source: Dictionary in row["need_provenance"]["source_files"]:
		assert(str(source["sha256"]).length() == 64)
	fixture.present = false
	_target_name = "orig"
	_row_options = {"paired_world": "P", "case_id": "a33"}
	_world_label = "P"
	assert(_perform_capture_cast([30, "06:00", "F2", "spoil_food", 60], 42000))
	assert(_casts.size() == 1 and _all_casts.size() == 1)
	assert(not _casts[0]["reply"]["ok"] and _casts[0]["reply"]["applied"] == 0)
	assert(_casts[0]["input_surface"] == "not invoked: missing paired target")
	await _reset_capture_world(20261013)
	assert(_casts.is_empty() and _all_casts.size() == 1)
	_world_label = "G"
	_row_options = {"paired_world": "G", "case_id": "a33"}
	assert(_perform_capture_cast([30, "06:00", "F2", "gift_food", 60], 42000))
	assert(_casts.size() == 1 and _all_casts.size() == 2)
	assert(_all_casts[0]["world_label"] == "P" and _all_casts[1]["world_label"] == "G")
	assert(_all_casts[0]["world_epoch"] != _all_casts[1]["world_epoch"])
	_camera.free()
	fixture.free()
	print("CAPTURE_PAIR_P8 assertions passed: source need, scarcity, pressure, missing-target no-op, cross-epoch cast retention")
	quit(0)
