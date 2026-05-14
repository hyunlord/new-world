//! Sparse per-tile causal log storage.
//!
//! V7 Phase 3-α (P3α-3): event-bearing tiles are stored sparsely in a
//! `HashMap<tile_idx, TileCausalLog>`. Tiles that never observe a causal
//! event do NOT consume memory. The key matches [`InfluenceGrid::idx`]
//! (`y * width + x`) so the same lookup formula serves both surfaces.
//!
//! Worst-case memory budget (locked by P3α-3):
//! - Dense alternative: 1024 × 1024 tiles × 448 bytes = 470 MB → REJECTED.
//! - Sparse choice: typical Phase 3 surface is <500 active tiles → ~225 KB.
//!   Stress upper bound (Phase 4 with agent decisions) ≈ 28K active tiles
//!   → ~12.6 MB — well within the design ceiling.

use std::collections::HashMap;

use super::event::CausalEvent;
use super::ring_buffer::TileCausalLog;

/// Sparse storage for per-tile causal logs.
///
/// Indexed by linear tile index (`y * width + x`). Lookups for tiles that
/// have never observed a causal event return `None` / `0` without
/// allocating an empty log.
#[derive(Debug, Default, Clone)]
pub struct CausalLogStorage {
    logs: HashMap<u32, TileCausalLog>,
}

impl CausalLogStorage {
    /// Construct an empty storage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `event` to the log for `tile_idx`, allocating the log lazily
    /// on first write. The log's FIFO eviction policy keeps memory bounded
    /// per active tile (see [`TileCausalLog`]).
    pub fn push(&mut self, tile_idx: u32, event: CausalEvent) {
        self.logs.entry(tile_idx).or_default().push(event);
    }

    /// Borrow the log for `tile_idx`, or `None` if the tile has never
    /// observed an event.
    pub fn get(&self, tile_idx: u32) -> Option<&TileCausalLog> {
        self.logs.get(&tile_idx)
    }

    /// Mutable borrow of the log for `tile_idx`, or `None` if absent.
    pub fn get_mut(&mut self, tile_idx: u32) -> Option<&mut TileCausalLog> {
        self.logs.get_mut(&tile_idx)
    }

    /// Number of tiles with at least one recorded event (the size of the
    /// sparse working set).
    pub fn active_tile_count(&self) -> usize {
        self.logs.len()
    }

    /// `true` when no tile has any recorded event.
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }

    /// Iterate `(tile_idx, &TileCausalLog)` pairs in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &TileCausalLog)> {
        self.logs.iter()
    }

    /// Drop every log. Used by `SimEngine::reset` / tests that need a
    /// clean slate without recreating the engine.
    pub fn clear(&mut self) {
        self.logs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::influence::InfluenceChannel;

    fn placed(tick: u64) -> CausalEvent {
        CausalEvent::BuildingPlaced {
            position: (0, 0),
            radius: 1,
            tick,
        }
    }

    fn influence(tick: u64) -> CausalEvent {
        CausalEvent::InfluenceChanged {
            channel: InfluenceChannel::Warmth,
            position: (0, 0),
            old: 0.0,
            new: 200.0,
            tick,
        }
    }

    #[test]
    fn new_storage_is_empty() {
        let s = CausalLogStorage::new();
        assert!(s.is_empty());
        assert_eq!(s.active_tile_count(), 0);
        assert!(s.get(0).is_none());
    }

    #[test]
    fn push_allocates_lazily() {
        let mut s = CausalLogStorage::new();
        // Only tile 42 receives an event — others stay absent.
        s.push(42, placed(1));
        assert_eq!(s.active_tile_count(), 1);
        assert!(s.get(0).is_none());
        assert_eq!(s.get(42).unwrap().len(), 1);
    }

    #[test]
    fn multi_event_single_tile() {
        let mut s = CausalLogStorage::new();
        s.push(7, placed(1));
        s.push(7, influence(1));
        s.push(7, placed(2));
        // Still one active tile (7), with 3 events.
        assert_eq!(s.active_tile_count(), 1);
        assert_eq!(s.get(7).unwrap().len(), 3);
    }

    #[test]
    fn multi_tile_isolation() {
        let mut s = CausalLogStorage::new();
        s.push(1, placed(1));
        s.push(2, placed(2));
        s.push(3, placed(3));
        assert_eq!(s.active_tile_count(), 3);
        for idx in [1, 2, 3] {
            assert_eq!(s.get(idx).unwrap().len(), 1);
        }
    }

    #[test]
    fn fifo_eviction_per_tile() {
        let mut s = CausalLogStorage::new();
        // Push 10 events onto tile 0 — log caps at 8 (FIFO eviction).
        for t in 1..=10u64 {
            s.push(0, placed(t));
        }
        assert_eq!(s.active_tile_count(), 1);
        assert_eq!(s.get(0).unwrap().len(), 8);
    }

    #[test]
    fn clear_resets_all_tiles() {
        let mut s = CausalLogStorage::new();
        s.push(1, placed(1));
        s.push(2, placed(2));
        s.clear();
        assert!(s.is_empty());
        assert!(s.get(1).is_none());
    }
}
