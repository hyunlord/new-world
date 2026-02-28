extends RefCounted

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0


## Draw the resources tab content into the parent panel's canvas.
## Returns the y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, _width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")

	var global_food: float = data.get("global_food", 0.0)
	var global_wood: float = data.get("global_wood", 0.0)
	var global_stone: float = data.get("global_stone", 0.0)
	var deltas: Dictionary = data.get("resource_deltas", {})
	var delta_food: float = deltas.get("food", 0.0)
	var delta_wood: float = deltas.get("wood", 0.0)
	var delta_stone: float = deltas.get("stone", 0.0)
	var summaries: Array = data.get("settlement_summaries", [])

	# ── 1. Resource Totals ─────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_RESOURCE_TOTALS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var resource_rows: Array = [
		{"name_key": "UI_FOOD",  "stock": global_food,  "delta": delta_food},
		{"name_key": "UI_WOOD",  "stock": global_wood,  "delta": delta_wood},
		{"name_key": "UI_STONE", "stock": global_stone, "delta": delta_stone},
	]

	for row in resource_rows:
		var res_name: String = Locale.ltr(row["name_key"])
		var stock: float = row["stock"]
		var delta: float = row["delta"]

		# Production / consumption labels derived from delta sign
		var prod_label: String
		var cons_label: String
		var prod_color: Color
		var cons_color: Color
		if delta >= 0.0:
			prod_label = "▲ +" + str(int(delta))
			cons_label = "▼ 0"
			prod_color = POSITIVE_COLOR
			cons_color = NEUTRAL_COLOR
		else:
			prod_label = "▲ 0"
			cons_label = "▼ " + str(int(-delta))
			prod_color = NEUTRAL_COLOR
			cons_color = NEGATIVE_COLOR

		# Net change
		var net_color: Color
		if delta > 0.0:
			net_color = POSITIVE_COLOR
		elif delta < 0.0:
			net_color = NEGATIVE_COLOR
		else:
			net_color = NEUTRAL_COLOR
		var net_sign: String = "+" if delta >= 0.0 else ""
		var net_label: String = net_sign + str(int(delta))

		# Row: "Food: 1234  |  Daily prod: ▲ +10  |  Net: +10"
		var prefix: String = (
			res_name + ": " + str(int(stock))
			+ "  |  " + Locale.ltr("UI_DAILY_PRODUCTION") + ": "
		)
		var suffix_cons: String = "  |  " + Locale.ltr("UI_DAILY_CONSUMPTION") + ": "
		var suffix_net: String = "  |  " + Locale.ltr("UI_NET_CHANGE") + ": "

		# Draw prefix in white
		canvas.draw_string(font, Vector2(cx, cy), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw production value in color
		canvas.draw_string(font, Vector2(cx + prefix_w, cy), prod_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, prod_color)
		var prod_w: float = font.get_string_size(prod_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw consumption separator in white
		canvas.draw_string(font, Vector2(cx + prefix_w + prod_w, cy), suffix_cons, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
		var cons_sep_w: float = font.get_string_size(suffix_cons, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw consumption value in color
		canvas.draw_string(font, Vector2(cx + prefix_w + prod_w + cons_sep_w, cy), cons_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, cons_color)
		var cons_w: float = font.get_string_size(cons_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw net separator in white
		canvas.draw_string(font, Vector2(cx + prefix_w + prod_w + cons_sep_w + cons_w, cy), suffix_net, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
		var net_sep_w: float = font.get_string_size(suffix_net, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw net value in color
		canvas.draw_string(font, Vector2(cx + prefix_w + prod_w + cons_sep_w + cons_w + net_sep_w, cy), net_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, net_color)

		cy += LINE_HEIGHT

	cy += SECTION_GAP

	# ── 2. Food Status by Settlement ───────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_FOOD_STATUS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var hunger_decay: float = GameConfig.HUNGER_DECAY_RATE
	var ticks_per_day: int = GameConfig.TICKS_PER_DAY

	for summary in summaries:
		var s_id = summary.get("id", 0)
		var settlement = summary.get("settlement", null)
		var s_pop: int = summary.get("pop", 0)
		var s_food: float = summary.get("food", 0.0)

		# Estimate daily consumption: pop * hunger_decay * ticks_per_day
		var daily_consumption: float = float(s_pop) * hunger_decay * float(ticks_per_day)
		var days_of_supply: float = 0.0
		if daily_consumption > 0.0:
			days_of_supply = s_food / daily_consumption

		var status: Dictionary = _resource_status(days_of_supply)
		var status_label: String = Locale.ltr(status["key"])
		var status_color: Color = status["color"]

		# Row prefix: "S1: F:500  ["
		var row_prefix: String = (
			"S" + str(s_id) + ": F:" + str(int(s_food))
			+ "  ["
		)
		# Row suffix: "]  (12d supply)"
		var row_suffix: String = "]  (" + str(int(days_of_supply)) + Locale.ltr("UI_DAYS_SUPPLY") + ")"

		canvas.draw_string(font, Vector2(cx, cy), row_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
		var rp_w: float = font.get_string_size(row_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		canvas.draw_string(font, Vector2(cx + rp_w, cy), status_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, status_color)
		var sl_w: float = font.get_string_size(status_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		canvas.draw_string(font, Vector2(cx + rp_w + sl_w, cy), row_suffix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)

		# Click region covers the full row prefix + status label width
		var row_total_w: float = font.get_string_size(row_prefix + status_label + row_suffix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		if settlement != null:
			click_regions.append({
				"rect": Rect2(cx, cy - body_size, row_total_w, body_size + 4),
				"action": "open_settlement",
				"id": s_id,
			})

		cy += LINE_HEIGHT

	cy += SECTION_GAP

	# ── 3. Buildings Summary ───────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_BUILDINGS_SUMMARY"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	for summary in summaries:
		var s_id = summary.get("id", 0)
		var settlement = summary.get("settlement", null)
		var building_count: int = 0
		if settlement != null:
			building_count = settlement.building_ids.size()

		var building_line: String = (
			"S" + str(s_id) + ": "
			+ str(building_count) + " " + Locale.ltr("UI_BUILDINGS_COUNT")
		)
		canvas.draw_string(font, Vector2(cx, cy), building_line, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	cy += SECTION_GAP

	return cy


## Returns status key and color based on days of supply.
func _resource_status(days_of_supply: float) -> Dictionary:
	if days_of_supply > GameConfig.STATS_RESOURCE_ABUNDANT_DAYS:
		return {"key": "UI_STATUS_ABUNDANT", "color": Color(0.4, 0.9, 0.4)}
	elif days_of_supply > GameConfig.STATS_RESOURCE_LOW_DAYS:
		return {"key": "UI_STATUS_STABLE", "color": Color(0.7, 0.7, 0.7)}
	elif days_of_supply > GameConfig.STATS_RESOURCE_DANGER_DAYS:
		return {"key": "UI_STATUS_LOW", "color": Color(0.9, 0.7, 0.2)}
	else:
		return {"key": "UI_STATUS_CRITICAL", "color": Color(0.9, 0.3, 0.3)}
