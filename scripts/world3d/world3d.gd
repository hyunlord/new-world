# world3d.gd (root of scenes/world3d/World3D.tscn, added under Main by main_bootstrap.gd when renderer_switch.gd
#   wants 3D): the 3D view of the colony (M3D-1). The 2D ColonyView stays in the tree, hidden, so its hit-tests and
#   OrderTool keep working; the HUD and panels (UI CanvasLayer) draw over this view unchanged.
# Purpose: builds the 3D layers in code (Terrain, Flora, Structures, Props, People, Animals, Overlay, Sky — each its
#   own script under scripts/world3d/) and the CameraRig that follows the 2D camera, and installs itself as the
#   ViewProjection provider: `screen_to_world` turns a screen point into the colony world pixel under it (a ray
#   marched against the terrain heights; a person or animal under the cursor yields the centre of that thing's 2D
#   drawn rect, so AgentLayer.entity_at / AnimalLayer.animal_at pick exactly what the 3D view shows) and
#   `world_to_screen` projects a world pixel's ground point to the screen.
# Bridge methods called: none directly (the layers read SimReads; see their headers).
extends Node3D

const ViewProjection = preload("res://scripts/world/view_projection.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const TerrainScript = preload("res://scripts/world3d/terrain3d.gd")
const FloraScript = preload("res://scripts/world3d/flora3d.gd")
const RigScript = preload("res://scripts/world3d/camera_rig.gd")
const SkyScript = preload("res://scripts/world3d/sky3d.gd")
const StructuresScript = preload("res://scripts/world3d/structures3d.gd")
const PropsScript = preload("res://scripts/world3d/props3d.gd")
const PlantsScript = preload("res://scripts/world3d/plants3d.gd")
const PeopleScript = preload("res://scripts/world3d/people3d.gd")
const AnimalsScript = preload("res://scripts/world3d/animals3d.gd")
const OverlayScript = preload("res://scripts/world3d/overlay3d.gd")

var world_sim: Node = null
var camera2d: Camera2D = null
var terrain: Node3D = null
var flora: Node3D = null
var rig: Node3D = null
var sky: Node3D = null
var structures: Node3D = null
var props: Node3D = null
var people: Node3D = null
var animals: Node3D = null
var overlay: Node3D = null
## Pickers: nodes with `pick(screen: Vector2, camera: Camera3D, order_mode: int) -> Vector2` (world px, or (-1, -1)).
var pickers: Array[Node] = []
static var _globals_added: bool = false
## Once per frame (World3D processes before its layers): the colony change counters and the tick. Layers read heavy
## snapshots only when the part of this stamp they depend on moved — a per-frame FFI read of get_trees alone cost
## ~6.5 ms (no 2D layer shares the SimReads frame cache in 3D).
var stamp: Dictionary = {}
var _catalog: Dictionary = {}


func _ready() -> void:
	# Global shader parameters the flora x-ray reads (camera_rig.gd writes them every frame).
	if not _globals_added:
		_globals_added = true
		RenderingServer.global_shader_parameter_add("w3d_camera", RenderingServer.GLOBAL_VAR_TYPE_VEC3, Vector3.ZERO)
		RenderingServer.global_shader_parameter_add("w3d_focus", RenderingServer.GLOBAL_VAR_TYPE_VEC3, Vector3.ZERO)
		RenderingServer.global_shader_parameter_add("w3d_xray", RenderingServer.GLOBAL_VAR_TYPE_FLOAT, 0.0)
		RenderingServer.global_shader_parameter_add("w3d_overlay", RenderingServer.GLOBAL_VAR_TYPE_SAMPLER2D, null)
		RenderingServer.global_shader_parameter_add("w3d_frame", RenderingServer.GLOBAL_VAR_TYPE_SAMPLER2D, null)
		RenderingServer.global_shader_parameter_add("w3d_map_size", RenderingServer.GLOBAL_VAR_TYPE_VEC2, Vector2(192, 192))
		RenderingServer.global_shader_parameter_add("w3d_snow", RenderingServer.GLOBAL_VAR_TYPE_FLOAT, 0.0)
	var main: Node = get_parent()
	world_sim = main.get_node_or_null("WorldSim")
	camera2d = main.get_node_or_null("Camera2D") as Camera2D
	terrain = _layer("Terrain", TerrainScript)
	terrain.call("setup", world_sim)
	flora = _layer("Flora", FloraScript)
	flora.call("setup", world_sim, terrain)
	sky = _layer("Sky", SkyScript)
	sky.call("setup", world_sim, terrain)
	structures = _layer("Structures", StructuresScript)
	structures.call("setup", world_sim, terrain)
	props = _layer("Props", PropsScript)
	props.call("setup", world_sim, terrain, sky)
	_layer("Plants", PlantsScript).call("setup", world_sim, terrain)
	people = _layer("People", PeopleScript)
	people.call("setup", world_sim, terrain)
	animals = _layer("Animals", AnimalsScript)
	animals.call("setup", world_sim, terrain)
	overlay = _layer("Overlay", OverlayScript)
	overlay.call("setup", world_sim, terrain)
	add_picker(people)
	add_picker(animals)
	rig = _layer("CameraRig", RigScript)
	rig.call("setup", camera2d, terrain)
	ViewProjection.provider = self
	_apply_render_scale()


## 3D resolution scale (params3d `render.scale_3d`, FSR upscaling): the HUD and panels stay at native resolution.
## On a HiDPI window the 3D pass was fragment-bound at 2× the logical pixels.
func _apply_render_scale() -> void:
	var vp: Viewport = get_viewport()
	var scale: float = clampf(float(Data3D.param("render.scale_3d", 1.0)), 0.25, 1.0)
	var arg_scale: String = ""
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("--scale3d="):
			arg_scale = arg.trim_prefix("--scale3d=")
	if arg_scale != "":
		scale = clampf(float(arg_scale), 0.25, 1.0)
	vp.scaling_3d_mode = Viewport.SCALING_3D_MODE_FSR if scale < 1.0 else Viewport.SCALING_3D_MODE_BILINEAR
	vp.scaling_3d_scale = scale


func _process(_delta: float) -> void:
	if world_sim == null:
		return
	var v: Dictionary = SimReads.read(world_sim, "get_colony_versions")
	stamp = {"tick": int(SimReads.read(world_sim, "get_time_snapshot").get("tick", 0)), "trees": int(v.get("trees_version", 0)),
		"overlay": int(v.get("overlay_version", 0)), "rooms": int(v.get("rooms_version", 0)), "frame": Engine.get_process_frames()}


## The content catalogue, read once (it is immutable for a world).
func catalog() -> Dictionary:
	if _catalog.is_empty() and world_sim != null:
		_catalog = SimReads.read(world_sim, "get_content_catalog")
	return _catalog


func _exit_tree() -> void:
	if ViewProjection.provider == self:
		ViewProjection.provider = null


func _layer(layer_name: String, script: GDScript) -> Node3D:
	var node: Node3D = Node3D.new()
	node.set_script(script)
	node.name = layer_name
	add_child(node)
	return node


## Register a layer that can claim clicks before the ground (people, animals).
func add_picker(node: Node) -> void:
	if not pickers.has(node):
		pickers.append(node)


func camera() -> Camera3D:
	return rig.get("camera") as Camera3D if rig != null else null


## ViewProjection provider: the colony world pixel under screen point `screen`.
func screen_to_world(screen: Vector2) -> Vector2:
	var cam: Camera3D = camera()
	if cam == null:
		return Vector2(-1, -1)
	var tool: Node = get_parent().get_node_or_null("ColonyView/OrderTool")
	var mode: int = int(tool.get("mode")) if tool != null else 0
	for picker: Node in pickers:
		if is_instance_valid(picker):
			var hit: Vector2 = picker.call("pick", screen, cam, mode)
			if hit.x >= 0.0:
				return hit
	var p: Vector3 = ground_hit(screen)
	var origin: Vector3 = cam.project_ray_origin(screen)
	var dir: Vector3 = cam.project_ray_normal(screen)
	var t_roof: float = float(structures.call("roof_hit", origin, dir)) if structures != null else INF
	if t_roof < origin.distance_to(p):
		p = origin + dir * t_roof
	return Vector2(p.x, p.z) * float(Data3D.param("camera.px_per_tile_at_1x", 64.0))


## ViewProjection provider: the screen point of world pixel `world` on the ground.
func world_to_screen(world: Vector2) -> Vector2:
	var cam: Camera3D = camera()
	if cam == null:
		return Vector2(-10000, -10000)
	var px: float = float(Data3D.param("camera.px_per_tile_at_1x", 64.0))
	var x: float = world.x / px
	var z: float = world.y / px
	var p: Vector3 = Vector3(x, float(terrain.call("height_at", x, z)), z)
	if cam.is_position_behind(p):
		return Vector2(-10000, -10000)
	return cam.unproject_position(p)


## The terrain point under screen point `screen` (ray marched against the height field, then refined).
func ground_hit(screen: Vector2) -> Vector3:
	var cam: Camera3D = camera()
	var origin: Vector3 = cam.project_ray_origin(screen)
	var dir: Vector3 = cam.project_ray_normal(screen)
	var step: float = 0.25
	var t: float = 0.0
	var prev: float = 0.0
	var limit: float = cam.far
	while t < limit:
		var p: Vector3 = origin + dir * t
		if p.y <= float(terrain.call("height_at", p.x, p.z)):
			var lo: float = prev
			var hi: float = t
			for _i: int in 12:
				var mid: float = (lo + hi) * 0.5
				var q: Vector3 = origin + dir * mid
				if q.y <= float(terrain.call("height_at", q.x, q.z)):
					hi = mid
				else:
					lo = mid
			return origin + dir * hi
		prev = t
		t += step
		step = minf(2.0, step * 1.04)
	# No hit (looking past the map): intersect y = 0.
	if absf(dir.y) > 1e-5:
		return origin + dir * (-origin.y / dir.y)
	return origin
