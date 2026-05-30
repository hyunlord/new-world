extends Camera2D

# V7 Phase 12-α — Camera2D zoom controller.
#
# Provides mouse-wheel zoom-in / zoom-out with smooth tween
# interpolation between 0.5× (overview, whole 64×64 world fits) and
# 4.0× (sprite-detail, 64×72 agent sprite renders at 64×72 px). Default
# zoom 2.0× brings the D1 STATE_TINTS palette (Phase 11-α + 450f39cd)
# into clearly observable resolution (16×18 px → 32×36 px on screen).
#
# Geometric zoom step (1.25× per wheel notch) gives ~4 notches to
# traverse the full 0.5×–4.0× range, matching common 2D-camera UX.
# Tween duration is short enough (0.15 s) that successive wheel
# notches feel responsive but smooth enough that the change is not
# a hard snap.
#
# Phase 4-γ SPRITE_SCALE invariant is preserved — this controller
# only modifies Camera2D.zoom, never agent sprite scale.

const ZOOM_MIN: Vector2 = Vector2(0.5, 0.5)
# V7 G Phase A — zoom-in ceiling raised 4.0 → 8.0 so the default view keeps
# ample zoom-in headroom. (B-1 later raised ZOOM_DEFAULT 3.0 → 5.0; from the
# 5.0 default the ×1.25 wheel still reaches the 8.0 ceiling.) ZOOM_MIN unchanged.
const ZOOM_MAX: Vector2 = Vector2(8.0, 8.0)
# V7 B-1 — default zoom raised 3.0× → 5.0× so agents (16×24 frame ×
# SPRITE_SCALE 0.25) render at 20×30 px instead of 12×18 px at the default
# view (SPRITE_SCALE is a Phase 4-γ invariant locked in 14 harnesses, so we
# scale the view, not the sprite). ZOOM_MIN (0.5) / ZOOM_MAX (8.0) unchanged,
# so the trackpad/wheel still reach the overview and the 8.0× close-up.
const ZOOM_DEFAULT: Vector2 = Vector2(5.0, 5.0)
const ZOOM_FACTOR: float = 1.25
const TWEEN_DURATION: float = 0.15

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


func _ready() -> void:
	# V7 Phase 13-α — print derives the default from ZOOM_DEFAULT so the
	# log never drifts from the constant (formerly hardcoded "2.0×").
	print("CameraController ready (default zoom %.1f×)" % ZOOM_DEFAULT.x)
	zoom = ZOOM_DEFAULT
	_target_zoom = ZOOM_DEFAULT
	# V7 H Phase A — resolve the WorldSim node once for pause + tracking.
	_world_sim = get_node_or_null("/root/Main/WorldSim")


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_apply_zoom_delta(ZOOM_FACTOR)
			get_viewport().set_input_as_handled()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_apply_zoom_delta(1.0 / ZOOM_FACTOR)
			get_viewport().set_input_as_handled()
	elif event is InputEventPanGesture:
		# V7 H Phase A — trackpad two-finger scroll → zoom (scroll up =
		# delta.y < 0 = zoom in). Mouse-wheel path above is unchanged.
		_apply_zoom_delta(1.0 - event.delta.y * PAN_ZOOM_SENSITIVITY)
		get_viewport().set_input_as_handled()
	elif event is InputEventMagnifyGesture:
		# V7 H Phase A — trackpad pinch → zoom (factor > 1 = zoom in).
		_apply_zoom_delta(event.factor)
		get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo \
			and event.keycode == KEY_P:
		# V7 H Phase A — KEY_P toggles pause. KEY_SPACE is intentionally NOT
		# used here (it belongs to the world_renderer overlay-channel cycle).
		_toggle_pause()
		get_viewport().set_input_as_handled()


func _apply_zoom_delta(factor: float) -> void:
	var new_zoom: Vector2 = _target_zoom * factor
	new_zoom.x = clamp(new_zoom.x, ZOOM_MIN.x, ZOOM_MAX.x)
	new_zoom.y = clamp(new_zoom.y, ZOOM_MIN.y, ZOOM_MAX.y)
	_target_zoom = new_zoom

	if _zoom_tween != null and _zoom_tween.is_valid():
		_zoom_tween.kill()
	_zoom_tween = create_tween()
	_zoom_tween.tween_property(self, "zoom", _target_zoom, TWEEN_DURATION) \
		.set_trans(Tween.TRANS_QUAD) \
		.set_ease(Tween.EASE_OUT)


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


func _process(delta: float) -> void:
	# V7 H Phase A — gently track the agent swarm centroid so Brownian-
	# dispersing agents (AgentMovementSystem) stay on-screen. Reads the same
	# snapshot the renderers consume; converts the mean tile to world pixels
	# on the SPRITE_ORIGIN + tile*TILE_SIZE basis, then eases toward it.
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_agent_snapshot")
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
	var target := Vector2(
		float(SPRITE_ORIGIN_X) + mean_tx * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
		float(SPRITE_ORIGIN_Y) + mean_ty * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
	)
	position = position.lerp(target, clampf(delta * CAMERA_TRACK_SPEED, 0.0, 1.0))
