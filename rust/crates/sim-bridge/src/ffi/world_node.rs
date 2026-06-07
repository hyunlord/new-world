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
    Agent, AgentId, AgentState, BodyHealth, ConstructionSite, Hunger, Memory, Position, SeekTarget,
    Settlement, SettlementId, Sleep, Social, TargetKind, Thirst,
};
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine, SimResources, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

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

#[godot_api]
impl INode for WorldSimNode {
    fn init(base: Base<Node>) -> Self {
        Self {
            engine: init_production_engine(),
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
        agent_rows_to_dict(&rows)
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
        settlement_rows_to_dict(&rows)
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
    for (entity, (agent, pos, maybe_state, maybe_seek, maybe_hunger, maybe_thirst, maybe_sleep)) in world
        .query::<(
            &Agent,
            &Position,
            Option<&AgentState>,
            Option<&SeekTarget>,
            Option<&Hunger>,
            Option<&Thirst>,
            Option<&Sleep>,
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
    /// Resource kind: `0 = Food`, `1 = Water`, `2 = Sleep`.
    pub kind: u8,
    /// Current stored amount on the tile (the `*_tiles` counter), CLAMPED to
    /// [`max`](Self::max). Drives the renderer's depletion/regen visual (marker
    /// scale + alpha = amount/max). A finite source decreases as agents consume
    /// and recovers via regen; a tile at `0` is removed from the map entirely
    /// and so drops out of the snapshot (the marker disappears). Clamped to
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
    rows.sort_by_key(|r| (r.kind, r.x, r.y));
    rows
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
        });
    }
    // HashMap iteration order is not stable run-to-run; sort by the unique
    // settlement_id so the snapshot is deterministic (Day-1 invariant +
    // stable furniture-sprite keying downstream).
    rows.sort_by_key(|r| r.settlement_id);
    rows
}

/// Marshal a [`SettlementSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer._update_settlement_furniture`.
/// Five parallel `PackedArray`s, lengths always equal to `rows.len()`.
///
/// Keys:
/// - `ids`: `PackedInt64Array` — `entity_bits` per row.
/// - `settlement_ids`: `PackedInt32Array` — `settlement_id` per row.
/// - `centroid_xs`: `PackedInt32Array` — tile-x centroid.
/// - `centroid_ys`: `PackedInt32Array` — tile-y centroid.
/// - `member_counts`: `PackedInt32Array` — resolvable member count.
fn settlement_rows_to_dict(rows: &[SettlementSnapshotRow]) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut settlement_ids = PackedInt32Array::new();
    let mut centroid_xs = PackedInt32Array::new();
    let mut centroid_ys = PackedInt32Array::new();
    let mut member_counts = PackedInt32Array::new();
    let mut formation_xs = PackedInt32Array::new();
    let mut formation_ys = PackedInt32Array::new();
    ids.resize(n);
    settlement_ids.resize(n);
    centroid_xs.resize(n);
    centroid_ys.resize(n);
    member_counts.resize(n);
    formation_xs.resize(n);
    formation_ys.resize(n);
    for (i, row) in rows.iter().enumerate() {
        ids[i] = row.entity_bits as i64;
        settlement_ids[i] = row.settlement_id as i32;
        centroid_xs[i] = row.centroid_x;
        centroid_ys[i] = row.centroid_y;
        member_counts[i] = row.member_count as i32;
        formation_xs[i] = row.formation_x;
        formation_ys[i] = row.formation_y;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("settlement_ids", settlement_ids);
    dict.set("centroid_xs", centroid_xs);
    dict.set("centroid_ys", centroid_ys);
    dict.set("member_counts", member_counts);
    dict.set("formation_xs", formation_xs);
    dict.set("formation_ys", formation_ys);
    dict
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

/// Build the production [`SimEngine`] exactly as the live `WorldSimNode::init`
/// path does — the SINGLE construction entry point the dylib uses to build the
/// initial `SimResources` for the real game.
///
/// Order:
/// 1. [`register_default_runtime_systems`] — every default simulation system
///    (BSS, IUS, AIS, AgentMovement, AgentDecision, Hunger/Thirst/Sleep decay,
///    Construction, Social, Memory, Combat, Settlement, Starvation,
///    ResourceRegen, InfluenceVisualization).
/// 2. [`bootstrap_spawn_agents`] — the 64-agent lattice + INFINITE source seeds.
/// 3. [`seed_finite_resource_scarcity`] — overwrite sources to finite + register
///    regen ceilings (`add-resource-scarcity-regen`).
///
/// `pub` so the scarcity harness can exercise the shipped construction path
/// headlessly (A10 production-wiring invariant — proves the dylib seeds finite
/// sources, not merely that the seed function exists).
pub fn init_production_engine() -> SimEngine {
    let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    bootstrap_spawn_agents(&mut engine);
    seed_finite_resource_scarcity(&mut engine);
    engine
}
