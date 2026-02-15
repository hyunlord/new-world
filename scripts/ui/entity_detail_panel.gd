class_name EntityDetailPanel
extends Control

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _sim_engine: RefCounted
var _entity_id: int = -1
var _was_paused: bool = false


func init(entity_manager: RefCounted, building_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_sim_engine = sim_engine


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP


func show_entity(entity_id: int) -> void:
	_entity_id = entity_id
	if _sim_engine != null:
		_was_paused = _sim_engine.is_paused
		_sim_engine.is_paused = true
		SimulationBus.pause_changed.emit(true)
	visible = true


func hide_panel() -> void:
	visible = false
	_entity_id = -1
	if _sim_engine != null and not _was_paused:
		_sim_engine.is_paused = false
		SimulationBus.pause_changed.emit(false)


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_E or event.keycode == KEY_ESCAPE:
			hide_panel()
			accept_event()
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var vp_size := get_viewport_rect().size
		var panel_w: float = vp_size.x * 0.5
		var panel_h: float = vp_size.y * 0.65
		var panel_x: float = (vp_size.x - panel_w) * 0.5
		var panel_y: float = (vp_size.y - panel_h) * 0.5
		var panel_rect := Rect2(panel_x, panel_y, panel_w, panel_h)
		var mb: InputEventMouseButton = event as InputEventMouseButton
		var close_area := Rect2(panel_x + panel_w - 30, panel_y + 5, 25, 25)
		if close_area.has_point(mb.position) or not panel_rect.has_point(mb.position):
			hide_panel()
			accept_event()
		else:
			accept_event()


func _draw() -> void:
	if not visible or _entity_manager == null or _entity_id < 0:
		return
	var entity: RefCounted = _entity_manager.get_entity(_entity_id)
	if entity == null or not entity.is_alive:
		hide_panel()
		return

	var vp_size := get_viewport_rect().size
	draw_rect(Rect2(Vector2.ZERO, vp_size), Color(0, 0, 0, 0.7))

	var panel_w: float = vp_size.x * 0.5
	var panel_h: float = vp_size.y * 0.65
	var panel_x: float = (vp_size.x - panel_w) * 0.5
	var panel_y: float = (vp_size.y - panel_h) * 0.5
	var pr := Rect2(panel_x, panel_y, panel_w, panel_h)
	draw_rect(pr, Color(0.06, 0.1, 0.06, 0.95))
	draw_rect(pr, Color(0.3, 0.4, 0.3), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = panel_x + 20.0
	var cy: float = panel_y + 28.0

	var job_colors: Dictionary = {
		"none": Color(0.6, 0.6, 0.6), "gatherer": Color(0.3, 0.8, 0.2),
		"lumberjack": Color(0.6, 0.35, 0.1), "builder": Color(0.9, 0.6, 0.1),
		"miner": Color(0.5, 0.6, 0.75),
	}
	var jc: Color = job_colors.get(entity.job, Color.WHITE)

	# Header
	draw_string(font, Vector2(cx, cy), "%s - %s" % [entity.entity_name, entity.job.capitalize()], HORIZONTAL_ALIGNMENT_LEFT, -1, 20, jc)
	draw_string(font, Vector2(panel_x + panel_w - 28, panel_y + 20), "[X]", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.7, 0.7, 0.7))
	cy += 6.0

	var age_days: int = entity.age / GameConfig.AGE_DAYS_DIVISOR
	var sid_text: String = "S%d" % entity.settlement_id if entity.settlement_id > 0 else "None"
	draw_string(font, Vector2(cx, cy + 14), "Settlement: %s  |  Age: %dd  |  Pos: (%d, %d)" % [sid_text, age_days, entity.position.x, entity.position.y], HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.7, 0.7, 0.7))
	cy += 22.0
	draw_line(Vector2(cx, cy), Vector2(panel_x + panel_w - 20, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 10.0

	# Status
	draw_string(font, Vector2(cx, cy + 12), "Status", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color.WHITE)
	cy += 18.0

	var action_text: String = entity.current_action
	if entity.action_target != Vector2i(-1, -1):
		action_text += " -> (%d, %d)" % [entity.action_target.x, entity.action_target.y]
	draw_string(font, Vector2(cx + 10, cy + 12), "Action: %s" % action_text, HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
	cy += 16.0

	if entity.cached_path.size() > 0:
		var remaining: int = entity.cached_path.size() - entity.path_index
		if remaining > 0:
			draw_string(font, Vector2(cx + 10, cy + 12), "Path: %d steps remaining" % remaining, HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
			cy += 16.0

	draw_string(font, Vector2(cx + 10, cy + 12), "Inventory: F:%.1f  W:%.1f  S:%.1f / %.0f" % [
		entity.inventory.get("food", 0.0), entity.inventory.get("wood", 0.0),
		entity.inventory.get("stone", 0.0), GameConfig.MAX_CARRY,
	], HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
	cy += 22.0

	# Needs
	draw_string(font, Vector2(cx, cy + 12), "Needs", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color.WHITE)
	cy += 18.0
	cy = _draw_need_bar(font, cx + 10, cy, panel_w - 60, "Hunger", entity.hunger, Color(0.9, 0.2, 0.2))
	cy = _draw_need_bar(font, cx + 10, cy, panel_w - 60, "Energy", entity.energy, Color(0.9, 0.8, 0.2))
	cy = _draw_need_bar(font, cx + 10, cy, panel_w - 60, "Social", entity.social, Color(0.3, 0.5, 0.9))
	cy += 8.0

	# Stats
	draw_string(font, Vector2(cx, cy + 12), "Stats", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color.WHITE)
	cy += 18.0
	draw_string(font, Vector2(cx + 10, cy + 12), "Speed: %.1f  |  Strength: %.1f" % [entity.speed, entity.strength], HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
	cy += 16.0
	draw_string(font, Vector2(cx + 10, cy + 12), "Total gathered: %.0f  |  Buildings built: %d" % [entity.total_gathered, entity.buildings_built], HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
	cy += 22.0

	# Action History
	draw_string(font, Vector2(cx, cy + 12), "Recent Actions", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color.WHITE)
	cy += 18.0
	var hist: Array = entity.action_history
	var idx: int = hist.size() - 1
	while idx >= 0 and cy < panel_y + panel_h - 30:
		var entry: Dictionary = hist[idx]
		draw_string(font, Vector2(cx + 10, cy + 11), "Tick %d: %s" % [entry.tick, entry.action], HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color(0.6, 0.6, 0.6))
		cy += 13.0
		idx -= 1

	draw_string(font, Vector2(vp_size.x * 0.5 - 30, panel_y + panel_h - 12), "[E: Close]", HORIZONTAL_ALIGNMENT_CENTER, -1, 14, Color(0.5, 0.5, 0.5))


func _draw_need_bar(font: Font, x: float, y: float, w: float, label: String, value: float, color: Color) -> float:
	draw_string(font, Vector2(x, y + 11), label + ":", HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.7, 0.7, 0.7))
	var bar_x: float = x + 55.0
	var bar_w: float = w - 100.0
	var bar_h: float = 10.0
	draw_rect(Rect2(bar_x, y + 2, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	draw_rect(Rect2(bar_x, y + 2, bar_w * value, bar_h), color)
	draw_string(font, Vector2(bar_x + bar_w + 5, y + 11), "%d%%" % int(value * 100), HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.8, 0.8, 0.8))
	return y + 16.0
