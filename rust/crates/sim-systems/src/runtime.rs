use hecs::{Entity, World};
use std::collections::HashMap;
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position, Skills,
    Social, Stress, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, EmotionType, GrowthStage, HexacoAxis, NeedType, RelationType,
    ValueType,
};
use sim_engine::{SimResources, SimSystem};

use crate::body;

/// Rust runtime system for tile resource regeneration.
///
/// This is the Rust execution counterpart of `resource_regen_system.gd`.
#[derive(Debug, Clone)]
pub struct ResourceRegenSystem {
    priority: u32,
    tick_interval: u64,
}

impl ResourceRegenSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for ResourceRegenSystem {
    fn name(&self) -> &'static str {
        "resource_regen_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        let width: u32 = resources.map.width;
        let height: u32 = resources.map.height;
        for y in 0..height {
            for x in 0..width {
                let tile = resources.map.get_mut(x, y);
                for deposit in &mut tile.resources {
                    if deposit.regen_rate <= 0.0 || deposit.amount >= deposit.max_amount {
                        continue;
                    }
                    let next_amount = deposit.amount + deposit.regen_rate;
                    deposit.amount = next_amount.min(deposit.max_amount);
                }
            }
        }
    }
}

/// Rust runtime system for base-needs decay and energy adjustment.
///
/// This ports the hot-path math from `needs_system.gd` using the existing
/// Rust body kernels (`needs_base_decay_step`, `action_energy_cost`,
/// `rest_energy_recovery`).
#[derive(Debug, Clone)]
pub struct NeedsRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl NeedsRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for NeedsRuntimeSystem {
    fn name(&self) -> &'static str {
        "needs_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Needs,
            Option<&Behavior>,
            Option<&BodyComponent>,
            Option<&Position>,
        )>();
        for (_, (needs, behavior_opt, body_opt, position_opt)) in &mut query {
            let mut tile_temp: f32 = config::WARMTH_TEMP_NEUTRAL as f32;
            let mut has_tile_temp = false;
            if let Some(position) = position_opt {
                let x = position.x;
                let y = position.y;
                if resources.map.in_bounds(x, y) {
                    let tile = resources.map.get(x as u32, y as u32);
                    tile_temp = tile.temperature;
                    has_tile_temp = true;
                }
            }

            let decays = body::needs_base_decay_step(
                needs.get(NeedType::Hunger) as f32,
                config::HUNGER_DECAY_RATE as f32,
                1.0,
                config::HUNGER_METABOLIC_MIN as f32,
                config::HUNGER_METABOLIC_RANGE as f32,
                config::ENERGY_DECAY_RATE as f32,
                config::SOCIAL_DECAY_RATE as f32,
                config::SAFETY_DECAY_RATE as f32,
                config::THIRST_DECAY_RATE as f32,
                config::WARMTH_DECAY_RATE as f32,
                tile_temp,
                has_tile_temp,
                config::WARMTH_TEMP_NEUTRAL as f32,
                config::WARMTH_TEMP_FREEZING as f32,
                config::WARMTH_TEMP_COLD as f32,
                true,
            );

            let mut energy = (needs.energy as f32 - decays[1]).clamp(0.0, 1.0);
            if let Some(behavior) = behavior_opt {
                if behavior.current_action != ActionType::Idle
                    && behavior.current_action != ActionType::Rest
                {
                    let end_norm = body_opt
                        .map(|body_component| {
                            (body_component.end_realized as f32 / config::BODY_REALIZED_MAX as f32)
                                .clamp(0.0, 1.0)
                        })
                        .unwrap_or(0.5);
                    let action_cost = body::action_energy_cost(
                        config::ENERGY_ACTION_COST as f32,
                        end_norm,
                        config::BODY_END_COST_REDUCTION as f32,
                    );
                    energy = (energy - action_cost).clamp(0.0, 1.0);
                } else if behavior.current_action == ActionType::Rest {
                    let rec_norm = body_opt
                        .map(|body_component| {
                            (body_component.rec_realized as f32 / config::BODY_REALIZED_MAX as f32)
                                .clamp(0.0, 1.0)
                        })
                        .unwrap_or(0.5);
                    let recovery = body::rest_energy_recovery(
                        config::BODY_REST_ENERGY_RECOVERY as f32,
                        rec_norm,
                        config::BODY_REC_RECOVERY_BONUS as f32,
                    );
                    energy = (energy + recovery).clamp(0.0, 1.0);
                }
            }

            needs.set(
                NeedType::Hunger,
                (needs.get(NeedType::Hunger) as f32 - decays[0]) as f64,
            );
            needs.set(
                NeedType::Belonging,
                (needs.get(NeedType::Belonging) as f32 - decays[2]) as f64,
            );
            needs.set(
                NeedType::Thirst,
                (needs.get(NeedType::Thirst) as f32 - decays[3]) as f64,
            );
            needs.set(
                NeedType::Warmth,
                (needs.get(NeedType::Warmth) as f32 - decays[4]) as f64,
            );
            needs.set(
                NeedType::Safety,
                (needs.get(NeedType::Safety) as f32 - decays[5]) as f64,
            );
            needs.energy = energy as f64;
            needs.set(NeedType::Sleep, energy as f64);
        }
    }
}

/// Rust runtime system for stress-state updates.
///
/// This system performs actual component writes (`Stress.level/reserve/allostatic_load/state`)
/// and is the first step of the strict Phase-5 active-write migration for stress.
#[derive(Debug, Clone)]
pub struct StressRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StressRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StressRuntimeSystem {
    fn name(&self) -> &'static str {
        "stress_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Needs, &mut Stress, Option<&Emotion>)>();
        for (_, (needs, stress, emotion_opt)) in &mut query {
            let hunger: f32 = needs.get(NeedType::Hunger) as f32;
            let thirst: f32 = needs.get(NeedType::Thirst) as f32;
            let warmth: f32 = needs.get(NeedType::Warmth) as f32;
            let safety: f32 = needs.get(NeedType::Safety) as f32;
            let social: f32 = needs.get(NeedType::Belonging) as f32;
            let energy: f32 = needs.energy as f32;

            let critical = body::needs_critical_severity_step(
                thirst,
                warmth,
                safety,
                config::THIRST_CRITICAL as f32,
                config::WARMTH_CRITICAL as f32,
                config::SAFETY_CRITICAL as f32,
            );
            let hunger_critical: f32 = ((0.15 - hunger) / 0.15).clamp(0.0, 1.0);

            let (negative_emotion, positive_emotion) = if let Some(emotion) = emotion_opt {
                let neg = (emotion.get(EmotionType::Fear)
                    + emotion.get(EmotionType::Anger)
                    + emotion.get(EmotionType::Sadness)
                    + emotion.get(EmotionType::Disgust)
                    + emotion.get(EmotionType::Surprise))
                    as f32
                    / 5.0;
                let pos = (emotion.get(EmotionType::Joy)
                    + emotion.get(EmotionType::Trust)
                    + emotion.get(EmotionType::Anticipation))
                    as f32
                    / 3.0;
                (neg, pos)
            } else {
                (0.0, 0.0)
            };

            let need_pressure = hunger_critical
                + critical[0]
                + critical[1]
                + critical[2]
                + (1.0 - energy).clamp(0.0, 1.0)
                + (1.0 - social).clamp(0.0, 1.0);
            let stress_delta = need_pressure * 0.02 + negative_emotion * 0.01 - positive_emotion * 0.005;

            let next_level: f32 = (stress.level as f32 + stress_delta).clamp(0.0, 1.0);
            let reserve_delta: f32 = if next_level > 0.6 {
                -0.002 * ((next_level - 0.6) / 0.4)
            } else {
                0.001 * ((0.6 - next_level) / 0.6)
            };
            let next_reserve: f32 = (stress.reserve as f32 + reserve_delta).clamp(0.0, 1.0);
            let allostatic_delta: f32 = if next_level > 0.7 {
                0.0015 * ((next_level - 0.7) / 0.3)
            } else {
                -0.0005
            };
            let next_allostatic: f32 =
                (stress.allostatic_load as f32 + allostatic_delta).clamp(0.0, 1.0);

            stress.level = next_level as f64;
            stress.reserve = next_reserve as f64;
            stress.allostatic_load = next_allostatic as f64;
            stress.recalculate_state();
        }
    }
}

/// Rust runtime system for emotion baseline/primary updates.
///
/// This performs active-write updates on `Emotion.primary` and `Emotion.baseline`
/// using stress/needs context and personality-adjusted decay.
#[derive(Debug, Clone)]
pub struct EmotionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl EmotionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn personality_z(personality_opt: Option<&Personality>, axis: HexacoAxis) -> f32 {
    let axis_val = personality_opt
        .map(|personality| personality.axis(axis) as f32)
        .unwrap_or(0.5);
    ((axis_val - 0.5) / 0.15).clamp(-3.0, 3.0)
}

#[inline]
fn needs_value(needs_opt: Option<&Needs>, need_type: NeedType, fallback: f32) -> f32 {
    needs_opt
        .map(|needs| needs.get(need_type) as f32)
        .unwrap_or(fallback)
        .clamp(0.0, 1.0)
}

impl SimSystem for EmotionRuntimeSystem {
    fn name(&self) -> &'static str {
        "emotion_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Emotion,
            Option<&Stress>,
            Option<&Needs>,
            Option<&Personality>,
        )>();
        for (_, (emotion, stress_opt, needs_opt, personality_opt)) in &mut query {
            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);

            let hunger = needs_value(needs_opt, NeedType::Hunger, 1.0);
            let thirst = needs_value(needs_opt, NeedType::Thirst, 1.0);
            let warmth = needs_value(needs_opt, NeedType::Warmth, 1.0);
            let safety = needs_value(needs_opt, NeedType::Safety, 1.0);
            let social = needs_value(needs_opt, NeedType::Belonging, 1.0);
            let energy = needs_opt
                .map(|needs| needs.energy as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);

            let deficit_hunger = (1.0 - hunger).clamp(0.0, 1.0);
            let deficit_thirst = (1.0 - thirst).clamp(0.0, 1.0);
            let deficit_warmth = (1.0 - warmth).clamp(0.0, 1.0);
            let deficit_safety = (1.0 - safety).clamp(0.0, 1.0);
            let deficit_social = (1.0 - social).clamp(0.0, 1.0);
            let deficit_energy = (1.0 - energy).clamp(0.0, 1.0);

            let z_e = personality_z(personality_opt, HexacoAxis::E);
            let z_a = personality_z(personality_opt, HexacoAxis::A);
            let z_c = personality_z(personality_opt, HexacoAxis::C);

            let baseline_fear = body::emotion_baseline_value(0.10, 0.02, z_e, 0.0, 1.0);
            let baseline_anger = body::emotion_baseline_value(0.08, -0.02, z_a, 0.0, 1.0);
            let baseline_sadness = body::emotion_baseline_value(0.08, 0.015, z_e, 0.0, 1.0);
            let baseline_disgust = body::emotion_baseline_value(0.06, -0.01, z_a, 0.0, 1.0);
            let baseline_surprise = body::emotion_baseline_value(0.10, 0.01, z_e, 0.0, 1.0);
            let baseline_joy = body::emotion_baseline_value(0.25, -0.02, z_e, 0.0, 1.0);
            let baseline_trust = body::emotion_baseline_value(0.22, 0.02, z_a, 0.0, 1.0);
            let baseline_anticipation = body::emotion_baseline_value(0.20, 0.01, z_c, 0.0, 1.0);

            let fear_target = (baseline_fear + stress_level * 0.50 + deficit_safety * 0.35).clamp(0.0, 1.0);
            let anger_target = (baseline_anger + stress_level * 0.35 + deficit_hunger * 0.20).clamp(0.0, 1.0);
            let sadness_target = (baseline_sadness + stress_level * 0.30 + deficit_social * 0.30).clamp(0.0, 1.0);
            let disgust_target = (baseline_disgust
                + stress_level * 0.18
                + (deficit_thirst * 0.10 + deficit_warmth * 0.10))
                .clamp(0.0, 1.0);
            let surprise_target = (baseline_surprise + stress_level * 0.15).clamp(0.0, 1.0);
            let joy_target = (baseline_joy
                - stress_level * 0.35
                + social * 0.20
                + energy * 0.15
                - deficit_energy * 0.10)
                .clamp(0.0, 1.0);
            let trust_target = (baseline_trust - stress_level * 0.25 + social * 0.20 + safety * 0.20)
                .clamp(0.0, 1.0);
            let anticipation_target =
                (baseline_anticipation - stress_level * 0.20 + energy * 0.20 + safety * 0.15)
                    .clamp(0.0, 1.0);

            let hl = body::emotion_adjusted_half_life(4.0, 0.25, z_c).max(0.001);
            let k = 0.693_147 / hl;
            let decay = (-k).exp();
            let blend = (1.0 - decay).clamp(0.0, 1.0);

            let update = |current: f64, target: f32| -> f64 {
                ((current as f32) * decay + target * blend).clamp(0.0, 1.0) as f64
            };

            emotion.baseline[EmotionType::Fear as usize] = baseline_fear as f64;
            emotion.baseline[EmotionType::Anger as usize] = baseline_anger as f64;
            emotion.baseline[EmotionType::Sadness as usize] = baseline_sadness as f64;
            emotion.baseline[EmotionType::Disgust as usize] = baseline_disgust as f64;
            emotion.baseline[EmotionType::Surprise as usize] = baseline_surprise as f64;
            emotion.baseline[EmotionType::Joy as usize] = baseline_joy as f64;
            emotion.baseline[EmotionType::Trust as usize] = baseline_trust as f64;
            emotion.baseline[EmotionType::Anticipation as usize] = baseline_anticipation as f64;

            *emotion.get_mut(EmotionType::Fear) = update(emotion.get(EmotionType::Fear), fear_target);
            *emotion.get_mut(EmotionType::Anger) = update(emotion.get(EmotionType::Anger), anger_target);
            *emotion.get_mut(EmotionType::Sadness) =
                update(emotion.get(EmotionType::Sadness), sadness_target);
            *emotion.get_mut(EmotionType::Disgust) =
                update(emotion.get(EmotionType::Disgust), disgust_target);
            *emotion.get_mut(EmotionType::Surprise) =
                update(emotion.get(EmotionType::Surprise), surprise_target);
            *emotion.get_mut(EmotionType::Joy) = update(emotion.get(EmotionType::Joy), joy_target);
            *emotion.get_mut(EmotionType::Trust) =
                update(emotion.get(EmotionType::Trust), trust_target);
            *emotion.get_mut(EmotionType::Anticipation) =
                update(emotion.get(EmotionType::Anticipation), anticipation_target);
        }
    }
}

/// Rust runtime system for reputation decay/event application.
///
/// This updates `Social.reputation_local/regional/tags` every tick from
/// emotion/stress/needs/action signals using the shared reputation kernels.
#[derive(Debug, Clone)]
pub struct ReputationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ReputationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn reputation_domain_code(action: ActionType) -> i32 {
    match action {
        ActionType::Socialize | ActionType::Teach | ActionType::Learn | ActionType::VisitPartner => 1, // sociability
        ActionType::Build
        | ActionType::Craft
        | ActionType::Forage
        | ActionType::Hunt
        | ActionType::Fish
        | ActionType::GatherWood
        | ActionType::GatherStone
        | ActionType::GatherHerbs
        | ActionType::DeliverToStockpile => 2, // competence
        ActionType::Fight => 3, // dominance
        ActionType::TakeFromStockpile => 4, // generosity
        _ => 0,                 // morality
    }
}

#[inline]
fn reputation_neg_bias(domain_code: i32) -> f32 {
    match domain_code {
        1 => config::REP_NEG_BIAS_SOCIABILITY as f32,
        2 => config::REP_NEG_BIAS_COMPETENCE as f32,
        3 => config::REP_NEG_BIAS_DOMINANCE as f32,
        4 => config::REP_NEG_BIAS_GENEROSITY as f32,
        _ => config::REP_NEG_BIAS_MORALITY as f32,
    }
}

#[inline]
fn reputation_to_signed(value_01: f64) -> f32 {
    ((value_01 as f32) * 2.0 - 1.0).clamp(-1.0, 1.0)
}

#[inline]
fn reputation_to_unit(value_signed: f32) -> f64 {
    (((value_signed.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0)) as f64
}

impl SimSystem for ReputationRuntimeSystem {
    fn name(&self) -> &'static str {
        "reputation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let ticks_per_year =
            (config::TICKS_PER_YEAR as f32 / config::REPUTATION_TICK_INTERVAL as f32).max(1.0);
        let pos_decay =
            (config::REP_POSITIVE_YEARLY_RETENTION as f32).powf((1.0 / ticks_per_year).max(0.0));
        let neg_decay =
            (config::REP_NEGATIVE_YEARLY_RETENTION as f32).powf((1.0 / ticks_per_year).max(0.0));

        let mut query = world.query::<(
            &mut Social,
            Option<&Emotion>,
            Option<&Stress>,
            Option<&Needs>,
            Option<&Behavior>,
            Option<&Values>,
        )>();
        for (_, (social, emotion_opt, stress_opt, needs_opt, behavior_opt, values_opt)) in &mut query {
            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let hunger = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let safety = needs_opt
                .map(|needs| needs.get(NeedType::Safety) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let social_need = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);

            let (positive_emotion, negative_emotion) = if let Some(emotion) = emotion_opt {
                let positive = (emotion.get(EmotionType::Joy)
                    + emotion.get(EmotionType::Trust)
                    + emotion.get(EmotionType::Anticipation)) as f32
                    / 3.0;
                let negative = (emotion.get(EmotionType::Fear)
                    + emotion.get(EmotionType::Anger)
                    + emotion.get(EmotionType::Sadness)
                    + emotion.get(EmotionType::Disgust)
                    + emotion.get(EmotionType::Surprise)) as f32
                    / 5.0;
                (positive, negative)
            } else {
                (0.5, 0.5)
            };

            let action = behavior_opt
                .map(|behavior| behavior.current_action)
                .unwrap_or(ActionType::Idle);
            let action_signal = match action {
                ActionType::Socialize | ActionType::Teach | ActionType::Learn | ActionType::VisitPartner => 0.10,
                ActionType::Fight | ActionType::MentalBreak | ActionType::Flee => -0.15,
                ActionType::Build
                | ActionType::Craft
                | ActionType::Forage
                | ActionType::Hunt
                | ActionType::Fish
                | ActionType::GatherWood
                | ActionType::GatherStone
                | ActionType::GatherHerbs => 0.05,
                _ => 0.0,
            };

            let valence =
                (positive_emotion - negative_emotion + action_signal - stress_level * 0.25)
                    .clamp(-1.0, 1.0);
            let magnitude = (0.25
                + stress_level * 0.35
                + (1.0 - social_need) * 0.20
                + (1.0 - safety) * 0.10
                + (1.0 - hunger) * 0.10)
                .clamp(0.0, 1.0);

            let domain_code = reputation_domain_code(action);
            let neg_bias = if valence < 0.0 {
                reputation_neg_bias(domain_code)
            } else {
                1.0
            };
            let event_delta = body::reputation_event_delta(
                valence,
                magnitude,
                config::REP_EVENT_DELTA_SCALE as f32,
                neg_bias,
            );

            let decayed_local =
                body::reputation_decay_value(reputation_to_signed(social.reputation_local), pos_decay, neg_decay);
            let decayed_regional =
                body::reputation_decay_value(reputation_to_signed(social.reputation_regional), pos_decay, neg_decay);
            let next_local = (decayed_local + event_delta).clamp(-1.0, 1.0);
            let next_regional = (decayed_regional + event_delta * 0.5).clamp(-1.0, 1.0);

            social.reputation_local = reputation_to_unit(next_local);
            social.reputation_regional = reputation_to_unit(next_regional);

            let mut tags = Vec::<String>::new();
            if next_local >= config::REP_TIER_RESPECTED as f32 {
                tags.push("respected".to_string());
            } else if next_local >= config::REP_TIER_GOOD as f32 {
                tags.push("good".to_string());
            } else if next_local <= config::REP_TIER_OUTCAST as f32 {
                tags.push("outcast".to_string());
            } else if next_local <= config::REP_TIER_SUSPECT as f32 {
                tags.push("suspect".to_string());
            }

            if let Some(values) = values_opt {
                let cooperation = values.get(ValueType::Cooperation) as f32;
                let power = values.get(ValueType::Power) as f32;
                if cooperation > 0.30 && next_local >= config::REP_TIER_GOOD as f32 {
                    tags.push("generous".to_string());
                }
                if power > 0.40 && next_local <= config::REP_TIER_SUSPECT as f32 {
                    tags.push("domineering".to_string());
                }
            }
            social.reputation_tags = tags;
        }
    }
}

/// Rust runtime system for social-event interaction updates.
///
/// This performs active writes on `Social.edges` and `Social.social_capital`,
/// replacing no-op baseline behavior with deterministic per-tick interaction updates.
#[derive(Debug, Clone)]
pub struct SocialEventRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl SocialEventRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn attachment_socialize_mult(personality_opt: Option<&Personality>) -> f32 {
    let index = personality_opt
        .map(|personality| match personality.attachment {
            AttachmentType::Secure => 0,
            AttachmentType::Anxious => 1,
            AttachmentType::Avoidant => 2,
            AttachmentType::Fearful => 3,
        })
        .unwrap_or(0);
    config::ATTACHMENT_SOCIALIZE_MULT[index] as f32
}

#[inline]
fn action_social_drive(action: ActionType) -> f32 {
    match action {
        ActionType::Socialize | ActionType::VisitPartner => 1.00,
        ActionType::Idle | ActionType::Rest => 0.45,
        ActionType::Build
        | ActionType::Craft
        | ActionType::Forage
        | ActionType::Hunt
        | ActionType::Fish
        | ActionType::GatherWood
        | ActionType::GatherStone
        | ActionType::GatherHerbs => 0.60,
        ActionType::Fight | ActionType::Flee | ActionType::MentalBreak => 0.15,
        _ => 0.25,
    }
}

impl SimSystem for SocialEventRuntimeSystem {
    fn name(&self) -> &'static str {
        "social_event_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(
            &mut Social,
            Option<&Personality>,
            Option<&Behavior>,
            Option<&Needs>,
            Option<&Stress>,
        )>();
        for (_, (social, personality_opt, behavior_opt, needs_opt, stress_opt)) in &mut query {
            let extraversion = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let agreeableness = personality_opt
                .map(|personality| personality.axis(HexacoAxis::A) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let social_need = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let action = behavior_opt
                .map(|behavior| behavior.current_action)
                .unwrap_or(ActionType::Idle);

            let social_drive = action_social_drive(action);
            let attach_mult = body::social_attachment_affinity_multiplier(
                attachment_socialize_mult(personality_opt),
                1.0,
            );
            let compatibility =
                (extraversion * 0.45 + agreeableness * 0.35 + social_need * 0.20).clamp(0.0, 1.0);

            for edge in &mut social.edges {
                let romantic_interest = if matches!(
                    edge.relation_type,
                    RelationType::Intimate | RelationType::Spouse
                ) {
                    80.0
                } else {
                    20.0
                };
                let proposal_prob =
                    body::social_proposal_accept_prob(romantic_interest, compatibility);
                let interaction = (social_drive * 0.5 + proposal_prob * 0.5).clamp(0.0, 1.0);

                let mut affinity_delta = ((interaction - 0.35) * 6.0
                    - stress_level * 1.5
                    + (social_need - 0.5) * 1.2)
                    * attach_mult;
                if stress_level > 0.7 && agreeableness < 0.4 {
                    affinity_delta -= 1.5;
                }
                let trust_delta =
                    (interaction * 0.04 + agreeableness * 0.02 - stress_level * 0.03).clamp(-0.06, 0.06);
                let familiarity_delta = (0.01 + interaction * 0.02).clamp(0.0, 0.04);

                edge.affinity = ((edge.affinity as f32 + affinity_delta).clamp(0.0, 100.0)) as f64;
                edge.trust = ((edge.trust as f32 + trust_delta).clamp(0.0, 1.0)) as f64;
                edge.familiarity =
                    ((edge.familiarity as f32 + familiarity_delta).clamp(0.0, 1.0)) as f64;
                edge.last_interaction_tick = tick;
                edge.update_type();
            }

            let mut strong_count = 0.0_f32;
            let mut weak_count = 0.0_f32;
            let mut bridge_count = 0.0_f32;
            for edge in &social.edges {
                if edge.affinity >= config::NETWORK_TIE_STRONG_MIN {
                    strong_count += 1.0;
                } else if edge.affinity >= config::NETWORK_TIE_WEAK_MIN {
                    weak_count += 1.0;
                }
                if edge.is_bridge {
                    bridge_count += 1.0;
                }
            }
            let rep_score = ((social.reputation_local as f32 + social.reputation_regional as f32) * 0.5)
                .clamp(0.0, 1.0);
            social.social_capital = body::network_social_capital_norm(
                strong_count,
                weak_count,
                bridge_count,
                rep_score,
                config::NETWORK_SOCIAL_CAP_STRONG_W as f32,
                config::NETWORK_SOCIAL_CAP_WEAK_W as f32,
                config::NETWORK_SOCIAL_CAP_BRIDGE_W as f32,
                config::NETWORK_SOCIAL_CAP_REP_W as f32,
                config::NETWORK_SOCIAL_CAP_NORM_DIV as f32,
            )
            .clamp(0.0, 1.0) as f64;
        }
    }
}

/// Rust runtime system for morale-driven behavior/need adjustment.
///
/// This computes a per-entity morale signal and applies active writes to
/// `Behavior.job_satisfaction`, `Behavior.occupation_satisfaction`,
/// `Needs.meaning`, and `Needs.transcendence`.
#[derive(Debug, Clone)]
pub struct MoraleRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MoraleRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn maslow_morale_multiplier(hunger: f32, energy: f32, safety: f32, belonging: f32) -> f32 {
    if hunger < 0.3 || energy < 0.3 {
        0.0
    } else if hunger < 0.6 || energy < 0.6 {
        0.4
    } else if safety < 0.3 {
        0.2
    } else if safety < 0.6 {
        0.6
    } else if belonging < 0.3 {
        0.7
    } else {
        1.0
    }
}

impl SimSystem for MoraleRuntimeSystem {
    fn name(&self) -> &'static str {
        "morale_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Needs,
            &mut Behavior,
            Option<&Emotion>,
            Option<&Stress>,
            Option<&Personality>,
            Option<&Social>,
        )>();
        for (_, (needs, behavior, emotion_opt, stress_opt, personality_opt, social_opt)) in &mut query {
            let hunger = needs.get(NeedType::Hunger) as f32;
            let energy = needs.energy as f32;
            let safety = needs.get(NeedType::Safety) as f32;
            let belonging = needs.get(NeedType::Belonging) as f32;

            let pa = emotion_opt
                .map(|emotion| {
                    ((emotion.get(EmotionType::Joy)
                        + emotion.get(EmotionType::Trust)
                        + emotion.get(EmotionType::Anticipation)) as f32
                        / 3.0)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let na = emotion_opt
                .map(|emotion| {
                    ((emotion.get(EmotionType::Fear)
                        + emotion.get(EmotionType::Anger)
                        + emotion.get(EmotionType::Sadness)
                        + emotion.get(EmotionType::Disgust)
                        + emotion.get(EmotionType::Surprise)) as f32
                        / 5.0)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let ls = ((hunger + energy + belonging) / 3.0).clamp(0.0, 1.0);
            let maslow_mult = maslow_morale_multiplier(hunger, energy, safety, belonging);

            let mut morale = (0.40 * pa - 0.30 * na + 0.30 * ls) * maslow_mult;
            let hygiene_threshold = 0.5;
            let hygiene_penalty_rate = 0.8;
            if safety < hygiene_threshold {
                morale -= (hygiene_threshold - safety) * hygiene_penalty_rate * 0.3;
            }
            if belonging < hygiene_threshold {
                morale -= (hygiene_threshold - belonging) * hygiene_penalty_rate * 0.2;
            }
            if hunger < hygiene_threshold {
                morale -= (hygiene_threshold - hunger) * hygiene_penalty_rate * 0.25;
            }
            if energy < hygiene_threshold {
                morale -= (hygiene_threshold - energy) * hygiene_penalty_rate * 0.25;
            }

            let extraversion = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let autonomy = needs.get(NeedType::Autonomy) as f32;
            let warr_autonomy = -1.5 * (autonomy - 0.6).powi(2) + 0.15;
            let warr_social = -2.0 * (belonging - 0.5).powi(2) + 0.12;
            let warr_info = (extraversion / 0.7).min(1.0) * 0.10;
            morale = (morale + (warr_autonomy + warr_social + warr_info).clamp(-0.3, 0.3))
                .clamp(-1.0, 1.0);

            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let settlement_morale = if let Some(social) = social_opt {
                ((morale + social.social_capital as f32) * 0.5).clamp(-1.0, 1.0)
            } else {
                morale
            };
            let patience = personality_opt
                .map(|personality| personality.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let migration_p = body::morale_migration_probability(
                settlement_morale,
                10.0,
                0.35,
                patience,
                0.3,
                0.95,
            );

            let behavior_weight = body::morale_behavior_weight_multiplier(
                morale,
                0.6,
                1.2,
                1.55,
                0.85,
                1.2,
                0.55,
                0.85,
                0.30,
                0.55,
            );
            let stress_penalty = (stress_level * 0.12).clamp(0.0, 0.12);
            behavior.job_satisfaction =
                (behavior.job_satisfaction + (behavior_weight - 1.0) * 0.12 - stress_penalty)
                    .clamp(0.0, 1.0);
            behavior.occupation_satisfaction = (behavior.occupation_satisfaction
                + (behavior_weight - 1.0) * 0.10
                - stress_penalty * 0.8)
                .clamp(0.0, 1.0);

            let morale_unit = ((morale + 1.0) * 0.5).clamp(0.0, 1.0);
            let next_meaning =
                (needs.get(NeedType::Meaning) as f32 * 0.85 + morale_unit * 0.15 - migration_p * 0.05)
                    .clamp(0.0, 1.0);
            let next_transcendence = (needs.get(NeedType::Transcendence) as f32 * 0.90
                + morale_unit * 0.10
                - migration_p * 0.03)
                .clamp(0.0, 1.0);
            needs.set(NeedType::Meaning, next_meaning as f64);
            needs.set(NeedType::Transcendence, next_transcendence as f64);
        }
    }
}

/// Rust runtime system for value drift updates.
///
/// This applies age-plasticity-scaled drift to selected `Values` axes using
/// personality, needs, stress, and social context signals.
#[derive(Debug, Clone)]
pub struct ValueRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ValueRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn to_bipolar(value_01: f32) -> f32 {
    (value_01.clamp(0.0, 1.0) * 2.0 - 1.0).clamp(-1.0, 1.0)
}

impl SimSystem for ValueRuntimeSystem {
    fn name(&self) -> &'static str {
        "value_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Values,
            Option<&Age>,
            Option<&Personality>,
            Option<&Needs>,
            Option<&Stress>,
            Option<&Social>,
        )>();
        for (_, (values, age_opt, personality_opt, needs_opt, stress_opt, social_opt)) in &mut query {
            let age_years = age_opt.map(|age| age.years as f32).unwrap_or(25.0).max(0.0);
            let plasticity = body::value_plasticity(age_years).clamp(0.10, 1.0);

            let h = personality_opt
                .map(|personality| personality.axis(HexacoAxis::H) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let a = personality_opt
                .map(|personality| personality.axis(HexacoAxis::A) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let x = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);

            let safety = needs_opt
                .map(|needs| needs.get(NeedType::Safety) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let belonging = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let social_cap = social_opt
                .map(|social| social.social_capital as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let stress_level = stress_opt
                .map(|stress| stress.level as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);

            let pressure = (stress_level * 0.6 + (1.0 - safety) * 0.4).clamp(0.0, 1.0);
            let step = (0.02 * plasticity * (1.0 - pressure * 0.35)).clamp(0.002, 0.03);

            let target_cooperation = to_bipolar((a + belonging + social_cap) / 3.0);
            let target_fairness = to_bipolar((h + a + (1.0 - pressure)) / 3.0);
            let target_family = to_bipolar((belonging + h) * 0.5);
            let target_friendship = to_bipolar((belonging + x) * 0.5);
            let target_law = to_bipolar((safety + h) * 0.5);
            let target_power = to_bipolar((pressure + (1.0 - a) * 0.5 + (1.0 - h) * 0.5).clamp(0.0, 1.0));
            let target_competition = to_bipolar(((pressure + x) * 0.5).clamp(0.0, 1.0));
            let target_peace = to_bipolar(((1.0 - stress_level) * 0.6 + a * 0.4).clamp(0.0, 1.0));

            let mut apply_drift = |value_type: ValueType, target: f32| {
                let current = values.get(value_type) as f32;
                let next = (current + (target - current) * step).clamp(-1.0, 1.0);
                values.set(value_type, next as f64);
            };

            apply_drift(ValueType::Cooperation, target_cooperation);
            apply_drift(ValueType::Fairness, target_fairness);
            apply_drift(ValueType::Family, target_family);
            apply_drift(ValueType::Friendship, target_friendship);
            apply_drift(ValueType::Law, target_law);
            apply_drift(ValueType::Power, target_power);
            apply_drift(ValueType::Competition, target_competition);
            apply_drift(ValueType::Peace, target_peace);
        }
    }
}

/// Rust runtime system for job-satisfaction updates.
///
/// Uses `body::job_satisfaction_score` to update
/// `Behavior.job_satisfaction` and `Behavior.occupation_satisfaction`.
#[derive(Debug, Clone)]
pub struct JobSatisfactionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl JobSatisfactionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[derive(Debug, Clone)]
struct JobSatProfile {
    personality_ideal: [f32; 6],
    value_weights: [f32; 6],
    primary_skill: &'static str,
    autonomy_level: f32,
    prestige: f32,
}

#[inline]
fn job_satisfaction_profile(job: &str) -> JobSatProfile {
    match job {
        "builder" => JobSatProfile {
            personality_ideal: [0.10, 0.00, -0.10, 0.20, 0.80, 0.10],
            value_weights: [0.20, 0.20, 0.10, 0.30, 0.10, 0.10],
            primary_skill: "SKILL_CONSTRUCTION",
            autonomy_level: 0.55,
            prestige: 0.45,
        },
        "miner" => JobSatProfile {
            personality_ideal: [0.05, -0.10, -0.15, 0.10, 0.75, -0.05],
            value_weights: [0.15, 0.15, 0.20, 0.25, 0.05, 0.20],
            primary_skill: "SKILL_MINING",
            autonomy_level: 0.45,
            prestige: 0.40,
        },
        "lumberjack" => JobSatProfile {
            personality_ideal: [0.10, 0.00, 0.10, 0.15, 0.65, 0.20],
            value_weights: [0.20, 0.15, 0.15, 0.20, 0.20, 0.10],
            primary_skill: "SKILL_WOODCUTTING",
            autonomy_level: 0.50,
            prestige: 0.38,
        },
        "hunter" => JobSatProfile {
            personality_ideal: [0.00, 0.10, 0.25, 0.10, 0.60, 0.20],
            value_weights: [0.10, 0.10, 0.30, 0.15, 0.20, 0.15],
            primary_skill: "SKILL_HUNTING",
            autonomy_level: 0.60,
            prestige: 0.55,
        },
        _ => JobSatProfile {
            personality_ideal: [0.10, 0.00, 0.15, 0.20, 0.50, 0.15],
            value_weights: [0.20, 0.20, 0.15, 0.20, 0.15, 0.10],
            primary_skill: "SKILL_FORAGING",
            autonomy_level: 0.50,
            prestige: 0.35,
        },
    }
}

impl SimSystem for JobSatisfactionRuntimeSystem {
    fn name(&self) -> &'static str {
        "job_satisfaction_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Behavior,
            Option<&Personality>,
            Option<&Values>,
            Option<&Needs>,
            Option<&Skills>,
            Option<&Age>,
        )>();
        for (_, (behavior, personality_opt, values_opt, needs_opt, skills_opt, age_opt)) in &mut query {
            if let Some(age) = age_opt {
                if matches!(age.stage, GrowthStage::Infant | GrowthStage::Toddler | GrowthStage::Child) {
                    behavior.job_satisfaction = 0.50;
                    behavior.occupation_satisfaction = 0.50;
                    continue;
                }
            }

            if behavior.job.is_empty() || behavior.job == "none" {
                behavior.job_satisfaction = 0.50;
                behavior.occupation_satisfaction = 0.50;
                continue;
            }

            let profile = job_satisfaction_profile(behavior.job.as_str());
            let personality_actual = if let Some(personality) = personality_opt {
                [
                    personality.axis(HexacoAxis::H) as f32,
                    personality.axis(HexacoAxis::E) as f32,
                    personality.axis(HexacoAxis::X) as f32,
                    personality.axis(HexacoAxis::A) as f32,
                    personality.axis(HexacoAxis::C) as f32,
                    personality.axis(HexacoAxis::O) as f32,
                ]
            } else {
                [0.5; 6]
            };

            let value_actual = if let Some(values) = values_opt {
                [
                    ((values.get(ValueType::Cooperation) as f32 + 1.0) * 0.5).clamp(0.0, 1.0),
                    ((values.get(ValueType::Fairness) as f32 + 1.0) * 0.5).clamp(0.0, 1.0),
                    ((values.get(ValueType::Competition) as f32 + 1.0) * 0.5).clamp(0.0, 1.0),
                    ((values.get(ValueType::HardWork) as f32 + 1.0) * 0.5).clamp(0.0, 1.0),
                    ((values.get(ValueType::Nature) as f32 + 1.0) * 0.5).clamp(0.0, 1.0),
                    ((values.get(ValueType::Power) as f32 + 1.0) * 0.5).clamp(0.0, 1.0),
                ]
            } else {
                [0.5; 6]
            };

            let skill_fit = skills_opt
                .map(|skills| skills.get_level(profile.primary_skill) as f32 / 10.0)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let autonomy = needs_opt
                .map(|needs| needs.get(NeedType::Autonomy) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let competence = needs_opt
                .map(|needs| needs.get(NeedType::Competence) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let meaning = needs_opt
                .map(|needs| needs.get(NeedType::Meaning) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);

            let sat = body::job_satisfaction_score(
                &personality_actual,
                &profile.personality_ideal,
                &value_actual,
                &profile.value_weights,
                skill_fit,
                autonomy,
                competence,
                meaning,
                profile.autonomy_level,
                profile.prestige,
                config::JOB_SAT_W_SKILL_FIT as f32,
                config::JOB_SAT_W_VALUE_FIT as f32,
                config::JOB_SAT_W_PERSONALITY_FIT as f32,
                config::JOB_SAT_W_NEED_FIT as f32,
            );

            behavior.job_satisfaction = sat.clamp(0.0, 1.0);
            behavior.occupation_satisfaction =
                (behavior.occupation_satisfaction * 0.80 + sat * 0.20).clamp(0.0, 1.0);
        }
    }
}

/// Rust runtime system for annual social-capital normalization.
///
/// This performs active writes on `Social.social_capital` using
/// relationship-edge strength/bridge topology and reputation context.
#[derive(Debug, Clone)]
pub struct NetworkRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl NetworkRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for NetworkRuntimeSystem {
    fn name(&self) -> &'static str {
        "network_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<&mut Social>();
        for (_, social) in &mut query {
            let mut strong_count = 0.0_f32;
            let mut weak_count = 0.0_f32;
            let mut bridge_count = 0.0_f32;

            for edge in &social.edges {
                let affinity = edge.affinity as f32;
                let strong_tie = affinity >= config::NETWORK_TIE_STRONG_MIN as f32
                    || matches!(
                        edge.relation_type,
                        RelationType::CloseFriend | RelationType::Intimate | RelationType::Spouse
                    );
                let weak_tie = affinity >= config::NETWORK_TIE_WEAK_MIN as f32
                    || matches!(edge.relation_type, RelationType::Friend | RelationType::Acquaintance);

                if strong_tie {
                    if edge.is_bridge {
                        bridge_count += 1.0;
                    } else {
                        strong_count += 1.0;
                    }
                } else if weak_tie {
                    if edge.is_bridge {
                        bridge_count += 0.5;
                    } else {
                        weak_count += 1.0;
                    }
                }
            }

            let rep_score =
                ((social.reputation_local as f32 + social.reputation_regional as f32) * 0.5)
                    .clamp(0.0, 1.0);
            social.social_capital = body::network_social_capital_norm(
                strong_count,
                weak_count,
                bridge_count,
                rep_score,
                config::NETWORK_SOCIAL_CAP_STRONG_W as f32,
                config::NETWORK_SOCIAL_CAP_WEAK_W as f32,
                config::NETWORK_SOCIAL_CAP_BRIDGE_W as f32,
                config::NETWORK_SOCIAL_CAP_REP_W as f32,
                config::NETWORK_SOCIAL_CAP_NORM_DIV as f32,
            )
            .clamp(0.0, 1.0) as f64;
        }
    }
}

/// Rust runtime system for occupation assignment and switching.
///
/// This performs active writes on `Behavior.occupation` and `Behavior.job`
/// from skill distribution and age-stage policy.
#[derive(Debug, Clone)]
pub struct OccupationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl OccupationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn skill_id_to_occupation(skill_id: &str) -> String {
    let upper = skill_id.trim();
    if let Some(rest) = upper.strip_prefix("SKILL_") {
        return rest.to_ascii_lowercase();
    }
    upper.to_ascii_lowercase()
}

#[inline]
fn occupation_to_skill_id(occupation: &str) -> String {
    format!("SKILL_{}", occupation.to_ascii_uppercase())
}

#[inline]
fn occupation_to_legacy_job(occupation: &str) -> &'static str {
    match occupation {
        "builder" | "building" | "construction" => "builder",
        "miner" | "mining" => "miner",
        "lumberjack" | "woodcutting" | "logging" => "lumberjack",
        "hunter" | "hunting" => "hunter",
        "none" | "laborer" => "gatherer",
        _ => "gatherer",
    }
}

impl SimSystem for OccupationRuntimeSystem {
    fn name(&self) -> &'static str {
        "occupation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Age, &Skills, &mut Behavior)>();
        for (_, (age, skills, behavior)) in &mut query {
            if matches!(age.stage, GrowthStage::Infant | GrowthStage::Toddler) {
                continue;
            }

            let mut skill_pairs: Vec<(&str, i32)> = skills
                .entries
                .iter()
                .map(|(id, entry)| (id.as_str(), i32::from(entry.level)))
                .collect();
            skill_pairs.sort_by(|left, right| left.0.cmp(right.0));

            let skill_levels: Vec<i32> = skill_pairs.iter().map(|(_, level)| *level).collect();
            let best_index = body::occupation_best_skill_index(&skill_levels);
            let (best_skill_id, best_skill_level) = if best_index >= 0 {
                let idx = best_index as usize;
                if idx < skill_pairs.len() {
                    (skill_pairs[idx].0, skill_pairs[idx].1)
                } else {
                    ("", 0)
                }
            } else {
                ("", 0)
            };

            if best_skill_level < config::OCCUPATION_MIN_SKILL_LEVEL as i32 {
                let new_occupation = if matches!(age.stage, GrowthStage::Child | GrowthStage::Teen)
                {
                    "none"
                } else {
                    "laborer"
                };
                if behavior.occupation != new_occupation {
                    behavior.occupation = new_occupation.to_string();
                }
                behavior.job = occupation_to_legacy_job(new_occupation).to_string();
                continue;
            }

            let new_occupation = skill_id_to_occupation(best_skill_id);
            let old_occupation = behavior.occupation.clone();
            if new_occupation != old_occupation
                && old_occupation != "none"
                && old_occupation != "laborer"
            {
                let current_occ_skill = occupation_to_skill_id(old_occupation.as_str());
                let current_level = i32::from(skills.get_level(current_occ_skill.as_str()));
                let should_switch = body::occupation_should_switch(
                    best_skill_level,
                    current_level,
                    config::OCCUPATION_CHANGE_HYSTERESIS as f32,
                );
                if !should_switch {
                    continue;
                }
            }

            if new_occupation != old_occupation {
                behavior.occupation = new_occupation.clone();
            }
            behavior.job = occupation_to_legacy_job(new_occupation.as_str()).to_string();
        }
    }
}

/// Rust runtime system for emotion/stress contagion propagation.
///
/// This performs active writes to `Emotion.primary` and `Stress.level` using
/// AoE and same-settlement network spread kernels.
#[derive(Debug, Clone)]
pub struct ContagionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ContagionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

/// Rust runtime system for age progression and growth-stage updates.
///
/// This performs active writes on `Age.ticks/years/stage`, mirrors growth
/// stage into `Identity.growth_stage`, and clears builder job for elders.
#[derive(Debug, Clone)]
pub struct AgeRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl AgeRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for AgeRuntimeSystem {
    fn name(&self) -> &'static str {
        "age_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&mut Age, Option<&mut Identity>, Option<&mut Behavior>)>();
        for (_, (age, identity_opt, behavior_opt)) in &mut query {
            if !age.alive {
                continue;
            }
            age.ticks = age.ticks.saturating_add(self.tick_interval);
            age.update_derived(config::TICKS_PER_YEAR as u64);
            if let Some(identity) = identity_opt {
                identity.growth_stage = age.stage;
            }
            if matches!(age.stage, GrowthStage::Elder) {
                if let Some(behavior) = behavior_opt {
                    if behavior.job == "builder" {
                        behavior.job = "none".to_string();
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ContagionSnapshot {
    entity: Entity,
    x: i32,
    y: i32,
    settlement_id: Option<u64>,
    emotions: [f32; 8],
    stress_2000: f32,
    valence: f32,
    x_axis: f32,
    e_axis: f32,
    a_axis: f32,
}

#[inline]
fn contagion_valence(emotions: &[f32; 8]) -> f32 {
    let joy = emotions[EmotionType::Joy as usize];
    let trust = emotions[EmotionType::Trust as usize];
    let anticipation = emotions[EmotionType::Anticipation as usize];
    let fear = emotions[EmotionType::Fear as usize];
    let sadness = emotions[EmotionType::Sadness as usize];
    let anger = emotions[EmotionType::Anger as usize];
    let disgust = emotions[EmotionType::Disgust as usize];
    let surprise = emotions[EmotionType::Surprise as usize];
    let positive = (joy + trust + anticipation) / 3.0;
    let negative = (fear + sadness + anger + disgust + surprise) / 5.0;
    ((positive - negative) * 100.0).clamp(-100.0, 100.0)
}

impl SimSystem for ContagionRuntimeSystem {
    fn name(&self) -> &'static str {
        "contagion_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        const AOE_RADIUS: i32 = 3;
        const NETWORK_HOP_RADIUS: i32 = 15;
        const CROWD_DILUTE_DIVISOR: f32 = 6.0;
        const REFRACTORY_SUSCEPTIBILITY: f32 = 0.25;
        const BASE_MIMICRY_WEIGHT: f32 = 0.08;
        const MAX_EMOTION_CONTAGION_DELTA: f32 = 0.08;
        const MAX_STRESS_CONTAGION_DELTA: f32 = 30.0;
        const STRESS_SCALE: f32 = 2000.0;
        const NETWORK_DECAY: f32 = 0.5;

        let mut snapshots: Vec<ContagionSnapshot> = Vec::new();
        let mut read_query =
            world.query::<(&Position, &Emotion, &Stress, Option<&Personality>, Option<&Identity>)>();
        for (entity, (position, emotion, stress, personality_opt, identity_opt)) in &mut read_query {
            let mut emotions = [0.0_f32; 8];
            for (idx, value) in emotion.primary.iter().enumerate() {
                emotions[idx] = *value as f32;
            }
            let x_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let e_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::E) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let a_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::A) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            snapshots.push(ContagionSnapshot {
                entity,
                x: position.x,
                y: position.y,
                settlement_id: identity_opt.and_then(|identity| identity.settlement_id.map(|id| id.0)),
                emotions,
                stress_2000: (stress.level as f32 * STRESS_SCALE).clamp(0.0, STRESS_SCALE),
                valence: contagion_valence(&emotions),
                x_axis,
                e_axis,
                a_axis,
            });
        }
        drop(read_query);

        if snapshots.len() < 2 {
            return;
        }

        let mut emotion_deltas: HashMap<Entity, [f32; 8]> = HashMap::new();
        let mut stress_deltas: HashMap<Entity, f32> = HashMap::new();
        let emotion_keys: [EmotionType; 5] = [
            EmotionType::Joy,
            EmotionType::Sadness,
            EmotionType::Fear,
            EmotionType::Anger,
            EmotionType::Trust,
        ];

        for recipient in &snapshots {
            let mut donor_count = 0_i32;
            let mut donor_emotion_sums = [0.0_f32; 8];
            let mut donor_stress_sum = 0.0_f32;
            for donor in &snapshots {
                if donor.entity == recipient.entity {
                    continue;
                }
                let dist = (donor.x - recipient.x).abs() + (donor.y - recipient.y).abs();
                if dist > AOE_RADIUS {
                    continue;
                }
                donor_count += 1;
                donor_stress_sum += donor.stress_2000;
                for (idx, sum) in donor_emotion_sums.iter_mut().enumerate() {
                    *sum += donor.emotions[idx];
                }
            }
            if donor_count <= 0 {
                continue;
            }

            let total_susceptibility = body::contagion_aoe_total_susceptibility(
                donor_count,
                CROWD_DILUTE_DIVISOR,
                false,
                REFRACTORY_SUSCEPTIBILITY,
                recipient.x_axis,
                recipient.e_axis,
            );
            let mut entry = [0.0_f32; 8];
            for emotion_key in emotion_keys {
                let idx = emotion_key as usize;
                let donor_avg = donor_emotion_sums[idx] / donor_count as f32;
                let gap = donor_avg - recipient.emotions[idx];
                entry[idx] = (gap * BASE_MIMICRY_WEIGHT * total_susceptibility)
                    .clamp(-MAX_EMOTION_CONTAGION_DELTA, MAX_EMOTION_CONTAGION_DELTA);
            }
            emotion_deltas.insert(recipient.entity, entry);

            let avg_stress = donor_stress_sum / donor_count as f32;
            let stress_gap = avg_stress - recipient.stress_2000;
            let stress_delta = body::contagion_stress_delta(
                stress_gap,
                10.0,
                0.04,
                total_susceptibility,
                MAX_STRESS_CONTAGION_DELTA,
            ) / STRESS_SCALE;
            if stress_delta > 0.0 {
                stress_deltas
                    .entry(recipient.entity)
                    .and_modify(|value| *value += stress_delta)
                    .or_insert(stress_delta);
            }
        }

        for recipient in &snapshots {
            let Some(settlement_id) = recipient.settlement_id else {
                continue;
            };
            let mut donors: Vec<&ContagionSnapshot> = Vec::new();
            for donor in &snapshots {
                if donor.entity == recipient.entity {
                    continue;
                }
                if donor.settlement_id != Some(settlement_id) {
                    continue;
                }
                let dist = (donor.x - recipient.x).abs() + (donor.y - recipient.y).abs();
                if dist <= NETWORK_HOP_RADIUS {
                    donors.push(donor);
                }
            }
            if donors.is_empty() {
                continue;
            }
            let donor_count = donors.len() as i32;
            let avg_valence = donors.iter().map(|donor| donor.valence).sum::<f32>() / donor_count as f32;
            let valence_gap = avg_valence - recipient.valence;
            let valence_delta = body::contagion_network_delta(
                donor_count,
                CROWD_DILUTE_DIVISOR,
                false,
                REFRACTORY_SUSCEPTIBILITY,
                NETWORK_DECAY,
                recipient.a_axis,
                valence_gap,
                0.04,
                4.0,
            );
            if valence_delta.abs() <= 0.01 {
                continue;
            }
            let joy_delta = (valence_delta / 100.0).clamp(-0.04, 0.04);
            let sadness_delta = (-valence_delta / 100.0).clamp(-0.04, 0.04);
            let entry = emotion_deltas.entry(recipient.entity).or_insert([0.0_f32; 8]);
            entry[EmotionType::Joy as usize] += joy_delta;
            entry[EmotionType::Trust as usize] += joy_delta * 0.5;
            entry[EmotionType::Sadness as usize] += sadness_delta;
            entry[EmotionType::Fear as usize] += sadness_delta * 0.5;
        }

        for recipient in &snapshots {
            let spiral_increment = body::contagion_spiral_increment(
                recipient.stress_2000,
                recipient.valence,
                500.0,
                -40.0,
                1500.0,
                60.0,
                3.0,
                12.0,
            ) / STRESS_SCALE;
            if spiral_increment > 0.0 {
                stress_deltas
                    .entry(recipient.entity)
                    .and_modify(|value| *value += spiral_increment)
                    .or_insert(spiral_increment);
            }
        }

        let mut write_query = world.query::<(&mut Emotion, &mut Stress)>();
        for (entity, (emotion, stress)) in &mut write_query {
            if let Some(delta_emotions) = emotion_deltas.get(&entity) {
                for (idx, delta) in delta_emotions.iter().enumerate() {
                    let next = (emotion.primary[idx] as f32 + *delta).clamp(0.0, 1.0);
                    emotion.primary[idx] = next as f64;
                }
            }
            if let Some(stress_delta) = stress_deltas.get(&entity) {
                let next_level = (stress.level as f32 + *stress_delta).clamp(0.0, 1.0);
                stress.level = next_level as f64;
                if *stress_delta > 0.0 {
                    let next_allostatic = (stress.allostatic_load as f32 + *stress_delta * 0.25)
                        .clamp(0.0, 1.0);
                    stress.allostatic_load = next_allostatic as f64;
                }
                stress.recalculate_state();
            }
        }
    }
}

/// Rust runtime system for upper-needs decay/fulfillment.
///
/// The step formula mirrors the `upper_needs_system.gd` Rust-bridge path.
#[derive(Debug, Clone)]
pub struct UpperNeedsRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl UpperNeedsRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn upper_needs_job_code(job: &str) -> i32 {
    match job {
        "builder" | "miner" => 1,
        "gatherer" | "lumberjack" => 2,
        _ => 0,
    }
}

#[inline]
fn upper_need_value(values: &Values, key: ValueType) -> f32 {
    values.get(key) as f32
}

impl SimSystem for UpperNeedsRuntimeSystem {
    fn name(&self) -> &'static str {
        "upper_needs_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Needs,
            Option<&Skills>,
            Option<&Values>,
            Option<&Behavior>,
            Option<&Identity>,
            Option<&Social>,
        )>();
        for (_, (needs, skills_opt, values_opt, behavior_opt, identity_opt, social_opt)) in
            &mut query
        {
            if let Some(identity) = identity_opt {
                if matches!(identity.growth_stage, GrowthStage::Infant | GrowthStage::Toddler) {
                    continue;
                }
            }

            let has_job = behavior_opt
                .map(|behavior| behavior.job.as_str() != "none")
                .unwrap_or(false);
            let has_settlement = identity_opt
                .and_then(|identity| identity.settlement_id)
                .is_some();
            let has_partner = social_opt.and_then(|social| social.spouse).is_some();

            let foraging = skills_opt
                .map(|skills| skills.get_level("SKILL_FORAGING") as i32)
                .unwrap_or(0);
            let woodcutting = skills_opt
                .map(|skills| skills.get_level("SKILL_WOODCUTTING") as i32)
                .unwrap_or(0);
            let mining = skills_opt
                .map(|skills| skills.get_level("SKILL_MINING") as i32)
                .unwrap_or(0);
            let construction = skills_opt
                .map(|skills| skills.get_level("SKILL_CONSTRUCTION") as i32)
                .unwrap_or(0);
            let hunting = skills_opt
                .map(|skills| skills.get_level("SKILL_HUNTING") as i32)
                .unwrap_or(0);
            let skill_levels = [foraging, woodcutting, mining, construction, hunting];
            let best_skill_norm = body::upper_needs_best_skill_normalized(&skill_levels, 100);

            let job_code = behavior_opt
                .map(|behavior| upper_needs_job_code(behavior.job.as_str()))
                .unwrap_or(0);
            let (craftsmanship, skill, hard_work, nature, independence, sacrifice) =
                if let Some(values) = values_opt {
                    (
                        upper_need_value(values, ValueType::Craftsmanship),
                        upper_need_value(values, ValueType::Skill),
                        upper_need_value(values, ValueType::HardWork),
                        upper_need_value(values, ValueType::Nature),
                        upper_need_value(values, ValueType::Independence),
                        upper_need_value(values, ValueType::Sacrifice),
                    )
                } else {
                    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
                };
            let alignment = body::upper_needs_job_alignment(
                job_code,
                craftsmanship,
                skill,
                hard_work,
                nature,
                independence,
            );

            let current_values = [
                needs.get(NeedType::Competence) as f32,
                needs.get(NeedType::Autonomy) as f32,
                needs.get(NeedType::SelfActualization) as f32,
                needs.get(NeedType::Meaning) as f32,
                needs.get(NeedType::Transcendence) as f32,
                needs.get(NeedType::Recognition) as f32,
                needs.get(NeedType::Belonging) as f32,
                needs.get(NeedType::Intimacy) as f32,
            ];
            let decay_values = [
                config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                config::UPPER_NEEDS_MEANING_DECAY as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                config::UPPER_NEEDS_BELONGING_DECAY as f32,
                config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ];
            let out = body::upper_needs_step(
                &current_values,
                &decay_values,
                config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
                config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
                config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
                config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
                config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
                config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
                config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
                config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
                best_skill_norm,
                alignment,
                sacrifice,
                has_job,
                has_settlement,
                has_partner,
            );

            needs.set(NeedType::Competence, out[0] as f64);
            needs.set(NeedType::Autonomy, out[1] as f64);
            needs.set(NeedType::SelfActualization, out[2] as f64);
            needs.set(NeedType::Meaning, out[3] as f64);
            needs.set(NeedType::Transcendence, out[4] as f64);
            needs.set(NeedType::Recognition, out[5] as f64);
            needs.set(NeedType::Belonging, out[6] as f64);
            needs.set(NeedType::Intimacy, out[7] as f64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgeRuntimeSystem, EmotionRuntimeSystem, JobSatisfactionRuntimeSystem,
        MoraleRuntimeSystem, NeedsRuntimeSystem, NetworkRuntimeSystem, ReputationRuntimeSystem,
        ResourceRegenSystem, ContagionRuntimeSystem, OccupationRuntimeSystem,
        SocialEventRuntimeSystem, StressRuntimeSystem, UpperNeedsRuntimeSystem,
        ValueRuntimeSystem,
    };
    use crate::body;
    use hecs::World;
    use sim_core::components::{
        Age, Behavior, Body as BodyComponent, Emotion, Identity, Needs, Personality, Position,
        SkillEntry, Skills, Social, Stress, Values,
    };
    use sim_core::ids::EntityId;
    use sim_core::world::TileResource;
    use sim_core::{
        config::GameConfig, ActionType, EmotionType, GameCalendar, GrowthStage, HexacoAxis, NeedType,
        RelationType, ResourceType, SettlementId, ValueType, WorldMap,
    };
    use sim_engine::{SimResources, SimSystem};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(2, 1, 123);
        map.get_mut(0, 0).resources.push(TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 5.0,
            regen_rate: 0.75,
        });
        map.get_mut(1, 0).resources.push(TileResource {
            resource_type: ResourceType::Wood,
            amount: 4.8,
            max_amount: 5.0,
            regen_rate: 1.0,
        });
        SimResources::new(calendar, map, 999)
    }

    #[test]
    fn resource_regen_system_applies_regen_and_caps_at_max() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut system = ResourceRegenSystem::new(5, 10);
        system.run(&mut world, &mut resources, 10);

        let first = &resources.map.get(0, 0).resources[0];
        assert!((first.amount - 3.75).abs() < 1e-6);

        let second = &resources.map.get(1, 0).resources[0];
        assert!((second.amount - 5.0).abs() < 1e-6);
    }

    #[test]
    fn needs_runtime_system_applies_decay_and_action_energy_cost() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).temperature = 0.2;

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.6);
        needs.set(NeedType::Belonging, 0.7);
        needs.set(NeedType::Thirst, 0.8);
        needs.set(NeedType::Warmth, 0.9);
        needs.set(NeedType::Safety, 0.85);
        needs.energy = 0.75;
        needs.set(NeedType::Sleep, 0.75);

        let behavior = Behavior {
            current_action: ActionType::GatherWood,
            ..Behavior::default()
        };
        let body_component = BodyComponent {
            end_realized: 6000,
            rec_realized: 4000,
            ..BodyComponent::default()
        };
        let position = Position::new(0, 0);
        let entity = world.spawn((needs, behavior, body_component, position));

        let mut system = NeedsRuntimeSystem::new(10, 4);
        system.run(&mut world, &mut resources, 4);

        let decays = body::needs_base_decay_step(
            0.6,
            sim_core::config::HUNGER_DECAY_RATE as f32,
            1.0,
            sim_core::config::HUNGER_METABOLIC_MIN as f32,
            sim_core::config::HUNGER_METABOLIC_RANGE as f32,
            sim_core::config::ENERGY_DECAY_RATE as f32,
            sim_core::config::SOCIAL_DECAY_RATE as f32,
            sim_core::config::SAFETY_DECAY_RATE as f32,
            sim_core::config::THIRST_DECAY_RATE as f32,
            sim_core::config::WARMTH_DECAY_RATE as f32,
            0.2,
            true,
            sim_core::config::WARMTH_TEMP_NEUTRAL as f32,
            sim_core::config::WARMTH_TEMP_FREEZING as f32,
            sim_core::config::WARMTH_TEMP_COLD as f32,
            true,
        );
        let action_cost = body::action_energy_cost(
            sim_core::config::ENERGY_ACTION_COST as f32,
            (6000.0 / sim_core::config::BODY_REALIZED_MAX as f32).clamp(0.0, 1.0),
            sim_core::config::BODY_END_COST_REDUCTION as f32,
        );
        let expected_energy = (0.75 - decays[1] - action_cost).clamp(0.0, 1.0);

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!((updated.get(NeedType::Hunger) as f32 - (0.6 - decays[0]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Belonging) as f32 - (0.7 - decays[2]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Thirst) as f32 - (0.8 - decays[3]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Warmth) as f32 - (0.9 - decays[4]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.get(NeedType::Safety) as f32 - (0.85 - decays[5]).clamp(0.0, 1.0)).abs() < 1e-6);
        assert!((updated.energy as f32 - expected_energy).abs() < 1e-6);
        assert!((updated.get(NeedType::Sleep) as f32 - expected_energy).abs() < 1e-6);
    }

    #[test]
    fn stress_runtime_system_updates_stress_state_and_components() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.05);
        needs.set(NeedType::Thirst, 0.10);
        needs.set(NeedType::Warmth, 0.08);
        needs.set(NeedType::Safety, 0.10);
        needs.set(NeedType::Belonging, 0.20);
        needs.energy = 0.15;

        let mut emotion = Emotion::default();
        emotion.add(EmotionType::Fear, 0.7);
        emotion.add(EmotionType::Anger, 0.6);
        emotion.add(EmotionType::Sadness, 0.5);

        let stress = Stress::default();
        let entity = world.spawn((needs, stress, emotion));

        let mut system = StressRuntimeSystem::new(34, 4);
        system.run(&mut world, &mut resources, 4);

        let updated = world
            .get::<&Stress>(entity)
            .expect("updated stress component should be queryable");
        assert!(updated.level > 0.0);
        assert!(updated.reserve <= 1.0);
        assert!(updated.allostatic_load >= 0.0);
    }

    #[test]
    fn emotion_runtime_system_updates_primary_and_baseline_values() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.1;
        *emotion.get_mut(EmotionType::Joy) = 0.2;

        let mut needs = Needs::default();
        needs.set(NeedType::Safety, 0.2);
        needs.set(NeedType::Belonging, 0.3);
        needs.set(NeedType::Hunger, 0.25);
        needs.energy = 0.3;

        let stress = Stress {
            level: 0.8,
            reserve: 0.5,
            allostatic_load: 0.2,
            ..Stress::default()
        };

        let personality = Personality::default();
        let entity = world.spawn((emotion, needs, stress, personality));
        let mut system = EmotionRuntimeSystem::new(32, 12);
        system.run(&mut world, &mut resources, 12);

        let updated = world
            .get::<&Emotion>(entity)
            .expect("updated emotion component should be queryable");
        assert!(updated.get(EmotionType::Fear) > 0.1);
        assert!(updated.get(EmotionType::Joy) < 0.2);
        assert!(updated.baseline(EmotionType::Trust) >= 0.0);
        assert!(updated.baseline(EmotionType::Trust) <= 1.0);
    }

    #[test]
    fn reputation_runtime_system_updates_reputation_state_and_tags() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut social = Social::default();
        social.reputation_local = 0.75;
        social.reputation_regional = 0.70;

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.90;
        *emotion.get_mut(EmotionType::Anger) = 0.80;
        *emotion.get_mut(EmotionType::Sadness) = 0.70;
        *emotion.get_mut(EmotionType::Disgust) = 0.60;
        *emotion.get_mut(EmotionType::Surprise) = 0.60;
        *emotion.get_mut(EmotionType::Joy) = 0.10;
        *emotion.get_mut(EmotionType::Trust) = 0.10;
        *emotion.get_mut(EmotionType::Anticipation) = 0.10;

        let stress = Stress {
            level: 0.90,
            reserve: 0.20,
            allostatic_load: 0.60,
            ..Stress::default()
        };

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.20);
        needs.set(NeedType::Safety, 0.10);
        needs.set(NeedType::Belonging, 0.20);

        let behavior = Behavior {
            current_action: ActionType::Idle,
            ..Behavior::default()
        };
        let mut values = Values::default();
        values.set(ValueType::Power, 0.60);

        let entity = world.spawn((social, emotion, stress, needs, behavior, values));
        let mut system = ReputationRuntimeSystem::new(38, 30);
        system.run(&mut world, &mut resources, 30);

        let updated = world
            .get::<&Social>(entity)
            .expect("updated social component should be queryable");
        assert!(updated.reputation_local < 0.75);
        assert!(updated.reputation_regional < 0.70);
        assert!((0.0..=1.0).contains(&updated.reputation_local));
        assert!((0.0..=1.0).contains(&updated.reputation_regional));
        assert!(
            updated.reputation_tags.iter().any(|tag| tag == "suspect" || tag == "outcast"),
            "negative reputation tier tag should be assigned"
        );
    }

    #[test]
    fn social_event_runtime_system_updates_edges_and_social_capital() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut social = Social::default();
        social.reputation_local = 0.55;
        social.reputation_regional = 0.50;
        let mut edge = sim_core::components::RelationshipEdge::new(EntityId(2));
        edge.affinity = 35.0;
        edge.trust = 0.35;
        edge.familiarity = 0.20;
        edge.relation_type = RelationType::Friend;
        edge.is_bridge = true;
        social.edges.push(edge);

        let personality = Personality::default();
        let behavior = Behavior {
            current_action: ActionType::Socialize,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.80);
        let stress = Stress {
            level: 0.20,
            ..Stress::default()
        };

        let entity = world.spawn((social, personality, behavior, needs, stress));
        let mut system = SocialEventRuntimeSystem::new(37, 30);
        system.run(&mut world, &mut resources, 90);

        let updated = world
            .get::<&Social>(entity)
            .expect("updated social component should be queryable");
        assert_eq!(updated.edges.len(), 1);
        assert!(updated.edges[0].affinity > 35.0);
        assert!(updated.edges[0].trust >= 0.35);
        assert!(updated.edges[0].familiarity > 0.20);
        assert_eq!(updated.edges[0].last_interaction_tick, 90);
        assert!((0.0..=1.0).contains(&updated.social_capital));
    }

    #[test]
    fn morale_runtime_system_updates_behavior_and_upper_needs() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.30);
        needs.set(NeedType::Safety, 0.25);
        needs.set(NeedType::Belonging, 0.35);
        needs.set(NeedType::Autonomy, 0.40);
        needs.set(NeedType::Meaning, 0.55);
        needs.set(NeedType::Transcendence, 0.50);
        needs.energy = 0.30;

        let behavior = Behavior {
            job_satisfaction: 0.65,
            occupation_satisfaction: 0.62,
            ..Behavior::default()
        };

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.70;
        *emotion.get_mut(EmotionType::Anger) = 0.60;
        *emotion.get_mut(EmotionType::Sadness) = 0.65;
        *emotion.get_mut(EmotionType::Joy) = 0.20;
        *emotion.get_mut(EmotionType::Trust) = 0.25;
        *emotion.get_mut(EmotionType::Anticipation) = 0.20;

        let stress = Stress {
            level: 0.75,
            ..Stress::default()
        };
        let personality = Personality::default();
        let social = Social::default();

        let entity = world.spawn((needs, behavior, emotion, stress, personality, social));
        let mut system = MoraleRuntimeSystem::new(40, 5);
        system.run(&mut world, &mut resources, 5);

        let updated_behavior = world
            .get::<&Behavior>(entity)
            .expect("updated behavior should be queryable");
        let updated_needs = world
            .get::<&Needs>(entity)
            .expect("updated needs should be queryable");
        assert!(updated_behavior.job_satisfaction < 0.65);
        assert!(updated_behavior.occupation_satisfaction < 0.62);
        assert!((0.0..=1.0).contains(&updated_needs.get(NeedType::Meaning)));
        assert!((0.0..=1.0).contains(&updated_needs.get(NeedType::Transcendence)));
        assert!(
            (updated_needs.get(NeedType::Meaning) - 0.55).abs() > f64::EPSILON
                || (updated_needs.get(NeedType::Transcendence) - 0.50).abs() > f64::EPSILON
        );
    }

    #[test]
    fn value_runtime_system_updates_value_axes_with_context() {
        let mut world = World::new();
        let mut resources = make_resources();

        let values = Values::default();
        let age = Age {
            years: 12.0,
            ..Age::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::H as usize] = 0.80;
        personality.axes[HexacoAxis::A as usize] = 0.75;
        personality.axes[HexacoAxis::X as usize] = 0.65;

        let mut needs = Needs::default();
        needs.set(NeedType::Safety, 0.85);
        needs.set(NeedType::Belonging, 0.90);
        let stress = Stress {
            level: 0.10,
            ..Stress::default()
        };
        let social = Social {
            social_capital: 0.70,
            ..Social::default()
        };

        let entity = world.spawn((values, age, personality, needs, stress, social));
        let mut system = ValueRuntimeSystem::new(55, 200);
        system.run(&mut world, &mut resources, 200);

        let updated = world
            .get::<&Values>(entity)
            .expect("updated values component should be queryable");
        assert!(updated.get(ValueType::Cooperation) > 0.0);
        assert!(updated.get(ValueType::Fairness) > 0.0);
        assert!(updated.get(ValueType::Family) > 0.0);
        assert!(updated.get(ValueType::Law) > 0.0);
    }

    #[test]
    fn job_satisfaction_runtime_system_updates_behavior_scores() {
        let mut world = World::new();
        let mut resources = make_resources();

        let behavior = Behavior {
            job: "builder".to_string(),
            job_satisfaction: 0.30,
            occupation_satisfaction: 0.35,
            ..Behavior::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::A as usize] = 0.75;
        personality.axes[HexacoAxis::C as usize] = 0.80;
        let mut values = Values::default();
        values.set(ValueType::HardWork, 0.70);
        values.set(ValueType::Cooperation, 0.60);
        values.set(ValueType::Fairness, 0.60);

        let mut needs = Needs::default();
        needs.set(NeedType::Autonomy, 0.65);
        needs.set(NeedType::Competence, 0.70);
        needs.set(NeedType::Meaning, 0.60);

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_CONSTRUCTION".to_string(),
            SkillEntry { level: 8, xp: 0.0 },
        );
        let age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };

        let entity = world.spawn((behavior, personality, values, needs, skills, age));
        let mut system = JobSatisfactionRuntimeSystem::new(40, 120);
        system.run(&mut world, &mut resources, 120);

        let updated = world
            .get::<&Behavior>(entity)
            .expect("updated behavior should be queryable");
        assert!(updated.job_satisfaction > 0.30);
        assert!(updated.occupation_satisfaction > 0.35);
        assert!((0.0..=1.0).contains(&updated.job_satisfaction));
    }

    #[test]
    fn network_runtime_system_recomputes_social_capital_from_edges() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut social = Social::default();
        social.reputation_local = 0.80;
        social.reputation_regional = 0.60;

        let mut strong_edge = sim_core::components::RelationshipEdge::new(EntityId(2));
        strong_edge.affinity = 72.0;
        strong_edge.relation_type = RelationType::CloseFriend;
        strong_edge.is_bridge = false;
        social.edges.push(strong_edge);

        let mut bridge_weak_edge = sim_core::components::RelationshipEdge::new(EntityId(3));
        bridge_weak_edge.affinity = 18.0;
        bridge_weak_edge.relation_type = RelationType::Friend;
        bridge_weak_edge.is_bridge = true;
        social.edges.push(bridge_weak_edge);

        let mut ignored_edge = sim_core::components::RelationshipEdge::new(EntityId(4));
        ignored_edge.affinity = 1.0;
        ignored_edge.relation_type = RelationType::Stranger;
        ignored_edge.is_bridge = false;
        social.edges.push(ignored_edge);

        let entity = world.spawn((social,));
        let mut system = NetworkRuntimeSystem::new(58, sim_core::config::REVOLUTION_TICK_INTERVAL);
        system.run(&mut world, &mut resources, sim_core::config::REVOLUTION_TICK_INTERVAL);

        let updated = world
            .get::<&Social>(entity)
            .expect("updated social component should be queryable");
        let expected = body::network_social_capital_norm(
            1.0,
            0.0,
            0.5,
            0.70,
            sim_core::config::NETWORK_SOCIAL_CAP_STRONG_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_WEAK_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_BRIDGE_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_REP_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_NORM_DIV as f32,
        );
        assert!((updated.social_capital as f32 - expected).abs() < 1e-6);
        assert!((0.0..=1.0).contains(&updated.social_capital));
    }

    #[test]
    fn occupation_runtime_system_assigns_and_switches_by_skill_margin() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut adult_skills = Skills::default();
        adult_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 20, xp: 0.0 },
        );
        adult_skills.entries.insert(
            "SKILL_MINING".to_string(),
            SkillEntry { level: 45, xp: 0.0 },
        );
        let adult_age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let adult_behavior = Behavior {
            occupation: "foraging".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let adult_entity = world.spawn((adult_age, adult_skills, adult_behavior));

        let mut teen_skills = Skills::default();
        teen_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 4, xp: 0.0 },
        );
        let teen_age = Age {
            stage: GrowthStage::Teen,
            ..Age::default()
        };
        let teen_behavior = Behavior {
            occupation: "laborer".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let teen_entity = world.spawn((teen_age, teen_skills, teen_behavior));

        let mut low_adult_skills = Skills::default();
        low_adult_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 3, xp: 0.0 },
        );
        let low_adult_age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let low_adult_behavior = Behavior {
            occupation: "foraging".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let low_adult_entity = world.spawn((low_adult_age, low_adult_skills, low_adult_behavior));

        let mut system = OccupationRuntimeSystem::new(36, sim_core::config::OCCUPATION_EVAL_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::OCCUPATION_EVAL_INTERVAL,
        );

        let adult_updated = world
            .get::<&Behavior>(adult_entity)
            .expect("adult behavior should be queryable");
        assert_eq!(adult_updated.occupation, "mining");
        assert_eq!(adult_updated.job, "miner");

        let teen_updated = world
            .get::<&Behavior>(teen_entity)
            .expect("teen behavior should be queryable");
        assert_eq!(teen_updated.occupation, "none");
        assert_eq!(teen_updated.job, "gatherer");

        let low_adult_updated = world
            .get::<&Behavior>(low_adult_entity)
            .expect("low-skill adult behavior should be queryable");
        assert_eq!(low_adult_updated.occupation, "laborer");
        assert_eq!(low_adult_updated.job, "gatherer");
    }

    #[test]
    fn contagion_runtime_system_propagates_emotion_and_stress() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut recipient_emotion = Emotion::default();
        *recipient_emotion.get_mut(EmotionType::Joy) = 0.10;
        *recipient_emotion.get_mut(EmotionType::Trust) = 0.10;
        *recipient_emotion.get_mut(EmotionType::Sadness) = 0.60;
        *recipient_emotion.get_mut(EmotionType::Fear) = 0.40;
        let recipient_stress = Stress {
            level: 0.20,
            allostatic_load: 0.05,
            ..Stress::default()
        };
        let mut recipient_personality = Personality::default();
        recipient_personality.axes[HexacoAxis::X as usize] = 0.80;
        recipient_personality.axes[HexacoAxis::E as usize] = 0.70;
        recipient_personality.axes[HexacoAxis::A as usize] = 0.75;
        let recipient_identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        let recipient = world.spawn((
            Position::new(0, 0),
            recipient_emotion,
            recipient_stress,
            recipient_personality,
            recipient_identity,
        ));

        let mut donor1_emotion = Emotion::default();
        *donor1_emotion.get_mut(EmotionType::Joy) = 0.90;
        *donor1_emotion.get_mut(EmotionType::Trust) = 0.80;
        *donor1_emotion.get_mut(EmotionType::Sadness) = 0.10;
        *donor1_emotion.get_mut(EmotionType::Fear) = 0.10;
        let donor1_stress = Stress {
            level: 0.80,
            allostatic_load: 0.30,
            ..Stress::default()
        };
        let donor1_identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        world.spawn((
            Position::new(1, 0),
            donor1_emotion,
            donor1_stress,
            Personality::default(),
            donor1_identity,
        ));

        let mut donor2_emotion = Emotion::default();
        *donor2_emotion.get_mut(EmotionType::Joy) = 0.85;
        *donor2_emotion.get_mut(EmotionType::Trust) = 0.75;
        *donor2_emotion.get_mut(EmotionType::Sadness) = 0.05;
        *donor2_emotion.get_mut(EmotionType::Fear) = 0.05;
        let donor2_stress = Stress {
            level: 0.70,
            allostatic_load: 0.20,
            ..Stress::default()
        };
        let donor2_identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        world.spawn((
            Position::new(2, 1),
            donor2_emotion,
            donor2_stress,
            Personality::default(),
            donor2_identity,
        ));

        let mut system = ContagionRuntimeSystem::new(38, 3);
        system.run(&mut world, &mut resources, 3);

        let mut query_one = world
            .query_one::<(&Emotion, &Stress)>(recipient)
            .expect("recipient components should be queryable");
        let (updated_emotion, updated_stress) = query_one
            .get()
            .expect("recipient emotion/stress tuple should be readable");
        let joy = updated_emotion.get(EmotionType::Joy);
        let trust = updated_emotion.get(EmotionType::Trust);
        let sadness = updated_emotion.get(EmotionType::Sadness);
        let stress_level = updated_stress.level;
        let allostatic = updated_stress.allostatic_load;
        drop(query_one);

        assert!(joy > 0.10);
        assert!(trust > 0.10);
        assert!(sadness < 0.60);
        assert!(stress_level > 0.20);
        assert!(allostatic >= 0.05);
    }

    #[test]
    fn age_runtime_system_updates_stage_identity_and_elder_builder_job() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            ticks: sim_core::config::AGE_ADULT_END - 10,
            years: 0.0,
            stage: GrowthStage::Adult,
            alive: true,
        };
        let identity = Identity {
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let behavior = Behavior {
            job: "builder".to_string(),
            ..Behavior::default()
        };
        let entity = world.spawn((age, identity, behavior));

        let mut system = AgeRuntimeSystem::new(48, 20);
        system.run(&mut world, &mut resources, 20);

        let updated_age = world.get::<&Age>(entity).expect("age should be queryable");
        assert_eq!(updated_age.stage, GrowthStage::Elder);
        assert!(updated_age.years > 0.0);
        drop(updated_age);

        let updated_identity = world
            .get::<&Identity>(entity)
            .expect("identity should be queryable");
        assert_eq!(updated_identity.growth_stage, GrowthStage::Elder);
        drop(updated_identity);

        let updated_behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(updated_behavior.job, "none");
    }

    #[test]
    fn upper_needs_runtime_system_updates_upper_need_buckets() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Competence, 0.45);
        needs.set(NeedType::Autonomy, 0.52);
        needs.set(NeedType::SelfActualization, 0.33);
        needs.set(NeedType::Meaning, 0.41);
        needs.set(NeedType::Transcendence, 0.29);
        needs.set(NeedType::Recognition, 0.35);
        needs.set(NeedType::Belonging, 0.38);
        needs.set(NeedType::Intimacy, 0.32);

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 78, xp: 0.0 },
        );
        skills.entries.insert(
            "SKILL_MINING".to_string(),
            SkillEntry { level: 40, xp: 0.0 },
        );

        let mut values = Values::default();
        values.set(ValueType::Nature, 0.7);
        values.set(ValueType::Independence, 0.4);
        values.set(ValueType::HardWork, 0.5);
        values.set(ValueType::Sacrifice, 0.2);

        let behavior = Behavior {
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let identity = Identity {
            settlement_id: Some(SettlementId(1)),
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let social = Social {
            spouse: Some(EntityId(2)),
            ..Social::default()
        };

        let entity = world.spawn((needs, skills, values, behavior, identity, social));
        let mut system = UpperNeedsRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        let best_skill_norm = body::upper_needs_best_skill_normalized(&[78, 0, 40, 0, 0], 100);
        let alignment = body::upper_needs_job_alignment(2, 0.0, 0.0, 0.5, 0.7, 0.4);
        let expected = body::upper_needs_step(
            &[0.45, 0.52, 0.33, 0.41, 0.29, 0.35, 0.38, 0.32],
            &[
                sim_core::config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                sim_core::config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_MEANING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_BELONGING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ],
            sim_core::config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
            sim_core::config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
            sim_core::config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
            best_skill_norm,
            alignment,
            0.2,
            true,
            true,
            true,
        );

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!((updated.get(NeedType::Competence) as f32 - expected[0]).abs() < 1e-6);
        assert!((updated.get(NeedType::Recognition) as f32 - expected[5]).abs() < 1e-6);
        assert!((updated.get(NeedType::Belonging) as f32 - expected[6]).abs() < 1e-6);
        assert!((updated.get(NeedType::Intimacy) as f32 - expected[7]).abs() < 1e-6);
    }
}
