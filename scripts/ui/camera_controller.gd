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
const ZOOM_MAX: Vector2 = Vector2(4.0, 4.0)
# V7 Phase 13-α — default zoom raised from 2.0× to 3.0× so 16×18 px agent
# sprites (Phase 4-γ SPRITE_SCALE = 0.25 invariant preserved) render at
# 48–54 px, above the human-perceptual threshold for distinguishing agents
# and buildings. Mouse-wheel controls (Phase 12-α) still let the user zoom
# out to overview.
const ZOOM_DEFAULT: Vector2 = Vector2(3.0, 3.0)
const ZOOM_FACTOR: float = 1.25
const TWEEN_DURATION: float = 0.15

var _zoom_tween: Tween = null
var _target_zoom: Vector2 = ZOOM_DEFAULT


func _ready() -> void:
	# V7 Phase 13-α — print derives the default from ZOOM_DEFAULT so the
	# log never drifts from the constant (formerly hardcoded "2.0×").
	print("CameraController ready (default zoom %.1f×)" % ZOOM_DEFAULT.x)
	zoom = ZOOM_DEFAULT
	_target_zoom = ZOOM_DEFAULT


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_apply_zoom_delta(ZOOM_FACTOR)
			get_viewport().set_input_as_handled()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_apply_zoom_delta(1.0 / ZOOM_FACTOR)
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
