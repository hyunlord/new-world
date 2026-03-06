extends RefCounted

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const BAR_HEIGHT: float = 12.0
const BAR_WIDTH: float = 200.0

## Housing capacity per building type (estimated)
const HOUSING_CAP: Dictionary = {
	"hut": 4,
	"cabin": 6,
	"longhouse": 12,
	"shelter": 4,
}
const DEFAULT_HOUSING_CAP: int = 4


## Draw the economy tab content into the parent panel's canvas.
## Returns the y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, _width: float, _click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var settlement = data.get("settlement", null)
	if settlement == null:
		return cy

	var members: Array = data.get("members", [])
	var building_manager = data.get("building_manager", null)
	var buildings: Array = data.get("buildings", [])
	var population: int = data.get("population", 0)

	# ── 1. Resources Section ───────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_RESOURCE_HEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	# Column headers
	var header_line: String = (
		"                "
		+ Locale.ltr("UI_RESOURCE_CURRENT")
		+ "    "
		+ Locale.ltr("UI_RESOURCE_DAILY_PROD")
		+ "    "
		+ Locale.ltr("UI_RESOURCE_DAILY_CONS")
		+ "    "
		+ Locale.ltr("UI_RESOURCE_NET")
	)
	canvas.draw_string(font, Vector2(cx, cy), header_line, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT

	# Resource rows
	var resource_types: Array = ["food", "wood", "stone"]
	var resource_label_keys: Dictionary = {
		"food": "UI_FOOD",
		"wood": "UI_WOOD",
		"stone": "UI_STONE",
	}
	for res in resource_types:
		var total: float = 0.0
		for member in members:
			total += float(_member_inventory(member).get(res, 0.0))
		var label: String = Locale.ltr(resource_label_keys[res])
		var current_str: String = str(roundi(total))
		cy = _draw_resource_row(canvas, font, cx, cy, label, current_str, "—", "—", "—", NEUTRAL_COLOR, body_size)

	cy += SECTION_GAP

	# ── 2. Buildings Section ───────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_BUILDINGS_LIST"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	if building_manager == null and buildings.is_empty():
		canvas.draw_string(font, Vector2(cx + 8.0, cy), "—", HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT + SECTION_GAP
	else:
		# Group buildings by type
		var building_counts: Dictionary = {}
		var settlement_buildings: Array = buildings
		if settlement_buildings.is_empty() and building_manager != null:
			for bid in _settlement_building_ids(settlement):
				var legacy_building: Variant = building_manager.get_building(bid)
				if legacy_building != null:
					settlement_buildings.append(legacy_building)
		for b in settlement_buildings:
			var btype: String = str(_building_value(b, "building_type", ""))
			if not building_counts.has(btype):
				building_counts[btype] = {"count": 0, "constructing": 0, "progress": 0.0}
			if bool(_building_value(b, "is_constructed", _building_value(b, "is_built", false))):
				building_counts[btype]["count"] += 1
			else:
				building_counts[btype]["constructing"] += 1
				building_counts[btype]["progress"] = float(_building_value(b, "construction_progress", _building_value(b, "build_progress", 0.0)))

		var total_built: int = 0
		var has_any: bool = false
		for btype in building_counts:
			var entry = building_counts[btype]
			var count: int = entry["count"]
			if count > 0:
				var loc_key: String = "BUILDING_" + btype.to_upper()
				var bname: String = Locale.ltr(loc_key)
				# Fallback: if key not found, Locale returns the key itself — use capitalize
				if bname == loc_key:
					bname = btype.capitalize()
				var row_text: String = bname + " ×" + str(count)
				canvas.draw_string(font, Vector2(cx + 8.0, cy), row_text, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
				cy += LINE_HEIGHT
				total_built += count
				has_any = true

		if not has_any:
			canvas.draw_string(font, Vector2(cx + 8.0, cy), "—", HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT

		# Total buildings line
		var total_label: String = Locale.ltr("UI_BUILDINGS_BUILT") + ": " + str(total_built)
		canvas.draw_string(font, Vector2(cx, cy), total_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT + SECTION_GAP

		# Under construction
		var has_constructing: bool = false
		for btype in building_counts:
			if building_counts[btype]["constructing"] > 0:
				has_constructing = true
				break

		if has_constructing:
			canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_UNDER_CONSTRUCTION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
			cy += LINE_HEIGHT

			for btype in building_counts:
				var entry = building_counts[btype]
				var constructing: int = entry["constructing"]
				if constructing <= 0:
					continue
				var progress: float = entry["progress"]
				var loc_key: String = "BUILDING_" + btype.to_upper()
				var bname: String = Locale.ltr(loc_key)
				if bname == loc_key:
					bname = btype.capitalize()
				var pct_str: String = str(roundi(progress * 100.0)) + "%"
				var row_text: String = bname + " (" + pct_str + ")"
				canvas.draw_string(font, Vector2(cx + 8.0, cy), row_text, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
				cy += LINE_HEIGHT - 4.0
				_draw_bar(canvas, cx + 8.0, cy, BAR_WIDTH, BAR_HEIGHT, progress, POSITIVE_COLOR)
				cy += BAR_HEIGHT + 6.0

			cy += SECTION_GAP

	# ── 3. Population Capacity Section ────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_POP_CAPACITY"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	# Calculate housing capacity
	var housing_cap: int = 0
	if not buildings.is_empty() or building_manager != null:
		var capacity_buildings: Array = buildings
		if capacity_buildings.is_empty() and building_manager != null:
			for bid in _settlement_building_ids(settlement):
				var legacy_building: Variant = building_manager.get_building(bid)
				if legacy_building != null:
					capacity_buildings.append(legacy_building)
		for b in capacity_buildings:
			if not bool(_building_value(b, "is_constructed", _building_value(b, "is_built", false))):
				continue
			var btype: String = str(_building_value(b, "building_type", ""))
			housing_cap += HOUSING_CAP.get(btype, DEFAULT_HOUSING_CAP)
	if housing_cap == 0:
		housing_cap = DEFAULT_HOUSING_CAP

	# Food-based capacity: total food / 2.0
	var total_food: float = 0.0
	for member in members:
		total_food += float(_member_inventory(member).get("food", 0.0))
	var food_cap: int = maxi(1, roundi(total_food / 2.0))

	var overall_cap: int = mini(housing_cap, food_cap)
	var cap_ratio: float = clampf(float(population) / float(maxi(overall_cap, 1)), 0.0, 1.0)

	# Housing capacity line
	var housing_label: String = Locale.ltr("UI_HOUSING_CAPACITY") + ": " + str(housing_cap)
	canvas.draw_string(font, Vector2(cx + 8.0, cy), housing_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT - 4.0

	# Food capacity line
	var food_cap_label: String = Locale.ltr("UI_FOOD_CAPACITY") + ": " + str(food_cap)
	canvas.draw_string(font, Vector2(cx + 8.0, cy), food_cap_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT + 2.0

	# Current / cap summary
	var cap_summary: String = Locale.ltr("UI_POP_CAPACITY") + ": " + str(population) + " / " + str(overall_cap)
	var cap_color: Color = POSITIVE_COLOR if population < overall_cap else NEGATIVE_COLOR
	canvas.draw_string(font, Vector2(cx, cy), cap_summary, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, cap_color)
	cy += LINE_HEIGHT - 4.0
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, cap_ratio, cap_color)
	cy += BAR_HEIGHT + 6.0

	# Limiting factor
	var limiting_key: String
	if housing_cap <= food_cap:
		limiting_key = "UI_HOUSING_CAPACITY"
	else:
		limiting_key = "UI_FOOD_CAPACITY"
	var limiting_text: String = Locale.ltr("UI_LIMITING_FACTOR") + ": " + Locale.ltr(limiting_key)
	canvas.draw_string(font, Vector2(cx, cy), limiting_text, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT + SECTION_GAP

	return cy


## Draw a single resource row with aligned columns.
## Returns the next y position.
func _draw_resource_row(canvas: Control, font: Font, x: float, y: float, label: String, current: String, prod: String, cons: String, net: String, net_color: Color, font_size: int) -> float:
	canvas.draw_string(font, Vector2(x, y), label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.8, 0.8, 0.8))
	canvas.draw_string(font, Vector2(x + 120.0, y), current, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.9, 0.9, 0.9))
	canvas.draw_string(font, Vector2(x + 220.0, y), prod, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, POSITIVE_COLOR)
	canvas.draw_string(font, Vector2(x + 320.0, y), cons, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, NEGATIVE_COLOR)
	canvas.draw_string(font, Vector2(x + 420.0, y), net, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, net_color)
	return y + LINE_HEIGHT


## Draw a filled progress bar with background.
func _draw_bar(canvas: Control, x: float, y: float, w: float, h: float, value: float, fill_color: Color, bg_color: Color = Color(0.15, 0.15, 0.2)) -> void:
	canvas.draw_rect(Rect2(x, y, w, h), bg_color)
	canvas.draw_rect(Rect2(x, y, w * clampf(value, 0.0, 1.0), h), fill_color)


func _member_inventory(member: Variant) -> Dictionary:
	if member is Dictionary:
		var inventory: Variant = member.get("inventory", {})
		return inventory if inventory is Dictionary else {}
	if member == null:
		return {}
	return member.inventory


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
