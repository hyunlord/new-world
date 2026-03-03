#![allow(unused_imports)]

use hecs::{Entity, World};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence, Memory,
    MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, EmotionType, GrowthStage, HexacoAxis, HexacoFacet,
    BuildingId, CopingStrategyId, EntityId, IntelligenceType, MentalBreakType, NeedType, RelationType, ResourceType,
    SettlementId, Sex, SocialClass, TechState, ValueType,
};
use sim_engine::{SimResources, SimSystem};

use crate::body;


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

/// Rust runtime system for per-entity economic tendency updates.
///
/// This performs active writes on `Economic` tendencies using personality,
/// values, belonging, wealth, and sex context through
/// `body::economic_tendencies_step`.
#[derive(Debug, Clone)]
pub struct EconomicTendencyRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl EconomicTendencyRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn value_or_default(values_opt: Option<&Values>, key: ValueType) -> f32 {
    values_opt.map(|values| values.get(key) as f32).unwrap_or(0.0)
}

impl SimSystem for EconomicTendencyRuntimeSystem {
    fn name(&self) -> &'static str {
        "economic_tendency_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Economic,
            Option<&Personality>,
            Option<&Values>,
            Option<&Needs>,
            Option<&Age>,
            Option<&Identity>,
        )>();
        for (_, (economic, personality_opt, values_opt, needs_opt, age_opt, identity_opt)) in
            &mut query
        {
            let Some(personality) = personality_opt else {
                continue;
            };
            if let Some(age) = age_opt {
                if matches!(age.stage, GrowthStage::Infant | GrowthStage::Child) {
                    continue;
                }
            }

            let h = personality.axis(HexacoAxis::H) as f32;
            let e = personality.axis(HexacoAxis::E) as f32;
            let x = personality.axis(HexacoAxis::X) as f32;
            let a = personality.axis(HexacoAxis::A) as f32;
            let c = personality.axis(HexacoAxis::C) as f32;
            let o = personality.axis(HexacoAxis::O) as f32;
            let age_years = age_opt.map(|age| age.years as f32).unwrap_or(0.0).max(0.0);
            let belonging = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let wealth_norm = (economic.wealth as f32).clamp(0.0, 1.0);
            let is_male = identity_opt
                .map(|identity| matches!(identity.sex, Sex::Male))
                .unwrap_or(false);

            let out = body::economic_tendencies_step(
                h,
                e,
                x,
                a,
                c,
                o,
                age_years,
                value_or_default(values_opt, ValueType::SelfControl),
                value_or_default(values_opt, ValueType::Law),
                value_or_default(values_opt, ValueType::Commerce),
                value_or_default(values_opt, ValueType::Competition),
                value_or_default(values_opt, ValueType::MartialProwess),
                value_or_default(values_opt, ValueType::Sacrifice),
                value_or_default(values_opt, ValueType::Cooperation),
                value_or_default(values_opt, ValueType::Family),
                value_or_default(values_opt, ValueType::Power),
                value_or_default(values_opt, ValueType::Fairness),
                belonging,
                wealth_norm,
                0.0,
                0.0,
                is_male,
                config::ECON_WEALTH_GENEROSITY_PENALTY as f32,
            );
            economic.saving_tendency = out[0] as f64;
            economic.risk_appetite = out[1] as f64;
            economic.generosity = out[2] as f64;
            economic.materialism = out[3] as f64;
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

/// Rust runtime system for social title grant/revoke evaluation.
///
/// This performs active writes on `Social.titles` using age, skill tiers,
/// and settlement leadership ownership.
#[derive(Debug, Clone)]
pub struct TitleRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TitleRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn skill_id_to_title_suffix(skill_id: &str) -> String {
    skill_id
        .strip_prefix("SKILL_")
        .unwrap_or(skill_id)
        .to_ascii_uppercase()
}

impl SimSystem for TitleRuntimeSystem {
    fn name(&self) -> &'static str {
        "title_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let leader_ids: HashSet<EntityId> = resources
            .settlements
            .values()
            .filter_map(|settlement| settlement.leader_id)
            .collect();
        let elder_min_age_years = config::TITLE_ELDER_MIN_AGE_YEARS as f32;
        let expert_level = config::TITLE_EXPERT_SKILL_LEVEL as i32;
        let master_level = config::TITLE_MASTER_SKILL_LEVEL as i32;

        let mut query = world.query::<(&Age, Option<&Skills>, &mut Social)>();
        for (entity, (age, skills_opt, social)) in &mut query {
            if !age.alive {
                continue;
            }

            let is_elder = body::title_is_elder(age.years as f32, elder_min_age_years);
            if is_elder {
                social.grant_title("TITLE_ELDER");
            } else {
                social.revoke_title("TITLE_ELDER");
            }

            if let Some(skills) = skills_opt {
                for (skill_id, entry) in &skills.entries {
                    let tier =
                        body::title_skill_tier(i32::from(entry.level), expert_level, master_level);
                    let title_suffix = skill_id_to_title_suffix(skill_id.as_str());
                    if title_suffix.is_empty() {
                        continue;
                    }
                    let master_title = format!("TITLE_MASTER_{title_suffix}");
                    let expert_title = format!("TITLE_EXPERT_{title_suffix}");

                    if tier >= 2 {
                        social.grant_title(master_title.as_str());
                        social.revoke_title(expert_title.as_str());
                    } else if tier >= 1 {
                        social.grant_title(expert_title.as_str());
                    } else {
                        social.revoke_title(master_title.as_str());
                        social.revoke_title(expert_title.as_str());
                    }
                }
            }

            let entity_id = EntityId(entity.id() as u64);
            if leader_ids.contains(&entity_id) {
                social.grant_title("TITLE_CHIEF");
            } else if social.has_title("TITLE_CHIEF") {
                social.revoke_title("TITLE_CHIEF");
                social.grant_title("TITLE_FORMER_CHIEF");
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StratificationMemberSnapshot {
    entity: Entity,
    wealth: f32,
    rep_overall: f32,
    rep_competence: f32,
    age_years: f32,
}

#[inline]
fn stratification_social_class(status_score: f32, is_leader: bool) -> SocialClass {
    if is_leader {
        return SocialClass::Ruler;
    }
    if status_score > config::STATUS_TIER_ELITE as f32 {
        SocialClass::Noble
    } else if status_score > config::STATUS_TIER_RESPECTED as f32 {
        SocialClass::Merchant
    } else if status_score > config::STATUS_TIER_MARGINAL as f32 {
        SocialClass::Commoner
    } else if status_score > config::STATUS_TIER_OUTCAST as f32 {
        SocialClass::Artisan
    } else {
        SocialClass::Outcast
    }
}

/// Rust runtime system for settlement stratification monitoring.
///
/// This performs active writes on `Settlement.gini_coefficient`,
/// `Settlement.leveling_effectiveness`, `Settlement.stratification_phase`,
/// `Social.social_class`, and `Economic.wealth_norm`.
#[derive(Debug, Clone)]
pub struct StratificationMonitorRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StratificationMonitorRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StratificationMonitorRuntimeSystem {
    fn name(&self) -> &'static str {
        "stratification_monitor"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut members_by_settlement: HashMap<SettlementId, Vec<StratificationMemberSnapshot>> =
            HashMap::new();

        let mut query = world.query::<(&Age, &Identity, &Social, Option<&Economic>)>();
        for (entity, (age, identity, social, economic_opt)) in &mut query {
            if !age.alive {
                continue;
            }
            let Some(settlement_id) = identity.settlement_id else {
                continue;
            };
            let wealth = economic_opt
                .map(|economic| economic.wealth as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let rep_overall =
                ((social.reputation_local as f32 + social.reputation_regional as f32) * 0.5)
                    .clamp(0.0, 1.0);
            let rep_competence = social.social_capital as f32;
            members_by_settlement
                .entry(settlement_id)
                .or_default()
                .push(StratificationMemberSnapshot {
                    entity,
                    wealth,
                    rep_overall,
                    rep_competence,
                    age_years: age.years as f32,
                });
        }
        drop(query);

        for (settlement_id, settlement) in resources.settlements.iter_mut() {
            let Some(members) = members_by_settlement.get(settlement_id) else {
                continue;
            };
            if members.is_empty() {
                continue;
            }

            let mut wealth_values: Vec<f32> = members.iter().map(|member| member.wealth).collect();
            let gini = body::stratification_gini(&wealth_values).clamp(0.0, 1.0);
            settlement.gini_coefficient = gini as f64;

            wealth_values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
            let p90_idx = ((wealth_values.len() as f32) * 0.90).floor() as usize;
            let p90 = wealth_values
                .get(p90_idx.min(wealth_values.len().saturating_sub(1)))
                .copied()
                .unwrap_or(0.0)
                .max(0.0001);

            let pop = members.len() as f32;
            let dunbar_factor = (config::LEVELING_DUNBAR_N as f32 / pop.max(1.0)).min(1.0);
            let mobility_factor = (1.0 - config::LEVELING_SEDENTISM_DEFAULT as f32).clamp(0.0, 1.0);
            let need_30days = pop
                * config::HUNGER_DECAY_RATE as f32
                * config::TICKS_PER_DAY as f32
                * 30.0;
            let surplus_ratio = (settlement.stockpile_food as f32) / need_30days.max(1.0);
            let leveling = (dunbar_factor * mobility_factor * (1.0 / (1.0 + surplus_ratio)))
                .clamp(0.0, 1.0);
            settlement.leveling_effectiveness = leveling as f64;
            settlement.stratification_phase = if gini < config::GINI_UNREST_THRESHOLD as f32 && leveling > 0.5 {
                "egalitarian".to_string()
            } else if gini < config::GINI_ENTRENCHED_THRESHOLD as f32 && leveling > 0.2 {
                "transitional".to_string()
            } else {
                "stratified".to_string()
            };

            for member in members {
                let entity_id = EntityId(member.entity.id() as u64);
                let is_leader = settlement
                    .leader_id
                    .map(|leader_id| leader_id == entity_id)
                    .unwrap_or(false);
                let leader_bonus = if is_leader {
                    config::STATUS_LEADER_CURRENT as f32
                } else {
                    0.0
                };
                let wealth_norm = (member.wealth / p90).clamp(0.0, 1.0);
                let status_score = body::stratification_status_score(
                    member.rep_overall,
                    wealth_norm,
                    leader_bonus,
                    member.age_years,
                    member.rep_competence,
                    config::STATUS_W_REPUTATION as f32,
                    config::STATUS_W_WEALTH as f32,
                    config::STATUS_W_LEADER as f32,
                    config::STATUS_W_AGE as f32,
                    config::STATUS_W_COMPETENCE as f32,
                );
                let class = stratification_social_class(status_score, is_leader);
                if let Ok(mut one) = world.query_one::<(&mut Social, Option<&mut Economic>)>(member.entity) {
                    if let Some((social, economic_opt)) = one.get() {
                        social.social_class = class;
                        if let Some(economic) = economic_opt {
                            economic.wealth_norm = wealth_norm as f64;
                        }
                    }
                }
            }
        }
    }
}

const FAMILY_PAIR_BASE_PROB: f32 = 0.45;

/// Rust runtime system for partner coupling and family formation.
///
/// This performs active writes on `Social.spouse` for single adult/elder
/// entities within the same settlement and emits `FamilyFormed`.
#[derive(Debug, Clone)]
pub struct FamilyRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl FamilyRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for FamilyRuntimeSystem {
    fn name(&self) -> &'static str {
        "family_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut candidates: HashMap<SettlementId, (Vec<Entity>, Vec<Entity>)> = HashMap::new();
        {
            let mut query = world.query::<(&Age, &Identity, &Social)>();
            for (entity, (age, identity, social)) in &mut query {
                if !age.alive {
                    continue;
                }
                if !matches!(age.stage, GrowthStage::Adult | GrowthStage::Elder) {
                    continue;
                }
                if social.spouse.is_some() {
                    continue;
                }
                let Some(settlement_id) = identity.settlement_id else {
                    continue;
                };
                let entry = candidates
                    .entry(settlement_id)
                    .or_insert_with(|| (Vec::new(), Vec::new()));
                match identity.sex {
                    Sex::Male => entry.0.push(entity),
                    Sex::Female => entry.1.push(entity),
                }
            }
        }

        let mut settlement_ids: Vec<SettlementId> = candidates.keys().copied().collect();
        settlement_ids.sort_by_key(|settlement_id| settlement_id.0);
        for settlement_id in settlement_ids {
            let Some((males, females)) = candidates.get_mut(&settlement_id) else {
                continue;
            };
            males.sort_by_key(|entity| entity.id());
            females.sort_by_key(|entity| entity.id());
            let pair_count = males.len().min(females.len());
            for pair_idx in 0..pair_count {
                if resources.rng.gen_range(0.0..1.0) > FAMILY_PAIR_BASE_PROB {
                    continue;
                }
                let male_entity = males[pair_idx];
                let female_entity = females[pair_idx];

                let male_id = EntityId(male_entity.id() as u64);
                let female_id = EntityId(female_entity.id() as u64);

                let male_available = world
                    .query_one::<&Social>(male_entity)
                    .ok()
                    .and_then(|mut query| query.get().map(|social| social.spouse.is_none()))
                    .unwrap_or(false);
                let female_available = world
                    .query_one::<&Social>(female_entity)
                    .ok()
                    .and_then(|mut query| query.get().map(|social| social.spouse.is_none()))
                    .unwrap_or(false);
                if !male_available || !female_available {
                    continue;
                }

                if let Ok(mut query) = world.query_one::<&mut Social>(male_entity) {
                    if let Some(social) = query.get() {
                        social.spouse = Some(female_id);
                    }
                }
                if let Ok(mut query) = world.query_one::<&mut Social>(female_entity) {
                    if let Some(social) = query.get() {
                        social.spouse = Some(male_id);
                    }
                }

                resources.event_bus.emit(sim_engine::GameEvent::FamilyFormed {
                    entity_a: male_id,
                    entity_b: female_id,
                });
            }
        }
    }
}

/// Rust runtime system for settlement leadership election and countdown.
///
/// This performs active writes on `Settlement.leader_id` and
/// `Settlement.leader_reelection_countdown` using adult/elder candidate scoring.
#[derive(Debug, Clone)]
pub struct LeaderRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl LeaderRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LeaderCandidateSnapshot {
    entity: Entity,
    charisma: f32,
    wisdom: f32,
    trustworthiness: f32,
    intimidation: f32,
    social_capital: f32,
    age_respect: f32,
    rep_overall: f32,
}

impl SimSystem for LeaderRuntimeSystem {
    fn name(&self) -> &'static str {
        "leader_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut candidates_by_settlement: HashMap<SettlementId, Vec<LeaderCandidateSnapshot>> = HashMap::new();
        let mut alive_settlement_by_entity: HashMap<EntityId, SettlementId> = HashMap::new();

        let mut query = world.query::<(
            &Age,
            &Identity,
            Option<&Personality>,
            Option<&Social>,
            Option<&Intelligence>,
            Option<&BodyComponent>,
        )>();
        for (entity, (age, identity, personality_opt, social_opt, intelligence_opt, body_opt)) in &mut query {
            if !age.alive {
                continue;
            }
            let Some(settlement_id) = identity.settlement_id else {
                continue;
            };

            let entity_id = EntityId(entity.id() as u64);
            alive_settlement_by_entity.insert(entity_id, settlement_id);

            if !matches!(age.stage, GrowthStage::Adult | GrowthStage::Elder) {
                continue;
            }

            let charisma = personality_opt
                .map(|personality| personality.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let wisdom = intelligence_opt
                .map(|intelligence| intelligence.g_factor as f32)
                .or_else(|| personality_opt.map(|personality| personality.axis(HexacoAxis::O) as f32))
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let trustworthiness = social_opt
                .map(|social| social.reputation_local as f32)
                .or_else(|| personality_opt.map(|personality| personality.axis(HexacoAxis::H) as f32))
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let intimidation = body_opt
                .map(|body| body.strength_norm())
                .or_else(|| personality_opt.map(|personality| (1.0 - personality.axis(HexacoAxis::A) as f32).clamp(0.0, 1.0)))
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let social_capital = social_opt
                .map(|social| social.social_capital as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let age_respect = body::leader_age_respect(age.years as f32);
            let rep_overall = social_opt
                .map(|social| ((social.reputation_local + social.reputation_regional) * 0.5) as f32)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);

            candidates_by_settlement
                .entry(settlement_id)
                .or_default()
                .push(LeaderCandidateSnapshot {
                    entity,
                    charisma,
                    wisdom,
                    trustworthiness,
                    intimidation,
                    social_capital,
                    age_respect,
                    rep_overall,
                });
        }
        drop(query);

        let decrement = self.tick_interval.min(u32::MAX as u64) as u32;
        let election_interval = config::LEADER_REELECTION_INTERVAL.min(u32::MAX as u64) as u32;
        let min_population = config::LEADER_MIN_POPULATION as usize;
        let tie_margin = config::LEADER_CHARISMA_TIE_MARGIN as f32;

        for (settlement_id, settlement) in resources.settlements.iter_mut() {
            let mut needs_election = settlement.leader_id.is_none();

            if let Some(current_leader) = settlement.leader_id {
                let still_valid = alive_settlement_by_entity
                    .get(&current_leader)
                    .map(|owner| *owner == *settlement_id)
                    .unwrap_or(false);
                if !still_valid {
                    settlement.leader_id = None;
                    settlement.leader_reelection_countdown = 0;
                    needs_election = true;
                } else {
                    settlement.leader_reelection_countdown =
                        settlement.leader_reelection_countdown.saturating_sub(decrement);
                    if settlement.leader_reelection_countdown == 0 {
                        needs_election = true;
                    }
                }
            } else {
                settlement.leader_reelection_countdown = 0;
            }

            if !needs_election {
                continue;
            }

            let Some(candidates) = candidates_by_settlement.get(settlement_id) else {
                continue;
            };
            if candidates.len() < min_population {
                continue;
            }

            let mut best_score = f32::NEG_INFINITY;
            let mut best_rep = f32::NEG_INFINITY;
            let mut best_entity_raw = u64::MAX;
            let mut best_entity = EntityId::NONE;

            for candidate in candidates {
                let score = body::leader_score(
                    candidate.charisma,
                    candidate.wisdom,
                    candidate.trustworthiness,
                    candidate.intimidation,
                    candidate.social_capital,
                    candidate.age_respect,
                    config::LEADER_W_CHARISMA as f32,
                    config::LEADER_W_WISDOM as f32,
                    config::LEADER_W_TRUSTWORTHINESS as f32,
                    config::LEADER_W_INTIMIDATION as f32,
                    config::LEADER_W_SOCIAL_CAPITAL as f32,
                    config::LEADER_W_AGE_RESPECT as f32,
                    candidate.rep_overall,
                );
                let candidate_raw = candidate.entity.id() as u64;
                let candidate_id = EntityId(candidate_raw);
                let better_score = score > best_score + tie_margin;
                let tie_with_best = (score - best_score).abs() <= tie_margin;
                let better_tie = tie_with_best
                    && (candidate.rep_overall > best_rep + 1e-6
                        || ((candidate.rep_overall - best_rep).abs() <= 1e-6
                            && candidate_raw < best_entity_raw));
                if better_score || better_tie {
                    best_score = score;
                    best_rep = candidate.rep_overall;
                    best_entity_raw = candidate_raw;
                    best_entity = candidate_id;
                }
            }

            if best_entity != EntityId::NONE {
                settlement.leader_id = Some(best_entity);
                settlement.leader_reelection_countdown = election_interval;
            }
        }
    }
}
