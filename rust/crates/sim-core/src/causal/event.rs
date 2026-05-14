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
//! `Copy` is intentionally NOT derived: [`DirtyRegion`] is `Clone` only,
//! so [`CausalEvent`] inherits the same constraint. Callers move/clone
//! events explicitly rather than rely on a bit-copy.

use crate::influence::{DirtyRegion, InfluenceChannel};

/// A single cause-effect record stamped into a [`TileCausalLog`].
///
/// Phase 3-α scope (locked by P3α-1): exactly three variants — building
/// placement, BSS dirty-region stamp, and IUS propagation tick. Future
/// phases may add `AgentDecision`, `Combat`, etc. — but the ring size
/// stays 8 (see [`TILE_CAUSAL_RING_SIZE`]).
///
/// [`TileCausalLog`]: super::TileCausalLog
/// [`TILE_CAUSAL_RING_SIZE`]: super::TILE_CAUSAL_RING_SIZE
#[derive(Debug, Clone, PartialEq)]
pub enum CausalEvent {
    /// A building placement event arrived from FFI and was accepted by BSS
    /// (in-bounds, non-zero radius). Recorded once per accepted event onto
    /// the centre tile so the "왜?" UI can attribute downstream stamps.
    BuildingPlaced {
        /// Building origin tile (top-left, in tiles).
        position: (u32, u32),
        /// Chebyshev influence radius in tiles (inclusive).
        radius: u32,
        /// Simulation tick at which BSS observed the event.
        tick: u64,
    },

    /// BSS marked a dirty region on `channel` after processing a building
    /// event. Recorded onto the region centre once per channel per event.
    StampDirty {
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
    InfluenceChanged {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_placed_records_fields() {
        let ev = CausalEvent::BuildingPlaced {
            position: (32, 32),
            radius: 12,
            tick: 7,
        };
        match ev {
            CausalEvent::BuildingPlaced { position, radius, tick } => {
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
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(10, 10, 20, 20),
            tick: 1,
        };
        if let CausalEvent::StampDirty { channel, region, tick } = ev {
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
            channel: InfluenceChannel::Spiritual,
            position: (32, 32),
            old: 0.0,
            new: 200.0,
            tick: 1,
        };
        if let CausalEvent::InfluenceChanged { channel, old, new, .. } = ev {
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
            position: (5, 5),
            radius: 2,
            tick: 42,
        };
        assert_eq!(ev, ev.clone());
    }
}
