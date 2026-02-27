## MapEditorController — 브러시 페인팅 + 스폰 포인트 관리
## WorldSetup에서 인스턴스화. UI 이벤트를 받아 WorldData/ResourceMap에 반영.
extends RefCounted

## 브러시 모드
enum BrushMode {
	TERRAIN,  ## 바이옴 직접 페인팅
	FOOD,     ## food 자원 추가
	WOOD,     ## wood 자원 추가
	STONE,    ## stone 자원 추가
	SPAWN,    ## 에이전트 스폰 포인트 배치
	ERASE,    ## 바이옴 → DEEP_WATER (삭제)
}

## 현재 브러시 설정
var brush_mode: int = BrushMode.TERRAIN
var brush_biome: int = 3  ## GameConfig.Biome.GRASSLAND = 3
var brush_size: int = 3
var brush_strength: float = 1.0
var spawn_count: int = 20

## 내부 참조 (typed를 RefCounted 이름 없이 untyped로 선언)
var _world_data = null       ## WorldData RefCounted
var _resource_map = null     ## ResourceMap RefCounted
var _world_renderer = null   ## WorldRenderer Node (Sprite2D)

## 스폰 포인트 목록. 각 항목: {position: Vector2i, count: int}
var _spawn_points: Array = []

## 바이옴별 기본 elevation (GameConfig.BIOME_DEFAULT_ELEVATION과 동일)
const _BIOME_ELEV: Dictionary = {
	0: 0.15,  ## DEEP_WATER
	1: 0.35,  ## SHALLOW_WATER
	2: 0.43,  ## BEACH
	3: 0.55,  ## GRASSLAND
	4: 0.58,  ## FOREST
	5: 0.60,  ## DENSE_FOREST
	6: 0.72,  ## HILL
	7: 0.85,  ## MOUNTAIN
	8: 0.93,  ## SNOW
}

## 바이옴별 기본 moisture
const _BIOME_MOIST: Dictionary = {
	0: 0.9,
	1: 0.8,
	2: 0.3,
	3: 0.4,
	4: 0.55,
	5: 0.75,
	6: 0.35,
	7: 0.3,
	8: 0.2,
}


## 참조 설정. WorldSetup._ready()에서 호출.
func setup(world_data, resource_map, world_renderer) -> void:
	_world_data = world_data
	_resource_map = resource_map
	_world_renderer = world_renderer


## 타일 좌표에 브러시 적용.
## WorldSetup의 마우스 입력 핸들러에서 호출.
func paint(tile_pos: Vector2i) -> void:
	if _world_data == null or _resource_map == null:
		return
	if brush_mode == BrushMode.SPAWN:
		_mark_spawn_point(tile_pos)
		return
	var tiles: Array = _get_brush_tiles(tile_pos, brush_size)
	for t in tiles:
		if not _world_data.is_valid(t.x, t.y):
			continue
		_apply_brush(t)
	## 브러시 스트로크마다 GPU flush
	if _world_renderer != null and _world_renderer.has_method("flush_pixel_updates"):
		_world_renderer.flush_pixel_updates()


## 스폰 포인트 목록 반환. WorldSetup이 시작 시 사용.
func get_spawn_points() -> Array:
	return _spawn_points.duplicate()


## 스폰 포인트 초기화.
func clear_spawn_points() -> void:
	_spawn_points.clear()


## 마우스 스크린 좌표 → 타일 좌표 변환.
## camera: Camera2D
func screen_to_tile(camera: Camera2D, _screen_pos: Vector2) -> Vector2i:
	var world_pos: Vector2 = camera.get_global_mouse_position()
	var tile_size: int = GameConfig.TILE_SIZE
	var tx: int = int(world_pos.x / tile_size)
	var ty: int = int(world_pos.y / tile_size)
	return Vector2i(tx, ty)


# ── 내부 함수 ────────────────────────────────────────────────


func _apply_brush(t: Vector2i) -> void:
	match brush_mode:
		BrushMode.TERRAIN:
			var e: float = _BIOME_ELEV.get(brush_biome, 0.55)
			var m: float = _BIOME_MOIST.get(brush_biome, 0.5)
			_world_data.set_tile(t.x, t.y, brush_biome, e, m, 0.5)
			if _world_renderer != null and _world_renderer.has_method("update_tile_pixel"):
				_world_renderer.update_tile_pixel(t.x, t.y)
		BrushMode.FOOD:
			var cur: float = _resource_map.get_food(t.x, t.y)
			_resource_map.set_food(t.x, t.y, minf(cur + 2.0 * brush_strength, 15.0))
		BrushMode.WOOD:
			var cur: float = _resource_map.get_wood(t.x, t.y)
			_resource_map.set_wood(t.x, t.y, minf(cur + 2.0 * brush_strength, 15.0))
		BrushMode.STONE:
			var cur: float = _resource_map.get_stone(t.x, t.y)
			_resource_map.set_stone(t.x, t.y, minf(cur + 2.0 * brush_strength, 15.0))
		BrushMode.ERASE:
			_world_data.set_tile(t.x, t.y, 0, 0.1, 0.5, 0.5)  ## DEEP_WATER = 0
			_resource_map.set_food(t.x, t.y, 0.0)
			_resource_map.set_wood(t.x, t.y, 0.0)
			_resource_map.set_stone(t.x, t.y, 0.0)
			if _world_renderer != null and _world_renderer.has_method("update_tile_pixel"):
				_world_renderer.update_tile_pixel(t.x, t.y)


## 원형 브러시 영역 타일 목록 반환.
func _get_brush_tiles(center: Vector2i, radius: int) -> Array:
	var tiles: Array = []
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			if dx * dx + dy * dy <= radius * radius:
				tiles.append(center + Vector2i(dx, dy))
	return tiles


## SPAWN 모드: 클릭 위치에 스폰 포인트 추가/갱신.
func _mark_spawn_point(tile_pos: Vector2i) -> void:
	if _world_data == null:
		return
	if not _world_data.is_valid(tile_pos.x, tile_pos.y):
		return
	## walkable 체크 (비-수중 바이옴)
	var biome: int = _world_data.get_biome(tile_pos.x, tile_pos.y)
	if biome <= 1:  ## DEEP_WATER=0, SHALLOW_WATER=1 → not walkable
		return
	## 10타일 이내 기존 포인트 갱신
	for i in range(_spawn_points.size()):
		var d: Vector2i = _spawn_points[i].position - tile_pos
		if d.x * d.x + d.y * d.y < 100:
			_spawn_points[i] = {position = tile_pos, count = spawn_count}
			return
	_spawn_points.append({position = tile_pos, count = spawn_count})
