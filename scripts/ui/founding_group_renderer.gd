extends Node2D
## Read-only founding trail and destination; no inferred or simulated membership.
const Marker = preload("res://scripts/ui/offscreen_marker.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const ORIGIN := Vector2(448, 28)
const TRAIL_COLOR := Color("E8D8A899")
const DASH_PX: float = 2.0
const GAP_PX: float = 3.0
var _world: Node
var _agents: Node
var _groups: Array[Dictionary] = []
var _drawn: Array[Dictionary] = []

func _ready() -> void:
	z_index = 2
	process_priority = 31
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_to_group("art_society_layers")
	_world = get_node_or_null("/root/Main/WorldSim")
	_agents = get_parent()
	if _world != null and _world.has_signal("world_reset"):
		_world.connect("world_reset", _reset)

func _reset() -> void:
	_groups.clear()
	_drawn.clear()
	queue_redraw()

func _process(_delta: float) -> void:
	if _world == null or not _world.has_method("get_society_snapshot"):
		return
	refresh_snapshots(_world.call("get_society_snapshot"), _world.call("get_agent_snapshot"))

func refresh_snapshots(society: Dictionary, agents: Dictionary) -> void:
	_groups.clear()
	var ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
	var founders: PackedInt64Array = society.get("founder_agent_ids", PackedInt64Array())
	var fissions: PackedInt32Array = society.get("founder_fission_ids", PackedInt32Array())
	var leaders: PackedByteArray = society.get("founder_leader", PackedByteArray())
	var sites_x: PackedInt32Array = society.get("founder_site_xs", PackedInt32Array())
	var sites_y: PackedInt32Array = society.get("founder_site_ys", PackedInt32Array())
	for index: int in founders.size():
		if index >= leaders.size() or leaders[index] != 1 or index >= fissions.size() or index >= sites_x.size() or index >= sites_y.size():
			continue
		var agent_index: int = ids.find(founders[index])
		if agent_index < 0 or agent_index >= xs.size() or agent_index >= ys.size():
			continue
		var start_tile := Vector2i(xs[agent_index], ys[agent_index])
		var site_tile := Vector2i(sites_x[index], sites_y[index])
		var parent: int = Selection.home_for_agent(founders[index])
		var members: Array[int] = []
		for member: int in mini(founders.size(), fissions.size()):
			if fissions[member] == fissions[index]:
				members.append(founders[member])
		_groups.append({"fission_id": fissions[index], "leader_id": founders[index], "parent_id": parent, "parent_number": Selection.number_for(parent), "founder_ids": members, "start_tile": [start_tile.x, start_tile.y], "site_tile": [site_tile.x, site_tile.y], "start": ORIGIN + Vector2(start_tile) * 16.0 + Vector2(8, 8), "end": ORIGIN + Vector2(site_tile) * 16.0 + Vector2(8, 8), "marker_color": Selection.color_for(parent)})
	queue_redraw()

func _draw() -> void:
	_drawn.clear()
	for group: Dictionary in _groups:
		var start: Vector2 = group["start"]
		var finish: Vector2 = group["end"]
		var direction: Vector2 = start.direction_to(finish)
		var distance: float = start.distance_to(finish)
		var cursor: float = 0.0
		while cursor < distance:
			draw_line(start + direction * cursor, start + direction * minf(cursor + DASH_PX, distance), TRAIL_COLOR, 1.0)
			cursor += DASH_PX + GAP_PX
		var marker: Dictionary = Marker.geometry(get_global_transform_with_canvas() * finish, get_viewport_rect(), Marker.playable_rect(self))
		if bool(marker["stake_visible"]):
			var dark := Color("352b23")
			draw_rect(Rect2(finish + Vector2(0, -6), Vector2(1, 6)), dark)
			draw_rect(Rect2(finish + Vector2(0, -7), Vector2(6, 5)), dark)
			draw_rect(Rect2(finish + Vector2(1, -6), Vector2(4, 3)), group["marker_color"])
		var tiles: float = start.distance_to(finish) / 16.0
		marker["distance_tiles"] = tiles
		marker["displayed_distance_tiles"] = roundi(tiles)
		var locale: Node = get_node_or_null("/root/Locale")
		var text: String = str(locale.call("ltr", "FOUNDING_SITE_DISTANCE_FMT")) % roundi(tiles) if locale != null else ""
		Marker.draw_marker(self, marker, group["marker_color"], text)
		var row: Dictionary = group.duplicate(true)
		row["start"] = [start.x, start.y]
		row["end"] = [finish.x, finish.y]
		row["marker_color"] = (group["marker_color"] as Color).to_html(false)
		row["trail_color"] = "E8D8A8"
		row["trail_alpha"] = TRAIL_COLOR.a
		row["dash_px"] = DASH_PX
		row["gap_px"] = GAP_PX
		row.merge(marker)
		row["marker_visible"] = bool(marker["stake_visible"]) or bool(marker["edge_visible"])
		row["trail_visible"] = distance > 0.0
		row["bundles"] = _bundle_metadata(group["founder_ids"])
		_drawn.append(row)

func _bundle_metadata(founders: Array) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	if _agents == null or not _agents.has_method("get_agent_sprite"):
		return rows
	for id: int in founders:
		var sprite: Sprite2D = _agents.call("get_agent_sprite", id) as Sprite2D
		if sprite == null:
			continue
		var anchor: Vector2 = sprite.get_meta("founder_bundle_anchor", Vector2.ZERO)
		rows.append({"agent_id": id, "agent_visible": sprite.visible, "visible": sprite.get_meta("founder_bundle_visible", false), "anchor": [anchor.x, anchor.y], "flip_h": sprite.flip_h, "suppressed": sprite.get_meta("founder_bundle_suppressed", "")})
	return rows

func capture_metadata() -> Dictionary:
	return {"founding_groups": _drawn.duplicate(true)}

func _input(event: InputEvent) -> void:
	if not visible or not (event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT):
		return
	for row: Dictionary in _drawn:
		if Marker.hit(row, event.position):
			var target: Array = row["end"]
			Marker.pan(get_viewport().get_camera_2d(), Vector2(target[0], target[1]))
			get_viewport().set_input_as_handled()
			return
