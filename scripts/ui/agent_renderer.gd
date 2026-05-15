extends Node2D

# V7 Phase 4-γ — AgentRenderer (MultiMeshInstance2D driver).
#
# Pulls a per-frame `(ids, xs, ys)` snapshot from `WorldSim` via the new
# `get_agent_snapshot()` FFI and drives a single `MultiMeshInstance2D` to
# draw every agent in one batched call.
#
# Per-instance palette variation is deterministic from `Entity::to_bits()`:
# hash → (hair_col, body_col, skin_col) → normalised Vector4 written to
# `MultiMesh.set_instance_custom_data` and consumed by the modified
# `palette_swap.gdshader` via `INSTANCE_CUSTOM.rgb`.
#
# Coordinate frame: mirrors `world_renderer.gd` so both layers share the
# same `(SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y)` + `TILE_SIZE` mapping. The
# sprite is centred on the tile.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const SPRITE_W := 64
const SPRITE_H := 72
# Sprite is scaled so the 64-px-wide character fits inside one 16-px
# world tile while keeping its 8:9 aspect ratio. 16/64 = 0.25.
const SPRITE_SCALE := 0.25
const PALETTE_HAIR_COLS := 8
const PALETTE_BODY_COLS := 4
const PALETTE_SKIN_COLS := 8

var world_sim: WorldSimNode
var multi_mesh_inst: MultiMeshInstance2D
var multi_mesh: MultiMesh

func _ready() -> void:
	print("AgentRenderer ready (V7 Phase 4-γ — MultiMeshInstance2D)")
	world_sim = get_node("../WorldSim") as WorldSimNode
	if world_sim == null:
		push_error("WorldSim node not found at ../WorldSim")
		return

	var quad := QuadMesh.new()
	quad.size = Vector2(SPRITE_W, SPRITE_H)

	multi_mesh = MultiMesh.new()
	multi_mesh.transform_format = MultiMesh.TRANSFORM_2D
	multi_mesh.use_colors = false
	multi_mesh.use_custom_data = true
	multi_mesh.mesh = quad
	multi_mesh.instance_count = 0

	var agent_tex: Texture2D = load("res://assets/sprites/agent_base.png") as Texture2D
	var palette_tex: Texture2D = load("res://assets/sprites/palette_lut.png") as Texture2D
	var shader: Shader = load("res://shaders/palette_swap.gdshader") as Shader

	var mat := ShaderMaterial.new()
	mat.shader = shader
	if palette_tex != null:
		mat.set_shader_parameter("palette_lut", palette_tex)

	multi_mesh_inst = MultiMeshInstance2D.new()
	multi_mesh_inst.multimesh = multi_mesh
	multi_mesh_inst.texture = agent_tex
	multi_mesh_inst.material = mat
	add_child(multi_mesh_inst)

func _process(_delta: float) -> void:
	if world_sim == null or multi_mesh == null:
		return
	var snap: Dictionary = world_sim.get_agent_snapshot()
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var n: int = ids.size()
	if xs.size() != n or ys.size() != n:
		push_error("AgentRenderer: parallel-array length mismatch")
		return
	if multi_mesh.instance_count != n:
		multi_mesh.instance_count = n
	for i in n:
		var tile_x: int = xs[i]
		var tile_y: int = ys[i]
		var px: float = float(SPRITE_ORIGIN_X + tile_x * TILE_SIZE + TILE_SIZE / 2)
		var py: float = float(SPRITE_ORIGIN_Y + tile_y * TILE_SIZE + TILE_SIZE / 2)
		var xform := Transform2D(0.0, Vector2(SPRITE_SCALE, SPRITE_SCALE), 0.0, Vector2(px, py))
		multi_mesh.set_instance_transform_2d(i, xform)
		multi_mesh.set_instance_custom_data(i, _palette_for_id(ids[i]))

func _palette_for_id(eid: int) -> Color:
	# Hash splat — three independent multipliers give visually distinct
	# columns across the 8/4/8 palette layout. Stable per agent within a
	# session because `eid` is `Entity::to_bits().get()`.
	var h: int = absi(eid * 2654435761) % PALETTE_HAIR_COLS
	var b: int = absi(eid * 40503) % PALETTE_BODY_COLS
	var s: int = absi(eid * 2246822519) % PALETTE_SKIN_COLS
	return Color(
		float(h) / float(PALETTE_HAIR_COLS - 1),
		float(b) / float(PALETTE_BODY_COLS - 1),
		float(s) / float(PALETTE_SKIN_COLS - 1),
		0.0
	)
