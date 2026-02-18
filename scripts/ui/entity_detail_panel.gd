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

## Trait valence colors
const TRAIT_COLORS: Dictionary = {
	"positive": Color(0.2, 0.8, 0.3),
	"negative": Color(0.9, 0.2, 0.2),
	"neutral": Color(0.5, 0.6, 0.8),
}
## Special color for Dark traits (psychopath, narcissist, etc.)
const DARK_TRAIT_COLOR: Color = Color(0.6, 0.0, 0.6)

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

## Scrollbar drag state
var _scrollbar_dragging: bool = false
var _scrollbar_rect: Rect2 = Rect2()

## Clickable name regions: [{rect: Rect2, entity_id: int}]
var _click_regions: Array = []
## Trait badge rects for tooltip: [{rect: Rect2, trait_def: Dictionary}]
var _trait_badge_regions: Array = []
## Reference to trait tooltip overlay
var _trait_tooltip: Control = null
## Currently active (clicked) trait id
var _active_trait_id: String = ""
## Trait effect summary panel expanded state
var _summary_expanded: bool = false
## Trait summary toggle click rect
var _summary_toggle_rect: Rect2 = Rect2()
## Which axes are expanded (show facets)
var _expanded_axes: Dictionary = {}
## Deceased detail mode
var _showing_deceased: bool = false
var _deceased_record: Dictionary = {}
## Section collapsed states (true = collapsed/hidden)
var _section_collapsed: Dictionary = {
	"status": false,
	"needs": false,
	"personality": true,
	"traits": false,
	"emotions": true,
	"trauma_scars": true,
	"violation_history": false,
	"family": false,
	"relationships": false,
	"stats": true,
	"recent_actions": false,
}
## Section header rects for click detection (cleared each _draw frame)
var _section_header_rects: Dictionary = {}


func init(entity_manager: RefCounted, building_manager: RefCounted = null, relationship_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_relationship_manager = relationship_manager


func set_entity_id(id: int) -> void:
	_entity_id = id
	_scroll_offset = 0.0
	_showing_deceased = false
	_deceased_record = {}
	_trait_badge_regions.clear()
	_active_trait_id = ""
	_summary_expanded = false
	_section_header_rects.clear()
	_summary_toggle_rect = Rect2()
	if _trait_tooltip != null:
		_trait_tooltip.request_hide()


func _ready() -> void:
	var locale_changed_cb: Callable = Callable(self, "_on_locale_changed")
	if not Locale.locale_changed.is_connected(locale_changed_cb):
		Locale.locale_changed.connect(locale_changed_cb)
	# Create trait tooltip as child overlay
	var TraitTooltipScript = preload("res://scripts/ui/trait_tooltip.gd")
	_trait_tooltip = TraitTooltipScript.new()
	add_child(_trait_tooltip)


func _on_locale_changed(_locale: String = "") -> void:
	queue_redraw()


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	# Scrollbar drag handling
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			if _scrollbar_rect.size.x > 0 and _scrollbar_rect.has_point(event.position):
				_scrollbar_dragging = true
				_update_scroll_from_mouse(event.position.y)
				accept_event()
				return
		else:
			if _scrollbar_dragging:
				_scrollbar_dragging = false
				accept_event()
				return

	if event is InputEventMouseMotion and _scrollbar_dragging:
		_update_scroll_from_mouse(event.position.y)
		accept_event()
		return

	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_scroll_offset = minf(_scroll_offset + 30.0, maxf(0.0, _content_height - size.y + 40.0))
			accept_event()
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_scroll_offset = maxf(_scroll_offset - 30.0, 0.0)
			accept_event()
		elif event.button_index == MOUSE_BUTTON_LEFT:
			# Summary toggle click
			if _summary_toggle_rect.size.x > 0 and _summary_toggle_rect.has_point(event.position):
				_summary_expanded = not _summary_expanded
				accept_event()
				return

			# Trait badge click -> toggle tooltip
			if _trait_tooltip != null:
				for region in _trait_badge_regions:
					if region.rect.has_point(event.position):
						var clicked_id: String = region.trait_def.get("id", "")
						if clicked_id == _active_trait_id:
							_trait_tooltip.request_hide()
							_active_trait_id = ""
						else:
							_trait_tooltip.show_immediate(region.trait_def, region.rect)
							_active_trait_id = clicked_id
						accept_event()
						return

			# Empty area click closes active tooltip
			if _active_trait_id != "" and _trait_tooltip != null:
				_trait_tooltip.request_hide()
				_active_trait_id = ""

			# Section header toggle
			for sec_id in _section_header_rects:
				if _section_header_rects[sec_id].has_point(event.position):
					_section_collapsed[sec_id] = not _section_collapsed.get(sec_id, false)
					accept_event()
					return

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


func _update_scroll_from_mouse(mouse_y: float) -> void:
	var track_top: float = _scrollbar_rect.position.y
	var track_height: float = _scrollbar_rect.size.y
	if track_height <= 0.0:
		return
	var ratio: float = clampf((mouse_y - track_top) / track_height, 0.0, 1.0)
	var scroll_max: float = maxf(0.0, _content_height - size.y + 40.0)
	_scroll_offset = ratio * scroll_max


## Get color for a trait, using valence with Dark trait override
func _get_trait_color(tdef: Dictionary) -> Color:
	if tdef.get("id", "").begins_with("d_"):
		return DARK_TRAIT_COLOR
	var valence: String = tdef.get("valence", "neutral")
	return TRAIT_COLORS.get(valence, Color.GRAY)


## Draw trait badges and optional trait effect summary for personality data.
func _draw_trait_section(font: Font, cx: float, cy: float, pd: RefCounted, entity: RefCounted = null) -> float:
	_trait_badge_regions.clear()
	_summary_toggle_rect = Rect2()

	var display_trait_ids: Array = []
	if entity != null and "display_traits" in entity:
		for i in range(entity.display_traits.size()):
			var dt: Dictionary = entity.display_traits[i]
			display_trait_ids.append(dt.get("id", ""))
	if display_trait_ids.is_empty():
		if _active_trait_id != "" and _trait_tooltip != null:
			_trait_tooltip.request_hide()
			_active_trait_id = ""
		return cy + 6.0

	var trait_defs: Array = []
	for tid in display_trait_ids:
		var tdef: Dictionary = TraitSystem.get_trait_definition(str(tid))
		if not tdef.is_empty():
			trait_defs.append(tdef)

	if trait_defs.is_empty():
		if _active_trait_id != "" and _trait_tooltip != null:
			_trait_tooltip.request_hide()
			_active_trait_id = ""
		return cy + 6.0

	trait_defs.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		var na: String = Locale.ltr(a.get("name_key", "TRAIT_" + a.get("id", "") + "_NAME"))
		var nb: String = Locale.ltr(b.get("name_key", "TRAIT_" + b.get("id", "") + "_NAME"))
		return na.naturalcasecmp_to(nb) < 0
	)

	var has_active_trait: bool = false
	for tdef in trait_defs:
		if tdef.get("id", "") == _active_trait_id:
			has_active_trait = true
			break
	if not has_active_trait and _active_trait_id != "":
		if _trait_tooltip != null:
			_trait_tooltip.request_hide()
		_active_trait_id = ""

	var trait_label: String = Locale.ltr("UI_TRAITS")
	draw_string(font, Vector2(cx + 10, cy + 12), trait_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	cy += 16.0

	var trait_x: float = cx + 15
	for tdef in trait_defs:
		var trait_id: String = tdef.get("id", "")
		var tname: String = Locale.ltr(tdef.get("name_key", "TRAIT_" + tdef.get("id", "") + "_NAME"))
		var salience: float = 0.0
		if entity != null and "display_traits" in entity:
			for dt in entity.display_traits:
				if dt.get("id", "") == trait_id:
					salience = float(dt.get("salience", 0.0))
					break
		var badge_label: String = tname
		if salience > 0.0:
			badge_label = "%s %.2f" % [tname, salience]
		var tcolor: Color = _get_trait_color(tdef)
		var text_w: float = font.get_string_size(badge_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		if trait_x + text_w + 16 > size.x - 20:
			cy += 26.0
			trait_x = cx + 15

		var badge_rect: Rect2 = Rect2(trait_x, cy, text_w + 12, 22)
		var is_active: bool = trait_id == _active_trait_id
		var fill_alpha: float = 0.4 if is_active else 0.25
		var border_alpha: float = 1.0 if is_active else 0.6
		var border_width: float = 2.0 if is_active else 1.0
		draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, fill_alpha))
		draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, border_alpha), false, border_width)
		draw_string(font, Vector2(trait_x + 6, cy + 16), badge_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), tcolor)
		_trait_badge_regions.append({"rect": badge_rect, "trait_def": tdef})
		trait_x += text_w + 18
	cy += 28.0

	var summary_title: String = Locale.ltr("UI_TRAIT_EFFECT_SUMMARY")
	var toggle_text: String = "%s %s" % ["▼" if _summary_expanded else "▶", summary_title]
	var toggle_w: float = font.get_string_size(toggle_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
	_summary_toggle_rect = Rect2(cx + 10, cy, toggle_w + 8, 18)
	draw_string(font, Vector2(cx + 14, cy + 13), toggle_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 1.0))
	cy += 22.0

	if _summary_expanded:
		cy = _draw_trait_summary(font, cx, cy, trait_defs, entity)

	return cy + 6.0


## Draw aggregate trait effect summary for currently visible traits.
func _draw_trait_summary(font: Font, cx: float, cy: float, trait_defs: Array, entity: RefCounted = null) -> float:
	var fs: int = GameConfig.get_font_size("popup_body")
	var indent: float = cx + 20.0
	var sub_indent: float = cx + 30.0
	var text_color: Color = Color(0.75, 0.75, 0.75)

	var behavior_totals: Dictionary = {}
	var emotion_totals: Dictionary = {}
	var relationship_totals: Dictionary = {}
	var active_ids: Dictionary = {}
	for tdef in trait_defs:
		var trait_id: String = tdef.get("id", "")
		active_ids[trait_id] = true

	if entity != null:
		for action in TraitSystem.get_known_behavior_actions():
			var weight: float = TraitSystem.get_effect_value(entity, "behavior_weight", str(action))
			if abs(weight - 1.0) > 0.01:
				behavior_totals[action] = weight - 1.0

		for emotion in TraitSystem.get_known_emotion_baselines():
			var baseline: float = TraitSystem.get_effect_value(entity, "emotion_baseline", str(emotion))
			if abs(baseline) > 0.005:
				emotion_totals[emotion] = baseline

	var synergies: Array = []
	var conflicts: Array = []
	var seen_synergy_pairs: Dictionary = {}
	var seen_conflict_pairs: Dictionary = {}
	for tdef in trait_defs:
		var trait_id: String = tdef.get("id", "")

		var trait_synergies: Array = tdef.get("synergies", [])
		for sid in trait_synergies:
			var synergy_id: String = str(sid)
			if not active_ids.has(synergy_id):
				continue
			var synergy_pair: Array = [trait_id, synergy_id]
			synergy_pair.sort()
			var synergy_key: String = "%s_%s" % [synergy_pair[0], synergy_pair[1]]
			if seen_synergy_pairs.has(synergy_key):
				continue
			seen_synergy_pairs[synergy_key] = true
			synergies.append({"a": trait_id, "b": synergy_id})

		var trait_conflicts: Array = tdef.get("anti_synergies", [])
		for aid in trait_conflicts:
			var conflict_id: String = str(aid)
			if not active_ids.has(conflict_id):
				continue
			var conflict_pair: Array = [trait_id, conflict_id]
			conflict_pair.sort()
			var conflict_key: String = "%s_%s" % [conflict_pair[0], conflict_pair[1]]
			if seen_conflict_pairs.has(conflict_key):
				continue
			seen_conflict_pairs[conflict_key] = true
			conflicts.append({"a": trait_id, "b": conflict_id})

	var has_any: bool = false

	if behavior_totals.size() > 0:
		has_any = true
		draw_string(font, Vector2(indent, cy + 12), Locale.ltr("UI_TRAIT_BEHAVIOR_WEIGHTS") + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.8, 0.85, 1.0))
		cy += 16.0
		var behavior_keys: Array = behavior_totals.keys()
		behavior_keys.sort_custom(func(a, b):
			var ka: String = str(a).to_upper()
			var kb: String = str(b).to_upper()
			var da: String = Locale.ltr("TRAIT_KEY_" + ka)
			var db: String = Locale.ltr("TRAIT_KEY_" + kb)
			if da == "TRAIT_KEY_" + ka:
				da = str(a).replace("_", " ").capitalize()
			if db == "TRAIT_KEY_" + kb:
				db = str(b).replace("_", " ").capitalize()
			return da.naturalcasecmp_to(db) < 0
		)
		for key in behavior_keys:
			var key_str: String = str(key)
			var value: float = float(behavior_totals[key_str])
			var display_key: String = Locale.ltr("TRAIT_KEY_" + key_str.to_upper())
			if display_key == "TRAIT_KEY_" + key_str.to_upper():
				display_key = key_str.replace("_", " ").capitalize()
			var pct: float = value * 100.0
			var sign: String = "+" if pct >= 0.0 else ""
			draw_string(font, Vector2(sub_indent, cy + 12), "%s: %s%.0f%%" % [display_key, sign, pct], HORIZONTAL_ALIGNMENT_LEFT, -1, fs, text_color)
			cy += 15.0

	if emotion_totals.size() > 0:
		has_any = true
		draw_string(font, Vector2(indent, cy + 12), Locale.ltr("UI_TRAIT_EMOTION_MODIFIERS") + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.8, 0.85, 1.0))
		cy += 16.0
		var emotion_keys: Array = emotion_totals.keys()
		emotion_keys.sort_custom(func(a, b):
			var ka: String = str(a).to_upper()
			var kb: String = str(b).to_upper()
			var da: String = Locale.ltr("TRAIT_KEY_" + ka)
			var db: String = Locale.ltr("TRAIT_KEY_" + kb)
			if da == "TRAIT_KEY_" + ka:
				da = str(a).replace("_", " ").capitalize()
			if db == "TRAIT_KEY_" + kb:
				db = str(b).replace("_", " ").capitalize()
			return da.naturalcasecmp_to(db) < 0
		)
		for key in emotion_keys:
			var key_str: String = str(key)
			var value: float = float(emotion_totals[key_str])
			var display_key: String = Locale.ltr("TRAIT_KEY_" + key_str.to_upper())
			if display_key == "TRAIT_KEY_" + key_str.to_upper():
				display_key = key_str.replace("_", " ").capitalize()
			# value is already a delta; convert to percent
			var pct: float = value * 100.0
			var sign: String = "+" if pct >= 0.0 else ""
			draw_string(font, Vector2(sub_indent, cy + 12), "%s: %s%.0f%%" % [display_key, sign, pct], HORIZONTAL_ALIGNMENT_LEFT, -1, fs, text_color)
			cy += 15.0

	if relationship_totals.size() > 0:
		has_any = true
		draw_string(font, Vector2(indent, cy + 12), Locale.ltr("UI_TRAIT_RELATIONSHIP_MODIFIERS") + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.8, 0.85, 1.0))
		cy += 16.0
		var rel_keys: Array = relationship_totals.keys()
		rel_keys.sort_custom(func(a, b):
			var ka: String = str(a).to_upper()
			var kb: String = str(b).to_upper()
			var da: String = Locale.ltr("TRAIT_KEY_" + ka)
			var db: String = Locale.ltr("TRAIT_KEY_" + kb)
			if da == "TRAIT_KEY_" + ka:
				da = str(a).replace("_", " ").capitalize()
			if db == "TRAIT_KEY_" + kb:
				db = str(b).replace("_", " ").capitalize()
			return da.naturalcasecmp_to(db) < 0
		)
		for key in rel_keys:
			var key_str: String = str(key)
			var value: float = float(relationship_totals[key_str])
			var display_key: String = Locale.ltr("TRAIT_KEY_" + key_str.to_upper())
			if display_key == "TRAIT_KEY_" + key_str.to_upper():
				display_key = key_str.replace("_", " ").capitalize()
			var pct: float = value * 100.0
			var sign: String = "+" if pct >= 0.0 else ""
			draw_string(font, Vector2(sub_indent, cy + 12), "%s: %s%.0f%%" % [display_key, sign, pct], HORIZONTAL_ALIGNMENT_LEFT, -1, fs, text_color)
			cy += 15.0

	if synergies.size() > 0:
		has_any = true
		draw_string(font, Vector2(indent, cy + 12), Locale.ltr("UI_TRAIT_SYNERGIES") + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.5, 1.0, 0.6))
		cy += 16.0
		for pair in synergies:
			var name_a: String = _get_trait_display_name(pair.get("a", ""), trait_defs)
			var name_b: String = _get_trait_display_name(pair.get("b", ""), trait_defs)
			draw_string(font, Vector2(sub_indent, cy + 12), "%s + %s" % [name_a, name_b], HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.4, 0.9, 0.5))
			cy += 15.0

	if conflicts.size() > 0:
		has_any = true
		draw_string(font, Vector2(indent, cy + 12), Locale.ltr("UI_TRAIT_ANTI_SYNERGIES") + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(1.0, 0.5, 0.5))
		cy += 16.0
		for pair in conflicts:
			var name_a: String = _get_trait_display_name(pair.get("a", ""), trait_defs)
			var name_b: String = _get_trait_display_name(pair.get("b", ""), trait_defs)
			draw_string(font, Vector2(sub_indent, cy + 12), "%s <-> %s" % [name_a, name_b], HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.9, 0.4, 0.4))
			cy += 15.0

	if not has_any:
		draw_string(font, Vector2(indent, cy + 12), Locale.ltr("UI_NONE"), HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.55, 0.55, 0.55))
		cy += 15.0

	return cy + 4.0


func _get_trait_display_name(trait_id: String, trait_defs: Array) -> String:
	for tdef in trait_defs:
		if tdef.get("id", "") != trait_id:
			continue
		var name_key: String = tdef.get("name_key", "TRAIT_" + trait_id + "_NAME")
		return Locale.ltr(name_key)
	return trait_id


func _draw() -> void:
	if not visible or _entity_manager == null or _entity_id < 0:
		return

	_click_regions.clear()
	_section_header_rects.clear()

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
	var job_label: String = Locale.tr_id("JOB", entity.job)
	var stage_label: String = Locale.tr_id("STAGE", entity.age_stage)
	var header_text: String = "%s %s - %s (%s)" % [gender_icon, entity.entity_name, job_label, stage_label]
	draw_string(font, Vector2(cx, cy), header_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), jc)
	# Gender icon colored separately
	draw_string(font, Vector2(cx, cy), gender_icon, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), gender_color)
	cy += 6.0

	var current_date: Dictionary = GameCalendarScript.tick_to_date(entity.birth_tick + entity.age)
	var ref_date: Dictionary = {"year": current_date.year, "month": current_date.month, "day": current_date.day}
	var age_detail: String = GameCalendarScript.format_age_detailed(entity.birth_date, ref_date)
	var sid_text: String = "S%d" % entity.settlement_id if entity.settlement_id > 0 else Locale.ltr("UI_NONE")
	var preg_text: String = ""
	if entity.pregnancy_tick >= 0:
		preg_text = "  |  %s" % Locale.ltr("UI_PREGNANT")
	# Birth date from birth_date dictionary.
	var birth_str: String = ""
	if not entity.birth_date.is_empty():
		birth_str = GameCalendarScript.format_birth_date(entity.birth_date)
	else:
		birth_str = Locale.ltr("UI_BIRTH_DATE_UNKNOWN")
	var life_stage_text: String = "%s | %s (%s)" % [stage_label, age_detail, birth_str]
	draw_string(font, Vector2(cx, cy + 14), "%s: %s  |  %s  |  %s: (%d, %d)%s" % [
		Locale.ltr("UI_SETTLEMENT"), sid_text,
		life_stage_text, Locale.ltr("UI_POS"),
		entity.position.x, entity.position.y, preg_text,
	], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 22.0
	_draw_separator(cx, cy, panel_w)
	cy += 10.0

	# ── Status ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_STATUS"), "status")
	if not _section_collapsed.get("status", false):
		var action_text: String = Locale.tr_id("STATUS", entity.current_action)
		if entity.action_target != Vector2i(-1, -1):
			action_text += " -> (%d, %d)" % [entity.action_target.x, entity.action_target.y]
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_ACTION"), action_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 16.0

		draw_string(font, Vector2(cx + 10, cy + 12), "%s: F:%.1f  W:%.1f  S:%.1f / %.0f" % [
			Locale.ltr("UI_INVENTORY"), entity.inventory.get("food", 0.0), entity.inventory.get("wood", 0.0),
			entity.inventory.get("stone", 0.0), GameConfig.MAX_CARRY,
		], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 22.0

	# ── Needs ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_NEEDS"), "needs")
	if not _section_collapsed.get("needs", false):
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("UI_HUNGER"), entity.hunger, Color(0.9, 0.2, 0.2))
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("UI_ENERGY"), entity.energy, Color(0.9, 0.8, 0.2))
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("UI_SOCIAL"), entity.social, Color(0.3, 0.5, 0.9))
		cy += 6.0

	# ── Personality (HEXACO 6-axis + expandable facets) ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_PERSONALITY"), "personality")

	var axis_labels: Dictionary = {
		"H": Locale.ltr("UI_HEXACO_H"),
		"E": Locale.ltr("UI_HEXACO_E"),
		"X": Locale.ltr("UI_HEXACO_X"),
		"A": Locale.ltr("UI_HEXACO_A"),
		"C": Locale.ltr("UI_HEXACO_C"),
		"O": Locale.ltr("UI_HEXACO_O"),
	}

	var pd = entity.personality
	if not _section_collapsed.get("personality", false) and pd != null:
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
					var fname: String = "    " + Locale.ltr("FACET_" + fk.to_upper())
					var dim_color: Color = Color(color.r * FACET_COLOR_DIM, color.g * FACET_COLOR_DIM, color.b * FACET_COLOR_DIM)
					cy = _draw_bar(font, cx + 25, cy, bar_w - 15, fname, fval, dim_color)
		cy += 4.0

	# ── Traits (independent section) ──
	if entity.personality != null:
		cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_TRAITS"), "traits")
		if not _section_collapsed.get("traits", false):
			cy = _draw_trait_section(font, cx, cy, entity.personality, entity)

	# ── Emotions (Plutchik 8) ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_EMOTIONS"), "emotions")
	if not _section_collapsed.get("emotions", false):
		if entity.emotion_data != null:
			var ed: RefCounted = entity.emotion_data
			# Draw 8 emotion bars with intensity labels
			for i in range(ed._emotion_order.size()):
				var emo_id: String = ed._emotion_order[i]
				var val: float = ed.get_emotion(emo_id) / 100.0  # Normalize to 0-1 for _draw_bar
				var display_label: String = ""
				if Locale.current_locale == "ko":
					var kr_label = ed.call("get_intensity_" + "label_" + "kr", emo_id)
					if kr_label != null:
						display_label = str(kr_label)
					if display_label == "":
						display_label = Locale.ltr("EMO_" + emo_id.to_upper())
				else:
					display_label = ed.get_intensity_label(emo_id)
					if display_label == "":
						display_label = Locale.ltr("EMO_" + emo_id.to_upper())
				cy = _draw_bar(font, cx + 10, cy, bar_w, display_label, val, EMOTION_COLORS.get(emo_id, Color.WHITE))

			# Valence-Arousal mood line
			cy += 4.0
			var va_text: String = "%s: %s %+.0f | %s %.0f" % [Locale.ltr("UI_MOOD"), Locale.ltr("UI_VALENCE"), ed.valence, Locale.ltr("UI_AROUSAL"), ed.arousal]
			draw_string(font, Vector2(cx + 10, cy + 12), va_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
			cy += 16.0

			# Active Dyads (threshold 30+)
			var active_dyads: Array = ed.get_active_dyads(30.0)
			if active_dyads.size() > 0:
				var dyad_x: float = cx + 10
				for di in range(active_dyads.size()):
					var dyad: Dictionary = active_dyads[di]
					var dyad_id: String = dyad.get("id", "")
					var dyad_text: String = Locale.ltr("DYAD_" + dyad_id.to_upper())
					if dyad_text == "DYAD_" + dyad_id.to_upper():
						dyad_text = dyad_id
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

			# Stress bar (display max: 1000, internal max: 2000)
			cy += 2.0
			var stress_val: float = ed.stress
			const STRESS_DISPLAY_MAX: float = 1000.0
			var stress_ratio: float = clampf(stress_val / STRESS_DISPLAY_MAX, 0.0, 1.0)
			var stress_label: String = "%s: %.0f" % [Locale.ltr("UI_STRESS"), stress_val]
			# Append state label if available
			if ed.stress_state == 1:
				stress_label += " [" + Locale.ltr("STRESS_STATE_TENSE") + "]"
			elif ed.stress_state == 2:
				stress_label += " [" + Locale.ltr("STRESS_STATE_CRISIS") + "]"
			elif ed.stress_state >= 3:
				stress_label += " [" + Locale.ltr("STRESS_STATE_BREAK_RISK") + "]"
			cy = _draw_bar(font, cx + 10, cy, bar_w, stress_label, stress_ratio, STRESS_COLOR)

			# Mental break indicator
			if ed.mental_break_type != "":
				var break_type_key: String = "MENTAL_BREAK_TYPE_" + ed.mental_break_type.to_upper()
				var break_type_name: String = Locale.ltr(break_type_key)
				var break_text: String = "%s: %s (%.1fh)" % [Locale.ltr("UI_MENTAL_BREAK"), break_type_name, ed.mental_break_remaining]
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

	# ── Trauma Scars ──
	var has_scars: bool = not entity.trauma_scars.is_empty()
	if has_scars:
		cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_TRAUMA_SCARS"), "trauma_scars")
		if not _section_collapsed.get("trauma_scars", true):
			for scar_entry in entity.trauma_scars:
				var scar_id: String = scar_entry.get("scar_id", "")
				var stacks: int = scar_entry.get("stacks", 1)
				var scar_name_key: String = "SCAR_" + scar_id
				var scar_name: String = Locale.ltr(scar_name_key)
				var scar_text: String = scar_name
				if stacks > 1:
					scar_text += " x%d" % stacks
				draw_string(font, Vector2(cx + 10, cy + 12), scar_text,
					HORIZONTAL_ALIGNMENT_LEFT, -1,
					GameConfig.get_font_size("popup_body"), Color(0.9, 0.4, 0.8))
				cy += 16.0
			cy += 4.0

	# ── Violation History (notable desensitization/PTSD only) ──
	if "violation_history" in entity and not entity.violation_history.is_empty():
		var notable: Array = []
		var action_ids: Array = entity.violation_history.keys()
		action_ids.sort()
		for action_id in action_ids:
			var record: Dictionary = entity.violation_history.get(action_id, {})
			var dm: float = float(record.get("desensitize_mult", 1.0))
			var pm: float = float(record.get("ptsd_mult", 1.0))
			if dm < 0.7:
				notable.append({"action": str(action_id), "type": "desensitized"})
			elif pm > 1.4:
				notable.append({"action": str(action_id), "type": "ptsd"})
		if not notable.is_empty():
			cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_VIOLATION_HISTORY"), "violation_history")
			if not _section_collapsed.get("violation_history", false):
				for item in notable:
					var action_text: String = str(item.get("action", ""))
					var item_type: String = str(item.get("type", ""))
					var action_label: String = Locale.tr_id("STATUS", action_text)
					var label_key: String = "UI_VIOLATION_DESENSITIZED_LABEL" if item_type == "desensitized" else "UI_VIOLATION_PTSD_LABEL"
					var line_color: Color = Color(0.6, 0.6, 0.6) if item_type == "desensitized" else Color(0.9, 0.5, 0.2)
					var line_text: String = "%s - %s" % [action_label, Locale.ltr(label_key)]
					draw_string(font, Vector2(cx + 10, cy + 12), line_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), line_color)
					cy += 16.0
				cy += 4.0

	# ── Family ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_FAMILY"), "family")
	if not _section_collapsed.get("family", false):
		# Partner
		if entity.partner_id >= 0:
			var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
			var partner_name: String = "☠ (%s)" % Locale.ltr("UI_DEAD")
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
			var prefix: String = "%s: " % Locale.ltr("UI_PARTNER")
			draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
			var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			var name_pos := Vector2(cx + 10 + prefix_w, cy + 12)
			draw_string(font, name_pos, partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
			_register_click_region(name_pos, partner_name, entity.partner_id, font, GameConfig.get_font_size("popup_body"))
			var name_w: float = font.get_string_size(partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			# Love + Compatibility suffix
			var suffix: String = " (%s: %d%%" % [Locale.ltr("UI_LOVE"), love_pct]
			if entity.personality != null and partner_alive and partner != null and partner.personality != null:
				var compat: float = PersonalitySystem.personality_compatibility(entity.personality, partner.personality)
				var compat_pct: int = int((compat + 1.0) / 2.0 * 100)
				suffix += ", %s: %d%%" % [Locale.ltr("UI_COMPAT"), compat_pct]
			suffix += ")"
			draw_string(font, Vector2(cx + 10 + prefix_w + name_w, cy + 12), suffix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
			cy += 16.0
		else:
			draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_PARTNER"), Locale.ltr("UI_NONE")], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
			cy += 16.0

		# Parents
		if entity.parent_ids.size() > 0:
			var prefix: String = "%s: " % Locale.ltr("UI_PARENTS")
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
			draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_PARENTS"), Locale.ltr("UI_FIRST_GENERATION")], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
			cy += 16.0

		# Children
		if entity.children_ids.size() > 0:
			var prefix: String = "%s: " % Locale.ltr("UI_CHILDREN")
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
		cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_KEY_RELATIONSHIPS"), "relationships")
		if not _section_collapsed.get("relationships", false):
			var rels: Array = _relationship_manager.get_relationships_for(entity.id)
			if rels.size() == 0:
				draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_NO_RELATIONSHIPS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
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
					var rel_label: String = " - %s (%s:%d %s:%d)" % [Locale.tr_id("RELATION", rel.type), Locale.ltr("UI_AFFINITY"), int(rel.affinity), Locale.ltr("UI_TRUST"), int(rel.trust)]
					if rel.romantic_interest > 0.0:
						rel_label += " %s:%d" % [Locale.ltr("UI_RELATIONSHIP_SCORE"), int(rel.romantic_interest)]
					draw_string(font, Vector2(cx + 10 + name_w, cy + 12), rel_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), type_color)
					cy += 15.0
			cy += 6.0

	# ── Stats ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_STATS_SECTION"), "stats")
	if not _section_collapsed.get("stats", false):
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %.1f  |  %s: %.1f" % [Locale.ltr("UI_SPEED"), entity.speed, Locale.ltr("UI_STRENGTH"), entity.strength], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %.0f  |  %s: %d" % [Locale.ltr("UI_TOTAL_GATHERED"), entity.total_gathered, Locale.ltr("UI_BUILDINGS_BUILT"), entity.buildings_built], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 22.0

	# ── Action History ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_RECENT_ACTIONS"), "recent_actions")
	if not _section_collapsed.get("recent_actions", false):
		var hist: Array = entity.action_history
		var idx: int = hist.size() - 1
		var hist_count: int = 0
		while idx >= 0 and hist_count < 5:
			var entry: Dictionary = hist[idx]
			var act_d: Dictionary = GameCalendarScript.tick_to_date(entry.tick)
			var date_str: String = ""
			if act_d.year == current_date.year:
				date_str = GameCalendarScript.format_short_datetime(entry.tick)
			else:
				date_str = GameCalendarScript.format_short_datetime_with_year(entry.tick)
			draw_string(font, Vector2(cx + 10, cy + 11), "%s: %s" % [date_str, Locale.tr_id("STATUS", entry.action)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.6, 0.6, 0.6))
			cy += 13.0
			idx -= 1
			hist_count += 1

	# Track content height for scrolling
	_content_height = cy + _scroll_offset + 20.0

	# Footer
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), Locale.ltr("UI_SCROLL_HINT"), HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))
	_draw_scrollbar()


func _draw_scrollbar() -> void:
	# Only show when content overflows
	if _content_height <= size.y:
		_scrollbar_rect = Rect2()
		return

	var panel_h: float = size.y
	var scrollbar_width: float = 6.0
	var scrollbar_margin: float = 3.0
	var scrollbar_x: float = size.x - scrollbar_width - scrollbar_margin
	var track_top: float = 4.0
	var track_bottom: float = panel_h - 4.0
	var track_height: float = track_bottom - track_top

	# Store track rect for drag input (wider hit area)
	_scrollbar_rect = Rect2(scrollbar_x - 4.0, track_top, scrollbar_width + 8.0, track_height)

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


func _draw_section_header(font: Font, x: float, y: float, title: String, section_id: String = "") -> float:
	var display_title: String = title
	if section_id != "":
		var arrow: String = "▶" if _section_collapsed.get(section_id, false) else "▼"
		display_title = arrow + " " + title
		# Register full-width rect for click detection
		_section_header_rects[section_id] = Rect2(x, y, size.x - x - 8.0, 18.0)
	draw_string(font, Vector2(x, y + 12), display_title, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
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
			return record.get("name", Locale.ltr("UI_UNKNOWN")) + " ☠"
	return Locale.ltr("UI_UNKNOWN")


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
	var header: String = "%s %s %s" % [gender_icon, r.get("name", Locale.ltr("UI_UNKNOWN")), Locale.ltr("UI_DECEASED_HEADER")]
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
		birth_str = "Y%d M%d D%d" % [bd.year, bd.month, bd.day]
	else:
		birth_str = Locale.ltr("UI_FIRST_GENERATION")
	var dd: Dictionary = GameCalendarScript.tick_to_date(death_tick)
	death_str = "Y%d M%d D%d" % [dd.year, dd.month, dd.day]
	var age_str: String = "?"
	if not birth_date.is_empty() and not death_date.is_empty():
		age_str = GameCalendarScript.format_age_detailed(birth_date, death_date)
	else:
		age_str = "%.0f%s" % [r.get("death_age_years", 0.0), Locale.ltr("UI_AGE_YEARS_UNIT")]
	var age_at_death_text: String = Locale.trf("UI_AGE_AT_DEATH_FMT", {"age": age_str})
	draw_string(font, Vector2(cx, cy + 14), "%s ~ %s (%s)" % [birth_str, death_str, age_at_death_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
	cy += 18.0

	if not birth_date.is_empty() and not death_date.is_empty():
		var survival: Dictionary = GameCalendarScript.calculate_detailed_age(birth_date, death_date)
		var total_days_str: String = GameCalendarScript.format_number(survival.total_days)
		var dur_parts: Array = []
		if survival.years > 0:
			dur_parts.append("%d %s" % [survival.years, Locale.ltr("UI_AGE_YEARS_UNIT")])
		if survival.months > 0:
			dur_parts.append("%d%s" % [survival.months, Locale.ltr("UI_AGE_MONTHS")])
		dur_parts.append("%d%s" % [survival.days, Locale.ltr("UI_AGE_DAYS")])
		draw_string(font, Vector2(cx, cy + 14), "%s: %s (%s%s)" % [Locale.ltr("UI_SURVIVAL_DURATION"), " ".join(dur_parts), total_days_str, Locale.ltr("UI_AGE_DAYS")], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
		cy += 18.0

	# Cause of death
	var cause_raw: String = r.get("death_cause", "unknown")
	var cause_display: String = Locale.tr_id("DEATH", cause_raw)
	draw_string(font, Vector2(cx, cy + 14), "%s: %s" % [Locale.ltr("UI_CAUSE_OF_DEATH"), cause_display], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.5, 0.5))
	cy += 18.0
	_draw_separator(cx, cy, panel_w)
	cy += 10.0

	# Job and settlement
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_INFO"))
	var job_id: String = str(r.get("job", "none"))
	var job_text: String = Locale.tr_id("JOB", job_id)
	var stage_id: String = str(r.get("age_stage", ""))
	var stage_text: String = Locale.tr_id("STAGE", stage_id) if stage_id != "" else Locale.ltr("UI_UNKNOWN")
	draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s | %s: %s" % [Locale.ltr("UI_JOB"), job_text, Locale.ltr("UI_STAGE"), stage_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 16.0
	draw_string(font, Vector2(cx + 10, cy + 12), "%s: %.0f | %s: %d" % [Locale.ltr("UI_GATHERED_LABEL"), r.get("total_gathered", 0.0), Locale.ltr("UI_BUILT_LABEL"), r.get("buildings_built", 0)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 22.0

	# Family (clickable)
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_FAMILY"))

	# Partner
	var partner_id: int = r.get("partner_id", -1)
	if partner_id > 0:
		var prefix: String = "%s: " % Locale.ltr("UI_PARTNER")
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
		var prefix: String = "%s: " % Locale.ltr("UI_PARENTS")
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
		var prefix: String = "%s: " % Locale.ltr("UI_CHILDREN")
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
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_PERSONALITY"))
	var PersonalityDataScript = load("res://scripts/core/personality_data.gd")
	var p_dict: Dictionary = r.get("personality", {})
	var pd: RefCounted
	if p_dict.has("facets"):
		pd = PersonalityDataScript.from_dict(p_dict)
	else:
		pd = PersonalityDataScript.new()
		pd.migrate_from_big_five(p_dict)
	var axis_labels: Dictionary = {
		"H": Locale.ltr("UI_HEXACO_H"),
		"E": Locale.ltr("UI_HEXACO_E"),
		"X": Locale.ltr("UI_HEXACO_X"),
		"A": Locale.ltr("UI_HEXACO_A"),
		"C": Locale.ltr("UI_HEXACO_C"),
		"O": Locale.ltr("UI_HEXACO_O"),
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
				var fname: String = "    " + Locale.ltr("FACET_" + fk.to_upper())
				var dim_color: Color = Color(color.r * FACET_COLOR_DIM, color.g * FACET_COLOR_DIM, color.b * FACET_COLOR_DIM)
				cy = _draw_bar(font, cx + 25, cy, bar_w - 15, fname, fval, dim_color)
	cy += 4.0

	# ── Traits ──
	cy = _draw_trait_section(font, cx, cy, pd)

	# Chronicle events
	var chronicle: Node = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle != null:
		cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_LIFE_EVENTS"))
		var events: Array = chronicle.get_personal_events(r.get("id", -1))
		var show_count: int = mini(8, events.size())
		if show_count == 0:
			draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_NO_RECORDED_EVENTS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
			cy += 16.0
		else:
			var idx: int = events.size() - 1
			var count: int = 0
			while idx >= 0 and count < show_count:
				var evt: Dictionary = events[idx]
				var desc: String
				if evt.has("l10n_key") and not evt.get("l10n_key", "").is_empty():
					var l10n_key: String = evt.get("l10n_key", "")
					var l10n_params: Dictionary = evt.get("l10n_params", {})
					desc = Locale.trf(l10n_key, l10n_params)
				else:
					desc = evt.get("description", "?")
				if desc.length() > 50:
					desc = desc.substr(0, 47) + "..."
				draw_string(font, Vector2(cx + 10, cy + 11), "Y%d M%d: %s" % [evt.get("year", 0), evt.get("month", 0), desc], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.6, 0.6, 0.6))
				cy += 13.0
				idx -= 1
				count += 1

	_content_height = cy + _scroll_offset + 20.0
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), Locale.ltr("UI_SCROLL_HINT"), HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))
	_draw_scrollbar()
