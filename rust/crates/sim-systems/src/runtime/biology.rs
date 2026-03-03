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


/// Rust runtime system for child hunger feeding from settlement stockpiles.
///
/// This performs active writes on `Needs.hunger` for child-stage entities and
/// decrements `Settlement.stockpile_food` according to childcare feed rules.
#[derive(Debug, Clone)]
pub struct ChildcareRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ChildcareRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn childcare_profile(stage: GrowthStage) -> Option<(f64, f64)> {
    match stage {
        GrowthStage::Infant => Some((
            config::CHILDCARE_HUNGER_THRESHOLD_INFANT,
            config::CHILDCARE_FEED_AMOUNT_INFANT,
        )),
        GrowthStage::Toddler => Some((
            config::CHILDCARE_HUNGER_THRESHOLD_TODDLER,
            config::CHILDCARE_FEED_AMOUNT_TODDLER,
        )),
        GrowthStage::Child => Some((
            config::CHILDCARE_HUNGER_THRESHOLD_CHILD,
            config::CHILDCARE_FEED_AMOUNT_CHILD,
        )),
        GrowthStage::Teen => Some((
            config::CHILDCARE_HUNGER_THRESHOLD_TEEN,
            config::CHILDCARE_FEED_AMOUNT_TEEN,
        )),
        _ => None,
    }
}

impl SimSystem for ChildcareRuntimeSystem {
    fn name(&self) -> &'static str {
        "childcare_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut candidates: Vec<(Entity, SettlementId, f64, f64)> = Vec::new();
        {
            let mut query = world.query::<(&Age, &Needs, &Identity)>();
            for (entity, (age, needs, identity)) in &mut query {
                let Some((threshold, feed_amount)) = childcare_profile(age.stage) else {
                    continue;
                };
                let Some(settlement_id) = identity.settlement_id else {
                    continue;
                };
                let hunger = needs.get(NeedType::Hunger);
                if hunger >= threshold {
                    continue;
                }
                candidates.push((entity, settlement_id, hunger, feed_amount));
            }
        }
        candidates.sort_by(|left, right| {
            left.2
                .partial_cmp(&right.2)
                .unwrap_or(Ordering::Equal)
        });

        for (entity, settlement_id, _hunger, feed_amount) in candidates {
            let Some(settlement) = resources.settlements.get_mut(&settlement_id) else {
                continue;
            };
            let available = settlement.stockpile_food.max(0.0);
            if available <= 0.0 {
                continue;
            }
            let withdrawn = body::childcare_take_food(available as f32, feed_amount as f32) as f64;
            if withdrawn <= 0.0 {
                continue;
            }
            settlement.stockpile_food = (available - withdrawn).max(0.0);

            if let Ok(mut needs) = world.get::<&mut Needs>(entity) {
                let next = body::childcare_hunger_after(
                    needs.get(NeedType::Hunger) as f32,
                    withdrawn as f32,
                    config::FOOD_HUNGER_RESTORE as f32,
                ) as f64;
                needs.set(NeedType::Hunger, next);
            }
        }
    }
}

const POPULATION_MIN_FOR_BIRTH: i32 = 5;
const POPULATION_FREE_HOUSING_CAP: i32 = 25;
const POPULATION_SHELTER_CAPACITY: i32 = 6;
const POPULATION_FOOD_PER_ALIVE: f32 = 0.5;

/// Rust runtime system for population growth births.
///
/// This performs active writes on:
/// - `SimResources.settlements[*].stockpile_food`
/// - `SimResources.settlements[*].members`
/// - ECS world via spawning a newborn entity
#[derive(Debug, Clone)]
pub struct PopulationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl PopulationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for PopulationRuntimeSystem {
    fn name(&self) -> &'static str {
        "population_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        if resources.settlements.is_empty() {
            return;
        }

        let mut alive_count: i32 = 0;
        {
            let mut query = world.query::<&Age>();
            for (_, age) in &mut query {
                if age.alive {
                    alive_count += 1;
                }
            }
        }

        let total_shelters: i32 = resources
            .buildings
            .values()
            .filter(|building| building.building_type == "shelter")
            .count() as i32;

        let total_food: f32 = resources
            .settlements
            .values()
            .map(|settlement| settlement.stockpile_food.max(0.0) as f32)
            .sum();

        let block_code = body::population_birth_block_code(
            alive_count,
            config::MAX_ENTITIES as i32,
            total_shelters,
            total_food,
            POPULATION_MIN_FOR_BIRTH,
            POPULATION_FREE_HOUSING_CAP,
            POPULATION_SHELTER_CAPACITY,
            POPULATION_FOOD_PER_ALIVE,
        );
        if block_code != 0 {
            return;
        }

        let mut selected_settlement_id: Option<SettlementId> = None;
        let mut selected_x: i32 = 0;
        let mut selected_y: i32 = 0;
        let mut best_food: f64 = -1.0;
        for settlement in resources.settlements.values() {
            if settlement.stockpile_food < config::BIRTH_FOOD_COST {
                continue;
            }
            if settlement.stockpile_food > best_food {
                best_food = settlement.stockpile_food;
                selected_settlement_id = Some(settlement.id);
                selected_x = settlement.x;
                selected_y = settlement.y;
            }
        }
        let Some(settlement_id) = selected_settlement_id else {
            return;
        };

        if let Some(settlement) = resources.settlements.get_mut(&settlement_id) {
            settlement.stockpile_food =
                (settlement.stockpile_food - config::BIRTH_FOOD_COST).max(0.0);
        }

        let age = Age {
            ticks: 0,
            years: 0.0,
            stage: GrowthStage::Infant,
            alive: true,
        };

        let identity = Identity {
            birth_tick: tick,
            settlement_id: Some(settlement_id),
            growth_stage: GrowthStage::Infant,
            sex: if resources.rng.gen_bool(0.5) {
                Sex::Male
            } else {
                Sex::Female
            },
            ..Identity::default()
        };

        let behavior = Behavior {
            current_action: ActionType::Idle,
            ..Behavior::default()
        };

        let entity = world.spawn((
            age,
            identity,
            behavior,
            Needs::default(),
            Emotion::default(),
            Stress::default(),
            Social::default(),
            Position::new(selected_x, selected_y),
        ));
        let entity_id = EntityId(entity.id() as u64);

        if let Ok(mut one) = world.query_one::<&mut Identity>(entity) {
            if let Some(identity_mut) = one.get() {
                identity_mut.name = format!("child_{}", entity.id());
            }
        }

        if let Some(settlement) = resources.settlements.get_mut(&settlement_id) {
            if !settlement.members.contains(&entity_id) {
                settlement.members.push(entity_id);
            }
        }

        resources
            .event_bus
            .emit(sim_engine::GameEvent::EntitySpawned { entity_id });
    }
}

const INTERGEN_MEANEY_THRESHOLD: f32 = 0.70;
const INTERGEN_MEANEY_REPAIR_RATE: f32 = 0.002;
const INTERGEN_MIN_LOAD: f32 = 0.05;
const INTERGEN_HPA_LOAD_WEIGHT: f32 = 0.60;

#[derive(Debug, Clone, Copy)]
struct IntergenParentProfile {
    load: f32,
    scar_index: f32,
    sex: Sex,
}

#[inline]
fn intergen_scar_index(memory_opt: Option<&Memory>) -> f32 {
    let Some(memory) = memory_opt else {
        return 0.0;
    };
    (memory.trauma_scars.len() as f32 / 5.0).clamp(0.0, 1.0)
}

#[inline]
fn intergen_parenting_quality(needs_opt: Option<&Needs>, stress: &Stress) -> f32 {
    let belonging = needs_opt
        .map(|needs| needs.get(NeedType::Belonging) as f32)
        .unwrap_or(0.5);
    let intimacy = needs_opt
        .map(|needs| needs.get(NeedType::Intimacy) as f32)
        .unwrap_or(0.5);
    let safety = needs_opt
        .map(|needs| needs.get(NeedType::Safety) as f32)
        .unwrap_or(0.5);
    let calm_factor = (1.0 - stress.level as f32).clamp(0.0, 1.0);
    (belonging * 0.35 + intimacy * 0.25 + safety * 0.20 + calm_factor * 0.20).clamp(0.0, 1.0)
}

/// Rust runtime system for intergenerational epigenetic transfer dynamics.
///
/// This performs active writes on `Stress.allostatic_load` and `Stress.level`
/// for parents (Meaney-style repair) and child-stage entities (transmission).
#[derive(Debug, Clone)]
pub struct IntergenerationalRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl IntergenerationalRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for IntergenerationalRuntimeSystem {
    fn name(&self) -> &'static str {
        "intergenerational_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut parent_profiles: HashMap<EntityId, IntergenParentProfile> = HashMap::new();
        {
            let mut query = world.query::<(&Stress, Option<&Memory>, Option<&Identity>)>();
            for (entity, (stress, memory_opt, identity_opt)) in &mut query {
                parent_profiles.insert(
                    EntityId(entity.id() as u64),
                    IntergenParentProfile {
                        load: (stress.allostatic_load as f32).clamp(0.0, 1.0),
                        scar_index: intergen_scar_index(memory_opt),
                        sex: identity_opt.map(|identity| identity.sex).unwrap_or(Sex::Male),
                    },
                );
            }
        }

        let mut parent_updates: Vec<(Entity, f32, f32, f32)> = Vec::new();
        {
            let mut query = world.query::<(&Age, &Social, &Stress, Option<&Needs>)>();
            for (entity, (age, social, stress, needs_opt)) in &mut query {
                if !age.alive {
                    continue;
                }
                if !matches!(age.stage, GrowthStage::Adult | GrowthStage::Elder) {
                    continue;
                }
                if social.children.is_empty() {
                    continue;
                }
                let current_load = (stress.allostatic_load as f32).clamp(0.0, 1.0);
                let parenting_quality = intergen_parenting_quality(needs_opt, stress);
                let next_load = body::intergen_meaney_repair_load(
                    current_load,
                    parenting_quality,
                    INTERGEN_MEANEY_THRESHOLD,
                    INTERGEN_MEANEY_REPAIR_RATE,
                    INTERGEN_MIN_LOAD,
                )
                .clamp(0.0, 1.0);
                if (next_load - current_load).abs() < 1e-6 {
                    continue;
                }
                let hpa_sensitivity = body::intergen_hpa_sensitivity(next_load, INTERGEN_HPA_LOAD_WEIGHT);
                let next_level = ((stress.level as f32) / hpa_sensitivity.max(0.001)).clamp(0.0, 1.0);
                parent_updates.push((entity, next_load, next_level, stress.level as f32));
                if let Some(profile) = parent_profiles.get_mut(&EntityId(entity.id() as u64)) {
                    profile.load = next_load;
                }
            }
        }

        for (entity, next_load, next_level, previous_level) in parent_updates {
            if let Ok(mut stress) = world.get::<&mut Stress>(entity) {
                stress.allostatic_load = next_load as f64;
                stress.level = next_level as f64;
                stress.recalculate_state();
                if (next_level - previous_level).abs() >= 1e-6 {
                    resources.event_bus.emit(sim_engine::GameEvent::StressChanged {
                        entity_id: EntityId(entity.id() as u64),
                        stress: next_level as f64,
                    });
                }
            }
        }

        let mut child_updates: Vec<(Entity, f32, f32, f32)> = Vec::new();
        {
            let mut query = world.query::<(&Age, &Social, &Stress, Option<&Needs>)>();
            for (entity, (age, social, stress, needs_opt)) in &mut query {
                if !age.alive || !age.stage.is_child_age() || social.parents.is_empty() {
                    continue;
                }
                let mut mother_profile: Option<IntergenParentProfile> = None;
                let mut father_profile: Option<IntergenParentProfile> = None;
                for parent_id in &social.parents {
                    let Some(profile) = parent_profiles.get(parent_id).copied() else {
                        continue;
                    };
                    match profile.sex {
                        Sex::Female if mother_profile.is_none() => mother_profile = Some(profile),
                        Sex::Male if father_profile.is_none() => father_profile = Some(profile),
                        _ => {}
                    }
                    if mother_profile.is_some() && father_profile.is_some() {
                        break;
                    }
                }
                let Some(mother) = mother_profile.or(father_profile) else {
                    continue;
                };
                let father = father_profile.or(mother_profile).unwrap_or(mother);

                let hunger = needs_opt
                    .map(|needs| needs.get(NeedType::Hunger) as f32)
                    .unwrap_or(0.5)
                    .clamp(0.0, 1.0);
                let adversity = (stress.level as f32).clamp(0.0, 1.0);
                let malnutrition = (1.0 - hunger).clamp(0.0, 1.0);
                let inputs = [
                    mother.load,
                    mother.load,
                    mother.scar_index,
                    0.50,
                    0.30,
                    0.20,
                    father.load,
                    father.load,
                    father.scar_index,
                    0.60,
                    0.25,
                    0.15,
                    0.30,
                    0.40,
                    0.10,
                    adversity,
                    mother.load,
                    malnutrition,
                    0.25,
                    0.10,
                    0.35,
                    INTERGEN_MIN_LOAD,
                    0.65,
                    0.35,
                ];
                let out = body::intergen_child_epigenetic_step(&inputs);
                let inherited_load = out[0].clamp(0.0, 1.0);
                let current_load = (stress.allostatic_load as f32).clamp(0.0, 1.0);
                let next_load = (current_load * 0.4 + inherited_load * 0.6).clamp(0.0, 1.0);
                let hpa_sensitivity = body::intergen_hpa_sensitivity(next_load, INTERGEN_HPA_LOAD_WEIGHT);
                let next_level = ((stress.level as f32) * hpa_sensitivity).clamp(0.0, 1.0);
                if (next_load - current_load).abs() < 1e-6 && (next_level - stress.level as f32).abs() < 1e-6 {
                    continue;
                }
                child_updates.push((entity, next_load, next_level, stress.level as f32));
            }
        }

        for (entity, next_load, next_level, previous_level) in child_updates {
            if let Ok(mut stress) = world.get::<&mut Stress>(entity) {
                stress.allostatic_load = next_load as f64;
                stress.level = next_level as f64;
                stress.recalculate_state();
                if (next_level - previous_level).abs() >= 1e-6 {
                    resources.event_bus.emit(sim_engine::GameEvent::StressChanged {
                        entity_id: EntityId(entity.id() as u64),
                        stress: next_level as f64,
                    });
                }
            }
        }
    }
}

const PARENTING_BASE_RATE: f32 = 0.002;
const PARENTING_MALADAPTIVE_MULT: f32 = 1.5;
const PARENTING_STRESS_DELTA_SCALE: f32 = 8.0;

/// Rust runtime system for parenting transition and observational coping updates.
///
/// This performs active writes on parent/child `Stress` and child `Coping`
/// based on parent regulation signals and Bandura-style modeling rates.
#[derive(Debug, Clone)]
pub struct ParentingRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl ParentingRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for ParentingRuntimeSystem {
    fn name(&self) -> &'static str {
        "parenting_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut parent_signal_by_entity: HashMap<EntityId, f32> = HashMap::new();
        let mut parent_updates: Vec<(Entity, f32, f32)> = Vec::new();
        {
            let mut query = world.query::<(&Age, &Social, &Stress)>();
            for (entity, (age, social, stress)) in &mut query {
                if !age.alive {
                    continue;
                }
                if !matches!(age.stage, GrowthStage::Adult | GrowthStage::Elder) {
                    continue;
                }
                if social.children.is_empty() {
                    continue;
                }

                let current_level = (stress.level as f32).clamp(0.0, 1.0);
                let epigenetic_load = (stress.allostatic_load as f32).clamp(0.0, 1.0);
                let adjusted_gain =
                    body::parenting_hpa_adjusted_stress_gain(1.0, epigenetic_load, INTERGEN_HPA_LOAD_WEIGHT)
                        .max(0.001);
                let next_level = (current_level / adjusted_gain).clamp(0.0, 1.0);
                parent_updates.push((entity, next_level, current_level));
                parent_signal_by_entity.insert(EntityId(entity.id() as u64), (1.0 - next_level).clamp(0.0, 1.0));
            }
        }

        for (entity, next_level, previous_level) in parent_updates {
            if (next_level - previous_level).abs() < 1e-6 {
                continue;
            }
            if let Ok(mut stress) = world.get::<&mut Stress>(entity) {
                stress.level = next_level as f64;
                stress.recalculate_state();
                resources.event_bus.emit(sim_engine::GameEvent::StressChanged {
                    entity_id: EntityId(entity.id() as u64),
                    stress: next_level as f64,
                });
            }
        }

        let mut query = world.query::<(&Age, &Social, &mut Coping, &mut Stress)>();
        for (entity, (age, social, coping, stress)) in &mut query {
            if !age.alive || !age.stage.is_child_age() || social.parents.is_empty() {
                continue;
            }

            let mut observation_sum = 0.0_f32;
            let mut observation_count = 0_u32;
            for parent_id in &social.parents {
                let Some(signal) = parent_signal_by_entity.get(parent_id).copied() else {
                    continue;
                };
                observation_sum += signal;
                observation_count += 1;
            }
            if observation_count == 0 {
                continue;
            }
            let observation_strength = (observation_sum / observation_count as f32).clamp(0.0, 1.0);
            let adaptive_rate = body::parenting_bandura_base_rate(
                PARENTING_BASE_RATE,
                1.0,
                observation_strength,
                false,
                PARENTING_MALADAPTIVE_MULT,
            );
            let maladaptive_rate = body::parenting_bandura_base_rate(
                PARENTING_BASE_RATE,
                1.0,
                1.0 - observation_strength,
                true,
                PARENTING_MALADAPTIVE_MULT,
            );

            let (next_strategy, stress_delta) = if adaptive_rate >= maladaptive_rate {
                (
                    CopingStrategyId::ProblemSolving,
                    -adaptive_rate * PARENTING_STRESS_DELTA_SCALE,
                )
            } else {
                (
                    CopingStrategyId::Denial,
                    maladaptive_rate * PARENTING_STRESS_DELTA_SCALE,
                )
            };

            coping.active_strategy = Some(next_strategy);
            let usage = coping.usage_counts.entry(next_strategy).or_insert(0);
            *usage = usage.saturating_add(1);

            let previous_level = stress.level as f32;
            let next_level = (previous_level + stress_delta).clamp(0.0, 1.0);
            let next_allostatic =
                ((stress.allostatic_load as f32) + stress_delta * 0.5).clamp(0.0, 1.0);
            if (next_level - previous_level).abs() < 1e-6 && (next_allostatic - stress.allostatic_load as f32).abs() < 1e-6 {
                continue;
            }
            stress.level = next_level as f64;
            stress.allostatic_load = next_allostatic as f64;
            stress.recalculate_state();
            resources.event_bus.emit(sim_engine::GameEvent::StressChanged {
                entity_id: EntityId(entity.id() as u64),
                stress: next_level as f64,
            });
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

// ── Personality Generator ────────────────────────────────────────────

/// Default HEXACO correlation matrix (Ashton & Lee 2004)
const HEXACO_CORRELATION: [[f32; 6]; 6] = [
    [1.00, 0.12, -0.11, 0.26, 0.18, 0.21],
    [0.12, 1.00, -0.13, -0.08, 0.15, -0.10],
    [-0.11, -0.13, 1.00, 0.05, 0.10, 0.08],
    [0.26, -0.08, 0.05, 1.00, 0.01, 0.03],
    [0.18, 0.15, 0.10, 0.01, 1.00, 0.03],
    [0.21, -0.10, 0.08, 0.03, 0.03, 1.00],
];

/// Heritability per axis
const HEXACO_HERITABILITY: [f32; 6] = [0.45, 0.58, 0.57, 0.47, 0.52, 0.63];

/// Sex difference Cohen's d per axis
const HEXACO_SEX_DIFF_D: [f32; 6] = [0.41, 0.96, 0.10, 0.28, 0.00, -0.04];

/// Intra-axis facet spread (z-score units)
const FACET_SPREAD: f32 = 0.75;

/// Cholesky decomposition of a 6x6 matrix (computed at init time)
fn cholesky_6x6(r: &[[f32; 6]; 6]) -> [[f32; 6]; 6] {
    let mut l = [[0.0_f32; 6]; 6];
    for i in 0..6 {
        for j in 0..=i {
            let mut sum = 0.0_f32;
            for k in 0..j {
                sum += l[i][k] * l[j][k];
            }
            if i == j {
                l[i][j] = (r[i][i] - sum).max(0.0).sqrt();
            } else if l[j][j].abs() > 1e-12 {
                l[i][j] = (r[i][j] - sum) / l[j][j];
            }
        }
    }
    l
}

/// Convert facet value (0..1) to z-score
#[inline]
fn bio_facet_to_zscore(v: f64) -> f32 {
    ((v - 0.5) * 4.0) as f32
}

/// Convert z-score to facet value (0..1)
#[inline]
fn bio_zscore_to_facet(z: f32) -> f64 {
    (z as f64 / 4.0 + 0.5).clamp(0.0, 1.0)
}

/// Rust runtime system for Cholesky-based HEXACO personality generation.
///
/// Ports `personality_generator.gd`: generates a new personality for newborn
/// entities using correlated sampling, parental inheritance, sex differences,
/// and culture shifts. Runs each tick and processes entities that need
/// personality initialization (facets all at default 0.5).
#[derive(Debug, Clone)]
pub struct PersonalityGeneratorRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    cholesky_l: [[f32; 6]; 6],
}

impl PersonalityGeneratorRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            cholesky_l: cholesky_6x6(&HEXACO_CORRELATION),
        }
    }
}

impl SimSystem for PersonalityGeneratorRuntimeSystem {
    fn name(&self) -> &'static str {
        "personality_generator_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        use sim_core::components::personality::AXIS_COUNT;

        // Identify entities needing personality init: facets all at 0.5 (default)
        let mut needs_init: Vec<(Entity, Sex, Option<EntityId>, Option<EntityId>)> = Vec::new();
        {
            let mut query = world.query::<(&Identity, &Personality, Option<&Social>)>();
            for (entity, (identity, personality, social_opt)) in &mut query {
                // Check if personality is at default (all facets ~0.5)
                let is_default = personality.facets.iter().all(|f| (*f - 0.5).abs() < 1e-6);
                if !is_default {
                    continue;
                }
                let (parent_a, parent_b) = social_opt
                    .map(|social| {
                        let pa = social.parents.first().copied();
                        let pb = social.parents.get(1).copied();
                        (pa, pb)
                    })
                    .unwrap_or((None, None));
                needs_init.push((entity, identity.sex, parent_a, parent_b));
            }
        }

        if needs_init.is_empty() {
            return;
        }

        // Snapshot parent personality z-scores (parent EntityId → axis z-scores)
        let mut parent_z: HashMap<EntityId, [f32; 6]> = HashMap::new();
        {
            // Collect which parent EntityIds we need
            let needed: HashSet<EntityId> = needs_init
                .iter()
                .flat_map(|(_, _, pa, pb)| pa.iter().chain(pb.iter()).copied())
                .collect();
            if !needed.is_empty() {
                let mut query = world.query::<&Personality>();
                for (entity, personality) in &mut query {
                    let eid = EntityId(entity.id() as u64);
                    if needed.contains(&eid) {
                        let mut zs = [0.0_f32; 6];
                        for i in 0..6 {
                            zs[i] = bio_facet_to_zscore(personality.axes[i]);
                        }
                        parent_z.insert(eid, zs);
                    }
                }
            }
        }

        // Generate personalities
        for (entity, sex, parent_a_eid, parent_b_eid) in &needs_init {
            let is_female = *sex == Sex::Female;
            let has_parents = parent_a_eid.is_some() && parent_b_eid.is_some();
            let z_pa = parent_a_eid
                .and_then(|eid| parent_z.get(&eid))
                .copied()
                .unwrap_or([0.0; 6]);
            let z_pb = parent_b_eid
                .and_then(|eid| parent_z.get(&eid))
                .copied()
                .unwrap_or([0.0; 6]);

            // Step 1: Sample 6 independent N(0,1) values
            let mut z_indep = [0.0_f32; 6];
            for i in 0..6 {
                let u1 = resources.rng.gen_range(0.0001_f32..1.0_f32);
                let u2 = resources.rng.gen_range(0.0_f32..1.0_f32);
                z_indep[i] = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
            }

            // Apply Cholesky to get correlated z-scores
            let mut z_random = [0.0_f32; 6];
            for i in 0..6 {
                let mut val = 0.0_f32;
                for j in 0..=i {
                    val += self.cholesky_l[i][j] * z_indep[j];
                }
                z_random[i] = val;
            }

            // Step 2: Per-axis child z-score using body kernel
            let mut z_axes = [0.0_f32; 6];
            for i in 0..AXIS_COUNT {
                z_axes[i] = body::personality_child_axis_z(
                    has_parents,
                    z_pa[i],
                    z_pb[i],
                    HEXACO_HERITABILITY[i],
                    z_random[i],
                    is_female,
                    HEXACO_SEX_DIFF_D[i],
                    0.0, // culture_shift (default)
                );
            }

            // Step 3: Distribute axis z-score to 4 facets with intra-axis variation
            let mut new_facets = [0.5_f64; 24];
            for axis_idx in 0..AXIS_COUNT {
                for f in 0..4 {
                    let u1 = resources.rng.gen_range(0.0001_f32..1.0_f32);
                    let u2 = resources.rng.gen_range(0.0_f32..1.0_f32);
                    let noise = FACET_SPREAD
                        * (-2.0 * u1.ln()).sqrt()
                        * (2.0 * std::f32::consts::PI * u2).cos();
                    let facet_z = z_axes[axis_idx] + noise;
                    new_facets[axis_idx * 4 + f] = bio_zscore_to_facet(facet_z);
                }
            }

            // Apply to entity
            if let Ok(mut personality) = world.get::<&mut Personality>(*entity) {
                personality.facets = new_facets;
                personality.recalculate_axes();
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// AttachmentRuntimeSystem — Ainsworth (1978) / Bowlby (1969)
// ═══════════════════════════════════════════════════════════════════════

/// Attachment classification thresholds (from GDScript GameConfig defaults).
const ATTACHMENT_SENSITIVITY_SECURE: f32 = 0.65;
const ATTACHMENT_CONSISTENCY_SECURE: f32 = 0.70;
const ATTACHMENT_SENSITIVITY_ANXIOUS: f32 = 0.45;
const ATTACHMENT_CONSISTENCY_DISORG: f32 = 0.30;
const ATTACHMENT_ABUSER_ACE_MIN: f32 = 4.0;
const ATTACHMENT_AVOIDANT_SENS_MAX: f32 = 0.35;
const ATTACHMENT_AVOIDANT_CONS_MIN: f32 = 0.50;

/// Age in years at which attachment type is determined (~1 year).
const ATTACHMENT_DETERMINATION_AGE: f64 = 1.0;

/// Determines attachment type for infants reaching ~1 year of age.
///
/// Ainsworth Strange Situation paradigm: caregiver sensitivity and consistency
/// (derived from parent personality A-axis and stress level) determine the
/// child's attachment pattern (Secure, Anxious, Avoidant, Fearful/Disorganized).
#[derive(Debug, Clone)]
pub struct AttachmentRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl AttachmentRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for AttachmentRuntimeSystem {
    fn name(&self) -> &'static str {
        "attachment_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        // Pass 1: Build EntityId→Entity map and collect candidate infants
        let mut id_map: HashMap<EntityId, Entity> = HashMap::new();
        let mut candidates: Vec<(Entity, Vec<EntityId>)> = Vec::new();

        {
            let mut query = world.query::<(&Age, &Social)>();
            for (entity, (age, social)) in &mut query {
                let eid = EntityId(entity.id() as u64);
                id_map.insert(eid, entity);

                // Candidate: age ~1 year, no attachment type yet
                if age.years == ATTACHMENT_DETERMINATION_AGE
                    && social.attachment_type.is_none()
                    && !social.parents.is_empty()
                {
                    candidates.push((entity, social.parents.clone()));
                }
            }
        }

        if candidates.is_empty() {
            return;
        }

        // Pass 2: Snapshot parent data and compute attachment type
        struct AttachmentResult {
            entity: Entity,
            attachment: AttachmentType,
        }

        let mut results: Vec<AttachmentResult> = Vec::new();

        for (entity, parent_ids) in &candidates {
            let mut total_sensitivity = 0.0_f32;
            let mut total_consistency = 0.0_f32;
            let mut parent_count = 0_u32;

            for pid in parent_ids {
                if let Some(&parent_entity) = id_map.get(pid) {
                    // Read parent personality A-axis for sensitivity
                    let a_axis = world
                        .get::<&Personality>(parent_entity)
                        .map(|p| p.axes[HexacoAxis::A as usize] as f32)
                        .unwrap_or(0.5);

                    // Read parent stress level for consistency (low stress = high consistency)
                    let stress_level = world
                        .get::<&Stress>(parent_entity)
                        .map(|s| s.level as f32)
                        .unwrap_or(0.0);

                    total_sensitivity += a_axis;
                    total_consistency += 1.0 - stress_level;
                    parent_count += 1;
                }
            }

            if parent_count == 0 {
                // No resolvable parents — default to Secure
                results.push(AttachmentResult {
                    entity: *entity,
                    attachment: AttachmentType::Secure,
                });
                continue;
            }

            let avg_sensitivity = total_sensitivity / parent_count as f32;
            let avg_consistency = total_consistency / parent_count as f32;

            // Child's ACE score (may be 0 for infants)
            let child_ace = world
                .get::<&Stress>(*entity)
                .map(|s| s.ace_score as f32 * 10.0) // denormalize to native 0..10
                .unwrap_or(0.0);

            let code = body::attachment_type_code(
                avg_sensitivity,
                avg_consistency,
                child_ace,
                false, // abuser_is_caregiver — simplified for initial port
                ATTACHMENT_SENSITIVITY_SECURE,
                ATTACHMENT_CONSISTENCY_SECURE,
                ATTACHMENT_SENSITIVITY_ANXIOUS,
                ATTACHMENT_CONSISTENCY_DISORG,
                ATTACHMENT_ABUSER_ACE_MIN,
                ATTACHMENT_AVOIDANT_SENS_MAX,
                ATTACHMENT_AVOIDANT_CONS_MIN,
            );

            let attachment = match code {
                0 => AttachmentType::Secure,
                1 => AttachmentType::Anxious,
                2 => AttachmentType::Avoidant,
                3 => AttachmentType::Fearful,
                _ => AttachmentType::Anxious, // fallback
            };

            results.push(AttachmentResult {
                entity: *entity,
                attachment,
            });
        }

        // Pass 3: Apply mutations
        for result in results {
            if let Ok(mut social) = world.get::<&mut Social>(result.entity) {
                social.attachment_type = Some(result.attachment);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// AceTrackerRuntimeSystem — Felitti et al. (1998) ACE Study
// ═══════════════════════════════════════════════════════════════════════

/// Minimum age (years) for ACE backfill (adults only).
const ACE_BACKFILL_MIN_AGE: f64 = 18.0;

/// ACE native scale max (Felitti 0-10).
const ACE_NATIVE_MAX: f64 = 10.0;

/// Backfills ACE (Adverse Childhood Experiences) scores for adult entities
/// that lack childhood history data.
///
/// Uses allostatic load, trauma scar count, and attachment type as proxies
/// for childhood adversity (Felitti dose-response model).
#[derive(Debug, Clone)]
pub struct AceTrackerRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl AceTrackerRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for AceTrackerRuntimeSystem {
    fn name(&self) -> &'static str {
        "ace_tracker_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        // Pass 1: Collect backfill targets — adults without ACE score
        struct AceTarget {
            entity: Entity,
            allostatic: f32,
            trauma_count: i32,
            attachment_code: i32,
        }

        let mut targets: Vec<AceTarget> = Vec::new();

        {
            let mut query = world.query::<(&Age, &Stress, &Social, &Memory)>();
            for (entity, (age, stress, social, memory)) in &mut query {
                if age.years < ACE_BACKFILL_MIN_AGE {
                    continue;
                }
                // Skip if already backfilled
                if stress.ace_backfilled {
                    continue;
                }

                let allostatic = stress.allostatic_load as f32 * 100.0; // to native 0..100
                let trauma_count = memory.trauma_scars.len() as i32;
                let attachment_code = match social.attachment_type {
                    Some(AttachmentType::Secure) => 0,
                    Some(AttachmentType::Anxious) => 1,
                    Some(AttachmentType::Avoidant) => 2,
                    Some(AttachmentType::Fearful) => 3,
                    None => 0, // default secure if unknown
                };

                targets.push(AceTarget {
                    entity,
                    allostatic,
                    trauma_count,
                    attachment_code,
                });
            }
        }

        // Pass 2: Compute and apply ACE scores
        for target in targets {
            let raw_ace = body::ace_backfill_score(
                target.allostatic,
                target.trauma_count,
                target.attachment_code,
            );
            // Normalize to 0..1 (native scale is 0..10)
            let normalized = (raw_ace as f64 / ACE_NATIVE_MAX).clamp(0.0, 1.0);

            if let Ok(mut stress) = world.get::<&mut Stress>(target.entity) {
                stress.ace_score = normalized;
                stress.ace_backfilled = true;
            }
        }
    }
}
