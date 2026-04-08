extends Sprite2D

const WorldDataClass = preload("res://scripts/core/world/world_data.gd")

## Resource overlay sprite (child node, rendered on top)
var _resource_overlay: Sprite2D
var _world_data_ref: RefCounted
var _resource_map_ref: RefCounted
var _img: Image


## Subtle resource node tint thresholds and colors blended into the base
## terrain image. Mirrored from the GDScript icon-display thresholds and
## guarded by Rust-side harness assertions:
##   harness_resource_food_tiles_above_threshold (food > 4.0)
##   harness_resource_wood_tiles_above_threshold (wood > 5.0)
##   harness_resource_stone_tiles_above_threshold (stone > 3.0)
const RESOURCE_TINT_FOOD_THRESHOLD: float = 4.0
const RESOURCE_TINT_WOOD_THRESHOLD: float = 5.0
const RESOURCE_TINT_STONE_THRESHOLD: float = 3.0
const RESOURCE_TINT_FOOD_COLOR: Color = Color(0.30, 0.70, 0.15)
const RESOURCE_TINT_WOOD_COLOR: Color = Color(0.45, 0.30, 0.12)
const RESOURCE_TINT_STONE_COLOR: Color = Color(0.60, 0.60, 0.55)
const RESOURCE_TINT_FOOD_BLEND: float = 0.20
const RESOURCE_TINT_WOOD_BLEND: float = 0.15
const RESOURCE_TINT_STONE_BLEND: float = 0.18


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
			# Subtle resource node tint blended into base terrain.
			# Zero per-frame cost (only computed during initial render_world).
			# Priority: food (most important to spot), then stone, then wood.
			if resource_map != null:
				var food: float = resource_map.get_food(x, y)
				var wood: float = resource_map.get_wood(x, y)
				var stone: float = resource_map.get_stone(x, y)
				if food > RESOURCE_TINT_FOOD_THRESHOLD:
					final_color = final_color.lerp(RESOURCE_TINT_FOOD_COLOR, RESOURCE_TINT_FOOD_BLEND)
				if stone > RESOURCE_TINT_STONE_THRESHOLD:
					final_color = final_color.lerp(RESOURCE_TINT_STONE_COLOR, RESOURCE_TINT_STONE_BLEND)
				if wood > RESOURCE_TINT_WOOD_THRESHOLD:
					final_color = final_color.lerp(RESOURCE_TINT_WOOD_COLOR, RESOURCE_TINT_WOOD_BLEND)
			img.set_pixel(x, y, final_color)

	_img = img
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


## Toggle resource overlay visibility
func toggle_resource_overlay() -> void:
	if _resource_overlay != null:
		_resource_overlay.visible = not _resource_overlay.visible


## Check if resource overlay is currently visible
func is_resource_overlay_visible() -> bool:
	return _resource_overlay != null and _resource_overlay.visible


## Forces the resource overlay visibility without toggling current state.
func set_resource_overlay_visible(overlay_visible: bool) -> void:
	if _resource_overlay != null:
		_resource_overlay.visible = overlay_visible


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
			if food > 2.0:
				var intensity: float = clampf(food / 8.0, 0.45, 0.7)
				color = Color(1.0, 0.85, 0.0, intensity)
			elif stone > 2.0:
				var intensity: float = clampf(stone / 8.0, 0.4, 0.65)
				color = Color(0.4, 0.6, 1.0, intensity)
			elif wood > 3.0:
				var intensity: float = clampf(wood / 10.0, 0.35, 0.6)
				color = Color(0.0, 0.8, 0.2, intensity)

			img.set_pixel(x, y, color)

	var tex: ImageTexture
	if _resource_overlay.texture == null:
		tex = ImageTexture.create_from_image(img)
		_resource_overlay.texture = tex
	else:
		tex = _resource_overlay.texture as ImageTexture
		tex.update(img)


## 단일 타일 픽셀 업데이트 (전체 재렌더링 없이)
## 에디터 브러시 페인팅 시 사용. flush_pixel_updates() 호출로 GPU 반영.
func update_tile_pixel(x: int, y: int) -> void:
	if _img == null or _world_data_ref == null:
		return
	var biome: int = _world_data_ref.get_biome(x, y)
	var elev: float = _world_data_ref.get_elevation(x, y)
	var base_color: Color = GameConfig.BIOME_COLORS.get(biome, Color.MAGENTA)
	var brightness: float = lerpf(0.7, 1.3, elev)
	var final_color := Color(
		clampf(base_color.r * brightness, 0.0, 1.0),
		clampf(base_color.g * brightness, 0.0, 1.0),
		clampf(base_color.b * brightness, 0.0, 1.0),
	)
	_img.set_pixel(x, y, final_color)


## 배치 업데이트 flush — 브러시 스트로크 끝에 한 번 호출하여 GPU에 반영
func flush_pixel_updates() -> void:
	if _img == null:
		return
	if texture is ImageTexture:
		(texture as ImageTexture).update(_img)
