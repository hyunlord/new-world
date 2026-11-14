# view_projection.gd — screen ↔ colony world-pixel conversion shared by the 2D and 3D views (M3D-1).
# Purpose: with no provider the conversion is the canvas transform (the 2D view, unchanged). The 3D view
#   (scripts/world3d/world3d.gd) installs itself as `provider`; then a screen point becomes the world pixel under it
#   (a ray to the terrain, or the drawn rect of the person or animal under it) and a world pixel the screen point of
#   that ground spot, so OrderTool, the camera's zoom anchor and panels that pin to the map work in both views.
# Bridge methods called: none.
extends RefCounted

## The active 3D view (an object with screen_to_world(Vector2) -> Vector2 and world_to_screen(Vector2) -> Vector2), or null.
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
