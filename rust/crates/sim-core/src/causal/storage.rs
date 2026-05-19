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

use super::event::{CausalEvent, EventId};
use super::ring_buffer::TileCausalLog;
use crate::influence::InfluenceChannel;

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

    /// Look up an event by its [`EventId`] across all per-tile ring buffers.
    ///
    /// V7 Phase 8-β substrate (P8β-NEW-2 + P8β-MOD-2). Used by
    /// `MemorySystem::classify_event` for Construction parent walks and
    /// by `AgentDecisionSystem::event_id_matches_arm` for memory-bias
    /// weight scoring.
    ///
    /// Complexity: O(N_tiles × RING_SIZE) where RING_SIZE = 8 — bounded
    /// and small in practice. Returns `None` if the event has been
    /// evicted (Phase 3-β graceful eviction precedent) or was never
    /// recorded.
    pub fn lookup(&self, event_id: EventId) -> Option<&CausalEvent> {
        self.logs
            .values()
            .flat_map(|log| log.iter())
            .find(|e| e.id() == event_id)
    }

    /// Find the most recent `StampDirty` event id on `tile_idx` that matches
    /// `channel`, scanning the ring from newest to oldest.
    ///
    /// V7 Phase 3-β (P3β-3 / P3β-4): used by IUS to attach each
    /// `InfluenceChanged` record to the same-channel `StampDirty` that
    /// produced its dirty region. Returns `None` when the tile has no log
    /// or the matching stamp has already been evicted — the chain
    /// terminates gracefully in that case (the "왜?" UI reports
    /// "cause evicted").
    pub fn find_recent_stamp_dirty(
        &self,
        tile_idx: u32,
        channel: InfluenceChannel,
    ) -> Option<EventId> {
        let log = self.logs.get(&tile_idx)?;
        log.as_slice().iter().rev().find_map(|ev| match ev {
            CausalEvent::StampDirty {
                id,
                channel: ch,
                ..
            } if *ch == channel => Some(*id),
            _ => None,
        })
    }

    /// Walk the parent chain backwards from `event_id` on `tile_idx`,
    /// returning references in `[child, parent, grand-parent, …]` order.
    ///
    /// V7 Phase 3-β (P3β-4): the "왜?" UI consumes this to render the
    /// `InfluenceChanged → StampDirty → BuildingPlaced` lineage. The walk
    /// terminates when:
    /// 1. the current event has `parent == None` (root), or
    /// 2. the parent id is not present in the same tile's ring (evicted).
    ///
    /// The returned chain may be shorter than three entries when eviction
    /// happens — that is the expected graceful-termination behaviour.
    pub fn trace_parents(&self, tile_idx: u32, event_id: EventId) -> Vec<&CausalEvent> {
        let mut chain = Vec::new();
        let Some(log) = self.logs.get(&tile_idx) else {
            return chain;
        };
        let mut cursor: Option<EventId> = Some(event_id);
        while let Some(id) = cursor {
            let Some(ev) = log.as_slice().iter().find(|e| e.id() == id) else {
                break;
            };
            chain.push(ev);
            cursor = ev.parent();
        }
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::influence::{DirtyRegion, InfluenceChannel};

    fn placed(tick: u64) -> CausalEvent {
        CausalEvent::BuildingPlaced {
            id: 0,
            parent: None,
            position: (0, 0),
            radius: 1,
            tick,
        }
    }

    fn influence(tick: u64) -> CausalEvent {
        CausalEvent::InfluenceChanged {
            id: 0,
            parent: None,
            channel: InfluenceChannel::Warmth,
            position: (0, 0),
            old: 0.0,
            new: 200.0,
            tick,
        }
    }

    fn placed_id(id: EventId, tick: u64) -> CausalEvent {
        CausalEvent::BuildingPlaced {
            id,
            parent: None,
            position: (0, 0),
            radius: 1,
            tick,
        }
    }

    fn stamp_id(id: EventId, parent: EventId, channel: InfluenceChannel, tick: u64) -> CausalEvent {
        CausalEvent::StampDirty {
            id,
            parent: Some(parent),
            channel,
            region: DirtyRegion::new(0, 0, 1, 1),
            tick,
        }
    }

    fn influence_id(
        id: EventId,
        parent: EventId,
        channel: InfluenceChannel,
        tick: u64,
    ) -> CausalEvent {
        CausalEvent::InfluenceChanged {
            id,
            parent: Some(parent),
            channel,
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

    #[test]
    fn find_recent_stamp_dirty_returns_newest_match() {
        let mut s = CausalLogStorage::new();
        s.push(5, placed_id(0, 0));
        s.push(5, stamp_id(1, 0, InfluenceChannel::Warmth, 0));
        s.push(5, stamp_id(2, 0, InfluenceChannel::Noise, 0));
        s.push(5, stamp_id(3, 0, InfluenceChannel::Warmth, 1));
        // Newest Warmth stamp is id=3.
        assert_eq!(
            s.find_recent_stamp_dirty(5, InfluenceChannel::Warmth),
            Some(3)
        );
        // Noise stamp still discoverable.
        assert_eq!(
            s.find_recent_stamp_dirty(5, InfluenceChannel::Noise),
            Some(2)
        );
        // Channel never stamped → None.
        assert_eq!(
            s.find_recent_stamp_dirty(5, InfluenceChannel::Light),
            None
        );
        // Tile without log → None.
        assert_eq!(
            s.find_recent_stamp_dirty(999, InfluenceChannel::Warmth),
            None
        );
    }

    #[test]
    fn trace_parents_walks_chain() {
        let mut s = CausalLogStorage::new();
        // Chain: BuildingPlaced(0) ← StampDirty(1) ← InfluenceChanged(2).
        s.push(7, placed_id(0, 0));
        s.push(7, stamp_id(1, 0, InfluenceChannel::Warmth, 0));
        s.push(7, influence_id(2, 1, InfluenceChannel::Warmth, 0));
        let chain = s.trace_parents(7, 2);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id(), 2);
        assert_eq!(chain[1].id(), 1);
        assert_eq!(chain[2].id(), 0);
        // Root has no parent.
        assert_eq!(chain[2].parent(), None);
    }

    #[test]
    fn trace_parents_terminates_on_evicted_parent() {
        let mut s = CausalLogStorage::new();
        // Influence references parent stamp id=99 which was never inserted.
        s.push(3, influence_id(5, 99, InfluenceChannel::Warmth, 0));
        let chain = s.trace_parents(3, 5);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].id(), 5);
    }

    #[test]
    fn trace_parents_missing_tile_returns_empty() {
        let s = CausalLogStorage::new();
        assert!(s.trace_parents(42, 0).is_empty());
    }
}
