extends Camera2D

const EntityDetailPanelV3Class = preload("res://scripts/ui/panels/entity_detail_panel_v3.gd")
const SnapshotDecoderClass = preload("res://scripts/rendering/snapshot_decoder.gd")

signal camera_state_changed(new_state: int)
signal camera_idle()

enum CameraState {
	IDLE_WIDE,
	IDLE_MEDIUM,
	IDLE_CLOSE,
	IDLE_BREATHE,
	FOLLOW,
	DRONE,
	HARD_CUT,
	DRIFT,
}

const DRAG_THRESHOLD: float = 5.0
const DETAIL_PANEL_START_X_RATIO: float = 0.55
const IDLE_WIDE_DURATION: float = 15.0
const IDLE_MEDIUM_DURATION: float = 15.0
const IDLE_CLOSE_DURATION: float = 15.0
const IDLE_BREATHE_DURATION: float = 5.0
const INPUT_SUPPRESS_DURATION: float = 10.0
const IDLE_SIGNAL_DURATION: float = 30.0
const FOLLOW_LOOK_AHEAD: float = 80.0
const DRIFT_WEIGHT: float = 0.02
const DRONE_ZOOM_OUT_TIME: float = 0.3
const DRONE_PAN_TIME: float = 0.6
const DRONE_ZOOM_IN_TIME: float = 0.3
const HARD_CUT_FLASH_TIME: float = 0.15
const ZOOM_WIDE: Vector2 = Vector2(0.3, 0.3)
const ZOOM_MEDIUM: Vector2 = Vector2(0.7, 0.7)
const ZOOM_CLOSE: Vector2 = Vector2(1.2, 1.2)
const ZOOM_DEFAULT: Vector2 = Vector2(0.5, 0.5)
const ACTION_SOCIALIZE: int = 6
const ACTION_EXPLORE: int = 11
const ACTION_FIGHT: int = 13
const ACTION_MENTAL_BREAK: int = 17

var current_state: int = CameraState.IDLE_WIDE
var state_timer: float = 0.0
var last_input_time: float = INPUT_SUPPRESS_DURATION
var follow_target_id: int = -1
var active_tween: Tween = null
var interest_target_id: int = -1
var drift_target: Vector2 = Vector2.ZERO

var _target_zoom: float = ZOOM_DEFAULT.x
var _is_dragging: bool = false
var _drag_start: Vector2 = Vector2.ZERO
var _left_dragging: bool = false
var _left_drag_start: Vector2 = Vector2.ZERO
var _left_was_dragged: bool = false
var _entity_manager: RefCounted
var _sim_engine: RefCounted
var _snapshot_decoder = SnapshotDecoderClass.new()
var _render_alpha: float = 0.0
var _idle_signal_emitted: bool = false
var _last_interest_target_id: int = -1
var _flash_layer: CanvasLayer
var _flash_rect: ColorRect
var _cast_bar: Variant = null
var _notification_manager: Variant = null
var _probe_observation_mode: bool = true
var _last_move_reason: String = "none"


func _ready() -> void:
	var center_px: Vector2 = _world_center_px()
	position = center_px
	zoom = ZOOM_WIDE
	_target_zoom = ZOOM_WIDE.x
	_update_camera_limits()
	make_current()
	_build_flash_overlay()
	_transition_to(CameraState.IDLE_WIDE)


func set_entity_manager(em: RefCounted) -> void:
	_entity_manager = em


func set_sim_engine(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


func set_probe_observation_mode(probe_enabled: bool) -> void:
	_probe_observation_mode = probe_enabled
	_clear_active_tween()
	if follow_target_id >= 0:
		stop_following()
	elif probe_enabled and current_state != CameraState.IDLE_MEDIUM:
		_transition_to(CameraState.IDLE_MEDIUM)


func get_verification_camera_debug() -> Dictionary:
	return {
		"probe_mode": _probe_observation_mode,
		"state": _camera_state_name(current_state),
		"last_move_reason": _last_move_reason,
		"follow_target_id": follow_target_id,
	}


func connect_ui_sources(cast_bar: Variant, notification_manager: Variant) -> void:
	if cast_bar != null:
		_cast_bar = cast_bar
		if not _cast_bar.agent_selected.is_connected(_on_ui_agent_selected):
			_cast_bar.agent_selected.connect(_on_ui_agent_selected)
		if not _cast_bar.agent_follow_requested.is_connected(_on_ui_agent_follow_requested):
			_cast_bar.agent_follow_requested.connect(_on_ui_agent_follow_requested)
	if notification_manager != null:
		_notification_manager = notification_manager
		if not _notification_manager.notification_clicked.is_connected(_on_ui_notification_clicked):
			_notification_manager.notification_clicked.connect(_on_ui_notification_clicked)
		if not _notification_manager.crisis_occurred.is_connected(_on_ui_crisis_occurred):
			_notification_manager.crisis_occurred.connect(_on_ui_crisis_occurred)


func follow_entity(entity_id: int) -> void:
	if entity_id < 0:
		return
	_mark_player_input(false)
	_clear_active_tween()
	follow_target_id = entity_id
	position_smoothing_enabled = false
	_note_camera_move("follow_request")
	_transition_to(CameraState.FOLLOW)


func stop_following() -> void:
	if follow_target_id < 0:
		return
	follow_target_id = -1
	position_smoothing_enabled = true
	_note_camera_move("stop_follow")
	_transition_to(CameraState.IDLE_MEDIUM)
	SimulationBus.follow_entity_stopped.emit()


func is_following() -> bool:
	return follow_target_id >= 0 and current_state == CameraState.FOLLOW


func get_following_id() -> int:
	return follow_target_id


var _zoom_tween: Tween

func set_target_zoom(value: float) -> void:
	_mark_player_input(false)
	_note_camera_move("manual_input")
	_target_zoom = clampf(value, GameConfig.CAMERA_ZOOM_MIN, GameConfig.CAMERA_ZOOM_MAX)
	zoom = zoom.lerp(Vector2(_target_zoom, _target_zoom), GameConfig.CAMERA_ZOOM_SPEED)
	position = _clamp_to_world_position(position)


## Smooth zoom transition for UI button clicks (faster than per-frame lerp).
func animate_zoom_to(target_value: float, duration: float = 0.35) -> void:
	_mark_player_input(false)
	_note_camera_move("zoom_button")
	target_value = clampf(target_value, GameConfig.CAMERA_ZOOM_MIN, GameConfig.CAMERA_ZOOM_MAX)
	_target_zoom = target_value

	if _zoom_tween != null:
		_zoom_tween.kill()

	_zoom_tween = create_tween().set_ease(Tween.EASE_IN_OUT).set_trans(Tween.TRANS_CUBIC)
	_zoom_tween.tween_property(self, "zoom", Vector2(target_value, target_value), duration)
	_zoom_tween.tween_callback(func() -> void:
		position = _clamp_to_world_position(position)
	)


func get_zoom_level() -> int:
	var zoom_value: float = _target_zoom if _is_player_active() else zoom.x
	for i in range(GameConfig.ZOOM_TIER_BOUNDARIES.size()):
		if zoom_value >= GameConfig.ZOOM_TIER_BOUNDARIES[i]:
			return i
	return GameConfig.ZOOM_TIER_COUNT - 1


func focus_world_tile(tile_position: Vector2, reason: String = "manual_focus") -> void:
	_mark_player_input(false)
	_clear_active_tween()
	if follow_target_id >= 0:
		stop_following()
	position = _clamp_to_world_position(_tile_to_world_position(tile_position))
	_note_camera_move(reason)
	_transition_to(CameraState.IDLE_MEDIUM)


func focus_entity(entity_id: int, reason: String = "manual_focus") -> void:
	if entity_id < 0:
		return
	var target_pos: Vector2 = _get_agent_world_position(entity_id)
	if target_pos == Vector2.INF:
		return
	_mark_player_input(false)
	_clear_active_tween()
	if follow_target_id >= 0:
		stop_following()
	_last_interest_target_id = interest_target_id
	interest_target_id = entity_id
	position = _clamp_to_world_position(target_pos)
	_note_camera_move(reason)
	_transition_to(CameraState.IDLE_CLOSE)


func setup_navigation(nav_manager: Node) -> void:
	if nav_manager and not nav_manager.entity_focus_requested.is_connected(_on_entity_focus_requested):
		nav_manager.entity_focus_requested.connect(_on_entity_focus_requested)


func _on_entity_focus_requested(entity_id: int, target_position: Vector2) -> void:
	handle_notification_clicked(entity_id, target_position)


func handle_notification_clicked(entity_id: int, target_position: Vector2) -> void:
	if entity_id >= 0:
		_last_interest_target_id = interest_target_id
		interest_target_id = entity_id
	_note_camera_move("notification")
	trigger_drone_shot(_tile_to_world_position(target_position), true)


func handle_crisis(entity_id: int, target_position: Vector2) -> void:
	if entity_id >= 0:
		_last_interest_target_id = interest_target_id
		interest_target_id = entity_id
	_note_camera_move("crisis")
	trigger_hard_cut(_tile_to_world_position(target_position))


func trigger_hard_cut(target_position: Vector2) -> void:
	if _probe_observation_mode:
		return
	_clear_active_tween()
	if follow_target_id >= 0:
		stop_following()
	position = _clamp_to_world_position(target_position)
	zoom = ZOOM_CLOSE
	_target_zoom = ZOOM_CLOSE.x
	_note_camera_move("crisis")
	_transition_to(CameraState.HARD_CUT)
	_show_flash()
	active_tween = create_tween()
	active_tween.tween_interval(HARD_CUT_FLASH_TIME)
	active_tween.tween_callback(Callable(self, "_finish_hard_cut"))


func trigger_drone_shot(target_position: Vector2, force: bool = false) -> void:
	if _probe_observation_mode:
		return
	if not force and _is_player_active():
		return
	_clear_active_tween()
	if follow_target_id >= 0:
		stop_following()
	_note_camera_move("notification")
	_transition_to(CameraState.DRONE)
	var clamped_target: Vector2 = _clamp_to_world_position(target_position)
	active_tween = create_tween().set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
	active_tween.tween_property(self, "zoom", ZOOM_WIDE, DRONE_ZOOM_OUT_TIME)
	active_tween.tween_property(self, "position", clamped_target, DRONE_PAN_TIME)
	active_tween.tween_property(self, "zoom", ZOOM_CLOSE, DRONE_ZOOM_IN_TIME)
	active_tween.tween_callback(Callable(self, "_finish_drone_shot"))


func trigger_drift(target_position: Vector2) -> void:
	if _probe_observation_mode:
		return
	if _is_player_active() or current_state == CameraState.FOLLOW:
		return
	drift_target = _clamp_to_world_position(target_position)
	_note_camera_move("idle_drift")
	_transition_to(CameraState.DRIFT)


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP or event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			if _is_pointer_over_detail_panel():
				return
	if event is InputEventPanGesture and _is_pointer_over_detail_panel():
		return

	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			_left_dragging = true
			_left_drag_start = event.position
			_left_was_dragged = false
		else:
			_left_dragging = false
			if _left_was_dragged:
				get_viewport().set_input_as_handled()

	if event is InputEventMouseButton:
		if event.pressed:
			if event.button_index == MOUSE_BUTTON_WHEEL_UP:
				_mark_player_input(false)
				_note_camera_move("manual_input")
				_zoom_at_mouse(GameConfig.CAMERA_ZOOM_STEP)
				get_viewport().set_input_as_handled()
			elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				_mark_player_input(false)
				_note_camera_move("manual_input")
				_zoom_at_mouse(-GameConfig.CAMERA_ZOOM_STEP)
				get_viewport().set_input_as_handled()
			elif event.button_index == MOUSE_BUTTON_MIDDLE:
				_mark_player_input(true)
				_note_camera_move("manual_input")
				_is_dragging = true
				_drag_start = event.position
				get_viewport().set_input_as_handled()
		elif event.button_index == MOUSE_BUTTON_MIDDLE:
			_is_dragging = false

	if event is InputEventMouseMotion:
		if _is_dragging:
			_mark_player_input(true)
			_note_camera_move("manual_input")
			position += (_drag_start - event.position) / zoom.x
			_drag_start = event.position
			get_viewport().set_input_as_handled()
		elif _left_dragging:
			var moved: float = event.position.distance_to(_left_drag_start)
			if moved > DRAG_THRESHOLD:
				_left_was_dragged = true
			if _left_was_dragged:
				_mark_player_input(true)
				_note_camera_move("manual_input")
				position += -event.relative / zoom.x
				get_viewport().set_input_as_handled()

	if event is InputEventMagnifyGesture:
		_mark_player_input(false)
		_note_camera_move("manual_input")
		var zoom_delta: float = (event.factor - 1.0) * 0.5
		_zoom_at_mouse(zoom_delta)
		get_viewport().set_input_as_handled()

	if event is InputEventPanGesture:
		_mark_player_input(true)
		_note_camera_move("manual_input")
		position += event.delta * 2.0 / zoom.x
		get_viewport().set_input_as_handled()

	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_ESCAPE and follow_target_id >= 0:
			_mark_player_input(false)
			stop_following()
			get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	_refresh_snapshots()
	state_timer += delta
	last_input_time += delta

	if not _idle_signal_emitted and last_input_time >= IDLE_SIGNAL_DURATION:
		_idle_signal_emitted = true
		camera_idle.emit()

	if _probe_observation_mode:
		# Allow follow even in probe mode
		if follow_target_id >= 0 and current_state == CameraState.FOLLOW:
			_process_follow(delta)
		else:
			_smooth_zoom(Vector2(_target_zoom, _target_zoom), delta)
		_process_manual_pan(delta)
		position = _clamp_to_world_position(position)
		return

	match current_state:
		CameraState.IDLE_WIDE:
			_process_idle_wide(delta)
		CameraState.IDLE_MEDIUM:
			_process_idle_medium(delta)
		CameraState.IDLE_CLOSE:
			_process_idle_close(delta)
		CameraState.IDLE_BREATHE:
			_process_idle_breathe(delta)
		CameraState.FOLLOW:
			_process_follow(delta)
		CameraState.DRIFT:
			_process_drift(delta)
		_:
			pass

	_process_manual_pan(delta)
	position = _clamp_to_world_position(position)


func _process_idle_wide(delta: float) -> void:
	if _is_player_active():
		_smooth_zoom(Vector2(_target_zoom, _target_zoom), delta)
		return
	_smooth_zoom(ZOOM_WIDE, delta)
	position = position.lerp(_compute_population_center(), _lerp_alpha(delta, 1.5))
	if state_timer >= IDLE_WIDE_DURATION:
		_transition_to(CameraState.IDLE_MEDIUM)


func _process_idle_medium(delta: float) -> void:
	if _is_player_active():
		_smooth_zoom(Vector2(_target_zoom, _target_zoom), delta)
		return
	_smooth_zoom(ZOOM_MEDIUM, delta)
	position = position.lerp(_compute_population_center(), _lerp_alpha(delta, 2.0))
	if state_timer >= IDLE_MEDIUM_DURATION:
		_last_interest_target_id = interest_target_id
		interest_target_id = _find_most_interesting_agent()
		_transition_to(CameraState.IDLE_CLOSE)


func _process_idle_close(delta: float) -> void:
	if _is_player_active():
		_smooth_zoom(Vector2(_target_zoom, _target_zoom), delta)
		return
	_smooth_zoom(ZOOM_CLOSE, delta)
	if interest_target_id >= 0:
		var target_pos: Vector2 = _get_agent_world_position(interest_target_id)
		if target_pos != Vector2.INF:
			position = position.lerp(target_pos, _lerp_alpha(delta, 3.0))
	if state_timer >= IDLE_CLOSE_DURATION:
		_transition_to(CameraState.IDLE_BREATHE)


func _process_idle_breathe(delta: float) -> void:
	if _is_player_active():
		_smooth_zoom(Vector2(_target_zoom, _target_zoom), delta)
		return
	_smooth_zoom(ZOOM_MEDIUM, delta)
	position = position.lerp(_compute_population_center(), _lerp_alpha(delta, 2.0))
	if state_timer >= IDLE_BREATHE_DURATION:
		_transition_to(CameraState.IDLE_WIDE)


func _process_follow(delta: float) -> void:
	if follow_target_id < 0:
		_transition_to(CameraState.IDLE_MEDIUM)
		return
	var target_snapshot: Dictionary = _get_follow_target_snapshot(follow_target_id)
	if target_snapshot.is_empty():
		stop_following()
		return
	var target_pos: Vector2 = _tile_to_world_position(Vector2(
		float(target_snapshot.get("x", 0.0)),
		float(target_snapshot.get("y", 0.0))
	))
	var velocity: Vector2 = Vector2(
		float(target_snapshot.get("vel_x", 0.0)),
		float(target_snapshot.get("vel_y", 0.0))
	)
	var look_ahead: Vector2 = Vector2.ZERO
	if velocity.length_squared() > 0.0001:
		look_ahead = velocity.normalized() * FOLLOW_LOOK_AHEAD
	position = position.lerp(
		_clamp_to_world_position(target_pos + look_ahead),
		_lerp_alpha(delta, 5.0)
	)
	if _is_player_active():
		_smooth_zoom(Vector2(_target_zoom, _target_zoom), delta)
	else:
		_smooth_zoom(ZOOM_CLOSE, delta)


func _process_drift(_delta: float) -> void:
	position += (drift_target - position) * DRIFT_WEIGHT
	if position.distance_to(drift_target) < 10.0 or state_timer > 10.0:
		_transition_to(CameraState.IDLE_MEDIUM)


func _process_manual_pan(delta: float) -> void:
	var pan_dir: Vector2 = Vector2.ZERO
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP):
		pan_dir.y -= 1.0
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN):
		pan_dir.y += 1.0
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT):
		pan_dir.x -= 1.0
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT):
		pan_dir.x += 1.0
	if pan_dir == Vector2.ZERO:
		return
	_mark_player_input(true)
	_note_camera_move("manual_input")
	position += pan_dir.normalized() * GameConfig.CAMERA_PAN_SPEED * delta / max(zoom.x, 0.001)


func _transition_to(new_state: int) -> void:
	current_state = new_state
	state_timer = 0.0
	camera_state_changed.emit(new_state)


func _finish_hard_cut() -> void:
	active_tween = null
	_transition_to(CameraState.IDLE_CLOSE)


func _finish_drone_shot() -> void:
	active_tween = null
	_transition_to(CameraState.IDLE_CLOSE)


func _refresh_snapshots() -> void:
	var curr_bytes: PackedByteArray = SimBridge.get_frame_snapshots()
	var prev_bytes: PackedByteArray = SimBridge.get_prev_frame_snapshots()
	var agent_count: int = SimBridge.get_agent_count()
	_render_alpha = clampf(SimBridge.get_render_alpha(), 0.0, 1.0)
	_snapshot_decoder.update(curr_bytes, prev_bytes, agent_count)


func _find_most_interesting_agent() -> int:
	if not _snapshot_decoder.has_data():
		return -1
	var best_id: int = -1
	var best_score: float = -999.0
	for index: int in range(_snapshot_decoder.agent_count):
		var entity_id: int = _snapshot_decoder.get_entity_id(index)
		var score: float = 0.0
		var action: int = _snapshot_decoder.get_action_state(index)
		if action == ACTION_FIGHT:
			score += 50.0
		elif action == ACTION_SOCIALIZE:
			score += 20.0
		elif action == ACTION_EXPLORE:
			score += 10.0
		var health_tier: int = _snapshot_decoder.get_health_tier(index)
		if health_tier == 0:
			score += 80.0
		elif health_tier == 1:
			score += 30.0
		score += float(_snapshot_decoder.get_stress_phase(index)) * 15.0
		if _snapshot_decoder.get_active_break(index) > 0 or action == ACTION_MENTAL_BREAK:
			score += 100.0
		if entity_id == _last_interest_target_id:
			score -= 40.0
		if score > best_score:
			best_score = score
			best_id = entity_id
	return best_id


func _compute_population_center() -> Vector2:
	if not _snapshot_decoder.has_data():
		return _world_center_px()
	var total: Vector2 = Vector2.ZERO
	var count: int = 0
	for index: int in range(_snapshot_decoder.agent_count):
		total += _tile_to_world_position(_snapshot_decoder.get_interpolated_position(index, _render_alpha))
		count += 1
	if count <= 0:
		return _world_center_px()
	return _clamp_to_world_position(total / float(count))


func _mark_player_input(cancel_follow: bool) -> void:
	last_input_time = 0.0
	_idle_signal_emitted = false
	if current_state == CameraState.DRONE or current_state == CameraState.DRIFT:
		_clear_active_tween()
		_transition_to(CameraState.IDLE_MEDIUM)
	if cancel_follow and current_state == CameraState.FOLLOW:
		stop_following()


func _clear_active_tween() -> void:
	if active_tween != null and is_instance_valid(active_tween):
		active_tween.kill()
	active_tween = null


func _is_player_active() -> bool:
	return last_input_time < INPUT_SUPPRESS_DURATION


func _lerp_alpha(delta: float, speed: float) -> float:
	return 1.0 - pow(0.05, delta * speed)


func _smooth_zoom(target: Vector2, delta: float) -> void:
	if _zoom_tween != null and _zoom_tween.is_running():
		position = _clamp_to_world_position(position)
		return
	zoom = zoom.lerp(target, 1.0 - pow(0.1, delta * 4.0))


func _zoom_at_mouse(delta: float) -> void:
	var direction: int = 0
	if delta > 0.0:
		direction = -1
	elif delta < 0.0:
		direction = 1
	if direction == 0:
		return
	var next_level: int = clampi(get_zoom_level() + direction, 0, GameConfig.CAMERA_ZOOM_LEVELS.size() - 1)
	set_target_zoom(float(GameConfig.CAMERA_ZOOM_LEVELS[next_level]))
	zoom = zoom.lerp(Vector2(_target_zoom, _target_zoom), GameConfig.CAMERA_ZOOM_SPEED)


func _is_pointer_over_detail_panel() -> bool:
	if not EntityDetailPanelV3Class.is_open:
		return false
	var viewport: Viewport = get_viewport()
	if viewport == null:
		return false
	var viewport_width: float = viewport.get_visible_rect().size.x
	return viewport.get_mouse_position().x > viewport_width * DETAIL_PANEL_START_X_RATIO


func _get_follow_target_snapshot(entity_id: int) -> Dictionary:
	var index: int = _find_agent_index(entity_id)
	if index >= 0:
		var tile_pos: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
		var velocity: Vector2 = _snapshot_decoder.get_velocity(index)
		return {
			"x": tile_pos.x,
			"y": tile_pos.y,
			"vel_x": velocity.x,
			"vel_y": velocity.y,
		}
	if _sim_engine != null and _sim_engine.has_method("get_entity_detail"):
		var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
		if not detail.is_empty() and bool(detail.get("alive", true)):
			return {
				"x": float(detail.get("x", 0.0)),
				"y": float(detail.get("y", 0.0)),
				"vel_x": 0.0,
				"vel_y": 0.0,
			}
	if _entity_manager != null and _entity_manager.has_method("get_entity"):
		var entity: Variant = _entity_manager.get_entity(entity_id)
		if entity != null and entity.is_alive:
			var entity_pos: Vector2 = Vector2(entity.position)
			return {
				"x": entity_pos.x,
				"y": entity_pos.y,
				"vel_x": 0.0,
				"vel_y": 0.0,
			}
	return {}


func _get_agent_world_position(entity_id: int) -> Vector2:
	var snapshot: Dictionary = _get_follow_target_snapshot(entity_id)
	if snapshot.is_empty():
		return Vector2.INF
	return _tile_to_world_position(Vector2(
		float(snapshot.get("x", 0.0)),
		float(snapshot.get("y", 0.0))
	))


func _find_agent_index(entity_id: int) -> int:
	if not _snapshot_decoder.has_data():
		return -1
	for index: int in range(_snapshot_decoder.agent_count):
		if _snapshot_decoder.get_entity_id(index) == entity_id:
			return index
	return -1


func _tile_to_world_position(tile_position: Vector2) -> Vector2:
	var half_tile: Vector2 = Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
	return tile_position * float(GameConfig.TILE_SIZE) + half_tile


func _world_center_px() -> Vector2:
	return Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE * 0.5


func _clamp_to_world_position(target_px: Vector2) -> Vector2:
	var world_px: Vector2 = Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE
	var half_view: Vector2 = _viewport_world_half_extents()
	var min_x: float = half_view.x
	var min_y: float = half_view.y
	var max_x: float = world_px.x - half_view.x
	var max_y: float = world_px.y - half_view.y
	if min_x > max_x:
		min_x = world_px.x * 0.5
		max_x = min_x
	if min_y > max_y:
		min_y = world_px.y * 0.5
		max_y = min_y
	return Vector2(
		clampf(target_px.x, min_x, max_x),
		clampf(target_px.y, min_y, max_y)
	)


func _viewport_world_half_extents() -> Vector2:
	var viewport: Viewport = get_viewport()
	if viewport == null:
		return Vector2.ZERO
	var visible_rect: Rect2 = viewport.get_visible_rect()
	var current_zoom: float = maxf(zoom.x, GameConfig.CAMERA_ZOOM_MIN)
	return visible_rect.size / current_zoom * 0.5


func _update_camera_limits() -> void:
	var world_px: Vector2 = Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE
	limit_left = 0
	limit_top = 0
	limit_right = int(world_px.x)
	limit_bottom = int(world_px.y)


func _build_flash_overlay() -> void:
	_flash_layer = CanvasLayer.new()
	_flash_layer.layer = 100
	_flash_layer.name = "CameraFlashLayer"
	add_child(_flash_layer)

	_flash_rect = ColorRect.new()
	_flash_rect.visible = false
	_flash_rect.color = Color(1.0, 1.0, 1.0, 0.0)
	_flash_rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_flash_rect.set_anchors_preset(Control.PRESET_FULL_RECT)
	_flash_layer.add_child(_flash_rect)


func _show_flash() -> void:
	if _flash_rect == null:
		return
	_flash_rect.visible = true
	_flash_rect.color = Color(1.0, 1.0, 1.0, 0.8)
	var flash_tween: Tween = create_tween()
	flash_tween.tween_property(_flash_rect, "color:a", 0.0, HARD_CUT_FLASH_TIME)
	flash_tween.tween_callback(func() -> void:
		_flash_rect.visible = false
	)


func _on_ui_agent_selected(entity_id: int) -> void:
	if _probe_observation_mode:
		return
	focus_entity(entity_id, "selection")


func _on_ui_agent_follow_requested(entity_id: int) -> void:
	if _probe_observation_mode:
		focus_entity(entity_id, "follow_request")
		return
	follow_entity(entity_id)


func _on_ui_notification_clicked(entity_id: int, target_position: Vector2) -> void:
	if _probe_observation_mode:
		return
	handle_notification_clicked(entity_id, target_position)


func _on_ui_crisis_occurred(entity_id: int, target_position: Vector2) -> void:
	if _probe_observation_mode:
		return
	handle_crisis(entity_id, target_position)


func _camera_state_name(state: int) -> String:
	match state:
		CameraState.IDLE_WIDE:
			return "idle_wide"
		CameraState.IDLE_MEDIUM:
			return "idle_medium"
		CameraState.IDLE_CLOSE:
			return "idle_close"
		CameraState.IDLE_BREATHE:
			return "idle_breathe"
		CameraState.FOLLOW:
			return "follow"
		CameraState.DRONE:
			return "drone"
		CameraState.HARD_CUT:
			return "hard_cut"
		CameraState.DRIFT:
			return "drift"
	return "idle_medium"


func _note_camera_move(reason: String) -> void:
	_last_move_reason = reason
