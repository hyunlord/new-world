class_name AgentActivity
extends RefCounted

const Cold = preload("res://scripts/ui/cold_effects.gd")
const DAILY_NAMES: Array[String] = ["tend_fire", "craft", "repair", "hearth_circle", "explore", "play", "sit_by_fire"]
const DailyIcons = preload("res://scripts/ui/activity_icons_p11.gd")
static var _explore_icon: ImageTexture

static func snapshot_target(snapshot: Dictionary, index: int) -> int:
	var targets: Variant = snapshot.get("target_kinds")
	return int(targets[index]) if targets is PackedByteArray and index >= 0 and index < targets.size() else -1

static func snapshot_wood(snapshot: Dictionary, index: int) -> int:
	var woods: Variant = snapshot.get("carried_woods")
	return int(woods[index]) if woods is PackedInt32Array and index >= 0 and index < woods.size() else 0

static func resolve(state: int, target_kind: int, has_targets: bool, moving: bool, carried: float, tile: Dictionary, _hour: float, carried_wood: int = 0, activity_kind: int = -1, arrived: bool = false) -> String:
	if state == 4:
		return daily_name(activity_kind, arrived)
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
				7: return "gather_wood"
				8: return daily_name(activity_kind, arrived)
		if not has_targets and bool(tile.get("wood", false)):
			return "gather_wood"
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
		if carried_wood > 0:
			return "carry_wood"
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

static func render_facing(activity: String, movement: Vector2, partner_delta: Vector2, previous: String, idle_seconds: float, facing_override: String = "") -> String:
	if not facing_override.is_empty():
		return facing_override
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
	var fps: float = 8.0 if activity in ["walk", "carry", "carry_wood", "run"] else (1.5 if activity == "idle" else 4.0)
	return posmod(floori((seconds + phase) * fps), maxi(count, 1))

static func elder_look(base: Dictionary, agent_id: int, life_stage: int) -> Dictionary:
	var look: Dictionary = base.duplicate()
	if life_stage == 2:
		look["hair"] = 5 + posmod(ArtHash.mix(agent_id), 2)
	else:
		look["hair"] = posmod(int(look["hair"]), 5)
	return look

static func build_frame_groups(entries: Dictionary) -> Dictionary:
	var groups: Dictionary = {}
	var audit_pages: Dictionary = {}
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
		frame["cold_audit_rejected"] = Cold.mark_audit(frame, audit_pages)
		groups[key].append(frame)
	for frames: Array in groups.values():
		frames.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a.get("frame", 0)) < int(b.get("frame", 0)))
	return groups

static func activity_icon(activity: String) -> String:
	if activity in ["carry_wood", "gather_wood"]:
		return wood_icon(ArtSpriteLibrary.entries())
	return str({"walk": "U1_01", "carry": "U1_05", "gather": "U1_06", "build": "U1_04", "eat": "U1_12", "drink": "U1_11", "sit": "U1_13", "asleep": "U1_13", "talk": "U1_01"}.get(activity, ""))

static func wood_icon(entries: Dictionary) -> String:
	for sprite_id: String in ["W1_12", "W1_16"]:
		var icon: Dictionary = entries.get(sprite_id, {})
		if str(icon.get("qa_status", "")) == "accepted" and bool(icon.get("drawable", true)):
			return sprite_id
	return "S2_15"

static func _real_carry(entry: Dictionary, body: String) -> bool:
	return (str(entry.get("body","")) == body
		and str(entry.get("activity","")) == "carry"
		and (str(entry.get("qa_status","")) == "accepted"
			or (str(entry.get("qa_status", "")) == "derived" and str(entry.get("source", "")) == "derived:same_cycle_phase_copy"))
		and bool(entry.get("drawable",true))
		and str(entry.get("fallback_kind","")) != "idle_icon"
		and str(entry.get("activity_icon","")).is_empty())

static func choose_frame(groups: Dictionary, body: String, activity: String, facing: String, seconds: float, phase: float) -> Dictionary:
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
				if _real_carry(entry,body) and str(entry.get("qa_status", "")) == "accepted":
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
			if real_carry and not _real_carry(entry,body):
				continue
			var basket: bool = bool(entry.get("contains_basket", false)) or str(entry.get("source_sprite", "")) == "direction_09_villager_gatherer"
			if wood and (bool(entry.get("contains_bush", false)) or str(entry.get("activity", "")) in ["gather", "carry"]):
				continue
			if basket and activity != "carry":
				continue
			safe.append(entry)
		if safe.is_empty():
			continue
		var chosen: Dictionary = safe[frame_index(activity, seconds, phase, safe.size())].duplicate()
		var idle_fallback: bool = str(chosen.get("activity", "")) == "idle" and activity != "idle"
		if wood and str(chosen.get("activity", "")) != activity:
			chosen["fallback_kind"] = "wood_icon"
			chosen["activity_icon"] = activity_icon(activity)
		elif idle_fallback:
			chosen["fallback_kind"] = "idle_icon"
			chosen["activity_icon"] = activity_icon(activity)
		elif str(chosen.get("fallback_kind", "")) != "idle_icon":
			chosen["activity_icon"] = ""
		return chosen
	return {}

static func show_activity_icon(choice: Dictionary, zoom: float) -> bool:
	return zoom >= 2.0 and str(choice.get("fallback_kind", "")) in ["idle_icon", "wood_icon", "activity_icon"] and not str(choice.get("activity_icon", "")).is_empty()

static func apply_activity_icon(sprite: Sprite2D, icon: Sprite2D, choice: Dictionary, zoom: float) -> bool:
	icon.visible = false
	if not sprite.visible or not show_activity_icon(choice, zoom):
		return false
	if str(choice["activity_icon"]) == "U1_04" or str(choice["activity_icon"]).begins_with("p11_activity_"):
		var kind: int = 1 if str(choice["activity_icon"]) == "U1_04" else int(str(choice["activity_icon"]).trim_prefix("p11_activity_"))
		icon.texture = DailyIcons.texture(kind)
		var size: Vector2 = icon.texture.get_size() if icon.texture != null else Vector2.ZERO
		icon.region_enabled = false
		icon.centered = false
		icon.offset = Vector2(-size.x * 0.5, -size.y)
		icon.scale = Vector2.ONE
		icon.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		icon.set_meta("sprite_id", str(choice["activity_icon"]))
		icon.set_meta("frame_id", str(choice["activity_icon"]))
		icon.set_meta("frame_size", Vector2i(size))
		icon.set_meta("anchor", Vector2(size.x * 0.5, size.y))
		icon.set_meta("drawable", icon.texture != null)
		icon.visible = icon.texture != null
	else:
		icon.visible = ArtSpriteLibrary.apply(icon, str(choice["activity_icon"]))
	icon.position = Vector2(0, sprite.offset.y - 2.0)
	icon.set_meta("art_kind", "activity_icon")
	return icon.visible


static func daily_name(kind: int, arrived: bool) -> String:
	if kind < 0 or kind >= DAILY_NAMES.size():
		return "idle"
	return DAILY_NAMES[kind] if arrived else "walk"

static func activity_rows(snapshot: Dictionary) -> Dictionary:
	var result: Dictionary = {}
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	for i: int in ids.size():
		result[int(ids[i])] = {"kind": int(snapshot["kinds"][i]), "place": Vector2i(snapshot["place_x"][i], snapshot["place_y"][i]), "until_tick": int(snapshot["until_tick"][i])}
	return result

## Exact pose/view candidates, without the generic choose_frame fallback hiding misses.
static func daily_chain(body: String, kind: int, direction: String, seconds: float, arrived: bool) -> Array[Dictionary]:
	var candidates: Array[Dictionary] = []
	var facing_direction: String = direction if direction in ["E", "W", "N", "S"] else "S"
	var sit_direction: String = "S" if facing_direction == "N" and body != "adult_b" else facing_direction
	if not arrived:
		if kind == 5 and body == "child" and facing_direction in ["E", "W"]:
			candidates.append({"pose": "run", "facing": facing_direction})
		candidates.append({"pose": "walk", "facing": facing_direction})
	else:
		match kind:
			0:
				if body != "child":
					candidates.append({"pose": "crouch", "facing": "S"})
				candidates.append({"pose": "sit", "facing": sit_direction})
			1, 3, 6:
				candidates.append({"pose": "sit", "facing": sit_direction})
			2:
				candidates.append({"pose": "build", "facing": "S" if body == "child" and facing_direction == "N" else facing_direction})
			4:
				candidates.append({"pose": "idle", "facing": facing_direction})
			5:
				if body == "child":
					if posmod(floori(seconds / 0.5), 2) == 1 and facing_direction in ["E", "W"]:
						candidates.append({"pose": "run", "facing": facing_direction})
					candidates.append({"pose": "hop", "facing": "S"})
	candidates.append({"pose": "idle", "facing": facing_direction})
	candidates.append({"pose": "idle", "facing": "S"})
	return candidates

static func daily_pose(groups: Dictionary, body: String, kind: int, direction: String, seconds: float, arrived: bool) -> Dictionary:
	for candidate: Dictionary in daily_chain(body, kind, direction, seconds, arrived):
		var view: String = "E" if str(candidate["facing"]) == "W" else str(candidate["facing"])
		if not (groups.get(body + "/" + str(candidate["pose"]) + "/" + view, []) as Array).is_empty():
			return candidate
	return {"pose": "idle", "facing": "S"}

static func with_daily_icon(choice: Dictionary, kind: int) -> Dictionary:
	if choice.is_empty() or kind < 0 or kind > 6:
		return choice
	var result: Dictionary = choice.duplicate()
	result["fallback_kind"] = "activity_icon"
	result["activity_icon"] = DailyIcons.icon_id(kind)
	return result

static func explore_icon() -> ImageTexture:
	if _explore_icon == null:
		var image: Image = Image.create(9, 9, false, Image.FORMAT_RGBA8)
		image.fill(Color.TRANSPARENT)
		# A small north-east direction arrow, kept separate from the agent atlas.
		for point: Vector2i in [Vector2i(2, 6), Vector2i(3, 5), Vector2i(4, 4), Vector2i(5, 3), Vector2i(6, 2), Vector2i(3, 2), Vector2i(4, 2), Vector2i(5, 2), Vector2i(6, 3), Vector2i(6, 4), Vector2i(6, 5)]:
			image.set_pixelv(point, Color8(245, 221, 150))
		_explore_icon = ImageTexture.create_from_image(image)
	return _explore_icon
