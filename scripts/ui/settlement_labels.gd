extends Node2D
## Names and selection use persistent AgentId/SettlementId joins, never spatial membership.
const Selection = preload("res://scripts/core/settlement_selection.gd")
const FONT = preload("res://assets/fonts/galmuri/Galmuri9.ttf")
const ABANDONED_SHADER = preload("res://shaders/abandoned_building.gdshader")
const ORIGIN := Vector2(448, 28)
var _world: Node
var _locale: Node
var _labels: Dictionary = {}
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
	_drawn.clear()

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
			label.add_theme_font_size_override("font_size", 10)
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
		label.position = ORIGIN + Vector2(tile) * 16.0 + Vector2(8.0, -22.0) - Vector2(label.size.x * 0.5, label.size.y + 3.0) / zoom
		var screen: Vector2 = label.get_global_transform_with_canvas() * Vector2.ZERO
		var bounds := Rect2(screen, label.size)
		_drawn.append({"id": id, "number": Selection.number_for(id), "parent_number": Selection.number_for(int(Selection._history[id].get("parent_id", -1))), "text": text, "tile": [tile.x, tile.y], "color": color.to_html(false), "abandoned": abandoned, "visible": get_viewport_rect().intersects(bounds), "screen_rect": [bounds.position.x, bounds.position.y, bounds.size.x, bounds.size.y], "screen_font_px": 10})
	for id: int in _labels.keys():
		if not seen.has(id):
			(_labels[id] as Label).queue_free()
			_labels.erase(id)
	queue_redraw()

func _draw() -> void:
	var zoom: float = maxf(0.01, get_global_transform_with_canvas().get_scale().x)
	for id: int in _labels:
		var label: Label = _labels[id]
		var color: Color = Color(0.55, 0.55, 0.55) if Selection.is_abandoned(id) else Selection.color_for(id)
		var from: Vector2 = label.position + Vector2(0.0, label.size.y + 1.0) / zoom
		draw_line(from, from + Vector2(label.size.x, 0.0) / zoom, color, (2.0 if Selection.selected_id == id else 1.0) / zoom)

func _input(event: InputEvent) -> void:
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
	var overview: Node = get_node_or_null("../SettlementOverviewRenderer")
	if overview != null and overview.has_method("capture_metadata"):
		result.merge(overview.call("capture_metadata"))
	return result
