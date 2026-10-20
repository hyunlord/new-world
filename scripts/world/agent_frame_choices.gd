# agent_frame_choices.gd: cached equivalent of AgentActivity.choose_frame/daily_pose used by agent_renderer.gd
#   when the allocation cache is on; configure() rebuilds it from the frame groups.
# Bridge methods: none.
extends RefCounted
## Prepared atlas choices; animation time and returned metadata remain per-frame.
const Activity = preload("res://scripts/world/agent_activity.gd")
var _groups: Dictionary = {}
var _choices: Dictionary = {}
var _wood_icon: String = ""
var _daily_choices: Dictionary = {}
const DAILY_BODY_KEYS: Dictionary = {"adult_a": 0, "adult_b": 1, "child": 2}
const DAILY_VIEW_KEYS: Dictionary = {"E": 0, "W": 1, "N": 2, "S": 3}

func configure(groups: Dictionary, entries: Dictionary) -> void:
	_groups = groups
	_wood_icon = Activity.wood_icon(entries)
	_choices.clear()
	_daily_choices.clear()

func choose(body: String, activity: String, facing: String, seconds: float, phase: float) -> Dictionary:
	if not _choices.has(body):
		_choices[body] = {}
	var activities: Dictionary = _choices[body]
	if not activities.has(activity):
		activities[activity] = {}
	var views: Dictionary = activities[activity]
	# West uses east frames; the renderer still sets flip_h from original facing.
	var view: String = "E" if facing == "W" else facing
	if not views.has(view):
		views[view] = _prepare(_groups, body, activity, view)
	var frames: Array = views[view]
	if frames.is_empty():
		return {}
	# Preserve choose_frame's fresh shallow dictionary contract for every caller.
	return frames[Activity.frame_index(activity, seconds, phase, frames.size())].duplicate()

func _prepare(groups: Dictionary, body: String, activity: String, facing: String) -> Array[Dictionary]:
	var view: String = "E" if facing == "W" else facing
	var keys: Array[String] = [body + "/" + activity + "/" + view, body + "/" + activity + "/S", body + "/idle/" + view]
	if activity == "crossed":
		keys = [body + "/crossed/" + view, body + "/crossed/S", body + "/sit/" + view, body + "/idle/" + view]
	var wood: bool = activity in ["carry_wood", "gather_wood"]
	if wood:
		var fallback: String = "walk" if activity == "carry_wood" else "crouch"
		keys = [body + "/" + activity + "/" + view, body + "/" + activity + "/S", body + "/" + activity + "/N", body + "/" + activity + "/E", body + "/" + fallback + "/" + view, body + "/" + fallback + "/S", body + "/" + fallback + "/E", body + "/" + fallback + "/N", body + "/" + fallback + "/W", body + "/idle/" + view, body + "/idle/S"]
	var real_carry: bool = false
	if activity == "carry":
		for carry_view: String in ["S","E","N","W"]:
			for entry: Dictionary in groups.get(body+"/carry/"+carry_view,[]):
				if Activity._real_carry(entry,body) and str(entry.get("qa_status", "")) == "accepted":
					real_carry = true
		if real_carry:
			keys = [body+"/carry/"+view,body+"/carry/S",body+"/carry/E",body+"/carry/N",body+"/carry/W"]
	for key: String in keys:
		var safe: Array[Dictionary] = []
		for entry: Dictionary in groups.get(key, []):
			if str(entry.get("body", "")) != body:
				continue
			if activity == "crossed" and bool(entry.get("cold_audit_rejected", false)):
				continue
			if real_carry and not Activity._real_carry(entry,body):
				continue
			var basket: bool = bool(entry.get("contains_basket", false)) or str(entry.get("source_sprite", "")) == "direction_09_villager_gatherer"
			if wood and (bool(entry.get("contains_bush", false)) or str(entry.get("activity", "")) in ["gather", "carry"]):
				continue
			if basket and activity != "carry":
				continue
			safe.append(entry)
		if safe.is_empty():
			continue
		var prepared: Array[Dictionary] = []
		for entry: Dictionary in safe:
			var chosen: Dictionary = entry.duplicate()
			var idle_fallback: bool = str(chosen.get("activity", "")) == "idle" and activity != "idle"
			if wood and str(chosen.get("activity", "")) != activity:
				chosen["fallback_kind"] = "wood_icon"
				chosen["activity_icon"] = _wood_icon
			elif idle_fallback:
				chosen["fallback_kind"] = "idle_icon"
				chosen["activity_icon"] = _wood_icon if wood else Activity.activity_icon(activity)
			elif str(chosen.get("fallback_kind", "")) != "idle_icon":
				chosen["activity_icon"] = ""
			prepared.append(chosen)
		return prepared
	return []

## Results are borrowed and read-only by caller discipline; configure invalidates them.
func daily_pose(body: String, kind: int, facing: String, seconds: float, arrived: bool) -> Dictionary:
	var phase: int = posmod(floori(seconds * 2.0), 2) if kind == 5 and arrived else 0
	var key: int = int(DAILY_BODY_KEYS[body]) * 112 + kind * 16 + int(DAILY_VIEW_KEYS[facing]) * 4 + int(arrived) * 2 + phase
	if not _daily_choices.has(key):
		_daily_choices[key] = Activity.daily_pose(_groups, body, kind, facing, float(phase) * 0.5, arrived)
	return _daily_choices[key]

## choose() owns a fresh shallow dictionary; decorate it without a second copy.
func with_daily_icon(choice: Dictionary, kind: int) -> Dictionary:
	if not choice.is_empty() and kind >= 0 and kind <= 6:
		choice["fallback_kind"] = "activity_icon"
		choice["activity_icon"] = Activity.DailyIcons.icon_id(kind)
	return choice
