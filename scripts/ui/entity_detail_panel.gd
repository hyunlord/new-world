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
var _deceased_proxy: RefCounted = null
## Section collapsed states (true = collapsed/hidden)
var _section_collapsed: Dictionary = {
	"status": false,
	"needs": false,
	"personality": true,
	"traits": false,
	"emotions": true,
	"trauma_scars": true,
	"violation_history": false,
	"childhood": true,
	"family": false,
	"relationships": false,
	"stats": true,
	"recent_actions": false,
	"life_events": false,
	"values": true,
}
## Section header rects for click detection (cleared each _draw frame)
var _section_header_rects: Dictionary = {}


class DeceasedEntityProxy extends RefCounted:
	# Living entity interface fields
	var id: int
	var entity_name: String
	var gender: String
	var age_stage: String
	var job: String
	var birth_tick: int
	var birth_date: Dictionary
	var settlement_id: int
	var age: int
	var is_alive: bool = false
	var partner_id: int = -1
	var parent_ids: Array = []
	var children_ids: Array = []
	var hunger: float
	var energy: float
	var social: float
	var thirst: float
	var warmth: float
	var safety: float
	var current_action: String
	var inventory: Dictionary = {}
	var action_target: Vector2i = Vector2i(-1, -1)
	var action_history: Array = []
	var pregnancy_tick: int = -1
	var position: Vector2 = Vector2.ZERO
	var display_traits: Array = []
	var trauma_scars: Array = []
	var violation_history: Dictionary = {}
	var values: Dictionary = {}
	var moral_stage: int = 1
	var value_violation_count: Dictionary = {}
	var speed: float = 1.0
	var strength: float = 1.0
	var total_gathered: float = 0.0
	var buildings_built: int = 0
	var personality: RefCounted
	var emotion_data: RefCounted

	# Deceased-only fields
	var death_cause: String
	var death_date: Dictionary
	var death_age_years: float
	var death_tick: int

	func _init(record: Dictionary) -> void:
		id = record.get("id", -1)
		entity_name = record.get("name", "")
		gender = record.get("gender", "")
		age_stage = record.get("age_stage", "adult")
		job = record.get("job", "none")
		birth_tick = record.get("birth_tick", 0)
		birth_date = record.get("birth_date", {}).duplicate()
		settlement_id = record.get("settlement_id", -1)
		death_cause = record.get("death_cause", "")
		death_date = record.get("death_date", {}).duplicate()
		death_age_years = record.get("death_age_years", 0.0)
		death_tick = record.get("death_tick", 0)
		age = death_tick - birth_tick
		hunger = record.get("hunger", 0.0)
		energy = record.get("energy", 0.0)
		social = record.get("social", 0.0)
		thirst = record.get("thirst", 0.85)
		warmth = record.get("warmth", 0.90)
		safety = record.get("safety", 0.60)
		current_action = str(record.get("current_action", "idle"))
		inventory = record.get("inventory", {}).duplicate()
		display_traits = record.get("display_traits", []).duplicate()
		trauma_scars = record.get("trauma_scars", []).duplicate()
		violation_history = record.get("violation_history", {}).duplicate()
		speed = record.get("speed", 1.0)
		strength = record.get("strength", 1.0)
		total_gathered = record.get("total_gathered", 0.0)
		buildings_built = record.get("buildings_built", 0)
		partner_id = record.get("partner_id", -1)
		parent_ids = record.get("parent_ids", []).duplicate()
		children_ids = record.get("children_ids", []).duplicate()

		# Reconstruct PersonalityData
		var p_dict: Dictionary = record.get("personality", {})
		if not p_dict.is_empty():
			var PScript = load("res://scripts/core/personality_data.gd")
			if p_dict.has("facets"):
				personality = PScript.from_dict(p_dict)
			else:
				personality = PScript.new()
				personality.migrate_from_big_five(p_dict)

		# Reconstruct EmotionData
		var e_dict: Dictionary = record.get("emotion_data", {})
		if not e_dict.is_empty():
			var EScript = load("res://scripts/core/emotion_data.gd")
			emotion_data = EScript.from_dict(e_dict)


func init(entity_manager: RefCounted, building_manager: RefCounted = null, relationship_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_relationship_manager = relationship_manager


func set_entity_id(id: int) -> void:
	_entity_id = id
	_scroll_offset = 0.0
	_showing_deceased = false
	_deceased_record = {}
	_deceased_proxy = null
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
		if salience > 0.0 and salience < 0.995:
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
		var tdef_sal: Dictionary = tdef.duplicate()
		tdef_sal["_salience"] = salience
		_trait_badge_regions.append({"rect": badge_rect, "trait_def": tdef_sal})
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

	var entity: RefCounted
	if _showing_deceased and _deceased_proxy != null:
		entity = _deceased_proxy
	else:
		entity = _entity_manager.get_entity(_entity_id)
		if entity == null or not entity.is_alive:
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
	var bg_color: Color = Color(0.06, 0.1, 0.06, 0.95)
	var border_color: Color = Color(0.3, 0.4, 0.3)
	if not entity.is_alive:
		bg_color = Color(0.08, 0.07, 0.07, 0.95)
		border_color = Color(0.35, 0.3, 0.28)
	draw_rect(Rect2(0, 0, panel_w, panel_h), bg_color)
	draw_rect(Rect2(0, 0, panel_w, panel_h), border_color, false, 1.0)

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
	if not entity.is_alive:
		jc = Color(0.55, 0.52, 0.50)

	# ── Header ──
	var gender_icon: String = "M" if entity.gender == "male" else "F"
	var gender_color: Color = GENDER_COLORS.get(entity.gender, Color.WHITE)
	var job_label: String = Locale.tr_id("JOB", entity.job)
	var stage_label: String = Locale.tr_id("STAGE", entity.age_stage)
	var name_prefix: String = "✝ " if not entity.is_alive else ""
	var header_text: String = "%s %s%s - %s (%s)" % [gender_icon, name_prefix, entity.entity_name, job_label, stage_label]
	draw_string(font, Vector2(cx, cy), header_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), jc)
	# Gender icon colored separately
	draw_string(font, Vector2(cx, cy), gender_icon, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), gender_color)
	cy += 6.0

	var current_date: Dictionary = GameCalendarScript.tick_to_date(entity.birth_tick + entity.age)
	var ref_date: Dictionary = {"year": current_date.year, "month": current_date.month, "day": current_date.day}
	var age_detail: String = GameCalendarScript.format_age_detailed(entity.birth_date, ref_date)
	var sid_text: String = "S%d" % entity.settlement_id if entity.settlement_id > 0 else Locale.ltr("UI_NONE")
	var birth_str: String = ""
	if not entity.birth_date.is_empty():
		birth_str = GameCalendarScript.format_birth_date(entity.birth_date)
	else:
		birth_str = Locale.ltr("UI_BIRTH_DATE_UNKNOWN")
	var life_stage_text: String = "%s | %s (%s)" % [stage_label, age_detail, birth_str]
	if entity.is_alive:
		var preg_text: String = ""
		if entity.pregnancy_tick >= 0:
			preg_text = "  |  %s" % Locale.ltr("UI_PREGNANT")
		draw_string(font, Vector2(cx, cy + 14), "%s: %s  |  %s  |  %s: (%d, %d)%s" % [
			Locale.ltr("UI_SETTLEMENT"), sid_text,
			life_stage_text, Locale.ltr("UI_POS"),
			entity.position.x, entity.position.y, preg_text,
		], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
		cy += 22.0
	else:
		draw_string(font, Vector2(cx, cy + 14), "%s: %s  |  %s" % [
			Locale.ltr("UI_SETTLEMENT"), sid_text, life_stage_text,
		], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
		cy += 16.0
		# Death date + cause
		var dd: Dictionary = entity.death_date
		draw_string(font, Vector2(cx, cy + 12),
			"✝ %d/%02d/%02d  (%.1f yr)" % [dd.get("year", 0), dd.get("month", 1), dd.get("day", 1), entity.death_age_years],
			HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.65, 0.55, 0.45))
		cy += 14.0
		var cause_raw: String = entity.death_cause
		var cause_display: String = Locale.tr_id("DEATH", cause_raw)
		if cause_display == cause_raw:
			cause_display = cause_raw.capitalize().replace("_", " ")
		draw_string(font, Vector2(cx, cy + 12),
			"%s: %s" % [Locale.ltr("UI_CAUSE_OF_DEATH"), cause_display],
			HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.5, 0.4))
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
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("NEED_THIRST"), entity.thirst, Color(0.392, 0.710, 0.965))
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("UI_ENERGY"), entity.energy, Color(0.9, 0.8, 0.2))
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("NEED_WARMTH"), entity.warmth, Color(1.0, 0.541, 0.396))
		cy = _draw_bar(font, cx + 10, cy, bar_w, Locale.ltr("NEED_SAFETY"), entity.safety, Color(0.584, 0.459, 0.804))
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

	# ── Values (가치관) ──
	cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_VALUES"), "values")
	if not _section_collapsed.get("values", true):
		if not entity.values.is_empty():
			var significant: Array = []
			for vkey in entity.values:
				var val: float = entity.values[vkey]
				if absf(val) > 0.30:
					significant.append({ "key": vkey, "value": val })
			significant.sort_custom(func(a, b):
				return absf(a["value"]) > absf(b["value"])
			)
			for item in significant:
				var v_key: String = item["key"]
				var v_val: float = item["value"]
				var display_name: String = Locale.ltr("VALUE_" + v_key)
				var bar_color: Color = Color(0.4, 0.7, 1.0) if v_val > 0 else Color(1.0, 0.45, 0.45)
				cy = _draw_bar(font, cx + 10, cy, bar_w, display_name, (v_val + 1.0) / 2.0, bar_color)
		if "moral_stage" in entity:
			var moral_stage_label: String = Locale.ltr("VALUE_MORAL_STAGE") + ": %d" % entity.moral_stage
			draw_string(font, Vector2(cx + 10, cy + 12), moral_stage_label,
				HORIZONTAL_ALIGNMENT_LEFT, bar_w, 11, Color(0.7, 0.7, 0.7))
			cy += 18
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

	# ── Childhood (Phase 5: ACE / Attachment / Epigenetic) ──
	var has_childhood_data: bool = (
		entity.has_meta("adulthood_applied")
		or entity.has_meta("childhood_data")
		or entity.has_meta("ace_score_total")
	)
	if has_childhood_data:
		cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_PANEL_CHILDHOOD"), "childhood")
		if not _section_collapsed.get("childhood", true):
			# ── Developmental stage (children only) ──
			var childhood_data = entity.get_meta("childhood_data", null)
			if childhood_data is Dictionary:
				var stage = childhood_data.get("current_stage", "")
				if stage != "" and stage != "adult":
					var stage_key: String = "STAGE_" + stage.to_upper()
					var childhood_stage_label: String = Locale.ltr(stage_key)
					draw_string(font, Vector2(cx + 10, cy + 12), childhood_stage_label,
						HORIZONTAL_ALIGNMENT_LEFT, -1,
						GameConfig.get_font_size("popup_body"), Color(0.7, 0.9, 0.7))
					cy += 16.0

			# ── ACE score bar (3-segment color: 0~3 green, 4~6 yellow, 7~10 red) ──
			var ace_tracker = entity.get_meta("ace_tracker", null)
			var ace_score: float = 0.0
			var ace_threat: float = 0.0
			var ace_deprivation: float = 0.0
			var is_backfilled: bool = false
			if ace_tracker != null:
				if ace_tracker.has_method("get_score_total"):
					ace_score = float(ace_tracker.get_score_total())
				elif "ace_score_total" in ace_tracker:
					ace_score = float(ace_tracker.ace_score_total)
				if ace_tracker.has_method("get_threat_deprivation_scores"):
					var td = ace_tracker.get_threat_deprivation_scores()
					ace_threat = float(td.get("threat", 0.0))
					ace_deprivation = float(td.get("deprivation", 0.0))
				if "is_backfilled" in ace_tracker:
					is_backfilled = bool(ace_tracker.is_backfilled)
			else:
				ace_score = float(entity.get_meta("ace_score_total", 0.0))

			var ace_ratio: float = clampf(ace_score / 10.0, 0.0, 1.0)
			var ace_color: Color
			if ace_score <= 3.0:
				ace_color = Color(0.2, 0.8, 0.3)
			elif ace_score <= 6.0:
				ace_color = Color(0.9, 0.8, 0.1)
			else:
				ace_color = Color(0.9, 0.2, 0.2)

			var ace_range_key: String = "ACE_SCORE_LOW"
			if ace_score > 6.0:
				ace_range_key = "ACE_SCORE_HIGH"
			elif ace_score > 3.0:
				ace_range_key = "ACE_SCORE_MID"

			var ace_label: String = "%s: %.1f/10 (%s)" % [
				Locale.ltr("UI_PANEL_ACE"),
				ace_score,
				Locale.ltr(ace_range_key)
			]
			if is_backfilled:
				ace_label += " [%s]" % Locale.ltr("UI_ACE_BACKFILL_ESTIMATED")
			cy = _draw_bar(font, cx + 10, cy, bar_w, ace_label, ace_ratio, ace_color)

			# threat/deprivation sub-bars (indented)
			if ace_threat > 0.0 or ace_deprivation > 0.0:
				var threat_ratio: float = clampf(ace_threat / 10.0, 0.0, 1.0)
				var dep_ratio: float = clampf(ace_deprivation / 10.0, 0.0, 1.0)
				cy = _draw_bar(font, cx + 20, cy, bar_w - 10,
					Locale.ltr("UI_ACE_THREAT_LABEL"), threat_ratio, Color(0.9, 0.4, 0.2))
				cy = _draw_bar(font, cx + 20, cy, bar_w - 10,
					Locale.ltr("UI_ACE_DEPRIVATION_LABEL"), dep_ratio, Color(0.4, 0.5, 0.9))

			# ── Attachment type ──
			var attachment_type = entity.get_meta("attachment_type", "")
			if str(attachment_type) != "":
				var attach_key: String = "ATTACHMENT_" + str(attachment_type).to_upper()
				var attach_label: String = "%s: %s" % [
					Locale.ltr("UI_PANEL_ATTACHMENT"),
					Locale.ltr(attach_key)
				]
				var attach_color: Color = Color(0.5, 0.8, 0.9)
				if str(attachment_type) == "anxious":
					attach_color = Color(0.9, 0.7, 0.2)
				elif str(attachment_type) == "avoidant":
					attach_color = Color(0.6, 0.6, 0.6)
				elif str(attachment_type) == "disorganized":
					attach_color = Color(0.9, 0.3, 0.5)
				draw_string(font, Vector2(cx + 10, cy + 12), attach_label,
					HORIZONTAL_ALIGNMENT_LEFT, -1,
					GameConfig.get_font_size("popup_body"), attach_color)
				cy += 16.0

			# ── Epigenetic load bar ──
			var epi_load = entity.get_meta("epigenetic_load_effective", -1.0)
			if float(epi_load) >= 0.0:
				var epi_ratio: float = clampf(float(epi_load), 0.0, 1.0)
				cy = _draw_bar(font, cx + 10, cy, bar_w,
					Locale.ltr("UI_PANEL_EPIGENETIC"), epi_ratio, Color(0.6, 0.3, 0.8))

			# ── Parental epigenetic lineage (Yehuda 2016 — attenuated transmission) ──
			var father_epi: float = float(entity.get_meta("parent_epi_father", -1.0))
			var mother_epi: float = float(entity.get_meta("parent_epi_mother", -1.0))
			if father_epi >= 0.0 or mother_epi >= 0.0:
				var lineage_parts: Array = []
				if father_epi >= 0.0:
					lineage_parts.append("♂ %.0f%%" % (father_epi * 100.0))
				if mother_epi >= 0.0:
					lineage_parts.append("♀ %.0f%%" % (mother_epi * 100.0))
				var lineage_text: String = "%s: %s" % [Locale.ltr("UI_PARENTAL_ORIGIN"), ", ".join(lineage_parts)]
				draw_string(font, Vector2(cx + 20, cy + 12), lineage_text,
					HORIZONTAL_ALIGNMENT_LEFT, -1,
					GameConfig.get_font_size("popup_body"), Color(0.55, 0.35, 0.75))
				cy += 16.0

			# ── HPA/Break multipliers (adults only) ──
			var adulthood_applied = entity.get_meta("adulthood_applied", false)
			if bool(adulthood_applied):
				var stress_mult = entity.get_meta("ace_stress_gain_mult", 1.0)
				var break_mult = entity.get_meta("ace_break_threshold_mult", 1.0)
				var hpa_text: String = "HPA x%.2f  |  Break x%.2f" % [float(stress_mult), float(break_mult)]
				var hpa_color: Color = Color(0.7, 0.7, 0.7)
				if float(stress_mult) > 1.6:
					hpa_color = Color(0.9, 0.2, 0.2)
				elif float(stress_mult) > 1.3:
					hpa_color = Color(0.9, 0.5, 0.2)
				draw_string(font, Vector2(cx + 10, cy + 12), hpa_text,
					HORIZONTAL_ALIGNMENT_LEFT, -1,
					GameConfig.get_font_size("popup_body"), hpa_color)
				cy += 16.0

				# ── HEXACO cap modifications (Teicher & Samson 2016 — permanent brain changes) ──
				# Iterate entity meta for hexaco_cap_* keys and display each modification
				if bool(adulthood_applied):
					var cap_metas: Array = entity.get_meta_list()
					var cap_lines: Array = []
					for meta_key in cap_metas:
						var key_str: String = str(meta_key)
						if not key_str.begins_with("hexaco_cap_"):
							continue
						var facet_id: String = key_str.substr("hexaco_cap_".length())
						var cap_data = entity.get_meta(meta_key, {})
						if typeof(cap_data) != TYPE_DICTIONARY:
							continue
						var line_parts: Array = []
						if cap_data.has("min"):
							line_parts.append("%s +%.0f ▲" % [Locale.ltr("UI_MIN"), (float(cap_data.get("min", 0.0)) * 100.0)])
						if cap_data.has("max"):
							line_parts.append("%s -%.0f ▼" % [Locale.ltr("UI_MAX"), ((1.0 - float(cap_data.get("max", 1.0))) * 100.0)])
						if not line_parts.is_empty():
							cap_lines.append("  %s: %s" % [facet_id, ", ".join(line_parts)])
					if not cap_lines.is_empty():
						var cap_header: String = Locale.ltr("UI_HEXACO_CAP_MODIFIED")
						draw_string(font, Vector2(cx + 10, cy + 12), cap_header,
							HORIZONTAL_ALIGNMENT_LEFT, -1,
							GameConfig.get_font_size("popup_body"), Color(0.9, 0.7, 0.3))
						cy += 16.0
						for cap_line in cap_lines:
							draw_string(font, Vector2(cx + 10, cy + 12), cap_line,
								HORIZONTAL_ALIGNMENT_LEFT, -1,
								GameConfig.get_font_size("popup_body"), Color(0.7, 0.6, 0.4))
							cy += 14.0

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
			elif entity.is_alive and "emotions" in entity:
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

	# ── Action History (alive only) ──
	if entity.is_alive:
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

	# ── Life Events (deceased only) ──
	if not entity.is_alive:
		var chronicle: Node = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
		if chronicle != null:
			cy = _draw_section_header(font, cx, cy, Locale.ltr("UI_LIFE_EVENTS"), "life_events")
			if not _section_collapsed.get("life_events", false):
				var events: Array = chronicle.get_personal_events(entity.id)
				var show_count: int = mini(8, events.size())
				if show_count == 0:
					draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_NO_RECORDED_EVENTS"),
						HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
					cy += 16.0
				else:
					var ev_idx: int = events.size() - 1
					var count: int = 0
					while ev_idx >= 0 and count < show_count:
						var evt: Dictionary = events[ev_idx]
						var desc: String
						if evt.has("l10n_key") and not evt.get("l10n_key", "").is_empty():
							var l10n_key: String = evt.get("l10n_key", "")
							var l10n_params_final: Dictionary = evt.get("l10n_params", {}).duplicate()
							desc = Locale.trf(l10n_key, l10n_params_final)
						else:
							desc = evt.get("description", "?")
						if desc.length() > 50:
							desc = desc.substr(0, 47) + "..."
						draw_string(font, Vector2(cx + 10, cy + 11),
							"Y%d M%d: %s" % [evt.get("year", 0), evt.get("month", 0), desc],
							HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.6, 0.6, 0.6))
						cy += 13.0
						ev_idx -= 1
						count += 1

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


## Draw a bipolar bar for values in range [-1.0, +1.0]
## Center = 0, right half = positive (pos_color), left half = negative (neg_color)
func _draw_value_bar(font: Font, x: float, y: float, w: float, label: String, value: float, pos_color: Color, neg_color: Color) -> float:
	var label_w: float = 140.0
	var pct_w: float = 50.0
	var bar_gap: float = 4.0
	var bar_h: float = 10.0
	draw_string(font, Vector2(x, y + 11), label, HORIZONTAL_ALIGNMENT_LEFT, int(label_w), GameConfig.get_font_size("bar_label"), Color(0.7, 0.7, 0.7))
	var bar_x: float = x + label_w + bar_gap
	var bar_w_actual: float = maxf(w - label_w - pct_w - bar_gap * 2, 20.0)
	var center_x: float = bar_x + bar_w_actual * 0.5
	draw_rect(Rect2(bar_x, y + 2, bar_w_actual, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	draw_line(Vector2(center_x, y + 2), Vector2(center_x, y + 2 + bar_h), Color(0.5, 0.5, 0.5, 0.5), 1.0)
	if absf(value) > 0.001:
		var fill_w: float = (bar_w_actual * 0.5) * absf(value)
		var fill_color: Color = pos_color if value > 0.0 else neg_color
		var fill_x: float = center_x if value > 0.0 else center_x - fill_w
		draw_rect(Rect2(fill_x, y + 2, fill_w, bar_h), fill_color)
	var pct_x: float = bar_x + bar_w_actual + bar_gap
	draw_string(font, Vector2(pct_x, y + 11), "%+d%%" % int(value * 100.0), HORIZONTAL_ALIGNMENT_RIGHT, int(pct_w), GameConfig.get_font_size("bar_label"), Color(0.8, 0.8, 0.8))
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
	_deceased_proxy = DeceasedEntityProxy.new(record)
	queue_redraw()


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
