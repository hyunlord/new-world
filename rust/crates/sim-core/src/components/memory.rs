//! V7 Phase 8-α — per-agent episodic memory substrate.
//!
//! Phase 8 anchor: Memory System. See `.harness/audit/section_9_plus_design.md`.
//! Sub-stage 8-α: data substrate only. Runtime system (`MemorySystem` at
//! priority 136), `CausalEvent::MemoryRecalled` variant, `DecisionReason::
//! MemoryReason` variant, and the `AgentDecisionSystem` 6th-cascade bias
//! mechanism are all Phase 8-β scope.
//!
//! Capacity policy (P8Plan-5): hard cap of [`MEMORY_CAP`] entries (32), with
//! lowest-salience eviction on overflow. Tie-break: oldest `encoded_tick`.
//! Mirrors Phase 3-β's `TILE_CAUSAL_RING_SIZE = 8` substrate symmetry — the
//! per-agent ring is 4× the per-tile ring because agents accumulate
//! memories from many tiles.
//!
//! Decay shape (P8Plan-2): linear per-tick decay applied by `MemorySystem`
//! in Phase 8-β. This module only exposes the `decay_one_tick(rate)` API;
//! the rate constant (`DECAY_RATE`) is owned by `memory_system.rs` in
//! sim-systems and not exported here.

use serde::{Deserialize, Serialize};

use crate::causal::event::EventId;

/// Per-agent maximum number of [`MemoryEntry`] records retained.
///
/// Bounded by Phase 3-β substrate symmetry (`TILE_CAUSAL_RING_SIZE = 8`)
/// scaled 4× for per-agent vs per-tile accumulation. At 10K agents the
/// total memory store is bounded by `10_000 * 32 * size_of::<MemoryEntry>()`
/// ≈ 12.8 MB.
///
/// Plan ref: phase8.md §2 P8Plan-5.
pub const MEMORY_CAP: usize = 32;

/// Salience threshold below which a memory entry becomes eligible for
/// eviction during the next overflow-driven insert.
///
/// Phase 8-α exposes the constant; Phase 8-β's `MemorySystem` consumes it
/// during the decay pass to mark "forgettable" entries. In Phase 8-α the
/// eviction policy is "lowest salience first" unconditionally — the floor
/// is not enforced here; it exists for `MemorySystem` to derive a
/// "salience below floor → eviction-preferred" signal.
///
/// Plan ref: phase8.md §2 P8Plan-5.
pub const SALIENCE_FLOOR: f64 = 0.05;

/// A single episodic memory entry: a reference to a past `CausalEvent` plus
/// per-agent metadata that the global causal log does not store.
///
/// Fields (P8Plan-1 lock):
/// - `event_id`: the [`EventId`] of the originating causal event. May
///   reference an event whose ring-buffer slot has been evicted; lookup
///   sites must handle the miss gracefully (Phase 3-β precedent).
/// - `encoded_tick`: the simulation tick at which this memory was encoded.
///   Used for recency-weighted scoring in Phase 8-β and as the tie-break
///   in lowest-salience eviction (oldest first).
/// - `valence`: emotional weight in `[-1.0, 1.0]`. Negative = unpleasant;
///   positive = pleasant.
/// - `salience`: current strength in `[0.0, 1.0]`. Initialised at encode
///   time; decays per tick via `decay_one_tick`; reinforced on recall via
///   `reinforce`.
/// - `reinforcement_count`: monotone counter (saturating at `u32::MAX`)
///   of recall events.
///
/// Plan ref: phase8.md §2 P8Plan-1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Originating [`CausalEvent`](crate::causal::event::CausalEvent)
    /// identifier. May reference an evicted event — lookup sites must
    /// handle the miss gracefully.
    pub event_id: EventId,
    /// Simulation tick at which this memory was encoded. Used as the
    /// tie-break in lowest-salience eviction (oldest first).
    pub encoded_tick: u64,
    /// Emotional weight in `[-1.0, 1.0]`. Negative = unpleasant;
    /// positive = pleasant. Clamped at construction by
    /// [`MemoryEntry::new`].
    pub valence: f64,
    /// Current strength in `[0.0, 1.0]`. Decays per tick via
    /// [`Memory::decay_one_tick`]; reinforced on recall via
    /// [`Memory::reinforce`]. Clamped at construction by
    /// [`MemoryEntry::new`].
    pub salience: f64,
    /// Monotone counter (saturating at `u32::MAX`) of recall events.
    /// Incremented by [`Memory::reinforce`].
    pub reinforcement_count: u32,
}

impl MemoryEntry {
    /// Construct an entry. Inputs are clamped to their respective ranges
    /// (`valence ∈ [-1.0, 1.0]`, `salience ∈ [0.0, 1.0]`).
    /// `reinforcement_count` starts at `0`.
    pub fn new(event_id: EventId, encoded_tick: u64, valence: f64, salience: f64) -> Self {
        Self {
            event_id,
            encoded_tick,
            valence: valence.clamp(-1.0, 1.0),
            salience: salience.clamp(0.0, 1.0),
            reinforcement_count: 0,
        }
    }
}

/// Per-agent bounded ring of episodic [`MemoryEntry`] records.
///
/// Storage: `Vec<MemoryEntry>` with `capacity = MEMORY_CAP` reserved at
/// construction. Overflow policy: lowest-salience eviction with
/// oldest-`encoded_tick` tie-break (in-place replace, no allocation).
///
/// The `Memory` type is NOT `Copy` (Vec backing). It IS `Clone` —
/// at 32 entries it is acceptable for snapshot capture.
///
/// Plan ref: phase8.md §2 P8Plan-1 + §3 Phase 8-α.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// Bounded by [`MEMORY_CAP`]. Maintained as an unsorted Vec; ordering
    /// is irrelevant — the lowest-salience eviction scan is O(n) on a
    /// capacity-32 collection.
    pub entries: Vec<MemoryEntry>,
}

impl Memory {
    /// Construct an empty memory with `MEMORY_CAP`-sized backing capacity
    /// reserved (avoids re-allocation under the first 32 inserts).
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MEMORY_CAP),
        }
    }

    /// Insert an entry, applying capacity policy:
    /// - If `entries.len() < MEMORY_CAP`: push.
    /// - Otherwise: find the lowest-salience entry (tie-break: oldest
    ///   `encoded_tick`), replace it in place. No allocation, O(n) scan.
    pub fn insert(&mut self, entry: MemoryEntry) {
        if self.entries.len() < MEMORY_CAP {
            self.entries.push(entry);
            return;
        }
        // Overflow: find lowest-salience (tie-break: oldest encoded_tick),
        // replace in place. The `.expect()` is provably unreachable —
        // `entries.len() == MEMORY_CAP >= 1` at this point.
        let evict_idx = self
            .entries
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.salience
                    .partial_cmp(&b.salience)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.encoded_tick.cmp(&b.encoded_tick))
            })
            .map(|(i, _)| i)
            .expect("entries is non-empty because len() == MEMORY_CAP");
        self.entries[evict_idx] = entry;
    }

    /// Linearly decay every entry's salience by `rate`, saturating at 0.0.
    /// Phase 8-β calls this once per tick from `MemorySystem`. Phase 8-α
    /// exposes the operation; the rate constant (`DECAY_RATE = 0.001`)
    /// is owned by `memory_system.rs`.
    pub fn decay_one_tick(&mut self, rate: f64) {
        for entry in &mut self.entries {
            entry.salience = (entry.salience - rate).max(0.0);
        }
    }

    /// Boost a single entry's salience (saturating at 1.0) and increment
    /// its reinforcement counter (saturating at `u32::MAX`).
    ///
    /// Returns `true` if the index was valid; `false` otherwise (no-op
    /// for out-of-bounds indices).
    pub fn reinforce(&mut self, idx: usize, boost: f64) -> bool {
        if let Some(entry) = self.entries.get_mut(idx) {
            entry.salience = (entry.salience + boost).min(1.0);
            entry.reinforcement_count = entry.reinforcement_count.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Linear scan for the first entry whose `event_id` matches.
    /// Returns the entry's index (for use with `reinforce`) or `None`.
    /// O(n) over a capacity-32 collection.
    pub fn find_by_event_id(&self, event_id: EventId) -> Option<usize> {
        self.entries.iter().position(|e| e.event_id == event_id)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(event_id: EventId, tick: u64, valence: f64, salience: f64) -> MemoryEntry {
        MemoryEntry::new(event_id, tick, valence, salience)
    }

    #[test]
    fn entry_construction_clamps_valence_and_salience() {
        let e = MemoryEntry::new(1, 0, 2.0, 5.0);
        assert_eq!(e.valence, 1.0);
        assert_eq!(e.salience, 1.0);

        let e = MemoryEntry::new(1, 0, -2.0, -1.0);
        assert_eq!(e.valence, -1.0);
        assert_eq!(e.salience, 0.0);

        let e = MemoryEntry::new(1, 0, 0.5, 0.7);
        assert_eq!(e.valence, 0.5);
        assert_eq!(e.salience, 0.7);
        assert_eq!(e.reinforcement_count, 0);
    }

    #[test]
    fn memory_new_is_empty_and_preallocates() {
        let m = Memory::new();
        assert_eq!(m.entries.len(), 0);
        assert!(m.entries.capacity() >= MEMORY_CAP);
    }

    #[test]
    fn insert_under_cap_appends() {
        let mut m = Memory::new();
        for i in 0..5 {
            m.insert(entry(i, i, 0.0, 0.5));
        }
        assert_eq!(m.entries.len(), 5);
    }

    #[test]
    fn insert_at_cap_evicts_lowest_salience() {
        let mut m = Memory::new();
        for i in 0..MEMORY_CAP {
            let salience = if i == 7 { 0.1 } else { 0.5 };
            m.insert(entry(i as EventId, i as u64, 0.0, salience));
        }
        m.insert(entry(999, 999, 0.0, 0.5));
        assert_eq!(m.entries.len(), MEMORY_CAP);
        assert!(m.entries.iter().any(|e| e.event_id == 999));
        assert!(!m.entries.iter().any(|e| e.event_id == 7));
    }

    #[test]
    fn insert_eviction_ties_break_oldest_tick() {
        let mut m = Memory::new();
        for i in 0..MEMORY_CAP {
            m.insert(entry(i as EventId, i as u64, 0.0, 0.5));
        }
        // All entries have salience 0.5; oldest tick is event_id 0 (tick 0).
        m.insert(entry(999, 999, 0.0, 0.5));
        assert!(!m.entries.iter().any(|e| e.event_id == 0));
        assert!(m.entries.iter().any(|e| e.event_id == 999));
    }

    #[test]
    fn decay_reduces_salience_uniformly() {
        let mut m = Memory::new();
        m.insert(entry(1, 0, 0.0, 0.5));
        m.insert(entry(2, 0, 0.0, 0.3));
        m.decay_one_tick(0.1);
        assert!((m.entries[0].salience - 0.4).abs() < 1e-9);
        assert!((m.entries[1].salience - 0.2).abs() < 1e-9);
    }

    #[test]
    fn decay_saturates_at_zero() {
        let mut m = Memory::new();
        m.insert(entry(1, 0, 0.0, 0.05));
        m.decay_one_tick(0.5);
        assert_eq!(m.entries[0].salience, 0.0);
    }

    #[test]
    fn decay_empty_is_noop() {
        let mut m = Memory::new();
        m.decay_one_tick(0.5);
        assert_eq!(m.entries.len(), 0);
    }

    #[test]
    fn reinforce_boosts_and_saturates() {
        let mut m = Memory::new();
        m.insert(entry(1, 0, 0.0, 0.95));
        let ok = m.reinforce(0, 0.2);
        assert!(ok);
        assert_eq!(m.entries[0].salience, 1.0);
        assert_eq!(m.entries[0].reinforcement_count, 1);
    }

    #[test]
    fn reinforce_increments_count_under_cap() {
        let mut m = Memory::new();
        m.insert(entry(1, 0, 0.0, 0.5));
        for _ in 0..5 {
            m.reinforce(0, 0.0);
        }
        assert_eq!(m.entries[0].reinforcement_count, 5);
    }

    #[test]
    fn reinforce_invalid_index_returns_false() {
        let mut m = Memory::new();
        assert!(!m.reinforce(0, 0.5));
        m.insert(entry(1, 0, 0.0, 0.5));
        assert!(!m.reinforce(99, 0.5));
    }

    #[test]
    fn find_by_event_id_hits_and_misses() {
        let mut m = Memory::new();
        m.insert(entry(42, 0, 0.0, 0.5));
        assert_eq!(m.find_by_event_id(42), Some(0));
        assert_eq!(m.find_by_event_id(99), None);
    }

    #[test]
    fn serde_round_trip_entry() {
        let e = entry(7, 100, -0.5, 0.7);
        let json = ron::to_string(&e).expect("serialize");
        let r: MemoryEntry = ron::from_str(&json).expect("deserialize");
        assert_eq!(e, r);
    }

    #[test]
    fn serde_round_trip_memory() {
        let mut m = Memory::new();
        m.insert(entry(1, 0, 0.0, 0.5));
        m.insert(entry(2, 1, 0.5, 0.8));
        let json = ron::to_string(&m).expect("serialize");
        let r: Memory = ron::from_str(&json).expect("deserialize");
        assert_eq!(m, r);
    }

    #[test]
    fn constants_are_stable() {
        assert_eq!(MEMORY_CAP, 32);
        assert!((SALIENCE_FLOOR - 0.05).abs() < 1e-9);
    }

    #[test]
    fn default_constructs_empty_memory() {
        let m = Memory::default();
        assert_eq!(m.entries.len(), 0);
    }
}
