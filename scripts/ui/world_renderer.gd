class_name WorldRenderer
extends Sprite2D

const WorldDataClass = preload("res://scripts/core/world_data.gd")

## Resource overlay sprite (child node, rendered on top)
var _resource_overlay: Sprite2D
var _world_data_ref: RefCounted
var _resource_map_ref: RefCounted


## Render the entire world as a single image texture
func render_world(world_data: RefCounted, resource_map: RefCounted = null) -> void:
	_world_data_ref = world_data
	_resource_map_ref = resource_map
	var w: int = world_data.width
	var h: int = world_data.height
	var img := Image.create(w, h, false, Image.FORMAT_RGB8)

	for y in range(h):
		for x in range(w):
			var biome: int = world_data.get_biome(x, y)
			var elev: float = world_data.get_elevation(x, y)
			var base_color: Color = GameConfig.BIOME_COLORS.get(biome, Color.MAGENTA)
			# Brightness variation based on elevation
			var brightness: float = lerpf(0.7, 1.3, elev)
			var final_color := Color(
				clampf(base_color.r * brightness, 0.0, 1.0),
				clampf(base_color.g * brightness, 0.0, 1.0),
				clampf(base_color.b * brightness, 0.0, 1.0),
			)
			img.set_pixel(x, y, final_color)

	var tex := ImageTexture.create_from_image(img)
	texture = tex
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	scale = Vector2(GameConfig.TILE_SIZE, GameConfig.TILE_SIZE)
	# Position so that top-left of world is at (0, 0)
	centered = false

	# Create resource overlay as child sprite
	if _resource_overlay == null:
		_resource_overlay = Sprite2D.new()
		_resource_overlay.centered = false
		_resource_overlay.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		# Overlay sits at same position, same scale (inherited from parent transform)
		# But since parent has scale=TILE_SIZE, child inherits it, so we use scale=1
		_resource_overlay.scale = Vector2.ONE
		add_child(_resource_overlay)

	# Initial resource overlay render
	if resource_map != null:
		update_resource_overlay()


## Update resource overlay texture (call periodically, e.g., every 100 ticks)
func update_resource_overlay() -> void:
	if _resource_map_ref == null or _world_data_ref == null:
		return
	var w: int = _world_data_ref.width
	var h: int = _world_data_ref.height
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)

	for y in range(h):
		for x in range(w):
			var food: float = _resource_map_ref.get_food(x, y)
			var wood: float = _resource_map_ref.get_wood(x, y)
			var stone: float = _resource_map_ref.get_stone(x, y)

			var color := Color(0.0, 0.0, 0.0, 0.0)
			# Priority: food > stone > wood (food is most important to see)
			if food > 1.0:
				var intensity: float = clampf(food / 10.0, 0.15, 0.5)
				color = Color(0.9, 0.8, 0.2, intensity)
			elif stone > 1.0:
				var intensity: float = clampf(stone / 8.0, 0.15, 0.45)
				color = Color(0.75, 0.75, 0.78, intensity)
			elif wood > 2.0:
				var intensity: float = clampf(wood / 12.0, 0.08, 0.3)
				color = Color(0.15, 0.5, 0.1, intensity)

			img.set_pixel(x, y, color)

	var tex: ImageTexture
	if _resource_overlay.texture == null:
		tex = ImageTexture.create_from_image(img)
		_resource_overlay.texture = tex
	else:
		tex = _resource_overlay.texture as ImageTexture
		tex.update(img)
