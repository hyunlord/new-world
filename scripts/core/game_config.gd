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

## System tick intervals
const NEEDS_TICK_INTERVAL: int = 2
const BEHAVIOR_TICK_INTERVAL: int = 10
const MOVEMENT_TICK_INTERVAL: int = 3

## Entity need decay rates (per needs tick)
const HUNGER_DECAY_RATE: float = 0.002
const ENERGY_DECAY_RATE: float = 0.002
const ENERGY_ACTION_COST: float = 0.004
const SOCIAL_DECAY_RATE: float = 0.002

## Starvation grace period
const STARVATION_GRACE_TICKS: int = 50

## Eating constants
const FOOD_HUNGER_RESTORE: float = 0.3
const HUNGER_EAT_THRESHOLD: float = 0.5

## World generation
const WORLD_SEED: int = 42
const NOISE_OCTAVES: int = 5
const ISLAND_FALLOFF: float = 0.7

## ══════════════════════════════════════
## Phase 1 Constants
## ══════════════════════════════════════

## Resource types
enum ResourceType { FOOD, WOOD, STONE }

## Biome-resource mapping: biome -> {food_min, food_max, wood_min, wood_max, stone_min, stone_max}
const BIOME_RESOURCES: Dictionary = {
	Biome.GRASSLAND: {"food_min": 5.0, "food_max": 10.0, "wood_min": 0.0, "wood_max": 0.0, "stone_min": 0.0, "stone_max": 0.0},
	Biome.FOREST: {"food_min": 2.0, "food_max": 5.0, "wood_min": 5.0, "wood_max": 8.0, "stone_min": 0.0, "stone_max": 0.0},
	Biome.DENSE_FOREST: {"food_min": 0.0, "food_max": 1.0, "wood_min": 8.0, "wood_max": 12.0, "stone_min": 0.0, "stone_max": 0.0},
	Biome.HILL: {"food_min": 0.0, "food_max": 0.0, "wood_min": 0.0, "wood_max": 1.0, "stone_min": 3.0, "stone_max": 6.0},
	Biome.MOUNTAIN: {"food_min": 0.0, "food_max": 0.0, "wood_min": 0.0, "wood_max": 0.0, "stone_min": 5.0, "stone_max": 10.0},
	Biome.BEACH: {"food_min": 1.0, "food_max": 2.0, "wood_min": 0.0, "wood_max": 0.0, "stone_min": 0.0, "stone_max": 1.0},
}

## Resource regen rates (per regen tick)
const FOOD_REGEN_RATE: float = 1.0
const WOOD_REGEN_RATE: float = 0.3
const STONE_REGEN_RATE: float = 0.0

## Resource regen tick interval
const RESOURCE_REGEN_TICK_INTERVAL: int = 50

## Building type definitions
const BUILDING_TYPES: Dictionary = {
	"stockpile": {"cost": {"wood": 2.0}, "build_ticks": 30, "radius": 8},
	"shelter": {"cost": {"wood": 4.0, "stone": 1.0}, "build_ticks": 50, "radius": 0},
	"campfire": {"cost": {"wood": 1.0}, "build_ticks": 20, "radius": 5},
}

## Job ratios (target distribution)
const JOB_RATIOS: Dictionary = {
	"gatherer": 0.5,
	"lumberjack": 0.25,
	"builder": 0.15,
	"miner": 0.1,
}

## New system tick intervals
const GATHERING_TICK_INTERVAL: int = 3
const CONSTRUCTION_TICK_INTERVAL: int = 5
const BUILDING_EFFECT_TICK_INTERVAL: int = 10
const JOB_ASSIGNMENT_TICK_INTERVAL: int = 50
const POPULATION_TICK_INTERVAL: int = 60

## Entity inventory
const MAX_CARRY: float = 10.0
const GATHER_AMOUNT: float = 2.0

## Population
const BIRTH_FOOD_COST: float = 3.0
const OLD_AGE_TICKS: int = 8640
const MAX_AGE_TICKS: int = 17280

## Pathfinding
const PATHFIND_MAX_STEPS: int = 200

## ══════════════════════════════════════
## Settlement & Migration
## ══════════════════════════════════════
const SETTLEMENT_MIN_DISTANCE: int = 25
const SETTLEMENT_BUILD_RADIUS: int = 15
const BUILDING_MIN_SPACING: int = 2
const MIGRATION_TICK_INTERVAL: int = 200
const MIGRATION_MIN_POP: int = 40
const MIGRATION_GROUP_SIZE_MIN: int = 5
const MIGRATION_GROUP_SIZE_MAX: int = 7
const MIGRATION_CHANCE: float = 0.05
const MIGRATION_SEARCH_RADIUS_MIN: int = 30
const MIGRATION_SEARCH_RADIUS_MAX: int = 80
const MAX_SETTLEMENTS: int = 5
const MIGRATION_COOLDOWN_TICKS: int = 1000
const MIGRATION_STARTUP_FOOD: float = 30.0
const MIGRATION_STARTUP_WOOD: float = 10.0
const MIGRATION_STARTUP_STONE: float = 3.0
const SETTLEMENT_CLEANUP_INTERVAL: int = 500
