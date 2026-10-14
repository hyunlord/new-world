extends Node2D
## Names and selection use persistent AgentId/SettlementId joins, never spatial membership.
const Meter = preload("res://scripts/ui/restless_meter.gd")
const Marker = preload("res://scripts/ui/offscreen_marker.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const FONT = preload("res://assets/fonts/galmuri/Galmuri9.ttf")
const ABANDONED_SHADER = preload("res://shaders/abandoned_building.gdshader")
const ORIGIN := Vector2(448, 28)
const LABEL_FONT_PX := 12
const LABEL_PADDING := Vector2(5, 3)
const LABEL_BACKPLATE := Color(0.08, 0.07, 0.06, 0.86)
const LABEL_SPRITE_GAP_PX := 6.0
var _selected_marker: Dictionary = {}
var _world: Node
var _locale: Node
var _labels: Dictionary = {}
var _meters: Dictionary = {}
var _drawn: Array[Dictionary] = []
var _abandoned_material: ShaderMaterial
var _last_tick: int = -1

func _ready() -> void:
	z_index = 12
	process_priority = 30
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_to_group("art_society_layers")
	_locale = get_node_or_null("/root/Locale")
	_abandoned_material = ShaderMaterial.new()
	_abandoned_material.shader = ABANDONED_SHADER
	configure(get_node_or_null("../WorldSim"))

func configure(world: Node) -> void:
	_world = world
	if _world != null and _world.has_signal("world_reset") and not _world.is_connected("world_reset", _reset):
		_world.connect("world_reset", _reset)
	refresh_snapshot()

func _reset() -> void:
	Selection.reset()
	_last_tick = -1
	for label: Label in _labels.values():
		label.queue_free()
	_labels.clear()
	for meter: Control in _meters.values():
		meter.queue_free()
	_meters.clear()
	_drawn.clear()
	_selected_marker.clear()

func refresh_snapshot() -> void:
	if _world == null or not _world.has_method("get_society_snapshot"):
		return
	var society: Dictionary = _world.call("get_society_snapshot")
	var settlements: Dictionary = _world.call("get_settlement_snapshot")
	Selection.observe(society, settlements)
	_last_tick = int(society.get("current_tick", -1))
	var overview: Node = get_node_or_null("../SettlementOverviewRenderer")
	if overview != null and overview.has_method("set_presentation"):
		var colors: Dictionary = {}
		var names: Dictionary = {}
		for id: int in Selection.alive_ids():
			colors[id] = Selection.color_for(id)
			names[id] = Selection.name_for(id, _locale)
		overview.call("set_presentation", colors, names)
	_update_labels()

func _process(_delta: float) -> void:
	if _world == null:
		return
	var time: Dictionary = _world.call("get_time_snapshot")
	if int(time.get("tick", -1)) != _last_tick:
		refresh_snapshot()
	_update_labels()
	_update_abandoned()

func _update_labels() -> void:
	_drawn.clear()
	var zoom: float = maxf(0.01, get_global_transform_with_canvas().get_scale().x)
	var seen: Dictionary = {}
	for id: int in Selection.known_ids():
		var alive: bool = Selection.alive_ids().has(id)
		var abandoned: bool = Selection.is_abandoned(id)
		var tile: Vector2i = Selection.formation_for(id)
		if (not alive and not abandoned) or tile.x < 0:
			continue
		seen[id] = true
		var label: Label = _labels.get(id) as Label
		if label == null:
			label = Label.new()
			label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			label.add_theme_font_override("font", FONT)
			label.add_theme_font_size_override("font_size", LABEL_FONT_PX)
			label.add_theme_color_override("font_shadow_color", Color(0.08, 0.07, 0.06, 0.9))
			label.add_theme_constant_override("shadow_offset_x", 1)
			label.add_theme_constant_override("shadow_offset_y", 1)
			add_child(label)
			_labels[id] = label
		var color: Color = Color(0.55, 0.55, 0.55) if abandoned else Selection.color_for(id)
		var text: String = Selection.name_for(id, _locale)
		if abandoned:
			var format: String = str(_locale.call("ltr", "SETTLEMENT_ABANDONED_FMT")) if _locale != null else ""
			text = format % Selection.number_for(id) if format.contains("%d") else format
		else:
			text += " (%d)" % Selection.displayed_members(id)
		label.text = text
		label.add_theme_color_override("font_color", color if abandoned else Color("EEEAD2"))
		label.scale = Vector2.ONE / zoom
		label.size = Vector2.ZERO
		label.position = ORIGIN + Vector2(tile) * 16.0 + Vector2(8.0, -38.0) - Vector2(label.size.x * 0.5, label.size.y + 3.0) / zoom
		var screen: Vector2 = label.get_global_transform_with_canvas() * Vector2.ZERO
		var bounds := Rect2(screen - LABEL_PADDING, label.size + LABEL_PADDING * 2.0)
		bounds = _clear_sprite_bounds(bounds)
		label.position = get_global_transform_with_canvas().affine_inverse() * (bounds.position + LABEL_PADDING)
		var meter: Control = _meters.get(id) as Control
		if meter == null:
			meter = Meter.new()
			add_child(meter)
			_meters[id] = meter
		meter.scale = Vector2.ONE / zoom
		meter.position = label.position + Vector2(label.size.x + LABEL_PADDING.x + 3.0, (label.size.y - Meter.SIZE.y) * 0.5) / zoom
		var meter_state: Dictionary = Selection.restlessness_for(id)
		meter_state["visible"] = bool(meter_state["visible"]) and zoom >= 2.0 and not abandoned
		meter.call("display", meter_state, Selection.color_for(id))
		_drawn.append({"id": id, "number": Selection.number_for(id), "parent_number": Selection.number_for(int(Selection._history[id].get("parent_id", -1))), "text": text, "tile": [tile.x, tile.y], "color": color.to_html(false), "abandoned": abandoned, "visible": get_viewport_rect().intersects(bounds), "screen_rect": [bounds.position.x, bounds.position.y, bounds.size.x, bounds.size.y], "screen_font_px": LABEL_FONT_PX, "backplate_alpha": LABEL_BACKPLATE.a, "padding_px": [LABEL_PADDING.x, LABEL_PADDING.y]})
	for id: int in _labels.keys():
		if not seen.has(id):
			(_labels[id] as Label).queue_free()
			_labels.erase(id)
			if _meters.has(id):
				(_meters[id] as Control).queue_free()
				_meters.erase(id)
	_selected_marker.clear()
	if Selection.selected_id >= 0:
		var tile: Vector2i = Selection.formation_for(Selection.selected_id)
		if tile.x >= 0:
			var target: Vector2 = ORIGIN + Vector2(tile) * 16.0 + Vector2(8, 8)
			_selected_marker = Marker.geometry(get_global_transform_with_canvas() * target, get_viewport_rect(), Marker.playable_rect(self))
			_selected_marker["id"] = Selection.selected_id
			_selected_marker["target_world"] = [target.x, target.y]
			_selected_marker["marker_color"] = Selection.color_for(Selection.selected_id).to_html(false)
			_selected_marker["stake_visible"] = false
	queue_redraw()

func _clear_sprite_bounds(bounds: Rect2) -> Rect2:
	var objects: Node = get_node_or_null("../WorldObjects")
	var content_top: float = get_viewport_rect().position.y + LABEL_SPRITE_GAP_PX
	var hud: Node = get_node_or_null("../UI/HUDTopbar")
	if hud == null:
		hud = get_node_or_null("../UI/HudTopbar")
	if hud != null:
		for child: Node in hud.get_children():
			if child is PanelContainer and child.is_visible_in_tree():
				content_top = maxf(content_top, (child.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, child.size)).end.y + LABEL_SPRITE_GAP_PX)
	var max_y: float = maxf(content_top, get_viewport_rect().end.y - bounds.size.y - LABEL_SPRITE_GAP_PX)
	bounds.position.y = clampf(bounds.position.y, content_top, max_y)
	var bounded_anchor: Rect2 = bounds
	if objects == null:
		return bounds
	var obstacles: Array[Rect2] = []
	for child: Node in objects.get_children():
		if child is Sprite2D and child.visible and child.get_meta("art_kind", "") == "building":
			var sprite := child as Sprite2D
			obstacles.append(sprite.get_global_transform_with_canvas() * sprite.get_rect())
	# Static roofs avoid walking-frame jitter; an obstructed top edge keeps the bounded plate.
	for _pass: int in obstacles.size():
		var previous_y: float = bounds.position.y
		for obstacle: Rect2 in obstacles:
			if bounds.grow(LABEL_SPRITE_GAP_PX).intersects(obstacle):
				var candidate_y: float = obstacle.position.y - bounds.size.y - LABEL_SPRITE_GAP_PX
				if candidate_y < content_top:
					return bounded_anchor
				bounds.position.y = minf(bounds.position.y, candidate_y)
		if is_equal_approx(previous_y, bounds.position.y):
			break
	return bounds

func _draw() -> void:
	var zoom: float = maxf(0.01, get_global_transform_with_canvas().get_scale().x)
	for id: int in _labels:
		var label: Label = _labels[id]
		var color: Color = Color(0.55, 0.55, 0.55) if Selection.is_abandoned(id) else Selection.color_for(id)
		var plate := Rect2(label.position - LABEL_PADDING / zoom, (label.size + LABEL_PADDING * 2.0) / zoom)
		draw_rect(plate, LABEL_BACKPLATE)
		var from := Vector2(plate.position.x, plate.end.y)
		draw_line(from, from + Vector2(plate.size.x, 0.0), color, (2.0 if Selection.selected_id == id else 1.0) / zoom)

	if not _selected_marker.is_empty():
		Marker.draw_marker(self, _selected_marker, Selection.color_for(Selection.selected_id))

func _input(event: InputEvent) -> void:
	if visible and event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT and Marker.hit(_selected_marker, event.position):
		var target: Array = _selected_marker["target_world"]
		Marker.pan(get_viewport().get_camera_2d(), Vector2(target[0], target[1]))
		get_viewport().set_input_as_handled()
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		Selection.clear()

func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT):
		return
	if select_at(event.position):
		get_viewport().set_input_as_handled()
		return
	var agents: Node = get_node_or_null("../AgentRenderer")
	if agents != null and agents.has_method("drawn_entity_at") and int(agents.call("drawn_entity_at", get_global_mouse_position())) >= 0:
		return
	var objects: Node = get_node_or_null("../WorldObjects")
	if objects != null:
		for child: Node in objects.get_children():
			if child is Sprite2D and child.get_meta("art_kind", "") == "building" and child.get_rect().has_point(child.to_local(get_global_mouse_position())):
				return
	Selection.clear()

func select_at(screen: Vector2) -> bool:
	for row: Dictionary in _drawn:
		if bool(row["abandoned"]):
			continue
		var rect: Array = row["screen_rect"]
		if Rect2(rect[0], rect[1], rect[2], rect[3]).grow(3.0).has_point(screen):
			Selection.select(int(row["id"]))
			return true
	var overview: Node2D = get_node_or_null("../SettlementOverviewRenderer") as Node2D
	if overview != null and overview.visible:
		var world: Vector2 = get_global_transform_with_canvas().affine_inverse() * screen
		var best: int = -1
		var distance: float = 80.0 * 80.0
		for id: int in Selection.alive_ids():
			var center: Vector2 = ORIGIN + Vector2(Selection.formation_for(id)) * 16.0 + Vector2(8, 8)
			var d: float = center.distance_squared_to(world)
			if d <= distance:
				best = id
				distance = d
		if best >= 0:
			Selection.select(best)
			return true
	return false

func _update_abandoned() -> void:
	var objects: Node = get_node_or_null("../WorldObjects")
	if objects == null:
		return
	for child: Node in objects.get_children():
		if not (child is Sprite2D) or child.get_meta("art_kind", "") != "building":
			continue
		var sprite := child as Sprite2D
		var tile := Vector2i(int(sprite.get_meta("tile_x", -1)), int(sprite.get_meta("tile_y", -1)))
		var abandoned: bool = Selection.retired_building(tile)
		sprite.set_meta("abandoned", abandoned)
		sprite.set_meta("abandoned_desaturation", 0.6 if abandoned else 0.0)
		sprite.set_meta("abandoned_alpha", 0.85 if abandoned else 1.0)
		if abandoned:
			sprite.material = _abandoned_material
		elif sprite.material == _abandoned_material:
			sprite.material = null

func capture_metadata() -> Dictionary:
	var result: Dictionary = Selection.capture_metadata(_locale)
	result["labels"] = _drawn.duplicate(true)
	result["selected_marker"] = _selected_marker.duplicate(true)
	var meters: Array[Dictionary] = []
	for id: int in _meters:
		var row: Dictionary = _meters[id].call("capture_metadata")
		row["id"] = id
		meters.append(row)
	result["meters"] = meters
	var overview: Node = get_node_or_null("../SettlementOverviewRenderer")
	if overview != null and overview.has_method("capture_metadata"):
		result.merge(overview.call("capture_metadata"))
	return result
