extends RefCounted

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const BAR_HEIGHT: float = 12.0

const HEXACO_COLORS: Dictionary = {
	"H": Color(0.9, 0.7, 0.2),
	"E": Color(0.4, 0.6, 0.9),
	"X": Color(0.9, 0.5, 0.2),
	"A": Color(0.3, 0.8, 0.5),
	"C": Color(0.2, 0.6, 0.9),
	"O": Color(0.7, 0.4, 0.9),
}

const AGE_BAR_COLOR: Color = Color(0.3, 0.5, 0.8)
const JOB_BAR_COLOR: Color = Color(0.5, 0.7, 0.4)
const LABEL_COL_WIDTH: float = 180.0


## Draw the population tab content into the parent panel's canvas.
## Returns the y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var members: Array = data.get("members", [])
	var population: int = data.get("population", 0)
	var male_count: int = data.get("male_count", 0)
	var female_count: int = data.get("female_count", 0)

	var bar_max_width: float = maxf(width - LABEL_COL_WIDTH - 80.0, 60.0)

	# ── 1. Demographics ────────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_TAB_POPULATION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	canvas.draw_string(font, Vector2(cx, cy), Locale.trf1("UI_TOTAL_POP_FMT", "n", population), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
	cy += LINE_HEIGHT

	var gender_line: String = (
		Locale.ltr("UI_GENDER_DISTRIBUTION") + ": "
		+ Locale.ltr("UI_MALE") + " " + str(male_count)
		+ " / "
		+ Locale.ltr("UI_FEMALE") + " " + str(female_count)
	)
	canvas.draw_string(font, Vector2(cx, cy), gender_line, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 2. Age Distribution ────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_AGE_DISTRIBUTION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var brackets: Array = GameConfig.SETTLEMENT_PANEL_AGE_BRACKETS
	# Count members per bracket
	var bracket_counts: Array = []
	for bracket in brackets:
		bracket_counts.append(0)

	for member in members:
		var age_years: int = int(floorf(float(member.age) / float(GameConfig.TICKS_PER_YEAR)))
		for i in range(brackets.size()):
			var b = brackets[i]
			if age_years >= b["min"] and age_years <= b["max"]:
				bracket_counts[i] += 1
				break

	for i in range(brackets.size()):
		var label: String = Locale.ltr(brackets[i]["label_key"])
		cy = _draw_bar_row(canvas, font, cx, cy, label, bracket_counts[i], population, bar_max_width, AGE_BAR_COLOR, small_size)

	cy += SECTION_GAP

	# ── 3. Job Distribution ────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_JOB_DISTRIBUTION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var job_counts: Dictionary = {}
	for member in members:
		var job: String = member.job if member.job != null else "none"
		job_counts[job] = job_counts.get(job, 0) + 1

	# Sort by count descending
	var job_keys: Array = job_counts.keys()
	job_keys.sort_custom(func(a, b): return job_counts[a] > job_counts[b])

	for job in job_keys:
		var job_key: String = "JOB_" + job.to_upper()
		var job_label: String
		var localized: String = Locale.ltr(job_key)
		# If the key wasn't found, Locale returns the key itself — fall back to capitalized job name
		if localized == job_key:
			job_label = job.capitalize()
		else:
			job_label = localized
		cy = _draw_bar_row(canvas, font, cx, cy, job_label, job_counts[job], population, bar_max_width, JOB_BAR_COLOR, small_size)

	cy += SECTION_GAP

	# ── 4. Average HEXACO Personality ─────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_PERSONALITY_AVG"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var axis_sums: Dictionary = {"H": 0.0, "E": 0.0, "X": 0.0, "A": 0.0, "C": 0.0, "O": 0.0}
	var personality_count: int = 0
	for member in members:
		if member.personality != null:
			for axis in axis_sums:
				axis_sums[axis] += member.personality.axes.get(axis, 0.5)
			personality_count += 1

	var axis_order: Array = ["H", "E", "X", "A", "C", "O"]
	for axis in axis_order:
		var avg_val: float = axis_sums[axis] / maxf(float(personality_count), 1.0)
		var axis_label: String = Locale.ltr("UI_HEXACO_" + axis)
		var bar_color: Color = HEXACO_COLORS.get(axis, NEUTRAL_COLOR)
		var value_text: String = "%.2f" % avg_val

		# Label left-aligned
		canvas.draw_string(font, Vector2(cx, cy), axis_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT - 4.0

		# Bar
		var bar_w: float = minf(bar_max_width + LABEL_COL_WIDTH - 40.0, 200.0)
		canvas.draw_rect(Rect2(cx, cy, bar_w, BAR_HEIGHT), Color(0.15, 0.15, 0.2))
		canvas.draw_rect(Rect2(cx, cy, bar_w * clampf(avg_val, 0.0, 1.0), BAR_HEIGHT), bar_color)
		canvas.draw_string(font, Vector2(cx + bar_w + 6.0, cy + BAR_HEIGHT - 2.0), value_text, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		cy += BAR_HEIGHT + 4.0

	cy += SECTION_GAP

	# ── 5. Notable Agents ─────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NOTABLE_AGENTS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	if members.size() > 0:
		# Precompute derived values per member
		var best_charisma_entity = null
		var best_charisma_val: float = -1.0
		var best_wisdom_entity = null
		var best_wisdom_val: float = -1.0
		var best_creativity_entity = null
		var best_creativity_val: float = -1.0
		var best_intimidation_entity = null
		var best_intimidation_val: float = -1.0

		for member in members:
			if member.personality == null:
				continue
			var axes: Dictionary = member.personality.axes
			var charisma_val: float = axes.get("X", 0.5)
			var wisdom_val: float = axes.get("O", 0.5)
			var creativity_val: float = axes.get("O", 0.5) * 0.7 + axes.get("X", 0.5) * 0.3
			var intimidation_val: float = axes.get("X", 0.5) * 0.5 + (1.0 - axes.get("A", 0.5)) * 0.5

			if charisma_val > best_charisma_val:
				best_charisma_val = charisma_val
				best_charisma_entity = member
			if wisdom_val > best_wisdom_val:
				best_wisdom_val = wisdom_val
				best_wisdom_entity = member
			if creativity_val > best_creativity_val:
				best_creativity_val = creativity_val
				best_creativity_entity = member
			if intimidation_val > best_intimidation_val:
				best_intimidation_val = intimidation_val
				best_intimidation_entity = member

		var notable_entries: Array = [
			{"label_key": "UI_HIGHEST_CHARISMA", "entity": best_charisma_entity, "value": best_charisma_val},
			{"label_key": "UI_HIGHEST_WISDOM",   "entity": best_wisdom_entity,   "value": best_wisdom_val},
			{"label_key": "UI_HIGHEST_CREATIVITY",    "entity": best_creativity_entity,    "value": best_creativity_val},
			{"label_key": "UI_HIGHEST_INTIMIDATION",  "entity": best_intimidation_entity,  "value": best_intimidation_val},
		]

		for entry in notable_entries:
			var entity_ref = entry["entity"]
			if entity_ref == null:
				continue
			var label_text: String = Locale.ltr(entry["label_key"]) + ": "
			var name_text: String = entity_ref.entity_name
			var val_text: String = " (%.2f)" % entry["value"]
			var full_line: String = label_text + name_text + val_text

			canvas.draw_string(font, Vector2(cx, cy), full_line, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)

			# Click region for entity name portion
			var label_w: float = font.get_string_size(label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
			var name_w: float = font.get_string_size(name_text, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
			click_regions.append({
				"rect": Rect2(cx + label_w, cy - body_size, name_w, body_size + 4),
				"entity_id": entity_ref.id,
			})
			cy += LINE_HEIGHT
	else:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	cy += SECTION_GAP

	return cy


## Draw a horizontal bar chart row with label, filled bar, and count text.
## Returns the y position after the row.
func _draw_bar_row(canvas: Control, font: Font, x: float, y: float, label: String, count: int, total: int, bar_max_width: float, color: Color, font_size: int) -> float:
	# Right-align label in first LABEL_COL_WIDTH pixels
	var label_size: Vector2 = font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size)
	var label_x: float = x + LABEL_COL_WIDTH - label_size.x
	canvas.draw_string(font, Vector2(label_x, y), label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, NEUTRAL_COLOR)

	# Bar
	var bar_x: float = x + LABEL_COL_WIDTH + 10.0
	var pct: float = float(count) / maxf(float(total), 1.0)
	var bar_w: float = bar_max_width * pct
	canvas.draw_rect(Rect2(bar_x, y - 10.0, bar_max_width, BAR_HEIGHT), Color(0.15, 0.15, 0.2))
	canvas.draw_rect(Rect2(bar_x, y - 10.0, bar_w, BAR_HEIGHT), color)

	# Count + percentage text
	var count_text: String = str(count) + " (" + str(roundi(pct * 100.0)) + "%)"
	canvas.draw_string(font, Vector2(bar_x + bar_max_width + 8.0, y), count_text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, NEUTRAL_COLOR)

	return y + LINE_HEIGHT
