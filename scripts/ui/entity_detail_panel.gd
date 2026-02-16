class_name EntityDetailPanel
extends Control

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _relationship_manager: RefCounted
var _entity_id: int = -1

## Personality bar colors
const PERSONALITY_COLORS: Dictionary = {
	"openness": Color(0.6, 0.4, 0.9),
	"agreeableness": Color(0.3, 0.8, 0.5),
	"extraversion": Color(0.9, 0.7, 0.2),
	"diligence": Color(0.2, 0.6, 0.9),
	"emotional_stability": Color(0.4, 0.8, 0.8),
}

## Emotion bar colors
const EMOTION_COLORS: Dictionary = {
	"happiness": Color(0.9, 0.8, 0.2),
	"loneliness": Color(0.4, 0.4, 0.7),
	"stress": Color(0.9, 0.5, 0.2),
	"grief": Color(0.5, 0.3, 0.5),
	"love": Color(0.9, 0.3, 0.4),
}

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
	var bar_w: float = panel_w - 80.0

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

	var age_years: float = GameConfig.get_age_years(entity.age)
	var sid_text: String = "S%d" % entity.settlement_id if entity.settlement_id > 0 else "None"
	var preg_text: String = ""
	if entity.pregnancy_tick >= 0:
		preg_text = "  |  Pregnant"
	# Birth date from birth_tick: "23세 (Y3 7월 15일생)" or "23세 (초기세대)"
	var birth_str: String = ""
	if entity.birth_tick >= 0:
		var GameCalendar = load("res://scripts/core/game_calendar.gd")
		var bd: Dictionary = GameCalendar.tick_to_date(entity.birth_tick)
		birth_str = " (Y%d %d월 %d일생)" % [bd.year, bd.month, bd.day]
	else:
		birth_str = " (초기세대)"
	draw_string(font, Vector2(cx, cy + 14), "Settlement: %s  |  %d세%s  |  Pos: (%d, %d)%s" % [sid_text, int(age_years), birth_str, entity.position.x, entity.position.y, preg_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
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

	# ── Personality ──
	cy = _draw_section_header(font, cx, cy, "Personality")
	var p_keys: Array = ["openness", "agreeableness", "extraversion", "diligence", "emotional_stability"]
	var p_labels: Array = ["Open", "Agree", "Extra", "Dilig", "Stab"]
	for i in range(p_keys.size()):
		var val: float = entity.personality.get(p_keys[i], 0.5)
		cy = _draw_bar(font, cx + 10, cy, bar_w, p_labels[i], val, PERSONALITY_COLORS[p_keys[i]])
	cy += 6.0

	# ── Emotions ──
	cy = _draw_section_header(font, cx, cy, "Emotions")
	var e_keys: Array = ["happiness", "loneliness", "stress", "grief", "love"]
	var e_labels: Array = ["Happy", "Lonely", "Stress", "Grief", "Love"]
	for i in range(e_keys.size()):
		var val: float = entity.emotions.get(e_keys[i], 0.0)
		cy = _draw_bar(font, cx + 10, cy, bar_w, e_labels[i], val, EMOTION_COLORS[e_keys[i]])
	cy += 6.0

	# ── Family ──
	cy = _draw_section_header(font, cx, cy, "Family")

	# Partner
	if entity.partner_id >= 0:
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		var partner_name: String = "(deceased)"
		var partner_alive: bool = false
		if partner != null and partner.is_alive:
			partner_name = partner.entity_name
			partner_alive = true
		else:
			# Check DeceasedRegistry
			var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
			if registry != null:
				var record: Dictionary = registry.get_record(entity.partner_id)
				if record.size() > 0:
					partner_name = record.get("name", "?") + "(d)"
		var love_pct: int = int(entity.emotions.get("love", 0.0) * 100)
		var prefix: String = "Partner: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		var name_pos := Vector2(cx + 10 + prefix_w, cy + 12)
		draw_string(font, name_pos, partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
		_register_click_region(name_pos, partner_name, entity.partner_id, font, GameConfig.get_font_size("popup_body"))
		var suffix: String = " (Love: %d%%)" % love_pct
		var name_w: float = font.get_string_size(partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
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
					pname += "(d)"
			else:
				var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
				if registry != null:
					var record: Dictionary = registry.get_record(pid)
					if record.size() > 0:
						pname = record.get("name", "?") + "(d)"
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
		for i in range(entity.children_ids.size()):
			var cid: int = entity.children_ids[i]
			var cname: String = "?"
			var child: RefCounted = _entity_manager.get_entity(cid)
			if child != null:
				var child_age: float = GameConfig.get_age_years(child.age)
				cname = "%s(%.0fy)" % [child.entity_name, child_age]
				if not child.is_alive:
					cname = "%s(d)" % child.entity_name
			else:
				var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
				if registry != null:
					var record: Dictionary = registry.get_record(cid)
					if record.size() > 0:
						cname = record.get("name", "?") + "(d)"
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
							other_name = record.get("name", "?") + "(d)"
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


func _draw_section_header(font: Font, x: float, y: float, title: String) -> float:
	draw_string(font, Vector2(x, y + 12), title, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	return y + 18.0


func _draw_separator(x: float, y: float, panel_w: float) -> void:
	draw_line(Vector2(x, y), Vector2(panel_w - 20, y), Color(0.3, 0.3, 0.3), 1.0)


func _draw_bar(font: Font, x: float, y: float, w: float, label: String, value: float, color: Color) -> float:
	draw_string(font, Vector2(x, y + 11), label + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("bar_label"), Color(0.7, 0.7, 0.7))
	var bar_x: float = x + 55.0
	var bar_w: float = w - 100.0
	var bar_h: float = 10.0
	draw_rect(Rect2(bar_x, y + 2, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	draw_rect(Rect2(bar_x, y + 2, bar_w * clampf(value, 0.0, 1.0), bar_h), color)
	draw_string(font, Vector2(bar_x + bar_w + 5, y + 11), "%d%%" % int(value * 100), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("bar_label"), Color(0.8, 0.8, 0.8))
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
			return entity.entity_name + "(d)"
	var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
	if registry != null:
		var record: Dictionary = registry.get_record(entity_id)
		if record.size() > 0:
			return record.get("name", "?") + "(d)"
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
	var GameCalendar = load("res://scripts/core/game_calendar.gd")
	var birth_str: String = "?"
	var death_str: String = "?"
	var birth_tick: int = r.get("birth_tick", 0)
	var death_tick: int = r.get("death_tick", 0)
	if birth_tick >= 0:
		var bd: Dictionary = GameCalendar.tick_to_date(birth_tick)
		birth_str = "Y%d %d월 %d일" % [bd.year, bd.month, bd.day]
	else:
		birth_str = "초기세대"
	var dd: Dictionary = GameCalendar.tick_to_date(death_tick)
	death_str = "Y%d %d월 %d일" % [dd.year, dd.month, dd.day]
	var age_str: String = "%.0f세" % r.get("death_age_years", 0.0)
	draw_string(font, Vector2(cx, cy + 14), "%s ~ %s (향년 %s)" % [birth_str, death_str, age_str], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
	cy += 18.0

	# Cause of death
	draw_string(font, Vector2(cx, cy + 14), "사인: %s" % r.get("death_cause", "unknown"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.5, 0.5))
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
	var personality: Dictionary = r.get("personality", {})
	var p_keys: Array = ["openness", "agreeableness", "extraversion", "diligence", "emotional_stability"]
	var p_labels: Array = ["Open", "Agree", "Extra", "Dilig", "Stab"]
	var bar_w: float = panel_w - 80.0
	for i in range(p_keys.size()):
		var val: float = personality.get(p_keys[i], 0.5)
		cy = _draw_bar(font, cx + 10, cy, bar_w, p_labels[i], val, PERSONALITY_COLORS.get(p_keys[i], Color.GRAY))
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
