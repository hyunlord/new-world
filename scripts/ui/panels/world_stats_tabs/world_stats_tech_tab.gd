extends RefCounted

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const GOLD_COLOR: Color = Color(1.0, 0.85, 0.3)
const WARNING_COLOR: Color = Color(0.9, 0.7, 0.2)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0

## Era order for comparison (lower index = earlier era)
const ERA_ORDER: Array = ["stone_age", "tribal", "bronze_age", "iron_age", "medieval", "renaissance"]


## Draw the Technology tab content into the world stats panel canvas.
## Returns the final y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var settlement_summaries: Array = data.get("settlement_summaries", [])
	var ttm = data.get("tech_tree_manager", null)

	if settlement_summaries.is_empty():
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_TECH_SUMMARY"),
				HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT
		return cy

	# ── 1. Technology Summary ───────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_TECH_SUMMARY"),
			HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	# Count unique known techs across all settlements
	var all_known_ids: Dictionary = {}
	for summary in settlement_summaries:
		var settlement = summary.get("settlement", null)
		if settlement == null:
			continue
		var tech_states: Dictionary = settlement.tech_states if "tech_states" in settlement else {}
		for tech_id in tech_states:
			var cts: Dictionary = tech_states[tech_id]
			var state: String = cts.get("state", "unknown")
			if state == "known_stable" or state == "known_low":
				all_known_ids[tech_id] = true

	var total_discovered: int = all_known_ids.size()
	canvas.draw_string(font, Vector2(cx + 8.0, cy),
			Locale.ltr("UI_TOTAL_TECHS") + ": " + str(total_discovered),
			HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
	cy += LINE_HEIGHT

	# Most advanced era
	var most_advanced_era: String = "stone_age"
	var most_advanced_era_index: int = 0
	for summary in settlement_summaries:
		var settlement = summary.get("settlement", null)
		if settlement == null:
			continue
		var era: String = settlement.tech_era if "tech_era" in settlement else "stone_age"
		var era_idx: int = ERA_ORDER.find(era)
		if era_idx < 0:
			era_idx = 0
		if era_idx > most_advanced_era_index:
			most_advanced_era_index = era_idx
			most_advanced_era = era

	var most_advanced_label: String = Locale.ltr("UI_MOST_ADVANCED") + ": " + Locale.ltr("ERA_" + most_advanced_era.to_upper())
	canvas.draw_string(font, Vector2(cx + 8.0, cy), most_advanced_label,
			HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, GOLD_COLOR)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 2. Technology by Settlement ─────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_TECH_BY_SETTLEMENT"),
			HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	for summary in settlement_summaries:
		var settlement = summary.get("settlement", null)
		if settlement == null:
			continue
		var s_id: int = summary.get("id", 0)
		var tech_states: Dictionary = settlement.tech_states if "tech_states" in settlement else {}
		var era: String = settlement.tech_era if "tech_era" in settlement else "stone_age"

		# Count known and forgotten
		var known_count: int = 0
		var forgotten_count: int = 0
		for tech_id in tech_states:
			var cts: Dictionary = tech_states[tech_id]
			var state: String = cts.get("state", "unknown")
			if state == "known_stable" or state == "known_low":
				known_count += 1
			elif state == "forgotten_recent" or state == "forgotten_long":
				forgotten_count += 1

		# Era badge color
		var era_color: Color
		match era:
			"stone_age":
				era_color = Color(0.6, 0.6, 0.6)
			"tribal":
				era_color = Color(0.6, 0.8, 0.3)
			"bronze_age":
				era_color = Color(0.8, 0.6, 0.2)
			_:
				era_color = NEUTRAL_COLOR

		var era_badge: String = "[" + Locale.ltr("ERA_" + era.to_upper()) + "]"
		var count_text: String = Locale.trf("UI_TECH_COUNT_FMT", {"known": known_count, "forgotten": forgotten_count})
		var row_prefix: String = "S" + str(s_id) + ": "

		# Draw prefix
		canvas.draw_string(font, Vector2(cx + 8.0, cy), row_prefix,
				HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
		var prefix_w: float = font.get_string_size(row_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw era badge in era color
		canvas.draw_string(font, Vector2(cx + 8.0 + prefix_w, cy), era_badge,
				HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, era_color)
		var badge_w: float = font.get_string_size(era_badge, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		# Draw count text
		var sep: String = " - "
		canvas.draw_string(font, Vector2(cx + 8.0 + prefix_w + badge_w, cy), sep + count_text,
				HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)

		# Click region for settlement row
		var row_rect := Rect2(cx, cy - body_size, width, body_size + 4.0)
		click_regions.append({"rect": row_rect, "action": "open_settlement", "id": settlement.id})

		cy += LINE_HEIGHT

	cy += SECTION_GAP

	# ── 3. Technology Comparison Matrix ─────────────────────────────────────
	if ttm != null:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_TECH_COMPARISON"),
				HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT

		# Collect all unique tech IDs encountered across all settlements
		var all_tech_ids_dict: Dictionary = {}
		for summary in settlement_summaries:
			var settlement = summary.get("settlement", null)
			if settlement == null:
				continue
			var tech_states: Dictionary = settlement.tech_states if "tech_states" in settlement else {}
			for tech_id in tech_states:
				all_tech_ids_dict[tech_id] = true

		# Sort techs by tier then id
		var all_tech_ids: Array = all_tech_ids_dict.keys()
		all_tech_ids.sort_custom(func(a: String, b: String) -> bool:
			var def_a: Dictionary = ttm.get_def(a)
			var def_b: Dictionary = ttm.get_def(b)
			var tier_a: int = def_a.get("tier", 0)
			var tier_b: int = def_b.get("tier", 0)
			if tier_a != tier_b:
				return tier_a < tier_b
			return a < b
		)

		if all_tech_ids.is_empty():
			canvas.draw_string(font, Vector2(cx + 8.0, cy), Locale.ltr("UI_TECH_SUMMARY"),
					HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT + SECTION_GAP
		else:
			# Column layout: tech name column + one column per settlement
			var name_col_w: float = 120.0
			var cell_w: float = 36.0
			var num_settlements: int = settlement_summaries.size()

			# Header row: settlement IDs
			canvas.draw_string(font, Vector2(cx + 8.0, cy), Locale.ltr("UI_TECH_NAME"),
					HORIZONTAL_ALIGNMENT_LEFT, name_col_w, small_size, NEUTRAL_COLOR)
			for i in range(num_settlements):
				var s_id: int = settlement_summaries[i].get("id", i)
				var col_x: float = cx + 8.0 + name_col_w + float(i) * cell_w
				canvas.draw_string(font, Vector2(col_x, cy), "S" + str(s_id),
						HORIZONTAL_ALIGNMENT_LEFT, cell_w, small_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT

			# Tech rows
			for tech_id in all_tech_ids:
				var tech_name: String = Locale.ltr(tech_id)
				# Truncate name if too long
				var display_name: String = tech_name
				var max_name_len: int = 14
				if display_name.length() > max_name_len:
					display_name = display_name.substr(0, max_name_len - 1) + "…"

				canvas.draw_string(font, Vector2(cx + 8.0, cy), display_name,
						HORIZONTAL_ALIGNMENT_LEFT, name_col_w, small_size, Color.WHITE)

				for i in range(num_settlements):
					var summary = settlement_summaries[i]
					var settlement = summary.get("settlement", null)
					var col_x: float = cx + 8.0 + name_col_w + float(i) * cell_w

					if settlement == null:
						canvas.draw_string(font, Vector2(col_x, cy), "—",
								HORIZONTAL_ALIGNMENT_LEFT, cell_w, small_size, Color(0.4, 0.4, 0.4))
						continue

					var tech_states: Dictionary = settlement.tech_states if "tech_states" in settlement else {}
					if not tech_states.has(tech_id):
						canvas.draw_string(font, Vector2(col_x, cy), "—",
								HORIZONTAL_ALIGNMENT_LEFT, cell_w, small_size, Color(0.4, 0.4, 0.4))
					else:
						var cts: Dictionary = tech_states[tech_id]
						var state: String = cts.get("state", "unknown")
						var cell_symbol: String
						var cell_color: Color
						match state:
							"known_stable":
								cell_symbol = "●"
								cell_color = Color(0.3, 0.8, 0.3)
							"known_low":
								cell_symbol = "◐"
								cell_color = Color(0.8, 0.8, 0.3)
							"forgotten_recent", "forgotten_long":
								cell_symbol = "✕"
								cell_color = Color(0.8, 0.3, 0.3)
							_:
								cell_symbol = "—"
								cell_color = Color(0.4, 0.4, 0.4)
						canvas.draw_string(font, Vector2(col_x, cy), cell_symbol,
								HORIZONTAL_ALIGNMENT_LEFT, cell_w, small_size, cell_color)

				cy += LINE_HEIGHT

			# Legend
			cy += 4.0
			var legend_x: float = cx + 8.0
			var legend_items: Array = [
				{"symbol": "●", "color": Color(0.3, 0.8, 0.3), "key": "UI_TECH_STATE_STABLE_SHORT"},
				{"symbol": "◐", "color": Color(0.8, 0.8, 0.3), "key": "UI_TECH_STATE_LOW_SHORT"},
				{"symbol": "✕", "color": Color(0.8, 0.3, 0.3), "key": "UI_TECH_STATE_FORGOTTEN_SHORT"},
			]
			for item in legend_items:
				var sym: String = item.symbol + " " + Locale.ltr(item.key) + "  "
				canvas.draw_string(font, Vector2(legend_x, cy), sym,
						HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, item.color)
				legend_x += font.get_string_size(sym, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
			cy += LINE_HEIGHT + SECTION_GAP

	# ── 4. Regression Warnings ──────────────────────────────────────────────
	var warnings: Array = []
	for summary in settlement_summaries:
		var settlement = summary.get("settlement", null)
		if settlement == null:
			continue
		var s_id: int = summary.get("id", 0)
		var tech_states: Dictionary = settlement.tech_states if "tech_states" in settlement else {}
		for tech_id in tech_states:
			var cts: Dictionary = tech_states[tech_id]
			var state: String = cts.get("state", "unknown")
			if state == "known_low":
				var tech_name: String = Locale.ltr(tech_id)
				var state_label: String = Locale.ltr("UI_TECH_STATE_LOW_SHORT")
				warnings.append({
					"s_id": s_id,
					"tech_name": tech_name,
					"state_label": state_label,
					"color": WARNING_COLOR,
				})
			elif state == "forgotten_recent":
				var tech_name: String = Locale.ltr(tech_id)
				var state_label: String = Locale.ltr("UI_TECH_STATE_FORGOTTEN_SHORT")
				warnings.append({
					"s_id": s_id,
					"tech_name": tech_name,
					"state_label": state_label,
					"color": NEGATIVE_COLOR,
				})

	if not warnings.is_empty():
		var warn_header: String = "⚠ " + Locale.ltr("UI_REGRESSION_WARNINGS")
		canvas.draw_string(font, Vector2(cx, cy), warn_header,
				HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, WARNING_COLOR)
		cy += LINE_HEIGHT

		for w in warnings:
			var warn_line: String = "S" + str(w.s_id) + ": " + w.tech_name + " — " + w.state_label
			canvas.draw_string(font, Vector2(cx + 8.0, cy), warn_line,
					HORIZONTAL_ALIGNMENT_LEFT, width - 8.0, body_size, w.color)
			cy += LINE_HEIGHT

		cy += SECTION_GAP

	return cy
