class_name WorldRenderer
extends Sprite2D

const WorldDataClass = preload("res://scripts/core/world_data.gd")


## Render the entire world as a single image texture
func render_world(world_data: RefCounted, resource_map: RefCounted = null) -> void:
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
			# Resource tinting
			if resource_map != null:
				var food: float = resource_map.get_food(x, y)
				var wood: float = resource_map.get_wood(x, y)
				var stone: float = resource_map.get_stone(x, y)
				var food_max: float = resource_map.get_max_for_biome(biome, GameConfig.ResourceType.FOOD)
				var wood_max: float = resource_map.get_max_for_biome(biome, GameConfig.ResourceType.WOOD)
				var stone_max: float = resource_map.get_max_for_biome(biome, GameConfig.ResourceType.STONE)
				if food_max > 0.0:
					var food_ratio: float = food / food_max
					final_color = final_color.lerp(Color(0.4, 0.8, 0.2), food_ratio * 0.15)
				if wood_max > 0.0:
					var wood_ratio: float = wood / wood_max
					final_color = final_color.lerp(Color(0.1, 0.35, 0.05), wood_ratio * 0.15)
				if stone_max > 0.0:
					var stone_ratio: float = stone / stone_max
					final_color = final_color.lerp(Color(0.7, 0.7, 0.72), stone_ratio * 0.15)
			img.set_pixel(x, y, final_color)

	var tex := ImageTexture.create_from_image(img)
	texture = tex
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	scale = Vector2(GameConfig.TILE_SIZE, GameConfig.TILE_SIZE)
	# Position so that top-left of world is at (0, 0)
	centered = false
