extends Node

## World constants
const WORLD_SIZE := Vector2i(256, 256)
const TILE_SIZE: int = 16
const CHUNK_SIZE: int = 32

## Simulation parameters
const TICKS_PER_SECOND: int = 10
const MAX_ENTITIES: int = 500
const INITIAL_SPAWN_COUNT: int = 20
const SPAWN_BATCH_SIZE: int = 5  ## 스폰 분산 — 한 프레임에 스폰할 최대 엔티티 수
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

## ── Body Potential Heritability (Bouchard & McGue 2003, Visscher 2008) ──────
## h² per axis: fraction of child potential explained by mid-parent value
## Remainder comes from population-level random draw (environmental + independent assortment)
const BODY_HERITABILITY: Dictionary = {
	"str": 0.75,   ## Strength: strong additive genetic component (Zempo 2017)
	"agi": 0.70,   ## Agility: high h² but sensitive to early motor environment
	"end": 0.72,   ## Endurance: VO2max h²=0.50–0.80 (Bouchard 1999 HERITAGE)
	"tou": 0.73,   ## Toughness: bone density h²=0.60–0.80 (Ralston 2000)
	"rec": 0.68,   ## Recuperation: HRV heritability ~0.64 (Singh 2017)
	"dr":  0.65,   ## Disease resistance: innate immunity h²=0.60–0.70 (Brodin 2015)
}
const BODY_TRAINABILITY_HERITABILITY: float = 0.50  ## ACTN3/ACE talent h²≈0.47 (Heritage 1999)
const BODY_MUTATION_RATE: float = 0.01              ## 1% chance of large mutation per axis
const BODY_MUTATION_SD: float = 0.30                ## mutation magnitude as fraction of mean

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

## [Anderson 1982 ACT*] Skill-unlocked action XP values
const SKILL_XP_HERB_GATHER: float   = 1.0   ## per herb_gather completion (lower than normal foraging)
const SKILL_XP_FINE_WOODWORK: float = 1.5   ## per fine_woodwork completion
const SKILL_XP_ORE_VEIN: float      = 1.5   ## per ore_vein completion
const SKILL_XP_TRAP_HUNT: float     = 3.0   ## per trap check (higher because slower action)

## ── speed/strength 파생 공식 ──────────────────────────────
## entity.speed = float(agi_realized) * BODY_SPEED_SCALE + BODY_SPEED_BASE
## agi=700(평균 성인, 훈련없음): speed=1.14,  agi=1500(전사): speed=2.1
const BODY_SPEED_BASE: float = 0.30
const BODY_SPEED_SCALE: float = 0.0012   ## float(agi_realized) * 0.0012
## entity.strength = float(str_realized) / 1000.0  → str=700: 0.70, str=1500: 1.50
const BODY_REALIZED_MAX: int = 15000     ## realized 정규화 기준 (str/agi/end/tou/rec), UI _draw_bar용
const BODY_REALIZED_DR_MAX: int = 10000  ## dr realized 정규화 기준, UI _draw_bar용

## ── Layer 1.5: Appearance Generation (Eagly 1991, Stulp 2015) ────────────────
const APPEARANCE_ATTRACT_MEAN: float = 0.50
const APPEARANCE_ATTRACT_SD:   float = 0.12
const APPEARANCE_HEIGHT_MEAN:  float = 0.50
const APPEARANCE_HEIGHT_SD:    float = 0.12
const APPEARANCE_HEIGHT_SD_CLAMP_LOW:  float = 0.05
const APPEARANCE_HEIGHT_SD_CLAMP_HIGH: float = 0.95
## Sex delta: males slightly taller on average (normalized scale)
const APPEARANCE_HEIGHT_SEX_DELTA_MALE: float = 0.04
const APPEARANCE_ATTRACT_HERITABILITY: float = 0.80
const APPEARANCE_HEIGHT_HERITABILITY:  float = 0.85
## Hair color weights [Brown=35%, Black=25%, Blonde=25%, Red=10%, Grey=3%, White=2%]
const HAIR_COLOR_WEIGHTS: Dictionary = {
	"black": 25, "brown": 35, "blonde": 25, "red": 10, "grey": 3, "white": 2
}
## Eye color weights [Brown=55%, Blue=20%, Green=15%, Grey=7%, Hazel=3%]
const EYE_COLOR_WEIGHTS: Dictionary = {
	"brown": 55, "blue": 20, "green": 15, "grey": 7, "hazel": 3
}
const DISTINGUISHING_MARK_CHANCE: float = 0.05
const DISTINGUISHING_MARK_IDS: Array = [
	"MARK_SCAR_FACE", "MARK_SCAR_HAND", "MARK_BIRTHMARK_NECK",
	"MARK_BIRTHMARK_CHEEK", "MARK_FRECKLES", "MARK_DIMPLES",
	"MARK_PROMINENT_NOSE", "MARK_SHARP_EYES"
]

## ── Layer 7: Speech Style (Human Definition v3 §13) ──────────────────────────
const SPEECH_TONE_VALUES:      Array = ["aggressive", "gentle", "formal", "casual", "sarcastic"]
const SPEECH_VERBOSITY_VALUES: Array = ["taciturn", "normal", "talkative"]
const SPEECH_HUMOR_VALUES:     Array = ["dry", "none", "witty", "slapstick"]

## ── Layer 7: Preferences (Linden et al. 2010) ────────────────────────────────
const PREFERENCE_FOOD_OPTIONS:   Array = ["food", "herbs", "meat"]
const PREFERENCE_COLOR_OPTIONS:  Array = ["red", "orange", "yellow", "green", "blue", "purple", "brown", "white", "black"]
const PREFERENCE_SEASON_OPTIONS: Array = ["spring", "summer", "autumn", "winter"]
const PREFERENCE_DISLIKE_IDS:    Array = [
	"DISLIKE_COLD", "DISLIKE_RAIN", "DISLIKE_CROWDS", "DISLIKE_SILENCE",
	"DISLIKE_HEIGHTS", "DISLIKE_CONFINED", "DISLIKE_CONFLICT", "DISLIKE_IDLENESS"
]
## Social event: affinity gain for matching preference
const SOCIAL_SHARED_PREFERENCE_AFFINITY_GAIN: float = 3.0

## ── Layer 6: Memory System [Baddeley & Hitch 1974, Ebbinghaus 1885] ───────────
## Max working memory entries before oldest (lowest intensity) are evicted
const MEMORY_WORKING_MAX: int = 100
## Intensity threshold for promotion to permanent_history
const MEMORY_PERMANENT_THRESHOLD: float = 0.5
## Ebbinghaus curve: intensity *= exp(-DECAY_RATE * dt_years)
const MEMORY_DECAY_TRIVIAL: float  = 1.386  ## half-life 0.5 years (ln2/0.5)
const MEMORY_DECAY_MODERATE: float = 0.347  ## half-life 2 years
const MEMORY_DECAY_STRONG: float   = 0.139  ## half-life 5 years
const MEMORY_DECAY_TRAUMA: float   = 0.014  ## half-life 50 years (near-permanent)
## [Conway & Pleydell-Pearce 2000] intensity at encoding by event type
const MEMORY_INTENSITY_MAP: Dictionary = {
	"casual_talk":   0.05,
	"share_food":    0.10,
	"deep_talk":     0.25,
	"argument":      0.30,
	"helped_work":   0.15,
	"comforted":     0.35,
	"flirt":         0.40,
	"proposal":      0.70,
	"marriage":      0.90,
	"child_born":    0.85,
	"partner_died":  0.95,
	"betrayal":      0.80,
	"trauma":        0.90,
	"promotion":     0.65,
	"achievement":   0.60,
	"migration":     0.55,
	"skill_unlock":  0.30,
	"first_meeting": 0.20,
}
## Decay rate lookup [threshold, rate] — first match wins (descending threshold)
const MEMORY_DECAY_BY_INTENSITY: Array = [
	[0.80, 0.014],  ## trauma-class: 50-year half-life
	[0.50, 0.139],  ## strong: 5-year half-life
	[0.20, 0.347],  ## moderate: 2-year half-life
	[0.00, 1.386],  ## trivial: 0.5-year half-life
]
## Event types allowed in permanent_history
const MEMORY_PERMANENT_TYPES: Array = [
	"birth", "marriage", "child_born", "partner_died", "war",
	"migration", "promotion", "betrayal", "trauma", "achievement", "proposal",
]
## Compression: annual, group same-type same-target entries > 1 year old
const MEMORY_COMPRESS_MIN_GROUP: int = 3
const MEMORY_COMPRESS_INTERVAL_TICKS: int = 4380  ## 1 year (= TICKS_PER_YEAR)

## ── Social Network Tie Classification [Granovetter 1973] ────────────────────
const NETWORK_TIE_WEAK_MIN:     float = 5.0
const NETWORK_TIE_MODERATE_MIN: float = 30.0
const NETWORK_TIE_STRONG_MIN:   float = 60.0
const NETWORK_TIE_INTIMATE_MIN: float = 85.0

## Social capital formula [Burt 2004]: strong×3 + weak×1 + bridge×5 + rep×10
const NETWORK_SOCIAL_CAP_STRONG_W: float = 3.0
const NETWORK_SOCIAL_CAP_WEAK_W:   float = 1.0
const NETWORK_SOCIAL_CAP_BRIDGE_W: float = 5.0
const NETWORK_SOCIAL_CAP_REP_W:    float = 10.0
## Normalization divisor — social capital ÷ this = 0~1 for ~100-person settlement
const NETWORK_SOCIAL_CAP_NORM_DIV: float = 200.0

## Information propagation probabilities [Granovetter 1973]
const NETWORK_PROPAGATION_STRONG: float = 0.80
const NETWORK_PROPAGATION_WEAK:   float = 0.20
const NETWORK_PROPAGATION_BRIDGE: float = 0.50

## ── Weber Authority Type [Weber 1922] ────────────────────────────────────────
const AUTHORITY_TRADITIONAL_TRADITION_MIN: float = 0.30
const AUTHORITY_TRADITIONAL_LAW_MAX:       float = 0.10
const AUTHORITY_RATIONAL_LAW_MIN:          float = 0.30
const AUTHORITY_TRADITIONAL_AGE_BOOST:     float = 0.15
const AUTHORITY_RATIONAL_TRUST_BOOST:      float = 0.15

## ── Obedience Formula [Milgram 1963] ─────────────────────────────────────────
const OBEDIENCE_W_AUTHORITY:         float = 0.25
const OBEDIENCE_W_AGREEABLENESS:     float = 0.20
const OBEDIENCE_W_CONSCIENTIOUSNESS: float = 0.15
const OBEDIENCE_W_LAW_VALUE:         float = 0.15
const OBEDIENCE_W_PROXIMITY:         float = 0.10
const OBEDIENCE_W_SOCIAL_PRESSURE:   float = 0.15
const OBEDIENCE_RESIST_THRESHOLD:    float = 0.30
const OBEDIENCE_CONFLICT_THRESHOLD:  float = 0.50

## ── Revolution Risk [Tilly 1978, Human Definition v3 §17] ───────────────────
const REVOLUTION_RISK_THRESHOLD:       float = 0.70
const REVOLUTION_COOLDOWN_TICKS:       int   = 8760   ## 2 years
const REVOLUTION_CHARISMA_MULTIPLIER:  float = 2.0
const REVOLUTION_TICK_INTERVAL:        int   = 4380   ## check annually

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
func tick_to_date(tick: int) -> Dictionary:
	var GameCalendar = load("res://scripts/core/simulation/game_calendar.gd")
	return GameCalendar.tick_to_date(tick)


## Convert age in ticks to years (float)
func get_age_years(age_ticks: int) -> float:
	return float(age_ticks) / float(TICKS_PER_YEAR)


## Get age stage string from age in ticks (6 stages)
func get_age_stage(age_ticks: int) -> String:
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

## Simulation runtime backend modes.
## gdscript: 기존 GDScript 실행.
## rust_shadow: GDScript 실행 + Rust 런타임 shadow 검증.
## rust_primary: Rust 런타임 실행.
const SIM_RUNTIME_MODE_GDSCRIPT: String = "gdscript"
const SIM_RUNTIME_MODE_RUST_SHADOW: String = "rust_shadow"
const SIM_RUNTIME_MODE_RUST_PRIMARY: String = "rust_primary"
const SIM_RUNTIME_MODE: String = SIM_RUNTIME_MODE_RUST_SHADOW

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
const CAMERA_ZOOM_MAX: float = 10.0
const CAMERA_ZOOM_STEP: float = 0.15
const CAMERA_PAN_SPEED: float = 500.0
const CAMERA_ZOOM_SPEED: float = 0.15

## System tick intervals
const NEEDS_TICK_INTERVAL: int = 4
const STRESS_SYSTEM_TICK_INTERVAL: int = 4
const BEHAVIOR_TICK_INTERVAL: int = 10
const MOVEMENT_TICK_INTERVAL: int = 3

## ── Phase B-3: Stress → Work Efficiency [Yerkes-Dodson 1908, McEwen 2004] ──
## Stress efficiency curve: maps stress level to action_timer multiplier.
## Higher multiplier = slower actions (longer timer).
## Inverted-U: slight boost at low stress, penalty at high stress.
const STRESS_EFFICIENCY_EUSTRESS_PEAK: float = 150.0    ## Optimal stress for peak performance
const STRESS_EFFICIENCY_EUSTRESS_BONUS: float = 0.90     ## Timer multiplier at eustress (0.9 = 10% faster)
const STRESS_EFFICIENCY_DISTRESS_START: float = 400.0    ## Stress level where penalties begin
const STRESS_EFFICIENCY_SEVERE_START: float = 700.0      ## Stress level where severe penalties begin
const STRESS_EFFICIENCY_MAX_PENALTY: float = 1.60        ## Maximum timer multiplier (60% slower)

## Feature flag: thirst/warmth/safety needs
## Set true once resource/tech systems are ready to support them
const NEEDS_EXPANSION_ENABLED: bool = true  ## was false — enabled Prompt 01

## ── Debug Panel ─────────────────────────────────────────────────────────────
## Set false before shipping. When false, F12 does nothing.
const DEBUG_PANEL_ENABLED: bool = true

## ── Debug log flags ─────────────────────────────────────────────────────────
## 기본값 전부 false. 개발 시 필요한 플래그만 true로 변경.
const DEBUG_STRESS_LOG:          bool = false  ## stress_system 매틱 로그
const DEBUG_MENTAL_BREAK_LOG:    bool = false  ## 멘탈 브레이크 발생/종료
const DEBUG_TRAUMA_LOG:          bool = false  ## 트라우마 스카 획득/재활성화
const DEBUG_TRAIT_VIOLATION_LOG: bool = false  ## 특성 위반 스트레스 이벤트
const DEBUG_DEMOGRAPHY_LOG:      bool = false  ## 인구/출생/사망 주기 리포트
const DEBUG_BALANCE_LOG:         bool = false  ## 500틱마다 밸런스 스냅샷
const DEBUG_EVENT_LOG:           bool = false  ## event_logger Gathered/spawned/born/died 콘솔 출력

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

## [Alderfer 1969 ERG] Frustration-regression thresholds and multipliers
## If a growth/relatedness need stays below THRESHOLD for ERG_FRUSTRATION_WINDOW ticks,
## the agent over-invests in the corresponding lower need.
const ERG_FRUSTRATION_WINDOW: int = 300          ## ticks of sustained deficit = ~1 in-game month
const ERG_GROWTH_FRUSTRATION_THRESHOLD: float = 0.25   ## competence/autonomy/self_actualization < this
const ERG_RELATEDNESS_FRUSTRATION_THRESHOLD: float = 0.25  ## belonging/intimacy < this
const ERG_EXISTENCE_SCORE_BOOST: float = 0.30    ## multiplier boost to gather_food/gather_wood when regressing
const ERG_RELATEDNESS_SCORE_BOOST: float = 0.25  ## multiplier boost to socialize/visit_partner when regressing
const ERG_STRESS_INJECT_RATE: float = 1.5        ## stress/tick injected during active regression

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

## ── Transcendence Need (Maslow 1969, Koltko-Rivera 2006) ─────────────────────
## Decay: ~40/year — slowest upper need (most stable once achieved, Maslow 1969)
const UPPER_NEEDS_TRANSCENDENCE_DECAY: float = 0.0000456
## Fulfillment: community membership + sacrifice-value alignment (Putnam 2000, Koltko-Rivera 2006)
const UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN: float = 0.0000200
const UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF: float = 0.0000600
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

## [Frederick et al. 2002, Zuckerman 1979, Trivers 1971, Richins & Dawson 1992]
## Economic tendency → behavior utility additive weights
## Format: { action_id: { tendency_key: weight, ... }, ... }
## weight 부호: + = 성향 높을수록 행동 점수 상승, - = 반대
const ECON_BEHAVIOR_WEIGHTS: Dictionary = {
	"gather_food":          { "saving": 0.15, "materialism": 0.10 },
	"gather_wood":          { "saving": 0.15, "materialism": 0.10 },
	"gather_stone":         { "saving": 0.10, "materialism": 0.10 },
	"herb_gather":          { "saving": 0.05, "materialism": 0.05 },
	"trap_hunt":            { "risk":   0.20, "materialism": 0.05 },
	"wander":               { "risk":   0.15, "saving": -0.05 },
	"rest":                 { "saving": -0.10, "risk": -0.05, "materialism": -0.05 },
	"sit_by_fire":          { "saving": -0.05, "risk": -0.10 },
	"socialize":            { "generosity": 0.15, "materialism": -0.10 },
	"visit_partner":        { "generosity": 0.10, "materialism": -0.05 },
	"deliver_to_stockpile": { "saving": 0.20, "generosity": 0.15 },
	"take_from_stockpile":  { "saving": -0.10, "generosity": -0.10, "materialism": 0.15 },
	"seek_shelter":         { "saving": 0.05, "risk": -0.15 },
	"build":                { "saving": 0.10, "materialism": 0.10 },
	"fine_woodwork":        { "saving": 0.05, "materialism": 0.05 },
	"ore_vein":             { "saving": 0.05, "materialism": 0.08 },
}

## ━━ ECONOMIC BEHAVIOR THRESHOLDS [Layer 4.7] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Modigliani 1966] Saving tendency → deliver score adjustment
const ECON_DELIVER_BASE_SCORE: float = 0.60          ## Base delivery score (carry > threshold)
const ECON_DELIVER_SAVING_SCALE: float = 0.35        ## saving × this added to deliver score
const ECON_DELIVER_MATERIALISM_SUPPRESS: float = 0.25 ## materialism × this subtracted from deliver score
## [Kasser & Ryan 1993] Materialism hoarding: carry threshold before delivering
## Default carry > 3.0. Materialistic agents only deliver at much higher carry.
const ECON_HOARD_MATERIALISM_THRESHOLD: float = 0.40  ## materialism above this → hoarding mode
const ECON_HOARD_CARRY_MULTIPLIER: float = 2.0        ## carry threshold multiplied by this when hoarding
## [Trivers 1971 / Engel 2011] Generosity sharing trigger
const ECON_SHARE_GENEROSITY_THRESHOLD: float = 0.30   ## generosity must exceed this to trigger sharing
const ECON_SHARE_NEIGHBOR_HUNGER_THRESHOLD: float = 0.35  ## target must be this hungry to be worth sharing
const ECON_SHARE_MIN_SURPLUS: float = 2.0             ## agent must have at least this much food to share
const ECON_SHARE_FOOD_AMOUNT: float = 1.0             ## food units transferred per share action
const ECON_SHARE_SCORE_BASE: float = 0.50             ## base action score for share_food
const ECON_SHARE_SCORE_GENEROSITY_SCALE: float = 0.40 ## generosity × this added to share_food score

## ━━ BLOOD TYPE SYSTEM [Layer 7 — ABO Genetics] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## 초기 집단 스폰 시 혈액형 표현형 가중치 [세계 인구 기준 근사치]
const BLOOD_TYPE_SPAWN_WEIGHTS: Dictionary = {"O": 45, "A": 40, "B": 11, "AB": 4}

## 표현형 → 초기 유전자형 분포 가중치 (초기 스폰용)
## A형: 2/3 확률로 AO (이형), 1/3 확률로 AA (동형)
const BLOOD_GENOTYPE_FROM_PHENOTYPE: Dictionary = {
	"A":  {"AA": 33, "AO": 67},
	"B":  {"BB": 33, "BO": 67},
	"AB": {"AB": 100},
	"O":  {"OO": 100}
}

## 두 유전자형 교배 시 자녀 유전자형 확률 테이블 (알파벳순 정렬 키)
const BLOOD_CROSS_TABLE: Dictionary = {
	"AA_AA": {"AA": 100},
	"AA_AO": {"AA": 50, "AO": 50},
	"AA_AB": {"AA": 50, "AB": 50},
	"AA_BB": {"AB": 100},
	"AA_BO": {"AB": 50, "AO": 50},
	"AA_OO": {"AO": 100},
	"AB_AB": {"AA": 25, "AB": 50, "BB": 25},
	"AB_OO": {"AO": 50, "BO": 50},
	"AO_AB": {"AA": 25, "AB": 25, "AO": 25, "BO": 25},
	"AO_AO": {"AA": 25, "AO": 50, "OO": 25},
	"AO_BB": {"AB": 50, "BO": 50},
	"AO_BO": {"AB": 25, "AO": 25, "BO": 25, "OO": 25},
	"AO_OO": {"AO": 50, "OO": 50},
	"BB_BB": {"BB": 100},
	"BB_AB": {"AB": 50, "BB": 50},
	"BB_BO": {"BB": 50, "BO": 50},
	"BB_OO": {"BO": 100},
	"BO_AB": {"AB": 25, "AO": 25, "BB": 25, "BO": 25},
	"BO_BO": {"BB": 25, "BO": 50, "OO": 25},
	"BO_OO": {"BO": 50, "OO": 50},
	"OO_OO": {"OO": 100}
}

## 유전자형 → 표현형 매핑
const BLOOD_GENOTYPE_TO_PHENOTYPE: Dictionary = {
	"AA": "A", "AO": "A", "BB": "B", "BO": "B", "AB": "AB", "OO": "O"
}

## ━━ ZODIAC SYSTEM [Layer 7 — GameCalendar 그레고리력 기반] ━━━━━━━━━━━━━━━━━
const ZODIAC_SIGN_NAMES: Array = [
	"aries", "taurus", "gemini", "cancer", "leo", "virgo",
	"libra", "scorpio", "sagittarius", "capricorn", "aquarius", "pisces"
]

## ━━ VALUE HERITABILITY [Knafo & Schwartz 2004, Plomin 1994] ━━━━━━━━━━━━━━━
## 가치별 유전율 h_v ∈ [0.10, 0.20]
## 문화 의존 큰 가치(전통/권위/순응) → 0.10~0.12
## 자기절제/근면 등 기질 연관 → 0.18~0.20
## 중간(공정/협력) → 0.13~0.17
const VALUE_HERITABILITY: Dictionary = {
	## 문화 의존 높음 (h_v ≈ 0.10~0.12)
	"TRADITION":    0.10, "LAW":         0.10, "DECORUM":      0.11,
	"LOYALTY":      0.11, "STOICISM":    0.11, "HARMONY":      0.12,
	"PEACE":        0.12, "FAMILY":      0.12, "COOPERATION":  0.12,
	"SACRIFICE":    0.12,
	## 중간 (h_v ≈ 0.13~0.17)
	"FAIRNESS":     0.13, "FRIENDSHIP":  0.13, "TRUTH":        0.14,
	"INTROSPECTION":0.14, "TRANQUILITY": 0.14, "COMMERCE":     0.15,
	"KNOWLEDGE":    0.15, "INDEPENDENCE":0.15, "NATURE":       0.15,
	"SKILL":        0.15, "ROMANCE":     0.15, "CRAFTSMANSHIP":0.16,
	"ELOQUENCE":    0.16, "ARTWORK":     0.16, "MERRIMENT":    0.16,
	"LEISURE":      0.17, "PERSEVERANCE":0.17,
	## 기질 연관 높음 (h_v ≈ 0.18~0.20)
	"SELF_CONTROL": 0.18, "HARD_WORK":   0.18, "CUNNING":      0.18,
	"COMPETITION":  0.19, "MARTIAL_PROWESS": 0.19, "POWER":    0.20,
}

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

## ━━ OCCUPATION SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Holland 1959 RIASEC, Super 1957 Career Development]
## Skill-based occupation: agent's highest skill becomes their profession.

const OCCUPATION_EVAL_INTERVAL: int = 240			## ticks between occupation re-evaluation (~20 days)
const OCCUPATION_MIN_SKILL_LEVEL: int = 10			## minimum skill level to count as occupation
const OCCUPATION_CHANGE_HYSTERESIS: float = 0.15	## new skill must exceed current by this margin (normalized) to trigger change
const OCCUPATION_SPECIALIZATION_BONUS: float = 1.2	## XP multiplier for practicing occupation skill

## ━━ TITLE SYSTEM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
## [Barth 1969 ethnic boundary theory, Turner 1974 social drama]
## Titles: accumulated social markers granted by condition.

const TITLE_EVAL_INTERVAL: int = 500				## ticks between title evaluation
const TITLE_ELDER_MIN_AGE_YEARS: float = 55.0		## minimum age for Elder title
const TITLE_MASTER_SKILL_LEVEL: int = 75			## skill level for "Master X" title
const TITLE_EXPERT_SKILL_LEVEL: int = 50			## skill level for "Expert X" title
const TITLE_VETERAN_BATTLES: int = 5				## battles survived for Veteran title

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

## [Ainsworth 1978 + Mikulincer & Shaver 2007]
## Multipliers applied to socialize action score by attachment type.
## Anxious: hyperactivated system → drives more frequent social seeking.
## Avoidant: deactivated system → suppresses social approach despite internal need.
const ATTACHMENT_SOCIALIZE_MULT: Dictionary = {
	"secure":       1.00,
	"anxious":      1.45,
	"avoidant":     0.55,
	"disorganized": 1.00,
}

## Social need recovery efficiency per attachment type.
## Anxious: contact never fully satisfies → partial recovery only.
## Avoidant: contact provides less comfort → reduced recovery.
const ATTACHMENT_SOCIAL_RECOVERY_MULT: Dictionary = {
	"secure":       1.00,
	"anxious":      0.65,
	"avoidant":     0.70,
	"disorganized": 0.80,
}

## Avoidant allostatic accumulation multiplier (Mikulincer 1998).
const ATTACHMENT_AVOIDANT_ALLO_MULT: float = 2.0

## Background stress rate for anxious attachment when social need is low (Cassidy & Berlin 1994).
const ATTACHMENT_ANXIOUS_STRESS_RATE: float = 0.02
const ATTACHMENT_ANXIOUS_STRESS_THRESHOLD: float = 0.40

## ── Tech Tree [Henrich 2004, Boyd & Richerson 1985] ──────────────────────────
## Scan interval: tech discovery checked monthly (was annual=4380)
## Monthly granularity gives first check at tick 365 (~12s at 3x speed)
## TechDiscoverySystem converts base_chance_per_year → per-check automatically
const TECH_DISCOVERY_INTERVAL_TICKS: int = 365  ## 1 month
## Discovery probability scaling
const TECH_DISCOVERY_POP_SCALE:   float = 0.005  ## +0.5% per person over pop_minimum
const TECH_DISCOVERY_MAX_BONUS:   float = 0.40   ## max bonus from all modifiers combined
## Era progression: era changes to "tribal" when these techs are all discovered
const TECH_ERA_TRIBAL_REQUIRES: Array = ["TECH_FIRE_MAKING", "TECH_BASIC_TOOLS", "TECH_SHELTER_BUILDING"]
## ── Tech V2 [Henrich 2004, Boyd & Richerson 1985, Tainter 1988] ────────────
## Scan radius (tiles) around settlement center for biome/resource checks
const TECH_BIOME_SCAN_RADIUS: int = 15
## Bonus to discovery chance per discovered soft prerequisite
const TECH_SOFT_PREREQ_BONUS: float = 0.05
## Effective carriers added per matching institution tag
const TECH_INSTITUTION_CARRIER_BONUS: int = 3
## Effective carriers added per matching artifact (building/item)
const TECH_ARTIFACT_CARRIER_VALUE: int = 2
## Threshold: effective_carriers / min_practitioners ratio for KNOWN_STABLE
const TECH_STABLE_THRESHOLD: float = 1.5
## Memory threshold below which forgotten_recent -> forgotten_long
const TECH_LONG_FORGOTTEN_MEMORY: float = 0.3
## Era progression: era changes to "bronze_age" when these techs are all discovered
const TECH_ERA_BRONZE_AGE_REQUIRES: Array = [
	"TECH_TRIBAL_ORGANIZATION", "TECH_POTTERY",
	"TECH_AGRICULTURE", "TECH_NATIVE_COPPER_SMELTING",
]
## Era directories for tech JSON loading (V2 replaces V1 TECH_DATA_DIRS)
const TECH_DATA_DIRS_V2: Array = [
	"res://data/tech/stone_age/",
	"res://data/tech/tribal/",
	"res://data/tech/bronze_age/",
]

## ── Tech Maintenance / Regression [Henrich 2004, Tainter 1988] ───────────────
## Atrophy: how quickly knowledge degrades when undermaintained
const TECH_ATROPHY_BASE_RATE: float = 1.0          ## +1.0 atrophy_years per year when under threshold
const TECH_ATROPHY_RECOVERY_RATE: float = 0.5       ## atrophy_years recovered per year when above threshold
## Cultural memory floor (0.0–1.0)
const TECH_CULTURAL_MEMORY_FLOOR: float = 0.05      ## never fully forgotten (oral legends persist)
## State transition thresholds
const TECH_KNOWN_STABLE_THRESHOLD_YEARS: float = 5.0   ## years at 0 atrophy → KNOWN_LOW upgrades to KNOWN_STABLE
const TECH_FORGOTTEN_RECENT_YEARS: float = 10.0        ## years in FORGOTTEN_RECENT → transitions to FORGOTTEN_LONG
## Population factor [Henrich 2004]: larger populations slow atrophy
const TECH_POP_MAINTENANCE_BONUS: float = 0.01   ## per person above min_practitioners → slows atrophy
const TECH_POP_MAINTENANCE_CAP: float = 0.5       ## max atrophy reduction from population
## Active use bonus
const TECH_ACTIVE_USE_ATROPHY_REDUCTION: float = 0.3  ## if actively used, atrophy rate × (1.0 - 0.3)
## Artifact/institution grace period extension
const TECH_ARTIFACT_GRACE_BONUS: float = 0.2    ## per artifact building → extends grace period multiplier
const TECH_FORGOTTEN_LONG_DECAY_MULTIPLIER: float = 0.4  ## oral legends fade slowly in FORGOTTEN_LONG state

## ── Tech Utilization [White 1959, Diamond 1997, Durkheim 1893] ─────────────────
## Modifier stacking caps (prevent runaway values)
const TECH_MODIFIER_STACK_CAP: float = 10.0            ## Maximum multiplier from stacking
const TECH_MODIFIER_ADDITIVE_STACK_CAP: float = 500.0  ## Max additive bonus (e.g., population cap)
const TECH_RECALC_COOLDOWN_TICKS: int = 5              ## Minimum ticks between recalculations

## Modifier target classification — determines how stacking works
const TECH_MODIFIER_MULTIPLIER_TARGETS: Array = [
	"food_production", "wood_production", "stone_production", "metal_production",
	"craft_quality", "craft_speed", "build_speed", "build_quality",
	"combat_effectiveness", "defense_strength", "weapon_quality", "armor_quality",
	"trade_efficiency", "storage_capacity", "knowledge_retention", "learning_speed",
	"disease_resistance", "healing_rate",
]
const TECH_MODIFIER_ADDITIVE_TARGETS: Array = [
	"population_cap", "trade_range", "max_building_tier", "settlement_stability",
	"lifespan_modifier",
]

## Era progression thresholds (active known techs per era tier)
const TECH_ERA_STONE_AGE_COUNT: int = 0      ## Starts here
const TECH_ERA_TRIBAL_COUNT: int = 5         ## Need 5 tribal-era techs known for era transition
const TECH_ERA_BRONZE_AGE_COUNT: int = 12    ## Need 12 bronze-era techs known

## Era base modifiers — settlement-wide bonuses from era alone
const TECH_ERA_MODIFIERS: Dictionary = {
	"stone_age":  {"population_cap": 20, "trade_range": 0, "max_building_tier": 1},
	"tribal":     {"population_cap": 100, "trade_range": 5, "max_building_tier": 2},
	"bronze_age": {"population_cap": 500, "trade_range": 15, "max_building_tier": 3},
}

## ── Tech Propagation [Lave & Wenger 1991, Vygotsky 1978, Rogers 2003] ────────
## Within-settlement: teacher-student pair model
const TEACHING_TICK_INTERVAL: int = 24                ## Check every game-day (24 ticks = 2 days)
const TEACHING_BASE_EFFECTIVENESS: float = 0.02       ## Base progress per teaching tick
const TEACHING_SKILL_GAP_MIN: int = 3                 ## Teacher must be ≥3 levels above student
const TEACHING_SKILL_GAP_OPTIMAL: int = 5             ## Sweet spot for ZPD
const TEACHING_SKILL_GAP_MAX: int = 10                ## Beyond this, teacher "too advanced" penalty
const TEACHING_MAX_STUDENTS: int = 3                  ## One teacher, max 3 students simultaneously
const TEACHING_WILLINGNESS_THRESHOLD: float = 0.3     ## Minimum relationship affinity to teach
const TEACHING_SESSION_TICKS: int = 72                ## 3 game-days per learning cycle
const TEACHING_ABANDON_TICKS: int = 480               ## 20 days without progress → abandon

## Cross-settlement propagation [Diamond 1997, Granovetter 1973, Boyd & Richerson 2005]
const CROSS_PROP_TRADE_BASE: float = 0.05             ## Base chance per trade interaction
const CROSS_PROP_MIGRATION_BASE: float = 0.8          ## Migrant brings knowledge (high)
const CROSS_PROP_WAR_CAPTURE_BASE: float = 0.3        ## Captured artisan chance
const CROSS_PROP_DIPLOMACY_BASE: float = 0.1          ## Diplomatic exchange chance
const CROSS_PROP_LANGUAGE_THRESHOLD: float = 0.6      ## language_divergence above this → severe penalty
const CROSS_PROP_LANGUAGE_BLOCK: float = 0.9          ## Above this → propagation impossible without translator

## Adoption curve thresholds [Rogers 2003 Diffusion of Innovations]
const ADOPTION_INNOVATOR_PCT: float = 0.025           ## 2.5% — already covered by discoverer
const ADOPTION_EARLY_ADOPTER_PCT: float = 0.16        ## 16% cumulative
const ADOPTION_EARLY_MAJORITY_PCT: float = 0.50       ## 50% cumulative
const ADOPTION_LATE_MAJORITY_PCT: float = 0.84        ## 84% cumulative

## Personality thresholds for adoption willingness
const ADOPTION_OPENNESS_WEIGHT: float = 0.35
const ADOPTION_CURIOSITY_WEIGHT: float = 0.25
const ADOPTION_CONSCIENTIOUSNESS_WEIGHT: float = 0.20
const ADOPTION_KNOWLEDGE_VALUE_WEIGHT: float = 0.20

## ── Settlement Detail Panel [C-1h] ───────────────────────────────────────────
## Auto-refresh interval while panel is open (in simulation ticks)
const SETTLEMENT_PANEL_REFRESH_TICKS: int = 60
## Max practitioners shown before "and N more..." truncation
const SETTLEMENT_PANEL_MAX_PRACTITIONERS: int = 20
## Age bracket definitions for population tab
const SETTLEMENT_PANEL_AGE_BRACKETS: Array = [
	{"label_key": "UI_AGE_CHILD", "min": 0, "max": 11},
	{"label_key": "UI_AGE_TEEN", "min": 12, "max": 17},
	{"label_key": "UI_AGE_YOUNG_ADULT", "min": 18, "max": 29},
	{"label_key": "UI_AGE_MIDDLE", "min": 30, "max": 54},
	{"label_key": "UI_AGE_ELDER", "min": 55, "max": 999},
]

## ── World Statistics Panel [C-1i] ────────────────────────────────────────────
## Auto-refresh interval while panel is open (in simulation ticks)
const STATS_PANEL_REFRESH_INTERVAL: int = 120
## Resource supply thresholds (days of supply at current consumption)
const STATS_RESOURCE_DANGER_DAYS: float = 7.0
const STATS_RESOURCE_LOW_DAYS: float = 30.0
const STATS_RESOURCE_ABUNDANT_DAYS: float = 90.0
## Recent events display limits
const STATS_RECENT_EVENTS_MAX: int = 20
const STATS_RECENT_PERIOD_TICKS: int = 365

## ── Combat System [Keeley 1996, Human Definition v3 §19] ─────────────────────
## Body part integrity thresholds for death
const COMBAT_HEAD_DEATH_THRESHOLD:  float = 0.70  ## head integrity < 0.30 → death
const COMBAT_TORSO_DEATH_THRESHOLD: float = 0.80  ## torso integrity < 0.20 → death
## Limb damage penalties
const COMBAT_LIMB_SPEED_PENALTY:    float = 0.30  ## −30% speed per damaged limb
const COMBAT_LIMB_STR_PENALTY:      float = 0.25  ## −25% effective strength per damaged limb
## Base weapon damage (no equipment era — improvised clubs/spears)
const COMBAT_BASE_WEAPON_DAMAGE:    float = 0.15
const COMBAT_BASE_ARMOR:            float = 0.0   ## no armor in stone age
## Morale thresholds [Human Definition v3 §19]
const COMBAT_MORALE_ROUT_THRESHOLD:   float = 0.20  ## morale < 0.2 → full rout (flee)
const COMBAT_MORALE_SHAKEN_THRESHOLD: float = 0.40  ## morale < 0.4 → shaken (−50% combat)
## Morale contributions
const COMBAT_MORALE_W_HAPPINESS:    float = 0.30
const COMBAT_MORALE_W_CHARISMA:     float = 0.30
const COMBAT_MORALE_W_CAUSE_BELIEF: float = 0.40
## Random roll range for duel resolution
const COMBAT_ROLL_RANDOM_RANGE:     float = 0.30

## ── Inter-settlement Tension [Tilly 1978, Keeley 1996] ──────────────────────
## Tension per pair: 0.0 (neutral) → 1.0 (raid imminent)
const TENSION_CHECK_INTERVAL_TICKS: int   = 2190     ## twice yearly
const TENSION_RESOURCE_DEFICIT_TRIGGER: float = 0.30 ## if settlement food/need ratio < this, tension rises
const TENSION_PROXIMITY_RADIUS:     int   = 20       ## tiles — only settlements within this range develop tension
const TENSION_PER_SHARED_RESOURCE:  float = 0.05     ## per tick where both settlements harvest same tile
const TENSION_DECAY_PER_YEAR:       float = 0.15     ## natural decay if no resource conflict
const TENSION_SKIRMISH_THRESHOLD:   float = 0.60     ## tension > 0.60 → skirmish event possible
const TENSION_SKIRMISH_CHANCE:      float = 0.35     ## probability of skirmish when above threshold
const TENSION_SKIRMISH_COOLDOWN:    int   = 4380     ## 1 year minimum between skirmishes per pair
## Post-skirmish tension change
const TENSION_WINNER_REDUCTION:     float = 0.30     ## attacker win → tension drops (grievance resolved)
const TENSION_LOSER_INCREASE:       float = 0.20     ## loser retaliates → tension rises

## --- 맵 에디터 ---
const MAP_EDITOR_BRUSH_MIN: int = 1
const MAP_EDITOR_BRUSH_MAX: int = 10
const MAP_EDITOR_BRUSH_DEFAULT: int = 3
const MAP_EDITOR_STRENGTH_MIN: float = 0.5
const MAP_EDITOR_STRENGTH_MAX: float = 2.0
const MAP_EDITOR_SPAWN_MIN: int = 5
const MAP_EDITOR_SPAWN_MAX: int = 100
const MAP_EDITOR_SPAWN_DEFAULT: int = 20

## 프리셋별 고정 시드 (재현 가능한 기본 맵)
const PRESET_SEED_ISLAND: int = 42001
const PRESET_SEED_CONTINENT: int = 42002
const PRESET_SEED_ARCHIPELAGO: int = 42003

## 프리셋 생성 파라미터
const PRESET_ISLAND_LAND_RATIO: float = 0.45
const PRESET_CONTINENT_LAND_RATIO: float = 0.70
const PRESET_ARCHIPELAGO_ISLAND_COUNT: int = 5

## 바이옴별 기본 elevation (브러시 TERRAIN 모드에서 사용)
const BIOME_DEFAULT_ELEVATION: Dictionary = {
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

## 바이옴별 기본 moisture (브러시 TERRAIN 모드에서 사용)
const BIOME_DEFAULT_MOISTURE: Dictionary = {
	0: 0.9,   ## DEEP_WATER
	1: 0.8,   ## SHALLOW_WATER
	2: 0.3,   ## BEACH
	3: 0.4,   ## GRASSLAND
	4: 0.55,  ## FOREST
	5: 0.75,  ## DENSE_FOREST
	6: 0.35,  ## HILL
	7: 0.3,   ## MOUNTAIN
	8: 0.2,   ## SNOW
}
