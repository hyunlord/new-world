use crate::chronicle::{ChronicleLog, ChronicleTimeline};
use crate::event_bus::EventBus;
use crate::event_store::EventStore;
use crate::events::LlmEvent;
use crate::explain_log::ExplainLog;
use crate::frame_snapshot::{build_agent_snapshots, AgentSnapshot};
use crate::llm_server::{LlmRuntime, LlmRuntimeError};
use crate::llm_worker::{LlmRequest, LlmRequestMeta, LlmResponse};
use crate::notification::SimNotification;
use crate::perf_tracker::PerfTracker;
use crate::snapshot::EngineSnapshot;
use crate::system_trait::{SimSystem, SystemEntry};
use hecs::World;
use log::{debug, info, warn};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use sim_core::world::TileResource;
use sim_core::{
    BandStore, Building, BuildingId, CausalLog, ChannelClampPolicy, ChannelId, ChildrenIndex,
    EffectQueue, EntityId, FurniturePlan, GameCalendar, InfluenceGrid, ItemStore, ResourceType,
    Room, Settlement, SettlementId, SimConfig, TerrainType, TileGrid, WallPlan, WorldMap,
};
use sim_data::{
    DataRegistry, InfluenceClampPolicyDef, NameGenerator, PersonalityDistribution, WorldRuleset,
};
/// SimEngine — the central tick loop coordinator.
///
/// Owns the ECS world, shared simulation resources, and the registered system list.
/// Calling `tick()` advances the simulation by one tick:
///   1. Each system whose `tick_interval` divides evenly into `current_tick` runs.
///   2. The event bus is flushed (subscribers notified).
///   3. The calendar advances.
///
/// # Determinism
/// Seed the RNG at construction to get reproducible runs.
/// System ordering is stable (sorted by priority at registration time).
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct RuntimeStatsSnapshot {
    pub tick: u64,
    pub pop: usize,
    pub food: f64,
    pub wood: f64,
    pub stone: f64,
    pub gatherers: u32,
    pub lumberjacks: u32,
    pub builders: u32,
    pub miners: u32,
    pub none_job: u32,
}

/// Current value plus recent delta for one inspectable diagnostic scalar.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiagnosticDelta {
    pub current: f64,
    pub delta: f64,
}

/// Recent survival-need diagnostics for one entity.
#[derive(Debug, Clone, Copy, Default)]
pub struct AgentNeedDiagnostics {
    pub hunger: DiagnosticDelta,
    pub warmth: DiagnosticDelta,
    pub safety: DiagnosticDelta,
    pub comfort: DiagnosticDelta,
    pub last_tick: u64,
}

/// Recent construction-progress diagnostics for one building.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConstructionDiagnostics {
    pub last_observed_progress: f64,
    pub progress_delta: f64,
    pub last_progress_tick: u64,
    pub last_sample_tick: u64,
}

// ── SimResources ──────────────────────────────────────────────────────────────

/// Shared non-component data passed to every system on each tick.
///
/// This is everything that isn't stored as ECS components:
/// world map, settlement records, calendar, RNG, and the event bus.
pub struct SimResources {
    /// Game calendar — tracks tick / day / year.
    pub calendar: GameCalendar,
    /// The tile grid for the world map.
    pub map: WorldMap,
    /// All settlements keyed by their ID.
    pub settlements: HashMap<SettlementId, Settlement>,
    /// All buildings keyed by their ID.
    pub buildings: HashMap<BuildingId, Building>,
    /// Inter-settlement tension cache (`min_id:max_id` -> 0.0..=1.0).
    pub tension_pairs: HashMap<String, f64>,
    /// Seeded RNG — use this for all randomness to preserve determinism.
    pub rng: SmallRng,
    /// Collect-then-drain event bus.
    pub event_bus: EventBus,
    /// Stats recorder history snapshots.
    pub stats_history: Vec<RuntimeStatsSnapshot>,
    /// Peak population observed by runtime stats recorder.
    pub stats_peak_population: usize,
    /// Runtime total births counter mirror.
    pub stats_total_births: u64,
    /// Runtime total deaths counter mirror.
    pub stats_total_deaths: u64,
    /// Per-entity stat-sync derived cache (8 composite scores).
    pub stat_sync_derived: HashMap<EntityId, [f32; 8]>,
    /// Per-entity stat-threshold active flags.
    pub stat_threshold_flags: HashMap<EntityId, u32>,
    /// Recent survival-need diagnostics keyed by entity ID.
    pub agent_need_diagnostics: HashMap<EntityId, AgentNeedDiagnostics>,
    /// Recent construction diagnostics keyed by building ID.
    pub construction_diagnostics: HashMap<BuildingId, ConstructionDiagnostics>,
    /// Structured chronicle log with bounded world and personal events.
    pub chronicle_log: ChronicleLog,
    /// Summarized chronicle timeline built from clustered raw events.
    pub chronicle_timeline: ChronicleTimeline,
    /// Personality distribution data for spawning agents (loaded from JSON at startup).
    pub personality_distribution: Option<PersonalityDistribution>,
    /// Name generator — generates culturally-appropriate names for new agents.
    pub name_generator: Option<NameGenerator>,
    /// Authoritative immutable runtime registry loaded from RON at startup.
    pub data_registry: Option<Arc<DataRegistry>>,
    /// Double-buffered spatial influence field shared across all systems.
    pub influence_grid: InfluenceGrid,
    /// Shared structural tile grid scaffold for future building/room systems.
    pub tile_grid: TileGrid,
    /// Shared detected room cache scaffold.
    pub rooms: Vec<Room>,
    /// Shared per-entity causal ring buffer scaffold.
    pub causal_log: CausalLog,
    /// Double-buffered effect queue shared across all runtime systems.
    pub effect_queue: EffectQueue,
    /// Central item registry for all world items.
    pub item_store: ItemStore,
    /// Central registry for provisional and promoted bands.
    pub band_store: BandStore,
    /// Per-faction territory influence grid (building-anchored, decaying).
    pub territory_grid: sim_core::territory_grid::TerritoryGrid,
    /// Accumulated border friction between settlement pairs.
    /// Key = (min_id, max_id) canonical ordering. Value = 0.0..=TERRITORY_FRICTION_MAX.
    /// Increases when settlement territories overlap, decays when overlap ceases.
    pub border_friction: HashMap<(SettlementId, SettlementId), f64>,
    /// Per-faction border hardness (0.0 = soft heatmap, 1.0 = crisp political boundary).
    /// Computed by TerritoryRuntimeSystem each cycle. Key = FactionId (u16).
    pub territory_hardness: HashMap<u16, f32>,
    /// Reverse index parent → child ids for genealogy lookups.
    pub children_index: ChildrenIndex,
    /// World-rules resource regen multipliers keyed by rule target tag.
    pub resource_regen_multipliers: BTreeMap<String, f64>,
    /// Per-entity ring-buffer of recent explanation log entries (stub — no systems write yet).
    pub explain_log: ExplainLog,
    /// Runtime-mutable simulation balance parameters (debug tuning).
    pub sim_config: SimConfig,
    /// Persisted narrative-analysis event history.
    pub event_store: EventStore,
    /// Pending UI-visible notifications waiting for Godot to drain them.
    pub pending_notifications: Vec<SimNotification>,
    /// Recent emitted notifications used for cooldown and deduplication.
    pub notification_history: Vec<SimNotification>,
    /// External llama-server process + worker runtime.
    pub llm_runtime: LlmRuntime,
    /// Hunger decay rate (default from config, overridable by WorldRuleset global_constants).
    pub hunger_decay_rate: f64,
    /// Warmth decay rate (default from config, overridable by WorldRuleset global_constants).
    pub warmth_decay_rate: f64,
    /// Food tile regen multiplier (default 1.0, overridable by WorldRuleset global_constants).
    pub food_regen_mul: f64,
    /// Wood tile regen multiplier (default 1.0, overridable by WorldRuleset global_constants).
    pub wood_regen_mul: f64,
    /// Whether farming/agriculture is enabled (default true).
    pub farming_enabled: bool,
    /// Base temperature bias applied to all tiles (-1.0 = frigid, 0.0 = default, 1.0 = scorching).
    pub temperature_bias: f64,
    /// Active season mode string (default "default").
    // TODO(A-9 phase 2): integrate with season-system-v1 (separate feature)
    pub season_mode: String,
    /// Disaster frequency multiplier (0.0 = no disasters, 1.0 = default, 10.0 = max).
    /// Cached only — disaster system not yet implemented (TODO: disaster-system-v1 separate feature).
    pub disaster_frequency_mul: f64,
    /// Mortality rate multiplier (default 1.0, overridable by WorldRuleset agent_constants).
    pub mortality_mul: f64,
    /// Global skill XP gain multiplier (default 1.0, overridable by WorldRuleset agent_constants).
    pub skill_xp_mul: f64,
    /// Body stat potential multiplier (default 1.0, overridable by WorldRuleset agent_constants).
    pub body_potential_mul: f64,
    /// Fertility/birth rate multiplier (default 1.0, overridable by WorldRuleset agent_constants).
    pub fertility_mul: f64,
    /// Lifespan multiplier for Siler model (default 1.0, overridable by WorldRuleset agent_constants).
    pub lifespan_mul: f64,
    /// Movement speed multiplier (default 1.0, overridable by WorldRuleset agent_constants).
    pub move_speed_mul: f64,
    /// P2-B3: pending wall placement plans queued by `generate_wall_ring_plans`.
    pub wall_plans: Vec<WallPlan>,
    /// P2-B3: pending furniture placement plans queued by `generate_wall_ring_plans`.
    pub furniture_plans: Vec<FurniturePlan>,
    /// P2-B3: monotonically increasing id source for new wall/furniture plans.
    pub next_plan_id: u64,
    /// Food economy diagnostic: total forage action completions.
    pub food_economy_forage_completions: u64,
    /// Food economy diagnostic: total food deposited by foraging (sum of yields).
    pub food_economy_produced: f64,
    /// Food economy diagnostic: total food withdrawn by childcare feeding.
    pub food_economy_childcare_drain: f64,
    /// Food economy diagnostic: total food consumed by births.
    pub food_economy_birth_drain: f64,
    /// Food economy diagnostic: total food consumed by crafting recipes.
    pub food_economy_craft_drain: f64,
    /// Food economy diagnostic: times the FOOD_SCARCITY_FORAGE_BOOST code path
    /// was exercised AND the agent chose Forage (excluding hunger force/soft-force).
    pub food_economy_scarcity_boost_applications: u64,
    /// Food economy diagnostic: times the scarcity boost was counterfactually
    /// effective — i.e., Forage won the action selection AND removing the 0.40
    /// boost would have made a different action win (Forage score - boost < 2nd best).
    pub food_economy_scarcity_boost_counterfactual: u64,
    /// Food economy inverse invariant: times the boost was flagged as applied
    /// but food_per_capita was >= threshold. Should ALWAYS be 0 by code construction.
    pub food_economy_boost_outside_scarcity: u64,
}

fn clamp_policy_from_def(value: &InfluenceClampPolicyDef) -> ChannelClampPolicy {
    match value {
        InfluenceClampPolicyDef::Sigmoid => ChannelClampPolicy::Sigmoid,
        InfluenceClampPolicyDef::UnitInterval => ChannelClampPolicy::UnitInterval,
    }
}

fn apply_world_rules_to_grid(grid: &mut InfluenceGrid, rules: &WorldRuleset) {
    let mut channels = ChannelId::default_channels();
    let mut applied_overrides = 0_usize;

    for rule in &rules.influence_channels {
        let Some(channel_id) = ChannelId::from_key(&rule.channel) else {
            warn!(
                "[WorldRules] unknown influence channel override: {}",
                rule.channel
            );
            continue;
        };

        let meta = &mut channels[channel_id.index()];
        if let Some(decay_rate) = rule.decay_rate {
            meta.decay_rate = decay_rate;
        }
        if let Some(default_radius) = rule.default_radius {
            meta.default_radius = default_radius;
        }
        if let Some(max_radius) = rule.max_radius {
            meta.max_radius = max_radius;
        }
        if let Some(wall_blocking_sensitivity) = rule.wall_blocking_sensitivity {
            meta.wall_blocking_sensitivity = wall_blocking_sensitivity;
        }
        if let Some(clamp_policy) = rule.clamp_policy.as_ref() {
            meta.clamp_policy = clamp_policy_from_def(clamp_policy);
        }
        applied_overrides += 1;
    }

    grid.set_channel_meta(&channels);
    if applied_overrides > 0 {
        info!(
            "[WorldRules] applied {} influence channel overrides",
            applied_overrides
        );
    }
}

/// Spawns special-zone tile clusters defined by a `WorldRuleset`.
///
/// Runs once during world initialization as part of `apply_world_rules`. Each zone
/// definition specifies a count range; a random number in that range of circular clusters
/// is placed on valid (passable, non-water) terrain and modifies tile terrain, resources,
/// temperature, and moisture. Zone placement is deterministic given `seed`.
fn spawn_special_zones(map: &mut WorldMap, zones: &[sim_data::RuleSpecialZone], seed: u64) {
    use rand::rngs::StdRng;

    if zones.is_empty() {
        return;
    }

    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(7777));

    for zone_def in zones {
        let r = zone_def.radius;
        // Guard: map must be wide/tall enough to fit the zone radius with margin.
        if map.width <= 2 * r || map.height <= 2 * r {
            warn!(
                "[WorldRules] zone '{}' radius {} too large for map {}×{}; skipping",
                zone_def.kind, r, map.width, map.height
            );
            continue;
        }

        let count_lo = zone_def.count.0.min(zone_def.count.1);
        let count_hi = zone_def.count.0.max(zone_def.count.1);
        let zone_count = rng.gen_range(count_lo..=count_hi);

        for _ in 0..zone_count {
            // Find a valid center tile (passable, non-water).
            let mut center_x = r;
            let mut center_y = r;
            for attempt in 0..200_u32 {
                let x = rng.gen_range(r..map.width - r);
                let y = rng.gen_range(r..map.height - r);
                let tile = map.get(x, y);
                if tile.passable
                    && !matches!(tile.terrain, TerrainType::DeepWater | TerrainType::ShallowWater)
                {
                    center_x = x;
                    center_y = y;
                    break;
                }
                if attempt == 199 {
                    center_x = x;
                    center_y = y;
                    warn!(
                        "[WorldRules] zone '{}' could not find valid terrain after 200 attempts; placing at ({}, {})",
                        zone_def.kind, x, y
                    );
                }
            }

            // Apply effects to all tiles within the circular cluster.
            let ri = r as i32;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    if dx * dx + dy * dy > ri * ri {
                        continue; // circular mask
                    }
                    let tx = center_x as i32 + dx;
                    let ty = center_y as i32 + dy;
                    if !map.in_bounds(tx, ty) {
                        continue;
                    }
                    let tile = map.get_mut(tx as u32, ty as u32);

                    if let Some(ref terrain_str) = zone_def.terrain_override {
                        match terrain_str.parse::<TerrainType>() {
                            Ok(terrain) => {
                                tile.terrain = terrain;
                                tile.passable = !matches!(
                                    terrain,
                                    TerrainType::DeepWater | TerrainType::Mountain
                                );
                            }
                            Err(_) => {
                                warn!(
                                    "[WorldRules] unknown terrain_override '{}' in zone '{}'",
                                    terrain_str, zone_def.kind
                                );
                            }
                        }
                    }

                    if let Some(temp_mod) = zone_def.temperature_mod {
                        tile.temperature = (tile.temperature + temp_mod).clamp(0.0, 1.0);
                    }
                    if let Some(moist_mod) = zone_def.moisture_mod {
                        tile.moisture = (tile.moisture + moist_mod).clamp(0.0, 1.0);
                    }

                    if let Some(ref boost) = zone_def.resource_boost {
                        match boost.resource.parse::<ResourceType>() {
                            Ok(resource_type) => {
                                if let Some(existing) = tile
                                    .resources
                                    .iter_mut()
                                    .find(|r| r.resource_type == resource_type)
                                {
                                    existing.amount += boost.amount;
                                    existing.max_amount =
                                        existing.max_amount.max(boost.max_amount);
                                    existing.regen_rate =
                                        existing.regen_rate.max(boost.regen_rate);
                                } else {
                                    tile.resources.push(TileResource {
                                        resource_type,
                                        amount: boost.amount,
                                        max_amount: boost.max_amount,
                                        regen_rate: boost.regen_rate,
                                    });
                                }
                            }
                            Err(_) => {
                                warn!(
                                    "[WorldRules] unknown resource '{}' in zone '{}'",
                                    boost.resource, zone_def.kind
                                );
                            }
                        }
                    }
                }
            }

            info!(
                "[WorldRules] spawned zone '{}' at ({}, {}), radius {}",
                zone_def.kind, center_x, center_y, r
            );
        }
    }
}

fn extract_resource_multipliers(rules: &WorldRuleset) -> BTreeMap<String, f64> {
    let mut multipliers = BTreeMap::new();
    for modifier in &rules.resource_modifiers {
        multipliers.insert(modifier.target.clone(), modifier.multiplier);
    }
    multipliers
}

impl SimResources {
    /// Create a fresh resource set.
    ///
    /// # Arguments
    /// - `calendar`: pre-constructed calendar (use `GameCalendar::new(&config)`)
    /// - `map`: world map (call `WorldMap::new(...)` or use world gen)
    /// - `seed`: RNG seed — same seed = identical simulation run
    pub fn new(calendar: GameCalendar, map: WorldMap, seed: u64) -> Self {
        let map_w = map.width;
        let map_h = map.height;
        let influence_grid =
            InfluenceGrid::new(map.width, map.height, ChannelId::default_channels());
        let tile_grid = TileGrid::new(map.width, map.height);
        Self {
            calendar,
            map,
            settlements: HashMap::new(),
            buildings: HashMap::new(),
            tension_pairs: HashMap::new(),
            rng: SmallRng::seed_from_u64(seed),
            event_bus: EventBus::new(),
            stats_history: Vec::new(),
            stats_peak_population: 0,
            stats_total_births: 0,
            stats_total_deaths: 0,
            stat_sync_derived: HashMap::new(),
            stat_threshold_flags: HashMap::new(),
            agent_need_diagnostics: HashMap::new(),
            construction_diagnostics: HashMap::new(),
            chronicle_log: ChronicleLog::new(),
            chronicle_timeline: ChronicleTimeline::new(),
            personality_distribution: None,
            name_generator: None,
            data_registry: None,
            influence_grid,
            tile_grid,
            rooms: Vec::new(),
            causal_log: CausalLog::new(),
            effect_queue: EffectQueue::new(),
            item_store: ItemStore::new(),
            band_store: BandStore::new(),
            territory_grid: sim_core::territory_grid::TerritoryGrid::new(map_w, map_h),
            border_friction: HashMap::new(),
            territory_hardness: HashMap::new(),
            children_index: ChildrenIndex::default(),
            resource_regen_multipliers: BTreeMap::new(),
            explain_log: ExplainLog::new(),
            sim_config: SimConfig::default(),
            event_store: EventStore::new(sim_core::config::EVENT_STORE_CAPACITY),
            pending_notifications: Vec::new(),
            notification_history: Vec::new(),
            llm_runtime: LlmRuntime::default(),
            hunger_decay_rate: sim_core::config::HUNGER_DECAY_RATE,
            warmth_decay_rate: sim_core::config::WARMTH_DECAY_RATE,
            food_regen_mul: 1.0,
            wood_regen_mul: 1.0,
            farming_enabled: true,
            temperature_bias: 0.0,
            season_mode: "default".to_string(),
            disaster_frequency_mul: 1.0,
            mortality_mul: 1.0,
            skill_xp_mul: 1.0,
            body_potential_mul: 1.0,
            fertility_mul: 1.0,
            lifespan_mul: 1.0,
            move_speed_mul: 1.0,
            wall_plans: Vec::new(),
            furniture_plans: Vec::new(),
            next_plan_id: 1,
            food_economy_forage_completions: 0,
            food_economy_produced: 0.0,
            food_economy_childcare_drain: 0.0,
            food_economy_birth_drain: 0.0,
            food_economy_craft_drain: 0.0,
            food_economy_scarcity_boost_applications: 0,
            food_economy_scarcity_boost_counterfactual: 0,
            food_economy_boost_outside_scarcity: 0,
        }
    }

    /// Applies world-rules overrides from the authoritative data registry.
    ///
    /// This is an init/lifecycle hook, not a hot-tick polling path.
    pub fn apply_world_rules(&mut self) {
        self.influence_grid
            .set_channel_meta(&ChannelId::default_channels());
        self.resource_regen_multipliers.clear();

        if let Some(registry) = self.data_registry.as_ref() {
            let count = registry.world_rules_raw.len();
            if count > 0 {
                let names: Vec<&str> = registry
                    .world_rules_raw
                    .iter()
                    .map(|ruleset| ruleset.name.as_str())
                    .collect();
                info!(
                    "[WorldRules] merging {} ruleset(s): {:?}",
                    count, names
                );
            }
        }

        let rules = self
            .data_registry
            .as_ref()
            .and_then(|registry| registry.world_rules_ref())
            .cloned();
        let Some(rules) = rules else {
            return;
        };

        apply_world_rules_to_grid(&mut self.influence_grid, &rules);
        self.resource_regen_multipliers = extract_resource_multipliers(&rules);

        // Apply global constant overrides (multipliers relative to config defaults).
        if let Some(globals) = &rules.global_constants {
            if let Some(mul) = globals.hunger_decay_mul {
                self.hunger_decay_rate = sim_core::config::HUNGER_DECAY_RATE * mul;
            }
            if let Some(mul) = globals.warmth_decay_mul {
                self.warmth_decay_rate = sim_core::config::WARMTH_DECAY_RATE * mul;
            }
            if let Some(mul) = globals.food_regen_mul {
                self.food_regen_mul = mul;
            }
            if let Some(mul) = globals.wood_regen_mul {
                self.wood_regen_mul = mul;
            }
            if let Some(enabled) = globals.farming_enabled {
                self.farming_enabled = enabled;
            }
            if let Some(bias) = globals.temperature_bias {
                self.temperature_bias = bias.clamp(-1.0, 1.0);
            }
            if let Some(ref mode) = globals.season_mode {
                self.season_mode = mode.clone();
            }
            if let Some(mul) = globals.disaster_frequency_mul {
                self.disaster_frequency_mul = mul.clamp(0.0, 10.0);
            }
            info!(
                "[WorldRules] global constants applied: hunger_decay={:.4}, warmth_decay={:.4}, food_regen={:.2}, season={}, disaster_freq={:.2}",
                self.hunger_decay_rate, self.warmth_decay_rate, self.food_regen_mul, self.season_mode, self.disaster_frequency_mul
            );
        }

        // Apply agent constant overrides (stored only — consumer integration out of scope).
        if let Some(ref agent) = rules.agent_constants {
            if let Some(mul) = agent.mortality_mul {
                self.mortality_mul = mul.max(0.0);
            }
            if let Some(mul) = agent.skill_xp_mul {
                self.skill_xp_mul = mul.max(0.0);
            }
            if let Some(mul) = agent.body_potential_mul {
                self.body_potential_mul = mul.max(0.0);
            }
            if let Some(mul) = agent.fertility_mul {
                self.fertility_mul = mul.clamp(0.0, 10.0);
            }
            if let Some(mul) = agent.lifespan_mul {
                self.lifespan_mul = mul.max(0.1);
            }
            if let Some(mul) = agent.move_speed_mul {
                self.move_speed_mul = mul.clamp(0.1, 5.0);
            }
            info!(
                "[WorldRules] agent constants: mortality={:.2}, skill_xp={:.2}, lifespan={:.2}, fertility={:.2}",
                self.mortality_mul, self.skill_xp_mul, self.lifespan_mul, self.fertility_mul
            );
        }

        // Spawn special-zone tile clusters (one-shot, runs at world init).
        if !rules.special_zones.is_empty() {
            let map_seed = self.map.seed;
            spawn_special_zones(&mut self.map, &rules.special_zones, map_seed);
            info!(
                "[WorldRules] spawned {} zone type(s)",
                rules.special_zones.len()
            );
        }

        info!(
            "[WorldRules] applied ruleset '{}' with {} resource modifiers",
            rules.name,
            rules.resource_modifiers.len()
        );
    }

    /// Starts the LLM server if the default config says it should be enabled.
    pub fn start_llm_if_enabled(&mut self) -> bool {
        if !self.llm_runtime.config().enabled_default {
            return false;
        }
        self.start_llm_server()
    }

    /// Starts the LLM server and emits lifecycle events.
    pub fn start_llm_server(&mut self) -> bool {
        match self.llm_runtime.start() {
            Ok(()) => {
                self.event_bus
                    .emit(crate::events::GameEvent::Llm(LlmEvent::ServerStarted));
                true
            }
            Err(error) => {
                warn!("[SimResources] failed to start LLM runtime: {}", error);
                self.event_bus.emit(crate::events::GameEvent::Llm(
                    LlmEvent::ServerHealthCheckFailed,
                ));
                false
            }
        }
    }

    /// Stops the LLM server and emits lifecycle events.
    pub fn stop_llm_server(&mut self) {
        if self.llm_runtime.is_running() {
            self.llm_runtime.stop();
            self.event_bus
                .emit(crate::events::GameEvent::Llm(LlmEvent::ServerStopped));
        }
    }

    /// Returns the currently selected AI narration quality tier.
    pub fn get_llm_quality(&self) -> u8 {
        self.llm_runtime.quality()
    }

    /// Updates the AI narration quality tier and emits lifecycle events when it
    /// starts or stops the external server.
    pub fn set_llm_quality(&mut self, quality: u8) {
        let was_running = self.llm_runtime.is_running();
        self.llm_runtime.set_quality(quality);
        let is_running = self.llm_runtime.is_running();
        if !was_running && is_running {
            self.event_bus
                .emit(crate::events::GameEvent::Llm(LlmEvent::ServerStarted));
        }
        if was_running && !is_running {
            self.event_bus
                .emit(crate::events::GameEvent::Llm(LlmEvent::ServerStopped));
        }
    }

    /// Returns true when the LLM server and worker can accept requests.
    pub fn is_llm_available(&self) -> bool {
        self.llm_runtime.is_available()
    }

    /// Returns a JSON status string for external callers.
    pub fn llm_status_json(&self) -> String {
        self.llm_runtime.status_json()
    }

    /// Drains recent LLM debug log lines for external diagnostics.
    pub fn drain_llm_debug_log(&self) -> Vec<String> {
        self.llm_runtime.drain_debug_log()
    }

    /// Attempts to submit an LLM request without blocking.
    pub fn submit_llm_request(&mut self, request: LlmRequest) -> Result<u64, LlmRuntimeError> {
        let entity_id = request.entity_id;
        let request_type = request.request_type;
        let request_id = self.llm_runtime.submit_request(request)?;
        self.event_bus
            .emit(crate::events::GameEvent::Llm(LlmEvent::RequestSubmitted {
                entity_id,
                request_type,
            }));
        Ok(request_id)
    }

    /// Attempts to submit a user-priority LLM request without blocking.
    pub fn submit_priority_llm_request(
        &mut self,
        request: LlmRequest,
    ) -> Result<u64, LlmRuntimeError> {
        let entity_id = request.entity_id;
        let request_type = request.request_type;
        let request_id = self.llm_runtime.submit_priority_request(request)?;
        self.event_bus
            .emit(crate::events::GameEvent::Llm(LlmEvent::RequestSubmitted {
                entity_id,
                request_type,
            }));
        Ok(request_id)
    }

    /// Drains all available LLM responses.
    pub fn drain_llm_responses(&mut self) -> Vec<LlmResponse> {
        self.llm_runtime.drain_responses()
    }

    /// Removes and returns request metadata for one in-flight LLM request.
    pub fn take_llm_request_meta(&mut self, request_id: u64) -> Option<LlmRequestMeta> {
        self.llm_runtime.take_request_meta(request_id)
    }
}

impl std::fmt::Debug for SimResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimResources")
            .field("tick", &self.calendar.tick)
            .field("settlements", &self.settlements.len())
            .field("buildings", &self.buildings.len())
            .field("tension_pairs", &self.tension_pairs.len())
            .field("stats_history", &self.stats_history.len())
            .field("stats_peak_population", &self.stats_peak_population)
            .field("stat_sync_derived", &self.stat_sync_derived.len())
            .field("stat_threshold_flags", &self.stat_threshold_flags.len())
            .field("agent_need_diagnostics", &self.agent_need_diagnostics.len())
            .field(
                "construction_diagnostics",
                &self.construction_diagnostics.len(),
            )
            .field("chronicle_world_events", &self.chronicle_log.world_len())
            .field("chronicle_summaries", &self.chronicle_timeline.len())
            .field("event_bus", &self.event_bus)
            .field("event_store", &self.event_store.len())
            .field("band_store", &self.band_store.len())
            .field("children_index", &self.children_index.map.len())
            .field("influence_grid_dims", &self.influence_grid.dimensions())
            .field(
                "influence_emitters",
                &self.influence_grid.active_emitter_count(),
            )
            .field("pending_notifications", &self.pending_notifications.len())
            .field("notification_history", &self.notification_history.len())
            .field("llm_available", &self.llm_runtime.is_available())
            .finish_non_exhaustive()
    }
}

// ── SimEngine ─────────────────────────────────────────────────────────────────

/// The simulation engine.
///
/// Lifecycle:
/// ```text
/// let engine = SimEngine::new(resources);
/// engine.register(MySystem::new());
/// engine.run_until(4380); // one in-game year
/// ```
pub struct SimEngine {
    /// ECS entity-component store.
    world: World,
    /// Non-component shared data.
    resources: SimResources,
    /// Registered systems, sorted ascending by priority.
    systems: Vec<SystemEntry>,
    /// Absolute tick counter (0-indexed; incremented after each `tick()` call).
    current_tick: u64,
    /// When true, per-system performance tracking is active.
    pub debug_mode: bool,
    /// Per-system and per-tick timing data (only updated when debug_mode is true).
    pub perf_tracker: PerfTracker,
    /// Latest per-agent render snapshots in stable raw-id order.
    frame_snapshots: Vec<AgentSnapshot>,
}

impl SimEngine {
    /// Create a new engine with the provided resources and an empty world.
    pub fn new(resources: SimResources) -> Self {
        Self {
            world: World::new(),
            resources,
            systems: Vec::new(),
            current_tick: 0,
            debug_mode: false,
            perf_tracker: PerfTracker::new(),
            frame_snapshots: Vec::new(),
        }
    }

    /// Register a system.
    ///
    /// Systems are inserted in ascending priority order (lower priority value = runs first).
    /// Within the same priority, systems run in registration order.
    ///
    /// `on_register()` is called immediately so the system can do one-time setup.
    pub fn register<S: SimSystem + 'static>(&mut self, system: S) {
        let name = system.name();
        let priority = system.priority();
        let mut entry = SystemEntry::new(Box::new(system));
        entry
            .system
            .on_register(&mut self.world, &mut self.resources);
        // Stable insertion: find first entry with strictly higher priority.
        let pos = self
            .systems
            .partition_point(|e| e.system.priority() <= priority);
        self.systems.insert(pos, entry);
        info!(
            "[SimEngine] registered '{}' (priority={}, pos={})",
            name, priority, pos
        );
    }

    /// Advance the simulation by exactly one tick.
    ///
    /// Order of operations:
    /// 1. Run all systems whose interval divides `current_tick`.
    /// 2. Flush the event bus (deliver to subscribers).
    /// 3. Advance the calendar.
    /// 4. Increment `current_tick`.
    pub fn tick(&mut self) {
        let tick = self.current_tick;
        debug!("[SimEngine] ── tick {} ──", tick);

        if self.debug_mode {
            self.perf_tracker.begin_tick();
        }

        for entry in self.systems.iter_mut() {
            if entry.should_run(tick) {
                debug!("[SimEngine] running '{}'", entry.system.name());
                if self.debug_mode {
                    self.perf_tracker.begin_system(entry.system.name());
                }
                entry.system.run(&mut self.world, &mut self.resources, tick);
                entry.last_run_tick = tick;
                if self.debug_mode {
                    self.perf_tracker.end_system(entry.system.name());
                }
            }
        }

        if self.debug_mode {
            self.perf_tracker.end_tick();
        }

        self.resources.influence_grid.tick_update();

        // Deliver events collected during this tick.
        self.resources.event_bus.flush();

        // Advance the in-game clock.
        self.resources.calendar.advance_tick();

        self.current_tick += 1;

        self.frame_snapshots = build_agent_snapshots(&self.world);
    }

    /// Run ticks until `current_tick == end_tick`.
    ///
    /// No-op if `current_tick >= end_tick`.
    pub fn run_until(&mut self, end_tick: u64) {
        while self.current_tick < end_tick {
            self.tick();
        }
    }

    /// Run exactly `n` ticks.
    pub fn run_ticks(&mut self, n: u64) {
        let target = self.current_tick + n;
        self.run_until(target);
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn resources(&self) -> &SimResources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut SimResources {
        &mut self.resources
    }

    /// Returns mutable references to both world and resources simultaneously,
    /// allowing callers to pass both to functions that need them concurrently.
    pub fn world_and_resources_mut(&mut self) -> (&mut World, &mut SimResources) {
        (&mut self.world, &mut self.resources)
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Rebuilds the cached per-agent render snapshots from the current ECS world.
    pub fn rebuild_frame_snapshots(&mut self) {
        self.frame_snapshots = build_agent_snapshots(&self.world);
    }

    /// Returns the current cached per-agent render snapshots.
    pub fn frame_snapshots(&self) -> &[AgentSnapshot] {
        &self.frame_snapshots
    }

    /// Clears all registered systems from the engine.
    ///
    /// Used by runtime bridge reconfiguration paths before re-registering
    /// system sets from external orchestrators.
    pub fn clear_systems(&mut self) {
        self.systems.clear();
    }

    /// Capture a lightweight read-only snapshot for diagnostics or save metadata.
    pub fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot {
            tick: self.current_tick,
            year: self.resources.calendar.year,
            day_of_year: self.resources.calendar.day_of_year,
            entity_count: self.world.len() as usize,
            settlement_count: self.resources.settlements.len(),
            system_count: self.systems.len(),
            events_dispatched: self.resources.event_bus.total_dispatched(),
        }
    }

    /// Restores scalar engine timeline values from a snapshot.
    ///
    /// This intentionally restores only timeline counters. ECS world/entities and
    /// settlements remain unchanged and are expected to be restored by a dedicated
    /// save subsystem.
    pub fn restore_from_snapshot(&mut self, snapshot: &EngineSnapshot) {
        self.current_tick = snapshot.tick;
        self.resources.calendar.tick = snapshot.tick;
        self.resources.calendar.year = snapshot.year;
        self.resources.calendar.day_of_year = snapshot.day_of_year;
    }
}

impl std::fmt::Debug for SimEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimEngine")
            .field("current_tick", &self.current_tick)
            .field("systems", &self.systems.len())
            .field("entities", &self.world.len())
            .field("settlements", &self.resources.settlements.len())
            .finish()
    }
}

impl Drop for SimEngine {
    fn drop(&mut self) {
        self.resources.stop_llm_server();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;
    use sim_core::config::GameConfig;
    use sim_core::{ChannelId, EmitterRecord, FalloffType};
    use sim_data::{InfluenceChannelRule, WorldRuleset};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    fn registry_data_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../sim-data/data")
            .canonicalize()
            .expect("registry data path should resolve")
    }

    fn load_registry() -> DataRegistry {
        DataRegistry::load_from_directory(&registry_data_path())
            .expect("registry should load for engine tests")
    }

    fn make_engine() -> SimEngine {
        let config = GameConfig::default();
        let cal = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 0);
        let res = SimResources::new(cal, map, 42);
        SimEngine::new(res)
    }

    #[test]
    fn tick_advances_counter() {
        let mut engine = make_engine();
        assert_eq!(engine.current_tick(), 0);
        engine.tick();
        assert_eq!(engine.current_tick(), 1);
        engine.tick();
        assert_eq!(engine.current_tick(), 2);
    }

    #[test]
    fn run_until_stops_at_target() {
        let mut engine = make_engine();
        engine.run_until(100);
        assert_eq!(engine.current_tick(), 100);
        // idempotent — no extra ticks
        engine.run_until(100);
        assert_eq!(engine.current_tick(), 100);
    }

    #[test]
    fn run_ticks_adds_to_current() {
        let mut engine = make_engine();
        engine.run_ticks(50);
        assert_eq!(engine.current_tick(), 50);
        engine.run_ticks(50);
        assert_eq!(engine.current_tick(), 100);
    }

    #[test]
    fn snapshot_reflects_state() {
        let mut engine = make_engine();
        engine.run_ticks(12); // 1 in-game day
        let snap = engine.snapshot();
        assert_eq!(snap.tick, 12);
        // calendar should have advanced 1 day
        assert_eq!(snap.day_of_year, 2);
    }

    #[test]
    fn restore_from_snapshot_sets_timeline_fields() {
        let mut engine = make_engine();
        engine.run_ticks(10);
        let mut snapshot = engine.snapshot();
        snapshot.tick = 123;
        snapshot.year = 4;
        snapshot.day_of_year = 77;
        engine.restore_from_snapshot(&snapshot);
        assert_eq!(engine.current_tick(), 123);
        assert_eq!(engine.resources().calendar.tick, 123);
        assert_eq!(engine.resources().calendar.year, 4);
        assert_eq!(engine.resources().calendar.day_of_year, 77);
    }

    struct CountSystem {
        count: u32,
    }
    impl crate::system_trait::SimSystem for CountSystem {
        fn name(&self) -> &'static str {
            "counter"
        }
        fn tick_interval(&self) -> u64 {
            1
        }
        fn run(&mut self, _w: &mut World, _r: &mut SimResources, _t: u64) {
            self.count += 1;
        }
    }

    #[test]
    fn system_runs_every_tick() {
        let mut engine = make_engine();
        engine.register(CountSystem { count: 0 });
        engine.run_ticks(5);
        // count is inside the boxed system; we can't inspect it here,
        // but the test verifies no panics and tick count is correct.
        assert_eq!(engine.current_tick(), 5);
    }

    #[test]
    fn systems_registered_in_priority_order() {
        struct P(u32);
        impl crate::system_trait::SimSystem for P {
            fn name(&self) -> &'static str {
                "p"
            }
            fn tick_interval(&self) -> u64 {
                1
            }
            fn priority(&self) -> u32 {
                self.0
            }
            fn run(&mut self, _w: &mut World, _r: &mut SimResources, _t: u64) {}
        }

        let mut engine = make_engine();
        engine.register(P(50));
        engine.register(P(10));
        engine.register(P(200));
        engine.register(P(10)); // duplicate priority — second slot

        // Internal ordering: 10, 10, 50, 200
        let priorities: Vec<u32> = engine.systems.iter().map(|e| e.system.priority()).collect();
        assert_eq!(priorities, vec![10, 10, 50, 200]);
    }

    #[test]
    fn rebuild_frame_snapshots_reflects_live_world() {
        let mut engine = make_engine();
        engine.world_mut().spawn((
            sim_core::components::Position::new(2, 3),
            sim_core::components::Identity::default(),
            sim_core::components::Age::default(),
        ));
        engine.rebuild_frame_snapshots();
        assert_eq!(engine.frame_snapshots().len(), 1);
        let x = engine.frame_snapshots()[0].x;
        assert_eq!(x, 2.0);
    }

    #[test]
    fn llm_status_is_json_even_when_runtime_is_unavailable() {
        let engine = make_engine();
        let status = engine.resources().llm_status_json();
        assert!(status.contains("\"running\":false"));
    }

    #[test]
    fn sim_resources_initialize_influence_grid_from_map_dimensions() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(24, 12, 0);
        let resources = SimResources::new(calendar, map, 7);
        assert_eq!(resources.influence_grid.dimensions(), (24, 12));
        assert_eq!(resources.tile_grid.dimensions(), (24, 12));
        assert!(resources.rooms.is_empty());
        assert_eq!(resources.causal_log.total_entries(), 0);
    }

    #[test]
    fn engine_tick_updates_influence_grid_buffers() {
        let mut engine = make_engine();
        engine
            .resources_mut()
            .influence_grid
            .register_emitter(EmitterRecord {
                x: 4,
                y: 4,
                channel: ChannelId::Warmth,
                radius: 3.0,
                base_intensity: 0.8,
                falloff: FalloffType::Constant,
                decay_rate: None,
                tags: Vec::new(),
                dirty: false,
            });

        assert_eq!(
            engine
                .resources()
                .influence_grid
                .sample(4, 4, ChannelId::Warmth),
            0.0
        );
        engine.tick();
        assert!(
            engine
                .resources()
                .influence_grid
                .sample(4, 4, ChannelId::Warmth)
                > 0.0
        );
    }

    #[test]
    fn world_rules_apply_channel_overrides_to_grid() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(24, 12, 0);
        let mut resources = SimResources::new(calendar, map, 7);
        resources.data_registry = Some(Arc::new(load_registry()));

        resources.apply_world_rules();

        let food_meta = resources.influence_grid.channel_meta(ChannelId::Food);
        assert!((food_meta.decay_rate - 0.18).abs() < f64::EPSILON);
        assert!((food_meta.default_radius - 7.0).abs() < f64::EPSILON);
        assert_eq!(food_meta.max_radius, 14);
        assert!((food_meta.wall_blocking_sensitivity - 0.2).abs() < f64::EPSILON);
        assert_eq!(food_meta.clamp_policy, ChannelClampPolicy::UnitInterval);
        // After A-9 multi-ruleset merge, the authoritative canonical registry
        // merges base_rules.ron (priority 0, surface_foraging × 1.0) with
        // scenarios/eternal_winter.ron (priority 10, surface_foraging × 0.3).
        // Highest-priority overlay wins: the merged multiplier is 0.3.
        assert_eq!(
            resources
                .resource_regen_multipliers
                .get("surface_foraging")
                .copied(),
            Some(0.3)
        );
    }

    #[test]
    fn world_rules_partial_override_preserves_defaults() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(24, 12, 0);
        let mut resources = SimResources::new(calendar, map, 7);
        let mut registry = load_registry();
        registry.world_rules = Some(WorldRuleset {
            name: "PartialRules".to_string(),
            priority: 1,
            resource_modifiers: Vec::new(),
            special_zones: Vec::new(),
            special_resources: Vec::new(),
            agent_constants: None,
            influence_channels: vec![InfluenceChannelRule {
                channel: "food".to_string(),
                decay_rate: Some(0.42),
                default_radius: None,
                max_radius: None,
                wall_blocking_sensitivity: None,
                clamp_policy: None,
            }],
            global_constants: None,
        });
        resources.data_registry = Some(Arc::new(registry));

        resources.apply_world_rules();

        let food_meta = resources.influence_grid.channel_meta(ChannelId::Food);
        let default_food_meta = ChannelId::Food.default_meta();
        assert!((food_meta.decay_rate - 0.42).abs() < f64::EPSILON);
        assert_eq!(food_meta.default_radius, default_food_meta.default_radius);
        assert_eq!(food_meta.max_radius, default_food_meta.max_radius);
        assert_eq!(
            food_meta.wall_blocking_sensitivity,
            default_food_meta.wall_blocking_sensitivity
        );
        assert_eq!(food_meta.clamp_policy, default_food_meta.clamp_policy);
    }

    #[test]
    fn world_rules_absent_rules_keep_defaults() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(24, 12, 0);
        let mut resources = SimResources::new(calendar, map, 7);
        resources
            .resource_regen_multipliers
            .insert("surface_foraging".to_string(), 2.5);

        resources.apply_world_rules();

        let food_meta = resources.influence_grid.channel_meta(ChannelId::Food);
        let default_food_meta = ChannelId::Food.default_meta();
        assert_eq!(food_meta, &default_food_meta);
        assert!(resources.resource_regen_multipliers.is_empty());
    }

    #[test]
    fn spawn_zones_adds_resource_to_tiles() {
        let mut map = WorldMap::new(50, 50, 42);
        let zones = [sim_data::RuleSpecialZone {
            kind: "test_oasis".to_string(),
            count: (1, 1),
            radius: 3,
            terrain_override: None,
            resource_boost: Some(sim_data::ZoneResourceBoost {
                resource: "Food".to_string(),
                amount: 15.0,
                max_amount: 20.0,
                regen_rate: 0.5,
            }),
            temperature_mod: None,
            moisture_mod: None,
        }];
        spawn_special_zones(&mut map, &zones, 42);
        let has_zone_food = (0..50_u32).any(|y| {
            (0..50_u32).any(|x| {
                map.get(x, y)
                    .resources
                    .iter()
                    .any(|r| r.resource_type == ResourceType::Food && r.regen_rate >= 0.49)
            })
        });
        assert!(has_zone_food, "spawn_special_zones should place Food tiles with boosted regen");
    }

    #[test]
    fn spawn_zones_overrides_terrain() {
        let mut map = WorldMap::new(50, 50, 99);
        let zones = [sim_data::RuleSpecialZone {
            kind: "snow_patch".to_string(),
            count: (1, 1),
            radius: 2,
            terrain_override: Some("Snow".to_string()),
            resource_boost: None,
            temperature_mod: None,
            moisture_mod: None,
        }];
        spawn_special_zones(&mut map, &zones, 99);
        let has_snow = (0..50_u32)
            .any(|y| (0..50_u32).any(|x| map.get(x, y).terrain == TerrainType::Snow));
        assert!(has_snow, "spawn_special_zones should override terrain to Snow");
    }

    #[test]
    fn spawn_zones_empty_list_is_noop() {
        let mut map = WorldMap::new(20, 20, 0);
        spawn_special_zones(&mut map, &[], 0);
        for y in 0..20_u32 {
            for x in 0..20_u32 {
                let tile = map.get(x, y);
                assert!(tile.resources.is_empty(), "empty zones must not add resources");
                assert_eq!(
                    tile.terrain,
                    TerrainType::Grassland,
                    "empty zones must not change terrain"
                );
            }
        }
    }
}
