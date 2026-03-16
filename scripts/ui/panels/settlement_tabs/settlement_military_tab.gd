extends RefCounted

const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const WARNING_COLOR: Color = Color(0.9, 0.7, 0.2)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const INDENT: float = 12.0


func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, _width: float, _click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var members: Array = data.get("members", [])
	var buildings: Array = _resolve_buildings(data)
	var population: int = int(data.get("population", members.size()))

	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_MILITARY_HEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var garrison_count: int = 0
	var combat_strength: float = 0.0
	for member in members:
		var job: String = str(_member_value(member, "job", ""))
		var action_text: String = str(_member_value(member, "current_action", "")).to_lower()
		var is_guard_role: bool = job in ["warrior", "guard", "hunter"]
		if not is_guard_role and action_text in ["flee", "hunt", "patrol", "guard"]:
			is_guard_role = true
		if not is_guard_role:
			continue
		garrison_count += 1
		combat_strength += float(_member_value(member, "combat_mult", _member_value(member, "strength", 1.0)))

	canvas.draw_string(font, Vector2(cx + INDENT, cy), "%s: %d / %d" % [Locale.ltr("UI_SETTLEMENT_GARRISON"), garrison_count, population], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT
	canvas.draw_string(font, Vector2(cx + INDENT, cy), "%s: %.1f" % [Locale.ltr("UI_SETTLEMENT_COMBAT_STRENGTH"), combat_strength], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT

	cy += SECTION_GAP
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_FORTIFICATIONS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var wall_count: int = 0
	var watchtower_count: int = 0
	for building in buildings:
		var building_type: String = str(_building_value(building, "building_type", ""))
		if building_type.contains("wall") or building_type.contains("palisade"):
			wall_count += 1
		elif building_type.contains("tower"):
			watchtower_count += 1

	if wall_count == 0 and watchtower_count == 0:
		canvas.draw_string(font, Vector2(cx + INDENT, cy), Locale.ltr("UI_SETTLEMENT_NO_FORTIFICATIONS"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, WARNING_COLOR)
		cy += LINE_HEIGHT
	else:
		if wall_count > 0:
			canvas.draw_string(font, Vector2(cx + INDENT, cy), "%s: %d" % [Locale.ltr("UI_SETTLEMENT_WALLS"), wall_count], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT
		if watchtower_count > 0:
			canvas.draw_string(font, Vector2(cx + INDENT, cy), "%s: %d" % [Locale.ltr("UI_SETTLEMENT_WATCHTOWERS"), watchtower_count], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT

	cy += SECTION_GAP
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_THREAT_HEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var threat_key: String = "UI_THREAT_LOW"
	var threat_color: Color = POSITIVE_COLOR
	if garrison_count == 0 and wall_count == 0 and watchtower_count == 0:
		threat_key = "UI_THREAT_HIGH"
		threat_color = NEGATIVE_COLOR
	elif garrison_count < 3 or (wall_count == 0 and watchtower_count == 0):
		threat_key = "UI_THREAT_MEDIUM"
		threat_color = WARNING_COLOR

	canvas.draw_string(font, Vector2(cx + INDENT, cy), "%s: %s" % [Locale.ltr("UI_SETTLEMENT_THREAT_LEVEL"), Locale.ltr(threat_key)], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, threat_color)
	cy += LINE_HEIGHT

	cy += SECTION_GAP
	canvas.draw_string(font, Vector2(cx + INDENT, cy), Locale.ltr("UI_SETTLEMENT_MILITARY_PHASE4"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, Color(0.35, 0.35, 0.45))
	return cy + LINE_HEIGHT + SECTION_GAP


func _resolve_buildings(data: Dictionary) -> Array:
	var buildings: Array = data.get("buildings", [])
	return buildings


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _member_value(member: Variant, key: String, default_value: Variant) -> Variant:
	if member is Dictionary:
		return member.get(key, default_value)
	if member == null:
		return default_value
	return member.get(key)
