extends Node2D
## Current settlement anchors only: retired selection history must not create links.
const FONT = preload("res://assets/fonts/galmuri/Galmuri9.ttf")
const HEARTH_ORIGIN := Vector2(456, 36)
var _world: Node
var _locale: Node
var _tick: int = -1
var _rows: Array[Dictionary] = []
var _zoom: float = 1.0
var _text: String = ""

func _init() -> void:
	z_index = -1

func configure(world: Node, locale: Node) -> void:
	if is_instance_valid(_world) and _world.has_signal("world_reset") and _world.is_connected("world_reset", reset):
		_world.disconnect("world_reset", reset)
	_world = world
	_locale = locale
	if is_instance_valid(_world) and _world.has_signal("world_reset"):
		_world.connect("world_reset", reset)
	reset()

func reset() -> void:
	_tick = -1
	_rows.clear()
	queue_redraw()

func update_view() -> void:
	_zoom = maxf(0.01, get_global_transform_with_canvas().get_scale().x)
	visible = _zoom < 2.0
	_text = str(_locale.call("ltr", "UI_RELATION_ESTRANGED")) if is_instance_valid(_locale) else ""
	if is_instance_valid(_world) and _world.has_method("get_settlement_relations") and _world.has_method("get_settlement_snapshot") and _world.has_method("get_time_snapshot"):
		var time: Dictionary = SimReads.read(_world, "get_time_snapshot")
		var tick: int = int(time.get("tick", -1))
		if tick != _tick:
			_rebuild(tick)
	queue_redraw()

func _rebuild(tick: int) -> void:
	_tick = tick
	_rows.clear()
	var settlements: Dictionary = SimReads.read(_world, "get_settlement_snapshot")
	var relations: Dictionary = SimReads.read(_world, "get_settlement_relations")
	var anchors: Dictionary = {}
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	var xs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	for i: int in range(mini(ids.size(), mini(xs.size(), ys.size()))):
		if xs[i] >= 0 and ys[i] >= 0:
			anchors[int(ids[i])] = HEARTH_ORIGIN + Vector2(xs[i], ys[i]) * 16.0
	var aa: PackedInt64Array = relations.get("a", PackedInt64Array())
	var bb: PackedInt64Array = relations.get("b", PackedInt64Array())
	var values: PackedInt32Array = relations.get("value", PackedInt32Array())
	var estranged: PackedByteArray = relations.get("estranged", PackedByteArray())
	for i: int in range(mini(mini(aa.size(), bb.size()), mini(values.size(), estranged.size()))):
		if not anchors.has(int(aa[i])) or not anchors.has(int(bb[i])):
			continue
		var color := Color("7FB069") if values[i] >= 150 else (Color("B8B8B0") if values[i] >= 0 else Color("D9893A"))
		if estranged[i] != 0:
			color = Color("C0392B")
		_rows.append({"a": int(aa[i]), "b": int(bb[i]), "value": int(values[i]), "estranged": estranged[i] != 0, "color": color, "from": anchors[int(aa[i])], "to": anchors[int(bb[i])]})

func line_width_world() -> float:
	return 2.0 / _zoom

func _draw() -> void:
	for row: Dictionary in _rows:
		var start: Vector2 = row.from
		var end: Vector2 = row.to
		var color: Color = row.color
		if row.estranged:
			draw_dashed_line(start, end, color, line_width_world(), 6.0 / _zoom)
			var size: Vector2 = FONT.get_string_size(_text, HORIZONTAL_ALIGNMENT_LEFT, -1, 12)
			draw_set_transform((start + end) * 0.5, 0.0, Vector2.ONE / _zoom)
			draw_string(FONT, Vector2(-size.x * 0.5, -4.0), _text, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, color)
			draw_set_transform(Vector2.ZERO)
		else:
			draw_line(start, end, color, line_width_world())

func capture_metadata() -> Dictionary:
	var lines: Array[Dictionary] = []
	var transform: Transform2D = get_global_transform_with_canvas()
	for row: Dictionary in _rows:
		var start: Vector2 = row.from
		var end: Vector2 = row.to
		var screen_start: Vector2 = transform * start
		var screen_end: Vector2 = transform * end
		var midpoint: Vector2 = (screen_start + screen_end) * 0.5
		lines.append({"a": row.a, "b": row.b, "value": row.value, "estranged": row.estranged, "color": (row.color as Color).to_html(false), "width_screen_px": 2.0, "from_world": [start.x, start.y], "to_world": [end.x, end.y], "from_screen": [screen_start.x, screen_start.y], "to_screen": [screen_end.x, screen_end.y], "midpoint_screen": [midpoint.x, midpoint.y], "dashed": row.estranged, "text": _text if row.estranged else ""})
	return {"tick": _tick, "zoom": _zoom, "visible": visible, "lines": lines}
