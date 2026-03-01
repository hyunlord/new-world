extends RefCounted

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const GOLD_COLOR: Color = Color(1.0, 0.85, 0.3)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const BAR_HEIGHT: float = 12.0
const BAR_WIDTH: float = 200.0

## Age distribution colors
const COLOR_CHILD: Color = Color(0.5, 0.8, 1.0)
const COLOR_TEEN: Color = Color(0.3, 0.7, 0.4)
const COLOR_ADULT: Color = Color(0.2, 0.5, 0.9)
const COLOR_ELDER: Color = Color(0.7, 0.5, 0.3)


## Draw the population tab content into the parent panel's canvas.
## Returns the y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var total_pop: int = data.get("total_population", 0)
	var peak_pop: int = data.get("peak_pop", 0)
	var total_births: int = data.get("total_births", 0)
	var total_deaths: int = data.get("total_deaths", 0)
	var total_male: int = data.get("total_male", 0)
	var total_female: int = data.get("total_female", 0)
	var avg_age: float = data.get("avg_age_years", 0.0)
	var history = data.get("history", [])

	# ── 1. World Population Summary ────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_TOTAL_POP"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	# Population with trend arrow
	var prev_pop: int = total_pop
	if history.size() >= 10:
		var old_snapshot = history[history.size() - 10]
		prev_pop = old_snapshot.get("total_population", total_pop)
	elif history.size() > 0:
		var old_snapshot = history[0]
		prev_pop = old_snapshot.get("total_population", total_pop)

	var trend = _trend_text(float(total_pop), float(prev_pop))
	var total_pop_text: String = Locale.trf1("UI_STAT_POP_FMT", "n", total_pop)
	var pop_line: String = total_pop_text + "  " + trend["text"]
	canvas.draw_string(font, Vector2(cx, cy), pop_line, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
	# Draw trend arrow in its own color after the main text
	var pop_prefix_w: float = font.get_string_size(total_pop_text + "  ", HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
	canvas.draw_string(font, Vector2(cx + pop_prefix_w, cy), trend["text"], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, trend["color"])
	cy += LINE_HEIGHT

	# Peak population
	var peak_line: String = Locale.trf4(
		"UI_STAT_CURRENT_FMT",
		"current",
		total_pop,
		"peak",
		peak_pop,
		"deaths",
		total_deaths,
		"births",
		total_births
	)
	canvas.draw_string(font, Vector2(cx, cy), peak_line, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT

	# Birth / death counts
	var births_label: String = Locale.ltr("UI_BIRTH_RATE") + ": " + str(total_births)
	var deaths_label: String = "  |  " + Locale.ltr("UI_DEATH_RATE") + ": " + str(total_deaths)
	canvas.draw_string(font, Vector2(cx, cy), births_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, POSITIVE_COLOR)
	var births_w: float = font.get_string_size(births_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
	canvas.draw_string(font, Vector2(cx + births_w, cy), deaths_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEGATIVE_COLOR)
	cy += LINE_HEIGHT

	# Gender ratio
	var gender_line: String = Locale.ltr("UI_GENDER_RATIO") + ": " + Locale.trf2("UI_STAT_GENDER_FMT", "m", total_male, "f", total_female)
	canvas.draw_string(font, Vector2(cx, cy), gender_line, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT

	# Average age
	var avg_age_line: String = Locale.ltr("UI_AVG_AGE") + ": " + "%.1f" % avg_age
	canvas.draw_string(font, Vector2(cx, cy), avg_age_line, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 2. Age Distribution Bar ────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_AGE_DISTRIBUTION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var age_dist = data.get("age_distribution", {})
	var n_child: int = age_dist.get("child", 0)
	var n_teen: int = age_dist.get("teen", 0)
	var n_adult: int = age_dist.get("adult", 0)
	var n_elder: int = age_dist.get("elder", 0)
	var age_total: int = n_child + n_teen + n_adult + n_elder
	var age_total_f: float = float(maxi(age_total, 1))

	var bar_w: float = width - 20.0
	var bar_x: float = cx
	var bar_y: float = cy

	# Draw stacked bar background
	canvas.draw_rect(Rect2(bar_x, bar_y, bar_w, BAR_HEIGHT + 2.0), Color(0.15, 0.15, 0.2))

	# Draw each segment
	var seg_x: float = bar_x
	var child_w: float = bar_w * float(n_child) / age_total_f
	var teen_w: float = bar_w * float(n_teen) / age_total_f
	var adult_w: float = bar_w * float(n_adult) / age_total_f
	var elder_w: float = bar_w * float(n_elder) / age_total_f

	if child_w > 0.0:
		canvas.draw_rect(Rect2(seg_x, bar_y, child_w, BAR_HEIGHT + 2.0), COLOR_CHILD)
	seg_x += child_w
	if teen_w > 0.0:
		canvas.draw_rect(Rect2(seg_x, bar_y, teen_w, BAR_HEIGHT + 2.0), COLOR_TEEN)
	seg_x += teen_w
	if adult_w > 0.0:
		canvas.draw_rect(Rect2(seg_x, bar_y, adult_w, BAR_HEIGHT + 2.0), COLOR_ADULT)
	seg_x += adult_w
	if elder_w > 0.0:
		canvas.draw_rect(Rect2(seg_x, bar_y, elder_w, BAR_HEIGHT + 2.0), COLOR_ELDER)

	cy += BAR_HEIGHT + 6.0

	# Legend row — child, teen, adult, elder
	var legend_items: Array = [
		{"color": COLOR_CHILD, "label": "child", "count": n_child},
		{"color": COLOR_TEEN, "label": "teen", "count": n_teen},
		{"color": COLOR_ADULT, "label": "adult", "count": n_adult},
		{"color": COLOR_ELDER, "label": "elder", "count": n_elder},
	]
	var legend_x: float = cx
	var swatch_size: float = 10.0
	for item in legend_items:
		var pct: float = 100.0 * float(item["count"]) / age_total_f
		var legend_text: String = str(item["count"]) + " (" + "%.0f" % pct + "%)"
		canvas.draw_rect(Rect2(legend_x, cy - swatch_size + 2.0, swatch_size, swatch_size), item["color"])
		canvas.draw_string(font, Vector2(legend_x + swatch_size + 3.0, cy), legend_text, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, item["color"])
		var legend_item_w: float = font.get_string_size(legend_text, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
		legend_x += swatch_size + 3.0 + legend_item_w + 12.0

	cy += LINE_HEIGHT + SECTION_GAP

	# ── 3. Population by Settlement ───────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_POP_BY_SETTLEMENT"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var summaries = data.get("settlement_summaries", [])
	for summary in summaries:
		var s_id = summary.get("id", 0)
		var s_pop: int = summary.get("pop", 0)
		var s_male: int = summary.get("male", 0)
		var s_female: int = summary.get("female", 0)
		var s_happiness: float = summary.get("avg_happiness", 0.5)
		var s_stress: float = summary.get("avg_stress", 0.0)
		var s_leader = summary.get("leader", null)
		var settlement = summary.get("settlement", null)

		# Population trend for this settlement
		var s_prev_pop: int = s_pop
		for snapshot in history:
			var snap_summaries = snapshot.get("settlement_summaries", [])
			for snap_s in snap_summaries:
				if snap_s.get("id", -1) == s_id:
					s_prev_pop = snap_s.get("pop", s_pop)
					break

		var s_trend = _trend_text(float(s_pop), float(s_prev_pop))
		var s_pop_text: String = Locale.trf1("UI_STAT_POP_FMT", "n", s_pop)
		var row_prefix: String = "S" + str(s_id) + ": " + s_pop_text + " (M:" + str(s_male) + " F:" + str(s_female) + ")"
		var row_text: String = row_prefix + "  " + s_trend["text"]

		# Settlement row — clickable
		var row_w: float = font.get_string_size(row_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		canvas.draw_string(font, Vector2(cx, cy), row_text, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)

		# Trend arrow color overlay
		var trend_x_offset: float = font.get_string_size(row_prefix + "  ", HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		canvas.draw_string(font, Vector2(cx + trend_x_offset, cy), s_trend["text"], HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, s_trend["color"])

		if settlement != null:
			click_regions.append({
				"rect": Rect2(cx, cy - body_size, row_w, body_size + 4),
				"action": "open_settlement",
				"id": s_id,
			})

		cy += LINE_HEIGHT

		# Happiness bar
		var happiness_label: String = Locale.ltr("UI_AVG_HAPPINESS") + " "
		canvas.draw_string(font, Vector2(cx + 8.0, cy), happiness_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		var h_label_w: float = font.get_string_size(happiness_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
		_draw_bar(canvas, cx + 8.0 + h_label_w, cy - small_size + 2.0, BAR_WIDTH, BAR_HEIGHT, s_happiness, POSITIVE_COLOR)
		cy += LINE_HEIGHT - 2.0

		# Stress bar
		var stress_label: String = Locale.ltr("UI_STRESS") + " "
		canvas.draw_string(font, Vector2(cx + 8.0, cy), stress_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		var st_label_w: float = font.get_string_size(stress_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
		_draw_bar(canvas, cx + 8.0 + st_label_w, cy - small_size + 2.0, BAR_WIDTH, BAR_HEIGHT, s_stress, NEGATIVE_COLOR, Color(0.25, 0.1, 0.1))
		cy += LINE_HEIGHT - 2.0

		# Leader
		if s_leader != null:
			var leader_prefix: String = Locale.ltr("UI_LEADER") + ": "
			var leader_name: String = s_leader.entity_name
			canvas.draw_string(font, Vector2(cx + 8.0, cy), leader_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
			var lp_w: float = font.get_string_size(leader_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
			canvas.draw_string(font, Vector2(cx + 8.0 + lp_w, cy), leader_name, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, GOLD_COLOR)
			var ln_w: float = font.get_string_size(leader_name, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
			var leader_id = s_leader.id
			click_regions.append({
				"rect": Rect2(cx + 8.0 + lp_w, cy - small_size, ln_w, small_size + 4),
				"action": "open_entity",
				"id": leader_id,
			})
		else:
			canvas.draw_string(font, Vector2(cx + 8.0, cy), Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)

		cy += LINE_HEIGHT + SECTION_GAP * 0.5

	return cy


## Returns trend arrow text and color based on the change between current and previous values.
func _trend_text(current: float, previous: float) -> Dictionary:
	var diff: float = current - previous
	if diff > 0.5:
		return {"text": "▲", "color": Color(0.4, 0.8, 0.4)}
	elif diff < -0.5:
		return {"text": "▼", "color": Color(0.8, 0.4, 0.4)}
	else:
		return {"text": "━", "color": Color(0.5, 0.5, 0.5)}


## Draw a filled progress bar with background.
func _draw_bar(canvas: Control, x: float, y: float, w: float, h: float, value: float, fill_color: Color, bg_color: Color = Color(0.15, 0.15, 0.2)) -> void:
	canvas.draw_rect(Rect2(x, y, w, h), bg_color)
	canvas.draw_rect(Rect2(x, y, w * clampf(value, 0.0, 1.0), h), fill_color)
