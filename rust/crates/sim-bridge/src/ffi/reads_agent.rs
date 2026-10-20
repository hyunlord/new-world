//! One-agent reads for the inspector: agent detail rows, temperament, and the
//! pairwise relationship snapshot. Pure collectors plus the Godot marshallers
//! the `#[func]` methods in `node.rs` forward to.

use godot::prelude::*;
use sim_core::components::{Agent, AgentState, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_engine::SimResources;

/// Revitalize Pass 16 — the temperament of the agent whose
/// `hecs::Entity::to_bits()` is `entity_bits`, or `None` when the bits are
/// not a live entity or it carries no `Temperament` (every scene before
/// `game_v20`). Bridge Identity Contract: `WorldSimNode::get_agent_temperament`
/// only forwards here.
pub fn collect_agent_temperament(
    world: &hecs::World,
    entity_bits: u64,
) -> Option<sim_core::components::Temperament> {
    let entity = hecs::Entity::from_bits(entity_bits)?;
    world
        .get::<&sim_core::components::Temperament>(entity)
        .ok()
        .map(|t| *t)
}

/// Marshals [`collect_agent_temperament`] into `{found, ns, ha, rd, p}`
/// (ints; `found: false` and zeros when missing).
pub(super) fn agent_temperament_to_dict(
    temperament: Option<sim_core::components::Temperament>,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let t = temperament.unwrap_or(sim_core::components::Temperament {
        ns: 0,
        ha: 0,
        rd: 0,
        p: 0,
    });
    dict.set("found", temperament.is_some());
    dict.set("ns", i64::from(t.ns));
    dict.set("ha", i64::from(t.ha));
    dict.set("rd", i64::from(t.rd));
    dict.set("p", i64::from(t.p));
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
/// - `state_tag` — same mapping as
///   [`AgentSnapshotRow`] (0=Idle, 1=Seeking, 2=Consuming(Agent),
///   3=Consuming(other), 4=Seeking/Consuming(Activity))
/// - `hunger` — `Hunger.value` (`f32`; `[0, SATURATION=100]`)
/// - `thirst` — `Thirst.value` (`f64`; `[0, SATURATION=100]`)
/// - `sleep` — `Sleep.fatigue` (`f64`; `[0, SATURATION=100]`)
/// - `target_kind` — `i32` encoding of `Option<TargetKind>`:
///   0=None (Idle), 1=Food, 2=Water, 3=Sleep, 4=ConstructionSite,
///   5=Agent (inner `AgentId` not surfaced), 6=GatherFood, 7=GatherWood,
///   8=Activity.
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
    let mut q = match world
        .query_one::<(&Agent, &Position, &AgentState, &Hunger, &Thirst, &Sleep)>(entity)
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
        AgentState::Seeking {
            target: TargetKind::Activity,
        }
        | AgentState::Consuming {
            target: TargetKind::Activity,
        } => 4,
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
        Some(TargetKind::Activity) => 8,
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
pub(super) fn agent_detail_to_dict(row: AgentDetailRow) -> VarDictionary {
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
pub fn collect_relationship_snapshot(resources: &SimResources) -> Vec<RelationshipSnapshotRow> {
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
pub(super) fn relationship_rows_to_variant_array(rows: &[RelationshipSnapshotRow]) -> VarArray {
    let mut arr = VarArray::new();
    for row in rows {
        arr.push(&Variant::from(relationship_row_to_dict(row)));
    }
    arr
}
