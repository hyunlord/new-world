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
    CopingStrategyId, IntelligenceType, MentalBreakType, NeedType, RelationType, ResourceType,
    Sex, ValueType,
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

/// Rust runtime system for population-scale job assignment/rebalance.
///
/// This performs active writes on `Behavior.job` and mirrors the core
/// assignment rules from `job_assignment_system.gd`:
/// - infant/toddler: `none`
/// - child/teen assignment constraints
/// - ratio-based unassigned fill
/// - one-per-tick rebalance when distributions drift
#[derive(Debug, Clone)]
pub struct JobAssignmentRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl JobAssignmentRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const JOB_ASSIGNMENT_ORDER: [&str; 4] = ["gatherer", "lumberjack", "builder", "miner"];
const JOB_ASSIGNMENT_SURVIVAL_RATIOS: [f32; 4] = [0.8, 0.1, 0.1, 0.0];
const JOB_ASSIGNMENT_CRISIS_RATIOS: [f32; 4] = [0.6, 0.2, 0.1, 0.1];
const JOB_ASSIGNMENT_DEFAULT_RATIOS: [f32; 4] = [0.5, 0.25, 0.15, 0.1];
const JOB_ASSIGNMENT_CRISIS_FOOD_PER_ALIVE: f32 = 1.5;
const JOB_ASSIGNMENT_REBALANCE_THRESHOLD: f32 = 1.5;

#[inline]
fn job_assignment_job_index(job: &str) -> Option<usize> {
    match job {
        "gatherer" => Some(0),
        "lumberjack" => Some(1),
        "builder" => Some(2),
        "miner" => Some(3),
        _ => None,
    }
}

#[inline]
fn job_assignment_ratios(resources: &SimResources, alive_count: i32) -> [f32; 4] {
    if alive_count < 10 {
        return JOB_ASSIGNMENT_SURVIVAL_RATIOS;
    }

    let mut total_food = 0.0_f32;
    let width = resources.map.width;
    let height = resources.map.height;
    for y in 0..height {
        for x in 0..width {
            let tile = resources.map.get(x, y);
            for deposit in &tile.resources {
                if matches!(deposit.resource_type, ResourceType::Food) {
                    total_food += (deposit.amount as f32).max(0.0);
                }
            }
        }
    }
    if total_food < alive_count as f32 * JOB_ASSIGNMENT_CRISIS_FOOD_PER_ALIVE {
        JOB_ASSIGNMENT_CRISIS_RATIOS
    } else {
        JOB_ASSIGNMENT_DEFAULT_RATIOS
    }
}

impl SimSystem for JobAssignmentRuntimeSystem {
    fn name(&self) -> &'static str {
        "job_assignment_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut alive_count: i32 = 0;
        let mut job_counts: [i32; 4] = [0; 4];
        let mut unassigned: Vec<(Entity, GrowthStage)> = Vec::new();

        let mut query = world.query::<(&Age, &mut Behavior)>();
        for (entity, (age, behavior)) in &mut query {
            alive_count += 1;
            match age.stage {
                GrowthStage::Infant | GrowthStage::Toddler => {
                    if behavior.job != "none" {
                        behavior.job = "none".to_string();
                    }
                    continue;
                }
                GrowthStage::Child => {
                    if behavior.job != "gatherer" {
                        behavior.job = "gatherer".to_string();
                    }
                    continue;
                }
                _ => {}
            }

            if !behavior.occupation.is_empty()
                && behavior.occupation != "none"
                && behavior.occupation != "laborer"
            {
                if let Some(idx) = job_assignment_job_index(behavior.job.as_str()) {
                    job_counts[idx] += 1;
                }
                continue;
            }

            if behavior.job == "none" {
                unassigned.push((entity, age.stage));
            } else if let Some(idx) = job_assignment_job_index(behavior.job.as_str()) {
                job_counts[idx] += 1;
            }
        }
        drop(query);

        if alive_count <= 0 {
            return;
        }
        let ratios = job_assignment_ratios(resources, alive_count);

        for (entity, stage) in &unassigned {
            let mut target_idx = if matches!(stage, GrowthStage::Teen) {
                0_usize
            } else {
                let raw_idx = body::job_assignment_best_job_code(&ratios, &job_counts, alive_count);
                if raw_idx < 0 || raw_idx as usize >= JOB_ASSIGNMENT_ORDER.len() {
                    0
                } else {
                    raw_idx as usize
                }
            };

            let mut target_job = JOB_ASSIGNMENT_ORDER[target_idx];
            if matches!(stage, GrowthStage::Elder) && target_job == "builder" {
                target_idx = 0;
                target_job = JOB_ASSIGNMENT_ORDER[target_idx];
            }

            if let Ok(mut one) = world.query_one::<&mut Behavior>(*entity) {
                if let Some(behavior) = one.get() {
                    behavior.job = target_job.to_string();
                    job_counts[target_idx] += 1;
                }
            }
        }

        if unassigned.is_empty() && alive_count >= 5 {
            let pair = body::job_assignment_rebalance_codes(
                &ratios,
                &job_counts,
                alive_count,
                JOB_ASSIGNMENT_REBALANCE_THRESHOLD,
            );
            let surplus_idx = pair[0];
            let deficit_idx = pair[1];
            if surplus_idx >= 0
                && deficit_idx >= 0
                && (surplus_idx as usize) < JOB_ASSIGNMENT_ORDER.len()
                && (deficit_idx as usize) < JOB_ASSIGNMENT_ORDER.len()
                && surplus_idx != deficit_idx
            {
                let surplus_job = JOB_ASSIGNMENT_ORDER[surplus_idx as usize];
                let deficit_job = JOB_ASSIGNMENT_ORDER[deficit_idx as usize];
                let mut rebalance_query = world.query::<&mut Behavior>();
                for (_, behavior) in &mut rebalance_query {
                    if behavior.job == surplus_job && behavior.current_action == ActionType::Idle {
                        behavior.job = deficit_job.to_string();
                        break;
                    }
                }
            }
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

/// Rust runtime system for mental-break trigger and recovery.
///
/// This performs active writes on `Stress.active_mental_break`,
/// `Stress.mental_break_remaining`, and `Stress.mental_break_count`.
#[derive(Debug, Clone)]
pub struct MentalBreakRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MentalBreakRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const MENTAL_BREAK_BASE_THRESHOLD: f32 = 520.0;
const MENTAL_BREAK_THRESHOLD_MIN: f32 = 420.0;
const MENTAL_BREAK_THRESHOLD_MAX: f32 = 900.0;
const MENTAL_BREAK_SCALE: f32 = 6000.0;
const MENTAL_BREAK_CAP_PER_TICK: f32 = 0.25;
const MENTAL_BREAK_STRESS_SCALE: f32 = 2000.0;
const MENTAL_BREAK_OUTRAGE_THRESHOLD: f32 = 60.0;

#[inline]
fn mental_break_type_from_code(code: i32) -> MentalBreakType {
    match code {
        1 => MentalBreakType::OutrageViolence,
        2 => MentalBreakType::Panic,
        3 => MentalBreakType::Rage,
        5 => MentalBreakType::Purge,
        _ => MentalBreakType::Shutdown,
    }
}

#[inline]
fn mental_break_duration_ticks(kind: MentalBreakType) -> u32 {
    match kind {
        MentalBreakType::OutrageViolence => 24,
        MentalBreakType::Panic => 16,
        MentalBreakType::Rage => 20,
        MentalBreakType::Purge => 18,
        _ => 30,
    }
}

impl SimSystem for MentalBreakRuntimeSystem {
    fn name(&self) -> &'static str {
        "mental_break_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Stress,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Emotion>,
        )>();
        for (_, (stress, needs_opt, personality_opt, emotion_opt)) in &mut query {
            if stress.active_mental_break.is_some() {
                let dec = self.tick_interval.min(u32::MAX as u64) as u32;
                if stress.mental_break_remaining > dec {
                    stress.mental_break_remaining -= dec;
                } else {
                    stress.mental_break_remaining = 0;
                    stress.active_mental_break = None;
                    stress.level = (stress.level as f32 * 0.80).clamp(0.0, 1.0) as f64;
                    stress.recalculate_state();
                }
                continue;
            }

            if stress.level < 0.70 {
                continue;
            }

            let c_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let e_axis = personality_opt
                .map(|personality| personality.axis(HexacoAxis::E) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let energy = needs_opt
                .map(|needs| needs.energy as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let hunger = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let resilience = c_axis;
            let reserve_scaled = (stress.reserve as f32 * 100.0).clamp(0.0, 100.0);
            let allostatic_scaled = (stress.allostatic_load as f32 * 100.0).clamp(0.0, 100.0);
            let stress_scaled = (stress.level as f32 * MENTAL_BREAK_STRESS_SCALE).clamp(0.0, 4000.0);

            let threshold = body::mental_break_threshold(
                MENTAL_BREAK_BASE_THRESHOLD,
                resilience,
                c_axis,
                e_axis,
                allostatic_scaled,
                energy,
                hunger,
                1.0,
                0.0,
                MENTAL_BREAK_THRESHOLD_MIN,
                MENTAL_BREAK_THRESHOLD_MAX,
                reserve_scaled,
                0.0,
            );
            let trigger_p = body::mental_break_chance(
                stress_scaled,
                threshold,
                reserve_scaled,
                allostatic_scaled,
                MENTAL_BREAK_SCALE,
                MENTAL_BREAK_CAP_PER_TICK,
            )
            .clamp(0.0, 1.0);
            if trigger_p <= 0.0 {
                continue;
            }

            let roll: f32 = resources.rng.gen_range(0.0..1.0);
            if roll >= trigger_p {
                continue;
            }

            let (fear, anger, sadness, disgust) = if let Some(emotion) = emotion_opt {
                (
                    emotion.get(EmotionType::Fear) as f32 * 100.0,
                    emotion.get(EmotionType::Anger) as f32 * 100.0,
                    emotion.get(EmotionType::Sadness) as f32 * 100.0,
                    emotion.get(EmotionType::Disgust) as f32 * 100.0,
                )
            } else {
                (20.0, 20.0, 30.0, 15.0)
            };
            let outrage = (anger * 0.6 + fear * 0.4).clamp(0.0, 100.0);
            let break_code = body::emotion_break_type_code(
                outrage,
                fear,
                anger,
                sadness,
                disgust,
                MENTAL_BREAK_OUTRAGE_THRESHOLD,
            );
            let break_type = mental_break_type_from_code(break_code);
            stress.active_mental_break = Some(break_type);
            stress.mental_break_count = stress.mental_break_count.saturating_add(1);
            stress.mental_break_remaining = mental_break_duration_ticks(break_type);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TraitViolationHistory {
    desensitize_mult: f32,
    ptsd_mult: f32,
    repeats: u32,
    last_violation_tick: u64,
}

/// Rust runtime system for trait-violation stress updates.
///
/// This performs active writes on `Stress.level/reserve/allostatic_load` using
/// trait-context mismatch checks and persistent desensitization/PTSD history.
#[derive(Debug, Clone)]
pub struct TraitViolationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    history: HashMap<Entity, TraitViolationHistory>,
}

impl TraitViolationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            history: HashMap::new(),
        }
    }
}

const TRAIT_VIOLATION_DESENSITIZE_DECAY: f32 = 0.85;
const TRAIT_VIOLATION_DESENSITIZE_MIN: f32 = 0.30;
const TRAIT_VIOLATION_PTSD_INCREASE: f32 = 1.10;
const TRAIT_VIOLATION_PTSD_MAX: f32 = 2.0;
const TRAIT_VIOLATION_ALLOSTATIC_THRESHOLD: f32 = 0.50;
const TRAIT_VIOLATION_BASE_INTRUSIVE_CHANCE: f32 = 0.005;
const TRAIT_VIOLATION_HISTORY_DECAY_TICKS: i32 = 365 * 12;

#[inline]
fn trait_violation_action_base(action: ActionType) -> Option<(&'static str, f32)> {
    match action {
        ActionType::TakeFromStockpile => Some(("steal", 22.0)),
        ActionType::Fight => Some(("harm_innocent", 20.0)),
        ActionType::MentalBreak => Some(("panic", 10.0)),
        ActionType::Flee => Some(("retreat", 12.0)),
        _ => None,
    }
}

#[inline]
fn trait_violation_matching_facet(
    action_key: &str,
    traits_opt: Option<&Traits>,
    personality_opt: Option<&Personality>,
) -> Option<f32> {
    let traits = traits_opt?;
    let personality = personality_opt?;
    let match_trait = |id: &str| traits.has_trait(id);
    match action_key {
        "steal" if match_trait("f_fair_minded") => Some(personality.facet(HexacoFacet::Fairness) as f32),
        "steal" if match_trait("f_sincere") => Some(personality.facet(HexacoFacet::Sincerity) as f32),
        "harm_innocent" if match_trait("f_sentimental") => {
            Some(personality.facet(HexacoFacet::Sentimentality) as f32)
        }
        "harm_innocent" if match_trait("f_gentle") => Some(personality.facet(HexacoFacet::Gentleness) as f32),
        "panic" if match_trait("f_calm") => Some(personality.facet(HexacoFacet::Anxiety) as f32),
        "retreat" if match_trait("f_fearless") => Some(personality.facet(HexacoFacet::Fearfulness) as f32),
        _ => None,
    }
}

impl SimSystem for TraitViolationRuntimeSystem {
    fn name(&self) -> &'static str {
        "trait_violation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let mut query = world.query::<(
            &mut Stress,
            Option<&Behavior>,
            Option<&Needs>,
            Option<&Social>,
            Option<&Traits>,
            Option<&Personality>,
        )>();
        for (entity, (stress, behavior_opt, needs_opt, social_opt, traits_opt, personality_opt)) in
            &mut query
        {
            let mut history = self.history.get(&entity).copied().unwrap_or(TraitViolationHistory {
                desensitize_mult: 1.0,
                ptsd_mult: 1.0,
                repeats: 0,
                last_violation_tick: tick,
            });

            if let Some(behavior) = behavior_opt {
                if let Some((action_key, base_stress)) =
                    trait_violation_action_base(behavior.current_action)
                {
                    if let Some(facet_value) =
                        trait_violation_matching_facet(action_key, traits_opt, personality_opt)
                    {
                        let survival_necessity = matches!(action_key, "steal")
                            && needs_opt
                                .map(|needs| (needs.get(NeedType::Hunger) as f32) < 0.2)
                                .unwrap_or(false);
                        let no_witness = social_opt
                            .map(|social| (social.social_capital as f32) < 0.2)
                            .unwrap_or(true);
                        let is_habit = history.repeats >= 3
                            && tick.saturating_sub(history.last_violation_tick) < 180;
                        let context_mult = body::trait_violation_context_modifier(
                            is_habit,
                            false,
                            survival_necessity,
                            no_witness,
                            0.0,
                            0.5,
                            0.4,
                            0.85,
                        );
                        if context_mult > 0.0 {
                            let facet_scale =
                                body::trait_violation_facet_scale(facet_value.clamp(0.0, 1.0), 0.6);
                            let stress_delta = (base_stress
                                * facet_scale
                                * history.desensitize_mult
                                * history.ptsd_mult
                                * context_mult
                                / 400.0)
                                .clamp(0.0, 0.25);
                            if stress_delta > 0.0 {
                                let next_level = (stress.level as f32 + stress_delta).clamp(0.0, 1.0);
                                let next_allostatic = (stress.allostatic_load as f32 + stress_delta * 0.45)
                                    .clamp(0.0, 1.0);
                                let next_reserve =
                                    (stress.reserve as f32 - stress_delta * 0.35).clamp(0.0, 1.0);
                                stress.level = next_level as f64;
                                stress.allostatic_load = next_allostatic as f64;
                                stress.reserve = next_reserve as f64;
                                stress.recalculate_state();

                                if next_allostatic < TRAIT_VIOLATION_ALLOSTATIC_THRESHOLD {
                                    history.desensitize_mult = (history.desensitize_mult
                                        * TRAIT_VIOLATION_DESENSITIZE_DECAY)
                                        .max(TRAIT_VIOLATION_DESENSITIZE_MIN);
                                } else {
                                    history.ptsd_mult = (history.ptsd_mult
                                        * TRAIT_VIOLATION_PTSD_INCREASE)
                                        .min(TRAIT_VIOLATION_PTSD_MAX);
                                }
                                history.repeats = history.repeats.saturating_add(1);
                                history.last_violation_tick = tick;
                            }
                        } else {
                            history.repeats = history.repeats.saturating_add(1);
                            history.last_violation_tick = tick;
                        }
                    }
                }
            }

            let ticks_since = tick.saturating_sub(history.last_violation_tick) as i32;
            let intrusive_p = body::trait_violation_intrusive_chance(
                TRAIT_VIOLATION_BASE_INTRUSIVE_CHANCE,
                history.ptsd_mult,
                ticks_since,
                TRAIT_VIOLATION_HISTORY_DECAY_TICKS,
                false,
            )
            .clamp(0.0, 1.0);
            if intrusive_p > 0.0 {
                let roll: f32 = resources.rng.gen_range(0.0..1.0);
                if roll < intrusive_p {
                    let intrusive_stress = (0.004 * history.ptsd_mult).clamp(0.0, 0.03);
                    stress.level = (stress.level as f32 + intrusive_stress).clamp(0.0, 1.0) as f64;
                    stress.allostatic_load = (stress.allostatic_load as f32 + intrusive_stress * 0.20)
                        .clamp(0.0, 1.0) as f64;
                    stress.recalculate_state();
                }
            }

            if ticks_since > TRAIT_VIOLATION_HISTORY_DECAY_TICKS {
                history.repeats = history.repeats.saturating_sub(1);
                history.desensitize_mult =
                    (history.desensitize_mult + 0.01).clamp(TRAIT_VIOLATION_DESENSITIZE_MIN, 1.0);
                history.ptsd_mult = (history.ptsd_mult - 0.01).clamp(1.0, TRAIT_VIOLATION_PTSD_MAX);
            }
            self.history.insert(entity, history);
        }
    }
}

/// Rust runtime system for trauma-scar baseline drift.
///
/// This performs active writes on `Emotion.baseline` from persistent
/// `Memory.trauma_scars` entries.
#[derive(Debug, Clone)]
pub struct TraumaScarRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TraumaScarRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const TRAUMA_SCAR_BASELINE_SCALE: f32 = 0.001;

#[inline]
fn trauma_apply_baseline_delta(emotion: &mut Emotion, emotion_type: EmotionType, delta: f32) {
    let idx = emotion_type as usize;
    let next = (emotion.baseline[idx] as f32 + delta).clamp(0.0, 1.0);
    emotion.baseline[idx] = next as f64;
}

fn trauma_scar_apply_baseline_shifts(emotion: &mut Emotion, scar_id: &str, scale: f32) {
    match scar_id {
        "hypervigilance" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.10 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        "anger_dysregulation" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Anger, 0.08 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        "violence_imprint" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.08 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.10 * scale);
        }
        "emotional_numbness" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.10 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Trust, -0.10 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.15 * scale);
        }
        "compulsive_consumption" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Anticipation, 0.02 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        "complicated_grief" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.15 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.15 * scale);
        }
        "dissociative_tendency" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.08 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.08 * scale);
        }
        "chronic_paranoia" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.12 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.15 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.10 * scale);
        }
        "anxious_attachment" => {
            trauma_apply_baseline_delta(emotion, EmotionType::Sadness, 0.12 * scale);
            trauma_apply_baseline_delta(emotion, EmotionType::Joy, -0.05 * scale);
        }
        _ => {
            trauma_apply_baseline_delta(emotion, EmotionType::Fear, 0.05 * scale);
        }
    }
}

impl SimSystem for TraumaScarRuntimeSystem {
    fn name(&self) -> &'static str {
        "trauma_scar_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(&Memory, &mut Emotion)>();
        for (_, (memory, emotion)) in &mut query {
            if memory.trauma_scars.is_empty() {
                continue;
            }
            for scar in &memory.trauma_scars {
                let severity = (scar.severity as f32).clamp(0.0, 1.0);
                let base_mult = 1.0 + severity * 0.25;
                let stacks = (scar.reactivation_count as i32).max(1);
                let sensitivity = body::trauma_scar_sensitivity_factor(base_mult, stacks)
                    .clamp(0.5, 3.0);
                let shift_scale = TRAUMA_SCAR_BASELINE_SCALE * sensitivity;
                trauma_scar_apply_baseline_shifts(emotion, scar.scar_id.as_str(), shift_scale);
            }
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

/// Rust runtime system for age/environment-adjusted intelligence updates.
///
/// This performs active writes on `Intelligence.values`, `Intelligence.g_factor`,
/// `Intelligence.ace_penalty`, and `Intelligence.nutrition_penalty`.
#[derive(Debug, Clone)]
pub struct IntelligenceRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    potential_baselines: HashMap<Entity, [f32; 8]>,
}

impl IntelligenceRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            potential_baselines: HashMap::new(),
        }
    }
}

const INTEL_CURVE_FLUID: [(f32, f32); 8] = [
    (0.0, 0.20),
    (5.0, 0.50),
    (15.0, 0.85),
    (22.0, 1.00),
    (35.0, 1.00),
    (55.0, 0.85),
    (75.0, 0.60),
    (100.0, 0.50),
];
const INTEL_CURVE_CRYSTALLIZED: [(f32, f32); 8] = [
    (0.0, 0.15),
    (5.0, 0.30),
    (15.0, 0.55),
    (25.0, 0.75),
    (50.0, 0.95),
    (65.0, 1.00),
    (80.0, 0.85),
    (100.0, 0.75),
];
const INTEL_CURVE_PHYSICAL: [(f32, f32); 9] = [
    (0.0, 0.10),
    (5.0, 0.35),
    (12.0, 0.65),
    (20.0, 0.90),
    (28.0, 1.00),
    (40.0, 0.85),
    (60.0, 0.60),
    (80.0, 0.45),
    (100.0, 0.35),
];

#[inline]
fn interpolate_curve(curve: &[(f32, f32)], age_years: f32) -> f32 {
    if curve.is_empty() {
        return 1.0;
    }
    if age_years <= curve[0].0 {
        return curve[0].1;
    }
    let last_idx = curve.len() - 1;
    if age_years >= curve[last_idx].0 {
        return curve[last_idx].1;
    }
    for idx in 1..curve.len() {
        let prev = curve[idx - 1];
        let next = curve[idx];
        if age_years <= next.0 {
            let span = (next.0 - prev.0).max(0.000_001);
            let t = ((age_years - prev.0) / span).clamp(0.0, 1.0);
            return prev.1 + (next.1 - prev.1) * t;
        }
    }
    curve[last_idx].1
}

#[inline]
fn is_fluid_intelligence(kind: IntelligenceType) -> bool {
    matches!(kind, IntelligenceType::Logical | IntelligenceType::Spatial)
}

#[inline]
fn intelligence_age_modifier(kind: IntelligenceType, age_years: f32) -> f32 {
    if is_fluid_intelligence(kind) {
        return interpolate_curve(&INTEL_CURVE_FLUID, age_years);
    }
    if matches!(kind, IntelligenceType::Kinesthetic) {
        return interpolate_curve(&INTEL_CURVE_PHYSICAL, age_years);
    }
    interpolate_curve(&INTEL_CURVE_CRYSTALLIZED, age_years)
}

impl SimSystem for IntelligenceRuntimeSystem {
    fn name(&self) -> &'static str {
        "intelligence_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Intelligence,
            Option<&Age>,
            Option<&Needs>,
            Option<&Skills>,
            Option<&Memory>,
            Option<&Identity>,
            Option<&Personality>,
        )>();
        for (entity, (intelligence, age_opt, needs_opt, skills_opt, memory_opt, identity_opt, personality_opt)) in &mut query {
            let baseline = self.potential_baselines.entry(entity).or_insert_with(|| {
                intelligence
                    .values
                    .map(|value| (value as f32).clamp(0.02, 0.98))
            });
            let age_ticks = age_opt.map(|age| age.ticks).unwrap_or(0);
            let age_years = age_opt
                .map(|age| {
                    if age.years > 0.0 {
                        age.years as f32
                    } else {
                        age.ticks as f32 / config::TICKS_PER_YEAR as f32
                    }
                })
                .unwrap_or(0.0)
                .max(0.0);

            if age_ticks <= config::INTEL_NUTRITION_CRIT_AGE_TICKS as u64
                && intelligence.nutrition_penalty < config::INTEL_NUTRITION_MAX_PENALTY
            {
                let hunger = needs_opt
                    .map(|needs| needs.get(NeedType::Hunger) as f32)
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                if hunger < config::INTEL_NUTRITION_HUNGER_THRESHOLD as f32 {
                    let severity = 1.0
                        - hunger / config::INTEL_NUTRITION_HUNGER_THRESHOLD as f32;
                    let delta = config::INTEL_NUTRITION_PENALTY_PER_TICK as f32 * severity;
                    intelligence.nutrition_penalty = (intelligence.nutrition_penalty as f32 + delta)
                        .min(config::INTEL_NUTRITION_MAX_PENALTY as f32)
                        as f64;
                }
            }

            if age_years >= config::INTEL_ACE_CRIT_AGE_YEARS as f32
                && intelligence.ace_penalty <= 0.0
            {
                let birth_tick = identity_opt.map(|identity| identity.birth_tick).unwrap_or(0);
                let cutoff = birth_tick + (config::INTEL_ACE_CRIT_AGE_YEARS as f32
                    * config::TICKS_PER_YEAR as f32) as u64;
                let scar_count = memory_opt
                    .map(|memory| {
                        memory
                            .trauma_scars
                            .iter()
                            .filter(|scar| scar.acquired_tick < cutoff)
                            .count() as u32
                    })
                    .unwrap_or(0);
                if scar_count >= config::INTEL_ACE_SCARS_THRESHOLD_MAJOR {
                    intelligence.ace_penalty = config::INTEL_ACE_PENALTY_MAJOR;
                } else if scar_count >= config::INTEL_ACE_SCARS_THRESHOLD_MINOR {
                    intelligence.ace_penalty = config::INTEL_ACE_PENALTY_MINOR;
                }
            }

            let active_skill_count = skills_opt
                .map(|skills| {
                    skills
                        .entries
                        .values()
                        .filter(|entry| {
                            u32::from(entry.level) >= config::INTEL_ACTIVITY_SKILL_THRESHOLD
                        })
                        .count() as i32
                })
                .unwrap_or(0);
            let activity_mod = body::cognition_activity_modifier(
                active_skill_count,
                config::INTEL_ACTIVITY_BUFFER as f32,
                config::INTEL_INACTIVITY_ACCEL as f32,
            );
            let ace_fluid_mult = body::cognition_ace_fluid_decline_mult(
                intelligence.ace_penalty as f32,
                config::INTEL_ACE_PENALTY_MINOR as f32,
                config::INTEL_ACE_FLUID_DECLINE_MULT as f32,
            );
            let env_penalty =
                (intelligence.nutrition_penalty + intelligence.ace_penalty) as f32;

            let intel_order = [
                IntelligenceType::Linguistic,
                IntelligenceType::Logical,
                IntelligenceType::Spatial,
                IntelligenceType::Musical,
                IntelligenceType::Kinesthetic,
                IntelligenceType::Interpersonal,
                IntelligenceType::Intrapersonal,
                IntelligenceType::Naturalistic,
            ];
            for (idx, kind) in intel_order.iter().enumerate() {
                let potential = baseline[idx];
                let base_mod = intelligence_age_modifier(*kind, age_years);
                let effective = body::intelligence_effective_value(
                    potential,
                    base_mod,
                    age_years,
                    is_fluid_intelligence(*kind),
                    activity_mod,
                    ace_fluid_mult,
                    env_penalty,
                    0.02,
                    0.98,
                );
                intelligence.values[idx] = effective as f64;
            }

            let openness = personality_opt
                .map(|personality| personality.axis(HexacoAxis::O) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            intelligence.g_factor = body::intelligence_g_value(
                false,
                0.0,
                0.0,
                config::INTEL_HERITABILITY_G as f32,
                config::INTEL_G_MEAN as f32,
                openness,
                config::INTEL_OPENNESS_G_WEIGHT as f32,
                0.0,
            )
            .clamp(0.0, 1.0) as f64;
        }
    }
}

/// Rust runtime system for memory decay, eviction, compression, and promotion.
///
/// This ports the core working-memory management loop from `memory_system.gd`
/// into Rust ECS state writes.
#[derive(Debug, Clone)]
pub struct MemoryRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MemoryRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const MEMORY_FORGET_THRESHOLD: f32 = 0.01;
const MEMORY_SUMMARY_SCALE: f32 = 0.70;

#[inline]
fn memory_is_permanent_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "birth"
            | "marriage"
            | "child_born"
            | "partner_died"
            | "war"
            | "migration"
            | "promotion"
            | "betrayal"
            | "trauma"
            | "achievement"
            | "proposal"
    )
}

#[inline]
fn memory_decay_rate_from_encoding(intensity: f64) -> f32 {
    config::memory_decay_rate(intensity.clamp(0.0, 1.0)) as f32
}

impl SimSystem for MemoryRuntimeSystem {
    fn name(&self) -> &'static str {
        "memory_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        let dt_years = self.tick_interval as f32 / config::TICKS_PER_YEAR as f32;
        let cutoff_tick = tick.saturating_sub(config::MEMORY_COMPRESS_INTERVAL_TICKS);
        let mut query = world.query::<&mut Memory>();
        for (_, memory) in &mut query {
            if !memory.short_term.is_empty() {
                let entries: Vec<MemoryEntry> = memory.short_term.drain(..).collect();
                let mut intensities: Vec<f32> = Vec::with_capacity(entries.len());
                let mut rates: Vec<f32> = Vec::with_capacity(entries.len());
                for entry in &entries {
                    intensities.push(entry.current_intensity as f32);
                    rates.push(memory_decay_rate_from_encoding(entry.intensity));
                }
                let decayed = body::memory_decay_batch(&intensities, &rates, dt_years);
                let mut remaining: Vec<MemoryEntry> = Vec::with_capacity(entries.len());
                for (idx, mut entry) in entries.into_iter().enumerate() {
                    let next_intensity = decayed
                        .get(idx)
                        .copied()
                        .unwrap_or(entry.current_intensity as f32)
                        .clamp(0.0, 1.0);
                    if next_intensity < MEMORY_FORGET_THRESHOLD {
                        continue;
                    }
                    entry.current_intensity = next_intensity as f64;
                    remaining.push(entry);
                }
                memory.short_term = remaining.into_iter().collect();
            }

            if memory.short_term.len() > config::MEMORY_WORKING_MAX {
                let mut entries: Vec<MemoryEntry> = memory.short_term.drain(..).collect();
                entries.sort_by(|a, b| {
                    a.current_intensity
                        .partial_cmp(&b.current_intensity)
                        .unwrap_or(Ordering::Equal)
                });
                let excess = entries.len() - config::MEMORY_WORKING_MAX;
                entries.drain(0..excess);
                memory.short_term = entries.into_iter().collect();
            }

            let mut old_entries: Vec<MemoryEntry> = Vec::new();
            let mut recent_entries: Vec<MemoryEntry> = Vec::new();
            for entry in memory.short_term.drain(..) {
                if entry.tick < cutoff_tick {
                    old_entries.push(entry);
                } else {
                    recent_entries.push(entry);
                }
            }

            let mut rebuilt: Vec<MemoryEntry> = Vec::with_capacity(old_entries.len() + recent_entries.len());
            if old_entries.len() >= config::MEMORY_COMPRESS_MIN_GROUP {
                let mut groups: HashMap<(String, Option<u64>), Vec<MemoryEntry>> = HashMap::new();
                for entry in old_entries {
                    let key = (entry.event_type.clone(), entry.target_id);
                    groups.entry(key).or_default().push(entry);
                }
                let mut keys: Vec<(String, Option<u64>)> = groups.keys().cloned().collect();
                keys.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
                for key in keys {
                    let Some(group) = groups.remove(&key) else {
                        continue;
                    };
                    if group.len() < config::MEMORY_COMPRESS_MIN_GROUP {
                        rebuilt.extend(group.into_iter());
                        continue;
                    }
                    let mut max_intensity = 0.0_f32;
                    let mut oldest_tick = u64::MAX;
                    for entry in &group {
                        max_intensity = max_intensity.max(entry.current_intensity as f32);
                        oldest_tick = oldest_tick.min(entry.tick);
                    }
                    let event_type = group[0].event_type.clone();
                    let target_id = group[0].target_id;
                    let summary_intensity = body::memory_summary_intensity(
                        max_intensity,
                        MEMORY_SUMMARY_SCALE,
                    )
                    .clamp(0.0, 1.0);
                    rebuilt.push(MemoryEntry {
                        event_type: format!("{event_type}_summary"),
                        target_id,
                        tick: oldest_tick,
                        intensity: summary_intensity as f64,
                        current_intensity: summary_intensity as f64,
                        is_permanent: false,
                    });
                }
                memory.last_compression_tick = tick;
            } else {
                rebuilt.extend(old_entries.into_iter());
            }
            rebuilt.extend(recent_entries.into_iter());
            memory.short_term = rebuilt.into_iter().collect();

            let mut permanent_keys: HashSet<(String, u64)> = memory
                .permanent
                .iter()
                .map(|entry| (entry.event_type.clone(), entry.tick))
                .collect();
            let mut promoted: Vec<MemoryEntry> = Vec::new();
            for entry in memory.short_term.iter_mut() {
                if entry.current_intensity < config::MEMORY_PERMANENT_THRESHOLD {
                    continue;
                }
                if !memory_is_permanent_event(entry.event_type.as_str()) {
                    continue;
                }
                let key = (entry.event_type.clone(), entry.tick);
                if !permanent_keys.insert(key) {
                    continue;
                }
                entry.is_permanent = true;
                let mut permanent_entry = entry.clone();
                permanent_entry.is_permanent = true;
                promoted.push(permanent_entry);
            }
            memory.permanent.extend(promoted.into_iter());
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

/// Rust runtime system for Siler-model mortality checks.
///
/// This performs active writes on `Age.alive` based on age-gated
/// monthly/annual hazard checks using `body::mortality_hazards_and_prob`.
#[derive(Debug, Clone)]
pub struct MortalityRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MortalityRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const MORTALITY_A1: f32 = 0.60;
const MORTALITY_B1: f32 = 1.30;
const MORTALITY_A2: f32 = 0.010;
const MORTALITY_A3: f32 = 0.00006;
const MORTALITY_B3: f32 = 0.090;
const MORTALITY_TECH_K1: f32 = 0.30;
const MORTALITY_TECH_K2: f32 = 0.20;
const MORTALITY_TECH_K3: f32 = 0.05;
const MORTALITY_TECH_LEVEL: f32 = 0.0;
const MORTALITY_CARE_HUNGER_MIN: f32 = 0.3;
const MORTALITY_CARE_PROTECTION_FACTOR: f32 = 0.6;
const MORTALITY_SEASON_INFANT_MOD: f32 = 1.0;
const MORTALITY_SEASON_BACKGROUND_MOD: f32 = 1.0;

impl SimSystem for MortalityRuntimeSystem {
    fn name(&self) -> &'static str {
        "mortality_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let ticks_per_year = config::TICKS_PER_YEAR as u64;
        let ticks_per_month = config::TICKS_PER_MONTH as u64;
        let mut query = world.query::<(
            &mut Age,
            Option<&Needs>,
            Option<&BodyComponent>,
            Option<&Stress>,
        )>();
        for (_, (age, needs_opt, body_opt, stress_opt)) in &mut query {
            if !age.alive {
                continue;
            }

            let age_ticks = age.ticks;
            let is_infant = age_ticks < ticks_per_year;
            let should_check = if is_infant {
                age_ticks > 0 && age_ticks % ticks_per_month == 0
            } else {
                age_ticks > 0 && age_ticks % ticks_per_year == 0
            };
            if !should_check {
                continue;
            }

            let age_years = (age_ticks as f32 / ticks_per_year as f32).max(0.0);
            let nutrition = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let dr_norm = body_opt
                .map(|body| {
                    (body.dr_realized as f32 / config::BODY_REALIZED_DR_MAX as f32).clamp(0.0, 1.0)
                })
                .unwrap_or(0.5);
            let frailty = stress_opt
                .map(|stress| (1.0 + stress.allostatic_load as f32 * 2.0).clamp(0.5, 4.0))
                .unwrap_or(1.0);
            let hazards = body::mortality_hazards_and_prob(
                age_years,
                MORTALITY_A1,
                MORTALITY_B1,
                MORTALITY_A2,
                MORTALITY_A3,
                MORTALITY_B3,
                MORTALITY_TECH_K1,
                MORTALITY_TECH_K2,
                MORTALITY_TECH_K3,
                MORTALITY_TECH_LEVEL,
                nutrition,
                MORTALITY_CARE_HUNGER_MIN,
                MORTALITY_CARE_PROTECTION_FACTOR,
                MORTALITY_SEASON_INFANT_MOD,
                MORTALITY_SEASON_BACKGROUND_MOD,
                frailty,
                dr_norm,
                config::BODY_DR_MORTALITY_REDUCTION as f32,
                is_infant,
            );
            let q_check = hazards[5].clamp(0.0, 0.999);
            let roll: f32 = resources.rng.gen_range(0.0..1.0);
            if q_check >= 0.999 || roll < q_check {
                age.alive = false;
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

/// Rust runtime system for coping strategy cooldown/selection updates.
///
/// This performs active writes on `Coping.active_strategy`,
/// `Coping.strategy_cooldowns`, and `Coping.usage_counts`.
#[derive(Debug, Clone)]
pub struct CopingRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl CopingRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

const COPING_COUNT_MAX: f32 = 15.0;
const COPING_CLEAR_STRESS_MAX: f32 = 0.20;
const COPING_CLEAR_ALLOSTATIC_MAX: f32 = 0.20;
const COPING_RECOVERY_STRESS_MAX: f32 = 0.55;
const COPING_RECOVERY_ALLOSTATIC_MAX: f32 = 0.60;

const COPING_STRATEGY_ORDER: [CopingStrategyId; 15] = [
    CopingStrategyId::StrategicPlanning,
    CopingStrategyId::InstrumentalSupport,
    CopingStrategyId::EmotionalSupport,
    CopingStrategyId::PositiveReframing,
    CopingStrategyId::Denial,
    CopingStrategyId::Acceptance,
    CopingStrategyId::Humor,
    CopingStrategyId::ReligiousCoping,
    CopingStrategyId::Venting,
    CopingStrategyId::ActiveDistraction,
    CopingStrategyId::BehavioralDisengagement,
    CopingStrategyId::SelfBlame,
    CopingStrategyId::SubstanceUse,
    CopingStrategyId::Rumination,
    CopingStrategyId::ProblemSolving,
];

#[inline]
fn coping_strategy_from_index(index: i32) -> Option<CopingStrategyId> {
    if index < 0 || (index as usize) >= COPING_STRATEGY_ORDER.len() {
        return None;
    }
    Some(COPING_STRATEGY_ORDER[index as usize])
}

#[inline]
fn coping_strategy_cooldown_ticks(strategy: CopingStrategyId) -> u32 {
    match strategy {
        CopingStrategyId::Denial => 96,
        CopingStrategyId::SubstanceUse => 120,
        CopingStrategyId::Rumination => 72,
        CopingStrategyId::BehavioralDisengagement => 72,
        _ => 36,
    }
}

#[inline]
fn coping_axis(personality_opt: Option<&Personality>, axis: HexacoAxis) -> f32 {
    personality_opt
        .map(|personality| personality.axis(axis) as f32)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

#[inline]
fn coping_need(needs_opt: Option<&Needs>, need_type: NeedType, fallback: f32) -> f32 {
    needs_opt
        .map(|needs| needs.get(need_type) as f32)
        .unwrap_or(fallback)
        .clamp(0.0, 1.0)
}

fn coping_utility_scores(
    personality_opt: Option<&Personality>,
    needs_opt: Option<&Needs>,
    stress_norm: f32,
    allostatic_norm: f32,
) -> [f32; 15] {
    let x = coping_axis(personality_opt, HexacoAxis::X);
    let a = coping_axis(personality_opt, HexacoAxis::A);
    let h = coping_axis(personality_opt, HexacoAxis::H);
    let e = coping_axis(personality_opt, HexacoAxis::E);
    let o = coping_axis(personality_opt, HexacoAxis::O);
    let c = coping_axis(personality_opt, HexacoAxis::C);

    let belonging = coping_need(needs_opt, NeedType::Belonging, 0.5);
    let safety = coping_need(needs_opt, NeedType::Safety, 0.5);
    let energy = needs_opt
        .map(|needs| needs.energy as f32)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    let overwhelm = ((stress_norm + allostatic_norm) * 0.5).clamp(0.0, 1.0);
    let social_buffer = ((belonging + safety) * 0.5).clamp(0.0, 1.0);

    let mut scores = [0.0_f32; 15];
    scores[0] = c * 0.80 + (1.0 - overwhelm) * 0.60 + safety * 0.20;
    scores[1] = x * 0.40 + a * 0.40 + social_buffer * 0.30 + overwhelm * 0.20;
    scores[2] = x * 0.50 + a * 0.40 + (1.0 - belonging) * 0.20;
    scores[3] = o * 0.50 + c * 0.20 + (1.0 - overwhelm) * 0.30;
    scores[4] = (1.0 - c) * 0.50 + overwhelm * 0.70;
    scores[5] = c * 0.40 + a * 0.20 + overwhelm * 0.30;
    scores[6] = x * 0.50 + o * 0.30 + (1.0 - overwhelm) * 0.20;
    scores[7] = (1.0 - o) * 0.20 + a * 0.40 + overwhelm * 0.40;
    scores[8] = x * 0.20 + (1.0 - a) * 0.60 + overwhelm * 0.60;
    scores[9] = x * 0.30 + o * 0.30 + overwhelm * 0.40;
    scores[10] = (1.0 - c) * 0.60 + overwhelm * 0.70;
    scores[11] = h * 0.20 + (1.0 - e) * 0.50 + overwhelm * 0.60;
    scores[12] = (1.0 - c) * 0.50 + (1.0 - h) * 0.40 + overwhelm * 0.80;
    scores[13] = (1.0 - x) * 0.30 + (1.0 - e) * 0.40 + overwhelm * 0.70;
    scores[14] = c * 0.70 + o * 0.30 + (1.0 - overwhelm) * 0.40 + energy * 0.20;
    for score in &mut scores {
        *score = (*score).clamp(0.001, 5.0);
    }
    scores
}

impl SimSystem for CopingRuntimeSystem {
    fn name(&self) -> &'static str {
        "coping_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let dec = self.tick_interval.min(u32::MAX as u64) as u32;
        let mut query = world.query::<(&mut Coping, Option<&Stress>, Option<&Personality>, Option<&Needs>)>();
        for (_, (coping, stress_opt, personality_opt, needs_opt)) in &mut query {
            for cooldown in coping.strategy_cooldowns.values_mut() {
                *cooldown = cooldown.saturating_sub(dec);
            }

            let Some(stress) = stress_opt else {
                continue;
            };
            let stress_norm = (stress.level as f32).clamp(0.0, 1.0);
            let allostatic_norm = (stress.allostatic_load as f32).clamp(0.0, 1.0);

            if stress_norm <= COPING_CLEAR_STRESS_MAX && allostatic_norm <= COPING_CLEAR_ALLOSTATIC_MAX {
                coping.active_strategy = None;
                continue;
            }

            let is_recovery =
                stress_norm <= COPING_RECOVERY_STRESS_MAX && allostatic_norm <= COPING_RECOVERY_ALLOSTATIC_MAX;
            let break_count = stress.mental_break_count.min(i32::MAX as u32) as i32;
            let owned_count = coping.usage_counts.len().min(i32::MAX as usize) as i32;
            let learn_p = body::coping_learn_probability(
                stress_norm * 2000.0,
                allostatic_norm * 100.0,
                is_recovery,
                break_count,
                owned_count,
                COPING_COUNT_MAX,
            )
            .clamp(0.0, 1.0);
            if learn_p <= 0.0 {
                continue;
            }

            let learn_roll: f32 = resources.rng.gen_range(0.0..1.0);
            if learn_roll >= learn_p {
                continue;
            }

            let scores = coping_utility_scores(personality_opt, needs_opt, stress_norm, allostatic_norm);
            let strategy_roll: f32 = resources.rng.gen_range(0.0..1.0);
            let strategy_idx = body::coping_softmax_index(&scores, strategy_roll);
            let Some(strategy) = coping_strategy_from_index(strategy_idx) else {
                continue;
            };
            if coping.is_on_cooldown(strategy) {
                continue;
            }

            coping.active_strategy = Some(strategy);
            coping
                .usage_counts
                .entry(strategy)
                .and_modify(|count| *count = count.saturating_add(1))
                .or_insert(1);
            coping
                .strategy_cooldowns
                .insert(strategy, coping_strategy_cooldown_ticks(strategy));
        }
    }
}

/// Rust runtime system for child-stage stress processing.
///
/// This performs active writes on `Stress.level/reserve/allostatic_load` for
/// infant/toddler/child/teen entities using child-stress body kernels.
#[derive(Debug, Clone)]
pub struct ChildStressProcessorRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ChildStressProcessorRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn child_stage_code(stage: GrowthStage) -> Option<i32> {
    match stage {
        GrowthStage::Infant => Some(0),
        GrowthStage::Toddler => Some(1),
        GrowthStage::Child => Some(2),
        GrowthStage::Teen => Some(3),
        _ => None,
    }
}

#[inline]
fn child_stage_multipliers(stage_code: i32) -> (f32, f32, f32) {
    match stage_code {
        0 => (0.90, 1.20, 1.20),
        1 => (1.00, 1.10, 1.10),
        2 => (1.10, 1.00, 1.00),
        3 => (1.20, 1.05, 1.00),
        _ => (1.00, 1.00, 1.00),
    }
}

impl SimSystem for ChildStressProcessorRuntimeSystem {
    fn name(&self) -> &'static str {
        "child_stress_processor"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &Age,
            &mut Stress,
            Option<&Needs>,
            Option<&Social>,
            Option<&Personality>,
        )>();
        for (_, (age, stress, needs_opt, social_opt, personality_opt)) in &mut query {
            let Some(stage_code) = child_stage_code(age.stage) else {
                continue;
            };

            let belonging = needs_opt
                .map(|needs| needs.get(NeedType::Belonging) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let safety = needs_opt
                .map(|needs| needs.get(NeedType::Safety) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let energy = needs_opt
                .map(|needs| needs.energy as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let hunger = needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let base_intensity = (1.0 - (belonging * 0.35 + safety * 0.35 + energy * 0.20 + hunger * 0.10))
                .clamp(0.0, 1.0);
            if base_intensity <= 0.05 {
                continue;
            }

            let caregiver_present = social_opt
                .map(|social| !social.parents.is_empty() || social.spouse.is_some())
                .unwrap_or(false);
            let attachment_quality = social_opt
                .map(|social| social.reputation_local as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let buffered_intensity = body::child_social_buffered_intensity(
                base_intensity,
                attachment_quality,
                caregiver_present,
                0.5,
            )
            .clamp(0.0, 1.0);
            let stress_type = body::child_stress_type_code(
                buffered_intensity,
                caregiver_present,
                attachment_quality,
            );
            let (spike_mult, vulnerability_mult, break_threshold_mult) =
                child_stage_multipliers(stage_code);
            let resilience = personality_opt
                .map(|personality| personality.axis(HexacoAxis::C) as f32)
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            let out = body::child_stress_apply_step(
                resilience,
                (stress.reserve as f32 * 100.0).clamp(0.0, 100.0),
                (stress.level as f32 * 2000.0).clamp(0.0, 2000.0),
                (stress.allostatic_load as f32 * 100.0).clamp(0.0, 100.0),
                buffered_intensity,
                spike_mult,
                vulnerability_mult,
                break_threshold_mult,
                stress_type,
            );
            stress.reserve = (out[1] / 100.0).clamp(0.0, 1.0) as f64;
            stress.level = (out[2] / 2000.0).clamp(0.0, 1.0) as f64;
            stress.allostatic_load = (out[3] / 100.0).clamp(0.0, 1.0) as f64;
            stress.recalculate_state();
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
        AgeRuntimeSystem, ChildStressProcessorRuntimeSystem, ContagionRuntimeSystem,
        EconomicTendencyRuntimeSystem,
        CopingRuntimeSystem, EmotionRuntimeSystem, IntelligenceRuntimeSystem, MemoryRuntimeSystem,
        JobAssignmentRuntimeSystem, JobSatisfactionRuntimeSystem, MoraleRuntimeSystem,
        MentalBreakRuntimeSystem, MortalityRuntimeSystem, NeedsRuntimeSystem, NetworkRuntimeSystem,
        OccupationRuntimeSystem, ReputationRuntimeSystem, ResourceRegenSystem,
        SocialEventRuntimeSystem, StressRuntimeSystem, TraitViolationRuntimeSystem,
        TraumaScarRuntimeSystem, UpperNeedsRuntimeSystem, ValueRuntimeSystem,
    };
    use crate::body;
    use hecs::World;
    use sim_core::components::{
        Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Needs, Personality,
        Position, SkillEntry, Skills, Social, Stress, Traits, Values, Intelligence, Memory,
        MemoryEntry,
        TraumaScar,
    };
    use sim_core::ids::EntityId;
    use sim_core::world::TileResource;
    use sim_core::{
        config::GameConfig, ActionType, EmotionType, GameCalendar, GrowthStage, HexacoAxis,
        HexacoFacet, CopingStrategyId, IntelligenceType, MentalBreakType, NeedType, RelationType, ResourceType,
        SettlementId, Sex, ValueType, WorldMap,
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
    fn mental_break_runtime_system_triggers_break_and_sets_runtime_fields() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut stress = Stress {
            level: 0.95,
            reserve: 0.10,
            allostatic_load: 0.90,
            ..Stress::default()
        };
        stress.recalculate_state();
        let mut needs = Needs::default();
        needs.energy = 0.05;
        needs.set(NeedType::Hunger, 0.05);
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::C as usize] = 0.20;
        personality.axes[HexacoAxis::E as usize] = 0.85;
        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Anger) = 0.95;
        *emotion.get_mut(EmotionType::Fear) = 0.80;
        *emotion.get_mut(EmotionType::Sadness) = 0.70;
        *emotion.get_mut(EmotionType::Disgust) = 0.50;
        let entity = world.spawn((stress, needs, personality, emotion));

        let mut system = MentalBreakRuntimeSystem::new(35, 1);
        for _ in 0..200 {
            system.run(&mut world, &mut resources, 1);
            let updated = world.get::<&Stress>(entity).expect("stress should be queryable");
            if updated.active_mental_break.is_some() {
                break;
            }
        }

        let updated = world.get::<&Stress>(entity).expect("stress should be queryable");
        assert!(updated.active_mental_break.is_some());
        assert!(updated.mental_break_remaining > 0);
        assert!(updated.mental_break_count >= 1);
    }

    #[test]
    fn mental_break_runtime_system_clears_active_break_after_countdown() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut stress = Stress {
            level: 0.90,
            reserve: 0.20,
            allostatic_load: 0.60,
            active_mental_break: Some(MentalBreakType::Shutdown),
            mental_break_remaining: 1,
            mental_break_count: 1,
            ..Stress::default()
        };
        stress.recalculate_state();
        let entity = world.spawn((stress,));

        let mut system = MentalBreakRuntimeSystem::new(35, 1);
        system.run(&mut world, &mut resources, 1);

        let updated = world.get::<&Stress>(entity).expect("stress should be queryable");
        assert!(updated.active_mental_break.is_none());
        assert_eq!(updated.mental_break_remaining, 0);
        assert!(updated.level <= 0.90);
    }

    #[test]
    fn trait_violation_runtime_system_increases_stress_on_violation() {
        let mut world = World::new();
        let mut resources = make_resources();

        let stress = Stress {
            level: 0.12,
            reserve: 0.82,
            allostatic_load: 0.18,
            ..Stress::default()
        };
        let behavior = Behavior {
            current_action: ActionType::TakeFromStockpile,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.80);
        let social = Social {
            social_capital: 0.80,
            ..Social::default()
        };
        let mut traits = Traits::default();
        traits.add_trait("f_fair_minded".to_string());
        let mut personality = Personality::default();
        personality.facets[HexacoFacet::Fairness as usize] = 0.95;

        let entity = world.spawn((stress, behavior, needs, social, traits, personality));
        let mut system = TraitViolationRuntimeSystem::new(36, 30);
        system.run(&mut world, &mut resources, 30);

        let updated = world
            .get::<&Stress>(entity)
            .expect("updated stress should be queryable");
        assert!(updated.level > 0.12);
        assert!(updated.reserve < 0.82);
        assert!(updated.allostatic_load > 0.18);
    }

    #[test]
    fn trait_violation_runtime_system_ptsd_path_amplifies_repeat_delta() {
        let mut world = World::new();
        let mut resources = make_resources();

        let stress = Stress {
            level: 0.05,
            reserve: 0.90,
            allostatic_load: 0.90,
            ..Stress::default()
        };
        let behavior = Behavior {
            current_action: ActionType::TakeFromStockpile,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.75);
        let social = Social {
            social_capital: 0.75,
            ..Social::default()
        };
        let mut traits = Traits::default();
        traits.add_trait("f_fair_minded".to_string());
        let mut personality = Personality::default();
        personality.facets[HexacoFacet::Fairness as usize] = 1.0;

        let entity = world.spawn((stress, behavior, needs, social, traits, personality));
        let mut system = TraitViolationRuntimeSystem::new(36, 30);

        system.run(&mut world, &mut resources, 30);
        let after_first = world
            .get::<&Stress>(entity)
            .expect("stress after first run should be queryable")
            .level;

        system.run(&mut world, &mut resources, 60);
        let after_second = world
            .get::<&Stress>(entity)
            .expect("stress after second run should be queryable")
            .level;

        let first_delta = after_first - 0.05;
        let second_delta = after_second - after_first;
        assert!(first_delta > 0.0);
        assert!(second_delta > first_delta);
    }

    #[test]
    fn trauma_scar_runtime_system_applies_baseline_drift_from_scars() {
        let mut world = World::new();
        let mut resources = make_resources();

        let memory = Memory {
            trauma_scars: vec![TraumaScar {
                scar_id: "chronic_paranoia".to_string(),
                acquired_tick: 100,
                severity: 0.90,
                reactivation_count: 2,
            }],
            ..Memory::default()
        };
        let mut emotion = Emotion::default();
        emotion.baseline[EmotionType::Joy as usize] = 0.60;
        emotion.baseline[EmotionType::Fear as usize] = 0.20;
        emotion.baseline[EmotionType::Sadness as usize] = 0.20;
        let entity = world.spawn((memory, emotion));

        let mut system = TraumaScarRuntimeSystem::new(36, 10);
        system.run(&mut world, &mut resources, 10);

        let updated = world
            .get::<&Emotion>(entity)
            .expect("emotion should be queryable");
        assert!(updated.baseline[EmotionType::Joy as usize] < 0.60);
        assert!(updated.baseline[EmotionType::Fear as usize] > 0.20);
        assert!(updated.baseline[EmotionType::Sadness as usize] > 0.20);
    }

    #[test]
    fn trauma_scar_runtime_system_clamps_baseline_with_repeated_updates() {
        let mut world = World::new();
        let mut resources = make_resources();

        let memory = Memory {
            trauma_scars: vec![TraumaScar {
                scar_id: "emotional_numbness".to_string(),
                acquired_tick: 200,
                severity: 1.0,
                reactivation_count: 4,
            }],
            ..Memory::default()
        };
        let mut emotion = Emotion::default();
        emotion.baseline[EmotionType::Joy as usize] = 0.01;
        emotion.baseline[EmotionType::Trust as usize] = 0.01;
        emotion.baseline[EmotionType::Sadness as usize] = 0.99;
        let entity = world.spawn((memory, emotion));

        let mut system = TraumaScarRuntimeSystem::new(36, 10);
        for _ in 0..5000 {
            system.run(&mut world, &mut resources, 10);
        }

        let updated = world
            .get::<&Emotion>(entity)
            .expect("emotion should be queryable");
        for idx in 0..updated.baseline.len() {
            assert!((0.0..=1.0).contains(&updated.baseline[idx]));
        }
    }

    #[test]
    fn coping_runtime_system_decrements_strategy_cooldowns() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut coping = Coping::default();
        coping
            .strategy_cooldowns
            .insert(CopingStrategyId::Denial, 40);
        let entity = world.spawn((coping,));

        let mut system = CopingRuntimeSystem::new(42, 30);
        system.run(&mut world, &mut resources, 30);

        let updated = world.get::<&Coping>(entity).expect("coping should be queryable");
        let remaining = updated
            .strategy_cooldowns
            .get(&CopingStrategyId::Denial)
            .copied()
            .unwrap_or(0);
        assert_eq!(remaining, 10);
    }

    #[test]
    fn coping_runtime_system_selects_strategy_and_updates_usage() {
        let mut world = World::new();
        let mut resources = make_resources();

        let coping = Coping::default();
        let stress = Stress {
            level: 0.30,
            allostatic_load: 0.20,
            mental_break_count: 8,
            ..Stress::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::C as usize] = 0.85;
        personality.axes[HexacoAxis::O as usize] = 0.70;
        personality.axes[HexacoAxis::A as usize] = 0.65;
        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.55);
        needs.set(NeedType::Safety, 0.60);
        needs.energy = 0.70;
        let entity = world.spawn((coping, stress, personality, needs));

        let mut system = CopingRuntimeSystem::new(42, 30);
        for step in 0..50 {
            system.run(&mut world, &mut resources, step * 30);
        }

        let updated = world.get::<&Coping>(entity).expect("coping should be queryable");
        assert!(updated.active_strategy.is_some());
        let total_uses: u32 = updated.usage_counts.values().copied().sum();
        assert!(total_uses >= 1);
        if let Some(active) = updated.active_strategy {
            let cooldown = updated.strategy_cooldowns.get(&active).copied().unwrap_or(0);
            assert!(cooldown > 0);
        }
    }

    #[test]
    fn child_stress_processor_runtime_system_updates_child_stress_fields() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            stage: GrowthStage::Child,
            years: 10.0,
            ..Age::default()
        };
        let stress = Stress {
            level: 0.10,
            reserve: 0.90,
            allostatic_load: 0.10,
            ..Stress::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.05);
        needs.set(NeedType::Safety, 0.05);
        needs.set(NeedType::Hunger, 0.10);
        needs.energy = 0.05;
        let social = Social::default();
        let personality = Personality::default();
        let entity = world.spawn((age, stress, needs, social, personality));

        let mut system = ChildStressProcessorRuntimeSystem::new(32, 2);
        system.run(&mut world, &mut resources, 2);

        let updated = world
            .get::<&Stress>(entity)
            .expect("stress should be queryable");
        assert!(updated.level > 0.10);
        assert!(updated.reserve < 0.90);
        assert!(updated.allostatic_load >= 0.10);
    }

    #[test]
    fn child_stress_processor_runtime_system_skips_non_child_stages() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            stage: GrowthStage::Adult,
            years: 30.0,
            ..Age::default()
        };
        let stress = Stress {
            level: 0.25,
            reserve: 0.60,
            allostatic_load: 0.30,
            ..Stress::default()
        };
        let needs = Needs::default();
        let entity = world.spawn((age, stress, needs));

        let mut system = ChildStressProcessorRuntimeSystem::new(32, 2);
        system.run(&mut world, &mut resources, 2);

        let updated = world
            .get::<&Stress>(entity)
            .expect("stress should be queryable");
        assert!((updated.level as f32 - 0.25).abs() < 1e-6);
        assert!((updated.reserve as f32 - 0.60).abs() < 1e-6);
        assert!((updated.allostatic_load as f32 - 0.30).abs() < 1e-6);
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
    fn economic_tendency_runtime_system_updates_tendencies_and_applies_male_risk_bias() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut personality = Personality::default();
        personality.axes[HexacoAxis::H as usize] = 0.75;
        personality.axes[HexacoAxis::E as usize] = 0.35;
        personality.axes[HexacoAxis::X as usize] = 0.70;
        personality.axes[HexacoAxis::A as usize] = 0.65;
        personality.axes[HexacoAxis::C as usize] = 0.40;
        personality.axes[HexacoAxis::O as usize] = 0.60;

        let mut values = Values::default();
        values.set(ValueType::SelfControl, 0.30);
        values.set(ValueType::Law, 0.20);
        values.set(ValueType::Commerce, 0.45);
        values.set(ValueType::Competition, 0.35);
        values.set(ValueType::MartialProwess, 0.10);
        values.set(ValueType::Sacrifice, 0.25);
        values.set(ValueType::Cooperation, 0.30);
        values.set(ValueType::Family, 0.40);
        values.set(ValueType::Power, 0.22);
        values.set(ValueType::Fairness, 0.50);

        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.72);

        let age = Age {
            years: 28.0,
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let economic = Economic {
            wealth: 0.86,
            ..Economic::default()
        };

        let male_identity = Identity {
            sex: Sex::Male,
            ..Identity::default()
        };
        let male = world.spawn((
            economic.clone(),
            personality.clone(),
            values.clone(),
            needs.clone(),
            age,
            male_identity,
        ));

        let female_identity = Identity {
            sex: Sex::Female,
            ..Identity::default()
        };
        let female = world.spawn((
            economic,
            personality.clone(),
            values.clone(),
            needs.clone(),
            Age {
                years: 28.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            female_identity,
        ));

        let mut system = EconomicTendencyRuntimeSystem::new(39, sim_core::config::ECON_TICK_INTERVAL);
        system.run(&mut world, &mut resources, sim_core::config::ECON_TICK_INTERVAL);

        let male_updated = world
            .get::<&Economic>(male)
            .expect("male economic component should be queryable");
        let expected_male = body::economic_tendencies_step(
            personality.axis(HexacoAxis::H) as f32,
            personality.axis(HexacoAxis::E) as f32,
            personality.axis(HexacoAxis::X) as f32,
            personality.axis(HexacoAxis::A) as f32,
            personality.axis(HexacoAxis::C) as f32,
            personality.axis(HexacoAxis::O) as f32,
            28.0,
            values.get(ValueType::SelfControl) as f32,
            values.get(ValueType::Law) as f32,
            values.get(ValueType::Commerce) as f32,
            values.get(ValueType::Competition) as f32,
            values.get(ValueType::MartialProwess) as f32,
            values.get(ValueType::Sacrifice) as f32,
            values.get(ValueType::Cooperation) as f32,
            values.get(ValueType::Family) as f32,
            values.get(ValueType::Power) as f32,
            values.get(ValueType::Fairness) as f32,
            needs.get(NeedType::Belonging) as f32,
            0.86,
            0.0,
            0.0,
            true,
            sim_core::config::ECON_WEALTH_GENEROSITY_PENALTY as f32,
        );
        assert!((male_updated.saving_tendency as f32 - expected_male[0]).abs() < 1e-6);
        assert!((male_updated.risk_appetite as f32 - expected_male[1]).abs() < 1e-6);
        assert!((male_updated.generosity as f32 - expected_male[2]).abs() < 1e-6);
        assert!((male_updated.materialism as f32 - expected_male[3]).abs() < 1e-6);
        drop(male_updated);

        let female_updated = world
            .get::<&Economic>(female)
            .expect("female economic component should be queryable");
        assert!(female_updated.risk_appetite < expected_male[1] as f64);
    }

    #[test]
    fn economic_tendency_runtime_system_skips_child_stage() {
        let mut world = World::new();
        let mut resources = make_resources();

        let economic = Economic {
            saving_tendency: 0.5,
            risk_appetite: 0.5,
            generosity: 0.5,
            materialism: 0.3,
            ..Economic::default()
        };
        let personality = Personality::default();
        let values = Values::default();
        let needs = Needs::default();
        let age = Age {
            years: 8.0,
            stage: GrowthStage::Child,
            ..Age::default()
        };
        let entity = world.spawn((economic, personality, values, needs, age));

        let mut system = EconomicTendencyRuntimeSystem::new(39, sim_core::config::ECON_TICK_INTERVAL);
        system.run(&mut world, &mut resources, sim_core::config::ECON_TICK_INTERVAL);

        let updated = world
            .get::<&Economic>(entity)
            .expect("economic component should be queryable");
        assert!((updated.saving_tendency as f32 - 0.5).abs() < 1e-6);
        assert!((updated.risk_appetite as f32 - 0.5).abs() < 1e-6);
        assert!((updated.generosity as f32 - 0.5).abs() < 1e-6);
        assert!((updated.materialism as f32 - 0.3).abs() < 1e-6);
    }

    #[test]
    fn intelligence_runtime_system_applies_nutrition_penalty_in_critical_window() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.0);
        let age = Age {
            ticks: 100,
            years: 0.02,
            stage: GrowthStage::Infant,
            alive: true,
        };
        let intelligence = Intelligence {
            values: [0.80; 8],
            g_factor: 0.50,
            ace_penalty: 0.0,
            nutrition_penalty: 0.0,
        };
        let entity = world.spawn((intelligence, age, needs, Skills::default()));

        let mut system = IntelligenceRuntimeSystem::new(18, 50);
        system.run(&mut world, &mut resources, 50);

        let updated = world
            .get::<&Intelligence>(entity)
            .expect("intelligence should be queryable");
        assert!(updated.nutrition_penalty > 0.0);
        assert!(updated.values[IntelligenceType::Logical as usize] < 0.80);
    }

    #[test]
    fn intelligence_runtime_system_applies_ace_penalty_and_fluid_decline() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            ticks: (60.0 * sim_core::config::TICKS_PER_YEAR as f32) as u64,
            years: 60.0,
            stage: GrowthStage::Adult,
            alive: true,
        };
        let identity = Identity {
            birth_tick: 0,
            ..Identity::default()
        };
        let memory = Memory {
            trauma_scars: vec![
                TraumaScar {
                    scar_id: "scar_a".to_string(),
                    acquired_tick: 1000,
                    severity: 0.6,
                    reactivation_count: 0,
                },
                TraumaScar {
                    scar_id: "scar_b".to_string(),
                    acquired_tick: 2000,
                    severity: 0.7,
                    reactivation_count: 0,
                },
                TraumaScar {
                    scar_id: "scar_c".to_string(),
                    acquired_tick: 3000,
                    severity: 0.8,
                    reactivation_count: 0,
                },
            ],
            ..Memory::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::O as usize] = 0.80;
        let intelligence = Intelligence {
            values: [0.90; 8],
            g_factor: 0.50,
            ace_penalty: 0.0,
            nutrition_penalty: 0.0,
        };
        let entity = world.spawn((intelligence, age, identity, memory, personality));

        let mut system = IntelligenceRuntimeSystem::new(18, 50);
        system.run(&mut world, &mut resources, 50);

        let updated = world
            .get::<&Intelligence>(entity)
            .expect("intelligence should be queryable");
        assert!((updated.ace_penalty as f32 - sim_core::config::INTEL_ACE_PENALTY_MAJOR as f32).abs() < 1e-6);
        assert!(
            updated.values[IntelligenceType::Logical as usize]
                < updated.values[IntelligenceType::Linguistic as usize]
        );
        assert!(updated.g_factor > 0.50);
    }

    #[test]
    fn memory_runtime_system_decays_evicts_and_promotes_entries() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut memory = Memory::default();
        for idx in 0..101 {
            memory.short_term.push_back(MemoryEntry {
                event_type: format!("event_{idx}"),
                target_id: None,
                tick: 9_000 + idx as u64,
                intensity: 0.55,
                current_intensity: 0.55,
                is_permanent: false,
            });
        }
        memory.short_term.push_back(MemoryEntry {
            event_type: "proposal".to_string(),
            target_id: Some(7),
            tick: 9_999,
            intensity: 0.80,
            current_intensity: 0.80,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "forgettable".to_string(),
            target_id: None,
            tick: 10_000,
            intensity: 0.05,
            current_intensity: 0.02,
            is_permanent: false,
        });

        let entity = world.spawn((memory,));
        let mut system = MemoryRuntimeSystem::new(18, sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS * 3,
        );

        let updated = world
            .get::<&Memory>(entity)
            .expect("memory component should be queryable");
        assert_eq!(updated.short_term.len(), sim_core::config::MEMORY_WORKING_MAX);
        assert!(updated.short_term.iter().all(|entry| entry.event_type != "forgettable"));
        assert!(
            updated
                .short_term
                .iter()
                .any(|entry| entry.event_type == "proposal" && entry.is_permanent)
        );
        assert!(
            updated
                .permanent
                .iter()
                .any(|entry| entry.event_type == "proposal" && entry.tick == 9_999 && entry.is_permanent)
        );
    }

    #[test]
    fn memory_runtime_system_compresses_old_entries_into_summary() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut memory = Memory::default();
        memory.short_term.push_back(MemoryEntry {
            event_type: "deep_talk".to_string(),
            target_id: Some(42),
            tick: 10,
            intensity: 0.90,
            current_intensity: 0.90,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "deep_talk".to_string(),
            target_id: Some(42),
            tick: 20,
            intensity: 0.90,
            current_intensity: 0.90,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "deep_talk".to_string(),
            target_id: Some(42),
            tick: 30,
            intensity: 0.90,
            current_intensity: 0.90,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "casual_talk".to_string(),
            target_id: Some(7),
            tick: 9_000,
            intensity: 0.10,
            current_intensity: 0.10,
            is_permanent: false,
        });

        let entity = world.spawn((memory,));
        let mut system = MemoryRuntimeSystem::new(18, sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS);
        let tick = sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS * 3;
        system.run(&mut world, &mut resources, tick);

        let updated = world
            .get::<&Memory>(entity)
            .expect("memory component should be queryable");
        assert_eq!(updated.short_term.len(), 2);
        assert_eq!(updated.last_compression_tick, tick);
        assert!(updated.short_term.iter().any(|entry| entry.event_type == "casual_talk"));

        let summary = updated
            .short_term
            .iter()
            .find(|entry| entry.event_type == "deep_talk_summary")
            .expect("deep_talk summary entry should exist");
        assert_eq!(summary.target_id, Some(42));
        assert_eq!(summary.tick, 10);

        let decayed = body::memory_decay_intensity(0.90, sim_core::config::memory_decay_rate(0.90) as f32, 1.0);
        let expected_summary = body::memory_summary_intensity(decayed, 0.70);
        assert!((summary.current_intensity as f32 - expected_summary).abs() < 1e-5);
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
    fn job_assignment_runtime_system_assigns_jobs_with_age_rules() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).resources[0].amount = 300.0;

        let adult_age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let adult_behavior = Behavior {
            job: "none".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let adult = world.spawn((adult_age, adult_behavior));

        let teen_age = Age {
            stage: GrowthStage::Teen,
            ..Age::default()
        };
        let teen_behavior = Behavior {
            job: "none".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let teen = world.spawn((teen_age, teen_behavior));

        let child_age = Age {
            stage: GrowthStage::Child,
            ..Age::default()
        };
        let child_behavior = Behavior {
            job: "none".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let child = world.spawn((child_age, child_behavior));

        let infant_age = Age {
            stage: GrowthStage::Infant,
            ..Age::default()
        };
        let infant_behavior = Behavior {
            job: "builder".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let infant = world.spawn((infant_age, infant_behavior));

        let mut system = JobAssignmentRuntimeSystem::new(8, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL,
        );

        let adult_updated = world
            .get::<&Behavior>(adult)
            .expect("adult behavior should be queryable");
        assert_ne!(adult_updated.job, "none");
        assert!(matches!(
            adult_updated.job.as_str(),
            "gatherer" | "lumberjack" | "builder" | "miner"
        ));
        drop(adult_updated);

        let teen_updated = world
            .get::<&Behavior>(teen)
            .expect("teen behavior should be queryable");
        assert_eq!(teen_updated.job, "gatherer");
        drop(teen_updated);

        let child_updated = world
            .get::<&Behavior>(child)
            .expect("child behavior should be queryable");
        assert_eq!(child_updated.job, "gatherer");
        drop(child_updated);

        let infant_updated = world
            .get::<&Behavior>(infant)
            .expect("infant behavior should be queryable");
        assert_eq!(infant_updated.job, "none");
    }

    #[test]
    fn job_assignment_runtime_system_rebalances_one_idle_surplus_job() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).resources[0].amount = 300.0;

        for _ in 0..5 {
            let age = Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            };
            let behavior = Behavior {
                job: "builder".to_string(),
                occupation: "none".to_string(),
                current_action: ActionType::Idle,
                ..Behavior::default()
            };
            world.spawn((age, behavior));
        }
        for job in ["gatherer", "lumberjack", "miner"] {
            let age = Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            };
            let behavior = Behavior {
                job: job.to_string(),
                occupation: "none".to_string(),
                current_action: ActionType::Idle,
                ..Behavior::default()
            };
            world.spawn((age, behavior));
        }

        let count_jobs = |world_ref: &World, job_name: &str| -> usize {
            let mut total = 0_usize;
            let mut query = world_ref.query::<&Behavior>();
            for (_, behavior) in &mut query {
                if behavior.job == job_name {
                    total += 1;
                }
            }
            total
        };

        let before_builder = count_jobs(&world, "builder");
        let before_gatherer = count_jobs(&world, "gatherer");
        assert_eq!(before_builder, 5);
        assert_eq!(before_gatherer, 1);

        let mut system = JobAssignmentRuntimeSystem::new(8, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL,
        );

        let after_builder = count_jobs(&world, "builder");
        let after_gatherer = count_jobs(&world, "gatherer");
        assert_eq!(after_builder, 4);
        assert_eq!(after_gatherer, 2);
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
    fn mortality_runtime_system_marks_high_risk_entity_dead() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            ticks: sim_core::config::TICKS_PER_YEAR as u64,
            years: 1.0,
            stage: GrowthStage::Adult,
            alive: true,
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.0);
        let body = BodyComponent {
            dr_realized: 0,
            ..BodyComponent::default()
        };
        let stress = Stress {
            allostatic_load: 1.0,
            ..Stress::default()
        };
        let entity = world.spawn((age, needs, body, stress));

        let mut system = MortalityRuntimeSystem::new(49, 1);
        for _ in 0..100 {
            system.run(&mut world, &mut resources, 1);
            let current_age = world.get::<&Age>(entity).expect("age should be queryable");
            if !current_age.alive {
                break;
            }
        }

        let updated_age = world.get::<&Age>(entity).expect("age should be queryable");
        assert!(!updated_age.alive);
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
