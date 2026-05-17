# P7-γ — Two-agent social interaction chronicle harness (V7 Phase 7 closure milestone)

> Lane: `--full` (sim-test `.rs` only, no GD harness — backend-only chronicle).
> Scope: Single closure test that walks two agents through a full
> Idle → Seeking{Agent(other)} → Consuming{Agent(other)} → Idle mutual cycle,
> proves the 3-link causal chain `AgentDecision{SocialReason} →
> SocialInteractionStarted → SocialInteractionCompleted`, and asserts
> familiarity bump + Social need decrement.
> Governance: v3.3.17. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/
> `.tres`, no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto
> credit expected.

---

## Section 1 — Implementation Intent

V7 Phase 7 closure milestone. Phase 7-α (`35fbd501`) landed the data
substrate; Phase 7-β (`de336f83`) wired the runtime (SocialInteractionSystem
priority 134, SocialDecaySystem priority 135, CausalEvent
SocialInteractionStarted/Completed, DecisionReason::SocialReason,
AgentDecisionSystem 5th cascade, SimResources::relationships +
interaction_progress sparse maps).

Phase 7-γ adds **exactly one new file**:
`rust/crates/sim-test/tests/harness_p7_gamma_social_chronicle.rs`.

The test mirrors `harness_p5_gamma_sleep_daynight_chronicle.rs` and
`harness_p6_gamma_construction_chronicle.rs` — first multi-agent chronicle
(two agents).

After P7-γ: V7 Phase 7 (Multi-agent Social System) closed. Phase 7-δ (UI
integration) remains optional and gated on explicit user mandate.

**No production code changes.** Test file only. Zero edits to sim-core /
sim-systems / sim-engine / sim-bridge.

---

## Section 2 — Locked facts (from plan §γ — must match implementation)

| Fact | Value |
|------|-------|
| P7γ-1: scope shape | Single new file `harness_p7_gamma_social_chronicle.rs`. No other edits. |
| P7γ-2: test fn name | `#[test] fn harness_p7_gamma_social_chronicle()` — single function, all assertions inside. |
| P7γ-3: grid size | `SimEngine::new(12, 12, MaterialRegistry::new())`. 12×12. |
| P7γ-4: agent setup | Agent 1 at (5,5), Agent 2 at (6,5). Both: Hunger/Thirst/Sleep::new(0.0,0.0), Social::new(0.0,1.0), AgentState::Idle. |
| P7γ-5: forced co-location | At tick 0 both agents placed at (6,5) via direct world-position write. |
| P7γ-6: tick budget | `N_TICKS = 80`. Math: loneliness 50.0 at tick 51 → mutual Seeking tick 51-52 → Consuming 52-53 → complete by tick 55-56 → Idle 56-57. |
| P7γ-7: per-tick capture | Per-agent state_log_X / loneliness_log_X / progress_log / relationship_log / causal_log on (6,5). |
| P7γ-8: first-occurrence helpers | t_s1, t_s2, t_c, t_i extracted via iter().find_map(). |
| P7γ-9: diagnostic dump | All assertion messages include state_log_a, state_log_b, causal_log dump. |
| P7γ-10: no GD harness | No `.gd` file. VLM no-godot-scope auto credit. |
| P7γ-11: regression sentinel | Assert t_i < N_TICKS (80). |
| P7γ-12: causal chain | Walk backwards from SocialInteractionCompleted → SocialInteractionStarted → AgentDecision{SocialReason}. |
| P7γ-13: no Need breach guard | Assert no Hunger/Thirst/Fatigue/Construction breach decisions fire. |
| Harness assertion count | Exactly 13 numbered assertions (1-13). |

**Determinism rationale**:
1. AgentMovementSystem (priority 120) suppressed during Seeking — no movement.
2. Tick ordering 90→100→110→120→125→130-132/135→133→134→1000 deterministic.
3. SocialDecaySystem priority 135 advances loneliness by 1.0 per tick.
4. AgentDecisionSystem Idle 5th cascade picks lowest AgentId — deterministic tie-break.
5. SocialInteractionSystem priority 134 processes mutual pairs in sorted canonical RelationshipKey order.

---

## Section 3 — What to build (1 file, plan-locked structure)

### `rust/crates/sim-test/tests/harness_p7_gamma_social_chronicle.rs` (NEW, sole file)

Single test function with 13 numbered assertions. Use this skeleton:

```rust
use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentState, Hunger, Position, RelationshipKey, RelationshipState,
    Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;

#[test]
fn harness_p7_gamma_social_chronicle() {
    const N_TICKS: u64 = 80;
    const SHARED_X: u32 = 6;
    const SHARED_Y: u32 = 5;

    let mut engine = SimEngine::new(12, 12, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);

    let agent_1 = engine.spawn_agent(5, 5);
    engine.world.insert(agent_1, (
        Hunger::new(0.0, 0.0),
        Thirst::new(0.0, 0.0),
        Sleep::new(0.0, 0.0),
        Social::new(0.0, 1.0),
        AgentState::Idle,
    )).expect("insert agent_1");

    let agent_2 = engine.spawn_agent(6, 5);
    engine.world.insert(agent_2, (
        Hunger::new(0.0, 0.0),
        Thirst::new(0.0, 0.0),
        Sleep::new(0.0, 0.0),
        Social::new(0.0, 1.0),
        AgentState::Idle,
    )).expect("insert agent_2");

    // Force co-location: agent_1 → (6, 5).
    {
        let mut p1 = engine.world.get::<&mut Position>(agent_1).unwrap();
        p1.x = SHARED_X; p1.y = SHARED_Y;
    }

    let id_1 = engine.world.get::<&Agent>(agent_1).unwrap().id;
    let id_2 = engine.world.get::<&Agent>(agent_2).unwrap().id;
    let rel_key = RelationshipKey::new(id_1, id_2);

    let width = engine.resources.tile_grid.width;
    let tile_idx_u32 = SHARED_Y * width + SHARED_X;
    let mut state_log_1: Vec<(u64, AgentState)> = vec![];
    let mut state_log_2: Vec<(u64, AgentState)> = vec![];
    let mut progress_log: Vec<(u64, u32)> = vec![];
    let mut familiarity_log: Vec<(u64, f64)> = vec![];
    let mut causal_log_acc: Vec<(u64, CausalEvent)> = vec![];
    let mut seen_event_ids: std::collections::BTreeSet<sim_core::causal::EventId> =
        std::collections::BTreeSet::new();

    state_log_1.push((0, *engine.world.get::<&AgentState>(agent_1).unwrap()));
    state_log_2.push((0, *engine.world.get::<&AgentState>(agent_2).unwrap()));

    for _ in 0..N_TICKS {
        engine.tick();
        let now = engine.resources.current_tick;

        state_log_1.push((now, *engine.world.get::<&AgentState>(agent_1)
            .map(|s| *s).unwrap_or(AgentState::Idle).borrow_or_clone_inline()));
        // (Generator: pattern simplification — `engine.world.get::<&AgentState>(e)`
        //  returns a hecs::Ref<&AgentState>; deref-copy is correct since AgentState
        //  is Copy. Use whatever syntax compiles.)
        state_log_2.push((now, /* same for agent_2 */ AgentState::Idle));

        progress_log.push((now,
            engine.resources.interaction_progress.get(&rel_key).copied().unwrap_or(0)));
        familiarity_log.push((now,
            engine.resources.relationships.get(&rel_key).map(|r| r.familiarity).unwrap_or(0.0)));

        // Capture by event-id (ring-rotation safe per attempt-2 evaluator feedback).
        if let Some(log) = engine.resources.causal_log.get(tile_idx_u32) {
            for ev in log.as_slice() {
                if seen_event_ids.insert(ev.id()) {
                    causal_log_acc.push((now, ev.clone()));
                }
            }
        }
    }

    // ============================================================
    // 13 NUMBERED ASSERTIONS (plan §γ locked)
    // ============================================================
    // A1: agent_1 reaches Seeking{Agent(id_2)} at some tick t_s1.
    // A2: agent_2 reaches Seeking{Agent(id_1)} at some tick t_s2. |t_s1 - t_s2| <= 1.
    // A3: both reach Consuming{Agent(*)} at t_c >= max(t_s1, t_s2).
    // A4: interaction_progress monotonic non-decreasing during Consuming.
    // A5: both return to Idle at t_i > t_c; t_i - t_c == 3 (REQUIRED_INTERACTION_PROGRESS).
    // A6: relationships[rel_key].familiarity == 0.1 after completion.
    // A7: SocialInteractionStarted emitted exactly once. (Type A)
    // A8: SocialInteractionCompleted emitted exactly once; id ordering started_id < completed_id. (Type A)
    // A9: AgentDecision{SocialReason} emitted >= 2 times (one per agent), all before Started. (Type A)
    // A10: Parent linkage: Completed.parent == Some(Started.id); Started.parent == Some(<first SocialReason id>). (Type A)
    // A11: interaction_progress[rel_key] removed or zero after completion.
    // A12: No Hunger/Thirst/Fatigue/Construction breach decisions during chronicle. (Type A)
    // A13: Regression sentinel: t_i < N_TICKS. (Type D)
}
```

**Critical implementation note** (per attempt-1/2 Evaluator feedback that
caused the score block at 88/100):

1. **Event-id based capture** (attempt-2 RE-CODE): replace any
   `slice[last_log_len..]` pattern with a `BTreeSet<EventId>` of seen ids.
   The tile causal ring rotates at `TILE_CAUSAL_RING_SIZE` (8); naive
   length-diff iteration loses events on rotation. The skeleton above
   already uses `seen_event_ids` correctly.

2. **Type annotations on assertions** (attempt-2 RE-CODE): A4, A8, A9, A10,
   A12, A13 (and A14 if added) should have inline `// Type A` or `// Type D`
   comment markers per the plan's threshold classification.

3. **A10 exact-one SocialReason per agent** (attempt-3 RE-CODE issue 5):
   Strengthen A9 → assert exactly 2 SocialReason events total
   (`social_decisions.len() == 2`), one with `agent == id_1` and one with
   `agent == id_2`.

---

## Section 4 — Locale

**No new locale keys required.** Test file only.

---

## Section 5 — Verification

```bash
cd rust && cargo build --workspace 2>&1 | tail -10
cd rust && cargo test --workspace 2>&1 | grep -E "test result|FAILED" | tail -20
cd rust && cargo test --test harness_p7_gamma_social_chronicle -- --nocapture 2>&1 | tail -50
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -10
```

Expected: workspace tests green, harness_p7_gamma 1 passed, clippy clean.

---

## Section 6 — Lane

`--full`. Sim-test `.rs` edit forces full lane.

---

## Section 7 — 인게임 확인사항

**None.** Phase 7-γ adds no FFI, no rendering, no GDScript. VLM
no-godot-scope auto credit expected.

---

## Self-check before dispatching the Generator

- [x] Exactly **one new file**: `harness_p7_gamma_social_chronicle.rs`.
- [x] Single `#[test] fn harness_p7_gamma_social_chronicle()`.
- [x] All 13 numbered assertions match plan §γ 1-through-13.
- [x] Event capture uses `BTreeSet<EventId>` (ring-rotation safe).
- [x] Each assertion message includes `state_log_1`, `state_log_2`,
      `causal_log` diagnostic dump.
- [x] No new `pub use` / module / public API change.
- [x] N_TICKS=80, Social::new(0.0, 1.0), REQUIRED_INTERACTION_PROGRESS=3,
      FAMILIARITY_BUMP=0.1, t_i - t_c == 3.
- [x] Type A/D annotation comments inline on each assertion (attempt-2
      Eval feedback).
- [x] A9: exact-2 SocialReason count (one per agent, attempt-3 Eval).
- [x] No env cost expected. adjusted_score ≥ 90 target.
