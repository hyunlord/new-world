# P8-α — `Memory` + `MemoryEntry` components + memory capacity policy

> Lane: `--full` (sim-core `.rs` edits — new component module + `components/mod.rs`
> extension force full lane by hook detection; cold-tier auto credit expected
> via Signal A+B+C+D since no GDScript/scenes touched).
> Scope: First sub-stage of V7 Phase 8 (Memory System). Lands the per-agent
> episodic memory substrate. No runtime system, no causal event variants, no
> `AgentDecisionSystem` change — Phase 6-α / 7-α data-only precedent applies.
> Governance: v3.3.17. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/`.tres`,
> no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto credit
> expected.

---

## Section 1 — Implementation Intent

V7 Phase 7 closed at `f1c12f9d` (chronicle harness) + `c924770d` (closure
declaration). Section 9+ design (`67c9a49d`) anchored Phase 8 to Memory
System. The `.harness/plans/phase8.md` plan (690 lines, local-only) locks
8 P8Plan-* decisions; this dispatch executes Phase 8-α exactly per
`phase8.md §3` "Phase 8-α" block.

Phase 8-α is structurally identical to Phase 7-α (`35fbd501`) and Phase 6-α
(`ba4e02b2`):
- Add new component module under `rust/crates/sim-core/src/components/`.
- Re-export the new types from `components/mod.rs`.
- No runtime system (`MemorySystem` priority 136 is β scope).
- No causal event variants (`CausalEvent::MemoryRecalled` is β scope).
- No `AgentDecisionSystem` change (cascade extension is β scope).
- No `TargetKind` extension (Memory is a decision-bias source, not a target).

**Key difference from Phase 7-α**: Memory is the **first per-agent collection
component** — earlier components (`Hunger` / `Thirst` / `Sleep` / `Social`) all
carry a small fixed scalar payload (`f64` + `f64`). `Memory` carries a bounded
`Vec<MemoryEntry>` (cap 32). This forces a careful look at:
- `Copy` is unavailable (Vec backing) — only `Clone`. (Same as
  `RelationshipState`'s HashMap-backed precedent in `relationship.rs`.)
- The capacity-bounded insert needs a lowest-salience eviction algorithm
  that doesn't allocate per insert (mutate-in-place on the Vec, no
  Vec::remove + push).
- Decay is `O(n)` per agent per tick, where `n ≤ MEMORY_CAP = 32` — a hard
  upper bound, not an asymptotic claim.

After P8-α:
- `sim_core::components::memory` module exists, exporting `Memory`,
  `MemoryEntry`, `MEMORY_CAP`, `SALIENCE_FLOOR`.
- `Memory` has 5 inherent methods: `new()`, `insert(entry)`,
  `decay_one_tick(rate)`, `reinforce(idx, boost)`, `find_by_event_id(event_id)`.
- A ≥12-assertion harness `harness_p8_alpha_memory_components.rs` proves
  construction, capacity-bounded insert with lowest-salience eviction,
  decay saturating at 0.0, reinforce saturating at 1.0 + count incrementing,
  find lookup hit + miss, serde round-trip, and Phase 7 regression.
- **Zero** runtime system changes.
- **Zero** `CausalEvent` / `DecisionReason` / `TargetKind` / `AgentState` changes.

---

## Section 2 — What to Build (locked facts)

### P8α-NEW-1: `rust/crates/sim-core/src/components/memory.rs` (NEW, ~180-220 lines)

```rust
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
/// ≈ 12.8 MB (each entry is 40 bytes: 8 + 8 + 8 + 8 + 4 + 4 padding).
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
/// - `valence`: emotional weight in `[-1.0, 1.0]`. Negative = unpleasant
///   (e.g. failed/aborted action, threshold-breach decision); positive =
///   pleasant (e.g. successful completion).
/// - `salience`: current strength in `[0.0, 1.0]`. Initialised at encode
///   time by the system; decays per tick via `decay_one_tick`; reinforced
///   on recall via `reinforce`.
/// - `reinforcement_count`: monotone counter (saturating at `u32::MAX`)
///   of recall events. Phase 8-β increments this on cascade-bias recall;
///   Phase 8-α only exposes the storage and the `reinforce()` helper.
///
/// Plan ref: phase8.md §2 P8Plan-1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub event_id: EventId,
    pub encoded_tick: u64,
    pub valence: f64,
    pub salience: f64,
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
/// expensive only for full rings, and at 32 entries × 40 bytes = 1.28 KB
/// per clone, this is acceptable for snapshot capture.
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
    ///
    /// This is the only mutation that can shrink the effective memory
    /// horizon. The Phase 8-β `MemorySystem` calls `insert` once per
    /// encode-worthy `CausalEvent` per tick.
    pub fn insert(&mut self, entry: MemoryEntry) {
        if self.entries.len() < MEMORY_CAP {
            self.entries.push(entry);
            return;
        }
        // Overflow: find lowest-salience (tie-break: oldest encoded_tick),
        // replace in place.
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
    ///
    /// Entries whose salience reaches `<= SALIENCE_FLOOR` become
    /// preferred-eviction candidates on the next overflow insert — but
    /// this method does NOT remove them. Phase 8-α scope is intentionally
    /// silent on eviction outside `insert()`.
    pub fn decay_one_tick(&mut self, rate: f64) {
        for entry in &mut self.entries {
            entry.salience = (entry.salience - rate).max(0.0);
        }
    }

    /// Boost a single entry's salience (saturating at 1.0) and increment
    /// its reinforcement counter (saturating at `u32::MAX`).
    /// Called by Phase 8-β's `AgentDecisionSystem` cascade-bias path on
    /// every cascade-bias recall.
    ///
    /// Returns `true` if the index was valid; `false` otherwise (no-op
    /// for out-of-bounds indices — the caller is expected to have just
    /// looked the index up via `find_by_event_id`).
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
        assert_eq!(m.entries.capacity(), MEMORY_CAP);
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
            m.insert(entry(i as EventId, i as u64, 0.0, 0.5));
        }
        // Lower one entry's salience.
        m.entries[7].salience = 0.1;
        // Insert a new entry — should evict index 7.
        m.insert(entry(999, 999, 0.0, 0.5));
        assert_eq!(m.entries.len(), MEMORY_CAP);
        assert!(m.entries.iter().any(|e| e.event_id == 999));
        assert!(!m.entries.iter().any(|e| e.event_id == 7));
    }

    #[test]
    fn insert_eviction_ties_break_oldest_tick() {
        let mut m = Memory::new();
        // Fill cap with equal salience 0.5 at varying ticks.
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
        let json = serde_json::to_string(&e).expect("serialize");
        let r: MemoryEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(e, r);
    }

    #[test]
    fn serde_round_trip_memory() {
        let mut m = Memory::new();
        m.insert(entry(1, 0, 0.0, 0.5));
        m.insert(entry(2, 1, 0.5, 0.8));
        let json = serde_json::to_string(&m).expect("serialize");
        let r: Memory = serde_json::from_str(&json).expect("deserialize");
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
```

### P8α-MOD-1: `rust/crates/sim-core/src/components/mod.rs` (MODIFIED)

Append in alphabetical position (between `hunger` and `position`):

```rust
pub mod memory;
```

Append in alphabetical re-export position (between `Hunger` and `Position`):

```rust
pub use memory::{Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR};
```

Also extend the file-level doc comment to mention Phase 8-α (after the
Phase 7-α paragraph, before the `See` line):

```text
//! Phase 8-α adds the per-agent episodic memory substrate
//! ([`Memory`], [`MemoryEntry`], [`MEMORY_CAP`], [`SALIENCE_FLOOR`]).
//! The runtime `MemorySystem` (priority 136), `CausalEvent::MemoryRecalled`,
//! `DecisionReason::MemoryReason`, and the `AgentDecisionSystem` 6th-cascade
//! bias mechanism are all Phase 8-β scope. `TargetKind` and `AgentState`
//! are intentionally unchanged — Memory is a decision-bias source, not a
//! target type.
```

### P8α-HARNESS-1: `rust/crates/sim-test/tests/harness_p8_alpha_memory_components.rs` (NEW)

Single `#[test] fn harness_p8_alpha_memory_components()` is **not** the
pattern — `harness_p7_alpha_social_components.rs` uses 37 separate
`#[test]` functions named `harness_p7_alpha_aN_*`. Follow that precedent:
one function per assertion, named `harness_p8_alpha_aN_*`. Minimum 12
assertions covering:

- **A1**: `MemoryEntry::new` clamps `valence` to `[-1.0, 1.0]`.
- **A2**: `MemoryEntry::new` clamps `salience` to `[0.0, 1.0]`.
- **A3**: `MemoryEntry::new` sets `reinforcement_count = 0`.
- **A4**: `Memory::new` is empty with `capacity == MEMORY_CAP`.
- **A5**: `Memory::insert` appends under cap.
- **A6**: `Memory::insert` at cap evicts lowest-salience entry.
- **A7**: `Memory::insert` eviction tie-breaks on oldest `encoded_tick`.
- **A8**: `Memory::decay_one_tick` reduces salience uniformly.
- **A9**: `Memory::decay_one_tick` saturates at 0.0.
- **A10**: `Memory::reinforce` boosts salience saturating at 1.0 + increments count.
- **A11**: `Memory::reinforce` returns `false` for invalid index.
- **A12**: `Memory::find_by_event_id` returns `Some(idx)` on hit, `None` on miss.
- **A13** (recommended): serde round-trip for `MemoryEntry`.
- **A14** (recommended): serde round-trip for `Memory` (non-empty).
- **A15** (recommended): `MEMORY_CAP == 32` and `SALIENCE_FLOOR == 0.05`
  constants stable (regression sentinel — these are referenced by Phase
  8-β's `MemorySystem`, and changing them silently would break β's tick
  budget and salience floor logic).
- **A16** (recommended): components mod re-exports `Memory`, `MemoryEntry`,
  `MEMORY_CAP`, `SALIENCE_FLOOR` symbols visible at `sim_core::components::*`.
- **A17** (recommended, regression): existing exports still visible
  (`Social`, `RelationshipKey`, `TargetKind`, etc.) — confirms Phase 7-α
  unchanged.

Drafter / Generator may choose 12-17 inclusive. Minimum is 12, plan §α
target is 12 with 4-5 recommended extras.

---

## Section 3 — How to Implement

### Step 3.1: Create `memory.rs`
Write the full `memory.rs` content from §2 P8α-NEW-1. All 14 inline
`#[test]` functions are mandatory — they mirror the integration tests
but verify at the unit level (single-module isolation, no cross-crate
deps).

### Step 3.2: Edit `components/mod.rs`
Two append-in-alphabetical-position changes from §2 P8α-MOD-1:
- `pub mod memory;` line.
- `pub use memory::{Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR};` line.
- File-level doc-comment paragraph addition before the `See` line.

### Step 3.3: Create the integration harness
Write `rust/crates/sim-test/tests/harness_p8_alpha_memory_components.rs`
following the per-assertion-function pattern of
`harness_p7_alpha_social_components.rs`. Each assertion is its own
`#[test] fn harness_p8_alpha_aN_description()`. Use `use
sim_core::components::{Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR};`
and `use sim_core::causal::event::EventId;` (re-export path is via
`pub use causal::event::EventId;` in causal/mod.rs — verify the actual
path by reading `rust/crates/sim-core/src/causal/mod.rs` first).

### Step 3.4: Verify
```bash
cd rust
cargo build --workspace 2>&1 | tail -5
cargo test --workspace 2>&1 | grep -E "test result:|FAIL" | tail -50
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
```

Expected:
- Build clean.
- Test count: `prior 751 + 14 (memory.rs inline) + 12-17 (harness) =
  777-782 passed; 0 failed`. (Generator can confirm exact number.)
- Clippy clean.
- Phase 7-γ chronicle + all p3/p4/p5/p6/p7 prior harnesses unchanged.
- No new `unwrap()` in production paths; the one `.expect()` in
  `insert()` is gated by the `entries.len() < MEMORY_CAP` short-circuit
  above it (provably never panics).

### Step 3.5: scope boundaries — DO NOT
- DO NOT modify `rust/crates/sim-core/src/causal/event.rs` (no new
  `CausalEvent` variant, no new `DecisionReason` variant — both are β).
- DO NOT modify `rust/crates/sim-core/src/components/agent_state.rs`
  (no `TargetKind` extension — Memory is a bias source, not a target).
- DO NOT create any `rust/crates/sim-systems/src/runtime/memory/`
  directory (no `MemorySystem` — that is β).
- DO NOT extend `rust/crates/sim-systems/src/runtime/decision/
  agent_decision.rs` (no cascade bias logic — that is β).
- DO NOT touch any `scripts/` or `scenes/` path (backend only).
- DO NOT add any FFI surface to `sim-bridge` (Phase 8-δ optional, mandate-gated).

---

## Section 4 — Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|---|---|:---:|---|
| 8α-T1 | `rust/crates/sim-core/src/components/memory.rs` (NEW, full impl + 14 inline tests) | 🟢 DISPATCH | — |
| 8α-T2 | `rust/crates/sim-core/src/components/mod.rs` (3-line edit + doc comment) | 🔴 DIRECT | T1 (needs `memory` mod) |
| 8α-T3 | `rust/crates/sim-test/tests/harness_p8_alpha_memory_components.rs` (NEW, 12-17 assertions) | 🟢 DISPATCH | T1, T2 |
| 8α-T4 | Build + test + clippy verification | 🔴 DIRECT | T1, T2, T3 |

Dispatch ratio: 2/4 = 50%. Sub-stage is small enough that direct
edits for `mod.rs` (3 lines) and verification (commands only) are
proportionate. Phase 7-α used the same 50% ratio for the same reason.

---

## Section 5 — Localization Checklist

No new locale keys. Phase 8-α is backend-only. The `MemoryReason`
DecisionReason discriminator + `MemoryRecalled` event will introduce
locale keys in Phase 8-β; UI-facing memory rendering (Phase 8-δ) will
introduce further keys gated on user mandate.

---

## Section 6 — Verification & Notion

### Gate command
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result:" | tail -10
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

### Expected output
```
test result: ok. <new total> passed; 0 failed; ...
```
Numbers: prior 751 + 14 (inline) + 12-17 (harness) = 777-782 ish.
Generator should record actual count.

### Pipeline expectation
- Lane: `--full` (sim-core `.rs` edit forces hot-tier detection in pre-commit).
- Cold-tier auto credit via Signal A+B+C+D — threshold drops to **75**, not 90.
- VLM: no-godot-scope auto credit (no `.gd`/`.gdshader`/`.tscn`/`.tres`/
  `scripts/`/`scenes/` path touched).
- Issue 14 fix (`ebbf6ddc`): FFI vacuous check returns CONFIRMED on this
  diff (no `sim-bridge/` files touched) → full FFI credit before the
  SKIP-path is even considered.
- Pattern D mitigation: smallest scope of Phase 8 sub-stages — target
  attempt-1 APPROVE for max score (no -10 attempt-3 penalty).
- Score expectation: **95-100/100** on attempt-1 APPROVE, hot-tier
  threshold 90 ≫ exceeded.

### Notion
No Notion page update — Phase 8 tracking lives in `.harness/audit/
v7_progress.md` (next governance update appends "Phase 8-α landed" on
commit).

### Post-commit
- `git push origin lead/main`.
- `git ls-remote origin refs/heads/lead/main` MUST match local HEAD.
- The commit body should follow the `feat(p8-alpha-memory-components):
  implementation [harness: plan x... code x... eval:APPROVE(codex)
  visual:... ffi:... regr:CLEAN]` format from Phase 7-α/β/γ commits.

### In-game verification (Section 7 — backend-only feature)
None. Phase 8-α adds no runtime behaviour. The Godot side cannot
observe `Memory` until Phase 8-β (via SimBridge addition or BehaviorMix
chronicle data) or Phase 8-δ (UI surface). Phase 8-α is "the substrate
compiles, the harness proves the substrate is correct".
