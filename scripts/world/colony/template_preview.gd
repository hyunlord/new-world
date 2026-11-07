# EXTERNAL DRAFT: bridge contract is documented in INTEGRATION.md; not runtime verified.
extends Node2D

signal changed

var template_id: String = ""
var station_id: String = ""
var template_rotation: int = 0
var anchor: Vector2i = Vector2i.ZERO
var footprint: Vector2i = Vector2i.ONE
var preview: Dictionary = {}
var tile_pixels: float = 16.0
var _world: Node
var _query: Array[Variant] = []
var _draw_scale: float = 0.0


func configure(world: Node) -> void:
	if _world != null and _world.is_connected("world_reset", cancel):
		_world.disconnect("world_reset", cancel)
	_world = world
	_query.clear()
	if _world != null:
		_world.connect("world_reset", cancel)


func begin(row: Dictionary, station: String) -> void:
	template_id = str(row["id"])
	station_id = station
	template_rotation = 0
	footprint = Vector2i(int(row["width"]), int(row["height"]))
	_query.clear()
	preview.clear()
	show()
	changed.emit()


func active() -> bool:
	return not template_id.is_empty()


func cancel() -> void:
	template_id = ""
	station_id = ""
	preview.clear()
	_query.clear()
	hide()
	queue_redraw()
	changed.emit()


func rotate_clockwise() -> void:
	template_rotation = (template_rotation + 1) % 4
	_query.clear()
	changed.emit()


func rotated_size() -> Vector2i:
	return footprint if template_rotation % 2 == 0 else Vector2i(footprint.y, footprint.x)


# Template cursor convention: cursor tile is anchor + floor(footprint / 2).
func cursor_origin(tile: Vector2i) -> Vector2i:
	var dimensions: Vector2i = rotated_size()
	return tile - Vector2i(floori(dimensions.x * 0.5), floori(dimensions.y * 0.5))


# OrderTool owns map projection. This node's local coordinates are TILE coordinates;
# set its transform to the same tile-to-world transform used by existing footprint outlines.
func track(tile: Vector2i, sim_tick: int, command_revision: int) -> void:
	if not active() or _world == null:
		return
	anchor = cursor_origin(tile)
	# Position changes invalidate on tick; paused command/material changes invalidate on append-only command log length.
	# Keep the counters separate: max/hash can hide a change or alias distinct authoritative inputs.
	var query: Array[Variant] = [template_id, anchor, template_rotation, station_id, sim_tick, command_revision]
	if query == _query:
		return
	_query = query
	var raw: Variant = _world.call("get_build_template_preview", template_id, anchor.x, anchor.y, template_rotation, station_id)
	preview = raw if raw is Dictionary else {}
	queue_redraw()
	changed.emit()


# UI requires current affordability; runtime rechecks Connected/knowledge/physical placement.
func place() -> Dictionary:
	if not active() or not bool(preview.get("accepted", false)) or not bool(preview.get("affordable", false)):
		return {"accepted": false, "reason": str(preview.get("reason", "unavailable"))}
	var raw: Variant = SimReads.mutate(_world, "cmd_build_template", [template_id, anchor.x, anchor.y, template_rotation, station_id])
	_query.clear()
	return raw if raw is Dictionary else {"accepted": false, "reason": "unavailable"}


func _process(_delta: float) -> void:
	if active() and visible:
		var pixels_per_tile: float = get_global_transform_with_canvas().x.length()
		if not is_equal_approx(_draw_scale, pixels_per_tile):
			_draw_scale = pixels_per_tile
			queue_redraw()


func _draw() -> void:
	if not active() or preview.is_empty():
		return
	var ink: Color = Color("8ee07a") if bool(preview.get("accepted", false)) and bool(preview.get("affordable", false)) else Color("d8a050")
	var bounds := Rect2(Vector2(anchor), Vector2(rotated_size()))
	# Local tiles -> viewport includes both this child's TILE scale and camera zoom.
	var pixels_per_tile: float = get_global_transform_with_canvas().x.length()
	var stroke: float = 2.0 / maxf(pixels_per_tile, 0.001)
	draw_rect(bounds, ink, false, stroke)
	var pieces: Array = preview.get("pieces", [])
	for value: Variant in pieces:
		if value is not Dictionary:
			continue
		var piece: Dictionary = value
		var piece_ink: Color = ink
		piece_ink.a = 0.55
		draw_rect(Rect2(float(piece["x"]), float(piece["y"]), float(piece["width"]), float(piece["height"])), piece_ink, false, stroke)
	# Anchor cross is geometric, with no new bitmap or per-template code branches.
	var origin: Vector2 = Vector2(anchor) + Vector2(0.5, 0.5)
	draw_line(origin - Vector2(0.3, 0), origin + Vector2(0.3, 0), ink, stroke)
	draw_line(origin - Vector2(0, 0.3), origin + Vector2(0, 0.3), ink, stroke)
