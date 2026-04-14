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
    ActionType, AttachmentType, Building, BuildingId, CopingStrategyId, EmotionType, EntityId,
    FurniturePlan, GrowthStage, HexacoAxis, HexacoFacet, IntelligenceType, MentalBreakType,
    NeedType, RelationType, ResourceType, SettlementId, Sex, SocialClass, TechState, ValueType,
    WallPlan,
};
use sim_data::DataRegistry;
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

#[inline]
fn resource_type_to_rule_tag(resource_type: ResourceType) -> &'static str {
    match resource_type {
        ResourceType::Food => "surface_foraging",
        ResourceType::Wood => "wood_harvesting",
        ResourceType::Stone => "stone_mining",
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
                    let multiplier = resources
                        .resource_regen_multipliers
                        .get(resource_type_to_rule_tag(deposit.resource_type))
                        .copied()
                        .unwrap_or(1.0);
                    let type_mul = match deposit.resource_type {
                        ResourceType::Food => resources.food_regen_mul,
                        ResourceType::Wood => resources.wood_regen_mul,
                        _ => 1.0,
                    };
                    let next_amount = deposit.amount + deposit.regen_rate * multiplier * type_mul;
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
    first_run: bool,
}

impl JobAssignmentRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            first_run: true,
        }
    }
}

const JOB_ASSIGNMENT_ORDER: [&str; 4] = ["gatherer", "lumberjack", "builder", "miner"];
const JOB_ASSIGNMENT_SURVIVAL_RATIOS: [f32; 4] = [0.8, 0.1, 0.1, 0.0];
const JOB_ASSIGNMENT_CRISIS_RATIOS: [f32; 4] = [0.6, 0.2, 0.1, 0.1];
const JOB_ASSIGNMENT_DEFAULT_RATIOS: [f32; 4] = [0.5, 0.25, 0.15, 0.1];
const JOB_ASSIGNMENT_CRISIS_FOOD_PER_ALIVE: f32 = 1.5;
const JOB_ASSIGNMENT_REBALANCE_THRESHOLD: f32 = 1.5;
const BUILDING_TYPE_STOCKPILE: &str = "stockpile";
const BUILDING_TYPE_CAMPFIRE: &str = "campfire";
const BUILDING_TYPE_SHELTER: &str = "shelter";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EarlyStructurePlan {
    Stockpile,
    Campfire,
    Shelter,
}

impl EarlyStructurePlan {
    #[inline]
    fn building_type(self) -> &'static str {
        match self {
            Self::Stockpile => BUILDING_TYPE_STOCKPILE,
            Self::Campfire => BUILDING_TYPE_CAMPFIRE,
            Self::Shelter => BUILDING_TYPE_SHELTER,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SettlementConstructionSnapshot {
    has_stockpile: bool,
    has_campfire: bool,
    has_incomplete_site: bool,
    complete_shelter_count: usize,
    has_incomplete_shelter: bool,
}

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
fn collect_alive_adult_counts(world: &World) -> HashMap<SettlementId, usize> {
    let mut counts: HashMap<SettlementId, usize> = HashMap::new();
    let mut query = world.query::<(Option<&Age>, Option<&Identity>)>();
    for (_, (age_opt, identity_opt)) in &mut query {
        let Some(age) = age_opt else {
            continue;
        };
        if !age.alive || age.stage != GrowthStage::Adult {
            continue;
        }
        let Some(settlement_id) = identity_opt.and_then(|identity| identity.settlement_id) else {
            continue;
        };
        *counts.entry(settlement_id).or_insert(0) += 1;
    }
    counts
}

#[inline]
fn settlement_construction_snapshot(
    resources: &SimResources,
    settlement_id: SettlementId,
) -> SettlementConstructionSnapshot {
    let mut snapshot = SettlementConstructionSnapshot::default();
    for building in resources.buildings.values() {
        if building.settlement_id != settlement_id {
            continue;
        }
        if !building.is_complete {
            snapshot.has_incomplete_site = true;
        }
        match building.building_type.as_str() {
            BUILDING_TYPE_STOCKPILE => snapshot.has_stockpile = true,
            BUILDING_TYPE_CAMPFIRE => snapshot.has_campfire = true,
            BUILDING_TYPE_SHELTER => {
                // P2-B3: legacy shelter Building entries are no longer
                // produced, but if any pre-existing entries linger we still
                // honour them so we don't double-build.
                if building.is_complete {
                    snapshot.complete_shelter_count += 1;
                } else {
                    snapshot.has_incomplete_shelter = true;
                }
            }
            _ => {}
        }
    }
    // P2-B3: detect shelter via wall_plans + already-stamped walls. The
    // legacy `complete_shelter_count` field is repurposed: any wall plans
    // outstanding OR walls already on the ring count as a shelter, so the
    // economy system stops generating duplicates.
    let has_pending_wall_plan = resources
        .wall_plans
        .iter()
        .any(|plan| plan.settlement_id == settlement_id);
    let has_pending_furniture_plan = resources
        .furniture_plans
        .iter()
        .any(|plan| plan.settlement_id == settlement_id);
    if has_pending_wall_plan || has_pending_furniture_plan {
        snapshot.has_incomplete_shelter = true;
        // Count in-progress shelter towards capacity so births aren't blocked
        // while agents place walls one at a time. Without this, population
        // stays at free_population_cap (25) until the entire ring is done.
        snapshot.complete_shelter_count = snapshot.complete_shelter_count.max(1);
    } else if shelter_walls_present(resources, settlement_id) {
        snapshot.complete_shelter_count = snapshot.complete_shelter_count.max(1);
    }
    snapshot
}

/// Returns true when a shelter wall ring is fully built around the
/// settlement center (P2-B3). Used to short-circuit plan regeneration
/// only after the ring is complete. A partial ring still counts as
/// "shelter under construction" and triggers fresh plan generation for
/// any missing perimeter tiles.
#[inline]
fn shelter_walls_present(resources: &SimResources, settlement_id: SettlementId) -> bool {
    let Some(settlement) = resources.settlements.get(&settlement_id) else {
        return false;
    };
    // Use the recorded shelter ring center; fall back to settlement center
    // for legacy saves where shelter_center is None.
    let (sx, sy) = settlement
        .shelter_center
        .unwrap_or((settlement.x, settlement.y));
    let r = config::BUILDING_SHELTER_WALL_RING_RADIUS;
    // Required = perimeter of (2R+1)² minus the door tile (matches the
    // count generated by `generate_wall_ring_plans`).
    let required = (8 * r - 1).max(1);
    let mut wall_count = 0_i32;
    for offset_y in -r..=r {
        for offset_x in -r..=r {
            let is_perimeter = offset_x.abs() == r || offset_y.abs() == r;
            if !is_perimeter {
                continue;
            }
            // Skip the door tile.
            if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
                && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y
            {
                continue;
            }
            let tile_x = sx + offset_x;
            let tile_y = sy + offset_y;
            if !resources.tile_grid.in_bounds(tile_x, tile_y) {
                continue;
            }
            if resources
                .tile_grid
                .get(tile_x as u32, tile_y as u32)
                .wall_material
                .is_some()
            {
                wall_count += 1;
            }
        }
    }
    wall_count >= required
}

/// Checks whether the shelter wall ring for the given settlement is
/// substantially built and, if so, marks the corresponding incomplete
/// shelter `Building` record as complete. This bridges the plan-queue
/// shelter lifecycle (P2-B3/B4) with the `resources.buildings` records
/// that other systems (migration, world) depend on for shelter counting.
///
/// Uses a threshold of `required - 2` placed walls (tolerating 1-2
/// positions that may overlap pre-existing buildings or take extra time)
/// rather than requiring perfect wall completion.
fn finalize_shelter_if_complete(
    resources: &mut SimResources,
    settlement_id: SettlementId,
) {
    let Some(settlement) = resources.settlements.get(&settlement_id) else {
        return;
    };
    let Some((cx, cy)) = settlement.shelter_center else {
        return;
    };

    // Count actually placed walls at perimeter positions.
    let r = config::BUILDING_SHELTER_WALL_RING_RADIUS;
    let required = (8 * r - 1).max(1);
    // Tolerate up to 2 missing walls (overlap with pre-existing buildings,
    // pending plans that haven't been claimed yet, etc.).
    let completion_threshold = required - 2;
    let mut wall_count = 0_i32;
    for offset_y in -r..=r {
        for offset_x in -r..=r {
            let is_perimeter = offset_x.abs() == r || offset_y.abs() == r;
            if !is_perimeter {
                continue;
            }
            if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
                && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y
            {
                continue;
            }
            let tile_x = cx + offset_x;
            let tile_y = cy + offset_y;
            if !resources.tile_grid.in_bounds(tile_x, tile_y) {
                continue;
            }
            if resources
                .tile_grid
                .get(tile_x as u32, tile_y as u32)
                .wall_material
                .is_some()
            {
                wall_count += 1;
            }
        }
    }
    if wall_count < completion_threshold {
        return;
    }

    // Find the first incomplete shelter Building for this settlement.
    let shelter_building_id = resources
        .buildings
        .iter()
        .find(|(_, b)| {
            b.settlement_id == settlement_id
                && b.building_type == BUILDING_TYPE_SHELTER
                && !b.is_complete
        })
        .map(|(id, _)| *id);

    if let Some(building_id) = shelter_building_id {
        if let Some(building) = resources.buildings.get_mut(&building_id) {
            building.construction_progress = 1.0;
            building.is_complete = true;
        }
        resources
            .event_bus
            .emit(sim_engine::GameEvent::BuildingConstructed {
                building_id,
                building_type: BUILDING_TYPE_SHELTER.to_string(),
            });
    }
}

/// Resolves the wall material id used by P2-B3 shelter ring plans.
///
/// Picks stone if the settlement has enough stone stockpiled to build a
/// minimal ring, otherwise falls back to wood. Mirrors the shelter material
/// hint logic in `influence::resolve_shelter_wall_material` so the two
/// systems agree on which material is consumed.
#[inline]
fn resolve_shelter_wall_material_for_plans(
    resources: &SimResources,
    settlement_id: SettlementId,
) -> String {
    let registry = resources.data_registry.as_deref();
    let stone_id = registry
        .and_then(|reg| {
            reg.materials
                .values()
                .find(|m| m.tags.iter().any(|t| t == "stone"))
                .map(|m| m.id.clone())
        })
        .unwrap_or_else(|| "granite".to_string());
    let wood_id = registry
        .and_then(|reg| {
            reg.materials
                .values()
                .find(|m| m.tags.iter().any(|t| t == "wood"))
                .map(|m| m.id.clone())
        })
        .unwrap_or_else(|| "oak".to_string());

    let stone_required = f64::from((8 * config::BUILDING_SHELTER_WALL_RING_RADIUS - 1).max(1))
        * config::BUILDING_SHELTER_STONE_COST_PER_WALL;
    let stone_in_stock = resources
        .settlements
        .get(&settlement_id)
        .map(|s| s.stockpile_stone)
        .unwrap_or(0.0);
    if stone_in_stock + f64::EPSILON >= stone_required {
        stone_id
    } else {
        wood_id
    }
}

/// Generates the wall ring + central fire pit plans for a settlement
/// (P2-B3). Skips door tile and out-of-bounds tiles. Floor + roof tiles
/// are stamped directly (not via plans) so the room-detection pipeline
/// can still produce a Shelter room and the shelter Safety/Energy effects
/// continue to apply via the room effect system.
fn generate_wall_ring_plans(
    resources: &mut SimResources,
    settlement_id: SettlementId,
    center_x: i32,
    center_y: i32,
    tick: u64,
) {
    let wall_material = resolve_shelter_wall_material_for_plans(resources, settlement_id);
    let wall_radius = config::BUILDING_SHELTER_WALL_RING_RADIUS;

    for offset_y in -wall_radius..=wall_radius {
        for offset_x in -wall_radius..=wall_radius {
            let is_perimeter = offset_x.abs() == wall_radius || offset_y.abs() == wall_radius;
            if !is_perimeter {
                continue;
            }
            if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
                && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y
            {
                continue;
            }
            let tile_x = center_x + offset_x;
            let tile_y = center_y + offset_y;
            if !resources.tile_grid.in_bounds(tile_x, tile_y) {
                continue;
            }
            // Skip tiles that overlap an existing building (stockpile, campfire, etc.).
            if resources
                .buildings
                .values()
                .any(|b| b.overlaps(tile_x, tile_y, 1, 1))
            {
                continue;
            }
            // Skip if a wall is already there (idempotent generation).
            if resources
                .tile_grid
                .get(tile_x as u32, tile_y as u32)
                .wall_material
                .is_some()
            {
                continue;
            }
            let plan_id = resources.next_plan_id;
            resources.next_plan_id = resources.next_plan_id.saturating_add(1);
            resources.wall_plans.push(WallPlan {
                id: plan_id,
                settlement_id,
                x: tile_x,
                y: tile_y,
                material_id: wall_material.clone(),
                claimed_by: None,
                created_tick: tick,
            });
        }
    }

    // Skip the fire pit plan if a furniture is already there at the center.
    if resources.tile_grid.in_bounds(center_x, center_y)
        && resources
            .tile_grid
            .get(center_x as u32, center_y as u32)
            .furniture_id
            .is_none()
    {
        let plan_id = resources.next_plan_id;
        resources.next_plan_id = resources.next_plan_id.saturating_add(1);
        resources.furniture_plans.push(FurniturePlan {
            id: plan_id,
            settlement_id,
            x: center_x,
            y: center_y,
            furniture_id: "fire_pit".to_string(),
            claimed_by: None,
            created_tick: tick,
        });
    }
}

/// Generates wall plans, floor stamps, furniture plans, and door markers
/// from a data-driven [`Blueprint`] definition. Replaces the hardcoded
/// `generate_wall_ring_plans()` for structures that define a blueprint
/// in their RON `StructureDef`.
///
/// Walls and furniture are generated as plans (for builders to claim).
/// Floors and doors are stamped immediately (not plan-based).
fn generate_plans_from_blueprint(
    resources: &mut SimResources,
    settlement_id: SettlementId,
    center_x: i32,
    center_y: i32,
    blueprint: &sim_data::Blueprint,
    tick: u64,
) {
    let wall_material = resolve_shelter_wall_material_for_plans(resources, settlement_id);

    // Generate wall plans from blueprint
    for wall_tile in &blueprint.walls {
        let tile_x = center_x + wall_tile.offset.0;
        let tile_y = center_y + wall_tile.offset.1;

        if !resources.tile_grid.in_bounds(tile_x, tile_y) {
            continue;
        }
        // Skip tiles that overlap an existing building footprint.
        if resources
            .buildings
            .values()
            .any(|b| b.overlaps(tile_x, tile_y, 1, 1))
        {
            continue;
        }
        // Skip if a wall is already there (idempotent generation).
        if resources
            .tile_grid
            .get(tile_x as u32, tile_y as u32)
            .wall_material
            .is_some()
        {
            continue;
        }

        let plan_id = resources.next_plan_id;
        resources.next_plan_id = resources.next_plan_id.saturating_add(1);
        resources.wall_plans.push(WallPlan {
            id: plan_id,
            settlement_id,
            x: tile_x,
            y: tile_y,
            material_id: wall_material.clone(),
            claimed_by: None,
            created_tick: tick,
        });
    }

    // Stamp floors from blueprint (immediate, not plan-based)
    for floor_tile in &blueprint.floors {
        let tile_x = center_x + floor_tile.offset.0;
        let tile_y = center_y + floor_tile.offset.1;
        if resources.tile_grid.in_bounds(tile_x, tile_y) {
            resources
                .tile_grid
                .set_floor(tile_x as u32, tile_y as u32, &floor_tile.material_tag);
        }
    }

    // Generate furniture plans from blueprint (same lifecycle as walls).
    // Builders claim and place furniture via PlaceFurniture action, keeping
    // blueprint furniture on the same code path as legacy fire_pit plans.
    for furn in &blueprint.furniture {
        let tile_x = center_x + furn.offset.0;
        let tile_y = center_y + furn.offset.1;
        if !resources.tile_grid.in_bounds(tile_x, tile_y) {
            continue;
        }
        // Skip if furniture already placed at this position.
        if resources
            .tile_grid
            .get(tile_x as u32, tile_y as u32)
            .furniture_id
            .is_some()
        {
            continue;
        }
        let plan_id = resources.next_plan_id;
        resources.next_plan_id = resources.next_plan_id.saturating_add(1);
        resources.furniture_plans.push(FurniturePlan {
            id: plan_id,
            settlement_id,
            x: tile_x,
            y: tile_y,
            furniture_id: furn.furniture_id.clone(),
            claimed_by: None,
            created_tick: tick,
        });
    }

    // Mark door positions (immediate, not plan-based)
    for &(dx, dy) in &blueprint.doors {
        let tile_x = center_x + dx;
        let tile_y = center_y + dy;
        if resources.tile_grid.in_bounds(tile_x, tile_y) {
            resources
                .tile_grid
                .set_door(tile_x as u32, tile_y as u32);
        }
    }
}

/// Removes wall/furniture plans that have lingered unclaimed beyond
/// `BUILDING_PLAN_STALE_TICKS`. Also drops orphaned claims pointing at
/// entities that are no longer in the world, and unclaims plans whose
/// claims have lingered without completing for too long (so the plan
/// can be retried by another builder).
///
/// IMPORTANT: claim ids are stored as `entity.id() as u64` (slot id only,
/// no generation). The alive set must use the same representation.
fn cleanup_stale_plans(world: &World, resources: &mut SimResources, tick: u64) {
    let stale_threshold = config::BUILDING_PLAN_STALE_TICKS;
    // Aggressively unclaim plans that have been held for more than this
    // many ticks without completion — usually means the builder picked a
    // different action and forgot the plan.
    let stuck_claim_threshold = 200_u64;
    let alive_set: HashSet<u64> = world
        .iter()
        .map(|entity_ref| entity_ref.entity().id() as u64)
        .collect();

    let release_dead = |claim: &mut Option<EntityId>| {
        if let Some(id) = claim {
            if !alive_set.contains(&id.0) {
                *claim = None;
            }
        }
    };

    for plan in resources.wall_plans.iter_mut() {
        release_dead(&mut plan.claimed_by);
        if plan.claimed_by.is_some()
            && tick.saturating_sub(plan.created_tick) > stuck_claim_threshold
        {
            // Force re-attempt by another builder; bump created_tick so the
            // age counter restarts and the plan does not immediately become
            // stale on the next sweep.
            plan.claimed_by = None;
            plan.created_tick = tick;
        }
    }
    for plan in resources.furniture_plans.iter_mut() {
        release_dead(&mut plan.claimed_by);
        if plan.claimed_by.is_some()
            && tick.saturating_sub(plan.created_tick) > stuck_claim_threshold
        {
            plan.claimed_by = None;
            plan.created_tick = tick;
        }
    }

    // Pre-compute settlements that have ANY shelter building (complete or
    // incomplete).  Wall plans only need protection while the shelter is
    // under construction, but furniture plans (lean_to, fire_pit) must
    // persist until a builder actually places them — which can happen well
    // after the shelter walls are finalized.
    let shelter_in_progress: HashSet<SettlementId> = resources
        .buildings
        .values()
        .filter(|b| b.building_type == BUILDING_TYPE_SHELTER && !b.is_complete)
        .map(|b| b.settlement_id)
        .collect();
    let shelter_any: HashSet<SettlementId> = resources
        .buildings
        .values()
        .filter(|b| b.building_type == BUILDING_TYPE_SHELTER)
        .map(|b| b.settlement_id)
        .collect();

    resources.wall_plans.retain(|plan| {
        if plan.claimed_by.is_some() {
            return true;
        }
        if shelter_in_progress.contains(&plan.settlement_id) {
            return true;
        }
        tick.saturating_sub(plan.created_tick) <= stale_threshold
    });
    resources.furniture_plans.retain(|plan| {
        if plan.claimed_by.is_some() {
            return true;
        }
        // Furniture plans are protected as long as the settlement has a
        // shelter building — even after wall completion.  This prevents
        // lean_to / fire_pit plans from being garbage-collected before a
        // builder can claim them.
        if shelter_any.contains(&plan.settlement_id) {
            return true;
        }
        tick.saturating_sub(plan.created_tick) <= stale_threshold
    });
}

#[inline]
fn legacy_structure_resource_cost(building_type: &str, resource_tag: &str) -> f64 {
    const LEGACY_COSTS: [(&str, &str, f64); 4] = [
        (
            BUILDING_TYPE_STOCKPILE,
            "wood",
            config::BUILDING_STOCKPILE_COST_WOOD,
        ),
        (
            BUILDING_TYPE_CAMPFIRE,
            "wood",
            config::BUILDING_CAMPFIRE_COST_WOOD,
        ),
        (
            BUILDING_TYPE_SHELTER,
            "wood",
            config::BUILDING_SHELTER_COST_WOOD,
        ),
        (
            BUILDING_TYPE_SHELTER,
            "stone",
            config::BUILDING_SHELTER_COST_STONE,
        ),
    ];

    LEGACY_COSTS
        .iter()
        .find(|(legacy_building_type, legacy_resource_tag, _)| {
            *legacy_building_type == building_type && *legacy_resource_tag == resource_tag
        })
        .map(|(_, _, amount)| *amount)
        .unwrap_or(0.0)
}

#[inline]
fn structure_resource_cost(
    building_type: &str,
    resource_tag: &str,
    registry: Option<&DataRegistry>,
) -> f64 {
    registry
        .and_then(|loaded| loaded.structure_resource_cost(building_type, resource_tag))
        .unwrap_or_else(|| legacy_structure_resource_cost(building_type, resource_tag))
}

#[inline]
fn settlement_resource_amount(settlement: &sim_core::Settlement, resource_tag: &str) -> f64 {
    match resource_tag {
        "food" => settlement.stockpile_food,
        "wood" => settlement.stockpile_wood,
        "stone" => settlement.stockpile_stone,
        _ => 0.0,
    }
}

#[inline]
fn settlement_can_afford_structure(
    settlement: &sim_core::Settlement,
    building_type: &str,
    registry: Option<&DataRegistry>,
) -> bool {
    if let Some(structure_def) = registry.and_then(|loaded| loaded.structure_def(building_type)) {
        if !structure_def.resource_costs.is_empty() {
            return structure_def
                .resource_costs
                .iter()
                .all(|(resource_tag, amount)| {
                    settlement_resource_amount(settlement, resource_tag.as_str()) + f64::EPSILON
                        >= *amount
                });
        }
    }

    ["food", "wood", "stone"]
        .iter()
        .copied()
        .all(|resource_tag| {
            settlement_resource_amount(settlement, resource_tag) + f64::EPSILON
                >= structure_resource_cost(building_type, resource_tag, registry)
        })
}

#[inline]
fn settlement_can_afford_plan(
    settlement: &sim_core::Settlement,
    plan: EarlyStructurePlan,
    registry: Option<&DataRegistry>,
) -> bool {
    settlement_can_afford_structure(settlement, plan.building_type(), registry)
}

#[inline]
fn structure_is_runtime_defined(building_type: &str, registry: Option<&DataRegistry>) -> bool {
    registry
        .map(|loaded| loaded.structure_def(building_type).is_some())
        .unwrap_or(true)
}

#[inline]
fn can_place_early_structure_plan(
    settlement: &sim_core::Settlement,
    plan: EarlyStructurePlan,
    registry: Option<&DataRegistry>,
) -> bool {
    structure_is_runtime_defined(plan.building_type(), registry)
        && settlement_can_afford_plan(settlement, plan, registry)
}

#[inline]
fn choose_early_structure_plan(
    settlement: &sim_core::Settlement,
    alive_adults: usize,
    snapshot: SettlementConstructionSnapshot,
    registry: Option<&DataRegistry>,
) -> Option<EarlyStructurePlan> {
    if snapshot.has_incomplete_site {
        return None;
    }
    if !snapshot.has_stockpile
        && can_place_early_structure_plan(settlement, EarlyStructurePlan::Stockpile, registry)
    {
        return Some(EarlyStructurePlan::Stockpile);
    }
    if !snapshot.has_campfire
        && can_place_early_structure_plan(settlement, EarlyStructurePlan::Campfire, registry)
    {
        return Some(EarlyStructurePlan::Campfire);
    }
    let shelter_capacity = snapshot.complete_shelter_count * config::BUILDING_SHELTER_CAPACITY;
    if shelter_capacity < alive_adults
        && !snapshot.has_incomplete_shelter
        && can_place_early_structure_plan(settlement, EarlyStructurePlan::Shelter, registry)
    {
        return Some(EarlyStructurePlan::Shelter);
    }
    None
}

/// Returns true when all tiles in the `width × height` footprint starting at `(x, y)`
/// are in-bounds, passable, and do not overlap any existing building footprint
/// (with a 1-tile gap enforced around each existing building).
#[inline]
fn building_site_is_available(
    resources: &SimResources,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> bool {
    // All tiles in the footprint must be in bounds and passable.
    for dy in 0..height as i32 {
        for dx in 0..width as i32 {
            let tx = x + dx;
            let ty = y + dy;
            if !resources.map.in_bounds(tx, ty) {
                return false;
            }
            if !resources.map.get(tx as u32, ty as u32).passable {
                return false;
            }
        }
    }
    // No overlap with existing buildings (1-tile spacing enforced).
    for building in resources.buildings.values() {
        if building.overlaps(x - 1, y - 1, width + 2, height + 2) {
            return false;
        }
    }
    true
}

#[inline]
fn find_build_site(
    resources: &SimResources,
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
) -> Option<(i32, i32)> {
    let search_radius = config::SETTLEMENT_BUILD_RADIUS.max(1);
    for radius in 1..=search_radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let x = origin_x + dx;
                let y = origin_y + dy;
                if building_site_is_available(resources, x, y, width, height) {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

#[inline]
fn next_building_id(resources: &SimResources) -> BuildingId {
    BuildingId(
        resources
            .buildings
            .keys()
            .map(|building_id| building_id.0)
            .max()
            .unwrap_or(0)
            + 1,
    )
}

#[inline]
fn place_early_structure_site(
    resources: &mut SimResources,
    settlement_id: SettlementId,
    plan: EarlyStructurePlan,
    tick: u64,
) -> Option<BuildingId> {
    let (origin_x, origin_y) = resources
        .settlements
        .get(&settlement_id)
        .map(|settlement| (settlement.x, settlement.y))?;
    // Read footprint from StructureDef when the data registry is loaded;
    // fall back to 1×1 for backward compatibility (e.g. headless harness tests).
    let building_type = plan.building_type();
    let (width, height) = resources
        .data_registry
        .as_deref()
        .and_then(|reg| reg.structures.get(building_type))
        .map(|def| def.min_size)
        .unwrap_or((1, 1));
    let (site_x, site_y) = find_build_site(resources, origin_x, origin_y, width, height)?;
    let building_id = next_building_id(resources);
    let building = Building::new(
        building_id,
        building_type.to_string(),
        settlement_id,
        site_x,
        site_y,
        width,
        height,
        tick,
    );
    resources.buildings.insert(building_id, building);
    if let Some(settlement) = resources.settlements.get_mut(&settlement_id) {
        if !settlement.buildings.contains(&building_id) {
            settlement.buildings.push(building_id);
        }
    }
    Some(building_id)
}

#[inline]
fn ensure_early_construction_sites(world: &World, resources: &mut SimResources, tick: u64) {
    let alive_adults = collect_alive_adult_counts(world);
    let mut settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
    settlement_ids.sort_by_key(|settlement_id| settlement_id.0);

    for settlement_id in settlement_ids {
        // P2-B4: check if an in-progress shelter is now complete (all
        // walls placed) and finalize its Building record so downstream
        // consumers (migration, world) see it as a completed shelter.
        finalize_shelter_if_complete(resources, settlement_id);

        let plan = {
            let Some(settlement) = resources.settlements.get(&settlement_id) else {
                continue;
            };
            let snapshot = settlement_construction_snapshot(resources, settlement_id);
            choose_early_structure_plan(
                settlement,
                alive_adults.get(&settlement_id).copied().unwrap_or(0),
                snapshot,
                resources.data_registry.as_deref(),
            )
        };
        let Some(plan) = plan else {
            continue;
        };
        // P2-B3/B4: shelter queues per-tile WallPlan + FurniturePlan entries
        // for builder agents. If a Blueprint is defined in the StructureDef,
        // use the data-driven path; otherwise fall back to the legacy hardcoded ring.
        // Stockpile and Campfire still use place_early_structure_site().
        if matches!(plan, EarlyStructurePlan::Shelter) {
            let blueprint = resources
                .data_registry
                .as_deref()
                .and_then(|reg| reg.structures.get("shelter"))
                .and_then(|def| def.blueprint.as_ref())
                .cloned();

            let origin = resources
                .settlements
                .get(&settlement_id)
                .map(|s| (s.x, s.y));
            if let Some((ox, oy)) = origin {
                // Shelter footprint = 5×5 (wall_ring_radius=2)
                let footprint =
                    2 * config::BUILDING_SHELTER_WALL_RING_RADIUS as u32 + 1;
                if let Some((site_x, site_y)) =
                    find_build_site(resources, ox, oy, footprint, footprint)
                {
                    // find_build_site returns top-left; center = top-left + radius
                    let cx =
                        site_x + config::BUILDING_SHELTER_WALL_RING_RADIUS;
                    let cy =
                        site_y + config::BUILDING_SHELTER_WALL_RING_RADIUS;
                    // Record actual ring center so shelter_walls_present and
                    // biology shelter detection check the correct location.
                    if let Some(s) = resources.settlements.get_mut(&settlement_id) {
                        s.shelter_center = Some((cx, cy));
                    }
                    if let Some(ref bp) = blueprint {
                        generate_plans_from_blueprint(
                            resources,
                            settlement_id,
                            cx,
                            cy,
                            bp,
                            tick,
                        );
                    } else {
                        // Fallback to legacy hardcoded ring
                        generate_wall_ring_plans(
                            resources,
                            settlement_id,
                            cx,
                            cy,
                            tick,
                        );
                    }
                    // Create an incomplete shelter Building record so
                    // the construction lifecycle can track and finalize
                    // the shelter once all walls are placed.
                    let building_id = next_building_id(resources);
                    let building = Building::new(
                        building_id,
                        BUILDING_TYPE_SHELTER.to_string(),
                        settlement_id,
                        site_x,
                        site_y,
                        footprint,
                        footprint,
                        tick,
                    );
                    resources.buildings.insert(building_id, building);
                    if let Some(s) =
                        resources.settlements.get_mut(&settlement_id)
                    {
                        if !s.buildings.contains(&building_id) {
                            s.buildings.push(building_id);
                        }
                    }
                }
            }
            continue;
        }
        let _ = place_early_structure_site(resources, settlement_id, plan, tick);
    }
}

#[inline]
fn collect_pending_site_targets(
    resources: &SimResources,
) -> HashMap<SettlementId, HashSet<(i32, i32)>> {
    let mut out: HashMap<SettlementId, HashSet<(i32, i32)>> = HashMap::new();

    // Legacy Building-based sites (stockpile, campfire, and any residual
    // shelter Buildings that pre-date the P2-B3 plan-queue model).
    for building in resources.buildings.values() {
        if building.is_complete {
            continue;
        }
        out.entry(building.settlement_id)
            .or_default()
            .insert((building.x, building.y));
    }

    // P2-B3: wall/furniture plans are first-class pending sites. Unclaimed
    // plans force builder assignment; claimed plans keep the existing claimer
    // recognized as "assigned" via target_matches so the retask-if-no-assigned
    // loop below does not pull a working builder off PlaceWall / PlaceFurniture.
    for plan in &resources.wall_plans {
        out.entry(plan.settlement_id)
            .or_default()
            .insert((plan.x, plan.y));
    }
    for plan in &resources.furniture_plans {
        out.entry(plan.settlement_id)
            .or_default()
            .insert((plan.x, plan.y));
    }

    out
}

#[inline]
fn retask_builder_for_construction(world: &mut World, entity: Entity) {
    if let Ok(mut one) = world.query_one::<&mut Behavior>(entity) {
        if let Some(behavior) = one.get() {
            behavior.job = "builder".to_string();
            behavior.current_action = ActionType::Idle;
            behavior.action_target_entity = None;
            behavior.action_target_x = None;
            behavior.action_target_y = None;
            behavior.action_progress = 0.0;
            behavior.action_duration = 0;
            behavior.action_timer = 0;
        }
    }
}

#[inline]
fn ensure_pending_sites_have_builder(world: &mut World, resources: &SimResources) {
    let pending_sites = collect_pending_site_targets(resources);
    if pending_sites.is_empty() {
        return;
    }

    #[derive(Default)]
    struct SettlementBuilderStatus {
        assigned_builder_count: usize,
        available_builders: Vec<Entity>,
        fallback_candidates: Vec<Entity>,
    }

    let mut statuses: HashMap<SettlementId, SettlementBuilderStatus> = HashMap::new();
    let mut query = world.query::<(Option<&Age>, Option<&Identity>, &Behavior)>();
    for (entity, (age_opt, identity_opt, behavior)) in &mut query {
        let Some(age) = age_opt else {
            continue;
        };
        if !age.alive || age.stage != GrowthStage::Adult {
            continue;
        }
        let Some(settlement_id) = identity_opt.and_then(|identity| identity.settlement_id) else {
            continue;
        };
        let Some(targets) = pending_sites.get(&settlement_id) else {
            continue;
        };
        let status = statuses.entry(settlement_id).or_default();
        let target_matches = matches!(
            (behavior.action_target_x, behavior.action_target_y),
            (Some(x), Some(y)) if targets.contains(&(x, y))
        );
        if behavior.job == "builder" {
            // P2-B3: recognize the new plan-queue construction actions
            // alongside the legacy Build action so a builder mid-PlaceWall
            // (or mid-PlaceFurniture) is counted as already assigned
            // rather than bucketed into available_builders and retasked
            // every tick.
            let is_construction_action = matches!(
                behavior.current_action,
                ActionType::Build | ActionType::PlaceWall | ActionType::PlaceFurniture
            );
            if is_construction_action && target_matches {
                status.assigned_builder_count += 1;
            } else {
                status.available_builders.push(entity);
            }
            continue;
        }
        if behavior.occupation.is_empty()
            || behavior.occupation == "none"
            || behavior.occupation == "laborer"
        {
            status.fallback_candidates.push(entity);
        }
    }
    drop(query);

    let mut settlement_ids: Vec<SettlementId> = pending_sites.keys().copied().collect();
    settlement_ids.sort_by_key(|settlement_id| settlement_id.0);
    for settlement_id in settlement_ids {
        let Some(status) = statuses.get(&settlement_id) else {
            continue;
        };
        if status.assigned_builder_count > 0 {
            continue;
        }
        if let Some(entity) = status
            .available_builders
            .first()
            .copied()
            .or_else(|| status.fallback_candidates.first().copied())
        {
            retask_builder_for_construction(world, entity);
        }
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

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        // On first run, reset all adult jobs to "none" for proper ratio-based distribution
        if self.first_run {
            self.first_run = false;
            let mut reset_query = world.query::<(&Age, &mut Behavior)>();
            for (_, (age, behavior)) in &mut reset_query {
                if matches!(age.stage, GrowthStage::Adult | GrowthStage::Teen | GrowthStage::Elder)
                    && (behavior.occupation.is_empty()
                        || behavior.occupation == "none"
                        || behavior.occupation == "laborer")
                {
                    behavior.job = "none".to_string();
                }
            }
            drop(reset_query);
        }

        let mut alive_count: i32 = 0;
        let mut job_counts: [i32; 4] = [0; 4];
        let mut unassigned: Vec<(Entity, GrowthStage)> = Vec::new();

        let mut query = world.query::<(&Age, &mut Behavior)>();
        for (entity, (age, behavior)) in &mut query {
            match age.stage {
                GrowthStage::Infant | GrowthStage::Toddler => {
                    if behavior.job != "none" {
                        behavior.job = "none".to_string();
                    }
                    continue; // Don't count in alive_count — not eligible for job assignment
                }
                GrowthStage::Child => {
                    if behavior.job != "gatherer" {
                        behavior.job = "gatherer".to_string();
                    }
                    continue; // Don't count in alive_count — force-assigned, not eligible
                }
                _ => {}
            }
            alive_count += 1; // Only count Adult/Teen/Elder

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
                    if behavior.job == surplus_job
                        && matches!(
                            behavior.current_action,
                            ActionType::Idle
                                | ActionType::Wander
                                | ActionType::Rest
                                | ActionType::Socialize
                                | ActionType::Forage
                        )
                    {
                        behavior.job = deficit_job.to_string();
                        break;
                    }
                }
            }
        }

        ensure_early_construction_sites(world, resources, tick);
        ensure_pending_sites_have_builder(world, resources);
        // P2-B3: prune stale unclaimed wall/furniture plans and orphaned claims.
        cleanup_stale_plans(world, resources, tick);
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
                            let before = settlement.stockpile_food;
                            let cap = config::FOOD_STOCKPILE_CAP;
                            settlement.stockpile_food =
                                (settlement.stockpile_food + gathered_f64).min(cap).max(0.0);
                            let actual = (settlement.stockpile_food - before).max(0.0);
                            resources.food_economy_produced += actual;
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
fn legacy_construction_build_ticks(building_type: &str) -> Option<i32> {
    const LEGACY_BUILD_TICKS: [(&str, i32); 3] = [
        (BUILDING_TYPE_STOCKPILE, 36),
        (BUILDING_TYPE_SHELTER, 60),
        (BUILDING_TYPE_CAMPFIRE, 24),
    ];

    LEGACY_BUILD_TICKS
        .iter()
        .find(|(legacy_building_type, _)| *legacy_building_type == building_type)
        .map(|(_, ticks)| *ticks)
}

#[inline]
fn construction_build_ticks(building_type: &str, registry: Option<&DataRegistry>) -> i32 {
    registry
        .and_then(|loaded| loaded.structure_build_ticks(building_type))
        .map(|ticks| ticks as i32)
        .or_else(|| legacy_construction_build_ticks(building_type))
        .unwrap_or(CONSTRUCTION_BUILD_TICKS_DEFAULT)
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
            let build_ticks = construction_build_ticks(
                building.building_type.as_str(),
                resources.data_registry.as_deref(),
            )
            .max(1);
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
        if resources.buildings.is_empty() {
            return;
        }
        let mut effects: Vec<BuildingEffectSnapshot> = Vec::new();
        for building in resources.buildings.values() {
            if !building.is_complete {
                continue;
            }
            let kind = match building.building_type.as_str() {
                "campfire" => BuildingEffectKind::Campfire,
                "shelter" => BuildingEffectKind::Shelter,
                _ => continue,
            };
            effects.push(BuildingEffectSnapshot {
                kind,
                x: building.x,
                y: building.y,
            });
        }
        if effects.is_empty() {
            return;
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
    use sim_core::components::{Age, Behavior, Identity, Position, Skills};
    use sim_core::config::GameConfig;
    use sim_core::{
        ActionType, Building, BuildingId, GameCalendar, GrowthStage, SettlementId, WorldMap,
    };
    use sim_data::DataRegistry;
    use sim_engine::SimResources;

    fn registry_data_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../sim-data/data")
            .canonicalize()
            .expect("registry data path should resolve")
    }

    fn load_registry() -> DataRegistry {
        DataRegistry::load_from_directory(&registry_data_path())
            .expect("registry should load for economy tests")
    }

    #[test]
    fn job_assignment_runtime_system_places_campfire_site_and_promotes_builder() {
        let game_config = GameConfig::default();
        let calendar = GameCalendar::new(&game_config);
        let mut resources = SimResources::new(calendar, WorldMap::new(18, 18, 11), 77);
        let settlement_id = SettlementId(1);
        let mut settlement = sim_core::Settlement::new(settlement_id, "alpha".to_string(), 9, 9, 0);
        settlement.stockpile_food = 20.0;
        settlement.stockpile_wood = 3.0;
        settlement.stockpile_stone = 1.0;
        settlement.buildings.push(BuildingId(1));
        resources.settlements.insert(settlement_id, settlement);
        resources.buildings.insert(
            BuildingId(1),
            Building {
                id: BuildingId(1),
                building_type: "stockpile".to_string(),
                settlement_id,
                x: 9,
                y: 9,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut world = World::new();
        for offset in 0..3 {
            world.spawn((
                Age {
                    stage: GrowthStage::Adult,
                    ..Age::default()
                },
                Identity {
                    settlement_id: Some(settlement_id),
                    name: format!("worker_{offset}"),
                    ..Identity::default()
                },
                Position::new(9 + offset, 10),
                Behavior {
                    job: "gatherer".to_string(),
                    current_action: ActionType::Forage,
                    action_timer: 5,
                    action_duration: 5,
                    ..Behavior::default()
                },
            ));
        }

        let mut system =
            JobAssignmentRuntimeSystem::new(8, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL,
        );

        let campfire = resources
            .buildings
            .values()
            .find(|building| building.building_type == "campfire" && !building.is_complete)
            .expect("job assignment should place an early campfire site");
        assert_eq!(campfire.settlement_id, settlement_id);

        let mut builder_count = 0_usize;
        let mut query = world.query::<(&Identity, &Behavior)>();
        for (_, (identity, behavior)) in &mut query {
            if identity.settlement_id != Some(settlement_id) {
                continue;
            }
            if behavior.job == "builder" {
                builder_count += 1;
                assert_eq!(behavior.current_action, ActionType::Idle);
                assert_eq!(behavior.action_timer, 0);
            }
        }
        assert_eq!(builder_count, 1);
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
                width: 1,
                height: 1,
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

    #[test]
    fn construction_build_ticks_reads_from_registry() {
        let registry = load_registry();

        assert_eq!(construction_build_ticks("campfire", Some(&registry)), 24);
        assert_eq!(construction_build_ticks("stockpile", Some(&registry)), 36);
        assert_eq!(
            construction_build_ticks("unknown_structure", Some(&registry)),
            50
        );
    }

    #[test]
    fn construction_cost_reads_from_registry() {
        let registry = load_registry();
        let mut settlement =
            sim_core::Settlement::new(SettlementId(1), "alpha".to_string(), 0, 0, 0);
        settlement.stockpile_wood = 4.0;
        settlement.stockpile_stone = 1.0;

        assert_eq!(
            structure_resource_cost("shelter", "wood", Some(&registry)),
            4.0
        );
        assert_eq!(
            structure_resource_cost("shelter", "stone", Some(&registry)),
            1.0
        );
        assert!(settlement_can_afford_plan(
            &settlement,
            EarlyStructurePlan::Shelter,
            Some(&registry),
        ));

        settlement.stockpile_wood = 3.0;
        assert!(!settlement_can_afford_plan(
            &settlement,
            EarlyStructurePlan::Shelter,
            Some(&registry),
        ));
    }

    #[test]
    fn legacy_fallback_when_registry_missing() {
        assert_eq!(construction_build_ticks(BUILDING_TYPE_CAMPFIRE, None), 24);
        assert_eq!(
            structure_resource_cost(BUILDING_TYPE_STOCKPILE, "wood", None),
            config::BUILDING_STOCKPILE_COST_WOOD
        );
        assert_eq!(
            structure_resource_cost(BUILDING_TYPE_SHELTER, "stone", None),
            config::BUILDING_SHELTER_COST_STONE
        );
    }

    #[test]
    fn world_rules_resource_modifier_scales_regen() {
        let game_config = GameConfig::default();
        let calendar = GameCalendar::new(&game_config);
        let mut resources = SimResources::new(calendar, WorldMap::new(4, 4, 3), 77);
        resources
            .resource_regen_multipliers
            .insert("surface_foraging".to_string(), 2.0);
        resources
            .map
            .get_mut(1, 1)
            .resources
            .push(sim_core::world::tile::TileResource {
                resource_type: ResourceType::Food,
                amount: 1.0,
                max_amount: 10.0,
                regen_rate: 0.75,
            });

        let mut world = World::new();
        let mut system = ResourceRegenSystem::new(5, 10);
        system.run(&mut world, &mut resources, 10);

        let tile = resources.map.get_mut(1, 1);
        let deposit = tile
            .resources
            .iter()
            .find(|resource| resource.resource_type == ResourceType::Food)
            .expect("food deposit should remain present");
        assert!((deposit.amount - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn world_rules_absent_modifier_keeps_regen_default() {
        let game_config = GameConfig::default();
        let calendar = GameCalendar::new(&game_config);
        let mut resources = SimResources::new(calendar, WorldMap::new(4, 4, 3), 77);
        resources
            .map
            .get_mut(1, 1)
            .resources
            .push(sim_core::world::tile::TileResource {
                resource_type: ResourceType::Food,
                amount: 1.0,
                max_amount: 10.0,
                regen_rate: 0.75,
            });

        let mut world = World::new();
        let mut system = ResourceRegenSystem::new(5, 10);
        system.run(&mut world, &mut resources, 10);

        let tile = resources.map.get_mut(1, 1);
        let deposit = tile
            .resources
            .iter()
            .find(|resource| resource.resource_type == ResourceType::Food)
            .expect("food deposit should remain present");
        assert!((deposit.amount - 1.75).abs() < f64::EPSILON);
    }
}
