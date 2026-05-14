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

use crate::influence::{DirtyRegion, InfluenceChannel};

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
}

impl CausalEvent {
    /// Unique id assigned at push time.
    pub fn id(&self) -> EventId {
        match self {
            CausalEvent::BuildingPlaced { id, .. }
            | CausalEvent::StampDirty { id, .. }
            | CausalEvent::InfluenceChanged { id, .. } => *id,
        }
    }

    /// Returns the stored parent reference unchanged.
    ///
    /// `None` denotes a chain root (only `BuildingPlaced` is a root in
    /// P3-β); `Some(id)` is the parent id captured at push time. The
    /// stored field is immutable once written — eviction of the parent
    /// event from the ring buffer does NOT rewrite this field. Only the
    /// backward lookup performed by
    /// [`CausalLogStorage::trace_parents`](super::storage::CausalLogStorage::trace_parents)
    /// observes eviction (by terminating gracefully when the referenced
    /// id is no longer present on the tile).
    pub fn parent(&self) -> Option<EventId> {
        match self {
            CausalEvent::BuildingPlaced { parent, .. }
            | CausalEvent::StampDirty { parent, .. }
            | CausalEvent::InfluenceChanged { parent, .. } => *parent,
        }
    }

    /// Simulation tick at which the event was recorded.
    pub fn tick(&self) -> u64 {
        match self {
            CausalEvent::BuildingPlaced { tick, .. }
            | CausalEvent::StampDirty { tick, .. }
            | CausalEvent::InfluenceChanged { tick, .. } => *tick,
        }
    }

    /// Influence channel for stamp / influence variants. `None` for
    /// `BuildingPlaced` (channel-agnostic root event).
    pub fn channel(&self) -> Option<InfluenceChannel> {
        match self {
            CausalEvent::BuildingPlaced { .. } => None,
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
}
