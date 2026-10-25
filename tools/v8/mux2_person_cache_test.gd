# Frozen full-update reference: person state, frame resources, anchors, uniforms and read cadence.
# Bridge methods: none executed; synthetic getter nodes satisfy the ordinary SimReads path.
extends SceneTree

const Agents = preload("res://scripts/world/colony/agent_layer.gd")
const Art = preload("res://scripts/world/art/art_data.gd")
var _checks: int = 0
var _failures: int = 0

class CountedAgents:
	extends "res://scripts/world/colony/agent_layer.gd"
	var mask_calls: int = 0

	func _mask(frames: String, facing: String, pose: String) -> Texture2D:
		mask_calls += 1
		return super._mask(frames, facing, pose)

class CountedReference:
	extends "res://tools/v8/fixtures/mux2_person_cache_reference.gd"
	var mask_calls: int = 0

	func _mask(frames: String, facing: String, pose: String) -> Texture2D:
		mask_calls += 1
		return super._mask(frames, facing, pose)

class SnapshotWorld:
	extends Node
	var snapshot: Dictionary = {}
	var reads: Dictionary = {}

	func _record(key: String) -> void:
		reads[key] = int(reads.get(key, 0)) + 1

	func get_agent_snapshot() -> Dictionary:
		_record("agents")
		return snapshot

	func get_world_info() -> Dictionary:
		_record("world")
		return {"move_interval_ticks": 3}

	func get_society_snapshot() -> Dictionary:
		_record("society")
		return {}

	func get_settlement_snapshot() -> Dictionary:
		_record("settlements")
		return {}

class TestCamera:
	extends Camera2D
	var speed: float = 1.0

	func get_speed() -> float:
		return speed

class Inspector:
	extends Control
	var selected: int = -1

	func get_selected_entity_bits() -> int:
		return selected


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_PERSON_CACHE_%s %s" % ["PASS" if valid else "FAIL", label])


func _snapshot(count: int) -> Dictionary:
	var result: Dictionary = {"ids": PackedInt64Array(), "agent_ids": PackedInt64Array(), "xs": PackedInt32Array(), "ys": PackedInt32Array(), "look_ids": PackedInt32Array(), "carry_counts": PackedInt32Array()}
	for key: String in ["life_stages", "states", "target_kinds", "drafted", "sick", "mood_bands", "fleeing", "variants"]:
		result[key] = PackedByteArray()
	for i: int in count:
		result["ids"].append(i + 1)
		result["agent_ids"].append(10000 + i)
		result["xs"].append(10 + i % 20)
		result["ys"].append(10 + floori(float(i) / 20.0))
		for key: String in ["look_ids", "carry_counts", "life_stages", "states", "target_kinds", "drafted", "sick", "mood_bands", "fleeing", "variants"]:
			result[key].append(0)
	return result


func _state(person: Agents.Person) -> Dictionary:
	var result: Dictionary = {}
	for key: String in ["bits", "agent_id", "tile", "from_pos", "to_pos", "elapsed", "duration", "facing", "west", "pose", "activity", "look", "frames", "variant", "elder", "seen", "bed", "drafted", "selected", "sick", "fleeing", "top_colour", "outline", "readable", "marker_position", "position", "modulate", "visible"]:
		result[key] = person.get(key)
	result["texture"] = person.body.texture
	result["anchor"] = person.body.get_meta("anchor", Vector2i(40, 104))
	result["flip"] = person.body.flip_h
	result["offset"] = person.body.offset
	result["body_visible"] = person.body.visible
	result["marks_visible"] = person.marks.visible
	result["shadow_visible"] = person.shadow.visible
	result["body_color"] = person.body.self_modulate
	result["body_z"] = [person.body.z_index, person.body.z_as_relative]
	result["shader"] = person.mat.shader.code
	for key: String in ["skin_delta", "hair_delta", "mask_tex", "has_mask", "outline_color", "outline_px"]:
		result[key] = person.mat.get_shader_parameter(key)
	return result


func _compare(actual: Node2D, expected: Node2D, aw: SnapshotWorld, ew: SnapshotWorld, delta: float, label: String) -> void:
	aw.reads.clear()
	ew.reads.clear()
	actual.call("_update", delta)
	expected.call("_update", delta)
	var ap: Dictionary = actual.get("_persons")
	var ep: Dictionary = expected.get("_persons")
	_check(label + "_complete_population", ap.keys() == ep.keys() and ap.size() == (aw.snapshot["ids"] as PackedInt64Array).size())
	var equal: bool = ap.size() == ep.size()
	for bits: int in ap:
		equal = equal and ep.has(bits) and _state(ap[bits]) == _state(ep[bits])
	_check(label + "_all_person_state_resources_uniforms", equal)
	_check(label + "_snapshot_frequency", aw.reads == {"agents": 1, "world": 1, "society": 1, "settlements": 1} and ew.reads == aw.reads)
	_check(label + "_clock_lod", actual.get("_clock") == expected.get("_clock") and actual.get("_lod") == expected.get("_lod"))
	_check(label + "_badges", actual.get("_coincident_groups") == expected.get("_coincident_groups") and actual.get("_coincident_badges") == expected.get("_coincident_badges"))
	var al: Node2D = actual.get("_badge_leaders")
	var el: Node2D = expected.get("_badge_leaders")
	_check(label + "_leader_segments", al.get("segments") == el.get("segments"))
	var ashadow: ImageTexture = actual.get("_shadow_tex")
	var eshadow: ImageTexture = expected.get("_shadow_tex")
	_check(label + "_reloaded_shadow_pixels", ashadow.get_image().get_data() == eshadow.get_image().get_data())


func _run() -> void:
	var main: Node = Node.new()
	main.name = "Main"
	root.add_child(main)
	var camera: TestCamera = TestCamera.new()
	camera.name = "Camera2D"
	main.add_child(camera)
	camera.make_current()
	var ui: Control = Control.new()
	ui.name = "UI"
	main.add_child(ui)
	var inspector: Inspector = Inspector.new()
	inspector.name = "AgentInspectorPanel"
	ui.add_child(inspector)
	var actual: Node2D = CountedAgents.new()
	var expected: Node2D = CountedReference.new()
	root.add_child(actual)
	root.add_child(expected)
	actual.set_process(false)
	expected.set_process(false)
	var aw: SnapshotWorld = SnapshotWorld.new()
	var ew: SnapshotWorld = SnapshotWorld.new()
	root.add_child(aw)
	root.add_child(ew)
	actual.set("_world_sim", aw)
	expected.set("_world_sim", ew)
	var snapshot: Dictionary = _snapshot(12)
	aw.snapshot = snapshot
	ew.snapshot = snapshot
	_compare(actual, expected, aw, ew, 0.0, "initial_shared_idle")
	if OS.get_cmdline_user_args().has("--expect-cache"):
		_check("shared_initial_pose_single_mask_lookup", int(actual.get("mask_calls")) == 1 and int(expected.get("mask_calls")) == 12)
	_compare(actual, expected, aw, ew, 0.0, "unchanged_idle")
	for phase: float in [0.124999, 0.125, 0.249999, 0.25]:
		for layer: Node2D in [actual, expected]:
			layer.set("_clock", phase)
			for person: Agents.Person in (layer.get("_persons") as Dictionary).values():
				person.elapsed = 0.0
				person.duration = 0.25
				person.facing = "E"
				person.west = true
		_compare(actual, expected, aw, ew, 0.0, "walk_boundary_%s" % phase)
		var walker: Agents.Person = (actual.get("_persons") as Dictionary)[1]
		_check("walk_boundary_reached_%s" % phase, walker.pose == ("E/walk_a" if phase >= 0.125 and phase < 0.25 else "E/idle") and walker.body.flip_h)
	snapshot["xs"][0] += 1
	_compare(actual, expected, aw, ew, 0.1, "ordinary_tile_motion_start")
	_compare(actual, expected, aw, ew, 0.2, "ordinary_tile_motion_finish")
	for i: int in 12:
		snapshot["states"][i] = 3
		snapshot["target_kinds"][i] = 3 if i < 4 else 9
		snapshot["carry_counts"][i] = 1 if i >= 8 else 0
		snapshot["fleeing"][i] = 1
		snapshot["sick"][i] = i % 2
		snapshot["drafted"][i] = i % 2
		snapshot["mood_bands"][i] = i % 3
	for layer: Node2D in [actual, expected]:
		layer.set("_mood_shown", true)
		for person: Agents.Person in (layer.get("_persons") as Dictionary).values():
			person.elapsed = person.duration
	inspector.selected = 2
	for phase: float in [0.249999, 0.25, 0.499999, 0.5]:
		actual.set("_clock", phase)
		expected.set("_clock", phase)
		_compare(actual, expected, aw, ew, 0.0, "work_sleep_carry_boundary_%s" % phase)
		var people: Dictionary = actual.get("_persons")
		_check("state_priorities_reached_%s" % phase, (people[1] as Agents.Person).pose == "E/sleep" and (people[5] as Agents.Person).pose == ("E/work_a" if phase >= 0.25 and phase < 0.5 else "E/idle") and (people[9] as Agents.Person).pose == "E/carry")
	for zoom: float in [0.5, 0.25, 1.0, 0.5]:
		camera.zoom = Vector2(zoom, zoom)
		_compare(actual, expected, aw, ew, 0.02, "lod_transition_%s" % zoom)
		_check("lod_threshold_reached_%s" % zoom, bool(actual.get("_lod")) == (zoom <= 0.25))
	for i: int in 12:
		snapshot["look_ids"][i] = 1 if i % 2 == 0 else 0
		snapshot["variants"][i] = i % 6
		snapshot["life_stages"][i] = 2
	_compare(actual, expected, aw, ew, 0.02, "appearance_variant_elder")
	for facing: String in ["E", "W", "S", "N"]:
		actual.call("face_for_test", 1, facing)
		expected.call("face_for_test", 1, facing)
		_check("direct_face_%s" % facing, _state((actual.get("_persons") as Dictionary)[1]) == _state((expected.get("_persons") as Dictionary)[1]))
	# A missing imported frame exercises null textures/masks and width80 on direct calls.
	for layer: Node2D in [actual, expected]:
		var person: Agents.Person = (layer.get("_persons") as Dictionary)[1]
		person.frames = "__mux2_missing_frame__"
		person.facing = "E"
		person.west = true
		person.pose = ""
		layer.call("_set_frame", person, "idle", 1)
	var missing: Agents.Person = (actual.get("_persons") as Dictionary)[1]
	_check("missing_frame_null_fallback", missing.body.texture == null and missing.body.offset == Vector2(-80, 0) and missing.body.get_meta("anchor") == Vector2i(0, -1) and not bool(missing.mat.get_shader_parameter("has_mask")) and _state(missing) == _state((expected.get("_persons") as Dictionary)[1]))
	# Use the actual F12 loader, restoring its path and document afterwards.
	var saved_path: String = Art.params_path
	var params: Dictionary = Art.params().duplicate(true)
	var path: String = "user://mux2-person-cache-%d.json" % Time.get_ticks_usec()
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	_check("reload_fixture_file", file != null)
	if file != null:
		params["people"]["walk_hz"] = 3.0
		params["people"]["work_hz"] = 5.0
		params["people"]["bob_px"] = 2
		params["people"]["half_overlap_alpha"] = 0.65
		params["people"]["outline_px"] = 2
		file.store_string(JSON.stringify(params))
		file.close()
		Art.params_path = path
		_check("f12_reload_success", Art.reload_params())
		_compare(actual, expected, aw, ew, 0.03, "f12_changed_params")
		_compare(actual, expected, aw, ew, 0.0, "f12_unchanged_next_frame")
		Art.params_path = saved_path
		_check("f12_restore_success", Art.reload_params())
		DirAccess.remove_absolute(path)
	# Recreate ordinary frames after forced missing frame; population increase exercises every row.
	aw.snapshot = _snapshot(1000)
	ew.snapshot = aw.snapshot
	camera.zoom = Vector2.ONE
	_compare(actual, expected, aw, ew, 0.02, "all_1000_rows")
	for key: String in ["look_ids", "variants", "life_stages", "carry_counts", "states", "target_kinds", "drafted", "sick", "mood_bands", "fleeing"]:
		aw.snapshot.erase(key)
	_compare(actual, expected, aw, ew, 0.02, "all_1000_optional_columns_missing")
	aw.snapshot = _snapshot(3)
	ew.snapshot = aw.snapshot
	_compare(actual, expected, aw, ew, 0.02, "removal")
	actual.call("_on_world_reset")
	expected.call("_on_world_reset")
	_compare(actual, expected, aw, ew, 0.02, "reset_recreation")
	actual.free()
	expected.free()
	aw.free()
	ew.free()
	main.free()
	print("MUX2_PERSON_CACHE_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
