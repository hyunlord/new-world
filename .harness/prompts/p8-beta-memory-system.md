# P8-β — `MemorySystem` priority 136 + `CausalEvent::MemoryRecalled` + `DecisionReason::MemoryReason` + `MemoryRecallTrigger` + `AgentDecisionSystem` 6th-cascade weighted-scoring bias

> Lane: `--full` (sim-core + sim-systems `.rs` edits across 4 files force hot-tier
> detection; cold-tier auto credit expected via Signal A+B+C+D since no
> GDScript/scenes touched).
> Scope: Second sub-stage of V7 Phase 8 (Memory System). Wires the runtime —
> per-tick encoding from this tick's causal events, per-tick decay, and the
> 6th-cascade weighted-scoring bias mechanism that emits the
> `MemoryRecalled → AgentDecision{MemoryReason}` causal pair when memory
> flips the cascade's natural winner.
> Governance: v3.3.17. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/`.tres`,
> no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto credit
> expected.

---

## Section 1 — Implementation Intent

V7 Phase 8-α closed at `0f1d4814` (100/100 perfect score). The `Memory` +
`MemoryEntry` substrate is landed with `MEMORY_CAP=32` + `SALIENCE_FLOOR=0.05`
+ 5 inherent methods (`new`, `insert`, `decay_one_tick`, `reinforce`,
`find_by_event_id`). The `.harness/plans/phase8.md` plan (690 lines,
local-only) locks 8 P8Plan-* decisions; this dispatch executes Phase 8-β
exactly per `phase8.md §3` "Phase 8-β" block.

Phase 8-β is structurally analogous to Phase 7-β (`de336f83`) but extends
to a **cascade-bias** mechanism rather than a new `Seeking → Consuming →
Completed` loop:

- Add a new `MemorySystem` runtime under `rust/crates/sim-systems/src/runtime/memory/`.
- Extend `CausalEvent` with one new variant (`MemoryRecalled`, 9th).
- Extend `DecisionReason` with one new variant (`MemoryReason`, 6th).
- Introduce one new exhaustive enum `MemoryRecallTrigger` (3 variants, only
  `CascadeBias` wired in this dispatch).
- Extend `AgentDecisionSystem` to compute per-cascade-arm memory weight
  deltas and emit the `MemoryRecalled → AgentDecision{MemoryReason}` pair
  **only when memory bias flips the natural-cascade winner**.
- All other prior systems (Hunger/Thirst/Sleep/Construction/SocialInteraction/
  SocialDecay) remain unchanged.
- No `TargetKind`, `AgentState`, or `Position` extension.

**Key difference from Phase 7-β**: this is the **first cascade-bias source**
in the AgentDecisionSystem. The 5 existing arms (Hunger/Thirst/Fatigue/
Construction/Social) compute their natural eligibility independently;
Phase 8-β adds a per-arm `+memory_weight_delta` that can shift the cascade's
natural winner. The 6th "arm" `MemoryReason` is **emergent** (a different
arm wins after bias) rather than an independently-eligible cascade arm. This
distinguishes Memory from the other 5 arms architecturally.

**Anti-recursion is load-bearing**: `AgentDecision{MemoryReason}` events
are NOT encoded as memories (planning §β per-event mapping table) and are
NOT considered as bias sources during weight computation. Both rules
prevent a meta-bias / infinite-recursion class of bug. Without them, a
memory-flipped cascade outcome could encode itself as a new memory whose
recall would re-trigger the same flip, ad infinitum.

After P8-β:
- `MemorySystem` registered at priority 136, tick_interval 1.
- `CausalEvent` has **9 variants** (8 prior + `MemoryRecalled`).
- `DecisionReason` has **6 variants** (5 prior + `MemoryReason`).
- `MemoryRecallTrigger` enum exists with 3 variants (`CascadeBias` wired,
  `SimilaritySearch` + `Periodic` reserved).
- `AgentDecisionSystem` cascade computes per-arm memory weight deltas,
  emits the bias chain on cascade flip.
- A ≥12-assertion harness `harness_p8_beta_memory_system.rs` proves the
  full encoding/decay/bias-flip/non-flip/anti-recursion contract.
- **Zero** changes to Hunger/Thirst/Sleep/Construction/SocialInteraction/
  SocialDecay system bodies.
- **Zero** changes to `TargetKind`, `AgentState`, `Position`.
- **Zero** schema additions on `SimResources` (per-agent component
  storage avoids cross-agent contention; planning §β explicit).

---

## Section 2 — What to Build (locked facts)

### P8β-NEW-1: `rust/crates/sim-systems/src/runtime/memory/mod.rs` (NEW, ~10 lines)

```rust
//! V7 Phase 8-β — Memory System module root.
//!
//! Hosts [`MemorySystem`] (priority 136, the only system in this module).
//! See `.harness/plans/phase8.md §3` Phase 8-β block for the full
//! per-event encoding table and cascade-bias semantics.

pub mod memory_system;

pub use memory_system::{MemorySystem, DECAY_RATE, REINFORCEMENT_BOOST, MAX_RECENCY_TICKS};
```

### P8β-NEW-2: `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` (NEW, ~250-320 lines)

Implementation notes:
- `MemorySystem` is a unit struct.
- `priority() = 136`, `tick_interval() = 1`.
- Two-phase tick: (1) encoding pass, (2) decay pass.

```rust
//! V7 Phase 8-β — MemorySystem (priority 136).
//!
//! Two-phase tick:
//! 1. **Encoding pass** — walk this tick's newly-emitted causal events
//!    (filter `causal_log` entries where `event.tick() == resources.current_tick`).
//!    For each event with an actor `AgentId`, look up that agent's
//!    `Memory` component and `insert()` a new `MemoryEntry` with
//!    `event_id = event.id()`, `encoded_tick = current_tick`, and the
//!    `(salience, valence)` pair from the per-event mapping below.
//! 2. **Decay pass** — apply `Memory::decay_one_tick(DECAY_RATE)` to every
//!    Memory component. No eviction here; eviction only happens on
//!    capacity-overflow insert.
//!
//! Per-event encoding mapping (planning §β locked):
//! | Source event                                       | Salience | Valence |
//! | AgentDecision { HungerThresholdBreach }            | 0.4      | -0.3   |
//! | AgentDecision { ThirstThresholdBreach }            | 0.4      | -0.3   |
//! | AgentDecision { FatigueThresholdBreach }           | 0.3      | -0.2   |
//! | AgentDecision { ConstructionReason }               | 0.5      | +0.1   |
//! | AgentDecision { SocialReason }                     | 0.5      | +0.2   |
//! | ConstructionStarted                                | 0.6      | +0.3   |
//! | ConstructionCompleted                              | 0.8      | +0.6   |
//! | SocialInteractionStarted                           | 0.6      | +0.4   |
//! | SocialInteractionCompleted                         | 0.8      | +0.7   |
//! | AgentDecision { MemoryReason }                     | — (not encoded — anti-recursion lock) |
//! | BuildingPlaced / StampDirty / InfluenceChanged     | — (non-actor events, skipped) |
//! | MemoryRecalled                                     | — (anti-recursion; not encoded) |
//!
//! Actor lookup: each `CausalEvent` variant's "actor" depends on its
//! shape (Generator MUST grep `event.rs` to confirm field names):
//! - `AgentDecision`: `agent: AgentId` field directly carries the actor.
//! - `ConstructionStarted` / `ConstructionCompleted`: NO `agent` field on
//!   these variants. The actor MUST be reconstructed via parent walk —
//!   the `parent` field references the originating
//!   `AgentDecision { ConstructionReason, agent: X, .. }`. Use
//!   `resources.causal_log.lookup(event.parent()?)?` to find the parent
//!   `AgentDecision` and extract its `agent` field. Lookup miss
//!   (parent evicted from ring buffer): skip the encoding (graceful
//!   eviction precedent — no panic, no fallback).
//! - `SocialInteractionStarted` / `SocialInteractionCompleted`: each
//!   of the two `agents: (AgentId, AgentId)` tuple entries (encode for
//!   BOTH participants).
//! - All other variants (BuildingPlaced/StampDirty/InfluenceChanged/
//!   MemoryRecalled): no actor → skipped.

use hecs::World;

use sim_core::components::{Agent, AgentId, Memory, MemoryEntry};
use sim_core::causal::{CausalEvent, DecisionReason};

use crate::runtime::system::RuntimeSystem;
use crate::engine::SimResources;

/// Per-tick salience decay rate. 1000 ticks ≈ 1.0 salience drop matches
/// Phase 7-γ chronicle 80-tick budget × ~12× headroom for Phase 8-γ
/// delay scenarios.
pub const DECAY_RATE: f64 = 0.001;

/// Salience boost applied on each cascade-bias recall. 10 recalls
/// saturates an entry from floor to ceiling (mirrors the
/// `FAMILIARITY_BUMP = 0.1` shape from Phase 7).
pub const REINFORCEMENT_BOOST: f64 = 0.1;

/// Maximum recency horizon for the cascade-bias `recency_factor` linear
/// ramp. Matches Phase 5 `make_stage1_engine` canonical tick budget.
pub const MAX_RECENCY_TICKS: u64 = 4380;

pub struct MemorySystem;

impl RuntimeSystem for MemorySystem {
    fn name(&self) -> &str { "MemorySystem" }
    fn priority(&self) -> u32 { 136 }
    fn tick_interval(&self) -> u64 { 1 }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let current_tick = resources.current_tick;

        // Phase 1: encoding pass — iterate this tick's causal events,
        // encode per-event mapping into the actor's Memory.
        // (Use whatever `CausalLogStorage` accessor enumerates current-tick
        // events — Generator should grep `causal_log` API and use the
        // appropriate iterator; if a "events_at_tick(t)" method does not
        // exist, the Generator must add one in causal/storage.rs as a
        // small helper, or filter via `iter().filter(|e| e.tick() == t)`.)

        // For each event with an actor:
        //   - Compute (salience, valence) per match block (planning §β).
        //   - Find the actor entity by AgentId (Generator: existing pattern
        //     in agent_decision.rs that resolves AgentId → entity).
        //   - world.query_one_mut::<&mut Memory>(actor_entity).ok().and_then(
        //         |memory| Some(memory.insert(MemoryEntry::new(
        //             event.id(), current_tick, valence, salience,
        //         )))
        //     );

        // Phase 2: decay pass — uniform linear decay across all Memory
        // components.
        for (_, memory) in world.query_mut::<&mut Memory>() {
            memory.decay_one_tick(DECAY_RATE);
        }
    }
}

/// Internal helper: classify a `CausalEvent` to its (salience, valence,
/// actor_agent_ids) tuple per the planning §β mapping table.
///
/// Construction events (`ConstructionStarted` / `ConstructionCompleted`)
/// do NOT carry an `agent` field directly — the Generator must reconstruct
/// the actor via the parent walk:
///   1. Read `event.parent()` → `Option<EventId>`.
///   2. If `None` → skip encoding (no actor reachable).
///   3. If `Some(parent_id)` → `resources.causal_log.lookup(parent_id)`.
///   4. If the lookup returns `Some(CausalEvent::AgentDecision { agent, reason: ConstructionReason, .. })`,
///      use that `agent`. Otherwise skip (parent evicted or chain broken).
///
/// Returns `None` for events that should not be encoded
/// (BuildingPlaced/StampDirty/InfluenceChanged — non-actor;
/// AgentDecision{MemoryReason} — anti-recursion; MemoryRecalled —
/// anti-recursion; Construction events whose parent is unreachable).
///
/// SocialInteractionStarted/Completed return TWO `AgentId`s (the
/// canonical `agents: (AgentId, AgentId)` tuple). All other actor events
/// return ONE.
///
/// Signature carries `&CausalLogStorage` (for the parent walk). The
/// concrete iterator over "this tick's events" lives in
/// `MemorySystem::tick`; this helper takes a single event and resolves
/// its actor.
///
/// IMPORTANT — `CausalLogStorage` API: at grep time (Step 0), the
/// storage exposed:
///   - `iter()` returning `(&u32 tile_idx, &TileCausalLog)`
///   - `get(tile_idx) -> Option<&TileCausalLog>`
///   - `trace_parents(tile_idx, event_id) -> Vec<&CausalEvent>` (per-tile)
/// but NO global `lookup(event_id) -> Option<&CausalEvent>` helper.
///
/// The Generator MUST add such a helper to `causal/storage.rs`:
///
/// ```rust
/// /// Look up an event by id across all per-tile ring buffers.
/// /// O(N tiles * RING_SIZE) — bounded and small in practice. Returns
/// /// `None` if the event has been evicted or never recorded.
/// pub fn lookup(&self, event_id: EventId) -> Option<&CausalEvent> {
///     self.iter()
///         .flat_map(|(_, log)| log.events())
///         .find(|e| e.id() == event_id)
/// }
/// ```
///
/// (The `TileCausalLog::events() -> impl Iterator<Item = &CausalEvent>`
/// accessor may also need adding if not already present — Generator
/// should confirm and add if missing.) The added helper is also the
/// substrate `agent_decision.rs::event_id_matches_arm` consumes.
fn classify_event(
    event: &CausalEvent,
    causal_log: &sim_core::causal::CausalLogStorage,
) -> Option<(f64, f64, Vec<AgentId>)> {
    match event {
        CausalEvent::AgentDecision { agent, reason, .. } => match reason {
            DecisionReason::HungerThresholdBreach => Some((0.4, -0.3, vec![*agent])),
            DecisionReason::ThirstThresholdBreach => Some((0.4, -0.3, vec![*agent])),
            DecisionReason::FatigueThresholdBreach => Some((0.3, -0.2, vec![*agent])),
            DecisionReason::ConstructionReason => Some((0.5,  0.1, vec![*agent])),
            DecisionReason::SocialReason         => Some((0.5,  0.2, vec![*agent])),
            DecisionReason::MemoryReason         => None, // anti-recursion
        },
        CausalEvent::ConstructionStarted { parent, .. } => {
            // Parent walk: the originating AgentDecision{ConstructionReason}
            // carries the actor.
            let parent_id = (*parent)?;
            // Generator: use whatever `causal_log` accessor exists for
            // EventId lookup. If none exists, add a small `lookup(EventId)
            // -> Option<&CausalEvent>` helper to `causal/storage.rs`.
            let parent_event = causal_log.lookup(parent_id)?;
            if let CausalEvent::AgentDecision { agent, .. } = parent_event {
                Some((0.6, 0.3, vec![*agent]))
            } else {
                None
            }
        }
        CausalEvent::ConstructionCompleted { parent, .. } => {
            // Parent walk: parent is ConstructionStarted; grandparent is
            // the originating AgentDecision{ConstructionReason}.
            // Generator may chain two lookups or short-circuit at
            // ConstructionStarted (whichever the causal_log API supports
            // cleanly).
            let parent_id = (*parent)?;
            let parent_event = causal_log.lookup(parent_id)?;
            match parent_event {
                CausalEvent::ConstructionStarted { parent: gp, .. } => {
                    let grandparent_id = (*gp)?;
                    let grandparent_event = causal_log.lookup(grandparent_id)?;
                    if let CausalEvent::AgentDecision { agent, .. } = grandparent_event {
                        Some((0.8, 0.6, vec![*agent]))
                    } else {
                        None
                    }
                }
                CausalEvent::AgentDecision { agent, .. } => {
                    // Defensive: short-circuit if the chain shape is non-standard.
                    Some((0.8, 0.6, vec![*agent]))
                }
                _ => None,
            }
        }
        CausalEvent::SocialInteractionStarted { agents, .. } => {
            Some((0.6, 0.4, vec![agents.0, agents.1]))
        }
        CausalEvent::SocialInteractionCompleted { agents, .. } => {
            Some((0.8, 0.7, vec![agents.0, agents.1]))
        }
        CausalEvent::BuildingPlaced { .. }
        | CausalEvent::StampDirty { .. }
        | CausalEvent::InfluenceChanged { .. }
        | CausalEvent::MemoryRecalled { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_is_136_interval_is_1() {
        let s = MemorySystem;
        assert_eq!(s.priority(), 136);
        assert_eq!(s.tick_interval(), 1);
        assert_eq!(s.name(), "MemorySystem");
    }

    // Generator: add 2-4 additional inline tests covering:
    // - classify_event for each actor-event variant (5+2 = 7 cases)
    // - classify_event for each non-actor / anti-recursion variant
    //   (BuildingPlaced/StampDirty/InfluenceChanged/AgentDecision{MemoryReason}/MemoryRecalled = 5 cases)
    // - SocialInteractionStarted/Completed return both agents.
}
```

Generator note on `agents: (AgentId, AgentId)`: grep the existing
`SocialInteractionStarted` / `SocialInteractionCompleted` variant
declarations in `event.rs` for the exact field name shape (`agents` vs
`partners` vs a 2-field destructure). Match the actual declared field.

### P8β-MOD-1: `rust/crates/sim-core/src/causal/event.rs` (MODIFIED)

#### Append to `DecisionReason` enum (6th variant):
```rust
/// Agent's `AgentDecisionSystem` cascade was flipped by a positive
/// or negative memory weight delta on a non-natural-winner arm.
/// Parent points to the `MemoryRecalled` event that surfaced the
/// load-bearing memory. V7 Phase 8-β / P8β-5.
MemoryReason,
```
And extend `as_str()`:
```rust
DecisionReason::MemoryReason => "memory_reason",
```

#### Append to `CausalEvent` enum (9th variant):
```rust
/// V7 Phase 8-β / P8Plan-6. Emitted by `AgentDecisionSystem` when a
/// cascade-bias memory weight delta flips the cascade's natural
/// winner. The `recalled_event` field references the top-contributor
/// memory entry's `event_id`; parent walks back to the original
/// chain root via the recalled event's own parent.
///
/// Chain link (parent: typically the `recalled_event`'s parent, or
/// the most-recent same-tile `InfluenceChanged` if the recalled
/// event's parent is None — see Generator note).
MemoryRecalled {
    id: EventId,
    parent: Option<EventId>,
    agent: AgentId,
    recalled_event: EventId,
    triggered_by: MemoryRecallTrigger,
    tick: u64,
},
```

#### New enum `MemoryRecallTrigger`:
```rust
/// Trigger taxonomy for `CausalEvent::MemoryRecalled`. Phase 8-β wires
/// only `CascadeBias`; the other two variants are reserved Phase 9+
/// extension points (mirrors the `TargetKind` extensibility pattern).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}
```

#### Extend accessor methods exhaustively:
- `id()` arm: `CausalEvent::MemoryRecalled { id, .. } => *id`
- `parent()` arm: `CausalEvent::MemoryRecalled { parent, .. } => *parent`
- `tick()` arm: `CausalEvent::MemoryRecalled { tick, .. } => *tick`
- `channel()` arm: `CausalEvent::MemoryRecalled { .. } => None`

#### Inline `#[cfg(test)] mod tests` extensions:
- `DecisionReason::MemoryReason.as_str()` returns `"memory_reason"`.
- All 6 `DecisionReason` variants are exhaustively matched (no `_` arm).
- `MemoryRecalled` `id()`/`parent()`/`tick()`/`channel()` accessors
  return expected values.
- `MemoryRecallTrigger` enum: all 3 variants are serialisable and
  round-trip through serde.

### P8β-MOD-2: `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (MODIFIED)

The Generator should:

1. Add a `CascadeArm` enum (5 variants — NOT including `MemoryReason`,
   since Memory is a bias source, not an arm).

   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
   enum CascadeArm {
       Hunger,
       Thirst,
       Fatigue,
       Construction,
       Social,
   }
   ```

2. Add a `recency_factor(encoded_tick, current_tick)` helper —
   linear ramp from 1.0 at elapsed=0 down to 0.0 at elapsed ≥
   `MAX_RECENCY_TICKS = 4380` (re-exported from `memory/memory_system.rs`).

   ```rust
   fn recency_factor(encoded_tick: u64, current_tick: u64) -> f64 {
       let elapsed = current_tick.saturating_sub(encoded_tick);
       if elapsed >= MAX_RECENCY_TICKS {
           0.0
       } else {
           1.0 - (elapsed as f64 / MAX_RECENCY_TICKS as f64)
       }
   }
   ```

3. Add a `event_id_matches_arm(event_id, arm, causal_log)` classifier —
   look up the event by `event_id` in `resources.causal_log` and return
   `bool` per planning §β table:

   | CausalEvent | Arm |
   |---|---|
   | `AgentDecision { HungerThresholdBreach }` | Hunger |
   | `AgentDecision { ThirstThresholdBreach }` | Thirst |
   | `AgentDecision { FatigueThresholdBreach }` | Fatigue |
   | `AgentDecision { ConstructionReason }` / `ConstructionStarted` / `ConstructionCompleted` | Construction |
   | `AgentDecision { SocialReason }` / `SocialInteractionStarted` / `SocialInteractionCompleted` | Social |
   | `AgentDecision { MemoryReason }` | — (return false, anti-recursion) |
   | `BuildingPlaced` / `StampDirty` / `InfluenceChanged` / `MemoryRecalled` | — (return false) |
   | Lookup miss (event evicted from ring buffer) | — (return false, Phase 3-β graceful eviction) |

4. Add a `memory_weight_delta(memory, arm, current_tick, causal_log)`
   computation:

   ```rust
   fn memory_weight_delta(
       memory: &Memory,
       arm: CascadeArm,
       current_tick: u64,
       causal_log: &CausalLogStorage,
   ) -> f64 {
       memory.entries.iter()
           .filter(|entry| event_id_matches_arm(entry.event_id, arm, causal_log))
           .map(|entry| {
               let recency = recency_factor(entry.encoded_tick, current_tick);
               entry.valence * entry.salience * recency
           })
           .sum()
   }
   ```

5. Cascade extension: after computing each arm's natural eligibility
   (existing logic — DO NOT replace), apply a memory weight delta to
   each arm's "score" and find the bias-adjusted winner. If the
   bias-adjusted winner differs from the natural winner, emit:

   - A `MemoryRecalled { id, parent, agent, recalled_event, triggered_by:
     CascadeBias, tick }` event with `recalled_event` = the top-contributor
     entry's `event_id` (the entry whose `|valence * salience * recency|`
     is highest among entries that matched the bias-winning arm). `parent`
     = the recalled entry's original event's `parent()`, or `None` if the
     original event has been evicted from the ring buffer.
   - An `AgentDecision { id, parent: Some(MemoryRecalled.id), agent,
     position, tick, reason: MemoryReason }` event.
   - Apply the bias-driven cascade arm (the agent transitions to the
     `Seeking { target }` consistent with that arm).

6. If memory bias does NOT flip the natural winner, do NOT emit
   `MemoryRecalled`. The cascade proceeds as Phase 5/6/7. This is the
   "memory observed but not load-bearing" case — it must NOT pollute
   the causal log.

7. Find-the-actor pattern: the Generator should grep the existing
   `AgentDecisionSystem` body for how it currently resolves agent
   identity (`Agent { id }` component query). The new memory weight
   computation reuses that pattern; it does NOT add a separate query.

8. Add `use sim_core::components::Memory;` and
   `use sim_core::causal::{CausalLogStorage, MemoryRecallTrigger};`
   imports at the top.

### P8β-MOD-3: `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFIED)

Add `pub mod memory;` in alphabetical position (between `influence` /
`needs` / `social` — Generator: grep file to confirm alphabetical
order and place accordingly).

### P8β-MOD-4: `rust/crates/sim-systems/src/lib.rs` (MODIFIED)

```rust
pub fn register_memory_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(MemorySystem));
}
```

And in `register_default_runtime_systems`, append AFTER
`register_social_systems(engine);`:

```rust
register_memory_systems(engine);
```

Engine internally sorts by priority, so the registration-order line is
documentation; the actual tick-order is locked at priority 136 by
`MemorySystem::priority()`.

### P8β-HARNESS-1: `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs` (NEW)

Follow the per-assertion-function pattern of
`harness_p7_beta_social_system.rs` and `harness_p8_alpha_memory_components.rs`.
Each assertion is its own `#[test] fn harness_p8_beta_aN_*` function.

Minimum 12 assertions, planning §β explicit list:

- **A1**: `MemorySystem::priority() == 136`, `tick_interval() == 1`,
  `name() == "MemorySystem"`.
- **A2**: Encoding for `AgentDecision { HungerThresholdBreach }` —
  emit one such event in tick T, run `MemorySystem::tick`, verify the
  actor's Memory has one entry with salience=0.4, valence=-0.3.
- **A3**: Encoding for `AgentDecision { ConstructionReason }` —
  salience=0.5, valence=+0.1.
- **A4**: Encoding for `AgentDecision { SocialReason }` —
  salience=0.5, valence=+0.2.
- **A5**: Encoding for `ConstructionStarted` — salience=0.6,
  valence=+0.3.
- **A6**: Encoding for `ConstructionCompleted` — salience=0.8,
  valence=+0.6.
- **A7**: Encoding for `SocialInteractionStarted` — salience=0.6,
  valence=+0.4, encoded on BOTH participating agents.
- **A8**: Encoding for `SocialInteractionCompleted` — salience=0.8,
  valence=+0.7, encoded on BOTH participating agents.
- **A9**: Non-actor events `BuildingPlaced` / `StampDirty` /
  `InfluenceChanged` are NOT encoded (no Memory entries appear).
- **A10**: Anti-recursion: `AgentDecision { MemoryReason }` events
  are NOT encoded; `MemoryRecalled` events are NOT encoded.
- **A11**: Decay pass — after `MemorySystem::tick`, every existing
  Memory entry's salience decreases by `DECAY_RATE = 0.001`.
- **A12**: Cascade flip — construct a deterministic scenario where
  the natural cascade winner would be Construction, but a strongly
  positive Social-arm memory weight delta flips the winner to Social.
  Verify the `MemoryRecalled` + `AgentDecision{MemoryReason}` pair is
  emitted, with the AgentDecision's parent = MemoryRecalled.id.
- **A13** (recommended): Cascade non-flip — same scenario as A12 but
  with weaker memory weight; cascade winner is unchanged (Construction
  still wins); NO `MemoryRecalled` event emitted.
- **A14** (recommended): `MemoryRecallTrigger::CascadeBias` is the
  only triggered variant in Phase 8-β; the other 2 variants are
  declared but never emitted at this stage.
- **A15** (recommended): Anti-recursion at the cascade level — a
  prior `AgentDecision{MemoryReason}` memory entry does NOT contribute
  to any arm's weight delta. (Strictly this is a consequence of A10
  + classify-event filtering; assertion is for defense-in-depth.)
- **A16** (recommended, regression): Phase 7-γ social chronicle
  scenarios remain CLEAN — running Phase 7-γ's canonical
  configuration end-to-end still produces zero
  `MemoryRecalled` events (no memories exist before encoding starts;
  Phase 7-γ's 80-tick budget is shorter than the decay-to-floor
  horizon by orders of magnitude).
- **A17** (recommended, regression): Phase 6-γ construction
  chronicle scenarios remain CLEAN.

Drafter / Generator may choose 12-17 inclusive. Minimum is 12, plan §β
target is 12 with several recommended extras for defense-in-depth.

---

## Section 3 — How to Implement

### Step 3.1: Extend `event.rs`
- Append `MemoryReason` to `DecisionReason` enum.
- Append `MemoryRecalled` variant to `CausalEvent` enum.
- Add new `MemoryRecallTrigger` enum (3 variants).
- Extend all 4 accessor methods (`id`, `parent`, `tick`, `channel`)
  exhaustively for the new variant.
- Extend inline tests to cover the new discriminator + accessors.

### Step 3.2: Create `runtime/memory/mod.rs` + `memory_system.rs`
- Mod root re-exports `MemorySystem`, `DECAY_RATE`, `REINFORCEMENT_BOOST`,
  `MAX_RECENCY_TICKS`.
- `memory_system.rs` per the §2 P8β-NEW-2 spec. The Generator should:
  - Grep `causal_log` API to find the per-tick event iterator (or add
    a small `iter_at_tick(t)` helper in `causal/storage.rs` if none
    exists — keep that helper minimal, just a `filter`).
  - Grep `AgentDecisionSystem` to find the existing agent-by-AgentId
    entity resolution pattern; reuse it.
  - Pass through `classify_event()` for every event from the
    `events_this_tick` iterator. `None` → skip. `Some((salience,
    valence, agents))` → for each agent in the vec, find the entity
    and insert a `MemoryEntry`.

### Step 3.3: Extend `AgentDecisionSystem`
- Add `CascadeArm` enum + `recency_factor` + `event_id_matches_arm` +
  `memory_weight_delta` (all `fn`-level helpers, not pub).
- Adjust the existing cascade decision-loop to compute per-arm weight
  deltas alongside natural eligibility. The natural-eligibility logic
  is UNCHANGED; the bias adds a `+memory_weight_delta` to a separate
  "adjusted_score" per arm.
- If `adjusted_winner != natural_winner`: emit `MemoryRecalled`,
  emit `AgentDecision{MemoryReason}` with parent linkage, apply the
  bias-driven `Seeking { target }` transition.
- If `adjusted_winner == natural_winner`: cascade proceeds as Phase
  5/6/7 unchanged. NO `MemoryRecalled` emitted.

### Step 3.4: Register `MemorySystem`
- `runtime/mod.rs`: `pub mod memory;`.
- `lib.rs`: `pub fn register_memory_systems(engine: &mut SimEngine) {
   engine.register_system(Box::new(MemorySystem));
  }`.
- Append `register_memory_systems(engine);` to
  `register_default_runtime_systems` after `register_social_systems(engine);`.

### Step 3.5: Create the integration harness
Write `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs`
with ≥12 `#[test] fn harness_p8_beta_aN_*` functions covering §2
P8β-HARNESS-1 list. The cascade-flip scenario (A12) needs careful
setup: construct an engine where Hunger/Thirst/Fatigue are pinned
zero-growth, Construction is naturally eligible, and an early
positive-valence Social memory has been manually injected into the
agent's Memory with high enough salience that
`memory_weight_delta(Social arm)` exceeds the natural Construction
score. Reuse the agent-construction pattern from
`harness_p7_beta_social_system.rs` for engine setup.

### Step 3.6: Verify
```bash
cd rust
cargo build --workspace 2>&1 | tail -5
cargo test --workspace 2>&1 | grep -E "test result:|FAIL" | tail -50
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
```

Expected:
- Build clean.
- Test count: prior 787 + (5-10 new inline tests in `event.rs` +
  `memory_system.rs`) + 12-17 new harness = roughly 805-820 passed; 0 failed.
- Clippy clean.
- All p3/p4/p5/p6/p7-α/β/γ + p8-α harness suites unchanged.
- Phase 7-γ social chronicle: still CLEAN (no MemoryRecalled events
  emitted in that 80-tick scenario because the chronicle starts with
  empty Memory and 80 ticks is below the threshold where any encoded
  memory's weight delta could flip Phase 7-γ's pinned cascade).

### Step 3.7: Scope boundaries — DO NOT
- DO NOT modify `Memory` or `MemoryEntry` (Phase 8-α scope locked).
- DO NOT modify `MEMORY_CAP` or `SALIENCE_FLOOR` values.
- DO NOT add new SimResources fields (Memory is per-agent component;
  planning §β explicit).
- DO NOT modify Hunger/Thirst/Sleep/Construction/SocialInteraction/
  SocialDecay system bodies.
- DO NOT modify `TargetKind`, `AgentState`, or `Position`.
- DO NOT touch any `scripts/` or `scenes/` path.
- DO NOT add FFI surface to `sim-bridge` (Phase 8-δ scope, mandate-gated).
- DO NOT add the `MemoryEncoded` variant — planning §β scope locks
  only `MemoryRecalled` as the new variant.

### Step 3.7b: Scope additions explicitly permitted (NOT scope creep)
- **ADD** `CausalLogStorage::lookup(event_id) -> Option<&CausalEvent>`
  helper in `causal/storage.rs`. Required substrate — both
  `classify_event` (Construction parent walk) and `event_id_matches_arm`
  (agent_decision.rs) consume it. Include 2-3 inline tests in
  `storage.rs` covering: hit, miss (evicted), miss (never recorded).
- **ADD** `TileCausalLog::events() -> impl Iterator<Item = &CausalEvent>`
  if not already present (verify by grep; the trait may already expose
  this via `iter()` or similar). The helper above depends on it.
- **EXTEND** the `CausalEventView` match arm in
  `rust/crates/sim-bridge/src/ffi/world_node.rs` to add the
  `CausalEvent::MemoryRecalled` case. Rust's exhaustiveness check
  forces this. Minimum viable: `kind: "memory_recalled"`,
  `agent_id: Some(*agent)`, common fields (`id`, `parent`, `tick`),
  all other shape fields `None`. Phase 8-δ will extend the view shape
  proper; for Phase 8-β this is compile-mandatory only.
- **EXTEND** prior-feature test files where the new `CausalEvent::
  MemoryRecalled` variant or `DecisionReason::MemoryReason` discriminant
  forces exhaustiveness adoption. Confirmed exhaustiveness sites
  (Step 0 grep + Pattern-C recovery observation):
  - `rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs`
    (3 match sites needing `MemoryRecalled => unreachable!()` or
    equivalent skip arm).
  - `rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs`
    (1 match site needing `MemoryRecalled`).
  - `rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs`
    (2 match sites needing `MemoryRecalled`, 1 site needing
    `MemoryReason`).
  - `rust/crates/sim-test/tests/harness_p7_beta_social_system.rs`
    (1 match site needing `MemoryReason`).
  Pattern: replace `=> todo!()` placeholder with
  `=> unreachable!("Phase 8-β: variant not exercised in this harness")`
  or whatever Rust syntax the surrounding match expects (e.g.
  `_ => { /* not under test */ }` may suffice for `match ... { ... }`
  expressions returning `()`). DO NOT add MemoryRecalled assertions to
  these prior-feature tests; only satisfy the exhaustiveness check.

These additions are EXPLICITLY in scope for Phase 8-β because the
planning §β cascade-bias mechanism cannot function without them. They
are NOT scope creep — they are the minimum substrate the planning
mandate requires and the compile-mandated exhaustiveness adoption.

---

## Section 4 — Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|---|---|:---:|---|
| 8β-T1 | `rust/crates/sim-core/src/causal/event.rs` (DecisionReason + CausalEvent + MemoryRecallTrigger + accessors + inline tests) | 🟢 DISPATCH | — |
| 8β-T2 | `rust/crates/sim-systems/src/runtime/memory/{mod.rs, memory_system.rs}` (NEW, full impl + inline tests) | 🟢 DISPATCH | T1 (imports MemoryRecallTrigger) |
| 8β-T3 | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (CascadeArm + helpers + cascade extension) | 🟢 DISPATCH | T1, T2 (imports MemorySystem constants) |
| 8β-T4 | `rust/crates/sim-systems/src/runtime/mod.rs` + `lib.rs` (registration) | 🔴 DIRECT | T2 |
| 8β-T5 | `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs` (NEW, ≥12 assertions) | 🟢 DISPATCH | T1, T2, T3, T4 |
| 8β-T6 | Build + test + clippy verification | 🔴 DIRECT | T1-T5 |

Dispatch ratio: 4/6 = 67%. Phase 7-β used same proportion.

---

## Section 5 — Localization Checklist

No new locale keys. Phase 8-β is backend-only. The `MemoryReason`
`as_str()` discriminator is internal (used by causal-log JSON view, not
player-facing UI). UI-facing memory rendering with locale keys is
Phase 8-δ scope, mandate-gated.

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
Numbers: prior 787 + (5-10 inline) + (12-17 harness) ≈ 805-820. Generator
should record actual count.

### Pipeline expectation
- Lane: `--full` (sim-core/sim-systems `.rs` edits across 4+ files).
- Cold-tier auto credit via Signal A+B+C+D — threshold drops to **75**.
- VLM: no-godot-scope auto credit (no Godot path touched).
- Issue 14 fix (`ebbf6ddc`): FFI vacuous check returns CONFIRMED on this
  diff (no `sim-bridge/` files touched) → full FFI credit.
- Pattern D mitigation: substantial scope; attempt-1 APPROVE is the
  target but attempt-2 or attempt-3 is plausible for the cascade-flip
  helper (`event_id_matches_arm` correctness has been a Codex iteration
  point before — Phase 7-β / Phase 7-γ precedent).
- Pattern B mitigation: pipeline graceful fallback verified live during
  Phase 8-α dispatch (Codex 600s timeout → Claude Code fallback in 81s).
- Score expectation: **90-100/100**. Attempt-1 APPROVE: 100. Attempt-3:
  90 (-10 Pattern D). Cold-tier threshold 75 ≪ exceeded either way.

### Notion
No Notion page update — Phase 8 tracking lives in `.harness/audit/
v7_progress.md` (next governance update appends "Phase 8-β landed" on
commit).

### Post-commit
- `git push origin lead/main`.
- `git ls-remote origin refs/heads/lead/main` MUST match local HEAD.
- Commit body follows the
  `feat(p8-beta-memory-system): implementation [harness: plan x... code
  x... eval:APPROVE(codex) visual:... ffi:... regr:CLEAN]` format from
  Phase 7-α/β/γ + Phase 8-α commits.

### In-game verification (Section 7 — backend-only feature)
None. Phase 8-β adds runtime memory encoding/decay + cascade-bias
emission, but no Godot side observable behaviour until Phase 8-δ
(SimBridge surface) or via chronicle harness output. Phase 8-γ
chronicle (next dispatch after this one) is the human-observable
proof that memory-biased decisions occur. Phase 8-β proof is purely
through the ≥12 assertion harness.
