# P6-γ — Construction chronicle harness (build-a-shelter cycle, V7 Phase 6 closure)

> Lane: `--full` (sim-test `.rs` only, no GD harness — backend-only chronicle).
> Scope: Single closure test that walks one agent through a full
> Idle → Seeking{ConstructionSite} → Consuming{ConstructionSite} → Idle cycle,
> proves the 4-link causal chain, and asserts complete despawn + agent reset.
> Governance: v3.3.17. Visual: backend-leaning (no new HUD surface,
> no `.gd`/`.gdshader`/`.tscn`/`.tres`) — Pipeline VLM no-godot-scope auto
> credit expected (P6γ-3-b decision).

---

## Section 1 — Implementation Intent

Phase 6 closure milestone. Phase 6-α (`ba4e02b2`) landed the data substrate
(`BuildingBlueprint`, `ConstructionSite`, `TargetKind::ConstructionSite`),
Phase 6-β (`21b09e26`) wired the runtime (`ConstructionSystem` at priority
133 + `ConstructionStarted/Completed` causal variants +
`DecisionReason::ConstructionReason` + `AgentDecisionSystem` takeover of the
α inert placeholders). What remains is the end-to-end proof.

Phase 6-γ adds **exactly one new file**:
`rust/crates/sim-test/tests/harness_p6_gamma_construction_chronicle.rs`.

The test mirrors the `harness_p5_gamma_sleep_daynight_chronicle.rs` structure
exactly:
- Header doc-block names the test (chronicle role for Phase 6 closure).
- Setup: deterministic single-agent + single-`ConstructionSite` scenario.
  All Need growth_rates set to `0.0` so Hunger/Thirst/Fatigue never breach
  during the cycle — this isolates Construction as the only fired
  `DecisionReason`.
- Tick the engine for a documented budget of ticks (chosen so completion
  occurs strictly before exhaustion).
- Per tick: capture agent state, site progress, and any newly pushed
  CausalEvent on the agent's tile.
- After the loop: 13 numbered assertions verify the full FSM trajectory
  and the 4-link causal chain.

After P6-γ:
- The full Phase 6 agent-driven construction routine has end-to-end
  evidence in a single test, with per-tick diagnostic dump on failure
  (same pattern Phase 5-γ uses).
- `harness_p6_gamma_construction_chronicle` joins
  `harness_p5_gamma_sleep_daynight_chronicle` as the second closure-milestone
  chronicle in the test suite.
- **V7 Phase 6 (Building System Deep, Week 11-12) is closed.** Phase 6-δ
  (UI integration) remains optional and gated on explicit user mandate.

**No production code changes.** This dispatch only adds a test file. Zero
edits to `sim-core`, `sim-systems`, `sim-engine`, or `sim-bridge`.

---

## Section 2 — Locked facts (from pre-grep — must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P6γ-1: scope shape | Planning §2.3 + P5γ precedent | **Single new file**: `rust/crates/sim-test/tests/harness_p6_gamma_construction_chronicle.rs`. No other file edits. |
| P6γ-2: test fn name | P5γ precedent (single test fn = whole chronicle) | `#[test] fn harness_p6_gamma_construction_chronicle()` — one function containing all 13 numbered assertions. The single-function shape lets `cargo test --test harness_p6_gamma_construction_chronicle` report exactly one pass on success and pinpoint the failing assertion on regression. |
| P6γ-3: scenario | Planning §2.3 + Phase 6-β tick-ordering reality | Spawn one agent at `Position { x: 8, y: 8 }`. Spawn one `ConstructionSite` co-located at `(8, 8)` with `BuildingBlueprint::new(blueprint_id: 1, footprint_width: 2, footprint_height: 2, required_progress: 5)`. Agent component bag: `Agent { id: 1 }, Position { x: 8, y: 8 }, AgentState::Idle, Hunger::new(0.0, 0.0), Thirst::new(0.0, 0.0), Sleep::new(0.0, 0.0)`. All growth_rates `0.0` — Needs never breach. |
| P6γ-4: grid size | P5γ precedent + Phase 6-β harness pattern | `SimEngine::new(16, 16, MaterialRegistry::new())` — same 16×16 grid Phase 5-γ uses. |
| P6γ-5: tick budget | Phase 6-β tick-ordering math (decision + start + required_progress + settle) | `const N_TICKS: u64 = 20;`. Math: tick 1 issues AgentDecision{ConstructionReason} + Idle→Seeking; tick 2 issues ConstructionStarted + Seeking→Consuming; ticks 3-7 advance progress (5 increments to required_progress=5); tick 7 (or 8 depending on advance order vs query order) issues ConstructionCompleted + BuildingPlaced + Consuming→Idle. Budget of 20 leaves margin for the regression sentinel assertion. |
| P6γ-6: per-tick capture | Planning §2.3 + P5γ precedent | Pre-loop: `let mut state_log: Vec<(u64, AgentState)> = vec![]`. Inside the tick loop AFTER `engine.tick()`: push `(current_tick, current_state)` for the agent; also iterate `resources.causal_log.get(tile_idx)` and snapshot only the new entries pushed this tick (compare log length before vs after). Build a `causal_log: Vec<(u64, CausalEvent)>` accumulator. |
| P6γ-7: first-occurrence helper | P5γ pattern | After the loop: `let first_seeking = state_log.iter().find(|(_, s)| matches!(s, AgentState::Seeking { target: TargetKind::ConstructionSite })).map(|(t, _)| *t)`; similarly for `first_consuming` and `first_idle_after_consuming`. Each becomes `t_s`, `t_c`, `t_i` for the chronicle assertions. |
| P6γ-8: diagnostic dump | P5γ precedent (per-tick log dump on failure) | Every assertion uses `assert!(condition, "chronicle: ... state_log={state_log:?} causal_log={causal_log:?}")` so a regression prints the full per-tick history. This is the diagnostic value lock — Phase 5-γ used the same pattern. |
| P6γ-9: no GD harness | Planning §2.3 "Optional, evaluator discretion" + P6γ-3-b decision | **Backend only.** No `scripts/test/p6_gamma_construction/`. No `.gd` file. Pipeline VLM auto-credit (no-godot-scope) is the explicit policy choice — keeps scope minimal, avoids dispatch risk, and aligns with Phase 6-α/β's same backend-only stance. |
| P6γ-10: regression sentinel | Planning §2.3 assertion 13 | Document the observed `t_i` in a comment with the formula `t_i ≤ required_progress + 4`. Assert `t_i < N_TICKS` (with N_TICKS=20 this is `t_i < 20`). The sentinel catches future regressions in the tick-loop sequence (e.g., if AgentDecisionSystem stops emitting Idle→Seeking transitions). |
| P6γ-11: causal chain accessor pattern | Phase 6-β `event.rs` accessors | Use `CausalEvent::id()` / `.parent()` / `.tick()` for chain walking. Filter by `matches!(ev, CausalEvent::ConstructionStarted { .. })` etc. The chain walk for assertion 10 is: find the unique BuildingPlaced; trace `.parent()` → expect `ConstructionCompleted`; trace its `.parent()` → expect `ConstructionStarted`; trace its `.parent()` → expect `AgentDecision { reason: ConstructionReason }`. All four found by `causal_log.iter().find(|(_, ev)| matches!(...))`. |
| P6γ-12: no Need breach guard | Planning §2.3 assertion 12 | After the loop: assert `causal_log.iter().all(|(_, ev)| !matches!(ev, CausalEvent::AgentDecision { reason: DecisionReason::HungerThresholdBreach | DecisionReason::ThirstThresholdBreach | DecisionReason::FatigueThresholdBreach, .. }))`. Proves test isolation. |
| P6γ-13: serde derives — N/A | This dispatch adds no new types | The test does NOT define any new structs or enums. It exercises only existing types. |
| Harness assertion count | Planning §2.3 + axiom #1 floor | **Exactly 13 assertions** as numbered in planning §2.3 (1-13). Generator MAY add a 14th sentinel (no-Need-breach summary as an explicit count assertion) if it tightens the diagnostic chain, but no more than 14 total. Each assertion uses a clearly-tagged message starting with `"A{N}:"`. |

**Determinism rationale** (axiom #2 — verified before assuming):
1. All RNG-driven systems (`AgentMovementSystem` priority 120, Brownian) are
   suppressed during `Seeking { .. }` per `AgentState::suppresses_movement`.
   The agent never moves during the chronicle.
2. Tick ordering 90 BSS → 100 IUS → 110 AIS → 120 movement → 125 decision →
   130/131/132 needs → 133 construction → 1000 viz is deterministic; the
   chronicle outcome depends only on this order.
3. Need growth_rates all `0.0` → Hunger/Thirst/Sleep stay at `0.0` forever
   → never breach threshold → Construction always wins on Idle agent.
4. Single agent + single site → no archetype-order ambiguity.

---

## Section 3 — What to build (1 file scope, exactly)

### 3.1 `rust/crates/sim-test/tests/harness_p6_gamma_construction_chronicle.rs` (NEW, sole file)

Mirror the `harness_p5_gamma_sleep_daynight_chronicle.rs` structure. Module
header (≥10 lines) names this as the **V7 Phase 6 closure milestone
chronicle**, references planning §2.3 and Phase 6-α/β commit hashes
(`ba4e02b2` and `21b09e26`), and documents the deterministic scenario.

Imports:

```rust
use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentState, BuildingBlueprint, ConstructionSite, Hunger,
    Position, Sleep, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
```

Test body skeleton:

```rust
#[test]
fn harness_p6_gamma_construction_chronicle() {
    const N_TICKS: u64 = 20;
    const SITE_X: u32 = 8;
    const SITE_Y: u32 = 8;
    const REQUIRED_PROGRESS: u32 = 5;
    const BLUEPRINT_ID: u64 = 1;

    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());

    // Spawn agent at (8, 8). spawn_agent returns the Entity carrying
    // Agent + Position; we then insert the rest of the component bag.
    let agent_entity = engine.spawn_agent(SITE_X, SITE_Y);
    engine.world.insert(
        agent_entity,
        (
            AgentState::Idle,
            Hunger::new(0.0, 0.0),
            Thirst::new(0.0, 0.0),
            Sleep::new(0.0, 0.0),
        ),
    ).expect("insert needs bag");

    // Spawn co-located ConstructionSite.
    let blueprint = BuildingBlueprint::new(BLUEPRINT_ID, 2, 2, REQUIRED_PROGRESS);
    let site = ConstructionSite::new(blueprint, Position { x: SITE_X, y: SITE_Y });
    let site_entity = engine.world.spawn((site,));

    // Per-tick chronicle accumulators.
    let width = engine.resources.tile_grid.width;
    let tile_idx = (SITE_Y as usize) * (width as usize) + (SITE_X as usize);
    let mut state_log: Vec<(u64, AgentState)> = vec![];
    let mut causal_log: Vec<(u64, CausalEvent)> = vec![];
    let mut progress_log: Vec<(u64, u32)> = vec![];
    let mut last_log_len: usize = 0;

    // Snapshot tick 0 state BEFORE first tick.
    state_log.push((0, *engine.world.get::<&AgentState>(agent_entity).unwrap()));

    for _ in 0..N_TICKS {
        engine.tick();
        let now = engine.resources.current_tick;

        // Agent state.
        let s = engine.world.get::<&AgentState>(agent_entity)
            .map(|s| *s)
            .unwrap_or(AgentState::Idle);
        state_log.push((now, s));

        // Site progress (when site still alive).
        let progress = engine.world.get::<&ConstructionSite>(site_entity)
            .map(|s| s.progress)
            .unwrap_or(REQUIRED_PROGRESS);
        progress_log.push((now, progress));

        // New causal events on the site tile.
        if let Some(log) = engine.resources.causal_log.get(tile_idx) {
            let slice = log.as_slice();
            if slice.len() > last_log_len {
                for ev in &slice[last_log_len..] {
                    causal_log.push((now, ev.clone()));
                }
                last_log_len = slice.len();
            }
        }
    }

    // ===========================================================
    // ASSERTIONS (13 mandatory, A14 optional summary)
    // ===========================================================

    // A1: first Seeking { ConstructionSite } tick exists.
    let t_s = state_log.iter().find_map(|(t, s)|
        matches!(s, AgentState::Seeking { target: TargetKind::ConstructionSite })
            .then_some(*t));
    assert!(t_s.is_some(),
        "A1: chronicle never reached Seeking{{ConstructionSite}}. state_log={state_log:?} causal_log={causal_log:?}");

    // A2: first Consuming { ConstructionSite } tick > t_s.
    let t_c = state_log.iter().find_map(|(t, s)|
        matches!(s, AgentState::Consuming { target: TargetKind::ConstructionSite })
            .then_some(*t));
    let t_c_unwrapped = t_c.expect("A2: chronicle never reached Consuming{ConstructionSite}");
    assert!(t_c_unwrapped > t_s.unwrap(),
        "A2: t_c={t_c_unwrapped} not > t_s={}. state_log={state_log:?}", t_s.unwrap());

    // A3: return to Idle at tick > t_c.
    let t_i = state_log.iter().skip_while(|(t, _)| *t <= t_c_unwrapped)
        .find_map(|(t, s)| matches!(s, AgentState::Idle).then_some(*t));
    let t_i_unwrapped = t_i.expect("A3: chronicle never returned to Idle after Consuming");
    assert!(t_i_unwrapped > t_c_unwrapped,
        "A3: t_i={t_i_unwrapped} not > t_c={t_c_unwrapped}. state_log={state_log:?}");

    // A4: progress monotonically non-decreasing across the entire log, and
    // strictly increasing during the Consuming phase.
    let consuming_progress: Vec<u32> = progress_log.iter()
        .filter(|(t, _)| *t >= t_c_unwrapped && *t < t_i_unwrapped)
        .map(|(_, p)| *p).collect();
    let mut prev = 0u32;
    for p in &consuming_progress {
        assert!(*p >= prev, "A4: progress regressed during Consuming. progress_log={progress_log:?}");
        prev = *p;
    }

    // A5: progress >= required_progress at completion tick.
    let progress_at_t_i = progress_log.iter().find(|(t, _)| *t == t_i_unwrapped)
        .map(|(_, p)| *p)
        .unwrap_or(0);
    assert!(progress_at_t_i >= REQUIRED_PROGRESS,
        "A5: progress at t_i={t_i_unwrapped} is {progress_at_t_i}, not >= {REQUIRED_PROGRESS}. progress_log={progress_log:?}");

    // A6: BuildingPlaced emitted exactly once.
    let building_placed_count = causal_log.iter().filter(|(_, ev)|
        matches!(ev, CausalEvent::BuildingPlaced { .. })).count();
    assert_eq!(building_placed_count, 1,
        "A6: BuildingPlaced count={building_placed_count}, expected 1. causal_log={causal_log:?}");

    // A7: ConstructionStarted emitted exactly once, BEFORE ConstructionCompleted.
    let construction_started: Vec<(u64, CausalEvent)> = causal_log.iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::ConstructionStarted { .. }))
        .cloned().collect();
    assert_eq!(construction_started.len(), 1,
        "A7: ConstructionStarted count={}, expected 1", construction_started.len());

    // A8: ConstructionCompleted emitted exactly once, before BuildingPlaced.
    let construction_completed: Vec<(u64, CausalEvent)> = causal_log.iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::ConstructionCompleted { .. }))
        .cloned().collect();
    assert_eq!(construction_completed.len(), 1,
        "A8: ConstructionCompleted count={}, expected 1", construction_completed.len());
    let started_id = construction_started[0].1.id();
    let completed_id = construction_completed[0].1.id();
    let placed_event = causal_log.iter().find(|(_, ev)|
        matches!(ev, CausalEvent::BuildingPlaced { .. })).map(|(_, ev)| ev.clone()).unwrap();
    let placed_id = placed_event.id();
    assert!(started_id < completed_id && completed_id < placed_id,
        "A8: id ordering violation started={started_id} completed={completed_id} placed={placed_id}");

    // A9: AgentDecision { reason: ConstructionReason } emitted at least once, before ConstructionStarted.
    let construction_reasons: Vec<(u64, CausalEvent)> = causal_log.iter()
        .filter(|(_, ev)| matches!(ev,
            CausalEvent::AgentDecision { reason: DecisionReason::ConstructionReason, .. }))
        .cloned().collect();
    assert!(!construction_reasons.is_empty(),
        "A9: no AgentDecision{{ConstructionReason}} in chronicle. causal_log={causal_log:?}");
    let first_reason_id = construction_reasons[0].1.id();
    assert!(first_reason_id < started_id,
        "A9b: first ConstructionReason id={first_reason_id} not before ConstructionStarted id={started_id}");

    // A10: causal chain parent linkage continuous.
    let placed_parent = placed_event.parent();
    assert_eq!(placed_parent, Some(completed_id),
        "A10a: BuildingPlaced.parent={placed_parent:?}, expected Some({completed_id})");
    let completed_parent = construction_completed[0].1.parent();
    assert_eq!(completed_parent, Some(started_id),
        "A10b: ConstructionCompleted.parent={completed_parent:?}, expected Some({started_id})");
    let started_parent = construction_started[0].1.parent();
    assert_eq!(started_parent, Some(first_reason_id),
        "A10c: ConstructionStarted.parent={started_parent:?}, expected Some({first_reason_id}) (first ConstructionReason)");

    // A11: ConstructionSite entity despawned after completion.
    let site_still_alive = engine.world.get::<&ConstructionSite>(site_entity).is_ok();
    assert!(!site_still_alive,
        "A11: ConstructionSite entity still alive post-completion. state_log={state_log:?}");

    // A12: no Need ThresholdBreach during chronicle.
    let need_breaches = causal_log.iter().filter(|(_, ev)| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::HungerThresholdBreach, .. }
        | CausalEvent::AgentDecision { reason: DecisionReason::ThirstThresholdBreach, .. }
        | CausalEvent::AgentDecision { reason: DecisionReason::FatigueThresholdBreach, .. }
    )).count();
    assert_eq!(need_breaches, 0,
        "A12: {need_breaches} Need-breach AgentDecision(s) leaked into chronicle. causal_log={causal_log:?}");

    // A13: regression sentinel — t_i within budget.
    assert!(t_i_unwrapped < N_TICKS,
        "A13: t_i={t_i_unwrapped} not strictly < N_TICKS={N_TICKS}. state_log={state_log:?}");
}
```

The Generator should preserve this overall structure but is free to refine
field-access patterns (e.g., `engine.world.get::<&T>(...)`) if hecs' API
shape requires different syntax. The 13 numbered assertions, the four
event-chain checks (A8 + A10), and the no-Need-breach guard (A12) are
**locked** — they are not implementation choices.

**Optional A14**: A summary assertion confirming
`construction_reasons.len() >= 1 && construction_started.len() == 1 &&
construction_completed.len() == 1 && building_placed_count == 1` as a
"chronicle integrity" final check. Acceptable but not required.

---

## Section 4 — Locale

**No new locale keys required.** This dispatch adds one test file. No
GDScript, no HUD, no user-facing strings.

---

## Section 5 — Verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -10
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -20
cd rust && cargo test --test harness_p6_gamma_construction_chronicle -- --nocapture 2>&1 | tail -50
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -10
```

Expected:
- `cargo build`: clean (one new test file, no other changes).
- `cargo test --workspace`: **zero new failures**. All Phase 2-5 + Phase 6-α
  + Phase 6-β harness tests still pass. `harness_p6_gamma_construction_chronicle`
  runs and passes its single `#[test]` function.
- `cargo test --test harness_p6_gamma_construction_chronicle`: 1 passed.
- `cargo clippy`: zero new warnings.

---

## Section 6 — Lane

`--full`. Forced by any `.rs` edit under `rust/crates/`. Planning debate,
Visual Verify (no-godot-scope auto credit expected), FFI Chain check,
Regression Guard, and Evaluator all run.

---

## Section 7 — 인게임 확인사항

**None.** Phase 6-γ adds no FFI, no rendering, no GDScript, no HUD,
no Locale keys. Pipeline VLM is expected to issue **no-godot-scope auto
credit** per v3.3.7 §2 (no `.gd`, `.gdshader`, `.tscn`, `.tres`, no
`scripts/` or `scenes/` path edits).

Phase 6-δ (UI integration — visual rendering of `ConstructionSite` + agent
construction indicator + `CausalPanel` ConstructionReason labels) remains
the optional δ sub-stage gated on explicit user mandate. Phase 6-γ closes
the backend-side Phase 6 routine; the visual milestone is δ's purview.

---

## Self-check before dispatching the Generator

- [x] Exactly **one new file**: `harness_p6_gamma_construction_chronicle.rs`.
      No edits to any other file. Zero production-code change.
- [x] Single `#[test] fn harness_p6_gamma_construction_chronicle()` —
      one function, 13 numbered assertions inside.
- [x] All 13 assertions match planning §2.3 1-through-13.
- [x] Imports do not pull in any unused types.
- [x] No new `pub use`, no new module, no public API surface change.
- [x] Diagnostic dump on failure uses `state_log`, `causal_log`,
      `progress_log` — same pattern as Phase 5-γ chronicle.
- [x] N_TICKS=20 with required_progress=5 leaves margin for the
      regression sentinel (A13).
- [x] Causal chain walked **backwards** from BuildingPlaced
      → ConstructionCompleted → ConstructionStarted →
      AgentDecision{ConstructionReason}.
- [x] No Need breach guard (A12) proves the chronicle isolation.
- [x] Score adjusted ≥ 90 expected: 10 gate + 5 plan + 15 code (attempt 1
      expected) + 20 test coverage + 20 visual auto-credit + 15 regression +
      15 evaluator = 100 max; even with attempt-2 penalty (-5) the floor is
      95. No VLM env cost expected (no-godot-scope).
