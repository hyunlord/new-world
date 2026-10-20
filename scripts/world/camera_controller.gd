# camera_controller.gd (Camera2D in scenes/main.tscn): stepped integer zoom (wheel, pinch, pan gesture), P pause by
#   disabling the WorldSim node's processing, T swarm auto-track, keys 1-4 sim speed, centring on the largest village.
# Bridge methods: set_sim_speed (direct); via SimReads get_agent_snapshot, get_settlement_snapshot, get_world_info;
#   signal world_reset.
extends Camera2D

# Integer ladder preserves native texels throughout camera transitions.
const ZOOM_STEPS: Array[float] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0]
const ZOOM_MIN: Vector2 = Vector2(1.0, 1.0)
const ZOOM_MAX: Vector2 = Vector2(8.0, 8.0)
const ZOOM_DEFAULT: Vector2 = Vector2(4.0, 4.0)
const TWEEN_DURATION: float = 0.12
const GESTURE_THRESHOLD: float = 0.25
var _gesture_accumulator: float = 0.0

# V7 H Phase A — camera agent-tracking coordinate basis. Mirrors
# world_renderer / agent_renderer (SPRITE_ORIGIN + tile*TILE_SIZE) so the
# tracked centroid lands on the same world-pixel grid the sprites use.
const TILE_SIZE: int = 16
const SPRITE_ORIGIN_X: int = 448
const SPRITE_ORIGIN_Y: int = 28
const CAMERA_TRACK_SPEED: float = 2.0          # lerp rate toward swarm centroid
const PAN_ZOOM_SENSITIVITY: float = 0.1        # trackpad two-finger-scroll → zoom

var _zoom_tween: Tween = null
var _target_zoom: Vector2 = ZOOM_DEFAULT

# V7 H Phase A — pause + tracking state. `_world_sim` is the Rust WorldSimNode
# whose process() drives the tick; gating its process_mode freezes the sim.
var _world_sim: Node = null
var _paused: bool = false
var _auto_track: bool = false
var _initial_center_pending: bool = true
var _manual_framing: bool = false


func _ready() -> void:
	# V7 Phase 13-α — print derives the default from ZOOM_DEFAULT so the
	# log never drifts from the constant (formerly hardcoded "2.0×").
	print("CameraController ready (default zoom %.1f×)" % ZOOM_DEFAULT.x)
	zoom = ZOOM_DEFAULT
	_target_zoom = ZOOM_DEFAULT
	# V7 H Phase A — resolve the WorldSim node once for pause + tracking.
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)
		_on_world_reset()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_step_zoom(1)
			get_viewport().set_input_as_handled()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_step_zoom(-1)
			get_viewport().set_input_as_handled()
	elif event is InputEventPanGesture:
		# V7 H Phase A — trackpad two-finger scroll → zoom (scroll up =
		# delta.y < 0 = zoom in). Mouse-wheel path above is unchanged.
		_accumulate_zoom(-event.delta.y * PAN_ZOOM_SENSITIVITY)
		get_viewport().set_input_as_handled()
	elif event is InputEventMagnifyGesture:
		# V7 H Phase A — trackpad pinch → zoom (factor > 1 = zoom in).
		_accumulate_zoom(event.factor - 1.0)
		get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo \
			and event.keycode == KEY_P:
		# V7 H Phase A — KEY_P toggles pause. KEY_SPACE is intentionally NOT
		# used here (it belongs to the world_renderer overlay-channel cycle).
		_toggle_pause()
		get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo \
			and event.keycode == KEY_T:
		set_auto_track(not _auto_track)
		get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo \
			and event.keycode in [KEY_1, KEY_2, KEY_3, KEY_4]:
		# V7 Section 16-γ — number keys drive the Rust WorldSimNode.set_sim_speed
		# accumulator scale (0.25× / 0.5× / 1× / 2×) so the player can slow the
		# sim and watch agents seek → walk → consume. Mirrors the KEY_P branch.
		var spd: float = {KEY_1: 0.25, KEY_2: 0.5, KEY_3: 1.0, KEY_4: 2.0}[event.keycode]
		if _world_sim != null:
			_world_sim.call("set_sim_speed", spd)
		print("[sim] speed = %.2fx" % spd)  # debug log — locale-exempt
		get_viewport().set_input_as_handled()


func _accumulate_zoom(amount: float) -> void:
	_gesture_accumulator += amount
	while absf(_gesture_accumulator) >= GESTURE_THRESHOLD:
		var direction: int = 1 if _gesture_accumulator > 0.0 else -1
		_step_zoom(direction)
		_gesture_accumulator -= float(direction) * GESTURE_THRESHOLD


func _nearest_step(value: float) -> float:
	var nearest: float = ZOOM_STEPS[0]
	for step: float in ZOOM_STEPS:
		if absf(value - step) < absf(value - nearest):
			nearest = step
	return nearest


func _set_quantized_zoom(value: float) -> void:
	zoom = Vector2.ONE * _nearest_step(value)


func _step_zoom(direction: int) -> void:
	var index: int = ZOOM_STEPS.find(_nearest_step(_target_zoom.x))
	index = clampi(index + direction, 0, ZOOM_STEPS.size() - 1)
	_target_zoom = Vector2.ONE * ZOOM_STEPS[index]
	if _zoom_tween != null and _zoom_tween.is_valid():
		_zoom_tween.kill()
	_zoom_tween = create_tween()
	_zoom_tween.tween_method(_set_quantized_zoom, zoom.x, _target_zoom.x, TWEEN_DURATION).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	_zoom_tween.tween_callback(func() -> void: zoom = _target_zoom)


func _toggle_pause() -> void:
	# V7 H Phase A — freeze/resume the simulation by gating WorldSimNode's
	# process() (the Rust tick) via process_mode. Disabling stops the tick
	# while leaving its #[func] getters callable, so renderers keep drawing
	# the frozen frame. Zero Rust change.
	_paused = not _paused
	if _world_sim != null:
		_world_sim.process_mode = (
			Node.PROCESS_MODE_DISABLED if _paused else Node.PROCESS_MODE_INHERIT
		)


func set_auto_track(enabled: bool) -> void:
	_auto_track = enabled
	_manual_framing = not enabled
	# Explicit framing (capture tools or player input) owns the camera now.
	_initial_center_pending = false


func _process(delta: float) -> void:
	if _initial_center_pending:
		_center_on_first_settlement()
	if not _auto_track:
		return
	# V7 H Phase A — gently track the agent swarm centroid so Brownian-
	# dispersing agents (AgentMovementSystem) stay on-screen. Reads the same
	# snapshot the renderers consume; converts the mean tile to world pixels
	# on the SPRITE_ORIGIN + tile*TILE_SIZE basis, then eases toward it.
	if _world_sim == null:
		return
	var snap: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("xs", null)
	var ys: Variant = snap_dict.get("ys", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array):
		return
	var xs_arr: PackedInt32Array = xs
	var ys_arr: PackedInt32Array = ys
	var n: int = xs_arr.size()
	if n == 0 or ys_arr.size() != n:
		return
	var sum_x: int = 0
	var sum_y: int = 0
	for i in n:
		sum_x += xs_arr[i]
		sum_y += ys_arr[i]
	var mean_tx: float = float(sum_x) / float(n)
	var mean_ty: float = float(sum_y) / float(n)
	var target: Vector2 = Vector2(
		float(SPRITE_ORIGIN_X) + mean_tx * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
		float(SPRITE_ORIGIN_Y) + mean_ty * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
	)
	position = position.lerp(target, clampf(delta * CAMERA_TRACK_SPEED, 0.0, 1.0))


func _tile_center(tile_x: float, tile_y: float) -> Vector2:
	return Vector2(
		float(SPRITE_ORIGIN_X) + tile_x * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
		float(SPRITE_ORIGIN_Y) + tile_y * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
	)


func _center_on_first_settlement() -> void:
	if _world_sim == null:
		return
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var ids: PackedInt32Array = snapshot.get("settlement_ids", PackedInt32Array())
	var counts: PackedInt32Array = snapshot.get("member_counts", PackedInt32Array())
	var xs: PackedInt32Array = snapshot.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("formation_ys", PackedInt32Array())
	if ids.is_empty() or counts.size() != ids.size() or xs.size() != ids.size() or ys.size() != ids.size():
		return
	var best: int = 0
	for index: int in range(1, ids.size()):
		if counts[index] > counts[best] or (counts[index] == counts[best] and ids[index] < ids[best]):
			best = index
	position = _tile_center(float(xs[best]), float(ys[best]))
	_initial_center_pending = false


func _on_world_reset() -> void:
	if _manual_framing or _world_sim == null:
		return
	_initial_center_pending = true
	var world_info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	position = _tile_center(float(world_info.get("width", 64)) / 2.0, float(world_info.get("height", 64)) / 2.0)
	_center_on_first_settlement()
