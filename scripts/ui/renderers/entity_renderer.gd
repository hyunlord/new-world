extends Node2D

const EntityDataClass = preload("res://scripts/core/entity/entity_data.gd")
const EntityManagerClass = preload("res://scripts/core/entity/entity_manager.gd")
const SnapshotDecoderClass = preload("res://scripts/rendering/snapshot_decoder.gd")
const RelationshipOverlayClass = preload("res://scripts/ui/relationship_overlay.gd")
const SocialBubbleScene = preload("res://scenes/ui/social_bubble.tscn")
const AGENT_TEXTURE_PATH: String = "res://assets/sprites/agent_base.png"
const AGENT_PALETTE_LUT_PATH: String = "res://assets/sprites/palette_lut.png"
const AGENT_VISUAL_SHADER_PATH: String = "res://shaders/stress_phase.gdshader"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _resource_map: RefCounted
var _settlement_manager: RefCounted = null
var _sim_engine: RefCounted = null
var _snapshot_decoder = SnapshotDecoderClass.new()
var _runtime_world_summary_cache: Dictionary = {}
var _runtime_world_summary_cache_tick: int = -1
var _selected_runtime_detail_cache: Dictionary = {}
var _selected_runtime_detail_cache_tick: int = -1
var _selected_runtime_detail_cache_id: int = -1
var _probe_runtime_detail_cache: Dictionary = {}
var _probe_runtime_detail_cache_tick: int = -1
var selected_entity_id: int = -1
var probe_observation_mode: bool = false
var _current_lod: int = 1
var resource_overlay_visible: bool = false
var _binary_snapshot_available: bool = false
var _render_alpha: float = 0.0
var _agent_sprites: Array[Sprite2D] = []
var _agent_bubbles: Array = []
var _agent_texture: Texture2D = null
var _agent_palette_lut: Texture2D = null
var _agent_visual_shader: Shader = null
var _relationship_overlay = null

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
const PROBE_FADED_ALPHA: float = 0.48
const PROBE_OUTLINE_ALPHA: float = 0.28
const PROBE_SELECTION_COLOR: Color = Color(1.0, 0.92, 0.35, 0.98)
const PROBE_SELECTION_RING_WIDTH: float = 2.5
const PROBE_SELECTION_HALO_ALPHA: float = 0.14
const PROBE_FORAGE_TARGET_COLOR: Color = Color(1.0, 0.86, 0.22, 0.95)
const PROBE_FORAGE_TARGET_PENDING_ALPHA: float = 0.32
const PROBE_FORAGE_LINE_WIDTH: float = 2.0
const PROBE_SURVIVAL_HUNGER_THRESHOLD: float = 0.35
const PROBE_SURVIVAL_WARMTH_THRESHOLD: float = 0.35
const PROBE_SURVIVAL_ENERGY_THRESHOLD: float = 0.30
const PROBE_SURVIVAL_SAFETY_THRESHOLD: float = 0.30
const PROBE_SURVIVAL_SELECTED_BADGE_SIZE: float = 5.6
const PROBE_SURVIVAL_UNSELECTED_BADGE_SIZE: float = 4.0
const PROBE_SURVIVAL_BADGE_GAP: float = 2.5
const PROBE_SURVIVAL_MAX_UNSELECTED: int = 2
const PROBE_SURVIVAL_SELECTED_ALPHA: float = 0.96
const PROBE_SURVIVAL_UNSELECTED_ALPHA: float = 0.74
const PROBE_SURVIVAL_LABEL_FONT_SIZE: int = 9
const PROBE_SURVIVAL_HALO_ALPHA: float = 0.18
const PROBE_SURVIVAL_HUNGER_COLOR: Color = Color(1.0, 0.79, 0.20, 1.0)
const PROBE_SURVIVAL_WARMTH_COLOR: Color = Color(0.35, 0.84, 1.0, 1.0)
const PROBE_SURVIVAL_ENERGY_COLOR: Color = Color(0.76, 0.56, 1.0, 1.0)
const PROBE_SURVIVAL_SAFETY_COLOR: Color = Color(1.0, 0.38, 0.34, 1.0)

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
const JOB_ICON_TO_KEY: Dictionary = {
	0: "none",
	1: "gatherer",
	2: "lumberjack",
	3: "builder",
	4: "miner",
}
const GROWTH_STAGE_KEYS: PackedStringArray = [
	"infant",
	"toddler",
	"child",
	"teen",
	"adult",
	"elder",
]

## Resource indicator colors
const RES_COLORS: Dictionary = {
	"food": Color(0.8, 0.9, 0.2),
	"wood": Color(0.2, 0.5, 0.1),
	"stone": Color(0.7, 0.7, 0.72),
}
const AGENT_FRAME_COLUMNS: int = 4
const AGENT_FRAME_ROWS: int = 3
const AGENT_FRAME_TIME_MS: int = 180
const AGENT_BASE_SPEED: float = 3.75
const AGENT_TEXTURE_OFFSET: Vector2 = Vector2(0.0, -4.0)
const ACTION_SOCIALIZE: int = 6
const ACTION_FIGHT: int = 13
const ACTION_TEACH: int = 15
const ACTION_LEARN: int = 16
const ACTION_VISIT_PARTNER: int = 27
const SOCIAL_INTERACTION_MAX_DISTANCE: float = 4.0
const MOOD_COLORS: Array[Color] = [
	Color("#B71C1C"),
	Color("#F44336"),
	Color("#FF9800"),
	Color("#CDDC39"),
	Color("#4CAF50"),
]
const STRESS_OUTLINE_COLORS: Array[Color] = [
	Color(0.0, 0.0, 0.0, 0.0),
	Color("#FF9800"),
	Color("#FF5722"),
	Color("#F44336"),
	Color("#B71C1C"),
]


## Initialize with entity manager reference
func init(entity_manager: RefCounted, building_manager: RefCounted = null, resource_map: RefCounted = null, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_resource_map = resource_map
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine
	_ensure_relationship_overlay()


func _is_leader(entity: RefCounted) -> bool:
	if _settlement_manager == null or entity.settlement_id <= 0:
		return false
	var s: RefCounted = _settlement_manager.get_settlement(entity.settlement_id)
	return s != null and s.leader_id == entity.id


func _get_legacy_snapshots() -> Array:
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


func _update_binary_snapshots() -> void:
	var curr_bytes: PackedByteArray = SimBridge.get_frame_snapshots()
	var prev_bytes: PackedByteArray = SimBridge.get_prev_frame_snapshots()
	var count: int = SimBridge.get_agent_count()
	_render_alpha = clampf(SimBridge.get_render_alpha(), 0.0, 1.0)
	_snapshot_decoder.update(curr_bytes, prev_bytes, count)
	_binary_snapshot_available = _snapshot_decoder.has_data()


func _ready() -> void:
	if _ensure_agent_visual_resources():
		_ensure_agent_sprite_capacity(32)
	SimulationBus.tick_completed.connect(_on_tick)


func set_probe_observation_mode(probe_enabled: bool) -> void:
	probe_observation_mode = probe_enabled
	queue_redraw()


func _on_tick(_tick: int) -> void:
	_probe_runtime_detail_cache_tick = -1
	_probe_runtime_detail_cache.clear()
	queue_redraw()


func _process(_delta: float) -> void:
	_update_binary_snapshots()
	# Always track cursor position for smooth tooltip following
	_hover_screen_pos = get_viewport().get_mouse_position()
	_update_hover()
	_update_agent_sprites()
	if _binary_snapshot_available or _hover_entity_id >= 0:
		queue_redraw()


func _update_hover() -> void:
	# Only check entity proximity every 3 frames (avoid per-frame L2 lookups)
	_hover_check_interval += 1
	if _hover_check_interval % 3 != 0:
		return

	var canvas_xform := get_canvas_transform()
	var mouse_world: Vector2 = canvas_xform.affine_inverse() * _hover_screen_pos
	var mouse_tile: Vector2 = mouse_world / float(GameConfig.TILE_SIZE)

	var best_id: int = -1
	var best_dist: float = 2.0  # hover radius in tiles
	if _binary_snapshot_available:
		for index in range(_snapshot_decoder.agent_count):
			var entity_pos: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
			var dist: float = entity_pos.distance_to(mouse_tile)
			if dist < best_dist:
				best_dist = dist
				best_id = _snapshot_decoder.get_entity_id(index)
	else:
		var alive: Array = _get_legacy_snapshots()
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
		_hover_tooltip_lines = PackedStringArray(["[%s] %s — %s" % [
			Locale.ltr("ARCHETYPE_QUIET_OBSERVER"),
			Locale.ltr("STATUS_IDLE"),
			Locale.ltr("UI_UNKNOWN"),
		]])
		return

	var archetype_key: String = str(detail.get("archetype_key", ""))
	if archetype_key.is_empty() and _sim_engine != null and _sim_engine.has_method("get_archetype_label"):
		archetype_key = _sim_engine.get_archetype_label(entity_id)
	if archetype_key.is_empty():
		archetype_key = "ARCHETYPE_QUIET_OBSERVER"

	var action_text: String = _localized_hover_action_text(str(detail.get("current_action", "Idle")))
	var motivation_text: String = _localized_hover_motivation_text(detail)
	var hover_line: String = "[%s] %s — %s" % [
		Locale.ltr(archetype_key),
		action_text,
		motivation_text,
	]
	_hover_tooltip_lines = PackedStringArray([hover_line])


func _localized_hover_action_text(action_raw: String) -> String:
	var normalized: String = _camel_to_upper_snake(action_raw)
	if normalized.is_empty():
		normalized = "IDLE"
	var locale_key: String = "STATUS_" + normalized
	var localized: String = Locale.ltr(locale_key)
	if localized == locale_key:
		return humanize_hover_text(action_raw)
	return localized


func _localized_hover_motivation_text(detail: Dictionary) -> String:
	var top_need_key: String = str(detail.get("top_need_key", ""))
	if top_need_key.is_empty():
		var need_keys: Array[String] = [
			"NEED_HUNGER",
			"NEED_THIRST",
			"NEED_SLEEP",
			"NEED_WARMTH",
			"NEED_SAFETY",
			"NEED_BELONGING",
			"NEED_INTIMACY",
			"NEED_RECOGNITION",
			"NEED_AUTONOMY",
			"NEED_COMPETENCE",
			"NEED_SELF_ACTUALIZATION",
			"NEED_MEANING",
			"NEED_TRANSCENDENCE",
			"NEED_ENERGY",
		]
		var detail_keys: Array[String] = [
			"need_hunger",
			"need_thirst",
			"need_sleep",
			"need_warmth",
			"need_safety",
			"need_belonging",
			"need_intimacy",
			"need_recognition",
			"need_autonomy",
			"need_competence",
			"need_self_actualization",
			"need_meaning",
			"need_transcendence",
			"energy",
		]
		var best_value: float = 99.0
		for index: int in range(detail_keys.size()):
			var value: float = float(detail.get(detail_keys[index], 99.0))
			if value < best_value:
				best_value = value
				top_need_key = need_keys[index]
	if top_need_key.is_empty():
		return Locale.ltr("UI_UNKNOWN")
	return Locale.ltr(top_need_key)


func _camel_to_upper_snake(value: String) -> String:
	var source: String = value.strip_edges().replace(" ", "_")
	if source.is_empty():
		return ""
	var out: PackedStringArray = PackedStringArray()
	var buffer: String = ""
	for index: int in range(source.length()):
		var character: String = source.substr(index, 1)
		var is_uppercase: bool = character == character.to_upper() and character != character.to_lower()
		if index > 0 and is_uppercase:
			var previous: String = source.substr(index - 1, 1)
			var prev_is_lower: bool = previous == previous.to_lower() and previous != previous.to_upper()
			if prev_is_lower and not buffer.is_empty():
				out.append(buffer)
				buffer = ""
		if character == "_":
			if not buffer.is_empty():
				out.append(buffer)
				buffer = ""
			continue
		buffer += character.to_upper()
	if not buffer.is_empty():
		out.append(buffer)
	return "_".join(out)


func humanize_hover_text(value: String) -> String:
	var normalized: String = value.strip_edges()
	if normalized.is_empty():
		return Locale.ltr("STATUS_IDLE")
	return normalized.replace("_", " ")


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
	if _binary_snapshot_available:
		_draw_binary_snapshots()
		return

	var alive: Array = _get_legacy_snapshots()
	if alive.is_empty() and _entity_manager == null:
		return
	var cam := get_viewport().get_camera_2d()
	var zl: float = cam.zoom.x if cam else 1.0

	# LOD transitions with hysteresis
	_update_lod(zl)

	# Viewport culling: compute visible tile range
	var viewport_size := get_viewport_rect().size
	var cam_pos := cam.global_position if cam else Vector2.ZERO
	var half_view := viewport_size / cam.zoom * 0.5 if cam else viewport_size * 0.5
	var min_tile_x: int = int((cam_pos.x - half_view.x) / GameConfig.TILE_SIZE) - 2
	var max_tile_x: int = int((cam_pos.x + half_view.x) / GameConfig.TILE_SIZE) + 2
	var min_tile_y: int = int((cam_pos.y - half_view.y) / GameConfig.TILE_SIZE) - 2
	var max_tile_y: int = int((cam_pos.y + half_view.y) / GameConfig.TILE_SIZE) + 2

	var half_tile := Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
	var selected_probe_pos: Vector2 = Vector2.INF

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
			var is_selected: bool = int(entity.get("entity_id", -1)) == selected_entity_id
			var outline_color: Color = _outline_color_for_probe(is_selected)
			color = _entity_color_for_probe(color, is_selected)
			# Minimum 3px dot ensures visibility at any zoom level
			var dot_size: float = maxf(3.0, 2.0 / zl)
			draw_circle(pos, dot_size + 1.0, outline_color)
			draw_circle(pos, dot_size, color)
			# Selection highlight even at LOD 0
			if is_selected:
				_draw_selection_indicator(pos, dot_size + 3.0, 16)
				selected_probe_pos = pos
		if selected_probe_pos != Vector2.INF:
			_draw_probe_selected_forage_overlay(selected_probe_pos)
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
		var is_selected: bool = eid == selected_entity_id

		# Gender tint
		var tint: Color = MALE_TINT if esex == "male" else FEMALE_TINT
		color = color.lerp(tint, GENDER_TINT_WEIGHT)
		color = _entity_color_for_probe(color, is_selected)
		var outline_color: Color = _outline_color_for_probe(is_selected)

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
				draw_circle(pos, size + OUTLINE_WIDTH, outline_color)
				draw_circle(pos, size, color)

		# Elder white dot (gray hair indicator)
		if eage_stage == "elder":
			draw_circle(pos + Vector2(0, -(size + 1.5)), 1.2, Color(0.9, 0.9, 0.9))

		if _current_lod >= 1:
			# Carrying indicator: skipped for snapshot entities (no carry data)

			# Hunger warning
			if float(entity.get("hunger", 1.0)) < HUNGER_WARNING_THRESHOLD and not probe_observation_mode:
				draw_circle(pos + Vector2(0, -(size + 5.0)), HUNGER_WARNING_RADIUS, Color.RED)

			## Leader crown [♛ = Unicode U+265B, locale-exempt symbol]
			if false: # TODO: leader check needs entity detail panel
				var crown_font: Font = ThemeDB.fallback_font
				draw_string(crown_font, pos + Vector2(-3.0, -(size + 10.0)), "\u265B", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(1.0, 0.82, 0.1))

			# Selection highlight
			if is_selected:
				_draw_selection_indicator(pos, SELECTION_RADIUS, 24)
				selected_probe_pos = pos
				# Partner heart marker: skipped for snapshot entities
				if false: # TODO: partner check needs entity detail panel
					pass

			_draw_probe_survival_indicators(pos, size, eid, is_selected)

		# LOD 2: Show names for all entities
		if _should_draw_name(is_selected):
			var entity_name: String = ename
			# Background rect for readability
			var name_font: Font = ThemeDB.fallback_font
			var name_size: Vector2 = name_font.get_string_size(entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 11)
			var bg_alpha: float = 0.82 if probe_observation_mode and is_selected else 0.6
			var text_color: Color = PROBE_SELECTION_COLOR if probe_observation_mode and is_selected else Color.WHITE
			draw_rect(Rect2(pos.x + size + 2, pos.y - size - 4 - name_size.y, name_size.x + 4, name_size.y + 2), Color(0, 0, 0, bg_alpha))
			draw_string(name_font, pos + Vector2(size + 4.0, -size - 3.0), entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, text_color)

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

	if selected_probe_pos != Vector2.INF:
		_draw_probe_selected_forage_overlay(selected_probe_pos)

	_draw_hover_tooltip()


func _draw_binary_snapshots() -> void:
	if not _snapshot_decoder.has_data():
		return

	var cam := get_viewport().get_camera_2d()
	var zl: float = cam.zoom.x if cam else 1.0

	_update_lod(zl)

	var viewport_size: Vector2 = get_viewport_rect().size
	var cam_pos: Vector2 = cam.global_position if cam else Vector2.ZERO
	var half_view: Vector2 = viewport_size / cam.zoom * 0.5 if cam else viewport_size * 0.5
	var min_tile_x: int = int((cam_pos.x - half_view.x) / GameConfig.TILE_SIZE) - 2
	var max_tile_x: int = int((cam_pos.x + half_view.x) / GameConfig.TILE_SIZE) + 2
	var min_tile_y: int = int((cam_pos.y - half_view.y) / GameConfig.TILE_SIZE) - 2
	var max_tile_y: int = int((cam_pos.y + half_view.y) / GameConfig.TILE_SIZE) + 2

	var half_tile: Vector2 = Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
	var selected_probe_pos: Vector2 = Vector2.INF

	if _current_lod == 0:
		for index in range(_snapshot_decoder.agent_count):
			var tile_pos: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
			if tile_pos.x < min_tile_x or tile_pos.x > max_tile_x:
				continue
			if tile_pos.y < min_tile_y or tile_pos.y > max_tile_y:
				continue
			var pos: Vector2 = tile_pos * float(GameConfig.TILE_SIZE) + half_tile
			var vis: Dictionary = JOB_VISUALS.get(_binary_job_key(_snapshot_decoder.get_job_icon(index)), JOB_VISUALS["none"])
			var color: Color = vis["color"]
			var tint: Color = MALE_TINT if _snapshot_decoder.get_sex(index) == 0 else FEMALE_TINT
			color = color.lerp(tint, GENDER_TINT_WEIGHT)
			var is_selected: bool = _snapshot_decoder.get_entity_id(index) == selected_entity_id
			var outline_color: Color = _outline_color_for_probe(is_selected)
			color = _entity_color_for_probe(color, is_selected)
			var dot_size: float = maxf(3.0, 2.0 / zl)
			draw_circle(pos, dot_size + 1.0, outline_color)
			draw_circle(pos, dot_size, color)
			if is_selected:
				_draw_selection_indicator(pos, dot_size + 3.0, 16)
				selected_probe_pos = pos
		if selected_probe_pos != Vector2.INF:
			_draw_probe_selected_forage_overlay(selected_probe_pos)
		_draw_hover_tooltip()
		return

	for index in range(_snapshot_decoder.agent_count):
		var tile_pos: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
		if tile_pos.x < min_tile_x or tile_pos.x > max_tile_x:
			continue
		if tile_pos.y < min_tile_y or tile_pos.y > max_tile_y:
			continue

		var pos: Vector2 = tile_pos * float(GameConfig.TILE_SIZE) + half_tile
		var job_key: String = _binary_job_key(_snapshot_decoder.get_job_icon(index))
		var growth_stage_key: String = _binary_growth_stage_key(_snapshot_decoder.get_growth_stage(index))
		var entity_id: int = _snapshot_decoder.get_entity_id(index)
		var is_selected: bool = entity_id == selected_entity_id
		var vis: Dictionary = JOB_VISUALS.get(job_key, JOB_VISUALS["none"])
		var size: float = float(vis["size"]) * float(AGE_SIZE_MULT.get(growth_stage_key, 1.0))

		if _current_lod >= 1:
			var danger_flags: int = _snapshot_decoder.get_danger_icon(index)
			if danger_flags & 0b0010 != 0 and not probe_observation_mode:
				draw_circle(pos + Vector2(0.0, -(size + 5.0)), HUNGER_WARNING_RADIUS, Color.RED)
			if is_selected:
				_draw_selection_indicator(pos, SELECTION_RADIUS, 24)
				selected_probe_pos = pos

			_draw_probe_survival_indicators(pos, size, entity_id, is_selected, danger_flags)

		if _should_draw_name(is_selected):
			var entity_name: String = _runtime_entity_name(entity_id)
			if entity_name.is_empty():
				entity_name = "#%d" % entity_id
			var name_font: Font = ThemeDB.fallback_font
			var name_size: Vector2 = name_font.get_string_size(entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 11)
			var bg_alpha: float = 0.82 if probe_observation_mode and is_selected else 0.6
			var text_color: Color = PROBE_SELECTION_COLOR if probe_observation_mode and is_selected else Color.WHITE
			draw_rect(Rect2(pos.x + size + 2.0, pos.y - size - 4.0 - name_size.y, name_size.x + 4.0, name_size.y + 2.0), Color(0.0, 0.0, 0.0, bg_alpha))
			draw_string(name_font, pos + Vector2(size + 4.0, -size - 3.0), entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, text_color)

	if _current_lod == 2 and resource_overlay_visible and _resource_map != null:
		var res_font: Font = ThemeDB.fallback_font
		for ty in range(maxi(0, min_tile_y), mini(_resource_map.height, max_tile_y + 1)):
			for tx in range(maxi(0, min_tile_x), mini(_resource_map.width, max_tile_x + 1)):
				var tpos: Vector2 = Vector2(tx, ty) * GameConfig.TILE_SIZE + half_tile
				var food: float = _resource_map.get_food(tx, ty)
				var wood: float = _resource_map.get_wood(tx, ty)
				var stone: float = _resource_map.get_stone(tx, ty)
				if food > 2.0:
					draw_string(res_font, tpos + Vector2(-3.0, 4.0), Locale.ltr("UI_RES_FOOD_SHORT"), HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(1.0, 0.85, 0.0, 0.9))
				elif stone > 2.0:
					draw_string(res_font, tpos + Vector2(-3.0, 4.0), Locale.ltr("UI_RES_STONE_SHORT"), HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(0.4, 0.6, 1.0, 0.9))
				elif wood > 3.0:
					draw_string(res_font, tpos + Vector2(-3.0, 4.0), Locale.ltr("UI_RES_WOOD_SHORT"), HORIZONTAL_ALIGNMENT_CENTER, -1, 8, Color(0.0, 0.8, 0.2, 0.9))

	if selected_probe_pos != Vector2.INF:
		_draw_probe_selected_forage_overlay(selected_probe_pos)

	_draw_hover_tooltip()


func _update_lod(zoom_level: float) -> void:
	if _current_lod == 0 and zoom_level > 0.9:
		_current_lod = 1
	elif _current_lod == 1 and zoom_level < 0.6:
		_current_lod = 0
	elif _current_lod == 1 and zoom_level > 4.2:
		_current_lod = 2
	elif _current_lod == 2 and zoom_level < 3.8:
		_current_lod = 1


func _entity_color_for_probe(color: Color, is_selected: bool) -> Color:
	if not probe_observation_mode or is_selected:
		return color
	return Color(color.r, color.g, color.b, PROBE_FADED_ALPHA)


func _outline_color_for_probe(is_selected: bool) -> Color:
	if not probe_observation_mode or is_selected:
		return OUTLINE_COLOR
	return Color(OUTLINE_COLOR.r, OUTLINE_COLOR.g, OUTLINE_COLOR.b, PROBE_OUTLINE_ALPHA)


func _should_draw_name(is_selected: bool) -> bool:
	if probe_observation_mode:
		return is_selected and _current_lod >= 1
	return _current_lod == 2


func _draw_selection_indicator(pos: Vector2, radius: float, points: int) -> void:
	if probe_observation_mode:
		draw_circle(pos, radius + 1.5, Color(PROBE_SELECTION_COLOR.r, PROBE_SELECTION_COLOR.g, PROBE_SELECTION_COLOR.b, PROBE_SELECTION_HALO_ALPHA))
		draw_arc(pos, radius + 1.0, 0.0, TAU, points, PROBE_SELECTION_COLOR, PROBE_SELECTION_RING_WIDTH)
		return
	draw_arc(pos, radius, 0.0, TAU, points, Color.WHITE, 1.5)


func _draw_probe_selected_forage_overlay(selected_pos: Vector2) -> void:
	if not probe_observation_mode or _sim_engine == null or _resource_map == null or selected_entity_id < 0:
		return
	var detail: Dictionary = _get_selected_runtime_detail()
	if detail.is_empty():
		return
	if str(detail.get("action_target_resource", "")) != "food":
		return
	var target_x: int = int(detail.get("action_target_x", -1))
	var target_y: int = int(detail.get("action_target_y", -1))
	if target_x < 0 or target_y < 0:
		return
	var half_tile: Vector2 = Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)
	var target_pos: Vector2 = Vector2(target_x, target_y) * float(GameConfig.TILE_SIZE) + half_tile
	var remaining_food: float = _resource_map.get_food(target_x, target_y)
	var marker_color: Color = PROBE_FORAGE_TARGET_COLOR
	var halo_alpha: float = PROBE_FORAGE_TARGET_PENDING_ALPHA
	if remaining_food <= 0.0:
		marker_color = Color(1.0, 0.42, 0.30, 0.95)
		halo_alpha = 0.22
	draw_line(selected_pos, target_pos, marker_color, PROBE_FORAGE_LINE_WIDTH, true)
	draw_circle(target_pos, 9.0, Color(marker_color.r, marker_color.g, marker_color.b, halo_alpha))
	draw_arc(target_pos, 7.5, 0.0, TAU, 24, marker_color, 2.0)
	if _current_lod >= 1:
		var resource_font: Font = ThemeDB.fallback_font
		draw_string(
			resource_font,
			target_pos + Vector2(0.0, -8.0),
			Locale.ltr("UI_RES_FOOD_SHORT"),
			HORIZONTAL_ALIGNMENT_CENTER,
			-1,
			9,
			marker_color
		)


func _draw_probe_survival_indicators(
	pos: Vector2,
	size: float,
	entity_id: int,
	is_selected: bool,
	danger_flags: int = -1
) -> void:
	if not probe_observation_mode or _current_lod < 1:
		return
	var indicators: Array[Dictionary] = _probe_survival_indicators(entity_id, danger_flags)
	if indicators.is_empty():
		return
	var visible_count: int = indicators.size() if is_selected else mini(indicators.size(), PROBE_SURVIVAL_MAX_UNSELECTED)
	var badge_size: float = PROBE_SURVIVAL_SELECTED_BADGE_SIZE if is_selected else PROBE_SURVIVAL_UNSELECTED_BADGE_SIZE
	var step: float = badge_size * 2.0 + PROBE_SURVIVAL_BADGE_GAP
	var row_width: float = step * float(visible_count - 1)
	var start_x: float = pos.x - row_width * 0.5
	var baseline_y: float = pos.y - size - (16.0 if is_selected else 12.0)
	for indicator_index: int in range(visible_count):
		var indicator: Dictionary = indicators[indicator_index]
		var badge_pos: Vector2 = Vector2(start_x + step * float(indicator_index), baseline_y)
		_draw_probe_survival_badge(badge_pos, badge_size, indicator, is_selected)


func _draw_probe_survival_badge(
	center: Vector2,
	size: float,
	indicator: Dictionary,
	is_selected: bool
) -> void:
	var color: Color = indicator.get("color", Color.WHITE)
	var alpha: float = PROBE_SURVIVAL_SELECTED_ALPHA if is_selected else PROBE_SURVIVAL_UNSELECTED_ALPHA
	var badge_color: Color = Color(color.r, color.g, color.b, alpha)
	if is_selected:
		draw_circle(center, size + 3.0, Color(color.r, color.g, color.b, PROBE_SURVIVAL_HALO_ALPHA))
	var outline_color: Color = Color(0.05, 0.05, 0.05, 0.88)
	var shape: String = str(indicator.get("shape", "circle"))
	match shape:
		"triangle":
			_draw_triangle_outlined(center, size + 0.2, badge_color)
		"square":
			_draw_square_outlined(center, size + 0.2, badge_color)
		"diamond":
			_draw_diamond_outlined(center, size + 0.2, badge_color)
		_:
			draw_circle(center, size + OUTLINE_WIDTH, outline_color)
			draw_circle(center, size, badge_color)
	if is_selected:
		var label: String = str(indicator.get("label", ""))
		if not label.is_empty():
			var font: Font = ThemeDB.fallback_font
			var label_size: Vector2 = font.get_string_size(
				label,
				HORIZONTAL_ALIGNMENT_LEFT,
				-1,
				PROBE_SURVIVAL_LABEL_FONT_SIZE
			)
			draw_string(
				font,
				center + Vector2(-label_size.x * 0.5, label_size.y * 0.35),
				label,
				HORIZONTAL_ALIGNMENT_LEFT,
				-1,
				PROBE_SURVIVAL_LABEL_FONT_SIZE,
				Color.WHITE
			)


func _probe_survival_indicators(entity_id: int, danger_flags: int = -1) -> Array[Dictionary]:
	var indicators: Array[Dictionary] = []
	var detail: Dictionary = _get_probe_entity_detail(entity_id)
	if detail.is_empty():
		if danger_flags >= 0 and danger_flags & 0b0010 != 0:
			indicators.append(_probe_indicator_spec(
				Locale.ltr("UI_PROBE_NEED_HUNGER_SHORT"),
				PROBE_SURVIVAL_HUNGER_COLOR,
				"circle",
				0.0
			))
		if danger_flags >= 0 and danger_flags & 0b0101 != 0:
			indicators.append(_probe_indicator_spec(
				Locale.ltr("UI_PROBE_NEED_DANGER_SHORT"),
				PROBE_SURVIVAL_SAFETY_COLOR,
				"triangle",
				0.0
			))
		return indicators

	var hunger: float = float(detail.get("need_hunger", 1.0))
	var warmth: float = float(detail.get("need_warmth", 1.0))
	var energy: float = float(detail.get("energy", 1.0))
	var safety: float = float(detail.get("need_safety", 1.0))

	if hunger <= PROBE_SURVIVAL_HUNGER_THRESHOLD:
		indicators.append(_probe_indicator_spec(
			Locale.ltr("UI_PROBE_NEED_HUNGER_SHORT"),
			PROBE_SURVIVAL_HUNGER_COLOR,
			"circle",
			hunger
		))
	if warmth <= PROBE_SURVIVAL_WARMTH_THRESHOLD:
		indicators.append(_probe_indicator_spec(
			Locale.ltr("UI_PROBE_NEED_WARMTH_SHORT"),
			PROBE_SURVIVAL_WARMTH_COLOR,
			"diamond",
			warmth
		))
	if energy <= PROBE_SURVIVAL_ENERGY_THRESHOLD:
		indicators.append(_probe_indicator_spec(
			Locale.ltr("UI_PROBE_NEED_FATIGUE_SHORT"),
			PROBE_SURVIVAL_ENERGY_COLOR,
			"square",
			energy
		))
	if safety <= PROBE_SURVIVAL_SAFETY_THRESHOLD:
		indicators.append(_probe_indicator_spec(
			Locale.ltr("UI_PROBE_NEED_DANGER_SHORT"),
			PROBE_SURVIVAL_SAFETY_COLOR,
			"triangle",
			safety
		))

	indicators.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return float(a.get("severity", 1.0)) < float(b.get("severity", 1.0))
	)
	return indicators


func _probe_indicator_spec(label: String, color: Color, shape: String, severity: float) -> Dictionary:
	return {
		"label": label,
		"color": color,
		"shape": shape,
		"severity": severity,
	}


func _get_probe_entity_detail(entity_id: int) -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_entity_detail") or entity_id < 0:
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _probe_runtime_detail_cache_tick:
		_probe_runtime_detail_cache_tick = tick
		_probe_runtime_detail_cache.clear()
	if not _probe_runtime_detail_cache.has(entity_id):
		_probe_runtime_detail_cache[entity_id] = _sim_engine.get_entity_detail(entity_id)
	return _probe_runtime_detail_cache.get(entity_id, {})


func _get_selected_runtime_detail() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_entity_detail") or selected_entity_id < 0:
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _selected_runtime_detail_cache_tick or selected_entity_id != _selected_runtime_detail_cache_id:
		_selected_runtime_detail_cache_tick = tick
		_selected_runtime_detail_cache_id = selected_entity_id
		_selected_runtime_detail_cache = _get_probe_entity_detail(selected_entity_id)
	return _selected_runtime_detail_cache


func _update_agent_sprites() -> void:
	if not _binary_snapshot_available or not _snapshot_decoder.has_data():
		_hide_agent_sprites()
		return

	if not _ensure_agent_visual_resources():
		_hide_agent_sprites()
		return

	var cam: Camera2D = get_viewport().get_camera_2d()
	var zoom_level: float = cam.zoom.x if cam else 1.0
	_update_lod(zoom_level)
	if _current_lod == 0:
		_hide_agent_sprites()
		return

	_ensure_agent_sprite_capacity(_snapshot_decoder.agent_count)

	var viewport_size: Vector2 = get_viewport_rect().size
	var cam_pos: Vector2 = cam.global_position if cam else Vector2.ZERO
	var half_view: Vector2 = viewport_size / cam.zoom * 0.5 if cam else viewport_size * 0.5
	var min_tile_x: float = float(int((cam_pos.x - half_view.x) / GameConfig.TILE_SIZE) - 2)
	var max_tile_x: float = float(int((cam_pos.x + half_view.x) / GameConfig.TILE_SIZE) + 2)
	var min_tile_y: float = float(int((cam_pos.y - half_view.y) / GameConfig.TILE_SIZE) - 2)
	var max_tile_y: float = float(int((cam_pos.y + half_view.y) / GameConfig.TILE_SIZE) + 2)
	var half_tile: Vector2 = Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)

	for index in range(_snapshot_decoder.agent_count):
		var sprite: Sprite2D = _agent_sprites[index]
		var social_bubble = _agent_bubbles[index]
		var tile_pos: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
		if tile_pos.x < min_tile_x or tile_pos.x > max_tile_x or tile_pos.y < min_tile_y or tile_pos.y > max_tile_y:
			sprite.visible = false
			if social_bubble != null:
				social_bubble.visible = false
			continue

		var entity_id: int = _snapshot_decoder.get_entity_id(index)
		var is_selected: bool = entity_id == selected_entity_id
		var emphasize_probe: bool = probe_observation_mode and selected_entity_id >= 0
		var growth_stage_key: String = _binary_growth_stage_key(_snapshot_decoder.get_growth_stage(index))
		var velocity: Vector2 = _snapshot_decoder.get_velocity(index)
		var action_state: int = _snapshot_decoder.get_action_state(index)
		var movement_dir: int = _resolve_social_facing_direction(index, action_state, _snapshot_decoder.get_movement_dir(index))
		var frame_data: Dictionary = _sprite_frame_data(movement_dir, velocity.length())
		var shader_material: ShaderMaterial = sprite.material as ShaderMaterial

		sprite.position = tile_pos * float(GameConfig.TILE_SIZE) + half_tile
		sprite.scale = Vector2.ONE * float(AGE_SIZE_MULT.get(growth_stage_key, 1.0))
		sprite.flip_h = bool(frame_data.get("flip_h", false))
		sprite.frame = int(frame_data.get("frame", 0))
		sprite.modulate = Color(1.0, 1.0, 1.0, 1.0 if (not emphasize_probe or is_selected) else 0.58)
		sprite.z_index = 2 if is_selected else 1
		sprite.visible = true

		_apply_palette(shader_material, _snapshot_decoder.get_sprite_var(index))
		_apply_stress(
			shader_material,
			_snapshot_decoder.get_stress_phase(index),
			_snapshot_decoder.get_mood_color(index),
			_snapshot_decoder.get_active_break(index) > 0
		)
		_apply_breathing(shader_material, entity_id, velocity.length())
		if social_bubble != null:
			social_bubble.update_state(action_state)
			if emphasize_probe and not is_selected:
				social_bubble.visible = false

	for index in range(_snapshot_decoder.agent_count, _agent_sprites.size()):
		_agent_sprites[index].visible = false
		if index < _agent_bubbles.size() and _agent_bubbles[index] != null:
			_agent_bubbles[index].visible = false


func _ensure_agent_sprite_capacity(required: int) -> void:
	while _agent_sprites.size() < required:
		var sprite: Sprite2D = Sprite2D.new()
		var sprite_material: ShaderMaterial = ShaderMaterial.new()
		sprite_material.resource_local_to_scene = true
		sprite_material.shader = _agent_visual_shader
		sprite_material.set_shader_parameter("palette_lut", _agent_palette_lut)
		sprite.texture = _agent_texture
		sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		sprite.centered = true
		sprite.offset = AGENT_TEXTURE_OFFSET
		sprite.hframes = AGENT_FRAME_COLUMNS
		sprite.vframes = AGENT_FRAME_ROWS
		sprite.material = sprite_material
		sprite.visible = false
		sprite.z_index = 1
		var social_bubble = SocialBubbleScene.instantiate()
		sprite.add_child(social_bubble)
		add_child(sprite)
		_agent_sprites.append(sprite)
		_agent_bubbles.append(social_bubble)


func _ensure_agent_visual_resources() -> bool:
	if _agent_texture == null:
		_agent_texture = _load_texture_from_png(AGENT_TEXTURE_PATH)
	if _agent_palette_lut == null:
		_agent_palette_lut = _load_texture_from_png(AGENT_PALETTE_LUT_PATH)
	if _agent_visual_shader == null:
		var shader_resource: Variant = load(AGENT_VISUAL_SHADER_PATH)
		if shader_resource is Shader:
			_agent_visual_shader = shader_resource
	return _agent_texture != null and _agent_palette_lut != null and _agent_visual_shader != null


func _load_texture_from_png(resource_path: String) -> Texture2D:
	var image: Image = Image.new()
	var err: Error = image.load(ProjectSettings.globalize_path(resource_path))
	if err != OK:
		return null
	return ImageTexture.create_from_image(image)


func _hide_agent_sprites() -> void:
	for sprite in _agent_sprites:
		sprite.visible = false
	for bubble in _agent_bubbles:
		if bubble != null:
			bubble.visible = false


func _ensure_relationship_overlay() -> void:
	if _relationship_overlay != null or _sim_engine == null:
		return
	_relationship_overlay = RelationshipOverlayClass.new()
	_relationship_overlay.init(_sim_engine)
	add_child(_relationship_overlay)


func _resolve_social_facing_direction(index: int, action_state: int, default_dir: int) -> int:
	var partner_index: int = _find_social_partner_index(index, action_state)
	if partner_index < 0:
		return default_dir
	var origin: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
	var partner_pos: Vector2 = _snapshot_decoder.get_interpolated_position(partner_index, _render_alpha)
	var direction: Vector2 = partner_pos - origin
	if direction.length_squared() <= 0.0001:
		return default_dir
	return _movement_dir_from_vector(direction)


func _find_social_partner_index(index: int, action_state: int) -> int:
	var wants_conflict: bool = _is_conflict_visual_action(action_state)
	var wants_social: bool = _is_social_visual_action(action_state)
	if not wants_conflict and not wants_social:
		return -1
	var origin: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
	var best_index: int = -1
	var best_distance_sq: float = SOCIAL_INTERACTION_MAX_DISTANCE * SOCIAL_INTERACTION_MAX_DISTANCE
	for other_index: int in range(_snapshot_decoder.agent_count):
		if other_index == index:
			continue
		var other_action: int = _snapshot_decoder.get_action_state(other_index)
		if wants_conflict and not _is_conflict_visual_action(other_action):
			continue
		if wants_social and not _is_social_visual_action(other_action):
			continue
		var other_pos: Vector2 = _snapshot_decoder.get_interpolated_position(other_index, _render_alpha)
		var distance_sq: float = origin.distance_squared_to(other_pos)
		if distance_sq < best_distance_sq:
			best_distance_sq = distance_sq
			best_index = other_index
	return best_index


func _is_social_visual_action(action_state: int) -> bool:
	return action_state == ACTION_SOCIALIZE \
		or action_state == ACTION_TEACH \
		or action_state == ACTION_LEARN \
		or action_state == ACTION_VISIT_PARTNER


func _is_conflict_visual_action(action_state: int) -> bool:
	return action_state == ACTION_FIGHT


func _movement_dir_from_vector(direction: Vector2) -> int:
	if direction.length_squared() <= 0.0001:
		return 0
	var angle: float = atan2(direction.y, direction.x)
	var octant: int = int(round(angle / (PI / 4.0)))
	return posmod(octant, 8)


func _sprite_frame_data(movement_dir: int, speed: float) -> Dictionary:
	var frame_column: int = 0
	if speed >= 0.05:
		frame_column = int(Time.get_ticks_msec() / float(AGENT_FRAME_TIME_MS)) % AGENT_FRAME_COLUMNS

	var frame_row: int = 1
	var flip_h: bool = false
	match movement_dir:
		1, 2, 3:
			frame_row = 2
			flip_h = movement_dir == 3
		4:
			frame_row = 1
			flip_h = true
		5, 6, 7:
			frame_row = 0
			flip_h = movement_dir == 5
		_:
			frame_row = 1
			flip_h = false

	return {
		"frame": frame_row * AGENT_FRAME_COLUMNS + frame_column,
		"flip_h": flip_h,
	}


func _apply_palette(shader_material: ShaderMaterial, sprite_var: int) -> void:
	var hair_index: int = (sprite_var >> 5) & 0x07
	var body_index: int = (sprite_var >> 3) & 0x03
	var skin_index: int = sprite_var & 0x07
	shader_material.set_shader_parameter("hair_index", float(hair_index))
	shader_material.set_shader_parameter("body_index", float(body_index))
	shader_material.set_shader_parameter("skin_index", float(skin_index))


func _apply_stress(shader_material: ShaderMaterial, stress_phase: int, mood_color_index: int, is_break: bool) -> void:
	var clamped_stress: int = clampi(stress_phase, 0, STRESS_OUTLINE_COLORS.size() - 1)
	var clamped_mood: int = clampi(mood_color_index, 0, MOOD_COLORS.size() - 1)
	shader_material.set_shader_parameter("stress_phase", clamped_stress)
	shader_material.set_shader_parameter("mood_color", MOOD_COLORS[clamped_mood])
	shader_material.set_shader_parameter("outline_color", STRESS_OUTLINE_COLORS[clamped_stress])
	shader_material.set_shader_parameter("active_break", is_break)


func _apply_breathing(shader_material: ShaderMaterial, entity_id: int, current_speed: float) -> void:
	var normalized_speed: float = clampf(current_speed / AGENT_BASE_SPEED, 0.0, 2.0)
	shader_material.set_shader_parameter("agent_offset", float(entity_id) * 0.37)
	shader_material.set_shader_parameter("speed_factor", normalized_speed)


func _binary_job_key(job_icon: int) -> String:
	return str(JOB_ICON_TO_KEY.get(job_icon, "none"))


func _binary_growth_stage_key(stage_code: int) -> String:
	if stage_code >= 0 and stage_code < GROWTH_STAGE_KEYS.size():
		return GROWTH_STAGE_KEYS[stage_code]
	return "adult"


func _runtime_entity_name(entity_id: int) -> String:
	if _sim_engine == null or not _sim_engine.has_method("get_entity_detail"):
		return ""
	var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
	if detail.is_empty():
		return ""
	return str(detail.get("name", ""))


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
	var best_entity_id: int = -1
	var best_dist: float = 3.0  # max click distance in tiles
	if _binary_snapshot_available:
		for index in range(_snapshot_decoder.agent_count):
			var entity_pos: Vector2 = _snapshot_decoder.get_interpolated_position(index, _render_alpha)
			var dist: float = entity_pos.distance_to(tile_pos)
			if dist < best_dist:
				best_dist = dist
				best_entity_id = _snapshot_decoder.get_entity_id(index)
	else:
		var alive: Array = _get_legacy_snapshots()
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
