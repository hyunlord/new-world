# P6-α — BuildingBlueprint + ConstructionSite + TargetKind::ConstructionSite (V7 Phase 6 entry)

> Lane: `--full` (sim-core `.rs` edits force full lane; new component module + enum variant force full lane).
> Scope: First sub-stage of V7 Phase 6 (Building System Deep). Lands the
> data substrate for agent-driven construction: a new
> `sim_core::components::construction` module exporting
> `BuildingBlueprint` and `ConstructionSite`, the 4th `TargetKind` variant
> (`ConstructionSite`) following the **Phase 5-γ Path (b) symmetry
> precedent** (NOT a new `AgentState` variant), and a ≥12-assertion
> harness in `sim-test/tests/`.
> Governance: v3.3.17. Visual: backend-leaning (no new HUD surface,
> no `.gd`/`.gdshader`/`.tscn`/`.tres`) — Pipeline VLM no-godot-scope
> auto credit expected.

---

## Section 1 — Implementation Intent

Phase 5 closed cleanly: α delivered `AgentId`, `Hunger`, and the priority-130
`HungerDecaySystem`; β added `Thirst`, the `AgentState` FSM
(`Idle/Seeking/Consuming`), `AgentDecisionSystem` at priority 125, and
`CausalEvent::AgentDecision`; γ added `Sleep`, the day/night clock, priority-132
`SleepDecaySystem`, `TargetKind::Sleep` (Path b), and a full-day chronicle
harness. All three sub-stages APPROVED.

Phase 6 — **Building System Deep (V7 Week 11-12)** — opens with α: the
data-substrate sub-stage. Phase 6 is decomposed in `.harness/plans/phase6.md`
into three locked sub-stages (α component+state / β system+causal /
γ chronicle harness). Per the plan, **α only adds component types and the
4th `TargetKind` variant — no runtime system, no causal event variants, no
agent decision changes**. Those land in β.

The key architectural choice — Path (b) symmetry — was set by precedent in
Phase 5-γ: a 3rd `TargetKind::Sleep` variant was added rather than introducing
a new `AgentState::Sleeping` variant. P6-α follows the same precedent
**exactly**: `TargetKind::ConstructionSite` becomes the 4th variant; the FSM
shape (`Idle / Seeking { target } / Consuming { target }`) is preserved
unchanged. This keeps the β consume-path uniform — once β lands, an agent in
`Consuming { TargetKind::ConstructionSite }` will progress the co-located
`ConstructionSite` exactly the way an agent in `Consuming { TargetKind::Sleep }`
decrements the co-located `Sleep.fatigue`.

After P6-α:
- `sim_core::components::construction` module exists, exporting
  `BlueprintId`, `BuildingBlueprint`, `ConstructionSite`.
- `sim_core::components::TargetKind` has **four variants**: `Food`, `Water`,
  `Sleep`, `ConstructionSite`. `AgentState` is unchanged (no new variant —
  Phase 5-γ Path (b) precedent).
- `sim_core::components::mod` re-exports `BlueprintId`, `BuildingBlueprint`,
  `ConstructionSite` alongside the existing component re-exports.
- A ≥12-assertion harness `harness_p6_alpha_construction_components.rs` in
  `sim-test/tests/` covers component construction, serde round-trip, progress
  monotonicity, `TargetKind::ConstructionSite` exhaustiveness, FSM
  instantiation validity, and Phase 2/3/4/5 regression containment.
- Zero runtime-system changes. Zero `CausalEvent` changes. Zero
  `AgentDecisionSystem` changes. (Those land in β.)

---

## Section 2 — Locked facts (from pre-grep — must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P6α-1: BlueprintId alias | Planning §2.1 + AgentId precedent | `pub type BlueprintId = u64;` exported from `construction.rs`. Mirrors `pub type AgentId = u64;` in `agent.rs`. |
| P6α-2: BuildingBlueprint contract | Planning §2.1 minimum viable + §6 open question #1 minimal resolution | `pub struct BuildingBlueprint { pub id: BlueprintId, pub footprint_width: u32, pub footprint_height: u32, pub required_progress: u32 }`. `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`. Constructor: `pub fn new(id: BlueprintId, footprint_width: u32, footprint_height: u32, required_progress: u32) -> Self`. Convenience: `pub fn footprint(&self) -> (u32, u32) { (self.footprint_width, self.footprint_height) }`. |
| P6α-3: ConstructionSite contract | Planning §2.1 explicit struct shape | `pub struct ConstructionSite { pub blueprint: BuildingBlueprint, pub progress: u32, pub position: Position }`. `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`. Constructor: `pub fn new(blueprint: BuildingBlueprint, position: Position) -> Self` — sets `progress: 0`. |
| P6α-4: progress semantics | Planning §2.1 monotonic increment | `pub fn advance(&mut self) -> bool` — saturating add of 1 to `progress`, capped at `blueprint.required_progress`. Returns `true` exactly when the call transitions `progress` from `< required_progress` to `== required_progress` (one-shot completion edge). Subsequent calls after completion return `false` and leave `progress` at `required_progress`. |
| P6α-5: is_complete helper | Planning §2.1 + Phase 5 saturation idiom | `pub fn is_complete(&self) -> bool { self.progress >= self.blueprint.required_progress }`. Inclusive (`>=`) so a blueprint with `required_progress == 0` is trivially complete at construction (edge case must be exercised by the harness). |
| P6α-6: TargetKind extension | Planning §2.1 + P6Plan-5-b (Path b precedent) | Append `ConstructionSite` as the **4th** variant after `Sleep`. Order: `Food, Water, Sleep, ConstructionSite`. `#[derive(...)]` set unchanged (Debug/Clone/Copy/PartialEq/Eq/Serialize/Deserialize already covers it). |
| P6α-7: AgentState NOT extended | Planning §2.1 + Phase 5-γ Path (b) precedent (P5γ-5 line) | `AgentState` enum body is **unchanged**. No new variant. `Idle / Seeking { target } / Consuming { target }` stays exactly as it is post-Phase 5-γ. The `target` field in Seeking/Consuming gains an additional valid value (`TargetKind::ConstructionSite`) but the enum surface itself does not grow. |
| P6α-8: suppresses_movement contract | agent_state.rs:76-78 existing pattern | `AgentState::suppresses_movement` continues to return `true` for `Seeking { .. }` (any target) and `false` otherwise. `Consuming { TargetKind::ConstructionSite }` therefore does NOT suppress movement — matching the Phase 5-γ Sleep variant exactly (verified at agent_state.rs:111). This is the correct semantic because Phase 6-β will tick construction progress in a dedicated system independent of the movement suppression flag. |
| P6α-9: mod.rs exports | components/mod.rs:21-33 existing pattern | Add `pub mod construction;` (alphabetically positioned between `agent_state` and `hunger`) and `pub use construction::{BlueprintId, BuildingBlueprint, ConstructionSite};` (alphabetically positioned between `agent_state::{AgentState, TargetKind}` and `hunger::Hunger`). |
| P6α-10: serde discipline | Phase 5 precedent (Sleep, Thirst, Hunger all serde round-trip tested) | Both new structs MUST round-trip via `ron::to_string` / `ron::from_str`. `TargetKind::ConstructionSite` MUST extend the existing `target_kind_serde_round_trip` test in agent_state.rs to include the new variant. `AgentState::Seeking { target: TargetKind::ConstructionSite }` and `Consuming { target: TargetKind::ConstructionSite }` MUST extend `serde_round_trip_each_variant` for completeness. |
| P6α-11: only_seeking_suppresses_movement extension | agent_state.rs:103-112 existing test pattern | The existing `only_seeking_suppresses_movement` test MUST be extended to cover the 4th variant in both Seeking (true) and Consuming (false) cases. |
| P6α-12: f64 vs u32 rationale | Phase 5 (Sleep/Thirst f64) vs Phase 6 (progress integer) | `progress` and `required_progress` are `u32`, not `f64`. Phase 5 needs are continuous-valued (fatigue/thirst/hunger) so f64 makes sense. Construction progress is a discrete tick counter — every `ConstructionSystem` tick adds exactly 1 unit. `u32` matches that semantics cleanly and avoids floating-point comparison hazards in completion checks. Determinism preserved trivially. |
| Harness assertion count | Axiom #1 + Phase 5-α/β/γ precedent | **≥12 assertions** in `harness_p6_alpha_construction_components.rs`. The harness is the chronicle of α — every locked fact above (P6α-1 through P6α-12) gets at least one corresponding assertion. |

**Determinism rationale** (axiom #2 — verified before assuming):
1. `BuildingBlueprint` and `ConstructionSite` are pure plain-old-data
   structs. No RNG, no clock, no allocation in hot paths.
2. `advance()` is a saturating integer add — fully deterministic.
3. `TargetKind` derives `Eq` already (existing — no change needed).
4. `Position` is `{ x: u32, y: u32 }` (verified in components/position.rs)
   so the embedded position field in `ConstructionSite` keeps the whole
   struct `Copy`-eligible — preserves the same idiom Phase 5 used.
5. No new hecs archetype churn — `BuildingBlueprint` is embedded in
   `ConstructionSite` (composition, not a separate component) so spawning
   a `ConstructionSite` adds exactly one archetype slot per agent's tile.

---

## Section 3 — What to build (4-file scope)

### 3.1 `rust/crates/sim-core/src/components/construction.rs` (NEW)

Module header doc-comment (≥6 lines):
- "V7 Phase 6-α / P6α-1 — Building-construction data substrate."
- Brief description of the two structs and their roles.
- Reference to `.harness/plans/phase6.md` §2.1.
- Phase 5-γ Path (b) precedent note: "`TargetKind::ConstructionSite` is
  added as a sibling of `Sleep`, not as a new `AgentState` variant —
  preserving the Phase 5-γ symmetry decision."

Module body (in order):

```rust
use serde::{Deserialize, Serialize};

use crate::components::position::Position;

/// Stable per-blueprint identifier. Mirrors `AgentId` (also `u64`).
pub type BlueprintId = u64;

/// Immutable design specification of a building, decoupled from any
/// individual construction in progress.
///
/// `footprint_width` × `footprint_height` define the rectangular tile
/// span the finished building occupies. `required_progress` is the
/// number of `ConstructionSystem` ticks needed to complete construction
/// (β scope — α only defines the field).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingBlueprint {
    pub id: BlueprintId,
    pub footprint_width: u32,
    pub footprint_height: u32,
    pub required_progress: u32,
}

impl BuildingBlueprint {
    pub fn new(
        id: BlueprintId,
        footprint_width: u32,
        footprint_height: u32,
        required_progress: u32,
    ) -> Self { ... }

    pub fn footprint(&self) -> (u32, u32) { ... }
}

/// An in-progress construction at a specific tile. Composed of the
/// immutable [`BuildingBlueprint`] plus mutable progress state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructionSite {
    pub blueprint: BuildingBlueprint,
    pub progress: u32,
    pub position: Position,
}

impl ConstructionSite {
    pub fn new(blueprint: BuildingBlueprint, position: Position) -> Self {
        Self { blueprint, progress: 0, position }
    }

    /// Saturating tick. Returns true exactly when this call transitioned
    /// progress to completion.
    pub fn advance(&mut self) -> bool { ... }

    pub fn is_complete(&self) -> bool { ... }
}

#[cfg(test)] mod tests { ... }
```

Inline `#[cfg(test)]` tests covering (minimum):
- Constructor sets `progress: 0`
- `advance()` increments by 1
- `advance()` saturates at `required_progress`
- `advance()` returns `true` exactly once (the completion edge)
- `is_complete()` matches `>=` semantics
- serde round-trip for both structs
- `required_progress == 0` edge case: `is_complete()` true immediately,
  `advance()` returns `false` on first call (already complete)
- `footprint()` returns `(width, height)` tuple

### 3.2 `rust/crates/sim-core/src/components/agent_state.rs` (MODIFY)

Single locus of change: append `ConstructionSite` to the `TargetKind`
enum after `Sleep` (the 4th variant). The doc comment on `TargetKind`
must be updated:

- "Phase 5-γ scope: three variants. Adding a target requires updating
  both the AgentDecisionSystem FSM and any UI that surfaces agent state."

becomes:

- "Phase 6-α scope: four variants. Adding a target requires updating
  both the AgentDecisionSystem FSM and any UI that surfaces agent state."

Add per-variant doc comment for `ConstructionSite`:

> "V7 Phase 6-α / P6α-6 — Construction site tile (decremented progress
> from `ConstructionSite::advance` once β lands the runtime system).
> Phase 5-γ Path (b) symmetry precedent."

Extend the three existing tests to cover the new variant:
- `only_seeking_suppresses_movement` — assert
  `Seeking { ConstructionSite }.suppresses_movement() == true` and
  `Consuming { ConstructionSite }.suppresses_movement() == false`.
- `serde_round_trip_each_variant` — add
  `Seeking { target: TargetKind::ConstructionSite }` and
  `Consuming { target: TargetKind::ConstructionSite }` to the `cases`
  array.
- `target_kind_serde_round_trip` — add `TargetKind::ConstructionSite` to
  the iteration array.

No other changes to `agent_state.rs`. No new variants in `AgentState`.

### 3.3 `rust/crates/sim-core/src/components/mod.rs` (MODIFY)

Two additions:

1. Add `pub mod construction;` between `pub mod agent_state;` and
   `pub mod hunger;` to preserve alphabetical ordering.
2. Add `pub use construction::{BlueprintId, BuildingBlueprint, ConstructionSite};`
   between `pub use agent_state::{AgentState, TargetKind};` and
   `pub use hunger::Hunger;` to preserve the existing alphabetical
   re-export ordering.

The module-level doc comment gets a one-line extension noting the
Phase 6-α addition (mirror the existing Phase 5-α/β/γ notes at
mod.rs:7-16).

### 3.4 `rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs` (NEW)

≥12 assertions. Use the Phase 5-α/β/γ harness file as the structural
template (single `#[test]` per assertion grouping is acceptable; a single
`#[test] fn harness_p6_alpha_substantial()` containing all assertions
also matches precedent — pick the layout that maximizes clarity of which
assertion failed).

Mandatory assertion coverage (12 minimum, list maps 1:1 to P6α-1 through
P6α-12 + regression — number them in the test source):

1. **A1** `BlueprintId` resolves to `u64` (e.g., `let _: BlueprintId = 0u64;`).
2. **A2** `BuildingBlueprint::new(id, w, h, req)` populates all four
   public fields verbatim.
3. **A3** `BuildingBlueprint::footprint()` returns `(footprint_width,
   footprint_height)` tuple.
4. **A4** `ConstructionSite::new(blueprint, position)` sets
   `progress: 0`, preserves blueprint and position verbatim.
5. **A5** `ConstructionSite::advance()` increments `progress` by 1 on
   the first call (starting from 0 with `required_progress == 5`,
   confirms `progress == 1` and return value `false`).
6. **A6** `ConstructionSite::advance()` returns `true` exactly once
   (build a site with `required_progress == 3`, call `advance` four
   times, assert returns `[false, false, true, false]`).
7. **A7** `ConstructionSite::is_complete()` matches `>=` semantics
   exactly (cover `progress == required_progress - 1` → false,
   `progress == required_progress` → true).
8. **A8** `required_progress == 0` edge case: fresh site is_complete
   true; first `advance()` returns `false`; `progress` stays at 0.
9. **A9** `TargetKind` has four variants — exhaustive match arm
   compile guarantee + an explicit equality check that
   `TargetKind::ConstructionSite != TargetKind::Sleep`.
10. **A10** `AgentState::Seeking { target: TargetKind::ConstructionSite }
    .suppresses_movement()` is `true`. `AgentState::Consuming { target:
    TargetKind::ConstructionSite }.suppresses_movement()` is `false`.
11. **A11** `AgentState::Seeking { target: TargetKind::ConstructionSite }
    .target()` returns `Some(TargetKind::ConstructionSite)`.
    `AgentState::Consuming { target: TargetKind::ConstructionSite }
    .target()` returns `Some(TargetKind::ConstructionSite)`.
12. **A12** serde RON round-trip for: `BuildingBlueprint`,
    `ConstructionSite`, `TargetKind::ConstructionSite`,
    `AgentState::Seeking { target: TargetKind::ConstructionSite }`,
    `AgentState::Consuming { target: TargetKind::ConstructionSite }`.
13. **A13 (regression)** Spawn an entity in a `hecs::World` carrying
    `Agent { id: 42 }`, `Position { x: 1, y: 1 }`, `Hunger::new(0.0, 1.0)`,
    `Thirst::new(0.0, 0.7)`, `Sleep::new(0.0, 0.5)`,
    `AgentState::default()`, `ConstructionSite::new(BuildingBlueprint::new(1,2,2,5), Position{x:1,y:1})`.
    Confirm all six components are queryable in one combined `query::<...>()`
    call. This proves the new component coexists with the entire Phase 5
    component set in a single archetype without conflict — the minimum
    regression evidence for Phase 5 substrate health post-Phase 6-α.

Assertion numbering MUST appear as a comment header at the top of the
test file (`// A1: ...`, `// A2: ...`) so the evaluator can map an
assertion failure directly to the spec entry.

---

## Section 4 — Locale

**No new locale keys required.** Phase 6-α is backend only — no new HUD
surface, no new GDScript path, no user-facing strings. Locale work is
deferred to Phase 6-δ (optional, gated on user mandate per planning
§2.4).

---

## Section 5 — Verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -20
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -20
cd rust && cargo test -p sim-test harness_p6_alpha -- --nocapture 2>&1 | tail -50
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -20
```

Expected:
- `cargo build`: clean (one new module, two new structs, one enum variant)
- `cargo test --workspace`: zero new failures. All Phase 2-5 harness tests
  still pass (T7.10.A-F + harness_phase2_substantial + harness_p3_* +
  harness_p4_* + harness_p5_alpha + harness_p5_beta + harness_p5_gamma
  all CLEAN).
- `cargo test -p sim-test harness_p6_alpha`: ≥12 assertions pass.
- `cargo clippy`: zero new warnings. The new struct derives + the new
  enum variant produce no new clippy noise.

---

## Section 6 — Lane

`--full`. Forced by: any edit under `rust/crates/sim-core/src/` triggers
`--full` per the hook tier rule in `tools/harness/install_hooks.sh`.

Planning debate, Visual Verify (no-godot-scope auto credit expected),
FFI Chain check, Regression Guard, and Evaluator all run.

---

## Section 7 — 인게임 확인사항 (in-game verification expectations)

**None.** Phase 6-α adds no runtime tick code, no rendering changes, no
HUD surface. Pipeline VLM is expected to issue **no-godot-scope auto
credit** per v3.3.7 §2 (no `.gd`, `.gdshader`, `.tscn`, `.tres`, no
`scripts/` or `scenes/` path edits).

The agent-driven construction visual milestone is reserved for Phase 6-δ
(user-mandated optional sub-stage). Phase 6-α / β / γ all run as pure
backend dispatches. The full Phase 6 routine becomes observable only
once β wires the runtime system and γ adds the chronicle harness.

---

## Self-check before dispatching the Generator

- [x] `TargetKind` is a 4-variant enum after this change — not 3, not 5.
- [x] `AgentState` enum body is **byte-for-byte unchanged** after this
      change (only its tests grow). Phase 5-γ Path (b) precedent.
- [x] `BuildingBlueprint` and `ConstructionSite` both derive
      `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`.
- [x] `BlueprintId = u64`. No alternative integer width.
- [x] `progress` is `u32`. Not `f64`. Not `f32`.
- [x] `ConstructionSite::advance()` returns `bool` (completion edge),
      not the new progress value.
- [x] No `CausalEvent` variants added. No `DecisionReason` variants
      added. No `AgentDecisionSystem` changes.
- [x] No new system in `sim-systems`. No new `priority()` slot consumed.
- [x] No `SimResources` schema change. No new tile substrate map.
- [x] No FFI change. No `sim-bridge` change.
- [x] Harness assertion file has at least 12 numbered assertions mapped
      to P6α-1 through P6α-12 + 1 regression assertion.
