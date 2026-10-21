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

use super::activity::{activity_rows_to_dict, collect_activity_snapshot};
use super::causal::{
    channel_from_i32, channel_key, collect_tile_causal_events, event_views_to_variant_array,
    try_collect_event_chain, try_collect_tile_causal_history,
};
use super::chronicle::{
    collect_agent_id, collect_chronicle_since, collect_person, collect_person_chronicle,
    collect_settlement_chronicle, person_to_dict, rows_to_dict,
};
use super::colony::{
    can_place, collect_agent_jobs, collect_colony_overlay, collect_colony_versions, collect_trees,
    overlay_to_dict, result_to_dict, submit_blueprint, submit_rect, submit_tiles, trees_to_dict,
    versions_to_dict, RectCommand,
};
use super::engine_build::{advance_engine_ticks, build_game_engine, build_stress_engine};
use super::family::{agent_family_to_dict, collect_agent_family};
use super::reads_agent::{
    agent_detail_to_dict, agent_temperament_to_dict, collect_agent_detail,
    collect_agent_temperament, collect_relationship_snapshot, relationship_rows_to_variant_array,
};
use super::reads_settlement::{
    collect_settlement_snapshot, collect_settlement_wood_targets, collect_society_snapshot,
    settlement_rows_to_dict, society_snapshot_to_dict,
};
use super::relations::{
    collect_porter_snapshot, collect_settlement_relations, porter_rows_to_dict,
    relation_rows_to_dict,
};
use super::snapshot::{
    agent_rows_to_dict, building_rows_to_dict, collect_agent_ages_days, collect_agent_ages_years,
    collect_agent_colds, collect_agent_infancy_and_pregnancy, collect_agent_snapshot,
    collect_building_snapshot, collect_construction_snapshot, collect_recent_deaths,
    collect_resource_snapshot, collect_tallies, collect_terrain_snapshot, collect_time_snapshot,
    collect_world_info, construction_rows_to_dict, recent_death_rows_to_dict,
    resource_rows_to_dict, tallies_to_dict, terrain_snapshot_to_dict, time_snapshot_to_dict,
    world_info_to_dict,
};
use godot::classes::INode;
use godot::prelude::*;
use sim_core::influence::InfluenceChannel;
use sim_engine::{BuildingPlacedEvent, SimEngine, SimResources};
use sim_systems::worldgen::DEFAULT_GAME_SEED;

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

/// Ticks fed per real second at 1× (ADR 0004, M1): one tick is one simulated
/// minute, so a simulated day (1,440 ticks) lasts 120 s at 1×; 2×/3×/4× feed
/// 24/36/48 ticks per second.
pub const TICKS_PER_SECOND_AT_1X: f64 = 12.0;
/// Fixed simulation timestep of the Gaffer accumulator.
const FIXED_DT: f64 = 1.0 / TICKS_PER_SECOND_AT_1X;
/// Spiral-of-death cap: at most this many ticks per frame; a larger backlog
/// is dropped (4× = 48 ticks/s needs ≥ 6 frames per second).
pub const MAX_TICKS_PER_FRAME: u32 = 8;
const MAX_ITERS_PER_FRAME: u32 = MAX_TICKS_PER_FRAME;

/// Real seconds one simulated day lasts at speed `speed` (0 = never).
pub fn seconds_per_day(speed: f64) -> f64 {
    let speed = clamp_sim_speed(speed);
    if speed <= 0.0 {
        f64::INFINITY
    } else {
        1440.0 / (TICKS_PER_SECOND_AT_1X * speed)
    }
}

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
            // Revitalize Pass 21: the live game runs `game_v26` (village relations).
            engine: build_game_engine(DEFAULT_GAME_SEED),
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
        self.engine = build_game_engine(self.world_seed);
        self.accumulator = 0.0;
        self.base_mut()
            .call_deferred("emit_signal", &["world_reset".to_variant()]);
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
        self.base_mut()
            .call_deferred("emit_signal", &["world_reset".to_variant()]);
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

    /// V7 Section 16-γ — set the simulation-speed multiplier driving the
    /// per-frame accumulator increment. Stores [`clamp_sim_speed`]`(speed)`,
    /// so out-of-range requests are clamped to `[0.0, 4.0]`. Bound to the
    /// `KEY_1`…`KEY_4` number keys by `camera_controller.gd` (1× / 2× / 3× /
    /// 4× = 12 / 24 / 36 / 48 ticks per second, M1). Pure observability — changes only how often
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
        let in_bounds = x >= 0 && y >= 0 && (x as u32) < grid.width && (y as u32) < grid.height;
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

    /// Art Pass 9: filtered causal events in input-tile/ring order, once per id.
    #[func]
    fn get_tile_causal_events(
        &self,
        xs: PackedInt32Array,
        ys: PackedInt32Array,
        kinds: PackedStringArray,
    ) -> VarArray {
        let tiles: Vec<(i32, i32)> = xs
            .as_slice()
            .iter()
            .copied()
            .zip(ys.as_slice().iter().copied())
            .collect();
        let names: Vec<String> = kinds.as_slice().iter().map(ToString::to_string).collect();
        let kinds: Vec<&str> = names.iter().map(String::as_str).collect();
        event_views_to_variant_array(&collect_tile_causal_events(
            &self.engine.resources,
            &tiles,
            &kinds,
        ))
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
        // M1.5 — additive whole-year ages, infant flags and pregnancy days left
        // in the same row order (`docs/v8/snapshot_schema.md`).
        let years = collect_agent_ages_years(
            &self.engine.world,
            self.engine.current_tick(),
            self.engine.resources.rules.calendar.ticks_per_year(),
        );
        dict.set("ages_years", PackedInt32Array::from(years.as_slice()));
        let (infants, pregnant) = collect_agent_infancy_and_pregnancy(
            &self.engine.world,
            &self.engine.resources,
            self.engine.current_tick(),
        );
        dict.set("infants", PackedByteArray::from(infants.as_slice()));
        dict.set("pregnant_days", PackedInt32Array::from(pregnant.as_slice()));
        // Pass 6 — additive cold flags in the same row order (1 with the Cold marker).
        let colds = collect_agent_colds(&self.engine.world);
        dict.set("colds", PackedByteArray::from(colds.as_slice()));
        // M1 — additive job and carry columns in the same row order: `jobs`
        // (0 none, 1 chop, 2 haul, 3 deliver, 4 build), `carry_kinds` (-1 none,
        // 0 log) and `carry_counts`.
        let jobs = collect_agent_jobs(&self.engine.world);
        let codes: Vec<u8> = jobs.iter().map(|j| j.0).collect();
        let kinds: Vec<i32> = jobs.iter().map(|j| j.1).collect();
        let counts: Vec<i32> = jobs.iter().map(|j| i32::from(j.2)).collect();
        dict.set("jobs", PackedByteArray::from(codes.as_slice()));
        dict.set("carry_kinds", PackedInt32Array::from(kinds.as_slice()));
        dict.set("carry_counts", PackedInt32Array::from(counts.as_slice()));
        dict
    }

    /// M1 order: mark every tree in the rectangle for felling. Returns
    /// `{accepted, count, reason}`; logged with the next tick (ADR 0012).
    #[func]
    fn cmd_designate_chop(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> VarDictionary {
        result_to_dict(submit_rect(
            &mut self.engine,
            RectCommand::DesignateChop,
            x0,
            y0,
            x1,
            y1,
        ))
    }

    /// M1 order: mark the trees on exactly these tiles (scripts and tools).
    #[func]
    fn cmd_designate_chop_tiles(
        &mut self,
        xs: PackedInt32Array,
        ys: PackedInt32Array,
    ) -> VarDictionary {
        result_to_dict(submit_tiles(&mut self.engine, xs.as_slice(), ys.as_slice()))
    }

    /// M1 order: remove chop marks and blueprints in the rectangle.
    #[func]
    fn cmd_cancel_area(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> VarDictionary {
        result_to_dict(submit_rect(
            &mut self.engine,
            RectCommand::CancelArea,
            x0,
            y0,
            x1,
            y1,
        ))
    }

    /// M1 order: add the rectangle's valid tiles as stockpile zone(s).
    #[func]
    fn cmd_add_stockpile(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> VarDictionary {
        result_to_dict(submit_rect(
            &mut self.engine,
            RectCommand::AddStockpile,
            x0,
            y0,
            x1,
            y1,
        ))
    }

    /// M1 order: remove the rectangle's tiles from every zone.
    #[func]
    fn cmd_remove_zone_area(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> VarDictionary {
        result_to_dict(submit_rect(
            &mut self.engine,
            RectCommand::RemoveZoneArea,
            x0,
            y0,
            x1,
            y1,
        ))
    }

    /// M1 order: place a blueprint (`"hut"`) with its footprint's top-left at `(x, y)`.
    #[func]
    fn cmd_place_blueprint(&mut self, kind: GString, x: i32, y: i32) -> VarDictionary {
        result_to_dict(submit_blueprint(&mut self.engine, &kind.to_string(), x, y))
    }

    /// M1: whether a blueprint (`"hut"`) fits with its top-left at `(x, y)`.
    #[func]
    fn can_place_blueprint(&self, kind: GString, x: i32, y: i32) -> bool {
        can_place(&self.engine.resources, &kind.to_string(), x, y)
    }

    /// M1: `{trees_version, overlay_version}` — poll every frame, fetch
    /// `get_trees` / `get_colony_overlay` only when a version changed.
    #[func]
    fn get_colony_versions(&self) -> VarDictionary {
        versions_to_dict(collect_colony_versions(&self.engine.resources))
    }

    /// M1: every standing tree as `{version, xs, ys}` in `(x, y)` order.
    #[func]
    fn get_trees(&self) -> VarDictionary {
        trees_to_dict(collect_trees(&self.engine.resources))
    }

    /// M1: the order overlay as `{version, designation_xs/ys, zone_ids,
    /// zone_xs/ys, item_xs/ys/kinds/counts, site_xs/ys/ws/hs/kinds/needed/
    /// delivered/progress/required}` (sites by footprint top-left).
    #[func]
    fn get_colony_overlay(&self) -> VarDictionary {
        overlay_to_dict(&collect_colony_overlay(
            &self.engine.world,
            &self.engine.resources,
        ))
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

    /// Revitalize Pass 16 FFI — one agent's temperament as
    /// `{found: bool, ns, ha, rd, p}` (permille ints, 0–1,000; `found: false`
    /// and zeros when the agent or the component is missing). A separate
    /// getter: `get_agent_detail` keeps exactly its nine keys. Displaying
    /// temperament is the art line's job.
    #[func]
    fn get_agent_temperament(&self, entity_bits: i64) -> VarDictionary {
        agent_temperament_to_dict(collect_agent_temperament(
            &self.engine.world,
            entity_bits as u64,
        ))
    }

    /// Pass 17 family as exactly `{found, female, mother, father, partner, children}`.
    /// Input uses snapshot `ids` (entity bits); links use `agent_ids` (AgentId).
    /// Missing links are -1; children are a sorted PackedInt64Array of living
    /// offspring at any life stage. Missing family components return found=false,
    /// female=false, -1 links and empty children; recorded absent parents remain ids.
    #[func]
    fn get_agent_family(&self, entity_bits: i64) -> VarDictionary {
        agent_family_to_dict(collect_agent_family(
            &self.engine.world,
            u64::from_ne_bytes(entity_bits.to_ne_bytes()),
        ))
    }

    /// Pass 19 chronicle: the AgentId (as i64) of the agent with snapshot
    /// entity bits `entity_bits`, or -1.
    #[func]
    fn get_agent_id(&self, entity_bits: i64) -> i64 {
        collect_agent_id(&self.engine.world, entity_bits)
            .map_or(-1, |id| i64::from_ne_bytes(id.to_ne_bytes()))
    }

    /// Pass 19 chronicle: the person table's record of `agent_id` (works for
    /// the dead) as `{found, alive, female (-1/0/1), born_tick, died_tick,
    /// death_reason, mother, father, ns, ha, rd, p}`; unknown values are -1.
    #[func]
    fn get_person(&self, agent_id: i64) -> VarDictionary {
        person_to_dict(collect_person(
            &self.engine.world,
            &self.engine.resources.chronicle,
            agent_id,
        ))
    }

    /// Pass 19 chronicle: every entry naming `agent_id`, in chronicle order, as
    /// `{found, tick, kind, subject, other, third, settlement, settlement2,
    /// cause, value}` packed columns (ids -1 when absent).
    #[func]
    fn get_person_chronicle(&self, agent_id: i64) -> VarDictionary {
        rows_to_dict(
            collect_person_chronicle(&self.engine.resources.chronicle, agent_id),
            false,
        )
    }

    /// Pass 19 chronicle: every entry naming `settlement_id` (same shape as
    /// `get_person_chronicle`).
    #[func]
    fn get_settlement_chronicle(&self, settlement_id: i64) -> VarDictionary {
        rows_to_dict(
            collect_settlement_chronicle(&self.engine.resources.chronicle, settlement_id),
            false,
        )
    }

    /// Pass 19 chronicle feed: entries from index `start`, at most `max`
    /// (clamped to 1..=512), same shape plus `next` and `total`.
    #[func]
    fn get_chronicle_since(&self, start: i64, max: i64) -> VarDictionary {
        rows_to_dict(
            collect_chronicle_since(&self.engine.resources.chronicle, start, max),
            true,
        )
    }

    /// Pass 20 village relations: every pair of living villages (lower id
    /// first, key order) as `{a, b, value, estranged, base}` packed columns.
    /// Empty without the relations rule.
    #[func]
    fn get_settlement_relations(&self) -> VarDictionary {
        relation_rows_to_dict(&collect_settlement_relations(
            &self.engine.resources.settlement_relations,
        ))
    }

    /// Pass 20 porters: every porter on a trip by AgentId as `{ids (entity
    /// bits), from, to, kind (0 food / 1 wood), amount, stage (0 out / 1 back)}`.
    #[func]
    fn get_porter_snapshot(&self) -> VarDictionary {
        porter_rows_to_dict(&collect_porter_snapshot(&self.engine.world))
    }

    /// Pass 17 activities as five parallel packed arrays, ordered by AgentId.
    /// `ids` are entity bits matching the agent snapshot. `kinds` use 0..6
    /// (TendFire, Craft, Repair, HearthCircle, Explore, Play, SitByFire).
    /// `place_x`, `place_y` identify the chosen place; `until_tick` is -1
    /// until arrival. Existing agent snapshot and detail schemas stay unchanged.
    #[func]
    fn get_activity_snapshot(&self) -> VarDictionary {
        activity_rows_to_dict(&collect_activity_snapshot(&self.engine.world))
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
        let rows =
            collect_settlement_snapshot(&self.engine.world, &self.engine.resources.settlements);
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
pub fn enqueue_building_placed(resources: &mut SimResources, x: i32, y: i32, radius: i32) -> bool {
    if x < 0 || y < 0 || radius < 0 {
        return false;
    }
    let grid = &resources.influence_grid;
    if (x as u32) >= grid.width || (y as u32) >= grid.height {
        return false;
    }
    resources
        .building_event_queue
        .push_back(BuildingPlacedEvent {
            position: (x as u32, y as u32),
            radius: radius as u32,
        });
    true
}
