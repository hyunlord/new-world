class_name WorldGenerator
extends RefCounted


## Generate a procedural world using FastNoiseLite
func generate(world_data: WorldData, seed_value: int) -> void:
	var elevation_noise := FastNoiseLite.new()
	elevation_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	elevation_noise.fractal_octaves = GameConfig.NOISE_OCTAVES
	elevation_noise.frequency = 0.008
	elevation_noise.seed = seed_value

	var moisture_noise := FastNoiseLite.new()
	moisture_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	moisture_noise.fractal_octaves = 4
	moisture_noise.frequency = 0.006
	moisture_noise.seed = seed_value + 1

	var temperature_noise := FastNoiseLite.new()
	temperature_noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
	temperature_noise.fractal_octaves = 3
	temperature_noise.frequency = 0.004
	temperature_noise.seed = seed_value + 2

	var w: int = world_data.width
	var h: int = world_data.height

	for y in range(h):
		for x in range(w):
			# Raw noise values remapped to [0, 1]
			var raw_e: float = (elevation_noise.get_noise_2d(x, y) + 1.0) * 0.5
			var raw_m: float = (moisture_noise.get_noise_2d(x, y) + 1.0) * 0.5
			var raw_t: float = (temperature_noise.get_noise_2d(x, y) + 1.0) * 0.5

			# Apply island mask
			var mask: float = _island_mask(x, y, w, h)
			var e: float = clampf(raw_e * mask, 0.0, 1.0)

			# Temperature affected by latitude and elevation
			var latitude_factor: float = 1.0 - abs(2.0 * y / h - 1.0) * 0.4
			var elevation_cooling: float = e * 0.3
			var t: float = clampf(raw_t * latitude_factor - elevation_cooling, 0.0, 1.0)

			# Classify biome
			var biome: int = _classify_biome(e, raw_m, t)
			world_data.set_tile(x, y, biome, e, raw_m, t)


## Island shape mask: edges become ocean
func _island_mask(x: int, y: int, w: int, h: int) -> float:
	var nx: float = 2.0 * x / w - 1.0
	var ny: float = 2.0 * y / h - 1.0
	var d: float = maxf(absf(nx), absf(ny))
	return clampf(1.0 - pow(d, GameConfig.ISLAND_FALLOFF * 3.0), 0.0, 1.0)


## Classify biome from elevation, moisture, temperature
func _classify_biome(e: float, m: float, t: float) -> int:
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
