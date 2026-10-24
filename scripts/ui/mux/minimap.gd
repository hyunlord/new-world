# minimap.gd — cached terrain, rooms, walls and household/animal dots with a live camera rectangle.
# Bridge methods via SimReads: get_terrain_snapshot, get_colony_versions, get_rooms, get_space, get_agent_snapshot, get_animals.
# Refresh starts at most twice per second; pixel work is spread over frames, with measured work time.
extends PanelContainer

signal map_used

const ArtData = preload("res://scripts/world/art/art_data.gd")
const People = preload("res://scripts/world/colony/agent_layer.gd")
const View = preload("res://scripts/world/colony/view.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const INTERVAL: float = 0.5
const WORK_BUDGET_USEC: int = 250
const MAP_SIZE: Vector2 = Vector2(200, 200)

class MapCanvas extends Control:
	var map: PanelContainer
	func _draw() -> void: map.call("draw_map", self)
	func _gui_input(event: InputEvent) -> void: map.call("map_input", event)

var _world: Node = null
var _camera: Camera2D = null
var _people_layer: Node = null
var _canvas: MapCanvas = null
var _toggle: Button = null
var _texture: ImageTexture = null
var _terrain_image: Image = null
var _static_image: Image = null
var _static_key: Vector2i = Vector2i(-1, -1)
var _image: Image = null
var _published: Image = null
var _size: Vector2i = Vector2i.ZERO
var _terrain: PackedByteArray = PackedByteArray()
var _palette: Dictionary = {}
var _snapshots: Dictionary = {}
var _layers: Array[Dictionary] = []
var _households: Dictionary = {}
var _phase: int = -1
var _index: int = 0
var _elapsed: float = INTERVAL
var _params_version: int = -1
var _dragging: bool = false
var _collapsed: bool = false
var _completed: int = 0
var _refreshes: int = 0
var _max_work_usec: int = 0
var _max_frame_usec: int = 0
var _total_usec: int = 0
var _frames: int = 0
var _max_draw_usec: int = 0

func setup(world: Node, camera: Camera2D) -> void:
	_world = world
	_camera = camera
	_people_layer = get_node_or_null("/root/Main/ColonyView/AgentLayer")
	if _canvas == null:
		theme = THEME
		var column: VBoxContainer = VBoxContainer.new()
		add_child(column)
		_toggle = Button.new()
		_toggle.text = _text("UI_MUX_MINIMAP")
		_toggle.pressed.connect(func() -> void: set_collapsed(not _collapsed))
		column.add_child(_toggle)
		_canvas = MapCanvas.new()
		_canvas.map = self
		_canvas.custom_minimum_size = MAP_SIZE
		_canvas.mouse_filter = Control.MOUSE_FILTER_STOP
		_canvas.clip_contents = true
		_canvas.tooltip_text = _text("UI_MUX_MINIMAP_HINT")
		column.add_child(_canvas)
		set_collapsed(false)
	var locale: Node = get_node_or_null("/root/Locale")
	if locale != null and not locale.is_connected("language_changed", _locale_changed):
		locale.connect("language_changed", _locale_changed)
	_locale_changed("")
	if _world != null and _world.has_signal("world_reset") and not _world.is_connected("world_reset", _reset):
		_world.connect("world_reset", _reset)

func set_collapsed(on: bool) -> void:
	_collapsed = on
	_dragging = false
	if _canvas != null:
		_canvas.visible = not on
		_toggle.tooltip_text = _text("UI_MUX_MINIMAP_EXPAND" if on else "UI_MUX_MINIMAP_COLLAPSE")

func is_collapsed() -> bool: return _collapsed
func is_building() -> bool: return _phase >= 0
func map_image() -> Image: return _published

func refresh_snapshots(snapshots: Dictionary) -> void:
	if is_building(): return
	var terrain: Dictionary = snapshots.get("terrain", {})
	var dims: Vector2i = Vector2i(int(terrain.get("width", 0)), int(terrain.get("height", 0)))
	var values: PackedByteArray = terrain.get("terrain", PackedByteArray())
	if dims.x <= 0 or dims.y <= 0 or values.size() != dims.x * dims.y: return
	_snapshots = snapshots
	_size = dims
	_index = 0
	_refreshes += 1
	if _terrain_image == null or _terrain_image.get_size() != dims or _params_version != ArtData.params_version() or _terrain != values:
		_static_image = null
		_terrain = values
		_palette.clear()
		_params_version = ArtData.params_version()
		_terrain_image = Image.create(dims.x, dims.y, false, Image.FORMAT_RGBA8)
		_phase = 0
	else:
		_begin_overlay()

func _begin_overlay() -> void:
	_layers = [_snapshots.get("rooms", {}), _snapshots.get("space", {}), _snapshots.get("agents", {}), _snapshots.get("animals", {})]
	var key: Vector2i = Vector2i(int(_layers[0].get("version", 0)), int(_layers[1].get("version", 0)))
	var rebuild: bool = _static_image == null or key != _static_key
	_static_key = key
	_image = _terrain_image.duplicate() if rebuild else _static_image.duplicate()
	var agents: Dictionary = _layers[2]
	if _people_layer == null:
		_households = People.household_keys(agents.get("agent_ids", PackedInt64Array()), agents.get("life_stages", PackedByteArray()), agents.get("mothers", PackedInt64Array()), agents.get("fathers", PackedInt64Array()), agents.get("partners", PackedInt64Array()))
	_phase = 1 if rebuild else 3
	_index = 0

func process_pending() -> void:
	var start: int = Time.get_ticks_usec()
	while is_building() and Time.get_ticks_usec() - start < WORK_BUDGET_USEC:
		if _phase == 0:
			if _index >= _terrain.size():
				_begin_overlay()
				continue
			_terrain_image.set_pixel(_index % _size.x, floori(float(_index) / _size.x), terrain_colour(_terrain[_index]))
			_index += 1
		elif _phase <= 4:
			_paint_layer()
		else:
			_published = _image
			if _texture == null or _texture.get_size() != Vector2(_size):
				_texture = ImageTexture.create_from_image(_image)
			else:
				_texture.update(_image)
			_completed += 1
			_phase = -1
	_max_work_usec = maxi(_max_work_usec, Time.get_ticks_usec() - start)

func _paint_layer() -> void:
	var row: Dictionary = _layers[_phase - 1]
	var prefix: String = "tile_" if _phase == 1 else ("wall_" if _phase == 2 else "")
	var xs: PackedInt32Array = row.get(prefix + "xs", PackedInt32Array())
	var ys: PackedInt32Array = row.get(prefix + "ys", PackedInt32Array())
	if _index >= mini(xs.size(), ys.size()):
		if _phase == 2: _static_image = _image.duplicate()
		_phase += 1
		_index = 0
		return
	var colour: Color = get_theme_color("font_color", "Heading") if _phase == 1 else get_theme_color("font_color", "Label")
	if _phase == 3:
		var ids: PackedInt64Array = row.get("agent_ids", PackedInt64Array())
		if _index < ids.size():
			var key: int = int(_people_layer.call("household_of", ids[_index])) if _people_layer != null else int(_households.get(ids[_index], ids[_index]))
			if key < 0: key = ids[_index]
			colour = People._ring_colour(key, ArtData.param("lod.household_ring_rgb", []))
	elif _phase == 4:
		var kinds: PackedInt32Array = row.get("kinds", PackedInt32Array())
		if _index < kinds.size():
			var rgb: Array = ArtData.binding("animals", kinds[_index]).get("lod_colour", [])
			if rgb.size() == 3: colour = Color8(int(rgb[0]), int(rgb[1]), int(rgb[2]))
	var point: Vector2i = Vector2i(xs[_index], ys[_index])
	if Rect2i(Vector2i.ZERO, _size).has_point(point): _image.set_pixelv(point, colour)
	_index += 1

func terrain_colour(code: int) -> Color:
	if _palette.has(code): return _palette[code]
	var binding: Dictionary = ArtData.binding("terrain", code)
	var mean: float = float(ArtData.param("ground.tone.mean_max", 0.38))
	var tint: Array = ArtData.param("water.map_mode_colour", []) if bool(binding.get("water", false)) else (ArtData.param("ground.map_mode.tint_by_ground", {}) as Dictionary).get(str(binding.get("ground", "")), [])
	var colour: Color = Color(mean * 0.9, mean * 0.9, mean * 0.9)
	if tint.size() == 3:
		colour = Color(float(tint[0]), float(tint[1]), float(tint[2]))
	if not bool(binding.get("water", false)):
		colour = Color.from_hsv(colour.h, minf(colour.s, float(ArtData.param("ground.tone.saturation_max", 0.30))), colour.v)
		var value: float = minf(1.0, mean / maxf(Tone.LR * colour.r + Tone.LG * colour.g + Tone.LB * colour.b, 0.000001))
		colour = Color(colour.r * value, colour.g * value, colour.b * value)
		var lift: float = maxf(0.0, float(ArtData.param("ground.tone.mean_min", 0.28)) - colour.get_luminance())
		colour = Color(colour.r + lift, colour.g + lift, colour.b + lift)
	_palette[code] = colour
	return colour

func _process(delta: float) -> void:
	if _collapsed or _canvas == null: return
	var start: int = Time.get_ticks_usec()
	_elapsed += delta
	if _world != null and _elapsed >= INTERVAL and not is_building():
		_elapsed = 0.0
		_poll_snapshots()
	process_pending()
	_canvas.queue_redraw()
	var elapsed: int = Time.get_ticks_usec() - start
	_max_frame_usec = maxi(_max_frame_usec, elapsed)
	_total_usec += elapsed
	_frames += 1

func _poll_snapshots() -> void:
	var versions: Dictionary = SimReads.read(_world, "get_colony_versions")
	var rooms: Dictionary = _snapshots.get("rooms", {})
	var space: Dictionary = _snapshots.get("space", {})
	if rooms.is_empty() or int(rooms.get("version", -1)) != int(versions.get("rooms_version", 0)):
		rooms = SimReads.read(_world, "get_rooms")
	if space.is_empty() or int(space.get("version", -1)) != int(versions.get("overlay_version", 0)):
		space = SimReads.read(_world, "get_space")
	refresh_snapshots({"terrain": SimReads.read(_world, "get_terrain_snapshot"), "rooms": rooms, "space": space, "agents": SimReads.read(_world, "get_agent_snapshot"), "animals": SimReads.read(_world, "get_animals")})

func draw_map(canvas: Control) -> void:
	if _texture == null: return
	var start: int = Time.get_ticks_usec()
	var rect: Rect2 = _map_rect()
	canvas.draw_texture_rect(_texture, rect, false)
	if _camera == null:
		_max_draw_usec = maxi(_max_draw_usec, Time.get_ticks_usec() - start)
		return
	var world_size: Vector2 = Vector2(_size) * View.TILE
	var extent: Vector2 = _camera.get_viewport_rect().size / _camera.zoom
	var visible_world: Rect2 = Rect2(_camera.get_screen_center_position() - extent * 0.5, extent)
	var view: Rect2 = Rect2(rect.position + visible_world.position / world_size * rect.size, visible_world.size / world_size * rect.size)
	canvas.draw_rect(view.intersection(rect), get_theme_color("font_color", "Label"), false, 1.0)
	_max_draw_usec = maxi(_max_draw_usec, Time.get_ticks_usec() - start)

func _map_rect() -> Rect2:
	if _size.x <= 0 or _size.y <= 0: return Rect2()
	var scale_factor: float = minf(_canvas.size.x / _size.x, _canvas.size.y / _size.y)
	var extent: Vector2 = Vector2(_size) * scale_factor
	return Rect2((_canvas.size - extent) * 0.5, extent)

func map_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		_dragging = event.pressed and _map_rect().has_point(event.position)
		if _dragging: _jump_from_pixel(event.position)
		_canvas.accept_event()
	elif event is InputEventMouseMotion and _dragging:
		_jump_from_pixel(event.position)
		_canvas.accept_event()

func _jump_from_pixel(point: Vector2) -> void:
	var rect: Rect2 = _map_rect()
	if rect.has_area(): jump_to_fraction((point - rect.position) / rect.size)

func jump_to_fraction(point: Vector2) -> void:
	if _camera == null or _size.x <= 0 or _size.y <= 0: return
	_camera.call("center_on_tile", Vector2i(clampi(floori(point.x * _size.x), 0, _size.x - 1), clampi(floori(point.y * _size.y), 0, _size.y - 1)))
	map_used.emit()

func performance_stats() -> Dictionary:
	return {"refreshes": _refreshes, "completed": _completed, "max_work_usec": _max_work_usec, "max_draw_usec": _max_draw_usec, "conservative_frame_usec": _max_frame_usec + _max_draw_usec, "max_frame_usec": _max_frame_usec, "mean_frame_usec": float(_total_usec) / maxi(1, _frames)}

func _reset() -> void:
	_terrain_image = null
	_static_image = null
	_snapshots.clear()
	_phase = -1
	_elapsed = INTERVAL

func _text(key: String) -> String:
	var locale: Node = get_node_or_null("/root/Locale")
	return str(locale.call("ltr", key)) if locale != null else key

func _locale_changed(_language: String) -> void:
	_toggle.text = _text("UI_MUX_MINIMAP")
	_canvas.tooltip_text = _text("UI_MUX_MINIMAP_HINT")
	_toggle.tooltip_text = _text("UI_MUX_MINIMAP_EXPAND" if _collapsed else "UI_MUX_MINIMAP_COLLAPSE")
