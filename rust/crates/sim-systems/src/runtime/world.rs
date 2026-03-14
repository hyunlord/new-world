#![allow(unused_imports)]
// TODO(v3.1): REFACTOR - move world/tech/migration tuning into World Rules compile/runtime data.
// TODO(v3.1): REFACTOR - replace direct settlement mutation paths with explicit World Rules and event-driven transitions.

use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{
    Age, Behavior, Body as BodyComponent, Coping, Economic, Emotion, Identity, Intelligence,
    Inventory, Memory, MemoryEntry, Needs, Personality, Position, Skills, Social, Stress, Traits,
    Values,
};
use sim_core::config;
use sim_core::{
    ActionType, AttachmentType, BuildingId, CopingStrategyId, EmotionType, EntityId, GrowthStage,
    HexacoAxis, HexacoFacet, IntelligenceType, ItemDerivedStats, ItemInstance, ItemOwner,
    MentalBreakType, NeedType, RelationType, ResourceType, SettlementId, Sex, SocialClass,
    TechState, ValueType,
};
use sim_engine::{SimEvent, SimEventType, SimResources, SimSystem};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::crafting;
use crate::body;

#[inline]
fn tension_pair_key(left: SettlementId, right: SettlementId) -> String {
    let (min_id, max_id) = if left.0 <= right.0 {
        (left.0, right.0)
    } else {
        (right.0, left.0)
    };
    format!("{min_id}:{max_id}")
}

#[inline]
fn tension_food_scarce(stockpile_food: f64, population: usize) -> bool {
    if population == 0 {
        return false;
    }
    let monthly_need =
        population as f32 * config::HUNGER_DECAY_RATE as f32 * config::TICKS_PER_DAY as f32 * 30.0;
    let ratio = (stockpile_food as f32) / monthly_need.max(1.0);
    ratio < config::TENSION_RESOURCE_DEFICIT_TRIGGER as f32
}

#[inline]
fn is_food_template(template: &str) -> bool {
    matches!(
        template,
        "raw_meat" | "berries" | "raw_fish" | "cooked_meat" | "dried_meat"
    )
}

#[inline]
fn maybe_grant_forage_berries(
    entity_id: EntityId,
    inventory: &mut Option<&mut Inventory>,
    resources: &mut SimResources,
    tick: u64,
) {
    let Some(inventory) = inventory.as_deref_mut() else {
        return;
    };
    let inventory_cap = inventory.max_tool_slots as usize + config::FORAGE_FOOD_BUFFER_SLOTS;
    if inventory.count() >= inventory_cap {
        return;
    }
    if resources.rng.gen_range(0.0..1.0) >= config::FORAGE_BERRIES_DROP_CHANCE {
        return;
    }

    let item_id = resources.item_store.allocate_id();
    resources.item_store.insert(ItemInstance {
        id: item_id,
        template_id: "berries".to_string(),
        material_id: "plant".to_string(),
        derived_stats: ItemDerivedStats::default(),
        current_durability: 100.0,
        quality: 0.5,
        owner: ItemOwner::Agent(entity_id),
        stack_count: 1,
        created_tick: tick,
        creator_id: Some(entity_id),
        equipped_slot: None,
    });
    inventory.add(item_id);
}

#[derive(Debug, Clone, Copy)]
struct TensionSettlementSnapshot {
    id: SettlementId,
    x: i32,
    y: i32,
    stockpile_food: f64,
    population: usize,
}

/// Rust runtime system for inter-settlement scarcity tension.
///
/// This performs active writes on `SimResources.tension_pairs`
/// using settlement distance, food scarcity pressure, and natural decay.
#[derive(Debug, Clone)]
pub struct TensionRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TensionRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TensionRuntimeSystem {
    fn name(&self) -> &'static str {
        "tension_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        if resources.settlements.len() < 2 {
            return;
        }
        let mut settlements: Vec<TensionSettlementSnapshot> =
            Vec::with_capacity(resources.settlements.len());
        for settlement in resources.settlements.values() {
            settlements.push(TensionSettlementSnapshot {
                id: settlement.id,
                x: settlement.x,
                y: settlement.y,
                stockpile_food: settlement.stockpile_food,
                population: settlement.population(),
            });
        }

        let proximity_radius = config::TENSION_PROXIMITY_RADIUS as f32;
        let dt_years = self.tick_interval as f32 / config::TICKS_PER_YEAR as f32;
        for left_idx in 0..settlements.len() {
            let left = settlements[left_idx];
            for right in settlements.iter().skip(left_idx + 1).copied() {
                let dx = (left.x - right.x) as f32;
                let dy = (left.y - right.y) as f32;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance > proximity_radius {
                    continue;
                }

                let left_scarce = tension_food_scarce(left.stockpile_food, left.population);
                let right_scarce = tension_food_scarce(right.stockpile_food, right.population);
                let scarcity_pressure = body::tension_scarcity_pressure(
                    left_scarce,
                    right_scarce,
                    config::TENSION_PER_SHARED_RESOURCE as f32,
                );

                let pair_key = tension_pair_key(left.id, right.id);
                let current = resources
                    .tension_pairs
                    .get(pair_key.as_str())
                    .copied()
                    .unwrap_or(0.0);
                let next = body::tension_next_value(
                    current as f32,
                    scarcity_pressure,
                    config::TENSION_DECAY_PER_YEAR as f32,
                    dt_years,
                )
                .clamp(0.0, 1.0);
                resources.tension_pairs.insert(pair_key, next as f64);
            }
        }
    }
}

/// Rust runtime system for technology utilization era updates.
///
/// This performs active writes on `Settlement.current_era` using known-tech counts.
#[derive(Debug, Clone)]
pub struct TechUtilizationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TechUtilizationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TechUtilizationRuntimeSystem {
    fn name(&self) -> &'static str {
        "tech_utilization_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        settlement_ids.sort_by_key(|settlement_id| settlement_id.0);

        for settlement_id in settlement_ids {
            let Some(settlement) = resources.settlements.get_mut(&settlement_id) else {
                continue;
            };
            let known_count = settlement
                .tech_states
                .values()
                .filter(|state| matches!(state, TechState::KnownLow | TechState::KnownStable))
                .count() as u32;

            let next_era = if known_count >= config::TECH_ERA_BRONZE_AGE_COUNT {
                "bronze_age"
            } else if known_count >= config::TECH_ERA_TRIBAL_COUNT {
                "tribal"
            } else {
                "stone_age"
            };

            if settlement.current_era == next_era {
                continue;
            }
            settlement.current_era = next_era.to_string();
            resources
                .event_bus
                .emit(sim_engine::GameEvent::EraAdvanced {
                    settlement_id,
                    new_era: settlement.current_era.clone(),
                });
        }
    }
}

const TECH_MAINT_MIN_PRACTITIONERS: usize = 3;
const TECH_MAINT_RECOVERY_POP: usize = 6;
const TECH_MAINT_LONG_RECOVERY_POP: usize = 9;

/// Rust runtime system for technology maintenance state transitions.
///
/// This performs active writes on `Settlement.tech_states` and emits
/// rediscovery events when forgotten tech becomes known again.
#[derive(Debug, Clone)]
pub struct TechMaintenanceRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TechMaintenanceRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TechMaintenanceRuntimeSystem {
    fn name(&self) -> &'static str {
        "tech_maintenance_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        let mut settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        settlement_ids.sort_by_key(|settlement_id| settlement_id.0);

        for settlement_id in settlement_ids {
            let population = resources
                .settlements
                .get(&settlement_id)
                .map(|settlement| settlement.members.len())
                .unwrap_or(0);
            let mut rediscovered: Vec<String> = Vec::new();

            {
                let Some(settlement) = resources.settlements.get_mut(&settlement_id) else {
                    continue;
                };
                let mut tech_ids: Vec<String> = settlement.tech_states.keys().cloned().collect();
                tech_ids.sort();

                for tech_id in tech_ids {
                    let Some(current_state) = settlement.tech_states.get(&tech_id).copied() else {
                        continue;
                    };
                    let next_state = match current_state {
                        TechState::KnownStable => {
                            if population < TECH_MAINT_MIN_PRACTITIONERS {
                                TechState::KnownLow
                            } else {
                                current_state
                            }
                        }
                        TechState::KnownLow => {
                            if population < (TECH_MAINT_MIN_PRACTITIONERS / 2).max(1) {
                                TechState::ForgottenRecent
                            } else if population >= TECH_MAINT_RECOVERY_POP {
                                TechState::KnownStable
                            } else {
                                current_state
                            }
                        }
                        TechState::ForgottenRecent => {
                            if population >= TECH_MAINT_RECOVERY_POP {
                                rediscovered.push(tech_id.clone());
                                TechState::KnownLow
                            } else if population == 0 {
                                TechState::ForgottenLong
                            } else {
                                current_state
                            }
                        }
                        TechState::ForgottenLong => {
                            if population >= TECH_MAINT_LONG_RECOVERY_POP {
                                rediscovered.push(tech_id.clone());
                                TechState::KnownLow
                            } else {
                                current_state
                            }
                        }
                        TechState::Unknown => current_state,
                    };

                    if next_state != current_state {
                        settlement.tech_states.insert(tech_id, next_state);
                    }
                }
            }

            for tech_id in rediscovered {
                resources
                    .event_bus
                    .emit(sim_engine::GameEvent::TechDiscovered {
                        settlement_id,
                        tech_id,
                    });
            }
        }
    }
}

const TECH_DISCOVERY_BASE_CHANCE: f32 = 0.02;
const TECH_DISCOVERY_MIN_POP: usize = 2;
const TECH_DISCOVERY_FORCE_POP: usize = 180;

/// Rust runtime system for technology discovery progression.
///
/// This performs active writes on `Settlement.tech_states` and emits
/// `TechDiscovered` for newly discovered or rediscovered tech entries.
#[derive(Debug, Clone)]
pub struct TechDiscoveryRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TechDiscoveryRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TechDiscoveryRuntimeSystem {
    fn name(&self) -> &'static str {
        "tech_discovery_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        let checks_per_year = (config::TICKS_PER_YEAR as f32 / self.tick_interval as f32).max(1.0);
        let mut settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        settlement_ids.sort_by_key(|settlement_id| settlement_id.0);

        for settlement_id in settlement_ids {
            let population = resources
                .settlements
                .get(&settlement_id)
                .map(|settlement| settlement.members.len())
                .unwrap_or(0);
            if population < TECH_DISCOVERY_MIN_POP {
                continue;
            }

            let mut discovered: Option<String> = None;
            {
                let Some(settlement) = resources.settlements.get_mut(&settlement_id) else {
                    continue;
                };
                let mut candidate_ids: Vec<String> = settlement
                    .tech_states
                    .iter()
                    .filter(|(_, state)| {
                        matches!(
                            state,
                            TechState::Unknown
                                | TechState::ForgottenRecent
                                | TechState::ForgottenLong
                        )
                    })
                    .map(|(tech_id, _)| tech_id.clone())
                    .collect();
                candidate_ids.sort();

                for tech_id in candidate_ids {
                    let pop_bonus = ((population as i32 - 2).max(0) as f32)
                        * config::TECH_DISCOVERY_POP_SCALE as f32;
                    let prob = body::tech_discovery_prob(
                        TECH_DISCOVERY_BASE_CHANCE,
                        pop_bonus,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        config::TECH_DISCOVERY_MAX_BONUS as f32,
                        checks_per_year,
                    )
                    .clamp(0.0, 1.0);
                    let should_discover = if population >= TECH_DISCOVERY_FORCE_POP {
                        true
                    } else {
                        resources.rng.gen_range(0.0..1.0) < prob
                    };
                    if !should_discover {
                        continue;
                    }
                    settlement
                        .tech_states
                        .insert(tech_id.clone(), TechState::KnownLow);
                    discovered = Some(tech_id);
                    break;
                }
            }

            if let Some(tech_id) = discovered {
                resources
                    .event_bus
                    .emit(sim_engine::GameEvent::TechDiscovered {
                        settlement_id,
                        tech_id,
                    });
            }
        }
    }
}

const TECH_PROP_LANG_PENALTY: f32 = 1.0;
const TECH_PROP_MAX_PROB: f32 = 0.95;
const TECH_PROP_CULTURE_KNOWLEDGE_WEIGHT: f32 = 0.3;
const TECH_PROP_CULTURE_TRADITION_WEIGHT: f32 = 0.4;
const TECH_PROP_CULTURE_MIN: f32 = 0.1;
const TECH_PROP_CULTURE_MAX: f32 = 2.0;
const TECH_PROP_CARRIER_SKILL_DIVISOR: f32 = 20.0;
const TECH_PROP_CARRIER_WEIGHT: f32 = 0.5;
const TECH_PROP_STABILITY_BONUS_STABLE: f32 = 1.3;
const TECH_PROP_STABILITY_BONUS_LOW: f32 = 0.7;

#[derive(Debug, Clone, Copy, Default)]
struct TechPropagationProfile {
    knowledge_sum: f32,
    tradition_sum: f32,
    member_count: u32,
    max_skill: i32,
}

impl TechPropagationProfile {
    fn record_member(&mut self, knowledge: f32, tradition: f32, skill_level: i32) {
        self.knowledge_sum += knowledge;
        self.tradition_sum += tradition;
        self.member_count += 1;
        self.max_skill = self.max_skill.max(skill_level);
    }

    fn knowledge_avg(&self) -> f32 {
        if self.member_count == 0 {
            0.0
        } else {
            self.knowledge_sum / self.member_count as f32
        }
    }

    fn tradition_avg(&self) -> f32 {
        if self.member_count == 0 {
            0.0
        } else {
            self.tradition_sum / self.member_count as f32
        }
    }
}

/// Rust runtime system for cross-settlement technology propagation.
///
/// This performs active writes on `Settlement.tech_states` by importing unknown
/// or forgotten tech entries from other settlements that already know the tech.
#[derive(Debug, Clone)]
pub struct TechPropagationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TechPropagationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for TechPropagationRuntimeSystem {
    fn name(&self) -> &'static str {
        "tech_propagation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        if resources.settlements.len() < 2 {
            return;
        }

        let mut profiles: HashMap<SettlementId, TechPropagationProfile> = HashMap::new();
        {
            let mut query =
                world.query::<(&Identity, Option<&Values>, Option<&Skills>, Option<&Age>)>();
            for (_, (identity, values_opt, skills_opt, age_opt)) in &mut query {
                if let Some(age) = age_opt {
                    if !age.alive {
                        continue;
                    }
                }
                let Some(settlement_id) = identity.settlement_id else {
                    continue;
                };
                let knowledge = values_opt
                    .map(|values| values.get(ValueType::Knowledge) as f32)
                    .unwrap_or(0.0);
                let tradition = values_opt
                    .map(|values| values.get(ValueType::Tradition) as f32)
                    .unwrap_or(0.0);
                let best_skill_level = skills_opt
                    .map(|skills| skills.best_skill_level() as i32)
                    .unwrap_or(0);

                profiles.entry(settlement_id).or_default().record_member(
                    knowledge,
                    tradition,
                    best_skill_level,
                );
            }
        }

        let mut settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        settlement_ids.sort_by_key(|settlement_id| settlement_id.0);

        for target_id in settlement_ids.iter().copied() {
            let candidate_tech_ids: Vec<String> = {
                let Some(target_settlement) = resources.settlements.get(&target_id) else {
                    continue;
                };
                if target_settlement.members.is_empty() {
                    continue;
                }
                let mut ids: Vec<String> = target_settlement
                    .tech_states
                    .iter()
                    .filter(|(_, state)| {
                        matches!(
                            state,
                            TechState::Unknown
                                | TechState::ForgottenRecent
                                | TechState::ForgottenLong
                        )
                    })
                    .map(|(tech_id, _)| tech_id.clone())
                    .collect();
                ids.sort();
                ids
            };
            if candidate_tech_ids.is_empty() {
                continue;
            }

            let target_profile = profiles.get(&target_id).copied().unwrap_or_default();
            let culture_mod = body::tech_propagation_culture_modifier(
                target_profile.knowledge_avg(),
                target_profile.tradition_avg(),
                TECH_PROP_CULTURE_KNOWLEDGE_WEIGHT,
                TECH_PROP_CULTURE_TRADITION_WEIGHT,
                TECH_PROP_CULTURE_MIN,
                TECH_PROP_CULTURE_MAX,
            );

            for tech_id in candidate_tech_ids {
                let mut source_pick: Option<(SettlementId, TechState, i32)> = None;
                for source_id in settlement_ids.iter().copied() {
                    if source_id == target_id {
                        continue;
                    }
                    let Some(source_settlement) = resources.settlements.get(&source_id) else {
                        continue;
                    };
                    let Some(source_state) = source_settlement.tech_states.get(&tech_id).copied()
                    else {
                        continue;
                    };
                    if !matches!(source_state, TechState::KnownLow | TechState::KnownStable) {
                        continue;
                    }

                    let source_skill = profiles
                        .get(&source_id)
                        .map(|profile| profile.max_skill)
                        .unwrap_or(0);
                    let source_score = source_settlement.members.len() as i32 + source_skill;
                    match source_pick {
                        Some((_, _, best_score)) if source_score <= best_score => {}
                        _ => source_pick = Some((source_id, source_state, source_score)),
                    }
                }

                let Some((source_id, source_state, _)) = source_pick else {
                    continue;
                };
                let source_skill = profiles
                    .get(&source_id)
                    .map(|profile| profile.max_skill)
                    .unwrap_or(0);
                let carrier_bonus = body::tech_propagation_carrier_bonus(
                    source_skill,
                    TECH_PROP_CARRIER_SKILL_DIVISOR,
                    TECH_PROP_CARRIER_WEIGHT,
                );
                let stability_bonus = if matches!(source_state, TechState::KnownStable) {
                    TECH_PROP_STABILITY_BONUS_STABLE
                } else {
                    TECH_PROP_STABILITY_BONUS_LOW
                };
                let base_prob = if matches!(source_state, TechState::KnownStable) {
                    config::CROSS_PROP_MIGRATION_BASE as f32
                } else {
                    config::CROSS_PROP_TRADE_BASE as f32
                };
                let final_prob = body::tech_propagation_final_prob(
                    base_prob,
                    TECH_PROP_LANG_PENALTY,
                    culture_mod,
                    carrier_bonus,
                    stability_bonus,
                    TECH_PROP_MAX_PROB,
                )
                .clamp(0.0, TECH_PROP_MAX_PROB);
                let should_import = if final_prob >= TECH_PROP_MAX_PROB {
                    true
                } else {
                    resources.rng.gen_range(0.0..1.0) < final_prob
                };
                if !should_import {
                    continue;
                }

                if let Some(target_settlement) = resources.settlements.get_mut(&target_id) {
                    target_settlement
                        .tech_states
                        .insert(tech_id.clone(), TechState::KnownLow);
                }
                resources
                    .event_bus
                    .emit(sim_engine::GameEvent::TechDiscovered {
                        settlement_id: target_id,
                        tech_id,
                    });
                break;
            }
        }
    }
}

#[inline]
fn migration_count_shelters(resources: &SimResources, settlement_id: SettlementId) -> usize {
    resources
        .buildings
        .values()
        .filter(|building| {
            building.is_complete
                && building.settlement_id == settlement_id
                && building.building_type == "shelter"
        })
        .count()
}

#[inline]
fn migration_food_in_radius(resources: &SimResources, cx: i32, cy: i32, radius: i32) -> f32 {
    let mut total_food = 0.0_f32;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx.abs() + dy.abs() > radius {
                continue;
            }
            let x = cx + dx;
            let y = cy + dy;
            if !resources.map.in_bounds(x, y) {
                continue;
            }
            let tile = resources.map.get(x as u32, y as u32);
            for deposit in &tile.resources {
                if deposit.resource_type == ResourceType::Food {
                    total_food += deposit.amount as f32;
                }
            }
        }
    }
    total_food
}

fn migration_find_site(
    resources: &mut SimResources,
    source_x: i32,
    source_y: i32,
) -> Option<(i32, i32)> {
    let min_radius = config::MIGRATION_SEARCH_RADIUS_MIN;
    let max_radius = config::MIGRATION_SEARCH_RADIUS_MAX;
    let min_settlement_distance = config::SETTLEMENT_MIN_DISTANCE;
    let settlement_positions: Vec<(i32, i32)> =
        resources.settlements.values().map(|s| (s.x, s.y)).collect();

    for _ in 0..20 {
        let dx = resources.rng.gen_range(-max_radius..=max_radius);
        let dy = resources.rng.gen_range(-max_radius..=max_radius);
        let manhattan = dx.abs() + dy.abs();
        if manhattan < min_radius || manhattan > max_radius {
            continue;
        }
        let x = source_x + dx;
        let y = source_y + dy;
        if !resources.map.in_bounds(x, y) {
            continue;
        }
        if !resources.map.get(x as u32, y as u32).passable {
            continue;
        }
        let mut far_enough = true;
        for (settlement_x, settlement_y) in settlement_positions.iter().copied() {
            let distance = (settlement_x - x).abs() + (settlement_y - y).abs();
            if distance < min_settlement_distance {
                far_enough = false;
                break;
            }
        }
        if !far_enough {
            continue;
        }
        let food_score = migration_food_in_radius(resources, x, y, 5);
        if food_score <= 3.0 {
            continue;
        }
        return Some((x, y));
    }
    None
}

/// Rust runtime system for settlement migration and founding.
///
/// This performs active writes on `SimResources.settlements`,
/// `Identity.settlement_id`, and `Behavior` migration fields.
#[derive(Debug, Clone)]
pub struct MigrationRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MigrationRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for MigrationRuntimeSystem {
    fn name(&self) -> &'static str {
        "migration_system"
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
        if resources.settlements.len() as u32 >= config::MAX_SETTLEMENTS {
            return;
        }

        for settlement in resources.settlements.values_mut() {
            if settlement.migration_cooldown > 0 {
                settlement.migration_cooldown = settlement
                    .migration_cooldown
                    .saturating_sub(self.tick_interval as u32);
            }
        }

        let mut settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        settlement_ids.sort_by_key(|settlement_id| settlement_id.0);

        for source_id in settlement_ids {
            if resources.settlements.len() as u32 >= config::MAX_SETTLEMENTS {
                break;
            }
            let Some(source_snapshot) = resources.settlements.get(&source_id) else {
                continue;
            };
            let source_x = source_snapshot.x;
            let source_y = source_snapshot.y;
            let source_population = source_snapshot.population();
            let source_food_stockpile = source_snapshot.stockpile_food;
            let source_cooldown = source_snapshot.migration_cooldown;
            if source_population < config::MIGRATION_MIN_POP as usize {
                continue;
            }
            if source_cooldown > 0 {
                continue;
            }

            let shelter_count = migration_count_shelters(resources, source_id);
            let overcrowded = source_population > shelter_count.saturating_mul(8);
            let nearby_food = migration_food_in_radius(resources, source_x, source_y, 20);
            let food_scarce =
                body::migration_food_scarce(nearby_food, source_population as i32, 0.3);
            let chance_roll: f32 = resources.rng.gen_range(0.0..1.0);
            let should_attempt = body::migration_should_attempt(
                overcrowded,
                food_scarce,
                chance_roll,
                config::MIGRATION_CHANCE as f32,
            );
            if !should_attempt {
                continue;
            }
            if source_food_stockpile < config::MIGRATION_STARTUP_FOOD {
                continue;
            }

            let mut candidates: Vec<Entity> = Vec::new();
            {
                let mut query = world.query::<(&Identity, &Age)>();
                for (entity, (identity, age)) in &mut query {
                    if !age.alive {
                        continue;
                    }
                    if identity.settlement_id != Some(source_id) {
                        continue;
                    }
                    candidates.push(entity);
                }
            }
            if candidates.len() < config::MIGRATION_GROUP_SIZE_MIN as usize {
                continue;
            }
            candidates.sort_by_key(|entity| entity.id());

            let group_size_roll: u32 = resources
                .rng
                .gen_range(config::MIGRATION_GROUP_SIZE_MIN..=config::MIGRATION_GROUP_SIZE_MAX);
            let group_size = (group_size_roll as usize).min(candidates.len());
            let migrants: Vec<Entity> = candidates.into_iter().take(group_size).collect();
            if migrants.len() < config::MIGRATION_GROUP_SIZE_MIN as usize {
                continue;
            }

            let Some((site_x, site_y)) = migration_find_site(resources, source_x, source_y) else {
                continue;
            };

            let next_settlement_raw = resources
                .settlements
                .keys()
                .map(|settlement_id| settlement_id.0)
                .max()
                .unwrap_or(0)
                + 1;
            let next_settlement_id = SettlementId(next_settlement_raw);
            let mut migrated_member_ids: Vec<EntityId> = Vec::with_capacity(migrants.len());
            for entity in migrants {
                if let Ok(mut one) =
                    world.query_one::<(&mut Identity, Option<&mut Behavior>)>(entity)
                {
                    if let Some((identity, behavior_opt)) = one.get() {
                        identity.settlement_id = Some(next_settlement_id);
                        if let Some(behavior) = behavior_opt {
                            behavior.current_action = ActionType::Migrate;
                            behavior.action_target_x = Some(site_x);
                            behavior.action_target_y = Some(site_y);
                            behavior.action_timer = 100;
                        }
                        migrated_member_ids.push(EntityId(entity.id() as u64));
                    }
                }
            }
            if migrated_member_ids.len() < config::MIGRATION_GROUP_SIZE_MIN as usize {
                continue;
            }

            let moved_set: HashSet<EntityId> = migrated_member_ids.iter().copied().collect();
            if let Some(source_settlement) = resources.settlements.get_mut(&source_id) {
                source_settlement.stockpile_food =
                    (source_settlement.stockpile_food - config::MIGRATION_STARTUP_FOOD).max(0.0);
                source_settlement.migration_cooldown = config::MIGRATION_COOLDOWN_TICKS as u32;
                source_settlement
                    .members
                    .retain(|member_id| !moved_set.contains(member_id));
            }

            let mut new_settlement = sim_core::Settlement::new(
                next_settlement_id,
                format!("settlement_{}", next_settlement_raw),
                site_x,
                site_y,
                tick,
            );
            new_settlement.stockpile_food = config::MIGRATION_STARTUP_FOOD;
            new_settlement.members = migrated_member_ids.clone();
            resources
                .settlements
                .insert(next_settlement_id, new_settlement);

            resources
                .event_bus
                .emit(sim_engine::GameEvent::SettlementFounded {
                    settlement_id: next_settlement_id,
                });
            for entity_id in migrated_member_ids {
                resources
                    .event_bus
                    .emit(sim_engine::GameEvent::MigrationOccurred {
                        entity_id,
                        from_settlement: source_id,
                        to_settlement: next_settlement_id,
                    });
            }
            break;
        }
    }
}

#[derive(Debug, Clone)]
pub struct MovementRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl MovementRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for MovementRuntimeSystem {
    fn name(&self) -> &'static str {
        "movement_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let max_x = f64::from(resources.map.width.saturating_sub(1));
        let max_y = f64::from(resources.map.height.saturating_sub(1));
        let mut query = world.query::<(
            &mut Position,
            &mut Behavior,
            Option<&mut Needs>,
            Option<&Age>,
            Option<&mut Inventory>,
        )>();
        for (entity, (position, behavior, mut needs_opt, age_opt, mut inventory_opt)) in &mut query
        {
            if age_opt.map(|age| !age.alive).unwrap_or(false) {
                position.vel_x = 0.0;
                position.vel_y = 0.0;
                position.movement_dir = 0;
                continue;
            }
            if behavior.action_timer > 0 {
                behavior.action_timer -= 1;
            }

            if behavior.action_timer <= 0 && behavior.current_action != ActionType::Idle {
                let completed_action = behavior.current_action;
                match completed_action {
                    ActionType::Eat => {
                        if let Some(inventory) = inventory_opt.as_mut() {
                            let food_item_id = inventory
                                .items
                                .iter()
                                .find(|item_id| {
                                    resources
                                        .item_store
                                        .get(**item_id)
                                        .map(|item| is_food_template(item.template_id.as_str()))
                                        .unwrap_or(false)
                                })
                                .copied();
                            if let Some(food_id) = food_item_id {
                                inventory.remove(food_id);
                                resources.item_store.remove(food_id);
                            }
                        }
                        if let Some(needs) = needs_opt.as_mut() {
                            needs.set(
                                NeedType::Hunger,
                                needs.get(NeedType::Hunger) + config::FOOD_HUNGER_RESTORE,
                            );
                        }
                    }
                    ActionType::Rest | ActionType::Sleep => {
                        if let Some(needs) = needs_opt.as_mut() {
                            needs.energy = (needs.energy + 0.5).clamp(0.0, 1.0);
                            needs.set(NeedType::Sleep, needs.energy);
                        }
                    }
                    ActionType::Socialize => {
                        if let Some(needs) = needs_opt.as_mut() {
                            needs.set(NeedType::Belonging, needs.get(NeedType::Belonging) + 0.3);
                        }
                    }
                    ActionType::Drink => {
                        if let Some(needs) = needs_opt.as_mut() {
                            needs.set(
                                NeedType::Thirst,
                                needs.get(NeedType::Thirst) + config::THIRST_DRINK_RESTORE,
                            );
                        }
                    }
                    ActionType::SitByFire => {
                        if let Some(needs) = needs_opt.as_mut() {
                            needs.set(
                                NeedType::Warmth,
                                needs.get(NeedType::Warmth) + config::WARMTH_FIRE_RESTORE,
                            );
                        }
                    }
                    ActionType::SeekShelter => {
                        if let Some(needs) = needs_opt.as_mut() {
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
                    ActionType::Forage
                    | ActionType::Hunt
                    | ActionType::TakeFromStockpile
                    | ActionType::GatherHerbs => {
                        if let Some(needs) = needs_opt.as_mut() {
                            needs.set(
                                NeedType::Hunger,
                                needs.get(NeedType::Hunger) + config::FOOD_HUNGER_RESTORE,
                            );
                        }
                        if matches!(completed_action, ActionType::Forage) {
                            maybe_grant_forage_berries(
                                EntityId(entity.id() as u64),
                                &mut inventory_opt,
                                resources,
                                _tick,
                            );
                        }
                    }
                    ActionType::GatherWood => {
                        behavior.carry = (behavior.carry + 1.0).min(config::MAX_CARRY as f32);
                    }
                    ActionType::GatherStone => {
                        behavior.carry = (behavior.carry + 1.0).min(config::MAX_CARRY as f32);
                    }
                    ActionType::Craft => {
                        let recipe_id = behavior.craft_recipe_id.take();
                        let material_id = behavior.craft_material_id.take();
                        if let (Some(recipe_id), Some(material_id)) = (recipe_id, material_id) {
                            let created_ids = crafting::craft_complete(
                                EntityId(entity.id() as u64),
                                recipe_id.as_str(),
                                material_id.as_str(),
                                resources,
                                _tick,
                            );
                            if let Some(inventory) = inventory_opt.as_mut() {
                                for item_id in created_ids {
                                    inventory.add(item_id);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                let tool_item_id =
                    crafting::action_tool_tag(completed_action).and_then(|tool_tag| {
                        inventory_opt
                            .as_deref()
                            .and_then(|inventory| {
                                crafting::find_best_tool(inventory, &resources.item_store, tool_tag)
                            })
                            .map(|(item_id, _)| item_id)
                    });
                if let Some(item_id) = tool_item_id {
                    let destroyed =
                        crafting::use_tool(EntityId(entity.id() as u64), item_id, resources, _tick);
                    if destroyed {
                        if let Some(inventory) = inventory_opt.as_mut() {
                            inventory.remove(item_id);
                        }
                    }
                }
                resources.event_store.push(SimEvent {
                    tick: _tick,
                    event_type: SimEventType::TaskCompleted,
                    actor: entity.id(),
                    target: None,
                    tags: vec!["behavior".to_string(), "task".to_string()],
                    cause: format!("{completed_action}"),
                    value: f64::from(behavior.action_duration),
                });
                behavior.current_action = ActionType::Idle;
                behavior.action_target_x = None;
                behavior.action_target_y = None;
                behavior.craft_recipe_id = None;
                behavior.craft_material_id = None;
                position.vel_x = 0.0;
                position.vel_y = 0.0;
                position.movement_dir = 0;
                continue;
            }

            if matches!(
                behavior.current_action,
                ActionType::Idle | ActionType::Rest | ActionType::Sleep
            ) {
                position.vel_x = 0.0;
                position.vel_y = 0.0;
                position.movement_dir = 0;
                continue;
            }

            let mut next_x = (position.x + position.vel_x).clamp(0.0, max_x);
            let mut next_y = (position.y + position.vel_y).clamp(0.0, max_y);

            if let (Some(target_x), Some(target_y)) =
                (behavior.action_target_x, behavior.action_target_y)
            {
                let dist_x = f64::from(target_x) - next_x;
                let dist_y = f64::from(target_y) - next_y;
                if (dist_x * dist_x + dist_y * dist_y).sqrt() <= 0.20 {
                    next_x = f64::from(target_x);
                    next_y = f64::from(target_y);
                }
            }

            let tile_x = next_x.round() as i32;
            let tile_y = next_y.round() as i32;
            if !resources.map.in_bounds(tile_x, tile_y)
                || !resources.map.get(tile_x as u32, tile_y as u32).passable
            {
                position.vel_x = 0.0;
                position.vel_y = 0.0;
                position.movement_dir = 0;
                continue;
            }

            position.x = next_x;
            position.y = next_y;
            position.movement_dir = movement_direction(position.vel_x, position.vel_y);
        }
    }
}

#[inline]
fn movement_direction(vel_x: f64, vel_y: f64) -> u8 {
    if vel_x.abs() < 0.01 && vel_y.abs() < 0.01 {
        return 0;
    }
    let angle = vel_y.atan2(vel_x);
    let octant = (angle / (std::f64::consts::PI / 4.0)).round() as i32;
    octant.rem_euclid(8) as u8
}

#[cfg(test)]
mod tests {
    use super::movement_direction;

    #[test]
    fn stage1_calculate_direction_8way() {
        assert_eq!(movement_direction(1.0, 0.0), 0);
        assert_eq!(movement_direction(0.0, -1.0), 6);
        assert_eq!(movement_direction(-1.0, 0.0), 4);
        assert_eq!(movement_direction(0.0, 1.0), 2);
        assert_eq!(movement_direction(0.0, 0.0), 0);
    }
}
