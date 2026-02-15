class_name WorldRenderer
extends Sprite2D

const WorldDataClass = preload("res://scripts/core/world_data.gd")


## Render the entire world as a single image texture
func render_world(world_data: RefCounted) -> void:
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
