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
        }
    }
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
            | CausalEvent::ConstructionCompleted { id, .. } => *id,
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
            | CausalEvent::ConstructionCompleted { parent, .. } => *parent,
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
            | CausalEvent::ConstructionCompleted { tick, .. } => *tick,
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
            | CausalEvent::ConstructionCompleted { .. } => None,
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
