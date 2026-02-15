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


func _draw() -> void:
	if not visible or _entity_manager == null or _entity_id < 0:
		return
	var entity: RefCounted = _entity_manager.get_entity(_entity_id)
	if entity == null or not entity.is_alive:
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
	draw_string(font, Vector2(cx, cy + 14), "Settlement: %s  |  Age: %.1fy  |  Pos: (%d, %d)%s" % [sid_text, age_years, entity.position.x, entity.position.y, preg_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
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
		var partner_name: String = partner.entity_name if partner != null and partner.is_alive else "(deceased)"
		var love_pct: int = int(entity.emotions.get("love", 0.0) * 100)
		draw_string(font, Vector2(cx + 10, cy + 12), "Partner: %s (Love: %d%%)" % [partner_name, love_pct], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		cy += 16.0
	else:
		draw_string(font, Vector2(cx + 10, cy + 12), "Partner: None", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		cy += 16.0

	# Parents
	if entity.parent_ids.size() > 0:
		var parent_names: String = ""
		for i in range(entity.parent_ids.size()):
			var pid: int = entity.parent_ids[i]
			var parent: RefCounted = _entity_manager.get_entity(pid)
			if parent != null:
				if parent_names.length() > 0:
					parent_names += ", "
				parent_names += parent.entity_name if parent.is_alive else parent.entity_name + "(d)"
		if parent_names.length() > 0:
			draw_string(font, Vector2(cx + 10, cy + 12), "Parents: %s" % parent_names, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
			cy += 16.0

	# Children
	if entity.children_ids.size() > 0:
		var child_text: String = "Children: "
		var child_parts: Array = []
		for i in range(entity.children_ids.size()):
			var cid: int = entity.children_ids[i]
			var child: RefCounted = _entity_manager.get_entity(cid)
			if child != null and child.is_alive:
				var child_age: float = GameConfig.get_age_years(child.age)
				child_parts.append("%s(%.0fy)" % [child.entity_name, child_age])
			elif child != null:
				child_parts.append("%s(d)" % child.entity_name)
		if child_parts.size() > 0:
			# Split long child lists across lines
			var line: String = child_text
			for i in range(child_parts.size()):
				if i > 0:
					line += ", "
				line += child_parts[i]
				if line.length() > 50 and i < child_parts.size() - 1:
					draw_string(font, Vector2(cx + 10, cy + 12), line, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 0.7))
					cy += 14.0
					line = "  "
			draw_string(font, Vector2(cx + 10, cy + 12), line, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.8, 0.7))
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
				var other_name: String = other.entity_name if other != null else "?"
				var type_color: Color = REL_TYPE_COLORS.get(rel.type, Color.GRAY)
				var rel_label: String = rel.type.replace("_", " ").capitalize()
				var line: String = "%s - %s (A:%d T:%d)" % [other_name, rel_label, int(rel.affinity), int(rel.trust)]
				if rel.romantic_interest > 0.0:
					line += " R:%d" % int(rel.romantic_interest)
				draw_string(font, Vector2(cx + 10, cy + 12), line, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), type_color)
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
