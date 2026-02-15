extends Node

## World constants
const WORLD_SIZE := Vector2i(256, 256)
const TILE_SIZE: int = 16
const CHUNK_SIZE: int = 32

## Simulation parameters
const TICKS_PER_SECOND: int = 10
const MAX_ENTITIES: int = 500
const INITIAL_SPAWN_COUNT: int = 20
const MAX_TICKS_PER_FRAME: int = 5

## Time conversion (1 tick = 1 game hour)
const TICK_HOURS: int = 1
const HOURS_PER_DAY: int = 24
const DAYS_PER_YEAR: int = 360

## Speed options (multipliers)
const SPEED_OPTIONS: Array[int] = [1, 2, 3, 5, 10]

## Biome enum
enum Biome {
	DEEP_WATER,
	SHALLOW_WATER,
	BEACH,
	GRASSLAND,
	FOREST,
	DENSE_FOREST,
	HILL,
	MOUNTAIN,
	SNOW,
}

## Biome colors
const BIOME_COLORS: Dictionary = {
	Biome.DEEP_WATER: Color(0.05, 0.10, 0.35),
	Biome.SHALLOW_WATER: Color(0.12, 0.30, 0.55),
	Biome.BEACH: Color(0.85, 0.80, 0.55),
	Biome.GRASSLAND: Color(0.35, 0.65, 0.20),
	Biome.FOREST: Color(0.15, 0.45, 0.10),
	Biome.DENSE_FOREST: Color(0.08, 0.30, 0.05),
	Biome.HILL: Color(0.55, 0.50, 0.35),
	Biome.MOUNTAIN: Color(0.45, 0.42, 0.40),
	Biome.SNOW: Color(0.90, 0.92, 0.95),
}

## Biome movement cost (0.0 = impassable)
const BIOME_MOVE_COST: Dictionary = {
	Biome.DEEP_WATER: 0.0,
	Biome.SHALLOW_WATER: 0.0,
	Biome.BEACH: 1.2,
	Biome.GRASSLAND: 1.0,
	Biome.FOREST: 1.3,
	Biome.DENSE_FOREST: 1.8,
	Biome.HILL: 1.5,
	Biome.MOUNTAIN: 0.0,
	Biome.SNOW: 2.0,
}

## Camera settings
const CAMERA_ZOOM_MIN: float = 0.25
const CAMERA_ZOOM_MAX: float = 4.0
const CAMERA_ZOOM_STEP: float = 0.1
const CAMERA_PAN_SPEED: float = 500.0
const CAMERA_ZOOM_SPEED: float = 0.15

## Entity need decay rates (per tick)
const HUNGER_DECAY_RATE: float = 0.003
const ENERGY_DECAY_RATE: float = 0.002
const ENERGY_ACTION_COST: float = 0.004
const SOCIAL_DECAY_RATE: float = 0.001

## World generation
const WORLD_SEED: int = 42
const NOISE_OCTAVES: int = 5
const ISLAND_FALLOFF: float = 0.7
