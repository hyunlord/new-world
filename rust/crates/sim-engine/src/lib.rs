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
    Agent, AgentId, BuildingId, BuildingKind, Position, RelationshipKey, RelationshipState,
    Settlement, SettlementId,
};
use sim_core::influence::{InfluenceGrid, MaterialBlockingCache};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_core::tile::TileGrid;
use sim_core::variety::WorldVariety;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

pub use sim_core::rng::NoiseFork;

/// FFI-originated event: a building was placed at `position` with influence
/// `radius` (Chebyshev distance, in tiles). Drained by
/// `sim_systems::runtime::influence::BuildingStampSystem` each tick which
/// translates each event into `InfluenceGrid::mark_dirty` calls on the
/// Warmth/Spiritual/Beauty/Light channels.
///
/// Phase 0 v0.1.3 §5 — R1 event_queue path (T7.7.B land).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BuildingPlacedEvent {
    /// Tile coordinates of the building origin (top-left corner).
    pub position: (u32, u32),
    /// Influence radius in tiles (Chebyshev distance, inclusive).
    pub radius: u32,
}

/// Cumulative population and social counters for display and measurement only.
/// Never read by simulation logic, so they cannot affect determinism.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tallies {
    /// Agents born after world initialization; excludes initial spawns.
    pub births: u64,
    /// Agents lost to hunger saturation.
    pub deaths_starvation: u64,
    /// Agents lost to thirst saturation.
    pub deaths_dehydration: u64,
    /// Agents lost to combat damage.
    pub deaths_combat: u64,
    /// Completed social interactions, counted once per pair rather than per agent.
    pub social_completed: u64,
    /// Units removed from finite food tiles by eating or gathering.
    pub food_harvested: u64,
    /// Hunger relief events from ground meals or stockpile relief.
    pub meals_eaten: u64,
    /// Ground hunger relief events that found no food to consume.
    pub meals_without_food: u64,
    /// Agents lost to old age under the aging rule (Pass 4).
    pub deaths_old_age: u64,
    /// Purposeful forage trips started by the Forage decision arm (Pass 4).
    pub forage_trips: u64,
    /// Stockpile deposits made by members who walked home to haul (Pass 4).
    pub haul_deposits: u64,
    /// Food regrowth units discarded because a source was at its ceiling.
    pub regen_lost_at_cap: u64,
    /// Food units served from a settlement store to a member (Pass 5).
    pub meals_from_store: u64,
    /// Ground meals resolved at a food tile (`Consuming { Food }`) (Pass 5).
    pub ground_meals: u64,
    /// Firewood trips started by the Firewood decision arm (Pass 5).
    pub wood_trips: u64,
    /// Wood units picked up from deadwood spots (Pass 5).
    pub wood_gathered: u64,
    /// Gathering trips interrupted by a critical need (Pass 5).
    pub gather_preempted: u64,
    /// Members marked `Cold`, summed over every hourly fire update (Pass 5).
    pub cold_member_hours: u64,
    /// Firewood trips started while the member's settlement was in urgent mode (Pass 6).
    pub urgent_wood_trips: u64,
    /// Winter regrowth units that found no deadwood spot below the cap (Pass 6).
    pub winter_wood_lost: u64,
    /// Wood gatherers re-targeted because the spot they were walking to was
    /// empty (Pass 7); counted in every version.
    pub wood_retargets: u64,
    /// Agents lost to cold-night wounds under the exposure rule (Pass 10).
    pub deaths_exposure: u64,
    /// Daughter settlements founded by fission (Pass 11).
    pub fissions: u64,
    /// Fissions whose group departed but founded nothing (Pass 11).
    pub fissions_cancelled: u64,
    /// Daily fission checks that found every other condition met but no free
    /// site (Pass 11).
    pub fission_blocked_no_site: u64,
    /// Settlements merged away as remnants (Pass 11).
    pub remnant_merges: u64,
    /// Members moved between settlements by the exclusivity pass of
    /// `SettlementSystem` (Pass 11 measurement; counted in every version).
    pub exclusivity_moves: u64,
    /// Wood added to settled wild regions by the winter-fall floor (Pass 12,
    /// `area_fall_floor_permille`): the units the floor added to the
    /// settled areas' accumulators (before the snow cap).
    pub area_floor_wood: u64,
    /// Winter-fall units lost to the snow cap in a settled wild region
    /// (Pass 12, floor on); included in `winter_wood_lost`.
    pub winter_wood_lost_settled: u64,
    /// Floor units placed from the separate settled-area accumulator (Pass
    /// 13, `FirewoodRule::floor_separate`).
    pub area_floor_placed: u64,
    /// Floor units of the separate accumulator that found no spot below its
    /// `max` (Pass 13, `floor_separate`); included in `winter_wood_lost`.
    pub area_floor_lost_at_max: u64,
    /// Pass 20 (`game_v25`): trips started (one per ordered pair and day).
    pub porter_trips: u64,
    /// Pass 20: porters who departed (one `PorterDeparted` entry each).
    pub porters_departed: u64,
    /// Pass 20: porters who delivered (one `PorterDelivered` entry each).
    pub porter_deliveries: u64,
    /// Pass 20: porters who turned back before delivering (a `PorterDelivered`
    /// entry with cause ≥ 10 each).
    pub porter_turnbacks: u64,
    /// Pass 20: porters whose load was lost (the porter died or its village
    /// dissolved).
    pub porter_loads_lost: u64,
    /// Pass 20: units taken from senders' stores.
    pub porter_units_withdrawn: u64,
    /// Pass 20: food units stored in receivers' stores.
    pub porter_food_delivered: u64,
    /// Pass 20: wood units stored in receivers' stores.
    pub porter_wood_delivered: u64,
    /// Pass 20: units carried back into the sender's store.
    pub porter_units_returned: u64,
    /// Pass 20: units lost with their porter or village.
    pub porter_units_lost: u64,
    /// Pass 20: forage disputes between foragers of two villages.
    pub forage_disputes: u64,
    /// Pass 20: disputes that became quarrels (one `Quarrel` entry each).
    pub quarrels: u64,
    /// Pass 20: pairs that became estranged (one `VillagesEstranged` each).
    pub villages_estranged: u64,
    /// Pass 20: pairs that reconciled (one `VillagesReconciled` each).
    pub villages_reconciled: u64,
    /// Pass 21 (`game_v26`): disputes past the pair's daily dispute cap
    /// (counted in `forage_disputes`, no `dispute_loss`).
    pub disputes_uncosted: u64,
    /// Pass 21: disputes that would have drawn for a quarrel but met the
    /// pair's daily quarrel cap (no draw; the pickup proceeds).
    pub quarrels_capped: u64,
    /// Pass 21: quarrels between villages already estranged (counted under
    /// every relations rule; a subset of `quarrels`).
    pub quarrels_while_estranged: u64,
}

/// A settlement's hearth fire, updated hourly by `FireSystem` (Pass 5).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FireState {
    /// Whether the last hourly update burned a full hour of wood.
    pub lit: bool,
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
/// affect determinism. It is saved with the rest of `SimResources` (M1.5
/// save files) so a loaded game shows the same recent markers.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

    /// M1.5: the tick within each interval at which the system runs: it is
    /// due when `current_tick % n == tick_offset % n` (`n` the interval).
    /// Default 0 (the old rule). A daily system at a clock time returns that
    /// time's tick within the day (`FertilitySystem`: 12:00).
    fn tick_offset(&self, _resources: &SimResources) -> u64 {
        0
    }

    /// Per-tick work. Called by [`SimEngine::tick`] when due.
    fn tick(&mut self, world: &mut World, resources: &mut SimResources);

    /// M1.5 save files: the system's internal state as bytes (its own
    /// encoding; hash collections in sorted key order so two saves of one
    /// state are equal). Default: no state (empty). A system that keeps
    /// state across ticks implements this and [`RuntimeSystem::load_state`];
    /// a pure cache of other state may stay empty and rebuild itself.
    fn save_state(&self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }

    /// Put back the state written by [`RuntimeSystem::save_state`] into a
    /// freshly constructed system. Default: accepts only empty bytes.
    fn load_state(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "{} keeps no state but the save holds {} bytes for it",
                self.name(),
                bytes.len()
            ))
        }
    }
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

/// An empty blocking cache: the value a loaded save starts with before
/// `sim_systems::save::load` rebuilds it from the loaded material registry
/// (the cache is derived state and is not written to save files).
fn empty_blocking_cache() -> MaterialBlockingCache {
    MaterialBlockingCache::build(&MaterialRegistry::new())
}

/// Shared world state owned by the engine.
///
/// `hecs` enforces single-writer semantics on the [`World`]; this struct
/// applies the same convention to non-ECS resources by handing every
/// system a `&mut SimResources` and trusting priorities to serialise
/// access.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SimResources {
    /// Construction-time policy; default preserves every legacy behavior.
    pub rules: WorldRules,
    /// Generation seed; legacy fixtures use zero.
    pub world_seed: u64,
    /// Building categories keyed by origin tile; absent categories mean Hearth.
    pub building_kinds: BTreeMap<(u32, u32), BuildingKind>,
    /// Settlement owner of each automatically planned construction site.
    pub site_owner: BTreeMap<(u32, u32), SettlementId>,
    /// Completed building ownership; populated only under owned housing rules.
    pub building_owner: BTreeMap<(u32, u32), SettlementId>,
    /// Assigned shelter tile per resident; populated only with night rest enabled.
    pub bed_of: BTreeMap<AgentId, (u32, u32)>,
    /// Display-only count of planning attempts with no qualifying tile.
    pub shelter_plan_skips: u64,
    /// Seasonal food regrowth accumulator in thousandths of a unit; stays zero
    /// without the season rule.
    pub food_regen_acc: u32,
    /// Deadwood amount per tile; empty without deadwood worldgen (Pass 5).
    pub wood_tiles: BTreeMap<(u32, u32), u8>,
    /// Deadwood regrowth ceiling per registered spot; empty without deadwood worldgen.
    pub wood_source_max: BTreeMap<(u32, u32), u8>,
    /// Wood regrowth accumulator in thousandths of a unit; stays zero without
    /// the firewood rule.
    pub wood_regen_acc: u32,
    /// Winter spread-regrowth accumulator (Pass 6) in thousandths of a unit
    /// times pulses per day; stays zero without `winter_regen_units_per_day`.
    pub wood_winter_acc: u64,
    /// Last deadwood spot that received a winter unit (Pass 6); `None` until
    /// the first one lands.
    pub wood_winter_cursor: Option<(u32, u32)>,
    /// Hearth fire per settlement; empty without the firewood rule.
    pub fires: BTreeMap<SettlementId, FireState>,
    /// Members whose last movement was a haul step home; display-only, used to
    /// count `Tallies::haul_deposits`. Empty without the forage rule.
    pub haulers: BTreeSet<AgentId>,
    /// Tick each active non-migrant social chase began; empty in legacy worlds.
    pub social_chase_since: BTreeMap<AgentId, u64>,
    /// Earliest tick each agent may retry a failed social chase.
    pub social_retry_after: BTreeMap<AgentId, u64>,

    /// Tile grid (T7.1 land — wall / floor / terrain SoA).
    pub tile_grid: TileGrid,

    /// 8-channel double-buffered influence grid (T7.3 land).
    pub influence_grid: InfluenceGrid,

    /// Material catalogue (T6.6 ~ T6.8 land — 105 materials).
    pub material_registry: MaterialRegistry,

    /// Pre-computed `(material, channel) → block` lookup (T7.4 land).
    #[serde(skip, default = "empty_blocking_cache")]
    pub material_blocking_cache: MaterialBlockingCache,

    /// Current tick — refreshed by [`SimEngine::tick`] before systems run.
    pub current_tick: u64,

    /// Saved separately in the v7 body so the v6 resource layout stays readable.
    #[serde(skip)]
    pub steward: sim_core::colony::steward::StewardState,

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
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub food_tiles: HashMap<(u32, u32), u8>,

    /// Sparse water-tile substrate (V7 Phase 5-β / P5β-7). Mirrors
    /// [`SimResources::food_tiles`] for the second need.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub water_tiles: HashMap<(u32, u32), u8>,

    /// Sparse sleep-tile substrate (V7 Phase 5-γ / P5γ-7). Mirrors
    /// [`SimResources::food_tiles`] for the third need.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub sleep_tiles: HashMap<(u32, u32), u8>,

    /// Regen ceiling per registered FOOD source (`add-resource-scarcity-regen`).
    ///
    /// Maps `(x, y) → original finite capacity` for each food source the
    /// production scene seeds via `seed_finite_resource_scarcity`. The
    /// `ResourceRegenSystem` (priority 140) periodically refills
    /// [`SimResources::food_tiles`] toward this ceiling. Empty by default — a
    /// harness that does NOT seed finite scarcity leaves it empty, so regen is
    /// a pure no-op there (the 12 shared harnesses are unperturbed).
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub food_source_max: HashMap<(u32, u32), u8>,

    /// Regen ceiling per registered WATER source. Mirrors
    /// [`SimResources::food_source_max`] for the water channel.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub water_source_max: HashMap<(u32, u32), u8>,

    /// Regen ceiling per registered SLEEP source. Mirrors
    /// [`SimResources::food_source_max`] for the sleep channel.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub sleep_source_max: HashMap<(u32, u32), u8>,

    /// Sparse per-pair relationship state (V7 Phase 7-β / P7β-13).
    /// Bumped by `SocialInteractionSystem` on `SocialInteractionCompleted`.
    /// Starts empty; never seeded by engine construction.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub relationships: HashMap<RelationshipKey, RelationshipState>,

    /// Sparse per-pair multi-tick interaction progress counter
    /// (V7 Phase 7-β / P7β-13). Incremented once per mutual
    /// `Consuming{Agent(other)}` pair per tick by `SocialInteractionSystem`;
    /// removed on completion or asymmetric-partner fallback. Starts empty.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub interaction_progress: HashMap<RelationshipKey, u32>,

    /// Active combat pairs for the current tick. Keys are canonical
    /// `(smaller AgentId, larger AgentId)` tuples inserted by
    /// `AgentDecisionSystem` when the combat cascade arm fires.
    /// `CombatSystem` (priority 137) consumes and removes completed pairs.
    /// Phase 9-β / P9β-1.
    #[serde(serialize_with = "sim_core::save_serde::sorted_set")]
    pub combat_pairs: HashSet<(AgentId, AgentId)>,

    /// Per-pair combat progress counter. Keyed by the same canonical
    /// `(smaller, larger)` tuple as `combat_pairs`. `CombatSystem`
    /// increments each pair's counter until it reaches
    /// `REQUIRED_COMBAT_PROGRESS`, at which point `CombatCompleted` fires.
    /// Phase 9-β / P9β-1.
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
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
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
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
    #[serde(serialize_with = "sim_core::save_serde::sorted_map")]
    pub building_registry: HashMap<BuildingId, (u32, u32)>,

    /// Bounded, display-only buffer of recent agent deaths (V7 viz-D
    /// `show-death-visual`). Pushed by the shared `survival::despawn_agent`
    /// helper, pruned each tick by [`SimEngine::tick`] once entries age past
    /// [`RECENT_DEATH_RETAIN_TICKS`]. Read by the `get_recent_deaths` FFI for
    /// the death-marker overlay. Never read by simulation logic (determinism-
    /// safe); not serde-serialized.
    pub recent_deaths: Vec<RecentDeath>,

    /// Lifetime population counters; independent of bounded event buffers.
    pub tallies: Tallies,

    /// Noise fork (Pass 9): from its tick on, keyed draws use its replicate.
    /// `None` (the default and every unforked run) means replicate 0.
    pub noise_fork: Option<NoiseFork>,

    /// The hidden world parameters this world was built from (Pass 10);
    /// `None` outside `game_v16`. Measurement only, never read by systems.
    pub world_variety: Option<WorldVariety>,

    /// What worldgen placed against what it was asked for (Pass 10).
    /// Measurement only, never read by systems.
    pub worldgen_report: WorldGenReport,

    /// Wild regions placed by worldgen (Pass 11, `game_v17`), in region
    /// order; empty in every earlier world. `FissionSystem` reads the centres
    /// as founding sites.
    pub wild_regions: Vec<WildRegion>,

    /// Area of each deadwood spot (Pass 11, `game_v17`): 0 = the camp area,
    /// r + 1 = wild region r. Empty in every earlier world. Read by the
    /// regional winter fall (`FirewoodRule::regional_wood`).
    pub wood_spot_area: BTreeMap<(u32, u32), u8>,

    /// Per-area winter-fall accumulators (Pass 11, regional wood only).
    pub wood_area_acc: BTreeMap<u8, u64>,

    /// Per-area winter-fall cursors (Pass 11, regional wood only).
    pub wood_area_cursor: BTreeMap<u8, (u32, u32)>,

    /// Pass 12: measurement-only accumulator of the extra the winter-fall
    /// floor adds (feeds `Tallies::area_floor_wood`; never read by systems).
    pub wood_area_floor_acc: u64,

    /// Pass 13 (`FirewoodRule::floor_separate`): per-area accumulators of the
    /// settled-area floor's extra, placed with ceiling `max`. Empty otherwise.
    pub wood_area_floor_sep: BTreeMap<u8, u64>,

    /// Hearth tiles that never anchor a new settlement again (Pass 11): the
    /// hearth of a cancelled founding, a merged remnant or a settlement that
    /// lost every member. Empty outside `game_v17`.
    pub retired_hearths: BTreeSet<(u32, u32)>,

    /// Every fission that departed (Pass 11), in departure order; read by
    /// `FissionSystem` and, for a settlement's split count when it forms
    /// (Pass 13 threshold key), by `SettlementSystem`.
    pub fission_log: Vec<FissionRecord>,

    /// Every settlement dissolution as (settlement, tick) (Pass 11);
    /// measurement only, never read by systems.
    pub dissolutions: Vec<(SettlementId, u64)>,

    /// Every daily fission check that reached the hazard draw (Pass 12,
    /// `FissionRule::hazard`), in check order; measurement only, never read
    /// by systems. Empty without a hazard.
    pub fission_checks: Vec<FissionCheck>,

    /// Pass 19: the chronicle — an append-only record of every life and
    /// village and a person table that outlives death. Write-only for the
    /// simulation (no system decides anything from it); read by probes and
    /// the bridge. Recorded in every scene.
    pub chronicle: sim_core::chronicle::Chronicle,

    /// Pass 20 (`game_v25`): every village pair's relation, the porter
    /// trips in flight and the measurement-only trip and decision logs.
    /// Empty without `SocietyRules::relations`.
    pub settlement_relations: sim_core::components::SettlementRelations,

    /// M1 colony: trees, ground items, zones, designations, footprints,
    /// reservations, the command log and the item ledger. Empty without
    /// `rules.colony` (the baseline world never fills it).
    pub colony: sim_core::colony::ColonyState,

    /// M1.5 births that are due (`BirthRule::Fertility`): `FertilitySystem`
    /// queues them, `SettlementSystem` spawns them on the next tick. Always
    /// empty under any other birth rule.
    pub due_births: Vec<sim_core::components::DueBirth>,

    /// M1.5 spatial indexes (ADR 0016 C3; `sim_core::spatial`): derived
    /// state, rebuilt from the world (the colony indexes when the colony's
    /// `overlay_version` changes, the agents' on demand once per tick). Never
    /// read in the baseline world. Not saved: a loaded engine starts with
    /// empty indexes and rebuilds them on first use (`ColonyIndex::version`
    /// is `None`, the agents' tick is `None`).
    #[serde(skip)]
    pub spatial: sim_core::spatial::SpatialIndex,

    /// M3 navigation (ADR 0021; `sim_core::nav`): the colony's version
    /// counter, every walking agent's path and this tick's unreachable goals
    /// (saved); the cost map, regions and memo are derived and rebuilt on
    /// first use. Never built in the baseline world.
    #[serde(default)]
    pub nav: sim_core::nav::NavState,
}

/// One daily fission check that reached the hazard draw (Pass 12);
/// measurement only. Factors and the hazard in thousandths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FissionCheck {
    /// Tick of the check.
    pub tick: u64,
    /// The settlement checked.
    pub settlement: SettlementId,
    /// Its members.
    pub members: u32,
    /// `sustain_capacity`.
    pub capacity: u32,
    /// Its stockpile food.
    pub food: u32,
    /// Size factor.
    pub g: i64,
    /// Crowding factor.
    pub q: i64,
    /// Scarcity factor.
    pub s: i64,
    /// Hazard.
    pub h: i64,
    /// The keyed draw in 0..1000.
    pub draw: u64,
    /// `draw < h`.
    pub split: bool,
    /// Whether a free site existed.
    pub site_free: bool,
    /// Pass 13 (`FissionRule::pressure`): the pressure P after today's
    /// accumulation (0 without pressure; the draw is then 0 and `split` is
    /// P ≥ θ and P > 0).
    pub pressure: u64,
    /// Pass 13: the settlement's threshold θ (0 without pressure).
    pub threshold: u64,
}

/// One departed fission group (Pass 11); measurement only.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FissionRecord {
    /// `FissionSystem` counter, from 1.
    pub fission_id: u32,
    /// The settlement the group left.
    pub parent: SettlementId,
    /// The founding site (a wild-region centre).
    pub site: (u32, u32),
    /// Tick of departure.
    pub depart_tick: u64,
    /// Group members, ascending.
    pub members: Vec<AgentId>,
    /// Adults in the group.
    pub adults: u32,
    /// The daughter settlement, once founded.
    pub daughter: Option<SettlementId>,
    /// Tick the daughter formed.
    pub founded_tick: Option<u64>,
    /// Tick the fission was cancelled.
    pub cancelled_tick: Option<u64>,
    /// Food and wood the group took from the parent's store.
    pub food_taken: u32,
    /// See `food_taken`.
    pub wood_taken: u32,
    /// Whether the group settled where its leader stood after the timeout.
    pub settled_at_leader: bool,
    /// Pass 13: the parent's pressure P at the split (0 without pressure).
    pub pressure_at_split: u64,
    /// Pass 13: the parent's threshold θ at the split (0 without pressure).
    pub threshold_at_split: u64,
}

/// One wild region placed by worldgen (Pass 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WildRegion {
    /// Centre tile (a walkable, non-water tile 22–28 from the camp hearth).
    pub centre: (u32, u32),
    /// Bushes placed around it.
    pub bushes: u32,
    /// Deadwood spots placed around it.
    pub deadwood: u32,
    /// Chebyshev distance from the centre to the nearest water tile
    /// (`u32::MAX` without water); measurement only.
    pub water_distance: u32,
}

/// Worldgen counts kept for measurement (Pass 10): never read by systems.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorldGenReport {
    /// Deadwood spots the generator was asked for.
    pub deadwood_spots_requested: u32,
    /// Deadwood spots it placed (fewer when the radius band ran out of tiles).
    pub deadwood_spots_placed: u32,
    /// Terrain label swaps made by the berry-edge repair.
    pub berry_repair_swaps: u32,
    /// Wild regions asked for (Pass 11).
    pub regions_requested: u32,
    /// Wild regions placed (fewer when no candidate centre was left).
    pub regions_placed: u32,
    /// Region bushes asked for over the placed regions.
    pub region_bushes_requested: u32,
    /// Region bushes placed.
    pub region_bushes_placed: u32,
    /// Region deadwood spots asked for over the placed regions.
    pub region_deadwood_requested: u32,
    /// Region deadwood spots placed.
    pub region_deadwood_placed: u32,
    /// Region bushes added by the Pass 12 fill (`region_fill`), by tier 2
    /// (Forest in the ring), 3 (Plain with a Forest neighbour in the ring),
    /// 4 (either, ring 7–9) and 5 (any Plain, ring 2–9); included in
    /// `region_bushes_placed`.
    pub region_fill_bushes: [u32; 4],
    /// Region deadwood spots added by the Pass 12 fill (tier 2, ring 7–9);
    /// included in `region_deadwood_placed`.
    pub region_fill_deadwood: u32,
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
            rules: WorldRules::default(),
            world_seed: 0,
            building_kinds: BTreeMap::new(),
            site_owner: BTreeMap::new(),
            building_owner: BTreeMap::new(),
            bed_of: BTreeMap::new(),
            shelter_plan_skips: 0,
            food_regen_acc: 0,
            wood_tiles: BTreeMap::new(),
            wood_source_max: BTreeMap::new(),
            wood_regen_acc: 0,
            wood_winter_acc: 0,
            wood_winter_cursor: None,
            fires: BTreeMap::new(),
            haulers: BTreeSet::new(),
            social_chase_since: BTreeMap::new(),
            social_retry_after: BTreeMap::new(),
            tile_grid: TileGrid::new(width, height),
            influence_grid: InfluenceGrid::new(width, height),
            material_registry: registry,
            material_blocking_cache: blocking_cache,
            current_tick: 0,
            steward: Default::default(),
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
            tallies: Tallies::default(),
            noise_fork: None,
            world_variety: None,
            worldgen_report: WorldGenReport::default(),
            wild_regions: Vec::new(),
            wood_spot_area: BTreeMap::new(),
            wood_area_acc: BTreeMap::new(),
            wood_area_cursor: BTreeMap::new(),
            wood_area_floor_acc: 0,
            wood_area_floor_sep: BTreeMap::new(),
            retired_hearths: BTreeSet::new(),
            fission_log: Vec::new(),
            dissolutions: Vec::new(),
            fission_checks: Vec::new(),
            chronicle: sim_core::chronicle::Chronicle::new(),
            settlement_relations: sim_core::components::SettlementRelations::default(),
            colony: sim_core::colony::ColonyState {
                next_zone_id: 1,
                ..Default::default()
            },
            due_births: Vec::new(),
            spatial: sim_core::spatial::SpatialIndex::default(),
            nav: sim_core::nav::NavState::default(),
        }
    }

    /// The noise replicate keyed draws use at simulation tick `tick` (Pass 9):
    /// the fork's replicate from its tick on, 0 before it and without a fork.
    pub fn noise_replicate_for(&self, tick: u64) -> u32 {
        self.noise_fork.map_or(0, |fork| fork.replicate_at(tick))
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

/// Wall-time bookkeeping of one registered system inside a [`TickProfile`]
/// (Revitalize Pass 15, measurement only).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemProfile {
    /// The system's [`RuntimeSystem::name`].
    pub name: String,
    /// The system's [`RuntimeSystem::priority`].
    pub priority: u32,
    /// The system's [`RuntimeSystem::tick_interval`].
    pub interval: u64,
    /// Ticks in which the system was due and ran.
    pub calls: u64,
    /// Summed wall time of those runs, nanoseconds (saturating).
    pub total_ns: u64,
    /// Longest single run, nanoseconds.
    pub max_ns: u64,
}

/// Per-system and per-tick wall times gathered by [`SimEngine::tick_profiled`]
/// (Revitalize Pass 15, measurement only; nothing in the simulation reads it).
///
/// `systems` and `last_tick_ns` follow the engine's execution order (the
/// order [`SimEngine::system_names`] returns). Build one with
/// [`SimEngine::new_tick_profile`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickProfile {
    /// One entry per registered system, in execution order.
    pub systems: Vec<SystemProfile>,
    /// Profiled ticks so far.
    pub ticks: u64,
    /// Wall time of every profiled whole tick, microseconds (rounded to the
    /// nearest µs, saturating at `u32::MAX`); `tick_us.len() == ticks`.
    pub tick_us: Vec<u32>,
    /// Summed wall time of the recent-deaths prune step, nanoseconds.
    pub prune_ns: u64,
    /// Each system's wall time in the most recent profiled tick,
    /// nanoseconds; 0 for a system that was not due in that tick.
    pub last_tick_ns: Vec<u64>,
}

impl TickProfile {
    /// Add one run of the system at `index` that took `ns` nanoseconds.
    fn record_system(&mut self, index: usize, ns: u64) {
        let entry = &mut self.systems[index];
        entry.calls += 1;
        entry.total_ns = entry.total_ns.saturating_add(ns);
        entry.max_ns = entry.max_ns.max(ns);
        self.last_tick_ns[index] = ns;
    }
}

/// Elapsed nanoseconds since `start`, saturating at `u64::MAX`.
fn elapsed_ns(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
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
        self.tick_inner(None);
    }

    /// Run one tick exactly as [`SimEngine::tick`] does and add its wall
    /// times to `profile` (Revitalize Pass 15, measurement only: the
    /// simulation never reads the profile, so a profiled run reaches the same
    /// state as an unprofiled one). If `profile` does not hold one entry per
    /// registered system it is first replaced by [`SimEngine::new_tick_profile`]
    /// (its earlier contents are discarded; no panic).
    pub fn tick_profiled(&mut self, profile: &mut TickProfile) {
        if profile.systems.len() != self.systems.len()
            || profile.last_tick_ns.len() != self.systems.len()
        {
            *profile = self.new_tick_profile();
        }
        self.tick_inner(Some(profile));
    }

    /// An empty [`TickProfile`] for this engine's registered systems, in
    /// execution order.
    pub fn new_tick_profile(&self) -> TickProfile {
        TickProfile {
            systems: self
                .systems
                .iter()
                .map(|s| SystemProfile {
                    name: s.name().to_string(),
                    priority: s.priority(),
                    interval: s.tick_interval(),
                    ..SystemProfile::default()
                })
                .collect(),
            last_tick_ns: vec![0; self.systems.len()],
            ..TickProfile::default()
        }
    }

    /// The body of [`SimEngine::tick`]; `std::time::Instant` is read only
    /// when `profile` is given, and the simulation statements are the same
    /// statements in the same order either way.
    fn tick_inner(&mut self, mut profile: Option<&mut TickProfile>) {
        let tick_start = profile.as_ref().map(|_| std::time::Instant::now());
        self.resources.current_tick = self.current_tick;
        // V7 viz-D — prune the display-only recent-deaths buffer once entries
        // age past the retain window. `saturating_sub` guards the defensive
        // `entry.tick > current_tick` case (age 0, kept). Order-independent →
        // deterministic.
        let tick = self.current_tick;
        let prune_start = profile.as_ref().map(|_| std::time::Instant::now());
        self.resources
            .recent_deaths
            .retain(|d| tick.saturating_sub(d.tick as u64) < RECENT_DEATH_RETAIN_TICKS);
        if let (Some(p), Some(start)) = (profile.as_deref_mut(), prune_start) {
            p.prune_ns = p.prune_ns.saturating_add(elapsed_ns(start));
        }
        for (index, system) in self.systems.iter_mut().enumerate() {
            let interval = system.tick_interval();
            let offset = system.tick_offset(&self.resources);
            let due = if offset == 0 {
                self.current_tick.is_multiple_of(interval)
            } else {
                interval != 0 && self.current_tick % interval == offset % interval
            };
            if due {
                match profile.as_deref_mut() {
                    None => system.tick(&mut self.world, &mut self.resources),
                    Some(p) => {
                        let start = std::time::Instant::now();
                        system.tick(&mut self.world, &mut self.resources);
                        p.record_system(index, elapsed_ns(start));
                    }
                }
            } else if let Some(p) = profile.as_deref_mut() {
                p.last_tick_ns[index] = 0;
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
            ((self
                .current_tick
                .saturating_add(self.resources.rules.clock_offset_ticks)
                % self.resources.ticks_per_day) as f64
                / self.resources.ticks_per_day as f64)
                * 24.0
        };
        if let (Some(p), Some(start)) = (profile, tick_start) {
            let us = elapsed_ns(start).saturating_add(500) / 1000;
            p.tick_us.push(u32::try_from(us).unwrap_or(u32::MAX));
            p.ticks += 1;
        }
    }

    /// Number of completed ticks since construction.
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// M1.5 save files: `(name, state)` of every registered system in
    /// execution order ([`RuntimeSystem::save_state`]).
    pub fn system_states(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        self.systems
            .iter()
            .map(|s| Ok((s.name().to_string(), s.save_state()?)))
            .collect()
    }

    /// M1.5 save files: replace the world, the resources and the tick
    /// counter, and load every registered system's state from `states`
    /// (as [`SimEngine::system_states`] wrote it). The registered systems
    /// must match `states` one to one, by position and name; otherwise
    /// nothing is replaced and an error names the first mismatch. A system
    /// whose `load_state` fails leaves the engine half-loaded: discard it.
    pub fn install_saved(
        &mut self,
        world: World,
        resources: SimResources,
        current_tick: u64,
        states: &[(String, Vec<u8>)],
    ) -> Result<(), String> {
        if states.len() != self.systems.len() {
            return Err(format!(
                "the save lists {} systems, the engine registers {}",
                states.len(),
                self.systems.len()
            ));
        }
        for (system, (name, _)) in self.systems.iter().zip(states) {
            if system.name() != name {
                return Err(format!(
                    "system mismatch: save has {name}, engine has {}",
                    system.name()
                ));
            }
        }
        for (system, (_, bytes)) in self.systems.iter_mut().zip(states) {
            system.load_state(bytes)?;
        }
        self.world = world;
        self.resources = resources;
        self.current_tick = current_tick;
        Ok(())
    }

    /// Fork the noise (Pass 9): from the current tick on (the next `tick()`
    /// call sees `current_tick` equal to it) every keyed draw uses `replicate`.
    /// Replicate 0 is the canonical run, so it is a no-op by definition.
    pub fn apply_noise_fork(&mut self, replicate: u32) {
        if replicate == 0 {
            return;
        }
        self.resources.noise_fork = Some(NoiseFork {
            tick: self.current_tick,
            replicate,
        });
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

/// Fork the noise of `engine` at its current tick (Pass 9 section 7.3); see
/// [`SimEngine::apply_noise_fork`]. Replicate 0 is a no-op.
pub fn apply_noise_fork(engine: &mut SimEngine, replicate: u32) {
    engine.apply_noise_fork(replicate);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::influence::InfluenceChannel;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn noise_fork_starts_at_the_current_tick_and_replicate_zero_is_a_no_op() {
        let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
        assert_eq!(engine.resources.noise_replicate_for(0), 0);
        apply_noise_fork(&mut engine, 0);
        assert_eq!(engine.resources.noise_fork, None);
        engine.tick();
        engine.tick();
        apply_noise_fork(&mut engine, 5);
        assert_eq!(
            engine.resources.noise_fork,
            Some(NoiseFork {
                tick: 2,
                replicate: 5
            })
        );
        assert_eq!(
            (
                engine.resources.noise_replicate_for(1),
                engine.resources.noise_replicate_for(2)
            ),
            (0, 5)
        );
    }

    #[test]
    fn generated_world_resources_start_legacy_and_empty() {
        let resources = SimResources::new(64, 64, MaterialRegistry::new());
        assert_eq!(resources.rules, WorldRules::default());
        assert_eq!(resources.world_seed, 0);
        assert!(resources.building_kinds.is_empty());
        assert!(resources.site_owner.is_empty());
        assert_eq!(resources.shelter_plan_skips, 0);
    }

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
    fn tick_profile_counts_due_ticks_and_every_whole_tick() {
        // Pass 15 6.1: two dummy systems (interval 1 and 3) over 7 ticks.
        let every = Arc::new(AtomicU32::new(0));
        let third = Arc::new(AtomicU32::new(0));
        let mut engine = SimEngine::new(32, 32, empty_registry());
        engine.register_system(Box::new(AtomicMock {
            name: "third",
            priority: 20,
            interval: 3,
            ticks_run: third.clone(),
        }));
        engine.register_system(Box::new(AtomicMock {
            name: "every",
            priority: 10,
            interval: 1,
            ticks_run: every.clone(),
        }));
        let mut profile = engine.new_tick_profile();
        let names: Vec<&str> = profile.systems.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, engine.system_names());
        assert_eq!(
            (profile.systems[0].priority, profile.systems[1].interval),
            (10, 3)
        );
        for _ in 0..7 {
            engine.tick_profiled(&mut profile);
        }
        // Ticks 0..7: "third" is due at 0, 3, 6.
        assert_eq!((profile.systems[0].calls, profile.systems[1].calls), (7, 3));
        assert_eq!(
            (every.load(Ordering::SeqCst), third.load(Ordering::SeqCst)),
            (7, 3)
        );
        assert_eq!(profile.ticks, 7);
        assert_eq!(profile.tick_us.len() as u64, profile.ticks);
        assert!(profile.systems.iter().all(|s| s.max_ns <= s.total_ns));
        assert_eq!(engine.current_tick(), 7);
        // Tick 7 is not a multiple of 3: "third" was not due in the latest tick.
        engine.tick_profiled(&mut profile);
        assert_eq!(profile.last_tick_ns[1], 0);
        assert_eq!(profile.systems[1].calls, 3);
        assert_eq!(profile.tick_us.len(), 8);
    }

    #[test]
    fn tick_profile_with_the_wrong_system_count_is_rebuilt() {
        let mut engine = SimEngine::new(32, 32, empty_registry());
        engine.register_system(Box::new(AtomicMock {
            name: "a",
            priority: 1,
            interval: 1,
            ticks_run: Arc::new(AtomicU32::new(0)),
        }));
        let mut profile = TickProfile {
            ticks: 99,
            tick_us: vec![1; 99],
            ..TickProfile::default()
        };
        engine.tick_profiled(&mut profile);
        assert_eq!(profile.systems.len(), 1);
        assert_eq!(profile.last_tick_ns.len(), 1);
        assert_eq!(
            (
                profile.ticks,
                profile.tick_us.len(),
                profile.systems[0].calls
            ),
            (1, 1, 1)
        );
    }

    #[test]
    fn profiled_and_plain_ticks_advance_the_same_clock() {
        let mut plain = SimEngine::new(32, 32, empty_registry());
        let mut profiled = SimEngine::new(32, 32, empty_registry());
        let mut profile = profiled.new_tick_profile();
        for _ in 0..50 {
            plain.tick();
            profiled.tick_profiled(&mut profile);
        }
        assert_eq!(plain.current_tick(), profiled.current_tick());
        assert_eq!(
            plain.resources.time_of_day.to_bits(),
            profiled.resources.time_of_day.to_bits()
        );
        assert_eq!(
            plain.resources.current_tick,
            profiled.resources.current_tick
        );
        assert_eq!(profile.ticks, 50);
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
