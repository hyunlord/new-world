# world3d.gd (root of scenes/world3d/World3D.tscn, added under Main by main_bootstrap.gd): the colony's only world
#   view (M3D-1; M3D-SWITCH E2 deleted the 2D one). The HUD and panels (UI CanvasLayer) draw over this view unchanged.
# Purpose: builds the 3D layers in code (Terrain, Flora, Structures, Props, People, Animals, Visitors, Overlay, Sky —
#   each its own script under scripts/world3d/), the screen-space map labels (Labels, labels3d.gd, in a CanvasLayer
#   under the HUD) and the CameraRig that follows the 2D camera, and installs itself as
#   the ViewProjection provider: `screen_to_world` turns a screen point into the colony world pixel under it (a ray
#   marched against the terrain heights, stopped by a roof), `world_to_screen` projects a world pixel's ground point to
#   the screen, `pick` / `pick_world` name the visitor group, person or animal whose body is under a screen point / at a
#   world point (pickers in that priority, the nearest body within each; the ids OrderTool's clicks, hunt and draft
#   orders use), and `person_rect` is a person's body on screen (camp framing, lesson highlight).
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
const VisitorsScript = preload("res://scripts/world3d/visitors3d.gd")
const LabelsScript = preload("res://scripts/world3d/labels3d.gd")

var _last_tick: int = -1
var _tick_frame: int = -1
var _heavy_frame: int = -1
## Seconds of scaled frame time (global shader param `w3d_time`); stops while Engine.time_scale is 0.
var anim_time: float = 0.0
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
var visitors: Node3D = null
## Pickers, in click priority: nodes with `bodies(order_mode: int) -> Array` of {kind, id, feet: Vector3, h: float,
## r: float (body half-width on screen per unit of screen height)[, species]}.
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
		# Animation clock of the 3D shaders (VAT crowds, sway, water): scaled frame time, so Engine.time_scale = 0
		# freezes every animated pixel (pixel oracles compare frames).
		RenderingServer.global_shader_parameter_add("w3d_time", RenderingServer.GLOBAL_VAR_TYPE_FLOAT, 0.0)
		RenderingServer.global_shader_parameter_add("w3d_night", RenderingServer.GLOBAL_VAR_TYPE_FLOAT, 0.0)
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
	visitors = _layer("Visitors", VisitorsScript)
	visitors.call("setup", world_sim, terrain)
	overlay = _layer("Overlay", OverlayScript)
	overlay.call("setup", world_sim, terrain)
	var label_layer: CanvasLayer = CanvasLayer.new()
	label_layer.name = "LabelLayer"
	label_layer.layer = 0  # under the HUD (UI is layer 1)
	add_child(label_layer)
	var labels: Control = LabelsScript.new()
	labels.name = "Labels"
	label_layer.add_child(labels)
	add_picker(visitors)
	add_picker(people)
	add_picker(animals)
	rig = _layer("CameraRig", RigScript)
	rig.call("setup", camera2d, terrain)
	if OS.get_cmdline_user_args().has("--style=s2"):
		for layer: Node3D in [terrain, structures, flora, props]:
			if not bool(layer.call("set_slice_pbr", true)):
				push_error("Style 2 material setup failed on " + str(layer.name))
		flora.call("configure_slice_vegetation", Rect2i())
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


func _process(delta: float) -> void:
	anim_time += delta
	RenderingServer.global_shader_parameter_set("w3d_time", anim_time)
	if world_sim == null:
		return
	var v: Dictionary = SimReads.read(world_sim, "get_colony_versions")
	stamp = {"tick": int(SimReads.read(world_sim, "get_time_snapshot").get("tick", 0)), "trees": int(v.get("trees_version", 0)),
		"overlay": int(v.get("overlay_version", 0)), "rooms": int(v.get("rooms_version", 0)), "frame": Engine.get_process_frames()}
	if int(stamp["tick"]) != _last_tick:
		_last_tick = int(stamp["tick"])
		_tick_frame = Engine.get_process_frames()


## Frame scheduler for the layers' heavy rebuilds (tree diff, terrain recompute and meshing, walls/roofs, plants): at
## most one per frame and none on a frame where the simulation stepped, so their costs never stack into a stall. A
## layer that is refused keeps its work for a later frame (a few frames at most at 12 ticks/s and 60 fps).
func claim_heavy() -> bool:
	var f: int = Engine.get_process_frames()
	if f == _tick_frame or f == _heavy_frame:
		return false
	_heavy_frame = f
	return true


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


## Register a layer that can claim clicks before the ground (visitors, people, animals).
func add_picker(node: Node) -> void:
	if not pickers.has(node):
		pickers.append(node)


func camera() -> Camera3D:
	return rig.get("camera") as Camera3D if rig != null else null


## ViewProjection provider: the colony world pixel under screen point `screen` (the ground, or a roof in front of it).
func screen_to_world(screen: Vector2) -> Vector2:
	var cam: Camera3D = camera()
	if cam == null:
		return Vector2(-1, -1)
	var p: Vector3 = ground_hit(screen)
	var origin: Vector3 = cam.project_ray_origin(screen)
	var dir: Vector3 = cam.project_ray_normal(screen)
	var t_roof: float = float(structures.call("roof_hit", origin, dir)) if structures != null else INF
	if t_roof < origin.distance_to(p):
		p = origin + dir * t_roof
	return Vector2(p.x, p.z) * _px()


func untemplated_room_at(tile: Vector2i) -> int:
	return int(structures.call("untemplated_room_at", tile)) if structures != null else -1


## ViewProjection provider: the body under screen point `screen` in order mode `mode`: the first picker with a body
## whose feet-to-head segment passes within `people.pick_radius_px` (or its own half-width) of the point, the nearest
## such body to the camera; {kind, id, world[, species]} or {}. A body counts only when both ends are on screen.
func pick(screen: Vector2, mode: int) -> Dictionary:
	var cam: Camera3D = camera()
	if cam == null:
		return {}
	var radius: float = float(Data3D.param("people.pick_radius_px", 18))
	var view: Rect2 = get_viewport().get_visible_rect().grow(8.0)
	for picker: Node in pickers:
		if not is_instance_valid(picker):
			continue
		var best: Dictionary = {}
		var best_depth: float = INF
		for body: Dictionary in picker.call("bodies", mode):
			var feet: Vector3 = body["feet"]
			var head: Vector3 = feet + Vector3(0, float(body["h"]), 0)
			if cam.is_position_behind(feet) or cam.is_position_behind(head):
				continue
			var a: Vector2 = cam.unproject_position(feet)
			var b: Vector2 = cam.unproject_position(head)
			if not (view.has_point(a) and view.has_point(b) and a.distance_to(b) < view.size.y * 0.5):
				continue
			if Geometry2D.get_closest_point_to_segment(screen, a, b).distance_to(screen) <= maxf(radius, (a - b).length() * float(body["r"])):
				var depth: float = cam.global_position.distance_to(feet)
				if depth < best_depth:
					best_depth = depth
					best = body
		if not best.is_empty():
			return _hit(best)
	return {}


## ViewProjection provider: the body standing at world pixel `world` (feet within half a tile; the nearest in the
## first picker that has one), as `pick`.
func pick_world(world: Vector2, mode: int) -> Dictionary:
	var at: Vector2 = world / _px()
	for picker: Node in pickers:
		if not is_instance_valid(picker):
			continue
		var best: Dictionary = {}
		var best_d: float = 0.5
		for body: Dictionary in picker.call("bodies", mode):
			var feet: Vector3 = body["feet"]
			var d: float = Vector2(feet.x, feet.z).distance_to(at)
			if d <= best_d:
				best_d = d
				best = body
		if not best.is_empty():
			return _hit(best)
	return {}


## ViewProjection provider: the viewport rect of person `bits`' body (feet to head, a quarter of its screen height to
## each side); far off screen when behind the camera; Rect2() while the person is not drawn.
func person_rect(bits: int) -> Rect2:
	var cam: Camera3D = camera()
	var body: Dictionary = people.call("body_of", bits) if people != null else {}
	if cam == null or body.is_empty():
		return Rect2()
	var feet: Vector3 = body["feet"]
	var head: Vector3 = feet + Vector3(0, float(body["h"]), 0)
	if cam.is_position_behind(feet) or cam.is_position_behind(head):
		return Rect2(Vector2(-100000, -100000), Vector2.ONE)
	var a: Vector2 = cam.unproject_position(feet)
	var b: Vector2 = cam.unproject_position(head)
	var half: float = maxf(1.0, a.distance_to(b) * 0.25)
	return Rect2(a, Vector2.ZERO).expand(b).grow_individual(half, 0.0, half, 0.0)


func _hit(body: Dictionary) -> Dictionary:
	var feet: Vector3 = body["feet"]
	var hit: Dictionary = {"kind": body["kind"], "id": body["id"], "world": Vector2(feet.x, feet.z) * _px()}
	if body.has("species"):
		hit["species"] = body["species"]
	return hit


func _px() -> float:
	return float(Data3D.param("camera.px_per_tile_at_1x", 64.0))


## ViewProjection provider: the screen point of world pixel `world` on the ground.
func world_to_screen(world: Vector2) -> Vector2:
	var cam: Camera3D = camera()
	if cam == null:
		return Vector2(-10000, -10000)
	var x: float = world.x / _px()
	var z: float = world.y / _px()
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
