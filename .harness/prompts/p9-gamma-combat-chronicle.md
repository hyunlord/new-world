---
feature: p9-gamma-combat-chronicle
phase: 9-γ
lane: --full
seed: 42
---

# P9γ — Combat Chronicle End-to-End Harness

## Goal

Create `rust/crates/sim-test/tests/harness_p9_gamma_combat_chronicle.rs` with **≥13 assertions**
proving the **complete end-to-end causal chain** from social interaction through memory-driven
combat triggering:

```
SocialInteractionCompleted → MemoryEncoded → MemoryRecalled{CombatContext}
  → AgentDecision{CombatReason} → CombatStarted → CombatCompleted
```

The test runs organically: real systems execute, events are emitted naturally, no fake stubs.
Only the initial combat memory substrate is pre-populated (same pattern as p9-beta
`setup_pair_with_combat_memory`).

## Substrate (Step 0 verified)

### Constants
| Constant | Value | Source |
|---|---|---|
| `SOCIAL_THRESHOLD` | `50.0` | `agent_decision.rs` |
| `SOCIAL_CONSUME_AMOUNT` | `30.0` | `agent_decision.rs:246` |
| `REQUIRED_INTERACTION_PROGRESS` | `3` | p7-γ harness confirms |
| `BIAS_FLIP_THRESHOLD` | `1.0` | `agent_decision.rs:192` |
| `DAMAGE_PER_COMBAT_TICK` | `10.0` | p9-β A1 locked |
| `REQUIRED_COMBAT_PROGRESS` | `1` | p9-β A2 locked |
| `DEFAULT_MAX_HP` | `100.0` | p9-α locked |
| `HOSTILITY_BUMP` | `0.1` | `sim-core/components/relationship.rs` |
| `DECAY_RATE` | `0.001` | `memory_system.rs` |
| `MAX_RECENCY_TICKS` | `4380` | `memory_system.rs` |

### classify_event mapping (memory_system.rs)
| Event | salience | valence | Agents encoded |
|---|---|---|---|
| `SocialInteractionCompleted` | 0.8 | +0.7 | both participants |
| `CombatStarted` | 0.8 | -0.6 | attacker only |
| `CombatCompleted` | 0.9 | -0.8 | attacker + defender |
| `AgentDecision{CombatReason}` | — | — | None (anti-recursion) |
| `MemoryRecalled` | — | — | None (anti-recursion) |

### System execution order (ascending priority = runs first within a tick)
1. `AgentMovementSystem`: 120
2. `AgentDecisionSystem`: 125
3. `ConstructionSystem`: 133
4. `SocialInteractionSystem`: 134  ← emits SocialInteractionCompleted
5. `MemorySystem`: 136             ← encodes events at `current_tick`
6. `CombatSystem`: 137             ← resolves combat; directly encodes CombatCompleted

**Key consequences of this ordering:**
- SIS(134) emits SIC; MemorySystem(136) encodes it — both in the same tick.
  The newly encoded SIC MemoryEntry starts at unmodified salience (decay runs in Phase 1 of
  MemorySystem on existing entries only; new entries skip decay).
- CombatSystem(137) runs AFTER MemorySystem(136), so CombatCompleted is not encodable by
  MemorySystem. CombatSystem encodes it directly into both agents' Memory.

### event_id_matches_arm (Combat arm)
These event types match `CascadeArm::Combat`:
- `CausalEvent::CombatStarted { .. }`
- `CausalEvent::CombatCompleted { .. }`
- `CausalEvent::AgentDecision { reason: DecisionReason::CombatReason, .. }`

`SocialInteractionCompleted` matches `CascadeArm::Social` only — it does NOT affect `combat_delta`.

### memory_weight_delta formula
```
combat_delta = Σ (entry.valence × entry.salience × recency_factor(entry.encoded_tick, current_tick))
  for all entries where event_id_matches_arm(entry.event_id, Combat, causal_log)
```
`recency_factor(t, now) = max(0.0, 1.0 - (now - t) / MAX_RECENCY_TICKS)`

At tick 4, 2 pre-populated entries (valence=-0.8, salience=0.9-4×0.001≈0.896, recency≈0.999):
`combat_delta ≈ 2 × (-0.8 × 0.896 × 0.999) ≈ -1.43 < -BIAS_FLIP_THRESHOLD(-1.0)` ✓

## Chronicle Scenario

### Setup
- World: 128×128 tiles
- Two agents spawned at the same tile `(SHARED_X=5, SHARED_Y=5)`
- First-spawned agent becomes attacker (smaller `AgentId` by construction — verify with assertion)
- Components for **both** agents:
  - `AgentState::Idle`
  - `Social` with `loneliness = 60.0` (above `SOCIAL_THRESHOLD=50.0`), `social_fill = 1.0`
  - `Memory::new()`
  - `BodyHealth::new()` (default 100.0 HP)
  - **NO** `Hunger`, **NO** `Thirst`, **NO** `Sleep` (keeps needs arms silent)
- **Attacker's causal_log** (tile index `SHARED_Y * W + SHARED_X`): 2 synthetic
  `AgentDecision { reason: CombatReason, .. }` events at tick=0, with ids `ev_id_a` and `ev_id_b`
- **Attacker's Memory**: 2 `MemoryEntry` entries pointing to `ev_id_a` and `ev_id_b`
  with `valence=-0.8`, `salience=0.9`, `encoded_tick=0`

This setup mirrors `setup_pair_with_combat_memory` from `harness_p9_beta_combat_system.rs` but
the agents also have `Social` components so the organic SIC chain fires first.

### Phase 1 — Social Interaction (ticks 1–3, up to PHASE1_MAX=10)

| Tick | AgentDecision(125) | SIS(134) | MemorySystem(136) | CombatSystem(137) |
|------|-------------------|----------|-------------------|-------------------|
| 1 | Both Idle → Consuming{Agent} (loneliness=60>50) | progress=1 | encodes AgentDecision{SocialReason} | — |
| 2 | still Consuming | progress=2 | encodes events | — |
| 3 | still Consuming | progress=3 → **SIC emitted**; loneliness-=30=30; both→Idle | **encodes SIC**: both agents get `MemoryEntry(SIC.id, tick=3, valence=0.7, salience=0.8)` | — |

After tick 3:
- `SocialInteractionCompleted` is in causal_log (tick=3)
- Both agents `Memory` contains a `MemoryEntry` for `SIC.id`
- Both agents `AgentState::Idle`
- Both agents `loneliness = 30.0` (< `SOCIAL_THRESHOLD=50.0`)

### Phase 2 — Memory-Driven Combat (tick 4, up to PHASE2_MAX=5 after SIC)

| System | Action |
|--------|--------|
| AgentDecision(125) | Attacker: Idle, loneliness=30<50 → no social. combat_delta≈-1.43 < -1.0 → **combat arm** |
| AgentDecision(125) | Emits `MemoryRecalled{CombatContext{def_id}}` on attacker's tile |
| AgentDecision(125) | Emits `AgentDecision{CombatReason}(parent=recall.id)` |
| AgentDecision(125) | Emits `CombatStarted(parent=decision.id, attacker=att_id, defender=def_id)` (att_id < def_id) |
| AgentDecision(125) | `combat_pairs.insert((att_id, def_id))`; attacker → `Consuming{Agent(def_id)}` |
| MemorySystem(136) | Encodes `CombatStarted` for attacker only (valence=-0.6, salience=0.8) |
| CombatSystem(137) | progress=0+1=1 >= 1 → `CombatCompleted` emitted; HP reduced; both → Idle |
| CombatSystem(137) | Direct memory encoding: **both** agents get `MemoryEntry(CC.id, tick=4, valence=-0.8, salience=0.9)` |
| CombatSystem(137) | `relationships[(att_id,def_id)].hostility += HOSTILITY_BUMP` |

## Assertions (16)

All assertions are **Type A** (structural invariants — values locked by prior harness contracts).

### Phase 1 — Social Interaction Evidence

**A1** [Type A]: Regression anchor — `SOCIAL_CONSUME_AMOUNT == 30.0`
```rust
assert_eq!(SOCIAL_CONSUME_AMOUNT, 30.0_f64);
```

**A2** [Type A]: `SocialInteractionCompleted` fires within `PHASE1_MAX=10` ticks for the
co-located pair. The test loops and records `T_sic` and `sic_id` when SIC fires; if the loop
exhausts without SIC, the test panics with a diagnostic.

**A3** [Type A]: `SIC.agents == (att_id, def_id)` (canonical `(smaller, larger)` ordering).

**A4** [Type A]: `SIC.parent.is_some()` — SIC carries a parent link to the originating
`SocialInteractionStarted` event (proves causal chain is not a root event).

### Phase 1b — Memory Encoded (SIC MemoryEntry)

**A5** [Type A]: After tick `T_sic`, attacker's `Memory` contains a `MemoryEntry` with
`event_id == sic_id` AND `valence == 0.7` (±1e-9) AND `salience == 0.8` (±1e-9).
(New entry is encoded at unmodified salience; decay only applies to pre-existing entries.)

**A6** [Type A]: After tick `T_sic`, defender's `Memory` also contains a `MemoryEntry` with
`event_id == sic_id`. (Both participants are encoded per `classify_event` returning `[id_a, id_b]`.)

**A7** [Type A]: After tick `T_sic`, both agents are `AgentState::Idle`.

**A8** [Type A]: After tick `T_sic`, attacker's `loneliness == INITIAL_LONELINESS - SOCIAL_CONSUME_AMOUNT == 30.0`
(strictly below `SOCIAL_THRESHOLD=50.0`, ensuring social arm is silent next tick).

### Phase 2 — MemoryRecalled{CombatContext} Chain

**A9** [Type A]: `MemoryRecalled` with `triggered_by == CombatContext { agent_id: def_id }` fires
within `PHASE2_MAX=5` ticks after `T_sic`. Record `T_combat`, `recall_id`, `recalled_event_id`.

**A10** [Type A]: `AgentDecision { reason: CombatReason, .. }` fires on the same tick `T_combat`.
Record `decision_id`.

**A11** [Type A]: The `AgentDecision{CombatReason}` event has `parent == Some(recall_id)`.

**A12** [Type A]: `CombatStarted { attacker: att_id, defender: def_id, .. }` fires on tick `T_combat`.
Record `started_id`.

**A13** [Type A]: `CombatStarted.parent == Some(decision_id)`.

### Phase 3 — CombatCompleted

**A14** [Type A]: `CombatCompleted` fires on the same tick `T_combat`
(REQUIRED_COMBAT_PROGRESS=1 → same-tick resolution; CombatSystem runs after AgentDecision
within the same engine.tick() call).

**A15** [Type A]: `CombatCompleted.parent == Some(started_id)`.

**A16** [Type A]: After tick `T_combat`, **both attacker and defender** each have a `MemoryEntry`
with `event_id == cc_id` AND `valence == -0.8` (±1e-9) AND `salience == 0.9` (±1e-9).
(CombatSystem directly encodes for both; the new entries start at unmodified salience.)

## Test File Structure

```
rust/crates/sim-test/tests/harness_p9_gamma_combat_chronicle.rs
```

### Module-level doc comment
```rust
//! V7 Phase 9-γ — Combat Chronicle end-to-end harness.
//!
//! feature: p9-gamma-combat-chronicle
//! plan_attempt: 1
//! code_attempt: 1
//! seed: 42
//! lane: --full
//!
//! 16 assertions (A1-A16) proving:
//!   SocialInteractionCompleted → MemoryEncoded → MemoryRecalled{CombatContext}
//!   → AgentDecision{CombatReason} → CombatStarted → CombatCompleted
```

### Constants
```rust
const W: u32 = 128;
const H: u32 = 128;
const SHARED_X: u32 = 5;
const SHARED_Y: u32 = 5;
const PHASE1_MAX: u64 = 10;    // max ticks to wait for SocialInteractionCompleted
const PHASE2_MAX: u64 = 5;     // max ticks to wait for CombatStarted after SIC
const INITIAL_LONELINESS: f64 = 60.0;  // > SOCIAL_THRESHOLD=50.0
```

### Helper: `fresh_engine()`
```rust
fn fresh_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}
```

### Helper: find events from causal_log
Provide helper closures or functions similar to `harness_p8_gamma_memory_chronicle.rs`:
- `find_social_interaction_completed(engine) -> Option<CausalEvent>`
- `find_memory_recalled_combat(engine, def_id) -> Option<CausalEvent>`
- `find_combat_started(engine) -> Option<CausalEvent>`
- `find_combat_completed(engine) -> Option<CausalEvent>`
- `find_agent_decision_combat(engine) -> Option<CausalEvent>`

### Single test function
```rust
#[test]
fn harness_p9_gamma_a_complete_combat_chronicle() {
    // Setup + Phase 1 loop + Phase 1b assertions + Phase 2 loop + Phase 2-3 assertions
}
```

### Imports (key ones)
```rust
use sim_core::causal::{CausalEvent, DecisionReason, EventId, MemoryRecallTrigger};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Memory, MemoryEntry, Social,
    DEFAULT_MAX_HP,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::combat::{DAMAGE_PER_COMBAT_TICK, REQUIRED_COMBAT_PROGRESS};
use sim_systems::runtime::memory::DECAY_RATE;
use sim_systems::runtime::decision::{BIAS_FLIP_THRESHOLD, SOCIAL_CONSUME_AMOUNT, SOCIAL_THRESHOLD};
```

(Check re-exports from sim_systems::runtime::* — mirror the import style from
`harness_p7_beta_social_system.rs` and `harness_p9_beta_combat_system.rs`.)

## Precedent Files (read these before implementing)

1. `rust/crates/sim-test/tests/harness_p8_gamma_memory_chronicle.rs` — phased tick loop pattern,
   multi-phase chronicle approach, event collection idioms.
2. `rust/crates/sim-test/tests/harness_p7_gamma_social_chronicle.rs` — `SocialInteractionCompleted`
   organic firing pattern, loneliness setup.
3. `rust/crates/sim-test/tests/harness_p9_beta_combat_system.rs` — `setup_pair_with_combat_memory`
   helper (pre-populate causal_log + MemoryEntries), causal_log search helpers, combat event checks.

## Out of Scope

These are already covered by earlier harnesses — do NOT add them to this file:
- Multiple simultaneous pairs (p9-β A29)
- Determinism across runs (p9-β A27)
- Defender death / HP=0 path (p9-β A12)
- Anti-recursion guard (p9-β A20)
- Phase 8-α memory exports (p9-β A24)

## Acceptance

The test file compiles, `cargo test -p sim-test harness_p9_gamma -- --nocapture` shows all 16
assertions passing, and `cargo clippy --workspace --all-targets -- -D warnings` is clean.
