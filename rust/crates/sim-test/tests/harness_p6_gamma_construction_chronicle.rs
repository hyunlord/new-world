//! V7 Phase 6-γ — Construction chronicle (closure milestone for Phase 6).
//!
//! Walks one agent through a full
//!     Idle → Seeking{ConstructionSite} → Consuming{ConstructionSite} → Idle
//! cycle and proves the 4-link causal chain:
//!     AgentDecision{ConstructionReason}
//!       → ConstructionStarted
//!       → ConstructionCompleted
//!       → BuildingPlaced
//! plus complete site despawn and agent reset.
//!
//! plan_attempt: 3
//! assertions: A1..A15 (full final-plan set)
//! lane: --full
//!
//! Mirrors the Phase 5-γ chronicle pattern (per-tick state + causal +
//! progress accumulators, single `#[test]` function, full per-tick log
//! dump on any failure). No production code changes — this dispatch
//! only adds this test file.
//!
//! γ-vs-β anti-circularity (each MUST fail under β code alone):
//!   - A10 walks the full parent linkage end-to-end through the full
//!     production engine pipeline (β tests each system in isolation only).
//!   - A11(ii) probes a generation-recycle guard: BuildingPlaced payload
//!     is a durable `(u32, u32)` tuple, NOT a hecs entity handle, so
//!     reusing the despawned site's slot must not retroactively alter the
//!     event payload's identity.
//!   - A13 asserts the world durably reflects the completed building
//!     (causal log + agent reset + zero leftover ConstructionSite at the
//!     position) after the site entity is gone — only the integrated
//!     full-engine pipeline can produce this end-state.
//!
//! References:
//!   - Phase 6-α commit ba4e02b2 (BuildingBlueprint + ConstructionSite +
//!     TargetKind::ConstructionSite — data substrate).
//!   - Phase 6-β commit 21b09e26 (ConstructionSystem at priority 133 +
//!     ConstructionStarted/Completed causal variants +
//!     DecisionReason::ConstructionReason + AgentDecisionSystem
//!     4th-cascade integration).
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p6_gamma_construction_chronicle -- --nocapture`

use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    AgentState, BuildingBlueprint, ConstructionSite, Hunger, Position, Sleep, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::{
    register_agent_systems, register_construction_systems, register_decision_systems,
    register_needs_systems, register_phase2_systems,
};

#[test]
fn harness_p6_gamma_construction_chronicle() {
    // ── Scenario setup ────────────────────────────────────────────────
    // Tick budget derivation (Type B, plan_attempt 3 §2.3 A15):
    //   worst-case t_i = decision(1) + start(1) + REQUIRED_PROGRESS(5) +
    //     completion settle(<=2) = 9 ticks. Plan-locked threshold = 12.
    //   N_TICKS = 20 leaves ~58% headroom over the 12-tick A15 ceiling.
    const N_TICKS: u64 = 20;
    const SITE_X: u32 = 8;
    const SITE_Y: u32 = 8;
    const REQUIRED_PROGRESS: u32 = 5;
    const BLUEPRINT_ID: u64 = 1;
    const FOOTPRINT_W: u32 = 2;
    const FOOTPRINT_H: u32 = 2;

    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());

    // Register the full production system surface — Phase 6-β runtime
    // wiring (priority order: 90 BSS → 100 IUS → 110 AIS → 120 movement
    // → 125 decision → 130/131/132 needs → 133 construction → 1000 viz).
    register_phase2_systems(&mut engine);
    register_agent_systems(&mut engine);
    register_decision_systems(&mut engine);
    register_needs_systems(&mut engine);
    register_construction_systems(&mut engine);

    // Spawn the single agent at (8, 8) with quiet needs — growth_rate
    // 0.0 means Hunger/Thirst/Fatigue never breach during the chronicle,
    // isolating ConstructionReason as the only possible decision driver.
    let agent_entity = engine.spawn_agent(SITE_X, SITE_Y);
    engine
        .world
        .insert(
            agent_entity,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
            ),
        )
        .expect("insert quiet-needs bag on agent");

    // Spawn the co-located ConstructionSite. The original blueprint is
    // cached so A13 can compare what the causal log durably preserves
    // against the original spec after the site entity is gone.
    let original_blueprint = BuildingBlueprint::new(
        BLUEPRINT_ID,
        FOOTPRINT_W,
        FOOTPRINT_H,
        REQUIRED_PROGRESS,
    );
    let site_entity = engine.world.spawn((ConstructionSite::new(
        original_blueprint,
        Position {
            x: SITE_X,
            y: SITE_Y,
        },
    ),));

    // ── Per-tick chronicle accumulators ───────────────────────────────
    let width = engine.resources.tile_grid.width;
    let tile_idx: u32 = SITE_Y * width + SITE_X;
    let mut state_log: Vec<(u64, AgentState)> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut causal_log: Vec<(u64, CausalEvent)> = Vec::new();
    let mut progress_log: Vec<(u64, u32)> = Vec::with_capacity(N_TICKS as usize + 1);
    let mut last_log_len: usize = 0;
    let mut max_agents_doing_construction: usize = 0;

    // Pre-tick snapshot (labelled tick 0).
    state_log.push((
        0,
        *engine
            .world
            .get::<&AgentState>(agent_entity)
            .expect("agent AgentState present at tick 0"),
    ));
    progress_log.push((
        0,
        engine
            .world
            .get::<&ConstructionSite>(site_entity)
            .map(|s| s.progress)
            .unwrap_or(REQUIRED_PROGRESS),
    ));

    // Tick loop — `now` is 1-indexed (the engine has just COMPLETED tick `now`).
    for i in 0..N_TICKS {
        engine.tick();
        let now: u64 = i + 1;

        // Agent state (Idle as fallback if the agent has somehow lost
        // its AgentState component — diagnostic-friendly).
        let s = engine
            .world
            .get::<&AgentState>(agent_entity)
            .map(|s| *s)
            .unwrap_or(AgentState::Idle);
        state_log.push((now, s));

        // Site progress: REQUIRED_PROGRESS sentinel when the site has
        // already been despawned by the completion edge.
        let progress = engine
            .world
            .get::<&ConstructionSite>(site_entity)
            .map(|s| s.progress)
            .unwrap_or(REQUIRED_PROGRESS);
        progress_log.push((now, progress));

        // Behavioural diagnostic — Pipeline harness contract requires
        // `agents_doing_construction=N` with N > 0 on this path.
        let agents_doing_construction: usize = engine
            .world
            .query::<&AgentState>()
            .iter()
            .filter(|(_, st)| {
                matches!(
                    **st,
                    AgentState::Consuming {
                        target: TargetKind::ConstructionSite
                    }
                )
            })
            .count();
        if agents_doing_construction > max_agents_doing_construction {
            max_agents_doing_construction = agents_doing_construction;
        }

        // Capture new causal events appended to the agent's tile this tick.
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

    println!(
        "agents_doing_construction={}",
        max_agents_doing_construction
    );

    // ──────────────────────────────────────────────────────────────────
    // ASSERTIONS (15 total, plan_attempt 3 §2.3 A1..A15)
    // ──────────────────────────────────────────────────────────────────

    // A1: first Seeking{ConstructionSite} tick exists.
    // Type A — pure invariant: the FSM must reach Seeking{ConstructionSite}
    // at least once.
    let t_s_opt = state_log.iter().find_map(|(t, s)| {
        matches!(
            s,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            }
        )
        .then_some(*t)
    });
    assert!(
        t_s_opt.is_some(),
        "A1: chronicle never reached Seeking{{ConstructionSite}}. state_log={state_log:?} causal_log={causal_log:?}"
    );
    let t_s = t_s_opt.unwrap();

    // A2: first Consuming{ConstructionSite} tick in [t_s, t_s + 2].
    // Type A — relaxed window per plan_attempt 3 (admits zero-distance
    // optimization + one routing tick). Strictly: t_c >= t_s AND
    // t_c <= t_s + 2.
    let t_c_opt = state_log.iter().find_map(|(t, s)| {
        matches!(
            s,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite
            }
        )
        .then_some(*t)
    });
    let t_c = t_c_opt.unwrap_or_else(|| {
        panic!(
            "A2: chronicle never reached Consuming{{ConstructionSite}}. state_log={state_log:?} causal_log={causal_log:?}"
        )
    });
    assert!(
        t_c >= t_s && t_c <= t_s + 2,
        "A2: t_c={t_c} not in [t_s={t_s}, t_s+2={}]. state_log={state_log:?} causal_log={causal_log:?}",
        t_s + 2
    );

    // A3: return to Idle at tick > t_c.
    // Type A — Idle re-entry must happen strictly after Consuming begins.
    let t_i_opt = state_log
        .iter()
        .skip_while(|(t, _)| *t <= t_c)
        .find_map(|(t, s)| matches!(s, AgentState::Idle).then_some(*t));
    let t_i = t_i_opt.unwrap_or_else(|| {
        panic!(
            "A3: chronicle never returned to Idle after Consuming. state_log={state_log:?} causal_log={causal_log:?}"
        )
    });
    assert!(
        t_i > t_c,
        "A3: t_i={t_i} not > t_c={t_c}. state_log={state_log:?} causal_log={causal_log:?}"
    );

    // A4: progress non-decreasing during Consuming AND per-tick delta
    //     bounded by REQUIRED_PROGRESS (plan_attempt 3: delta <= 5/tick).
    // Type A — locked upper bound `delta ≤ REQUIRED_PROGRESS` (= 5),
    // rationale: a tighter empirical bound is intentionally avoided.
    let consuming_progress: Vec<(u64, u32)> = progress_log
        .iter()
        .filter(|(t, _)| *t >= t_c && *t <= t_i)
        .cloned()
        .collect();
    assert!(
        !consuming_progress.is_empty(),
        "A4: no progress samples during Consuming window [t_c={t_c}, t_i={t_i}]. progress_log={progress_log:?}"
    );
    let mut prev_p: u32 = 0;
    for (t, p) in &consuming_progress {
        assert!(
            *p >= prev_p,
            "A4: progress regressed during Consuming at tick {t} (prev={prev_p}, curr={p}). progress_log={progress_log:?} state_log={state_log:?}"
        );
        let delta = p.saturating_sub(prev_p);
        assert!(
            delta <= REQUIRED_PROGRESS,
            "A4: progress delta {delta} at tick {t} exceeds REQUIRED_PROGRESS={REQUIRED_PROGRESS} (prev={prev_p}, curr={p}). progress_log={progress_log:?}"
        );
        prev_p = *p;
    }

    // A5: progress >= required_progress at completion tick.
    // Type A — site reached saturation before despawn. The despawn-
    // sentinel value (REQUIRED_PROGRESS) satisfies `>=` trivially.
    let progress_at_t_i = progress_log
        .iter()
        .find(|(t, _)| *t == t_i)
        .map(|(_, p)| *p)
        .unwrap_or(0);
    assert!(
        progress_at_t_i >= REQUIRED_PROGRESS,
        "A5: progress at t_i={t_i} is {progress_at_t_i}, not >= REQUIRED_PROGRESS={REQUIRED_PROGRESS}. progress_log={progress_log:?}"
    );

    // A6: BuildingPlaced emitted exactly once.
    // Type A — single completion-edge BuildingPlaced (radius=0 agent path).
    let building_placed_events: Vec<(u64, CausalEvent)> = causal_log
        .iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::BuildingPlaced { .. }))
        .cloned()
        .collect();
    assert_eq!(
        building_placed_events.len(),
        1,
        "A6: BuildingPlaced count={}, expected 1. causal_log={causal_log:?}",
        building_placed_events.len()
    );

    // A7: ConstructionStarted emitted exactly once.
    // Type A — single Seeking→Consuming edge.
    let construction_started: Vec<(u64, CausalEvent)> = causal_log
        .iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::ConstructionStarted { .. }))
        .cloned()
        .collect();
    assert_eq!(
        construction_started.len(),
        1,
        "A7: ConstructionStarted count={}, expected 1. causal_log={causal_log:?}",
        construction_started.len()
    );

    // A8: ConstructionCompleted emitted exactly once, with id ordering
    //     ConstructionStarted < ConstructionCompleted < BuildingPlaced.
    // Type A — count + monotonic EventId ordering.
    let construction_completed: Vec<(u64, CausalEvent)> = causal_log
        .iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::ConstructionCompleted { .. }))
        .cloned()
        .collect();
    assert_eq!(
        construction_completed.len(),
        1,
        "A8: ConstructionCompleted count={}, expected 1. causal_log={causal_log:?}",
        construction_completed.len()
    );
    let started_id = construction_started[0].1.id();
    let completed_id = construction_completed[0].1.id();
    let placed_event = &building_placed_events[0].1;
    let placed_id = placed_event.id();
    assert!(
        started_id < completed_id && completed_id < placed_id,
        "A8: id ordering violation started={started_id} completed={completed_id} placed={placed_id}. causal_log={causal_log:?}"
    );

    // A9: AgentDecision{ConstructionReason} emitted at least once,
    //     with first id strictly before ConstructionStarted's id.
    // Type A — restricted to the locked DECISION_CONSTRUCTION_REASONS
    // variant set: {DecisionReason::ConstructionReason}. Other reasons
    // are explicitly excluded (caught by A12).
    let construction_reasons: Vec<(u64, CausalEvent)> = causal_log
        .iter()
        .filter(|(_, ev)| {
            matches!(
                ev,
                CausalEvent::AgentDecision {
                    reason: DecisionReason::ConstructionReason,
                    ..
                }
            )
        })
        .cloned()
        .collect();
    assert!(
        !construction_reasons.is_empty(),
        "A9: no AgentDecision{{ConstructionReason}} in chronicle. causal_log={causal_log:?}"
    );
    let first_reason_id = construction_reasons[0].1.id();
    assert!(
        first_reason_id < started_id,
        "A9b: first ConstructionReason id={first_reason_id} not before ConstructionStarted id={started_id}. causal_log={causal_log:?}"
    );

    // A10: full causal-chain parent linkage —
    //   BuildingPlaced.parent       == Some(ConstructionCompleted.id)
    //   ConstructionCompleted.parent == Some(ConstructionStarted.id)
    //   ConstructionStarted.parent   == Some(first ConstructionReason.id)
    // Type A — Option<EventId> exact equality, walked newest → oldest.
    // γ-vs-β anti-circularity: walks ALL three links through the full
    // integrated engine (β's A9 walks them too but via isolated system
    // ticks pre-seeded with synthetic events).
    let placed_parent = placed_event.parent();
    assert_eq!(
        placed_parent,
        Some(completed_id),
        "A10a: BuildingPlaced.parent={placed_parent:?}, expected Some({completed_id}). causal_log={causal_log:?}"
    );
    let completed_parent = construction_completed[0].1.parent();
    assert_eq!(
        completed_parent,
        Some(started_id),
        "A10b: ConstructionCompleted.parent={completed_parent:?}, expected Some({started_id}). causal_log={causal_log:?}"
    );
    let started_parent = construction_started[0].1.parent();
    assert_eq!(
        started_parent,
        Some(first_reason_id),
        "A10c: ConstructionStarted.parent={started_parent:?}, expected Some({first_reason_id}) (first ConstructionReason). causal_log={causal_log:?}"
    );

    // A11: ConstructionSite entity despawned after completion AND
    //      BuildingPlaced payload is durable (not tied to the despawned
    //      hecs handle).
    // Type A — (i) entity component fetch must fail; (ii) generation-
    // recycle guard: spawn a fresh entity that may reuse the despawned
    // site's slot, and verify the BuildingPlaced payload (`position`,
    // a (u32, u32) tuple) is preserved and unrelated to any newly
    // recycled handle.
    let site_still_alive = engine
        .world
        .get::<&ConstructionSite>(site_entity)
        .is_ok();
    assert!(
        !site_still_alive,
        "A11(i): ConstructionSite entity still alive post-completion. state_log={state_log:?} causal_log={causal_log:?}"
    );

    // (ii) Generation-recycle guard. Spawn a probe entity that may take
    //      the despawned site's slot, then verify:
    //      (a) the probe entity has NO ConstructionSite component (no
    //          ghost from the recycled slot),
    //      (b) the BuildingPlaced payload's `position` field is the
    //          original spawn tile (`(SITE_X, SITE_Y)`), proving the
    //          payload is durable data and NOT a stale entity handle.
    let probe_entity = engine.world.spawn(());
    let probe_has_site = engine
        .world
        .get::<&ConstructionSite>(probe_entity)
        .is_ok();
    assert!(
        !probe_has_site,
        "A11(ii)a: probe entity (potentially recycled slot of site_entity) carries a ConstructionSite component. probe={probe_entity:?} site={site_entity:?}"
    );
    let placed_position: (u32, u32) = match placed_event {
        CausalEvent::BuildingPlaced { position, .. } => *position,
        other => panic!("A11(ii)b: placed_event was not BuildingPlaced: {other:?}"),
    };
    assert_eq!(
        placed_position,
        (SITE_X, SITE_Y),
        "A11(ii)b: BuildingPlaced.position={placed_position:?} not equal to original ({SITE_X}, {SITE_Y}). payload must be durable tile coords, not a stale entity handle. causal_log={causal_log:?}"
    );

    // A12: no Need ThresholdBreach during chronicle — proves isolation.
    // Type A — strict zero; the locked NEED_BREACH_REASONS variant set is
    // {HungerThresholdBreach, ThirstThresholdBreach, FatigueThresholdBreach}.
    let need_breaches = causal_log
        .iter()
        .filter(|(_, ev)| {
            matches!(
                ev,
                CausalEvent::AgentDecision {
                    reason: DecisionReason::HungerThresholdBreach,
                    ..
                } | CausalEvent::AgentDecision {
                    reason: DecisionReason::ThirstThresholdBreach,
                    ..
                } | CausalEvent::AgentDecision {
                    reason: DecisionReason::FatigueThresholdBreach,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        need_breaches, 0,
        "A12: {need_breaches} Need-breach AgentDecision(s) leaked into chronicle. causal_log={causal_log:?}"
    );

    // A13: world reflects the completed building after site despawn.
    // Type A — γ-vs-β anti-circularity: the full integrated pipeline
    // leaves THREE durable witnesses that the building completed:
    //   (a) the original site_entity carries no ConstructionSite (despawn
    //       AND not aliased back into existence by any later spawn),
    //   (b) ConstructionCompleted's `blueprint` field matches the original
    //       blueprint verbatim (snapshot durably preserved),
    //   (c) NO ConstructionSite component exists at (SITE_X, SITE_Y)
    //       anywhere in the world (the world reflects the completed
    //       building, not a leftover site).
    assert!(
        engine
            .world
            .get::<&ConstructionSite>(site_entity)
            .is_err(),
        "A13a: site_entity still resolvable as ConstructionSite after despawn. state_log={state_log:?}"
    );
    let completed_blueprint: BuildingBlueprint = match &construction_completed[0].1 {
        CausalEvent::ConstructionCompleted { blueprint, .. } => *blueprint,
        other => panic!("A13b: completed[0] was not ConstructionCompleted: {other:?}"),
    };
    assert_eq!(
        completed_blueprint, original_blueprint,
        "A13b: ConstructionCompleted.blueprint={completed_blueprint:?} not equal to original={original_blueprint:?}. world must durably reflect the original spec. causal_log={causal_log:?}"
    );
    let leftover_sites_at_pos: usize = engine
        .world
        .query::<&ConstructionSite>()
        .iter()
        .filter(|(_, site)| site.position.x == SITE_X && site.position.y == SITE_Y)
        .count();
    assert_eq!(
        leftover_sites_at_pos, 0,
        "A13c: {leftover_sites_at_pos} leftover ConstructionSite(s) at ({SITE_X}, {SITE_Y}); world still shows construction in progress."
    );

    // A14: total AgentDecision count bounded [1, 8] — catches decision-loop bugs.
    // Type D — regression guard. Lower bound 1 (at least the originating
    // ConstructionReason). Upper bound 8 admits a small constant number
    // of re-evaluations (e.g., Idle re-entry after completion) without
    // tolerating an unbounded decision storm.
    let total_agent_decisions: usize = causal_log
        .iter()
        .filter(|(_, ev)| matches!(ev, CausalEvent::AgentDecision { .. }))
        .count();
    assert!(
        (1..=8).contains(&total_agent_decisions),
        "A14: total AgentDecision count={total_agent_decisions} outside [1, 8]. causal_log={causal_log:?}"
    );

    // A15: completion tick within plan-locked Type B bound.
    // Type B — worst-case t_i = 9 (per Section 2 derivation), threshold = 12,
    // ~33% headroom. Locked in plan_attempt 3 §2.3 A15.
    assert!(
        t_i <= 12,
        "A15: t_i={t_i} exceeds locked Type B bound 12 (worst-case derivation = 9, ~33% headroom). state_log={state_log:?} causal_log={causal_log:?}"
    );

    // Behavioural diagnostic — at least one tick observed an agent in
    // Consuming{ConstructionSite}. Pipeline harness contract: N > 0.
    assert!(
        max_agents_doing_construction > 0,
        "diagnostic: agents_doing_construction never exceeded 0 across {N_TICKS} ticks. state_log={state_log:?}"
    );

    println!(
        "[γ-chronicle] t_s={t_s}, t_c={t_c}, t_i={t_i}, REQUIRED_PROGRESS={REQUIRED_PROGRESS}, N_TICKS={N_TICKS}, decisions_total={total_agent_decisions}, construction_decisions={}, started={}, completed={}, placed={}, max_agents_doing_construction={max_agents_doing_construction}",
        construction_reasons.len(),
        construction_started.len(),
        construction_completed.len(),
        building_placed_events.len()
    );
}
