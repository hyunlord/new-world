/// GameConfig — Rust port of `scripts/core/simulation/game_config.gd`
///
/// All simulation-relevant constants ported verbatim.
/// UI-only constants (font sizes, biome colors, camera settings) are omitted.
/// Dictionary constants with string keys are provided as helper methods.
/// Dictionary constants with axis keys (body, intel) are provided as arrays indexed by enum.
use serde::{Deserialize, Serialize};

// ── World ─────────────────────────────────────────────────────────────────────
pub const WORLD_WIDTH: u32 = 256;
pub const WORLD_HEIGHT: u32 = 256;
pub const TILE_SIZE: u32 = 16;
pub const CHUNK_SIZE: u32 = 32;

// ── Simulation Parameters ─────────────────────────────────────────────────────
pub const TICKS_PER_SECOND: u32 = 10;
pub const MAX_ENTITIES: u32 = 500;
pub const INITIAL_SPAWN_COUNT: u32 = 20;
pub const SPAWN_BATCH_SIZE: u32 = 5;
pub const MAX_TICKS_PER_FRAME: u32 = 5;

// ── Time ──────────────────────────────────────────────────────────────────────
/// 1 tick = 2 game hours
pub const TICK_HOURS: u32 = 2;
/// 12 ticks = 1 day
pub const TICKS_PER_DAY: u32 = 12;
/// 365 days = 1 year
pub const DAYS_PER_YEAR: u32 = 365;
/// ~30.4 days × 12 ticks/day (display utility only)
pub const TICKS_PER_MONTH: u32 = 365;
/// 365 × 12 = 4380 ticks per year
pub const TICKS_PER_YEAR: u32 = 4380;

// ── Age Stage Thresholds (in ticks) ──────────────────────────────────────────
/// 6 stages: infant ≤2y, toddler 3-5y, child 6-11y, teen 12-14y, adult 15-55y, elder 56+
pub const AGE_INFANT_END: u64 = 13140;   // 3 years
pub const AGE_TODDLER_END: u64 = 26280;  // 6 years
pub const AGE_CHILD_END: u64 = 52560;    // 12 years
pub const AGE_TEEN_END: u64 = 65700;     // 15 years
pub const AGE_ADULT_END: u64 = 245280;   // 56 years
pub const AGE_MAX: u64 = 525600;         // 120 years (theoretical max)
pub const PREGNANCY_DURATION: u64 = 3360;      // 280 days × 12 ticks/day
pub const PREGNANCY_DURATION_STDEV: u64 = 120; // ~10 days × 12 ticks/day

// ── Body Potential (int scale 0~10,000) ──────────────────────────────────────
pub const BODY_POTENTIAL_MEAN: i32 = 1050;
pub const BODY_POTENTIAL_SD: i32 = 175;
pub const BODY_POTENTIAL_MIN: i32 = 50;
pub const BODY_POTENTIAL_MAX: i32 = 10000;

/// Sex delta (male) per axis: str, agi, end, tou, rec, dr
/// Order matches BodyAxis: [str, agi, end, tou, rec, dr]
pub const BODY_SEX_DELTA_MALE: [i32; 6] = [160, 30, -15, 100, -15, -80];

/// Body potential heritability h² per axis [Bouchard 2003, Visscher 2008]
/// Order: [str=0.75, agi=0.70, end=0.72, tou=0.73, rec=0.68, dr=0.65]
pub const BODY_HERITABILITY: [f64; 6] = [0.75, 0.70, 0.72, 0.73, 0.68, 0.65];

pub const BODY_TRAINABILITY_HERITABILITY: f64 = 0.50;
pub const BODY_MUTATION_RATE: f64 = 0.01;
pub const BODY_MUTATION_SD: f64 = 0.30;

// ── Trainability (int scale 0~1,000) ─────────────────────────────────────────
pub const TRAINABILITY_MEAN: i32 = 500;
pub const TRAINABILITY_SD: i32 = 150;
pub const TRAINABILITY_MIN: i32 = 50;
pub const TRAINABILITY_MAX: i32 = 1000;

// ── Innate Immunity (int scale 0~1,000) ──────────────────────────────────────
pub const INNATE_IMMUNITY_MEAN: i32 = 500;
pub const INNATE_IMMUNITY_SD: i32 = 100;
pub const INNATE_IMMUNITY_SEX_DELTA_FEMALE: i32 = 80;

// ── Body Training XP ─────────────────────────────────────────────────────────
pub const XP_FOR_FULL_PROGRESS: f64 = 10000.0;
pub const GATHER_XP_END: f64 = 0.50;
pub const GATHER_XP_STR: f64 = 0.30;
pub const GATHER_XP_AGI: f64 = 0.20;
pub const CONSTRUCT_XP_STR: f64 = 0.80;
pub const CONSTRUCT_XP_TOU: f64 = 0.30;
pub const CONSTRUCT_XP_AGI: f64 = 0.20;

// ── Skill XP per action ───────────────────────────────────────────────────────
pub const SKILL_XP_FORAGING: f64 = 2.0;
pub const SKILL_XP_WOODCUTTING: f64 = 2.0;
pub const SKILL_XP_MINING: f64 = 2.0;
pub const SKILL_XP_CONSTRUCTION: f64 = 3.0;
pub const SKILL_XP_HUNTING: f64 = 2.0;
pub const SKILL_XP_HERB_GATHER: f64 = 1.0;
pub const SKILL_XP_FINE_WOODWORK: f64 = 1.5;
pub const SKILL_XP_ORE_VEIN: f64 = 1.5;
pub const SKILL_XP_TRAP_HUNT: f64 = 3.0;

// ── Body Derived Stats ────────────────────────────────────────────────────────
pub const BODY_SPEED_BASE: f64 = 0.30;
pub const BODY_SPEED_SCALE: f64 = 0.0012;
pub const BODY_REALIZED_MAX: i32 = 15000;
pub const BODY_REALIZED_DR_MAX: i32 = 10000;
pub const BODY_END_COST_REDUCTION: f64 = 0.40;
pub const BODY_REST_ENERGY_RECOVERY: f64 = 0.006;
pub const BODY_REC_RECOVERY_BONUS: f64 = 0.60;
pub const BODY_DR_MORTALITY_REDUCTION: f64 = 0.35;

// ── Body XP per activity ──────────────────────────────────────────────────────
pub const BODY_XP_GATHER_FOOD: f64 = 0.8;
pub const BODY_XP_GATHER_WOOD: f64 = 1.2;
pub const BODY_XP_GATHER_STONE: f64 = 1.5;
pub const BODY_XP_BUILD: f64 = 1.0;
pub const BODY_XP_REST: f64 = 0.3;

// ── Appearance [Eagly 1991, Stulp 2015] ──────────────────────────────────────
pub const APPEARANCE_ATTRACT_MEAN: f64 = 0.50;
pub const APPEARANCE_ATTRACT_SD: f64 = 0.12;
pub const APPEARANCE_HEIGHT_MEAN: f64 = 0.50;
pub const APPEARANCE_HEIGHT_SD: f64 = 0.12;
pub const APPEARANCE_HEIGHT_SEX_DELTA_MALE: f64 = 0.04;
pub const APPEARANCE_ATTRACT_HERITABILITY: f64 = 0.80;
pub const APPEARANCE_HEIGHT_HERITABILITY: f64 = 0.85;
pub const DISTINGUISHING_MARK_CHANCE: f64 = 0.05;

// ── Memory System [Baddeley 1974, Ebbinghaus 1885] ───────────────────────────
pub const MEMORY_WORKING_MAX: usize = 100;
pub const MEMORY_PERMANENT_THRESHOLD: f64 = 0.5;
pub const MEMORY_DECAY_TRIVIAL: f64 = 1.386;   // half-life 0.5 years
pub const MEMORY_DECAY_MODERATE: f64 = 0.347;  // half-life 2 years
pub const MEMORY_DECAY_STRONG: f64 = 0.139;    // half-life 5 years
pub const MEMORY_DECAY_TRAUMA: f64 = 0.014;    // half-life 50 years
pub const MEMORY_COMPRESS_MIN_GROUP: usize = 3;
pub const MEMORY_COMPRESS_INTERVAL_TICKS: u64 = 4380; // 1 year

/// Memory intensity at encoding by event type.
/// Returns the intensity value (0.0–1.0) for a known event type, or 0.05 as default.
pub fn memory_intensity(event_type: &str) -> f64 {
    match event_type {
        "casual_talk"    => 0.05,
        "share_food"     => 0.10,
        "deep_talk"      => 0.25,
        "argument"       => 0.30,
        "helped_work"    => 0.15,
        "comforted"      => 0.35,
        "flirt"          => 0.40,
        "proposal"       => 0.70,
        "marriage"       => 0.90,
        "child_born"     => 0.85,
        "partner_died"   => 0.95,
        "betrayal"       => 0.80,
        "trauma"         => 0.90,
        "promotion"      => 0.65,
        "achievement"    => 0.60,
        "migration"      => 0.55,
        "skill_unlock"   => 0.30,
        "first_meeting"  => 0.20,
        _                => 0.05,
    }
}

/// Ebbinghaus decay rate by intensity threshold.
/// Returns the decay rate for the first matching [threshold, rate] pair.
pub fn memory_decay_rate(intensity: f64) -> f64 {
    if intensity >= 0.80 { return 0.014; }  // trauma-class
    if intensity >= 0.50 { return 0.139; }  // strong
    if intensity >= 0.20 { return 0.347; }  // moderate
    1.386                                   // trivial
}

// ── Social Network [Granovetter 1973] ────────────────────────────────────────
pub const NETWORK_TIE_WEAK_MIN: f64 = 5.0;
pub const NETWORK_TIE_MODERATE_MIN: f64 = 30.0;
pub const NETWORK_TIE_STRONG_MIN: f64 = 60.0;
pub const NETWORK_TIE_INTIMATE_MIN: f64 = 85.0;

/// Social capital formula [Burt 2004]: strong×3 + weak×1 + bridge×5 + rep×10
pub const NETWORK_SOCIAL_CAP_STRONG_W: f64 = 3.0;
pub const NETWORK_SOCIAL_CAP_WEAK_W: f64 = 1.0;
pub const NETWORK_SOCIAL_CAP_BRIDGE_W: f64 = 5.0;
pub const NETWORK_SOCIAL_CAP_REP_W: f64 = 10.0;
pub const NETWORK_SOCIAL_CAP_NORM_DIV: f64 = 200.0;

pub const NETWORK_PROPAGATION_STRONG: f64 = 0.80;
pub const NETWORK_PROPAGATION_WEAK: f64 = 0.20;
pub const NETWORK_PROPAGATION_BRIDGE: f64 = 0.50;

pub const SOCIAL_SHARED_PREFERENCE_AFFINITY_GAIN: f64 = 3.0;

// ── Weber Authority [Weber 1922] ──────────────────────────────────────────────
pub const AUTHORITY_TRADITIONAL_TRADITION_MIN: f64 = 0.30;
pub const AUTHORITY_TRADITIONAL_LAW_MAX: f64 = 0.10;
pub const AUTHORITY_RATIONAL_LAW_MIN: f64 = 0.30;
pub const AUTHORITY_TRADITIONAL_AGE_BOOST: f64 = 0.15;
pub const AUTHORITY_RATIONAL_TRUST_BOOST: f64 = 0.15;

// ── Obedience [Milgram 1963] ──────────────────────────────────────────────────
pub const OBEDIENCE_W_AUTHORITY: f64 = 0.25;
pub const OBEDIENCE_W_AGREEABLENESS: f64 = 0.20;
pub const OBEDIENCE_W_CONSCIENTIOUSNESS: f64 = 0.15;
pub const OBEDIENCE_W_LAW_VALUE: f64 = 0.15;
pub const OBEDIENCE_W_PROXIMITY: f64 = 0.10;
pub const OBEDIENCE_W_SOCIAL_PRESSURE: f64 = 0.15;
pub const OBEDIENCE_RESIST_THRESHOLD: f64 = 0.30;
pub const OBEDIENCE_CONFLICT_THRESHOLD: f64 = 0.50;

// ── Revolution Risk [Tilly 1978] ──────────────────────────────────────────────
pub const REVOLUTION_RISK_THRESHOLD: f64 = 0.70;
pub const REVOLUTION_COOLDOWN_TICKS: u64 = 8760;   // 2 years
pub const REVOLUTION_CHARISMA_MULTIPLIER: f64 = 2.0;
pub const REVOLUTION_TICK_INTERVAL: u64 = 4380;    // annual check

// ── System Tick Intervals ─────────────────────────────────────────────────────
pub const NEEDS_TICK_INTERVAL: u64 = 4;
pub const STRESS_SYSTEM_TICK_INTERVAL: u64 = 4;
pub const BEHAVIOR_TICK_INTERVAL: u64 = 10;
pub const MOVEMENT_TICK_INTERVAL: u64 = 3;
pub const UPPER_NEEDS_TICK_INTERVAL: u64 = 5;

// ── Stress → Work Efficiency [Yerkes-Dodson 1908, McEwen 2004] ───────────────
pub const STRESS_EFFICIENCY_EUSTRESS_PEAK: f64 = 150.0;
pub const STRESS_EFFICIENCY_EUSTRESS_BONUS: f64 = 0.90;
pub const STRESS_EFFICIENCY_DISTRESS_START: f64 = 400.0;
pub const STRESS_EFFICIENCY_SEVERE_START: f64 = 700.0;
pub const STRESS_EFFICIENCY_MAX_PENALTY: f64 = 1.60;

// ── Needs Decay ───────────────────────────────────────────────────────────────
pub const HUNGER_DECAY_RATE: f64 = 0.002;
pub const HUNGER_METABOLIC_MIN: f64 = 0.3;
pub const HUNGER_METABOLIC_RANGE: f64 = 0.7;
pub const ENERGY_DECAY_RATE: f64 = 0.003;
pub const ENERGY_ACTION_COST: f64 = 0.005;
pub const SOCIAL_DECAY_RATE: f64 = 0.001;

/// [Maslow L1 — thirst]
pub const THIRST_DECAY_RATE: f64 = 0.0024;
pub const THIRST_DRINK_RESTORE: f64 = 0.35;
pub const THIRST_CRITICAL: f64 = 0.15;
pub const THIRST_LOW: f64 = 0.35;

/// [Cannon 1932 — warmth / homeostasis]
pub const WARMTH_DECAY_RATE: f64 = 0.0016;
pub const WARMTH_FIRE_RESTORE: f64 = 0.035;
pub const WARMTH_SHELTER_RESTORE: f64 = 0.018;
pub const WARMTH_CRITICAL: f64 = 0.10;
pub const WARMTH_LOW: f64 = 0.30;
pub const WARMTH_TEMP_NEUTRAL: f64 = 0.5;
pub const WARMTH_TEMP_COLD: f64 = 0.3;
pub const WARMTH_TEMP_FREEZING: f64 = 0.15;

/// [Maslow L2 — safety]
pub const SAFETY_DECAY_RATE: f64 = 0.0006;
pub const SAFETY_SHELTER_RESTORE: f64 = 0.002;
pub const SAFETY_CRITICAL: f64 = 0.15;
pub const SAFETY_LOW: f64 = 0.35;

// ── ERG Frustration [Alderfer 1969] ──────────────────────────────────────────
pub const ERG_FRUSTRATION_WINDOW: u64 = 300;
pub const ERG_GROWTH_FRUSTRATION_THRESHOLD: f64 = 0.25;
pub const ERG_RELATEDNESS_FRUSTRATION_THRESHOLD: f64 = 0.25;
pub const ERG_EXISTENCE_SCORE_BOOST: f64 = 0.30;
pub const ERG_RELATEDNESS_SCORE_BOOST: f64 = 0.25;
pub const ERG_STRESS_INJECT_RATE: f64 = 1.5;

// ── Eating ────────────────────────────────────────────────────────────────────
pub const STARVATION_GRACE_TICKS: u64 = 25;
pub const FOOD_HUNGER_RESTORE: f64 = 0.3;
pub const HUNGER_EAT_THRESHOLD: f64 = 0.5;

// ── Upper Needs [Deci & Ryan 1985, Maslow 1943] ───────────────────────────────
pub const UPPER_NEEDS_COMPETENCE_DECAY: f64 = 0.000149;
pub const UPPER_NEEDS_AUTONOMY_DECAY: f64 = 0.000137;
pub const UPPER_NEEDS_SELF_ACTUATION_DECAY: f64 = 0.0000913;
pub const UPPER_NEEDS_MEANING_DECAY: f64 = 0.0000799;
pub const UPPER_NEEDS_RECOGNITION_DECAY: f64 = 0.000171;
pub const UPPER_NEEDS_BELONGING_DECAY: f64 = 0.000285;
pub const UPPER_NEEDS_INTIMACY_DECAY: f64 = 0.000205;
pub const UPPER_NEEDS_TRANSCENDENCE_DECAY: f64 = 0.0000456;
pub const UPPER_NEEDS_COMPETENCE_JOB_GAIN: f64 = 0.000200;
pub const UPPER_NEEDS_AUTONOMY_JOB_GAIN: f64 = 0.000150;
pub const UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN: f64 = 0.000325;
pub const UPPER_NEEDS_INTIMACY_PARTNER_GAIN: f64 = 0.000250;
pub const UPPER_NEEDS_RECOGNITION_SKILL_COEFF: f64 = 0.000200;
pub const UPPER_NEEDS_MEANING_BASE_GAIN: f64 = 0.0000250;
pub const UPPER_NEEDS_MEANING_ALIGNED_GAIN: f64 = 0.0000900;
pub const UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN: f64 = 0.0000200;
pub const UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF: f64 = 0.0000600;
pub const UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF: f64 = 0.000100;
pub const UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS: f64 = 0.08;
pub const UPPER_NEEDS_SKILLUP_SELF_ACT_BONUS: f64 = 0.05;

// ── Childcare ─────────────────────────────────────────────────────────────────
pub const CHILDCARE_TICK_INTERVAL: u64 = 10;

// ── World Generation ──────────────────────────────────────────────────────────
pub const WORLD_SEED: u64 = 42;
pub const NOISE_OCTAVES: u32 = 5;
pub const ISLAND_FALLOFF: f64 = 0.7;

// ── Resources ────────────────────────────────────────────────────────────────
pub const FOOD_REGEN_RATE: f64 = 1.0;
pub const WOOD_REGEN_RATE: f64 = 0.3;
pub const STONE_REGEN_RATE: f64 = 0.0;
pub const RESOURCE_REGEN_TICK_INTERVAL: u64 = 120;

// ── Pathfinding ───────────────────────────────────────────────────────────────
pub const PATHFIND_MAX_STEPS: u32 = 200;

// ── Entity Inventory ──────────────────────────────────────────────────────────
pub const MAX_CARRY: f64 = 10.0;
pub const GATHER_AMOUNT: f64 = 2.0;

// ── Population ────────────────────────────────────────────────────────────────
pub const BIRTH_FOOD_COST: f64 = 3.0;
pub const POPULATION_TICK_INTERVAL: u64 = 30;
pub const JOB_ASSIGNMENT_TICK_INTERVAL: u64 = 24;
pub const GATHERING_TICK_INTERVAL: u64 = 3;
pub const CONSTRUCTION_TICK_INTERVAL: u64 = 5;
pub const BUILDING_EFFECT_TICK_INTERVAL: u64 = 10;

// ── Settlement & Migration ────────────────────────────────────────────────────
pub const SETTLEMENT_MIN_DISTANCE: i32 = 25;
pub const SETTLEMENT_BUILD_RADIUS: i32 = 15;
pub const BUILDING_MIN_SPACING: i32 = 2;
pub const MIGRATION_TICK_INTERVAL: u64 = 100;
pub const MIGRATION_MIN_POP: u32 = 40;
pub const MIGRATION_GROUP_SIZE_MIN: u32 = 5;
pub const MIGRATION_GROUP_SIZE_MAX: u32 = 7;
pub const MIGRATION_CHANCE: f64 = 0.05;
pub const MIGRATION_SEARCH_RADIUS_MIN: i32 = 30;
pub const MIGRATION_SEARCH_RADIUS_MAX: i32 = 80;
pub const MAX_SETTLEMENTS: u32 = 5;
pub const MIGRATION_COOLDOWN_TICKS: u64 = 500;
pub const MIGRATION_STARTUP_FOOD: f64 = 30.0;
pub const MIGRATION_STARTUP_WOOD: f64 = 10.0;
pub const MIGRATION_STARTUP_STONE: f64 = 3.0;
pub const SETTLEMENT_CLEANUP_INTERVAL: u64 = 250;

// ── Leader System [Weber 1922, French & Raven 1959] ──────────────────────────
pub const LEADER_REELECTION_INTERVAL: u64 = 13140; // 3 years
pub const LEADER_CHECK_INTERVAL: u64 = 12;
pub const LEADER_MIN_POPULATION: u32 = 3;
pub const LEADER_CHARISMA_TIE_MARGIN: f64 = 0.05;
pub const LEADER_W_CHARISMA: f64 = 0.25;
pub const LEADER_W_WISDOM: f64 = 0.15;
pub const LEADER_W_TRUSTWORTHINESS: f64 = 0.15;
pub const LEADER_W_INTIMIDATION: f64 = 0.15;
pub const LEADER_W_SOCIAL_CAPITAL: f64 = 0.15;
pub const LEADER_W_AGE_RESPECT: f64 = 0.15;
pub const LEADER_WISDOM_RESISTANCE_COEFF: f64 = 0.30;

// ── Intelligence System [Gardner 1983, CHC/Visser 2006] ──────────────────────
pub const INTEL_G_MEAN: f64 = 0.50;
pub const INTEL_G_SD: f64 = 0.15;
pub const INTEL_RESIDUAL_SD: f64 = 0.12;

/// g-loading per intelligence type [Visser 2006].
/// Order: [linguistic, logical, spatial, musical, kinesthetic, naturalistic, interpersonal, intrapersonal]
pub const INTEL_G_LOADING: [f64; 8] = [0.70, 0.75, 0.65, 0.30, 0.15, 0.60, 0.45, 0.40];

pub const INTEL_HERITABILITY_G: f64 = 0.60;
pub const INTEL_HERITABILITY_FLUID: f64 = 0.55;
pub const INTEL_HERITABILITY_CRYSTALLIZED: f64 = 0.50;
pub const INTEL_HERITABILITY_PHYSICAL: f64 = 0.60;
pub const INTEL_OPENNESS_G_WEIGHT: f64 = 0.10;

/// Sex shifts: male +0.11 on spatial (index 2)
pub const INTEL_SEX_DIFF_SPATIAL_MALE: f64 = 0.11;
/// Sex shifts: female +0.017 on linguistic (index 0)
pub const INTEL_SEX_DIFF_LINGUISTIC_FEMALE: f64 = 0.017;

pub const INTEL_NUTRITION_CRIT_AGE_TICKS: u64 = 730;
pub const INTEL_NUTRITION_HUNGER_THRESHOLD: f64 = 0.3;
pub const INTEL_NUTRITION_MAX_PENALTY: f64 = 0.15;
pub const INTEL_NUTRITION_PENALTY_PER_TICK: f64 = 0.0003;
pub const INTEL_ACE_SCARS_THRESHOLD_MINOR: u32 = 1;
pub const INTEL_ACE_SCARS_THRESHOLD_MAJOR: u32 = 3;
pub const INTEL_ACE_PENALTY_MINOR: f64 = 0.07;
pub const INTEL_ACE_PENALTY_MAJOR: f64 = 0.15;
pub const INTEL_ACE_CRIT_AGE_YEARS: f64 = 12.0;
pub const INTEL_ACE_FLUID_DECLINE_MULT: f64 = 1.5;
pub const INTEL_STRESS_LEARNING_THRESHOLD_LOW: f64 = 0.6;
pub const INTEL_STRESS_LEARNING_PENALTY_LOW: f64 = 0.85;
pub const INTEL_STRESS_LEARNING_THRESHOLD_HIGH: f64 = 0.8;
pub const INTEL_STRESS_LEARNING_PENALTY_HIGH: f64 = 0.70;
pub const INTEL_ACTIVITY_SKILL_THRESHOLD: u32 = 10;
pub const INTEL_ACTIVITY_BUFFER: f64 = 0.70;
pub const INTEL_INACTIVITY_ACCEL: f64 = 1.20;
pub const INTEL_LEARN_MULT_M: f64 = 0.35;
pub const INTEL_LEARN_MULT_K: f64 = 2.0;
pub const INTEL_CONSCIENTIOUSNESS_WEIGHT: f64 = 0.15;

/// 8×8 Cholesky residual correlation matrix for intelligence.
/// Order: LIN, LOG, SPA, MUS, KIN, NAT, INTER, INTRA
pub const INTEL_RESIDUAL_CORR: [[f64; 8]; 8] = [
    [1.00, 0.08, 0.05, 0.06, 0.03, 0.05, 0.08, 0.07],
    [0.08, 1.00, 0.09, 0.04, 0.02, 0.06, 0.03, 0.04],
    [0.05, 0.09, 1.00, 0.06, 0.08, 0.07, 0.03, 0.03],
    [0.06, 0.04, 0.06, 1.00, 0.15, 0.04, 0.06, 0.10],
    [0.03, 0.02, 0.08, 0.15, 1.00, 0.06, 0.03, 0.03],
    [0.05, 0.06, 0.07, 0.04, 0.06, 1.00, 0.05, 0.05],
    [0.08, 0.03, 0.03, 0.06, 0.03, 0.05, 1.00, 0.37],
    [0.07, 0.04, 0.03, 0.10, 0.03, 0.05, 0.37, 1.00],
];

// ── Reputation System [Fiske 2007, Nowak 2005] ────────────────────────────────
pub const REPUTATION_TICK_INTERVAL: u64 = 30;
pub const REP_W_MORALITY: f64 = 0.30;
pub const REP_W_SOCIABILITY: f64 = 0.20;
pub const REP_W_COMPETENCE: f64 = 0.25;
pub const REP_W_DOMINANCE: f64 = 0.05;
pub const REP_W_GENEROSITY: f64 = 0.20;
pub const REP_NEG_BIAS_MORALITY: f64 = 2.5;
pub const REP_NEG_BIAS_SOCIABILITY: f64 = 1.2;
pub const REP_NEG_BIAS_COMPETENCE: f64 = 1.5;
pub const REP_NEG_BIAS_DOMINANCE: f64 = 1.0;
pub const REP_NEG_BIAS_GENEROSITY: f64 = 2.0;
pub const REP_POSITIVE_YEARLY_RETENTION: f64 = 0.794;
pub const REP_NEGATIVE_YEARLY_RETENTION: f64 = 0.870;
pub const REP_GOSSIP_PROBABILITY: f64 = 0.65;
pub const REP_GOSSIP_HOP_DECAY: f64 = 0.80;
pub const REP_GOSSIP_MAX_HOPS: u32 = 3;
pub const REP_DIRECT_OBSERVATION_CREDIBILITY: f64 = 0.90;
pub const REP_DISTORTION_PROSOCIAL: f64 = 0.07;
pub const REP_DISTORTION_ENJOYMENT: f64 = 0.15;
pub const REP_DISTORTION_MANIPULATION: f64 = 0.25;
pub const REP_DISTORTION_VENTING: f64 = 0.20;
pub const REP_EVENT_DELTA_SCALE: f64 = 0.50;
pub const REP_GOSSIP_DELTA_SCALE: f64 = 0.35;
pub const REP_RECOVERY_RATIO: f64 = 5.0;
pub const REP_TIER_RESPECTED: f64 = 0.60;
pub const REP_TIER_GOOD: f64 = 0.20;
pub const REP_TIER_SUSPECT: f64 = -0.20;
pub const REP_TIER_OUTCAST: f64 = -0.60;

// ── Economic System ───────────────────────────────────────────────────────────
pub const ECON_TICK_INTERVAL: u64 = 120;
pub const WEALTH_W_FOOD: f64 = 0.55;
pub const WEALTH_W_WOOD: f64 = 0.25;
pub const WEALTH_W_STONE: f64 = 0.20;
pub const ECON_WEALTH_GENEROSITY_PENALTY: f64 = 0.90;
pub const ECON_MATERIALISM_JOY_THRESHOLD: f64 = 0.70;
pub const ECON_MATERIALISM_JOY_PENALTY: f64 = 3.0;
pub const ECON_DELIVER_BASE_SCORE: f64 = 0.60;
pub const ECON_DELIVER_SAVING_SCALE: f64 = 0.35;
pub const ECON_DELIVER_MATERIALISM_SUPPRESS: f64 = 0.25;
pub const ECON_HOARD_MATERIALISM_THRESHOLD: f64 = 0.40;
pub const ECON_HOARD_CARRY_MULTIPLIER: f64 = 2.0;
pub const ECON_SHARE_GENEROSITY_THRESHOLD: f64 = 0.30;
pub const ECON_SHARE_NEIGHBOR_HUNGER_THRESHOLD: f64 = 0.35;
pub const ECON_SHARE_MIN_SURPLUS: f64 = 2.0;
pub const ECON_SHARE_FOOD_AMOUNT: f64 = 1.0;
pub const ECON_SHARE_SCORE_BASE: f64 = 0.50;
pub const ECON_SHARE_SCORE_GENEROSITY_SCALE: f64 = 0.40;
pub const THEFT_SCARCITY_FOOD_DAYS: f64 = 3.0;

// ── Stratification [Boehm 1999, Scheidel 2017] ───────────────────────────────
pub const STRAT_TICK_INTERVAL: u64 = 500;
pub const STATUS_W_REPUTATION: f64 = 0.35;
pub const STATUS_W_WEALTH: f64 = 0.25;
pub const STATUS_W_LEADER: f64 = 0.20;
pub const STATUS_W_AGE: f64 = 0.10;
pub const STATUS_W_COMPETENCE: f64 = 0.10;
pub const STATUS_LEADER_CURRENT: f64 = 0.30;
pub const STATUS_LEADER_FORMER: f64 = 0.15;
pub const STATUS_TIER_ELITE: f64 = 0.65;
pub const STATUS_TIER_RESPECTED: f64 = 0.35;
pub const STATUS_TIER_MARGINAL: f64 = -0.35;
pub const STATUS_TIER_OUTCAST: f64 = -0.60;
pub const LEVELING_DUNBAR_N: f64 = 150.0;
pub const LEVELING_SEDENTISM_DEFAULT: f64 = 0.80;
pub const GINI_UNREST_THRESHOLD: f64 = 0.40;
pub const GINI_ENTRENCHED_THRESHOLD: f64 = 0.50;
pub const GINI_CRISIS_THRESHOLD: f64 = 0.60;

// ── Attachment [Ainsworth 1978, Mikulincer 2007] ──────────────────────────────
/// Socialize score multiplier per attachment type: [secure, anxious, avoidant, disorganized]
pub const ATTACHMENT_SOCIALIZE_MULT: [f64; 4] = [1.00, 1.45, 0.55, 1.00];
/// Social recovery multiplier per type: [secure, anxious, avoidant, disorganized]
pub const ATTACHMENT_SOCIAL_RECOVERY_MULT: [f64; 4] = [1.00, 0.65, 0.70, 0.80];
pub const ATTACHMENT_AVOIDANT_ALLO_MULT: f64 = 2.0;
pub const ATTACHMENT_ANXIOUS_STRESS_RATE: f64 = 0.02;
pub const ATTACHMENT_ANXIOUS_STRESS_THRESHOLD: f64 = 0.40;

// ── Job Satisfaction [Holland 1959, Hackman 1976] ─────────────────────────────
pub const JOB_SAT_TICK_INTERVAL: u64 = 120;
pub const JOB_SAT_W_SKILL_FIT: f64 = 0.35;
pub const JOB_SAT_W_VALUE_FIT: f64 = 0.25;
pub const JOB_SAT_W_PERSONALITY_FIT: f64 = 0.25;
pub const JOB_SAT_W_NEED_FIT: f64 = 0.15;
pub const JOB_SAT_HIGH_THRESHOLD: f64 = 0.70;
pub const JOB_SAT_HIGH_SPEED_MULT: f64 = 1.15;
pub const JOB_SAT_LOW_THRESHOLD: f64 = 0.40;
pub const JOB_SAT_LOW_SPEED_MULT: f64 = 0.90;
pub const JOB_SAT_CRITICAL_THRESHOLD: f64 = 0.25;
pub const JOB_SAT_CRITICAL_SPEED_MULT: f64 = 0.80;
pub const JOB_SAT_DRIFT_BASE: f64 = 0.15;

// ── Occupation [Holland 1959, Super 1957] ─────────────────────────────────────
pub const OCCUPATION_EVAL_INTERVAL: u64 = 240;
pub const OCCUPATION_MIN_SKILL_LEVEL: u32 = 10;
pub const OCCUPATION_CHANGE_HYSTERESIS: f64 = 0.15;
pub const OCCUPATION_SPECIALIZATION_BONUS: f64 = 1.2;

// ── Title System ──────────────────────────────────────────────────────────────
pub const TITLE_EVAL_INTERVAL: u64 = 500;
pub const TITLE_ELDER_MIN_AGE_YEARS: f64 = 55.0;
pub const TITLE_MASTER_SKILL_LEVEL: u32 = 75;
pub const TITLE_EXPERT_SKILL_LEVEL: u32 = 50;
pub const TITLE_VETERAN_BATTLES: u32 = 5;

// ── Tech System [Henrich 2004, Boyd & Richerson 1985] ────────────────────────
pub const TECH_DISCOVERY_INTERVAL_TICKS: u64 = 365; // monthly
pub const TECH_DISCOVERY_POP_SCALE: f64 = 0.005;
pub const TECH_DISCOVERY_MAX_BONUS: f64 = 0.40;
pub const TECH_BIOME_SCAN_RADIUS: i32 = 15;
pub const TECH_SOFT_PREREQ_BONUS: f64 = 0.05;
pub const TECH_INSTITUTION_CARRIER_BONUS: u32 = 3;
pub const TECH_ARTIFACT_CARRIER_VALUE: u32 = 2;
pub const TECH_STABLE_THRESHOLD: f64 = 1.5;
pub const TECH_LONG_FORGOTTEN_MEMORY: f64 = 0.3;
pub const TECH_ATROPHY_BASE_RATE: f64 = 1.0;
pub const TECH_ATROPHY_RECOVERY_RATE: f64 = 0.5;
pub const TECH_CULTURAL_MEMORY_FLOOR: f64 = 0.05;
pub const TECH_KNOWN_STABLE_THRESHOLD_YEARS: f64 = 5.0;
pub const TECH_FORGOTTEN_RECENT_YEARS: f64 = 10.0;
pub const TECH_POP_MAINTENANCE_BONUS: f64 = 0.01;
pub const TECH_POP_MAINTENANCE_CAP: f64 = 0.5;
pub const TECH_ACTIVE_USE_ATROPHY_REDUCTION: f64 = 0.3;
pub const TECH_ARTIFACT_GRACE_BONUS: f64 = 0.2;
pub const TECH_FORGOTTEN_LONG_DECAY_MULTIPLIER: f64 = 0.4;
pub const TECH_MODIFIER_STACK_CAP: f64 = 10.0;
pub const TECH_MODIFIER_ADDITIVE_STACK_CAP: f64 = 500.0;
pub const TECH_RECALC_COOLDOWN_TICKS: u64 = 5;
pub const TECH_ERA_TRIBAL_COUNT: u32 = 5;
pub const TECH_ERA_BRONZE_AGE_COUNT: u32 = 12;

// ── Tech Propagation [Rogers 2003, Lave 1991] ─────────────────────────────────
pub const TEACHING_TICK_INTERVAL: u64 = 24;
pub const TEACHING_BASE_EFFECTIVENESS: f64 = 0.02;
pub const TEACHING_SKILL_GAP_MIN: u32 = 3;
pub const TEACHING_SKILL_GAP_OPTIMAL: u32 = 5;
pub const TEACHING_SKILL_GAP_MAX: u32 = 10;
pub const TEACHING_MAX_STUDENTS: u32 = 3;
pub const TEACHING_WILLINGNESS_THRESHOLD: f64 = 0.3;
pub const TEACHING_SESSION_TICKS: u64 = 72;
pub const TEACHING_ABANDON_TICKS: u64 = 480;
pub const CROSS_PROP_TRADE_BASE: f64 = 0.05;
pub const CROSS_PROP_MIGRATION_BASE: f64 = 0.8;
pub const CROSS_PROP_WAR_CAPTURE_BASE: f64 = 0.3;
pub const CROSS_PROP_DIPLOMACY_BASE: f64 = 0.1;
pub const CROSS_PROP_LANGUAGE_THRESHOLD: f64 = 0.6;
pub const CROSS_PROP_LANGUAGE_BLOCK: f64 = 0.9;
pub const ADOPTION_OPENNESS_WEIGHT: f64 = 0.35;
pub const ADOPTION_CURIOSITY_WEIGHT: f64 = 0.25;
pub const ADOPTION_CONSCIENTIOUSNESS_WEIGHT: f64 = 0.20;
pub const ADOPTION_KNOWLEDGE_VALUE_WEIGHT: f64 = 0.20;

// ── Combat [Keeley 1996] ──────────────────────────────────────────────────────
pub const COMBAT_HEAD_DEATH_THRESHOLD: f64 = 0.70;
pub const COMBAT_TORSO_DEATH_THRESHOLD: f64 = 0.80;
pub const COMBAT_LIMB_SPEED_PENALTY: f64 = 0.30;
pub const COMBAT_LIMB_STR_PENALTY: f64 = 0.25;
pub const COMBAT_BASE_WEAPON_DAMAGE: f64 = 0.15;
pub const COMBAT_BASE_ARMOR: f64 = 0.0;
pub const COMBAT_MORALE_ROUT_THRESHOLD: f64 = 0.20;
pub const COMBAT_MORALE_SHAKEN_THRESHOLD: f64 = 0.40;
pub const COMBAT_MORALE_W_HAPPINESS: f64 = 0.30;
pub const COMBAT_MORALE_W_CHARISMA: f64 = 0.30;
pub const COMBAT_MORALE_W_CAUSE_BELIEF: f64 = 0.40;
pub const COMBAT_ROLL_RANDOM_RANGE: f64 = 0.30;

// ── Inter-settlement Tension [Tilly 1978, Keeley 1996] ────────────────────────
pub const TENSION_CHECK_INTERVAL_TICKS: u64 = 2190;  // twice yearly
pub const TENSION_RESOURCE_DEFICIT_TRIGGER: f64 = 0.30;
pub const TENSION_PROXIMITY_RADIUS: i32 = 20;
pub const TENSION_PER_SHARED_RESOURCE: f64 = 0.05;
pub const TENSION_DECAY_PER_YEAR: f64 = 0.15;
pub const TENSION_SKIRMISH_THRESHOLD: f64 = 0.60;
pub const TENSION_SKIRMISH_CHANCE: f64 = 0.35;
pub const TENSION_SKIRMISH_COOLDOWN: u64 = 4380;  // 1 year
pub const TENSION_WINNER_REDUCTION: f64 = 0.30;
pub const TENSION_LOSER_INCREASE: f64 = 0.20;

// ── Stats Panel [C-1i] ────────────────────────────────────────────────────────
pub const STATS_PANEL_REFRESH_INTERVAL: u64 = 120;
pub const STATS_RESOURCE_DANGER_DAYS: f64 = 7.0;
pub const STATS_RESOURCE_LOW_DAYS: f64 = 30.0;
pub const STATS_RESOURCE_ABUNDANT_DAYS: f64 = 90.0;
pub const STATS_RECENT_EVENTS_MAX: usize = 20;
pub const STATS_RECENT_PERIOD_TICKS: u64 = 365;

// ── Derived stat helpers ──────────────────────────────────────────────────────

/// Returns the intelligence group for an index (0=linguistic … 7=intrapersonal).
/// "fluid" = [logical, spatial], "crystallized" = [linguistic, musical, interpersonal, intrapersonal, naturalistic],
/// "physical" = [kinesthetic]
pub fn intel_group(index: usize) -> &'static str {
    match index {
        0 => "crystallized", // linguistic
        1 => "fluid",        // logical
        2 => "fluid",        // spatial
        3 => "crystallized", // musical
        4 => "physical",     // kinesthetic
        5 => "crystallized", // naturalistic
        6 => "crystallized", // interpersonal
        7 => "crystallized", // intrapersonal
        _  => "crystallized",
    }
}

/// Returns value heritability for a value key (Knafo & Schwartz 2004).
pub fn value_heritability(key: &str) -> f64 {
    match key {
        "TRADITION" | "LAW"           => 0.10,
        "DECORUM"   | "LOYALTY"       => 0.11,
        "STOICISM"  | "HARMONY"       => 0.11,
        "PEACE"     | "FAMILY"        => 0.12,
        "COOPERATION" | "SACRIFICE"   => 0.12,
        "FAIRNESS"  | "FRIENDSHIP"    => 0.13,
        "TRUTH"     | "INTROSPECTION" => 0.14,
        "TRANQUILITY" | "COMMERCE"    => 0.14,
        "KNOWLEDGE" | "INDEPENDENCE"  => 0.15,
        "NATURE"    | "SKILL"         => 0.15,
        "ROMANCE"   | "CRAFTSMANSHIP" => 0.15,
        "ELOQUENCE" | "ARTWORK"       => 0.16,
        "MERRIMENT" | "LEISURE"       => 0.16,
        "PERSEVERANCE"                => 0.17,
        "SELF_CONTROL" | "HARD_WORK"  => 0.18,
        "CUNNING"   | "COMPETITION"   => 0.18,
        "MARTIAL_PROWESS"             => 0.19,
        "POWER"                       => 0.20,
        _                             => 0.15, // default
    }
}

// ── Runtime config struct ─────────────────────────────────────────────────────
/// Runtime configuration derived from the module-level constants.
///
/// Passed to subsystems (e.g. `GameCalendar::new`) that need numeric parameters
/// without taking a dependency on every constant directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub ticks_per_day: u32,
    pub days_per_year: u32,
    pub ticks_per_year: u32,
    pub max_entities: u32,
    pub world_width: u32,
    pub world_height: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            ticks_per_day: TICKS_PER_DAY,
            days_per_year: DAYS_PER_YEAR,
            ticks_per_year: TICKS_PER_YEAR,
            max_entities: MAX_ENTITIES,
            world_width: WORLD_WIDTH,
            world_height: WORLD_HEIGHT,
        }
    }
}

// ── Snapshot marker (used in PhaseR-3 FFI bridge) ────────────────────────────
/// Serializable summary of the current config for diagnostics / save metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub ticks_per_year: u32,
    pub days_per_year: u32,
    pub max_entities: u32,
    pub world_width: u32,
    pub world_height: u32,
}

impl ConfigSummary {
    pub fn current() -> Self {
        Self {
            ticks_per_year: TICKS_PER_YEAR,
            days_per_year: DAYS_PER_YEAR,
            max_entities: MAX_ENTITIES,
            world_width: WORLD_WIDTH,
            world_height: WORLD_HEIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_per_year_consistent() {
        assert_eq!(TICKS_PER_DAY * DAYS_PER_YEAR, TICKS_PER_YEAR);
    }

    #[test]
    fn body_heritability_all_valid() {
        for h in BODY_HERITABILITY {
            assert!(h > 0.0 && h <= 1.0, "heritability out of range: {h}");
        }
    }

    #[test]
    fn intel_corr_matrix_diagonal_ones() {
        for i in 0..8 {
            assert!((INTEL_RESIDUAL_CORR[i][i] - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn memory_intensity_trauma_class() {
        assert!(memory_intensity("partner_died") > 0.8);
        assert!(memory_intensity("marriage") > 0.8);
    }

    #[test]
    fn memory_decay_rate_returns_correct_rate() {
        assert!((memory_decay_rate(0.95) - 0.014).abs() < 1e-9); // trauma
        assert!((memory_decay_rate(0.60) - 0.139).abs() < 1e-9); // strong
        assert!((memory_decay_rate(0.30) - 0.347).abs() < 1e-9); // moderate
        assert!((memory_decay_rate(0.05) - 1.386).abs() < 1e-9); // trivial
    }
}
