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

## [Layer 1.5] Body Attributes — potential/trainability/realized 3-레이어
## ── Potential 스케일 (int 0~10,000) ─────────────────────
const BODY_POTENTIAL_MEAN: int = 1050   ## 기준 평균 — 1050×0.95≈998≈평균성인 realized 1,000
const BODY_POTENTIAL_SD: int = 175      ## SD (기저값의 25%)
const BODY_POTENTIAL_MIN: int = 50      ## 최솟값
const BODY_POTENTIAL_MAX: int = 10000   ## 최댓값 (설계상 개인 최대)

## 성별 보정 (potential에만 적용, trainability에는 적용 안 함)
## 근거: Refalo 2025 메타분석 — trainability 성별차 0.69% (무시 가능)
const BODY_SEX_DELTA_MALE: Dictionary = {
	"str":  160,   ## 남성 +160 → 남 860, 여 540, 비율 63%
	"agi":   30,
	"end":  -15,   ## 여성 지구력 우위
	"tou":  100,   ## 남성 골밀도/결합조직 우위
	"rec":  -15,   ## 여성 회복 소폭 우위
	"dr":   -80,   ## 여성 면역 우위 (Davis 2015)
}

## ── Trainability 스케일 (int 0~1,000) ───────────────────
const TRAINABILITY_MEAN: int = 500
const TRAINABILITY_SD: int = 150
const TRAINABILITY_MIN: int = 50
const TRAINABILITY_MAX: int = 1000

## ── 선천 면역력 (int 0~1,000) ────────────────────────────
const INNATE_IMMUNITY_MEAN: int = 500
const INNATE_IMMUNITY_SD: int = 100
const INNATE_IMMUNITY_SEX_DELTA_FEMALE: int = 80   ## 여성 +80

## ── 훈련 XP 시스템 ────────────────────────────────────────
## XP_FOR_FULL_PROGRESS: 이 XP에 도달하면 평균 trainability로 ceiling 100% 달성
const XP_FOR_FULL_PROGRESS: float = 10000.0

## 활동별 XP 획득량 (per gathering/construction 성공)
const GATHER_XP_END: float = 0.50
const GATHER_XP_STR: float = 0.30
const GATHER_XP_AGI: float = 0.20
const CONSTRUCT_XP_STR: float = 0.80
const CONSTRUCT_XP_TOU: float = 0.30
const CONSTRUCT_XP_AGI: float = 0.20

## [Skill XP per action — these feed StatQuery.add_xp(), separate from body training_xp]
const SKILL_XP_FORAGING: float     = 2.0   ## per gather_food completion
const SKILL_XP_WOODCUTTING: float  = 2.0   ## per gather_wood completion
const SKILL_XP_MINING: float       = 2.0   ## per gather_stone completion
const SKILL_XP_CONSTRUCTION: float = 3.0   ## per build tick (higher because build is slower)
const SKILL_XP_HUNTING: float      = 2.0   ## reserved for future hunting action

## ── speed/strength 파생 공식 ──────────────────────────────
## entity.speed = float(agi_realized) * BODY_SPEED_SCALE + BODY_SPEED_BASE
## agi=700(평균 성인, 훈련없음): speed=1.14,  agi=1500(전사): speed=2.1
const BODY_SPEED_BASE: float = 0.30
const BODY_SPEED_SCALE: float = 0.0012   ## float(agi_realized) * 0.0012
## entity.strength = float(str_realized) / 1000.0  → str=700: 0.70, str=1500: 1.50
const BODY_REALIZED_MAX: int = 15000     ## realized 정규화 기준 (str/agi/end/tou/rec), UI _draw_bar용
const BODY_REALIZED_DR_MAX: int = 10000  ## dr realized 정규화 기준, UI _draw_bar용

## ── Body Attribute Gameplay Loop ─────────────────────────────────────────────
## 훈련 XP: 행동 완료 시 entity.body.training_xp[axis] 에 누적 (age_system이 yearly 재계산)
## Heritage 1999 (trainability h²=0.47), Ahtiainen 2016 (개인차 8.5배)
const BODY_XP_GATHER_FOOD: float  = 0.8   ## end+str (채집: 지구력+근력)
const BODY_XP_GATHER_WOOD: float  = 1.2   ## str+end (벌목: 근력 위주)
const BODY_XP_GATHER_STONE: float = 1.5   ## str+tou (채석: 근력+강인함)
const BODY_XP_BUILD: float        = 1.0   ## str+agi (건설: 근력+민첩)
const BODY_XP_REST: float         = 0.3   ## rec (휴식: 회복력 단련)

## Endurance → energy action cost reduction (Borg 1982 RPE scale)
## energy_cost = ENERGY_ACTION_COST * (1 - BODY_END_COST_REDUCTION * end_norm)
## end_norm = realized["end"] / BODY_REALIZED_MAX  (0.0~1.0)
const BODY_END_COST_REDUCTION: float = 0.40  ## 최대 40% 절감 (END=1.0일 때)

## Recuperation → rest energy recovery (Buchheit & Laursen 2013)
## energy_recovery = BODY_REST_ENERGY_RECOVERY * (1 + BODY_REC_RECOVERY_BONUS * rec_norm)
const BODY_REST_ENERGY_RECOVERY: float = 0.006  ## base rest recovery/tick (> ENERGY_DECAY_RATE=0.003)
const BODY_REC_RECOVERY_BONUS: float   = 0.60   ## 최대 60% 회복 가속

## Disease resistance → mortality modifier
## dr_modifier = 1.0 - BODY_DR_MORTALITY_REDUCTION * dr_norm
const BODY_DR_MORTALITY_REDUCTION: float = 0.35  ## 최대 35% 사망률 감소

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
	var GameCalendar = load("res://scripts/core/simulation/game_calendar.gd")
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

## Feature flag: thirst/warmth/safety needs
## Set true once resource/tech systems are ready to support them
const NEEDS_EXPANSION_ENABLED: bool = false

## Entity need decay rates (per needs tick, adjusted for TICK_HOURS=2)
const HUNGER_DECAY_RATE: float = 0.002
## Metabolic curve: hunger decays slower when already hungry (Keys et al. 1950)
const HUNGER_METABOLIC_MIN: float = 0.3   # Minimum metabolic rate at hunger=0
const HUNGER_METABOLIC_RANGE: float = 0.7  # 1.0 - HUNGER_METABOLIC_MIN
const ENERGY_DECAY_RATE: float = 0.003
const ENERGY_ACTION_COST: float = 0.005
const SOCIAL_DECAY_RATE: float = 0.001

## [Maslow (1943) — L1 생리적 욕구: 수분]
## hunger 기준(0.002) × 1.2 = 0.0024 (탈수가 기아보다 빠름)
## 생물학적 근거: 수분 없이는 3일, 음식 없이는 3주
const THIRST_DECAY_RATE: float = 0.0024
const THIRST_DRINK_RESTORE: float = 0.35
const THIRST_CRITICAL: float = 0.15
const THIRST_LOW: float = 0.35

## [Cannon (1932) 항상성 — 체온 유지]
## 온난한 환경에서는 소모 거의 없고 추위/혹한에서 급증
const WARMTH_DECAY_RATE: float = 0.0016
const WARMTH_FIRE_RESTORE: float = 0.035
const WARMTH_SHELTER_RESTORE: float = 0.018
const WARMTH_CRITICAL: float = 0.10
const WARMTH_LOW: float = 0.30
const WARMTH_TEMP_NEUTRAL: float = 0.5
const WARMTH_TEMP_COLD: float = 0.3
const WARMTH_TEMP_FREEZING: float = 0.15

## [Maslow (1943) — L2 안전 욕구]
## hunger 기준 × 0.3 (느린 소모, 이벤트/환경 기반)
const SAFETY_DECAY_RATE: float = 0.0006
const SAFETY_SHELTER_RESTORE: float = 0.002
const SAFETY_CRITICAL: float = 0.15
const SAFETY_LOW: float = 0.35

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

## ━━ UPPER NEEDS SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Deci & Ryan 1985 SDT, Maslow 1943, Alderfer 1969 ERG, Bandura 1977 Self-Efficacy]
## 시스템이 5틱마다 실행됨 (UPPER_NEEDS_TICK_INTERVAL).
## 감쇠값 유도: decay_per_year / 1000 / 4380 * 5
const UPPER_NEEDS_TICK_INTERVAL: int = 5

## ── 감쇠 (5틱당) ──────────────────────────────────────────────────
const UPPER_NEEDS_COMPETENCE_DECAY:       float = 0.000149   ## 130/yr — 유능감
const UPPER_NEEDS_AUTONOMY_DECAY:         float = 0.000137   ## 120/yr — 자율성
const UPPER_NEEDS_SELF_ACTUATION_DECAY:   float = 0.0000913  ## 80/yr  — 자아실현
const UPPER_NEEDS_MEANING_DECAY:          float = 0.0000799  ## 70/yr  — 의미
const UPPER_NEEDS_RECOGNITION_DECAY:      float = 0.000171   ## 150/yr — 사회적 인정
const UPPER_NEEDS_BELONGING_DECAY:        float = 0.000285   ## 250/yr — 소속감
const UPPER_NEEDS_INTIMACY_DECAY:         float = 0.000205   ## 180/yr — 친밀감

## ── 충족 (5틱당) ──────────────────────────────────────────────────
## 목표 균형점: 활동 중인 에이전트 0.65–0.80 유지
const UPPER_NEEDS_COMPETENCE_JOB_GAIN:           float = 0.000200  ## 비율 1.34× — 직업 보유 시
const UPPER_NEEDS_AUTONOMY_JOB_GAIN:             float = 0.000150  ## 비율 1.09× — 직업 보유 시
const UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN:     float = 0.000325  ## 비율 1.14× — 정착지 소속 시
const UPPER_NEEDS_INTIMACY_PARTNER_GAIN:         float = 0.000250  ## 비율 1.22× — 파트너 있을 시
const UPPER_NEEDS_RECOGNITION_SKILL_COEFF:       float = 0.000200  ## × (best_skill/100) — lv100=1.17×
const UPPER_NEEDS_MEANING_BASE_GAIN:             float = 0.0000250 ## 항상 소량 회복
const UPPER_NEEDS_MEANING_ALIGNED_GAIN:          float = 0.0000900 ## × alignment(0–1) — 정합 시 1.44×
const UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF:    float = 0.000100  ## × (best_skill/100) — lv100=1.09×

## ── 레벨업 일회성 보너스 ─────────────────────────────────────────
## [Bandura 1977] 숙련 달성 이벤트 → 즉각 자기효능감 스파이크
const UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS: float = 0.08
const UPPER_NEEDS_SKILLUP_SELF_ACT_BONUS:   float = 0.05

## ━━ LEADER SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Weber 1922 Charismatic Authority, French & Raven 1959 Social Power]

## [Boehm 1999] Re-election interval per settlement (3 years).
## First election is IMMEDIATE when leader_id == -1.
const LEADER_REELECTION_INTERVAL: int = 13140  ## 3 years × 4380 ticks/year

## How often LeaderSystem checks for vacant leader / re-election due (ticks).
const LEADER_CHECK_INTERVAL: int = 12          ## ~1 day — lightweight check

## Minimum adult members required to elect a leader.
const LEADER_MIN_POPULATION: int = 3

## Charisma score tie-breaking: if top candidates are within this margin, use POPULARITY.
const LEADER_CHARISMA_TIE_MARGIN: float = 0.05

## [French & Raven 1959, Boehm 1999, Henrich & Gil-White 2001]
## Composite leader score weights — primitive/tribal era balanced across authority bases.
const LEADER_W_CHARISMA: float = 0.25          ## Referent power — ability to inspire
const LEADER_W_WISDOM: float = 0.15            ## Expert power — sound judgment
const LEADER_W_TRUSTWORTHINESS: float = 0.15   ## Social trust — reliability
const LEADER_W_INTIMIDATION: float = 0.15      ## Coercive power — physical authority
const LEADER_W_SOCIAL_CAPITAL: float = 0.15    ## Network — relationships in settlement
const LEADER_W_AGE_RESPECT: float = 0.15       ## Traditional — elder deference

## [Milgram 1974] Conformity pressure reduction for wise agents.
## Effective enforcement = base_enforcement × (1.0 - LEADER_WISDOM_RESISTANCE_COEFF × wisdom_norm)
## wisdom_norm = DERIVED_WISDOM / 1000.0 (0.0~1.0)
## A fully wise agent (wisdom=1.0) resists 30% of conformity pressure.
const LEADER_WISDOM_RESISTANCE_COEFF: float = 0.30

## ━━ INTELLIGENCE SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Gardner 1983 labels + CHC/Visser 2006 g-factor hybrid]

## ── Intelligence Generation ────────────────────────────
const INTEL_G_MEAN: float = 0.50
const INTEL_G_SD: float = 0.15
const INTEL_RESIDUAL_SD: float = 0.12

## g-loading weights per intelligence [Visser 2006]
const INTEL_G_LOADING: Dictionary = {
	"linguistic": 0.70, "logical": 0.75, "spatial": 0.65,
	"musical": 0.30, "kinesthetic": 0.15,
	"naturalistic": 0.60, "interpersonal": 0.45, "intrapersonal": 0.40,
}

## Sex difference shifts (applied to potentials) [Voyer 1995, Hyde 1988]
const INTEL_SEX_DIFF_MALE: Dictionary = {
	"spatial": 0.11,
}
const INTEL_SEX_DIFF_FEMALE: Dictionary = {
	"linguistic": 0.017,
}

## Heritability per group
const INTEL_HERITABILITY_G: float = 0.60
const INTEL_HERITABILITY_FLUID: float = 0.55
const INTEL_HERITABILITY_CRYSTALLIZED: float = 0.50
const INTEL_HERITABILITY_PHYSICAL: float = 0.60

## HEXACO → g influence (Openness r≈0.20 → weight 0.10)
const INTEL_OPENNESS_G_WEIGHT: float = 0.10

## ── Development Curves [Salthouse 2009, 2012] ──────────
const INTEL_GROUP_FLUID: Array = ["logical", "spatial"]
const INTEL_GROUP_CRYSTALLIZED: Array = ["linguistic", "musical", "interpersonal", "intrapersonal", "naturalistic"]
const INTEL_GROUP_PHYSICAL: Array = ["kinesthetic"]

## Piecewise curve breakpoints: [age, modifier]. 1.0 = young adult baseline.
const INTEL_CURVE_FLUID: Array = [
	[0, 0.20], [5, 0.50], [15, 0.85], [22, 1.00],
	[35, 1.00], [55, 0.85], [75, 0.60], [100, 0.50],
]
const INTEL_CURVE_CRYSTALLIZED: Array = [
	[0, 0.15], [5, 0.30], [15, 0.55], [25, 0.75],
	[50, 0.95], [65, 1.00], [80, 0.85], [100, 0.75],
]
const INTEL_CURVE_PHYSICAL: Array = [
	[0, 0.10], [5, 0.35], [12, 0.65], [20, 0.90],
	[28, 1.00], [40, 0.85], [60, 0.60], [80, 0.45], [100, 0.35],
]

## ── Environment Modifiers ───────────────────────────────
## [Georgieff 2007] Early childhood nutrition damage
const INTEL_NUTRITION_CRIT_AGE_TICKS: int = 730
const INTEL_NUTRITION_HUNGER_THRESHOLD: float = 0.3
const INTEL_NUTRITION_MAX_PENALTY: float = 0.15
const INTEL_NUTRITION_PENALTY_PER_TICK: float = 0.0003

## [Lupien 2009] ACE-based cognitive damage
const INTEL_ACE_SCARS_THRESHOLD_MINOR: int = 1
const INTEL_ACE_SCARS_THRESHOLD_MAJOR: int = 3
const INTEL_ACE_PENALTY_MINOR: float = 0.07
const INTEL_ACE_PENALTY_MAJOR: float = 0.15
const INTEL_ACE_CRIT_AGE_YEARS: float = 12.0
const INTEL_ACE_FLUID_DECLINE_MULT: float = 1.5

## [Lupien 2009] Acute stress → learning penalty
const INTEL_STRESS_LEARNING_THRESHOLD_LOW: float = 0.6
const INTEL_STRESS_LEARNING_PENALTY_LOW: float = 0.85
const INTEL_STRESS_LEARNING_THRESHOLD_HIGH: float = 0.8
const INTEL_STRESS_LEARNING_PENALTY_HIGH: float = 0.70

## [Hertzog 2009] Cognitive activity buffer
const INTEL_ACTIVITY_SKILL_THRESHOLD: int = 10
const INTEL_ACTIVITY_BUFFER: float = 0.70
const INTEL_INACTIVITY_ACCEL: float = 1.20

## ── Skill Learning ──────────────────────────────────────
const INTEL_LEARN_MULT_M: float = 0.35
const INTEL_LEARN_MULT_K: float = 2.0
const INTEL_CONSCIENTIOUSNESS_WEIGHT: float = 0.15

## ── Cholesky Residual Correlation Matrix ─────────────────
## 8×8 residual correlation (after removing g contribution)
## Order: LIN, LOG, SPA, MUS, KIN, NAT, INTER, INTRA
const INTEL_RESIDUAL_CORR: Array = [
	[1.00, 0.08, 0.05, 0.06, 0.03, 0.05, 0.08, 0.07],
	[0.08, 1.00, 0.09, 0.04, 0.02, 0.06, 0.03, 0.04],
	[0.05, 0.09, 1.00, 0.06, 0.08, 0.07, 0.03, 0.03],
	[0.06, 0.04, 0.06, 1.00, 0.15, 0.04, 0.06, 0.10],
	[0.03, 0.02, 0.08, 0.15, 1.00, 0.06, 0.03, 0.03],
	[0.05, 0.06, 0.07, 0.04, 0.06, 1.00, 0.05, 0.05],
	[0.08, 0.03, 0.03, 0.06, 0.03, 0.05, 1.00, 0.37],
	[0.07, 0.04, 0.03, 0.10, 0.03, 0.05, 0.37, 1.00],
]

## ━━ REPUTATION SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Fiske 2007, Nowak & Sigmund 2005, Baumeister 2001]

const REPUTATION_TICK_INTERVAL: int = 30

## [Fiske 2007] Overall reputation weights
const REP_W_MORALITY: float = 0.30
const REP_W_SOCIABILITY: float = 0.20
const REP_W_COMPETENCE: float = 0.25
const REP_W_DOMINANCE: float = 0.05
const REP_W_GENEROSITY: float = 0.20

## [Baumeister 2001, Rothbart & Park 1986] Negativity bias multipliers per domain
const REP_NEG_BIAS_MORALITY: float = 2.5
const REP_NEG_BIAS_SOCIABILITY: float = 1.2
const REP_NEG_BIAS_COMPETENCE: float = 1.5
const REP_NEG_BIAS_DOMINANCE: float = 1.0
const REP_NEG_BIAS_GENEROSITY: float = 2.0

## Decay rates (per game year, applied fractionally each reputation tick)
## [Klein 1992] Positive impression half-life ~3 years
const REP_POSITIVE_YEARLY_RETENTION: float = 0.794
## [Walker 2003] Negative impression half-life ~5 years
const REP_NEGATIVE_YEARLY_RETENTION: float = 0.870

## [Dunbar 1997] Probability that a social interaction includes gossip
const REP_GOSSIP_PROBABILITY: float = 0.65
## [Ohtsuki & Iwasa 2004] Hop-by-hop credibility decay
const REP_GOSSIP_HOP_DECAY: float = 0.80
const REP_GOSSIP_MAX_HOPS: int = 3
## Direct observation base credibility
const REP_DIRECT_OBSERVATION_CREDIBILITY: float = 0.90

## Gossip distortion rates by motive [Beersma & Van Kleef 2012, Giardini 2022]
const REP_DISTORTION_PROSOCIAL: float = 0.07
const REP_DISTORTION_ENJOYMENT: float = 0.15
const REP_DISTORTION_MANIPULATION: float = 0.25
const REP_DISTORTION_VENTING: float = 0.20

## Gossip transmission probability by domain (morality spreads fastest)
const REP_GOSSIP_TRANSMIT_MORALITY: float = 0.45
const REP_GOSSIP_TRANSMIT_SOCIABILITY: float = 0.20
const REP_GOSSIP_TRANSMIT_COMPETENCE: float = 0.25
const REP_GOSSIP_TRANSMIT_DOMINANCE: float = 0.30
const REP_GOSSIP_TRANSMIT_GENEROSITY: float = 0.35

## Event delta scaling — multiplied with valence*magnitude per reputation event
const REP_EVENT_DELTA_SCALE: float = 0.50
## Gossip delta scaling — multiplied with credibility when transferring reputation via gossip
const REP_GOSSIP_DELTA_SCALE: float = 0.35

## [Gottman 1994] Reputation recovery: positive acts needed per negative act
const REP_RECOVERY_RATIO: float = 5.0

## Status tier thresholds (overall reputation score)
const REP_TIER_RESPECTED: float = 0.60
const REP_TIER_GOOD: float = 0.20
const REP_TIER_SUSPECT: float = -0.20
const REP_TIER_OUTCAST: float = -0.60

## ━━ ECONOMIC TENDENCY SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Kahneman & Tversky 1979, Modigliani 1966, Engel 2011, Piff 2010]

const ECON_TICK_INTERVAL: int = 120

## Wealth score formula weights
const WEALTH_W_FOOD: float = 0.55
const WEALTH_W_WOOD: float = 0.25
const WEALTH_W_STONE: float = 0.20

## [Piff 2010] Wealth→generosity feedback: high wealth reduces generosity
const ECON_WEALTH_GENEROSITY_PENALTY: float = 0.90

## [Dittmar 2014] Materialism→Joy penalty
const ECON_MATERIALISM_JOY_THRESHOLD: float = 0.70
const ECON_MATERIALISM_JOY_PENALTY: float = 3.0

## Theft temptation constants
const THEFT_SCARCITY_FOOD_DAYS: float = 3.0

## ━━ JOB SATISFACTION SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Holland 1959, Hackman & Oldham 1976, Deci & Ryan 1985, Judge 2001]

const JOB_SAT_TICK_INTERVAL: int = 120

## Satisfaction composite weights
const JOB_SAT_W_SKILL_FIT: float = 0.35
const JOB_SAT_W_VALUE_FIT: float = 0.25
const JOB_SAT_W_PERSONALITY_FIT: float = 0.25
const JOB_SAT_W_NEED_FIT: float = 0.15

## Satisfaction → work speed multipliers [Judge 2001, r≈0.30]
const JOB_SAT_HIGH_THRESHOLD: float = 0.70
const JOB_SAT_HIGH_SPEED_MULT: float = 1.15
const JOB_SAT_LOW_THRESHOLD: float = 0.40
const JOB_SAT_LOW_SPEED_MULT: float = 0.90
const JOB_SAT_CRITICAL_THRESHOLD: float = 0.25
const JOB_SAT_CRITICAL_SPEED_MULT: float = 0.80

## [Judge 2001, r≈-0.40] Drift probability per season when dissatisfied
const JOB_SAT_DRIFT_BASE: float = 0.15

## ━━ STRATIFICATION MONITOR ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Boehm 1999, Kohler 2017, Scheidel 2017]

const STRAT_TICK_INTERVAL: int = 500

## Status score weights
const STATUS_W_REPUTATION: float = 0.35
const STATUS_W_WEALTH: float = 0.25
const STATUS_W_LEADER: float = 0.20
const STATUS_W_AGE: float = 0.10
const STATUS_W_COMPETENCE: float = 0.10

## Leader bonus values
const STATUS_LEADER_CURRENT: float = 0.30
const STATUS_LEADER_FORMER: float = 0.15

## Status tier thresholds
const STATUS_TIER_ELITE: float = 0.65
const STATUS_TIER_RESPECTED: float = 0.35
const STATUS_TIER_MARGINAL: float = -0.35
const STATUS_TIER_OUTCAST: float = -0.60

## [Boehm 1999, Dunbar 1998] Leveling effectiveness parameters
const LEVELING_DUNBAR_N: float = 150.0
const LEVELING_SEDENTISM_DEFAULT: float = 0.80

## [Kohler 2017, Scheidel 2017] Gini instability thresholds
const GINI_UNREST_THRESHOLD: float = 0.40
const GINI_ENTRENCHED_THRESHOLD: float = 0.50
const GINI_CRISIS_THRESHOLD: float = 0.60


