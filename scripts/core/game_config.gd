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

## Time conversion (1 tick = 2 game hours)
const TICK_HOURS: int = 2
const TICKS_PER_DAY: int = 12
const DAYS_PER_YEAR: int = 365
const TICKS_PER_MONTH: int = 365    # ~30.4 days * 12 ticks/day
const TICKS_PER_YEAR: int = 4380    # 365 * 12

## Age stage thresholds (in simulation ticks) — 6 stages
## infant ≤2y, toddler 3-5y, child 6-11y, teen 12-14y, adult 15-55y, elder 56+
const AGE_INFANT_END: int = 13140    # 3 years  (< 3y → infant)
const AGE_TODDLER_END: int = 26280   # 6 years  (< 6y → toddler)
const AGE_CHILD_END: int = 52560     # 12 years (< 12y → child)
const AGE_TEEN_END: int = 65700      # 15 years (< 15y → teen)
const AGE_ADULT_END: int = 245280    # 56 years (< 56y → adult, else elder)
const AGE_MAX: int = 525600          # 120 years (theoretical max)
const PREGNANCY_DURATION: int = 3360  # 280 days × 12 ticks/day (mean gestation)
const PREGNANCY_DURATION_STDEV: int = 120  # ~10 days × 12 ticks/day

## UI Scale (adjustable at runtime, saved with game)
var ui_scale: float = 1.0
const UI_SCALE_MIN: float = 0.7
const UI_SCALE_MAX: float = 1.5

## Base font sizes (multiplied by ui_scale)
const UI_FONT_SIZES: Dictionary = {
	"hud": 18,
	"hud_secondary": 15,
	"panel_title": 18,
	"panel_body": 14,
	"panel_hint": 13,
	"bar_label": 13,
	"popup_title": 22,
	"popup_heading": 16,
	"popup_body": 14,
	"popup_small": 13,
	"popup_close": 14,
	"popup_close_btn": 16,
	"help_title": 26,
	"help_section": 18,
	"help_body": 16,
	"help_footer": 14,
	"legend_title": 14,
	"legend_body": 12,
	"hint": 13,
	"toast": 15,
	"minimap_label": 13,
	"stats_title": 14,
	"stats_body": 12,
}

## Base UI element sizes (multiplied by ui_scale)
const UI_SIZES: Dictionary = {
	"minimap": 250,
	"minimap_large": 350,
	"mini_stats_width": 250,
	"mini_stats_height": 220,
	"select_panel_width": 320,
	"select_panel_height": 280,
	"hud_height": 34,
}


func get_font_size(key: String) -> int:
	return maxi(8, int(UI_FONT_SIZES.get(key, 14) * ui_scale))


func get_ui_size(key: String) -> int:
	return maxi(20, int(UI_SIZES.get(key, 100) * ui_scale))


## Convert simulation tick to calendar date (delegates to GameCalendar)
static func tick_to_date(tick: int) -> Dictionary:
	var GameCalendar = load("res://scripts/core/game_calendar.gd")
	return GameCalendar.tick_to_date(tick)


## Convert age in ticks to years (float)
static func get_age_years(age_ticks: int) -> float:
	return float(age_ticks) / float(TICKS_PER_YEAR)


## Get age stage string from age in ticks (6 stages)
static func get_age_stage(age_ticks: int) -> String:
	if age_ticks < AGE_INFANT_END:
		return "infant"
	elif age_ticks < AGE_TODDLER_END:
		return "toddler"
	elif age_ticks < AGE_CHILD_END:
		return "child"
	elif age_ticks < AGE_TEEN_END:
		return "teen"
	elif age_ticks < AGE_ADULT_END:
		return "adult"
	else:
		return "elder"

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

## Entity need decay rates (per needs tick, adjusted for TICK_HOURS=2)
const HUNGER_DECAY_RATE: float = 0.002
## Metabolic curve: hunger decays slower when already hungry (Keys et al. 1950)
const HUNGER_METABOLIC_MIN: float = 0.3   # Minimum metabolic rate at hunger=0
const HUNGER_METABOLIC_RANGE: float = 0.7  # 1.0 - HUNGER_METABOLIC_MIN
const ENERGY_DECAY_RATE: float = 0.003
const ENERGY_ACTION_COST: float = 0.005
const SOCIAL_DECAY_RATE: float = 0.001

## Starvation grace period (in NeedsSystem ticks, ~4 days)
const STARVATION_GRACE_TICKS: int = 25

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

## Resource regen tick interval (time-based, 10 days)
const RESOURCE_REGEN_TICK_INTERVAL: int = 120

## Building type definitions
const BUILDING_TYPES: Dictionary = {
	"stockpile": {"cost": {"wood": 2.0}, "build_ticks": 36, "radius": 8},
	"shelter": {"cost": {"wood": 4.0, "stone": 1.0}, "build_ticks": 60, "radius": 0},
	"campfire": {"cost": {"wood": 1.0}, "build_ticks": 24, "radius": 5},
}

## Job ratios (target distribution)
const JOB_RATIOS: Dictionary = {
	"gatherer": 0.5,
	"lumberjack": 0.25,
	"builder": 0.15,
	"miner": 0.1,
}

## Action-based tick intervals (NOT scaled — affect agent feel)
const GATHERING_TICK_INTERVAL: int = 3
const CONSTRUCTION_TICK_INTERVAL: int = 5
const BUILDING_EFFECT_TICK_INTERVAL: int = 10

## Time-based tick intervals (scaled for TICK_HOURS=2)
const JOB_ASSIGNMENT_TICK_INTERVAL: int = 24
const POPULATION_TICK_INTERVAL: int = 30

## Entity inventory
const MAX_CARRY: float = 10.0
const GATHER_AMOUNT: float = 2.0

## Population
const BIRTH_FOOD_COST: float = 3.0

## Pathfinding
const PATHFIND_MAX_STEPS: int = 200

## ══════════════════════════════════════
## Settlement & Migration
## ══════════════════════════════════════
const SETTLEMENT_MIN_DISTANCE: int = 25
const SETTLEMENT_BUILD_RADIUS: int = 15
const BUILDING_MIN_SPACING: int = 2
const MIGRATION_TICK_INTERVAL: int = 100
const MIGRATION_MIN_POP: int = 40
const MIGRATION_GROUP_SIZE_MIN: int = 5
const MIGRATION_GROUP_SIZE_MAX: int = 7
const MIGRATION_CHANCE: float = 0.05
const MIGRATION_SEARCH_RADIUS_MIN: int = 30
const MIGRATION_SEARCH_RADIUS_MAX: int = 80
const MAX_SETTLEMENTS: int = 5
const MIGRATION_COOLDOWN_TICKS: int = 500
const MIGRATION_STARTUP_FOOD: float = 30.0
const MIGRATION_STARTUP_WOOD: float = 10.0
const MIGRATION_STARTUP_STONE: float = 3.0
const SETTLEMENT_CLEANUP_INTERVAL: int = 250

## ══════════════════════════════════════
## Childcare & Child Development
## ══════════════════════════════════════
const CHILDCARE_TICK_INTERVAL: int = 10

## Per-stage hunger threshold for childcare feeding (higher = feed sooner)
const CHILDCARE_HUNGER_THRESHOLDS: Dictionary = {
	"infant": 0.85,
	"toddler": 0.80,
	"child": 0.75,
	"teen": 0.70,
}

## Feed amounts per childcare tick (food units from stockpile)
const CHILDCARE_FEED_AMOUNTS: Dictionary = {
	"infant": 0.40,
	"toddler": 0.50,
	"child": 0.50,
	"teen": 0.60,
}

## Hunger decay multiplier by age stage (applied in NeedsSystem)
## WHO: infant caloric need is 30-50% of adult
const CHILD_HUNGER_DECAY_MULT: Dictionary = {
	"infant": 0.15,
	"toddler": 0.25,
	"child": 0.35,
	"teen": 0.70,
}

## Child-specific starvation grace ticks (longer than adult STARVATION_GRACE_TICKS=25)
## Academic basis: Gurven & Kaplan 2007, Pontzer 2018 — child starvation rare in forager societies
const CHILD_STARVATION_GRACE_TICKS: Dictionary = {
	"infant": 50,
	"toddler": 40,
	"child": 30,
	"teen": 20,
}

## Gathering efficiency by age stage (1.0 = full adult rate)
const CHILD_GATHER_EFFICIENCY: Dictionary = {
	"child": 0.4,
	"teen": 0.8,
	"elder": 0.5,
}

## Movement skip modulo by age stage (skip 1 in N ticks; higher N = faster)
## infant/toddler: skip every other tick → 50%, child: skip 1/3 → 70%
## teen: skip 1/10 → 90%, elder: skip 1/3 → 67%
const CHILD_MOVE_SKIP_MOD: Dictionary = {
	"infant": 2,
	"toddler": 2,
	"child": 3,
	"teen": 10,
	"elder": 3,
}


