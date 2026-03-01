extends RefCounted

const CivTechState = preload("res://scripts/core/tech/civ_tech_state.gd")

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const SEPARATOR_COLOR: Color = Color(0.3, 0.3, 0.4)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const BAR_HEIGHT: float = 12.0
const BAR_WIDTH: float = 200.0
const TAB_ACTIVE_COLOR: Color = Color(0.3, 0.6, 0.9)


## Draw the overview tab content into the parent panel's canvas.
## Returns the y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, _width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var settlement = data.get("settlement", null)
	if settlement == null:
		return cy

	# ── 1. Leader Section ──────────────────────────────────────────────────
	var leader = data.get("leader", null)
	if leader != null:
		var charisma_raw: float = 0.5
		if leader.personality != null:
			charisma_raw = leader.personality.axes.get("X", 0.5)
		var charisma_fmt: String = "%.2f" % charisma_raw
		var leader_label: String = (
			"♛ " + Locale.ltr("UI_LEADER") + ": " + leader.entity_name
			+ " (" + Locale.trf1("UI_CHARISMA_FMT", "value", charisma_fmt) + ")"
		)
		canvas.draw_string(font, Vector2(cx, cy), leader_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, SECTION_HEADER_COLOR)
		# click region for leader name — measure the name portion width
		var prefix: String = "♛ " + Locale.ltr("UI_LEADER") + ": "
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		var name_w: float = font.get_string_size(leader.entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		click_regions.append({
			"rect": Rect2(cx + prefix_w, cy - body_size, name_w, body_size + 4),
			"entity_id": leader.id,
		})
	else:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 2. Era Section ─────────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_ERA_SECTION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var tech_era: String = settlement.tech_era
	var era_color: Color
	match tech_era:
		"stone_age":
			era_color = Color(0.6, 0.6, 0.6)
		"tribal":
			era_color = Color(0.6, 0.8, 0.3)
		"bronze_age":
			era_color = Color(0.8, 0.6, 0.2)
		_:
			era_color = NEUTRAL_COLOR

	var era_key: String = "ERA_" + tech_era.to_upper()
	var era_badge: String = "[" + Locale.ltr(era_key) + "]"
	canvas.draw_string(font, Vector2(cx, cy), era_badge, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, era_color)
	cy += LINE_HEIGHT

	# Era progress bar
	var known_count: int = settlement.get_known_techs().size()
	var required: int
	match tech_era:
		"stone_age":
			required = GameConfig.TECH_ERA_TRIBAL_COUNT
		"tribal":
			required = GameConfig.TECH_ERA_BRONZE_AGE_COUNT
		_:
			required = GameConfig.TECH_ERA_BRONZE_AGE_COUNT

	var progress_val: float = clampf(float(known_count) / float(maxi(required, 1)), 0.0, 1.0)
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, progress_val, TAB_ACTIVE_COLOR)
	var progress_label: String = Locale.trf("UI_ERA_PROGRESS_FMT", {"era": era_badge, "count": known_count, "required": required})
	canvas.draw_string(font, Vector2(cx + BAR_WIDTH + 6.0, cy + BAR_HEIGHT - 2.0), progress_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += BAR_HEIGHT + SECTION_GAP

	# ── 3. Population Summary ──────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_POP_SUMMARY"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT
	canvas.draw_string(font, Vector2(cx, cy), Locale.trf1("UI_TOTAL_POP_FMT", "n", data.get("population", 0)), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
	cy += LINE_HEIGHT
	canvas.draw_string(
		font,
		Vector2(cx, cy),
		Locale.trf3(
			"UI_POP_SUMMARY_FMT",
			"adults",
			data.get("adults", 0),
			"children",
			data.get("children", 0),
			"elders",
			data.get("elders", 0)
		),
		HORIZONTAL_ALIGNMENT_LEFT,
		-1,
		body_size,
		NEUTRAL_COLOR
	)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 4. Active Tech Modifiers ───────────────────────────────────────────
	var tech_tree_manager = data.get("tech_tree_manager", null)
	if tech_tree_manager != null and settlement.tech_states.size() > 0:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_ACTIVE_MODIFIERS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT
		var any_modifier_drawn: bool = false
		for tech_id in settlement.tech_states:
			var cts = settlement.tech_states[tech_id]
			if not CivTechState.is_active(cts):
				continue
			var tech_def: Dictionary = tech_tree_manager.get_def(tech_id)
			if tech_def.is_empty():
				continue
			var modifiers = tech_def.get("modifiers", [])
			for mod in modifiers:
				var target_name: String = mod.get("target", "")
				var target_label: String = Locale.ltr("MOD_TARGET_" + target_name.to_upper())
				var mod_value = mod.get("value", 1.0)
				var mod_type: String = mod.get("type", "multiplier")
				var mod_text: String
				if mod_type == "multiplier":
					mod_text = target_label + " ×" + "%.2f" % float(mod_value)
				else:
					mod_text = target_label + " +" + "%.2f" % float(mod_value)
				canvas.draw_string(font, Vector2(cx + 8.0, cy), mod_text, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, POSITIVE_COLOR)
				cy += LINE_HEIGHT
				any_modifier_drawn = true
		if not any_modifier_drawn:
			cy -= LINE_HEIGHT  # remove header gap if nothing drawn, re-add section gap below
		cy += SECTION_GAP

	# ── 5. Capabilities ────────────────────────────────────────────────────
	if tech_tree_manager != null:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_CAPABILITIES"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT
		var cap_drawn: bool = false
		for tech_id in settlement.tech_states:
			var cts = settlement.tech_states[tech_id]
			var tech_def: Dictionary = tech_tree_manager.get_def(tech_id)
			if tech_def.is_empty():
				continue
			var capabilities = tech_def.get("capabilities", [])
			for cap in capabilities:
				var cap_name: String = cap if cap is String else cap.get("name", str(cap))
				var is_known: bool = CivTechState.is_active(cts)
				if is_known:
					canvas.draw_string(font, Vector2(cx + 8.0, cy), "✓ " + cap_name, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, POSITIVE_COLOR)
				else:
					canvas.draw_string(font, Vector2(cx + 8.0, cy), "✗ " + cap_name, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
				cy += LINE_HEIGHT
				cap_drawn = true
		if not cap_drawn:
			cy -= LINE_HEIGHT
		cy += SECTION_GAP

	# ── 6. Settlement State ────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETTLEMENT_STATE"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var avg_stress: float = data.get("avg_stress", 0.0)
	var avg_happiness: float = data.get("avg_happiness", 0.5)
	var stability: float = clampf(1.0 - avg_stress, 0.0, 1.0)

	# Stability bar
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_STABILITY"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT - 4.0
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, stability, POSITIVE_COLOR)
	cy += BAR_HEIGHT + 4.0

	# Happiness bar
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_AVG_HAPPINESS"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT - 4.0
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, avg_happiness, Color(0.9, 0.75, 0.2))
	cy += BAR_HEIGHT + 4.0

	# Stress bar
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_STRESS"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT - 4.0
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, avg_stress, NEGATIVE_COLOR, Color(0.25, 0.1, 0.1))
	cy += BAR_HEIGHT + SECTION_GAP

	return cy


## Draw a filled progress bar with background.
func _draw_bar(canvas: Control, x: float, y: float, w: float, h: float, value: float, fill_color: Color, bg_color: Color = Color(0.15, 0.15, 0.2)) -> void:
	canvas.draw_rect(Rect2(x, y, w, h), bg_color)
	canvas.draw_rect(Rect2(x, y, w * clampf(value, 0.0, 1.0), h), fill_color)
