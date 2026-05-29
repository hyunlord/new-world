extends Node

# V7 Phase 14-ζ — zoom level-of-detail (LOD) controller.
#
# Songs-of-Syx-style zoom adaptation: as the camera zooms out,
# decorative per-agent detail fades and an overview affordance
# fades in. This node owns the zoom→tier mapping and toggles the
# *whole-node* `visible` flag of two decorative sibling
# renderers. It modifies NO invariant file — the Phase 14-δ/ε
# Option-B precedent: orchestrate siblings, never refactor
# invariant-heavy renderers.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - Two of the four zoom behaviours sketched in the Phase 14-ζ
#     brief are implemented as clean whole-node toggles (trail
#     hide at FAR, settlement overview show at MEDIUM/FAR). The
#     other two — head-icon hide (agent_renderer.gd Phase 4-γ
#     MultiMesh + ICON_OFFSET_PX) and resource-sprite
#     simplification (world_renderer.gd Phase 14-β) — would
#     require modifying invariant-heavy renderers and are
#     deferred to a future phase that explicitly authorises that
#     surface.
#   - Tier thresholds key off the LIVE Camera2D.zoom (tweened by
#     camera_controller Phase 12-α), so the toggle tracks the
#     smooth zoom, not just the wheel-notch target.

enum Tier { FAR, MEDIUM, CLOSE }

# Tier thresholds over Camera2D.zoom.x (Phase 12-α range 0.5–4.0,
# Phase 13-α default 3.0). zoom >= CLOSE_MIN -> CLOSE; zoom <
# FAR_MAX -> FAR; in between -> MEDIUM. Chosen so the Phase 13-α
# default (3.0) lands in CLOSE (full detail) and the user must
# zoom out past 1.0 to reach the overview affordance.
const ZOOM_FAR_MAX: float = 1.0
const ZOOM_CLOSE_MIN: float = 2.0

var _camera: Camera2D = null
var _trail_renderer: Node2D = null
var _overview_renderer: Node2D = null
var _current_tier: int = Tier.CLOSE


func _ready() -> void:
	_camera = get_node_or_null("/root/Main/Camera2D") as Camera2D
	_trail_renderer = get_node_or_null("/root/Main/ActivityTrailRenderer") as Node2D
	_overview_renderer = get_node_or_null("/root/Main/SettlementOverviewRenderer") as Node2D
	# Apply the initial tier once so visibility matches the Phase
	# 13-α default zoom (3.0 → CLOSE) before the first change.
	_apply_tier(Tier.CLOSE)


func _process(_delta: float) -> void:
	if _camera == null:
		return
	var z: float = _camera.zoom.x
	var tier: int = _tier_for_zoom(z)
	if tier != _current_tier:
		_apply_tier(tier)


func _tier_for_zoom(z: float) -> int:
	if z < ZOOM_FAR_MAX:
		return Tier.FAR
	if z >= ZOOM_CLOSE_MIN:
		return Tier.CLOSE
	return Tier.MEDIUM


func _apply_tier(tier: int) -> void:
	_current_tier = tier
	# Trails: shown at CLOSE + MEDIUM, hidden at FAR (overview)
	# where the polyline mesh would read as map-wide noise.
	if _trail_renderer != null:
		_trail_renderer.visible = (tier != Tier.FAR)
	# Settlement overview region discs: shown at MEDIUM + FAR (as
	# the player pulls back), hidden at CLOSE where individual
	# sprites are legible.
	if _overview_renderer != null:
		_overview_renderer.visible = (tier != Tier.CLOSE)
