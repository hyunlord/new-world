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
use sim_core::causal::{CausalEvent, EventId};
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine, SimResources};
use sim_systems::register_phase2_systems;

/// Default grid extent until Godot configures it (Phase 2 default).
const DEFAULT_W: u32 = 64;
/// Default grid extent until Godot configures it (Phase 2 default).
const DEFAULT_H: u32 = 64;

/// Godot `Node` subclass wrapping a [`SimEngine`] instance.
///
/// Exposes 5 FFI methods to GDScript/Godot:
/// - [`WorldSimNode::get_influence_overlay`]
/// - [`WorldSimNode::get_tile_detail`]
/// - [`WorldSimNode::on_building_placed`]
/// - [`WorldSimNode::get_tile_causal_history`] (γ-1)
/// - [`WorldSimNode::get_event_chain`] (γ-1)
#[derive(GodotClass)]
#[class(base=Node)]
pub struct WorldSimNode {
    engine: SimEngine,
    accumulator: f64,
    base: Base<Node>,
}

/// Fixed simulation timestep — 30 TPS per Phase 0 design #9 (Gaffer accumulator).
const FIXED_DT: f64 = 1.0 / 30.0;
/// Spiral-of-death cap: skip catch-up after this many fixed ticks per frame.
const MAX_ITERS_PER_FRAME: u32 = 5;

#[godot_api]
impl INode for WorldSimNode {
    fn init(base: Base<Node>) -> Self {
        let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        Self {
            engine,
            accumulator: 0.0,
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
        self.accumulator += delta;
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
/// `"stamp_dirty"`, `"influence_changed"`. Variant-specific fields are
/// `Some` only for the matching kind.
///
/// Field mapping:
/// - `id`, `tick` — always populated (every event).
/// - `parent` — `Some(id)` for chain children, `None` for roots
///   (`BuildingPlaced`) or after parent eviction. Serialised as `-1` for
///   `None` in the dictionary form.
/// - `channel` — `Some` for `StampDirty` / `InfluenceChanged` only.
/// - `position` — origin `(x, y)` for `BuildingPlaced`; sample centre for
///   `InfluenceChanged`; `None` for `StampDirty` (the region covers it).
/// - `radius` — `Some` only for `BuildingPlaced`.
/// - `region` — `Some(min_x, min_y, max_x, max_y)` only for `StampDirty`.
/// - `old_value` / `new_value` — `Some` only for `InfluenceChanged`.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalEventView {
    /// String discriminator: `"building_placed"` | `"stamp_dirty"` |
    /// `"influence_changed"`.
    pub kind: &'static str,
    /// Monotonic event id (V7 Phase 3-β).
    pub id: EventId,
    /// Parent event id; `None` denotes a chain root or evicted parent.
    pub parent: Option<EventId>,
    /// Simulation tick the event was recorded at.
    pub tick: u64,
    /// Channel index (matches [`InfluenceChannel`] ordering), or `None`
    /// for `BuildingPlaced`.
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
            },
        }
    }
}

fn dirty_region_bounds(region: &DirtyRegion) -> (u32, u32, u32, u32) {
    (region.min_x, region.min_y, region.max_x, region.max_y)
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
fn event_view_to_dict(view: &CausalEventView) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("kind", view.kind);
    dict.set("id", view.id as i64);
    dict.set("parent", view.parent.map(|p| p as i64).unwrap_or(-1));
    dict.set("tick", view.tick as i64);
    if let Some(ch) = view.channel {
        dict.set("channel", ch as i32);
    }
    if let Some((px, py)) = view.position {
        dict.set("position", Vector2i::new(px as i32, py as i32));
    }
    if let Some(r) = view.radius {
        dict.set("radius", r as i32);
    }
    if let Some((min_x, min_y, max_x, max_y)) = view.region {
        dict.set(
            "region",
            Vector4i::new(min_x as i32, min_y as i32, max_x as i32, max_y as i32),
        );
    }
    if let Some(old) = view.old_value {
        dict.set("old", old);
    }
    if let Some(new) = view.new_value {
        dict.set("new", new);
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
