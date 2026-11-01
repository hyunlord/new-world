# Pure current-action counts from read-only agent/activity snapshots. No bridge calls.
extends RefCounted

const ACTIVITIES: Array[String] = [
	"UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR",
	"UI_ACTIVITY_HEARTH_CIRCLE", "UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE",
]
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
	if state == 1 and target >= 1 and target <= 9 and target != 8:
		return "INSPECTOR_STATE_SEEKING"
	if state == 2 and target == 5:
		return "UI_UNKNOWN" # Consuming Agent includes combat as well as social interaction.
	if state == 3:
		match target:
			1: return "MIND_BEHAVIOUR_EAT"
			2: return "MIND_BEHAVIOUR_DRINK"
			3: return "MIND_BEHAVIOUR_SLEEP"
			4, 6, 7: return "MIND_BEHAVIOUR_DO_JOB"
			9:
				var drafted: Variant = people.get("drafted")
				if not _integer_at(drafted, i) or int(drafted[i]) not in [0, 1]:
					return "UI_UNKNOWN"
				return "MIND_BEHAVIOUR_FIGHT_DRAFTED" if int(drafted[i]) == 1 else "MIND_BEHAVIOUR_DO_JOB"
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

static func render(summary: Dictionary, translate: Callable) -> Dictionary:
	if not bool(summary.get("available", false)):
		return {"text": str(translate.call("LOOP_PLAQUE_ACTIVITY_UNAVAILABLE")), "tooltip": ""}
	if int(summary["total"]) == 0:
		return {"text": str(translate.call("LOOP_PLAQUE_RESIDENTS_EMPTY")), "tooltip": ""}
	var counts: Dictionary = summary["counts"]
	var keys: Array = counts.keys()
	keys.sort_custom(func(a: String, b: String) -> bool:
		return int(counts[a]) > int(counts[b]) if counts[a] != counts[b] else ORDER.find(a) < ORDER.find(b))
	var shown: PackedStringArray = []
	var full: PackedStringArray = []
	var remaining: int = 0
	for i: int in keys.size():
		var key: String = str(keys[i])
		var part: String = str(translate.call("LOOP_PLAQUE_ACTIVITY_COUNT_FMT")).format({"activity": translate.call(key), "count": counts[key]})
		full.append(part)
		if i < 3:
			shown.append(part)
		else:
			remaining += int(counts[key])
	if remaining > 0:
		shown.append(str(translate.call("LOOP_PLAQUE_ACTIVITY_OTHER_FMT")).format({"count": remaining}))
	var prefix: String = str(translate.call("LOOP_PLAQUE_RESIDENTS_NOW_FMT"))
	return {"text": prefix.format({"activities": " · ".join(shown)}), "tooltip": prefix.format({"activities": " · ".join(full)})}
