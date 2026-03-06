extends Node2D

const EntityDataClass = preload("res://scripts/core/entity/entity_data.gd")
const EntityManagerClass = preload("res://scripts/core/entity/entity_manager.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _resource_map: RefCounted
var _settlement_manager: RefCounted = null
var _sim_engine: RefCounted = null
var _runtime_world_summary_cache: Dictionary = {}
var _runtime_world_summary_cache_tick: int = -1
var selected_entity_id: int = -1
var _current_lod: int = 1
var resource_overlay_visible: bool = false

## Hover tooltip state
var _hover_entity_id: int = -1
var _hover_tooltip_lines: PackedStringArray = PackedStringArray()
var _hover_screen_pos: Vector2 = Vector2.ZERO
var _hover_check_interval: int = 0

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
func init(entity_manager: RefCounted, building_manager: RefCounted = null, resource_map: RefCounted = null, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_resource_map = resource_map
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine


func _is_leader(entity: RefCounted) -> bool:
	if _settlement_manager == null or entity.settlement_id <= 0:
		return false
	var s: RefCounted = _settlement_manager.get_settlement(entity.settlement_id)
	return s != null and s.leader_id == entity.id


func _get_snapshots() -> Array:
	if _sim_engine != null and _sim_engine.has_method("get_agent_snapshots"):
		var snaps: Array = _sim_engine.get_agent_snapshots()
		if not snaps.is_empty():
			return snaps
	# Legacy fallback: convert EntityData objects to snapshot dicts
	if _entity_manager != null:
		var legacy: Array = _entity_manager.get_alive_entities()
		var result: Array = []
		for e in legacy:
			result.append({
				"x": int(e.position.x),
				"y": int(e.position.y),
				"job": str(e.get("job") if e.get("job") != null else "none"),
				"sex": str(e.get("gender") if e.get("gender") != null else "male"),
				"growth_stage": str(e.get("age_stage") if e.get("age_stage") != null else "adult"),
				"entity_id": int(e.get("id") if e.get("id") != null else -1),
				"name": str(e.get("entity_name") if e.get("entity_name") != null else ""),
				"hunger": float(e.get("hunger") if e.get("hunger") != null else 1.0),
			})
		return result
	return []


func _ready() -> void:
	SimulationBus.tick_completed.connect(_on_tick)


func _on_tick(_tick: int) -> void:
	queue_redraw()


func _process(_delta: float) -> void:
	# Always track cursor position for smooth tooltip following
	_hover_screen_pos = get_viewport().get_mouse_position()
	_update_hover()
	if _hover_entity_id >= 0:
		queue_redraw()


func _update_hover() -> void:
	# Only check entity proximity every 3 frames (avoid per-frame L2 lookups)
	_hover_check_interval += 1
	if _hover_check_interval % 3 != 0:
		return

	var canvas_xform := get_canvas_transform()
	var mouse_world: Vector2 = canvas_xform.affine_inverse() * _hover_screen_pos
	var mouse_tile: Vector2 = mouse_world / float(GameConfig.TILE_SIZE)

	var alive: Array = _get_snapshots()
	var best_id: int = -1
	var best_dist: float = 2.0  # hover radius in tiles
	for entity in alive:
		var ex: float = float(entity.get("x", 0.0))
		var ey: float = float(entity.get("y", 0.0))
		var dist: float = Vector2(ex, ey).distance_to(mouse_tile)
		if dist < best_dist:
			best_dist = dist
			best_id = int(entity.get("entity_id", -1))

	if best_id != _hover_entity_id:
		_hover_entity_id = best_id
		if best_id >= 0:
			_build_tooltip_text(best_id)
		else:
			_hover_tooltip_lines = PackedStringArray()


func _build_tooltip_text(entity_id: int) -> void:
	var detail: Dictionary = {}
	if _sim_engine != null and _sim_engine.has_method("get_entity_detail"):
		detail = _sim_engine.get_entity_detail(entity_id)

	if detail.is_empty():
		_hover_tooltip_lines = PackedStringArray(["Agent %d" % entity_id])
		return

	# Line 1: name sex age job
	var name_str: String = str(detail.get("name", "???"))
	var sex_icon: String = "♂" if str(detail.get("sex", "")).to_lower() == "male" else "♀"
	var age_str: String = str(int(detail.get("age_years", 0)))
	var job_str: String = str(detail.get("job", "none"))
	var job_tr: String = Locale.tr_id("JOB", job_str)

	var line1: String = "%s %s %s%s %s" % [name_str, sex_icon, age_str, Locale.ltr("UI_AGE_SUFFIX"), job_tr]

	# Line 2: mood | stress state
	var mood_val: float = float(detail.get("mood_score", 0.0))
	var mood_text: String
	if mood_val > 0.3:
		mood_text = Locale.ltr("UI_MOOD_GOOD")
	elif mood_val > -0.3:
		mood_text = Locale.ltr("UI_MOOD_NEUTRAL")
	else:
		mood_text = Locale.ltr("UI_MOOD_BAD")

	var stress_state: String = str(detail.get("stress_state", "calm"))
	# Locale.tr_id() calls .to_upper() internally, so "Alert" → STRESS_STATE_ALERT.
	# New locale keys STRESS_STATE_ALERT/RESISTANCE/EXHAUSTION/COLLAPSE added to
	# both en and ko ui.json to cover the Rust enum variants.
	var stress_tr: String = Locale.tr_id("STRESS_STATE", stress_state)

	var line2: String = "%s: %s | %s: %s" % [
		Locale.ltr("UI_MOOD"), mood_text,
		Locale.ltr("UI_STRESS"), stress_tr
	]

	_hover_tooltip_lines = PackedStringArray([line1, line2])


func _draw_hover_tooltip() -> void:
	if _hover_tooltip_lines.is_empty():
		return

	var font: Font = ThemeDB.fallback_font
	var font_size: int = 11
	var line_h: float = 16.0
	var padding := Vector2(8.0, 5.0)

	# Measure text width
	var max_w: float = 0.0
	for line in _hover_tooltip_lines:
		var lw: float = font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
		max_w = maxf(max_w, lw)

	var box_w: float = max_w + padding.x * 2.0
	var box_h: float = _hover_tooltip_lines.size() * line_h + padding.y * 2.0

	# Position: offset from cursor, clamped to viewport
	var pos: Vector2 = _hover_screen_pos + Vector2(16.0, -box_h - 8.0)
	var vp_size: Vector2 = get_viewport_rect().size
	pos.x = clampf(pos.x, 4.0, vp_size.x - box_w - 4.0)
	pos.y = clampf(pos.y, 4.0, vp_size.y - box_h - 4.0)

	# Convert screen position to canvas (draw) coordinates
	var canvas_xform := get_canvas_transform()
	var local_pos: Vector2 = canvas_xform.affine_inverse() * pos

	# Background box
	draw_rect(Rect2(local_pos, Vector2(box_w, box_h)), Color(0.078, 0.078, 0.078, 0.93))
	# Border
	draw_rect(Rect2(local_pos, Vector2(box_w, box_h)), Color(0.3, 0.3, 0.3, 0.5), false, 1.0)
	# Text lines
	for i in range(_hover_tooltip_lines.size()):
		var text_color: Color
		if i == 0:
			text_color = Color(0.949, 0.851, 0.420)  # gold — name line
		else:
			text_color = Color(0.70, 0.70, 0.70)     # gray — info line
		draw_string(
			font,
			local_pos + padding + Vector2(0.0, (i + 1) * line_h - 3.0),
			_hover_tooltip_lines[i],
			HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, text_color
		)


func _draw() -> void:
	var alive: Array = _get_snapshots()
	if alive.is_empty() and _entity_manager == null:
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

	var half_tile := Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)

	# LOD 0: draw minimal dots so entities are visible even at max zoom out
	if _current_lod == 0:
		for i in range(alive.size()):
			var entity: Dictionary = alive[i]
			var ex: float = float(entity.get("x", 0.0))
			var ey: float = float(entity.get("y", 0.0))
			if ex < min_tile_x or ex > max_tile_x:
				continue
			if ey < min_tile_y or ey > max_tile_y:
				continue
			var pos := Vector2(ex, ey) * GameConfig.TILE_SIZE + half_tile
			var ejob: String = str(entity.get("job", "none"))
			var vis: Dictionary = JOB_VISUALS.get(ejob, JOB_VISUALS["none"])
			var color: Color = vis["color"]
			var esex: String = str(entity.get("sex", "male"))
			var tint: Color = MALE_TINT if esex == "male" else FEMALE_TINT
			color = color.lerp(tint, GENDER_TINT_WEIGHT)
			# Minimum 3px dot ensures visibility at any zoom level
			var dot_size: float = maxf(3.0, 2.0 / zl)
			draw_circle(pos, dot_size + 1.0, OUTLINE_COLOR)
			draw_circle(pos, dot_size, color)
			# Selection highlight even at LOD 0
			if int(entity.get("entity_id", -1)) == selected_entity_id:
				draw_arc(pos, dot_size + 3.0, 0, TAU, 16, Color.WHITE, 1.5)
		return

	for i in range(alive.size()):
		var entity: Dictionary = alive[i]
		var ex: float = float(entity.get("x", 0.0))
		var ey: float = float(entity.get("y", 0.0))

		# Viewport culling
		if ex < min_tile_x or ex > max_tile_x:
			continue
		if ey < min_tile_y or ey > max_tile_y:
			continue

		var pos := Vector2(ex, ey) * GameConfig.TILE_SIZE + half_tile
		var ejob: String = str(entity.get("job", "none"))
		var esex: String = str(entity.get("sex", "male"))
		var eage_stage: String = str(entity.get("growth_stage", "adult"))
		var eid: int = int(entity.get("entity_id", -1))
		var ename: String = str(entity.get("name", ""))

		var vis: Dictionary = JOB_VISUALS.get(ejob, JOB_VISUALS["none"])
		var base_size: float = vis["size"]
		var color: Color = vis["color"]

		# Gender tint
		var tint: Color = MALE_TINT if esex == "male" else FEMALE_TINT
		color = color.lerp(tint, GENDER_TINT_WEIGHT)

		# Age size scaling
		var size: float = base_size * AGE_SIZE_MULT.get(eage_stage, 1.0)

		# Draw outlined shape
		match ejob:
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
		if eage_stage == "elder":
			draw_circle(pos + Vector2(0, -(size + 1.5)), 1.2, Color(0.9, 0.9, 0.9))

		if _current_lod >= 1:
			# Carrying indicator: skipped for snapshot entities (no carry data)

			# Hunger warning
			if float(entity.get("hunger", 1.0)) < HUNGER_WARNING_THRESHOLD:
				draw_circle(pos + Vector2(0, -(size + 5.0)), HUNGER_WARNING_RADIUS, Color.RED)

			## Leader crown [♛ = Unicode U+265B, locale-exempt symbol]
			if false: # TODO: leader check needs entity detail panel
				var crown_font: Font = ThemeDB.fallback_font
				draw_string(crown_font, pos + Vector2(-3.0, -(size + 10.0)), "\u265B", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(1.0, 0.82, 0.1))

			# Selection highlight
			if eid == selected_entity_id:
				draw_arc(pos, SELECTION_RADIUS, 0, TAU, 24, Color.WHITE, 1.5)
				# Draw line to action target: skipped for snapshot entities
				if false: # TODO: action target needs entity detail panel
					pass
				# Partner heart marker: skipped for snapshot entities
				if false: # TODO: partner check needs entity detail panel
					pass

		# LOD 2: Show names for all entities
		if _current_lod == 2:
			var entity_name: String = ename
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
					draw_string(res_font, tpos + Vector2(-3, 4), Locale.ltr("UI_RES_FOOD_SHORT"), HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(1.0, 0.85, 0.0, 0.9))
				elif stone > 2.0:
					draw_string(res_font, tpos + Vector2(-3, 4), Locale.ltr("UI_RES_STONE_SHORT"), HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(0.4, 0.6, 1.0, 0.9))
				elif wood > 3.0:
					draw_string(res_font, tpos + Vector2(-3, 4), Locale.ltr("UI_RES_WOOD_SHORT"), HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(0.0, 0.8, 0.2, 0.9))

	_draw_hover_tooltip()


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
	if _entity_manager == null and _sim_engine == null:
		return
	var now: float = Time.get_ticks_msec() / 1000.0

	# Convert screen position to world position
	var canvas_transform := get_canvas_transform()
	var world_pos: Vector2 = canvas_transform.affine_inverse() * screen_pos
	var tile_pos: Vector2 = world_pos / float(GameConfig.TILE_SIZE)
	var tile: Vector2i = Vector2i(int(floor(tile_pos.x)), int(floor(tile_pos.y)))

	# Check building at tile first
	var building: Variant = null
	if _building_manager != null:
		building = _building_manager.get_building_at(tile.x, tile.y)
	if building == null:
		building = _get_runtime_building_at(tile.x, tile.y)
	if building != null:
		var building_id: int = int(_building_value(building, "id", -1))
		if building_id >= 0:
			var is_double: bool = (building_id == _last_click_building_id
				and (now - _last_click_time) < DOUBLE_CLICK_THRESHOLD
				and screen_pos.distance_to(_last_click_pos) < DOUBLE_CLICK_DRAG_THRESHOLD)

			selected_entity_id = -1
			SimulationBus.entity_deselected.emit()
			SimulationBus.building_selected.emit(building_id)

			if is_double:
				SimulationBus.ui_notification.emit("open_building_detail", "command")

			_last_click_building_id = building_id
			_last_click_entity_id = -1
			_last_click_time = now
			_last_click_pos = screen_pos
			return

	# Find entity at or near this tile
	var alive: Array = _get_snapshots()
	var best_entity_id: int = -1
	var best_dist: float = 3.0  # max click distance in tiles
	for i in range(alive.size()):
		var entity: Dictionary = alive[i]
		var entity_pos: Vector2 = Vector2(
			float(entity.get("x", 0.0)),
			float(entity.get("y", 0.0))
		)
		var dist: float = entity_pos.distance_to(tile_pos)
		if dist < best_dist:
			best_dist = dist
			best_entity_id = int(entity.get("entity_id", -1))

	if best_entity_id != -1:
		var is_double: bool = (best_entity_id == _last_click_entity_id
			and (now - _last_click_time) < DOUBLE_CLICK_THRESHOLD
			and screen_pos.distance_to(_last_click_pos) < DOUBLE_CLICK_DRAG_THRESHOLD)

		selected_entity_id = best_entity_id
		SimulationBus.building_deselected.emit()
		SimulationBus.entity_selected.emit(best_entity_id)

		if is_double:
			SimulationBus.ui_notification.emit("open_entity_detail", "command")

		_last_click_entity_id = best_entity_id
		_last_click_building_id = -1
		_last_click_time = now
		_last_click_pos = screen_pos
	else:
		selected_entity_id = -1
		_last_click_entity_id = -1
		_last_click_building_id = -1
		SimulationBus.entity_deselected.emit()
		SimulationBus.building_deselected.emit()


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _get_runtime_building_at(tile_x: int, tile_y: int) -> Variant:
	if _sim_engine == null or not _sim_engine.has_method("get_world_summary"):
		return null
	var tick: int = int(_sim_engine.current_tick)
	if tick != _runtime_world_summary_cache_tick:
		_runtime_world_summary_cache_tick = tick
		_runtime_world_summary_cache = _sim_engine.get_world_summary()
	var settlement_summaries: Variant = _runtime_world_summary_cache.get("settlement_summaries", [])
	if not (settlement_summaries is Array):
		return null
	for i in range(settlement_summaries.size()):
		var settlement_summary_raw: Variant = settlement_summaries[i]
		if not (settlement_summary_raw is Dictionary):
			continue
		var settlement_summary: Dictionary = settlement_summary_raw
		var settlement_raw: Variant = settlement_summary.get("settlement", {})
		if not (settlement_raw is Dictionary):
			continue
		var settlement: Dictionary = settlement_raw
		var buildings: Variant = settlement.get("buildings", [])
		if not (buildings is Array):
			continue
		for j in range(buildings.size()):
			var building_raw: Variant = buildings[j]
			if not (building_raw is Dictionary):
				continue
			var building: Dictionary = building_raw
			if int(building.get("tile_x", -1)) == tile_x and int(building.get("tile_y", -1)) == tile_y:
				return building
	return null
