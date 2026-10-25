//! The causal "why?" reads: `CausalEventView` (a flattened `CausalEvent`), its
//! owned-dict schema, the tile-history / event-chain collectors with their
//! bounds-checked FFI mirrors, and the influence-channel helpers used by the
//! overlay and tile-detail getters.

use godot::prelude::*;
use sim_core::causal::{CausalEvent, EventId, MemoryRecallTrigger};
use sim_core::components::AgentId;
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_engine::SimResources;

#[cfg(test)]
use sim_core::material::MaterialRegistry;
#[cfg(test)]
use sim_engine::SimEngine;

/// Map a channel index `i32` to `InfluenceChannel`, or `None` if out of range.
pub(super) fn channel_from_i32(ix: i32) -> Option<InfluenceChannel> {
    if ix < 0 {
        return None;
    }
    InfluenceChannel::all().get(ix as usize).copied()
}

/// Return the dictionary key string for a channel.
/// Exhaustive match — compile-time coverage of all 8 channels.
pub(super) fn channel_key(ch: InfluenceChannel) -> &'static str {
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
            CausalEvent::AgentBorn {
                id,
                parent,
                agent,
                tick,
                ..
            } => Self {
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
                id, parent, tick, ..
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
                id, parent, tick, ..
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
            // M5 (ADR 0027): a pressure touched a person. `reason` is the pressure's
            // name, `triggered_by` what it did.
            CausalEvent::PressureCause {
                id,
                parent,
                agent,
                position,
                kind,
                effect,
                tick,
            } => Self {
                kind: "pressure_cause",
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
                reason: Some(
                    sim_core::colony::story::PressureKind::checked(*kind)
                        .map_or("unknown", |k| k.name()),
                ),
                triggered_by: Some(match effect {
                    0 => "cold",
                    1 => "wound",
                    2 => "sickness",
                    _ => "thirst",
                }),
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
pub fn event_view_to_owned_dict(
    view: &CausalEventView,
) -> std::collections::BTreeMap<&'static str, FfiFieldValue> {
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
            FfiFieldValue::Region4i(min_x as i32, min_y as i32, max_x as i32, max_y as i32),
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
        dict.insert("recalled_event", FfiFieldValue::I64(recalled_event as i64));
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
    log.as_slice()
        .iter()
        .map(CausalEventView::from_event)
        .collect()
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

/// Collect matching events once, preserving tile input order and oldest-first rings.
/// Invalid coordinates are ignored, just like the single-tile getter.
pub fn collect_tile_causal_events(
    resources: &SimResources,
    tiles: &[(i32, i32)],
    kinds: &[&str],
) -> Vec<CausalEventView> {
    let mut seen = std::collections::BTreeSet::new();
    let mut events = Vec::new();
    for &(x, y) in tiles {
        for event in try_collect_tile_causal_history(resources, x, y) {
            if kinds.contains(&event.kind) && seen.insert(event.id) {
                events.push(event);
            }
        }
    }
    events
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
pub(super) fn event_views_to_variant_array(views: &[CausalEventView]) -> VarArray {
    let mut arr = VarArray::new();
    for view in views {
        arr.push(&Variant::from(event_view_to_dict(view)));
    }
    arr
}

#[cfg(test)]
mod causal_batch_tests {
    use super::*;

    fn fixture() -> SimEngine {
        let mut engine = SimEngine::new(2, 2, MaterialRegistry::new());
        let placed = |id| CausalEvent::BuildingPlaced {
            id,
            parent: None,
            position: (0, 0),
            radius: 1,
            tick: 3,
        };
        engine.resources.causal_log.push(0, placed(9));
        engine.resources.causal_log.push(0, placed(2));
        engine.resources.causal_log.push(1, placed(2));
        engine.resources.causal_log.push(1, placed(7));
        engine.resources.causal_log.push(
            1,
            CausalEvent::InfluenceChanged {
                id: 11,
                parent: None,
                channel: InfluenceChannel::Warmth,
                position: (1, 0),
                old: 0.0,
                new: 1.0,
                tick: 4,
            },
        );
        engine
    }

    #[test]
    fn empty_input_and_invalid_tiles_are_empty() {
        let engine = fixture();
        assert!(
            collect_tile_causal_events(&engine.resources, &[], &["building_placed"]).is_empty()
        );
        assert!(collect_tile_causal_events(
            &engine.resources,
            &[(-1, 0), (2, 2)],
            &["building_placed"]
        )
        .is_empty());
    }

    #[test]
    fn shared_events_are_emitted_once() {
        let engine = fixture();
        let events = collect_tile_causal_events(
            &engine.resources,
            &[(0, 0), (1, 0), (0, 0)],
            &["building_placed"],
        );
        assert_eq!(
            events.iter().map(|event| event.id).collect::<Vec<_>>(),
            vec![9, 2, 7]
        );
    }

    #[test]
    fn kinds_filter_excludes_other_events() {
        let engine = fixture();
        assert!(
            collect_tile_causal_events(&engine.resources, &[(0, 0)], &["combat_started"])
                .is_empty()
        );
        assert!(collect_tile_causal_events(&engine.resources, &[(0, 0)], &[]).is_empty());
        let events = collect_tile_causal_events(
            &engine.resources,
            &[(0, 0), (1, 0)],
            &["influence_changed"],
        );
        assert_eq!(
            events.iter().map(|event| event.id).collect::<Vec<_>>(),
            vec![11]
        );
    }

    #[test]
    fn input_tile_and_ring_order_win_over_event_id_order() {
        let engine = fixture();
        let events =
            collect_tile_causal_events(&engine.resources, &[(1, 0), (0, 0)], &["building_placed"]);
        assert_eq!(
            events.iter().map(|event| event.id).collect::<Vec<_>>(),
            vec![2, 7, 9]
        );
    }
}
