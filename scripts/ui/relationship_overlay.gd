extends Node2D
class_name RelationshipOverlay

const SnapshotDecoderClass = preload("res://scripts/rendering/snapshot_decoder.gd")

const FRIENDLY_COLOR: Color = Color(0.2, 0.9, 0.3, 0.6)
const HOSTILE_COLOR: Color = Color(0.9, 0.2, 0.2, 0.6)
const NEUTRAL_COLOR: Color = Color(0.5, 0.5, 0.5, 0.3)
const FRIENDLY_THRESHOLD: float = 0.3
const HOSTILE_THRESHOLD: float = -0.3
const MIN_LINE_WIDTH: float = 1.0
const MAX_LINE_WIDTH: float = 4.0

var _sim_engine: RefCounted = null
var _snapshot_decoder: SnapshotDecoder = SnapshotDecoderClass.new()
var _render_alpha: float = 0.0
var _overlay_enabled: bool = false
var _selected_entity_id: int = -1
var _relationships: Array[Dictionary] = []


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


func _ready() -> void:
	z_index = 0
	visible = true
	SimulationBus.entity_selected.connect(_on_entity_selected)
	SimulationBus.entity_deselected.connect(_on_entity_deselected)


func _process(_delta: float) -> void:
	if not _overlay_enabled or _selected_entity_id < 0:
		return
	_refresh_snapshots()
	queue_redraw()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_R:
		toggle()
		get_viewport().set_input_as_handled()


func toggle() -> void:
	_overlay_enabled = not _overlay_enabled
	if _overlay_enabled and _selected_entity_id >= 0:
		_refresh_relationships()
		SimulationBus.notify(Locale.ltr("UI_RELATIONSHIP_OVERLAY_TOGGLE"), "info")
	queue_redraw()


func show_for_entity(entity_id: int) -> void:
	_selected_entity_id = entity_id
	if _overlay_enabled:
		_refresh_relationships()
	queue_redraw()


func hide_overlay() -> void:
	_selected_entity_id = -1
	_relationships.clear()
	queue_redraw()


func _draw() -> void:
	if not _overlay_enabled or _selected_entity_id < 0:
		return
	var selected_pos: Vector2 = _get_agent_world_position(_selected_entity_id)
	if selected_pos == Vector2.INF:
		return

	for entry: Dictionary in _relationships:
		var target_id: int = int(entry.get("target_id", -1))
		if target_id < 0:
			continue
		var target_pos: Vector2 = _get_agent_world_position(target_id)
		if target_pos == Vector2.INF:
			continue
		var affinity: float = float(entry.get("affinity", 0.0))
		var line_color: Color = NEUTRAL_COLOR
		if affinity > FRIENDLY_THRESHOLD:
			line_color = FRIENDLY_COLOR
		elif affinity < HOSTILE_THRESHOLD:
			line_color = HOSTILE_COLOR
		var line_width: float = clampf(absf(affinity) * MAX_LINE_WIDTH, MIN_LINE_WIDTH, MAX_LINE_WIDTH)
		draw_line(selected_pos, target_pos, line_color, line_width, true)


func _refresh_snapshots() -> void:
	var curr_bytes: PackedByteArray = SimBridge.get_frame_snapshots()
	var prev_bytes: PackedByteArray = SimBridge.get_prev_frame_snapshots()
	var agent_count: int = SimBridge.get_agent_count()
	_render_alpha = clampf(SimBridge.get_render_alpha(), 0.0, 1.0)
	_snapshot_decoder.update(curr_bytes, prev_bytes, agent_count)


func _refresh_relationships() -> void:
	_relationships.clear()
	if _sim_engine == null or not _sim_engine.has_method("get_entity_tab") or _selected_entity_id < 0:
		return
	var social_tab: Dictionary = _sim_engine.get_entity_tab(_selected_entity_id, "social")
	var rows: Array = social_tab.get("relationships", [])
	for row_raw: Variant in rows:
		if row_raw is Dictionary:
			_relationships.append(row_raw)
	_relationships.sort_custom(func(left: Dictionary, right: Dictionary) -> bool:
		return absf(float(left.get("affinity", 0.0))) > absf(float(right.get("affinity", 0.0)))
	)


func _get_agent_world_position(entity_id: int) -> Vector2:
	for index: int in range(_snapshot_decoder.agent_count):
		if _snapshot_decoder.get_entity_id(index) == entity_id:
			return _tile_to_world_position(_snapshot_decoder.get_interpolated_position(index, _render_alpha))
	if _sim_engine != null and _sim_engine.has_method("get_entity_detail"):
		var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
		if not detail.is_empty():
			return _tile_to_world_position(Vector2(
				float(detail.get("x", 0.0)),
				float(detail.get("y", 0.0))
			))
	return Vector2.INF


func _tile_to_world_position(tile_position: Vector2) -> Vector2:
	var half_tile: Vector2 = Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
	return tile_position * float(GameConfig.TILE_SIZE) + half_tile


func _on_entity_selected(entity_id: int) -> void:
	show_for_entity(entity_id)


func _on_entity_deselected() -> void:
	hide_overlay()
