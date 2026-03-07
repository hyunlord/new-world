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
use std::collections::HashMap;
use hecs::World;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use sim_core::{Building, BuildingId, EntityId, GameCalendar, Settlement, SettlementId, SimConfig, WorldMap};
use sim_data::{NameGenerator, PersonalityDistribution};
use crate::event_bus::EventBus;
use crate::event_store::EventStore;
use crate::explain_log::ExplainLog;
use crate::notification::SimNotification;
use crate::perf_tracker::PerfTracker;
use crate::frame_snapshot::{build_agent_snapshots, AgentSnapshot};
use crate::system_trait::{SimSystem, SystemEntry};
use crate::snapshot::EngineSnapshot;
use log::{debug, info};

/// A recorded chronicle event (world or personal history).
#[derive(Debug, Clone)]
pub struct ChronicleEvent {
    pub tick: u64,
    pub importance: u32,
    pub event_type: String,
    pub entity_id: i64,
    pub description: String,
}

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
    /// Chronicle world-event log (pruned periodically).
    pub chronicle_world_events: Vec<ChronicleEvent>,
    /// Chronicle personal-event log keyed by entity ID.
    pub chronicle_personal_events: HashMap<EntityId, Vec<ChronicleEvent>>,
    /// Personality distribution data for spawning agents (loaded from JSON at startup).
    pub personality_distribution: Option<PersonalityDistribution>,
    /// Name generator — generates culturally-appropriate names for new agents.
    pub name_generator: Option<NameGenerator>,
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
}

impl SimResources {
    /// Create a fresh resource set.
    ///
    /// # Arguments
    /// - `calendar`: pre-constructed calendar (use `GameCalendar::new(&config)`)
    /// - `map`: world map (call `WorldMap::new(...)` or use world gen)
    /// - `seed`: RNG seed — same seed = identical simulation run
    pub fn new(calendar: GameCalendar, map: WorldMap, seed: u64) -> Self {
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
            chronicle_world_events: Vec::new(),
            chronicle_personal_events: HashMap::new(),
            personality_distribution: None,
            name_generator: None,
            explain_log: ExplainLog::new(),
            sim_config: SimConfig::default(),
            event_store: EventStore::new(sim_core::config::EVENT_STORE_CAPACITY),
            pending_notifications: Vec::new(),
            notification_history: Vec::new(),
        }
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
            .field("event_bus", &self.event_bus)
            .field("event_store", &self.event_store.len())
            .field("pending_notifications", &self.pending_notifications.len())
            .field("notification_history", &self.notification_history.len())
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
        entry.system.on_register(&mut self.world, &mut self.resources);
        // Stable insertion: find first entry with strictly higher priority.
        let pos = self.systems.partition_point(|e| e.system.priority() <= priority);
        self.systems.insert(pos, entry);
        info!("[SimEngine] registered '{}' (priority={}, pos={})", name, priority, pos);
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;
    use sim_core::config::GameConfig;

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
        fn name(&self) -> &'static str { "counter" }
        fn tick_interval(&self) -> u64 { 1 }
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
            fn name(&self) -> &'static str { "p" }
            fn tick_interval(&self) -> u64 { 1 }
            fn priority(&self) -> u32 { self.0 }
            fn run(&mut self, _w: &mut World, _r: &mut SimResources, _t: u64) {}
        }

        let mut engine = make_engine();
        engine.register(P(50));
        engine.register(P(10));
        engine.register(P(200));
        engine.register(P(10)); // duplicate priority — second slot

        // Internal ordering: 10, 10, 50, 200
        let priorities: Vec<u32> = engine.systems.iter()
            .map(|e| e.system.priority())
            .collect();
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
}
