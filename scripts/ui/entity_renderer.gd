class_name EntityRenderer
extends Node2D

const EntityDataClass = preload("res://scripts/core/entity_data.gd")
const EntityManagerClass = preload("res://scripts/core/entity_manager.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _resource_map: RefCounted
var selected_entity_id: int = -1
var _current_lod: int = 1
var resource_overlay_visible: bool = false

## Double-click detection
var _last_click_time: float = 0.0
var _last_click_pos: Vector2 = Vector2.ZERO
var _last_click_entity_id: int = -1
var _last_click_building_id: int = -1
const DOUBLE_CLICK_THRESHOLD: float = 0.4
const DOUBLE_CLICK_DRAG_THRESHOLD: float = 5.0

const SELECTION_RADIUS: float = 7.0
const HUNGER_WARNING_RADIUS: float = 2.0
const HUNGER_WARNING_THRESHOLD: float = 0.2

## Gender tint colors (blended with job color)
const MALE_TINT: Color = Color(0.2, 0.4, 0.85)
const FEMALE_TINT: Color = Color(0.9, 0.3, 0.45)
const GENDER_TINT_WEIGHT: float = 0.2

## Entity outline for visibility
const OUTLINE_COLOR: Color = Color(1.0, 1.0, 1.0, 0.7)
const OUTLINE_WIDTH: float = 1.5
const FOLLOW_HIGHLIGHT_COLOR: Color = Color(0.3, 0.6, 1.0)

## Age size multipliers
const AGE_SIZE_MULT: Dictionary = {
	"infant": 0.45,
	"toddler": 0.55,
	"child": 0.65,
	"teen": 0.85,
	"adult": 1.0,
	"elder": 0.9,
}

## Job visual definitions: shape, size, color
const JOB_VISUALS: Dictionary = {
	"none": {"size": 4.5, "color": Color(0.6, 0.6, 0.6)},
	"gatherer": {"size": 5.5, "color": Color(0.3, 0.8, 0.2)},
	"lumberjack": {"size": 6.5, "color": Color(0.6, 0.35, 0.1)},
	"builder": {"size": 6.5, "color": Color(0.9, 0.6, 0.1)},
	"miner": {"size": 5.5, "color": Color(0.5, 0.6, 0.75)},
}

## Resource indicator colors
const RES_COLORS: Dictionary = {
	"food": Color(0.8, 0.9, 0.2),
	"wood": Color(0.2, 0.5, 0.1),
	"stone": Color(0.7, 0.7, 0.72),
}


## Initialize with entity manager reference
func init(entity_manager: RefCounted, building_manager: RefCounted = null, resource_map: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_resource_map = resource_map


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	if _entity_manager == null:
		return
	var cam := get_viewport().get_camera_2d()
	var zl: float = cam.zoom.x if cam else 1.0

	# LOD transitions with hysteresis
	if _current_lod == 0 and zl > 0.9:
		_current_lod = 1
	elif _current_lod == 1 and zl < 0.6:
		_current_lod = 0
	elif _current_lod == 1 and zl > 4.2:
		_current_lod = 2
	elif _current_lod == 2 and zl < 3.8:
		_current_lod = 1

	# Viewport culling: compute visible tile range
	var viewport_size := get_viewport_rect().size
	var cam_pos := cam.global_position if cam else Vector2.ZERO
	var half_view := viewport_size / cam.zoom * 0.5 if cam else viewport_size * 0.5
	var min_tile_x: int = int((cam_pos.x - half_view.x) / GameConfig.TILE_SIZE) - 2
	var max_tile_x: int = int((cam_pos.x + half_view.x) / GameConfig.TILE_SIZE) + 2
	var min_tile_y: int = int((cam_pos.y - half_view.y) / GameConfig.TILE_SIZE) - 2
	var max_tile_y: int = int((cam_pos.y + half_view.y) / GameConfig.TILE_SIZE) + 2

	var alive: Array = _entity_manager.get_alive_entities()
	var half_tile := Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)

	# LOD 0: draw minimal dots so entities are visible even at max zoom out
	if _current_lod == 0:
		for i in range(alive.size()):
			var entity: RefCounted = alive[i]
			if entity.position.x < min_tile_x or entity.position.x > max_tile_x:
				continue
			if entity.position.y < min_tile_y or entity.position.y > max_tile_y:
				continue
			var pos := Vector2(entity.position) * GameConfig.TILE_SIZE + half_tile
			var vis: Dictionary = JOB_VISUALS.get(entity.job, JOB_VISUALS["none"])
			var color: Color = vis["color"]
			var tint: Color = MALE_TINT if entity.gender == "male" else FEMALE_TINT
			color = color.lerp(tint, GENDER_TINT_WEIGHT)
			# Minimum 3px dot ensures visibility at any zoom level
			var dot_size: float = maxf(3.0, 2.0 / zl)
			draw_circle(pos, dot_size + 1.0, OUTLINE_COLOR)
			draw_circle(pos, dot_size, color)
			# Selection highlight even at LOD 0
			if entity.id == selected_entity_id:
				draw_arc(pos, dot_size + 3.0, 0, TAU, 16, Color.WHITE, 1.5)
		return

	for i in range(alive.size()):
		var entity: RefCounted = alive[i]

		# Viewport culling
		if entity.position.x < min_tile_x or entity.position.x > max_tile_x:
			continue
		if entity.position.y < min_tile_y or entity.position.y > max_tile_y:
			continue

		var pos := Vector2(entity.position) * GameConfig.TILE_SIZE + half_tile

		var vis: Dictionary = JOB_VISUALS.get(entity.job, JOB_VISUALS["none"])
		var base_size: float = vis["size"]
		var color: Color = vis["color"]

		# Gender tint
		var tint: Color = MALE_TINT if entity.gender == "male" else FEMALE_TINT
		color = color.lerp(tint, GENDER_TINT_WEIGHT)

		# Age size scaling
		var size: float = base_size * AGE_SIZE_MULT.get(entity.age_stage, 1.0)

		# Draw outlined shape
		match entity.job:
			"lumberjack":
				_draw_triangle_outlined(pos, size, color)
			"builder":
				_draw_square_outlined(pos, size, color)
			"miner":
				_draw_diamond_outlined(pos, size, color)
			_:
				# Circle with outline
				draw_circle(pos, size + OUTLINE_WIDTH, OUTLINE_COLOR)
				draw_circle(pos, size, color)

		# Elder white dot (gray hair indicator)
		if entity.age_stage == "elder":
			draw_circle(pos + Vector2(0, -(size + 1.5)), 1.2, Color(0.9, 0.9, 0.9))

		if _current_lod >= 1:
			# Carrying indicator (small dot above entity)
			if entity.get_total_carry() > 0.0:
				var best_res: String = _get_dominant_resource(entity)
				var dot_color: Color = RES_COLORS.get(best_res, Color.WHITE)
				draw_circle(pos + Vector2(0, -(size + 3.0)), 1.5, dot_color)

			# Hunger warning
			if entity.hunger < HUNGER_WARNING_THRESHOLD:
				draw_circle(pos + Vector2(0, -(size + 5.0)), HUNGER_WARNING_RADIUS, Color.RED)

			# Selection highlight
			if entity.id == selected_entity_id:
				draw_arc(pos, SELECTION_RADIUS, 0, TAU, 24, Color.WHITE, 1.5)
				# Draw line to action target
				if entity.action_target != Vector2i(-1, -1):
					var target_pos := Vector2(entity.action_target) * GameConfig.TILE_SIZE + half_tile
					draw_dashed_line(pos, target_pos, Color(1, 1, 1, 0.3), 1.0, 4.0)
				# Partner heart marker
				if entity.partner_id != -1:
					var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
					if partner != null and partner.is_alive:
						var ppos := Vector2(partner.position) * GameConfig.TILE_SIZE + half_tile
						_draw_heart(ppos + Vector2(0, -(size + 6.0)), 3.0, Color(1.0, 0.3, 0.4))
						draw_dashed_line(pos, ppos, Color(1.0, 0.3, 0.4, 0.3), 1.0, 4.0)

		# LOD 2: Show names for all entities
		if _current_lod == 2:
			var entity_name: String = entity.entity_name
			# Background rect for readability
			var name_font: Font = ThemeDB.fallback_font
			var name_size: Vector2 = name_font.get_string_size(entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 11)
			draw_rect(Rect2(pos.x + size + 2, pos.y - size - 4 - name_size.y, name_size.x + 4, name_size.y + 2), Color(0, 0, 0, 0.6))
			draw_string(name_font, pos + Vector2(size + 4.0, -size - 3.0), entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color.WHITE)

	# Resource text markers at high zoom (LOD 2)
	if _current_lod == 2 and resource_overlay_visible and _resource_map != null:
		var res_font: Font = ThemeDB.fallback_font
		for ty in range(maxi(0, min_tile_y), mini(_resource_map.height, max_tile_y + 1)):
			for tx in range(maxi(0, min_tile_x), mini(_resource_map.width, max_tile_x + 1)):
				var tpos := Vector2(tx, ty) * GameConfig.TILE_SIZE + half_tile
				var food: float = _resource_map.get_food(tx, ty)
				var wood: float = _resource_map.get_wood(tx, ty)
				var stone: float = _resource_map.get_stone(tx, ty)
				if food > 2.0:
					draw_string(res_font, tpos + Vector2(-3, 4), "F", HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(1.0, 0.85, 0.0, 0.9))
				elif stone > 2.0:
					draw_string(res_font, tpos + Vector2(-3, 4), "S", HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(0.4, 0.6, 1.0, 0.9))
				elif wood > 3.0:
					draw_string(res_font, tpos + Vector2(-3, 4), "W", HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(0.0, 0.8, 0.2, 0.9))


func _draw_triangle(center: Vector2, size: float, color: Color) -> void:
	var points := PackedVector2Array([
		center + Vector2(0, -size),
		center + Vector2(-size * 0.87, size * 0.5),
		center + Vector2(size * 0.87, size * 0.5),
	])
	draw_colored_polygon(points, color)


func _draw_square(center: Vector2, size: float, color: Color) -> void:
	var half: float = size * 0.5
	draw_rect(Rect2(center.x - half, center.y - half, size, size), color)


func _draw_diamond(center: Vector2, size: float, color: Color) -> void:
	var points := PackedVector2Array([
		center + Vector2(0, -size),
		center + Vector2(size, 0),
		center + Vector2(0, size),
		center + Vector2(-size, 0),
	])
	draw_colored_polygon(points, color)


func _draw_heart(center: Vector2, size: float, color: Color) -> void:
	var points := PackedVector2Array([
		center + Vector2(0, size * 0.4),
		center + Vector2(-size, -size * 0.2),
		center + Vector2(-size * 0.5, -size * 0.7),
		center + Vector2(0, -size * 0.3),
		center + Vector2(size * 0.5, -size * 0.7),
		center + Vector2(size, -size * 0.2),
	])
	draw_colored_polygon(points, color)


func _draw_triangle_outlined(center: Vector2, s: float, color: Color) -> void:
	var outline_s: float = s + OUTLINE_WIDTH
	var outline_points := PackedVector2Array([
		center + Vector2(0, -outline_s),
		center + Vector2(-outline_s * 0.87, outline_s * 0.5),
		center + Vector2(outline_s * 0.87, outline_s * 0.5),
	])
	draw_colored_polygon(outline_points, OUTLINE_COLOR)
	_draw_triangle(center, s, color)


func _draw_square_outlined(center: Vector2, s: float, color: Color) -> void:
	var outline_half: float = (s + OUTLINE_WIDTH * 2) * 0.5
	draw_rect(Rect2(center.x - outline_half, center.y - outline_half, outline_half * 2, outline_half * 2), OUTLINE_COLOR)
	_draw_square(center, s, color)


func _draw_diamond_outlined(center: Vector2, s: float, color: Color) -> void:
	var outline_s: float = s + OUTLINE_WIDTH
	var outline_points := PackedVector2Array([
		center + Vector2(0, -outline_s),
		center + Vector2(outline_s, 0),
		center + Vector2(0, outline_s),
		center + Vector2(-outline_s, 0),
	])
	draw_colored_polygon(outline_points, OUTLINE_COLOR)
	_draw_diamond(center, s, color)


func _get_dominant_resource(entity: RefCounted) -> String:
	var best: String = "food"
	var best_amount: float = 0.0
	var keys: Array = entity.inventory.keys()
	for j in range(keys.size()):
		var res: String = keys[j]
		var amount: float = entity.inventory[res]
		if amount > best_amount:
			best_amount = amount
			best = res
	return best


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_handle_click(event.global_position)


func _handle_click(screen_pos: Vector2) -> void:
	if _entity_manager == null:
		return
	var now: float = Time.get_ticks_msec() / 1000.0

	# Convert screen position to world position
	var canvas_transform := get_canvas_transform()
	var world_pos: Vector2 = canvas_transform.affine_inverse() * screen_pos
	var tile := Vector2i(int(world_pos.x) / GameConfig.TILE_SIZE, int(world_pos.y) / GameConfig.TILE_SIZE)

	# Check building at tile first
	if _building_manager != null:
		var building = _building_manager.get_building_at(tile.x, tile.y)
		if building != null:
			var is_double: bool = (building.id == _last_click_building_id
				and (now - _last_click_time) < DOUBLE_CLICK_THRESHOLD
				and screen_pos.distance_to(_last_click_pos) < DOUBLE_CLICK_DRAG_THRESHOLD)

			selected_entity_id = -1
			SimulationBus.entity_deselected.emit()
			SimulationBus.building_selected.emit(building.id)

			if is_double:
				SimulationBus.ui_notification.emit("open_building_detail", "command")

			_last_click_building_id = building.id
			_last_click_entity_id = -1
			_last_click_time = now
			_last_click_pos = screen_pos
			return

	# Find entity at or near this tile
	var alive: Array = _entity_manager.get_alive_entities()
	var best_entity: RefCounted = null
	var best_dist: float = 3.0  # max click distance in tiles
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var dist: float = Vector2(entity.position - tile).length()
		if dist < best_dist:
			best_dist = dist
			best_entity = entity

	if best_entity:
		var is_double: bool = (best_entity.id == _last_click_entity_id
			and (now - _last_click_time) < DOUBLE_CLICK_THRESHOLD
			and screen_pos.distance_to(_last_click_pos) < DOUBLE_CLICK_DRAG_THRESHOLD)

		selected_entity_id = best_entity.id
		SimulationBus.building_deselected.emit()
		SimulationBus.entity_selected.emit(best_entity.id)

		if is_double:
			SimulationBus.ui_notification.emit("open_entity_detail", "command")

		_last_click_entity_id = best_entity.id
		_last_click_building_id = -1
		_last_click_time = now
		_last_click_pos = screen_pos
	else:
		selected_entity_id = -1
		_last_click_entity_id = -1
		_last_click_building_id = -1
		SimulationBus.entity_deselected.emit()
		SimulationBus.building_deselected.emit()
