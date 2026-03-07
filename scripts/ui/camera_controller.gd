extends Camera2D

const EntityDetailPanelV3Class = preload("res://scripts/ui/panels/entity_detail_panel_v3.gd")

var _target_zoom: float = 1.0
var _is_dragging: bool = false
var _drag_start: Vector2 = Vector2.ZERO

## Left-click drag pan state
var _left_dragging: bool = false
var _left_drag_start: Vector2 = Vector2.ZERO
var _left_was_dragged: bool = false
const DRAG_THRESHOLD: float = 5.0
## Fraction of viewport width from the left edge where detail panel begins.
## Right 45% of screen is reserved for the entity detail panel.
const DETAIL_PANEL_START_X_RATIO: float = 0.55

## Entity following
var _following_entity_id: int = -1
var _entity_manager: RefCounted
var _sim_engine: RefCounted


func _ready() -> void:
	# Start at world center
	var center_px := Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE * 0.5
	position = center_px
	zoom = Vector2(1.5, 1.5)
	_target_zoom = 1.5
	make_current()


## Sets the entity manager used to look up entity positions during follow mode.
func set_entity_manager(em: RefCounted) -> void:
	_entity_manager = em


## Sets the simulation engine used to follow Rust-authoritative entity snapshots.
func set_sim_engine(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


## Begins following the given entity, emitting a follow request on SimulationBus.
func follow_entity(entity_id: int) -> void:
	_following_entity_id = entity_id
	SimulationBus.follow_entity_requested.emit(entity_id)


## Stops following the current entity and emits a follow-stopped signal.
func stop_following() -> void:
	if _following_entity_id >= 0:
		_following_entity_id = -1
		SimulationBus.follow_entity_stopped.emit()


## Returns true if the camera is currently tracking an entity.
func is_following() -> bool:
	return _following_entity_id >= 0


## Returns the ID of the entity currently being followed, or -1 if none.
func get_following_id() -> int:
	return _following_entity_id


func focus_world_tile(tile_position: Vector2) -> void:
	if _following_entity_id >= 0:
		stop_following()
	var target_px: Vector2 = tile_position * GameConfig.TILE_SIZE + Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
	position = _clamp_to_world_position(target_px)


func focus_entity(entity_id: int) -> void:
	var snapshot: Dictionary = _get_follow_target_snapshot(entity_id)
	if snapshot.is_empty():
		return
	focus_world_tile(Vector2(
		float(snapshot.get("x", 0.0)),
		float(snapshot.get("y", 0.0))
	))


func _unhandled_input(event: InputEvent) -> void:
	# ★ FIX: When detail panel is open and pointer is over it, skip camera scroll gestures.
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP or event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			if _is_pointer_over_detail_panel():
				return  # don't zoom — let entity_detail_panel_v2 scroll
	if event is InputEventPanGesture and _is_pointer_over_detail_panel():
		return  # don't pan camera — let entity_detail_panel_v2 scroll

	# Left-click drag pan
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			_left_dragging = true
			_left_drag_start = event.position
			_left_was_dragged = false
		else:
			_left_dragging = false
			if _left_was_dragged:
				# Was a drag — consume so EntityRenderer doesn't select
				get_viewport().set_input_as_handled()

	# Mouse wheel zoom
	if event is InputEventMouseButton:
		if event.pressed:
			if event.button_index == MOUSE_BUTTON_WHEEL_UP:
				_zoom_at_mouse(GameConfig.CAMERA_ZOOM_STEP)
				get_viewport().set_input_as_handled()
			elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				_zoom_at_mouse(-GameConfig.CAMERA_ZOOM_STEP)
				get_viewport().set_input_as_handled()
			elif event.button_index == MOUSE_BUTTON_MIDDLE:
				_is_dragging = true
				_drag_start = event.position
				get_viewport().set_input_as_handled()
		else:
			if event.button_index == MOUSE_BUTTON_MIDDLE:
				_is_dragging = false

	# Mouse motion: middle-click drag or left-click drag
	if event is InputEventMouseMotion:
		if _is_dragging:
			position += (_drag_start - event.position) / zoom.x
			_drag_start = event.position
			if _following_entity_id >= 0:
				stop_following()
			get_viewport().set_input_as_handled()
		elif _left_dragging:
			var moved: float = event.position.distance_to(_left_drag_start)
			if moved > DRAG_THRESHOLD:
				_left_was_dragged = true
			if _left_was_dragged:
				position += -event.relative / zoom.x
				if _following_entity_id >= 0:
					stop_following()
				get_viewport().set_input_as_handled()

	# macOS trackpad: pinch zoom
	if event is InputEventMagnifyGesture:
		var zoom_delta: float = (event.factor - 1.0) * 0.5
		_zoom_at_mouse(zoom_delta)
		get_viewport().set_input_as_handled()

	# macOS trackpad: two-finger scroll pan
	if event is InputEventPanGesture:
		position += event.delta * 2.0 / zoom.x
		get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	# Entity following
	if _following_entity_id >= 0:
		var followed: Dictionary = _get_follow_target_snapshot(_following_entity_id)
		if not followed.is_empty():
			var target := Vector2(
				float(followed.get("x", 0)),
				float(followed.get("y", 0))
			) * GameConfig.TILE_SIZE + Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
			position = position.lerp(target, 5.0 * delta)
		elif _entity_manager != null:
			var entity: RefCounted = _entity_manager.get_entity(_following_entity_id)
			if entity != null and entity.is_alive:
				var legacy_target := Vector2(entity.position) * GameConfig.TILE_SIZE + Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
				position = position.lerp(legacy_target, 5.0 * delta)
			elif entity != null:
				stop_following()

	# Smooth zoom
	var new_zoom: float = lerpf(zoom.x, _target_zoom, GameConfig.CAMERA_ZOOM_SPEED)
	zoom = Vector2(new_zoom, new_zoom)

	# WASD / arrow key panning
	var pan_dir := Vector2.ZERO
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP):
		pan_dir.y -= 1
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN):
		pan_dir.y += 1
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT):
		pan_dir.x -= 1
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT):
		pan_dir.x += 1
	if pan_dir != Vector2.ZERO:
		position += pan_dir.normalized() * GameConfig.CAMERA_PAN_SPEED * delta / zoom.x
		if _following_entity_id >= 0:
			stop_following()

	# Clamp to world bounds
	var world_px := Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE
	position.x = clampf(position.x, 0, world_px.x)
	position.y = clampf(position.y, 0, world_px.y)


func _zoom_at_mouse(delta: float) -> void:
	_target_zoom = clampf(_target_zoom + delta, GameConfig.CAMERA_ZOOM_MIN, GameConfig.CAMERA_ZOOM_MAX)


func _is_pointer_over_detail_panel() -> bool:
	if not EntityDetailPanelV3Class.is_open:
		return false
	var viewport: Viewport = get_viewport()
	if viewport == null:
		return false
	var vp_w: float = viewport.get_visible_rect().size.x
	return viewport.get_mouse_position().x > vp_w * DETAIL_PANEL_START_X_RATIO


func _get_follow_target_snapshot(entity_id: int) -> Dictionary:
	if _sim_engine != null and _sim_engine.has_method("get_agent_snapshots"):
		var snapshots: Array = _sim_engine.get_agent_snapshots()
		for i in range(snapshots.size()):
			var snapshot_raw: Variant = snapshots[i]
			if not (snapshot_raw is Dictionary):
				continue
			var snapshot: Dictionary = snapshot_raw
			if int(snapshot.get("entity_id", -1)) == entity_id:
				return snapshot
	if _sim_engine != null and _sim_engine.has_method("get_entity_detail"):
		var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
		if not detail.is_empty() and bool(detail.get("alive", true)):
			return {
				"x": detail.get("x", 0),
				"y": detail.get("y", 0),
			}
	return {}


func _clamp_to_world_position(target_px: Vector2) -> Vector2:
	var world_px: Vector2 = Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE
	return Vector2(
		clampf(target_px.x, 0.0, world_px.x),
		clampf(target_px.y, 0.0, world_px.y)
	)
