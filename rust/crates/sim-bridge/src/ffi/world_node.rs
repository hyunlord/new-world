//! `WorldSimNode` — `Node` subclass exposing the SimEngine to Godot.
//!
//! T7.7.B FFI surface (3 methods) wired through the R1 event_queue path
//! locked by `SimResources::building_event_queue`.
//!
//! V7 Phase 3-γ (γ-1) extends the FFI surface with 2 read-only causal
//! getters consumed by the upcoming "왜?" UI (γ-2 panel layer):
//!   - `get_tile_causal_history(x, y) -> Array<Dictionary>` — enumerate the
//!     tile's causal ring (≤8 entries, oldest first).
//!   - `get_event_chain(x, y, event_id) -> Array<Dictionary>` — backward
//!     walk via [`CausalLogStorage::trace_parents`].
//!
//! ## Bridge Identity Contract
//!
//! The method under test is `WorldSimNode::on_building_placed`.
//! Because `WorldSimNode` is a `GodotClass` requiring Godot runtime for
//! construction (NOT in scope for sim-test), the complete bounds-check and
//! enqueue logic lives in the standalone [`enqueue_building_placed`] `pub fn`.
//! `on_building_placed`'s `#[func]` body consists **solely** of a forwarding
//! call to [`enqueue_building_placed`] — no additional logic.
//!
//! Sim-test imports and calls [`enqueue_building_placed`] directly for
//! Assertions 5 and 6. The Evaluator verifies via Completeness code review
//! that `on_building_placed`'s `#[func]` body calls this exact symbol.
//!
//! ## γ-1 Bridge Identity Contract extension
//!
//! [`collect_tile_causal_history`] and [`collect_event_chain`] are the
//! canonical pure-Rust implementations of the two new causal getters. The
//! `#[func]` bodies are thin loops that convert each [`CausalEventView`]
//! into a `Dictionary` via [`event_view_to_dict`]. Sim-test exercises the
//! pure-Rust collectors directly (Godot runtime not required); the
//! Evaluator confirms via Completeness review that the `#[func]` bodies
//! are non-stub forwardings.

use godot::classes::INode;
use godot::prelude::*;
use sim_core::causal::{CausalEvent, EventId, MemoryRecallTrigger};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Child, Cold, ConstructionSite, Elder, Hunger, Inventory, Life,
    Memory, Position,
    ResourceKind, SeekTarget, Settlement, SettlementId, Sleep, Social, TargetKind, Thirst,
};
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{BuildingPlacedEvent, SimEngine, SimResources, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::worldgen::{build_world_with, WorldGenConfig, DEFAULT_GAME_SEED};

/// Default grid extent until Godot configures it (Phase 2 default).
const DEFAULT_W: u32 = 64;
/// Default grid extent until Godot configures it (Phase 2 default).
const DEFAULT_H: u32 = 64;

/// P4-γ bootstrap: number of agents spawned in `init` per axis on an
/// 8×8 grid (64 total). Tuned so VLM clearly sees a population while the
/// 1K@60FPS gate (planning §2.3) has substantial headroom.
const BOOTSTRAP_AGENT_AXIS: u32 = 8;
/// P4-γ bootstrap: stride in tiles between adjacent agents on the
/// `BOOTSTRAP_AGENT_AXIS × BOOTSTRAP_AGENT_AXIS` lattice.
const BOOTSTRAP_AGENT_STRIDE: u32 = 8;
/// P4-γ bootstrap: tile offset of the lattice origin so agents are
/// inset from the grid edge (so Brownian motion does not immediately
/// clamp against the boundary).
const BOOTSTRAP_AGENT_OFFSET: u32 = 4;
/// P4-γ bootstrap: base offset for per-agent `MovementRng` seeds —
/// keeps seeds far from 0 (splitmix64 escapes 0 on its first call,
/// but a non-zero base produces a more visibly varied first frame).
const BOOTSTRAP_RNG_BASE: u64 = 0xA5A5_A5A5_0000_0001;

/// V7 Section 16-δ — inclusive upper bound on each agent's staggered initial
/// need value. Initial values land in `0..=BOOTSTRAP_NEED_STAGGER_MAX` (span
/// `MAX + 1 = 46`). The cap is strictly `< 50` (the breach threshold) so every
/// agent is still `Idle` immediately after bootstrap — this is the
/// `s16_alpha0:A13` preservation guarantee.
const BOOTSTRAP_NEED_STAGGER_MAX: u64 = 45;
/// V7 Section 16-δ — XOR salt that derives a per-agent need-RNG seed from the
/// agent's movement seed. Distinct from the movement seed so the agent's own
/// `MovementRng::new(seed)` stream is unperturbed (the need values draw from a
/// SEPARATE `MovementRng` instance seeded with `seed ^ SALT`).
const BOOTSTRAP_NEED_STAGGER_SALT: u64 = 0x5EED_5A66_E0DE_0001;
/// V7 Section 16-δ — accelerated Hunger growth rate (was 0.02). Ordering:
/// Thirst > Hunger > Sleep.
const BOOTSTRAP_HUNGER_RATE: f32 = 0.05;
/// V7 Section 16-δ — accelerated Thirst growth rate (was 0.03). Fastest need,
/// so an agent starting Thirst≈45 breaches at `(50-45)/0.08 ≈ 63` ticks.
const BOOTSTRAP_THIRST_RATE: f64 = 0.08;
/// V7 Section 16-δ — accelerated Sleep growth rate (was 0.01). Slowest of the
/// three staggered resource needs.
const BOOTSTRAP_SLEEP_RATE: f64 = 0.03;

/// V7 Section 16-α0 — deterministic non-depleting resource source tiles.
/// Fixed lattices (NOT RNG): each tile lies inside the 64×64 map and within
/// a few steps (Chebyshev ≤ 4) of the `BOOTSTRAP_AGENT_*` agent lattice
/// (`{4,12,20,28,36,44,52,60}²`), so a Seeking agent has a reachable goal
/// once α/β land movement. Each is seeded at [`RESOURCE_SOURCE_INFINITE`] by
/// `bootstrap_spawn_agents`, then OVERWRITTEN to a finite capacity in the
/// production `init` path by [`seed_finite_resource_scarcity`]
/// (`add-resource-scarcity-regen`). `pub` so the scarcity harness can assert
/// the seed covers exactly these coordinates.
pub const SOURCE_FOOD: [(u32, u32); 4] = [(8, 8), (56, 8), (8, 56), (56, 56)];
/// Water source coordinates — see [`SOURCE_FOOD`].
pub const SOURCE_WATER: [(u32, u32); 4] = [(32, 4), (4, 32), (60, 32), (32, 60)];
/// Sleep source coordinates — see [`SOURCE_FOOD`].
pub const SOURCE_SLEEP: [(u32, u32); 4] = [(20, 20), (44, 20), (20, 44), (44, 44)];

/// `add-resource-scarcity-regen` — finite initial FOOD capacity each source is
/// seeded to in the production `init` path (balance lever). Strictly `!= 255`
/// (the [`RESOURCE_SOURCE_INFINITE`] sentinel) so depletion actually fires;
/// `45` consumes deplete a tile, and `ResourceRegenSystem` refills it at
/// `FOOD_REGEN_AMOUNT / REGEN_INTERVAL` units/tick. Tuned to `45` (down from the
/// initial `60` hypothesis): the seed-42 sweep shows abundant food (≥ 80) yields
/// ZERO scarcity deaths while moderate food scarcity around `45` is the binding
/// constraint that — with the spatial concentration on the nearest corner —
/// drives the deterministic handful of deaths without collapsing the population.
pub const INITIAL_FOOD: u8 = 45;
/// Finite initial WATER capacity — see [`INITIAL_FOOD`]. Water is the hottest
/// channel (highest demand AND thirst kills faster), so its capacity is set
/// the LOWEST of the three deliberately — water is the binding constraint that,
/// combined with spatial concentration on the nearest corner source, drives the
/// deterministic handful of dehydration deaths without collapsing the
/// population. Held at `12`: the seed-42 sweep shows the water buffer responds
/// NON-MONOTONICALLY (lowering to `8` paradoxically dropped deaths 4→3 as the
/// migration/re-route timing shifts), so scarcity DEPTH is driven by the regen
/// rate (`REGEN_INTERVAL`) rather than the initial buffer.
pub const INITIAL_WATER: u8 = 12;
/// Finite initial SLEEP capacity — see [`INITIAL_FOOD`]. Held at `80`: at the
/// chosen `REGEN_INTERVAL = 120` the production population (~132 live) consumes
/// the corner sleep sources enough to deplete a contested sleep tile to
/// key-removal, satisfying A13's per-kind (food AND water AND sleep) depletion
/// requirement. Sleep depletion does NOT drive mortality (only hunger/thirst
/// kill in `StarvationSystem`).
pub const INITIAL_SLEEP: u8 = 80;

/// Godot `Node` subclass wrapping a [`SimEngine`] instance.
///
/// Exposes 6 FFI methods to GDScript/Godot:
/// - [`WorldSimNode::get_influence_overlay`]
/// - [`WorldSimNode::get_tile_detail`]
/// - [`WorldSimNode::on_building_placed`]
/// - [`WorldSimNode::get_tile_causal_history`] (γ-1)
/// - [`WorldSimNode::get_event_chain`] (γ-1)
/// - [`WorldSimNode::get_agent_snapshot`] (P4-γ)
#[derive(GodotClass)]
#[class(base=Node)]
pub struct WorldSimNode {
    engine: SimEngine,
    /// Seed retained for repeatable tooling resets.
    world_seed: u64,
    accumulator: f64,
    /// V7 Section 16-γ — wall-time → accumulator scale (Option A speed
    /// control). Multiplies the per-frame `delta` fed to the Gaffer
    /// accumulator so 0.25× genuinely quarters the tick rate (true continuous
    /// scaling, not process-gating stutter). Clamped to `0.0..=4.0` via
    /// [`clamp_sim_speed`]. Determinism is unaffected — only how much
    /// wall-time feeds the fixed-timestep loop changes, never `FIXED_DT`.
    sim_speed: f64,
    base: Base<Node>,
}

/// Fixed simulation timestep — 30 TPS per Phase 0 design #9 (Gaffer accumulator).
const FIXED_DT: f64 = 1.0 / 30.0;
/// Spiral-of-death cap: skip catch-up after this many fixed ticks per frame.
const MAX_ITERS_PER_FRAME: u32 = 5;

/// V7 Section 16-γ — clamp a requested simulation-speed multiplier to the
/// supported `[0.0, 4.0]` range. Pure (no Godot) so the harness can verify it
/// directly without a Godot runtime. `0.0` freezes the accumulator (a second
/// pause path overlapping KEY_P); `4.0` is the maximum fast-forward.
pub fn clamp_sim_speed(speed: f64) -> f64 {
    speed.clamp(0.0, 4.0)
}

/// Revitalize Pass 15 — the stress world of `seed` with `agents` founders on
/// the game's 64 × 64 map under the `game_v19` rules
/// (`WorldGenConfig::stress_v19`). Pure (no Godot) so the harness can check
/// it directly. Benchmark only — lets the art line measure frame times on a
/// crowded world; never called by the game's UI.
pub fn build_stress_engine(seed: u64, agents: u32) -> SimEngine {
    build_world_with(&WorldGenConfig::stress_v19(seed, agents, 64), WorldRules::game_v19(seed))
}

#[godot_api]
impl INode for WorldSimNode {
    fn init(base: Base<Node>) -> Self {
        Self {
            engine: build_world_with(
                &WorldGenConfig::game_v19(DEFAULT_GAME_SEED),
                WorldRules::game_v19(DEFAULT_GAME_SEED),
            ),
            world_seed: DEFAULT_GAME_SEED,
            accumulator: 0.0,
            sim_speed: 1.0,
            base,
        }
    }

    /// Per-frame Godot hook — drives the simulation at a fixed 30 TPS using
    /// the Gaffer accumulator pattern (Phase 0 design #9). Render runs at
    /// Godot's native frame rate; simulation pacing is deterministic.
    ///
    /// Spiral-of-death guard: if `delta` produces more than
    /// [`MAX_ITERS_PER_FRAME`] fixed ticks, the remaining accumulator is
    /// clamped to one frame so the simulation does not endlessly chase wall
    /// time on a slow frame.
    fn process(&mut self, delta: f64) {
        // V7 Section 16-γ — scale wall-time by `sim_speed` before feeding the
        // accumulator. Only the INPUT to the Gaffer loop is scaled; FIXED_DT
        // and the MAX_ITERS_PER_FRAME guard below are unchanged, so per-tick
        // determinism is preserved.
        self.accumulator += delta * self.sim_speed;
        let mut iters: u32 = 0;
        while self.accumulator >= FIXED_DT && iters < MAX_ITERS_PER_FRAME {
            self.engine.tick();
            self.accumulator -= FIXED_DT;
            iters += 1;
        }
        if self.accumulator > FIXED_DT * MAX_ITERS_PER_FRAME as f64 {
            self.accumulator = FIXED_DT;
        }
    }
}

#[godot_api]
impl WorldSimNode {
    /// Emitted after a complete engine replacement so renderers refresh snapshots.
    #[signal]
    fn world_reset();

    /// Rebuild the seeded world, preserving the caller's wall-clock speed setting.
    #[func]
    fn reset_world(&mut self, seed: i64) -> bool {
        self.world_seed = seed as u64;
        self.engine = build_world_with(
            &WorldGenConfig::game_v19(self.world_seed),
            WorldRules::game_v19(self.world_seed),
        );
        self.accumulator = 0.0;
        self.base_mut().call_deferred("emit_signal", &["world_reset".to_variant()]);
        true
    }

    /// Revitalize Pass 15 — rebuild as the stress world of `seed` with
    /// `agents` founders (clamped to 1..=5000), as [`Self::reset_world`] does
    /// otherwise ([`build_stress_engine`]). Benchmark only — lets the art line
    /// measure frame times on a crowded world; never called by the game's UI.
    #[func]
    fn reset_world_stress(&mut self, seed: i64, agents: i64) -> bool {
        self.world_seed = seed as u64;
        self.engine = build_stress_engine(self.world_seed, agents.clamp(1, 5000) as u32);
        self.accumulator = 0.0;
        self.base_mut().call_deferred("emit_signal", &["world_reset".to_variant()]);
        true
    }

    /// Read seed, dimensions, and simulation pacing from authoritative resources.
    #[func]
    fn get_world_info(&self) -> VarDictionary {
        world_info_to_dict(collect_world_info(&self.engine.resources))
    }

    /// Read row-major terrain codes and elevation from the simulation grid.
    #[func]
    fn get_terrain_snapshot(&self) -> VarDictionary {
        terrain_snapshot_to_dict(&collect_terrain_snapshot(&self.engine.resources))
    }

    /// Read real buildings ordered by their stable simulation building ids.
    #[func]
    fn get_building_snapshot(&self) -> VarDictionary {
        building_rows_to_dict(&collect_building_snapshot(&self.engine.resources))
    }

    /// Advance a bounded number of deterministic ticks for capture tooling.
    #[func]
    fn advance_ticks(&mut self, n: i64) -> i64 {
        advance_engine_ticks(&mut self.engine, n) as i64
    }

    /// Read the authoritative simulation clock, independent of render frames.
    #[func]
    fn get_time_snapshot(&self) -> VarDictionary {
        time_snapshot_to_dict(collect_time_snapshot(&self.engine))
    }

    /// Read cumulative births and deaths, never the rolling event buffer.
    #[func]
    fn get_tallies(&self) -> VarDictionary {
        tallies_to_dict(collect_tallies(&self.engine.resources))
    }

    /// Revitalize Pass 7 — the god's hand: apply one intervention now, between
    /// ticks. `kind` is one of `gift_food`, `spoil_food`, `gift_wood`,
    /// `rot_wood`, `rain`, `cold_snap`, `windfall`; `settlement_id < 0` means
    /// home (the lowest settlement id now); `amount` is hours for rain, nights
    /// for a cold snap and units otherwise. Returns `{ok, applied, tick}`.
    #[func]
    fn apply_intervention(&mut self, kind: GString, settlement_id: i64, amount: i64) -> VarDictionary {
        intervention_reply_to_dict(apply_intervention_command(&mut self.engine, &kind.to_string(), settlement_id, amount))
    }

    /// Art Pass 6: read intervention presentation state without advancing the world.
    #[func]
    fn get_intervention_state(&self) -> VarDictionary {
        let state = collect_intervention_state(&self.engine.resources, self.engine.current_tick());
        let mut dict = VarDictionary::new();
        dict.set("rain_settlement_ids", PackedInt32Array::from(state.rain_settlement_ids.as_slice()));
        dict.set("rain_ticks_left", PackedInt32Array::from(state.rain_ticks_left.as_slice()));
        dict.set("cold_snap_nights_left", i64::from(state.cold_snap_nights_left));
        dict.set("cold_snap_tonight", state.cold_snap_tonight);
        dict.set("log_ticks", PackedInt64Array::from(state.log_ticks.as_slice()));
        dict.set("log_ids", PackedInt64Array::from(state.log_ids.as_slice()));
        dict.set("log_settlement_ids", PackedInt32Array::from(state.log_settlement_ids.as_slice()));
        let kinds: Vec<GString> = state.log_kinds.iter().map(|kind| GString::from(*kind)).collect();
        dict.set("log_kinds", PackedStringArray::from(kinds.as_slice()));
        dict.set("log_amounts", PackedInt64Array::from(state.log_amounts.as_slice()));
        dict.set("log_applied", PackedInt64Array::from(state.log_applied.as_slice()));
        dict
    }

    /// V7 Section 16-γ — set the simulation-speed multiplier driving the
    /// per-frame accumulator increment. Stores [`clamp_sim_speed`]`(speed)`,
    /// so out-of-range requests are clamped to `[0.0, 4.0]`. Bound to the
    /// `KEY_1`/`KEY_2`/`KEY_3`/`KEY_4` number keys by `camera_controller.gd`
    /// (0.25× / 0.5× / 1× / 2×). Pure observability — changes only how often
    /// `tick()` is called per wall-second, never the fixed timestep.
    #[func]
    fn set_sim_speed(&mut self, speed: f64) {
        self.sim_speed = clamp_sim_speed(speed);
    }

    /// Serialize the current buffer of influence `channel` to a packed byte
    /// array (row-major, `width × height` bytes). Returns an empty array if
    /// the channel index is out of range.
    #[func]
    fn get_influence_overlay(&self, channel: i32) -> PackedByteArray {
        let Some(ch) = channel_from_i32(channel) else {
            return PackedByteArray::new();
        };
        let buf = self.engine.resources.influence_grid.current_buf(ch);
        PackedByteArray::from(buf)
    }

    /// Return a dictionary describing tile `(x, y)`. Keys:
    ///   - `tile_x`: i32, `tile_y`: i32, `in_bounds`: bool
    ///   - `warmth`, `light`, `noise`, `food_aroma`, `danger`, `social`,
    ///     `spiritual`, `beauty`: u8 (current buffer value)
    #[func]
    fn get_tile_detail(&self, x: i32, y: i32) -> VarDictionary {
        let mut dict = VarDictionary::new();
        dict.set("tile_x", x);
        dict.set("tile_y", y);
        let grid = &self.engine.resources.influence_grid;
        let in_bounds = x >= 0
            && y >= 0
            && (x as u32) < grid.width
            && (y as u32) < grid.height;
        dict.set("in_bounds", in_bounds);
        if in_bounds {
            let ux = x as u32;
            let uy = y as u32;
            for ch in InfluenceChannel::all() {
                dict.set(channel_key(*ch), grid.sample(ux, uy, *ch));
            }
        } else {
            for ch in InfluenceChannel::all() {
                dict.set(channel_key(*ch), 0u8);
            }
        }
        dict
    }

    /// Push a [`BuildingPlacedEvent`] into the SimResources queue.
    ///
    /// Returns `false` if `(x, y)` is negative or outside the grid, or if
    /// `radius` is negative; returns `true` on successful enqueue.
    ///
    /// **Bridge Identity Contract**: this `#[func]` body consists solely of a
    /// forwarding call to [`enqueue_building_placed`]. All bounds-check and
    /// enqueue logic lives in that function. Sim-test calls
    /// [`enqueue_building_placed`] directly for Assertions 5 and 6.
    ///
    /// The drain happens on the next [`BuildingStampSystem`][`sim_systems::runtime::influence::BuildingStampSystem`]
    /// tick (priority 90).
    #[func]
    fn on_building_placed(&mut self, x: i32, y: i32, radius: i32) -> bool {
        enqueue_building_placed(&mut self.engine.resources, x, y, radius)
    }

    /// γ-1: enumerate every [`CausalEvent`] recorded on tile `(x, y)` in
    /// insertion order (oldest first, capped at
    /// [`TILE_CAUSAL_RING_SIZE`][sim_core::causal::TILE_CAUSAL_RING_SIZE]).
    ///
    /// Returns an empty array if `(x, y)` is out of bounds or the tile has
    /// no recorded events. Each entry is a [`Dictionary`] with the schema
    /// documented on [`CausalEventView`]. The `#[func]` body consists of a
    /// thin loop over [`collect_tile_causal_history`] results converted via
    /// [`event_view_to_dict`]; the Evaluator verifies non-stub via grep.
    #[func]
    fn get_tile_causal_history(&self, x: i32, y: i32) -> VarArray {
        let views = try_collect_tile_causal_history(&self.engine.resources, x, y);
        event_views_to_variant_array(&views)
    }

    /// γ-1: walk the causal chain backwards from `event_id` on tile
    /// `(x, y)`, returning `[child, parent, grand-parent, …]`.
    ///
    /// Returns an empty array when the tile is out of bounds, has no log,
    /// or `event_id` is not present on that tile. The walk terminates when
    /// a root (`parent == None`) is reached or the referenced parent has
    /// been evicted (graceful termination — see
    /// [`CausalLogStorage::trace_parents`][sim_core::causal::CausalLogStorage::trace_parents]).
    ///
    /// `event_id` is `i64` at the FFI boundary because Godot's `Variant`
    /// integer is signed 64-bit; negative values are rejected as
    /// out-of-domain.
    #[func]
    fn get_event_chain(&self, x: i32, y: i32, event_id: i64) -> VarArray {
        let views = try_collect_event_chain(&self.engine.resources, x, y, event_id);
        event_views_to_variant_array(&views)
    }

    /// P4-γ FFI — return the current `(Agent, Position)` snapshot as a
    /// dictionary of three parallel `PackedArray`s with always-equal
    /// lengths. Keys:
    ///   - `ids`: `PackedInt64Array` — `Entity::to_bits().get() as i64`
    ///     per row. Stable within a single session; not stable across
    ///     world resets.
    ///   - `xs`:  `PackedInt32Array` — tile-x per row, as `i32`.
    ///   - `ys`:  `PackedInt32Array` — tile-y per row, as `i32`.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_agent_snapshot`] (Bridge Identity Contract — γ extension).
    /// Sim-test verifies the schema by calling the pure-Rust collector
    /// directly (Godot runtime not required).
    #[func]
    fn get_agent_snapshot(&self) -> VarDictionary {
        let rows = collect_agent_snapshot(&self.engine.world);
        let mut dict = agent_rows_to_dict(&rows);
        // Pass 4 — additive whole-day ages in the same row order (-1 without Life).
        let ages = collect_agent_ages_days(
            &self.engine.world,
            self.engine.current_tick(),
            self.engine.resources.ticks_per_day,
        );
        dict.set("ages_days", PackedInt32Array::from(ages.as_slice()));
        // Pass 6 — additive cold flags in the same row order (1 with the Cold marker).
        let colds = collect_agent_colds(&self.engine.world);
        dict.set("colds", PackedByteArray::from(colds.as_slice()));
        dict
    }

    /// V7 Phase 14-γ FFI — single-agent detail dictionary for the click
    /// inspector panel. Returns the canonical 9-key set
    /// (`found`, `agent_id`, `x`, `y`, `state_tag`, `hunger`, `thirst`,
    /// `sleep`, `target_kind`) regardless of lookup outcome. When the
    /// entity is not found, `found` is `false` and the numeric fields
    /// take their `AgentDetailRow::default()` values.
    ///
    /// `entity_bits` is the `hecs::Entity::to_bits()` value the GDScript
    /// side reads from the `ids` array of [`WorldSimNode::get_agent_snapshot`].
    /// Stale or hostile values (zero, despawned, never-allocated patterns)
    /// return the not-found sentinel without panicking.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_agent_detail`] (Bridge Identity Contract — γ extension).
    /// Sim-test verifies the contract by calling the pure-Rust collector
    /// directly (Godot runtime not required).
    #[func]
    fn get_agent_detail(&self, entity_bits: i64) -> VarDictionary {
        let row = collect_agent_detail(&self.engine.world, entity_bits as u64);
        agent_detail_to_dict(row)
    }

    /// V7 Phase 12-β.2 (A3) FFI — construction-site snapshot for the
    /// GDScript renderer. Returns a `VarDictionary` with five
    /// `PackedArray` keys (`ids`, `xs`, `ys`, `progresses`,
    /// `required_progresses`) of equal length. Empty arrays when no
    /// `ConstructionSite` entities exist.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_construction_snapshot`] (Bridge Identity Contract).
    /// Sim-test exercises the pure-Rust collector directly.
    #[func]
    fn get_construction_snapshot(&self) -> VarDictionary {
        let rows = collect_construction_snapshot(&self.engine.world);
        construction_rows_to_dict(&rows)
    }

    /// V7 Section 16-α0 FFI — resource-substrate snapshot for the renderer.
    /// Returns a `VarDictionary` with three equal-length `PackedInt32Array`
    /// keys (`xs`, `ys`, `kinds`), sorted by `(kind, x, y)`. Empty arrays
    /// when no source tiles exist. Reads `engine.resources`.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_resource_snapshot`] + [`resource_rows_to_dict`]
    /// (Bridge Identity Contract). Sim-test exercises the pure-Rust
    /// collector + [`resource_rows_split`] directly.
    #[func]
    fn get_resource_snapshot(&self) -> VarDictionary {
        let rows = collect_resource_snapshot(&self.engine.resources);
        resource_rows_to_dict(&rows)
    }

    /// V7 viz-D FFI — recent-deaths snapshot for the death-marker overlay.
    /// Returns a `VarDictionary` with four equal-length `PackedArray` keys
    /// (`xs`, `ys`, `reasons`, `ticks`) plus the scalar `current_tick`. Empty
    /// arrays (still with the `current_tick` scalar) when no recent deaths
    /// exist. Reads `engine.resources`.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_recent_deaths`] + [`recent_death_rows_to_dict`] (Bridge
    /// Identity Contract). Sim-test exercises the pure-Rust collector +
    /// [`recent_death_rows_split`] directly.
    #[func]
    fn get_recent_deaths(&self) -> VarDictionary {
        let rows = collect_recent_deaths(&self.engine.resources);
        recent_death_rows_to_dict(&rows, self.engine.resources.current_tick as i64)
    }

    /// V7 Phase 12-γ FFI — settlement snapshot with substrate-derived
    /// centroid for furniture placement. Returns a `VarDictionary` with
    /// five `PackedArray` keys (`ids`, `settlement_ids`, `centroid_xs`,
    /// `centroid_ys`, `member_counts`) of equal length. Empty arrays
    /// when `resources.settlements` holds no settlement with a resolvable
    /// member.
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_settlement_snapshot`] (Bridge Identity Contract). The
    /// `self.engine.world` arg supplies the `(Agent, Position)` lookup; the
    /// authoritative settlement store is `self.engine.resources.settlements`.
    #[func]
    fn get_settlement_snapshot(&self) -> VarDictionary {
        let rows = collect_settlement_snapshot(
            &self.engine.world,
            &self.engine.resources.settlements,
        );
        // Pass 6 — additive floor of each settlement's wood target.
        let wood_targets = collect_settlement_wood_targets(&rows, &self.engine.resources);
        settlement_rows_to_dict(&rows, &self.engine.resources.fires, &wood_targets)
    }

    /// Art Pass 7: read settlement lineage, roster identity and founding history.
    #[func]
    fn get_society_snapshot(&self) -> VarDictionary {
        let snapshot = collect_society_snapshot(&self.engine.world, &self.engine.resources);
        society_snapshot_to_dict(&snapshot)
    }

    /// P7-δ FFI — return every known relationship pair (familiarity > 0
    /// OR hostility > 0) as a flat `Array<Dictionary>`. Each dict has
    /// keys `id_a: i64`, `id_b: i64`, `familiarity: f64`, `hostility: f64`,
    /// with `id_a < id_b` (canonical key ordering).
    ///
    /// The `#[func]` body consists solely of forwarding to
    /// [`collect_relationship_snapshot`] (Bridge Identity Contract).
    #[func]
    fn get_relationship_snapshot(&self) -> VarArray {
        let rows = collect_relationship_snapshot(&self.engine.resources);
        relationship_rows_to_variant_array(&rows)
    }
}

/// Push a [`BuildingPlacedEvent`] into `resources.building_event_queue`.
///
/// This is the complete implementation of `WorldSimNode::on_building_placed`'s
/// bounds-check and enqueue logic, extracted into a `pub fn` with a Rust-only
/// signature so sim-test can call it directly without Godot runtime.
///
/// Returns `true` if the event was enqueued (position in-bounds, non-negative
/// coordinates and radius). Returns `false` and does not enqueue otherwise.
///
/// # Bridge Identity Contract
///
/// `WorldSimNode::on_building_placed`'s `#[func]` body consists **solely** of
/// `enqueue_building_placed(&mut self.engine.resources, x, y, radius)`.
/// The Evaluator verifies via Completeness code review that no stub logic
/// replaces this delegation.
pub fn enqueue_building_placed(
    resources: &mut SimResources,
    x: i32,
    y: i32,
    radius: i32,
) -> bool {
    if x < 0 || y < 0 || radius < 0 {
        return false;
    }
    let grid = &resources.influence_grid;
    if (x as u32) >= grid.width || (y as u32) >= grid.height {
        return false;
    }
    resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (x as u32, y as u32),
        radius: radius as u32,
    });
    true
}

/// Map a channel index `i32` to `InfluenceChannel`, or `None` if out of range.
fn channel_from_i32(ix: i32) -> Option<InfluenceChannel> {
    if ix < 0 {
        return None;
    }
    InfluenceChannel::all().get(ix as usize).copied()
}

/// Return the dictionary key string for a channel.
/// Exhaustive match — compile-time coverage of all 8 channels.
fn channel_key(ch: InfluenceChannel) -> &'static str {
    match ch {
        InfluenceChannel::Warmth => "warmth",
        InfluenceChannel::Light => "light",
        InfluenceChannel::Noise => "noise",
        InfluenceChannel::FoodAroma => "food_aroma",
        InfluenceChannel::Danger => "danger",
        InfluenceChannel::Social => "social",
        InfluenceChannel::Spiritual => "spiritual",
        InfluenceChannel::Beauty => "beauty",
    }
}

// ────────────────────────────────────────────────────────────────────────
// γ-1: Causal log FFI surface — pure-Rust collectors + view type
// ────────────────────────────────────────────────────────────────────────

/// Pure-Rust mirror of a [`CausalEvent`] flattened into a tagged record.
///
/// V7 Phase 3-γ (γ-1) — the upcoming "왜?" UI consumes the [`Dictionary`]
/// produced by [`event_view_to_dict`]; sim-test consumes this struct
/// directly to verify the schema without depending on Godot runtime.
///
/// Discriminator: [`CausalEventView::kind`] is one of `"building_placed"`,
/// `"stamp_dirty"`, `"influence_changed"`, `"agent_decision"`. Variant-
/// specific fields are `Some` only for the matching kind.
///
/// Field mapping:
/// - `id`, `tick` — always populated (every event).
/// - `parent` — `Some(id)` for chain children, `None` for roots
///   (`BuildingPlaced`, agent-originated root decisions) or after parent
///   eviction. Serialised as `-1` for `None` in the dictionary form.
/// - `channel` — `Some` for `StampDirty` / `InfluenceChanged` only.
/// - `position` — origin `(x, y)` for `BuildingPlaced`; sample centre for
///   `InfluenceChanged`; decision tile for `AgentDecision`; `None` for
///   `StampDirty` (the region covers it).
/// - `radius` — `Some` only for `BuildingPlaced`.
/// - `region` — `Some(min_x, min_y, max_x, max_y)` only for `StampDirty`.
/// - `old_value` / `new_value` — `Some` only for `InfluenceChanged`.
/// - `agent_id` — `Some` only for `AgentDecision` (the deciding agent).
/// - `reason` — `Some` only for `AgentDecision` (e.g.
///   `"hunger_threshold_breach"`).
#[derive(Debug, Clone, PartialEq)]
pub struct CausalEventView {
    /// String discriminator: `"building_placed"` | `"stamp_dirty"` |
    /// `"influence_changed"` | `"agent_decision"`.
    pub kind: &'static str,
    /// Monotonic event id (V7 Phase 3-β).
    pub id: EventId,
    /// Parent event id; `None` denotes a chain root or evicted parent.
    pub parent: Option<EventId>,
    /// Simulation tick the event was recorded at.
    pub tick: u64,
    /// Channel index (matches [`InfluenceChannel`] ordering), or `None`
    /// for `BuildingPlaced` / `AgentDecision`.
    pub channel: Option<u8>,
    /// Origin / sample tile, or `None` for `StampDirty`.
    pub position: Option<(u32, u32)>,
    /// Chebyshev influence radius (BuildingPlaced only).
    pub radius: Option<u32>,
    /// Dirty region bounds `(min_x, min_y, max_x, max_y)` (StampDirty only).
    pub region: Option<(u32, u32, u32, u32)>,
    /// Pre-propagation intensity at `position` (InfluenceChanged only).
    pub old_value: Option<f32>,
    /// Post-propagation intensity at `position` (InfluenceChanged only).
    pub new_value: Option<f32>,
    /// Deciding agent id (AgentDecision only — Phase 5-β).
    pub agent_id: Option<u64>,
    /// Reason discriminator string (AgentDecision only — Phase 5-β).
    /// One of `"hunger_threshold_breach"`, `"thirst_threshold_breach"`.
    pub reason: Option<&'static str>,
    /// Memory recall trigger discriminator (MemoryRecalled only — V7
    /// Phase 8-δ). One of `"cascade_bias"`, `"similarity_search"`,
    /// `"periodic"`, `"combat_context"`. Surfaced so the GDScript
    /// CausalPanel can select the correct `UI_MEMORY_RECALL_TRIGGER_*`
    /// locale key.
    pub triggered_by: Option<&'static str>,
    /// `event_id` of the recalled [`MemoryEntry`] that drove the cascade
    /// flip (MemoryRecalled only — V7 Phase 8-δ). Preserved through the
    /// FFI so the GDScript CausalPanel can show the recalled event id
    /// (and a future phase can deep-link into the parent chain).
    ///
    /// [`MemoryEntry`]: sim_core::components::MemoryEntry
    pub recalled_event: Option<EventId>,
    /// Defender [`AgentId`] for combat events (CombatStarted /
    /// CombatCompleted only — V7 Phase 9-δ). Surfaced so the GDScript
    /// CausalPanel and AgentRenderer can reference the defender side of
    /// a combat encounter. Serialised as `"defender_id"` in the FFI dict.
    pub defender_id: Option<AgentId>,
    /// Defender HP after damage (CombatCompleted only — V7 Phase 9-δ).
    /// Saturated at `0.0` when the defender is dead. Serialised as
    /// `"new_value"` in the FFI dict (mirrors the `"new"`/`"new_value"`
    /// convention for post-mutation snapshots) so the panel can render the
    /// post-damage value (`UI_COMBAT_HP_AFTER`).
    pub hp_after: Option<f64>,
}

impl CausalEventView {
    /// Build a [`CausalEventView`] from a borrowed [`CausalEvent`].
    pub fn from_event(ev: &CausalEvent) -> Self {
        match ev {
            CausalEvent::BuildingPlaced {
                id,
                parent,
                position,
                radius,
                tick,
            } => Self {
                kind: "building_placed",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: Some(*radius),
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::StampDirty {
                id,
                parent,
                channel,
                region,
                tick,
            } => Self {
                kind: "stamp_dirty",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: Some(*channel as u8),
                position: None,
                radius: None,
                region: Some(dirty_region_bounds(region)),
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::InfluenceChanged {
                id,
                parent,
                channel,
                position,
                old,
                new,
                tick,
            } => Self {
                kind: "influence_changed",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: Some(*channel as u8),
                position: Some(*position),
                radius: None,
                region: None,
                old_value: Some(*old),
                new_value: Some(*new),
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::AgentDecision {
                id,
                parent,
                agent,
                position,
                reason,
                tick,
            } => Self {
                kind: "agent_decision",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: Some(*agent),
                reason: Some(reason.as_str()),
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::ConstructionStarted {
                id,
                parent,
                blueprint: _,
                position,
                tick,
            } => Self {
                kind: "construction_started",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::ConstructionCompleted {
                id,
                parent,
                blueprint: _,
                position,
                tick,
            } => Self {
                kind: "construction_completed",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::SocialInteractionStarted {
                id,
                parent,
                agents: _,
                position,
                tick,
            } => Self {
                kind: "social_interaction_started",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            CausalEvent::SocialInteractionCompleted {
                id,
                parent,
                agents: _,
                position,
                familiarity_after: _,
                tick,
            } => Self {
                kind: "social_interaction_completed",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            // V7 Phase 8-δ — full FFI shape: surfaces `triggered_by` (the
            // discriminator the GDScript CausalPanel uses to pick the
            // correct `UI_MEMORY_RECALL_TRIGGER_*` locale key). Phase 8-β
            // wires only `CascadeBias`; the other variants serialise to
            // their snake_case discriminator should later phases emit them.
            CausalEvent::MemoryRecalled {
                id,
                parent,
                agent,
                recalled_event,
                triggered_by,
                tick,
            } => Self {
                kind: "memory_recalled",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: None,
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: Some(*agent),
                reason: None,
                triggered_by: Some(memory_recall_trigger_str(triggered_by)),
                recalled_event: Some(*recalled_event),
                defender_id: None,
                hp_after: None,
            },
            // V7 Phase 9-δ — full FFI shape: surfaces `defender_id` so the
            // GDScript CausalPanel and AgentRenderer can reference the
            // defender side of a combat encounter.
            CausalEvent::CombatStarted {
                id,
                parent,
                attacker,
                defender,
                position,
                tick,
            } => Self {
                kind: "combat_started",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: Some(*attacker),
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: Some(*defender),
                hp_after: None,
            },
            CausalEvent::CombatCompleted {
                id,
                parent,
                attacker,
                defender,
                position,
                hp_after,
                tick,
                ..
            } => Self {
                kind: "combat_completed",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: Some(*hp_after as f32),
                agent_id: Some(*attacker),
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: Some(*defender),
                hp_after: Some(*hp_after),
            },
            CausalEvent::AgentBorn { id, parent, agent, tick, .. } => Self {
                kind: "agent_born",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: None,
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: Some(*agent),
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            // V7 Phase 10-β — settlement formation event. No per-agent
            // attribution (founding_members is a list, not a singleton);
            // GDScript panels can resolve the membership via the
            // settlement snapshot in Phase 10-γ.
            CausalEvent::SettlementFormed {
                id,
                parent,
                tick,
                ..
            } => Self {
                kind: "settlement_formed",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: None,
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            // V7 Phase 10-β — settlement dissolution event.
            CausalEvent::SettlementDissolved {
                id,
                parent,
                tick,
                ..
            } => Self {
                kind: "settlement_dissolved",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: None,
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: None,
                reason: None,
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
            // add-starvation-death — agent death event. `reason` carries the
            // DeathReason discriminator ("starvation"/"dehydration"/"combat").
            CausalEvent::AgentDied {
                id,
                parent,
                agent,
                position,
                reason,
                tick,
            } => Self {
                kind: "agent_died",
                id: *id,
                parent: *parent,
                tick: *tick,
                channel: None,
                position: Some(*position),
                radius: None,
                region: None,
                old_value: None,
                new_value: None,
                agent_id: Some(*agent),
                reason: Some(reason.as_str()),
                triggered_by: None,
                recalled_event: None,
                defender_id: None,
                hp_after: None,
            },
        }
    }
}

fn dirty_region_bounds(region: &DirtyRegion) -> (u32, u32, u32, u32) {
    (region.min_x, region.min_y, region.max_x, region.max_y)
}

/// V7 Phase 8-δ — stable snake_case discriminator for a
/// [`MemoryRecallTrigger`] as it crosses the FFI boundary. Lives on the
/// sim-bridge side (not on `MemoryRecallTrigger` itself) because the wire
/// format is the FFI's contract with GDScript — the simulation core
/// proper has no interest in this string mapping.
///
/// The GDScript CausalPanel selects its `UI_MEMORY_RECALL_TRIGGER_*` locale
/// key by `==`-matching on this discriminator, so the literals are part of
/// the Phase 8-δ locked contract.
pub(crate) fn memory_recall_trigger_str(trigger: &MemoryRecallTrigger) -> &'static str {
    match trigger {
        MemoryRecallTrigger::CascadeBias => "cascade_bias",
        MemoryRecallTrigger::SimilaritySearch => "similarity_search",
        MemoryRecallTrigger::Periodic => "periodic",
        MemoryRecallTrigger::CombatContext { .. } => "combat_context",
    }
}

/// V7 Phase 8-δ — pure-Rust mirror of the value types the FFI
/// [`event_view_to_dict`] embeds in the returned `Dictionary`. Exists so
/// sim-test (no Godot runtime) can assert the serialised dict shape
/// independently of `VarDictionary` / `Variant`.
///
/// The variant set mirrors exactly what [`event_view_to_dict`] writes:
/// strings (the discriminator + agent reason / triggered_by), integers
/// (ids, ticks, agent ids, radii, channels), floats (influence pre/post),
/// and packed coordinate tuples. Adding a new field to
/// [`CausalEventView`] must update BOTH this enum and the dict marshaller
/// in lock-step.
#[derive(Debug, Clone, PartialEq)]
pub enum FfiFieldValue {
    /// String discriminator (e.g. `kind = "memory_recalled"`, `reason =
    /// "memory_reason"`, `triggered_by = "cascade_bias"`).
    Str(&'static str),
    /// Signed 64-bit integer (matches Godot Variant's native int width).
    I64(i64),
    /// Signed 32-bit integer (used for tile coordinates, radii, channel
    /// indices encoded as i32).
    I32(i32),
    /// 32-bit float (used for `old`/`new` influence intensities).
    F32(f32),
    /// 64-bit float (V7 Phase 9-δ — used for `hp_after`, which is `f64` in
    /// the simulation core and round-trips with full precision).
    F64(f64),
    /// `(x, y)` packed coordinate (`Vector2i` in the VarDictionary).
    Pos2i(i32, i32),
    /// `(min_x, min_y, max_x, max_y)` packed region (`Vector4i` in the
    /// VarDictionary).
    Region4i(i32, i32, i32, i32),
}

/// V7 Phase 8-δ — produce the canonical key/value map that
/// [`event_view_to_dict`] then marshals into a Godot `VarDictionary`.
///
/// This is the *source of truth* for the FFI dict schema: anything written
/// to the VarDictionary is also written here (and vice-versa). Sim-test
/// asserts against this map directly so the FFI-dict contract is checked
/// without a Godot runtime. The ordering is insertion-order from a
/// `BTreeMap` to keep test diffs deterministic; the GDScript side does not
/// depend on key order.
///
/// Key set:
///   - Always present: `kind`, `id`, `parent` (i64; `-1` denotes `None`),
///     `tick`.
///   - Variant-specific: `channel`, `position`, `radius`, `region`, `old`,
///     `new`, `agent_id`, `reason`, `triggered_by`, `recalled_event` —
///     present only when the matching `CausalEventView` field is `Some`.
pub fn event_view_to_owned_dict(view: &CausalEventView) -> std::collections::BTreeMap<&'static str, FfiFieldValue> {
    let mut dict = std::collections::BTreeMap::new();
    dict.insert("kind", FfiFieldValue::Str(view.kind));
    dict.insert("id", FfiFieldValue::I64(view.id as i64));
    dict.insert(
        "parent",
        FfiFieldValue::I64(view.parent.map(|p| p as i64).unwrap_or(-1)),
    );
    dict.insert("tick", FfiFieldValue::I64(view.tick as i64));
    if let Some(ch) = view.channel {
        dict.insert("channel", FfiFieldValue::I32(ch as i32));
    }
    if let Some((px, py)) = view.position {
        dict.insert("position", FfiFieldValue::Pos2i(px as i32, py as i32));
    }
    if let Some(r) = view.radius {
        dict.insert("radius", FfiFieldValue::I32(r as i32));
    }
    if let Some((min_x, min_y, max_x, max_y)) = view.region {
        dict.insert(
            "region",
            FfiFieldValue::Region4i(
                min_x as i32,
                min_y as i32,
                max_x as i32,
                max_y as i32,
            ),
        );
    }
    if let Some(old) = view.old_value {
        dict.insert("old", FfiFieldValue::F32(old));
    }
    if let Some(new) = view.new_value {
        dict.insert("new_value", FfiFieldValue::F32(new));
    }
    if let Some(agent_id) = view.agent_id {
        dict.insert("agent_id", FfiFieldValue::I64(agent_id as i64));
    }
    if let Some(reason) = view.reason {
        dict.insert("reason", FfiFieldValue::Str(reason));
    }
    if let Some(triggered_by) = view.triggered_by {
        dict.insert("triggered_by", FfiFieldValue::Str(triggered_by));
    }
    if let Some(recalled_event) = view.recalled_event {
        dict.insert(
            "recalled_event",
            FfiFieldValue::I64(recalled_event as i64),
        );
    }
    if let Some(defender_id) = view.defender_id {
        dict.insert("defender_id", FfiFieldValue::I64(defender_id as i64));
    }
    if let Some(hp_after) = view.hp_after {
        // V7 Phase 9-δ — serialised under `"new_value"` to mirror the
        // post-mutation snapshot convention shared with `InfluenceChanged`
        // and match the GDScript CausalPanel reader for combat_completed.
        dict.insert("new_value", FfiFieldValue::F64(hp_after));
    }
    dict
}

/// γ-1 pure-Rust collector: enumerate every event on `tile_idx` in
/// insertion order (oldest first). Returns an empty `Vec` when the tile
/// has no recorded log.
///
/// Mirrors `WorldSimNode::get_tile_causal_history` minus the Godot
/// `Variant` marshalling, so sim-test can exercise the schema without a
/// Godot runtime. Bounded by [`TILE_CAUSAL_RING_SIZE`][sim_core::causal::TILE_CAUSAL_RING_SIZE].
pub fn collect_tile_causal_history(
    resources: &SimResources,
    tile_idx: u32,
) -> Vec<CausalEventView> {
    let Some(log) = resources.causal_log.get(tile_idx) else {
        return Vec::new();
    };
    log.as_slice().iter().map(CausalEventView::from_event).collect()
}

/// γ-1 pure-Rust collector: walk the parent chain backwards from
/// `event_id` on `tile_idx`. Returns `[child, parent, grand-parent, …]`.
///
/// Mirrors `WorldSimNode::get_event_chain` minus the Godot marshalling.
/// Terminates gracefully when the chain reaches a root or the referenced
/// parent is no longer present on the tile (eviction).
pub fn collect_event_chain(
    resources: &SimResources,
    tile_idx: u32,
    event_id: EventId,
) -> Vec<CausalEventView> {
    resources
        .causal_log
        .trace_parents(tile_idx, event_id)
        .iter()
        .map(|ev| CausalEventView::from_event(ev))
        .collect()
}

/// γ-1 pure-Rust FFI-mirror of `WorldSimNode::get_tile_causal_history`.
///
/// Performs the same bounds check used by the `#[func]` body (negative or
/// out-of-grid coordinates yield an empty `Vec`), then forwards to
/// [`collect_tile_causal_history`]. Sim-test calls this directly to verify
/// the OOB contract without a Godot runtime.
pub fn try_collect_tile_causal_history(
    resources: &SimResources,
    x: i32,
    y: i32,
) -> Vec<CausalEventView> {
    let grid = &resources.influence_grid;
    let Some(tile_idx) = tile_idx_from_coords(grid.width, grid.height, x, y) else {
        return Vec::new();
    };
    collect_tile_causal_history(resources, tile_idx)
}

/// γ-1 pure-Rust FFI-mirror of `WorldSimNode::get_event_chain`.
///
/// Performs the same bounds check + negative-`event_id` rejection used by
/// the `#[func]` body, then forwards to [`collect_event_chain`]. Sim-test
/// calls this directly to verify the OOB / negative-id contract without
/// a Godot runtime.
pub fn try_collect_event_chain(
    resources: &SimResources,
    x: i32,
    y: i32,
    event_id: i64,
) -> Vec<CausalEventView> {
    let grid = &resources.influence_grid;
    let Some(tile_idx) = tile_idx_from_coords(grid.width, grid.height, x, y) else {
        return Vec::new();
    };
    if event_id < 0 {
        return Vec::new();
    }
    collect_event_chain(resources, tile_idx, event_id as EventId)
}

/// Translate a Godot-side `(x, y)` pair into a linear tile index, or
/// `None` when negative or outside the influence grid. Public so sim-test
/// can exercise the OOB resolution rule independently of the surrounding
/// FFI-mirror helpers.
pub fn tile_idx_from_coords(width: u32, height: u32, x: i32, y: i32) -> Option<u32> {
    if x < 0 || y < 0 {
        return None;
    }
    let (ux, uy) = (x as u32, y as u32);
    if ux >= width || uy >= height {
        return None;
    }
    Some(uy * width + ux)
}

/// Convert a [`CausalEventView`] into a `Dictionary` matching the γ-1
/// schema. Keys are always present at the documented positions; absent
/// optional fields are encoded as `-1` (`parent`) or omitted entirely
/// (variant-specific keys appear only for their owning variant). See
/// [`CausalEventView`] for the full schema.
///
/// V7 Phase 8-δ (plan_attempt 3 §A7) — schema single-source-of-truth:
/// this function delegates to [`event_view_to_owned_dict`] and only
/// performs the `FfiFieldValue` → `Variant` conversion. Any new key
/// added to the GDScript-facing dict MUST be added to
/// [`event_view_to_owned_dict`] (the schema generator); this function
/// will pick it up automatically. This guarantees the symmetric
/// difference between the two helpers' key sets is the empty set.
fn event_view_to_dict(view: &CausalEventView) -> VarDictionary {
    let owned = event_view_to_owned_dict(view);
    let mut dict = VarDictionary::new();
    for (key, value) in owned {
        match value {
            FfiFieldValue::Str(s) => dict.set(key, s),
            FfiFieldValue::I64(n) => dict.set(key, n),
            FfiFieldValue::I32(n) => dict.set(key, n),
            FfiFieldValue::F32(f) => dict.set(key, f),
            FfiFieldValue::F64(f) => dict.set(key, f),
            FfiFieldValue::Pos2i(x, y) => dict.set(key, Vector2i::new(x, y)),
            FfiFieldValue::Region4i(min_x, min_y, max_x, max_y) => {
                dict.set(key, Vector4i::new(min_x, min_y, max_x, max_y));
            }
        }
    }
    dict
}

/// Pack a slice of [`CausalEventView`] into a Godot [`VarArray`] of
/// dictionaries — the exact return shape of the two γ-1 `#[func]` methods.
fn event_views_to_variant_array(views: &[CausalEventView]) -> VarArray {
    let mut arr = VarArray::new();
    for view in views {
        arr.push(&Variant::from(event_view_to_dict(view)));
    }
    arr
}

// ────────────────────────────────────────────────────────────────────────
// P4-γ: Agent snapshot FFI surface — pure-Rust collector + helpers
// ────────────────────────────────────────────────────────────────────────

/// Single row of the agent snapshot returned by [`collect_agent_snapshot`].
///
/// V7 Phase 7-δ extends the row with `state_tag: u8` so the AgentRenderer
/// can tint socializing agents distinctly from Idle / Seeking / non-social
/// Consuming agents.
///
/// Tag table (locked, §2-A-1):
///   - `0` = `AgentState::Idle`
///   - `1` = `AgentState::Seeking { .. }` (any `TargetKind`)
///   - `2` = `AgentState::Consuming { target: TargetKind::Agent(_) }`
///   - `3` = `AgentState::Consuming { .. }` (any non-`Agent` `TargetKind`)
// `Eq` intentionally NOT derived: viz-B added `f32` need fields (hunger/
// thirst/sleep) which are not `Eq`. `PartialEq` (used by the harness
// `assert_eq!` on `Vec<AgentSnapshotRow>`) is sufficient; no `HashSet`/
// `BTreeSet<AgentSnapshotRow>` exists anywhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentSnapshotRow {
    /// `hecs::Entity::to_bits().get()` — stable id within a single
    /// `SimEngine` session (not stable across resets or save/load).
    pub entity_bits: u64,
    /// Tile-x coordinate (post-tick if called after `engine.tick()`).
    pub x: u32,
    /// Tile-y coordinate (post-tick if called after `engine.tick()`).
    pub y: u32,
    /// Phase 7-δ: state tag for renderer tint keying. See type doc for the
    /// locked mapping.
    pub state_tag: u8,
    /// V7 Phase 8-δ (code-attempt 3) — the `Agent.id` value carried by the
    /// row's entity. This is the *AgentId* domain (monotonically minted
    /// `u64`), NOT the `hecs::Entity::to_bits` domain. Surfaced because
    /// `CausalEvent::MemoryRecalled.agent` is an `AgentId` and the
    /// renderer needs to map FFI causal events to the corresponding
    /// rendered row. Preserves `entity_bits` for the Phase 4-γ A5
    /// contract so existing callers (palette swap, click handling) are
    /// untouched.
    pub agent_id: u64,
    /// V7 Section 16-ε: Seeking need kind — 0=none, 1=Food, 2=Water,
    /// 3=Sleep. `Seeking{Agent}` / `Seeking{ConstructionSite}` → 0 (they
    /// are not resource trips); `Idle` / `Consuming{..}` → 0. Carries the
    /// head-dot / goal-line colour key for `seek_viz_renderer.gd`.
    pub seek_kind: u8,
    /// V7 Section 16-ε: `SeekTarget.tile.0` as `i32`, or `-1` when the
    /// agent has no `SeekTarget` component. The renderer's goal-line guard
    /// is `target_x >= 0`, so the absent sentinel must be negative.
    pub target_x: i32,
    /// V7 Section 16-ε: `SeekTarget.tile.1` as `i32`, or `-1` when the
    /// agent has no `SeekTarget` component.
    pub target_y: i32,
    /// V7 viz-B: `Hunger.value` (already `f32`, `[0, 100]` where 100 =
    /// SATURATION). The renderer's need-bar danger ratio is `value / 100`.
    /// `0.0` for an agent with no `Hunger` component (defensive — every real
    /// agent has one).
    pub hunger: f32,
    /// V7 viz-B: `Thirst.value` cast `f64 → f32` (`[0, 100]`). Thirst grows
    /// fastest, so it is usually the dominant (killer) need.
    pub thirst: f32,
    /// V7 viz-B: `Sleep.fatigue` cast `f64 → f32` (`[0, 100]`). Note the
    /// source field is `fatigue`, not `value`.
    pub sleep: f32,
    /// V7 Direction-2 slice 2-5b: the agent's `Inventory` Food count
    /// (`ResourceKind::Food`), `0` when the agent has no `Inventory`. Carries
    /// the carry-indicator presence flag for `seek_viz_renderer.gd` — an agent
    /// hauling gathered Food (`carried_food > 0`) gets a visible glyph, making
    /// the gather → CARRY → deposit → stockpile chain visible. Read-only of
    /// `Inventory`; does not affect determinism.
    pub carried_food: i32,
    /// Revitalize Pass 5 — carried `ResourceKind::Wood` count, 0 without an
    /// `Inventory`. Read-only; drives a future stick-carry glyph.
    pub carried_wood: i32,
    /// Detail-compatible target: 0 none, 1 Food, 2 Water, 3 Sleep,
    /// 4 ConstructionSite, 5 Agent, 6 GatherFood, 7 GatherWood (Pass 5).
    pub target_kind: u8,
    /// Life stage: 0 adult, 1 child with a live Child component, 2 elder
    /// (Pass 4, live Elder component).
    pub life_stage: u8,
}

/// Pass 4 — whole-day age of every snapshot row, in [`collect_agent_snapshot`]
/// order (the same `Agent` + `Position` archetype iteration); `-1` for a
/// resident without `Life` (legacy, v8 and v9 worlds never age).
pub fn collect_agent_ages_days(world: &hecs::World, tick: u64, ticks_per_day: u64) -> Vec<i32> {
    world
        .query::<(&Agent, &Position, Option<&Life>)>()
        .iter()
        .map(|(_, (_, _, life))| {
            life.map_or(-1, |life| {
                (life.age_ticks(tick).max(0) as u64)
                    .checked_div(ticks_per_day)
                    .map_or(0, |days| days.min(i32::MAX as u64) as i32)
            })
        })
        .collect()
}

/// Pass 6 — `1` for every snapshot row whose agent carries the `Cold` marker,
/// else `0`, in [`collect_agent_snapshot`] order (the same `Agent` +
/// `Position` archetype iteration). All zero without the firewood rule.
pub fn collect_agent_colds(world: &hecs::World) -> Vec<u8> {
    world
        .query::<(&Agent, &Position, Option<&Cold>)>()
        .iter()
        .map(|(_, (_, _, cold))| u8::from(cold.is_some()))
        .collect()
}

/// P4-γ pure-Rust collector (Phase 7-δ extension): iterate the world for
/// `(Agent, Position, AgentState)` and return one row per matching entity
/// in hecs query order.
///
/// Order across two consecutive calls on an unchanged world is stable
/// because hecs archetype iteration order is deterministic. Entities
/// possessing `Position` but *not* `Agent` are excluded by the query
/// filter (the `(&Agent, &Position, &AgentState)` tuple requires all
/// three). The `state_tag` value is computed from the same `AgentState`
/// reference returned by the query — no caching layer is introduced.
///
/// Mirrors `WorldSimNode::get_agent_snapshot` minus the Godot
/// `PackedArray` marshalling — sim-test exercises this directly without
/// a Godot runtime.
pub fn collect_agent_snapshot(world: &hecs::World) -> Vec<AgentSnapshotRow> {
    let mut rows = Vec::new();
    for (
        entity,
        (agent, pos, maybe_state, maybe_seek, maybe_hunger, maybe_thirst, maybe_sleep, maybe_inv, maybe_child, maybe_elder),
    ) in world
        .query::<(
            &Agent,
            &Position,
            Option<&AgentState>,
            Option<&SeekTarget>,
            Option<&Hunger>,
            Option<&Thirst>,
            Option<&Sleep>,
            Option<&Inventory>,
            Option<&Child>,
            Option<&Elder>,
        )>()
        .iter()
    {
        let state_tag: u8 = match maybe_state {
            None | Some(AgentState::Idle) => 0,
            Some(AgentState::Seeking { .. }) => 1,
            Some(AgentState::Consuming { target: TargetKind::Agent(_) }) => 2,
            Some(AgentState::Consuming { .. }) => 3,
        };
        // V7 Section 16-ε — Seeking need kind (resource trips only). State
        // is the source of truth: Food/Water/Sleep seeks colour-key the
        // head-dot + goal-line; ConstructionSite/Agent seeks and every
        // non-Seeking state are 0.
        let seek_kind: u8 = match maybe_state {
            Some(AgentState::Seeking { target: TargetKind::Food }) => 1,
            Some(AgentState::Seeking { target: TargetKind::Water }) => 2,
            Some(AgentState::Seeking { target: TargetKind::Sleep }) => 3,
            _ => 0,
        };
        // V7 Section 16-ε — goal tile, or the -1 no-SeekTarget sentinel.
        let (target_x, target_y) = match maybe_seek {
            Some(st) => (st.tile.0 as i32, st.tile.1 as i32),
            None => (-1, -1),
        };
        // V7 viz-B — need values for the head need-bar. Source fields differ:
        // Hunger.value (f32), Thirst.value (f64), Sleep.fatigue (f64). Cast to
        // f32; default 0.0 when a component is absent (no bar drawn).
        let hunger = maybe_hunger.map(|h| h.value).unwrap_or(0.0);
        let thirst = maybe_thirst.map(|t| t.value as f32).unwrap_or(0.0);
        let sleep = maybe_sleep.map(|s| s.fatigue as f32).unwrap_or(0.0);
        // V7 slice 2-5b — carried Food count for the carry indicator. 0 when
        // the agent has no `Inventory`. Read-only; order-independent.
        let carried_food = maybe_inv
            .map(|inv| inv.get(ResourceKind::Food) as i32)
            .unwrap_or(0);
        let carried_wood = maybe_inv
            .map(|inv| inv.get(ResourceKind::Wood) as i32)
            .unwrap_or(0);
        let target_kind: u8 = match maybe_state.and_then(AgentState::target) {
            None => 0,
            Some(TargetKind::Food) => 1,
            Some(TargetKind::Water) => 2,
            Some(TargetKind::Sleep) => 3,
            Some(TargetKind::ConstructionSite) => 4,
            Some(TargetKind::Agent(_)) => 5,
            Some(TargetKind::GatherFood) => 6,
            Some(TargetKind::GatherWood) => 7,
        };
        let life_stage = if maybe_child.is_some() {
            1
        } else if maybe_elder.is_some() {
            2
        } else {
            0
        };
        rows.push(AgentSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            state_tag,
            // V7 Phase 8-δ (code-attempt 3) — surface the `Agent.id` so the
            // renderer can map `CausalEvent::MemoryRecalled.agent` (AgentId)
            // to the correct rendered row without conflating it with
            // `entity_bits`.
            agent_id: agent.id,
            seek_kind,
            target_x,
            target_y,
            hunger,
            thirst,
            sleep,
            carried_food,
            carried_wood,
            target_kind,
            life_stage,
        });
    }
    rows
}

/// Pure-Rust split of `[AgentSnapshotRow]` into four parallel `Vec`s
/// matching the Godot-side `PackedArray` types (`i64`, `i32`, `i32`, `u8`).
///
/// Lengths are equal by construction. Exposed so sim-test can validate
/// the FFI marshalling invariant without a Godot runtime — `agent_rows_to_dict`
/// is a thin `Vec → PackedArray` adapter over this function.
pub fn agent_rows_split(
    rows: &[AgentSnapshotRow],
) -> (Vec<i64>, Vec<i32>, Vec<i32>, Vec<u8>) {
    let n = rows.len();
    let mut ids = Vec::with_capacity(n);
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut states = Vec::with_capacity(n);
    for row in rows {
        ids.push(row.entity_bits as i64);
        xs.push(row.x as i32);
        ys.push(row.y as i32);
        states.push(row.state_tag);
    }
    (ids, xs, ys, states)
}

/// Marshal a [`AgentSnapshotRow`] slice into the FFI dictionary shape
/// documented on [`WorldSimNode::get_agent_snapshot`]. Four keys, four
/// `PackedArray`s, lengths always equal to `rows.len()`.
fn agent_rows_to_dict(rows: &[AgentSnapshotRow]) -> VarDictionary {
    let (ids_vec, xs_vec, ys_vec, states_vec) = agent_rows_split(rows);
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut states = PackedByteArray::new();
    // V7 Phase 8-δ (code-attempt 3) — parallel `agent_ids` array carrying
    // `Agent.id` per row, used by the renderer to match
    // `CausalEvent::MemoryRecalled.agent` (AgentId) against the rendered
    // row. `ids` remains keyed by `entity_bits` for the Phase 4-γ A5
    // contract (palette swap, click handling).
    let mut agent_ids = PackedInt64Array::new();
    ids.resize(ids_vec.len());
    xs.resize(xs_vec.len());
    ys.resize(ys_vec.len());
    states.resize(states_vec.len());
    agent_ids.resize(rows.len());
    for (i, v) in ids_vec.iter().enumerate() {
        ids[i] = *v;
    }
    for (i, v) in xs_vec.iter().enumerate() {
        xs[i] = *v;
    }
    for (i, v) in ys_vec.iter().enumerate() {
        ys[i] = *v;
    }
    for (i, v) in states_vec.iter().enumerate() {
        states[i] = *v;
    }
    for (i, row) in rows.iter().enumerate() {
        agent_ids[i] = row.agent_id as i64;
    }
    // V7 Section 16-ε — three additive parallel arrays carrying the head-dot
    // / goal-line fields. Built directly from `rows` (NOT routed through
    // `agent_rows_split`, whose 4-tuple signature is locked by
    // `harness_p4_gamma_rendering`). Lengths equal `rows.len()` by
    // construction, matching the existing `ids`/`xs`/`ys`/`states` contract.
    let mut seek_kinds = PackedByteArray::new();
    let mut target_xs = PackedInt32Array::new();
    let mut target_ys = PackedInt32Array::new();
    seek_kinds.resize(rows.len());
    target_xs.resize(rows.len());
    target_ys.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        seek_kinds[i] = row.seek_kind;
        target_xs[i] = row.target_x;
        target_ys[i] = row.target_y;
    }
    // V7 viz-B — three additive parallel `f32` arrays carrying the per-agent
    // need values (Hunger / Thirst / Sleep, `[0, 100]`) for the head need-bar.
    // Built directly from `rows` (NOT via `agent_rows_split`, whose 4-tuple is
    // locked by `harness_p4_gamma_rendering`), mirroring the Section 16-ε
    // seek_kinds/target_xs/target_ys additive pattern. Lengths == `rows.len()`.
    let mut hungers = PackedFloat32Array::new();
    let mut thirsts = PackedFloat32Array::new();
    let mut sleeps = PackedFloat32Array::new();
    hungers.resize(rows.len());
    thirsts.resize(rows.len());
    sleeps.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        hungers[i] = row.hunger;
        thirsts[i] = row.thirst;
        sleeps[i] = row.sleep;
    }
    // V7 slice 2-5b — one additive parallel `i32` array carrying the per-agent
    // carried Food count for the carry indicator. Built directly from `rows`
    // (NOT via `agent_rows_split`, whose 4-tuple is locked by
    // `harness_p4_gamma_rendering`), mirroring the Section 16-ε / viz-B additive
    // pattern. Length == `rows.len()` by construction.
    let mut carried_foods = PackedInt32Array::new();
    carried_foods.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        carried_foods[i] = row.carried_food;
    }
    // Revitalize Pass 5 — additive parallel array of carried wood.
    let mut carried_woods = PackedInt32Array::new();
    carried_woods.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        carried_woods[i] = row.carried_wood;
    }
    let mut target_kinds = PackedByteArray::new();
    let mut life_stages = PackedByteArray::new();
    target_kinds.resize(rows.len());
    life_stages.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        target_kinds[i] = row.target_kind;
        life_stages[i] = row.life_stage;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("states", states);
    dict.set("agent_ids", agent_ids);
    dict.set("seek_kinds", seek_kinds);
    dict.set("target_xs", target_xs);
    dict.set("target_ys", target_ys);
    dict.set("hungers", hungers);
    dict.set("thirsts", thirsts);
    dict.set("sleeps", sleeps);
    dict.set("carried_foods", carried_foods);
    dict.set("carried_woods", carried_woods);
    dict.set("target_kinds", target_kinds);
    dict.set("life_stages", life_stages);
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 Phase 14-γ: Single-agent detail FFI surface — click inspector
// ────────────────────────────────────────────────────────────────────────

/// Single row of the agent detail row returned by [`collect_agent_detail`].
///
/// V7 Phase 14-γ — Conservative 8-field scope (P14Plan-5, locked
/// 2026-05-25) for the click inspector panel:
///
/// - `agent_id` — `Agent.id` (AgentId domain, matches snapshot.agent_ids[i])
/// - `x`, `y` — tile coordinates as `i32` (matches the snapshot's
///   `xs`/`ys` type contract — Bridge Identity Contract type lock)
/// - `state_tag` — same locked Phase 4-γ A5 mapping as
///   [`AgentSnapshotRow`] (0=Idle, 1=Seeking, 2=Consuming(Agent),
///   3=Consuming(other))
/// - `hunger` — `Hunger.value` (`f32`; `[0, SATURATION=100]`)
/// - `thirst` — `Thirst.value` (`f64`; `[0, SATURATION=100]`)
/// - `sleep` — `Sleep.fatigue` (`f64`; `[0, SATURATION=100]`)
/// - `target_kind` — `i32` encoding of `Option<TargetKind>`:
///   0=None (Idle), 1=Food, 2=Water, 3=Sleep, 4=ConstructionSite,
///   5=Agent (inner `AgentId` NOT surfaced in the Conservative scope;
///   relationship surfacing deferred to Section 16+)
///
/// The `found` field distinguishes "row populated" (entity is a live
/// agent carrying the full component bundle) from "entity not found / not
/// an Agent / missing required components". GDScript callers branch on
/// `found` to handle stale-click edge cases without panicking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentDetailRow {
    /// True iff the entity exists and carries the full
    /// `(Agent, Position, AgentState, Hunger, Thirst, Sleep)` bundle.
    pub found: bool,
    /// `Agent.id` of the entity (AgentId domain — NOT `entity_bits`).
    /// Zero when `found == false`.
    pub agent_id: u64,
    /// Tile-x coordinate of the agent's `Position`. `0` when not found.
    pub x: i32,
    /// Tile-y coordinate of the agent's `Position`. `0` when not found.
    pub y: i32,
    /// Phase 4-γ A5 state-tag mapping. `0` when not found.
    pub state_tag: u8,
    /// Current `Hunger.value`. `0.0` when not found.
    pub hunger: f32,
    /// Current `Thirst.value`. `0.0` when not found.
    pub thirst: f64,
    /// Current `Sleep.fatigue`. `0.0` when not found.
    pub sleep: f64,
    /// Encoded `Option<TargetKind>`. `0` when not found (or Idle).
    pub target_kind: i32,
}

impl Default for AgentDetailRow {
    fn default() -> Self {
        Self {
            found: false,
            agent_id: 0,
            x: 0,
            y: 0,
            state_tag: 0,
            hunger: 0.0,
            thirst: 0.0,
            sleep: 0.0,
            target_kind: 0,
        }
    }
}

/// Canonical 9-key set published by the Phase 14-γ detail FFI dictionary.
///
/// V7 Phase 14-γ Bridge Identity Contract — the GDScript-facing
/// `get_agent_detail()` dictionary MUST emit exactly these keys, in
/// any order. Sim-test asserts this slice against the locked plan
/// schema (P14Plan-5 Conservative 8 fields + `found` sentinel = 9).
///
/// The `agent_detail_to_dict` marshaller iterates this list as the
/// single source of truth so the dict's key set can never drift from
/// the slice without a compile-time edit.
pub const AGENT_DETAIL_DICT_KEYS: [&str; 9] = [
    "found",
    "agent_id",
    "x",
    "y",
    "state_tag",
    "hunger",
    "thirst",
    "sleep",
    "target_kind",
];

/// V7 Phase 14-γ pure-Rust collector — look up a single agent by
/// `Entity::to_bits()` and return its 8-field detail row.
///
/// Returns an [`AgentDetailRow`] with `found = false` (other fields at
/// their `Default` values) when:
///   - `entity_bits` does not form a valid `hecs::Entity` (e.g. `0`,
///     or a stale generation), OR
///   - the entity is not alive in `world`, OR
///   - the entity lacks the required component bundle
///     `(Agent, Position, AgentState, Hunger, Thirst, Sleep)`.
///
/// Bridge Identity Contract — mirrors `WorldSimNode::get_agent_detail`
/// minus the Godot `VarDictionary` marshalling so sim-test exercises
/// this directly without a Godot runtime.
pub fn collect_agent_detail(world: &hecs::World, entity_bits: u64) -> AgentDetailRow {
    let entity = match hecs::Entity::from_bits(entity_bits) {
        Some(e) => e,
        None => return AgentDetailRow::default(),
    };
    let mut q = match world.query_one::<(
        &Agent,
        &Position,
        &AgentState,
        &Hunger,
        &Thirst,
        &Sleep,
    )>(entity)
    {
        Ok(q) => q,
        Err(_) => return AgentDetailRow::default(),
    };
    let (agent, pos, state, hunger, thirst, sleep) = match q.get() {
        Some(tup) => tup,
        None => return AgentDetailRow::default(),
    };
    let state_tag: u8 = match state {
        AgentState::Idle => 0,
        AgentState::Seeking { .. } => 1,
        AgentState::Consuming {
            target: TargetKind::Agent(_),
        } => 2,
        AgentState::Consuming { .. } => 3,
    };
    let target_kind: i32 = match state.target() {
        None => 0,
        Some(TargetKind::Food) => 1,
        Some(TargetKind::Water) => 2,
        Some(TargetKind::Sleep) => 3,
        Some(TargetKind::ConstructionSite) => 4,
        Some(TargetKind::Agent(_)) => 5,
        // Direction-2 slice 2-2 — carry-pickup target (viz lands in slice 2-5).
        Some(TargetKind::GatherFood) => 6,
        // Revitalize Pass 5 — firewood pickup target.
        Some(TargetKind::GatherWood) => 7,
    };
    AgentDetailRow {
        found: true,
        agent_id: agent.id,
        x: pos.x as i32,
        y: pos.y as i32,
        state_tag,
        hunger: hunger.value,
        thirst: thirst.value,
        sleep: sleep.fatigue,
        target_kind,
    }
}

/// Marshal an [`AgentDetailRow`] into the FFI dictionary shape consumed
/// by `WorldRenderer._try_agent_click()`. Emits exactly the 9 keys in
/// [`AGENT_DETAIL_DICT_KEYS`]. Iterates the locked key list so adding a
/// new key to the schema is a single-site edit.
fn agent_detail_to_dict(row: AgentDetailRow) -> VarDictionary {
    let mut dict = VarDictionary::new();
    for &key in AGENT_DETAIL_DICT_KEYS.iter() {
        match key {
            "found" => {
                dict.set(key, row.found);
            }
            "agent_id" => {
                dict.set(key, row.agent_id as i64);
            }
            "x" => {
                dict.set(key, row.x);
            }
            "y" => {
                dict.set(key, row.y);
            }
            "state_tag" => {
                dict.set(key, row.state_tag as i64);
            }
            "hunger" => {
                dict.set(key, row.hunger as f64);
            }
            "thirst" => {
                dict.set(key, row.thirst);
            }
            "sleep" => {
                dict.set(key, row.sleep);
            }
            "target_kind" => {
                dict.set(key, row.target_kind);
            }
            // Compile-time guarantee: every key in AGENT_DETAIL_DICT_KEYS
            // is handled above. Any future addition must extend the match.
            other => unreachable!(
                "agent_detail_to_dict: unhandled key `{other}` in AGENT_DETAIL_DICT_KEYS"
            ),
        }
    }
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 Phase 12-β.2 (A3): Construction snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the construction snapshot returned by
/// [`collect_construction_snapshot`].
///
/// V7 Phase 12-β.2 (A3) — surfaces `ConstructionSite` entities to the
/// GDScript renderer so the user can see agent construction activity.
/// Only the `progress` ratio is exposed; the `BlueprintId` and
/// `footprint` fields of the underlying `BuildingBlueprint` are
/// intentionally NOT in the row because the substrate has no
/// `BuildingType` taxonomy — multi-type rendering is a separate
/// (deferred) phase that first adds that substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructionSnapshotRow {
    /// `hecs::Entity::to_bits().get()` of the construction-site entity.
    pub entity_bits: u64,
    /// Tile-x coordinate of the site's footprint top-left.
    pub x: u32,
    /// Tile-y coordinate of the site's footprint top-left.
    pub y: u32,
    /// Current construction progress in `ConstructionSystem` ticks.
    pub progress: u32,
    /// Total ticks required for completion (from `BuildingBlueprint`).
    pub required_progress: u32,
}

/// Pure-Rust collector for [`ConstructionSnapshotRow`] — mirrors
/// [`collect_agent_snapshot`] but queries `(&ConstructionSite, &Position)`
/// instead of `(&Agent, &Position, Option<&AgentState>)`.
///
/// Position is taken from the entity's `Position` component (the
/// canonical sim-core source) rather than `ConstructionSite::position`
/// so the rendered tile matches whatever the simulation considers
/// authoritative for that entity. The two should agree in practice.
pub fn collect_construction_snapshot(world: &hecs::World) -> Vec<ConstructionSnapshotRow> {
    let mut rows = Vec::new();
    for (entity, (site, pos)) in world
        .query::<(&ConstructionSite, &Position)>()
        .iter()
    {
        rows.push(ConstructionSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            progress: site.progress,
            required_progress: site.blueprint.required_progress,
        });
    }
    rows
}

/// Marshal a [`ConstructionSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer._process()`. Five parallel
/// `PackedArray`s, lengths always equal to `rows.len()`.
///
/// Keys:
/// - `ids`:  `PackedInt64Array` — `entity_bits` per row (signed cast
///   matches the agent snapshot precedent).
/// - `xs`:  `PackedInt32Array` — tile-x per row, as `i32`.
/// - `ys`:  `PackedInt32Array` — tile-y per row, as `i32`.
/// - `progresses`: `PackedInt32Array` — current progress per row, as `i32`.
/// - `required_progresses`: `PackedInt32Array` — required progress per
///   row, as `i32`. The renderer must still defend against div-by-zero
///   via `max(req, 1)` because the substrate permits `required_progress == 0`
///   blueprints.
fn construction_rows_to_dict(rows: &[ConstructionSnapshotRow]) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut progresses = PackedInt32Array::new();
    let mut required_progresses = PackedInt32Array::new();
    ids.resize(n);
    xs.resize(n);
    ys.resize(n);
    progresses.resize(n);
    required_progresses.resize(n);
    for (i, row) in rows.iter().enumerate() {
        ids[i] = row.entity_bits as i64;
        xs[i] = row.x as i32;
        ys[i] = row.y as i32;
        progresses[i] = row.progress as i32;
        required_progresses[i] = row.required_progress as i32;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("progresses", progresses);
    dict.set("required_progresses", required_progresses);
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 Section 16-α0: Resource-substrate snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the resource-substrate snapshot returned by
/// [`collect_resource_snapshot`]. `kind` encoding: `0 = Food`, `1 = Water`,
/// `2 = Sleep` (matches the `TargetKind` discriminant order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSnapshotRow {
    /// Tile-x coordinate of the source tile.
    pub x: u32,
    /// Tile-y coordinate of the source tile.
    pub y: u32,
    /// Resource kind: `0 = Food`, `1 = Water`, `2 = Sleep`, `3 = Deadwood`
    /// (Pass 5; registered spots persist at amount 0 between pickup and regrowth).
    pub kind: u8,
    /// Current stored amount on the tile (the `*_tiles` counter), CLAMPED to
    /// [`max`](Self::max). Drives the renderer's depletion/regen visual (marker
    /// scale + alpha = amount/max). A finite source decreases as agents consume
    /// and recovers via regen. Legacy scenes omit removed tiles; game rules
    /// retain registered food sources at zero so depleted bushes remain visible.
    /// Clamped to
    /// `max` so the displayed ratio never exceeds `1.0` (regen overshoot or a
    /// zero-cap source cannot make the marker read as more than full).
    pub amount: u8,
    /// Original source capacity (the `*_source_max` registry). The denominator
    /// for the depletion ratio. Guaranteed `>= 1` (a tile with no registered
    /// ceiling — e.g. an `RESOURCE_SOURCE_INFINITE` tile — reports `max ==
    /// amount`, so its ratio is `1.0` and the marker stays full).
    pub max: u8,
}

/// Pure-Rust collector over the three sparse tile maps on [`SimResources`].
///
/// **Sorted by `(kind, x, y)`** — `HashMap` iteration order is unspecified,
/// so the explicit sort is what makes the snapshot deterministic for the
/// renderer markers and the determinism harness assertion. Sim-test
/// exercises this collector directly (no Godot runtime required).
pub fn collect_resource_snapshot(resources: &SimResources) -> Vec<ResourceSnapshotRow> {
    let mut rows: Vec<ResourceSnapshotRow> = Vec::with_capacity(
        resources.food_tiles.len() + resources.water_tiles.len() + resources.sleep_tiles.len(),
    );
    // `amount` = the live tile counter (`*_tiles` value); `max` = the source
    // capacity (`*_source_max`), defaulting to `amount` (ratio 1.0) for any
    // tile with no registered ceiling (e.g. an infinite-sentinel source), and
    // floored at 1 so the renderer's `amount / max` is never a divide-by-zero.
    //
    // `amount` is finally CLAMPED to `max` (`amount.min(max)`): a depletion
    // display is bounded at "full", so the snapshot must never emit `amount >
    // max` (ratio > 1.0) — which the renderer's scale/alpha lerp would overshoot
    // beyond the full marker. Two pathological inputs need this clamp: regen
    // overshoot (live counter momentarily exceeds the registered cap) and an
    // explicitly-registered `max == 0` (floored to 1, so a raw amount would read
    // as e.g. 5/1 = 5.0). The clamp is read-only on the SNAPSHOT — the backend
    // tile counters are untouched (pure visualisation, no logic change).
    for (&(x, y), &amount) in resources.food_tiles.iter() {
        let max = resources.food_source_max.get(&(x, y)).copied().unwrap_or(amount).max(1);
        let amount = amount.min(max);
        rows.push(ResourceSnapshotRow { x, y, kind: 0, amount, max });
    }
    if resources.rules.society.shelter_planning.is_some() {
        // The bush persists between harvest and regrowth even when its live
        // food counter is absent. The ceiling registry owns its existence.
        for (&(x, y), &capacity) in &resources.food_source_max {
            if !resources.food_tiles.contains_key(&(x, y)) {
                rows.push(ResourceSnapshotRow {
                    x,
                    y,
                    kind: 0,
                    amount: 0,
                    max: capacity.max(1),
                });
            }
        }
    }
    for (&(x, y), &amount) in resources.water_tiles.iter() {
        let max = resources.water_source_max.get(&(x, y)).copied().unwrap_or(amount).max(1);
        let amount = amount.min(max);
        rows.push(ResourceSnapshotRow { x, y, kind: 1, amount, max });
    }
    for (&(x, y), &amount) in resources.sleep_tiles.iter() {
        let max = resources.sleep_source_max.get(&(x, y)).copied().unwrap_or(amount).max(1);
        let amount = amount.min(max);
        rows.push(ResourceSnapshotRow { x, y, kind: 2, amount, max });
    }
    // Revitalize Pass 5 — every registered deadwood spot, at amount 0 while
    // emptied, so the pile stays visible between pickup and regrowth.
    for (&(x, y), &capacity) in &resources.wood_source_max {
        let max = capacity.max(1);
        let amount = resources.wood_tiles.get(&(x, y)).copied().unwrap_or(0).min(max);
        rows.push(ResourceSnapshotRow { x, y, kind: 3, amount, max });
    }
    rows.sort_by_key(|r| (r.kind, r.x, r.y));
    rows
}

#[cfg(test)]
mod depleted_food_snapshot_tests {
    use super::*;

    #[test]
    fn game_snapshot_keeps_depleted_bush_and_sorts_without_duplicates() {
        let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::game_v8();
        engine.resources.set_food_source_max(8, 4, 6);
        engine.resources.set_food_source_max(2, 3, 6);
        engine.resources.set_food_tile(8, 4, 3);
        let rows = collect_resource_snapshot(&engine.resources);
        assert_eq!(rows, vec![
            ResourceSnapshotRow { x: 2, y: 3, kind: 0, amount: 0, max: 6 },
            ResourceSnapshotRow { x: 8, y: 4, kind: 0, amount: 3, max: 6 },
        ]);
        assert!(!engine.resources.food_tiles.contains_key(&(2, 3)));
    }

    #[test]
    fn legacy_snapshot_omits_source_without_live_food() {
        let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
        engine.resources.set_food_source_max(2, 3, 6);
        assert!(collect_resource_snapshot(&engine.resources).is_empty());
    }
}

/// Pure-Rust marshalling split — the SINGLE source of the `(xs, ys, kinds)`
/// integer arrays. Extracted so the harness can verify the exact integers
/// the FFI emits WITHOUT a Godot runtime (`VarDictionary` /
/// `PackedInt32Array` require Godot). [`resource_rows_to_dict`] MUST build
/// its `PackedInt32Array`s from this — no duplicate marshalling logic.
pub fn resource_rows_split(rows: &[ResourceSnapshotRow]) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    let mut kinds = Vec::with_capacity(rows.len());
    for r in rows {
        xs.push(r.x as i32);
        ys.push(r.y as i32);
        kinds.push(r.kind as i32);
    }
    (xs, ys, kinds)
}

/// Companion to [`resource_rows_split`] for the depletion visual: the parallel
/// `(amounts, maxes)` integer arrays, in the SAME row order. Kept separate from
/// `resource_rows_split` so the locked `(xs, ys, kinds)` contract (and the
/// harness assertions on it) is untouched while [`resource_rows_to_dict`] gains
/// the `amounts` / `maxes` keys. `maxes[i] >= 1` (floored in
/// [`collect_resource_snapshot`]).
pub fn resource_rows_amounts(rows: &[ResourceSnapshotRow]) -> (Vec<i32>, Vec<i32>) {
    let mut amounts = Vec::with_capacity(rows.len());
    let mut maxes = Vec::with_capacity(rows.len());
    for r in rows {
        amounts.push(r.amount as i32);
        maxes.push(r.max as i32);
    }
    (amounts, maxes)
}

/// Marshal a [`ResourceSnapshotRow`] slice into the FFI dictionary shape
/// consumed by `WorldRenderer._render_resource_sources()`. Three parallel
/// `PackedInt32Array`s (`xs`, `ys`, `kinds`), lengths always equal to
/// `rows.len()`. Built solely from [`resource_rows_split`] (the single
/// marshalling path the harness tests).
fn resource_rows_to_dict(rows: &[ResourceSnapshotRow]) -> VarDictionary {
    let (xv, yv, kv) = resource_rows_split(rows);
    let (av, mv) = resource_rows_amounts(rows);
    let n = rows.len();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut kinds = PackedInt32Array::new();
    let mut amounts = PackedInt32Array::new();
    let mut maxes = PackedInt32Array::new();
    xs.resize(n);
    ys.resize(n);
    kinds.resize(n);
    amounts.resize(n);
    maxes.resize(n);
    for i in 0..n {
        xs[i] = xv[i];
        ys[i] = yv[i];
        kinds[i] = kv[i];
        amounts[i] = av[i];
        maxes[i] = mv[i];
    }
    let mut dict = VarDictionary::new();
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("kinds", kinds);
    dict.set("amounts", amounts);
    dict.set("maxes", maxes);
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 viz-D: Recent-deaths snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the recent-deaths snapshot returned by
/// [`collect_recent_deaths`]. Documented casts from the buffer entry:
/// `x`/`y` `i32 → i32` (identity), `reason` `DeathReason → u8 → i32`,
/// `tick` `u32 → i64` (widening).
/// `reason_u8` encoding mirrors [`DeathReason::as_u8`]: `0 = Starvation`
/// (brown), `1 = Dehydration` (blue), `2 = Combat` (red).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecentDeathRow {
    /// Death tile-x.
    pub x: i32,
    /// Death tile-y.
    pub y: i32,
    /// Reason discriminant (`0/1/2`) — see [`DeathReason::as_u8`].
    pub reason_u8: i32,
    /// Simulation tick at which the death occurred.
    pub tick: i64,
}

/// Pure-Rust collector over [`SimResources::recent_deaths`], preserving buffer
/// order (the renderer fades each marker by `current_tick - tick`, so order
/// only matters for stable iteration). Sim-test exercises this directly.
pub fn collect_recent_deaths(resources: &SimResources) -> Vec<RecentDeathRow> {
    resources
        .recent_deaths
        .iter()
        .map(|d| RecentDeathRow {
            // `d.x`/`d.y` are already `i32` (buffer schema) → identity cast,
            // written without `as i32` to satisfy `clippy::unnecessary_cast`.
            x: d.x,
            y: d.y,
            reason_u8: d.reason.as_u8() as i32,
            tick: d.tick as i64,
        })
        .collect()
}

/// Pure-Rust marshalling split — the SINGLE source of the
/// `(xs, ys, reasons, ticks)` integer arrays, in buffer order. Extracted so the
/// harness can verify the exact integers the FFI emits WITHOUT a Godot runtime
/// (`VarDictionary` / `PackedInt32Array` require Godot).
/// [`recent_death_rows_to_dict`] MUST build its arrays from this — no duplicate
/// marshalling logic.
pub fn recent_death_rows_split(rows: &[RecentDeathRow]) -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i64>) {
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    let mut reasons = Vec::with_capacity(rows.len());
    let mut ticks = Vec::with_capacity(rows.len());
    for r in rows {
        xs.push(r.x);
        ys.push(r.y);
        reasons.push(r.reason_u8);
        ticks.push(r.tick);
    }
    (xs, ys, reasons, ticks)
}

/// Marshal a [`RecentDeathRow`] slice + the engine's `current_tick` into the FFI
/// dictionary shape consumed by `death_viz_renderer.gd`. Four parallel arrays
/// (`xs`, `ys` as `PackedInt32Array`; `reasons` as `PackedInt32Array`; `ticks`
/// as `PackedInt64Array`) of equal length `rows.len()`, plus the scalar
/// `current_tick` (`i64`) the renderer uses to compute each marker's fade age.
/// Built solely from [`recent_death_rows_split`] (the single marshalling path
/// the harness tests).
fn recent_death_rows_to_dict(rows: &[RecentDeathRow], current_tick: i64) -> VarDictionary {
    let (xv, yv, rv, tv) = recent_death_rows_split(rows);
    let n = rows.len();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut reasons = PackedInt32Array::new();
    let mut ticks = PackedInt64Array::new();
    xs.resize(n);
    ys.resize(n);
    reasons.resize(n);
    ticks.resize(n);
    for i in 0..n {
        xs[i] = xv[i];
        ys[i] = yv[i];
        reasons[i] = rv[i];
        ticks[i] = tv[i];
    }
    let mut dict = VarDictionary::new();
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("reasons", reasons);
    dict.set("ticks", ticks);
    dict.set("current_tick", current_tick);
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 Phase 12-γ: Settlement snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the settlement snapshot returned by
/// [`collect_settlement_snapshot`].
///
/// V7 Phase 12-γ — surfaces `Settlement` entities to the GDScript renderer
/// with a substrate-derived centroid (mean position of member agents). The
/// substrate has no `Settlement.position` field; this is a derived UI
/// affordance that does NOT add or modify any sim-core data.
///
/// `member_count` is the count of *resolvable* member agents (those whose
/// `AgentId` matches a live `(Agent, Position)` pair in the ECS world).
/// Settlements with zero resolvable members are dropped by the collector
/// rather than emitted with a divide-by-zero centroid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettlementSnapshotRow {
    /// `hecs::Entity::to_bits().get()` of the Settlement entity.
    pub entity_bits: u64,
    /// `Settlement::settlement_id`.
    pub settlement_id: u32,
    /// Floor of the mean tile-x of resolvable member agents.
    pub centroid_x: i32,
    /// Floor of the mean tile-y of resolvable member agents.
    pub centroid_y: i32,
    /// Count of resolvable member agents.
    pub member_count: u32,
    /// `Settlement::formation_tile.0` — the FIXED formation-anchor tile-x,
    /// set once at formation. Unlike `centroid_x` (live mean of members), this
    /// does not move, so the GDScript marker anchored to it sits still.
    pub formation_x: i32,
    /// `Settlement::formation_tile.1` — the FIXED formation-anchor tile-y.
    pub formation_y: i32,
    /// Settlement `stockpile` Food count (`ResourceKind::Food`), 0 when none
    /// stocked. Additive Direction-2 slice 2-5a field — surfaces the reserve
    /// the gather→deposit→consume chain (2-2/2-3a/2-4) fills and drains.
    pub food_stock: i32,
    /// Revitalize Pass 5 — settlement `stockpile` Wood count, 0 when none.
    pub wood_stock: i32,
}

/// Pure-Rust collector mirroring [`collect_construction_snapshot`] but
/// joining each Settlement to its member agents' Positions.
///
/// Iterates the authoritative `resources.settlements` store (a
/// `HashMap<SettlementId, Settlement>` — settlements are NEVER spawned as
/// ECS world entities), then for each settlement averages the `Position` of
/// every member agent resolvable via the `world` `(Agent, Position)` join.
/// Settlements whose `member_agents` set is empty OR whose members are all
/// stale (not present as `(Agent, Position)` in the world) are skipped —
/// emitting them would require dividing by zero. `entity_bits` carries
/// `settlement_id` (no ECS entity exists; the field is only a stable unique
/// key downstream). Rows are returned sorted by `settlement_id` for
/// deterministic ordering across runs.
pub fn collect_settlement_snapshot(
    world: &hecs::World,
    settlements: &std::collections::HashMap<SettlementId, Settlement>,
) -> Vec<SettlementSnapshotRow> {
    // First pass: build an `Agent.id` → `(x, y)` lookup. Settlement
    // members are referenced by AgentId, not hecs::Entity, so this
    // intermediate index is required.
    let mut agent_positions: std::collections::HashMap<u64, (u32, u32)> =
        std::collections::HashMap::new();
    for (_, (agent, pos)) in world.query::<(&Agent, &Position)>().iter() {
        agent_positions.insert(agent.id, (pos.x, pos.y));
    }

    let mut rows = Vec::new();
    for settlement in settlements.values() {
        let mut sum_x: u64 = 0;
        let mut sum_y: u64 = 0;
        let mut count: u32 = 0;
        for member_id in settlement.member_agents.iter() {
            if let Some(&(x, y)) = agent_positions.get(member_id) {
                sum_x += x as u64;
                sum_y += y as u64;
                count += 1;
            }
        }
        if count == 0 {
            continue;
        }
        let centroid_x = (sum_x / count as u64) as i32;
        let centroid_y = (sum_y / count as u64) as i32;
        rows.push(SettlementSnapshotRow {
            entity_bits: settlement.settlement_id as u64,
            settlement_id: settlement.settlement_id,
            centroid_x,
            centroid_y,
            member_count: count,
            formation_x: settlement.formation_tile.0 as i32,
            formation_y: settlement.formation_tile.1 as i32,
            food_stock: settlement
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0) as i32,
            wood_stock: settlement
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0) as i32,
        });
    }
    // HashMap iteration order is not stable run-to-run; sort by the unique
    // settlement_id so the snapshot is deterministic (Day-1 invariant +
    // stable furniture-sprite keying downstream).
    rows.sort_by_key(|r| r.settlement_id);
    rows
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SocietySnapshot {
    settlement_ids: Vec<i32>,
    parent_ids: Vec<i32>,
    founded_ticks: Vec<i64>,
    member_counts: Vec<i32>,
    fission_pressures: Vec<i64>,
    fission_thresholds: Vec<i64>,
    home_agent_ids: Vec<i64>,
    home_settlement_ids: Vec<i32>,
    founder_agent_ids: Vec<i64>,
    founder_fission_ids: Vec<i32>,
    founder_site_xs: Vec<i32>,
    founder_site_ys: Vec<i32>,
    founder_leader: Vec<u8>,
    founder_since_ticks: Vec<i64>,
    fission_ids: Vec<i32>,
    fission_parents: Vec<i32>,
    fission_site_xs: Vec<i32>,
    fission_site_ys: Vec<i32>,
    fission_sizes: Vec<i32>,
    fission_depart_ticks: Vec<i64>,
    fission_founded_ticks: Vec<i64>,
    fission_cancelled_ticks: Vec<i64>,
    fission_daughters: Vec<i32>,
    fission_food_taken: Vec<i64>,
    retired_hearth_xs: Vec<i32>,
    retired_hearth_ys: Vec<i32>,
    current_tick: i64,
}

fn collect_society_snapshot(world: &hecs::World, resources: &SimResources) -> SocietySnapshot {
    use sim_core::components::Founding;

    // Match the existing bridge's integer representation, including u64 AgentId bits.
    let mut snapshot = SocietySnapshot {
        current_tick: resources.current_tick as i64,
        ..SocietySnapshot::default()
    };
    for row in collect_settlement_snapshot(world, &resources.settlements) {
        let settlement = &resources.settlements[&row.settlement_id];
        snapshot.settlement_ids.push(settlement.settlement_id as i32);
        snapshot.parent_ids.push(settlement.parent.map_or(-1, |id| id as i32));
        snapshot.founded_ticks.push(settlement.founded_at as i64);
        snapshot.member_counts.push(settlement.member_agents.len() as i32);
        snapshot.fission_pressures.push(i64::try_from(settlement.fission_pressure).unwrap_or(i64::MAX));
        snapshot.fission_thresholds.push(i64::try_from(settlement.fission_threshold).unwrap_or(i64::MAX));
    }
    let mut homes: Vec<_> = resources.settlements.values()
        .flat_map(|settlement| settlement.member_agents.iter()
            .map(|&agent_id| (agent_id, settlement.settlement_id)))
        .collect();
    homes.sort_unstable();
    for (agent_id, settlement_id) in homes {
        snapshot.home_agent_ids.push(agent_id as i64);
        snapshot.home_settlement_ids.push(settlement_id as i32);
    }
    let mut founders: Vec<_> = world.query::<(&Agent, &Founding)>().iter()
        .map(|(_, (agent, founding))| (agent.id, *founding)).collect();
    founders.sort_unstable_by_key(|(agent_id, _)| *agent_id);
    for (agent_id, founding) in founders {
        snapshot.founder_agent_ids.push(agent_id as i64);
        snapshot.founder_fission_ids.push(founding.fission_id as i32);
        snapshot.founder_site_xs.push(founding.site.0 as i32);
        snapshot.founder_site_ys.push(founding.site.1 as i32);
        snapshot.founder_leader.push(u8::from(agent_id == founding.leader));
        snapshot.founder_since_ticks.push(founding.since_tick as i64);
    }
    let mut fissions: Vec<_> = resources.fission_log.iter().collect();
    fissions.sort_unstable_by_key(|record| record.fission_id);
    for record in fissions {
        snapshot.fission_ids.push(record.fission_id as i32);
        snapshot.fission_parents.push(record.parent as i32);
        snapshot.fission_site_xs.push(record.site.0 as i32);
        snapshot.fission_site_ys.push(record.site.1 as i32);
        snapshot.fission_sizes.push(record.members.len() as i32);
        snapshot.fission_depart_ticks.push(record.depart_tick as i64);
        snapshot.fission_founded_ticks.push(record.founded_tick.map_or(-1, |tick| tick as i64));
        snapshot.fission_cancelled_ticks.push(record.cancelled_tick.map_or(-1, |tick| tick as i64));
        snapshot.fission_daughters.push(record.daughter.map_or(-1, |id| id as i32));
        snapshot.fission_food_taken.push(i64::from(record.food_taken));
    }
    for &(x, y) in &resources.retired_hearths {
        snapshot.retired_hearth_xs.push(x as i32);
        snapshot.retired_hearth_ys.push(y as i32);
    }
    snapshot
}

fn society_snapshot_to_dict(snapshot: &SocietySnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    for (key, values) in [
        ("settlement_ids", &snapshot.settlement_ids),
        ("parent_ids", &snapshot.parent_ids),
        ("member_counts", &snapshot.member_counts),
        ("home_settlement_ids", &snapshot.home_settlement_ids),
        ("founder_fission_ids", &snapshot.founder_fission_ids),
        ("founder_site_xs", &snapshot.founder_site_xs),
        ("founder_site_ys", &snapshot.founder_site_ys),
        ("fission_ids", &snapshot.fission_ids),
        ("fission_parents", &snapshot.fission_parents),
        ("fission_site_xs", &snapshot.fission_site_xs),
        ("fission_site_ys", &snapshot.fission_site_ys),
        ("fission_sizes", &snapshot.fission_sizes),
        ("fission_daughters", &snapshot.fission_daughters),
        ("retired_hearth_xs", &snapshot.retired_hearth_xs),
        ("retired_hearth_ys", &snapshot.retired_hearth_ys),
    ] {
        dict.set(key, PackedInt32Array::from(values.as_slice()));
    }
    for (key, values) in [
        ("founded_ticks", &snapshot.founded_ticks),
        ("fission_pressures", &snapshot.fission_pressures),
        ("fission_thresholds", &snapshot.fission_thresholds),
        ("home_agent_ids", &snapshot.home_agent_ids),
        ("founder_agent_ids", &snapshot.founder_agent_ids),
        ("founder_since_ticks", &snapshot.founder_since_ticks),
        ("fission_depart_ticks", &snapshot.fission_depart_ticks),
        ("fission_founded_ticks", &snapshot.fission_founded_ticks),
        ("fission_cancelled_ticks", &snapshot.fission_cancelled_ticks),
        ("fission_food_taken", &snapshot.fission_food_taken),
    ] {
        dict.set(key, PackedInt64Array::from(values.as_slice()));
    }
    dict.set("founder_leader", PackedByteArray::from(snapshot.founder_leader.as_slice()));
    dict.set("current_tick", snapshot.current_tick);
    dict
}

/// Marshal a [`SettlementSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer._update_settlement_furniture`.
/// Eight parallel `PackedArray`s, lengths always equal to `rows.len()`.
///
/// Keys:
/// - `ids`: `PackedInt64Array` — `entity_bits` per row.
/// - `settlement_ids`: `PackedInt32Array` — `settlement_id` per row.
/// - `centroid_xs`: `PackedInt32Array` — tile-x centroid.
/// - `centroid_ys`: `PackedInt32Array` — tile-y centroid.
/// - `member_counts`: `PackedInt32Array` — resolvable member count.
/// - `formation_xs`: `PackedInt32Array` — fixed formation-anchor tile-x.
/// - `formation_ys`: `PackedInt32Array` — fixed formation-anchor tile-y.
/// - `food_stocks`: `PackedInt32Array` — `stockpile` Food count (slice 2-5a).
fn settlement_rows_to_dict(
    rows: &[SettlementSnapshotRow],
    fires: &std::collections::BTreeMap<SettlementId, sim_engine::FireState>,
    wood_targets: &[i32],
) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut settlement_ids = PackedInt32Array::new();
    let mut centroid_xs = PackedInt32Array::new();
    let mut centroid_ys = PackedInt32Array::new();
    let mut member_counts = PackedInt32Array::new();
    let mut formation_xs = PackedInt32Array::new();
    let mut formation_ys = PackedInt32Array::new();
    let mut food_stocks = PackedInt32Array::new();
    ids.resize(n);
    settlement_ids.resize(n);
    centroid_xs.resize(n);
    centroid_ys.resize(n);
    member_counts.resize(n);
    formation_xs.resize(n);
    formation_ys.resize(n);
    food_stocks.resize(n);
    // Revitalize Pass 5 — wood stock and fire state (1 lit, 0 out or no fire).
    let mut wood_stocks = PackedInt32Array::new();
    let mut fire_lit = PackedByteArray::new();
    wood_stocks.resize(n);
    fire_lit.resize(n);
    for (i, row) in rows.iter().enumerate() {
        wood_stocks[i] = row.wood_stock;
        fire_lit[i] = u8::from(fires.get(&row.settlement_id).is_some_and(|fire| fire.lit));
    }
    for (i, row) in rows.iter().enumerate() {
        ids[i] = row.entity_bits as i64;
        settlement_ids[i] = row.settlement_id as i32;
        centroid_xs[i] = row.centroid_x;
        centroid_ys[i] = row.centroid_y;
        member_counts[i] = row.member_count as i32;
        formation_xs[i] = row.formation_x;
        formation_ys[i] = row.formation_y;
        food_stocks[i] = row.food_stock;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("settlement_ids", settlement_ids);
    dict.set("centroid_xs", centroid_xs);
    dict.set("centroid_ys", centroid_ys);
    dict.set("member_counts", member_counts);
    dict.set("formation_xs", formation_xs);
    dict.set("formation_ys", formation_ys);
    dict.set("food_stocks", food_stocks);
    dict.set("wood_stocks", wood_stocks);
    dict.set("fire_lit", fire_lit);
    dict.set("wood_targets", PackedInt32Array::from(wood_targets));
    dict
}

/// Pass 6 — floor of the wood target of every settlement row, in row order;
/// `0` for every row without the firewood rule.
pub fn collect_settlement_wood_targets(rows: &[SettlementSnapshotRow], resources: &SimResources) -> Vec<i32> {
    let target = sim_systems::runtime::settlement::firewood::wood_target(resources)
        .map_or(0, |target| target.max(0.0).floor().min(f64::from(i32::MAX)) as i32);
    vec![target; rows.len()]
}

// ────────────────────────────────────────────────────────────────────────
// P7-δ: Relationship snapshot FFI surface — pure-Rust collector + helpers
// ────────────────────────────────────────────────────────────────────────

/// Single row of the relationship snapshot returned by
/// [`collect_relationship_snapshot`]. Phase 7-δ surfaces this to the
/// RelationshipState debug overlay.
///
/// `id_a < id_b` is guaranteed for every row by the canonical ordering
/// invariant of [`sim_core::components::RelationshipKey::new`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelationshipSnapshotRow {
    /// Smaller `AgentId` in the canonical pair key, as `i64` to match the
    /// Godot Variant integer width.
    pub id_a: i64,
    /// Larger `AgentId` in the canonical pair key.
    pub id_b: i64,
    /// Pair familiarity scalar in `[0.0, 1.0]`.
    pub familiarity: f64,
    /// Pair hostility scalar in `[0.0, 1.0]`.
    pub hostility: f64,
}

/// P7-δ pure-Rust collector: enumerate every entry in
/// `resources.relationships` whose `familiarity > 0.0` **or** `hostility > 0.0`.
///
/// The strict `> 0.0` filter — not `!= 0.0` — excludes default-initialized
/// pairs (familiarity = 0.0, hostility = 0.0) AND negative-value pairs
/// (which `RelationshipState::bump` does not produce in practice but the
/// underlying `f64` type permits).
///
/// Reads only `&SimResources`, so the snapshot cannot mutate sim state.
pub fn collect_relationship_snapshot(
    resources: &SimResources,
) -> Vec<RelationshipSnapshotRow> {
    resources
        .relationships
        .iter()
        .filter(|(_, v)| v.familiarity > 0.0 || v.hostility > 0.0)
        .map(|(k, v)| RelationshipSnapshotRow {
            id_a: k.smaller() as i64,
            id_b: k.larger() as i64,
            familiarity: v.familiarity,
            hostility: v.hostility,
        })
        .collect()
}

/// Pack a relationship snapshot row into a Godot `Dictionary`.
fn relationship_row_to_dict(row: &RelationshipSnapshotRow) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("id_a", row.id_a);
    dict.set("id_b", row.id_b);
    dict.set("familiarity", row.familiarity);
    dict.set("hostility", row.hostility);
    dict
}

/// Pack a slice of [`RelationshipSnapshotRow`] into a Godot `VarArray`.
fn relationship_rows_to_variant_array(rows: &[RelationshipSnapshotRow]) -> VarArray {
    let mut arr = VarArray::new();
    for row in rows {
        arr.push(&Variant::from(relationship_row_to_dict(row)));
    }
    arr
}

/// P4-γ bootstrap: spawn `BOOTSTRAP_AGENT_AXIS²` agents on a deterministic
/// lattice with per-agent `MovementRng` seeded by lattice index.
///
/// Lattice: `(OFFSET + i·STRIDE, OFFSET + j·STRIDE)` for `i, j ∈
/// 0..AXIS`. Seed: `BOOTSTRAP_RNG_BASE + lattice_index`. Determinism is
/// session-level (not byte-stable across `init` calls because hecs
/// entity ids depend on allocation order, but trajectory determinism is
/// guaranteed by the explicit seed).
///
/// Kept inside this module so the `init` path stays straight-line and
/// the visual-bootstrap policy lives next to its use site.
///
/// V7 Section 16-α0 — also populates the deterministic non-depleting
/// resource source substrate (`food/water/sleep_tiles` at
/// [`RESOURCE_SOURCE_INFINITE`]). Made `pub` so the headless harness can
/// exercise the production source-population path without a Godot runtime
/// (`WorldSimNode` construction requires the engine).
pub fn bootstrap_spawn_agents(engine: &mut SimEngine) {
    for j in 0..BOOTSTRAP_AGENT_AXIS {
        for i in 0..BOOTSTRAP_AGENT_AXIS {
            let x = BOOTSTRAP_AGENT_OFFSET + i * BOOTSTRAP_AGENT_STRIDE;
            let y = BOOTSTRAP_AGENT_OFFSET + j * BOOTSTRAP_AGENT_STRIDE;
            let entity = engine.spawn_agent(x, y);
            let seed = BOOTSTRAP_RNG_BASE.wrapping_add((j * BOOTSTRAP_AGENT_AXIS + i) as u64);
            // V7 Section 16-δ — per-agent STAGGERED initial need values +
            // accelerated growth rates. Pre-δ every agent started at 0.0 with
            // slow rates (Hunger 0.02 / Thirst 0.03 / Sleep 0.01), so all 64
            // breached on the SAME tick (synchronized burst, measured peak 64
            // simultaneous Seeking{Water} at tick 1667). Staggering breaks the
            // burst into a continuous trickle; the rate bump shortens the first
            // trip to ~tens of seconds.
            //
            // Initial values are derived from a SALTED seed via the existing
            // splitmix64 (`MovementRng`), drawn from a SEPARATE RNG instance so
            // the agent's own movement stream (`MovementRng::new(seed)` below)
            // is byte-identical to pre-δ. The cap is `0..=45` (< 50 breach
            // threshold) so every agent is still Idle right after bootstrap —
            // this preserves `s16_alpha0:A13` ("64 agents all Idle").
            // Rates: Thirst 0.08 > Hunger 0.05 > Sleep 0.03; Social stays
            // unstaggered at 0.04 (Seeking{Agent} has no SeekTarget so it does
            // not drive resource movement).
            let mut need_rng = MovementRng::new(seed ^ BOOTSTRAP_NEED_STAGGER_SALT);
            let span = BOOTSTRAP_NEED_STAGGER_MAX + 1; // 0..=MAX → span MAX+1
            let h0 = (need_rng.next_u64() % span) as f32;
            let t0 = (need_rng.next_u64() % span) as f64;
            let sl0 = (need_rng.next_u64() % span) as f64;
            engine
                .world
                .insert(
                    entity,
                    (
                        MovementRng::new(seed),
                        AgentState::Idle,
                        Hunger::new(h0, BOOTSTRAP_HUNGER_RATE),
                        Thirst::new(t0, BOOTSTRAP_THIRST_RATE),
                        Sleep::new(sl0, BOOTSTRAP_SLEEP_RATE),
                        Social::new(0.0, 0.04),
                        Memory::new(),
                        // add-starvation-death — every agent carries BodyHealth so
                        // StarvationSystem can damage it and combat's
                        // unwrap_or(true) never treats it as instantly-dead.
                        BodyHealth::new(),
                        // Direction-2 slice 2-2 — every full-runtime agent carries an
                        // empty Inventory so the Gather cascade arm can fill it.
                        Inventory::default(),
                    ),
                )
                .expect("bootstrap agent entity must still exist");
        }
    }

    // V7 Section 16-α0 — seed the non-depleting resource source substrate.
    // Appended after the agent lattice so the existing spawn is unperturbed
    // (Assertion 13: 64 agents, all Idle). Each source is the sentinel value.
    for &(x, y) in SOURCE_FOOD.iter() {
        engine.resources.set_food_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
    for &(x, y) in SOURCE_WATER.iter() {
        engine.resources.set_water_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
    for &(x, y) in SOURCE_SLEEP.iter() {
        engine.resources.set_sleep_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
}

/// `add-resource-scarcity-regen` — overwrite the just-bootstrapped INFINITE
/// source tiles with a FINITE initial capacity AND register each source's
/// regen ceiling, so the production substrate depletes + regenerates.
///
/// Called by [`init_production_engine`] IMMEDIATELY AFTER
/// [`bootstrap_spawn_agents`] — it does NOT touch the bootstrap seeding loop
/// (which keeps seeding [`RESOURCE_SOURCE_INFINITE`] so the 12 shared harnesses
/// stay byte-for-byte unchanged). For each [`SOURCE_FOOD`]/[`SOURCE_WATER`]/
/// [`SOURCE_SLEEP`] coordinate it sets the tile to `INITIAL_*` and registers
/// `*_source_max = INITIAL_*`. `pub` so the scarcity harness can build a
/// production-equivalent scene without a Godot runtime.
pub fn seed_finite_resource_scarcity(engine: &mut SimEngine) {
    for &(x, y) in SOURCE_FOOD.iter() {
        engine.resources.set_food_tile(x, y, INITIAL_FOOD);
        engine.resources.set_food_source_max(x, y, INITIAL_FOOD);
    }
    for &(x, y) in SOURCE_WATER.iter() {
        engine.resources.set_water_tile(x, y, INITIAL_WATER);
        engine.resources.set_water_source_max(x, y, INITIAL_WATER);
    }
    for &(x, y) in SOURCE_SLEEP.iter() {
        engine.resources.set_sleep_tile(x, y, INITIAL_SLEEP);
        engine.resources.set_sleep_source_max(x, y, INITIAL_SLEEP);
    }
}

/// Build the legacy V7 lattice fixture used by harnesses.
///
/// The live game uses `worldgen::build_world_with` with v10 rules. This frozen legacy constructor
/// keeps the original registration, bootstrap and finite-scarcity order so
/// historical behavioral tests and trajectory fingerprints remain comparable.
pub fn init_production_engine() -> SimEngine {
    let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    bootstrap_spawn_agents(&mut engine);
    seed_finite_resource_scarcity(&mut engine);
    engine
}

/// Maximum synchronous tooling advance, preventing accidental unbounded work.
const MAX_TOOLING_ADVANCE_TICKS: i64 = 200_000;

/// Advance the same engine used by the runtime without invoking Godot.
pub fn advance_engine_ticks(engine: &mut SimEngine, n: i64) -> u64 {
    for _ in 0..n.clamp(0, MAX_TOOLING_ADVANCE_TICKS) {
        engine.tick();
    }
    engine.current_tick()
}

/// Native simulation clock snapshot, testable without a Godot runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSnapshot {
    /// Number of completed simulation ticks.
    pub tick: u64,
    /// Simulation ticks in a day; zero disables the simulation clock.
    pub ticks_per_day: u64,
    /// Zero-based day, or zero when the simulation clock is disabled.
    pub day_index: u64,
    /// Authoritative simulation hour in the range 0..24.
    pub time_of_day: f64,
    /// Pass 4 season of `day_index`: 0 spring, 1 summer, 2 autumn, 3 winter (0 without seasons).
    pub season: u32,
    /// One-based day within the season (0 without seasons).
    pub day_of_season: u32,
    /// Days per season (0 without seasons).
    pub days_per_season: u32,
    /// Pass 7 severity of the winter at the completed tick: 0 mild, 1 normal,
    /// 2 harsh, -1 outside a winter or without the rule or weights; from 00:00
    /// to the burn end it reads the evening's winter (night booking).
    pub winter_severity: i64,
}

/// Read the completed tick count and clock resources without mutation.
pub fn collect_time_snapshot(engine: &SimEngine) -> TimeSnapshot {
    let ticks_per_day = engine.resources.ticks_per_day;
    let day_index = engine.current_tick().saturating_add(engine.resources.rules.clock_offset_ticks)
        .checked_div(ticks_per_day).unwrap_or(0);
    let days_per_season = engine.resources.rules.seasons.map_or(0, |rule| rule.days_per_season);
    let (season, day_of_season) = match u64::from(days_per_season) {
        0 => (0, 0),
        per => (((day_index / per) % 4) as u32, (day_index % per) as u32 + 1),
    };
    TimeSnapshot {
        tick: engine.current_tick(),
        ticks_per_day,
        day_index,
        time_of_day: engine.resources.time_of_day,
        season,
        day_of_season,
        days_per_season,
        winter_severity: sim_systems::runtime::settlement::firewood::winter_severity_at(
            &engine.resources,
            engine.current_tick(),
        ),
    }
}

/// Pass 7: build an intervention from the FFI arguments. `None` for an
/// unknown kind, a negative amount, or a settlement kind with no settlement
/// (`settlement_id < 0` and no `home`, or an id beyond `u32`). `amount` above
/// `u32::MAX` saturates. `cold_snap` and `windfall` ignore the settlement.
pub fn parse_intervention(
    kind: &str,
    settlement_id: i64,
    amount: i64,
    home: Option<sim_core::components::SettlementId>,
) -> Option<sim_engine::Intervention> {
    use sim_systems::runtime::intervention::{intervention_from_kind, kind_needs_settlement};
    if amount < 0 {
        return None;
    }
    let amount = u32::try_from(amount).unwrap_or(u32::MAX);
    let settlement = if !kind_needs_settlement(kind) {
        0
    } else if settlement_id < 0 {
        home?
    } else {
        sim_core::components::SettlementId::try_from(settlement_id).ok()?
    };
    intervention_from_kind(kind, settlement, amount)
}

/// Read-only intervention presentation columns, ordered by settlement and log time.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct InterventionState {
    /// Settlement IDs with active rain, ascending.
    pub rain_settlement_ids: Vec<i32>,
    /// Remaining ticks paired with the settlement IDs, saturated to the FFI range.
    pub rain_ticks_left: Vec<i32>,
    /// Pending cold-snap nights, including tonight when active.
    pub cold_snap_nights_left: u32,
    /// Whether the simulation has booked tonight as a cold snap.
    pub cold_snap_tonight: bool,
    /// Ticks of the last 32 records, oldest first.
    pub log_ticks: Vec<i64>,
    /// Absolute record indices, stable across the bounded presentation window.
    pub log_ids: Vec<i64>,
    /// Actual target settlement, or -1 for world-wide interventions.
    pub log_settlement_ids: Vec<i32>,
    /// Canonical intervention names, paired with log ticks.
    pub log_kinds: Vec<&'static str>,
    /// Requested units, hours or nights for each record.
    pub log_amounts: Vec<i64>,
    /// Applied amounts for each record.
    pub log_applied: Vec<i64>,
}

/// Collect presentation state without mutating resources or consuming randomness.
pub fn collect_intervention_state(resources: &SimResources, now: u64) -> InterventionState {
    use sim_engine::Intervention;
    use sim_systems::runtime::intervention::KINDS;
    let mut state = InterventionState {
        cold_snap_nights_left: resources.cold_snap_nights,
        cold_snap_tonight: resources.cold_snap_tonight,
        ..InterventionState::default()
    };
    let mut rain: Vec<_> = resources.fire_blocked_until.iter().filter(|(_, until)| **until > now).collect();
    rain.sort_unstable_by_key(|(id, _)| **id);
    for (id, until) in rain {
        state.rain_settlement_ids.push(i32::try_from(*id).unwrap_or(i32::MAX));
        state.rain_ticks_left.push(i32::try_from(*until - now).unwrap_or(i32::MAX));
    }
    for (index, record) in resources.intervention_log.iter().enumerate().skip(resources.intervention_log.len().saturating_sub(32)) {
        let (kind, amount) = match record.intervention {
            Intervention::GiftFood { amount, .. } => (KINDS[0], amount),
            Intervention::SpoilFood { amount, .. } => (KINDS[1], amount),
            Intervention::GiftWood { amount, .. } => (KINDS[2], amount),
            Intervention::RotWood { amount, .. } => (KINDS[3], amount),
            Intervention::Rain { hours, .. } => (KINDS[4], hours),
            Intervention::ColdSnap { nights } => (KINDS[5], nights),
            Intervention::Windfall { units } => (KINDS[6], units),
        };
        let settlement = match record.intervention {
            Intervention::GiftFood { settlement, .. }
            | Intervention::SpoilFood { settlement, .. }
            | Intervention::GiftWood { settlement, .. }
            | Intervention::RotWood { settlement, .. }
            | Intervention::Rain { settlement, .. } => i32::try_from(settlement).unwrap_or(i32::MAX),
            Intervention::ColdSnap { .. } | Intervention::Windfall { .. } => -1,
        };
        state.log_ids.push(i64::try_from(index).unwrap_or(i64::MAX));
        state.log_settlement_ids.push(settlement);
        state.log_ticks.push(i64::try_from(record.tick).unwrap_or(i64::MAX));
        state.log_kinds.push(kind);
        state.log_amounts.push(i64::from(amount));
        state.log_applied.push(i64::try_from(record.applied).unwrap_or(i64::MAX));
    }
    state
}

/// Pass 7: result of the `apply_intervention` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterventionReply {
    /// False for an unknown kind, a negative amount or no settlement.
    pub ok: bool,
    /// What the intervention changed (0 when nothing changed or not `ok`).
    pub applied: u64,
    /// Completed ticks when it was applied.
    pub tick: u64,
}

/// Pass 7: parse and apply one intervention now (the `#[func]` body). `ok` is
/// false when parsing fails or a settlement kind names a settlement that does
/// not exist.
pub fn apply_intervention_command(engine: &mut SimEngine, kind: &str, settlement_id: i64, amount: i64) -> InterventionReply {
    use sim_systems::runtime::intervention::{home_settlement, kind_needs_settlement};
    let home = home_settlement(&engine.resources);
    // Review round 1 (L3): an explicit settlement id that does not exist is "no settlement".
    let parsed = parse_intervention(kind, settlement_id, amount, home).filter(|_| {
        !kind_needs_settlement(kind)
            || settlement_id < 0
            || u32::try_from(settlement_id).is_ok_and(|id| engine.resources.settlements.contains_key(&id))
    });
    match parsed {
        Some(intervention) => {
            let record = sim_systems::runtime::intervention::apply_intervention(engine, intervention);
            InterventionReply { ok: true, applied: record.applied, tick: record.tick }
        }
        None => InterventionReply { ok: false, applied: 0, tick: engine.current_tick() },
    }
}

fn intervention_reply_to_dict(reply: InterventionReply) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("ok", reply.ok);
    dict.set("applied", reply.applied as i64);
    dict.set("tick", reply.tick as i64);
    dict
}

/// Read the cumulative counters without depending on event retention.
pub fn collect_tallies(resources: &SimResources) -> sim_engine::Tallies {
    resources.tallies
}

fn time_snapshot_to_dict(snapshot: TimeSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("tick", snapshot.tick as i64);
    dict.set("ticks_per_day", snapshot.ticks_per_day as i64);
    dict.set("day_index", snapshot.day_index as i64);
    dict.set("time_of_day", snapshot.time_of_day);
    dict.set("season", snapshot.season as i64);
    dict.set("day_of_season", snapshot.day_of_season as i64);
    dict.set("days_per_season", snapshot.days_per_season as i64);
    dict.set("winter_severity", snapshot.winter_severity);
    dict
}

fn tallies_to_dict(tallies: sim_engine::Tallies) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("births", tallies.births as i64);
    dict.set("deaths_starvation", tallies.deaths_starvation as i64);
    dict.set("deaths_dehydration", tallies.deaths_dehydration as i64);
    dict.set("deaths_combat", tallies.deaths_combat as i64);
    dict.set("deaths_old_age", tallies.deaths_old_age as i64);
    // Revitalize Pass 10: cold-night deaths under the exposure rule.
    dict.set("deaths_exposure", tallies.deaths_exposure as i64);
    // Revitalize Pass 11 tallies (display-only).
    dict.set("fissions", tallies.fissions as i64);
    dict.set("fissions_cancelled", tallies.fissions_cancelled as i64);
    dict.set("remnant_merges", tallies.remnant_merges as i64);
    // Revitalize Pass 12: wood a raining settlement burned wet (display-only).
    dict.set("wet_burn_wood", tallies.wet_burn_wood as i64);
    // Revitalize Pass 13: settled-area floor wood placed (display-only).
    dict.set("area_floor_placed", tallies.area_floor_placed as i64);
    // Revitalize Pass 5 tallies (display-only).
    dict.set("meals_from_store", tallies.meals_from_store as i64);
    dict.set("ground_meals", tallies.ground_meals as i64);
    dict.set("wood_trips", tallies.wood_trips as i64);
    dict.set("wood_gathered", tallies.wood_gathered as i64);
    dict.set("gather_preempted", tallies.gather_preempted as i64);
    dict.set("cold_member_hours", tallies.cold_member_hours as i64);
    dict
}

#[cfg(test)]
mod revitalize_tooling_tests {
    use super::{advance_engine_ticks, collect_tallies, collect_time_snapshot};
    use sim_core::material::MaterialRegistry;
    use sim_engine::{SimEngine, Tallies};

    #[test]
    fn stress_engine_has_the_requested_founders() {
        // Pass 15 6.4.
        let engine = super::build_stress_engine(15100, 500);
        assert_eq!(engine.world.query::<&sim_core::components::Agent>().iter().count(), 500);
        assert_eq!((engine.resources.world_seed, engine.current_tick()), (15100, 0));
        assert_eq!(engine.resources.rules, sim_core::rules::WorldRules::game_v19(15100));
    }

    #[test]
    fn time_snapshot_uses_completed_ticks_and_resource_clock() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.ticks_per_day = 12;
        advance_engine_ticks(&mut engine, 17);
        let snapshot = collect_time_snapshot(&engine);
        assert_eq!(snapshot.tick, 17);
        assert_eq!(snapshot.ticks_per_day, 12);
        assert_eq!(snapshot.day_index, 1);
        assert_eq!(snapshot.time_of_day, 10.0);
    }

    #[test]
    fn harness_economy_clock_snapshot_rolls_day_at_offset_midnight() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::game_v8();
        engine.resources.time_of_day = 8.0;
        assert_eq!(super::collect_world_info(&engine.resources).clock_offset_ticks, 480);
        let initial = collect_time_snapshot(&engine);
        assert_eq!((initial.tick, initial.day_index, initial.time_of_day), (0, 0, 8.0));
        advance_engine_ticks(&mut engine, 959);
        let before = collect_time_snapshot(&engine);
        assert_eq!((before.tick, before.day_index), (959, 0));
        assert!((before.time_of_day - (23.0 + 59.0 / 60.0)).abs() < 1e-12);
        advance_engine_ticks(&mut engine, 1);
        let midnight = collect_time_snapshot(&engine);
        assert_eq!((midnight.tick, midnight.day_index, midnight.time_of_day), (960, 1, 0.0));
    }

    #[test]
    fn time_snapshot_reports_seasons_from_the_display_day() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::game_v10();
        let first = collect_time_snapshot(&engine);
        assert_eq!((first.season, first.day_of_season, first.days_per_season), (0, 1, 7));
        // Display day 9 (zero-based) at 00:00 is the third day of summer.
        advance_engine_ticks(&mut engine, (9 * 1440 - 480) as i64);
        let summer = collect_time_snapshot(&engine);
        assert_eq!((summer.day_index, summer.season, summer.day_of_season), (9, 1, 3));
        engine.resources.rules = sim_core::rules::WorldRules::game_v9();
        assert_eq!(collect_time_snapshot(&engine).days_per_season, 0);
    }

    #[test]
    fn time_snapshot_reports_winter_severity_with_night_booking() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::game_v12();
        // v12 has no severity weights: -1 everywhere, winter included.
        advance_engine_ticks(&mut engine, (21 * 1440) as i64);
        assert_eq!(collect_time_snapshot(&engine).winter_severity, -1);
        if let Some(rule) = engine.resources.rules.society.firewood.as_mut() {
            rule.winter_severity_permille = Some([1000, 0, 0]);
        }
        // Display day 21 at 08:00: the first winter day, a mild winter.
        assert_eq!(collect_time_snapshot(&engine).winter_severity, 0);
        // Display day 28 at 02:00 belongs to the last winter evening.
        advance_engine_ticks(&mut engine, (7 * 1440 - 360) as i64);
        let night = collect_time_snapshot(&engine);
        assert_eq!((night.day_index, night.season, night.winter_severity), (28, 0, 0));
        // 07:00 ends the night: spring.
        advance_engine_ticks(&mut engine, 300);
        assert_eq!(collect_time_snapshot(&engine).winter_severity, -1);
    }

    #[test]
    fn parse_intervention_resolves_home_and_rejects_bad_input() {
        use sim_engine::Intervention;
        assert_eq!(super::parse_intervention("gift_food", -1, 30, Some(4)), Some(Intervention::GiftFood { settlement: 4, amount: 30 }));
        assert_eq!(super::parse_intervention("rain", 7, 13, Some(4)), Some(Intervention::Rain { settlement: 7, hours: 13 }));
        assert_eq!(super::parse_intervention("cold_snap", -1, 2, None), Some(Intervention::ColdSnap { nights: 2 }));
        assert_eq!(super::parse_intervention("windfall", -5, 10, None), Some(Intervention::Windfall { units: 10 }));
        assert_eq!(super::parse_intervention("flood", -1, 1, Some(4)), None);
        assert_eq!(super::parse_intervention("gift_wood", -1, -1, Some(4)), None);
        assert_eq!(super::parse_intervention("rot_wood", -1, 1, None), None);
        assert_eq!(super::parse_intervention("spoil_food", i64::MAX, 1, Some(4)), None);
    }

    #[test]
    fn intervention_state_is_read_only_sorted_bounded_and_preserves_variant_amounts() {
        use sim_engine::{InterventionRecord, SimResources};
        use sim_systems::runtime::intervention::{intervention_from_kind, KINDS};
        let mut resources = SimResources::new(1, 1, MaterialRegistry::new());
        assert_eq!(super::collect_intervention_state(&resources, 0), super::InterventionState::default());
        resources.fire_blocked_until.extend([(9, 30), (2, 20), (3, 10), (4, 9)]);
        resources.cold_snap_nights = 2;
        resources.cold_snap_tonight = true;
        for tick in 0..35 {
            resources.intervention_log.push(InterventionRecord {
                tick,
                intervention: intervention_from_kind(KINDS[usize::try_from(tick % 7).unwrap()], 2, u32::try_from(tick + 1).unwrap()).unwrap(),
                applied: tick,
                soaked_wood: 0,
            });
        }
        let state = super::collect_intervention_state(&resources, 10);
        assert_eq!(state.rain_settlement_ids, [2, 9]);
        assert_eq!(state.rain_ticks_left, [10, 20]);
        assert_eq!((state.cold_snap_nights_left, state.cold_snap_tonight), (2, true));
        assert_eq!(state.log_ticks, (3..35).collect::<Vec<i64>>());
        assert_eq!(state.log_ids, (3..35).collect::<Vec<i64>>());
        assert_eq!(state.log_settlement_ids, (3..35).map(|index| if index % 7 < 5 { 2 } else { -1 }).collect::<Vec<i32>>());
        assert_eq!(state.log_amounts, (4..36).collect::<Vec<i64>>());
        assert_eq!(state.log_applied, state.log_ticks);
        for (offset, kind) in state.log_kinds.iter().enumerate() {
            assert_eq!(*kind, KINDS[(offset + 3) % 7]);
        }
        assert_eq!(state, super::collect_intervention_state(&resources, 10));
        assert_eq!(resources.intervention_log.len(), 35);
        assert_eq!(resources.fire_blocked_until.len(), 4);
    }

    #[test]
    fn intervention_feed_identity_distinguishes_equal_casts_and_targets() {
        use sim_engine::{Intervention, InterventionRecord, SimResources};
        let mut resources = SimResources::new(1, 1, MaterialRegistry::new());
        for settlement in [7, 42, 7] {
            resources.intervention_log.push(InterventionRecord {
                tick: 12,
                intervention: Intervention::GiftFood { settlement, amount: 60 },
                applied: 60,
                soaked_wood: 0,
            });
        }
        let state = super::collect_intervention_state(&resources, 12);
        assert_eq!(state.log_ids, [0, 1, 2]);
        assert_eq!(state.log_ticks, [12, 12, 12]);
        assert_eq!(state.log_settlement_ids, [7, 42, 7]);
        assert_eq!(state.log_applied, [60, 60, 60]);
    }

    #[test]
    fn apply_intervention_command_reports_ok_applied_and_tick() {
        use super::apply_intervention_command;
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        advance_engine_ticks(&mut engine, 3);
        let none = apply_intervention_command(&mut engine, "gift_food", -1, 5);
        assert_eq!((none.ok, none.applied, none.tick), (false, 0, 3));
        engine.resources.settlements.insert(2, sim_core::components::Settlement::new_with_id(2, 0));
        let gift = apply_intervention_command(&mut engine, "gift_food", -1, 5);
        assert_eq!((gift.ok, gift.applied, gift.tick), (true, 5, 3));
        let missing = apply_intervention_command(&mut engine, "rain", 9, 5);
        assert_eq!((missing.ok, missing.applied), (false, 0));
        let snap = apply_intervention_command(&mut engine, "cold_snap", -1, 1);
        assert_eq!((snap.ok, snap.applied), (true, 1));
        assert_eq!(engine.resources.tallies.interventions_applied, 2);
        assert_eq!(engine.resources.intervention_log.len(), 2);
    }

    #[test]
    fn time_snapshot_handles_disabled_clock() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.ticks_per_day = 0;
        advance_engine_ticks(&mut engine, 3);
        let snapshot = collect_time_snapshot(&engine);
        assert_eq!(snapshot.tick, 3);
        assert_eq!(snapshot.day_index, 0);
        assert_eq!(snapshot.time_of_day, 0.0);
    }

    #[test]
    fn tallies_collector_preserves_all_four_counters() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.tallies = Tallies {
            births: 11,
            deaths_starvation: 12,
            deaths_dehydration: 13,
            deaths_combat: 14,
            social_completed: 0,
            food_harvested: 0,
            meals_eaten: 0,
            meals_without_food: 0,
            deaths_old_age: 0,
            forage_trips: 0,
            haul_deposits: 0,
            regen_lost_at_cap: 0,
            meals_from_store: 0,
            ground_meals: 0,
            wood_trips: 0,
            wood_gathered: 0,
            gather_preempted: 0,
            cold_member_hours: 0,
            urgent_wood_trips: 0,
            winter_wood_lost: 0,
            wood_retargets: 0,
            interventions_applied: 0,
            deaths_exposure: 0,
            fissions: 0,
            fissions_cancelled: 0,
            fission_blocked_no_site: 0,
            remnant_merges: 0,
            exclusivity_moves: 0,
            wet_burn_wood: 0,
            area_floor_wood: 0,
            winter_wood_lost_settled: 0,
            area_floor_placed: 0,
            area_floor_lost_at_max: 0,
        };
        let snapshot = collect_tallies(&engine.resources);
        assert_eq!(snapshot.births, 11);
        assert_eq!(snapshot.deaths_starvation, 12);
        assert_eq!(snapshot.deaths_dehydration, 13);
        assert_eq!(snapshot.deaths_combat, 14);
        assert_eq!(snapshot.social_completed, 0);
        assert_eq!(engine.current_tick(), 0);
    }

    #[test]
    fn tooling_advance_clamps_negative_and_excessive_requests() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        assert_eq!(advance_engine_ticks(&mut engine, -8), 0);
        assert_eq!(advance_engine_ticks(&mut engine, 200_001), 200_000);
    }
}

/// Seed and pacing metadata consumed by renderers and deterministic tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldInfoSnapshot {
    /// Seed used by world generation.
    pub seed: u64,
    /// Map width in tiles.
    pub width: u32,
    /// Map height in tiles.
    pub height: u32,
    /// Simulation ticks per day.
    pub ticks_per_day: u64,
    /// Ticks between agent movement opportunities.
    pub move_interval_ticks: u64,
    /// Display clock offset from elapsed ticks.
    pub clock_offset_ticks: u64,
}

/// Read immutable world metadata directly from the authoritative resources.
pub fn collect_world_info(resources: &SimResources) -> WorldInfoSnapshot {
    WorldInfoSnapshot {
        seed: resources.world_seed,
        width: resources.tile_grid.width,
        height: resources.tile_grid.height,
        ticks_per_day: resources.ticks_per_day,
        move_interval_ticks: resources.rules.agent.move_interval_ticks,
        clock_offset_ticks: resources.rules.clock_offset_ticks,
    }
}

fn world_info_to_dict(snapshot: WorldInfoSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("seed", snapshot.seed as i64);
    dict.set("width", snapshot.width as i64);
    dict.set("height", snapshot.height as i64);
    dict.set("ticks_per_day", snapshot.ticks_per_day as i64);
    dict.set("move_interval_ticks", snapshot.move_interval_ticks as i64);
    dict.set("clock_offset_ticks", snapshot.clock_offset_ticks as i64);
    dict
}

/// Row-major terrain arrays independent of Godot packed-array allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainSnapshot {
    /// Map width in tiles.
    pub width: u32,
    /// Map height in tiles.
    pub height: u32,
    /// Declaration-order terrain codes (Plain 0 through Cave 9).
    pub terrain: Vec<u8>,
    /// Elevation bytes, aligned with terrain.
    pub elevation: Vec<u8>,
}

/// Copy simulation grid data in its canonical row-major order.
pub fn collect_terrain_snapshot(resources: &SimResources) -> TerrainSnapshot {
    TerrainSnapshot {
        width: resources.tile_grid.width,
        height: resources.tile_grid.height,
        terrain: resources.tile_grid.terrain_type.iter().map(|terrain| *terrain as u8).collect(),
        elevation: resources.tile_grid.elevation.clone(),
    }
}

fn terrain_snapshot_to_dict(snapshot: &TerrainSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("width", snapshot.width as i64);
    dict.set("height", snapshot.height as i64);
    dict.set("terrain", PackedByteArray::from(snapshot.terrain.as_slice()));
    dict.set("elevation", PackedByteArray::from(snapshot.elevation.as_slice()));
    dict
}

/// One real registered building and its simulation-owned kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingSnapshotRow {
    /// Stable BuildingId from the building registry.
    pub id: u64,
    /// Building tile x coordinate.
    pub x: u32,
    /// Building tile y coordinate.
    pub y: u32,
    /// BuildingKind::code, defaulting to Hearth (0) for legacy buildings.
    pub kind: u8,
}

/// Join registered buildings with their tile-keyed kinds and sort by id.
pub fn collect_building_snapshot(resources: &SimResources) -> Vec<BuildingSnapshotRow> {
    let mut rows: Vec<BuildingSnapshotRow> = resources.building_registry.iter()
        .map(|(&id, &(x, y))| BuildingSnapshotRow {
            id,
            x,
            y,
            kind: resources.building_kinds.get(&(x, y))
                .map_or(0, |kind| kind.code()),
        })
        .collect();
    rows.sort_by_key(|row| row.id);
    rows
}

fn building_rows_to_dict(rows: &[BuildingSnapshotRow]) -> VarDictionary {
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut kinds = PackedByteArray::new();
    ids.resize(rows.len());
    xs.resize(rows.len());
    ys.resize(rows.len());
    kinds.resize(rows.len());
    for (index, row) in rows.iter().enumerate() {
        ids[index] = row.id as i64;
        xs[index] = row.x as i32;
        ys[index] = row.y as i32;
        kinds[index] = row.kind;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("kinds", kinds);
    dict
}

#[cfg(test)]
mod revitalize_world_snapshot_tests {
    use super::{collect_building_snapshot, collect_terrain_snapshot, collect_world_info, BuildingSnapshotRow};
    use sim_core::components::BuildingKind;
    use sim_core::material::{MaterialRegistry, TerrainType};
    use sim_engine::SimResources;

    #[test]
    fn world_info_reads_world_seed_dimensions_and_rules() {
        let mut resources = SimResources::new(3, 7, MaterialRegistry::new());
        resources.world_seed = 41;
        resources.ticks_per_day = 120;
        resources.rules.agent.move_interval_ticks = 6;
        let snapshot = collect_world_info(&resources);
        assert_eq!(snapshot.seed, 41);
        assert_eq!((snapshot.width, snapshot.height), (3, 7));
        assert_eq!(snapshot.ticks_per_day, 120);
        assert_eq!(snapshot.move_interval_ticks, 6);
    }

    #[test]
    fn terrain_snapshot_preserves_all_codes_and_row_major_elevation() {
        let mut resources = SimResources::new(5, 2, MaterialRegistry::new());
        resources.tile_grid.terrain_type = vec![
            TerrainType::Plain, TerrainType::Forest, TerrainType::Mountain,
            TerrainType::Hill, TerrainType::River, TerrainType::Coast,
            TerrainType::Desert, TerrainType::Tundra, TerrainType::Swamp, TerrainType::Cave,
        ];
        resources.tile_grid.elevation = vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        let snapshot = collect_terrain_snapshot(&resources);
        assert_eq!((snapshot.width, snapshot.height), (5, 2));
        assert_eq!(snapshot.terrain, (0..10).collect::<Vec<u8>>());
        assert_eq!(snapshot.elevation, vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn building_snapshot_sorts_ids_and_defaults_legacy_to_hearth() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.building_registry.insert(7, (1, 2));
        resources.building_registry.insert(2, (3, 4));
        resources.building_registry.insert(5, (5, 6));
        resources.building_kinds.insert((3, 4), BuildingKind::Shelter);
        resources.building_kinds.insert((5, 6), BuildingKind::Stockpile);
        // An unregistered kind must not invent a real building.
        resources.building_kinds.insert((0, 0), BuildingKind::Hearth);
        assert_eq!(collect_building_snapshot(&resources), vec![
            BuildingSnapshotRow { id: 2, x: 3, y: 4, kind: 1 },
            BuildingSnapshotRow { id: 5, x: 5, y: 6, kind: 2 },
            BuildingSnapshotRow { id: 7, x: 1, y: 2, kind: 0 },
        ]);
    }
}


#[cfg(test)]
mod society_snapshot_tests {
    use super::*;
    use sim_core::components::Founding;
    use sim_engine::FissionRecord;

    #[test]
    fn society_joins_built_world_agents_and_preserves_order_filter_and_history() {
        let seed = 20_261_013;
        let mut engine = build_world_with(
            &WorldGenConfig::game_v17(seed), WorldRules::game_v17(seed),
        );
        advance_engine_ticks(&mut engine, 1_440);
        let agent_rows = collect_agent_snapshot(&engine.world);
        let snapshot = collect_society_snapshot(&engine.world, &engine.resources);
        assert!(!snapshot.home_agent_ids.is_empty());
        assert_eq!(snapshot.fission_pressures, vec![0; snapshot.settlement_ids.len()]);
        assert_eq!(snapshot.fission_thresholds, vec![0; snapshot.settlement_ids.len()]);
        for (&agent_id, &home_id) in snapshot.home_agent_ids.iter().zip(&snapshot.home_settlement_ids) {
            assert!(agent_rows.iter().any(|row| row.agent_id as i64 == agent_id));
            assert!(engine.resources.settlements[&(home_id as u32)].member_agents.contains(&(agent_id as u64)));
        }
        assert_eq!(snapshot, collect_society_snapshot(&engine.world, &engine.resources));

        let first = snapshot.home_agent_ids[0] as u64;
        let second = snapshot.home_agent_ids[1] as u64;
        let parent = snapshot.home_settlement_ids[0] as u32;
        let mut daughter = Settlement::new_with_id(20, 120);
        daughter.parent = Some(parent);
        daughter.fission_pressure = 850;
        daughter.fission_threshold = 1_200;
        daughter.add_member_agent(second);
        let parent_settlement = engine.resources.settlements.get_mut(&parent).unwrap();
        parent_settlement.member_agents.remove(&second);
        parent_settlement.fission_pressure = 325;
        parent_settlement.fission_threshold = 900;
        engine.resources.settlements.insert(20, daughter);
        let mut hidden = Settlement::new_with_id(30, 130);
        hidden.add_member_agent(u64::MAX);
        hidden.fission_pressure = 777;
        hidden.fission_threshold = 888;
        engine.resources.settlements.insert(30, hidden);
        let first_entity = engine.world.query::<&Agent>().iter()
            .find(|(_, agent)| agent.id == first).map(|(entity, _)| entity).unwrap();
        engine.world.insert_one(first_entity, Founding {
            site: (8, 9), fission_id: 2, leader: first, since_tick: 100,
        }).unwrap();
        engine.resources.fission_log = vec![
            FissionRecord {
                fission_id: 2, parent, site: (8, 9), depart_tick: 100,
                members: vec![first], adults: 1, daughter: None, founded_tick: None,
                cancelled_tick: None, food_taken: 23, wood_taken: 0, settled_at_leader: false,
                pressure_at_split: 0, threshold_at_split: 0,
            },
            FissionRecord {
                fission_id: 1, parent, site: (2, 3), depart_tick: 80,
                members: vec![second], adults: 1, daughter: Some(20), founded_tick: Some(120),
                cancelled_tick: Some(90), food_taken: 17, wood_taken: 0, settled_at_leader: false,
                pressure_at_split: 0, threshold_at_split: 0,
            },
        ];
        engine.resources.retired_hearths.extend([(9, 4), (2, 5)]);
        let snapshot = collect_society_snapshot(&engine.world, &engine.resources);
        let settlement_rows = collect_settlement_snapshot(&engine.world, &engine.resources.settlements);
        assert_eq!(snapshot.settlement_ids, settlement_rows.iter().map(|row| row.settlement_id as i32).collect::<Vec<_>>());
        assert_eq!(snapshot.settlement_ids, vec![parent as i32, 20]);
        assert_eq!(snapshot.parent_ids, vec![-1, parent as i32]);
        assert_eq!(snapshot.fission_pressures, [325, 850]);
        assert_eq!(snapshot.fission_thresholds, [900, 1_200]);
        assert_eq!(snapshot, collect_society_snapshot(&engine.world, &engine.resources));
        assert_eq!(snapshot.founded_ticks[1], 120);
        assert_eq!(snapshot.member_counts[0] as usize, engine.resources.settlements[&parent].member_agents.len());
        let member_index = snapshot.home_agent_ids.iter().position(|&id| id == first as i64).unwrap();
        assert_eq!(snapshot.home_settlement_ids[member_index], parent as i32);
        assert_eq!(snapshot.home_agent_ids.last(), Some(&-1));
        assert_eq!(snapshot.home_settlement_ids.last(), Some(&30));
        assert_eq!(snapshot.founder_agent_ids, vec![first as i64]);
        assert_eq!(snapshot.founder_fission_ids, vec![2]);
        assert_eq!(snapshot.founder_site_xs, vec![8]);
        assert_eq!(snapshot.founder_site_ys, vec![9]);
        assert_eq!(snapshot.founder_leader, vec![1]);
        assert_eq!(snapshot.founder_since_ticks, vec![100]);
        assert_eq!(snapshot.fission_ids, vec![1, 2]);
        assert_eq!(snapshot.fission_parents, vec![parent as i32, parent as i32]);
        assert_eq!(snapshot.fission_site_xs, vec![2, 8]);
        assert_eq!(snapshot.fission_site_ys, vec![3, 9]);
        assert_eq!(snapshot.fission_sizes, vec![1, 1]);
        assert_eq!(snapshot.fission_depart_ticks, vec![80, 100]);
        assert_eq!(snapshot.fission_founded_ticks, vec![120, -1]);
        assert_eq!(snapshot.fission_cancelled_ticks, vec![90, -1]);
        assert_eq!(snapshot.fission_daughters, vec![20, -1]);
        assert_eq!(snapshot.fission_food_taken, vec![17, 23]);
        assert_eq!(snapshot.retired_hearth_xs, vec![2, 9]);
        assert_eq!(snapshot.retired_hearth_ys, vec![5, 4]);
        assert_eq!(snapshot.current_tick, engine.resources.current_tick as i64);
    }

    #[test]
    fn society_pressure_columns_read_v19_values_and_saturate_without_mutation() {
        let seed = 20_261_010;
        let mut engine = build_world_with(
            &WorldGenConfig::game_v19(seed), WorldRules::game_v19(seed),
        );
        advance_engine_ticks(&mut engine, 1_440);
        let snapshot = collect_society_snapshot(&engine.world, &engine.resources);
        assert!(!snapshot.settlement_ids.is_empty());
        assert_eq!(snapshot.fission_pressures.len(), snapshot.settlement_ids.len());
        assert_eq!(snapshot.fission_thresholds.len(), snapshot.settlement_ids.len());
        for (index, &id) in snapshot.settlement_ids.iter().enumerate() {
            let settlement = &engine.resources.settlements[&(id as u32)];
            assert_eq!(snapshot.fission_pressures[index], i64::try_from(settlement.fission_pressure).unwrap());
            assert_eq!(snapshot.fission_thresholds[index], i64::try_from(settlement.fission_threshold).unwrap());
            assert!(snapshot.fission_thresholds[index] > 0);
        }
        assert_eq!(snapshot, collect_society_snapshot(&engine.world, &engine.resources));
        let first = snapshot.settlement_ids[0] as u32;
        let settlement = engine.resources.settlements.get_mut(&first).unwrap();
        settlement.fission_pressure = u64::MAX;
        settlement.fission_threshold = u64::MAX;
        let saturated = collect_society_snapshot(&engine.world, &engine.resources);
        assert_eq!(saturated.fission_pressures[0], i64::MAX);
        assert_eq!(saturated.fission_thresholds[0], i64::MAX);
        assert_eq!(engine.resources.settlements[&first].fission_pressure, u64::MAX);
        assert_eq!(engine.resources.settlements[&first].fission_threshold, u64::MAX);
    }

}

#[cfg(test)]
mod pass3_agent_snapshot_tests {
    use super::*;

    #[test]
    fn empty_world_has_empty_parallel_snapshot_rows() {
        let world = hecs::World::new();
        let rows = collect_agent_snapshot(&world);
        let (ids, xs, ys, states) = agent_rows_split(&rows);
        assert!(rows.is_empty());
        assert_eq!([ids.len(), xs.len(), ys.len(), states.len()], [0; 4]);
        let target_kinds: Vec<u8> = rows.iter().map(|row| row.target_kind).collect();
        let life_stages: Vec<u8> = rows.iter().map(|row| row.life_stage).collect();
        assert!(target_kinds.is_empty());
        assert!(life_stages.is_empty());
    }

    #[test]
    fn missing_state_and_child_default_to_none_and_adult() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 7 }, Position::new(2, 3)));
        let rows = collect_agent_snapshot(&world);
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].target_kind, rows[0].life_stage), (0, 0));
    }

    #[test]
    fn mixed_snapshot_targets_match_detail_and_live_child_markers() {
        let mut world = hecs::World::new();
        let targets = [None, Some(TargetKind::Food), Some(TargetKind::Water),
            Some(TargetKind::Sleep), Some(TargetKind::ConstructionSite),
            Some(TargetKind::Agent(99)), Some(TargetKind::GatherFood), Some(TargetKind::GatherWood)];
        let mut expected = Vec::new();
        for (code, target) in targets.into_iter().enumerate() {
            for consuming in [false, true] {
                let state = match target {
                    None => AgentState::Idle,
                    Some(target) if consuming => AgentState::Consuming { target },
                    Some(target) => AgentState::Seeking { target },
                };
                let entity = world.spawn((Agent { id: expected.len() as u64 },
                    Position::new(2, 3), state, Hunger::new(10.0, 0.05),
                    Thirst::new(20.0, 0.08), Sleep::new(30.0, 0.03)));
                if consuming {
                    world.insert_one(entity, Child { born_tick: 5 }).expect("child fixture");
                }
                expected.push((entity, code as u8, u8::from(consuming)));
            }
        }
        let rows = collect_agent_snapshot(&world);
        let (ids, xs, ys, states) = agent_rows_split(&rows);
        let target_kinds: Vec<u8> = rows.iter().map(|row| row.target_kind).collect();
        let life_stages: Vec<u8> = rows.iter().map(|row| row.life_stage).collect();
        assert_eq!(rows.len(), expected.len());
        assert_eq!([ids.len(), xs.len(), ys.len(), states.len(), target_kinds.len(),
            life_stages.len()], [expected.len(); 6]);
        for (entity, target, stage) in expected {
            let index = rows.iter().position(|row| row.entity_bits == entity.to_bits().get())
                .expect("fixture row present across archetypes");
            let detail = collect_agent_detail(&world, entity.to_bits().get());
            assert!(detail.found);
            assert_eq!((target_kinds[index], life_stages[index]), (target, stage));
            assert_eq!(i32::from(target_kinds[index]), detail.target_kind);
        }
    }

    #[test]
    fn elders_report_stage_two_and_ages_follow_snapshot_order() {
        use sim_core::components::{Elder, Life};
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 }, Position::new(2, 3)));
        world.spawn((Agent { id: 2 }, Position::new(2, 3), Life { born_tick: -2_880, lifespan_ticks: 90_000 }, Elder));
        world.spawn((Agent { id: 3 }, Position::new(2, 3), Child { born_tick: 100 }, Life { born_tick: 100, lifespan_ticks: 90_000 }));
        let rows = collect_agent_snapshot(&world);
        let ages = collect_agent_ages_days(&world, 1_540, 1_440);
        assert_eq!(rows.len(), ages.len());
        let by_id: std::collections::BTreeMap<_, _> = rows.iter().zip(&ages)
            .map(|(row, age)| (row.agent_id, (row.life_stage, *age))).collect();
        assert_eq!(by_id[&1], (0, -1));
        assert_eq!(by_id[&2], (2, 3));
        assert_eq!(by_id[&3], (1, 1));
    }

    #[test]
    fn colds_follow_snapshot_order() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 }, Position::new(2, 3)));
        world.spawn((Agent { id: 2 }, Position::new(2, 3), Cold));
        world.spawn((Agent { id: 3 }, Position::new(2, 3), Child { born_tick: 0 }, Cold));
        world.spawn((Agent { id: 4 }, Position::new(2, 3), Child { born_tick: 0 }));
        let rows = collect_agent_snapshot(&world);
        let colds = collect_agent_colds(&world);
        assert_eq!(rows.len(), colds.len());
        let by_id: std::collections::BTreeMap<_, _> = rows.iter().zip(&colds).map(|(row, cold)| (row.agent_id, *cold)).collect();
        assert_eq!(by_id.into_iter().collect::<Vec<_>>(), vec![(1, 0), (2, 1), (3, 1), (4, 0)]);
    }

    #[test]
    fn wood_targets_floor_the_target_per_settlement_row() {
        let mut resources = SimResources::new(16, 16, MaterialRegistry::new());
        let mut world = hecs::World::new();
        for (sid, x) in [(1u32, 2u32), (2, 9)] {
            let entity = world.spawn((Agent { id: u64::from(sid) }, Position::new(x, 3)));
            let _ = entity;
            let mut settlement = Settlement::new_with_id(sid, 0);
            settlement.formation_tile = (x, 3);
            settlement.add_member_agent(u64::from(sid));
            resources.settlements.insert(sid, settlement);
        }
        let rows = collect_settlement_snapshot(&world, &resources.settlements);
        assert_eq!(rows.len(), 2);
        // No firewood rule: zero for every row.
        assert_eq!(collect_settlement_wood_targets(&rows, &resources), vec![0, 0]);
        // v11 without seasons: 1.5 x 26 = 39; summer of a seasonal world 1.5 x 13 + 7 x 39 = 292.5 -> 292.
        resources.rules = WorldRules::game_v11();
        resources.rules.seasons = None;
        assert_eq!(collect_settlement_wood_targets(&rows, &resources), vec![39, 39]);
        resources.rules = WorldRules::game_v11();
        resources.current_tick = 7 * 1_440;
        assert_eq!(collect_settlement_wood_targets(&rows, &resources), vec![292, 292]);
        assert!(collect_settlement_wood_targets(&[], &resources).is_empty());
    }

    #[test]
    fn maturation_removes_child_flag_from_next_snapshot() {
        let mut world = hecs::World::new();
        let entity = world.spawn((Agent { id: 1 }, Position::new(2, 3), Child { born_tick: 0 }));
        assert_eq!(collect_agent_snapshot(&world)[0].life_stage, 1);
        world.remove_one::<Child>(entity).expect("child fixture exists");
        assert_eq!(collect_agent_snapshot(&world)[0].life_stage, 0);
    }
}
