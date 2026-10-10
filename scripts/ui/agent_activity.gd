class_name AgentActivity
extends RefCounted

static func resolve(state: int, target_kind: int, has_targets: bool, moving: bool, carried: float, tile: Dictionary, _hour: float) -> String:
	if state == 2:
		return "talk"
	if state == 3:
		if has_targets:
			match target_kind:
				1: return "eat"
				2: return "drink"
				3: return "asleep"
				4: return "build"
				5: return "talk"
				6: return "gather"
		if bool(tile.get("shelter", false)):
			return "asleep"
		if bool(tile.get("construction", false)):
			return "build"
		if bool(tile.get("food", false)):
			return "gather"
		if bool(tile.get("water", false)):
			return "drink"
		return "eat"
	if state == 1 or moving:
		return "carry" if carried > 0.0 else "walk"
	return "idle"

static func hidden(activity: String, has_targets: bool, tile: Dictionary) -> bool:
	return activity == "asleep" and has_targets and bool(tile.get("shelter", false))

static func facing(delta: Vector2, previous: String) -> String:
	if delta.length_squared() < 0.000001:
		return previous
	if absf(delta.x) >= absf(delta.y):
		return "W" if delta.x < 0.0 else "E"
	return "N" if delta.y < 0.0 else "S"

static func idle_elapsed(activity: String, movement: Vector2, previous: float, delta: float) -> float:
	return previous + maxf(delta, 0.0) if activity == "idle" and movement.length_squared() < 0.000001 else 0.0

static func render_facing(activity: String, movement: Vector2, partner_delta: Vector2, previous: String, idle_seconds: float) -> String:
	if movement.length_squared() >= 0.000001:
		return facing(movement, previous)
	if activity == "talk":
		return facing(partner_delta, previous)
	return "S" if activity == "idle" and idle_seconds >= 1.0 else previous

static func camp_pose(activity: String, life_stage: int, agent_id: int, day: int, hour: float, near_camp: bool) -> String:
	if activity != "idle" or not near_camp or hour < 6.0 or hour >= 23.0:
		return activity
	var animated: bool = posmod(ArtHash.mix(agent_id + day), 3) != 0
	if life_stage == 1:
		return "hop" if animated else "idle"
	return "sit" if life_stage == 2 or animated else "idle"

static func frame_index(activity: String, seconds: float, phase: float, count: int) -> int:
	var fps: float = 8.0 if activity in ["walk", "carry", "run"] else (1.5 if activity == "idle" else 4.0)
	return posmod(floori((seconds + phase) * fps), maxi(count, 1))

static func elder_look(base: Dictionary, agent_id: int, life_stage: int) -> Dictionary:
	var look: Dictionary = base.duplicate()
	if life_stage == 2:
		look["hair"] = 3 + posmod(ArtHash.mix(agent_id), 2)
	else:
		look["hair"] = posmod(int(look["hair"]), 3)
	return look

static func build_frame_groups(entries: Dictionary) -> Dictionary:
	var groups: Dictionary = {}
	var names: Array = entries.keys()
	names.sort()
	for id: String in names:
		var entry: Dictionary = entries[id]
		if not bool(entry.get("drawable", true)) or str(entry.get("qa_status", "")) not in ["accepted", "derived", "procedural", "fallback"]:
			continue
		var body: String = str(entry.get("body", ""))
		if body not in ["adult_a", "adult_b", "child"]:
			continue
		var key: String = body + "/" + str(entry.get("activity", "")) + "/" + str(entry.get("view", ""))
		if not groups.has(key):
			groups[key] = []
		var frame: Dictionary = entry.duplicate()
		frame["sprite_id"] = id
		groups[key].append(frame)
	for frames: Array in groups.values():
		frames.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a.get("frame", 0)) < int(b.get("frame", 0)))
	return groups

static func activity_icon(activity: String) -> String:
	return str({"walk": "U1_01", "carry": "U1_05", "gather": "U1_06", "build": "U1_04", "eat": "U1_12", "drink": "U1_11", "sit": "U1_13", "asleep": "U1_13", "talk": "U1_01"}.get(activity, ""))

static func choose_frame(groups: Dictionary, body: String, activity: String, facing: String, seconds: float, phase: float) -> Dictionary:
	var view: String = "E" if facing == "W" else facing
	var keys: Array[String] = [body + "/" + activity + "/" + view, body + "/" + activity + "/S", body + "/idle/" + view]
	for key: String in keys:
		var safe: Array[Dictionary] = []
		for entry: Dictionary in groups.get(key, []):
			if str(entry.get("body", "")) != body:
				continue
			var basket: bool = bool(entry.get("contains_basket", false)) or str(entry.get("source_sprite", "")) == "direction_09_villager_gatherer"
			if basket and activity != "carry":
				continue
			safe.append(entry)
		if safe.is_empty():
			continue
		var chosen: Dictionary = safe[frame_index(activity, seconds, phase, safe.size())].duplicate()
		var idle_fallback: bool = str(chosen.get("activity", "")) == "idle" and activity != "idle"
		if idle_fallback:
			chosen["fallback_kind"] = "idle_icon"
			chosen["activity_icon"] = activity_icon(activity)
		elif str(chosen.get("fallback_kind", "")) != "idle_icon":
			chosen["activity_icon"] = ""
		return chosen
	return {}

static func show_activity_icon(choice: Dictionary, zoom: float) -> bool:
	return zoom >= 2.0 and str(choice.get("fallback_kind", "")) == "idle_icon" and not str(choice.get("activity_icon", "")).is_empty()

static func apply_activity_icon(sprite: Sprite2D, icon: Sprite2D, choice: Dictionary, zoom: float) -> bool:
	icon.visible = false
	if not sprite.visible or not show_activity_icon(choice, zoom):
		return false
	icon.visible = ArtSpriteLibrary.apply(icon, str(choice["activity_icon"]))
	icon.position = Vector2(0, sprite.offset.y - 2.0)
	icon.set_meta("art_kind", "activity_icon")
	return icon.visible
