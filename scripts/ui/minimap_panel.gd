class_name MinimapPanel
extends Control

var _world_data: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _camera: Camera2D

var _minimap_texture: ImageTexture
var _minimap_rect: TextureRect
var _needs_update: bool = true

var minimap_size: int = 250


func init(world_data: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted, camera: Camera2D) -> void:
	_world_data = world_data
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_camera = camera


func _ready() -> void:
	set_anchors_preset(Control.PRESET_TOP_RIGHT)
	offset_right = -10
	offset_left = -(10 + minimap_size)
	offset_top = 38
	offset_bottom = 38 + minimap_size
	custom_minimum_size = Vector2(minimap_size, minimap_size)

	_minimap_rect = TextureRect.new()
	_minimap_rect.set_anchors_preset(Control.PRESET_FULL_RECT)
	_minimap_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED
	_minimap_rect.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_minimap_rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_minimap_rect)

	mouse_filter = Control.MOUSE_FILTER_STOP


func _draw() -> void:
	# Background
	draw_rect(Rect2(Vector2.ZERO, size), Color(0, 0, 0, 0.7))
	# Border
	draw_rect(Rect2(Vector2.ZERO, size), Color.WHITE, false, 1.0)

	# Camera viewport rectangle
	if _camera == null or _world_data == null:
		return
	var scale_x: float = size.x / float(_world_data.width)
	var scale_y: float = size.y / float(_world_data.height)

	var viewport_size := _camera.get_viewport_rect().size / _camera.zoom
	var cam_pos := _camera.global_position

	var rect_x: float = (cam_pos.x / float(GameConfig.TILE_SIZE) - viewport_size.x * 0.5 / float(GameConfig.TILE_SIZE)) * scale_x
	var rect_y: float = (cam_pos.y / float(GameConfig.TILE_SIZE) - viewport_size.y * 0.5 / float(GameConfig.TILE_SIZE)) * scale_y
	var rect_w: float = viewport_size.x / float(GameConfig.TILE_SIZE) * scale_x
	var rect_h: float = viewport_size.y / float(GameConfig.TILE_SIZE) * scale_y

	draw_rect(Rect2(rect_x, rect_y, rect_w, rect_h), Color(1, 1, 1, 0.6), false, 1.0)

	# Settlement labels
	if _settlement_manager != null:
		var font: Font = ThemeDB.fallback_font
		var active: Array = _settlement_manager.get_active_settlements()
		for i in range(active.size()):
			var s: RefCounted = active[i]
			var sx: float = float(s.center_x) * scale_x
			var sy: float = float(s.center_y) * scale_y
			var label_text: String = "S%d" % s.id
			draw_string(font, Vector2(sx + 3, sy - 2), label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("minimap_label"), Color(1, 1, 1, 0.8))


func _process(_delta: float) -> void:
	queue_redraw()


func request_update() -> void:
	_needs_update = true


func update_minimap() -> void:
	if not _needs_update or _world_data == null:
		return
	_needs_update = false

	var w: int = _world_data.width
	var h: int = _world_data.height
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)

	# 1. Biome base colors
	for y in range(h):
		for x in range(w):
			var biome: int = _world_data.get_biome(x, y)
			var color: Color = GameConfig.BIOME_COLORS.get(biome, Color.MAGENTA)
			img.set_pixel(x, y, color)

	# 2. Building markers (3x3)
	if _building_manager != null:
		var buildings: Array = _building_manager.get_all_buildings()
		for i in range(buildings.size()):
			var b = buildings[i]
			var color: Color
			match b.building_type:
				"stockpile":
					color = Color.YELLOW
				"shelter":
					color = Color.ORANGE
				_:
					color = Color.RED
			_draw_marker(img, b.tile_x, b.tile_y, color, w, h)

	# 3. Entity dots (1px)
	if _entity_manager != null:
		var alive: Array = _entity_manager.get_alive_entities()
		for i in range(alive.size()):
			var e: RefCounted = alive[i]
			var color: Color = Color.WHITE
			match e.job:
				"gatherer":
					color = Color.GREEN
				"lumberjack":
					color = Color(0.6, 0.3, 0.1)
				"builder":
					color = Color.ORANGE
				"miner":
					color = Color(0.5, 0.7, 1.0)
			var px: int = clampi(e.position.x, 0, w - 1)
			var py: int = clampi(e.position.y, 0, h - 1)
			img.set_pixel(px, py, color)

	if _minimap_texture == null:
		_minimap_texture = ImageTexture.create_from_image(img)
		_minimap_rect.texture = _minimap_texture
	else:
		_minimap_texture.update(img)


func _draw_marker(img: Image, tx: int, ty: int, color: Color, w: int, h: int) -> void:
	for dy in range(-1, 2):
		for dx in range(-1, 2):
			var px: int = clampi(tx + dx, 0, w - 1)
			var py: int = clampi(ty + dy, 0, h - 1)
			img.set_pixel(px, py, color)


func resize(new_size: int) -> void:
	minimap_size = new_size
	offset_left = -(10 + minimap_size)
	offset_bottom = 38 + minimap_size
	custom_minimum_size = Vector2(minimap_size, minimap_size)
	size = Vector2(minimap_size, minimap_size)
	_needs_update = true


func apply_ui_scale(base_size: int) -> void:
	resize(base_size)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		if _world_data == null or _camera == null:
			return
		var mb: InputEventMouseButton = event as InputEventMouseButton
		var local_pos: Vector2 = mb.position
		var scale_x: float = float(_world_data.width) / size.x
		var scale_y: float = float(_world_data.height) / size.y
		var world_x: int = int(local_pos.x * scale_x)
		var world_y: int = int(local_pos.y * scale_y)
		_camera.position = Vector2(world_x * GameConfig.TILE_SIZE, world_y * GameConfig.TILE_SIZE)
		accept_event()
