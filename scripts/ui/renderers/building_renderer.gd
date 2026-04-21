extends Node2D

var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _sim_engine: RefCounted
var _current_lod: int = 1
var _building_textures: Dictionary = {}
## Caches `_get_variant_count()` results so filesystem directory scans
## (FileAccess.file_exists × up to 17 per type) only happen once per type
## rather than on every _draw() call, keeping FPS ≥ 55 when sprites are live.
var _variant_count_cache: Dictionary = {}
var _runtime_minimap_cache: Dictionary = {}
var _runtime_minimap_cache_tick: int = -1
var _runtime_world_summary_cache: Dictionary = {}
var _runtime_world_summary_cache_tick: int = -1
var _last_redraw_cam_pos: Vector2 = Vector2.INF
var _last_redraw_zoom: float = -1.0
var _last_redraw_tick: int = -1
var _redraw_cooldown: float = 0.0
var _tile_grid_cache: Dictionary = {}
var _tile_grid_cache_tick: int = -1


func init(building_manager: RefCounted, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine


func _process(delta: float) -> void:
	_redraw_cooldown = maxf(0.0, _redraw_cooldown - delta)
	var cam: Camera2D = get_viewport().get_camera_2d()
	if cam == null:
		return
	var cam_pos: Vector2 = cam.global_position
	var cam_zoom: float = cam.zoom.x
	var runtime_tick: int = -1
	if _sim_engine != null:
		runtime_tick = int(_sim_engine.get("current_tick"))
	var camera_changed: bool = (
		_last_redraw_cam_pos == Vector2.INF
		or cam_pos.distance_to(_last_redraw_cam_pos) > 8.0
		or absf(cam_zoom - _last_redraw_zoom) > 0.01
	)
	var data_changed: bool = runtime_tick != _last_redraw_tick
	if _redraw_cooldown > 0.0 and not data_changed:
		return
	if camera_changed or data_changed:
		_last_redraw_cam_pos = cam_pos
		_last_redraw_zoom = cam_zoom
		_last_redraw_tick = runtime_tick
		_redraw_cooldown = 0.1
		queue_redraw()


func force_redraw() -> void:
	_redraw_cooldown = 0.0
	_last_redraw_cam_pos = Vector2.INF
	queue_redraw()


func _draw() -> void:
	if _building_manager == null and _sim_engine == null:
		return
	var cam := get_viewport().get_camera_2d()
	var zl: float = cam.zoom.x if cam else 1.0
	_update_lod(zl)

	# === DEBUG: one-shot building/wall position log ===
	if _sim_engine != null:
		var tick: int = int(_sim_engine.get("current_tick"))
		if tick % 500 == 0 and tick > 0:
			var dbg_buildings: Array = _get_runtime_buildings()
			for b in dbg_buildings:
				print("[BUILDING] type=", b.get("building_type", ""), " x=", b.get("tile_x", 0), " y=", b.get("tile_y", 0), " w=", b.get("width", 0), " h=", b.get("height", 0))
			var dbg_data: Dictionary = _get_tile_grid_data()
			var wxs: PackedInt32Array = dbg_data.get("wall_x", PackedInt32Array())
			var wys: PackedInt32Array = dbg_data.get("wall_y", PackedInt32Array())
			if wxs.size() > 0:
				print("[WALLS] count=", wxs.size(), " first=(", wxs[0], ",", wys[0], ") last=(", wxs[wxs.size()-1], ",", wys[wys.size()-1], ")")

	# Viewport culling
	var viewport_size := get_viewport_rect().size
	var cam_pos := cam.global_position if cam else Vector2.ZERO
	var half_view := viewport_size / cam.zoom * 0.5 if cam else viewport_size * 0.5
	var min_tile_x: int = int((cam_pos.x - half_view.x) / GameConfig.TILE_SIZE) - 2
	var max_tile_x: int = int((cam_pos.x + half_view.x) / GameConfig.TILE_SIZE) + 2
	var min_tile_y: int = int((cam_pos.y - half_view.y) / GameConfig.TILE_SIZE) - 2
	var max_tile_y: int = int((cam_pos.y + half_view.y) / GameConfig.TILE_SIZE) + 2

	var buildings: Array = []
	if _building_manager != null:
		buildings = _building_manager.get_all_buildings()
	if buildings.is_empty():
		buildings = _get_runtime_buildings()
	var tile_size: int = GameConfig.TILE_SIZE
	var half: float = tile_size * 0.5
	var font: Font = ThemeDB.fallback_font
	var font_size: int = 10

	# === Tile grid wall/floor/furniture rendering (P2-B3.5) ===
	if _sim_engine != null:
		_draw_tile_grid_walls(tile_size, min_tile_x, max_tile_x, min_tile_y, max_tile_y)

	for i in range(buildings.size()):
		var b = buildings[i]

		# Viewport culling
		var tile_x: int = int(_building_value(b, "tile_x", 0))
		var tile_y: int = int(_building_value(b, "tile_y", 0))
		var building_type: String = str(_building_value(b, "building_type", ""))
		var is_built: bool = bool(_building_value(b, "is_built", _building_value(b, "is_constructed", false)))
		var build_progress: float = float(_building_value(b, "build_progress", _building_value(b, "construction_progress", 0.0)))

		if tile_x < min_tile_x or tile_x > max_tile_x:
			continue
		if tile_y < min_tile_y or tile_y > max_tile_y:
			continue

		# Skip legacy building sprite when tile_grid has data for this building
		if _has_tile_grid_data_at(tile_x, tile_y):
			continue

		var cx: float = float(tile_x) * tile_size + half
		var cy: float = float(tile_y) * tile_size + half
		var alpha: float = 1.0 if is_built else 0.4

		if _current_lod >= GameConfig.ZOOM_Z3:
			if zl < 0.4:
				continue
			var strategic_color: Color = Color(0.6, 0.35, 0.15, alpha)
			match building_type:
				"stockpile":
					strategic_color = Color(0.6, 0.35, 0.15, alpha)
				"shelter":
					strategic_color = Color(0.9, 0.55, 0.2, alpha)
				"campfire":
					strategic_color = Color(0.9, 0.2, 0.15, alpha)
			draw_rect(Rect2(cx - 1.5, cy - 1.5, 3.0, 3.0), strategic_color, true)
			continue

		# Use building id as variant seed for deterministic per-building variant.
		var building_id: int = int(_building_value(b, "id", tile_x * 1000 + tile_y))
		_draw_building_sprite(building_type, building_id, cx, cy, alpha, tile_size, zl)
		_draw_building_interior(b, tile_x, tile_y, tile_size, zl)

		# Construction progress bar
		if not is_built:
			var building_size: float = tile_size * 0.8
			var bar_w: float = building_size
			var bar_h: float = 3.0
			var bar_x: float = cx - bar_w * 0.5
			var bar_y: float = cy + building_size * 0.5 + 2.0
			draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.6))
			draw_rect(Rect2(bar_x, bar_y, bar_w * build_progress, bar_h), Color(0.2, 0.8, 0.2, 0.8))

		if _current_lod == GameConfig.ZOOM_Z1 and building_type == "stockpile" and is_built:
			var storage: Dictionary = {}
			var storage_raw: Variant = _building_value(b, "storage", {})
			if storage_raw is Dictionary:
				storage = storage_raw
			var food: int = int(round(storage.get("food", 0.0)))
			var wood: int = int(round(storage.get("wood", 0.0)))
			var stone: int = int(round(storage.get("stone", 0.0)))
			var text: String = Locale.trf3("UI_STATS_RESOURCES_FMT", "food", food, "wood", wood, "stone", stone)
			draw_string(font, Vector2(cx - 20, cy + half + 14), text, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size, Color.WHITE)
		elif _current_lod <= GameConfig.ZOOM_Z2:
			var building_label: String = Locale.tr_id("BUILDING", building_type)
			if building_label.is_empty() or building_label == building_type:
				building_label = Locale.tr_id("BUILDING_TYPE", building_type)
			if building_label.is_empty() or building_label == building_type:
				building_label = building_type.capitalize()
			draw_string(
				font,
				Vector2(cx, cy - (tile_size * 0.8) - 4.0),
				building_label,
				HORIZONTAL_ALIGNMENT_CENTER,
				64.0,
				9,
				Color(0.95, 0.84, 0.58, 0.92)
			)
			# Worker count under building name (safe — defaults to 0 if not in data)
			var worker_count: int = int(_building_value(b, "worker_count", _building_value(b, "assigned_workers", 0)))
			if worker_count > 0:
				draw_string(
					font,
					Vector2(cx, cy - (tile_size * 0.8) + 8.0),
					"%d" % worker_count,
					HORIZONTAL_ALIGNMENT_CENTER,
					64.0,
					7,
					Color(0.6, 0.7, 0.6, 0.6)
				)

	# Settlement labels at settlement-scale zoom
	if _current_lod >= GameConfig.ZOOM_Z3 and zl >= 0.4 and zl < 0.8:
		var settlements: Array = _get_runtime_settlements()
		if settlements.is_empty() and _settlement_manager != null:
			settlements = _settlement_manager.get_active_settlements()
		for i in range(settlements.size()):
			var s: Variant = settlements[i]
			var sx: float = float(_settlement_value(s, "center_x", 0)) * tile_size + half
			var sy: float = float(_settlement_value(s, "center_y", 0)) * tile_size + half
			var sid: int = int(_settlement_value(s, "id", 0))
			var pop: int = int(_settlement_value(s, "population", _settlement_member_count(s)))
			var label: String = Locale.trf2("UI_SETTLEMENT_LABEL_FMT", "id", sid, "pop", pop)
			draw_string(font, Vector2(sx - 15, sy - 8), label, HORIZONTAL_ALIGNMENT_CENTER, -1, 10, Color(1, 1, 0.6, 0.9))


func _update_lod(zl: float) -> void:
	_current_lod = _compute_zoom_tier(zl)


static func _compute_zoom_tier(zoom_value: float) -> int:
	for i in range(GameConfig.ZOOM_TIER_BOUNDARIES.size()):
		if zoom_value >= GameConfig.ZOOM_TIER_BOUNDARIES[i]:
			return i
	return GameConfig.ZOOM_TIER_COUNT - 1


func _get_variant_count(variant_dir: String) -> int:
	if _variant_count_cache.has(variant_dir):
		return _variant_count_cache[variant_dir]
	if not DirAccess.dir_exists_absolute(variant_dir):
		_variant_count_cache[variant_dir] = 0
		return 0
	var count: int = 0
	for i in range(1, 100):
		if not FileAccess.file_exists("%s/%d.png" % [variant_dir, i]):
			break
		count += 1
	_variant_count_cache[variant_dir] = count
	return count


## Picks a zero-based variant index for a given entity.
##
## Contract (harness plan G1/G2):
##   - Returns an integer in the range [0, variant_count - 1] when
##     variant_count > 0.
##   - Returns 0 when variant_count <= 0 (degenerate fallback — callers
##     are responsible for checking variant_count before invoking any path
##     resolver that assumes a valid index).
##   - Deterministic for a given (entity_id, variant_count) pair.
##   - Produces >= 3 distinct indices when fed 100 consecutive ids and
##     variant_count = 5 (no constant-bypass regressions).
##
## The picker stays zero-based so callers can uniformly rely on 0-indexed
## semantics; the +1 conversion to 1-based filenames (1.png, 2.png, ...)
## happens exclusively inside `building_variant_path()` /
## `furniture_variant_path()`.
func _pick_variant_for_entity(entity_id: int, variant_count: int) -> int:
	if variant_count <= 0:
		return 0
	# posmod keeps the result non-negative even for negative entity_ids.
	return posmod(entity_id, variant_count)


## Picks a zero-based variant index for a given tile position.
## See `_pick_variant_for_entity` contract — same [0, variant_count-1] range.
func _pick_variant_for_tile(tile_x: int, tile_y: int, variant_count: int) -> int:
	if variant_count <= 0:
		return 0
	return posmod((tile_x * 31) + (tile_y * 17), variant_count)


func _deterministic_seed_for_tile(tx: int, ty: int) -> int:
	return (tx * 31) + (ty * 17)


# Pure path resolvers — the sole authority on sprite path construction.
# Also invoked by the sim-test harness (A14) via source-level contract.
# Changing these signatures or strings REQUIRES a matching harness update.
static func building_variant_dir(building_type: String) -> String:
	if building_type.is_empty():
		return ""
	return "res://assets/sprites/buildings/" + building_type


## Resolves a variant filename from a zero-based variant index.
## Callers pass the picker's 0-based output (G1 contract); the +1 conversion
## to the on-disk 1-based filename convention (1.png, 2.png, ...) happens
## here — never on the caller side.
static func building_variant_path(building_type: String, variant_idx: int) -> String:
	if building_type.is_empty() or variant_idx < 0:
		return ""
	return "%s/%d.png" % [building_variant_dir(building_type), variant_idx + 1]


static func building_legacy_path(building_type: String) -> String:
	if building_type.is_empty():
		return ""
	return "res://assets/sprites/buildings/" + building_type + ".png"


static func furniture_variant_dir(furniture_id: String) -> String:
	if furniture_id.is_empty():
		return ""
	return "res://assets/sprites/furniture/" + furniture_id


## Resolves a furniture variant filename from a zero-based variant index.
## See `building_variant_path` — same 0-based-in / 1-based-filename rule.
static func furniture_variant_path(furniture_id: String, variant_idx: int) -> String:
	if furniture_id.is_empty() or variant_idx < 0:
		return ""
	return "%s/%d.png" % [furniture_variant_dir(furniture_id), variant_idx + 1]


func _load_building_texture(building_type: String, entity_id: int = 0) -> Texture2D:
	# Try variant folder first
	var variant_dir: String = building_variant_dir(building_type)
	if variant_dir.is_empty():
		return null
	var variant_count: int = _get_variant_count(ProjectSettings.globalize_path(variant_dir))
	if variant_count > 0:
		var variant_idx: int = _pick_variant_for_entity(entity_id, variant_count)
		var cache_key: String = "%s/%d" % [building_type, variant_idx]
		if _building_textures.has(cache_key):
			return _building_textures[cache_key]
		var variant_path: String = building_variant_path(building_type, variant_idx)
		if FileAccess.file_exists(variant_path):
			var tex: Texture2D = _load_texture_from_res_path(variant_path)
			_building_textures[cache_key] = tex
			return tex
		_building_textures[cache_key] = null
		return null

	# Legacy flat file fallback
	if _building_textures.has(building_type):
		return _building_textures[building_type]
	var path: String = building_legacy_path(building_type)
	if not FileAccess.file_exists(path):
		_building_textures[building_type] = null
		return null
	var tex: Texture2D = load(path) as Texture2D
	if tex == null:
		_building_textures[building_type] = null
		return null
	_building_textures[building_type] = tex
	return tex


func _load_furniture_texture(furniture_id: String, seed_value: int = 0) -> Texture2D:
	var variant_dir: String = furniture_variant_dir(furniture_id)
	if variant_dir.is_empty():
		return null
	var variant_count: int = _get_variant_count(ProjectSettings.globalize_path(variant_dir))
	if variant_count <= 0:
		return null
	var variant_idx: int = _pick_variant_for_entity(seed_value, variant_count)
	var cache_key: String = "furniture/%s/%d" % [furniture_id, variant_idx]
	if _building_textures.has(cache_key):
		return _building_textures[cache_key]
	var variant_path: String = furniture_variant_path(furniture_id, variant_idx)
	if FileAccess.file_exists(variant_path):
		var tex: Texture2D = _load_texture_from_res_path(variant_path)
		_building_textures[cache_key] = tex
		return tex
	_building_textures[cache_key] = null
	return null


## Loads a wall material sprite for the given tile position.
## Variant selection is deterministic via _pick_variant_for_tile (prime-multiplier hash).
## Cache key: "wall_mat/{material_id}/{variant_idx}".
## Returns null if no sprite folder exists — caller falls back to solid fill.
func _load_wall_material_texture(material_id: String, tile_x: int, tile_y: int) -> Texture2D:
	if material_id.is_empty():
		return null
	var variant_dir_res: String = "res://assets/sprites/walls/" + material_id
	var variant_count: int = _get_variant_count(ProjectSettings.globalize_path(variant_dir_res))
	if variant_count <= 0:
		return null
	var variant_idx: int = _pick_variant_for_tile(tile_x, tile_y, variant_count)
	var cache_key: String = "wall_mat/%s/%d" % [material_id, variant_idx]
	if _building_textures.has(cache_key):
		return _building_textures[cache_key]
	var variant_path: String = "%s/%d.png" % [variant_dir_res, variant_idx + 1]
	var tex: Texture2D = _load_texture_from_res_path(variant_path)
	_building_textures[cache_key] = tex  # cache null too — avoids repeated filesystem scans
	return tex


## Loads a floor material sprite for the given tile position.
## Mirror of _load_wall_material_texture for floors.
## Returns null if no sprite folder exists — caller falls back to solid fill.
func _load_floor_material_texture(material_id: String, tile_x: int, tile_y: int) -> Texture2D:
	if material_id.is_empty():
		return null
	var variant_dir_res: String = "res://assets/sprites/floors/" + material_id
	var variant_count: int = _get_variant_count(ProjectSettings.globalize_path(variant_dir_res))
	if variant_count <= 0:
		return null
	var variant_idx: int = _pick_variant_for_tile(tile_x, tile_y, variant_count)
	var cache_key: String = "floor_mat/%s/%d" % [material_id, variant_idx]
	if _building_textures.has(cache_key):
		return _building_textures[cache_key]
	var variant_path: String = "%s/%d.png" % [variant_dir_res, variant_idx + 1]
	var tex: Texture2D = _load_texture_from_res_path(variant_path)
	_building_textures[cache_key] = tex
	return tex


## Loads a Texture2D from a res:// path with a two-stage strategy:
##   1. ResourceLoader (fast path — works when a .import sidecar exists).
##   2. Image.load_from_file (fallback — works for raw PNGs without .import,
##      e.g. all 144 Round-1 variant sprites in headless harness runs).
##
## This function is the ONLY place that calls load() or Image.load_from_file
## for variant/legacy sprites. Both _load_building_texture and
## _load_furniture_texture delegate here so the fallback is applied uniformly.
func _load_texture_from_res_path(res_path: String) -> Texture2D:
	if res_path.is_empty():
		return null
	# Stage 1: ResourceLoader — avoids console spam when import is missing.
	if ResourceLoader.exists(res_path, "Texture2D"):
		var tex: Texture2D = ResourceLoader.load(res_path, "Texture2D") as Texture2D
		if tex != null:
			return tex
	# Stage 2: Raw PNG load via Image API (no .import sidecar required).
	var abs_path: String = ProjectSettings.globalize_path(res_path)
	var img: Image = Image.load_from_file(abs_path)
	if img != null:
		return ImageTexture.create_from_image(img)
	return null


func _draw_building_sprite(building_type: String, entity_id: int, cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var tex: Texture2D = _load_building_texture(building_type, entity_id)
	if tex == null:
		match building_type:
			"campfire":
				_draw_campfire_fallback(cx, cy, alpha, tile_size, zoom_level)
			"shelter":
				_draw_shelter_fallback(cx, cy, alpha, tile_size, zoom_level)
			"stockpile":
				_draw_stockpile_fallback(cx, cy, alpha, tile_size, zoom_level)
		return

	var scale_factor: float = float(tile_size) * 2.0 / 32.0 * _zoom_shape_scale(zoom_level)
	var draw_size: Vector2 = Vector2(32.0, 32.0) * scale_factor
	var draw_pos: Vector2 = Vector2(cx - draw_size.x * 0.5, cy - draw_size.y * 0.5)
	var tex_color: Color = Color(1.0, 1.0, 1.0, alpha)
	draw_texture_rect(tex, Rect2(draw_pos, draw_size), false, tex_color)


func _draw_stockpile_fallback(cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var size: float = tile_size * 0.8 * _zoom_shape_scale(zoom_level)
	var half_size: float = size * 0.5
	var fill_color := Color(0.55, 0.35, 0.15, alpha)
	var outline_color := Color(0.9, 0.7, 0.3, alpha)

	draw_rect(Rect2(cx - half_size, cy - half_size, size, size), fill_color, true)
	draw_rect(Rect2(cx - half_size, cy - half_size, size, size), outline_color, false, 2.0)


func _draw_shelter_fallback(cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var size: float = tile_size * 0.8 * _zoom_shape_scale(zoom_level)
	var half_size: float = size * 0.5
	var fill_color := Color(0.7, 0.4, 0.2, alpha)
	var outline_color := Color(1.0, 0.8, 0.4, alpha)

	var points := PackedVector2Array([
		Vector2(cx, cy - half_size),
		Vector2(cx - half_size, cy + half_size),
		Vector2(cx + half_size, cy + half_size),
	])

	draw_colored_polygon(points, fill_color)
	draw_polyline(PackedVector2Array([points[0], points[1], points[2], points[0]]), outline_color, 2.0)


func _draw_campfire_fallback(cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var size: float = tile_size * 0.8 * _zoom_shape_scale(zoom_level)
	var radius: float = size * 0.35
	var fill_color := Color(1.0, 0.4, 0.1, alpha)
	var glow_color := Color(1.0, 0.4, 0.1, alpha * 0.15)

	draw_circle(Vector2(cx, cy), radius, fill_color)
	draw_arc(Vector2(cx, cy), tile_size * 3.0, 0, TAU, 32, glow_color, 1.5)


func _zoom_shape_scale(zoom_level: float) -> float:
	return clampf(2.5 / maxf(zoom_level, 0.5), 0.9, 2.2)


func _draw_building_interior(building: Variant, tile_x: int, tile_y: int, tile_size: int, zoom_level: float) -> void:
	if zoom_level < 2.0:
		return
	var building_type: String = str(_building_value(building, "building_type", ""))
	var dimensions: Vector2i = _building_dimensions(building_type, building)
	var width_tiles: int = maxi(1, dimensions.x)
	var height_tiles: int = maxi(1, dimensions.y)
	var px: float = float(tile_x) * tile_size
	var py: float = float(tile_y) * tile_size
	var width_px: float = float(width_tiles * tile_size)
	var height_px: float = float(height_tiles * tile_size)

	if building_type not in ["campfire", "stockpile"]:
		draw_rect(Rect2(px, py, width_px, height_px), Color(0.10, 0.08, 0.03, 0.32), true)

	if building_type == "workshop":
		var wall_color := Color(0.35, 0.29, 0.16, 0.8)
		var wall_width: float = maxf(2.0, tile_size * 0.18)
		var door_left: float = px + width_px * 0.5 - tile_size * 0.4
		var door_right: float = door_left + tile_size * 0.8
		draw_line(Vector2(px, py), Vector2(px + width_px, py), wall_color, wall_width)
		draw_line(Vector2(px, py), Vector2(px, py + height_px), wall_color, wall_width)
		draw_line(Vector2(px + width_px, py), Vector2(px + width_px, py + height_px), wall_color, wall_width)
		draw_line(Vector2(px, py + height_px), Vector2(door_left, py + height_px), wall_color, wall_width)
		draw_line(Vector2(door_right, py + height_px), Vector2(px + width_px, py + height_px), wall_color, wall_width)
		draw_line(
			Vector2(door_left + tile_size * 0.1, py + height_px),
			Vector2(door_right - tile_size * 0.1, py + height_px),
			Color(0.41, 0.28, 0.09, 0.85),
			maxf(1.0, wall_width * 0.5)
		)

	var font: Font = ThemeDB.fallback_font
	var icon_size: int = maxi(7, int(tile_size * 0.55))
	match building_type:
		"stockpile":
			_draw_furniture_icon(font, px + tile_size * 1.0, py + tile_size * 1.1, "📦", icon_size)
			_draw_furniture_icon(font, px + tile_size * 2.0, py + tile_size * 1.1, "📦", icon_size)
		"shelter":
			for fy: int in range(mini(height_tiles, 2)):
				for fx: int in range(mini(width_tiles, 3)):
					_draw_furniture_icon(
						font,
						px + (float(fx) + 0.5) * tile_size,
						py + (float(fy) + 1.2) * tile_size,
						"🛏️",
						icon_size
					)
		"campfire":
			_draw_furniture_icon(font, px + width_px * 0.5, py + height_px * 0.5 + tile_size * 0.15, "🔥", maxi(9, int(tile_size * 0.7)))
		"workshop":
			_draw_furniture_icon(font, px + tile_size * 1.0, py + tile_size * 1.1, "🪓", icon_size)
			_draw_furniture_icon(font, px + tile_size * 2.0, py + tile_size * 1.1, "⚒️", icon_size)
		_:
			pass


func _draw_furniture_icon(font: Font, x: float, y: float, icon: String, size: int) -> void:
	draw_string(font, Vector2(x, y), icon, HORIZONTAL_ALIGNMENT_CENTER, -1, size, Color(1.0, 1.0, 1.0, 0.92))


func _building_dimensions(building_type: String, building: Variant) -> Vector2i:
	var width_tiles: int = int(_building_value(building, "width", _building_value(building, "tile_w", 0)))
	var height_tiles: int = int(_building_value(building, "height", _building_value(building, "tile_h", 0)))
	if width_tiles > 0 and height_tiles > 0:
		return Vector2i(width_tiles, height_tiles)
	match building_type:
		"campfire":
			return Vector2i(1, 1)
		"shelter":
			return Vector2i(3, 2)
		"stockpile", "workshop":
			return Vector2i(3, 3)
		_:
			return Vector2i(2, 2)


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _settlement_value(settlement: Variant, key: String, default_value: Variant) -> Variant:
	if settlement is Dictionary:
		return settlement.get(key, default_value)
	if settlement == null:
		return default_value
	return settlement.get(key)


func _settlement_member_count(settlement: Variant) -> int:
	if settlement is Dictionary:
		var member_ids: Variant = settlement.get("member_ids", [])
		if member_ids is Array:
			return member_ids.size()
		return int(settlement.get("population", 0))
	if settlement == null:
		return 0
	return settlement.member_ids.size()


func _get_runtime_minimap_snapshot() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_minimap_snapshot"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _runtime_minimap_cache_tick:
		_runtime_minimap_cache_tick = tick
		_runtime_minimap_cache = _sim_engine.get_minimap_snapshot()
	return _runtime_minimap_cache


func _get_runtime_buildings() -> Array:
	var snapshot: Dictionary = _get_runtime_minimap_snapshot()
	var buildings: Variant = snapshot.get("buildings", [])
	return buildings if buildings is Array else []


func _get_runtime_settlements() -> Array:
	var summary: Dictionary = _get_runtime_world_summary()
	var settlement_summaries: Variant = summary.get("settlement_summaries", [])
	if settlement_summaries is Array:
		var settlements: Array = []
		for settlement_summary_raw: Variant in settlement_summaries:
			if not (settlement_summary_raw is Dictionary):
				continue
			var settlement_summary: Dictionary = settlement_summary_raw
			var settlement_raw: Variant = settlement_summary.get("settlement", {})
			if settlement_raw is Dictionary:
				settlements.append(settlement_raw)
		if not settlements.is_empty():
			return settlements
	var snapshot: Dictionary = _get_runtime_minimap_snapshot()
	var fallback_settlements: Variant = snapshot.get("settlements", [])
	return fallback_settlements if fallback_settlements is Array else []


func _get_runtime_world_summary() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_world_summary"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _runtime_world_summary_cache_tick:
		_runtime_world_summary_cache_tick = tick
		_runtime_world_summary_cache = _sim_engine.get_world_summary()
	return _runtime_world_summary_cache


# === Tile grid wall/floor/furniture rendering (P2-B3.5) ===

func _get_tile_grid_data() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_tile_grid_walls"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _tile_grid_cache_tick:
		_tile_grid_cache_tick = tick
		_tile_grid_cache = _sim_engine.get_tile_grid_walls()
	return _tile_grid_cache


func _draw_tile_grid_walls(tile_size: int, min_x: int, max_x: int, min_y: int, max_y: int) -> void:
	var data: Dictionary = _get_tile_grid_data()
	if data.is_empty():
		return

	var ts: float = float(tile_size)

	# Render config — Rust-authoritative values via tile_grid_walls() bridge output
	var r_floor_alpha: float = float(data.get("render_floor_alpha", 0.55))
	var r_floor_border: float = float(data.get("render_floor_border_width", 0.5))
	var r_icon_scale: float = float(data.get("render_furniture_icon_scale", 0.7))
	var r_autotile: bool = bool(data.get("render_wall_autotile", true))
	var r_bridge_px: float = float(data.get("render_wall_bridge_px", 2.0))

	# Draw floors first (under walls)
	# floor_material is tracked in Rust but not yet exported as a parallel PackedStringArray —
	# all tiles default to "packed_earth" until a sim-bridge export is added (Feature 3.5).
	var floor_xs: PackedInt32Array = data.get("floor_x", PackedInt32Array())
	var floor_ys: PackedInt32Array = data.get("floor_y", PackedInt32Array())
	for i in range(floor_xs.size()):
		var fx: int = floor_xs[i]
		var fy: int = floor_ys[i]
		if fx < min_x or fx > max_x or fy < min_y or fy > max_y:
			continue
		var floor_tex: Texture2D = _load_floor_material_texture("packed_earth", fx, fy)
		if floor_tex != null:
			draw_texture_rect(floor_tex, Rect2(float(fx) * ts, float(fy) * ts, ts, ts), false)
		else:
			draw_rect(Rect2(float(fx) * ts, float(fy) * ts, ts, ts), Color(0.30, 0.25, 0.15, r_floor_alpha), true)
			draw_rect(Rect2(float(fx) * ts, float(fy) * ts, ts, ts), Color(0.35, 0.28, 0.14, 0.3), false, r_floor_border)

	# Draw walls — at Z3+ fill the full tile so adjacent walls merge visually
	var wall_xs: PackedInt32Array = data.get("wall_x", PackedInt32Array())
	var wall_ys: PackedInt32Array = data.get("wall_y", PackedInt32Array())
	var wall_mats: PackedStringArray = data.get("wall_material", PackedStringArray())
	var wall_inset: float = 0.0
	# Build wall_set for autotile adjacency lookup
	var wall_set: Dictionary = {}
	for i in range(wall_xs.size()):
		wall_set[Vector2i(wall_xs[i], wall_ys[i])] = true
	for i in range(wall_xs.size()):
		var wx: int = wall_xs[i]
		var wy: int = wall_ys[i]
		if wx < min_x or wx > max_x or wy < min_y or wy > max_y:
			continue
		var mat: String = wall_mats[i] if i < wall_mats.size() else ""
		var wall_color: Color = _wall_material_color(mat)
		_draw_wall_tile(wx, wy, ts, wall_color, mat, wall_set, wall_inset, r_autotile, r_bridge_px)

	# Draw doors (gap indicator)
	var door_xs: PackedInt32Array = data.get("door_x", PackedInt32Array())
	var door_ys: PackedInt32Array = data.get("door_y", PackedInt32Array())
	for i in range(door_xs.size()):
		var dx: int = door_xs[i]
		var dy: int = door_ys[i]
		if dx < min_x or dx > max_x or dy < min_y or dy > max_y:
			continue
		draw_rect(Rect2(float(dx) * ts + 2.0, float(dy) * ts + 2.0, ts - 4.0, ts - 4.0), Color(0.45, 0.30, 0.12, 0.5), true)

	# Draw furniture
	var furn_xs: PackedInt32Array = data.get("furniture_x", PackedInt32Array())
	var furn_ys: PackedInt32Array = data.get("furniture_y", PackedInt32Array())
	var furn_ids: PackedStringArray = data.get("furniture_id", PackedStringArray())
	var furn_font: Font = ThemeDB.fallback_font
	for i in range(furn_xs.size()):
		var fux: int = furn_xs[i]
		var fuy: int = furn_ys[i]
		if fux < min_x or fux > max_x or fuy < min_y or fuy > max_y:
			continue
		var fid: String = furn_ids[i] if i < furn_ids.size() else ""
		var pos_x: float = float(fux) * ts + ts * 0.5
		var pos_y: float = float(fuy) * ts + ts * 0.6
		# Sprite attempt before emoji fallback
		var furn_tex: Texture2D = _load_furniture_texture(fid, _deterministic_seed_for_tile(fux, fuy))
		if furn_tex != null:
			var draw_size: Vector2 = Vector2(ts, ts)
			var draw_pos: Vector2 = Vector2(float(fux) * ts, float(fuy) * ts)
			draw_texture_rect(furn_tex, Rect2(draw_pos, draw_size), false, Color(1.0, 1.0, 1.0, 0.85))
		else:
			var icon: String = _tile_furniture_icon(fid)
			if not icon.is_empty():
				draw_string(
					furn_font,
					Vector2(pos_x, pos_y),
					icon, HORIZONTAL_ALIGNMENT_CENTER, -1,
					int(ts * r_icon_scale), Color(1.0, 1.0, 1.0, 0.85)
				)
				if fid == "storage_pit":
					draw_string(
						furn_font,
						Vector2(pos_x, pos_y + ts * 0.3),
						Locale.ltr("BUILDING_TYPE_STOCKPILE"),
						HORIZONTAL_ALIGNMENT_CENTER, -1, 8,
						Color(0.95, 0.84, 0.58, 0.9)
					)


func _draw_wall_tile(wx: int, wy: int, ts: float, color: Color, material_id: String, wall_set: Dictionary, inset: float, autotile: bool = true, bridge_px: float = 2.0) -> void:
	var px: float = float(wx) * ts
	var py: float = float(wy) * ts

	# Fill: sprite first, solid color fallback if sprite unavailable
	var tex: Texture2D = _load_wall_material_texture(material_id, wx, wy)
	if tex != null:
		draw_texture_rect(tex, Rect2(px, py, ts, ts), false)
	else:
		draw_rect(Rect2(px, py, ts, ts), color, true)

	# Draw outline only on edges where there is NO adjacent wall (perimeter edges)
	var outline_color: Color = color.darkened(0.35)
	var line_w: float = maxf(1.0, ts * 0.08)

	# Top edge — no wall above
	if not wall_set.has(Vector2i(wx, wy - 1)):
		draw_line(Vector2(px, py), Vector2(px + ts, py), outline_color, line_w)
	# Bottom edge — no wall below
	if not wall_set.has(Vector2i(wx, wy + 1)):
		draw_line(Vector2(px, py + ts), Vector2(px + ts, py + ts), outline_color, line_w)
	# Left edge — no wall left
	if not wall_set.has(Vector2i(wx - 1, wy)):
		draw_line(Vector2(px, py), Vector2(px, py + ts), outline_color, line_w)
	# Right edge — no wall right
	if not wall_set.has(Vector2i(wx + 1, wy)):
		draw_line(Vector2(px + ts, py), Vector2(px + ts, py + ts), outline_color, line_w)


func _wall_material_color(material_id: String) -> Color:
	match material_id:
		"granite", "basalt":
			return Color(0.45, 0.42, 0.38, 0.85)
		"limestone", "sandstone":
			return Color(0.55, 0.50, 0.40, 0.85)
		"oak", "birch", "pine":
			return Color(0.50, 0.35, 0.18, 0.85)
		_:
			return Color(0.50, 0.45, 0.38, 0.85)


func _tile_furniture_icon(furniture_id: String) -> String:
	match furniture_id:
		"fire_pit": return "🔥"
		"lean_to": return "🛏"
		"storage_pit": return "📦"
		"workbench": return "⚒"
		"drying_rack": return "🪓"
		"totem": return "🗿"
		"hearth": return "🔥"
		_: return ""


func _has_tile_grid_data_at(tile_x: int, tile_y: int) -> bool:
	return _has_tile_grid_walls_at(tile_x, tile_y) or _has_tile_grid_furniture_at(tile_x, tile_y)


func _has_tile_grid_furniture_at(tile_x: int, tile_y: int) -> bool:
	var data: Dictionary = _get_tile_grid_data()
	var furn_xs: PackedInt32Array = data.get("furniture_x", PackedInt32Array())
	var furn_ys: PackedInt32Array = data.get("furniture_y", PackedInt32Array())
	for i in range(furn_xs.size()):
		if absi(furn_xs[i] - tile_x) <= 3 and absi(furn_ys[i] - tile_y) <= 3:
			return true
	return false


func _has_tile_grid_walls_at(tile_x: int, tile_y: int) -> bool:
	var data: Dictionary = _get_tile_grid_data()
	var wall_xs: PackedInt32Array = data.get("wall_x", PackedInt32Array())
	var wall_ys: PackedInt32Array = data.get("wall_y", PackedInt32Array())
	for i in range(wall_xs.size()):
		if absi(wall_xs[i] - tile_x) <= 3 and absi(wall_ys[i] - tile_y) <= 3:
			return true
	return false
