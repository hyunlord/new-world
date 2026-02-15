class_name BuildingDetailPanel
extends Control

var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _sim_engine: RefCounted
var _building_id: int = -1
var _was_paused: bool = false


func init(building_manager: RefCounted, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP


func show_building(building_id: int) -> void:
	_building_id = building_id
	if _sim_engine != null:
		_was_paused = _sim_engine.is_paused
		_sim_engine.is_paused = true
		SimulationBus.pause_changed.emit(true)
	visible = true


func hide_panel() -> void:
	visible = false
	_building_id = -1
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
		var panel_w: float = vp_size.x * 0.45
		var panel_x: float = (vp_size.x - panel_w) * 0.5
		var panel_y: float = (vp_size.y - vp_size.y * 0.5) * 0.5
		var close_area := Rect2(panel_x + panel_w - 30, panel_y + 5, 25, 25)
		var mb: InputEventMouseButton = event as InputEventMouseButton
		if close_area.has_point(mb.position):
			hide_panel()
			accept_event()


func _draw() -> void:
	if not visible or _building_manager == null or _building_id < 0:
		return

	var building = null
	var all_buildings: Array = _building_manager.get_all_buildings()
	for i in range(all_buildings.size()):
		if all_buildings[i].id == _building_id:
			building = all_buildings[i]
			break
	if building == null:
		hide_panel()
		return

	var vp_size := get_viewport_rect().size
	draw_rect(Rect2(Vector2.ZERO, vp_size), Color(0, 0, 0, 0.7))

	var panel_w: float = vp_size.x * 0.45
	var panel_h: float = vp_size.y * 0.5
	var panel_x: float = (vp_size.x - panel_w) * 0.5
	var panel_y: float = (vp_size.y - panel_h) * 0.5
	var pr := Rect2(panel_x, panel_y, panel_w, panel_h)
	draw_rect(pr, Color(0.08, 0.06, 0.02, 0.95))
	draw_rect(pr, Color(0.4, 0.3, 0.2), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = panel_x + 20.0
	var cy: float = panel_y + 28.0

	# Header
	var icon: String = "\u25A0"
	var type_color: Color = Color(0.55, 0.35, 0.15)
	match building.building_type:
		"shelter":
			icon = "\u25B2"
			type_color = Color(0.7, 0.4, 0.2)
		"campfire":
			icon = "\u25CF"
			type_color = Color(1.0, 0.4, 0.1)
	draw_string(font, Vector2(cx, cy), "%s %s" % [icon, building.building_type.capitalize()], HORIZONTAL_ALIGNMENT_LEFT, -1, 18, type_color)
	draw_string(font, Vector2(panel_x + panel_w - 28, panel_y + 20), "[X]", HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.7, 0.7, 0.7))
	cy += 8.0

	var sid_text: String = "S%d" % building.settlement_id if building.settlement_id > 0 else "None"
	draw_string(font, Vector2(cx, cy + 14), "Location: (%d, %d)  |  Settlement: %s" % [building.tile_x, building.tile_y, sid_text], HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.7, 0.7, 0.7))
	cy += 22.0
	draw_line(Vector2(cx, cy), Vector2(panel_x + panel_w - 20, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 10.0

	# Status
	if building.is_built:
		draw_string(font, Vector2(cx, cy + 12), "Status: Active", HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.3, 0.9, 0.3))
	else:
		draw_string(font, Vector2(cx, cy + 12), "Status: Under Construction (%d%%)" % int(building.build_progress * 100), HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.9, 0.8, 0.2))
		cy += 18.0
		var bar_w: float = panel_w - 60
		draw_rect(Rect2(cx + 10, cy, bar_w, 12), Color(0.2, 0.2, 0.2, 0.8))
		draw_rect(Rect2(cx + 10, cy, bar_w * building.build_progress, 12), Color(0.2, 0.8, 0.2, 0.8))
	cy += 22.0

	# Type-specific info
	draw_string(font, Vector2(cx, cy + 12), "Details", HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color.WHITE)
	cy += 18.0

	match building.building_type:
		"stockpile":
			if building.is_built:
				var food: float = building.storage.get("food", 0.0)
				var wood: float = building.storage.get("wood", 0.0)
				var stone: float = building.storage.get("stone", 0.0)
				draw_string(font, Vector2(cx + 10, cy + 12), "Food: %.1f" % food, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.4, 0.8, 0.2))
				cy += 16.0
				draw_string(font, Vector2(cx + 10, cy + 12), "Wood: %.1f" % wood, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.6, 0.4, 0.2))
				cy += 16.0
				draw_string(font, Vector2(cx + 10, cy + 12), "Stone: %.1f" % stone, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.7, 0.7, 0.7))
				cy += 16.0
				draw_string(font, Vector2(cx + 10, cy + 12), "Total: %.1f" % (food + wood + stone), HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.8, 0.8, 0.8))
			else:
				draw_string(font, Vector2(cx + 10, cy + 12), "Storage available after construction", HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.6, 0.6, 0.6))
		"shelter":
			draw_string(font, Vector2(cx + 10, cy + 12), "Housing: Provides energy regeneration bonus", HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.8, 0.8, 0.8))
			cy += 16.0
			draw_string(font, Vector2(cx + 10, cy + 12), "Capacity: 6 entities per shelter", HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.8, 0.8, 0.8))
		"campfire":
			draw_string(font, Vector2(cx + 10, cy + 12), "Warmth: Provides social bonus to nearby entities", HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.8, 0.8, 0.8))
			cy += 16.0
			var radius: int = GameConfig.BUILDING_TYPES.get("campfire", {}).get("radius", 5)
			draw_string(font, Vector2(cx + 10, cy + 12), "Effect radius: %d tiles" % radius, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.8, 0.8, 0.8))

	# Build cost reference
	cy += 28.0
	draw_string(font, Vector2(cx, cy + 12), "Build Cost", HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color.WHITE)
	cy += 18.0
	var cost: Dictionary = GameConfig.BUILDING_TYPES.get(building.building_type, {}).get("cost", {})
	var cost_parts: Array = []
	var cost_keys: Array = cost.keys()
	for i in range(cost_keys.size()):
		cost_parts.append("%s: %.0f" % [cost_keys[i].capitalize(), cost[cost_keys[i]]])
	draw_string(font, Vector2(cx + 10, cy + 12), " | ".join(cost_parts), HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.7, 0.7, 0.7))

	draw_string(font, Vector2(vp_size.x * 0.5 - 30, panel_y + panel_h - 12), "[E: Close]", HORIZONTAL_ALIGNMENT_CENTER, -1, 12, Color(0.5, 0.5, 0.5))
