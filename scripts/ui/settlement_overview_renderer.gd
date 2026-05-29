extends Node2D

# V7 Phase 14-ζ — settlement overview region renderer.
#
# Overview-zoom affordance: when the player zooms out (Tier.MEDIUM
# / FAR, toggled by zoom_lod_controller.gd), this node draws one
# translucent filled circle per Settlement at its
# substrate-derived centroid, radius scaling with member_count.
# At overview zoom — where individual 16×18 px agent sprites
# collapse below the perceptual threshold — these discs make the
# populated regions legible at a glance.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - Uses the EXISTING settlement snapshot FFI (Phase 12-γ); no
#     new backend surface. The snapshot exposes centroid_xs /
#     centroid_ys / member_counts. There is no true polygon
#     boundary in the substrate, so "boundary" is approximated as
#     a member-count-scaled disc centred on the centroid.
#   - Starts hidden; zoom_lod_controller flips `visible`. It never
#     mutates sim state (read-only snapshot consumer). Same
#     SPRITE_ORIGIN + TILE_SIZE basis as
#     world_renderer._update_settlement_furniture so the discs
#     centre on the Phase 12-γ hearth placeholders.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const Z_OVERVIEW := 1                       # above terrain (0), below trails (2)/agents
const OVERVIEW_ALPHA := 0.28
const OVERVIEW_COLOR: Color = Color(0.35, 0.75, 1.0, OVERVIEW_ALPHA)  # soft blue region
const BASE_RADIUS_PX := 24.0                # radius for a 1-member settlement
const PER_MEMBER_RADIUS_PX := 6.0           # additional radius per resolvable member
const MAX_RADIUS_PX := 160.0                # clamp so one settlement can't fill the map

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
	var xs: Variant = snap_dict.get("centroid_xs", null)
	var ys: Variant = snap_dict.get("centroid_ys", null)
	var members: Variant = snap_dict.get("member_counts", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array \
			and members is PackedInt32Array):
		return
	var xs_arr: PackedInt32Array = xs
	var ys_arr: PackedInt32Array = ys
	var members_arr: PackedInt32Array = members
	var n: int = xs_arr.size()
	if ys_arr.size() != n or members_arr.size() != n:
		return
	var new_centroids: PackedVector2Array = PackedVector2Array()
	var new_radii: PackedFloat32Array = PackedFloat32Array()
	for i in n:
		var px: float = float(SPRITE_ORIGIN_X + xs_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		new_centroids.append(Vector2(px, py))
		var mc: int = int(members_arr[i])
		var r: float = clampf(BASE_RADIUS_PX + PER_MEMBER_RADIUS_PX * float(mc), \
				BASE_RADIUS_PX, MAX_RADIUS_PX)
		new_radii.append(r)
	_centroids = new_centroids
	_radii = new_radii
	queue_redraw()


func _draw() -> void:
	for i in _centroids.size():
		draw_circle(_centroids[i], _radii[i], OVERVIEW_COLOR)
