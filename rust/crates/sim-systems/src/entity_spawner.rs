//! Entity spawner: populates hecs::World with fully-initialized agents.
//!
//! Ports GDScript EntityManager.spawn_entity() (551 lines) to Rust.
//! Academic references:
//! - HEXACO personality: Ashton & Lee (2009)
//! - Body heritability: Bouchard & McGue (2003)
//! - Intelligence: Gardner (1983), Bouchard & McGue (2003)
//! - Appearance: Eagly (1991), Stulp (2015)

use crate::runtime::derive_steering_params;
use crate::values_init::initialize_values;
use rand::Rng;
use rand_distr::{Distribution, Normal, StandardNormal};
use sim_core::components::{
    Age, AgentKnowledge, Behavior, Body, BodyHealth, Coping, Economic, EffectFlags, Emotion, Faith,
    Identity, InfluenceReceiver, Intelligence, Inventory, KnowledgeEntry, LlmCapable, Memory,
    NarrativeCache, Needs, Personality, Position, Skills, Social, Stress, Temperament, Traits,
    TransmissionSource,
};
use sim_core::enums::{GrowthStage, Sex};
use sim_core::{
    SettlementId, TemperamentBiasRow, TemperamentPrsWeightRow, TemperamentRuleSet,
    TemperamentShiftRuleView,
};
use sim_data::{PersonalityDistribution, TemperamentRules};
use sim_engine::engine::SimResources;

// ── Body generation constants ─────────────────────────────────────────────────

const BODY_POTENTIAL_MEAN: f64 = 1050.0;
const BODY_POTENTIAL_SD: f64 = 175.0;
const BODY_POTENTIAL_MIN: i32 = 400;
const BODY_POTENTIAL_MAX: i32 = 1800;
const BODY_SEX_DELTA_STR: f64 = 60.0; // male higher
const BODY_SEX_DELTA_END: f64 = 20.0;
const BODY_SEX_DELTA_TOU: f64 = 80.0;
const BODY_SEX_DELTA_REC: f64 = -10.0; // male lower
const BODY_SEX_DELTA_DR: f64 = -20.0;
const TRAINABILITY_MEAN: f64 = 500.0;
const TRAINABILITY_SD: f64 = 100.0;
const TRAINABILITY_MIN: i32 = 200;
const TRAINABILITY_MAX: i32 = 800;
const INNATE_IMMUNITY_MEAN: f64 = 500.0;
const INNATE_IMMUNITY_SD: f64 = 100.0;
const INNATE_IMMUNITY_FEMALE_BONUS: f64 = 50.0;
const APPEARANCE_ATTRACT_MEAN: f64 = 0.5;
const APPEARANCE_ATTRACT_SD: f64 = 0.12;
const APPEARANCE_HEIGHT_MEAN_MALE: f64 = 0.55;
const APPEARANCE_HEIGHT_MEAN_FEMALE: f64 = 0.50;
const APPEARANCE_HEIGHT_SD: f64 = 0.08;

/// Ticks per in-game year: 12 ticks/day × 365 days/year
const TICKS_PER_YEAR: u64 = 4380;

// ── SpawnConfig ───────────────────────────────────────────────────────────────

/// Configuration for spawning a single agent.
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub settlement_id: Option<SettlementId>,
    pub position: (i32, i32),
    /// Age in ticks (0 = newborn)
    pub initial_age_ticks: u64,
    /// None = random
    pub sex: Option<Sex>,
    pub parent_a: Option<hecs::Entity>,
    pub parent_b: Option<hecs::Entity>,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Returns a random age in years, weighted toward working-age adults.
fn weighted_random_age(rng: &mut impl Rng) -> u32 {
    let roll: f64 = rng.gen();
    if roll < 0.10 {
        rng.gen_range(0..=4)
    } else if roll < 0.25 {
        rng.gen_range(5..=14)
    } else if roll < 0.65 {
        rng.gen_range(15..=30)
    } else if roll < 0.90 {
        rng.gen_range(30..=50)
    } else if roll < 0.98 {
        rng.gen_range(50..=69)
    } else {
        rng.gen_range(70..=80)
    }
}

/// Map calendar tick to a zodiac sign string.
fn zodiac_from_calendar_tick(tick: u64) -> &'static str {
    let day_of_year = ((tick % TICKS_PER_YEAR) / 12) as u32; // 12 ticks per day
    let (month, day) = day_of_year_to_month_day(day_of_year + 1);
    zodiac_from_day(month, day)
}

fn day_of_year_to_month_day(doy: u32) -> (u32, u32) {
    let month_lengths: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut rem = doy.clamp(1, 365) - 1;
    for (i, &len) in month_lengths.iter().enumerate() {
        if rem < len {
            return (i as u32 + 1, rem + 1);
        }
        rem -= len;
    }
    (12, 31)
}

fn zodiac_from_day(month: u32, day: u32) -> &'static str {
    match (month, day) {
        (1, d) if d >= 20 => "aquarius",
        (1, _) => "capricorn",
        (2, d) if d >= 19 => "pisces",
        (2, _) => "aquarius",
        (3, d) if d >= 21 => "aries",
        (3, _) => "pisces",
        (4, d) if d >= 20 => "taurus",
        (4, _) => "aries",
        (5, d) if d >= 21 => "gemini",
        (5, _) => "taurus",
        (6, d) if d >= 21 => "cancer",
        (6, _) => "gemini",
        (7, d) if d >= 23 => "leo",
        (7, _) => "cancer",
        (8, d) if d >= 23 => "virgo",
        (8, _) => "leo",
        (9, d) if d >= 23 => "libra",
        (9, _) => "virgo",
        (10, d) if d >= 23 => "scorpio",
        (10, _) => "libra",
        (11, d) if d >= 22 => "sagittarius",
        (11, _) => "scorpio",
        (12, d) if d >= 22 => "capricorn",
        (12, _) => "sagittarius",
        _ => "capricorn",
    }
}

fn blood_type_from_genotype(genotype: &str) -> &'static str {
    match genotype {
        "OO" => "O",
        "AA" | "AO" => "A",
        "BB" | "BO" => "B",
        "AB" => "AB",
        _ => "O",
    }
}

fn random_blood_genotype(rng: &mut impl Rng) -> String {
    // Distribution: O=43%, A=27%, B=25%, AB=5%
    let roll: f64 = rng.gen();
    let phenotype = if roll < 0.43 {
        "O"
    } else if roll < 0.70 {
        "A"
    } else if roll < 0.95 {
        "B"
    } else {
        "AB"
    };
    let geno_roll: f64 = rng.gen();
    match phenotype {
        "O" => "OO".to_string(),
        "A" => {
            if geno_roll < 0.20 {
                "AA".to_string()
            } else {
                "AO".to_string()
            }
        }
        "B" => {
            if geno_roll < 0.15 {
                "BB".to_string()
            } else {
                "BO".to_string()
            }
        }
        "AB" => "AB".to_string(),
        _ => "OO".to_string(),
    }
}

// ── Personality generation ────────────────────────────────────────────────────

fn generate_personality(
    sex: Sex,
    dist: &PersonalityDistribution,
    rng: &mut impl Rng,
    parent_a_pers: Option<&Personality>,
    parent_b_pers: Option<&Personality>,
) -> Personality {
    let axis_ids = ["H", "E", "X", "A", "C", "O"];

    // 1. Cholesky decompose correlation matrix
    let l = crate::math_utils::cholesky_decompose(&dist.correlation_matrix.matrix);

    // 2. Sample 6 independent standard normal z-scores
    let z_indep: Vec<f64> = (0..6).map(|_| StandardNormal.sample(rng)).collect();

    // 3. Correlate via Cholesky
    let z_corr = crate::math_utils::cholesky_multiply(&l, &z_indep);

    // 4. Per-axis: inheritance + sex difference
    let mut z_axes = [0.0f64; 6];
    for i in 0..6 {
        let h2 = dist.heritability.get(axis_ids[i]).copied().unwrap_or(0.5);
        let d = dist
            .sex_difference_d
            .get(axis_ids[i])
            .copied()
            .unwrap_or(0.0);

        let z_mid = match (parent_a_pers, parent_b_pers) {
            (Some(pa), Some(pb)) => {
                0.5 * ((pa.axes[i] - 0.5) / dist.sd + (pb.axes[i] - 0.5) / dist.sd)
            }
            _ => 0.0,
        };

        let env_factor = (1.0 - 0.5 * h2 * h2).sqrt();
        let mut z_child = h2 * z_mid + env_factor * z_corr[i];

        // Cohen's d: positive = female higher
        match sex {
            Sex::Female => z_child += d / 2.0,
            Sex::Male => z_child -= d / 2.0,
        }
        z_axes[i] = z_child;
    }

    // 5. 24 facets with intra-axis spread
    let mut facets = [0.5f64; 24];
    for axis_idx in 0..6 {
        for offset in 0..4 {
            let noise: f64 = StandardNormal.sample(rng);
            let fz: f64 = z_axes[axis_idx] + noise * dist.facet_spread;
            facets[axis_idx * 4 + offset] = (0.5 + fz * dist.sd).clamp(0.01, 0.99);
        }
    }

    // 6. Axes from facet averages
    let mut axes = [0.0f64; 6];
    for i in 0..6 {
        axes[i] = facets[i * 4..i * 4 + 4].iter().sum::<f64>() / 4.0;
    }

    Personality { axes, facets }
}

// ── Body generation ───────────────────────────────────────────────────────────

fn sample_normal_clamped(mean: f64, sd: f64, min: i32, max: i32, rng: &mut impl Rng) -> i32 {
    let normal = Normal::new(mean, sd).unwrap_or_else(|_| Normal::new(mean, 1.0).unwrap());
    let v: f64 = normal.sample(rng);
    (v.round() as i32).clamp(min, max)
}

fn generate_body(sex: Sex, rng: &mut impl Rng) -> Body {
    let is_male = sex == Sex::Male;
    let sex_sign = if is_male { 1.0_f64 } else { -1.0_f64 };

    // ACTN3 gene: [-1, 1], affects strength vs endurance tradeoff
    let actn3: f64 = rng.gen_range(-1.0_f64..=1.0_f64);

    let str_p = sample_normal_clamped(
        BODY_POTENTIAL_MEAN + sex_sign * BODY_SEX_DELTA_STR,
        BODY_POTENTIAL_SD,
        BODY_POTENTIAL_MIN,
        BODY_POTENTIAL_MAX,
        rng,
    );
    let agi_p = sample_normal_clamped(
        BODY_POTENTIAL_MEAN,
        BODY_POTENTIAL_SD,
        BODY_POTENTIAL_MIN,
        BODY_POTENTIAL_MAX,
        rng,
    );
    let end_p = sample_normal_clamped(
        BODY_POTENTIAL_MEAN + sex_sign * BODY_SEX_DELTA_END,
        BODY_POTENTIAL_SD,
        BODY_POTENTIAL_MIN,
        BODY_POTENTIAL_MAX,
        rng,
    );
    let tou_p = sample_normal_clamped(
        BODY_POTENTIAL_MEAN + sex_sign * BODY_SEX_DELTA_TOU,
        BODY_POTENTIAL_SD,
        BODY_POTENTIAL_MIN,
        BODY_POTENTIAL_MAX,
        rng,
    );
    let rec_p = sample_normal_clamped(
        BODY_POTENTIAL_MEAN + sex_sign * BODY_SEX_DELTA_REC,
        BODY_POTENTIAL_SD,
        BODY_POTENTIAL_MIN,
        BODY_POTENTIAL_MAX,
        rng,
    );
    let dr_p = sample_normal_clamped(
        BODY_POTENTIAL_MEAN + sex_sign * BODY_SEX_DELTA_DR,
        BODY_POTENTIAL_SD,
        BODY_POTENTIAL_MIN,
        BODY_POTENTIAL_MAX,
        rng,
    );

    // Trainability — ACTN3 correlations
    let str_t_base: f64 = Normal::new(TRAINABILITY_MEAN, TRAINABILITY_SD)
        .unwrap()
        .sample(rng);
    let str_t =
        ((str_t_base + actn3 * 75.0).round() as i32).clamp(TRAINABILITY_MIN, TRAINABILITY_MAX);

    let end_t_base: f64 = Normal::new(TRAINABILITY_MEAN, TRAINABILITY_SD)
        .unwrap()
        .sample(rng);
    let end_t =
        ((end_t_base - actn3 * 50.0).round() as i32).clamp(TRAINABILITY_MIN, TRAINABILITY_MAX);

    let tou_ind: f64 = Normal::new(TRAINABILITY_MEAN, TRAINABILITY_SD)
        .unwrap()
        .sample(rng);
    let tou_t = ((0.70 * tou_ind + 0.30 * str_t_base).round() as i32)
        .clamp(TRAINABILITY_MIN, TRAINABILITY_MAX);

    let rec_ind: f64 = Normal::new(TRAINABILITY_MEAN, TRAINABILITY_SD)
        .unwrap()
        .sample(rng);
    let rec_t = ((0.60 * rec_ind + 0.40 * end_t_base + actn3 * 20.0).round() as i32)
        .clamp(TRAINABILITY_MIN, TRAINABILITY_MAX);

    let agi_t_raw: f64 = Normal::new(TRAINABILITY_MEAN, TRAINABILITY_SD)
        .unwrap()
        .sample(rng);
    let agi_t = (agi_t_raw.round() as i32).clamp(TRAINABILITY_MIN, TRAINABILITY_MAX);

    let dr_t_raw: f64 = Normal::new(TRAINABILITY_MEAN, TRAINABILITY_SD)
        .unwrap()
        .sample(rng);
    let dr_t = (dr_t_raw.round() as i32).clamp(TRAINABILITY_MIN, TRAINABILITY_MAX);

    // Innate immunity
    let imm_base: f64 = Normal::new(INNATE_IMMUNITY_MEAN, INNATE_IMMUNITY_SD)
        .unwrap()
        .sample(rng);
    let innate_immunity = (imm_base
        + if !is_male {
            INNATE_IMMUNITY_FEMALE_BONUS
        } else {
            0.0
        })
    .round() as i32;

    // Appearance
    let attractiveness_f64: f64 = Normal::new(APPEARANCE_ATTRACT_MEAN, APPEARANCE_ATTRACT_SD)
        .unwrap()
        .sample(rng);
    let attractiveness = attractiveness_f64.clamp(0.0, 1.0) as f32;

    let height_mean = if is_male {
        APPEARANCE_HEIGHT_MEAN_MALE
    } else {
        APPEARANCE_HEIGHT_MEAN_FEMALE
    };
    let height_f64: f64 = Normal::new(height_mean, APPEARANCE_HEIGHT_SD)
        .unwrap()
        .sample(rng);
    let height = height_f64.clamp(0.0, 1.0) as f32;

    let blood_genotype = random_blood_genotype(rng);

    Body {
        str_potential: str_p,
        agi_potential: agi_p,
        end_potential: end_p,
        tou_potential: tou_p,
        rec_potential: rec_p,
        dr_potential: dr_p,
        str_trainability: str_t,
        agi_trainability: agi_t,
        end_trainability: end_t,
        tou_trainability: tou_t,
        rec_trainability: rec_t,
        dr_trainability: dr_t,
        // Realized starts at potential at birth (no training yet)
        str_realized: str_p,
        agi_realized: agi_p,
        end_realized: end_p,
        tou_realized: tou_p,
        rec_realized: rec_p,
        dr_realized: dr_p,
        innate_immunity,
        attractiveness,
        height,
        health: 1.0,
        blood_genotype,
        ..Body::default()
    }
}

// ── Intelligence generation ───────────────────────────────────────────────────

fn generate_intelligence(personality: &Personality, rng: &mut impl Rng) -> Intelligence {
    // g factor: general intelligence
    let g_f64: f64 = Normal::new(0.5, 0.15).unwrap().sample(rng);
    let g = g_f64.clamp(0.0, 1.0);

    // Personality correlations per Gardner type (approximate)
    // [Linguistic, Logical, Spatial, Musical, Kinesthetic, Interpersonal, Intrapersonal, Naturalistic]
    let pers_cors = [
        personality.axes[5],                                   // Openness → Linguistic
        personality.axes[4],                                   // Conscientiousness → Logical
        personality.axes[5],                                   // Openness → Spatial
        personality.axes[5] * 0.5 + personality.axes[2] * 0.5, // Openness+Extraversion → Musical
        personality.axes[0].max(personality.axes[2]),          // Honesty/Extraversion → Kinesthetic
        personality.axes[2],                                   // Extraversion → Interpersonal
        personality.axes[1],                                   // Emotionality → Intrapersonal
        personality.axes[5],                                   // Openness → Naturalistic
    ];

    let mut values = [0.0f64; 8];
    for i in 0..8 {
        let genetic: f64 = Normal::new(0.5, 0.10).unwrap().sample(rng);
        let genetic = genetic.clamp(0.0, 1.0);
        let rand_noise: f64 = Normal::new(0.0, 0.05).unwrap().sample(rng);
        values[i] =
            (g * 0.4 + pers_cors[i] * 0.2 + genetic * 0.2 + rand_noise + 0.2 * 0.5).clamp(0.0, 1.0);
    }

    Intelligence {
        values,
        g_factor: g,
        ace_penalty: 0.0,
        nutrition_penalty: 0.0,
    }
}

// ── Needs generation ──────────────────────────────────────────────────────────

fn generate_needs(rng: &mut impl Rng) -> Needs {
    // 13 needs in NeedType enum order:
    // Hunger=0, Thirst=1, Sleep=2, Warmth=3, Safety=4,
    // Belonging=5, Intimacy=6, Recognition=7,
    // Autonomy=8, Competence=9, SelfActualization=10, Meaning=11, Transcendence=12
    let values = [
        0.7 + rng.gen::<f64>() * 0.3,   // Hunger (0)
        0.70 + rng.gen::<f64>() * 0.15, // Thirst (1)
        0.80 + rng.gen::<f64>() * 0.15, // Sleep (2)
        0.85 + rng.gen::<f64>() * 0.10, // Warmth (3)
        0.55 + rng.gen::<f64>() * 0.10, // Safety (4)
        0.70,                           // Belonging (5)
        0.70,                           // Intimacy (6)
        0.60,                           // Recognition (7)
        0.60,                           // Autonomy (8)
        0.60,                           // Competence (9)
        0.50,                           // SelfActualization (10)
        0.50,                           // Meaning (11)
        0.40,                           // Transcendence (12)
    ];
    Needs {
        values,
        energy: 0.7 + rng.gen::<f64>() * 0.3,
        starvation_grace_ticks: 0,
        growth_frustration_ticks: 0,
        relatedness_frustration_ticks: 0,
    }
}

// ── Speech style generation ───────────────────────────────────────────────────

/// Returns (tone, verbosity, humor) as owned Strings.
/// Port of GDScript EntityManager._generate_speech_style().
fn generate_speech_style(personality: &Personality) -> (String, String, String) {
    let h = personality.axes[0]; // Honesty-Humility
    let e = personality.axes[1]; // Emotionality
    let x = personality.axes[2]; // Extraversion
    let a = personality.axes[3]; // Agreeableness
    let c = personality.axes[4]; // Conscientiousness
    let o = personality.axes[5]; // Openness
                                 // Spawn-time emotions are zero
    let anger = 0.0_f64;
    let joy = 0.0_f64;
    let fear = 0.0_f64;

    let agg = (0.30 * (1.0 - a)
        + 0.20 * (1.0 - h)
        + 0.15 * x
        + 0.10 * (1.0 - c)
        + 0.20 * anger
        + 0.05 * (1.0 - fear))
        .clamp(0.0, 1.0);
    let gent = (0.30 * a + 0.20 * h + 0.15 * e + 0.10 * c + 0.15 * (1.0 - anger) + 0.10 * joy)
        .clamp(0.0, 1.0);
    let form = (0.35 * c
        + 0.25 * h
        + 0.15 * (1.0 - x)
        + 0.10 * (1.0 - o)
        + 0.10 * fear
        + 0.05 * (1.0 - joy))
        .clamp(0.0, 1.0);
    let cas =
        (0.35 * x + 0.20 * o + 0.15 * joy + 0.15 * (1.0 - c) + 0.10 * a + 0.05 * (1.0 - fear))
            .clamp(0.0, 1.0);
    let sarc =
        (0.30 * o + 0.25 * (1.0 - a) + 0.15 * x + 0.15 * anger + 0.10 * joy + 0.05 * (1.0 - h))
            .clamp(0.0, 1.0);

    let tone = if agg >= 0.65 && anger >= 0.55 {
        "aggressive"
    } else if sarc >= 0.62 && o >= 0.55 && anger >= 0.35 {
        "sarcastic"
    } else if form >= 0.62 && c >= 0.65 {
        "formal"
    } else if gent >= 0.62 && a >= 0.65 && h >= 0.60 && anger <= 0.35 {
        "gentle"
    } else {
        let scores = [
            ("aggressive", agg),
            ("gentle", gent),
            ("formal", form),
            ("casual", cas),
            ("sarcastic", sarc),
        ];
        scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(&("casual", cas))
            .0
    };

    let ver =
        (0.55 * x + 0.15 * o + 0.10 * joy + 0.05 * anger + 0.10 * (1.0 - fear) + 0.05 * (1.0 - c))
            .clamp(0.0, 1.0);
    let verbosity = if ver >= 0.67 || x >= 0.75 {
        "talkative"
    } else if ver <= 0.33 || x <= 0.25 || (fear >= 0.70 && x <= 0.55) {
        "taciturn"
    } else {
        "normal"
    };

    let hum = (0.35 * o
        + 0.20 * x
        + 0.15 * joy
        + 0.15 * (1.0 - a)
        + 0.10 * (1.0 - fear)
        + 0.05 * (1.0 - c))
        .clamp(0.0, 1.0);
    let humor = if hum < 0.40 {
        "none"
    } else {
        let slap = (0.45 * x + 0.30 * joy + 0.15 * (1.0 - c) + 0.10 * (1.0 - fear)).clamp(0.0, 1.0);
        let wit =
            (0.45 * o + 0.20 * x + 0.15 * joy + 0.10 * c + 0.10 * h - 0.20 * anger).clamp(0.0, 1.0);
        let dry = (0.35 * c + 0.25 * (1.0 - x) + 0.20 * o + 0.10 * (1.0 - joy) + 0.10 * anger)
            .clamp(0.0, 1.0);
        if slap >= 0.60 && x >= 0.65 && joy >= 0.55 {
            "slapstick"
        } else if wit >= 0.60 && o >= 0.60 {
            "witty"
        } else if slap >= wit && slap >= dry {
            "slapstick"
        } else if wit >= dry {
            "witty"
        } else {
            "dry"
        }
    };

    (tone.to_string(), verbosity.to_string(), humor.to_string())
}

fn temperament_rule_set_from_data_rules(rules: &TemperamentRules) -> TemperamentRuleSet {
    TemperamentRuleSet {
        prs_weights: rules
            .prs_weights
            .iter()
            .map(|row| TemperamentPrsWeightRow {
                axis: row.axis.clone(),
                weights: row.weights.clone(),
            })
            .collect(),
        bias_matrix: rules
            .bias_matrix
            .iter()
            .map(|row| TemperamentBiasRow {
                axis: row.axis.clone(),
                values: row.values.clone(),
            })
            .collect(),
        shift_rules: rules
            .shift_rules
            .iter()
            .map(|rule| {
                let trigger_event = match &rule.trigger {
                    sim_data::CauseTrigger::Event(event_key) => event_key.clone(),
                };
                TemperamentShiftRuleView {
                    trigger_event,
                    causal_log: rule.causal_log.clone(),
                }
            })
            .collect(),
    }
}

fn generate_temperament(
    personality: &Personality,
    registry: Option<&sim_data::DataRegistry>,
) -> Temperament {
    if let Some(rules) = registry.and_then(|data_registry| data_registry.temperament_rules_ref()) {
        let shared_rules = temperament_rule_set_from_data_rules(rules);
        Temperament::from_personality_with_rules(personality, &shared_rules)
    } else {
        Temperament::from_personality(personality)
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Spawn a single agent into the ECS world with all components initialized.
///
/// # Arguments
/// - `world`: hecs ECS world
/// - `resources`: simulation resources (provides RNG, calendar, personality distribution)
/// - `config`: spawn configuration (position, age, sex, parents)
pub fn spawn_agent(
    world: &mut hecs::World,
    resources: &mut SimResources,
    config: &SpawnConfig,
) -> hecs::Entity {
    // Determine sex
    let sex = config.sex.unwrap_or_else(|| {
        if resources.rng.gen::<bool>() {
            Sex::Male
        } else {
            Sex::Female
        }
    });

    // Compute age
    let age_years = config.initial_age_ticks as f64 / TICKS_PER_YEAR as f64;
    let growth_stage = GrowthStage::from_age_ticks(config.initial_age_ticks);

    // Clone personality distribution to avoid holding borrow across rng use
    let personality = {
        let dist_opt = resources.personality_distribution.clone();
        if let Some(ref dist) = dist_opt {
            generate_personality(sex, dist, &mut resources.rng, None, None)
        } else {
            Personality::default()
        }
    };

    // Body
    let body = generate_body(sex, &mut resources.rng);

    // Intelligence
    let intelligence = generate_intelligence(&personality, &mut resources.rng);

    // Needs
    let needs = generate_needs(&mut resources.rng);

    // Speech style (deterministic from personality, no extra RNG)
    let (speech_tone, speech_verbosity, speech_humor) = generate_speech_style(&personality);

    // Calendar-based birth info
    let birth_tick = resources.calendar.tick;
    let zodiac = zodiac_from_calendar_tick(birth_tick);

    // Blood type derived from genotype
    let blood_type = blood_type_from_genotype(&body.blood_genotype).to_string();

    // Name: use NameGenerator if available, else placeholder.
    let settlement_id_u32 = config.settlement_id.map(|s| s.0 as u32).unwrap_or(0u32);
    let name = if let Some(ref mut ng) = resources.name_generator {
        ng.generate_name(
            sex,
            "proto_nature",
            settlement_id_u32,
            None,
            &mut resources.rng,
        )
    } else {
        let entity_count = world.len() as u64;
        format!("Agent {}", entity_count + 1)
    };

    // Build identity
    let identity = Identity {
        name,
        birth_tick,
        sex,
        species_id: "human".to_string(),
        settlement_id: config.settlement_id,
        growth_stage,
        zodiac_sign: zodiac.to_string(),
        blood_type,
        speech_tone,
        speech_verbosity,
        speech_humor,
        ..Identity::default()
    };

    // Age component
    let age = Age {
        ticks: config.initial_age_ticks,
        years: age_years,
        stage: growth_stage,
        alive: true,
    };

    // Position
    let position = Position::new(config.position.0, config.position.1);

    // Values: initialize from HEXACO personality before personality is moved.
    let values = initialize_values(&personality, None, None, &mut resources.rng);
    let steering = derive_steering_params(&personality);
    let temperament = generate_temperament(
        &personality,
        resources.data_registry.as_ref().map(std::sync::Arc::as_ref),
    );
    let openness = (personality.facets[20]
        + personality.facets[21]
        + personality.facets[22]
        + personality.facets[23])
        / 4.0;
    let innovation_potential =
        (intelligence.g_factor * 0.5 + openness * 0.3 + resources.rng.gen::<f64>() * 0.2)
            .clamp(0.2, 0.8);

    // hecs DynamicBundle is implemented for tuples up to 15 elements.
    // We currently spawn 27 components total: the first 15 in the spawn bundle,
    // then insert the remaining 12 overflow components.
    let entity = world.spawn((
        identity,
        age,
        position,
        personality,
        body,
        intelligence,
        needs,
        Stress::default(),
        Behavior::default(),
        Emotion::default(),
        values,
        Coping::default(),
        Social::default(),
        Economic::default(),
        Skills::default(),
    ));

    // Insert remaining components that would exceed the tuple bundle limit.
    world
        .insert(
            entity,
            (
                Memory::default(),
                Traits::default(),
                Faith::default(),
                steering,
                temperament,
                InfluenceReceiver::default(),
                EffectFlags::default(),
                Inventory::new(),
                BodyHealth::default(),
                AgentKnowledge::default(),
                LlmCapable::default(),
                NarrativeCache::default(),
            ),
        )
        .unwrap_or_else(|e| log::warn!("[entity_spawner] insert extra components failed: {e}"));

    if age_years >= 15.0 {
        if let Ok(mut knowledge) = world.get::<&mut AgentKnowledge>(entity) {
            knowledge.learn(KnowledgeEntry {
                knowledge_id: "TECH_FIRE_MAKING".to_string(),
                proficiency: 0.6 + resources.rng.gen::<f64>() * 0.3,
                source: TransmissionSource::Oral,
                acquired_tick: 0,
                last_used_tick: 0,
                teacher_id: 0,
            });
            knowledge.learn(KnowledgeEntry {
                knowledge_id: "TECH_STONE_KNAPPING".to_string(),
                proficiency: 0.5 + resources.rng.gen::<f64>() * 0.3,
                source: TransmissionSource::Observed,
                acquired_tick: 0,
                last_used_tick: 0,
                teacher_id: 0,
            });
            knowledge.learn(KnowledgeEntry {
                knowledge_id: "TECH_FORAGING".to_string(),
                proficiency: 0.7 + resources.rng.gen::<f64>() * 0.2,
                source: TransmissionSource::Oral,
                acquired_tick: 0,
                last_used_tick: 0,
                teacher_id: 0,
            });
            knowledge.innovation_potential = innovation_potential;
        }
    }

    entity
}

/// Spawn `count` agents near the center of the map.
pub fn spawn_initial_population(
    world: &mut hecs::World,
    resources: &mut SimResources,
    count: usize,
    settlement_id: SettlementId,
) {
    let center_x = (resources.map.width / 2) as i32;
    let center_y = (resources.map.height / 2) as i32;

    for _ in 0..count {
        let age_years = weighted_random_age(&mut resources.rng);
        let initial_age_ticks = age_years as u64 * TICKS_PER_YEAR;

        // Collect walkable tiles into owned Vec to release borrow on map before spawning
        let walkable = resources.map.walkable_tiles_near(center_x, center_y, 5);
        let pos = if walkable.is_empty() {
            (center_x, center_y)
        } else {
            let idx = resources.rng.gen_range(0..walkable.len());
            walkable[idx]
        };

        let config = SpawnConfig {
            settlement_id: Some(settlement_id),
            position: pos,
            initial_age_ticks,
            sex: None,
            parent_a: None,
            parent_b: None,
        };

        spawn_agent(world, resources, &config);
    }
    log::info!("[entity_spawner] Spawned {} agents into hecs::World", count);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use sim_core::config::GameConfig;
    use sim_core::{GameCalendar, WorldMap};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let cal = GameCalendar::new(&config);
        let map = WorldMap::new(32, 32, 0);
        SimResources::new(cal, map, 42)
    }

    #[test]
    fn spawn_single_agent_adds_one_entity() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (5, 5),
            initial_age_ticks: 15 * TICKS_PER_YEAR,
            sex: None,
            parent_a: None,
            parent_b: None,
        };

        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        assert_eq!(world.len(), 1, "world should have exactly one entity");

        // Check identity has a valid sex
        let identity = world.get::<&Identity>(entity).expect("Identity missing");
        assert!(
            identity.sex == Sex::Male || identity.sex == Sex::Female,
            "sex must be Male or Female"
        );

        // Check needs has 13 values
        let needs = world.get::<&Needs>(entity).expect("Needs missing");
        assert_eq!(needs.values.len(), 13, "needs must have 13 values");

        // Check age is ~15 years
        let age = world.get::<&Age>(entity).expect("Age missing");
        assert!(
            (age.years - 15.0).abs() < 0.01,
            "age should be ~15 years, got {}",
            age.years
        );
    }

    #[test]
    fn spawn_with_explicit_sex() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (0, 0),
            initial_age_ticks: 20 * TICKS_PER_YEAR,
            sex: Some(Sex::Female),
            parent_a: None,
            parent_b: None,
        };

        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        let identity = world.get::<&Identity>(entity).expect("Identity missing");
        assert_eq!(
            identity.sex,
            Sex::Female,
            "sex should be Female as specified"
        );
    }

    #[test]
    fn spawn_multiple_agents_increments_count() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        for i in 0..5 {
            let cfg = SpawnConfig {
                settlement_id: None,
                position: (i, i),
                initial_age_ticks: 25 * TICKS_PER_YEAR,
                sex: None,
                parent_a: None,
                parent_b: None,
            };
            spawn_agent(&mut world, &mut resources, &cfg);
        }
        assert_eq!(world.len(), 5, "world should have 5 entities");
    }

    #[test]
    fn zodiac_from_tick_returns_valid_sign() {
        let valid = [
            "capricorn",
            "aquarius",
            "pisces",
            "aries",
            "taurus",
            "gemini",
            "cancer",
            "leo",
            "virgo",
            "libra",
            "scorpio",
            "sagittarius",
        ];
        for tick in [0u64, 100, 500, 1000, 2000, 4000, 4379] {
            let sign = zodiac_from_calendar_tick(tick);
            assert!(
                valid.contains(&sign),
                "unexpected zodiac sign '{sign}' for tick {tick}"
            );
        }
    }

    #[test]
    fn body_health_is_one_at_spawn() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (0, 0),
            initial_age_ticks: 0,
            sex: Some(Sex::Male),
            parent_a: None,
            parent_b: None,
        };
        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        let body = world.get::<&Body>(entity).expect("Body missing");
        assert!(
            (body.health - 1.0).abs() < f32::EPSILON,
            "health should be 1.0 at spawn"
        );
    }

    #[test]
    fn adult_spawn_starts_with_three_knowledge_entries() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (0, 0),
            initial_age_ticks: 20 * TICKS_PER_YEAR,
            sex: Some(Sex::Female),
            parent_a: None,
            parent_b: None,
        };

        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        let knowledge = world
            .get::<&AgentKnowledge>(entity)
            .expect("AgentKnowledge missing");

        assert_eq!(knowledge.known_count(), 3);
        assert!(knowledge.has_knowledge("TECH_FIRE_MAKING"));
        assert!(knowledge.has_knowledge("TECH_STONE_KNAPPING"));
        assert!(knowledge.has_knowledge("TECH_FORAGING"));
        assert!(knowledge.learning.is_none());
        assert!(knowledge.teaching_target.is_none());
        assert!(
            (0.2..=0.8).contains(&knowledge.innovation_potential),
            "innovation potential should be clamped into expected range"
        );
    }

    #[test]
    fn child_spawn_starts_without_knowledge_entries() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (0, 0),
            initial_age_ticks: 10 * TICKS_PER_YEAR,
            sex: Some(Sex::Male),
            parent_a: None,
            parent_b: None,
        };

        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        let knowledge = world
            .get::<&AgentKnowledge>(entity)
            .expect("AgentKnowledge missing");

        assert_eq!(knowledge.known_count(), 0);
        assert_eq!(knowledge.innovation_potential, 0.0);
    }

    #[test]
    fn growth_stage_infant_for_newborn() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (0, 0),
            initial_age_ticks: 0,
            sex: None,
            parent_a: None,
            parent_b: None,
        };
        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        let identity = world.get::<&Identity>(entity).expect("Identity missing");
        assert_eq!(
            identity.growth_stage,
            GrowthStage::Infant,
            "newborn should be Infant"
        );
    }

    #[test]
    fn traits_and_faith_components_present() {
        let mut world = hecs::World::new();
        let mut resources = make_resources();

        let cfg = SpawnConfig {
            settlement_id: None,
            position: (0, 0),
            initial_age_ticks: 10 * TICKS_PER_YEAR,
            sex: None,
            parent_a: None,
            parent_b: None,
        };
        let entity = spawn_agent(&mut world, &mut resources, &cfg);
        // These were inserted after spawn — verify they exist
        assert!(
            world.get::<&Traits>(entity).is_ok(),
            "Traits component should be present"
        );
        assert!(
            world.get::<&Faith>(entity).is_ok(),
            "Faith component should be present"
        );
        assert!(
            world.get::<&Temperament>(entity).is_ok(),
            "Temperament component should be present"
        );
        assert!(
            world.get::<&InfluenceReceiver>(entity).is_ok(),
            "InfluenceReceiver component should be present"
        );
        assert!(
            world.get::<&Inventory>(entity).is_ok(),
            "Inventory component should be present"
        );
    }

    #[test]
    fn generate_temperament_uses_registry_rules_when_present() {
        let personality = Personality {
            axes: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
            facets: [0.0; 24],
        };
        let mut resources = make_resources();
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sim-data/data");
        let registry = sim_data::DataRegistry::load_from_directory(&data_dir)
            .expect("expected sim-data registry fixture to load");
        let rules = registry
            .temperament_rules_ref()
            .expect("expected temperament rules fixture");
        assert_eq!(rules.prs_weights.len(), 4);
        resources.data_registry = Some(std::sync::Arc::new(registry));

        let generated = generate_temperament(
            &personality,
            resources.data_registry.as_ref().map(std::sync::Arc::as_ref),
        );

        assert!((generated.latent.ns - 0.036).abs() < 1e-9);
        assert!((generated.latent.ha - 0.023).abs() < 1e-9);
        assert!((generated.latent.rd - 0.036).abs() < 1e-9);
        assert!((generated.latent.p - 0.037).abs() < 1e-9);
    }
}
