extends Node2D

# V7 Phase 14-ζ — settlement overview region renderer.
#                 (Region fix A+B, HEAD 475fbfe1)
#
# Overview-zoom affordance: when the player zooms out (Tier.MEDIUM
# / FAR, toggled by zoom_lod_controller.gd), this node draws one
# translucent filled circle per Settlement at its FIXED
# formation_tile, sized to the actual membership region.
# At overview zoom — where individual 16×18 px agent sprites
# collapse below the perceptual threshold — these discs make the
# populated regions legible at a glance.
#
# Region fix (2026-06-02):
#   - CENTER = the FIXED formation_tile (snapshot `formation_xs` /
#     `formation_ys`, added with the marker fix, commit 475fbfe1),
#     NOT the live member centroid. The centroid is the moving mean
#     of member positions, so the old disc shook every frame; the
#     formation tile is constant, so the disc now sits still and is
#     co-located with the settlement marker.
#   - RADIUS = the ACTUAL region: SETTLEMENT_REGION_RADIUS_TILES (5,
#     mirrors sim-core SETTLEMENT_PROXIMITY_RADIUS) × TILE_SIZE (16)
#     = 80 px, fixed for every settlement. The old member-count
#     scaling inflated big settlements to ~160 px (≈2× the real
#     region) and caused excessive overlap.
#
# Honest disclosure (verified Step 0 grep):
#   - Uses the EXISTING settlement snapshot FFI (Phase 12-γ); no
#     new backend surface. There is no true polygon boundary in the
#     substrate, so the Chebyshev-5 square region is approximated as
#     a fixed-radius disc centred on the formation tile.
#   - Starts hidden; zoom_lod_controller flips `visible`. It never
#     mutates sim state (read-only snapshot consumer). Same
#     SPRITE_ORIGIN + TILE_SIZE basis as
#     world_renderer._update_settlement_furniture so the discs
#     centre on the Phase 12-γ hearth placeholders / marker.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const Z_OVERVIEW := 1                       # above terrain (0), below trails (2)/agents
const OVERVIEW_ALPHA := 0.28
const OVERVIEW_COLOR: Color = Color(0.35, 0.75, 1.0, OVERVIEW_ALPHA)  # soft blue region
const SETTLEMENT_REGION_RADIUS_TILES := 5   # mirrors sim-core SETTLEMENT_PROXIMITY_RADIUS

var _world_sim: Node = null
# Cached per-frame parallel arrays consumed by _draw.
var _centroids: PackedVector2Array = PackedVector2Array()
var _radii: PackedFloat32Array = PackedFloat32Array()


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	z_index = Z_OVERVIEW
	visible = false   # zoom_lod_controller shows this only at MEDIUM/FAR


func _process(_delta: float) -> void:
	# Skip all snapshot work while hidden (CLOSE zoom) — overview off.
	if not visible:
		return
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_settlement_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("formation_xs", null)
	var ys: Variant = snap_dict.get("formation_ys", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array):
		return
	var xs_arr: PackedInt32Array = xs
	var ys_arr: PackedInt32Array = ys
	var n: int = xs_arr.size()
	if ys_arr.size() != n:
		return
	# Radius is the fixed actual region, identical for every settlement.
	var r: float = float(SETTLEMENT_REGION_RADIUS_TILES * TILE_SIZE)
	var new_centroids: PackedVector2Array = PackedVector2Array()
	var new_radii: PackedFloat32Array = PackedFloat32Array()
	for i in n:
		var px: float = float(SPRITE_ORIGIN_X + xs_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		new_centroids.append(Vector2(px, py))
		new_radii.append(r)
	_centroids = new_centroids
	_radii = new_radii
	queue_redraw()


func _draw() -> void:
	for i in _centroids.size():
		draw_circle(_centroids[i], _radii[i], OVERVIEW_COLOR)
