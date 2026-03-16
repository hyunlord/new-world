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

	# ── 1. Alerts Section ──────────────────────────────────────────────────
	var alerts: Array[Dictionary] = []
	var stockpile_food: float = float(data.get("stockpile_food", 0.0))
	if stockpile_food < 20.0:
		alerts.append({
			"text": Locale.ltr("ALERT_SETTLEMENT_FOOD"),
			"color": NEGATIVE_COLOR,
		})
	var at_risk_tech_names: PackedStringArray = _at_risk_tech_names(settlement)
	if not at_risk_tech_names.is_empty():
		alerts.append({
			"text": "%s: %s" % [
				Locale.ltr("ALERT_SETTLEMENT_TECH_RISK"),
				", ".join(at_risk_tech_names),
			],
			"color": Color(0.9, 0.7, 0.2),
		})
	if not alerts.is_empty():
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("PANEL_OVERVIEW_ALERTS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT
		for alert: Dictionary in alerts:
			canvas.draw_string(
				font,
				Vector2(cx + 8.0, cy),
				"⚠ " + str(alert.get("text", "")),
				HORIZONTAL_ALIGNMENT_LEFT,
				-1,
				body_size,
				alert.get("color", NEGATIVE_COLOR)
			)
			cy += LINE_HEIGHT
		cy += SECTION_GAP

	# ── 2. Leader Section ──────────────────────────────────────────────────
	var leader = data.get("leader", null)
	if leader != null:
		var charisma_raw: float = 0.5
		var leader_axes: Dictionary = _entity_axes(leader)
		if not leader_axes.is_empty():
			charisma_raw = float(leader_axes.get("X", 0.5))
		var charisma_fmt: String = "%.2f" % charisma_raw
		var leader_label: String = (
			"♛ " + Locale.ltr("UI_LEADER") + ": " + _entity_name(leader)
			+ " (" + Locale.trf1("UI_CHARISMA_FMT", "value", charisma_fmt) + ")"
		)
		canvas.draw_string(font, Vector2(cx, cy), leader_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, SECTION_HEADER_COLOR)
		# click region for leader name — measure the name portion width
		var prefix: String = "♛ " + Locale.ltr("UI_LEADER") + ": "
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		var leader_name: String = _entity_name(leader)
		var name_w: float = font.get_string_size(leader_name, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		click_regions.append({
			"rect": Rect2(cx + prefix_w, cy - body_size, name_w, body_size + 4),
			"entity_id": _entity_id(leader),
		})
	else:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 3. Era Section ─────────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_ERA_SECTION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var tech_era: String = _settlement_tech_era(settlement)
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
	var known_count: int = _settlement_known_techs(settlement).size()
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
	var progress_label: String = Locale.trf3(
		"UI_ERA_PROGRESS_FMT",
		"era",
		era_badge,
		"count",
		known_count,
		"required",
		required
	)
	canvas.draw_string(font, Vector2(cx + BAR_WIDTH + 6.0, cy + BAR_HEIGHT - 2.0), progress_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += BAR_HEIGHT + SECTION_GAP

	# ── 4. Population Summary ──────────────────────────────────────────────
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

	# ── 5. Active Tech Modifiers ───────────────────────────────────────────
	var tech_tree_manager = data.get("tech_tree_manager", null)
	var settlement_tech_states: Dictionary = _settlement_tech_states(settlement)
	if tech_tree_manager != null and settlement_tech_states.size() > 0:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_ACTIVE_MODIFIERS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT
		var any_modifier_drawn: bool = false
		for tech_id in settlement_tech_states:
			var cts = settlement_tech_states[tech_id]
			if not CivTechState.is_active(cts):
				continue
			var tech_def: Dictionary = tech_tree_manager.get_def(tech_id)
			if tech_def.is_empty():
				continue
			for mod_text: String in _format_tech_modifiers(tech_def):
				canvas.draw_string(font, Vector2(cx + 8.0, cy), mod_text, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, POSITIVE_COLOR)
				cy += LINE_HEIGHT
				any_modifier_drawn = true
		if not any_modifier_drawn:
			cy -= LINE_HEIGHT  # remove header gap if nothing drawn, re-add section gap below
		cy += SECTION_GAP

	# ── 6. Capabilities ────────────────────────────────────────────────────
	if tech_tree_manager != null:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_CAPABILITIES"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
		cy += LINE_HEIGHT
		var cap_drawn: bool = false
		for tech_id in settlement_tech_states:
			var cts = settlement_tech_states[tech_id]
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

	# ── 7. Settlement State ────────────────────────────────────────────────
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

	# ── 8. Happiness Factors ───────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETT_HAPPINESS_FACTORS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT
	canvas.draw_string(
		font,
		Vector2(cx + 8.0, cy),
		"%s: +12  ·  %s: +8  ·  %s: -5" % [
			Locale.ltr("UI_RES_FOOD_SHORT"),
			Locale.ltr("UI_SETT_SHELTER"),
			Locale.ltr("UI_SETT_SAFETY"),
		],
		HORIZONTAL_ALIGNMENT_LEFT,
		-1,
		small_size,
		NEUTRAL_COLOR
	)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 9. Production / Consumption ───────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETT_PRODUCTION_FLOW"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT
	canvas.draw_string(
		font,
		Vector2(cx + 8.0, cy),
		"▲ %s  %s +3.2/%s  ·  %s +1.5/%s" % [
			Locale.ltr("UI_SETT_PRODUCE"),
			Locale.ltr("UI_RES_FOOD_SHORT"),
			Locale.ltr("UI_DAY"),
			Locale.ltr("UI_RES_WOOD_SHORT"),
			Locale.ltr("UI_DAY"),
		],
		HORIZONTAL_ALIGNMENT_LEFT,
		-1,
		small_size,
		POSITIVE_COLOR
	)
	cy += LINE_HEIGHT
	canvas.draw_string(
		font,
		Vector2(cx + 8.0, cy),
		"▼ %s  %s -2.8/%s" % [
			Locale.ltr("UI_SETT_CONSUME"),
			Locale.ltr("UI_RES_FOOD_SHORT"),
			Locale.ltr("UI_DAY"),
		],
		HORIZONTAL_ALIGNMENT_LEFT,
		-1,
		small_size,
		NEGATIVE_COLOR
	)
	cy += LINE_HEIGHT + SECTION_GAP

	# ── 10. Resource Storage ──────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_SETT_STORAGE_LOCATION"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT
	for storage_line: String in [
		"📦 " + Locale.ltr("UI_SETT_IN_STOCKPILE"),
		"👤 " + Locale.ltr("UI_SETT_ON_AGENTS"),
		"🌿 " + Locale.ltr("UI_SETT_ON_GROUND"),
	]:
		canvas.draw_string(font, Vector2(cx + 8.0, cy), storage_line, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT
	cy += SECTION_GAP

	# ── 11. Population Trend Placeholder ──────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), "━━━ %s ━━━" % Locale.ltr("UI_STATS_POP_GRAPH"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, SEPARATOR_COLOR)
	cy += LINE_HEIGHT

	return cy


## Draw a filled progress bar with background.
func _draw_bar(canvas: Control, x: float, y: float, w: float, h: float, value: float, fill_color: Color, bg_color: Color = Color(0.15, 0.15, 0.2)) -> void:
	canvas.draw_rect(Rect2(x, y, w, h), bg_color)
	canvas.draw_rect(Rect2(x, y, w * clampf(value, 0.0, 1.0), h), fill_color)


func _entity_id(entity: Variant) -> int:
	if entity is Dictionary:
		return int(entity.get("id", -1))
	if entity == null:
		return -1
	return int(entity.id)


func _entity_name(entity: Variant) -> String:
	if entity is Dictionary:
		return str(entity.get("entity_name", entity.get("name", "?")))
	if entity == null:
		return "?"
	return str(entity.entity_name)


func _entity_axes(entity: Variant) -> Dictionary:
	if entity is Dictionary:
		var personality: Variant = entity.get("personality", {})
		if personality is Dictionary:
			var axes: Variant = personality.get("axes", {})
			if axes is Dictionary:
				return axes
		return {}
	if entity == null or entity.personality == null:
		return {}
	return entity.personality.axes


func _settlement_tech_era(settlement: Variant) -> String:
	if settlement is Dictionary:
		return str(settlement.get("tech_era", "stone_age"))
	if settlement == null:
		return "stone_age"
	return str(settlement.tech_era)


func _settlement_tech_states(settlement: Variant) -> Dictionary:
	if settlement is Dictionary:
		var tech_states: Variant = settlement.get("tech_states", {})
		return tech_states if tech_states is Dictionary else {}
	if settlement == null:
		return {}
	return settlement.tech_states


func _settlement_known_techs(settlement: Variant) -> Array:
	if settlement != null and not (settlement is Dictionary) and settlement.has_method("get_known_techs"):
		return settlement.get_known_techs()
	var tech_states: Dictionary = _settlement_tech_states(settlement)
	var known: Array = []
	for tech_id in tech_states.keys():
		var cts: Dictionary = tech_states[tech_id]
		if CivTechState.is_active(cts):
			known.append(tech_id)
	return known


func _at_risk_tech_names(settlement: Variant) -> PackedStringArray:
	var names: PackedStringArray = PackedStringArray()
	var tech_states: Dictionary = _settlement_tech_states(settlement)
	for tech_id_variant: Variant in tech_states.keys():
		var tech_id: String = str(tech_id_variant)
		var cts: Dictionary = tech_states[tech_id]
		if not CivTechState.is_active(cts):
			continue
		if int(cts.get("practitioner_count", 0)) != 1:
			continue
		if Locale.has_key(tech_id):
			names.append(Locale.ltr(tech_id))
		else:
			names.append(tech_id)
	return names


func _format_tech_modifiers(tech_def: Dictionary) -> PackedStringArray:
	var lines: PackedStringArray = PackedStringArray()
	var direct_modifiers: Variant = tech_def.get("modifiers", [])
	if direct_modifiers is Array:
		for mod_raw: Variant in direct_modifiers:
			if not (mod_raw is Dictionary):
				continue
			var mod: Dictionary = mod_raw
			var target_name: String = str(mod.get("target", ""))
			if target_name.is_empty():
				continue
			var target_label: String = _modifier_target_label(target_name)
			var mod_value: float = float(mod.get("value", 1.0))
			var mod_type: String = str(mod.get("type", "multiplier"))
			if mod_type == "multiplier":
				lines.append("%s ×%.2f" % [target_label, mod_value])
			else:
				lines.append("%s +%.2f" % [target_label, mod_value])
	if not lines.is_empty():
		return lines
	var effects_raw: Variant = tech_def.get("effects", {})
	if effects_raw is Dictionary:
		var effects: Dictionary = effects_raw
		for target_variant: Variant in effects.keys():
			var target_name: String = str(target_variant)
			lines.append(
				"%s %+.2f" % [
					_modifier_target_label(target_name),
					float(effects[target_name]),
				]
			)
	return lines


func _modifier_target_label(target_name: String) -> String:
	var locale_key: String = "MOD_TARGET_" + target_name.to_upper()
	if Locale.has_key(locale_key):
		return Locale.ltr(locale_key)
	var words: PackedStringArray = PackedStringArray()
	for word: String in target_name.split("_", false):
		words.append(word.capitalize())
	return " ".join(words)
