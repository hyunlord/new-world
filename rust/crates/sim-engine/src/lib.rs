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
use sim_core::causal::{CausalLogStorage, EventId};
use sim_core::components::{Agent, AgentId, Position};
use sim_core::influence::{InfluenceGrid, MaterialBlockingCache};
use sim_core::material::MaterialRegistry;
use sim_core::tile::TileGrid;
use std::collections::VecDeque;
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
}

impl SimResources {
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
        let blocking_cache = MaterialBlockingCache::build(&registry);
        Self {
            world: World::new(),
            resources: SimResources {
                tile_grid: TileGrid::new(width, height),
                influence_grid: InfluenceGrid::new(width, height),
                material_registry: registry,
                material_blocking_cache: blocking_cache,
                current_tick: 0,
                building_event_queue: VecDeque::new(),
                causal_log: CausalLogStorage::new(),
                next_event_id: AtomicU64::new(0),
                next_agent_id: AtomicU64::new(0),
            },
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
    /// tick counter.
    pub fn tick(&mut self) {
        self.resources.current_tick = self.current_tick;
        for system in &mut self.systems {
            if self.current_tick.is_multiple_of(system.tick_interval()) {
                system.tick(&mut self.world, &mut self.resources);
            }
        }
        self.current_tick += 1;
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
