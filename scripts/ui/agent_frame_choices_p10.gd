extends RefCounted
## Prepared atlas choices; animation time and returned metadata remain per-frame.
const Activity = preload("res://scripts/ui/agent_activity.gd")
var _groups: Dictionary = {}
var _choices: Dictionary = {}
var _wood_icon: String = ""

func configure(groups: Dictionary, entries: Dictionary) -> void:
	_groups = groups
	_wood_icon = Activity.wood_icon(entries)
	_choices.clear()

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
