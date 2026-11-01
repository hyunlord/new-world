# Current-action classifier, paused locale/scale rendering and observer identity regression.
# Bridge methods: set_sim_speed, advance_ticks (setup and bounded transition probe), get_time_snapshot,
# get_agent_snapshot, get_activity_snapshot, get_command_log, save_world; cmd_draft via SimReads
# only in the explicit commanded block AFTER passive identity and tick-transition checks.
# Screenshots and raw save bytes go to WORLDSIM_TEST_SETTINGS_DIR; commanded evidence is separate.
extends SceneTree
const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
# Independent expected stable-code table for the native JobKind ABI.
const PURPOSE_KEYS: Array[String] = ["UI_UNKNOWN", "LOOP_PURPOSE_CHOP", "LOOP_PURPOSE_HAUL", "LOOP_PURPOSE_DELIVER", "LOOP_PURPOSE_BUILD", "LOOP_PURPOSE_QUARRY", "LOOP_PURPOSE_SUPPLY", "LOOP_PURPOSE_CRAFT", "LOOP_PURPOSE_GATHER", "LOOP_PURPOSE_DIG", "LOOP_PURPOSE_DECONSTRUCT", "LOOP_PURPOSE_CLEAR", "LOOP_PURPOSE_DIG_CLAY", "LOOP_PURPOSE_TEACH", "LOOP_PURPOSE_LEARN", "LOOP_PURPOSE_HUNT", "LOOP_PURPOSE_EQUIP", "LOOP_PURPOSE_TRADE", "LOOP_PURPOSE_HARVEST"]
var failures: int = 0
var activity: Script

func _init() -> void:
	call_deferred("_run")

func check(tag: String, ok: bool) -> void:
	print("CURRENT_ACTIVITY %s %s" % ["PASS" if ok else "FAIL", tag])
	if not ok:
		failures += 1

func frames() -> void:
	for _i: int in 12:
		await process_frame

func person(state: int, target: int, fleeing: int = 0) -> Dictionary:
	return {"ids": PackedInt64Array([101]), "agent_ids": PackedInt64Array([7]),
		"states": PackedByteArray([state]), "target_kinds": PackedByteArray([target]),
		"fleeing": PackedByteArray([fleeing]), "drafted": PackedByteArray([0]), "jobs": PackedByteArray([4]), "carried_woods": PackedByteArray([1])}

func expect(tag: String, people: Dictionary, acts: Dictionary, key: String) -> void:
	var before: Dictionary = people.duplicate(true)
	var acts_before: Dictionary = acts.duplicate(true)
	var result: Dictionary = activity.call("summarize", people, acts, 60)
	check(tag, result["available"] and result["counts"] == {key: 1} and result["total"] == 1)
	check(tag + " immutable", people == before and acts == acts_before)

func pure() -> void:
	expect("idle job/carry not working", person(0, 0), {}, "UI_ACTION_IDLE")
	var seeks: Dictionary = {1: "LOOP_SEEK_FOOD", 2: "LOOP_SEEK_WATER", 3: "LOOP_SEEK_REST", 4: "SEEK:LOOP_PURPOSE_BUILD", 5: "UI_UNKNOWN", 6: "LOOP_SEEK_GATHER_FOOD", 7: "SEEK:LOOP_PURPOSE_GATHER", 9: "SEEK:LOOP_PURPOSE_BUILD"}
	for target: int in seeks:
		expect("seeking purpose %d" % target, person(1, target), {}, seeks[target])
	for code: int in range(1, 19):
		var job: Dictionary = person(1, 9)
		job["jobs"] = PackedByteArray([code])
		job["target_xs"] = PackedInt32Array([999]) # Tool/material fetch, not a proven final worksite.
		expect("preparing job %d" % code, job, {}, "SEEK:" + PURPOSE_KEYS[code])
		job["states"] = PackedByteArray([3])
		expect("current job %d" % code, job, {}, "LOOP_PURPOSE_LEARN_PARTICIPATION" if code == 14 else "WORK:" + PURPOSE_KEYS[code])
		job["states"] = PackedByteArray([0])
		job["target_kinds"] = PackedByteArray([0])
		expect("idle retained job %d" % code, job, {}, "UI_ACTION_IDLE")
		job["states"] = PackedByteArray([3])
		job["target_kinds"] = PackedByteArray([1])
		expect("eating retained job %d" % code, job, {}, "MIND_BEHAVIOUR_EAT")
	for state: int in [1, 3]:
		for code: int in [0, 99]:
			var invalid_job: Dictionary = person(state, 9)
			invalid_job["jobs"] = PackedByteArray([code])
			expect("unknown job %d state%d" % [code, state], invalid_job, {}, "UI_UNKNOWN")
		var missing_job: Dictionary = person(state, 9)
		missing_job.erase("jobs")
		expect("missing job state%d" % state, missing_job, {}, "UI_UNKNOWN")
		missing_job["jobs"] = PackedByteArray()
		expect("short job state%d" % state, missing_job, {}, "UI_UNKNOWN")
	for row: Array in [[1, "MIND_BEHAVIOUR_EAT"], [2, "MIND_BEHAVIOUR_DRINK"], [3, "MIND_BEHAVIOUR_SLEEP"], [4, "WORK:LOOP_PURPOSE_BUILD"], [6, "MIND_BEHAVIOUR_GATHER"], [7, "WORK:LOOP_PURPOSE_GATHER"], [9, "WORK:LOOP_PURPOSE_BUILD"]]:
		expect("consuming %d" % row[0], person(3, row[0]), {}, row[1])
	expect("social or combat is ambiguous", person(2, 5), {}, "UI_UNKNOWN")
	var hold: Dictionary = person(3, 9)
	hold["drafted"] = PackedByteArray([1])
	expect("drafted hold is following orders", hold, {}, "MIND_BEHAVIOUR_FIGHT_DRAFTED")
	hold["states"] = PackedByteArray([1])
	expect("drafted seek ignores leftover job", hold, {}, "MIND_BEHAVIOUR_FIGHT_DRAFTED")
	hold["states"] = PackedByteArray([3])
	hold.erase("drafted")
	expect("missing drafted ambiguous job", hold, {}, "UI_UNKNOWN")
	hold["drafted"] = PackedByteArray([2])
	expect("invalid drafted ambiguous job", hold, {}, "UI_UNKNOWN")
	expect("flee priority", person(3, 1, 1), {}, "MIND_BEHAVIOUR_FLEE")
	var labels: Array[String] = ["UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR", "UI_ACTIVITY_HEARTH_CIRCLE", "UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE"]
	for kind: int in labels.size():
		var acts: Dictionary = {"ids": PackedInt64Array([101]), "kinds": PackedByteArray([kind]), "until_tick": PackedInt64Array([61])}
		expect("arrived kind %d" % kind, person(4, 8), acts, labels[kind])
		acts["until_tick"] = PackedInt64Array([-1])
		expect("walking kind %d" % kind, person(4, 8), acts, "INSPECTOR_STATE_SEEKING")
		acts["until_tick"] = PackedInt64Array([60])
		expect("deadline reached kind %d" % kind, person(4, 8), acts, "UI_UNKNOWN")
		acts["until_tick"] = PackedInt64Array([59])
		expect("expired kind %d" % kind, person(4, 8), acts, "UI_UNKNOWN")
	expect("join entity not AgentId", person(4, 8), {"ids": [7], "kinds": [0], "until_tick": [61]}, "UI_UNKNOWN")
	expect("duplicate activity", person(4, 8), {"ids": [101, 101], "kinds": [0, 1], "until_tick": [61, 61]}, "UI_UNKNOWN")
	expect("residual activity ignored", person(0, 0), {"ids": [101], "kinds": [0], "until_tick": [61]}, "UI_ACTION_IDLE")
	expect("unsupported activity", person(4, 8), {"ids": [101], "kinds": [90], "until_tick": [61]}, "UI_UNKNOWN")
	expect("unsupported state", person(99, 0), {}, "UI_UNKNOWN")
	expect("invalid consuming target", person(3, 0), {}, "UI_UNKNOWN")
	var short_row: Dictionary = person(3, 1)
	short_row["target_kinds"] = PackedByteArray()
	expect("short columns", short_row, {}, "UI_UNKNOWN")
	short_row = person(3, 1)
	short_row.erase("fleeing")
	expect("missing flee column", short_row, {}, "UI_UNKNOWN")
	check("unavailable distinct from empty", not activity.call("summarize", {}, {}, 60)["available"] and activity.call("summarize", {"ids": []}, {}, 60)["available"])
	check("duplicate people invalid", not activity.call("summarize", {"ids": [101, 101]}, {}, 60)["available"])
	var counts: Dictionary = {"UI_ACTION_IDLE": 1, "MIND_BEHAVIOUR_EAT": 4, "MIND_BEHAVIOUR_DRINK": 3, "MIND_BEHAVIOUR_SLEEP": 2}
	var locale: Node = root.get_node("Locale")
	var rendered: Dictionary = activity.call("render", {"available": true, "total": 10, "counts": counts}, Callable(locale, "ltr"))
	check("top three truthful remainder", str(rendered["text"]).contains(str(locale.call("ltr", "LOOP_PLAQUE_ACTIVITY_OTHER_FMT")).format({"count": 1})))
	check("full tooltip contains omitted action", str(rendered["tooltip"]).contains(str(locale.call("ltr", "UI_ACTION_IDLE"))))

func _run() -> void:
	var directory: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not directory.is_absolute_path() or not DirAccess.dir_exists_absolute(directory) or directory.simplify_path().begins_with(ProjectSettings.globalize_path("res://")):
		push_error("CURRENT_ACTIVITY requires an existing absolute external WORLDSIM_TEST_SETTINGS_DIR")
		quit(1)
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	world.call("advance_ticks", 60)
	await frames()
	var plaque: Control = main.get_node("UI/TaskPlaque")
	check("ambient activity row exists", plaque.has_method("activity_text") and plaque.find_child("CurrentActivity", true, false) != null)
	if not plaque.has_method("activity_text"):
		quit(1)
		return
	activity = load("res://scripts/ui/panels/current_activity_text.gd") as Script
	pure()
	var tick: Dictionary = world.call("get_time_snapshot")
	var log: Dictionary = world.call("get_command_log", 0)
	var people: Dictionary = world.call("get_agent_snapshot")
	var acts: Dictionary = world.call("get_activity_snapshot")
	var expected: Dictionary = activity.call("summarize", people, acts, int(tick["tick"]))
	verify_live_purposes(world, plaque, people, directory)
	var before: String = directory.path_join("before.save")
	check("initial save succeeds", bool(world.call("save_world", before)))
	var hud: Node = main.get_node("UI/MuxHud")
	var store: RefCounted = hud.get("store")
	var cards: Node = hud.get_node("CardPanel")
	var lifecycle: Dictionary = cards.get("_active").duplicate(true)
	var cooldown: Dictionary = cards.get("_cooldown").duplicate(true)
	var locale_ordinal: int = 0
	for language: String in ["ko", "en", "ko"]:
		locale_ordinal += 1
		root.get_node("Locale").call("set_language", language)
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			await frames()
			var tag: String = "%d-%s-%.2f" % [locale_ordinal, language, factor]
			var rendered: Dictionary = activity.call("render", expected, Callable(root.get_node("Locale"), "ltr"))
			check(tag + " real snapshot row", plaque.call("activity_text") == rendered["text"])
			var label: Label = plaque.find_child("CurrentActivity", true, false)
			check(tag + " full counts tooltip", label.tooltip_text == rendered["tooltip"])
			var raw_total: int = 0
			for count: Variant in expected["counts"].values():
				raw_total += int(count)
			check(tag + " count conservation", raw_total == people["ids"].size())
			check(tag + " requested scale", plaque.scale == Vector2.ONE * factor)
			check(tag + " visible label", label.is_visible_in_tree() and root.get_visible_rect().encloses(label.get_global_rect()))
			var panel: Control = plaque.get("_panel")
			check(tag + " label in plaque", panel.get_global_rect().encloses(label.get_global_rect()))
			check(tag + " chapter retained", not str((plaque.get("_chapter_label") as Label).text).is_empty())
			for obstacle: String in ["_actions", "cards", "alerts", "helper"]:
				var control: Control = hud.get(obstacle) as Control
				if control != null and control.is_visible_in_tree():
					check(tag + " clear " + obstacle, not panel.get_global_rect().intersects(control.get_global_rect()))
			check(tag + " card lifecycle stable", cards.get("_active") == lifecycle and cards.get("_cooldown") == cooldown)
			check(tag + " tick stable", world.call("get_time_snapshot") == tick)
			check(tag + " command log stable", world.call("get_command_log", 0) == log)
			await RenderingServer.frame_post_draw
			check(tag + " screenshot", root.get_texture().get_image().save_png(directory.path_join(tag + ".png")) == OK)
	# Explicit synthetic PRESENTATION stress, separate from the live snapshot screenshots above.
	# No snapshot/cache is replaced: only rendered label text is temporarily exercised and restored.
	for language: String in ["ko", "en"]:
		root.get_node("Locale").call("set_language", language)
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			await frames()
			var label: Label = plaque.find_child("CurrentActivity", true, false)
			var original_text: String = label.text
			var fixture: Dictionary = {"available": true, "total": 3995, "counts": {
				"SEEK:LOOP_PURPOSE_SUPPLY": 999, "SEEK:LOOP_PURPOSE_DELIVER": 999,
				"LOOP_PURPOSE_LEARN_PARTICIPATION": 999, "UI_UNKNOWN": 998}}
			var stress: Dictionary = activity.call("render", fixture, Callable(root.get_node("Locale"), "ltr"))
			label.text = KoreanText.words(str(stress["text"]))
			await frames()
			var tag: String = "synthetic-layout-%s-%.2f" % [language, factor]
			var panel: Control = plaque.get("_panel")
			check(tag + " visible and contained", root.get_visible_rect().encloses(panel.get_global_rect()) and panel.get_global_rect().encloses(label.get_global_rect()))
			check(tag + " requested scale", plaque.scale == Vector2.ONE * factor)
			for obstacle: String in ["_actions", "cards", "alerts", "helper"]:
				var control: Control = hud.get(obstacle) as Control
				if control != null and control.is_visible_in_tree():
					check(tag + " clear " + obstacle, not panel.get_global_rect().intersects(control.get_global_rect()))
			await RenderingServer.frame_post_draw
			check(tag + " screenshot", root.get_texture().get_image().save_png(directory.path_join(tag + ".png")) == OK)
			label.text = original_text
			await frames()
	# A normal public foreground open/close must restore the ambient row without a native command.
	var priority: Control = main.get_node("UI/PriorityPanel")
	priority.call("open_roles")
	await frames()
	check("foreground suppresses ambient row", not plaque.is_visible_in_tree())
	priority.hide()
	await frames()
	check("foreground restores ambient row", plaque.is_visible_in_tree())
	check("primary text API retained", not str(plaque.call("plaque_text")).is_empty())
	var after: String = directory.path_join("after.save")
	check("final save succeeds", bool(world.call("save_world", after)))
	var before_bytes: PackedByteArray = FileAccess.get_file_as_bytes(before)
	check("save bytes identical and nonempty", not before_bytes.is_empty() and before_bytes == FileAccess.get_file_as_bytes(after))
	check("raw agent snapshots stable", world.call("get_agent_snapshot") == people and world.call("get_activity_snapshot") == acts)
	await live_transition(world, plaque, directory)
	await paused_draft_command(world, plaque, directory)
	print("CURRENT_ACTIVITY failures=%d" % failures)
	quit(1 if failures else 0)


# Explicitly advances AFTER the paused save-identity block; never claims equal saves across time.
func live_transition(world: Node, plaque: Node, directory: String) -> void:
	var before_log: Dictionary = world.call("get_command_log", 0)
	var previous: Dictionary = world.call("get_agent_snapshot")
	var observed: bool = false
	var evidence: Array[Dictionary] = []
	var start_tick: int = int((world.call("get_time_snapshot") as Dictionary)["tick"])
	for _step: int in 299:
		world.call("advance_ticks", 1)
		await process_frame
		await process_frame
		var current: Dictionary = world.call("get_agent_snapshot")
		for i: int in current["ids"].size():
			var prior: int = previous["ids"].find(current["ids"][i])
			var target: int = int(current["target_kinds"][i])
			if prior < 0 or int(previous["states"][prior]) != 1 or int(current["states"][i]) != 3:
				continue
			if int(previous["target_kinds"][prior]) != target or _expected_current_purpose(current, i).is_empty() or int(current["fleeing"][i]) != 0 or int(current["drafted"][i]) != 0:
				continue
			var key: String = _expected_current_purpose(current, i)
			var expected_count: int = 0
			for j: int in current["ids"].size():
				if int(current["fleeing"][j]) == 0 and int(current["drafted"][j]) == 0 and int(current["states"][j]) == 3 and _expected_current_purpose(current, j) == key:
					expected_count += 1
			var summary: Dictionary = plaque.get("_activity_summary")
			check("live sought to consumed count", int(summary["counts"].get(key, 0)) == expected_count)
			var part: String = str(root.get_node("Locale").call("ltr", "LOOP_PLAQUE_ACTIVITY_COUNT_FMT")).format({"activity": _expected_purpose_label(key), "count": expected_count})
			check("live consumed label in full tooltip", str((plaque.get("_activity_label") as Label).tooltip_text).contains(part))
			evidence.append({"tick": world.call("get_time_snapshot"), "entity": current["ids"][i], "before": previous, "after": current, "label": plaque.call("activity_text"), "tooltip": (plaque.get("_activity_label") as Label).tooltip_text})
			observed = true
			break
		if observed:
			break
		previous = current
	check("real seeking to consuming transition within 299 ticks", observed)
	var end_tick: int = int((world.call("get_time_snapshot") as Dictionary)["tick"])
	check("probe remains before automatic tick360", start_tick == 60 and end_tick < 360)
	check("transition advance submits no command", world.call("get_command_log", 0) == before_log)
	var output: FileAccess = FileAccess.open(directory.path_join("transition.json"), FileAccess.WRITE)
	check("transition evidence writable", output != null)
	if output != null:
		output.store_string(JSON.stringify({"observed": observed, "limit_ticks": 299, "start_tick": start_tick, "end_tick": end_tick, "evidence": evidence}, "\t"))


# Intentionally changes native state through the normal UI route, separate from passive identity.
func paused_draft_command(world: Node, plaque: Node, directory: String) -> void:
	await frames()
	var before_tick: Dictionary = world.call("get_time_snapshot")
	var before_log: Dictionary = world.call("get_command_log", 0)
	var people: Dictionary = world.call("get_agent_snapshot")
	var selected: int = -1
	var selected_entity: int = -1
	for i: int in people["agent_ids"].size():
		if int(people["drafted"][i]) == 0 and int(people["life_stages"][i]) == 0:
			var id: int = int(people["agent_ids"][i])
			if selected < 0 or id < selected:
				selected = id
				selected_entity = int(people["ids"][i])
	check("draft fixture has live undrafted adult", selected >= 0)
	if selected < 0:
		return
	var receipt: Dictionary = SimReads.mutate(world, "cmd_draft", [selected, true])
	check("explicit draft accepted", bool(receipt.get("accepted", false)))
	var immediate_log: Dictionary = world.call("get_command_log", 0)
	var delta: Dictionary = world.call("get_command_log", int(before_log["total"]))
	check("exactly one explicit command receipt", int(immediate_log["total"]) == int(before_log["total"]) + 1 and delta["kinds"] == PackedStringArray(["SetDrafted"]) and delta["accepted"] == PackedByteArray([1]))
	await frames()
	var current: Dictionary = world.call("get_agent_snapshot")
	var index: int = current["ids"].find(selected_entity)
	check("native draft sets current hold", index >= 0 and int(current["drafted"][index]) == 1 and int(current["states"][index]) == 3 and int(current["target_kinds"][index]) == 9)
	var expected_count: int = 0
	for i: int in current["ids"].size():
		if int(current["fleeing"][i]) == 0 and int(current["drafted"][i]) == 1 and int(current["states"][i]) == 3 and int(current["target_kinds"][i]) == 9:
			expected_count += 1
	var summary: Dictionary = plaque.get("_activity_summary")
	check("same-tick current draft count refreshed", int(summary["counts"].get("MIND_BEHAVIOUR_FIGHT_DRAFTED", 0)) == expected_count and expected_count > 0)
	var part: String = str(root.get_node("Locale").call("ltr", "LOOP_PLAQUE_ACTIVITY_COUNT_FMT")).format({"activity": root.get_node("Locale").call("ltr", "MIND_BEHAVIOUR_FIGHT_DRAFTED"), "count": expected_count})
	check("same-tick current draft label", str((plaque.get("_activity_label") as Label).tooltip_text).contains(part))
	check("draft command advances no ticks", world.call("get_time_snapshot") == before_tick)
	check("render adds no commands", world.call("get_command_log", 0) == immediate_log)
	var output: FileAccess = FileAccess.open(directory.path_join("paused-draft-command.json"), FileAccess.WRITE)
	check("draft command evidence writable", output != null)
	if output != null:
		output.store_string(JSON.stringify({"oracle": "INTENTIONAL_COMMAND_NOT_PASSIVE_IDENTITY", "selected_agent": selected, "before_tick": before_tick, "after_tick": world.call("get_time_snapshot"), "before_log": before_log, "after_log": immediate_log, "receipt": receipt, "before_people": people, "after_people": current, "plaque_counts": summary}, "\t"))


func _expected_current_purpose(people: Dictionary, i: int) -> String:
	if int(people["fleeing"][i]) != 0 or int(people["drafted"][i]) != 0:
		return ""
	var state: int = int(people["states"][i])
	var target: int = int(people["target_kinds"][i])
	if state not in [1, 3]:
		return ""
	if target == 9:
		var code: int = int(people["jobs"][i])
		if code <= 0 or code >= PURPOSE_KEYS.size():
			return ""
		if state == 3 and code == 14:
			return "LOOP_PURPOSE_LEARN_PARTICIPATION"
		return ("SEEK:" if state == 1 else "WORK:") + PURPOSE_KEYS[code]
	var working: Dictionary = {1: "MIND_BEHAVIOUR_EAT", 2: "MIND_BEHAVIOUR_DRINK", 3: "MIND_BEHAVIOUR_SLEEP", 4: "WORK:LOOP_PURPOSE_BUILD", 6: "MIND_BEHAVIOUR_GATHER", 7: "WORK:LOOP_PURPOSE_GATHER"}
	var seeking: Dictionary = {1: "LOOP_SEEK_FOOD", 2: "LOOP_SEEK_WATER", 3: "LOOP_SEEK_REST", 4: "SEEK:LOOP_PURPOSE_BUILD", 6: "LOOP_SEEK_GATHER_FOOD", 7: "SEEK:LOOP_PURPOSE_GATHER"}
	return str((seeking if state == 1 else working).get(target, ""))

func _expected_purpose_label(key: String) -> String:
	var locale: Node = root.get_node("Locale")
	if key.begins_with("SEEK:") or key.begins_with("WORK:"):
		var wrapper: String = "LOOP_PURPOSE_PREPARING_FMT" if key.begins_with("SEEK:") else "LOOP_PURPOSE_TASK_FMT"
		return str(locale.call("ltr", wrapper)).format({"purpose": locale.call("ltr", key.substr(5))})
	return str(locale.call("ltr", key))

func verify_live_purposes(world: Node, plaque: Node, people: Dictionary, directory: String) -> void:
	var counts: Dictionary = {}
	var active_jobs: int = 0
	for i: int in people["ids"].size():
		var key: String = _expected_current_purpose(people, i)
		if key.is_empty():
			continue
		counts[key] = int(counts.get(key, 0)) + 1
		if int(people["target_kinds"][i]) == 9:
			active_jobs += 1
	check("native current job fixture populated", active_jobs > 0)
	var summary: Dictionary = plaque.get("_activity_summary")
	var tooltip: String = str((plaque.get("_activity_label") as Label).tooltip_text)
	for key: String in counts:
		check("native purpose count " + key, int(summary["counts"].get(key, 0)) == int(counts[key]))
		var part: String = str(root.get_node("Locale").call("ltr", "LOOP_PLAQUE_ACTIVITY_COUNT_FMT")).format({"activity": _expected_purpose_label(key), "count": counts[key]})
		check("native purpose label " + key, tooltip.contains(part))
	var file: FileAccess = FileAccess.open(directory.path_join("live-purpose.json"), FileAccess.WRITE)
	check("native purpose evidence writable", file != null)
	if file != null:
		file.store_string(JSON.stringify({"tick": world.call("get_time_snapshot"), "people": people, "independent_purpose_counts": counts, "plaque_summary": summary, "visible_text": plaque.call("activity_text"), "tooltip": tooltip}, "\t"))
