class_name AgentActivity
extends RefCounted

static func resolve(state: int, target_kind: int, has_targets: bool, moving: bool, carried: float, tile: Dictionary, hour: float) -> String:
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
	if hour >= 18.0 and hour < 23.0 and bool(tile.get("near_hearth", false)):
		return "sit"
	return "idle"

static func hidden(activity: String, has_targets: bool, tile: Dictionary) -> bool:
	return activity == "asleep" and has_targets and bool(tile.get("shelter", false))

static func facing(delta: Vector2, previous: String) -> String:
	if delta.length_squared() < 0.000001:
		return previous
	if absf(delta.x) >= absf(delta.y):
		return "W" if delta.x < 0.0 else "E"
	return "N" if delta.y < 0.0 else "S"

static func frame_index(activity: String, seconds: float, phase: float, count: int) -> int:
	var fps: float = 8.0 if activity in ["walk", "carry", "run"] else (1.5 if activity == "idle" else 4.0)
	return posmod(floori((seconds + phase) * fps), maxi(count, 1))
