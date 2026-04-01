extends RefCounted

## PresetMapGenerator: 4종 프리셋 맵 생성 (island / continent / archipelago / random)
## class_name 사용 금지 — RefCounted headless 호환
## preload로 참조: preload("res://scripts/core/world/preset_map_generator.gd")

const WorldGeneratorScript = preload("res://scripts/core/world/world_generator.gd")

## 프리셋 생성 진입점
## preset_id: "island" | "continent" | "archipelago" | "random"
## seed_value: random일 때 사용 (island/continent/archipelago는 내부 고정 시드)
## Generate world tiles and resources using a named preset layout and optional seed.
func generate_preset(world_data: RefCounted, resource_map: RefCounted,
		preset_id: String, seed_value: int) -> void:
	var rng := RandomNumberGenerator.new()
	world_data.begin_tile_update()
	match preset_id:
		"island":
			rng.seed = GameConfig.PRESET_SEED_ISLAND
			_generate_island(world_data, rng)
		"continent":
			rng.seed = GameConfig.PRESET_SEED_CONTINENT
			_generate_continent(world_data, rng)
		"archipelago":
			rng.seed = GameConfig.PRESET_SEED_ARCHIPELAGO
			_generate_archipelago(world_data, rng)
		_:  ## "random" or unknown
			_generate_random(world_data, seed_value)
			rng.seed = seed_value
	world_data.end_tile_update()
	_populate_resources(world_data, resource_map, rng)


## ── Island 생성 ──────────────────────────────────────────────────────────────
## falloff=3.5로 섬 마스크 강화, 중앙부 elevation 보장, 강 시뮬레이션
func _generate_island(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	var w: int = world_data.width
	var h: int = world_data.height

	var elev_noise := FastNoiseLite.new()
	elev_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	elev_noise.fractal_octaves = GameConfig.NOISE_OCTAVES
	elev_noise.frequency = 0.008
	elev_noise.seed = rng.randi()

	var moist_noise := FastNoiseLite.new()
	moist_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	moist_noise.fractal_octaves = 4
	moist_noise.frequency = 0.006
	moist_noise.seed = rng.randi()

	for y in range(h):
		for x in range(w):
			var raw_e: float = (elev_noise.get_noise_2d(x, y) + 1.0) * 0.5
			var raw_m: float = (moist_noise.get_noise_2d(x, y) + 1.0) * 0.5
			## 강한 섬 마스크 (falloff=3.5)
			var mask: float = _island_mask(x, y, w, h, 3.5)
			var e: float = clampf(raw_e * mask, 0.0, 1.0)
			## 중앙 30% 영역 elevation 최소 0.55 보장
			var cx: float = absf(2.0 * x / w - 1.0)
			var cy: float = absf(2.0 * y / h - 1.0)
			if cx < 0.3 and cy < 0.3:
				e = maxf(e, 0.55)
			var biome: int = _classify_biome(e, raw_m, 0.5)
			world_data.set_tile(x, y, biome, e, raw_m, 0.5)

	## 강 시뮬레이션: 고지대에서 시작해 낮은 곳으로 흐름
	_simulate_rivers(world_data, rng, 3)


## ── Continent 생성 ────────────────────────────────────────────────────────────
## 가장자리도 육지, 좌측 산맥, 중앙 평야, 강 2~3개
func _generate_continent(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	var w: int = world_data.width
	var h: int = world_data.height

	var elev_noise := FastNoiseLite.new()
	elev_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	elev_noise.fractal_octaves = GameConfig.NOISE_OCTAVES
	elev_noise.frequency = 0.007
	elev_noise.seed = rng.randi()

	var moist_noise := FastNoiseLite.new()
	moist_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	moist_noise.fractal_octaves = 4
	moist_noise.frequency = 0.006
	moist_noise.seed = rng.randi()

	for y in range(h):
		for x in range(w):
			var raw_e: float = (elev_noise.get_noise_2d(x, y) + 1.0) * 0.5
			var raw_m: float = (moist_noise.get_noise_2d(x, y) + 1.0) * 0.5
			var e: float = raw_e
			## 좌측 1/4 지점에 산맥 (elevation 강제 boost)
			var xf: float = float(x) / w
			if xf > 0.20 and xf < 0.35:
				e = clampf(e + 0.35, 0.0, 1.0)
			## 중앙 평야 (elevation 제한)
			if xf > 0.40 and xf < 0.75:
				e = clampf(e, 0.48, 0.68)
			## 대륙은 가장자리 바다 없음 (elevation 최소 0.42 보장)
			e = maxf(e, 0.42)
			var biome: int = _classify_biome(e, raw_m, 0.5)
			world_data.set_tile(x, y, biome, e, raw_m, 0.5)

	_simulate_rivers(world_data, rng, 3)


## ── Archipelago 생성 ──────────────────────────────────────────────────────────
## 전체 바다 위에 PRESET_ARCHIPELAGO_ISLAND_COUNT개 원형 섬
func _generate_archipelago(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	var w: int = world_data.width
	var h: int = world_data.height

	## 전체를 바다로 초기화
	for y in range(h):
		for x in range(w):
			world_data.set_tile(x, y, GameConfig.Biome.DEEP_WATER, 0.15, 0.9, 0.5)

	## 섬 배치 (최소 40타일 간격)
	var island_centers: Array = []
	var attempts: int = 0
	var margin: int = 50

	while island_centers.size() < GameConfig.PRESET_ARCHIPELAGO_ISLAND_COUNT and attempts < 200:
		attempts += 1
		var cx: int = rng.randi_range(margin, w - margin)
		var cy: int = rng.randi_range(margin, h - margin)
		## 기존 섬과 거리 체크
		var too_close: bool = false
		for ic in island_centers:
			var dx: int = cx - ic.x
			var dy: int = cy - ic.y
			if dx * dx + dy * dy < 1600:  ## 40 타일
				too_close = true
				break
		if too_close:
			continue
		island_centers.append(Vector2i(cx, cy))
		var radius: int = rng.randi_range(20, 45)
		_paint_circular_island(world_data, cx, cy, radius)

	_simulate_rivers(world_data, rng, island_centers.size())


## 원형 섬 페인팅 (가우시안 감쇠)
func _paint_circular_island(world_data: RefCounted, cx: int, cy: int, radius: int) -> void:
	var w: int = world_data.width
	var h: int = world_data.height
	for dy in range(-radius - 5, radius + 6):
		for dx in range(-radius - 5, radius + 6):
			var px: int = cx + dx
			var py: int = cy + dy
			if px < 0 or px >= w or py < 0 or py >= h:
				continue
			var dist: float = sqrt(float(dx * dx + dy * dy))
			## 가우시안 감쇠: 중심 0.75, 가장자리 0.35
			var t: float = clampf(1.0 - dist / float(radius), 0.0, 1.0)
			var e: float = lerpf(0.35, 0.75, t * t)
			if e < 0.30:
				continue
			var m: float = lerpf(0.4, 0.6, t)
			var biome: int = _classify_biome(e, m, 0.5)
			world_data.set_tile(px, py, biome, e, m, 0.5)


## ── Random 생성 ───────────────────────────────────────────────────────────────
## 기존 WorldGenerator에 위임
func _generate_random(world_data: RefCounted, seed_value: int) -> void:
	var gen = WorldGeneratorScript.new()
	gen.generate(world_data, seed_value)


## ── 섬 마스크 ─────────────────────────────────────────────────────────────────
func _island_mask(x: int, y: int, w: int, h: int, falloff: float) -> float:
	var nx: float = 2.0 * x / w - 1.0
	var ny: float = 2.0 * y / h - 1.0
	var d: float = maxf(absf(nx), absf(ny))
	return clampf(1.0 - pow(d, falloff * 3.0), 0.0, 1.0)


## ── 바이옴 분류 ───────────────────────────────────────────────────────────────
func _classify_biome(e: float, m: float, _t: float) -> int:
	if e < 0.30:
		return GameConfig.Biome.DEEP_WATER
	if e < 0.40:
		return GameConfig.Biome.SHALLOW_WATER
	if e < 0.45:
		return GameConfig.Biome.BEACH
	if e < 0.65:
		if m < 0.3:
			return GameConfig.Biome.GRASSLAND
		if m < 0.6:
			return GameConfig.Biome.FOREST
		return GameConfig.Biome.DENSE_FOREST
	if e < 0.80:
		return GameConfig.Biome.HILL
	if e < 0.90:
		return GameConfig.Biome.MOUNTAIN
	return GameConfig.Biome.SNOW


## ── 강 시뮬레이션 ─────────────────────────────────────────────────────────────
## 고지대(HILL/MOUNTAIN) 타일에서 시작해 낮은 곳으로 흘러 SHALLOW_WATER 타일 생성
func _simulate_rivers(world_data: RefCounted, rng: RandomNumberGenerator, count: int) -> void:
	var w: int = world_data.width
	var h: int = world_data.height
	## HILL 이상 타일 수집
	var high_tiles: Array = []
	for y in range(10, h - 10):
		for x in range(10, w - 10):
			var b: int = world_data.get_biome(x, y)
			if b == GameConfig.Biome.HILL or b == GameConfig.Biome.MOUNTAIN:
				high_tiles.append(Vector2i(x, y))
	if high_tiles.is_empty():
		return

	var river_count: int = mini(count, high_tiles.size())
	for _i in range(river_count):
		var start_idx: int = rng.randi() % high_tiles.size()
		var pos: Vector2i = high_tiles[start_idx]
		## 최대 80타일 흘러내려감
		for _step in range(80):
			var cur_e: float = world_data.get_elevation(pos.x, pos.y)
			if cur_e < 0.35:
				break
			world_data.set_tile(pos.x, pos.y, GameConfig.Biome.SHALLOW_WATER,
				cur_e, 0.8, 0.5)
			## 4방향 중 가장 낮은 이웃으로 이동
			var dirs: Array = [Vector2i(1,0), Vector2i(-1,0), Vector2i(0,1), Vector2i(0,-1)]
			var best: Vector2i = pos
			var best_e: float = cur_e
			for d in dirs:
				var nx2: int = pos.x + d.x
				var ny2: int = pos.y + d.y
				if nx2 < 0 or nx2 >= w or ny2 < 0 or ny2 >= h:
					continue
				var ne: float = world_data.get_elevation(nx2, ny2)
				if ne < best_e:
					best_e = ne
					best = Vector2i(nx2, ny2)
			if best == pos:
				break
			pos = best


## ── 자원 자동 배치 ────────────────────────────────────────────────────────────
## BIOME_RESOURCES 기반 + 약간의 랜덤 노이즈
func _populate_resources(world_data: RefCounted, resource_map: RefCounted,
		rng: RandomNumberGenerator) -> void:
	var w: int = world_data.width
	var h: int = world_data.height

	for y in range(h):
		for x in range(w):
			var biome: int = world_data.get_biome(x, y)
			var food_base: float = 0.0
			var wood_base: float = 0.0
			var stone_base: float = 0.0
			match biome:
				GameConfig.Biome.GRASSLAND:
					food_base = 5.0
					wood_base = 1.0
				GameConfig.Biome.FOREST:
					food_base = 3.0
					wood_base = 6.0
				GameConfig.Biome.DENSE_FOREST:
					food_base = 2.0
					wood_base = 9.0
				GameConfig.Biome.BEACH:
					food_base = 2.0
				GameConfig.Biome.HILL:
					stone_base = 4.0
					food_base = 1.0
				GameConfig.Biome.MOUNTAIN:
					stone_base = 7.0
				GameConfig.Biome.SHALLOW_WATER:
					food_base = 2.0

			if food_base > 0.0:
				var noise: float = rng.randf_range(0.7, 1.3)
				resource_map.set_food(x, y, minf(food_base * noise, 15.0))
			if wood_base > 0.0:
				var noise: float = rng.randf_range(0.7, 1.3)
				resource_map.set_wood(x, y, minf(wood_base * noise, 15.0))
			if stone_base > 0.0:
				var noise: float = rng.randf_range(0.7, 1.3)
				resource_map.set_stone(x, y, minf(stone_base * noise, 15.0))
			# Surface stone scatter on flat terrain — deterministic hash, no rng consumed.
			# Scatters sparse stone deposits on biomes that lack natural stone.
			var sh: int = (x * 2749 + y * 5281) % 100
			var sh2: int = (x * 1237 + y * 4567) % 100
			if biome == GameConfig.Biome.GRASSLAND and sh < 8:
				resource_map.set_stone(x, y, 15.0 + float(sh2 % 26))   # 15–40 units
			elif biome == GameConfig.Biome.FOREST and sh < 5:
				resource_map.set_stone(x, y, 10.0 + float(sh2 % 16))   # 10–25 units
			elif biome == GameConfig.Biome.BEACH and sh < 12:
				resource_map.set_stone(x, y, 10.0 + float(sh2 % 21))   # 10–30 units
			elif biome == GameConfig.Biome.HILL and sh < 30:
				resource_map.set_stone(x, y, 50.0 + float(sh2 % 51))   # 50–100 units (rich deposit)


## ── 스폰 위치 추천 ────────────────────────────────────────────────────────────
## walkable + 식량 >= 3.0인 타일 중 최대 5개 반환
## Return up to 5 walkable tile positions near the map center suitable for spawning entities.
func get_spawn_suggestions(world_data: RefCounted,
		resource_map: RefCounted) -> Array:
	var w: int = world_data.width
	var h: int = world_data.height
	@warning_ignore("integer_division")
	var cx: int = w / 2
	@warning_ignore("integer_division")
	var cy: int = h / 2
	var candidates: Array = []

	for y in range(h):
		for x in range(w):
			if not world_data.is_walkable(x, y):
				continue
			var food: float = resource_map.get_food(x, y)
			if food < 3.0:
				continue
			var dx: int = x - cx
			var dy: int = y - cy
			var dist_sq: int = dx * dx + dy * dy
			candidates.append({"pos": Vector2i(x, y), "dist_sq": dist_sq, "food": food})

	if candidates.is_empty():
		## fallback: 그냥 walkable한 중앙 근처 타일
		for r in range(5, 60):
			for dy in range(-r, r + 1):
				for dx in range(-r, r + 1):
					var px: int = cx + dx
					var py: int = cy + dy
					if px < 0 or px >= w or py < 0 or py >= h:
						continue
					if world_data.is_walkable(px, py):
						return [{"pos": Vector2i(px, py), "dist_sq": 0, "food": 0.0}]
		return []

	## 거리 오름차순 정렬 후 최적 1개만 반환
	## 복수 스폰 포인트는 유저가 스폰 브러시로 직접 배치해야 함
	candidates.sort_custom(func(a, b): return a.dist_sq < b.dist_sq)
	var result: Array = []
	if not candidates.is_empty():
		result.append(candidates[0].pos)
	return result
