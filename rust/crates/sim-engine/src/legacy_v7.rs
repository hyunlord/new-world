//! Frozen format6/7 resource layout, differing from current Chronicle entry widths.

use super::*;

/// Resources decoded using the exact published format6/7 field order.
#[derive(serde::Deserialize)]
pub struct LegacySimResources {
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
    pub chronicle: sim_core::chronicle::legacy_v7::LegacyChronicle,

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

impl From<LegacySimResources> for SimResources {
    fn from(old: LegacySimResources) -> Self {
        Self {
            rules: old.rules,
            world_seed: old.world_seed,
            building_kinds: old.building_kinds,
            site_owner: old.site_owner,
            building_owner: old.building_owner,
            bed_of: old.bed_of,
            shelter_plan_skips: old.shelter_plan_skips,
            food_regen_acc: old.food_regen_acc,
            wood_tiles: old.wood_tiles,
            wood_source_max: old.wood_source_max,
            wood_regen_acc: old.wood_regen_acc,
            wood_winter_acc: old.wood_winter_acc,
            wood_winter_cursor: old.wood_winter_cursor,
            fires: old.fires,
            haulers: old.haulers,
            social_chase_since: old.social_chase_since,
            social_retry_after: old.social_retry_after,
            tile_grid: old.tile_grid,
            influence_grid: old.influence_grid,
            material_registry: old.material_registry,
            material_blocking_cache: old.material_blocking_cache,
            current_tick: old.current_tick,
            steward: old.steward,
            building_event_queue: old.building_event_queue,
            causal_log: old.causal_log,
            next_event_id: old.next_event_id,
            next_agent_id: old.next_agent_id,
            food_tiles: old.food_tiles,
            water_tiles: old.water_tiles,
            sleep_tiles: old.sleep_tiles,
            food_source_max: old.food_source_max,
            water_source_max: old.water_source_max,
            sleep_source_max: old.sleep_source_max,
            relationships: old.relationships,
            interaction_progress: old.interaction_progress,
            combat_pairs: old.combat_pairs,
            combat_progress: old.combat_progress,
            time_of_day: old.time_of_day,
            ticks_per_day: old.ticks_per_day,
            settlements: old.settlements,
            next_settlement_id: old.next_settlement_id,
            building_registry: old.building_registry,
            recent_deaths: old.recent_deaths,
            tallies: old.tallies,
            noise_fork: old.noise_fork,
            world_variety: old.world_variety,
            worldgen_report: old.worldgen_report,
            wild_regions: old.wild_regions,
            wood_spot_area: old.wood_spot_area,
            wood_area_acc: old.wood_area_acc,
            wood_area_cursor: old.wood_area_cursor,
            wood_area_floor_acc: old.wood_area_floor_acc,
            wood_area_floor_sep: old.wood_area_floor_sep,
            retired_hearths: old.retired_hearths,
            fission_log: old.fission_log,
            dissolutions: old.dissolutions,
            fission_checks: old.fission_checks,
            chronicle: old.chronicle.into(),
            settlement_relations: old.settlement_relations,
            colony: old.colony,
            due_births: old.due_births,
            spatial: old.spatial,
            nav: old.nav,
            content_state: Default::default(),
        }
    }
}
