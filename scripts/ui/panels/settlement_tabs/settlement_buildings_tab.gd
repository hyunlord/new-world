extends RefCounted

const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const WARNING_COLOR: Color = Color(0.9, 0.7, 0.2)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const INDENT: float = 12.0


func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var settlement = data.get("settlement", null)
	if settlement == null:
		return cy

	var buildings: Array = _resolve_buildings(data, settlement)

	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_BUILDINGS_HEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	canvas.draw_string(
		font,
		Vector2(cx + INDENT, cy),
		"%s: %d" % [Locale.ltr("UI_SETTLEMENT_BUILDINGS_TOTAL"), buildings.size()],
		HORIZONTAL_ALIGNMENT_LEFT,
		-1,
		body_size,
		NEUTRAL_COLOR
	)
	cy += LINE_HEIGHT

	if buildings.is_empty():
		canvas.draw_string(font, Vector2(cx + INDENT, cy), Locale.ltr("UI_SETTLEMENT_BUILDINGS_NONE"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		return cy + LINE_HEIGHT + SECTION_GAP

	cy += SECTION_GAP * 0.5
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_BUILDINGS_BY_TYPE"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var type_counts: Dictionary = {}
	for building in buildings:
		var building_type: String = _building_type(building)
		type_counts[building_type] = int(type_counts.get(building_type, 0)) + 1

	var type_keys: Array = type_counts.keys()
	type_keys.sort()
	for building_type in type_keys:
		canvas.draw_string(font, Vector2(cx + INDENT, cy), "%s x %d" % [_building_label(building_type), int(type_counts[building_type])], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	cy += SECTION_GAP
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_BUILDINGS_LIST"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	for building in buildings:
		var building_type: String = _building_type(building)
		var state_key: String = _building_state_key(building)
		var state_color: Color = _building_state_color(state_key)
		var workers: int = int(_building_value(
			building,
			"assigned_builder_count",
			_building_value(building, "worker_count", _building_value(building, "workers", 0))
		))
		var pos_x: int = int(_building_value(building, "tile_x", _building_value(building, "x", _building_value(building, "pos_x", 0))))
		var pos_y: int = int(_building_value(building, "tile_y", _building_value(building, "y", _building_value(building, "pos_y", 0))))
		var progress: float = float(_building_value(building, "construction_progress", _building_value(building, "build_progress", 1.0)))
		var condition: float = float(_building_value(building, "condition", 1.0))
		var primary_line: String = "%s  [%s]" % [_building_label(building_type), Locale.ltr(state_key)]
		canvas.draw_string(font, Vector2(cx + INDENT, cy), primary_line, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, state_color)
		var region_top: float = cy - LINE_HEIGHT
		cy += LINE_HEIGHT

		var condition_pct: int = int(round(condition * 100.0))
		var info_line: String
		if _is_constructed(building):
			info_line = "%s %d%%  %s: %d  %s" % [
				Locale.ltr("UI_BUILDING_CONDITION"),
				condition_pct,
				Locale.ltr("UI_SETTLEMENT_WORKERS"),
				workers,
				Locale.trf2("UI_POS_FMT", "x", pos_x, "y", pos_y),
			]
		else:
			info_line = "%s: %d%%  %s: %d  %s" % [
				Locale.ltr("UI_UNDER_CONSTRUCTION"),
				int(round(progress * 100.0)),
				Locale.ltr("UI_SETTLEMENT_WORKERS"),
				workers,
				Locale.trf2("UI_POS_FMT", "x", pos_x, "y", pos_y),
			]
		canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy), info_line, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

		var building_id: int = int(_building_value(building, "id", -1))
		if building_id >= 0:
			click_regions.append({
				"rect": Rect2(cx, region_top, width, LINE_HEIGHT * 2.0),
				"building_id": building_id,
			})

	return cy + SECTION_GAP


func _resolve_buildings(data: Dictionary, settlement: Variant) -> Array:
	var buildings: Array = data.get("buildings", [])
	if not buildings.is_empty():
		return buildings
	var building_manager = data.get("building_manager", null)
	if building_manager == null:
		return []
	var resolved: Array = []
	for building_id in _settlement_building_ids(settlement):
		var legacy_building: Variant = building_manager.get_building(building_id)
		if legacy_building != null:
			resolved.append(legacy_building)
	return resolved


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _settlement_building_ids(settlement: Variant) -> Array:
	if settlement is Dictionary:
		var building_ids: Variant = settlement.get("building_ids", [])
		return building_ids if building_ids is Array else []
	if settlement == null:
		return []
	return settlement.building_ids


func _building_type(building: Variant) -> String:
	return str(_building_value(building, "building_type", _building_value(building, "type", "unknown")))


func _building_label(building_type: String) -> String:
	var label: String = Locale.tr_id("BUILDING", building_type)
	return label if label != "" else building_type.capitalize()


func _is_constructed(building: Variant) -> bool:
	return bool(_building_value(building, "is_constructed", _building_value(building, "is_built", false)))


func _building_state_key(building: Variant) -> String:
	var state: String = str(_building_value(building, "construction_state", ""))
	if state == "":
		return "BUILDING_STATE_COMPLETE" if _is_constructed(building) else "BUILDING_STATE_ADVANCING"
	match state:
		"complete":
			return "BUILDING_STATE_COMPLETE"
		"advancing":
			return "BUILDING_STATE_ADVANCING"
		"stalled":
			return "BUILDING_STATE_STALLED"
		"active":
			return "BUILDING_STATE_ACTIVE"
		"building":
			return "BUILDING_STATE_BUILDING"
		"damaged":
			return "BUILDING_STATE_DAMAGED"
		"inactive":
			return "BUILDING_STATE_INACTIVE"
		_:
			return "BUILDING_STATE_COMPLETE" if _is_constructed(building) else "BUILDING_STATE_ADVANCING"


func _building_state_color(state_key: String) -> Color:
	match state_key:
		"BUILDING_STATE_COMPLETE", "BUILDING_STATE_ACTIVE":
			return POSITIVE_COLOR
		"BUILDING_STATE_STALLED", "BUILDING_STATE_BUILDING":
			return WARNING_COLOR
		"BUILDING_STATE_DAMAGED", "BUILDING_STATE_INACTIVE":
			return NEGATIVE_COLOR
		_:
			return NEUTRAL_COLOR
