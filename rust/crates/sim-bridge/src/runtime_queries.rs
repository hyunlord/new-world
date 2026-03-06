use godot::builtin::{Array, PackedInt32Array, VarDictionary};
use hecs::{Entity, World};
use sim_core::components::{
    Age, Behavior, Emotion, Identity, Personality, Position, Skills, Social, Stress,
};
use sim_core::enums::{RelationType, Sex, TechState, TerrainType};
use sim_core::world::TileResource;
use sim_core::{Building, BuildingId, EntityId, Settlement, SettlementId};
use sim_engine::RuntimeStatsSnapshot;
use sim_systems::entity_spawner::{self, SpawnConfig};
use std::collections::HashMap;

use crate::runtime_registry::RuntimeState;

const BIOME_DEEP_WATER: i32 = 0;
const BIOME_SHALLOW_WATER: i32 = 1;
const BIOME_BEACH: i32 = 2;
const BIOME_GRASSLAND: i32 = 3;
const BIOME_FOREST: i32 = 4;
const BIOME_DENSE_FOREST: i32 = 5;
const BIOME_HILL: i32 = 6;
const BIOME_MOUNTAIN: i32 = 7;
const BIOME_SNOW: i32 = 8;

const BIOME_MOVE_COST_DEEP_WATER: f32 = 0.0;
const BIOME_MOVE_COST_SHALLOW_WATER: f32 = 0.0;
const BIOME_MOVE_COST_BEACH: f32 = 1.2;
const BIOME_MOVE_COST_GRASSLAND: f32 = 1.0;
const BIOME_MOVE_COST_FOREST: f32 = 1.3;
const BIOME_MOVE_COST_DENSE_FOREST: f32 = 1.8;
const BIOME_MOVE_COST_HILL: f32 = 1.5;
const BIOME_MOVE_COST_MOUNTAIN: f32 = 0.0;
const BIOME_MOVE_COST_SNOW: f32 = 2.0;

/// One-time bootstrap payload copied from the Godot-generated world.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RuntimeBootstrapPayload {
    pub(crate) world: RuntimeBootstrapWorld,
    pub(crate) founding_settlement: RuntimeBootstrapSettlement,
    #[serde(default)]
    pub(crate) agents: Vec<RuntimeBootstrapAgent>,
}

/// Full flat world payload copied from `WorldData` and `ResourceMap`.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RuntimeBootstrapWorld {
    pub(crate) width: u32,
    pub(crate) height: u32,
    #[serde(default)]
    pub(crate) biomes: Vec<i32>,
    #[serde(default)]
    pub(crate) elevation: Vec<f32>,
    #[serde(default)]
    pub(crate) moisture: Vec<f32>,
    #[serde(default)]
    pub(crate) temperature: Vec<f32>,
    #[serde(default)]
    pub(crate) food: Vec<f32>,
    #[serde(default)]
    pub(crate) wood: Vec<f32>,
    #[serde(default)]
    pub(crate) stone: Vec<f32>,
}

/// Founding settlement bootstrap settings.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RuntimeBootstrapSettlement {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) x: i32,
    pub(crate) y: i32,
    #[serde(default)]
    pub(crate) stockpile_food: f64,
    #[serde(default)]
    pub(crate) stockpile_wood: f64,
    #[serde(default)]
    pub(crate) stockpile_stone: f64,
}

/// One agent spawn entry.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RuntimeBootstrapAgent {
    pub(crate) x: i32,
    pub(crate) y: i32,
    #[serde(default)]
    pub(crate) age_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootstrapWorldResult {
    spawned: usize,
    settlement_id: u64,
    building_count: usize,
    entity_count: usize,
}

/// Applies startup bootstrap data into the live runtime and returns a summary.
pub(crate) fn bootstrap_world(state: &mut RuntimeState, payload: RuntimeBootstrapPayload) -> VarDictionary {
    let mut out = VarDictionary::new();
    let Some(result) = bootstrap_world_core(state, payload) else {
        return out;
    };

    out.set("spawned", result.spawned as i64);
    out.set("settlement_id", result.settlement_id as i64);
    out.set("building_count", result.building_count as i64);
    out.set("entity_count", result.entity_count as i64);
    out
}

fn bootstrap_world_core(
    state: &mut RuntimeState,
    payload: RuntimeBootstrapPayload,
) -> Option<BootstrapWorldResult> {
    if payload.world.width == 0 || payload.world.height == 0 {
        return None;
    }

    state.accumulator = 0.0;
    state.speed_index = 0;
    state.paused = true;
    if let Ok(mut captured_events) = state.captured_events.lock() {
        captured_events.clear();
    }
    *state.engine.world_mut() = World::new();
    {
        let resources = state.engine.resources_mut();
        resources.map = sim_core::WorldMap::new(payload.world.width, payload.world.height, resources.map.seed);
        resources.settlements.clear();
        resources.buildings.clear();
        resources.tension_pairs.clear();
        resources.stats_history.clear();
        resources.stats_peak_population = 0;
        resources.stats_total_births = 0;
        resources.stats_total_deaths = 0;
        resources.stat_sync_derived.clear();
        resources.stat_threshold_flags.clear();
        resources.chronicle_world_events.clear();
        resources.chronicle_personal_events.clear();
        sync_world_tiles(resources, &payload.world);
    }

    let settlement_id = SettlementId(payload.founding_settlement.id.max(1));
    let mut settlement = Settlement::new(
        settlement_id,
        if payload.founding_settlement.name.trim().is_empty() {
            format!("Settlement {}", settlement_id.0)
        } else {
            payload.founding_settlement.name.clone()
        },
        payload.founding_settlement.x,
        payload.founding_settlement.y,
        state.engine.current_tick(),
    );
    settlement.stockpile_food = payload.founding_settlement.stockpile_food.max(0.0);
    settlement.stockpile_wood = payload.founding_settlement.stockpile_wood.max(0.0);
    settlement.stockpile_stone = payload.founding_settlement.stockpile_stone.max(0.0);

    let stockpile_building_id = BuildingId(1);
    if settlement.stockpile_food > 0.0 || settlement.stockpile_wood > 0.0 || settlement.stockpile_stone > 0.0 {
        let mut stockpile = Building::new(
            stockpile_building_id,
            "stockpile".to_string(),
            settlement_id,
            payload.founding_settlement.x,
            payload.founding_settlement.y,
            state.engine.current_tick(),
        );
        stockpile.construction_progress = 1.0;
        stockpile.is_complete = true;
        settlement.buildings.push(stockpile_building_id);
        state.engine.resources_mut().buildings.insert(stockpile_building_id, stockpile);
    }

    let mut spawned_entities: Vec<Entity> = Vec::with_capacity(payload.agents.len());
    for agent in &payload.agents {
        let config = SpawnConfig {
            settlement_id: Some(settlement_id),
            position: (agent.x, agent.y),
            initial_age_ticks: agent.age_ticks,
            sex: None,
            parent_a: None,
            parent_b: None,
        };
        let (world, resources) = state.engine.world_and_resources_mut();
        let entity = entity_spawner::spawn_agent(world, resources, &config);
        settlement.members.push(EntityId(entity.id() as u64));
        spawned_entities.push(entity);
    }

    settlement.leader_id = pick_leader(state.engine.world(), &spawned_entities)
        .map(|entity| EntityId(entity.id() as u64));
    state.engine.resources_mut().settlements.insert(settlement_id, settlement);
    seed_initial_relationships(state.engine.world_mut(), &spawned_entities);

    Some(BootstrapWorldResult {
        spawned: payload.agents.len(),
        settlement_id: settlement_id.0,
        building_count: state.engine.resources().buildings.len(),
        entity_count: state.engine.world().len() as usize,
    })
}

/// Returns a bridge-friendly settlement detail snapshot.
pub(crate) fn settlement_detail(state: &RuntimeState, settlement_id_raw: i64) -> VarDictionary {
    let mut out = VarDictionary::new();
    let settlement_id = SettlementId(settlement_id_raw.max(0) as u64);
    let Some(settlement) = state.engine.resources().settlements.get(&settlement_id) else {
        return out;
    };

    let raw_lookup = build_raw_entity_id_lookup(state.engine.world());
    let members = collect_member_summaries(state.engine.world(), settlement, &raw_lookup);
    let leader = settlement
        .leader_id
        .and_then(|leader_id| runtime_bits_from_raw_id(&raw_lookup, leader_id.0))
        .and_then(|entity_id| member_summary_by_runtime_id(state.engine.world(), entity_id));
    let buildings = collect_building_summaries(state, settlement);
    let aggregates = settlement_aggregates(&members);

    out.set("id", settlement.id.0 as i64);
    out.set("name", settlement.name.clone());
    out.set("center_x", settlement.x as i64);
    out.set("center_y", settlement.y as i64);
    out.set("tech_era", settlement.current_era.clone());
    out.set(
        "leader_id",
        settlement
            .leader_id
            .and_then(|id| runtime_bits_from_raw_id(&raw_lookup, id.0))
            .unwrap_or(-1),
    );
    out.set("population", aggregates.population as i64);
    out.set("adults", aggregates.adults as i64);
    out.set("children", aggregates.children as i64);
    out.set("teens", aggregates.teens as i64);
    out.set("elders", aggregates.elders as i64);
    out.set("male_count", aggregates.male as i64);
    out.set("female_count", aggregates.female as i64);
    out.set("avg_happiness", aggregates.avg_happiness);
    out.set("avg_stress", aggregates.avg_stress);
    out.set("stockpile_food", settlement.stockpile_food);
    out.set("stockpile_wood", settlement.stockpile_wood);
    out.set("stockpile_stone", settlement.stockpile_stone);
    out.set("members", members);
    out.set("leader", leader.unwrap_or_default());
    out.set("buildings", buildings);
    out.set("building_ids", int_id_array_from_u64_ids(settlement.buildings.iter().map(|id| id.0)));
    out.set(
        "member_ids",
        int_id_array_from_u64_ids(
            settlement
                .members
                .iter()
                .filter_map(|id| runtime_bits_from_raw_id(&raw_lookup, id.0).map(|bits| bits as u64)),
        ),
    );
    out.set("tech_states", tech_states_dict(settlement));
    out
}

/// Returns a bridge-friendly building detail snapshot.
pub(crate) fn building_detail(state: &RuntimeState, building_id_raw: i64) -> VarDictionary {
    let mut out = VarDictionary::new();
    let building_id = BuildingId(building_id_raw.max(0) as u64);
    let Some(building) = state.engine.resources().buildings.get(&building_id) else {
        return out;
    };
    let settlement = state.engine.resources().settlements.get(&building.settlement_id);
    out.set("id", building.id.0 as i64);
    out.set("building_type", building.building_type.clone());
    out.set("settlement_id", building.settlement_id.0 as i64);
    out.set("tile_x", building.x as i64);
    out.set("tile_y", building.y as i64);
    out.set("is_constructed", building.is_complete);
    out.set("is_built", building.is_complete);
    out.set("construction_progress", building.construction_progress as f64);
    out.set("build_progress", building.construction_progress as f64);
    out.set("condition", building.condition as f64);
    out.set(
        "tech_era",
        settlement
            .map(|entry| entry.current_era.clone())
            .unwrap_or_else(|| "stone_age".to_string()),
    );
    out.set(
        "storage",
        stockpile_storage_dict(settlement),
    );
    out
}

/// Returns a bridge-friendly world summary snapshot for HUD and stats UI.
pub(crate) fn world_summary(state: &RuntimeState) -> VarDictionary {
    let mut out = VarDictionary::new();
    let world = state.engine.world();
    let resources = state.engine.resources();

    let members = collect_alive_members(world);
    let aggregates = member_aggregates(&members);
    let mut settlement_summaries = Array::<VarDictionary>::new();
    for settlement in resources.settlements.values() {
        let detail = settlement_detail(state, settlement.id.0 as i64);
        let mut summary = VarDictionary::new();
        summary.set("id", settlement.id.0 as i64);
        summary.set("settlement", detail.clone());
        summary.set("pop", detail.get("population").unwrap_or_default());
        summary.set("male", detail.get("male_count").unwrap_or_default());
        summary.set("female", detail.get("female_count").unwrap_or_default());
        summary.set("avg_happiness", detail.get("avg_happiness").unwrap_or_default());
        summary.set("avg_stress", detail.get("avg_stress").unwrap_or_default());
        summary.set("leader", detail.get("leader").unwrap_or_default());
        summary.set("tech_era", detail.get("tech_era").unwrap_or_default());
        summary.set("food", settlement.stockpile_food);
        summary.set("wood", settlement.stockpile_wood);
        summary.set("stone", settlement.stockpile_stone);
        settlement_summaries.push(&summary);
    }

    let history = runtime_history_array(&resources.stats_history);
    let mut resource_deltas = VarDictionary::new();
    if resources.stats_history.len() >= 2 {
        let latest = &resources.stats_history[resources.stats_history.len() - 1];
        let previous = &resources.stats_history[resources.stats_history.len() - 2];
        resource_deltas.set("food", latest.food - previous.food);
        resource_deltas.set("wood", latest.wood - previous.wood);
        resource_deltas.set("stone", latest.stone - previous.stone);
    } else {
        resource_deltas.set("food", 0.0_f64);
        resource_deltas.set("wood", 0.0_f64);
        resource_deltas.set("stone", 0.0_f64);
    }
    let total_food = resources
        .settlements
        .values()
        .map(|s| s.stockpile_food.max(0.0))
        .sum::<f64>();
    let total_wood = resources
        .settlements
        .values()
        .map(|s| s.stockpile_wood.max(0.0))
        .sum::<f64>();
    let total_stone = resources
        .settlements
        .values()
        .map(|s| s.stockpile_stone.max(0.0))
        .sum::<f64>();
    out.set("tick", state.engine.current_tick() as i64);
    out.set("total_population", aggregates.population as i64);
    out.set("peak_pop", resources.stats_peak_population as i64);
    out.set("total_births", resources.stats_total_births as i64);
    out.set("total_deaths", resources.stats_total_deaths as i64);
    out.set("total_male", aggregates.male as i64);
    out.set("total_female", aggregates.female as i64);
    out.set("avg_age_years", aggregates.avg_age_years);
    out.set("avg_happiness", aggregates.avg_happiness);
    out.set("avg_stress", aggregates.avg_stress);
    out.set("entity_count", world.len() as i64);
    out.set("settlement_count", resources.settlements.len() as i64);
    out.set("building_count", resources.buildings.len() as i64);
    out.set("food", total_food);
    out.set("wood", total_wood);
    out.set("stone", total_stone);
    out.set("global_food", total_food);
    out.set("global_wood", total_wood);
    out.set("global_stone", total_stone);
    out.set("resource_deltas", resource_deltas);
    out.set("age_distribution", age_distribution_dict(&members));
    out.set("settlement_summaries", settlement_summaries);
    out.set("history", history);
    out
}

/// Returns a compact minimap payload copied from the runtime state.
pub(crate) fn minimap_snapshot(state: &RuntimeState) -> VarDictionary {
    let mut out = VarDictionary::new();
    let map = &state.engine.resources().map;
    let world = state.engine.world();

    let mut biomes = PackedInt32Array::new();
    for y in 0..map.height {
        for x in 0..map.width {
            biomes.push(biome_id_from_terrain(map.get(x, y).terrain));
        }
    }

    let mut buildings = Array::<VarDictionary>::new();
    for building in state.engine.resources().buildings.values() {
        let mut d = VarDictionary::new();
        d.set("id", building.id.0 as i64);
        d.set("building_type", building.building_type.clone());
        d.set("tile_x", building.x as i64);
        d.set("tile_y", building.y as i64);
        buildings.push(&d);
    }

    let mut entities = Array::<VarDictionary>::new();
    for (entity, (position, behavior_opt, age_opt)) in
        world.query::<(&Position, Option<&Behavior>, Option<&Age>)>().iter()
    {
        if age_opt.map(|age| !age.alive).unwrap_or(false) {
            continue;
        }
        let mut d = VarDictionary::new();
        d.set("entity_id", entity.to_bits().get() as i64);
        d.set("x", position.x);
        d.set("y", position.y);
        d.set(
            "job",
            behavior_opt
                .map(|behavior| behavior.job.clone())
                .unwrap_or_else(|| "none".to_string()),
        );
        entities.push(&d);
    }

    let mut settlements = Array::<VarDictionary>::new();
    for settlement in state.engine.resources().settlements.values() {
        let mut d = VarDictionary::new();
        d.set("id", settlement.id.0 as i64);
        d.set("center_x", settlement.x as i64);
        d.set("center_y", settlement.y as i64);
        settlements.push(&d);
    }

    out.set("width", map.width as i64);
    out.set("height", map.height as i64);
    out.set("biomes", biomes);
    out.set("buildings", buildings);
    out.set("entities", entities);
    out.set("settlements", settlements);
    out
}

fn sync_world_tiles(resources: &mut sim_engine::SimResources, payload: &RuntimeBootstrapWorld) {
    for y in 0..payload.height {
        for x in 0..payload.width {
            let idx = (y * payload.width + x) as usize;
            let biome = payload.biomes.get(idx).copied().unwrap_or(BIOME_GRASSLAND);
            let tile = resources.map.get_mut(x, y);
            tile.terrain = terrain_from_biome_id(biome);
            tile.elevation = payload.elevation.get(idx).copied().unwrap_or(0.5);
            tile.moisture = payload.moisture.get(idx).copied().unwrap_or(0.5);
            tile.temperature = payload.temperature.get(idx).copied().unwrap_or(0.5);
            tile.passable = move_cost_from_biome_id(biome) > 0.0;
            tile.move_cost = move_cost_from_biome_id(biome);
            tile.resources.clear();
            let food = payload.food.get(idx).copied().unwrap_or(0.0);
            let wood = payload.wood.get(idx).copied().unwrap_or(0.0);
            let stone = payload.stone.get(idx).copied().unwrap_or(0.0);
            if food > 0.0 {
                tile.resources.push(TileResource {
                    resource_type: sim_core::enums::ResourceType::Food,
                    amount: f64::from(food),
                    max_amount: f64::from(food),
                    regen_rate: 0.0,
                });
            }
            if wood > 0.0 {
                tile.resources.push(TileResource {
                    resource_type: sim_core::enums::ResourceType::Wood,
                    amount: f64::from(wood),
                    max_amount: f64::from(wood),
                    regen_rate: 0.0,
                });
            }
            if stone > 0.0 {
                tile.resources.push(TileResource {
                    resource_type: sim_core::enums::ResourceType::Stone,
                    amount: f64::from(stone),
                    max_amount: f64::from(stone),
                    regen_rate: 0.0,
                });
            }
        }
    }
}

fn terrain_from_biome_id(biome: i32) -> TerrainType {
    match biome {
        BIOME_DEEP_WATER => TerrainType::DeepWater,
        BIOME_SHALLOW_WATER => TerrainType::ShallowWater,
        BIOME_BEACH => TerrainType::Beach,
        BIOME_GRASSLAND => TerrainType::Grassland,
        BIOME_FOREST => TerrainType::Forest,
        BIOME_DENSE_FOREST => TerrainType::DenseForest,
        BIOME_HILL => TerrainType::Hill,
        BIOME_MOUNTAIN => TerrainType::Mountain,
        BIOME_SNOW => TerrainType::Snow,
        _ => TerrainType::Grassland,
    }
}

fn biome_id_from_terrain(terrain: TerrainType) -> i32 {
    match terrain {
        TerrainType::DeepWater => BIOME_DEEP_WATER,
        TerrainType::ShallowWater => BIOME_SHALLOW_WATER,
        TerrainType::Beach => BIOME_BEACH,
        TerrainType::Grassland => BIOME_GRASSLAND,
        TerrainType::Forest => BIOME_FOREST,
        TerrainType::DenseForest => BIOME_DENSE_FOREST,
        TerrainType::Hill => BIOME_HILL,
        TerrainType::Mountain => BIOME_MOUNTAIN,
        TerrainType::Snow => BIOME_SNOW,
    }
}

fn move_cost_from_biome_id(biome: i32) -> f32 {
    match biome {
        BIOME_DEEP_WATER => BIOME_MOVE_COST_DEEP_WATER,
        BIOME_SHALLOW_WATER => BIOME_MOVE_COST_SHALLOW_WATER,
        BIOME_BEACH => BIOME_MOVE_COST_BEACH,
        BIOME_GRASSLAND => BIOME_MOVE_COST_GRASSLAND,
        BIOME_FOREST => BIOME_MOVE_COST_FOREST,
        BIOME_DENSE_FOREST => BIOME_MOVE_COST_DENSE_FOREST,
        BIOME_HILL => BIOME_MOVE_COST_HILL,
        BIOME_MOUNTAIN => BIOME_MOVE_COST_MOUNTAIN,
        BIOME_SNOW => BIOME_MOVE_COST_SNOW,
        _ => BIOME_MOVE_COST_GRASSLAND,
    }
}

fn pick_leader(world: &World, spawned: &[Entity]) -> Option<Entity> {
    let mut best: Option<(Entity, u64)> = None;
    for entity in spawned {
        let Ok(age) = world.get::<&Age>(*entity) else {
            continue;
        };
        if matches!(age.stage, sim_core::enums::GrowthStage::Adult | sim_core::enums::GrowthStage::Elder) {
            let score = age.ticks;
            if best.map(|(_, best_score)| score > best_score).unwrap_or(true) {
                best = Some((*entity, score));
            }
        }
    }
    best.map(|(entity, _)| entity).or_else(|| spawned.first().copied())
}

fn seed_initial_relationships(world: &mut World, spawned: &[Entity]) {
    if spawned.len() < 2 {
        return;
    }

    let mut adult_males: Vec<Entity> = Vec::new();
    let mut adult_females: Vec<Entity> = Vec::new();
    for entity in spawned {
        let Ok(identity) = world.get::<&Identity>(*entity) else {
            continue;
        };
        let Ok(age) = world.get::<&Age>(*entity) else {
            continue;
        };
        if !matches!(age.stage, sim_core::enums::GrowthStage::Adult | sim_core::enums::GrowthStage::Elder) {
            continue;
        }
        match identity.sex {
            Sex::Male => adult_males.push(*entity),
            Sex::Female => adult_females.push(*entity),
        }
    }

    if let (Some(male), Some(female)) = (adult_males.first().copied(), adult_females.first().copied()) {
        apply_relationship(world, male, female, 90.0, 1.0, RelationType::Spouse, true);
    }

    for pair in spawned.chunks(2).take(4) {
        if pair.len() == 2 {
            apply_relationship(world, pair[0], pair[1], 45.0, 0.7, RelationType::Friend, false);
        }
    }
}

fn apply_relationship(
    world: &mut World,
    left: Entity,
    right: Entity,
    affinity: f64,
    trust: f64,
    relation_type: RelationType,
    set_spouse: bool,
) {
    let right_id = EntityId(right.id() as u64);
    let left_id = EntityId(left.id() as u64);

    {
        let Ok(mut left_social) = world.get::<&mut Social>(left) else {
            return;
        };
        let edge = left_social.get_or_create_edge(right_id);
        edge.affinity = affinity;
        edge.trust = trust;
        edge.familiarity = 1.0;
        edge.relation_type = relation_type;
        if set_spouse {
            left_social.spouse = Some(right_id);
        }
    }
    {
        let Ok(mut right_social) = world.get::<&mut Social>(right) else {
            return;
        };
        let edge = right_social.get_or_create_edge(left_id);
        edge.affinity = affinity;
        edge.trust = trust;
        edge.familiarity = 1.0;
        edge.relation_type = relation_type;
        if set_spouse {
            right_social.spouse = Some(left_id);
        }
    }
}

pub(crate) fn build_raw_entity_id_lookup(world: &World) -> HashMap<u64, u64> {
    let mut lookup = HashMap::new();
    for (entity, _) in world.query::<()>().iter() {
        lookup.insert(entity.id() as u64, entity.to_bits().get());
    }
    lookup
}

/// Converts a stored raw hecs entity id into the bridge-facing runtime bits id.
pub(crate) fn runtime_bits_from_raw_id(lookup: &HashMap<u64, u64>, raw_id: u64) -> Option<i64> {
    lookup.get(&raw_id).copied().map(|bits| bits as i64)
}

fn collect_member_summaries(
    world: &World,
    settlement: &Settlement,
    raw_lookup: &HashMap<u64, u64>,
) -> Array<VarDictionary> {
    let mut members = Array::<VarDictionary>::new();
    for member_id in &settlement.members {
        let Some(runtime_id) = runtime_bits_from_raw_id(raw_lookup, member_id.0) else {
            continue;
        };
        if let Some(member) = member_summary_by_runtime_id(world, runtime_id) {
            members.push(&member);
        }
    }
    members
}

fn collect_alive_members(world: &World) -> Array<VarDictionary> {
    let mut members = Array::<VarDictionary>::new();
    for (entity, (identity, age)) in world.query::<(&Identity, &Age)>().iter() {
        if !age.alive {
            continue;
        }
        if let Some(summary) = member_summary(world, entity, identity, age) {
            members.push(&summary);
        }
    }
    members
}

fn member_summary_by_runtime_id(world: &World, entity_id_raw: i64) -> Option<VarDictionary> {
    let entity = Entity::from_bits(entity_id_raw.max(0) as u64)?;
    let identity = world.get::<&Identity>(entity).ok()?;
    let age = world.get::<&Age>(entity).ok()?;
    member_summary(world, entity, &identity, &age)
}

fn member_summary(world: &World, entity: Entity, identity: &Identity, age: &Age) -> Option<VarDictionary> {
    if !age.alive {
        return None;
    }
    let mut out = VarDictionary::new();
    out.set("id", entity.to_bits().get() as i64);
    out.set("entity_name", identity.name.clone());
    out.set("name", identity.name.clone());
    out.set("gender", format!("{:?}", identity.sex).to_lowercase());
    out.set("settlement_id", identity.settlement_id.map(|id| id.0 as i64).unwrap_or(-1));
    out.set("is_alive", true);
    out.set("age", age.ticks as i64);
    out.set("age_years", age.years);
    out.set("age_stage", format!("{:?}", age.stage).to_lowercase());

    let mut personality = VarDictionary::new();
    if let Ok(component) = world.get::<&Personality>(entity) {
        let mut axes = VarDictionary::new();
        axes.set("H", component.axes[0]);
        axes.set("E", component.axes[1]);
        axes.set("X", component.axes[2]);
        axes.set("A", component.axes[3]);
        axes.set("C", component.axes[4]);
        axes.set("O", component.axes[5]);
        personality.set("axes", axes);
    }
    out.set("personality", personality);

    let mut emotions = VarDictionary::new();
    if let Ok(component) = world.get::<&Emotion>(entity) {
        emotions.set("happiness", component.primary[0]);
    }
    if let Ok(component) = world.get::<&Stress>(entity) {
        emotions.set("stress", component.level);
    }
    out.set("emotions", emotions);

    let mut inventory = VarDictionary::new();
    inventory.set("food", 0.0);
    inventory.set("wood", 0.0);
    inventory.set("stone", 0.0);
    out.set("inventory", inventory);
    let mut skill_levels = VarDictionary::new();
    if let Ok(component) = world.get::<&Skills>(entity) {
        for (skill_id, entry) in &component.entries {
            skill_levels.set(skill_id.clone(), entry.level as i64);
        }
    }
    out.set("skill_levels", skill_levels);

    let job = world
        .get::<&Behavior>(entity)
        .map(|behavior| behavior.job.clone())
        .unwrap_or_else(|_| "none".to_string());
    out.set("job", job);
    Some(out)
}

fn collect_building_summaries(state: &RuntimeState, settlement: &Settlement) -> Array<VarDictionary> {
    let mut buildings = Array::<VarDictionary>::new();
    for building_id in &settlement.buildings {
        if let Some(building) = state.engine.resources().buildings.get(building_id) {
            let mut out = VarDictionary::new();
            out.set("id", building.id.0 as i64);
            out.set("building_type", building.building_type.clone());
            out.set("tile_x", building.x as i64);
            out.set("tile_y", building.y as i64);
            out.set("settlement_id", building.settlement_id.0 as i64);
            out.set("is_constructed", building.is_complete);
            out.set("is_built", building.is_complete);
            out.set("construction_progress", building.construction_progress as f64);
            out.set("build_progress", building.construction_progress as f64);
            out.set("storage", stockpile_storage_dict(Some(settlement)));
            buildings.push(&out);
        }
    }
    buildings
}

fn stockpile_storage_dict(settlement: Option<&Settlement>) -> VarDictionary {
    let mut storage = VarDictionary::new();
    storage.set("food", settlement.map(|entry| entry.stockpile_food).unwrap_or(0.0));
    storage.set("wood", settlement.map(|entry| entry.stockpile_wood).unwrap_or(0.0));
    storage.set("stone", settlement.map(|entry| entry.stockpile_stone).unwrap_or(0.0));
    storage
}

fn tech_states_dict(settlement: &Settlement) -> VarDictionary {
    let mut out = VarDictionary::new();
    for (tech_id, state) in &settlement.tech_states {
        let mut entry = VarDictionary::new();
        entry.set("state", tech_state_label(*state));
        entry.set("tech_id", tech_id.clone());
        entry.set("discovered_tick", 0_i64);
        entry.set("discoverer_id", -1_i64);
        entry.set("practitioner_count", 0_i64);
        entry.set("effective_carriers", 0_i64);
        entry.set("atrophy_years", 0_i64);
        entry.set("cultural_memory", 1.0_f64);
        entry.set("last_active_use_tick", 0_i64);
        entry.set("rediscovered_count", 0_i64);
        entry.set("acquisition_method", "invented");
        entry.set("source_settlement_id", -1_i64);
        entry.set("propagation_rate", 0.0_f64);
        entry.set("adoption_curve_phase", "innovator");
        entry.set("total_ever_learned", 0_i64);
        entry.set("cross_settlement_sources", Array::<i64>::new());
        out.set(tech_id.clone(), entry);
    }
    out
}

fn tech_state_label(state: TechState) -> &'static str {
    match state {
        TechState::Unknown => "unknown",
        TechState::KnownLow => "known_low",
        TechState::KnownStable => "known_stable",
        TechState::ForgottenRecent => "forgotten_recent",
        TechState::ForgottenLong => "forgotten_long",
    }
}

fn int_id_array_from_u64_ids<I>(ids: I) -> Array<i64>
where
    I: Iterator<Item = u64>,
{
    let mut out = Array::<i64>::new();
    for id in ids {
        out.push(id as i64);
    }
    out
}

struct MemberAggregates {
    population: usize,
    adults: usize,
    children: usize,
    teens: usize,
    elders: usize,
    male: usize,
    female: usize,
    avg_age_years: f64,
    avg_happiness: f64,
    avg_stress: f64,
}

fn settlement_aggregates(members: &Array<VarDictionary>) -> MemberAggregates {
    member_aggregates(members)
}

fn member_aggregates(members: &Array<VarDictionary>) -> MemberAggregates {
    let mut adults = 0_usize;
    let mut children = 0_usize;
    let mut teens = 0_usize;
    let mut elders = 0_usize;
    let mut male = 0_usize;
    let mut female = 0_usize;
    let mut total_age_years = 0.0_f64;
    let mut total_happiness = 0.0_f64;
    let mut total_stress = 0.0_f64;

    for member in members.iter_shared() {
        let age_years = member
            .get("age_years")
            .map(|value| value.to::<f64>())
            .unwrap_or_default();
        let age_stage = member
            .get("age_stage")
            .map(|value| value.to::<String>())
            .unwrap_or_default();
        let gender = member
            .get("gender")
            .map(|value| value.to::<String>())
            .unwrap_or_default();
        if gender == "male" {
            male += 1;
        } else if gender == "female" {
            female += 1;
        }
        match age_stage.as_str() {
            "child" | "infant" | "toddler" => children += 1,
            "teen" => teens += 1,
            "elder" => elders += 1,
            _ => adults += 1,
        }
        total_age_years += age_years;
        let emotions = member
            .get("emotions")
            .map(|value| value.to::<VarDictionary>())
            .unwrap_or_default();
        total_happiness += emotions.get("happiness").map(|value| value.to::<f64>()).unwrap_or(0.0);
        total_stress += emotions.get("stress").map(|value| value.to::<f64>()).unwrap_or(0.0);
    }

    let population = members.len();
    MemberAggregates {
        population,
        adults,
        children,
        teens,
        elders,
        male,
        female,
        avg_age_years: if population > 0 {
            total_age_years / population as f64
        } else {
            0.0
        },
        avg_happiness: if population > 0 {
            total_happiness / population as f64
        } else {
            0.0
        },
        avg_stress: if population > 0 {
            total_stress / population as f64
        } else {
            0.0
        },
    }
}

fn age_distribution_dict(members: &Array<VarDictionary>) -> VarDictionary {
    let mut out = VarDictionary::new();
    let aggregates = member_aggregates(members);
    out.set("child", aggregates.children as i64);
    out.set("teen", aggregates.teens as i64);
    out.set("adult", aggregates.adults as i64);
    out.set("elder", aggregates.elders as i64);
    out
}

fn runtime_history_array(history: &[RuntimeStatsSnapshot]) -> Array<VarDictionary> {
    let mut out = Array::<VarDictionary>::new();
    for snapshot in history {
        let mut entry = VarDictionary::new();
        entry.set("tick", snapshot.tick as i64);
        entry.set("total_population", snapshot.pop as i64);
        entry.set("pop", snapshot.pop as i64);
        entry.set("food", snapshot.food);
        entry.set("wood", snapshot.wood);
        entry.set("stone", snapshot.stone);
        out.push(&entry);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_registry::RuntimeConfig;
    use sim_core::config::TICKS_PER_YEAR;

    fn test_bootstrap_payload() -> RuntimeBootstrapPayload {
        RuntimeBootstrapPayload {
            world: RuntimeBootstrapWorld {
                width: 4,
                height: 4,
                biomes: vec![BIOME_GRASSLAND; 16],
                elevation: vec![0.5; 16],
                moisture: vec![0.5; 16],
                temperature: vec![0.5; 16],
                food: vec![1.0; 16],
                wood: vec![2.0; 16],
                stone: vec![3.0; 16],
            },
            founding_settlement: RuntimeBootstrapSettlement {
                id: 1,
                name: "Settlement 1".to_string(),
                x: 2,
                y: 2,
                stockpile_food: 15.0,
                stockpile_wood: 5.0,
                stockpile_stone: 2.0,
            },
            agents: vec![
                RuntimeBootstrapAgent {
                    x: 1,
                    y: 1,
                    age_ticks: 20 * TICKS_PER_YEAR as u64,
                },
                RuntimeBootstrapAgent {
                    x: 2,
                    y: 1,
                    age_ticks: 24 * TICKS_PER_YEAR as u64,
                },
                RuntimeBootstrapAgent {
                    x: 2,
                    y: 2,
                    age_ticks: 12 * TICKS_PER_YEAR as u64,
                },
            ],
        }
    }

    #[test]
    fn biome_mapping_round_trips_known_ids() {
        assert_eq!(terrain_from_biome_id(BIOME_DEEP_WATER), TerrainType::DeepWater);
        assert_eq!(biome_id_from_terrain(TerrainType::DenseForest), BIOME_DENSE_FOREST);
        assert_eq!(move_cost_from_biome_id(BIOME_MOUNTAIN), BIOME_MOVE_COST_MOUNTAIN);
    }

    #[test]
    fn raw_entity_id_lookup_maps_to_runtime_bits() {
        let mut state = RuntimeState::from_seed(7, RuntimeConfig::default());
        let entity = state.engine.world_mut().spawn(());
        let lookup = build_raw_entity_id_lookup(state.engine.world());
        assert_eq!(
            runtime_bits_from_raw_id(&lookup, entity.id() as u64),
            Some(entity.to_bits().get() as i64)
        );
    }

    #[test]
    fn bootstrap_world_core_populates_runtime_state() {
        let mut state = RuntimeState::from_seed(7, RuntimeConfig::default());
        let result = bootstrap_world_core(&mut state, test_bootstrap_payload()).expect("bootstrap should succeed");
        assert_eq!(result.entity_count, 3);
        assert_eq!(result.building_count, 1);
        assert_eq!(result.settlement_id, 1);

        let resources = state.engine.resources();
        let settlement = resources
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should exist");
        assert_eq!(settlement.members.len(), 3);
        assert_eq!(settlement.buildings.len(), 1);
        assert_eq!(settlement.stockpile_food, 15.0);
        assert_eq!(settlement.stockpile_wood, 5.0);
        assert_eq!(settlement.stockpile_stone, 2.0);
        assert!(settlement.leader_id.is_some());
        assert_eq!(resources.buildings.len(), 1);
        assert_eq!(state.engine.world().len(), 3);
    }
}
