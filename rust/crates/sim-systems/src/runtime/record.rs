#![allow(unused_imports)]
// TODO(v3.1): REFACTOR - split legacy threshold/chronicle behaviors from reusable observation and telemetry paths.

use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence,
    Memory, MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, BuildingId, CopingStrategyId, EmotionType, EntityId, GrowthStage,
    HexacoAxis, HexacoFacet, IntelligenceType, MentalBreakType, NeedType, RelationType,
    ResourceType, SettlementId, Sex, SocialClass, TechState, ValueType,
};
use sim_engine::{SimResources, SimSystem};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::body;

const STATS_RECORDER_MAX_HISTORY: usize = 200;

/// Rust runtime system for stat-sync derived composite cache refresh.
///
/// This performs active writes on `SimResources.stat_sync_derived`.
#[derive(Debug, Clone)]
pub struct StatSyncRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatSyncRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatSyncRuntimeSystem {
    fn name(&self) -> &'static str {
        "stat_sync_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut next_cache: HashMap<EntityId, [f32; 8]> = HashMap::new();

        let mut query = world.query::<(
            &Age,
            Option<&Personality>,
            Option<&Emotion>,
            Option<&BodyComponent>,
            Option<&Values>,
            Option<&Needs>,
            Option<&Intelligence>,
        )>();
        for (
            entity,
            (age, personality_opt, emotion_opt, body_opt, values_opt, needs_opt, intel_opt),
        ) in &mut query
        {
            if !age.alive {
                continue;
            }
            let x = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::X) as f32)
                .unwrap_or(0.5);
            let a = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::A) as f32)
                .unwrap_or(0.5);
            let h = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::H) as f32)
                .unwrap_or(0.5);
            let e = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::E) as f32)
                .unwrap_or(0.5);
            let o = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::O) as f32)
                .unwrap_or(0.5);
            let c = personality_opt
                .map(|personality_component| personality_component.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5);

            let joy = emotion_opt
                .map(|emotion| emotion.get(EmotionType::Joy) as f32)
                .unwrap_or(0.0);
            let anticipation = emotion_opt
                .map(|emotion| emotion.get(EmotionType::Anticipation) as f32)
                .unwrap_or(0.0);
            let anger = emotion_opt
                .map(|emotion| emotion.get(EmotionType::Anger) as f32)
                .unwrap_or(0.0);

            let str_pot = body_opt
                .map(|body_component| {
                    (body_component.str_potential as f32 / config::BODY_POTENTIAL_MAX as f32)
                        .clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let attractiveness = body_opt
                .map(|body_component| body_component.attractiveness)
                .unwrap_or(0.5);
            let height = body_opt
                .map(|body_component| body_component.height)
                .unwrap_or(0.5);

            let value_norm = |value_type: ValueType| -> f32 {
                values_opt
                    .map(|values| ((values.get(value_type) as f32 + 1.0) * 0.5).clamp(0.0, 1.0))
                    .unwrap_or(0.5)
            };
            let romance = value_norm(ValueType::Romance);
            let truth = value_norm(ValueType::Truth);
            let artwork = value_norm(ValueType::Artwork);
            let knowledge = value_norm(ValueType::Knowledge);
            let merriment = value_norm(ValueType::Merriment);
            let friendship = value_norm(ValueType::Friendship);
            let competition = value_norm(ValueType::Competition);
            let recognition = needs_opt
                .map(|needs| needs.get(NeedType::Recognition) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);

            let i_ling = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Linguistic) as f32)
                .unwrap_or(0.5);
            let i_log = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Logical) as f32)
                .unwrap_or(0.5);
            let i_spa = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Spatial) as f32)
                .unwrap_or(0.5);
            let i_mus = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Musical) as f32)
                .unwrap_or(0.5);
            let i_kin = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Kinesthetic) as f32)
                .unwrap_or(0.5);
            let i_inter = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Interpersonal) as f32)
                .unwrap_or(0.5);
            let i_intra = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Intrapersonal) as f32)
                .unwrap_or(0.5);
            let i_nat = intel_opt
                .map(|intelligence| intelligence.get(IntelligenceType::Naturalistic) as f32)
                .unwrap_or(0.5);

            let inputs = [
                x,
                a,
                h,
                e,
                o,
                c,
                joy,
                anticipation,
                anger,
                str_pot,
                romance,
                truth,
                artwork,
                knowledge,
                merriment,
                friendship,
                competition,
                recognition,
                i_ling,
                i_log,
                i_spa,
                i_mus,
                i_kin,
                i_inter,
                i_intra,
                i_nat,
                attractiveness,
                height,
                age.years as f32,
            ];
            let derived = body::stat_sync_derived_scores(&inputs);
            next_cache.insert(EntityId(entity.id() as u64), derived);
        }
        resources.stat_sync_derived = next_cache;
    }
}

pub const STAT_THRESHOLD_FLAG_HUNGER_LOW: u32 = 1 << 0;
pub const STAT_THRESHOLD_FLAG_STRESS_HIGH: u32 = 1 << 1;

/// Rust runtime system for threshold evaluation and effect application.
///
/// This performs active writes on `SimResources.stat_threshold_flags` and
/// entity `Behavior.current_action`.
#[derive(Debug, Clone)]
pub struct StatThresholdRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatThresholdRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatThresholdRuntimeSystem {
    fn name(&self) -> &'static str {
        "stat_threshold_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let previous_flags = resources.stat_threshold_flags.clone();
        let mut next_flags: HashMap<EntityId, u32> = HashMap::new();

        let mut query = world.query::<(&Age, &Needs, &Stress, &mut Behavior)>();
        for (entity, (age, needs, stress, behavior)) in &mut query {
            if !age.alive {
                continue;
            }
            let entity_id = EntityId(entity.id() as u64);
            let current_flags = previous_flags.get(&entity_id).copied().unwrap_or(0);

            let hunger_value = (needs.get(NeedType::Hunger) as f32 * 1000.0).round() as i32;
            let stress_value = (stress.level as f32 * 1000.0).round() as i32;

            let hunger_active = body::stat_threshold_is_active(
                hunger_value,
                200,
                0,
                50,
                (current_flags & STAT_THRESHOLD_FLAG_HUNGER_LOW) != 0,
            );
            let stress_active = body::stat_threshold_is_active(
                stress_value,
                700,
                1,
                40,
                (current_flags & STAT_THRESHOLD_FLAG_STRESS_HIGH) != 0,
            );

            let mut flags = 0_u32;
            if hunger_active {
                flags |= STAT_THRESHOLD_FLAG_HUNGER_LOW;
            }
            if stress_active {
                flags |= STAT_THRESHOLD_FLAG_STRESS_HIGH;
            }
            if flags != 0 {
                next_flags.insert(entity_id, flags);
            }

            if stress_active {
                behavior.current_action = ActionType::Rest;
            } else if hunger_active {
                behavior.current_action = ActionType::Forage;
            } else if matches!(
                behavior.current_action,
                ActionType::Rest | ActionType::Forage
            ) {
                behavior.current_action = ActionType::Idle;
            }
        }

        resources.stat_threshold_flags = next_flags;
    }
}

/// Rust runtime system for aggregated simulation stats snapshots.
///
/// This performs active writes on `SimResources.stats_history` and
/// `SimResources.stats_peak_population`.
#[derive(Debug, Clone)]
pub struct StatsRecorderRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatsRecorderRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatsRecorderRuntimeSystem {
    fn name(&self) -> &'static str {
        "stats_recorder"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let mut pop = 0_usize;
        let mut gatherers = 0_u32;
        let mut lumberjacks = 0_u32;
        let mut builders = 0_u32;
        let mut miners = 0_u32;
        let mut none_job = 0_u32;

        let mut query = world.query::<(&Age, Option<&Behavior>)>();
        for (_, (age, behavior_opt)) in &mut query {
            if !age.alive {
                continue;
            }
            pop += 1;
            let Some(behavior) = behavior_opt else {
                none_job = none_job.saturating_add(1);
                continue;
            };
            match behavior.job.as_str() {
                "gatherer" => gatherers = gatherers.saturating_add(1),
                "lumberjack" => lumberjacks = lumberjacks.saturating_add(1),
                "builder" => builders = builders.saturating_add(1),
                "miner" => miners = miners.saturating_add(1),
                _ => none_job = none_job.saturating_add(1),
            }
        }

        let mut food = 0.0_f64;
        let mut wood = 0.0_f64;
        let mut stone = 0.0_f64;
        for settlement in resources.settlements.values() {
            food += settlement.stockpile_food.max(0.0);
            wood += settlement.stockpile_wood.max(0.0);
            stone += settlement.stockpile_stone.max(0.0);
        }

        if pop > resources.stats_peak_population {
            resources.stats_peak_population = pop;
        }
        resources
            .stats_history
            .push(sim_engine::RuntimeStatsSnapshot {
                tick,
                pop,
                food,
                wood,
                stone,
                gatherers,
                lumberjacks,
                builders,
                miners,
                none_job,
            });
        if resources.stats_history.len() > STATS_RECORDER_MAX_HISTORY {
            let overflow = resources.stats_history.len() - STATS_RECORDER_MAX_HISTORY;
            resources.stats_history.drain(0..overflow);
        }
    }
}

// ── Chronicle System ─────────────────────────────────────────────────

/// Constants matching chronicle_system.gd
const CHRONICLE_PRUNE_INTERVAL_YEARS: i32 = 10;
const CHRONICLE_LOW_IMPORTANCE_MAX_AGE_YEARS: i32 = 20;
const CHRONICLE_MED_IMPORTANCE_MAX_AGE_YEARS: i32 = 50;

/// Rust runtime system for chronicle event pruning.
///
/// Periodically removes old low-significance chronicle entries while keeping
/// bounded high-significance history intact.
#[derive(Debug, Clone)]
// TODO(v3.1): DELETE - replace legacy chronicle pruning path with v3.1 causal log + observation/oracle layers.
pub struct ChronicleRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    last_prune_year: i32,
}

impl ChronicleRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            last_prune_year: 0,
        }
    }
}

impl SimSystem for ChronicleRuntimeSystem {
    fn name(&self) -> &'static str {
        "chronicle_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, tick: u64) {
        let ticks_per_year = config::TICKS_PER_YEAR as i32;
        if ticks_per_year <= 0 {
            return;
        }
        let current_year = tick as i32 / ticks_per_year;

        if !body::chronicle_should_prune(
            current_year,
            self.last_prune_year,
            CHRONICLE_PRUNE_INTERVAL_YEARS,
        ) {
            return;
        }
        self.last_prune_year = current_year;

        let low_cutoff = body::chronicle_cutoff_tick(
            current_year,
            CHRONICLE_LOW_IMPORTANCE_MAX_AGE_YEARS,
            ticks_per_year,
        )
        .max(0) as u64;
        let med_cutoff = body::chronicle_cutoff_tick(
            current_year,
            CHRONICLE_MED_IMPORTANCE_MAX_AGE_YEARS,
            ticks_per_year,
        )
        .max(0) as u64;

        resources
            .chronicle_log
            .prune_by_significance(low_cutoff, med_cutoff);
    }
}
