//! WorldSim simulation engine.
//!
//! V7 reset 후 첫 multi-crate land (Phase 2 Tile Grid + Influence System).
//! Phase 0 design v0.1.3 patch Section 1.3.3 base.
//!
//! Provides:
//! - [`RuntimeSystem`] trait — uniform tick interface for all simulation
//!   systems (priority, tick_interval, tick).
//! - [`SimResources`] — shared world state owned by the engine and
//!   passed by mutable reference to every system tick.
//! - [`SimEngine`] — registers systems (priority-sorted) and drives
//!   the tick loop.
//!
//! # Tick scheduling
//!
//! Systems are sorted by `priority()` (lower runs first). On every tick,
//! each system whose `tick_interval()` divides the current tick is
//! invoked. The engine's `current_tick` is propagated to
//! [`SimResources::current_tick`] before any system runs, so systems
//! can branch on it deterministically.
//!
//! # Hard Gate 6 budget targets
//! - Hot tier: tick_interval = 1, ≤ 0.5 ms @ 1K agents
//! - Warm tier: tick_interval = 1 (with internal staggering), ≤ 2 ms
//! - Cold tier: dirty-region only, ≤ 5 ms (rare events)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use hecs::{Entity, World};
use sim_core::causal::event::DeathReason;
use sim_core::causal::{CausalLogStorage, EventId};
use sim_core::components::{
    Agent, AgentId, BuildingId, Position, RelationshipKey, RelationshipState, Settlement,
    SettlementId,
};
use sim_core::influence::{InfluenceGrid, MaterialBlockingCache};
use sim_core::material::MaterialRegistry;
use sim_core::tile::TileGrid;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

/// FFI-originated event: a building was placed at `position` with influence
/// `radius` (Chebyshev distance, in tiles). Drained by
/// `sim_systems::runtime::influence::BuildingStampSystem` each tick which
/// translates each event into `InfluenceGrid::mark_dirty` calls on the
/// Warmth/Spiritual/Beauty/Light channels.
///
/// Phase 0 v0.1.3 §5 — R1 event_queue path (T7.7.B land).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingPlacedEvent {
    /// Tile coordinates of the building origin (top-left corner).
    pub position: (u32, u32),
    /// Influence radius in tiles (Chebyshev distance, inclusive).
    pub radius: u32,
}

/// A recently-recorded agent death (V7 viz-D `show-death-visual`).
///
/// Pushed by the shared `survival::despawn_agent` helper (the single death
/// path that both `StarvationSystem` and `CombatSystem` route through) and
/// pruned each tick once it ages past [`RECENT_DEATH_RETAIN_TICKS`]. The
/// `death_viz_renderer.gd` overlay reads this buffer (via the `get_recent_deaths`
/// FFI) and draws a fading, reason-coloured marker at the death tile.
///
/// Display-only: this buffer is never read by simulation logic, so it cannot
/// affect determinism. `SimResources` is NOT serde-serialized, so the field
/// carries no save/load or lockstep weight.
#[derive(Debug, Clone, Copy)]
pub struct RecentDeath {
    /// Death tile-x (the dead agent's last `Position` tile). Stored as `i32`
    /// per the viz-D buffer schema so the FFI `x as i32` cast is identity and
    /// the harness compares natively (`i32 == i32`). The `position.0` value
    /// pushed by `despawn_agent` is widened from the `u32` `Position` field.
    pub x: i32,
    /// Death tile-y (the dead agent's last `Position` tile). See [`Self::x`].
    pub y: i32,
    /// Typed cause of death (drives the marker colour).
    pub reason: DeathReason,
    /// Simulation tick at which the death occurred (drives the fade age).
    /// `u32` per the viz-D buffer schema; the FFI widens it to `i64`.
    pub tick: u32,
}

/// Ticks a [`RecentDeath`] is retained before [`SimEngine::tick`] prunes it.
///
/// Must be `>=` the renderer's `FADE_TICKS` (90) so the fade window always has
/// data to draw; `120` gives margin (the marker is fully faded by 90, gone from
/// the buffer by 120).
pub const RECENT_DEATH_RETAIN_TICKS: u64 = 120;

/// Uniform interface implemented by every simulation system.
///
/// Phase 0 design v0.1.3 patch Section 1.3.3 base. Systems are stored
/// inside [`SimEngine`] as `Box<dyn RuntimeSystem + Send>` so they can be
/// shipped across threads if a future scheduler parallelises ticks.
pub trait RuntimeSystem {
    /// Stable identifier (used for logs / panel display).
    fn name(&self) -> &str;

    /// Scheduling priority — **lower runs first**.
    ///
    /// Phase 2 reservations:
    /// - 90: BuildingStampSystem
    /// - 100: InfluenceUpdateSystem
    /// - 110: AgentInfluenceSampleSystem
    /// - 1000: InfluenceVisualizationSystem
    fn priority(&self) -> u32;

    /// How often the system runs, in ticks. `1` means every tick.
    /// `n > 1` means the system runs when `current_tick % n == 0`.
    fn tick_interval(&self) -> u64;

    /// Per-tick work. Called by [`SimEngine::tick`] when due.
    fn tick(&mut self, world: &mut World, resources: &mut SimResources);
}

/// Sentinel quantity marking a tile as a non-depleting **source**
/// (V7 Section 16-α0). A tile in [`SimResources::food_tiles`] /
/// `water_tiles` / `sleep_tiles` whose counter equals this value is never
/// decremented or removed by the `Consuming` cascade — it persists for
/// the lifetime of the run, guaranteeing agents always have a reachable
/// goal. Finite tiles (any other non-zero value) keep their existing
/// decrement-and-remove behavior. Reuses the existing `u8::MAX` saturation
/// point of the sparse tile maps, so it adds zero new data structures and
/// stays ripple-free across FFI and save format.
pub const RESOURCE_SOURCE_INFINITE: u8 = u8::MAX;

/// Shared world state owned by the engine.
///
/// `hecs` enforces single-writer semantics on the [`World`]; this struct
/// applies the same convention to non-ECS resources by handing every
/// system a `&mut SimResources` and trusting priorities to serialise
/// access.
pub struct SimResources {
    /// Tile grid (T7.1 land — wall / floor / terrain SoA).
    pub tile_grid: TileGrid,

    /// 8-channel double-buffered influence grid (T7.3 land).
    pub influence_grid: InfluenceGrid,

    /// Material catalogue (T6.6 ~ T6.8 land — 105 materials).
    pub material_registry: MaterialRegistry,

    /// Pre-computed `(material, channel) → block` lookup (T7.4 land).
    pub material_blocking_cache: MaterialBlockingCache,

    /// Current tick — refreshed by [`SimEngine::tick`] before systems run.
    pub current_tick: u64,

    /// FFI-originated building placement events, drained each tick by
    /// `BuildingStampSystem`. Pushed by `sim_bridge::WorldSimNode::on_building_placed`
    /// (which delegates to `sim_bridge::ffi::enqueue_building_placed`).
    pub building_event_queue: VecDeque<BuildingPlacedEvent>,

    /// Sparse per-tile causal event log (V7 Phase 3-α). BSS pushes
    /// `BuildingPlaced` + `StampDirty` records; IUS pushes
    /// `InfluenceChanged` records once per drained dirty region per
    /// channel. Consumed by the "왜?" UI (Week 6) to attribute
    /// influence-grid state to the events that produced it.
    pub causal_log: CausalLogStorage,

    /// Monotonic source of [`EventId`]s for the causal log
    /// (V7 Phase 3-β / P3β-1). Allocated once per recorded event via
    /// [`SimResources::issue_event_id`]. `Relaxed` ordering is sufficient
    /// — uniqueness is the only invariant, and per-tick ordering is
    /// preserved by the priority-sorted system schedule.
    pub next_event_id: AtomicU64,

    /// Monotonic source of [`AgentId`]s for the canonical `Agent`
    /// component (V7 Phase 5-α / P5α-2). Allocated by
    /// [`SimResources::issue_agent_id`] (mirrors `next_event_id`).
    /// `Relaxed` ordering — agents are spawned single-threaded by the
    /// priority-sorted scheduler and only uniqueness matters.
    pub next_agent_id: AtomicU64,

    /// Sparse food-tile substrate (V7 Phase 5-β / P5β-7).
    ///
    /// Maps `(x, y)` tile coordinate → remaining food units (`u8`,
    /// 0..=255). Absent keys are interpreted as zero. The Phase 5-β
    /// AgentDecisionSystem reads this map to locate a Consume target
    /// and decrements the counter when an agent consumes from the
    /// tile. Saturating at `u8::MAX` is intentional — Phase 5-β does
    /// not need a richer food economy (calorie content, freshness, etc.)
    /// and the sparse map keeps the substrate ripple-free across
    /// TileGrid, FFI, and save format.
    pub food_tiles: HashMap<(u32, u32), u8>,

    /// Sparse water-tile substrate (V7 Phase 5-β / P5β-7). Mirrors
    /// [`SimResources::food_tiles`] for the second need.
    pub water_tiles: HashMap<(u32, u32), u8>,

    /// Sparse sleep-tile substrate (V7 Phase 5-γ / P5γ-7). Mirrors
    /// [`SimResources::food_tiles`] for the third need.
    pub sleep_tiles: HashMap<(u32, u32), u8>,

    /// Regen ceiling per registered FOOD source (`add-resource-scarcity-regen`).
    ///
    /// Maps `(x, y) → original finite capacity` for each food source the
    /// production scene seeds via `seed_finite_resource_scarcity`. The
    /// `ResourceRegenSystem` (priority 140) periodically refills
    /// [`SimResources::food_tiles`] toward this ceiling. Empty by default — a
    /// harness that does NOT seed finite scarcity leaves it empty, so regen is
    /// a pure no-op there (the 12 shared harnesses are unperturbed).
    pub food_source_max: HashMap<(u32, u32), u8>,

    /// Regen ceiling per registered WATER source. Mirrors
    /// [`SimResources::food_source_max`] for the water channel.
    pub water_source_max: HashMap<(u32, u32), u8>,

    /// Regen ceiling per registered SLEEP source. Mirrors
    /// [`SimResources::food_source_max`] for the sleep channel.
    pub sleep_source_max: HashMap<(u32, u32), u8>,

    /// Sparse per-pair relationship state (V7 Phase 7-β / P7β-13).
    /// Bumped by `SocialInteractionSystem` on `SocialInteractionCompleted`.
    /// Starts empty; never seeded by engine construction.
    pub relationships: HashMap<RelationshipKey, RelationshipState>,

    /// Sparse per-pair multi-tick interaction progress counter
    /// (V7 Phase 7-β / P7β-13). Incremented once per mutual
    /// `Consuming{Agent(other)}` pair per tick by `SocialInteractionSystem`;
    /// removed on completion or asymmetric-partner fallback. Starts empty.
    pub interaction_progress: HashMap<RelationshipKey, u32>,

    /// Active combat pairs for the current tick. Keys are canonical
    /// `(smaller AgentId, larger AgentId)` tuples inserted by
    /// `AgentDecisionSystem` when the combat cascade arm fires.
    /// `CombatSystem` (priority 137) consumes and removes completed pairs.
    /// Phase 9-β / P9β-1.
    pub combat_pairs: HashSet<(AgentId, AgentId)>,

    /// Per-pair combat progress counter. Keyed by the same canonical
    /// `(smaller, larger)` tuple as `combat_pairs`. `CombatSystem`
    /// increments each pair's counter until it reaches
    /// `REQUIRED_COMBAT_PROGRESS`, at which point `CombatCompleted` fires.
    /// Phase 9-β / P9β-1.
    pub combat_progress: HashMap<(AgentId, AgentId), u32>,

    /// Current simulated time-of-day in `[0.0, 24.0)` (V7 Phase 5-γ /
    /// P5γ-2). Refreshed by [`SimEngine::tick`] before systems run,
    /// derived deterministically from `current_tick % ticks_per_day`.
    /// Starts at 0.0 (midnight).
    pub time_of_day: f64,

    /// Number of ticks in one simulated day (V7 Phase 5-γ / P5γ-3).
    /// Defaults to 1440 (24 × 60 = one tick-per-minute). Public so
    /// harness scenarios can override before running. When `0`, the
    /// clock-advance step in `SimEngine::tick` is a no-op (zero-guard).
    pub ticks_per_day: u64,

    /// Sparse settlement registry (V7 Phase 10-α / P10Plan-1).
    ///
    /// Maps each [`SettlementId`] to its [`Settlement`] aggregate.
    /// Populated and maintained by `SettlementSystem` (priority 138,
    /// Phase 10-β scope). Starts empty — no settlements exist at world
    /// construction time. Follows the established SimResources HashMap
    /// sparse-collection pattern (`relationships`, `combat_pairs`, etc.).
    pub settlements: HashMap<SettlementId, Settlement>,

    /// Monotonic source of [`SettlementId`]s (V7 Phase 10-α / P10Plan-1).
    ///
    /// Unlike `next_event_id` / `next_agent_id` (AtomicU64), settlement
    /// ids are minted single-threaded by `SettlementSystem` under `&mut
    /// SimResources`, so a plain `u32` counter suffices. Allocated via
    /// [`SimResources::issue_settlement_id`].
    pub next_settlement_id: SettlementId,

    /// Sparse registry of placed building positions, keyed by their
    /// `BuildingId` (which is the `EventId` of the originating
    /// `BuildingPlaced` causal event). Populated by `BuildingStampSystem`
    /// on every drained FFI event AND by `ConstructionSystem` on every
    /// completion edge. Consumed by `SettlementSystem` (priority 138) for
    /// formation/membership scans.
    ///
    /// The causal log alone cannot serve this purpose — its per-tile ring
    /// buffer is capped at 8 events and same-tile `StampDirty` +
    /// `InfluenceChanged` events evict `BuildingPlaced` records within a
    /// single tick. The registry is the authoritative
    /// "where are the buildings?" lookup. V7 Phase 10-β.
    pub building_registry: HashMap<BuildingId, (u32, u32)>,

    /// Bounded, display-only buffer of recent agent deaths (V7 viz-D
    /// `show-death-visual`). Pushed by the shared `survival::despawn_agent`
    /// helper, pruned each tick by [`SimEngine::tick`] once entries age past
    /// [`RECENT_DEATH_RETAIN_TICKS`]. Read by the `get_recent_deaths` FFI for
    /// the death-marker overlay. Never read by simulation logic (determinism-
    /// safe); not serde-serialized.
    pub recent_deaths: Vec<RecentDeath>,
}

impl SimResources {
    /// Direct constructor for [`SimResources`] (V7 Phase 7-β / P7β-13).
    ///
    /// `SimEngine::new` delegates to this so the per-pair social HashMaps
    /// (`relationships`, `interaction_progress`) and every other sparse
    /// substrate field have a single canonical "starts empty" initialiser.
    /// Tests can also call this directly to verify P7β-13's empty-default
    /// invariant without spinning up a full engine.
    pub fn new(width: u32, height: u32, registry: MaterialRegistry) -> Self {
        let blocking_cache = MaterialBlockingCache::build(&registry);
        Self {
            tile_grid: TileGrid::new(width, height),
            influence_grid: InfluenceGrid::new(width, height),
            material_registry: registry,
            material_blocking_cache: blocking_cache,
            current_tick: 0,
            building_event_queue: VecDeque::new(),
            causal_log: CausalLogStorage::new(),
            next_event_id: AtomicU64::new(0),
            next_agent_id: AtomicU64::new(0),
            food_tiles: HashMap::new(),
            water_tiles: HashMap::new(),
            sleep_tiles: HashMap::new(),
            food_source_max: HashMap::new(),
            water_source_max: HashMap::new(),
            sleep_source_max: HashMap::new(),
            relationships: HashMap::new(),
            interaction_progress: HashMap::new(),
            combat_pairs: HashSet::new(),
            combat_progress: HashMap::new(),
            time_of_day: 0.0,
            ticks_per_day: 1440,
            settlements: HashMap::new(),
            // Phase 10-β plan A2: `settlement_id == 0` is the uninitialized
            // sentinel — `SettlementFormed.settlement_id == 0` must never
            // be observed in the causal log. Start the counter at 1 so the
            // first issued id is `1` and `0` remains reserved.
            next_settlement_id: 1,
            building_registry: HashMap::new(),
            recent_deaths: Vec::new(),
        }
    }

    /// Allocate the next monotonic [`EventId`] (V7 Phase 3-β / P3β-1).
    ///
    /// The counter outlives ring-buffer eviction: even after the
    /// originating event is dropped, descendants retain the id reference,
    /// and chain lookups simply terminate gracefully on miss (see
    /// [`CausalLogStorage::trace_parents`]).
    pub fn issue_event_id(&self) -> EventId {
        self.next_event_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Allocate the next monotonic [`AgentId`] (V7 Phase 5-α / P5α-2).
    ///
    /// Mirrors [`SimResources::issue_event_id`]. Called internally by
    /// [`SimEngine::spawn_agent`] — callers should normally use that
    /// rather than minting ids by hand.
    pub fn issue_agent_id(&self) -> AgentId {
        self.next_agent_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Allocate the next monotonic [`SettlementId`] (V7 Phase 10-α / P10Plan-1).
    ///
    /// Uses a plain `u32` counter (unlike `next_event_id` / `next_agent_id`
    /// which are `AtomicU64`) because `SettlementSystem` runs single-threaded
    /// under `&mut SimResources`. Called by `SettlementSystem` (Phase 10-β)
    /// when a new cluster meets the formation threshold.
    pub fn issue_settlement_id(&mut self) -> SettlementId {
        let id = self.next_settlement_id;
        self.next_settlement_id = self.next_settlement_id.wrapping_add(1);
        id
    }

    /// Set the food-tile counter at `(x, y)` (V7 Phase 5-β / P5β-7).
    ///
    /// `amount == 0` removes the entry entirely (sparse-map invariant —
    /// the AgentDecisionSystem treats absent keys as zero, so leaving a
    /// `0` entry would still be observable). Non-zero values
    /// insert/overwrite. Callers in the harness use this to populate the
    /// substrate before triggering `Consuming { Food }` transitions.
    pub fn set_food_tile(&mut self, x: u32, y: u32, amount: u8) {
        if amount == 0 {
            self.food_tiles.remove(&(x, y));
        } else {
            self.food_tiles.insert((x, y), amount);
        }
    }

    /// Set the water-tile counter at `(x, y)` (V7 Phase 5-β / P5β-7).
    /// Mirrors [`SimResources::set_food_tile`] for the water channel.
    pub fn set_water_tile(&mut self, x: u32, y: u32, amount: u8) {
        if amount == 0 {
            self.water_tiles.remove(&(x, y));
        } else {
            self.water_tiles.insert((x, y), amount);
        }
    }

    /// Set the sleep-tile counter at `(x, y)` (V7 Phase 5-γ / P5γ-7).
    /// Mirrors [`SimResources::set_food_tile`] for the sleep channel —
    /// `0` removes the entry, non-zero inserts/overwrites.
    pub fn set_sleep_tile(&mut self, x: u32, y: u32, amount: u8) {
        if amount == 0 {
            self.sleep_tiles.remove(&(x, y));
        } else {
            self.sleep_tiles.insert((x, y), amount);
        }
    }

    /// Register (or clear) the FOOD regen ceiling at `(x, y)`
    /// (`add-resource-scarcity-regen`). `max == 0` removes the entry (the
    /// source is no longer regenerated); non-zero inserts/overwrites. Mirrors
    /// [`SimResources::set_food_tile`]'s sparse-map invariant.
    pub fn set_food_source_max(&mut self, x: u32, y: u32, max: u8) {
        if max == 0 {
            self.food_source_max.remove(&(x, y));
        } else {
            self.food_source_max.insert((x, y), max);
        }
    }

    /// Register (or clear) the WATER regen ceiling at `(x, y)`. Mirrors
    /// [`SimResources::set_food_source_max`] for the water channel.
    pub fn set_water_source_max(&mut self, x: u32, y: u32, max: u8) {
        if max == 0 {
            self.water_source_max.remove(&(x, y));
        } else {
            self.water_source_max.insert((x, y), max);
        }
    }

    /// Register (or clear) the SLEEP regen ceiling at `(x, y)`. Mirrors
    /// [`SimResources::set_food_source_max`] for the sleep channel.
    pub fn set_sleep_source_max(&mut self, x: u32, y: u32, max: u8) {
        if max == 0 {
            self.sleep_source_max.remove(&(x, y));
        } else {
            self.sleep_source_max.insert((x, y), max);
        }
    }
}

/// Owns the world, the resources, and the priority-sorted system list.
pub struct SimEngine {
    /// Entity component storage.
    pub world: World,

    /// Shared, non-ECS resources.
    pub resources: SimResources,

    systems: Vec<Box<dyn RuntimeSystem + Send>>,
    current_tick: u64,
}

impl SimEngine {
    /// Build an engine for a `width × height` world. The blocking cache
    /// is derived from `registry` automatically; influence + tile grids
    /// start empty.
    pub fn new(width: u32, height: u32, registry: MaterialRegistry) -> Self {
        Self {
            world: World::new(),
            resources: SimResources::new(width, height, registry),
            systems: Vec::new(),
            current_tick: 0,
        }
    }

    /// Register a new system and re-sort by priority (lower first).
    pub fn register_system(&mut self, system: Box<dyn RuntimeSystem + Send>) {
        self.systems.push(system);
        self.systems.sort_by_key(|s| s.priority());
    }

    /// Number of registered systems. V7 reset baseline = 0.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Names of registered systems in execution order.
    /// Useful for diagnostics; allocates a fresh `Vec` on each call.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.name()).collect()
    }

    /// Run one tick: dispatches every due system, then advances the
    /// tick counter and updates the day/night clock.
    pub fn tick(&mut self) {
        self.resources.current_tick = self.current_tick;
        // V7 viz-D — prune the display-only recent-deaths buffer once entries
        // age past the retain window. `saturating_sub` guards the defensive
        // `entry.tick > current_tick` case (age 0, kept). Order-independent →
        // deterministic.
        let tick = self.current_tick;
        self.resources
            .recent_deaths
            .retain(|d| tick.saturating_sub(d.tick as u64) < RECENT_DEATH_RETAIN_TICKS);
        for system in &mut self.systems {
            if self.current_tick.is_multiple_of(system.tick_interval()) {
                system.tick(&mut self.world, &mut self.resources);
            }
        }
        self.current_tick += 1;
        // V7 Phase 5-γ / P5γ-2 — advance day/night clock at end-of-tick
        // so that after N tick() calls, `time_of_day` reflects
        // `(N % ticks_per_day) * 24 / ticks_per_day`. The zero-guard
        // prevents a division-by-zero panic if a scenario stops the
        // clock by setting `ticks_per_day = 0`.
        self.resources.time_of_day = if self.resources.ticks_per_day == 0 {
            0.0
        } else {
            ((self.current_tick % self.resources.ticks_per_day) as f64
                / self.resources.ticks_per_day as f64)
                * 24.0
        };
    }

    /// Number of completed ticks since construction.
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Spawn an agent at tile-coordinate `(x, y)` (V7 Phase 4-α / P4α-2-a;
    /// Phase 5-α: now mints an [`AgentId`] internally).
    ///
    /// Convenience wrapper over `self.world.spawn((Position, Agent { id }))`.
    /// The id is allocated monotonically via
    /// [`SimResources::issue_agent_id`]; callers should never construct
    /// `Agent { id }` themselves outside of harness migration code.
    /// Returns the freshly-allocated `Entity` so the caller can hold a
    /// stable handle for later queries.
    pub fn spawn_agent(&mut self, x: u32, y: u32) -> Entity {
        let id = self.resources.issue_agent_id();
        self.world.spawn((Position::new(x, y), Agent { id }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::influence::InfluenceChannel;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn empty_registry() -> MaterialRegistry {
        MaterialRegistry::new()
    }

    /// Send-safe mock system that increments an atomic counter on each tick.
    struct AtomicMock {
        name: &'static str,
        priority: u32,
        interval: u64,
        ticks_run: Arc<AtomicU32>,
    }

    impl RuntimeSystem for AtomicMock {
        fn name(&self) -> &str {
            self.name
        }
        fn priority(&self) -> u32 {
            self.priority
        }
        fn tick_interval(&self) -> u64 {
            self.interval
        }
        fn tick(&mut self, _: &mut World, _: &mut SimResources) {
            self.ticks_run.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_engine_new() {
        let engine = SimEngine::new(256, 256, empty_registry());
        assert_eq!(engine.resources.tile_grid.width, 256);
        assert_eq!(engine.resources.tile_grid.height, 256);
        assert_eq!(engine.system_count(), 0);
        assert_eq!(engine.current_tick(), 0);
    }

    #[test]
    fn test_engine_tick_no_systems() {
        // V7 reset baseline = 0 systems — tick must not panic.
        let mut engine = SimEngine::new(256, 256, empty_registry());
        for _ in 0..10 {
            engine.tick();
        }
        assert_eq!(engine.current_tick(), 10);
    }

    #[test]
    fn test_resources_access_initial_zero() {
        let engine = SimEngine::new(256, 256, empty_registry());
        for ch in InfluenceChannel::all() {
            assert_eq!(engine.resources.influence_grid.sample(0, 0, *ch), 0);
            assert_eq!(engine.resources.influence_grid.sample(255, 255, *ch), 0);
        }
    }

    #[test]
    fn test_register_and_run_single_system() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut engine = SimEngine::new(256, 256, empty_registry());
        engine.register_system(Box::new(AtomicMock {
            name: "mock",
            priority: 100,
            interval: 1,
            ticks_run: counter.clone(),
        }));
        assert_eq!(engine.system_count(), 1);
        for _ in 0..5 {
            engine.tick();
        }
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_priority_sort_lower_first() {
        let mut engine = SimEngine::new(256, 256, empty_registry());
        // Register out of priority order; system_names() must reflect sort.
        engine.register_system(Box::new(AtomicMock {
            name: "high",
            priority: 1000,
            interval: 1,
            ticks_run: Arc::new(AtomicU32::new(0)),
        }));
        engine.register_system(Box::new(AtomicMock {
            name: "low",
            priority: 10,
            interval: 1,
            ticks_run: Arc::new(AtomicU32::new(0)),
        }));
        engine.register_system(Box::new(AtomicMock {
            name: "mid",
            priority: 500,
            interval: 1,
            ticks_run: Arc::new(AtomicU32::new(0)),
        }));
        assert_eq!(engine.system_names(), vec!["low", "mid", "high"]);
    }

    #[test]
    fn test_tick_interval_skip() {
        // interval=5 → fires at tick 0, 5, 10, 15 across 20 ticks → 4 runs.
        let counter = Arc::new(AtomicU32::new(0));
        let mut engine = SimEngine::new(256, 256, empty_registry());
        engine.register_system(Box::new(AtomicMock {
            name: "interval5",
            priority: 100,
            interval: 5,
            ticks_run: counter.clone(),
        }));
        for _ in 0..20 {
            engine.tick();
        }
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }
}
