# Pure current-action counts from read-only agent/activity snapshots. No bridge calls.
extends RefCounted

const ACTIVITIES: Array[String] = [
	"UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR",
	"UI_ACTIVITY_HEARTH_CIRCLE", "UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE",
]
# Native JobKind::code(), job.rs:210; zero means no job. These are task purposes, not phases.
const JOB_PURPOSES: Array[String] = ["UI_UNKNOWN", "LOOP_PURPOSE_CHOP", "LOOP_PURPOSE_HAUL", "LOOP_PURPOSE_DELIVER", "LOOP_PURPOSE_BUILD", "LOOP_PURPOSE_QUARRY", "LOOP_PURPOSE_SUPPLY", "LOOP_PURPOSE_CRAFT", "LOOP_PURPOSE_GATHER", "LOOP_PURPOSE_DIG", "LOOP_PURPOSE_DECONSTRUCT", "LOOP_PURPOSE_CLEAR", "LOOP_PURPOSE_DIG_CLAY", "LOOP_PURPOSE_TEACH", "LOOP_PURPOSE_LEARN", "LOOP_PURPOSE_HUNT", "LOOP_PURPOSE_EQUIP", "LOOP_PURPOSE_TRADE", "LOOP_PURPOSE_HARVEST", "C2_JOB_TILL_FIELD", "C2_JOB_SOW_FIELD", "C2_JOB_TEND_FIELD", "C2_JOB_HARVEST_FIELD", "C2_JOB_CAPTURE", "C2_JOB_LIVESTOCK_PRODUCT", "C2_JOB_SLAUGHTER"]

const ORDER: Array[String] = [
	"MIND_BEHAVIOUR_FLEE", "MIND_BEHAVIOUR_EAT", "MIND_BEHAVIOUR_DRINK", "MIND_BEHAVIOUR_SLEEP",
	"MIND_BEHAVIOUR_DO_JOB", "MIND_BEHAVIOUR_FIGHT_DRAFTED", "INSPECTOR_STATE_SEEKING",
	"UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR", "UI_ACTIVITY_HEARTH_CIRCLE",
	"UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE", "UI_ACTION_IDLE", "UI_UNKNOWN",
]

static func _array(value: Variant) -> bool:
	return typeof(value) in [TYPE_ARRAY, TYPE_PACKED_BYTE_ARRAY, TYPE_PACKED_INT32_ARRAY, TYPE_PACKED_INT64_ARRAY]

static func _integer_at(column: Variant, index: int) -> bool:
	return _array(column) and index < column.size() and typeof(column[index]) == TYPE_INT

static func summarize(people: Dictionary, activities: Dictionary, tick: int) -> Dictionary:
	var ids: Variant = people.get("ids")
	if tick < 0 or not _array(ids):
		return {"available": false, "total": 0, "counts": {}}
	var seen: Dictionary = {}
	for id: Variant in ids:
		if typeof(id) != TYPE_INT or seen.has(id):
			return {"available": false, "total": 0, "counts": {}}
		seen[id] = true
	var activity_rows: Dictionary = {}
	var activity_ids: Variant = activities.get("ids", [])
	if _array(activity_ids):
		for i: int in activity_ids.size():
			if _integer_at(activity_ids, i):
				var id: int = int(activity_ids[i])
				activity_rows[id] = -1 if activity_rows.has(id) else i
	var counts: Dictionary = {}
	for i: int in ids.size():
		var key: String = _category(people, i, activities, int(activity_rows.get(ids[i], -1)), tick)
		counts[key] = int(counts.get(key, 0)) + 1
	return {"available": true, "total": ids.size(), "counts": counts}

static func _category(people: Dictionary, i: int, activities: Dictionary, row: int, tick: int) -> String:
	var fleeing: Variant = people.get("fleeing")
	if not _integer_at(fleeing, i) or int(fleeing[i]) not in [0, 1]:
		return "UI_UNKNOWN"
	if int(fleeing[i]) == 1:
		return "MIND_BEHAVIOUR_FLEE"
	var states: Variant = people.get("states")
	var targets: Variant = people.get("target_kinds")
	if not _integer_at(states, i) or not _integer_at(targets, i):
		return "UI_UNKNOWN"
	var state: int = int(states[i])
	var target: int = int(targets[i])
	if state == 0 and target == 0:
		return "UI_ACTION_IDLE"
	if state == 1:
		match target:
			1: return "LOOP_SEEK_FOOD"
			2: return "LOOP_SEEK_WATER"
			3: return "LOOP_SEEK_REST"
			4: return "SEEK:LOOP_PURPOSE_BUILD"
			6: return "LOOP_SEEK_GATHER_FOOD"
			7: return "SEEK:LOOP_PURPOSE_GATHER"
			9: return _job_purpose(people, i, true)
	if state == 2 and target == 5:
		return "UI_UNKNOWN" # Consuming Agent includes combat as well as social interaction.
	if state == 3:
		match target:
			1: return "MIND_BEHAVIOUR_EAT"
			2: return "MIND_BEHAVIOUR_DRINK"
			3: return "MIND_BEHAVIOUR_SLEEP"
			4: return "WORK:LOOP_PURPOSE_BUILD"
			6: return "MIND_BEHAVIOUR_GATHER"
			7: return "WORK:LOOP_PURPOSE_GATHER"
			9: return _job_purpose(people, i, false)
	if state == 4 and target == 8 and row >= 0:
		var kinds: Variant = activities.get("kinds")
		var until: Variant = activities.get("until_tick")
		if _integer_at(kinds, row) and _integer_at(until, row):
			var kind: int = int(kinds[row])
			var deadline: int = int(until[row])
			if kind >= 0 and kind < ACTIVITIES.size():
				if deadline == -1:
					return "INSPECTOR_STATE_SEEKING"
				if deadline > tick:
					return ACTIVITIES[kind]
	return "UI_UNKNOWN"

static func _job_purpose(people: Dictionary, i: int, seeking: bool) -> String:
	var drafted: Variant = people.get("drafted")
	if not _integer_at(drafted, i) or int(drafted[i]) not in [0, 1]:
		return "UI_UNKNOWN"
	if int(drafted[i]) == 1:
		return "MIND_BEHAVIOUR_FIGHT_DRAFTED"
	var jobs: Variant = people.get("jobs")
	if not _integer_at(jobs, i):
		return "UI_UNKNOWN"
	var job: int = int(jobs[i])
	if job <= 0 or job >= JOB_PURPOSES.size():
		return "UI_UNKNOWN"
	# A learner can wait for the teacher in Consuming Job; don't claim learning progress.
	if job == 14 and not seeking:
		return "LOOP_PURPOSE_LEARN_PARTICIPATION"
	return ("SEEK:" if seeking else "WORK:") + JOB_PURPOSES[job]

static func _label(key: String, translate: Callable) -> String:
	if key.begins_with("SEEK:") or key.begins_with("WORK:"):
		var wrapper: String = "LOOP_PURPOSE_PREPARING_FMT" if key.begins_with("SEEK:") else "LOOP_PURPOSE_TASK_FMT"
		return str(translate.call(wrapper)).format({"purpose": translate.call(key.substr(5))})
	return str(translate.call(key))

static func _rank(key: String) -> int:
	var existing: int = ORDER.find(key)
	if existing >= 0:
		return existing
	var purpose: int = JOB_PURPOSES.find(key.substr(5)) if key.begins_with("SEEK:") or key.begins_with("WORK:") else -1
	return ORDER.size() + purpose * 2 + (1 if key.begins_with("SEEK:") else 0) if purpose >= 0 else ORDER.size() + JOB_PURPOSES.size() * 2

static func render(summary: Dictionary, translate: Callable) -> Dictionary:
	if not bool(summary.get("available", false)):
		return {"text": str(translate.call("LOOP_PLAQUE_ACTIVITY_UNAVAILABLE")), "tooltip": ""}
	if int(summary["total"]) == 0:
		return {"text": str(translate.call("LOOP_PLAQUE_RESIDENTS_EMPTY")), "tooltip": ""}
	var counts: Dictionary = summary["counts"]
	var keys: Array = counts.keys()
	keys.sort_custom(func(a: String, b: String) -> bool:
		return int(counts[a]) > int(counts[b]) if counts[a] != counts[b] else (_rank(a) < _rank(b) if _rank(a) != _rank(b) else a.naturalnocasecmp_to(b) < 0))
	var shown: PackedStringArray = []
	var full: PackedStringArray = []
	var remaining: int = 0
	for i: int in keys.size():
		var key: String = str(keys[i])
		var part: String = str(translate.call("LOOP_PLAQUE_ACTIVITY_COUNT_FMT")).format({"activity": _label(key, translate), "count": counts[key]})
		full.append(part)
		if i < 3:
			shown.append(part)
		else:
			remaining += int(counts[key])
	if remaining > 0:
		shown.append(str(translate.call("LOOP_PLAQUE_ACTIVITY_OTHER_FMT")).format({"count": remaining}))
	var prefix: String = str(translate.call("LOOP_PLAQUE_RESIDENTS_NOW_FMT"))
	return {"text": prefix.format({"activities": " · ".join(shown)}), "tooltip": prefix.format({"activities": " · ".join(full)})}
