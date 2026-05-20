//! Cause-effect event variants recorded into per-tile ring buffers.
//!
//! V7 Phase 3-α (Week 5-6) — adapts the A-4 32-event-per-entity ring buffer
//! to a tile-level 8-event sparse log. The minimum recording surface lives
//! at the two influence pipeline junctions:
//!
//! 1. `BuildingStampSystem` (priority 90) drains the FFI event queue —
//!    emits [`CausalEvent::BuildingPlaced`] (event arrival) followed by
//!    [`CausalEvent::StampDirty`] for every channel that received a dirty
//!    region.
//! 2. `InfluenceUpdateSystem` (priority 100) drains dirty regions per
//!    channel — emits [`CausalEvent::InfluenceChanged`] once per drained
//!    region at the region centre (the same sample the propagation
//!    primitive uses as its source).
//!
//! V7 Phase 3-β extends every variant with `id: EventId` + `parent:
//! Option<EventId>` so the "왜?" UI can walk the chain
//! `BuildingPlaced → StampDirty → InfluenceChanged` backwards. IDs are
//! issued from a single global `AtomicU64` counter on `SimResources`
//! (eviction-tolerant — see `SimResources::issue_event_id`).
//!
//! `Copy` is intentionally NOT derived: [`DirtyRegion`] is `Clone` only,
//! so [`CausalEvent`] inherits the same constraint. Callers move/clone
//! events explicitly rather than rely on a bit-copy.

use crate::components::AgentId;
use crate::influence::{DirtyRegion, InfluenceChannel};

/// Reason an agent transitioned from `Idle` to `Seeking` (V7 Phase 5-β /
/// P5β-3). Encoded into [`CausalEvent::AgentDecision`] so the "왜?" UI
/// can surface "왜 이 agent가 이 길을 갔나?" in a stable, machine-
/// readable form. Phase 5-β scope is intentionally narrow: only need-
/// driven breaches. Phase 5-γ adds the Fatigue branch.
/// Phase 6-β adds the Construction branch — the 4th cascade step, lowest
/// priority among the four drives. Mood/morale/social reasons land in δ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionReason {
    /// Hunger crossed `HUNGER_THRESHOLD` upward this tick.
    HungerThresholdBreach,
    /// Thirst crossed `THIRST_THRESHOLD` upward this tick.
    ThirstThresholdBreach,
    /// Sleep fatigue crossed `FATIGUE_THRESHOLD` upward this tick
    /// (V7 Phase 5-γ / P5γ-6).
    FatigueThresholdBreach,
    /// Agent transitioned from Idle to Seeking{ConstructionSite} after
    /// detecting a co-located active ConstructionSite while no Need was
    /// breached. V7 Phase 6-β / P6β-5. Construction is the lowest-priority
    /// drive — Hunger/Thirst/Fatigue always win.
    ConstructionReason,
    /// Agent transitioned from Idle to Seeking{Agent(other)} after
    /// detecting a co-located partner whose loneliness also breached
    /// `SOCIAL_THRESHOLD`. V7 Phase 7-β / P7β-5. Social is the lowest-
    /// priority drive — Needs and Construction always win.
    SocialReason,
    /// Agent's `AgentDecisionSystem` cascade was flipped by a positive
    /// or negative memory weight delta on a non-natural-winner arm.
    /// Parent points to the `MemoryRecalled` event that surfaced the
    /// load-bearing memory. V7 Phase 8-β / P8β-5.
    MemoryReason,
    /// Agent's combat cascade arm activated by a negative memory weight
    /// delta that strictly exceeded `BIAS_FLIP_THRESHOLD` in the negative
    /// direction. A co-located idle peer is the combat target.
    /// V7 Phase 9-β / P9β-5.
    CombatReason,
}

impl DecisionReason {
    /// Stable string discriminator used by FFI views (CausalEventView).
    /// Kept short + snake_case to match the existing `"building_placed"`
    /// style.
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionReason::HungerThresholdBreach => "hunger_threshold_breach",
            DecisionReason::ThirstThresholdBreach => "thirst_threshold_breach",
            DecisionReason::FatigueThresholdBreach => "fatigue_threshold_breach",
            DecisionReason::ConstructionReason => "construction_reason",
            DecisionReason::SocialReason => "social_reason",
            DecisionReason::MemoryReason => "memory_reason",
            DecisionReason::CombatReason => "combat_reason",
        }
    }
}

/// Trigger taxonomy for [`CausalEvent::MemoryRecalled`]. V7 Phase 8-β /
/// P8β-MOD-1.
///
/// Phase 8-β wires only `CascadeBias`; `SimilaritySearch` and `Periodic`
/// are declared (and round-trip through serde) but never emitted by
/// production systems in this phase. Mirrors the `TargetKind`
/// extensibility pattern from Phase 7-α.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MemoryRecallTrigger {
    /// Phase 8-β scope: `AgentDecisionSystem` cascade scoring summoned
    /// this memory into the weight calculation and the weight shift
    /// flipped the natural-cascade winner.
    CascadeBias,
    /// Reserved — Phase 9+ similarity search (e.g. "recall any prior
    /// `SocialInteractionCompleted` with this partner").
    SimilaritySearch,
    /// Reserved — Phase 9+ periodic background recall (sleep-time
    /// consolidation, mood-driven rumination, etc.).
    Periodic,
    /// Phase 9-β scope: `AgentDecisionSystem` combat cascade arm fired
    /// this memory recall. `agent_id` is the enemy agent targeted by
    /// the combat transition.
    CombatContext {
        /// The enemy agent targeted by the combat cascade.
        agent_id: AgentId,
    },
}

/// Unique identifier for a [`CausalEvent`] within a single simulation run.
///
/// Issued monotonically from `SimResources::next_event_id` (an
/// `AtomicU64` counter). The id outlives ring-buffer eviction — even
/// after a parent event is evicted, descendants retain the id reference;
/// the lookup simply fails gracefully (see
/// [`CausalLogStorage::trace_parents`](super::storage::CausalLogStorage::trace_parents)).
///
/// V7 Phase 3-β (P3β-1).
///
/// [`CausalLogStorage::trace_parents`]: super::CausalLogStorage::trace_parents
pub type EventId = u64;

/// A single cause-effect record stamped into a [`TileCausalLog`].
///
/// Phase 3-α scope (locked by P3α-1): exactly three variants — building
/// placement, BSS dirty-region stamp, and IUS propagation tick. Future
/// phases may add `AgentDecision`, `Combat`, etc. — but the ring size
/// stays 8 (see [`TILE_CAUSAL_RING_SIZE`]).
///
/// Phase 3-β (P3β-2) adds `id: EventId` + `parent: Option<EventId>` to
/// every variant. Chain semantic (P3β-3):
/// `BuildingPlaced { parent: None }` → `StampDirty { parent: Some(building_id) }`
/// → `InfluenceChanged { parent: Some(stamp_id_for_same_channel) }`.
///
/// [`TileCausalLog`]: super::TileCausalLog
/// [`TILE_CAUSAL_RING_SIZE`]: super::TILE_CAUSAL_RING_SIZE
#[derive(Debug, Clone, PartialEq)]
pub enum CausalEvent {
    /// A building placement event arrived from FFI and was accepted by BSS
    /// (in-bounds, non-zero radius). Recorded once per accepted event onto
    /// the centre tile so the "왜?" UI can attribute downstream stamps.
    ///
    /// Chain root: `parent == None`.
    BuildingPlaced {
        /// This event's unique id (P3β-1).
        id: EventId,
        /// Parent event id — always `None` for a root building placement.
        parent: Option<EventId>,
        /// Building origin tile (top-left, in tiles).
        position: (u32, u32),
        /// Chebyshev influence radius in tiles (inclusive).
        radius: u32,
        /// Simulation tick at which BSS observed the event.
        tick: u64,
    },

    /// BSS marked a dirty region on `channel` after processing a building
    /// event. Recorded onto the region centre once per channel per event.
    ///
    /// Chain link: `parent = Some(<id of the originating BuildingPlaced>)`.
    StampDirty {
        /// This event's unique id (P3β-1).
        id: EventId,
        /// Parent event id — the `BuildingPlaced` that triggered this stamp.
        parent: Option<EventId>,
        /// Influence channel that received the dirty mark.
        channel: InfluenceChannel,
        /// Region drained by IUS in the same tick.
        region: DirtyRegion,
        /// Simulation tick at which the dirty mark was emitted.
        tick: u64,
    },

    /// IUS observed a non-empty dirty region for `channel` and propagated
    /// influence from `position` (region centre). `old`/`new` capture the
    /// current vs pending intensity at the centre after the propagation
    /// primitive ran — minimal sample per V7 Phase 3-α (the full per-cell
    /// delta would explode the ring; the centre sample suffices for the
    /// "왜?" UI which traces source ⇒ destination, not every tile in
    /// between).
    ///
    /// Chain link: `parent = Some(<id of the most recent same-channel
    /// StampDirty on this tile>)`. The lookup may return `None` if the
    /// matching `StampDirty` was already evicted from the centre tile's
    /// ring buffer — in that case the chain terminates here, which the
    /// "왜?" UI reports as "cause evicted".
    InfluenceChanged {
        /// This event's unique id (P3β-1).
        id: EventId,
        /// Parent event id — the matching-channel `StampDirty`, or `None`
        /// when that stamp has already been evicted from the ring.
        parent: Option<EventId>,
        /// Influence channel that propagated.
        channel: InfluenceChannel,
        /// Region centre tile (same sample IUS uses for the propagation
        /// primitive's source).
        position: (u32, u32),
        /// Intensity at `position` BEFORE this tick's propagation (pulled
        /// from `current[channel]`).
        old: f32,
        /// Intensity at `position` AFTER this tick's propagation (pulled
        /// from `pending[channel]`, before the IUS swap).
        new: f32,
        /// Simulation tick at which IUS recorded the change.
        tick: u64,
    },

    /// First agent-originated event (V7 Phase 5-β / P5β-3).
    ///
    /// Emitted by `AgentDecisionSystem` (priority 125) the moment an
    /// agent's [`Hunger`]/[`Thirst`] crosses its threshold and the
    /// agent transitions `Idle → Seeking { target }`. The variant is
    /// recorded on the agent's current tile so the "왜?" UI can surface
    /// "이 agent가 왜 이 방향으로 움직였나?" right where the agent stood
    /// when the decision was taken.
    ///
    /// Chain semantics: `AgentDecision { parent: Some(<recent same-tile
    /// InfluenceChanged>) }` when an influence event on the agent's tile
    /// preceded the breach (allowing the chain `BuildingPlaced →
    /// StampDirty → InfluenceChanged → AgentDecision` to close); `None`
    /// when no such causal antecedent exists.
    ///
    /// [`Hunger`]: crate::components::Hunger
    /// [`Thirst`]: crate::components::Thirst
    AgentDecision {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — most recent same-tile `InfluenceChanged`,
        /// or `None` for need-only breaches with no influence trigger.
        parent: Option<EventId>,
        /// The deciding agent's `AgentId` (P5α-2 surface).
        agent: AgentId,
        /// Agent tile at decision time.
        position: (u32, u32),
        /// Why the FSM transitioned out of `Idle`.
        reason: DecisionReason,
        /// Simulation tick at which the decision was made.
        tick: u64,
    },

    /// Agent transitioned from `Seeking { ConstructionSite }` to
    /// `Consuming { ConstructionSite }` (V7 Phase 6-β / P6β-3). Emitted by
    /// `AgentDecisionSystem` the moment an agent reaches an active
    /// construction site. `parent` links to the originating
    /// `AgentDecision { reason: ConstructionReason, agent: <this agent> }`
    /// on the same tile.
    ///
    /// `position` follows the `(u32, u32)` tuple precedent shared with
    /// `BuildingPlaced` and `AgentDecision`. `blueprint` snapshots the
    /// site's blueprint at start time (the site entity may be despawned
    /// before this record is consumed).
    ConstructionStarted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `AgentDecision { ConstructionReason }`.
        parent: Option<EventId>,
        /// Immutable design spec of the building being constructed.
        blueprint: crate::components::BuildingBlueprint,
        /// Construction tile (matches `ConstructionSite.position`).
        position: (u32, u32),
        /// Simulation tick at which the transition occurred.
        tick: u64,
    },

    /// Construction progress reached `required_progress` on this tick
    /// (V7 Phase 6-β / P6β-4). Emitted by `ConstructionSystem` BEFORE the
    /// closing `BuildingPlaced` so the parent chain
    /// `BuildingPlaced → ConstructionCompleted → ConstructionStarted →
    /// AgentDecision { ConstructionReason }` walks correctly.
    ///
    /// The site entity is despawned in the same tick — `position` is the
    /// durable identifier (the recycled hecs handle would be misleading
    /// after despawn), and the embedded `blueprint` mirrors
    /// `ConstructionStarted` exactly.
    ConstructionCompleted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `ConstructionStarted`.
        parent: Option<EventId>,
        /// Immutable design spec of the completed building.
        blueprint: crate::components::BuildingBlueprint,
        /// Construction tile (matches `ConstructionSite.position`).
        position: (u32, u32),
        /// Simulation tick at which completion fired.
        tick: u64,
    },

    /// Mutual social handshake closed: both agents transitioned from
    /// Seeking{Agent(other)} to Consuming{Agent(other)} on the same tile.
    /// Emitted ONCE per handshake by `AgentDecisionSystem` (deduplicated by
    /// emitting only from the smaller-AgentId agent's transition).
    /// `parent` links to the same-tile `AgentDecision{SocialReason,
    /// agent: emitter}`. V7 Phase 7-β / P7β-3.
    SocialInteractionStarted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the smaller-AgentId agent's `AgentDecision{
        /// SocialReason}`.
        parent: Option<EventId>,
        /// Canonical pair `(smaller AgentId, larger AgentId)`.
        agents: (AgentId, AgentId),
        /// Shared tile coordinate at handshake time.
        position: (u32, u32),
        /// Simulation tick at which the handshake closed.
        tick: u64,
    },

    /// Mutual social interaction reached `REQUIRED_INTERACTION_PROGRESS`.
    /// Emitted by `SocialInteractionSystem` before the agent state
    /// transitions back to Idle. `familiarity_after` snapshots the
    /// post-bump `RelationshipState::familiarity` value (saturating at
    /// 1.0). `parent` links to the originating `SocialInteractionStarted`.
    /// V7 Phase 7-β / P7β-4.
    SocialInteractionCompleted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `SocialInteractionStarted`.
        parent: Option<EventId>,
        /// Canonical pair `(smaller AgentId, larger AgentId)`.
        agents: (AgentId, AgentId),
        /// Shared tile coordinate at completion.
        position: (u32, u32),
        /// Familiarity scalar after `FAMILIARITY_BUMP` applied (clamped
        /// to `[0.0, 1.0]`).
        familiarity_after: f64,
        /// Simulation tick at which completion fired.
        tick: u64,
    },

    /// V7 Phase 8-β / P8Plan-6. Emitted by `AgentDecisionSystem` when a
    /// cascade-bias memory weight delta flips the cascade's natural
    /// winner. The `recalled_event` field references the top-contributor
    /// memory entry's `event_id`; `parent` carries the recalled event's
    /// own parent so the lineage walk continues through the recall.
    MemoryRecalled {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — typically the recalled event's parent, or
        /// `None` when the recalled event has no parent or has been
        /// evicted from the ring buffer.
        parent: Option<EventId>,
        /// The agent whose cascade was flipped.
        agent: AgentId,
        /// `event_id` of the top-contributor [`MemoryEntry`] driving the
        /// flip.
        ///
        /// [`MemoryEntry`]: crate::components::MemoryEntry
        recalled_event: EventId,
        /// Why the recall fired. Phase 8-β only emits `CascadeBias`.
        triggered_by: MemoryRecallTrigger,
        /// Simulation tick at which the recall was emitted.
        tick: u64,
    },

    /// Agent transitioned to `Consuming { Agent(defender) }` via the
    /// combat cascade arm. Emitted ONCE per pair by `AgentDecisionSystem`
    /// from the smaller-`AgentId` agent's evaluation (deduplication,
    /// mirrors `SocialInteractionStarted` pattern).
    /// `parent` links to the emitting agent's `AgentDecision{CombatReason}`.
    /// V7 Phase 9-β / P9β-1.
    CombatStarted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the attacker's `AgentDecision{CombatReason}`.
        parent: Option<EventId>,
        /// Agent initiating combat (the one whose cascade triggered).
        attacker: AgentId,
        /// Agent being attacked.
        defender: AgentId,
        /// Shared tile coordinate at combat start.
        position: (u32, u32),
        /// Simulation tick at which combat started.
        tick: u64,
    },

    /// `CombatSystem` applied `DAMAGE_PER_COMBAT_TICK` to the defender.
    /// `parent` links to the originating `CombatStarted`. `hp_after`
    /// snapshots the defender's HP after damage (saturates at 0.0).
    /// NOTE: field is `hp_after: f64`, NOT `defender_died: bool` — P9β-3.
    /// V7 Phase 9-β / P9β-1.
    CombatCompleted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `CombatStarted`.
        parent: Option<EventId>,
        /// Agent that initiated combat.
        attacker: AgentId,
        /// Agent that received damage.
        defender: AgentId,
        /// Shared tile coordinate at completion.
        position: (u32, u32),
        /// Defender HP after `apply_damage(DAMAGE_PER_COMBAT_TICK)`
        /// (saturated at 0.0). P9β-3: this is NOT `defender_died: bool`.
        hp_after: f64,
        /// Simulation tick at which damage was applied.
        tick: u64,
    },
}

impl CausalEvent {
    /// Unique id assigned at push time.
    pub fn id(&self) -> EventId {
        match self {
            CausalEvent::BuildingPlaced { id, .. }
            | CausalEvent::StampDirty { id, .. }
            | CausalEvent::InfluenceChanged { id, .. }
            | CausalEvent::AgentDecision { id, .. }
            | CausalEvent::ConstructionStarted { id, .. }
            | CausalEvent::ConstructionCompleted { id, .. }
            | CausalEvent::SocialInteractionStarted { id, .. }
            | CausalEvent::SocialInteractionCompleted { id, .. }
            | CausalEvent::MemoryRecalled { id, .. }
            | CausalEvent::CombatStarted { id, .. }
            | CausalEvent::CombatCompleted { id, .. } => *id,
        }
    }

    /// Returns the stored parent reference unchanged.
    ///
    /// `None` denotes a chain root (only `BuildingPlaced` is an
    /// uncondi­tional root; `AgentDecision` is a root when no influence
    /// antecedent exists); `Some(id)` is the parent id captured at push
    /// time. The stored field is immutable once written — eviction of
    /// the parent event from the ring buffer does NOT rewrite this
    /// field. Only the backward lookup performed by
    /// [`CausalLogStorage::trace_parents`](super::storage::CausalLogStorage::trace_parents)
    /// observes eviction (by terminating gracefully when the referenced
    /// id is no longer present on the tile).
    pub fn parent(&self) -> Option<EventId> {
        match self {
            CausalEvent::BuildingPlaced { parent, .. }
            | CausalEvent::StampDirty { parent, .. }
            | CausalEvent::InfluenceChanged { parent, .. }
            | CausalEvent::AgentDecision { parent, .. }
            | CausalEvent::ConstructionStarted { parent, .. }
            | CausalEvent::ConstructionCompleted { parent, .. }
            | CausalEvent::SocialInteractionStarted { parent, .. }
            | CausalEvent::SocialInteractionCompleted { parent, .. }
            | CausalEvent::MemoryRecalled { parent, .. }
            | CausalEvent::CombatStarted { parent, .. }
            | CausalEvent::CombatCompleted { parent, .. } => *parent,
        }
    }

    /// Simulation tick at which the event was recorded.
    pub fn tick(&self) -> u64 {
        match self {
            CausalEvent::BuildingPlaced { tick, .. }
            | CausalEvent::StampDirty { tick, .. }
            | CausalEvent::InfluenceChanged { tick, .. }
            | CausalEvent::AgentDecision { tick, .. }
            | CausalEvent::ConstructionStarted { tick, .. }
            | CausalEvent::ConstructionCompleted { tick, .. }
            | CausalEvent::SocialInteractionStarted { tick, .. }
            | CausalEvent::SocialInteractionCompleted { tick, .. }
            | CausalEvent::MemoryRecalled { tick, .. }
            | CausalEvent::CombatStarted { tick, .. }
            | CausalEvent::CombatCompleted { tick, .. } => *tick,
        }
    }

    /// Influence channel for stamp / influence variants. `None` for
    /// `BuildingPlaced` (channel-agnostic root event), `AgentDecision`
    /// (need-driven, not channel-driven), and the Phase 6-β
    /// `ConstructionStarted` / `ConstructionCompleted` variants (parallel
    /// to `BuildingPlaced`).
    pub fn channel(&self) -> Option<InfluenceChannel> {
        match self {
            CausalEvent::BuildingPlaced { .. }
            | CausalEvent::AgentDecision { .. }
            | CausalEvent::ConstructionStarted { .. }
            | CausalEvent::ConstructionCompleted { .. }
            | CausalEvent::SocialInteractionStarted { .. }
            | CausalEvent::SocialInteractionCompleted { .. }
            | CausalEvent::MemoryRecalled { .. }
            | CausalEvent::CombatStarted { .. }
            | CausalEvent::CombatCompleted { .. } => None,
            CausalEvent::StampDirty { channel, .. }
            | CausalEvent::InfluenceChanged { channel, .. } => Some(*channel),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_placed_records_fields() {
        let ev = CausalEvent::BuildingPlaced {
            id: 0,
            parent: None,
            position: (32, 32),
            radius: 12,
            tick: 7,
        };
        match ev {
            CausalEvent::BuildingPlaced { id, parent, position, radius, tick } => {
                assert_eq!(id, 0);
                assert_eq!(parent, None);
                assert_eq!(position, (32, 32));
                assert_eq!(radius, 12);
                assert_eq!(tick, 7);
            }
            _ => panic!("expected BuildingPlaced"),
        }
    }

    #[test]
    fn stamp_dirty_carries_region() {
        let ev = CausalEvent::StampDirty {
            id: 1,
            parent: Some(0),
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(10, 10, 20, 20),
            tick: 1,
        };
        if let CausalEvent::StampDirty { id, parent, channel, region, tick } = ev {
            assert_eq!(id, 1);
            assert_eq!(parent, Some(0));
            assert_eq!(channel, InfluenceChannel::Warmth);
            assert_eq!(region.min_x, 10);
            assert_eq!(region.max_x, 20);
            assert_eq!(tick, 1);
        } else {
            panic!("expected StampDirty");
        }
    }

    #[test]
    fn influence_changed_carries_delta() {
        let ev = CausalEvent::InfluenceChanged {
            id: 7,
            parent: Some(1),
            channel: InfluenceChannel::Spiritual,
            position: (32, 32),
            old: 0.0,
            new: 200.0,
            tick: 1,
        };
        if let CausalEvent::InfluenceChanged { id, parent, channel, old, new, .. } = ev {
            assert_eq!(id, 7);
            assert_eq!(parent, Some(1));
            assert_eq!(channel, InfluenceChannel::Spiritual);
            assert_eq!(old, 0.0);
            assert_eq!(new, 200.0);
        } else {
            panic!("expected InfluenceChanged");
        }
    }

    #[test]
    fn clone_preserves_equality() {
        let ev = CausalEvent::BuildingPlaced {
            id: 42,
            parent: None,
            position: (5, 5),
            radius: 2,
            tick: 42,
        };
        assert_eq!(ev, ev.clone());
    }

    #[test]
    fn accessors_round_trip() {
        let placed = CausalEvent::BuildingPlaced {
            id: 10,
            parent: None,
            position: (1, 1),
            radius: 1,
            tick: 3,
        };
        assert_eq!(placed.id(), 10);
        assert_eq!(placed.parent(), None);
        assert_eq!(placed.tick(), 3);
        assert_eq!(placed.channel(), None);

        let stamp = CausalEvent::StampDirty {
            id: 11,
            parent: Some(10),
            channel: InfluenceChannel::Noise,
            region: DirtyRegion::new(0, 0, 1, 1),
            tick: 3,
        };
        assert_eq!(stamp.id(), 11);
        assert_eq!(stamp.parent(), Some(10));
        assert_eq!(stamp.channel(), Some(InfluenceChannel::Noise));

        let influence = CausalEvent::InfluenceChanged {
            id: 12,
            parent: Some(11),
            channel: InfluenceChannel::Noise,
            position: (1, 1),
            old: 0.0,
            new: 100.0,
            tick: 3,
        };
        assert_eq!(influence.id(), 12);
        assert_eq!(influence.parent(), Some(11));
        assert_eq!(influence.channel(), Some(InfluenceChannel::Noise));
    }

    #[test]
    fn construction_started_records_fields() {
        use crate::components::BuildingBlueprint;
        let bp = BuildingBlueprint::new(7, 2, 2, 5);
        let ev = CausalEvent::ConstructionStarted {
            id: 200,
            parent: Some(50),
            blueprint: bp,
            position: (3, 4),
            tick: 17,
        };
        match ev {
            CausalEvent::ConstructionStarted {
                id,
                parent,
                blueprint,
                position,
                tick,
            } => {
                assert_eq!(id, 200);
                assert_eq!(parent, Some(50));
                assert_eq!(blueprint, bp);
                assert_eq!(position, (3, 4));
                assert_eq!(tick, 17);
            }
            _ => panic!("expected ConstructionStarted"),
        }
    }

    #[test]
    fn construction_completed_records_fields() {
        use crate::components::BuildingBlueprint;
        let bp = BuildingBlueprint::new(8, 3, 3, 7);
        let ev = CausalEvent::ConstructionCompleted {
            id: 201,
            parent: Some(200),
            blueprint: bp,
            position: (9, 11),
            tick: 23,
        };
        match ev {
            CausalEvent::ConstructionCompleted {
                id,
                parent,
                blueprint,
                position,
                tick,
            } => {
                assert_eq!(id, 201);
                assert_eq!(parent, Some(200));
                assert_eq!(blueprint, bp);
                assert_eq!(position, (9, 11));
                assert_eq!(tick, 23);
            }
            _ => panic!("expected ConstructionCompleted"),
        }
    }

    #[test]
    fn decision_reason_construction_as_str() {
        assert_eq!(
            DecisionReason::ConstructionReason.as_str(),
            "construction_reason"
        );
    }

    #[test]
    fn decision_reason_social_as_str() {
        // V7 Phase 7-β / P7β-5 — `SocialReason` discriminator locked at
        // `"social_reason"` (snake_case, matches the existing style).
        assert_eq!(DecisionReason::SocialReason.as_str(), "social_reason");
    }

    #[test]
    fn social_interaction_started_records_fields() {
        // V7 Phase 7-β / P7β-3 — locked field shapes:
        //   id: EventId, parent: Option<EventId>, agents: (AgentId, AgentId),
        //   position: (u32, u32), tick: u64.
        let ev = CausalEvent::SocialInteractionStarted {
            id: 300,
            parent: Some(42),
            agents: (3u64, 5u64),
            position: (7u32, 9u32),
            tick: 11,
        };
        match ev.clone() {
            CausalEvent::SocialInteractionStarted {
                id,
                parent,
                agents,
                position,
                tick,
            } => {
                assert_eq!(id, 300);
                assert_eq!(parent, Some(42));
                assert_eq!(agents, (3u64, 5u64));
                assert_eq!(position, (7u32, 9u32));
                assert_eq!(tick, 11);
            }
            _ => panic!("expected SocialInteractionStarted"),
        }
        // Accessors round-trip and channel() is None (channel-agnostic).
        assert_eq!(ev.id(), 300);
        assert_eq!(ev.parent(), Some(42));
        assert_eq!(ev.tick(), 11);
        assert_eq!(ev.channel(), None);
    }

    #[test]
    fn social_interaction_completed_records_fields() {
        // V7 Phase 7-β / P7β-4 — locked field shapes add `familiarity_after: f64`.
        let ev = CausalEvent::SocialInteractionCompleted {
            id: 301,
            parent: Some(300),
            agents: (3u64, 5u64),
            position: (7u32, 9u32),
            familiarity_after: 0.37,
            tick: 14,
        };
        match ev.clone() {
            CausalEvent::SocialInteractionCompleted {
                id,
                parent,
                agents,
                position,
                familiarity_after,
                tick,
            } => {
                assert_eq!(id, 301);
                assert_eq!(parent, Some(300));
                assert_eq!(agents, (3u64, 5u64));
                assert_eq!(position, (7u32, 9u32));
                assert_eq!(familiarity_after, 0.37);
                assert_eq!(tick, 14);
            }
            _ => panic!("expected SocialInteractionCompleted"),
        }
        assert_eq!(ev.id(), 301);
        assert_eq!(ev.parent(), Some(300));
        assert_eq!(ev.tick(), 14);
        assert_eq!(ev.channel(), None);
    }

    #[test]
    fn agent_decision_accessors_and_reasons() {
        let ev = CausalEvent::AgentDecision {
            id: 100,
            parent: Some(12),
            agent: 7,
            position: (4, 9),
            reason: DecisionReason::HungerThresholdBreach,
            tick: 42,
        };
        assert_eq!(ev.id(), 100);
        assert_eq!(ev.parent(), Some(12));
        assert_eq!(ev.tick(), 42);
        assert_eq!(ev.channel(), None);

        // Reason discriminator round-trip.
        assert_eq!(
            DecisionReason::HungerThresholdBreach.as_str(),
            "hunger_threshold_breach"
        );
        assert_eq!(
            DecisionReason::ThirstThresholdBreach.as_str(),
            "thirst_threshold_breach"
        );
        assert_eq!(
            DecisionReason::FatigueThresholdBreach.as_str(),
            "fatigue_threshold_breach"
        );

        // Root agent decision (no influence antecedent).
        let root = CausalEvent::AgentDecision {
            id: 101,
            parent: None,
            agent: 8,
            position: (0, 0),
            reason: DecisionReason::ThirstThresholdBreach,
            tick: 43,
        };
        assert_eq!(root.parent(), None);
    }
}
