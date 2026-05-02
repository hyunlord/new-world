use godot::builtin::{Array, GString, PackedInt32Array, PackedStringArray, VarDictionary};
use hecs::{Entity, World};
use sim_core::components::{
    Age, AgentKnowledge, Behavior, Emotion, Identity, Needs, Personality, Position, Skills, Social,
    Stress,
};
use sim_core::config;
use sim_core::enums::{ActionType, NeedType, RelationType, Sex, TechState, TerrainType};
use sim_core::world::TileResource;
use sim_core::{Building, BuildingId, EntityId, Settlement, SettlementId, Temperament};
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
    #[serde(default)]
    pub(crate) startup_mode: RuntimeBootstrapMode,
    pub(crate) world: RuntimeBootstrapWorld,
    pub(crate) founding_settlement: RuntimeBootstrapSettlement,
    #[serde(default)]
    pub(crate) agents: Vec<RuntimeBootstrapAgent>,
    /// Scenario ruleset name to activate at bootstrap time.
    /// Maps GDScript scenario_id → Rust WorldRuleset name:
    ///   "default"          → no scenario (base rules only)
    ///   "eternal_winter"   → "EternalWinter"
    ///   "perpetual_summer" → "PerpetualSummer"
    ///   "barren_world"     → "BarrenWorld"
    ///   "abundance"        → "Abundance"
    #[serde(default)]
    pub(crate) scenario_id: String,
}

/// Startup bootstrap modes exposed from the Godot setup flow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeBootstrapMode {
    Probe,
    #[default]
    Sandbox,
}

impl RuntimeBootstrapMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Probe => "probe",
            Self::Sandbox => "sandbox",
        }
    }
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
    #[serde(default)]
    pub(crate) sex: Option<RuntimeBootstrapSex>,
}

/// Optional spawn-time sex override carried by the Godot bootstrap payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeBootstrapSex {
    Male,
    Female,
}

impl RuntimeBootstrapSex {
    fn as_sim_sex(self) -> Sex {
        match self {
            Self::Male => Sex::Male,
            Self::Female => Sex::Female,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootstrapWorldResult {
    spawned: usize,
    settlement_id: u64,
    building_count: usize,
    entity_count: usize,
    startup_mode: RuntimeBootstrapMode,
}

#[derive(Debug, Clone, Copy)]
struct ProbeBootstrapProfile {
    hunger: f64,
    sleep: f64,
    energy: f64,
}

fn probe_bootstrap_profile(index: usize) -> Option<ProbeBootstrapProfile> {
    match index {
        0 => Some(ProbeBootstrapProfile {
            hunger: config::PROBE_START_PRIMARY_HUNGER,
            sleep: 0.80,
            energy: config::PROBE_START_PRIMARY_ENERGY,
        }),
        1 => Some(ProbeBootstrapProfile {
            hunger: config::PROBE_START_SECONDARY_HUNGER,
            sleep: config::PROBE_START_SECONDARY_SLEEP,
            energy: config::PROBE_START_SECONDARY_ENERGY,
        }),
        _ => None,
    }
}

fn apply_probe_survival_bootstrap(world: &mut World, entity: Entity, index: usize) {
    let Some(profile) = probe_bootstrap_profile(index) else {
        return;
    };
    let Ok(mut query) = world.query_one::<&mut Needs>(entity) else {
        return;
    };
    let Some(needs) = query.get() else {
        return;
    };
    needs.set(NeedType::Hunger, profile.hunger);
    needs.set(NeedType::Sleep, profile.sleep);
    needs.set(NeedType::Warmth, config::PROBE_START_BASE_WARMTH);
    needs.set(NeedType::Safety, config::PROBE_START_BASE_SAFETY);
    needs.energy = profile.energy.clamp(0.0, 1.0);
}

/// Applies startup bootstrap data into the live runtime and returns a summary.
pub(crate) fn bootstrap_world(
    state: &mut RuntimeState,
    payload: RuntimeBootstrapPayload,
) -> VarDictionary {
    let mut out = VarDictionary::new();
    let Some(result) = bootstrap_world_core(state, payload) else {
        return out;
    };

    out.set("spawned", result.spawned as i64);
    out.set("settlement_id", result.settlement_id as i64);
    out.set("building_count", result.building_count as i64);
    out.set("entity_count", result.entity_count as i64);
    out.set("startup_mode", result.startup_mode.as_str());
    out
}

/// Generates a Korean place-name for a settlement based on its ID.
/// 20 unique names before appending a numeric suffix.
pub(crate) fn generate_settlement_name(id: sim_core::SettlementId) -> String {
    const NAMES: &[&str] = &[
        "돌무지", "새벽골", "물가마을", "높은둔덕", "갈대밭", "바람재", "솔밭", "어울터",
        "큰바위", "나루터", "별빛골", "참나무", "이끼언덕", "갈밭", "여울목", "달빛마을",
        "소나무골", "안골", "고인돌", "둥지",
    ];
    let idx = id.0.saturating_sub(1) as usize;
    if idx < NAMES.len() {
        NAMES[idx].to_string()
    } else {
        format!("{} {}", NAMES[idx % NAMES.len()], idx / NAMES.len() + 1)
    }
}

fn bootstrap_world_core(
    state: &mut RuntimeState,
    payload: RuntimeBootstrapPayload,
) -> Option<BootstrapWorldResult> {
    let startup_mode = payload.startup_mode;
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
        resources.map = sim_core::WorldMap::new(
            payload.world.width,
            payload.world.height,
            resources.map.seed,
        );
        resources.settlements.clear();
        resources.buildings.clear();
        resources.band_store = sim_core::BandStore::new();
        resources.tension_pairs.clear();
        resources.stats_history.clear();
        resources.stats_peak_population = 0;
        resources.stats_total_births = 0;
        resources.stats_total_deaths = 0;
        resources.stat_sync_derived.clear();
        resources.stat_threshold_flags.clear();
        resources.chronicle_log.clear();
        resources.chronicle_timeline.clear();
        sync_world_tiles(resources, &payload.world);
    }

    // Activate the requested scenario ruleset (non-default only).
    let scenario_ruleset_name = match payload.scenario_id.as_str() {
        "eternal_winter" => Some("EternalWinter"),
        "perpetual_summer" => Some("PerpetualSummer"),
        "barren_world" => Some("BarrenWorld"),
        "abundance" => Some("Abundance"),
        _ => None,
    };
    if let Some(name) = scenario_ruleset_name {
        if let Err(e) = state.engine.resources_mut().activate_scenario_by_name(name) {
            log::warn!("[bootstrap] activate_scenario '{}' failed: {}", name, e);
        }
    }

    // Validate settlement location has stone access; shift spawn point if needed.
    let (sx, sy) = state.engine.resources().map.find_settlement_location_with_stone(
        payload.founding_settlement.x,
        payload.founding_settlement.y,
        config::SETTLEMENT_STONE_ACCESS_RADIUS,
    );

    let settlement_id = SettlementId(payload.founding_settlement.id.max(1));
    let mut settlement = Settlement::new(
        settlement_id,
        if payload.founding_settlement.name.trim().is_empty() {
            generate_settlement_name(settlement_id)
        } else {
            payload.founding_settlement.name.clone()
        },
        sx,
        sy,
        state.engine.current_tick(),
    );
    settlement.stockpile_food = payload.founding_settlement.stockpile_food.max(0.0);
    settlement.stockpile_wood = payload.founding_settlement.stockpile_wood.max(0.0);
    settlement.stockpile_stone = payload.founding_settlement.stockpile_stone.max(0.0);

    // Initialize all stone-age techs as Unknown so TechDiscovery can find them
    for tech_id in sim_core::STONE_AGE_TECH_IDS {
        settlement.tech_states.insert(
            tech_id.to_string(),
            sim_core::TechState::Unknown,
        );
    }

    let stockpile_building_id = BuildingId(1);
    if settlement.stockpile_food > 0.0
        || settlement.stockpile_wood > 0.0
        || settlement.stockpile_stone > 0.0
    {
        let mut stockpile = Building::new(
            stockpile_building_id,
            "stockpile".to_string(),
            settlement_id,
            payload.founding_settlement.x,
            payload.founding_settlement.y,
            2,
            2,
            state.engine.current_tick(),
        );
        stockpile.construction_progress = 1.0;
        stockpile.is_complete = true;
        settlement.buildings.push(stockpile_building_id);
        state
            .engine
            .resources_mut()
            .buildings
            .insert(stockpile_building_id, stockpile);
    }

    let mut spawned_entities: Vec<Entity> = Vec::with_capacity(payload.agents.len());
    for (spawn_index, agent) in payload.agents.iter().enumerate() {
        let config = SpawnConfig {
            settlement_id: Some(settlement_id),
            position: (agent.x, agent.y),
            initial_age_ticks: agent.age_ticks,
            sex: agent.sex.map(RuntimeBootstrapSex::as_sim_sex),
            parent_a: None,
            parent_b: None,
        };
        let (world, resources) = state.engine.world_and_resources_mut();
        let entity = entity_spawner::spawn_agent(world, resources, &config);
        if startup_mode == RuntimeBootstrapMode::Probe {
            apply_probe_survival_bootstrap(world, entity, spawn_index);
        }
        settlement.members.push(EntityId(entity.id() as u64));
        spawned_entities.push(entity);
    }

    // ── Sync agent knowledge → settlement tech_states ──
    {
        let world = state.engine.world();
        let mut tech_counts: HashMap<String, usize> = HashMap::new();
        for entity in &spawned_entities {
            if let Ok(knowledge) = world.get::<&AgentKnowledge>(*entity) {
                for entry in &knowledge.known {
                    *tech_counts.entry(entry.knowledge_id.clone()).or_insert(0) += 1;
                }
            }
        }
        let total = spawned_entities.len();
        for (tech_id, count) in &tech_counts {
            if settlement.tech_states.contains_key(tech_id) {
                let new_state = if *count >= 3 {
                    TechState::KnownStable
                } else {
                    TechState::KnownLow
                };
                settlement.tech_states.insert(tech_id.clone(), new_state);
            }
        }
        log::info!(
            "[Bootstrap] Settlement tech sync: {} agents, {} techs known (of {} total)",
            total,
            tech_counts.len(),
            settlement.tech_states.len()
        );
    }

    settlement.leader_id = pick_leader(state.engine.world(), &spawned_entities)
        .map(|entity| EntityId(entity.id() as u64));
    state
        .engine
        .resources_mut()
        .settlements
        .insert(settlement_id, settlement);
    seed_initial_relationships(state.engine.world_mut(), &spawned_entities);

    Some(BootstrapWorldResult {
        spawned: payload.agents.len(),
        settlement_id: settlement_id.0,
        building_count: state.engine.resources().buildings.len(),
        entity_count: state.engine.world().len() as usize,
        startup_mode,
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
    out.set(
        "building_ids",
        int_id_array_from_u64_ids(settlement.buildings.iter().map(|id| id.0)),
    );
    out.set(
        "member_ids",
        int_id_array_from_u64_ids(
            settlement.members.iter().filter_map(|id| {
                runtime_bits_from_raw_id(&raw_lookup, id.0).map(|bits| bits as u64)
            }),
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
    let progress_diag = state
        .engine
        .resources()
        .construction_diagnostics
        .get(&building_id)
        .copied()
        .unwrap_or_default();
    let current_tick = state.engine.current_tick();
    let ticks_since_progress = if progress_diag.last_progress_tick == 0 {
        current_tick
    } else {
        current_tick.saturating_sub(progress_diag.last_progress_tick)
    };
    let (settlement_builder_count, adjacent_builder_count, assigned_builders) =
        collect_construction_assignments(state.engine.world(), building);
    let stall_reason = construction_stall_reason(
        building,
        progress_diag.progress_delta,
        settlement_builder_count,
        assigned_builders.len() as u32,
        adjacent_builder_count,
        ticks_since_progress,
    );
    let settlement = state
        .engine
        .resources()
        .settlements
        .get(&building.settlement_id);
    out.set("id", building.id.0 as i64);
    out.set("building_type", building.building_type.clone());
    out.set("settlement_id", building.settlement_id.0 as i64);
    out.set("tile_x", building.x as i64);
    out.set("tile_y", building.y as i64);
    out.set("width", building.width as i64);
    out.set("height", building.height as i64);
    out.set("is_constructed", building.is_complete);
    out.set("is_built", building.is_complete);
    out.set(
        "construction_progress",
        building.construction_progress as f64,
    );
    out.set("build_progress", building.construction_progress as f64);
    out.set("construction_progress_delta", progress_diag.progress_delta);
    out.set("recent_progress_delta", progress_diag.progress_delta);
    out.set("ticks_since_progress", ticks_since_progress as i64);
    out.set(
        "construction_state",
        construction_state_label(building, progress_diag.progress_delta),
    );
    out.set("stall_reason", stall_reason);
    out.set("assigned_builder_count", assigned_builders.len() as i64);
    out.set("settlement_builder_count", settlement_builder_count as i64);
    out.set("adjacent_builder_count", adjacent_builder_count as i64);
    out.set(
        "assigned_builders",
        builder_assignment_array(&assigned_builders),
    );
    out.set("condition", building.condition as f64);
    out.set(
        "tech_era",
        settlement
            .map(|entry| entry.current_era.clone())
            .unwrap_or_else(|| "stone_age".to_string()),
    );
    out.set("storage", stockpile_storage_dict(settlement));
    out
}

#[derive(Debug, Clone)]
struct BuilderAssignmentSummary {
    runtime_id: i64,
    name: String,
    in_range: bool,
    distance_tiles: f64,
}

fn collect_construction_assignments(
    world: &World,
    building: &Building,
) -> (u32, u32, Vec<BuilderAssignmentSummary>) {
    let mut settlement_builder_count = 0_u32;
    let mut adjacent_builder_count = 0_u32;
    let mut assigned_builders: Vec<BuilderAssignmentSummary> = Vec::new();

    let mut query = world.query::<(&Identity, &Behavior, Option<&Age>, Option<&Position>)>();
    for (entity, (identity, behavior, age_opt, position_opt)) in &mut query {
        if identity.settlement_id != Some(building.settlement_id) {
            continue;
        }
        let Some(age) = age_opt else {
            continue;
        };
        if !age.alive || age.stage != sim_core::GrowthStage::Adult {
            continue;
        }
        if behavior.job == "builder" {
            settlement_builder_count = settlement_builder_count.saturating_add(1);
        }
        if behavior.current_action != ActionType::Build
            || behavior.action_target_x != Some(building.x)
            || behavior.action_target_y != Some(building.y)
        {
            continue;
        }

        let (in_range, distance_tiles) = position_opt
            .map(|position| {
                let dx = (position.x - f64::from(building.x)).abs();
                let dy = (position.y - f64::from(building.y)).abs();
                let chebyshev = dx.max(dy);
                (dx <= 1.0 && dy <= 1.0, chebyshev)
            })
            .unwrap_or((false, f64::INFINITY));
        if in_range {
            adjacent_builder_count = adjacent_builder_count.saturating_add(1);
        }
        assigned_builders.push(BuilderAssignmentSummary {
            runtime_id: entity.to_bits().get() as i64,
            name: identity.name.clone(),
            in_range,
            distance_tiles,
        });
    }

    assigned_builders.sort_by(|left, right| {
        left.distance_tiles
            .partial_cmp(&right.distance_tiles)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.name.cmp(&right.name))
    });

    (
        settlement_builder_count,
        adjacent_builder_count,
        assigned_builders,
    )
}

fn builder_assignment_array(assignments: &[BuilderAssignmentSummary]) -> Array<VarDictionary> {
    let mut out = Array::<VarDictionary>::new();
    for assignment in assignments {
        let mut row = VarDictionary::new();
        row.set("entity_id", assignment.runtime_id);
        row.set("name", assignment.name.clone());
        row.set("in_range", assignment.in_range);
        row.set("distance_tiles", assignment.distance_tiles);
        out.push(&row);
    }
    out
}

fn construction_state_label(building: &Building, progress_delta: f64) -> &'static str {
    if building.is_complete {
        "complete"
    } else if progress_delta > f64::EPSILON {
        "advancing"
    } else {
        "stalled"
    }
}

fn construction_stall_reason(
    building: &Building,
    progress_delta: f64,
    settlement_builder_count: u32,
    assigned_builder_count: u32,
    adjacent_builder_count: u32,
    ticks_since_progress: u64,
) -> &'static str {
    if building.is_complete {
        return "complete";
    }
    if progress_delta > f64::EPSILON {
        return "advancing";
    }
    if settlement_builder_count == 0 {
        return "no_builder";
    }
    if assigned_builder_count == 0 {
        return "priority_too_low";
    }
    if adjacent_builder_count == 0 {
        return "builder_travel";
    }
    if ticks_since_progress < config::CONSTRUCTION_TICK_INTERVAL {
        return "waiting_tick";
    }
    "unknown"
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
        summary.set(
            "avg_happiness",
            detail.get("avg_happiness").unwrap_or_default(),
        );
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
    out.set("band_count", resources.band_store.len() as i64);
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
        d.set("width", building.width as i64);
        d.set("height", building.height as i64);
        buildings.push(&d);
    }

    let mut entities = Array::<VarDictionary>::new();
    for (entity, (position, behavior_opt, age_opt)) in world
        .query::<(&Position, Option<&Behavior>, Option<&Age>)>()
        .iter()
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
        if matches!(
            age.stage,
            sim_core::enums::GrowthStage::Adult | sim_core::enums::GrowthStage::Elder
        ) {
            let score = age.ticks;
            if best
                .map(|(_, best_score)| score > best_score)
                .unwrap_or(true)
            {
                best = Some((*entity, score));
            }
        }
    }
    best.map(|(entity, _)| entity)
        .or_else(|| spawned.first().copied())
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
        if !matches!(
            age.stage,
            sim_core::enums::GrowthStage::Adult | sim_core::enums::GrowthStage::Elder
        ) {
            continue;
        }
        match identity.sex {
            Sex::Male => adult_males.push(*entity),
            Sex::Female => adult_females.push(*entity),
        }
    }

    if let (Some(male), Some(female)) =
        (adult_males.first().copied(), adult_females.first().copied())
    {
        apply_relationship(world, male, female, 90.0, 1.0, RelationType::Spouse, true);
    }

    for pair in spawned.chunks(2).take(4) {
        if pair.len() == 2 {
            apply_relationship(
                world,
                pair[0],
                pair[1],
                45.0,
                0.7,
                RelationType::Friend,
                false,
            );
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

pub(crate) fn runtime_band_id_raw(band_id: Option<sim_core::BandId>) -> i64 {
    band_id.map(|band_id| band_id.0 as i64).unwrap_or(-1)
}

fn member_summary(
    world: &World,
    entity: Entity,
    identity: &Identity,
    age: &Age,
) -> Option<VarDictionary> {
    if !age.alive {
        return None;
    }
    let mut out = VarDictionary::new();
    out.set("id", entity.to_bits().get() as i64);
    out.set("entity_name", identity.name.clone());
    out.set("name", identity.name.clone());
    out.set("gender", format!("{:?}", identity.sex).to_lowercase());
    out.set(
        "settlement_id",
        identity.settlement_id.map(|id| id.0 as i64).unwrap_or(-1),
    );
    out.set("band_id", runtime_band_id_raw(identity.band_id));
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

    // TCI temperament axes for UI display — uses shared helper for consistency
    // with runtime_get_entity_detail (same code path, same f64 precision).
    if let Ok(temperament) = world.get::<&Temperament>(entity) {
        let td = crate::temperament_detail::extract_temperament_detail(&temperament);
        out.set("tci_ns", td.tci_ns);
        out.set("tci_ha", td.tci_ha);
        out.set("tci_rd", td.tci_rd);
        out.set("tci_p", td.tci_p);
        out.set("temperament_label_key", td.temperament_label_key);
    }

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

fn collect_building_summaries(
    state: &RuntimeState,
    settlement: &Settlement,
) -> Array<VarDictionary> {
    let mut buildings = Array::<VarDictionary>::new();
    for building_id in &settlement.buildings {
        if let Some(building) = state.engine.resources().buildings.get(building_id) {
            let mut out = VarDictionary::new();
            out.set("id", building.id.0 as i64);
            out.set("building_type", building.building_type.clone());
            out.set("tile_x", building.x as i64);
            out.set("tile_y", building.y as i64);
            out.set("width", building.width as i64);
            out.set("height", building.height as i64);
            out.set("settlement_id", building.settlement_id.0 as i64);
            out.set("is_constructed", building.is_complete);
            out.set("is_built", building.is_complete);
            out.set(
                "construction_progress",
                building.construction_progress as f64,
            );
            out.set("build_progress", building.construction_progress as f64);
            out.set("storage", stockpile_storage_dict(Some(settlement)));
            buildings.push(&out);
        }
    }
    buildings
}

fn stockpile_storage_dict(settlement: Option<&Settlement>) -> VarDictionary {
    let mut storage = VarDictionary::new();
    storage.set(
        "food",
        settlement.map(|entry| entry.stockpile_food).unwrap_or(0.0),
    );
    storage.set(
        "wood",
        settlement.map(|entry| entry.stockpile_wood).unwrap_or(0.0),
    );
    storage.set(
        "stone",
        settlement.map(|entry| entry.stockpile_stone).unwrap_or(0.0),
    );
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
        total_happiness += emotions
            .get("happiness")
            .map(|value| value.to::<f64>())
            .unwrap_or(0.0);
        total_stress += emotions
            .get("stress")
            .map(|value| value.to::<f64>())
            .unwrap_or(0.0);
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

/// Returns tile-grid wall/floor/door/furniture data as packed arrays for
/// efficient GDScript rendering. Only non-empty tiles are included.
pub(crate) fn tile_grid_walls(state: &RuntimeState) -> VarDictionary {
    let resources = state.engine.resources();
    let (width, height) = resources.tile_grid.dimensions();

    let mut wall_x = PackedInt32Array::new();
    let mut wall_y = PackedInt32Array::new();
    let mut wall_material = PackedStringArray::new();

    let mut floor_x = PackedInt32Array::new();
    let mut floor_y = PackedInt32Array::new();

    let mut door_x = PackedInt32Array::new();
    let mut door_y = PackedInt32Array::new();

    let mut furniture_x = PackedInt32Array::new();
    let mut furniture_y = PackedInt32Array::new();
    let mut furniture_ids = PackedStringArray::new();

    for y in 0..height {
        for x in 0..width {
            let tile = resources.tile_grid.get(x, y);
            if let Some(ref mat) = tile.wall_material {
                wall_x.push(x as i32);
                wall_y.push(y as i32);
                wall_material.push(&GString::from(mat.as_str()));
            }
            if tile.floor_material.is_some() {
                floor_x.push(x as i32);
                floor_y.push(y as i32);
            }
            if tile.is_door {
                door_x.push(x as i32);
                door_y.push(y as i32);
            }
            if let Some(ref furn) = tile.furniture_id {
                furniture_x.push(x as i32);
                furniture_y.push(y as i32);
                furniture_ids.push(&GString::from(furn.as_str()));
            }
        }
    }

    // Compute adjacency counts for autotile bridge rendering
    let mut adj_right_count: i32 = 0;
    let mut adj_down_count: i32 = 0;
    for y in 0..height {
        for x in 0..width {
            if resources.tile_grid.get(x, y).wall_material.is_none() {
                continue;
            }
            if x + 1 < width && resources.tile_grid.get(x + 1, y).wall_material.is_some() {
                adj_right_count += 1;
            }
            if y + 1 < height && resources.tile_grid.get(x, y + 1).wall_material.is_some() {
                adj_down_count += 1;
            }
        }
    }

    let mut out = VarDictionary::new();
    out.set("wall_x", wall_x);
    out.set("wall_y", wall_y);
    out.set("wall_material", wall_material);
    out.set("floor_x", floor_x);
    out.set("floor_y", floor_y);
    out.set("door_x", door_x);
    out.set("door_y", door_y);
    out.set("furniture_x", furniture_x);
    out.set("furniture_y", furniture_y);
    out.set("furniture_id", furniture_ids);

    // Render config — Rust-authoritative values read by building_renderer.gd.
    // Uses building_render_config() for compile-time coupling with harness tests.
    let render_cfg = sim_core::config::building_render_config();
    out.set("render_floor_alpha", render_cfg.floor_alpha);
    out.set("render_floor_border_width", render_cfg.floor_border_width);
    out.set("render_furniture_icon_scale", render_cfg.furniture_icon_scale);
    out.set("render_wall_autotile", render_cfg.wall_autotile_enabled);
    out.set("render_wall_bridge_px", render_cfg.wall_autotile_bridge_px);
    out.set("wall_adj_right_count", adj_right_count as i64);
    out.set("wall_adj_down_count", adj_down_count as i64);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_registry::RuntimeConfig;
    use sim_core::components::{Age, Behavior, Identity, Position, Skills};
    use sim_core::config::TICKS_PER_YEAR;
    use sim_core::NeedType;
    use sim_core::{ActionType, BandId, Building, BuildingId, SettlementId};
    use sim_systems::runtime::{
        BehaviorRuntimeSystem, GatheringRuntimeSystem, MovementRuntimeSystem, NeedsRuntimeSystem,
    };

    fn test_bootstrap_payload() -> RuntimeBootstrapPayload {
        RuntimeBootstrapPayload {
            startup_mode: RuntimeBootstrapMode::Probe,
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
                    sex: Some(RuntimeBootstrapSex::Male),
                },
                RuntimeBootstrapAgent {
                    x: 2,
                    y: 1,
                    age_ticks: 24 * TICKS_PER_YEAR as u64,
                    sex: Some(RuntimeBootstrapSex::Female),
                },
            ],
            scenario_id: String::new(),
        }
    }

    #[test]
    fn biome_mapping_round_trips_known_ids() {
        assert_eq!(
            terrain_from_biome_id(BIOME_DEEP_WATER),
            TerrainType::DeepWater
        );
        assert_eq!(
            biome_id_from_terrain(TerrainType::DenseForest),
            BIOME_DENSE_FOREST
        );
        assert_eq!(
            move_cost_from_biome_id(BIOME_MOUNTAIN),
            BIOME_MOVE_COST_MOUNTAIN
        );
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
        let result = bootstrap_world_core(&mut state, test_bootstrap_payload())
            .expect("bootstrap should succeed");
        assert_eq!(result.entity_count, 2);
        assert_eq!(result.building_count, 1);
        assert_eq!(result.settlement_id, 1);
        assert_eq!(result.startup_mode, RuntimeBootstrapMode::Probe);

        let resources = state.engine.resources();
        let settlement = resources
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should exist");
        assert_eq!(settlement.members.len(), 2);
        assert_eq!(settlement.buildings.len(), 1);
        assert_eq!(settlement.stockpile_food, 15.0);
        assert_eq!(settlement.stockpile_wood, 5.0);
        assert_eq!(settlement.stockpile_stone, 2.0);
        assert!(settlement.leader_id.is_some());
        assert_eq!(resources.buildings.len(), 1);
        assert_eq!(state.engine.world().len(), 2);

        let mut male_count = 0;
        let mut female_count = 0;
        let mut query = state.engine.world().query::<&Identity>();
        for (_, identity) in &mut query {
            match identity.sex {
                Sex::Male => male_count += 1,
                Sex::Female => female_count += 1,
            }
        }
        assert_eq!(male_count, 1);
        assert_eq!(female_count, 1);
    }

    #[test]
    fn probe_bootstrap_applies_survival_profiles_to_initial_agents() {
        let mut state = RuntimeState::from_seed(7, RuntimeConfig::default());
        bootstrap_world_core(&mut state, test_bootstrap_payload()).expect("bootstrap should work");

        let mut entries: Vec<(String, f64, f64, f64)> = Vec::new();
        let mut query = state.engine.world().query::<(&Identity, &Needs)>();
        for (_, (identity, needs)) in &mut query {
            entries.push((
                identity.name.clone(),
                needs.get(NeedType::Hunger),
                needs.get(NeedType::Sleep),
                needs.energy,
            ));
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "Agent 1");
        assert!((entries[0].1 - config::PROBE_START_PRIMARY_HUNGER).abs() < 1e-6);
        assert!((entries[0].3 - config::PROBE_START_PRIMARY_ENERGY).abs() < 1e-6);

        assert_eq!(entries[1].0, "Agent 2");
        assert!((entries[1].1 - config::PROBE_START_SECONDARY_HUNGER).abs() < 1e-6);
        assert!((entries[1].2 - config::PROBE_START_SECONDARY_SLEEP).abs() < 1e-6);
        assert!((entries[1].3 - config::PROBE_START_SECONDARY_ENERGY).abs() < 1e-6);
    }

    #[test]
    fn probe_bootstrap_drives_visible_survival_actions_and_recovery() {
        let mut state = RuntimeState::from_seed(7, RuntimeConfig::default());
        bootstrap_world_core(&mut state, test_bootstrap_payload()).expect("bootstrap should work");

        state
            .engine
            .register(NeedsRuntimeSystem::new(10, config::NEEDS_TICK_INTERVAL));
        state.engine.register(BehaviorRuntimeSystem::new(
            20,
            config::BEHAVIOR_TICK_INTERVAL,
        ));
        state.engine.register(GatheringRuntimeSystem::new(
            25,
            config::GATHERING_TICK_INTERVAL,
        ));
        state.engine.register(MovementRuntimeSystem::new(
            30,
            config::MOVEMENT_TICK_INTERVAL,
        ));

        let initial_food = state
            .engine
            .resources()
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should exist")
            .stockpile_food;

        let mut agent_one_saw_forage = false;
        let mut agent_two_saw_rest = false;
        for _ in 0..12 {
            state.engine.tick();
            let mut query = state.engine.world().query::<(&Identity, &Behavior)>();
            for (_, (identity, behavior)) in &mut query {
                match identity.name.as_str() {
                    "Agent 1" if behavior.current_action == ActionType::Forage => {
                        agent_one_saw_forage = true;
                    }
                    "Agent 2" if behavior.current_action == ActionType::Rest => {
                        agent_two_saw_rest = true;
                    }
                    _ => {}
                }
            }
        }
        assert!(agent_one_saw_forage);
        assert!(agent_two_saw_rest);

        for _ in 0..220 {
            state.engine.tick();
        }

        let mut after_by_name: HashMap<String, (f64, f64, f64)> = HashMap::new();
        {
            let mut query = state.engine.world().query::<(&Identity, &Needs)>();
            for (_, (identity, needs)) in &mut query {
                after_by_name.insert(
                    identity.name.clone(),
                    (
                        needs.get(NeedType::Hunger),
                        needs.get(NeedType::Sleep),
                        needs.energy,
                    ),
                );
            }
        }

        let agent_one = after_by_name
            .get("Agent 1")
            .copied()
            .expect("Agent 1 needs should exist");
        let agent_two = after_by_name
            .get("Agent 2")
            .copied()
            .expect("Agent 2 needs should exist");
        assert!(agent_one.0 > config::PROBE_START_PRIMARY_HUNGER);
        assert!(agent_two.1 > config::PROBE_START_SECONDARY_SLEEP);
        assert!(agent_two.2 > config::PROBE_START_SECONDARY_ENERGY);
        assert!(
            state
                .engine
                .resources()
                .settlements
                .get(&SettlementId(1))
                .expect("settlement should exist")
                .stockpile_food
                > initial_food
        );
    }

    #[test]
    fn bootstrap_payload_defaults_to_sandbox_mode() {
        let payload: RuntimeBootstrapPayload = serde_json::from_str(
            r#"{
                "world": {"width": 1, "height": 1},
                "founding_settlement": {"id": 1, "name": "", "x": 0, "y": 0},
                "agents": []
            }"#,
        )
        .expect("payload should deserialize");
        assert_eq!(payload.startup_mode, RuntimeBootstrapMode::Sandbox);
    }

    #[test]
    fn collect_construction_assignments_counts_targeted_builder() {
        let mut state = RuntimeState::from_seed(7, RuntimeConfig::default());
        let settlement_id = SettlementId(1);
        state.engine.world_mut().spawn((
            Identity {
                name: "Builder One".to_string(),
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Behavior {
                job: "builder".to_string(),
                current_action: ActionType::Build,
                action_target_x: Some(6),
                action_target_y: Some(6),
                ..Behavior::default()
            },
            Age {
                alive: true,
                ..Age::default()
            },
            Position {
                x: 6.0,
                y: 5.0,
                ..Position::default()
            },
            Skills::default(),
        ));
        let building = Building {
            id: BuildingId(9),
            building_type: "campfire".to_string(),
            settlement_id,
            x: 6,
            y: 6,
            construction_progress: 0.4,
            is_complete: false,
            construction_started_tick: 0,
            width: 1,
            height: 1,
            condition: 1.0,
        };

        let (settlement_builders, adjacent_builders, assignments) =
            collect_construction_assignments(state.engine.world(), &building);
        assert_eq!(settlement_builders, 1);
        assert_eq!(adjacent_builders, 1);
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].name, "Builder One");
        assert!(assignments[0].in_range);
    }

    #[test]
    fn runtime_band_id_raw_maps_optional_band_ids() {
        assert_eq!(runtime_band_id_raw(Some(BandId(4))), 4);
        assert_eq!(runtime_band_id_raw(None), -1);
    }

    #[test]
    fn construction_stall_reason_identifies_priority_gap() {
        assert_eq!(
            construction_stall_reason(
                &Building {
                    id: BuildingId(9),
                    building_type: "campfire".to_string(),
                    settlement_id: SettlementId(1),
                    x: 6,
                    y: 6,
                    construction_progress: 0.4,
                    is_complete: false,
                    construction_started_tick: 0,
                    width: 1,
                    height: 1,
                    condition: 1.0,
                },
                0.0,
                2,
                0,
                0,
                sim_core::config::CONSTRUCTION_TICK_INTERVAL,
            ),
            "priority_too_low"
        );
    }
}
