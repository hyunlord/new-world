//! V7 Phase 7-γ — Two-agent social interaction chronicle harness
//! (V7 Phase 7 closure milestone).
//!
//! Walks two co-located agents through a full mutual
//!     Idle → Seeking{Agent(other)} → Consuming{Agent(other)} → Idle
//! cycle and proves the 3-link causal chain
//!     AgentDecision{SocialReason}
//!       → SocialInteractionStarted
//!       → SocialInteractionCompleted
//! plus familiarity bump (0.0 → 0.1), Social-need decrement on completion,
//! chronicle isolation (no non-Social breach decisions fire), and clean
//! cleanup of the per-pair `interaction_progress` map.
//!
//! plan: p7-gamma-social-chronicle (plan_attempt 2, seed 42, agent_count 2)
//! assertions: A0..A16 (plan §γ locked — A16 is a diff-scope meta
//!             constraint, naturally satisfied by this single-file dispatch;
//!             documented inline below, not asserted at runtime).
//! lane: --full (backend-only — no GD harness, no FFI surface change).
//!
//! Locked constants (mirroring sim-systems::runtime::decision):
//!   - N_TICKS                       = 80
//!   - SOCIAL_THRESHOLD              = 50.0
//!   - REQUIRED_INTERACTION_PROGRESS = 3
//!   - FAMILIARITY_BUMP              = 0.1
//!
//! Deterministic scenario (priority order
//!   90 BSS → 100 IUS → 110 AIS → 120 movement → 125 decision →
//!   130/131/132 needs → 133 construction → 134 social → 135 social_decay
//!   → 1000 viz):
//!   - Both agents start at AgentState::Idle, Social::new(0.0, 1.0); all
//!     other needs are pinned to growth_rate 0.0 so Social wins the
//!     cascade.
//!   - SocialDecaySystem advances loneliness by 1.0/tick.
//!   - AgentMovementSystem (priority 120) is suppressed during
//!     Seeking{Agent(_)} via AgentState::suppresses_movement, so neither
//!     agent moves once breach occurs.
//!   - Agents are forcibly co-located at (6, 5) before tick 1 runs, so the
//!     mutual handshake is always available.
//!
//! References:
//!   - Phase 7-α commit 35fbd501 (Social, RelationshipKey, RelationshipState,
//!     TargetKind::Agent — data substrate).
//!   - Phase 7-β commit de336f83 (SocialInteractionSystem at priority 134,
//!     SocialDecaySystem at 135, CausalEvent::SocialInteractionStarted /
//!     Completed, DecisionReason::SocialReason, AgentDecisionSystem 5th
//!     cascade, SimResources::relationships + interaction_progress sparse
//!     maps).
//!
//! Note on A16 ("no new locale keys / FFI surface introduced"): this
//! dispatch adds a single test file under `rust/crates/sim-test/tests/`.
//! Locale, FFI, scripts, and scenes are out of dispatch scope by
//! construction. A runtime assertion would require shelling out to
//! `git diff`, which is brittle and intentionally avoided in P5γ/P6γ
//! chronicles for the same reason.
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p7_gamma_social_chronicle -- --nocapture`

use std::collections::BTreeSet;

use sim_core::causal::{CausalEvent, DecisionReason, EventId};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Position, RelationshipKey, Sleep, Social, TargetKind,
    Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;

#[test]
fn harness_p7_gamma_social_chronicle() {
    // ── Locked constants (plan §γ — mirror sim-systems::runtime::decision)
    const N_TICKS: u64 = 80;
    const REQUIRED_INTERACTION_PROGRESS: u32 = 3;
    const SOCIAL_THRESHOLD: f64 = 50.0;
    const FAMILIARITY_BUMP: f64 = 0.1;
    const SHARED_X: u32 = 6;
    const SHARED_Y: u32 = 5;

    // ── Engine setup (12×12 grid — sufficient for a 2-agent scenario) ─
    let mut engine = SimEngine::new(12, 12, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);

    // Spawn agent_1 at (5, 5), then force-relocate to (6, 5) below.
    let entity_1 = engine.spawn_agent(5, 5);
    engine
        .world
        .insert(
            entity_1,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 1.0),
                AgentState::Idle,
            ),
        )
        .expect("insert needs/state bag on agent_1");

    // Spawn agent_2 already at the shared tile (6, 5).
    let entity_2 = engine.spawn_agent(SHARED_X, SHARED_Y);
    engine
        .world
        .insert(
            entity_2,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 1.0),
                AgentState::Idle,
            ),
        )
        .expect("insert needs/state bag on agent_2");

    // Force co-location BEFORE tick 1: move agent_1 onto agent_2's tile.
    // The tick-0 snapshot already shows both at (6, 5).
    {
        let mut p1 = engine
            .world
            .get::<&mut Position>(entity_1)
            .expect("agent_1 has Position");
        p1.x = SHARED_X;
        p1.y = SHARED_Y;
    }

    // Capture canonical AgentIds and the (smaller, larger) RelationshipKey.
    let id_1 = engine
        .world
        .get::<&Agent>(entity_1)
        .expect("agent_1 has Agent component")
        .id;
    let id_2 = engine
        .world
        .get::<&Agent>(entity_2)
        .expect("agent_2 has Agent component")
        .id;
    let rel_key = RelationshipKey::new(id_1, id_2);

    // ── A0: clean precondition state at tick 0, before any tick runs ──
    // Type A. Anchors A5, A7, A11. Without this, FAMILIARITY_BUMP == 0.1
    // claims have no baseline (a pipeline that silently default-inserts
    // a 0.1 familiarity entry would pass A7 trivially). Plan §γ A0
    // requires four sub-conditions: (a) relationships entry None,
    // (b) both Social.loneliness == 0.0, (c) both Position == (6, 5)
    // after spawn + relocation, (d) both AgentState == Idle.
    {
        let rel_at_zero = engine.resources.relationships.get(&rel_key);
        assert!(
            rel_at_zero.is_none(),
            "A0(a): relationships entry must be None at tick 0; got {rel_at_zero:?}"
        );
        let prog_at_zero = engine.resources.interaction_progress.get(&rel_key);
        assert!(
            prog_at_zero.is_none(),
            "A0(a): interaction_progress entry must be None at tick 0; got {prog_at_zero:?}"
        );
        let lone_1 = engine
            .world
            .get::<&Social>(entity_1)
            .expect("agent_1 has Social")
            .loneliness;
        let lone_2 = engine
            .world
            .get::<&Social>(entity_2)
            .expect("agent_2 has Social")
            .loneliness;
        assert_eq!(lone_1, 0.0, "A0(b): agent_1 loneliness must be 0.0 at tick 0");
        assert_eq!(lone_2, 0.0, "A0(b): agent_2 loneliness must be 0.0 at tick 0");
        let pos_1 = {
            let p = engine
                .world
                .get::<&Position>(entity_1)
                .expect("agent_1 has Position");
            (p.x, p.y)
        };
        let pos_2 = {
            let p = engine
                .world
                .get::<&Position>(entity_2)
                .expect("agent_2 has Position");
            (p.x, p.y)
        };
        assert_eq!(
            pos_1,
            (SHARED_X, SHARED_Y),
            "A0(c): agent_1 Position must be ({SHARED_X}, {SHARED_Y}) after relocation"
        );
        assert_eq!(
            pos_2,
            (SHARED_X, SHARED_Y),
            "A0(c): agent_2 Position must be ({SHARED_X}, {SHARED_Y}) (spawn tile)"
        );
        let st_1 = *engine
            .world
            .get::<&AgentState>(entity_1)
            .expect("agent_1 has AgentState");
        let st_2 = *engine
            .world
            .get::<&AgentState>(entity_2)
            .expect("agent_2 has AgentState");
        assert_eq!(
            st_1,
            AgentState::Idle,
            "A0(d): agent_1 state must be Idle at tick 0"
        );
        assert_eq!(
            st_2,
            AgentState::Idle,
            "A0(d): agent_2 state must be Idle at tick 0"
        );
    }

    // ── Per-tick chronicle accumulators ───────────────────────────────
    let width = engine.resources.tile_grid.width;
    let tile_idx: u32 = SHARED_Y * width + SHARED_X;
    let mut state_log_1: Vec<(u64, AgentState)> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut state_log_2: Vec<(u64, AgentState)> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut loneliness_log_1: Vec<(u64, f64)> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut loneliness_log_2: Vec<(u64, f64)> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut progress_log: Vec<(u64, Option<u32>)> = Vec::with_capacity(N_TICKS as usize + 1);
    // Per-tick Position capture for both agents (plan §γ A14 — position
    // invariance: the observed (x, y) set across all ticks must equal
    // exactly {(6, 5)}). The set form catches the failure mode where
    // AgentMovementSystem displaces an agent for even a single tick.
    let mut position_log_1: Vec<(u64, (u32, u32))> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut position_log_2: Vec<(u64, (u32, u32))> = Vec::with_capacity(N_TICKS as usize + 1);
    // Ring-rotation-safe causal capture (plan §γ scenario_preconditions):
    // The per-tile causal ring buffer caps at TILE_CAUSAL_RING_SIZE=8;
    // a length-delta approach (`slice[last_len..]`) silently DROPS events
    // once the ring rotates (push evicts front, length stays at 8). We track
    // already-recorded EventIds in a BTreeSet and append any unseen id we
    // see at any point in the ring. Provided the per-tick burst on this
    // tile never exceeds 8 events (true for this 2-agent isolated chronicle
    // — A8/A9/A10 cap the social events at 1+1+2=4), every emitted event is
    // captured at least once before eviction.
    let mut causal_log_acc: Vec<(u64, CausalEvent)> = Vec::new();
    let mut seen_event_ids: BTreeSet<EventId> = BTreeSet::new();

    // Tick loop. `now` is taken from `engine.resources.current_tick` AFTER
    // `engine.tick()` returns — this matches the value systems used at
    // emission time, which is exactly the value stamped into `CausalEvent::
    // *.tick`. The engine assigns `resources.current_tick` at the START of
    // each tick and increments its own counter at the END, so the post-tick
    // read returns the tick value that was JUST executed (0 after the
    // first call, 1 after the second, …). This frame alignment lets A8 /
    // A10 compare `event.tick` against `t_c` / `t_s` directly without
    // off-by-one drift.
    //
    // Per scenario_preconditions: capture events AS THEY ARE EMITTED across
    // every tick so the 8-slot causal ring buffer cannot evict early
    // SocialReason / Started records before the post-run assertions land.
    for _ in 0..N_TICKS {
        engine.tick();
        let now: u64 = engine.resources.current_tick;

        let s1 = engine
            .world
            .get::<&AgentState>(entity_1)
            .map(|s| *s)
            .unwrap_or(AgentState::Idle);
        let s2 = engine
            .world
            .get::<&AgentState>(entity_2)
            .map(|s| *s)
            .unwrap_or(AgentState::Idle);
        state_log_1.push((now, s1));
        state_log_2.push((now, s2));

        let l1 = engine
            .world
            .get::<&Social>(entity_1)
            .map(|s| s.loneliness)
            .unwrap_or(0.0);
        let l2 = engine
            .world
            .get::<&Social>(entity_2)
            .map(|s| s.loneliness)
            .unwrap_or(0.0);
        loneliness_log_1.push((now, l1));
        loneliness_log_2.push((now, l2));

        let prog = engine.resources.interaction_progress.get(&rel_key).copied();
        progress_log.push((now, prog));

        let pos_1 = engine
            .world
            .get::<&Position>(entity_1)
            .map(|p| (p.x, p.y))
            .unwrap_or((u32::MAX, u32::MAX));
        let pos_2 = engine
            .world
            .get::<&Position>(entity_2)
            .map(|p| (p.x, p.y))
            .unwrap_or((u32::MAX, u32::MAX));
        position_log_1.push((now, pos_1));
        position_log_2.push((now, pos_2));

        if let Some(log) = engine.resources.causal_log.get(tile_idx) {
            for ev in log.as_slice() {
                let eid = ev.id();
                if seen_event_ids.insert(eid) {
                    causal_log_acc.push((now, ev.clone()));
                }
            }
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // ASSERTIONS (plan §γ A0..A15; A16 documented above, not runtime)
    // ──────────────────────────────────────────────────────────────────

    // ── A1: agent_1 enters Seeking{Agent(id_2)} on tick t_s1, t_s1 <= 60.
    // Type A. Plan §γ locked: physical invariant of the AgentDecision
    // cascade — Social::new(0.0, 1.0) + SOCIAL_THRESHOLD=50.0 forces breach
    // by tick ~50-51 with at most one cascade tick of latency. Upper bound
    // of 60 absorbs single-tick boundary effects without permitting drift;
    // no lower bound (existence + upper bound are sufficient).
    let t_s1 = state_log_1
        .iter()
        .find_map(|(t, s)| match s {
            AgentState::Seeking { target: TargetKind::Agent(o) } if *o == id_2 => Some(*t),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "A1: agent_1 never reached Seeking{{Agent({id_2})}} over {N_TICKS} ticks. \
                 state_log_1={state_log_1:?} causal_log={causal_log_acc:?}"
            )
        });
    assert!(
        t_s1 <= 60,
        "A1: t_s1={t_s1} must be <= 60. \
         state_log_1={state_log_1:?} loneliness_log_1={loneliness_log_1:?} causal_log={causal_log_acc:?}"
    );

    // ── A2: agent_2 enters Seeking{Agent(id_1)} on EXACTLY the same tick ─
    // Type A. Fully symmetric setup ⇒ both cross threshold on the same
    // tick; any 1+ tick gap is a symmetry-breaking bug (e.g. query-order-
    // dependent decay or partner-tie-break drifting Seeking observation).
    let t_s2 = state_log_2
        .iter()
        .find_map(|(t, s)| match s {
            AgentState::Seeking { target: TargetKind::Agent(o) } if *o == id_1 => Some(*t),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "A2: agent_2 never reached Seeking{{Agent({id_1})}} over {N_TICKS} ticks. \
                 state_log_2={state_log_2:?} causal_log={causal_log_acc:?}"
            )
        });
    assert_eq!(
        t_s2, t_s1,
        "A2: t_s2={t_s2} must equal t_s1={t_s1} EXACTLY (gap=0). \
         state_log_1={state_log_1:?} state_log_2={state_log_2:?}"
    );

    // ── A3: neither agent ever Seeks the wrong target ─────────────────
    // Type A. Partner-targeting invariant — only the OTHER agent is a
    // valid partner in this 2-agent universe; self-targeting or any
    // other id signals a partner-selection bug.
    for (t, s) in &state_log_1 {
        if let AgentState::Seeking { target: TargetKind::Agent(x) } = s {
            assert_eq!(
                *x, id_2,
                "A3: agent_1 Seeking wrong target at tick {t}: {x} != id_2={id_2}. \
                 state_log_1={state_log_1:?}"
            );
        }
    }
    for (t, s) in &state_log_2 {
        if let AgentState::Seeking { target: TargetKind::Agent(x) } = s {
            assert_eq!(
                *x, id_1,
                "A3: agent_2 Seeking wrong target at tick {t}: {x} != id_1={id_1}. \
                 state_log_2={state_log_2:?}"
            );
        }
    }

    // ── A4: both reach Consuming{Agent(partner)} no earlier than the ──
    // latest Seeking entry, within 1 tick of each other, with the right
    // partner target. Type A. Three invariants in one:
    //   (a) causal ordering: cannot Consume before Seeking;
    //   (b) symmetry-closeness for partner-detection;
    //   (c) target identity: each Consuming target.id == partner.id.
    let (t_c1, t_c1_target) = state_log_1
        .iter()
        .find_map(|(t, s)| match s {
            AgentState::Consuming { target: TargetKind::Agent(o) } => Some((*t, *o)),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "A4: agent_1 never reached Consuming{{Agent(_)}}. \
                 state_log_1={state_log_1:?} causal_log={causal_log_acc:?}"
            )
        });
    let (t_c2, t_c2_target) = state_log_2
        .iter()
        .find_map(|(t, s)| match s {
            AgentState::Consuming { target: TargetKind::Agent(o) } => Some((*t, *o)),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "A4: agent_2 never reached Consuming{{Agent(_)}}. \
                 state_log_2={state_log_2:?} causal_log={causal_log_acc:?}"
            )
        });
    let t_c = t_c1.max(t_c2);
    assert!(
        t_c >= t_s1.max(t_s2),
        "A4: t_c={t_c} must be >= max(t_s1, t_s2) = {} (causal ordering). \
         state_log_1={state_log_1:?} state_log_2={state_log_2:?}",
        t_s1.max(t_s2)
    );
    // Plan §γ A4 — locked per-agent upper bound: Seeking→Consuming handoff
    // for co-located agents must complete within 2 ticks of mutual breach.
    assert!(
        t_c1 >= t_s1 && t_c1 - t_s1 <= 2,
        "A4: agent_1 t_c1 - t_s1 = {} (t_c1={t_c1}, t_s1={t_s1}) outside [0, 2]. \
         state_log_1={state_log_1:?}",
        t_c1.saturating_sub(t_s1)
    );
    assert!(
        t_c2 >= t_s2 && t_c2 - t_s2 <= 2,
        "A4: agent_2 t_c2 - t_s2 = {} (t_c2={t_c2}, t_s2={t_s2}) outside [0, 2]. \
         state_log_2={state_log_2:?}",
        t_c2.saturating_sub(t_s2)
    );
    assert!(
        t_c1.abs_diff(t_c2) <= 1,
        "A4: |t_c1 - t_c2| must be <= 1 (got t_c1={t_c1}, t_c2={t_c2}). \
         state_log_1={state_log_1:?} state_log_2={state_log_2:?}"
    );
    assert_eq!(
        t_c1_target, id_2,
        "A4: agent_1 Consuming target must be id_2={id_2}; got {t_c1_target}"
    );
    assert_eq!(
        t_c2_target, id_1,
        "A4: agent_2 Consuming target must be id_1={id_1}; got {t_c2_target}"
    );

    // ── A5: interaction_progress is monotonically non-decreasing during
    //        the Consuming window `[t_c, t_c + 6)` AND reaches exactly
    //        REQUIRED_INTERACTION_PROGRESS at some point AND had a
    //        nonzero value at some sampled tick during the full run
    //        (anti-bypass anchor — plan §γ A4 / A16). Type A.
    //
    //        Monotonicity rules out spurious decrements; max == REQUIRED
    //        defeats the "write-once zero" gaming vector; had_nonzero
    //        proves SocialInteractionSystem actually executed (the only
    //        code path that writes nonzero progress).
    let mut progress_seq: Vec<u32> = Vec::new();
    {
        let window_end = t_c + 6;
        let mut prev_some: Option<u32> = None;
        for offset in 0..(window_end - t_c) {
            let t = t_c + offset;
            let entry = progress_log
                .iter()
                .find(|(tt, _)| *tt == t)
                .unwrap_or_else(|| {
                    panic!(
                        "A5: progress_log missing entry for tick {t}. \
                         progress_log={progress_log:?}"
                    )
                });
            if let Some(curr) = entry.1 {
                progress_seq.push(curr);
                if let Some(prev) = prev_some {
                    assert!(
                        curr >= prev,
                        "A5: progress regressed at tick {t} (prev={prev}, curr={curr}). \
                         progress_log={progress_log:?}"
                    );
                }
                prev_some = Some(curr);
            } else {
                // Absent entry after cleanup — treated as 0 per plan §γ A4.
                // Don't break monotonicity (a None after a Some is the
                // documented post-completion cleanup, not a regression).
            }
        }
    }
    let max_progress = progress_seq.iter().copied().max().unwrap_or_else(|| {
        panic!(
            "A5: progress_seq empty across window [t_c={t_c}, t_c+6={}). \
             progress_log={progress_log:?}",
            t_c + 6
        )
    });
    assert_eq!(
        max_progress, REQUIRED_INTERACTION_PROGRESS,
        "A5: max(progress_seq) = {max_progress}, expected exactly \
         REQUIRED_INTERACTION_PROGRESS = {REQUIRED_INTERACTION_PROGRESS}. \
         progress_seq={progress_seq:?} progress_log={progress_log:?}"
    );
    // had_nonzero: any sampled progress value > 0 across the FULL run
    // (plan §γ A16 anti-bypass anchor). A Generator that never invokes
    // SocialInteractionSystem cannot satisfy this.
    let had_nonzero = progress_log
        .iter()
        .any(|(_, p)| matches!(p, Some(v) if *v > 0));
    assert!(
        had_nonzero,
        "A5/A16: interaction_progress never observed nonzero across \
         {N_TICKS}-tick run — SocialInteractionSystem did not execute. \
         progress_log={progress_log:?}"
    );

    // ── A6: both agents return to Idle within [REQUIRED, REQUIRED+2] ──
    // ticks of their Consuming start. Type C — lower bound = required
    // increments; upper bound absorbs intra-tick observation phase /
    // completion→Idle handoff slack.
    let t_i1 = state_log_1
        .iter()
        .skip_while(|(t, _)| *t <= t_c1)
        .find_map(|(t, s)| matches!(s, AgentState::Idle).then_some(*t))
        .unwrap_or_else(|| {
            panic!(
                "A6: agent_1 never returned to Idle after Consuming start. \
                 state_log_1={state_log_1:?}"
            )
        });
    let t_i2 = state_log_2
        .iter()
        .skip_while(|(t, _)| *t <= t_c2)
        .find_map(|(t, s)| matches!(s, AgentState::Idle).then_some(*t))
        .unwrap_or_else(|| {
            panic!(
                "A6: agent_2 never returned to Idle after Consuming start. \
                 state_log_2={state_log_2:?}"
            )
        });
    let delta_1 = t_i1 - t_c1;
    let delta_2 = t_i2 - t_c2;
    // Type A — plan §γ A5 locked: REQUIRED_INTERACTION_PROGRESS == 3, one
    // increment per tick, transition-on-reach contract ⇒ both deltas must
    // be exactly 3. Any non-3 value = constant drift, increment timing
    // change, or completion-trigger bug.
    assert_eq!(
        delta_1, REQUIRED_INTERACTION_PROGRESS as u64,
        "A6: agent_1 delta t_i1 - t_c1 = {delta_1}, expected exactly \
         REQUIRED_INTERACTION_PROGRESS = {REQUIRED_INTERACTION_PROGRESS}. \
         state_log_1={state_log_1:?}"
    );
    assert_eq!(
        delta_2, REQUIRED_INTERACTION_PROGRESS as u64,
        "A6: agent_2 delta t_i2 - t_c2 = {delta_2}, expected exactly \
         REQUIRED_INTERACTION_PROGRESS = {REQUIRED_INTERACTION_PROGRESS}. \
         state_log_2={state_log_2:?}"
    );
    let t_i = t_i1.max(t_i2);

    // ── A7: familiarity transitioned None → ~FAMILIARITY_BUMP after one
    //        completed interaction. Type C. f64-safe equality |Δ| < 1e-9
    //        (0.1 has no finite binary representation). Anchored by A0
    //        (entry was None at tick 0), this proves a real transition
    //        occurred — not a hardcoded default.
    let final_fam = engine
        .resources
        .relationships
        .get(&rel_key)
        .map(|r| r.familiarity)
        .unwrap_or_else(|| {
            panic!(
                "A7: relationships entry must exist after one completion; got None. \
                 relationships={:?}",
                engine.resources.relationships
            )
        });
    assert!(
        (final_fam - FAMILIARITY_BUMP).abs() < 1e-9,
        "A7: familiarity = {final_fam}, expected ≈ {FAMILIARITY_BUMP} (|Δ| < 1e-9). \
         relationships={:?}",
        engine.resources.relationships
    );

    // ── A8: SocialInteractionStarted emitted exactly once on shared tile,
    //        with partner ids matching `{id_1, id_2}` as a set, emission
    //        tick in `[t_c, t_c + 1]`, and position == (6, 5).
    //        Type A. Global count guards a Generator that emits to a wrong
    //        tile slot. Partner-id check guards a stub {0, 0} payload.
    //        Tick-window check pins emission to the Consuming transition.
    let started: Vec<&(u64, CausalEvent)> = causal_log_acc
        .iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::SocialInteractionStarted { .. }))
        .collect();
    assert_eq!(
        started.len(),
        1,
        "A8: SocialInteractionStarted count = {}, expected 1. causal_log={causal_log_acc:?}",
        started.len()
    );
    let started_id = started[0].1.id();
    match &started[0].1 {
        CausalEvent::SocialInteractionStarted {
            agents,
            position,
            tick,
            ..
        } => {
            let got: BTreeSet<AgentId> = [agents.0, agents.1].into_iter().collect();
            let want: BTreeSet<AgentId> = [id_1, id_2].into_iter().collect();
            assert_eq!(
                got, want,
                "A8: Started.agents set = {got:?}, expected {want:?}. \
                 causal_log={causal_log_acc:?}"
            );
            assert!(
                *tick >= t_c && *tick <= t_c + 1,
                "A8: Started.tick = {tick} outside [t_c={t_c}, t_c+1={}]. \
                 causal_log={causal_log_acc:?}",
                t_c + 1
            );
            assert_eq!(
                *position,
                (SHARED_X, SHARED_Y),
                "A8: Started.position = {position:?}, expected ({SHARED_X}, {SHARED_Y}). \
                 causal_log={causal_log_acc:?}"
            );
        }
        other => panic!("A8: started[0] must be SocialInteractionStarted; got {other:?}"),
    }

    // ── A9: SocialInteractionCompleted emitted exactly once, with id ──
    //       strictly greater than Started.id (event-id monotonicity).
    //       Type A. Causal id monotonicity invariant + Started/Completed
    //       pairing symmetry.
    let completed: Vec<&(u64, CausalEvent)> = causal_log_acc
        .iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::SocialInteractionCompleted { .. }))
        .collect();
    assert_eq!(
        completed.len(),
        1,
        "A9: SocialInteractionCompleted count = {}, expected 1. causal_log={causal_log_acc:?}",
        completed.len()
    );
    let completed_id = completed[0].1.id();
    assert!(
        started_id < completed_id,
        "A9: id ordering violation — started_id={started_id} not < completed_id={completed_id}. \
         causal_log={causal_log_acc:?}"
    );

    // ── A10: AgentDecision{SocialReason} emitted exactly twice — once per
    //        agent — each before SocialInteractionStarted.id; each event's
    //        tick equals that agent's `t_s`; and at the emission tick the
    //        agent's Social.loneliness is `>= SOCIAL_THRESHOLD`. Type A.
    //        Per-agent attribution + tick alignment + loneliness check
    //        forces real Social-need-driven cascade, not unconditional or
    //        re-emitted stubs.
    let social_decisions: Vec<&(u64, CausalEvent)> = causal_log_acc
        .iter()
        .filter(|(_, ev)| {
            matches!(
                ev,
                CausalEvent::AgentDecision {
                    reason: DecisionReason::SocialReason,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(
        social_decisions.len(),
        2,
        "A10: AgentDecision{{SocialReason}} count = {}, expected exactly 2 \
         (one per agent's Idle→Seeking transition). causal_log={causal_log_acc:?}",
        social_decisions.len()
    );
    let mut sr_agent_set: BTreeSet<AgentId> = BTreeSet::new();
    for (_, dec) in &social_decisions {
        assert!(
            dec.id() < started_id,
            "A10: SocialReason id {} not before SocialInteractionStarted id {}. \
             causal_log={causal_log_acc:?}",
            dec.id(),
            started_id
        );
        match dec {
            CausalEvent::AgentDecision {
                agent,
                tick,
                reason: DecisionReason::SocialReason,
                ..
            } => {
                assert!(
                    sr_agent_set.insert(*agent),
                    "A10: duplicate SocialReason for agent {agent} — expected exactly \
                     one per agent. causal_log={causal_log_acc:?}"
                );
                let (expected_ts, lone_log) = if *agent == id_1 {
                    (t_s1, &loneliness_log_1)
                } else if *agent == id_2 {
                    (t_s2, &loneliness_log_2)
                } else {
                    panic!(
                        "A10: SocialReason agent_id={agent} matches neither id_1={id_1} \
                         nor id_2={id_2}. causal_log={causal_log_acc:?}"
                    );
                };
                assert_eq!(
                    *tick, expected_ts,
                    "A10: SocialReason for agent {agent} fired at tick {tick}, \
                     expected t_s={expected_ts}. causal_log={causal_log_acc:?}"
                );
                let lone_at_tick = lone_log
                    .iter()
                    .find(|(t, _)| *t == *tick)
                    .map(|(_, l)| *l)
                    .unwrap_or_else(|| {
                        panic!(
                            "A10: loneliness_log missing entry for agent {agent} at tick {tick}. \
                             loneliness_log={lone_log:?}"
                        )
                    });
                assert!(
                    lone_at_tick >= SOCIAL_THRESHOLD,
                    "A10: agent {agent} loneliness at tick {tick} = {lone_at_tick}, \
                     must be >= SOCIAL_THRESHOLD={SOCIAL_THRESHOLD} for SocialReason emission. \
                     loneliness_log={lone_log:?}"
                );
            }
            other => panic!("A10: filtered element must be AgentDecision; got {other:?}"),
        }
    }
    let expected_agents: BTreeSet<AgentId> = [id_1, id_2].into_iter().collect();
    assert_eq!(
        sr_agent_set, expected_agents,
        "A10: SocialReason agent set {sr_agent_set:?} != {{id_1, id_2}} = {expected_agents:?}. \
         causal_log={causal_log_acc:?}"
    );

    // ── A11: full causal-chain parent linkage
    //           Completed.parent == Some(Started.id)
    //           Started.parent   == Some(first SocialReason.id)
    //        where `first SocialReason` is selected by sorting SocialReason
    //        events by (.id() ascending, agent_id ascending) per plan §γ.
    //        Type A. The 3-link chain is the explicit Phase 7-γ contract;
    //        any broken parent = causal infrastructure regression that
    //        breaks the "왜?" UI traceback path.
    let completed_parent = completed[0].1.parent();
    assert_eq!(
        completed_parent,
        Some(started_id),
        "A11a: Completed.parent = {completed_parent:?}, expected Some({started_id}). \
         causal_log={causal_log_acc:?}"
    );
    let mut sr_sorted: Vec<(EventId, AgentId, Option<EventId>)> = social_decisions
        .iter()
        .map(|(_, ev)| match ev {
            CausalEvent::AgentDecision { id, agent, .. } => (*id, *agent, ev.parent()),
            other => panic!("A11: filtered element must be AgentDecision; got {other:?}"),
        })
        .collect();
    sr_sorted.sort_by_key(|(id, _, _)| *id);
    let first_sr_id = sr_sorted[0].0;
    let started_parent = started[0].1.parent();
    assert_eq!(
        started_parent,
        Some(first_sr_id),
        "A11b: Started.parent = {started_parent:?}, expected Some({first_sr_id}) \
         (lowest-id SocialReason). sr_sorted={sr_sorted:?} \
         causal_log={causal_log_acc:?}"
    );
    // Plan §γ A10 (plan_attempt 3) — second (higher-id) SocialReason must
    // precede Started.id, MUST be causally rooted (positive clause:
    // `parent().is_some()`), AND must NOT be a descendant of Completed
    // (negative clause: it is a co-cause, not a post-event). The positive
    // clause defends against a Generator that emits the second agent's
    // SocialReason orphaned/unparented; the negative clause defends against
    // post-hoc reparenting onto Completed.
    let sr_high = sr_sorted[1];
    assert!(
        sr_high.0 < started_id,
        "A11c: sr_high.id = {} not strictly < Started.id = {started_id}. \
         sr_sorted={sr_sorted:?}",
        sr_high.0
    );
    assert!(
        sr_high.2.is_some(),
        "A11c (plan_attempt 3 positive clause): sr_high.parent must be \
         Some(_) — the second agent's SocialReason decision must be \
         causally rooted, not orphaned. sr_high={sr_high:?} \
         sr_sorted={sr_sorted:?} causal_log={causal_log_acc:?}"
    );
    assert!(
        sr_high.2 != Some(completed_id),
        "A11c: sr_high.parent = {:?} must NOT be Some(completed.id={completed_id}) \
         (second SocialReason is a co-cause, not a descendant of Completed). \
         sr_sorted={sr_sorted:?}",
        sr_high.2
    );

    // ── A12: interaction_progress entry removed or zero after completion,
    //        verified at BOTH tick `t_i + 1` (from progress_log) AND at the
    //        end of the loop (live world read). Type A. Stale nonzero
    //        residue would corrupt the next cycle; sampling both points
    //        guards immediate-cleanup AND deferred-cleanup implementations.
    let prog_after = progress_log
        .iter()
        .find(|(t, _)| *t == t_i + 1)
        .map(|(_, p)| *p)
        .unwrap_or_else(|| {
            panic!(
                "A12: progress_log missing entry for tick {} (t_i+1). progress_log={progress_log:?}",
                t_i + 1
            )
        });
    assert!(
        prog_after.is_none() || prog_after == Some(0),
        "A12: interaction_progress at tick {} = {prog_after:?}, must be None or Some(0)",
        t_i + 1
    );
    let final_prog = engine.resources.interaction_progress.get(&rel_key).copied();
    assert!(
        final_prog.is_none() || final_prog == Some(0),
        "A12: interaction_progress at loop end = {final_prog:?}, must be None or Some(0)"
    );

    // ── A13: loneliness reset below SOCIAL_THRESHOLD at tick t_i + 1 ──
    //        (one tick after both agents are back in Idle). Type A. Both
    //        agents in [0, SOCIAL_THRESHOLD); negative leak rejected.
    //        Without this reset, both agents would immediately re-breach
    //        next decision tick — the "interaction" would have no effect
    //        on the need that motivated it.
    let lone_1_after = loneliness_log_1
        .iter()
        .find(|(t, _)| *t == t_i + 1)
        .map(|(_, l)| *l)
        .unwrap_or_else(|| {
            panic!(
                "A13: loneliness_log_1 missing entry for tick {}. loneliness_log_1={loneliness_log_1:?}",
                t_i + 1
            )
        });
    let lone_2_after = loneliness_log_2
        .iter()
        .find(|(t, _)| *t == t_i + 1)
        .map(|(_, l)| *l)
        .unwrap_or_else(|| {
            panic!(
                "A13: loneliness_log_2 missing entry for tick {}. loneliness_log_2={loneliness_log_2:?}",
                t_i + 1
            )
        });
    assert!(
        (0.0..SOCIAL_THRESHOLD).contains(&lone_1_after),
        "A13: agent_1 loneliness at tick {} = {lone_1_after}, must be in [0, {SOCIAL_THRESHOLD}). \
         loneliness_log_1={loneliness_log_1:?}",
        t_i + 1
    );
    assert!(
        (0.0..SOCIAL_THRESHOLD).contains(&lone_2_after),
        "A13: agent_2 loneliness at tick {} = {lone_2_after}, must be in [0, {SOCIAL_THRESHOLD}). \
         loneliness_log_2={loneliness_log_2:?}",
        t_i + 1
    );

    // ── A14: chronicle isolation — count every AgentDecision whose
    //        `reason != DecisionReason::SocialReason`. Type A. The whitelist
    //        is intentionally inverted (assert "all non-Social", not a
    //        fixed enumeration) to be future-proof against new
    //        DecisionReason variants. Any nonzero count = scenario leakage
    //        or unrelated cascade firing, which weakens every other
    //        "Social wins" claim in this suite.
    let non_social = causal_log_acc
        .iter()
        .filter(|(_, ev)| {
            matches!(
                ev,
                CausalEvent::AgentDecision { reason, .. }
                    if *reason != DecisionReason::SocialReason
            )
        })
        .count();
    assert_eq!(
        non_social, 0,
        "A14: {non_social} non-Social AgentDecision(s) leaked into chronicle. \
         causal_log={causal_log_acc:?}"
    );

    // ── A14b: position invariance — both agents remain at (6, 5) for all
    //        observed ticks. Type A. Plan §γ A14: any movement (e.g. a
    //        regression where AgentMovementSystem stops honoring
    //        AgentState::suppresses_movement) breaks Seeking→Consuming
    //        detection silently. The set form catches even a single-tick
    //        displacement.
    let mut positions_observed: BTreeSet<(u32, u32)> = BTreeSet::new();
    for (_, p) in &position_log_1 {
        positions_observed.insert(*p);
    }
    for (_, p) in &position_log_2 {
        positions_observed.insert(*p);
    }
    let expected_positions: BTreeSet<(u32, u32)> = [(SHARED_X, SHARED_Y)].into_iter().collect();
    assert_eq!(
        positions_observed, expected_positions,
        "A14b: observed positions set {positions_observed:?} != expected \
         {{({SHARED_X}, {SHARED_Y})}}. position_log_1={position_log_1:?} \
         position_log_2={position_log_2:?}"
    );

    // ── A14c: AgentState::suppresses_movement() == true whenever either
    //        agent is in Seeking{Agent(_)} or Consuming{Agent(_)}. Type A.
    //        Plan §γ A15. Direct assertion on the state predicate that
    //        keeps agents co-located — independent of Position observation,
    //        giving redundant defense against drift.
    for (log_name, log) in [("state_log_1", &state_log_1), ("state_log_2", &state_log_2)] {
        for (t, s) in log {
            match s {
                AgentState::Seeking { target: TargetKind::Agent(_) }
                | AgentState::Consuming { target: TargetKind::Agent(_) } => {
                    assert!(
                        s.suppresses_movement(),
                        "A14c: {log_name} at tick {t} state {s:?} must have \
                         suppresses_movement()==true. log={log:?}"
                    );
                }
                _ => {}
            }
        }
    }

    // ── A14d: anti-bypass anchor (plan §γ A16) — SocialInteractionSystem
    //        was actually executed. Type A. `had_nonzero` (asserted in A5
    //        block above) + exactly one Started + one Completed prove the
    //        γ-specific code path fired. A Generator that hardcodes
    //        familiarity = 0.1 without invoking the system would fail this.
    let started_count = started.len();
    let completed_count = completed.len();
    assert_eq!(
        started_count + completed_count,
        2,
        "A14d/A16: started_count + completed_count = {} (started={started_count}, \
         completed={completed_count}), expected exactly 2 (1 + 1). \
         causal_log={causal_log_acc:?}",
        started_count + completed_count
    );

    // ── A15: regression sentinel — full cycle completes within N_TICKS.
    //        Type D — regression guard against tick-loop slowdown / config
    //        drift. Theoretical timing: breach ~52, Consuming ~53,
    //        Idle ~56, leaving ~24 ticks of margin to N_TICKS=80. If a
    //        legitimate config change requires more ticks, this sentinel
    //        forces N_TICKS to be revisited rather than silently absorbing
    //        the slowdown.
    assert!(
        t_i < N_TICKS,
        "A15: t_i = {t_i} not strictly < N_TICKS = {N_TICKS}. \
         state_log_1={state_log_1:?} state_log_2={state_log_2:?}"
    );

    // ── A17: familiarity symmetry / canonical store (plan_attempt 3) ─────
    //        Read familiarity via `RelationshipKey::new(id_1, id_2)` AND
    //        `RelationshipKey::new(id_2, id_1)` independently and assert
    //        both return `Some(_)` with EQUAL values (epsilon 1e-12).
    //        Type A. Positively defends against a per-agent familiarity
    //        store desync — a per-agent implementation would either fail
    //        one of the lookups (None vs Some) or return divergent values
    //        for the same pair. Canonical-store contract: `RelationshipKey
    //        ::new(a, b) == RelationshipKey::new(b, a)`, and the lookup
    //        must therefore agree.
    let key_ab = RelationshipKey::new(id_1, id_2);
    let key_ba = RelationshipKey::new(id_2, id_1);
    let fam_ab = engine
        .resources
        .relationships
        .get(&key_ab)
        .map(|r| r.familiarity);
    let fam_ba = engine
        .resources
        .relationships
        .get(&key_ba)
        .map(|r| r.familiarity);
    assert!(
        fam_ab.is_some(),
        "A17: relationships.get(&RelationshipKey::new(id_1={id_1}, id_2={id_2})) \
         returned None; canonical-store contract violated. relationships={:?}",
        engine.resources.relationships
    );
    assert!(
        fam_ba.is_some(),
        "A17: relationships.get(&RelationshipKey::new(id_2={id_2}, id_1={id_1})) \
         returned None; canonical-store contract violated. relationships={:?}",
        engine.resources.relationships
    );
    let v_ab = fam_ab.unwrap();
    let v_ba = fam_ba.unwrap();
    assert!(
        (v_ab - v_ba).abs() < 1e-12,
        "A17: familiarity symmetry violation — \
         (id_1, id_2)={v_ab} vs (id_2, id_1)={v_ba}, |Δ|={} >= 1e-12. \
         A per-agent or non-canonical store would diverge here. \
         relationships={:?}",
        (v_ab - v_ba).abs(),
        engine.resources.relationships
    );

    // A16 — meta-constraint, not asserted at runtime (see module docs).

    println!(
        "[γ-chronicle p7] id_1={id_1} id_2={id_2} t_s1={t_s1} t_s2={t_s2} \
         t_c1={t_c1} t_c2={t_c2} t_i1={t_i1} t_i2={t_i2} t_i={t_i} \
         delta_1={delta_1} delta_2={delta_2} final_fam={final_fam} \
         started_id={started_id} completed_id={completed_id} \
         social_decisions={} N_TICKS={N_TICKS}",
        social_decisions.len()
    );

    // Diagnostic counter required by attempt-1 RE-CODE feedback (issue 3):
    // emit `agents_doing_social_interaction=N` with `N > 0`.
    let agents_doing_social_interaction = social_decisions.len();
    println!(
        "[γ-chronicle p7] agents_doing_social_interaction={agents_doing_social_interaction}"
    );
    assert!(
        agents_doing_social_interaction > 0,
        "diagnostic: agents_doing_social_interaction must be > 0 (got 0)"
    );
}
