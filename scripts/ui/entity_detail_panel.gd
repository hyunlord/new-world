class_name EntityDetailPanel
extends Control

const GameCalendarScript = preload("res://scripts/core/game_calendar.gd")
const TraitSystem = preload("res://scripts/systems/trait_system.gd")
const PersonalitySystem = preload("res://scripts/core/personality_system.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _relationship_manager: RefCounted
var _entity_id: int = -1

## Personality bar colors
const PERSONALITY_COLORS: Dictionary = {
	"H": Color(0.9, 0.7, 0.2),
	"E": Color(0.4, 0.6, 0.9),
	"X": Color(0.9, 0.5, 0.2),
	"A": Color(0.3, 0.8, 0.5),
	"C": Color(0.2, 0.6, 0.9),
	"O": Color(0.7, 0.4, 0.9),
}
const FACET_COLOR_DIM: float = 0.7

## Trait sentiment colors
const TRAIT_COLORS: Dictionary = {
	"positive": Color(0.3, 0.8, 0.4),
	"negative": Color(0.9, 0.3, 0.3),
	"neutral": Color(0.9, 0.8, 0.3),
}

## Plutchik emotion wheel colors
const EMOTION_COLORS: Dictionary = {
	"joy": Color(1.0, 0.9, 0.2),          # Yellow
	"trust": Color(0.5, 0.8, 0.3),        # Green
	"fear": Color(0.2, 0.7, 0.4),         # Teal-green
	"surprise": Color(0.3, 0.6, 0.9),     # Blue
	"sadness": Color(0.3, 0.3, 0.8),      # Indigo
	"disgust": Color(0.6, 0.3, 0.7),      # Purple
	"anger": Color(0.9, 0.2, 0.2),        # Red
	"anticipation": Color(0.9, 0.6, 0.2), # Orange
}

## Stress bar color
const STRESS_COLOR: Color = Color(0.9, 0.3, 0.2)
const STRESS_BG_COLOR: Color = Color(0.3, 0.15, 0.1)

## Dyad badge colors
const DYAD_BADGE_BG: Color = Color(0.25, 0.2, 0.35, 0.6)
const DYAD_BADGE_BORDER: Color = Color(0.6, 0.5, 0.8, 0.7)

## Relationship type colors
const REL_TYPE_COLORS: Dictionary = {
	"stranger": Color(0.5, 0.5, 0.5),
	"acquaintance": Color(0.6, 0.6, 0.6),
	"friend": Color(0.3, 0.7, 0.3),
	"close_friend": Color(0.2, 0.8, 0.5),
	"romantic": Color(0.9, 0.4, 0.5),
	"partner": Color(0.9, 0.3, 0.4),
	"rival": Color(0.9, 0.2, 0.2),
}

## Gender display colors
const GENDER_COLORS: Dictionary = {
	"male": Color(0.4, 0.6, 0.9),
	"female": Color(0.9, 0.4, 0.6),
}

## Scroll state
var _scroll_offset: float = 0.0
var _content_height: float = 0.0

## Clickable name regions: [{rect: Rect2, entity_id: int}]
var _click_regions: Array = []
## Which axes are expanded (show facets)
var _expanded_axes: Dictionary = {}
## Deceased detail mode
var _showing_deceased: bool = false
var _deceased_record: Dictionary = {}


func init(entity_manager: RefCounted, building_manager: RefCounted = null, relationship_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_relationship_manager = relationship_manager


func set_entity_id(id: int) -> void:
	_entity_id = id
	_scroll_offset = 0.0
	_showing_deceased = false
	_deceased_record = {}


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_scroll_offset = minf(_scroll_offset + 30.0, maxf(0.0, _content_height - size.y + 40.0))
			accept_event()
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_scroll_offset = maxf(_scroll_offset - 30.0, 0.0)
			accept_event()
		elif event.button_index == MOUSE_BUTTON_LEFT:
			# Check click regions for name navigation
			for region in _click_regions:
				if region.rect.has_point(event.position):
					if region.has("axis_id"):
						var aid: String = region.axis_id
						_expanded_axes[aid] = not _expanded_axes.get(aid, false)
						accept_event()
						return
					_navigate_to_entity(region.entity_id)
					accept_event()
					return
	# macOS trackpad two-finger scroll
	elif event is InputEventPanGesture:
		_scroll_offset += event.delta.y * 15.0
		_scroll_offset = clampf(_scroll_offset, 0.0, maxf(0.0, _content_height - size.y + 40.0))
		accept_event()


func _draw() -> void:
	if not visible or _entity_manager == null or _entity_id < 0:
		return

	_click_regions.clear()

	# Deceased mode
	if _showing_deceased and _deceased_record.size() > 0:
		_draw_deceased()
		return

	var entity: RefCounted = _entity_manager.get_entity(_entity_id)
	if entity == null or not entity.is_alive:
		# Try deceased registry
		var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
		if registry != null:
			var record: Dictionary = registry.get_record(_entity_id)
			if record.size() > 0:
				_show_deceased(record)
				return
		visible = false
		return

	var panel_w: float = size.x
	var panel_h: float = size.y

	# Background
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.06, 0.1, 0.06, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.3, 0.4, 0.3), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = 20.0
	var cy: float = 28.0 - _scroll_offset
	var bar_w: float = panel_w - 40.0

	var job_colors: Dictionary = {
		"none": Color(0.6, 0.6, 0.6), "gatherer": Color(0.3, 0.8, 0.2),
		"lumberjack": Color(0.6, 0.35, 0.1), "builder": Color(0.9, 0.6, 0.1),
		"miner": Color(0.5, 0.6, 0.75),
	}
	var jc: Color = job_colors.get(entity.job, Color.WHITE)

	# ── Header ──
	var gender_icon: String = "M" if entity.gender == "male" else "F"
	var gender_color: Color = GENDER_COLORS.get(entity.gender, Color.WHITE)
	var header_text: String = "%s %s - %s (%s)" % [gender_icon, entity.entity_name, entity.job.capitalize(), entity.age_stage.capitalize()]
	draw_string(font, Vector2(cx, cy), header_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), jc)
	# Gender icon colored separately
	draw_string(font, Vector2(cx, cy), gender_icon, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), gender_color)
	cy += 6.0

	var current_date: Dictionary = GameCalendarScript.tick_to_date(entity.birth_tick + entity.age)
	var ref_date: Dictionary = {"year": current_date.year, "month": current_date.month, "day": current_date.day}
	var age_detail: String = GameCalendarScript.format_age_detailed(entity.birth_date, ref_date)
	var sid_text: String = "S%d" % entity.settlement_id if entity.settlement_id > 0 else "None"
	var preg_text: String = ""
	if entity.pregnancy_tick >= 0:
		preg_text = "  |  Pregnant"
	# Birth date from birth_date dictionary.
	var birth_str: String = ""
	if not entity.birth_date.is_empty():
		birth_str = GameCalendarScript.format_birth_date(entity.birth_date)
	else:
		birth_str = "출생일 불명"
	var life_stage_text: String = "%s | %s (%s)" % [entity.age_stage.capitalize(), age_detail, birth_str]
	draw_string(font, Vector2(cx, cy + 14), "Settlement: %s  |  %s  |  Pos: (%d, %d)%s" % [sid_text, life_stage_text, entity.position.x, entity.position.y, preg_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 22.0
	_draw_separator(cx, cy, panel_w)
	cy += 10.0

	# ── Status ──
	cy = _draw_section_header(font, cx, cy, "Status")

	var action_text: String = entity.current_action
	if entity.action_target != Vector2i(-1, -1):
		action_text += " -> (%d, %d)" % [entity.action_target.x, entity.action_target.y]
	draw_string(font, Vector2(cx + 10, cy + 12), "Action: %s" % action_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	cy += 16.0

	if entity.cached_path.size() > 0:
		var remaining: int = entity.cached_path.size() - entity.path_index
		if remaining > 0:
			draw_string(font, Vector2(cx + 10, cy + 12), "Path: %d steps remaining" % remaining, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
			cy += 16.0

	draw_string(font, Vector2(cx + 10, cy + 12), "Inventory: F:%.1f  W:%.1f  S:%.1f / %.0f" % [
		entity.inventory.get("food", 0.0), entity.inventory.get("wood", 0.0),
		entity.inventory.get("stone", 0.0), GameConfig.MAX_CARRY,
	], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	cy += 22.0

	# ── Needs ──
	cy = _draw_section_header(font, cx, cy, "Needs")
	cy = _draw_bar(font, cx + 10, cy, bar_w, "Hunger", entity.hunger, Color(0.9, 0.2, 0.2))
	cy = _draw_bar(font, cx + 10, cy, bar_w, "Energy", entity.energy, Color(0.9, 0.8, 0.2))
	cy = _draw_bar(font, cx + 10, cy, bar_w, "Social", entity.social, Color(0.3, 0.5, 0.9))
	cy += 6.0

	# ── Personality (HEXACO 6-axis + expandable facets) ──
	cy = _draw_section_header(font, cx, cy, "Personality")

	var axis_labels: Dictionary = {
		"H": "H (정직)", "E": "E (감정)", "X": "X (외향)",
		"A": "A (우호)", "C": "C (성실)", "O": "O (개방)",
	}

	var pd = entity.personality
	for axis_id in pd.AXIS_IDS:
		var axis_val: float = pd.axes.get(axis_id, 0.5)
		var color: Color = PERSONALITY_COLORS.get(axis_id, Color.GRAY)
		var is_expanded: bool = _expanded_axes.get(axis_id, false)
		var arrow: String = "▼" if is_expanded else "▶"
		var label: String = "%s %s" % [arrow, axis_labels.get(axis_id, axis_id)]

		var axis_y: float = cy
		cy = _draw_bar(font, cx + 10, cy, bar_w, label, axis_val, color)

		_click_regions.append({
			"rect": Rect2(cx + 10, axis_y, bar_w, 16.0),
			"entity_id": -1,
			"axis_id": axis_id,
		})

		if is_expanded:
			var fkeys: Array = pd.FACET_KEYS[axis_id]
			for fk in fkeys:
				var fval: float = pd.facets.get(fk, 0.5)
				var fname: String = "    " + fk.substr(fk.find("_") + 1).replace("_", " ").capitalize()
				var dim_color: Color = Color(color.r * FACET_COLOR_DIM, color.g * FACET_COLOR_DIM, color.b * FACET_COLOR_DIM)
				cy = _draw_bar(font, cx + 25, cy, bar_w - 15, fname, fval, dim_color)
	cy += 4.0

	# ── Traits (filtered: composites suppress overlapping singles, max 5) ──
	var display_traits: Array = TraitSystem.filter_display_traits(pd.active_traits)
	if display_traits.size() > 0:
		var trait_label: String = "Traits"
		draw_string(font, Vector2(cx + 10, cy + 12), trait_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 16.0
		var trait_x: float = cx + 15
		for trait_id in display_traits:
			var tdef: Dictionary = TraitSystem.get_trait_definition(trait_id)
			var tname: String = tdef.get("name_kr", trait_id)
			var sentiment: String = tdef.get("sentiment", "neutral")
			var tcolor: Color = TRAIT_COLORS.get(sentiment, Color.GRAY)
			var text_w: float = font.get_string_size(tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			# Wrap to next line if too wide
			if trait_x + text_w + 16 > size.x - 20:
				cy += 18.0
				trait_x = cx + 15
			# Badge background (rounded feel with semi-transparent fill)
			var badge_rect := Rect2(trait_x, cy, text_w + 12, 16)
			draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, 0.25))
			# Badge border
			draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, 0.6), false, 1.0)
			# Badge text
			draw_string(font, Vector2(trait_x + 6, cy + 12), tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), tcolor)
			trait_x += text_w + 18
		cy += 22.0
	cy += 6.0

	# ── Emotions (Plutchik 8) ──
	cy = _draw_section_header(font, cx, cy, "Emotions")
	if entity.emotion_data != null:
		var ed: RefCounted = entity.emotion_data
		# Draw 8 emotion bars with intensity labels
		for i in range(ed._emotion_order.size()):
			var emo_id: String = ed._emotion_order[i]
			var val: float = ed.get_emotion(emo_id) / 100.0  # Normalize to 0-1 for _draw_bar
			var label_en: String = ed._emotion_labels_en.get(emo_id, emo_id)
			var label_kr: String = ed.get_intensity_label_kr(emo_id)
			var display_label: String = label_en
			if label_kr != "":
				display_label = "%s (%s)" % [label_en, label_kr]
			cy = _draw_bar(font, cx + 10, cy, bar_w, display_label, val, EMOTION_COLORS.get(emo_id, Color.WHITE))

		# Valence-Arousal mood line
		cy += 4.0
		var va_text: String = "Mood: Valence %+.0f | Arousal %.0f" % [ed.valence, ed.arousal]
		draw_string(font, Vector2(cx + 10, cy + 12), va_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
		cy += 16.0

		# Active Dyads (threshold 30+)
		var active_dyads: Array = ed.get_active_dyads(30.0)
		if active_dyads.size() > 0:
			var dyad_x: float = cx + 10
			for di in range(active_dyads.size()):
				var dyad: Dictionary = active_dyads[di]
				var dyad_id: String = dyad.get("id", "")
				var dyad_kr: String = ed._dyad_labels_kr.get(dyad_id, dyad_id)
				var dyad_text: String = dyad_kr
				var text_w: float = font.get_string_size(dyad_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
				# Check if badge fits on current line
				if dyad_x + text_w + 12 > cx + bar_w + 10:
					dyad_x = cx + 10
					cy += 18.0
				var badge_rect := Rect2(dyad_x, cy, text_w + 12, 16)
				draw_rect(badge_rect, DYAD_BADGE_BG)
				draw_rect(badge_rect, DYAD_BADGE_BORDER, false, 1.0)
				draw_string(font, Vector2(dyad_x + 6, cy + 12), dyad_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.7, 0.95))
				dyad_x += text_w + 18
			cy += 20.0

		# Stress bar
		cy += 2.0
		var stress_val: float = ed.stress
		var z_C: float = 0.0
		if pd != null:
			z_C = pd.to_zscore(pd.axes.get("C", 0.5))
		var break_threshold: float = 300.0 + 50.0 * z_C
		var stress_ratio: float = clampf(stress_val / break_threshold, 0.0, 1.0)
		var stress_label: String = "Stress: %.0f / %.0f" % [stress_val, break_threshold]
		cy = _draw_bar(font, cx + 10, cy, bar_w, stress_label, stress_ratio, STRESS_COLOR)

		# Mental break indicator
		if ed.mental_break_type != "":
			var break_text: String = "MENTAL BREAK: %s (%.1fh)" % [ed.mental_break_type.to_upper(), ed.mental_break_remaining]
			draw_string(font, Vector2(cx + 10, cy + 12), break_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(1.0, 0.2, 0.2))
			cy += 16.0
	else:
		# Fallback: legacy 5-emotion display
		var e_keys: Array = ["happiness", "loneliness", "stress", "grief", "love"]
		var e_labels: Array = ["Happy", "Lonely", "Stress", "Grief", "Love"]
		var legacy_colors: Dictionary = {
			"happiness": Color(0.9, 0.8, 0.2), "loneliness": Color(0.4, 0.4, 0.7),
			"stress": Color(0.9, 0.5, 0.2), "grief": Color(0.5, 0.3, 0.5), "love": Color(0.9, 0.3, 0.4),
		}
		for i in range(e_keys.size()):
			var val: float = entity.emotions.get(e_keys[i], 0.0)
			cy = _draw_bar(font, cx + 10, cy, bar_w, e_labels[i], val, legacy_colors.get(e_keys[i], Color.WHITE))
	cy += 6.0

	# ── Family ──
	cy = _draw_section_header(font, cx, cy, "Family")

	# Partner
	if entity.partner_id >= 0:
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		var partner_name: String = "☠ (deceased)"
		var partner_alive: bool = false
		if partner != null and partner.is_alive:
			partner_name = partner.entity_name
			partner_alive = true
		else:
			if partner != null:
				partner_name = "%s ☠" % partner.entity_name
			# Check DeceasedRegistry
			var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
			if registry != null:
				var record: Dictionary = registry.get_record(entity.partner_id)
				if record.size() > 0:
					partner_name = record.get("name", "?") + " ☠"
		var love_pct: int = 0
		if entity.emotion_data != null:
			love_pct = int(entity.emotion_data.get_dyad("love"))
		else:
			love_pct = int(entity.emotions.get("love", 0.0) * 100)
		var prefix: String = "Partner: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		var name_pos := Vector2(cx + 10 + prefix_w, cy + 12)
		draw_string(font, name_pos, partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
		_register_click_region(name_pos, partner_name, entity.partner_id, font, GameConfig.get_font_size("popup_body"))
		var name_w: float = font.get_string_size(partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		# Love + Compatibility suffix
		var suffix: String = " (Love: %d%%" % love_pct
		if entity.personality != null and partner_alive and partner != null and partner.personality != null:
			var compat: float = PersonalitySystem.personality_compatibility(entity.personality, partner.personality)
			var compat_pct: int = int((compat + 1.0) / 2.0 * 100)
			suffix += ", Compat: %d%%" % compat_pct
		suffix += ")"
		draw_string(font, Vector2(cx + 10 + prefix_w + name_w, cy + 12), suffix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		cy += 16.0
	else:
		draw_string(font, Vector2(cx + 10, cy + 12), "Partner: None", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		cy += 16.0

	# Parents
	if entity.parent_ids.size() > 0:
		var prefix: String = "Parents: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
		var name_x: float = cx + 10 + font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		for i in range(entity.parent_ids.size()):
			var pid: int = entity.parent_ids[i]
			var pname: String = "?"
			var parent: RefCounted = _entity_manager.get_entity(pid)
			if parent != null:
				pname = parent.entity_name
				if not parent.is_alive:
					pname += " ☠"
			else:
				var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
				if registry != null:
					var record: Dictionary = registry.get_record(pid)
					if record.size() > 0:
						pname = record.get("name", "?") + " ☠"
			if i > 0:
				draw_string(font, Vector2(name_x, cy + 12), ", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
				name_x += font.get_string_size(", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			var npos := Vector2(name_x, cy + 12)
			draw_string(font, npos, pname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
			_register_click_region(npos, pname, pid, font, GameConfig.get_font_size("popup_body"))
			name_x += font.get_string_size(pname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		cy += 16.0
	else:
		draw_string(font, Vector2(cx + 10, cy + 12), "Parents: 1st generation", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		cy += 16.0

	# Children
	if entity.children_ids.size() > 0:
		var prefix: String = "Children: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 0.7))
		var name_x: float = cx + 10 + font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		var child_ref_date: Dictionary = {"year": current_date.year, "month": current_date.month, "day": current_date.day}
		for i in range(entity.children_ids.size()):
			var cid: int = entity.children_ids[i]
			var cname: String = "?"
			var child: RefCounted = _entity_manager.get_entity(cid)
			if child != null:
				var child_age_short: String = GameCalendarScript.format_age_short(child.birth_date, child_ref_date)
				cname = "%s (%s)" % [child.entity_name, child_age_short]
				if not child.is_alive:
					cname = "%s ☠" % child.entity_name
			else:
				var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
				if registry != null:
					var record: Dictionary = registry.get_record(cid)
					if record.size() > 0:
						cname = record.get("name", "?") + " ☠"
			if i > 0:
				draw_string(font, Vector2(name_x, cy + 12), ", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 0.7))
				name_x += font.get_string_size(", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			# Wrap to next line if too long
			if name_x > size.x - 40:
				cy += 14.0
				name_x = cx + 20
			var npos := Vector2(name_x, cy + 12)
			draw_string(font, npos, cname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
			_register_click_region(npos, cname, cid, font, GameConfig.get_font_size("popup_body"))
			name_x += font.get_string_size(cname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		cy += 16.0
	cy += 6.0

	# ── Key Relationships ──
	if _relationship_manager != null:
		cy = _draw_section_header(font, cx, cy, "Key Relationships")
		var rels: Array = _relationship_manager.get_relationships_for(entity.id)
		if rels.size() == 0:
			draw_string(font, Vector2(cx + 10, cy + 12), "No relationships yet", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
			cy += 16.0
		else:
			var show_count: int = mini(5, rels.size())
			for i in range(show_count):
				var other_id: int = rels[i].other_id
				var rel: RefCounted = rels[i].rel
				var other: RefCounted = _entity_manager.get_entity(other_id)
				var other_name: String = "?"
				if other != null:
					other_name = other.entity_name
				else:
					var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
					if registry != null:
						var record: Dictionary = registry.get_record(other_id)
						if record.size() > 0:
							other_name = record.get("name", "?") + " ☠"
				var type_color: Color = REL_TYPE_COLORS.get(rel.type, Color.GRAY)
				# Draw clickable name
				var npos := Vector2(cx + 10, cy + 12)
				draw_string(font, npos, other_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
				_register_click_region(npos, other_name, other_id, font, GameConfig.get_font_size("popup_body"))
				var name_w: float = font.get_string_size(other_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
				var rel_label: String = " - %s (A:%d T:%d)" % [rel.type.replace("_", " ").capitalize(), int(rel.affinity), int(rel.trust)]
				if rel.romantic_interest > 0.0:
					rel_label += " R:%d" % int(rel.romantic_interest)
				draw_string(font, Vector2(cx + 10 + name_w, cy + 12), rel_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), type_color)
				cy += 15.0
		cy += 6.0

	# ── Stats ──
	cy = _draw_section_header(font, cx, cy, "Stats")
	draw_string(font, Vector2(cx + 10, cy + 12), "Speed: %.1f  |  Strength: %.1f" % [entity.speed, entity.strength], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	cy += 16.0
	draw_string(font, Vector2(cx + 10, cy + 12), "Total gathered: %.0f  |  Buildings built: %d" % [entity.total_gathered, entity.buildings_built], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	cy += 22.0

	# ── Action History ──
	cy = _draw_section_header(font, cx, cy, "Recent Actions")
	var hist: Array = entity.action_history
	var idx: int = hist.size() - 1
	var hist_count: int = 0
	while idx >= 0 and hist_count < 5:
		var entry: Dictionary = hist[idx]
		draw_string(font, Vector2(cx + 10, cy + 11), "Tick %d: %s" % [entry.tick, entry.action], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.6, 0.6, 0.6))
		cy += 13.0
		idx -= 1
		hist_count += 1

	# Track content height for scrolling
	_content_height = cy + _scroll_offset + 20.0

	# Footer
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), "Scroll for more | Click background to close", HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))
	_draw_scrollbar()


func _draw_scrollbar() -> void:
	# Only show when content overflows
	if _content_height <= size.y:
		return

	var panel_h: float = size.y
	var scrollbar_width: float = 6.0
	var scrollbar_margin: float = 3.0
	var scrollbar_x: float = size.x - scrollbar_width - scrollbar_margin
	var track_top: float = 4.0
	var track_bottom: float = panel_h - 4.0
	var track_height: float = track_bottom - track_top

	# Draw track background
	draw_rect(Rect2(scrollbar_x, track_top, scrollbar_width, track_height), Color(0.15, 0.15, 0.15, 0.3))

	# Calculate grabber size and position
	var visible_ratio: float = clampf(panel_h / _content_height, 0.05, 1.0)
	var grabber_height: float = maxf(track_height * visible_ratio, 20.0)
	var scroll_max: float = maxf(0.0, _content_height - panel_h)
	var scroll_ratio: float = clampf(_scroll_offset / scroll_max, 0.0, 1.0) if scroll_max > 0.0 else 0.0
	var grabber_y: float = track_top + (track_height - grabber_height) * scroll_ratio

	# Draw grabber
	draw_rect(Rect2(scrollbar_x, grabber_y, scrollbar_width, grabber_height), Color(0.6, 0.6, 0.6, 0.5))


func _draw_section_header(font: Font, x: float, y: float, title: String) -> float:
	draw_string(font, Vector2(x, y + 12), title, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	return y + 18.0


func _draw_separator(x: float, y: float, panel_w: float) -> void:
	draw_line(Vector2(x, y), Vector2(panel_w - 20, y), Color(0.3, 0.3, 0.3), 1.0)


func _draw_bar(font: Font, x: float, y: float, w: float, label: String, value: float, color: Color) -> float:
	var label_w: float = 130.0
	var pct_w: float = 45.0
	var bar_gap: float = 4.0
	var bar_h: float = 10.0

	draw_string(font, Vector2(x, y + 11), label, HORIZONTAL_ALIGNMENT_LEFT, int(label_w), GameConfig.get_font_size("bar_label"), Color(0.7, 0.7, 0.7))

	var bar_x: float = x + label_w + bar_gap
	var bar_w: float = maxf(w - label_w - pct_w - bar_gap * 2, 20.0)
	draw_rect(Rect2(bar_x, y + 2, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	draw_rect(Rect2(bar_x, y + 2, bar_w * clampf(value, 0.0, 1.0), bar_h), color)

	var pct_x: float = bar_x + bar_w + bar_gap
	draw_string(font, Vector2(pct_x, y + 11), "%d%%" % int(value * 100), HORIZONTAL_ALIGNMENT_RIGHT, int(pct_w), GameConfig.get_font_size("bar_label"), Color(0.8, 0.8, 0.8))
	return y + 16.0


func _register_click_region(pos: Vector2, text: String, entity_id: int, font: Font, font_size: int) -> void:
	var text_size: Vector2 = font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size)
	_click_regions.append({"rect": Rect2(pos.x, pos.y - font_size, text_size.x, font_size + 4), "entity_id": entity_id})


func _navigate_to_entity(entity_id: int) -> void:
	# Check alive entity first
	var entity: RefCounted = _entity_manager.get_entity(entity_id)
	if entity != null and entity.is_alive:
		set_entity_id(entity_id)
		_showing_deceased = false
		SimulationBus.entity_selected.emit(entity_id)
		return
	# Check DeceasedRegistry
	var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
	if registry != null:
		var record: Dictionary = registry.get_record(entity_id)
		if record.size() > 0:
			_show_deceased(record)
			return


func _show_deceased(record: Dictionary) -> void:
	_showing_deceased = true
	_deceased_record = record
	_entity_id = record.get("id", -1)
	_scroll_offset = 0.0


func show_entity_or_deceased(entity_id: int) -> void:
	var entity: RefCounted = _entity_manager.get_entity(entity_id)
	if entity != null and entity.is_alive:
		_showing_deceased = false
		set_entity_id(entity_id)
	else:
		var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
		if registry != null:
			var record: Dictionary = registry.get_record(entity_id)
			if record.size() > 0:
				_show_deceased(record)


func _lookup_name(entity_id: int) -> String:
	var entity: RefCounted = _entity_manager.get_entity(entity_id)
	if entity != null:
		if entity.is_alive:
			return entity.entity_name
		else:
			return entity.entity_name + " ☠"
	var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
	if registry != null:
		var record: Dictionary = registry.get_record(entity_id)
		if record.size() > 0:
			return record.get("name", "?") + " ☠"
	return "?"


func _draw_deceased() -> void:
	_click_regions.clear()
	var r: Dictionary = _deceased_record
	var panel_w: float = size.x
	var panel_h: float = size.y

	# Background (darker, death theme)
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.08, 0.06, 0.08, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.4, 0.3, 0.4), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = 20.0
	var cy: float = 28.0 - _scroll_offset

	# Header
	var gender_icon: String = "M" if r.get("gender", "") == "male" else "F"
	var header: String = "%s %s (Deceased)" % [gender_icon, r.get("name", "?")]
	draw_string(font, Vector2(cx, cy), header, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), Color(0.7, 0.5, 0.5))
	cy += 6.0

	# Life span
	var birth_str: String = "?"
	var death_str: String = "?"
	var birth_tick: int = r.get("birth_tick", 0)
	var death_tick: int = r.get("death_tick", 0)
	var birth_date: Dictionary = r.get("birth_date", {})
	var death_date: Dictionary = r.get("death_date", {})
	if birth_tick >= 0:
		var bd: Dictionary = GameCalendarScript.tick_to_date(birth_tick)
		birth_str = "Y%d %d월 %d일" % [bd.year, bd.month, bd.day]
	else:
		birth_str = "초기세대"
	var dd: Dictionary = GameCalendarScript.tick_to_date(death_tick)
	death_str = "Y%d %d월 %d일" % [dd.year, dd.month, dd.day]
	var age_str: String = "?"
	if not birth_date.is_empty() and not death_date.is_empty():
		age_str = GameCalendarScript.format_age_detailed(birth_date, death_date)
	else:
		age_str = "%.0f세" % r.get("death_age_years", 0.0)
	draw_string(font, Vector2(cx, cy + 14), "%s ~ %s (향년 %s)" % [birth_str, death_str, age_str], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
	cy += 18.0

	if not birth_date.is_empty() and not death_date.is_empty():
		var survival: Dictionary = GameCalendarScript.calculate_detailed_age(birth_date, death_date)
		var total_days_str: String = GameCalendarScript.format_number(survival.total_days)
		var dur_parts: Array = []
		if survival.years > 0:
			dur_parts.append("%d년" % survival.years)
		if survival.months > 0:
			dur_parts.append("%d개월" % survival.months)
		dur_parts.append("%d일" % survival.days)
		draw_string(font, Vector2(cx, cy + 14), "생존 기간: %s (%s일)" % [" ".join(dur_parts), total_days_str], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
		cy += 18.0

	# Cause of death
	var cause_raw: String = r.get("death_cause", "unknown")
	var cause_map: Dictionary = {
		"starvation": "아사",
		"old_age": "노령",
		"infant_mortality": "영아 사망",
		"background": "사고/질병",
		"maternal_death": "출산 사망",
		"stillborn": "사산",
	}
	var cause_display: String = cause_map.get(cause_raw, cause_raw)
	draw_string(font, Vector2(cx, cy + 14), "사인: %s" % cause_display, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.5, 0.5))
	cy += 18.0
	_draw_separator(cx, cy, panel_w)
	cy += 10.0

	# Job and settlement
	cy = _draw_section_header(font, cx, cy, "Info")
	var job_text: String = r.get("job", "none").capitalize()
	var stage_text: String = r.get("age_stage", "?").capitalize()
	draw_string(font, Vector2(cx + 10, cy + 12), "Job: %s | Stage: %s" % [job_text, stage_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 16.0
	draw_string(font, Vector2(cx + 10, cy + 12), "Gathered: %.0f | Built: %d" % [r.get("total_gathered", 0.0), r.get("buildings_built", 0)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 22.0

	# Family (clickable)
	cy = _draw_section_header(font, cx, cy, "Family")

	# Partner
	var partner_id: int = r.get("partner_id", -1)
	if partner_id > 0:
		var prefix: String = "Partner: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		var pname: String = _lookup_name(partner_id)
		var npos := Vector2(cx + 10 + prefix_w, cy + 12)
		draw_string(font, npos, pname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
		_register_click_region(npos, pname, partner_id, font, GameConfig.get_font_size("popup_body"))
		cy += 16.0

	# Parents
	var parent_ids: Array = r.get("parent_ids", [])
	if parent_ids.size() > 0:
		var prefix: String = "Parents: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
		var name_x: float = cx + 10 + font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		for i in range(parent_ids.size()):
			var pid: int = parent_ids[i]
			var pname: String = _lookup_name(pid)
			if i > 0:
				draw_string(font, Vector2(name_x, cy + 12), ", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
				name_x += font.get_string_size(", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			var npos := Vector2(name_x, cy + 12)
			draw_string(font, npos, pname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
			_register_click_region(npos, pname, pid, font, GameConfig.get_font_size("popup_body"))
			name_x += font.get_string_size(pname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		cy += 16.0

	# Children
	var children_ids: Array = r.get("children_ids", [])
	if children_ids.size() > 0:
		var prefix: String = "Children: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 0.7))
		var name_x: float = cx + 10 + font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		for i in range(children_ids.size()):
			var cid: int = children_ids[i]
			var cname: String = _lookup_name(cid)
			if i > 0:
				draw_string(font, Vector2(name_x, cy + 12), ", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 0.7))
				name_x += font.get_string_size(", ", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			if name_x > size.x - 40:
				cy += 14.0
				name_x = cx + 20
			var npos := Vector2(name_x, cy + 12)
			draw_string(font, npos, cname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
			_register_click_region(npos, cname, cid, font, GameConfig.get_font_size("popup_body"))
			name_x += font.get_string_size(cname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		cy += 16.0
	cy += 6.0

	# Personality
	cy = _draw_section_header(font, cx, cy, "Personality")
	var PersonalityDataScript = load("res://scripts/core/personality_data.gd")
	var p_dict: Dictionary = r.get("personality", {})
	var pd: RefCounted
	if p_dict.has("facets"):
		pd = PersonalityDataScript.from_dict(p_dict)
	else:
		pd = PersonalityDataScript.new()
		pd.migrate_from_big_five(p_dict)
	var axis_labels: Dictionary = {
		"H": "H (정직)", "E": "E (감정)", "X": "X (외향)",
		"A": "A (우호)", "C": "C (성실)", "O": "O (개방)",
	}
	var bar_w: float = panel_w - 40.0

	for axis_id in pd.AXIS_IDS:
		var axis_val: float = pd.axes.get(axis_id, 0.5)
		var color: Color = PERSONALITY_COLORS.get(axis_id, Color.GRAY)
		var is_expanded: bool = _expanded_axes.get(axis_id, false)
		var arrow: String = "▼" if is_expanded else "▶"
		var label: String = "%s %s" % [arrow, axis_labels.get(axis_id, axis_id)]

		var axis_y: float = cy
		cy = _draw_bar(font, cx + 10, cy, bar_w, label, axis_val, color)

		_click_regions.append({
			"rect": Rect2(cx + 10, axis_y, bar_w, 16.0),
			"entity_id": -1,
			"axis_id": axis_id,
		})

		if is_expanded:
			var fkeys: Array = pd.FACET_KEYS[axis_id]
			for fk in fkeys:
				var fval: float = pd.facets.get(fk, 0.5)
				var fname: String = "    " + fk.substr(fk.find("_") + 1).replace("_", " ").capitalize()
				var dim_color: Color = Color(color.r * FACET_COLOR_DIM, color.g * FACET_COLOR_DIM, color.b * FACET_COLOR_DIM)
				cy = _draw_bar(font, cx + 25, cy, bar_w - 15, fname, fval, dim_color)
	cy += 4.0

	# ── Traits (filtered: composites suppress overlapping singles, max 5) ──
	var display_traits: Array = TraitSystem.filter_display_traits(pd.active_traits)
	if display_traits.size() > 0:
		var trait_label: String = "Traits"
		draw_string(font, Vector2(cx + 10, cy + 12), trait_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 16.0
		var trait_x: float = cx + 15
		for trait_id in display_traits:
			var tdef: Dictionary = TraitSystem.get_trait_definition(trait_id)
			var tname: String = tdef.get("name_kr", trait_id)
			var sentiment: String = tdef.get("sentiment", "neutral")
			var tcolor: Color = TRAIT_COLORS.get(sentiment, Color.GRAY)
			var text_w: float = font.get_string_size(tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			# Wrap to next line if too wide
			if trait_x + text_w + 16 > size.x - 20:
				cy += 18.0
				trait_x = cx + 15
			# Badge background (rounded feel with semi-transparent fill)
			var badge_rect := Rect2(trait_x, cy, text_w + 12, 16)
			draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, 0.25))
			# Badge border
			draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, 0.6), false, 1.0)
			# Badge text
			draw_string(font, Vector2(trait_x + 6, cy + 12), tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), tcolor)
			trait_x += text_w + 18
		cy += 22.0
	cy += 6.0

	# Chronicle events
	var chronicle: Node = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle != null:
		cy = _draw_section_header(font, cx, cy, "Life Events")
		var events: Array = chronicle.get_personal_events(r.get("id", -1))
		var show_count: int = mini(8, events.size())
		if show_count == 0:
			draw_string(font, Vector2(cx + 10, cy + 12), "No recorded events", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
			cy += 16.0
		else:
			var idx: int = events.size() - 1
			var count: int = 0
			while idx >= 0 and count < show_count:
				var evt: Dictionary = events[idx]
				var desc: String = evt.get("description", "?")
				if desc.length() > 50:
					desc = desc.substr(0, 47) + "..."
				draw_string(font, Vector2(cx + 10, cy + 11), "Y%d M%d: %s" % [evt.get("year", 0), evt.get("month", 0), desc], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.6, 0.6, 0.6))
				cy += 13.0
				idx -= 1
				count += 1

	_content_height = cy + _scroll_offset + 20.0
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), "Scroll for more | Click background to close", HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))
	_draw_scrollbar()
