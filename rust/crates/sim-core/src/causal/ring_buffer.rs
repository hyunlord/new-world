//! Per-tile causal event ring buffer with FIFO eviction.
//!
//! V7 Phase 3-α (P3α-2): each tile that observes any causal event owns
//! a fixed-capacity ring of 8 [`CausalEvent`]s. When a 9th event arrives,
//! the OLDEST entry is dropped (front-eviction) and the new event is
//! appended at the back. The fixed capacity bounds worst-case memory
//! without needing a per-tile growth policy.
//!
//! Capacity selection — 8 matches the design budget:
//! - typical "왜?" UI consultation depth (1 building placed → 6 channels
//!   stamped → 1 IUS tick = 8 events) fits exactly without truncation;
//! - 8 × ~56 bytes per [`CausalEvent`] = ~448 bytes per active tile;
//! - sparse storage caps active tiles at the influence event surface
//!   (typically <500 tiles), keeping the worst case well under 1 MB.

use arrayvec::ArrayVec;

use super::event::CausalEvent;

/// Maximum number of causal events retained per tile.
///
/// Locked by P3α-2. Adjusting this constant requires a corresponding
/// memory-budget audit (see crate doc above).
pub const TILE_CAUSAL_RING_SIZE: usize = 8;

/// FIFO ring of [`CausalEvent`]s for a single tile.
///
/// Insertion semantics: when the ring is full, the front entry is
/// evicted before pushing the new event at the back. This preserves the
/// MOST RECENT 8 events — the relevant set for the "왜?" UI which
/// answers "why is THIS tile in THIS state RIGHT NOW?".
#[derive(Debug, Default, Clone)]
pub struct TileCausalLog {
    events: ArrayVec<CausalEvent, TILE_CAUSAL_RING_SIZE>,
}

impl TileCausalLog {
    /// Construct an empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `event`, evicting the oldest entry if the ring is full.
    ///
    /// O(N) worst case due to the `remove(0)` shift, but N is bounded by
    /// [`TILE_CAUSAL_RING_SIZE`] = 8 so this is effectively constant.
    pub fn push(&mut self, event: CausalEvent) {
        if self.events.is_full() {
            // FIFO eviction — drop the oldest event.
            self.events.remove(0);
        }
        self.events.push(event);
    }

    /// Number of events currently retained (0..=[`TILE_CAUSAL_RING_SIZE`]).
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// `true` when no events have been recorded for this tile.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Iterate events in insertion order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &CausalEvent> {
        self.events.iter()
    }

    /// Borrow the events as a slice (oldest first).
    pub fn as_slice(&self) -> &[CausalEvent] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::influence::{DirtyRegion, InfluenceChannel};

    fn placed(tick: u64) -> CausalEvent {
        CausalEvent::BuildingPlaced {
            position: (0, 0),
            radius: 1,
            tick,
        }
    }

    fn stamp(tick: u64) -> CausalEvent {
        CausalEvent::StampDirty {
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(0, 0, 1, 1),
            tick,
        }
    }

    #[test]
    fn ring_size_is_eight() {
        assert_eq!(TILE_CAUSAL_RING_SIZE, 8);
    }

    #[test]
    fn new_log_is_empty() {
        let log = TileCausalLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn push_appends_to_back() {
        let mut log = TileCausalLog::new();
        log.push(placed(1));
        log.push(placed(2));
        assert_eq!(log.len(), 2);
        let ticks: Vec<u64> = log
            .iter()
            .map(|e| match e {
                CausalEvent::BuildingPlaced { tick, .. } => *tick,
                _ => 0,
            })
            .collect();
        assert_eq!(ticks, vec![1, 2]);
    }

    #[test]
    fn fifo_eviction_at_capacity() {
        let mut log = TileCausalLog::new();
        // Fill ring with ticks 1..=8.
        for t in 1..=8u64 {
            log.push(placed(t));
        }
        assert_eq!(log.len(), TILE_CAUSAL_RING_SIZE);

        // 9th push evicts the oldest (tick 1) and appends tick 9.
        log.push(placed(9));
        assert_eq!(log.len(), TILE_CAUSAL_RING_SIZE);

        let ticks: Vec<u64> = log
            .iter()
            .map(|e| match e {
                CausalEvent::BuildingPlaced { tick, .. } => *tick,
                _ => 0,
            })
            .collect();
        assert_eq!(ticks, vec![2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn mixed_variants_share_ring() {
        let mut log = TileCausalLog::new();
        log.push(placed(1));
        log.push(stamp(1));
        assert_eq!(log.len(), 2);
        assert!(matches!(log.as_slice()[0], CausalEvent::BuildingPlaced { .. }));
        assert!(matches!(log.as_slice()[1], CausalEvent::StampDirty { .. }));
    }

    #[test]
    fn ring_never_exceeds_capacity() {
        let mut log = TileCausalLog::new();
        for t in 0..32u64 {
            log.push(placed(t));
        }
        assert_eq!(log.len(), TILE_CAUSAL_RING_SIZE);
    }
}
