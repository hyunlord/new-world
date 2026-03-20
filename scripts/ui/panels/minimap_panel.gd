extends Control

var _world_data: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _camera: Camera2D
var _sim_engine: RefCounted

var _minimap_texture: ImageTexture
var _minimap_rect: TextureRect
var _needs_update: bool = true
var _runtime_snapshot_cache: Dictionary = {}
var _runtime_snapshot_cache_tick: int = -1

var minimap_size: int = 140


## Initializes the panel with WorldData, EntityManager, BuildingManager, SettlementManager, and the main Camera2D.
func init(world_data: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted, camera: Camera2D, sim_engine: RefCounted = null) -> void:
	_world_data = world_data
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_camera = camera
	_sim_engine = sim_engine


func _ready() -> void:
	set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	offset_left = 8
	offset_right = 8 + minimap_size
	offset_bottom = -50
	offset_top = offset_bottom - minimap_size
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
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.02, 0.03, 0.06, 0.85))
	# Border
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.25, 0.30, 0.38), false, 1.5)

	# Camera viewport rectangle
	if _camera == null:
		return
	var world_size: Vector2i = _get_world_dimensions()
	if world_size.x <= 0 or world_size.y <= 0:
		return
	var scale_x: float = size.x / float(world_size.x)
	var scale_y: float = size.y / float(world_size.y)

	var viewport_size := _camera.get_viewport_rect().size / _camera.zoom
	var cam_pos := _camera.global_position

	var rect_x: float = (cam_pos.x / float(GameConfig.TILE_SIZE) - viewport_size.x * 0.5 / float(GameConfig.TILE_SIZE)) * scale_x
	var rect_y: float = (cam_pos.y / float(GameConfig.TILE_SIZE) - viewport_size.y * 0.5 / float(GameConfig.TILE_SIZE)) * scale_y
	var rect_w: float = viewport_size.x / float(GameConfig.TILE_SIZE) * scale_x
	var rect_h: float = viewport_size.y / float(GameConfig.TILE_SIZE) * scale_y

	draw_rect(Rect2(rect_x, rect_y, rect_w, rect_h), Color(1, 1, 1, 0.6), false, 1.0)

	# Settlement labels
	var settlements: Array = _get_runtime_settlements()
	if settlements.is_empty() and _settlement_manager != null:
		settlements = _settlement_manager.get_active_settlements()
	if not settlements.is_empty():
		var font: Font = ThemeDB.fallback_font
		for i in range(settlements.size()):
			var s: Variant = settlements[i]
			var sx: float = float(_settlement_value(s, "center_x", 0)) * scale_x
			var sy: float = float(_settlement_value(s, "center_y", 0)) * scale_y
			var label_text: String = "S%d" % int(_settlement_value(s, "id", 0))
			draw_string(font, Vector2(sx + 3, sy - 2), label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("minimap_label"), Color(1, 1, 1, 0.8))


func _process(_delta: float) -> void:
	if _sim_engine != null:
		var tick: int = int(_sim_engine.current_tick)
		if tick != _runtime_snapshot_cache_tick:
			_needs_update = true
	update_minimap()
	queue_redraw()


## Marks the minimap texture as stale so it will be regenerated on the next update_minimap call.
func request_update() -> void:
	_needs_update = true


## Regenerates the minimap texture from current world, building, and entity data if a update is pending.
func update_minimap() -> void:
	if not _needs_update:
		return
	_needs_update = false

	var snapshot: Dictionary = _get_runtime_snapshot()
	var world_size: Vector2i = _get_world_dimensions()
	var w: int = world_size.x
	var h: int = world_size.y
	if w <= 0 or h <= 0:
		return
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)

	# 1. Biome base colors
	if not snapshot.is_empty():
		var biomes: Variant = snapshot.get("biomes", PackedInt32Array())
		for y in range(h):
			for x in range(w):
				var idx: int = y * w + x
				var biome: int = int(biomes[idx]) if idx < biomes.size() else 0
				var color: Color = GameConfig.BIOME_COLORS.get(biome, Color.MAGENTA)
				img.set_pixel(x, y, color)
	elif _world_data != null:
		for y in range(h):
			for x in range(w):
				var biome: int = _world_data.get_biome(x, y)
				var color: Color = GameConfig.BIOME_COLORS.get(biome, Color.MAGENTA)
				img.set_pixel(x, y, color)

	# 2. Building markers (3x3)
	var buildings: Array = []
	if not snapshot.is_empty():
		var runtime_buildings: Variant = snapshot.get("buildings", [])
		if runtime_buildings is Array:
			buildings = runtime_buildings
	if buildings.is_empty() and _building_manager != null:
		buildings = _building_manager.get_all_buildings()
	if not buildings.is_empty():
		for i in range(buildings.size()):
			var b: Variant = buildings[i]
			var color: Color
			match str(_building_value(b, "building_type", "")):
				"stockpile":
					color = Color.YELLOW
				"shelter":
					color = Color.ORANGE
				_:
					color = Color.RED
			_draw_marker(img, int(_building_value(b, "tile_x", 0)), int(_building_value(b, "tile_y", 0)), color, w, h)

	# 3. Entity dots (1px)
	var alive: Array = []
	if not snapshot.is_empty():
		var runtime_entities: Variant = snapshot.get("entities", [])
		if runtime_entities is Array:
			alive = runtime_entities
	if alive.is_empty() and _entity_manager != null:
		alive = _entity_manager.get_alive_entities()
	if not alive.is_empty():
		for i in range(alive.size()):
			var e: Variant = alive[i]
			var color: Color = Color.WHITE
			match str(_entity_value(e, "job", "none")):
				"gatherer":
					color = Color.GREEN
				"lumberjack":
					color = Color(0.6, 0.3, 0.1)
				"builder":
					color = Color.ORANGE
				"miner":
					color = Color(0.5, 0.7, 1.0)
			var px: int = clampi(int(_entity_value(e, "x", _entity_position_component(e, "x"))), 0, w - 1)
			var py: int = clampi(int(_entity_value(e, "y", _entity_position_component(e, "y"))), 0, h - 1)
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
	offset_left = 8
	offset_right = 8 + minimap_size
	offset_bottom = -50
	offset_top = offset_bottom - minimap_size
	custom_minimum_size = Vector2(minimap_size, minimap_size)
	size = Vector2(minimap_size, minimap_size)
	_needs_update = true


func apply_ui_scale(base_size: int) -> void:
	resize(base_size)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		if _camera == null:
			return
		var world_size: Vector2i = _get_world_dimensions()
		if world_size.x <= 0 or world_size.y <= 0:
			return
		var mb: InputEventMouseButton = event as InputEventMouseButton
		var local_pos: Vector2 = mb.position
		var scale_x: float = float(world_size.x) / size.x
		var scale_y: float = float(world_size.y) / size.y
		var world_x: int = int(local_pos.x * scale_x)
		var world_y: int = int(local_pos.y * scale_y)
		_camera.position = Vector2(world_x * GameConfig.TILE_SIZE, world_y * GameConfig.TILE_SIZE)
		accept_event()


func _get_runtime_snapshot() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_minimap_snapshot"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _runtime_snapshot_cache_tick:
		_runtime_snapshot_cache_tick = tick
		_runtime_snapshot_cache = _sim_engine.get_minimap_snapshot()
	return _runtime_snapshot_cache


func _get_world_dimensions() -> Vector2i:
	var snapshot: Dictionary = _get_runtime_snapshot()
	if not snapshot.is_empty():
		return Vector2i(int(snapshot.get("width", 0)), int(snapshot.get("height", 0)))
	if _world_data != null:
		return Vector2i(_world_data.width, _world_data.height)
	return Vector2i.ZERO


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _entity_value(entity: Variant, key: String, default_value: Variant) -> Variant:
	if entity is Dictionary:
		return entity.get(key, default_value)
	if entity == null:
		return default_value
	return entity.get(key)


func _entity_position_component(entity: Variant, axis: String) -> int:
	if entity == null or entity is Dictionary:
		return 0
	var entity_position: Vector2i = entity.position
	return entity_position.x if axis == "x" else entity_position.y


func _settlement_value(settlement: Variant, key: String, default_value: Variant) -> Variant:
	if settlement is Dictionary:
		return settlement.get(key, default_value)
	if settlement == null:
		return default_value
	return settlement.get(key)


func _get_runtime_settlements() -> Array:
	var snapshot: Dictionary = _get_runtime_snapshot()
	var settlements: Variant = snapshot.get("settlements", [])
	return settlements if settlements is Array else []
