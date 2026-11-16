# view_projection.gd — screen ↔ colony world-pixel conversion and hit-tests of the 3D view (M3D-1; M3D-SWITCH E1, E2).
# Purpose: the 3D view (scripts/world3d/world3d.gd) installs itself as `provider`; then a screen point becomes the
#   world pixel under it (a ray to the terrain or a roof), a world pixel the screen point of that ground spot, `pick`
#   names the person, animal, caravan or wave drawn under a screen point ({kind, id, world[, species]}), and
#   `person_rect` is a person's body on screen. OrderTool, the camera (zoom anchor, camp framing), the lesson highlight,
#   the map labels and the panels that pin to the map go through here, so none of them needs the view's layers. Before
#   the view is ready (or after it left the tree) every answer is the neutral "nothing there".
# Bridge methods: none.
extends RefCounted

## The active 3D view (an object with screen_to_world(Vector2) -> Vector2, world_to_screen(Vector2) -> Vector2,
## pick(screen, mode) / pick_world(world, mode) -> Dictionary and person_rect(bits) -> Rect2), or null.
static var provider: Object = null


static func active() -> bool:
	return provider != null and is_instance_valid(provider)


## The colony world pixel under screen point `screen` (viewport coordinates); (-1, -1) without a view.
static func screen_to_world(screen: Vector2) -> Vector2:
	return provider.call("screen_to_world", screen) if active() else Vector2(-1, -1)


## The viewport point where colony world pixel `world` is drawn; far off screen without a view.
static func world_to_screen(world: Vector2) -> Vector2:
	return provider.call("world_to_screen", world) if active() else Vector2(-10000, -10000)


## What is drawn under screen point `screen` in order mode `mode` (OrderTool.Mode; people answer only with no mode,
## animals also in the hunt mode): {kind: "caravan"|"wave"|"person"|"animal", id, world (feet, world px), species
## (animals)}, or {}.
static func pick(screen: Vector2, mode: int = 0) -> Dictionary:
	return provider.call("pick", screen, mode) if active() else {}


## As `pick` for the thing standing at world pixel `world` (tools' clicks at world positions).
static func pick_world(world: Vector2, mode: int = 0) -> Dictionary:
	return provider.call("pick_world", world, mode) if active() else {}


## The viewport rect of the body of the person with entity bits `bits` (Rect2() while not drawn yet).
static func person_rect(bits: int) -> Rect2:
	return provider.call("person_rect", bits) if active() else Rect2()
