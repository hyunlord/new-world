extends RefCounted

const CivTechState = preload("res://scripts/core/tech/civ_tech_state.gd")
const TechState = preload("res://scripts/core/tech/tech_state.gd")

const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const KNOWN_STABLE_COLOR: Color = Color(0.4, 0.8, 0.4)
const KNOWN_LOW_COLOR: Color = Color(0.9, 0.7, 0.2)
const FORGOTTEN_COLOR: Color = Color(0.7, 0.5, 0.2)
const UNKNOWN_COLOR: Color = Color(0.5, 0.5, 0.5)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const INDENT: float = 20.0

## Track which tech practitioner lists are expanded.
var _expanded_techs: Dictionary = {}  # {tech_id: bool}
## Track toggle button rects for click detection.
var _toggle_rects: Array = []  # [{rect: Rect2, tech_id: String}]

const ERAS: Array = ["stone_age", "tribal", "bronze_age"]


## Draw the Technology tab content onto canvas. Returns final y position.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array) -> float:
	_toggle_rects.clear()

	var settlement = data.get("settlement")
	if settlement == null:
		return cy

	var ttm = data.get("tech_tree_manager")
	if ttm == null:
		return cy

	var members: Array = data.get("members", [])
	var population: int = data.get("population", 0)
	var entity_manager = data.get("entity_manager")
	var tech_states: Dictionary = {}
	if "tech_states" in settlement:
		tech_states = settlement.tech_states

	for era in ERAS:
		cy = _draw_era_section(canvas, data, font, cx, cy, width, click_regions,
				era, ttm, tech_states, members, population, entity_manager)

	return cy


## Draw one era section. Returns updated cy.
func _draw_era_section(canvas: Control, _data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array,
		era: String, ttm, tech_states: Dictionary, members: Array, population: int, entity_manager) -> float:

	# Collect all techs for this era, sorted by tier then id
	var all_ids: Array = ttm.get_all_ids()
	var era_techs: Array = []
	for tid in all_ids:
		var def: Dictionary = ttm.get_def(tid)
		if def.get("era", "") == era:
			era_techs.append({"id": tid, "def": def, "tier": def.get("tier", 0)})
	era_techs.sort_custom(func(a, b): return a.tier < b.tier if a.tier != b.tier else a.id < b.id)

	if era_techs.is_empty():
		return cy

	# Era header
	cy += SECTION_GAP
	var era_label: String = "── " + Locale.ltr("ERA_" + era.to_upper()) + " " + Locale.ltr("UI_SETTLEMENT_TECHS") + " ──"
	canvas.draw_string(font, Vector2(cx, cy + 14.0), era_label,
			HORIZONTAL_ALIGNMENT_LEFT, width,
			GameConfig.get_font_size("popup_heading"), SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT + 4.0

	# Check if all techs in this era are unknown (not in tech_states)
	var any_encountered: bool = false
	for entry in era_techs:
		if tech_states.has(entry.id):
			any_encountered = true
			break

	if not any_encountered:
		canvas.draw_string(font, Vector2(cx + INDENT, cy + 12.0), Locale.ltr("UI_TECH_ALL_UNDISCOVERED"),
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT,
				GameConfig.get_font_size("popup_body"), UNKNOWN_COLOR)
		cy += LINE_HEIGHT
		return cy

	for entry in era_techs:
		var tech_id: String = entry.id
		var tech_def: Dictionary = entry.def
		cy = _draw_tech_entry(canvas, font, cx, cy, width, click_regions,
				tech_id, tech_def, tech_states, members, population, entity_manager)
		cy += 4.0

	return cy


## Draw a single tech entry. Returns updated cy.
func _draw_tech_entry(canvas: Control, font: Font, cx: float, cy: float, width: float, click_regions: Array,
		tech_id: String, tech_def: Dictionary, tech_states: Dictionary, members: Array, population: int, entity_manager) -> float:

	var cts: Dictionary = tech_states.get(tech_id, {})
	var state_str: String = cts.get("state", "unknown")

	var state_enum: int = TechState.STATE_FROM_STRING.get(state_str, TechState.State.UNKNOWN)
	var tech_name: String = Locale.ltr(tech_id)

	if TechState.is_known(state_enum):
		cy = _draw_known_tech(canvas, font, cx, cy, width, click_regions,
				tech_id, tech_name, tech_def, cts, state_enum, members, population, entity_manager)
	elif TechState.is_forgotten(state_enum):
		cy = _draw_forgotten_tech(canvas, font, cx, cy, width, click_regions,
				tech_id, tech_name, tech_def, cts, state_enum, entity_manager)
	else:
		cy = _draw_unknown_tech(canvas, font, cx, cy, width,
				tech_id, tech_name, tech_def, members, population)

	return cy


## Draw a known tech (known_low or known_stable). Returns updated cy.
func _draw_known_tech(canvas: Control, font: Font, cx: float, cy: float, width: float, click_regions: Array,
		tech_id: String, tech_name: String, tech_def: Dictionary, cts: Dictionary, state_enum: int,
		members: Array, population: int, entity_manager) -> float:

	var state_str: String = cts.get("state", "unknown")
	var practitioner_count: int = cts.get("practitioner_count", 0)
	var phase: String = cts.get("adoption_curve_phase", "innovator")
	var discoverer_id: int = cts.get("discoverer_id", -1)
	var discovered_tick: int = cts.get("discovered_tick", 0)

	var state_color: Color = KNOWN_STABLE_COLOR if state_enum == TechState.State.KNOWN_STABLE else KNOWN_LOW_COLOR

	# Tech name + state label on same line
	var state_key: String = "TECH_STATE_" + state_str.to_upper()
	var state_label: String = Locale.ltr(state_key)
	canvas.draw_string(font, Vector2(cx + INDENT, cy + 14.0), tech_name,
			HORIZONTAL_ALIGNMENT_LEFT, -1,
			GameConfig.get_font_size("popup_body"), Color.WHITE)
	var name_w: float = font.get_string_size(tech_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
	canvas.draw_string(font, Vector2(cx + INDENT + name_w + 8.0, cy + 14.0), "[" + state_label + "]",
			HORIZONTAL_ALIGNMENT_LEFT, -1,
			GameConfig.get_font_size("popup_small"), state_color)
	cy += LINE_HEIGHT

	# Practitioner info
	var phase_label: String = Locale.ltr("LABEL_ADOPTION_" + phase.to_upper())
	var prac_text: String = Locale.trf("UI_PRACTITIONERS_FMT", {
		"count": practitioner_count,
		"total": population,
		"phase": phase_label,
	})
	canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), prac_text,
			HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.0,
			GameConfig.get_font_size("popup_small"), NEUTRAL_COLOR)
	cy += LINE_HEIGHT

	# Needs more practitioners warning
	if state_enum == TechState.State.KNOWN_LOW:
		var maintenance: Dictionary = tech_def.get("maintenance", {})
		var min_prac: int = maintenance.get("min_practitioners", 3)
		if practitioner_count < min_prac:
			var needed: int = min_prac - practitioner_count
			var needs_text: String = Locale.trf("UI_NEEDS_MORE_FMT", {"count": needed})
			canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), needs_text,
					HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.0,
					GameConfig.get_font_size("popup_small"), KNOWN_LOW_COLOR)
			cy += LINE_HEIGHT

	# Discoverer
	if discoverer_id > 0 and entity_manager != null:
		var discoverer = entity_manager.get_entity(discoverer_id)
		if discoverer != null:
			var disc_name: String = discoverer.entity_name
			var disc_text: String = Locale.trf("UI_DISCOVERER_FMT", {"name": disc_name, "tick": discovered_tick})
			var disc_rect := Rect2(cx + INDENT * 2.0, cy, width - INDENT * 2.0, LINE_HEIGHT)
			canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), disc_text,
					HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.0,
					GameConfig.get_font_size("popup_small"), Color(0.6, 0.8, 1.0))
			click_regions.append({"rect": disc_rect, "entity_id": discoverer_id})
			cy += LINE_HEIGHT

	# Effects from tech_def.effects dict
	var effects: Dictionary = tech_def.get("effects", {})
	if not effects.is_empty():
		canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), Locale.ltr("UI_TECH_EFFECTS") + ":",
				HORIZONTAL_ALIGNMENT_LEFT, -1,
				GameConfig.get_font_size("popup_small"), NEUTRAL_COLOR)
		cy += LINE_HEIGHT
		for effect_key in effects.keys():
			var effect_val = effects[effect_key]
			var val_f: float = float(effect_val)
			var sign_str: String = "+" if val_f >= 0.0 else ""
			var effect_line: String = "  " + effect_key + ": " + sign_str + ("%.2f" % val_f)
			var eff_color: Color = POSITIVE_COLOR if val_f >= 0.0 else NEGATIVE_COLOR
			canvas.draw_string(font, Vector2(cx + INDENT * 2.5, cy + 12.0), effect_line,
					HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.5,
					GameConfig.get_font_size("popup_small"), eff_color)
			cy += LINE_HEIGHT

	# Toggle button for practitioner list
	var toggle_label: String = Locale.ltr("UI_HIDE_PRACTITIONERS") if _expanded_techs.get(tech_id, false) else Locale.ltr("UI_SHOW_PRACTITIONERS")
	var toggle_rect := Rect2(cx + INDENT * 2.0, cy, width - INDENT * 2.0, LINE_HEIGHT)
	canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), "[" + toggle_label + "]",
			HORIZONTAL_ALIGNMENT_LEFT, -1,
			GameConfig.get_font_size("popup_small"), Color(0.5, 0.7, 1.0))
	_toggle_rects.append({"rect": toggle_rect, "tech_id": tech_id})
	cy += LINE_HEIGHT

	# Expanded practitioner list
	if _expanded_techs.get(tech_id, false):
		cy = _draw_practitioner_list(canvas, font, cx, cy, width, click_regions,
				tech_id, tech_def, members)

	return cy


## Draw the practitioner list for a known tech. Returns updated cy.
func _draw_practitioner_list(canvas: Control, font: Font, cx: float, cy: float, width: float, click_regions: Array,
		tech_id: String, tech_def: Dictionary, members: Array) -> float:

	# Find primary skill from discovery.required_skills
	var discovery: Dictionary = tech_def.get("discovery", {})
	var required_skills: Dictionary = discovery.get("required_skills", {})
	# Also check unlocks.skills for practitioners
	var unlocks: Dictionary = tech_def.get("unlocks", {})
	var unlocked_skills: Array = unlocks.get("skills", [])

	# Collect skill ids to check: required_skills keys + first unlocked skill
	var check_skill_ids: Array = []
	for sk in required_skills.keys():
		if not sk in check_skill_ids:
			check_skill_ids.append(sk)
	for sk in unlocked_skills:
		if not sk in check_skill_ids:
			check_skill_ids.append(sk)

	var practitioners: Array = []
	if check_skill_ids.is_empty():
		# Fallback: use tech_id as skill key (some techs name their skill after themselves)
		for entity in members:
			if entity.skill_levels.has(tech_id) and entity.skill_levels[tech_id] > 0:
				practitioners.append({"entity": entity, "skill_id": tech_id, "level": entity.skill_levels[tech_id]})
	else:
		for entity in members:
			var best_skill_id: String = ""
			var best_level: int = 0
			for sk_id in check_skill_ids:
				var lvl: int = entity.skill_levels.get(sk_id, 0)
				if lvl > best_level:
					best_level = lvl
					best_skill_id = sk_id
			if best_level > 0:
				practitioners.append({"entity": entity, "skill_id": best_skill_id, "level": best_level})

	if practitioners.is_empty():
		canvas.draw_string(font, Vector2(cx + INDENT * 3.0, cy + 12.0), Locale.ltr("UI_TECH_UNDISCOVERED"),
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 3.0,
				GameConfig.get_font_size("popup_small"), UNKNOWN_COLOR)
		cy += LINE_HEIGHT
		return cy

	var max_shown: int = GameConfig.SETTLEMENT_PANEL_MAX_PRACTITIONERS
	var shown: int = 0
	for prac in practitioners:
		if shown >= max_shown:
			break
		var entity = prac.entity
		var lvl: int = prac.level
		var is_teaching: bool = entity.teaching_target_id > -1
		var teach_suffix: String = " [T]" if is_teaching else ""
		var prac_line: String = "· " + entity.entity_name + " (Lv." + str(lvl) + ")" + teach_suffix
		var prac_rect := Rect2(cx + INDENT * 3.0, cy, width - INDENT * 3.0, LINE_HEIGHT)
		var prac_color: Color = Color(0.6, 0.9, 0.6) if is_teaching else NEUTRAL_COLOR
		canvas.draw_string(font, Vector2(cx + INDENT * 3.0, cy + 12.0), prac_line,
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 3.0,
				GameConfig.get_font_size("popup_small"), prac_color)
		click_regions.append({"rect": prac_rect, "entity_id": entity.id})
		cy += LINE_HEIGHT
		shown += 1

	var remaining: int = practitioners.size() - shown
	if remaining > 0:
		var more_text: String = Locale.trf("UI_AND_N_MORE", {"n": remaining})
		canvas.draw_string(font, Vector2(cx + INDENT * 3.0, cy + 12.0), more_text,
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 3.0,
				GameConfig.get_font_size("popup_small"), UNKNOWN_COLOR)
		cy += LINE_HEIGHT

	return cy


## Draw a forgotten tech (forgotten_recent or forgotten_long). Returns updated cy.
func _draw_forgotten_tech(canvas: Control, font: Font, cx: float, cy: float, width: float, click_regions: Array,
		_tech_id: String, tech_name: String, _tech_def: Dictionary, cts: Dictionary, state_enum: int, entity_manager) -> float:

	var state_str: String = cts.get("state", "forgotten_recent")
	var cultural_memory: float = cts.get("cultural_memory", 0.0)
	var discoverer_id: int = cts.get("discoverer_id", -1)
	var discovered_tick: int = cts.get("discovered_tick", 0)

	var forgotten_color: Color = FORGOTTEN_COLOR if state_enum == TechState.State.FORGOTTEN_RECENT else Color(0.5, 0.4, 0.4)
	var state_label: String = Locale.ltr("TECH_STATE_" + state_str.to_upper())
	var line: String = tech_name + " — [" + state_label + "]"
	canvas.draw_string(font, Vector2(cx + INDENT, cy + 14.0), line,
			HORIZONTAL_ALIGNMENT_LEFT, width - INDENT,
			GameConfig.get_font_size("popup_body"), forgotten_color)
	cy += LINE_HEIGHT

	# Cultural memory bar
	var memory_label: String = Locale.ltr("UI_TECH_MEMORY") + ": "
	canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), memory_label,
			HORIZONTAL_ALIGNMENT_LEFT, -1,
			GameConfig.get_font_size("popup_small"), NEUTRAL_COLOR)
	var label_w: float = font.get_string_size(memory_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
	var bar_x: float = cx + INDENT * 2.0 + label_w
	var bar_w: float = minf(80.0, width - INDENT * 2.0 - label_w - 4.0)
	var bar_h: float = 8.0
	var bar_y: float = cy + 4.0
	canvas.draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	var fill_w: float = bar_w * clampf(cultural_memory, 0.0, 1.0)
	if fill_w > 0.0:
		canvas.draw_rect(Rect2(bar_x, bar_y, fill_w, bar_h), forgotten_color)
	cy += LINE_HEIGHT

	# Discoverer
	if discoverer_id > 0 and entity_manager != null:
		var discoverer = entity_manager.get_entity(discoverer_id)
		if discoverer != null:
			var disc_text: String = Locale.trf("UI_DISCOVERER_FMT", {"name": discoverer.entity_name, "tick": discovered_tick})
			var disc_rect := Rect2(cx + INDENT * 2.0, cy, width - INDENT * 2.0, LINE_HEIGHT)
			canvas.draw_string(font, Vector2(cx + INDENT * 2.0, cy + 12.0), disc_text,
					HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.0,
					GameConfig.get_font_size("popup_small"), Color(0.6, 0.8, 1.0))
			click_regions.append({"rect": disc_rect, "entity_id": discoverer_id})
			cy += LINE_HEIGHT

	return cy


## Draw an unknown (undiscovered) tech. Returns updated cy.
func _draw_unknown_tech(canvas: Control, font: Font, cx: float, cy: float, width: float,
		tech_id: String, tech_name: String, tech_def: Dictionary, members: Array, population: int) -> float:

	var unknown_label: String = "❌ " + tech_name + " — " + Locale.ltr("UI_TECH_UNDISCOVERED")
	canvas.draw_string(font, Vector2(cx + INDENT, cy + 14.0), unknown_label,
			HORIZONTAL_ALIGNMENT_LEFT, width - INDENT,
			GameConfig.get_font_size("popup_body"), UNKNOWN_COLOR)
	cy += LINE_HEIGHT

	# Discovery conditions
	var discovery: Dictionary = tech_def.get("discovery", {})

	# Required skills
	var required_skills: Dictionary = discovery.get("required_skills", {})
	for skill_id in required_skills.keys():
		var req_level: int = required_skills[skill_id]
		var best_level: int = 0
		for entity in members:
			var lvl: int = entity.skill_levels.get(skill_id, 0)
			if lvl > best_level:
				best_level = lvl
		var met: bool = best_level >= req_level
		var skill_color: Color = POSITIVE_COLOR if met else NEGATIVE_COLOR
		var skill_label: String = Locale.ltr(skill_id)
		var skill_line: String = "  " + skill_label + ": " + str(best_level) + "/" + str(req_level)
		canvas.draw_string(font, Vector2(cx + INDENT * 2.5, cy + 12.0), skill_line,
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.5,
				GameConfig.get_font_size("popup_small"), skill_color)
		cy += LINE_HEIGHT

	# Required population
	var req_pop: int = discovery.get("required_population", 0)
	if req_pop > 0:
		var pop_met: bool = population >= req_pop
		var pop_color: Color = POSITIVE_COLOR if pop_met else NEGATIVE_COLOR
		var pop_line: String = "  " + Locale.trf("UI_STAT_POP_FMT", {"n": population}) + " / " + str(req_pop) + " req."
		canvas.draw_string(font, Vector2(cx + INDENT * 2.5, cy + 12.0), pop_line,
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.5,
				GameConfig.get_font_size("popup_small"), pop_color)
		cy += LINE_HEIGHT

	# Required techs (prereqs)
	var prereq_logic: Dictionary = tech_def.get("prereq_logic", {})
	var all_of: Array = prereq_logic.get("all_of", [])
	if all_of.is_empty() and tech_def.has("prerequisites"):
		all_of = tech_def.get("prerequisites", [])
	for prereq_id in all_of:
		var prereq_name: String = Locale.ltr(prereq_id)
		var prereq_line: String = "  → " + prereq_name
		canvas.draw_string(font, Vector2(cx + INDENT * 2.5, cy + 12.0), prereq_line,
				HORIZONTAL_ALIGNMENT_LEFT, width - INDENT * 2.5,
				GameConfig.get_font_size("popup_small"), NEGATIVE_COLOR)
		cy += LINE_HEIGHT

	return cy


## Handle a click at pos. Returns true if a toggle was hit and state changed.
func handle_click(pos: Vector2) -> bool:
	for tr in _toggle_rects:
		if tr["rect"].has_point(pos):
			var tid: String = tr["tech_id"]
			_expanded_techs[tid] = not _expanded_techs.get(tid, false)
			return true
	return false
