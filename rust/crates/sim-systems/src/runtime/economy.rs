#![allow(unused_imports)]
// TODO(v3.1): REFACTOR - economy/building logic is still v2-style and should move toward material/tag recipes and the 2-layer building model.

use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence,
    Memory, MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits, Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, BuildingId, ChannelId, CopingStrategyId, EmitterRecord,
    EmotionType, EntityId, FalloffType, GrowthStage, HexacoAxis, HexacoFacet, IntelligenceType,
    MentalBreakType, NeedType, RelationType, ResourceType, SettlementId, Sex, SocialClass,
    TechState, ValueType,
};
use sim_engine::{ConstructionDiagnostics, SimResources, SimSystem};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

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
        for (_, (behavior, personality_opt, values_opt, needs_opt, skills_opt, age_opt)) in
            &mut query
        {
            if let Some(age) = age_opt {
                if matches!(
                    age.stage,
                    GrowthStage::Infant | GrowthStage::Toddler | GrowthStage::Child
                ) {
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

/// Rust runtime system for resource gathering from map tiles.
///
/// This performs active writes on tile resource deposits and settlement
/// stockpile aggregates, then emits `ResourceGathered`.
#[derive(Debug, Clone)]
pub struct GatheringRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl GatheringRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn gathering_target_resource(action: ActionType) -> Option<(ResourceType, &'static str)> {
    match action {
        ActionType::Forage | ActionType::GatherHerbs => Some((ResourceType::Food, "food")),
        ActionType::GatherWood => Some((ResourceType::Wood, "wood")),
        ActionType::GatherStone => Some((ResourceType::Stone, "stone")),
        _ => None,
    }
}

impl SimSystem for GatheringRuntimeSystem {
    fn name(&self) -> &'static str {
        "gathering_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let gather_amount = (config::GATHER_AMOUNT as f32).max(0.0);
        if gather_amount <= 0.0 {
            return;
        }

        let mut query = world.query::<(&Behavior, &Position, Option<&Age>, Option<&Identity>)>();
        for (entity, (behavior, position, age_opt, identity_opt)) in &mut query {
            if let Some(age) = age_opt {
                if matches!(age.stage, GrowthStage::Infant | GrowthStage::Toddler) {
                    continue;
                }
            }

            let Some((resource_type, resource_name)) =
                gathering_target_resource(behavior.current_action)
            else {
                continue;
            };
            let tile_x = position.tile_x();
            let tile_y = position.tile_y();
            if !resources.map.in_bounds(tile_x, tile_y) {
                continue;
            }

            let mut harvested = 0.0_f32;
            {
                let tile = resources.map.get_mut(tile_x as u32, tile_y as u32);
                for deposit in &mut tile.resources {
                    if deposit.resource_type != resource_type || deposit.amount <= 0.0 {
                        continue;
                    }
                    harvested = gather_amount.min(deposit.amount as f32);
                    if harvested <= 0.0 {
                        continue;
                    }
                    deposit.amount = ((deposit.amount as f32 - harvested).max(0.0)) as f64;
                    break;
                }
            }
            if harvested <= 0.0 {
                continue;
            }

            if let Some(settlement_id) = identity_opt.and_then(|identity| identity.settlement_id) {
                if let Some(settlement) = resources.settlements.get_mut(&settlement_id) {
                    let gathered_f64 = harvested as f64;
                    match resource_type {
                        ResourceType::Food => {
                            settlement.stockpile_food =
                                (settlement.stockpile_food + gathered_f64).max(0.0);
                        }
                        ResourceType::Wood => {
                            settlement.stockpile_wood =
                                (settlement.stockpile_wood + gathered_f64).max(0.0);
                        }
                        ResourceType::Stone => {
                            settlement.stockpile_stone =
                                (settlement.stockpile_stone + gathered_f64).max(0.0);
                        }
                    }
                }
            }

            resources
                .event_bus
                .emit(sim_engine::GameEvent::ResourceGathered {
                    entity_id: EntityId(entity.id() as u64),
                    resource: resource_name.to_string(),
                    amount: harvested as f64,
                });
        }
    }
}

const CONSTRUCTION_BUILD_TICKS_DEFAULT: i32 = 50;

#[inline]
fn construction_build_ticks(building_type: &str) -> i32 {
    match building_type {
        "stockpile" => 36,
        "shelter" => 60,
        "campfire" => 24,
        _ => CONSTRUCTION_BUILD_TICKS_DEFAULT,
    }
}

#[inline]
fn construction_skill_multiplier(skills_opt: Option<&Skills>) -> f32 {
    let level = skills_opt
        .map(|skills| skills.get_level("SKILL_CONSTRUCTION") as f32)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    1.0 + (level / 100.0) * 0.70
}

#[inline]
fn refresh_construction_diagnostics(
    resources: &mut SimResources,
    tick: u64,
    progress_before: &HashMap<BuildingId, f64>,
) {
    let snapshot_ids: HashSet<BuildingId> = resources.buildings.keys().copied().collect();
    resources
        .construction_diagnostics
        .retain(|building_id, _| snapshot_ids.contains(building_id));

    let progress_snapshots: Vec<(BuildingId, f64)> = resources
        .buildings
        .iter()
        .map(|(building_id, building)| (*building_id, f64::from(building.construction_progress)))
        .collect();

    for (building_id, progress) in progress_snapshots {
        let baseline_progress = progress_before
            .get(&building_id)
            .copied()
            .or_else(|| {
                resources
                    .construction_diagnostics
                    .get(&building_id)
                    .map(|entry| entry.last_observed_progress)
            })
            .unwrap_or(progress);
        let delta = progress - baseline_progress;
        match resources.construction_diagnostics.get_mut(&building_id) {
            Some(diagnostics) => {
                diagnostics.progress_delta = delta;
                diagnostics.last_observed_progress = progress;
                diagnostics.last_sample_tick = tick;
                if delta.abs() > f64::EPSILON {
                    diagnostics.last_progress_tick = tick;
                }
            }
            None => {
                resources.construction_diagnostics.insert(
                    building_id,
                    ConstructionDiagnostics {
                        last_observed_progress: progress,
                        progress_delta: delta,
                        last_progress_tick: if progress > 0.0 { tick } else { 0 },
                        last_sample_tick: tick,
                    },
                );
            }
        }
    }
}

/// Rust runtime system for construction progress updates.
///
/// This performs active writes on `Building.construction_progress` and
/// `Building.is_complete`, then emits `BuildingConstructed` on completion.
#[derive(Debug, Clone)]
pub struct ConstructionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ConstructionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for ConstructionRuntimeSystem {
    fn name(&self) -> &'static str {
        "construction_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let progress_before: HashMap<BuildingId, f64> = resources
            .buildings
            .iter()
            .map(|(building_id, building)| {
                (*building_id, f64::from(building.construction_progress))
            })
            .collect();
        let mut query = world.query::<(&Behavior, &Position, Option<&Age>, Option<&Skills>)>();
        for (_, (behavior, position, age_opt, skills_opt)) in &mut query {
            if behavior.current_action != ActionType::Build {
                continue;
            }
            let Some(age) = age_opt else {
                continue;
            };
            if age.stage != GrowthStage::Adult {
                continue;
            }

            let Some(target_x) = behavior.action_target_x else {
                continue;
            };
            let Some(target_y) = behavior.action_target_y else {
                continue;
            };

            let dx = (position.x - f64::from(target_x)).abs();
            let dy = (position.y - f64::from(target_y)).abs();
            if dx > 1.0 || dy > 1.0 {
                continue;
            }

            let mut target_building_id: Option<BuildingId> = None;
            for (building_id, building) in resources.buildings.iter() {
                if building.is_complete {
                    continue;
                }
                if building.x != target_x || building.y != target_y {
                    continue;
                }
                match target_building_id {
                    Some(current) if building_id.0 >= current.0 => {}
                    _ => target_building_id = Some(*building_id),
                }
            }
            let Some(building_id) = target_building_id else {
                continue;
            };

            let Some(building) = resources.buildings.get_mut(&building_id) else {
                continue;
            };
            let build_ticks = construction_build_ticks(building.building_type.as_str()).max(1);
            let mut ticks_per_cycle = build_ticks / config::CONSTRUCTION_TICK_INTERVAL as i32;
            if ticks_per_cycle < 1 {
                ticks_per_cycle = 1;
            }
            let progress_per_tick =
                (1.0 / ticks_per_cycle as f32) * construction_skill_multiplier(skills_opt);
            if progress_per_tick <= 0.0 {
                continue;
            }

            building.construction_progress =
                (building.construction_progress + progress_per_tick).min(1.0);
            if building.construction_progress < 1.0 || building.is_complete {
                continue;
            }

            building.construction_progress = 1.0;
            building.is_complete = true;
            resources
                .event_bus
                .emit(sim_engine::GameEvent::BuildingConstructed {
                    building_id,
                    building_type: building.building_type.clone(),
                });
        }
        refresh_construction_diagnostics(resources, tick, &progress_before);
    }
}

/// Rust runtime system for movement progression and arrival effects.
///
/// This performs active writes on `Position`, `Behavior`, and selected `Needs`
/// fields by applying movement skip policy, passable-tile step movement, and
/// action-complete restore effects.
#[derive(Debug, Clone, Copy)]
enum BuildingEffectKind {
    Campfire,
    Shelter,
}

#[derive(Debug, Clone, Copy)]
struct BuildingEffectSnapshot {
    kind: BuildingEffectKind,
    x: i32,
    y: i32,
}

const SHELTER_WALL_CARDINAL_OFFSETS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[inline]
fn refresh_campfire_warmth_emitter(resources: &mut SimResources, x: i32, y: i32) {
    if x < 0 || y < 0 {
        return;
    }
    let x_u32 = x as u32;
    let y_u32 = y as u32;
    if !resources.map.in_bounds(x, y) {
        return;
    }
    resources
        .influence_grid
        .remove_emitter(x_u32, y_u32, ChannelId::Warmth);
    resources.influence_grid.register_emitter(EmitterRecord {
        x: x_u32,
        y: y_u32,
        channel: ChannelId::Warmth,
        radius: f64::from(config::BUILDING_CAMPFIRE_RADIUS.max(1)),
        intensity: config::WARMTH_CAMPFIRE_EMITTER_INTENSITY,
        falloff: FalloffType::Linear,
        dirty: false,
    });
}

#[inline]
fn refresh_structure_wall_blocking(resources: &mut SimResources) {
    resources.influence_grid.clear_wall_blocking();

    let shelter_centers: Vec<(i32, i32)> = resources
        .buildings
        .values()
        .filter(|building| building.is_complete && building.building_type == "shelter")
        .map(|building| (building.x, building.y))
        .collect();

    for (center_x, center_y) in shelter_centers {
        apply_shelter_wall_blocking(resources, center_x, center_y);
    }
}

#[inline]
fn apply_shelter_wall_blocking(resources: &mut SimResources, center_x: i32, center_y: i32) {
    let wall_radius = config::BUILDING_SHELTER_WALL_RING_RADIUS.max(1);
    for (offset_x, offset_y) in SHELTER_WALL_CARDINAL_OFFSETS {
        if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
            && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y
        {
            continue;
        }

        let tile_x = center_x + offset_x * wall_radius;
        let tile_y = center_y + offset_y * wall_radius;
        if !resources.map.in_bounds(tile_x, tile_y) {
            continue;
        }

        resources.influence_grid.set_wall_blocking(
            tile_x as u32,
            tile_y as u32,
            config::BUILDING_SHELTER_WALL_BLOCK,
        );
    }
}

/// Rust runtime system for passive building aura effects.
///
/// This performs active writes on `Needs.belonging`, `Needs.warmth`,
/// `Needs.safety`, and `Needs.energy` based on nearby completed buildings.
#[derive(Debug, Clone)]
// TODO(v3.1): DELETE - replace building aura scans with room caches and Influence Grid propagation (A-6/A-2).
pub struct BuildingEffectRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl BuildingEffectRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for BuildingEffectRuntimeSystem {
    fn name(&self) -> &'static str {
        "building_effect_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        refresh_structure_wall_blocking(resources);
        if resources.buildings.is_empty() {
            return;
        }
        let mut effects: Vec<BuildingEffectSnapshot> = Vec::new();
        let mut campfire_positions: Vec<(i32, i32)> = Vec::new();
        for building in resources.buildings.values() {
            if !building.is_complete {
                continue;
            }
            let kind = match building.building_type.as_str() {
                "campfire" => BuildingEffectKind::Campfire,
                "shelter" => BuildingEffectKind::Shelter,
                _ => continue,
            };
            if matches!(kind, BuildingEffectKind::Campfire) {
                campfire_positions.push((building.x, building.y));
            }
            effects.push(BuildingEffectSnapshot {
                kind,
                x: building.x,
                y: building.y,
            });
        }
        if effects.is_empty() {
            return;
        }
        for (x, y) in campfire_positions {
            // Campfire warmth is now routed through the influence grid so
            // nearby agents must sample the Warmth channel on later ticks.
            refresh_campfire_warmth_emitter(resources, x, y);
        }

        let ticks_per_day = resources.calendar.ticks_per_day.max(1) as u64;
        let hour = ((resources.calendar.tick % ticks_per_day) * config::TICK_HOURS as u64) as i32;
        let is_night = !(6..20).contains(&hour);
        let campfire_social_boost = body::building_campfire_social_boost(is_night, 0.01, 0.02);

        let mut query = world.query::<(&Position, &mut Needs)>();
        for (_, (position, needs)) in &mut query {
            for effect in effects.iter().copied() {
                match effect.kind {
                    BuildingEffectKind::Campfire => {
                        let dx = (position.x - f64::from(effect.x)).abs();
                        let dy = (position.y - f64::from(effect.y)).abs();
                        if dx + dy > f64::from(config::BUILDING_CAMPFIRE_RADIUS) {
                            continue;
                        }
                        needs.set(
                            NeedType::Belonging,
                            needs.get(NeedType::Belonging) + campfire_social_boost as f64,
                        );
                    }
                    BuildingEffectKind::Shelter => {
                        let dx = (position.x - f64::from(effect.x)).abs();
                        let dy = (position.y - f64::from(effect.y)).abs();
                        if dx + dy > f64::from(config::BUILDING_SHELTER_RADIUS) {
                            continue;
                        }
                        needs.energy = (needs.energy + config::BUILDING_SHELTER_ENERGY_RESTORE)
                            .clamp(0.0, 1.0);
                        needs.set(
                            NeedType::Warmth,
                            needs.get(NeedType::Warmth) + config::WARMTH_SHELTER_RESTORE,
                        );
                        needs.set(
                            NeedType::Safety,
                            needs.get(NeedType::Safety) + config::SAFETY_SHELTER_RESTORE,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{Age, Behavior, Position, Skills};
    use sim_core::config::GameConfig;
    use sim_core::{
        ActionType, Building, BuildingId, GameCalendar, GrowthStage, SettlementId, WorldMap,
    };
    use sim_engine::SimResources;

    #[test]
    fn refresh_structure_wall_blocking_builds_shelter_ring_with_doorway_gap() {
        let game_config = GameConfig::default();
        let calendar = GameCalendar::new(&game_config);
        let mut resources = SimResources::new(calendar, WorldMap::new(12, 12, 7), 99);
        resources.buildings.insert(
            BuildingId(9),
            Building {
                id: BuildingId(9),
                building_type: "shelter".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        refresh_structure_wall_blocking(&mut resources);

        assert!(
            (resources.influence_grid.wall_blocking_at(6, 5) - config::BUILDING_SHELTER_WALL_BLOCK)
                .abs()
                < 1e-6
        );
        assert_eq!(resources.influence_grid.wall_blocking_at(5, 6), 0.0);
    }

    #[test]
    fn construction_runtime_system_records_recent_progress_delta() {
        let game_config = GameConfig::default();
        let calendar = GameCalendar::new(&game_config);
        let mut resources = SimResources::new(calendar, WorldMap::new(12, 12, 7), 99);
        resources.buildings.insert(
            BuildingId(3),
            Building {
                id: BuildingId(3),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 0.0,
                is_complete: false,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut world = World::new();
        world.spawn((
            Behavior {
                job: "builder".to_string(),
                current_action: ActionType::Build,
                action_target_x: Some(5),
                action_target_y: Some(5),
                ..Behavior::default()
            },
            Position {
                x: 5.0,
                y: 4.0,
                ..Position::default()
            },
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Skills::default(),
        ));

        let mut system = ConstructionRuntimeSystem::new(28, config::CONSTRUCTION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            config::CONSTRUCTION_TICK_INTERVAL,
        );

        let diagnostics = resources
            .construction_diagnostics
            .get(&BuildingId(3))
            .expect("construction diagnostics should exist");
        assert!(diagnostics.progress_delta > 0.0);
        assert_eq!(
            diagnostics.last_progress_tick,
            config::CONSTRUCTION_TICK_INTERVAL
        );
    }
}
