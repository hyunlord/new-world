# P7-α — `Social` + `RelationshipKey`/`RelationshipState` components + `TargetKind::Agent(AgentId)` 5th variant

> Lane: `--full` (sim-core `.rs` edits — new component modules + `TargetKind`
> enum extension + `AgentDecisionSystem` exhaustive-arm extension force full lane).
> Scope: First sub-stage of V7 Phase 7 (Multi-agent Social System). Lands the
> data substrate for agent-to-agent interaction. No runtime system, no causal
> event variants, no `AgentDecisionSystem` runtime resolution — Phase 6-α
> inert-arm precedent applies.
> Governance: v3.3.17. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/`.tres`,
> no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto credit
> expected.

---

## Section 1 — Implementation Intent

V7 Foundation (Week 1-12) closed at `66435f06` / `a0666b6c`. Section 8+ design
(`0ed3ec16`) anchored Phase 7 to Multi-agent Social System. The `.harness/plans/
phase7.md` plan (394 lines) locks 8 P7Plan-* decisions; this dispatch executes
Phase 7-α exactly per `phase7.md §3` "Phase 7-α" block.

Phase 7-α is structurally identical to Phase 6-α (`ba4e02b2`):
- Add new component module(s) for the agent-to-agent substrate.
- Extend `TargetKind` with one new variant (5th).
- Extend `AgentDecisionSystem` only enough to **preserve match-arm
  exhaustiveness** — runtime resolution is β scope, mirroring the
  Phase 6-α inert `Consuming { ConstructionSite } => { /* no-op */ }` placeholder
  at `agent_decision.rs:287-291` and the Seeking branch's
  `TargetKind::ConstructionSite => false` placeholder at
  `agent_decision.rs:213-221`.

**Key difference from Phase 6-α**: this is the **first payload-carrying
`TargetKind` variant**. `TargetKind::Agent(AgentId)` embeds the partner's
`AgentId` (a `u64` newtype-equivalent alias), unlike the four unit variants
`Food` / `Water` / `Sleep` / `ConstructionSite`. This forces a careful look at
derives — see Section 2 locked fact P7α-DERIVES.

After P7-α:
- `sim_core::components::social` module exists, exporting `Social`.
- `sim_core::components::relationship` module exists, exporting `RelationshipKey`
  and `RelationshipState`.
- `TargetKind` has **5 variants**: `Food`, `Water`, `Sleep`, `ConstructionSite`,
  `Agent(AgentId)`.
- `AgentState` enum **body is byte-for-byte unchanged** (no new variant —
  Phase 5-γ Sleep / Phase 6-α ConstructionSite symmetry precedent).
- A ≥12-assertion harness `harness_p7_alpha_social_components.rs` proves
  component construction, serde, `RelationshipKey` canonicalisation, saturation,
  `TargetKind::Agent` exhaustiveness, and Phase 6 regression.
- **Zero** runtime system changes (no Phase 7-β `SocialInteractionSystem` yet).
- **Zero** `CausalEvent` changes.
- **Zero** `DecisionReason` changes.
- **Zero** `SimResources` schema changes (Phase 7-β lands the
  `relationships` + `interaction_progress` HashMaps).

---

## Section 2 — Locked facts (from pre-grep + plan §3 — must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P7α-1: `Social` struct contract | plan §3 §α + Sleep mirror | `pub struct Social { pub loneliness: f64, pub growth_rate: f64 }`. `SATURATION = 100.0_f64`. `new(initial, growth_rate)` clamps `initial` to `[0.0, SATURATION]`. `tick()` does saturating add of `growth_rate` then floors at `0.0`. Direct structural mirror of `Sleep` (verified at `sim-core/src/components/sleep.rs`). |
| P7α-2: `Social` derives | plan §3 §α | `#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]`. Exact same set as `Sleep`. NOT `Eq` (f64 fields). |
| P7α-3: file layout | plan §3 §α (explicit) | **Two separate files**: `social.rs` and `relationship.rs`. NOT consolidated into one file. Plan §3 §α explicitly lists `Files added: social.rs ... relationship.rs ...`. |
| P7α-4: `RelationshipKey` contract | plan §3 §α | `pub struct RelationshipKey(pub AgentId, pub AgentId)`. **Canonicalisation enforced by constructor**: `RelationshipKey::new(a, b)` returns `RelationshipKey(min(a,b), max(a,b))`. Two getter accessors: `pub fn smaller(&self) -> AgentId` returns `.0`; `pub fn larger(&self) -> AgentId` returns `.1`. Direct field access (`.0`, `.1`) remains possible since fields are `pub` — but callers who use `new()` are guaranteed the canonical order. |
| P7α-5: `RelationshipKey` derives | plan §3 §α | `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`. `Hash + Eq` mandatory because Phase 7-β uses `HashMap<RelationshipKey, RelationshipState>`. |
| P7α-6: `RelationshipState` contract | plan §3 §α | `pub struct RelationshipState { pub familiarity: f64 }`. `SATURATION = 1.0_f64`. `RelationshipState::new()` returns `Self { familiarity: 0.0 }`. `bump(amount: f64)` does saturating add of `amount`, capping at `SATURATION`, flooring at `0.0`. |
| P7α-7: `RelationshipState` derives | plan §3 §α | `#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]`. NOT `Eq` / `Hash` — it is a value, not a key. |
| P7α-8: `TargetKind::Agent` variant | plan §3 §α | Append `Agent(AgentId)` as the **5th variant** after `ConstructionSite`. Order: `Food, Water, Sleep, ConstructionSite, Agent(AgentId)`. Payload type is `AgentId` (verified at `sim-core/src/components/agent.rs` = `pub type AgentId = u64`). |
| P7α-DERIVES: `TargetKind` derives | grep-verified at `agent_state.rs:26` | Current set: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`. **Unchanged.** `AgentId = u64` already satisfies `Debug + Clone + Copy + PartialEq + Eq + Hash + Serialize + Deserialize`, so `TargetKind` keeps its existing derive set. **Do NOT add `Hash` to `TargetKind` itself** — no current consumer hashes a `TargetKind` and the derive set should grow only when there is a callsite requirement. |
| P7α-9: `AgentState` unchanged | plan §3 §α + Phase 6-α/5-γ symmetry | `AgentState` enum body is byte-for-byte unchanged. Only its test extensions grow. |
| P7α-10: suppresses_movement contract | `agent_state.rs:76-78` existing | `AgentState::suppresses_movement` returns `true` only for `Seeking { .. }` (any target). `Consuming { Agent(_) }` returns `false`. Mirrors Phase 5-γ Sleep and Phase 6-α ConstructionSite contracts exactly. |
| P7α-11: mod.rs exports | `components/mod.rs:21-33` existing | Insert `pub mod relationship;` and `pub mod social;` in alphabetical position. Re-exports: `pub use relationship::{RelationshipKey, RelationshipState};` and `pub use social::Social;` in alphabetical position. |
| P7α-12: exhaustive-arm placeholders in `agent_decision.rs` | plan §3 §α Verification + Phase 6-α inert precedent | Add **inert** arms for `TargetKind::Agent(_)` at both Phase 6-α-style call sites: (a) the Seeking-branch `has_resource` switch at `agent_decision.rs:213-221` returns `false` for `Agent(_)`; (b) the Consuming-branch big match at `agent_decision.rs:287-291` adds `Agent(_) => { /* no-op, Phase 7-β scope */ }`. Both must match the doc-comment style of the existing Construction placeholders. No runtime semantics change. |
| P7α-13: SimResources unchanged | plan §3 §α | **Zero changes** to `SimResources` in this dispatch. `relationships` and `interaction_progress` HashMaps land in Phase 7-β alongside the system that uses them. |
| P7α-14: Constants out of scope | plan §3 §β | `SOCIAL_THRESHOLD`, `SOCIAL_CONSUME_AMOUNT`, `REQUIRED_INTERACTION_PROGRESS`, `FAMILIARITY_BUMP` are **Phase 7-β** constants exported from `agent_decision.rs`. Phase 7-α does NOT define them. The only constants this dispatch ships are `Social::SATURATION = 100.0` and `RelationshipState::SATURATION = 1.0`. |
| Harness assertion count | axiom #1 + plan §3 §α + Phase 6-α precedent | **≥12 assertions** in `harness_p7_alpha_social_components.rs`. Each locked fact P7α-1 through P7α-12 + 1 regression assertion. |

**Determinism rationale** (axiom #2 — verified before assuming):
1. New components are plain-old-data structs. No RNG, no clock, no allocation
   in hot paths.
2. `RelationshipKey::new()` is a pure `min/max` over `u64` — deterministic.
3. `Social::tick()` is saturating arithmetic on `f64` — deterministic, no FP
   accumulator drift hazard (one `+` per tick, then `min`/`max` clamps).
4. `RelationshipState::bump()` is saturating arithmetic on `f64` — same
   determinism property.
5. `TargetKind::Agent(AgentId)` is `Copy + Eq + Hash` via its `AgentId = u64`
   payload, preserving the existing enum's value semantics.

---

## Section 3 — What to build (4 files added + 2 files modified)

### 3.1 `rust/crates/sim-core/src/components/social.rs` (NEW)

Module header doc-comment (≥8 lines):
- "V7 Phase 7-α / P7α-1 — Social need component."
- Describe purpose: tracks per-agent `loneliness` need, mirror of `Sleep`.
- Reference `.harness/plans/phase7.md §3` and Phase 7 anchor in
  `.harness/audit/section_8_plus_design.md`.
- Note Phase 5-γ Sleep precedent (direct structural mirror).
- Note that the `SocialInteractionSystem` priority 134 (β scope) consumes
  this component.

Module body — direct mirror of `sleep.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Per-agent social need state.
///
/// `loneliness` is the current loneliness level (0 = fully social,
/// [`Social::SATURATION`] = lonely). `growth_rate` is the per-tick
/// increment applied by a future SocialDecaySystem (Phase 7-β scope).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Social {
    pub loneliness: f64,
    pub growth_rate: f64,
}

impl Social {
    pub const SATURATION: f64 = 100.0;

    pub fn new(initial: f64, growth_rate: f64) -> Self {
        Self {
            loneliness: initial.clamp(0.0, Self::SATURATION),
            growth_rate,
        }
    }

    pub fn tick(&mut self) {
        self.loneliness = (self.loneliness + self.growth_rate).min(Self::SATURATION);
        if self.loneliness < 0.0 {
            self.loneliness = 0.0;
        }
    }
}

#[cfg(test)] mod tests { /* mirror sleep.rs tests */ }
```

`#[cfg(test)]` tests covering (minimum, mirroring `sleep.rs`):
- `SATURATION` constant value
- `new()` clamps negative initial
- `new()` clamps over-SATURATION initial
- `tick()` adds growth_rate
- `tick()` clamps at SATURATION
- serde RON round-trip

### 3.2 `rust/crates/sim-core/src/components/relationship.rs` (NEW)

Module header doc-comment (≥8 lines):
- "V7 Phase 7-α / P7α-4 — Per-pair relationship state."
- Describe `RelationshipKey` canonicalisation invariant (smaller AgentId first).
- Describe `RelationshipState::familiarity` as the single Phase 7 relationship
  attribute (richer types deferred to Section 9+ per Section 8+ design §2).
- Note that Phase 7-β stores `HashMap<RelationshipKey, RelationshipState>` on
  `SimResources` and increments `familiarity` by `FAMILIARITY_BUMP = 0.1` per
  completed `SocialInteractionCompleted` event.

Module body:

```rust
use serde::{Deserialize, Serialize};

use crate::components::agent::AgentId;

/// Canonicalised key for a per-pair relationship lookup.
///
/// `RelationshipKey::new(a, b)` always orders the smaller AgentId first.
/// Both fields are `pub` so direct construction is possible, but callers
/// that go through `new()` are guaranteed canonical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipKey(pub AgentId, pub AgentId);

impl RelationshipKey {
    pub fn new(a: AgentId, b: AgentId) -> Self {
        if a <= b { Self(a, b) } else { Self(b, a) }
    }
    pub fn smaller(&self) -> AgentId { self.0 }
    pub fn larger(&self) -> AgentId { self.1 }
}

/// Per-pair relationship state. Phase 7 ships only `familiarity` —
/// richer relationship semantics (kinship, rivalry, dependency, etc.)
/// are deferred to Section 9+.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationshipState {
    pub familiarity: f64,
}

impl RelationshipState {
    pub const SATURATION: f64 = 1.0;

    pub fn new() -> Self {
        Self { familiarity: 0.0 }
    }

    pub fn bump(&mut self, amount: f64) {
        self.familiarity = (self.familiarity + amount).clamp(0.0, Self::SATURATION);
    }
}

impl Default for RelationshipState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)] mod tests { /* canonicalisation + saturation + serde */ }
```

`#[cfg(test)]` tests covering:
- `RelationshipKey::new(a, b)` canonicalises both orderings
- `smaller()` / `larger()` accessors return the right field
- Same-AgentId pair (`new(a, a)`) is valid and produces `(a, a)`
- `RelationshipState::new()` initializes `familiarity = 0.0`
- `bump(0.1)` accumulates correctly
- `bump()` saturates at 1.0 across multiple bumps
- `bump()` with negative amount floors at 0.0
- serde RON round-trip for both types

### 3.3 `rust/crates/sim-core/src/components/agent_state.rs` (MODIFY)

Single locus of change: extend `TargetKind` with `Agent(AgentId)` as the 5th
variant. Update the doc comment on `TargetKind`:

- "Phase 6-α scope: four variants. Adding a target requires updating both the
  AgentDecisionSystem FSM and any UI that surfaces agent state."

becomes:

- "Phase 7-α scope: five variants. Adding a target requires updating both the
  AgentDecisionSystem FSM and any UI that surfaces agent state."

Add per-variant doc comment for `Agent(AgentId)`:

> "V7 Phase 7-α / P7α-8 — Co-located partner agent for the Multi-agent Social
> System (Phase 5-γ Sleep / Phase 6-α ConstructionSite payload symmetry
> extended to AgentId). First payload-carrying `TargetKind` variant. The
> embedded `AgentId` identifies the partner; canonicalisation across a pair is
> handled by `RelationshipKey::new`, not by this variant."

Add `use` statement for `AgentId` at the top of the file (it's already in the
same module tree but `agent_state.rs` doesn't currently import it).

Extend the three existing tests to cover `Agent(AgentId)`:
- `only_seeking_suppresses_movement` — assert
  `Seeking { Agent(7) }.suppresses_movement() == true` and
  `Consuming { Agent(7) }.suppresses_movement() == false`.
- `serde_round_trip_each_variant` — add `Seeking { target: TargetKind::Agent(7) }`
  and `Consuming { target: TargetKind::Agent(7) }` to the `cases` array.
- `target_kind_serde_round_trip` — add `TargetKind::Agent(7)` to the iteration
  array.

No new variants in `AgentState`. No other changes.

### 3.4 `rust/crates/sim-core/src/components/mod.rs` (MODIFY)

Three insertions:
1. `pub mod relationship;` between `pub mod position;` and `pub mod sleep;`
   (alphabetical position).
2. `pub mod social;` between `pub mod sleep;` and `pub mod thirst;`
   (alphabetical position).
3. `pub use relationship::{RelationshipKey, RelationshipState};` between
   `pub use position::Position;` and `pub use sleep::Sleep;`.
4. `pub use social::Social;` between `pub use sleep::Sleep;` and
   `pub use thirst::Thirst;`.

Module-level doc-comment gets a one-line extension noting the Phase 7-α
addition (mirror the existing Phase 5-α/β/γ + Phase 6-α notes at `mod.rs:7-19`).

### 3.5 `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (MODIFY — exhaustive-arm only)

Two inert match-arm additions to keep the workspace compiling. **No runtime
semantics change**. Phase 7-β owns all runtime resolution for
`TargetKind::Agent(_)`.

**(a)** Seeking-branch `has_resource` switch (around lines 196-222 — the
existing block that decides whether a Seeking agent can transition to
Consuming). Add a new arm:

```rust
// V7 Phase 7-α: TargetKind::Agent(_) is the 5th TargetKind variant but α
// adds no decision logic that routes agents into Seeking{Agent(_)} —
// runtime resolution lands in Phase 7-β. If an agent reaches this branch
// (e.g., via a future test or β prototype), treat it as "no partner
// present" so the agent stays in Seeking without resolving against a
// non-existent runtime path. Mirrors the Phase 6-α ConstructionSite inert
// pattern at agent_decision.rs:213-221.
TargetKind::Agent(_) => false,
```

Position: immediately after the existing `TargetKind::ConstructionSite => false`
arm.

**(b)** Consuming-branch big match (around lines 242-292). Add a new arm:

```rust
// V7 Phase 7-α: Agent(_) consume is β scope. The SocialInteractionSystem
// (β, priority 134) is the sole owner of social-interaction progress,
// mutual-handshake validation, and FSM exit; α MUST NOT advance
// familiarity, mutate Social/RelationshipState, or transition the FSM
// state here. The match arm exists only to satisfy exhaustiveness — kept
// strictly inert and state-preserving so Phase 7-β remains the only
// place these semantics land. Mirrors the Phase 6-α ConstructionSite
// inert pattern at agent_decision.rs:287-291.
TargetKind::Agent(_) => {
    // Intentional no-op: preserve state, do not touch relationships,
    // do not bump familiarity, do not advance interaction_progress.
    // β scope.
}
```

Position: immediately after the existing `TargetKind::ConstructionSite => { /*
no-op */ }` arm.

If `agent_decision.rs` has additional exhaustive `match` arms over `TargetKind`
(grep for `match target` or similar inside the file), each must be extended
with the same inert pattern. The Generator must grep for **all** exhaustive
`TargetKind` matches in the file before declaring this section complete.

### 3.6 `rust/crates/sim-test/tests/harness_p7_alpha_social_components.rs` (NEW)

≥12 numbered assertions. Use the Phase 6-α harness file as the structural
template. Single `#[test]` per assertion OR single bundled `#[test]
fn harness_p7_alpha_substantial()` containing all numbered assertions — pick the
shape that maximises clarity of which assertion failed (Phase 6-α used the
bundled shape).

Mandatory assertion coverage (numbered A1-A12+):

1. **A1** `Social::SATURATION == 100.0_f64`. `Social::new(150.0, 1.0).loneliness
   == 100.0` (clamped over). `Social::new(-5.0, 1.0).loneliness == 0.0`
   (clamped under).
2. **A2** `Social::tick()` semantics: starting from `(0.0, 3.0)`, after one
   tick `loneliness == 3.0`, after second tick `6.0`. Saturates at 100.0.
3. **A3** `Social` serde RON round-trip preserves all fields.
4. **A4** `RelationshipKey::new(a, b)` canonicalises both ways:
   `new(7, 3) == new(3, 7)`. Both yield `(3, 7)`.
5. **A5** `RelationshipKey::new(a, a).0 == a` and `.1 == a` (same-AgentId
   pair).
6. **A6** `RelationshipKey::smaller()` returns `.0` and `larger()` returns
   `.1`. After `new(7, 3)`, `smaller() == 3` and `larger() == 7`.
7. **A7** `RelationshipState::new().familiarity == 0.0`. After
   `bump(0.1)`, `familiarity == 0.1`. After 11 successive `bump(0.1)` calls,
   `familiarity == 1.0` (saturates). After `bump(-2.0)` from `0.5`,
   `familiarity == 0.0` (floors).
8. **A8** `RelationshipKey` and `RelationshipState` both serde RON round-trip.
9. **A9** `TargetKind` has 5 variants — exhaustive `match` compile guarantee
   covering all five + an explicit equality check
   `TargetKind::Agent(7) != TargetKind::Agent(8)` and
   `TargetKind::Agent(7) != TargetKind::ConstructionSite`.
10. **A10** `AgentState::Seeking { target: TargetKind::Agent(42) }
    .suppresses_movement() == true`. `AgentState::Consuming { target:
    TargetKind::Agent(42) }.suppresses_movement() == false`.
11. **A11** `AgentState::Seeking { target: TargetKind::Agent(42) }.target() ==
    Some(TargetKind::Agent(42))`. Same for `Consuming`.
12. **A12** Serde RON round-trip for: `TargetKind::Agent(42)`,
    `AgentState::Seeking { target: TargetKind::Agent(42) }`,
    `AgentState::Consuming { target: TargetKind::Agent(42) }`.
13. **A13 (regression)** Spawn an entity in a fresh `hecs::World` carrying
    `Agent { id: 42 }`, `Position { x: 1, y: 1 }`, `Hunger::new(0.0, 1.0)`,
    `Thirst::new(0.0, 0.7)`, `Sleep::new(0.0, 0.5)`, `Social::new(0.0, 0.4)`,
    `AgentState::default()`, plus a co-located `ConstructionSite::new(
    BuildingBlueprint::new(1,2,2,5), Position{x:1,y:1})`. Confirm **all seven**
    components are queryable in one combined `query::<...>()` call. This
    proves the new `Social` component coexists with the entire post-Phase 6
    component set in a single archetype.

Assertion numbering MUST appear as a comment header at the top of the test
file (`// A1: ...`, `// A2: ...`) so the evaluator can map an assertion failure
directly to the spec entry.

---

## Section 4 — Locale

**No new locale keys required.** Phase 7-α is backend only. Locale work is
deferred to Phase 7-δ (optional, gated on user mandate per plan §3 §δ).

---

## Section 5 — Verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -20
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -20
cd rust && cargo test -p sim-test harness_p7_alpha -- --nocapture 2>&1 | tail -50
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
```

Expected:
- `cargo build`: clean.
- `cargo test --workspace`: zero new failures. **All Phase 2-6 harness tests
  still pass** (T7.10.A-F + harness_phase2_substantial + harness_p3_* +
  harness_p4_* + harness_p5_* + harness_p6_* all CLEAN).
- `cargo test -p sim-test harness_p7_alpha`: ≥12 assertions pass.
- `cargo clippy`: zero new warnings.

---

## Section 6 — Lane

`--full`. Forced by `sim-core/src/components/` `.rs` additions +
`sim-systems/src/runtime/decision/agent_decision.rs` edit. Planning debate,
Visual Verify (no-godot-scope auto credit expected), FFI Chain check,
Regression Guard, and Evaluator all run.

---

## Section 7 — 인게임 확인사항

**None.** Phase 7-α adds no FFI, no rendering, no GDScript, no HUD, no Locale
keys. Pipeline VLM is expected to issue **no-godot-scope auto credit** per
v3.3.7 §2 (no `.gd`, `.gdshader`, `.tscn`, `.tres`, no `scripts/` or `scenes/`
path edits).

Phase 7-δ (UI integration — visual rendering of social interaction + agent
"socializing" indicator + `CausalPanel` SocialReason labels) remains the
optional δ sub-stage gated on explicit user mandate. Phase 7-β / γ continue
backend-only.

---

## Self-check before dispatching the Generator

- [x] **Two** new component files (`social.rs` + `relationship.rs`), NOT one
      consolidated file. Plan §3 §α explicit.
- [x] `Social` mirrors `Sleep` structurally exactly.
- [x] `RelationshipKey::new(a, b)` canonicalises smaller-first.
- [x] `RelationshipState::familiarity` saturates at 1.0.
- [x] `TargetKind` is a 5-variant enum after this change — Food, Water, Sleep,
      ConstructionSite, Agent(AgentId). Agent is the **first payload-carrying**
      variant.
- [x] `TargetKind` derive set unchanged: `Debug, Clone, Copy, PartialEq, Eq,
      Serialize, Deserialize`. No `Hash` added (no consumer requires it).
- [x] `AgentState` enum body **byte-for-byte unchanged**.
- [x] `agent_decision.rs` gains exactly **two** inert `Agent(_) =>` arms
      (Seeking-branch `has_resource` switch + Consuming-branch big match), no
      runtime semantics. Phase 6-α inert-arm precedent followed verbatim.
- [x] No `CausalEvent` variants added. No `DecisionReason` variants added.
- [x] No `SimResources` schema change (Phase 7-β scope).
- [x] No Phase 7-β constants exported (Phase 7-β scope).
- [x] No FFI change. No `sim-bridge` change.
- [x] Harness has ≥12 numbered assertions + 1 regression assertion (13 total
      recommended).
