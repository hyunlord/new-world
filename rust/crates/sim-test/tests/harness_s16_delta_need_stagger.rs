//! V7 Section 16-δ — need stagger + acceleration harness.
//!
//! δ replaces the identical bootstrap need values (all `0.0`) and slow growth
//! rates (Hunger 0.02 / Thirst 0.03 / Sleep 0.01) with **per-agent staggered
//! initial values** (deterministic, `0..=BOOTSTRAP_NEED_STAGGER_MAX = 45`,
//! strictly below the 50 breach threshold so every agent is still Idle right
//! after bootstrap) and **accelerated growth rates** (Thirst 0.08 > Hunger
//! 0.05 > Sleep 0.03). This de-synchronizes the breach burst (pre-δ: all 64
//! agents breached on the SAME tick — measured peak 64 simultaneous
//! `Seeking{Water}` at tick 1667) into a continuous trickle.
//!
//! These tests reproduce the EXACT production bootstrap path —
//! `SimEngine::new(64, 64, MaterialRegistry::new())` then
//! `register_default_runtime_systems` then `bootstrap_spawn_agents` — so the
//! values under test are the real production values.
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_delta_need_stagger -- --nocapture

use std::collections::{HashMap, HashSet};

use hecs::Entity;
use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_core::components::{Agent, AgentState, Hunger, Sleep, TargetKind, Thirst};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;

// ── constants mirrored from the production bootstrap / locked spec ──────────

/// Map extent — mirrors `world_node::DEFAULT_W` / `DEFAULT_H` (private).
const DEFAULT_W: u32 = 64;
const DEFAULT_H: u32 = 64;

/// Expected agent count: `BOOTSTRAP_AGENT_AXIS²` = 8² = 64.
const EXPECTED_AGENTS: usize = 64;

/// Locked δ stagger cap — initial need values land in `0..=45`.
const STAGGER_MAX: u64 = 45;

/// Locked δ growth rates (Thirst > Hunger > Sleep).
const EXPECTED_HUNGER_RATE: f32 = 0.05;
const EXPECTED_THIRST_RATE: f64 = 0.08;
const EXPECTED_SLEEP_RATE: f64 = 0.03;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build a fresh engine via the EXACT production bootstrap path:
/// `new` → `register_default_runtime_systems` → `bootstrap_spawn_agents`.
/// (Tick-0 assertions simply do not call `engine.tick()`.)
fn bootstrapped_engine() -> SimEngine {
    let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    bootstrap_spawn_agents(&mut engine);
    engine
}

/// Collect the bootstrap initial need values for all agents as parallel
/// `f64` vectors `(hunger, thirst, sleep_fatigue)`. Hunger is widened from
/// `f32`; all three are derived as whole numbers (`rng % 46`).
fn collect_need_values(engine: &SimEngine) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut hs = Vec::new();
    let mut ts = Vec::new();
    let mut ss = Vec::new();
    for (_, (_, h, t, s)) in engine
        .world
        .query::<(&Agent, &Hunger, &Thirst, &Sleep)>()
        .iter()
    {
        hs.push(h.value as f64);
        ts.push(t.value);
        ss.push(s.fatigue);
    }
    (hs, ts, ss)
}

/// Distinct-value count over a slice of whole-number need values. Values are
/// exact whole numbers (`rng % 46`), so casting to `i64` is lossless for the
/// distinctness check.
fn distinct_count(values: &[f64]) -> usize {
    values.iter().map(|v| *v as i64).collect::<HashSet<_>>().len()
}

/// Sort a need-value vector with `f64::total_cmp` (whole numbers compare
/// exactly; total_cmp avoids any NaN-ordering concern).
fn sorted(values: &[f64]) -> Vec<f64> {
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    v
}

// ─── Assertion 1: initial need values within [0, 45] ───────────────────────
#[test]
fn harness_s16_delta_initial_need_values_within_bound() {
    // Type: A — mathematical invariant. value = rng % 46 ∈ [0, 45] < 50 breach
    // threshold. Any value ≥ 46 means the derivation is wrong; ≥ 50 breaks A13.
    let engine = bootstrapped_engine();
    let (hs, ts, ss) = collect_need_values(&engine);
    assert_eq!(hs.len(), EXPECTED_AGENTS, "A1.0: must inspect all 64 agents");

    let mut violations = 0usize;
    for (h, (t, s)) in hs.iter().zip(ts.iter().zip(ss.iter())) {
        for v in [*h, *t, *s] {
            if !(0.0..=STAGGER_MAX as f64).contains(&v) {
                violations += 1;
            }
        }
    }
    assert_eq!(
        violations, 0,
        "A1: every initial Hunger/Thirst/Sleep value must be ∈ [0, {STAGGER_MAX}]; got {violations} out-of-bound"
    );
    println!("[S16-δ A1] all 64×3 initial need values ∈ [0, {STAGGER_MAX}], 0 violations ✓");
}

// ─── Assertion 2: stagger spread exists ────────────────────────────────────
#[test]
fn harness_s16_delta_stagger_spread_exists() {
    // Type: C — distributional. Pre-δ baseline: all 64 identical (distinct=1).
    // Expected distinct ≈ 46·(1−(45/46)^64) ≈ 35. Threshold 10 ≈ 29% of
    // expected → robust margin while catching the pre-δ collapse.
    let engine = bootstrapped_engine();
    let (hs, ts, ss) = collect_need_values(&engine);

    let dh = distinct_count(&hs);
    let dt = distinct_count(&ts);
    let ds = distinct_count(&ss);
    // Type C measurement requirement — observed distinct counts at the
    // bootstrap seed (recorded 2026-05-31, seed-derived deterministic):
    //   Hunger distinct = 32, Thirst distinct = 31, Sleep distinct = 31.
    println!("[S16-δ A2] observed distinct counts — Hunger={dh}, Thirst={dt}, Sleep={ds}");

    assert!(dh >= 10, "A2.1: Hunger initial values must have ≥10 distinct; got {dh}");
    assert!(dt >= 10, "A2.2: Thirst initial values must have ≥10 distinct; got {dt}");
    assert!(ds >= 10, "A2.3: Sleep initial values must have ≥10 distinct; got {ds}");
    // Explicit "not all identical" guard (subsumed by ≥10 but stated per plan).
    assert!(dh > 1 && dt > 1 && ds > 1, "A2.4: need values must not be all identical");
    println!("[S16-δ A2] stagger spread confirmed (≥10 distinct each need) ✓");
}

// ─── Assertion 3: stagger determinism ──────────────────────────────────────
#[test]
fn harness_s16_delta_stagger_determinism() {
    // Type: A — determinism invariant. Seeded splitmix64, no wall-clock / OS
    // randomness → two independent bootstraps produce byte-identical SORTED
    // value lists (sorted to isolate value-derivation from ECS iteration order).
    let a = bootstrapped_engine();
    let b = bootstrapped_engine();
    let (ah, at, as_) = collect_need_values(&a);
    let (bh, bt, bs) = collect_need_values(&b);

    assert_eq!(sorted(&ah), sorted(&bh), "A3.1: sorted Hunger values must be identical across engines");
    assert_eq!(sorted(&at), sorted(&bt), "A3.2: sorted Thirst values must be identical across engines");
    assert_eq!(sorted(&as_), sorted(&bs), "A3.3: sorted Sleep values must be identical across engines");
    println!("[S16-δ A3] two bootstraps → identical sorted need-value lists (deterministic) ✓");
}

// ─── Assertion 4: all Idle at bootstrap (A13 preserved) ────────────────────
#[test]
fn harness_s16_delta_all_idle_at_bootstrap_a13_preserved() {
    // Type: A — invariant preserving s16_alpha0:A13. Every staggered value
    // ∈ [0,45] < 50 breach threshold ⇒ no need breaches at bootstrap ⇒ no
    // agent can be Seeking/Consuming. A single non-Idle agent ⇒ cap/derivation wrong.
    let engine = bootstrapped_engine();
    let mut total = 0usize;
    let mut non_idle = 0usize;
    for (_, (_, state)) in engine.world.query::<(&Agent, &AgentState)>().iter() {
        total += 1;
        if *state != AgentState::Idle {
            non_idle += 1;
        }
    }
    assert_eq!(total, EXPECTED_AGENTS, "A4.0: must spawn exactly {EXPECTED_AGENTS} agents; got {total}");
    assert_eq!(non_idle, 0, "A4: all {EXPECTED_AGENTS} agents must be Idle at bootstrap; got {non_idle} non-Idle");
    println!("[S16-δ A4] {total} agents, all Idle at bootstrap (A13 preserved) ✓");
}

// ─── Assertion 5: breach de-synchronization (headline) ─────────────────────
#[test]
fn harness_s16_delta_breach_desynchronization_headline() {
    // Type: C — headline de-sync. Pre-δ baseline: all 64 breached on the SAME
    // tick (range 0). With stagger 0..=45 + Thirst rate 0.08, first-Seek ticks
    // expected to span ≈ 63..625 (range ≈ 562). Threshold range ≥ 200 ≈ 35% of
    // expected (robust); distinct ≥ 10 confirms a continuous trickle. Social
    // (rate 0.04, start 0.0) breaches at 1250 > 700 → does not contaminate.
    let mut engine = bootstrapped_engine();
    let mut first_seek: HashMap<Entity, u64> = HashMap::new();
    for tick in 1..=700u64 {
        engine.tick();
        for (e, (_, state)) in engine.world.query::<(&Agent, &AgentState)>().iter() {
            if matches!(state, AgentState::Seeking { .. }) {
                first_seek.entry(e).or_insert(tick);
            }
        }
    }

    let ticks: Vec<u64> = first_seek.values().copied().collect();
    assert!(
        !ticks.is_empty(),
        "A5.0: at least one agent must enter Seeking within 700 ticks"
    );
    let min = *ticks.iter().min().unwrap();
    let max = *ticks.iter().max().unwrap();
    let range = max - min;
    let distinct = ticks.iter().copied().collect::<HashSet<_>>().len();
    // Type C measurement requirement — observed at the bootstrap seed
    // (recorded 2026-05-31): min first-Seek = 64, max = 627, range = 563,
    // distinct first-Seek ticks = 40, agents that breached within 700 = 64.
    println!(
        "[S16-δ A5] first-Seek window: min={min}, max={max}, range={range}, distinct={distinct}, breached={}",
        ticks.len()
    );

    assert!(range >= 200, "A5.1: first-Seek tick range must be ≥ 200 (de-synchronized); got {range}");
    assert!(distinct >= 10, "A5.2: distinct first-Seek ticks must be ≥ 10 (continuous trickle); got {distinct}");
    println!("[S16-δ A5] breach de-synchronization confirmed (range {range} ≥ 200, distinct {distinct} ≥ 10) ✓");
}

// ─── Assertion 6: accelerated growth rates + early breach ──────────────────
#[test]
fn harness_s16_delta_accelerated_growth_rates() {
    // Type: A — rate fields are exact locked consts → exact equality (within
    // float epsilon). Ordering Thirst > Hunger > Sleep is locked spec. Early-
    // breach sub-check: a regression to old 0.02/0.03/0.01 rates would prevent
    // a continuous early trickle within 700 ticks.
    let mut engine = bootstrapped_engine();

    // (a)/(b) read the growth-rate fields of one bootstrap agent.
    let (hr, tr, sr) = {
        let mut iter = engine.world.query::<(&Agent, &Hunger, &Thirst, &Sleep)>();
        let (_, (_, h, t, s)) = iter.iter().next().expect("A6.0: at least one agent");
        (h.growth_rate, t.growth_rate, s.growth_rate)
    };
    assert!(
        (hr - EXPECTED_HUNGER_RATE).abs() < 1e-6,
        "A6.1: Hunger rate must == {EXPECTED_HUNGER_RATE}; got {hr}"
    );
    assert!(
        (tr - EXPECTED_THIRST_RATE).abs() < 1e-9,
        "A6.2: Thirst rate must == {EXPECTED_THIRST_RATE}; got {tr}"
    );
    assert!(
        (sr - EXPECTED_SLEEP_RATE).abs() < 1e-9,
        "A6.3: Sleep rate must == {EXPECTED_SLEEP_RATE}; got {sr}"
    );
    // Ordering Thirst > Hunger > Sleep (locked spec).
    assert!(
        tr > hr as f64 && (hr as f64) > sr,
        "A6.4: rate ordering must be Thirst({tr}) > Hunger({hr}) > Sleep({sr})"
    );
    println!("[S16-δ A6] rates: Hunger={hr}, Thirst={tr}, Sleep={sr}; ordering Thirst>Hunger>Sleep ✓");

    // (c) at least one agent enters Seeking within 700 ticks.
    let mut seeking_seen = false;
    for _ in 0..700u64 {
        engine.tick();
        if engine
            .world
            .query::<(&Agent, &AgentState)>()
            .iter()
            .any(|(_, (_, st))| matches!(st, AgentState::Seeking { .. }))
        {
            seeking_seen = true;
            break;
        }
    }
    assert!(seeking_seen, "A6.5: ≥1 agent must enter Seeking within 700 ticks (rates were actually raised)");
    println!("[S16-δ A6] ≥1 agent entered Seeking within 700 ticks (accelerated) ✓");
}

// ─── Assertion 7: gathering loop still completes ───────────────────────────
#[test]
fn harness_s16_delta_gathering_loop_still_completes() {
    // Type: D — regression guard. δ changes ONLY bootstrap initial values and
    // growth rates; it must NOT break the α0/α/β gathering loop. Observe ≥1
    // agent transition Seeking → Consuming with the targeted need strictly
    // decreasing (consume amount 30 ≫ growth ⇒ net decrease).
    let mut engine = bootstrapped_engine();

    // Per-entity value captured on entry to a resource Consuming state.
    let mut consuming_entry: HashMap<Entity, (TargetKind, f64)> = HashMap::new();
    let mut loop_completed = false;

    for _ in 0..1000u64 {
        engine.tick();
        // Snapshot (entity, state, need values) this tick.
        let mut snapshot: Vec<(Entity, AgentState, f64, f64, f64)> = Vec::new();
        for (e, (_, state, h, t, s)) in engine
            .world
            .query::<(&Agent, &AgentState, &Hunger, &Thirst, &Sleep)>()
            .iter()
        {
            snapshot.push((e, *state, h.value as f64, t.value, s.fatigue));
        }

        for (e, state, hv, tv, sv) in snapshot {
            match state {
                // Resource consume (Food/Water/Sleep) — capture targeted need on entry.
                AgentState::Consuming { target } if is_resource_target(target) => {
                    let v = need_value_for(target, hv, tv, sv);
                    consuming_entry.entry(e).or_insert((target, v));
                }
                // Left the consuming state — compare the targeted need.
                _ => {
                    if let Some((target, v_before)) = consuming_entry.remove(&e) {
                        let v_after = need_value_for(target, hv, tv, sv);
                        if v_after < v_before - 1e-6 {
                            loop_completed = true;
                            println!(
                                "[S16-δ A7] Seeking→Consuming→Idle on {:?}: {v_before} → {v_after} (Δ={}) ✓",
                                target,
                                v_before - v_after
                            );
                        }
                    }
                }
            }
        }

        if loop_completed {
            break;
        }
    }

    assert!(
        loop_completed,
        "A7: ≥1 agent must complete Seeking → Consuming with its targeted need strictly decreasing (gathering loop preserved)"
    );
    println!("[S16-δ A7] gathering loop survives δ (Seeking→Consuming, need decreased) ✓");
}

/// Whether a `TargetKind` is one of the three staggered resource needs.
fn is_resource_target(target: TargetKind) -> bool {
    matches!(target, TargetKind::Food | TargetKind::Water | TargetKind::Sleep)
}

/// Map a resource `TargetKind` to its agent's current need value.
fn need_value_for(target: TargetKind, hunger: f64, thirst: f64, sleep_fatigue: f64) -> f64 {
    match target {
        TargetKind::Food => hunger,
        TargetKind::Water => thirst,
        TargetKind::Sleep => sleep_fatigue,
        _ => f64::NAN,
    }
}
