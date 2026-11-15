# view_projection.gd — screen ↔ colony world-pixel conversion and hit-tests shared by the 2D and 3D views (M3D-1;
#   M3D-SWITCH E1).
# Purpose: the 3D view (scripts/world3d/world3d.gd) installs itself as `provider`; then a screen point becomes the
#   world pixel under it (a ray to the terrain or a roof), a world pixel the screen point of that ground spot, `pick`
#   names the person, animal, caravan or wave drawn under a screen point ({kind, id, world[, species]}), and
#   `person_rect` is a person's body on screen. OrderTool, the camera (zoom anchor, camp framing), the lesson highlight
#   and the panels that pin to the map go through here, so none of them needs a view's layers. With no provider the
#   2D escape hatch answers (`--renderer=2d`): the canvas transform and the hidden-in-3D ColonyView layers' own hit-tests
#   (the `_2d` helpers below; removed with the 2D view in M3D-SWITCH E2).
# Bridge methods: none.
extends RefCounted

## The active 3D view (an object with screen_to_world(Vector2) -> Vector2, world_to_screen(Vector2) -> Vector2,
## pick(screen, mode) / pick_world(world, mode) -> Dictionary and person_rect(bits) -> Rect2), or null.
static var provider: Object = null


static func active() -> bool:
	return provider != null and is_instance_valid(provider)


## The colony world pixel under screen point `screen` (viewport coordinates).
static func screen_to_world(item: CanvasItem, screen: Vector2) -> Vector2:
	if active():
		return provider.call("screen_to_world", screen)
	return item.get_canvas_transform().affine_inverse() * screen


## The viewport point where colony world pixel `world` is drawn.
static func world_to_screen(item: CanvasItem, world: Vector2) -> Vector2:
	if active():
		return provider.call("world_to_screen", world)
	return item.get_viewport().get_canvas_transform() * world


## What is drawn under screen point `screen` in order mode `mode` (OrderTool.Mode; people answer only with no mode,
## animals also in the hunt mode): {kind: "caravan"|"wave"|"person"|"animal", id, world (feet, world px), species
## (animals)}, or {}. `cycle` (a click) lets the 2D view step through people standing on one spot.
static func pick(item: CanvasItem, screen: Vector2, mode: int = 0, cycle: bool = false) -> Dictionary:
	if active():
		return provider.call("pick", screen, mode)
	return _pick_2d(item, screen_to_world(item, screen), mode, cycle)


## As `pick` for the thing standing at world pixel `world` (tools' clicks at world positions).
static func pick_world(item: CanvasItem, world: Vector2, mode: int = 0, cycle: bool = false) -> Dictionary:
	if active():
		return provider.call("pick_world", world, mode)
	return _pick_2d(item, world, mode, cycle)


## The viewport rect of the body of the person with entity bits `bits` (Rect2() while not drawn yet). `framing` asks
## for the 2D view's stable framing envelope instead of the click rect.
static func person_rect(item: CanvasItem, bits: int, framing: bool = false) -> Rect2:
	if active():
		return provider.call("person_rect", bits)
	var layer: Node2D = item.get_node_or_null("/root/Main/ColonyView/AgentLayer") as Node2D
	if layer == null or not layer.is_node_ready():
		return Rect2()
	if framing:
		return layer.call("framing_body_rect", bits)
	var raw: Rect2 = layer.call("drawn_rect", bits)
	return layer.get_global_transform_with_canvas() * raw if raw.has_area() else Rect2()


## Whether people are drawn as LOD markers (only the 2D view's quarter zoom).
static func people_lod(item: CanvasItem) -> bool:
	if active():
		return false
	var layer: Node = item.get_node_or_null("/root/Main/ColonyView/AgentLayer")
	return layer != null and bool(layer.call("lod_active"))


## The 2D escape hatch's hit-test at world pixel `p`: visitors, then people, then animals (as the 2D click order).
static func _pick_2d(item: CanvasItem, p: Vector2, mode: int, cycle: bool) -> Dictionary:
	var view: Node = item.get_node_or_null("/root/Main/ColonyView")
	if view == null:
		return {}
	var visitors: Node = view.get_node_or_null("VisitorLayer")
	if visitors != null and mode == 0:
		var caravan: int = int(visitors.call("caravan_at", p))
		if caravan >= 0:
			return {"kind": "caravan", "id": caravan, "world": p}
		var wave: int = int(visitors.call("wave_at", p))
		if wave >= 0:
			return {"kind": "wave", "id": wave, "world": p}
	var agents: Node = view.get_node_or_null("AgentLayer")
	if agents != null and mode == 0:
		var bits: int = int(agents.call("entity_at", p, cycle))
		if bits >= 0:
			return {"kind": "person", "id": bits, "world": p}
	var animals: Node = view.get_node_or_null("AnimalLayer")
	if animals != null:
		var id: int = int(animals.call("animal_at", p))
		if id >= 0:
			return {"kind": "animal", "id": id, "world": p, "species": int(animals.call("kind_of", id))}
	return {}
