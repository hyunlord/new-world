//! V7 feature `add-resource-scarcity-regen` — Fix D (memory-arm determinism).
//!
//! # Why this harness exists
//!
//! The resource-scarcity scene exposed a determinism root cause (documented in
//! the project memory `resource-scarcity-checkpoint`): the cascade-bias sum
//! `memory_weight_delta` re-derived each memory entry's cascade arm at READ
//! time via `event_id_matches_arm(entry.event_id, arm, causal_log)`, which did
//! a `causal_log.lookup`. That per-tile ring buffer is an 8-slot FIFO, so an
//! entry whose source event had been evicted resolved to "no arm" and silently
//! stopped contributing. Because `event_id` allocation order (and therefore
//! which event occupies which ring slot at a given tick) is not byte-stable
//! across processes, the same logical memory could be load-bearing in one run
//! and inert in another — flipping a cascade decision (`Idle` ↔ `Seeking`) from
//! an otherwise identical agent state, producing a dehydration victim-swap
//! (run A kills a different agent than run B).
//!
//! Fix D records the arm on the [`MemoryEntry`] at ENCODE time (from
//! `classify_event`) and has `memory_weight_delta` read that stored tag. The
//! delta becomes a pure function of each entry's own `(arm, valence, salience,
//! encoded_tick)` — independent of `event_id` values AND ring-buffer eviction.
//!
//! # Assertions
//!   A1  encode-time arm tagging — MemorySystem stores the classified arm ...... Type A
//!   A2  full production-scarcity scene is DETERMINISTIC under a per-tick
//!       behavioural lockstep (cap 2200, two independent in-process runs) ...... Type A
//!
//! Fingerprint scope (A2): the lockstep hash covers BEHAVIOURAL state only —
//! agent position, `AgentState`, all needs (bit-exact), `BodyHealth`, the
//! dead-agent set, the per-reason death split, and resource-tile key-sets. It
//! deliberately EXCLUDES `event_id` values and memory `event_id` contents:
//! Fix D leaves `event_id` a non-load-bearing label that MAY still differ
//! across runs (it does not determinise the id counter). Determinism of the
//! observable simulation outcome is the contract, not of internal labels.
//!
//! Run:
//!   `cargo test -p sim-test --test harness_fix_d_memory_arm -- --nocapture`
//!   (release strongly recommended for A2: `cargo test --release ...`)

use std::collections::BTreeSet;
use std::hash::BuildHasher;

use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, seed_finite_resource_scarcity};
use sim_core::causal::event::{CausalEvent, DeathReason, DecisionReason, EventId};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Memory, MemoryArm, MemoryEntry, Position, Sleep,
    TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::memory::{MemorySystem, MAX_RECENCY_TICKS};

const W: u32 = 64;
const H: u32 = 64;

/// Production-EQUIVALENT scene WITH finite scarcity — byte-for-byte the same
/// construction `harness_resource_scarcity::finite_scene` uses (the scene that
/// exhibited the nondeterminism): default runtime + bootstrap lattice + finite
/// seeding + the 3 startup buildings that seed an early settlement.
fn finite_scene() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    seed_finite_resource_scarcity(&mut e);
    for pos in [(32u32, 32u32), (24, 32), (40, 32)] {
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent { position: pos, radius: 8 });
    }
    e
}

/// Direct construction-path assertion of the distinct-seed precondition: the
/// two engines' resource HashMaps were built with INDEPENDENT
/// `RandomState::new()` seeds.
///
/// `HashMap::hasher()` exposes each map's actual `RandomState` — the seed the
/// construction path drew via `HashMap::new()` → `RandomState::new()`. Hashing
/// the same fixed keys through both hashers yields a different value for at
/// least one key iff the seeds are independent. This reads the REAL engine
/// hashers directly — NOT an iteration-order sample of a small key set (which
/// could coincide and falsely report INVALID). Two distinct 64-bit-seeded
/// `RandomState`s disagree on at least one of these fixed keys with
/// overwhelming probability.
fn engines_independently_seeded(a: &SimEngine, b: &SimEngine) -> bool {
    let ha = a.resources.food_tiles.hasher();
    let hb = b.resources.food_tiles.hasher();
    [(0u32, 0u32), (1, 2), (3, 5), (7, 11), (13, 17)]
        .into_iter()
        .any(|k| ha.hash_one(k) != hb.hash_one(k))
}

/// Encode a single event for a fresh actor agent and return the stored
/// [`MemoryArm`] on the resulting entry (`None` if the event was not encoded).
///
/// `make` builds the [`CausalEvent`] from `(event_id, actor_agent_id,
/// current_tick)`. `MemorySystem` only encodes events whose `tick() ==
/// current_tick` AND whose `classify_event` agent list contains a live agent,
/// so the event must be built at `current_tick` for the spawned actor.
fn encode_event_arm(make: impl FnOnce(EventId, AgentId, u64) -> CausalEvent) -> Option<MemoryArm> {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    let ent = e.spawn_agent(10, 10);
    e.world.insert_one(ent, Memory::new()).expect("attach Memory");
    let aid = e.world.get::<&Agent>(ent).expect("agent present").id;
    let tick = e.resources.current_tick;
    let idx = 10 * W + 10;
    let ev_id = e.resources.issue_event_id();
    e.resources.causal_log.push(idx, make(ev_id, aid, tick));
    let mut sys = MemorySystem;
    sys.tick(&mut e.world, &mut e.resources);
    let mem = e.world.get::<&Memory>(ent).expect("memory present");
    mem.entries.iter().find(|m| m.event_id == ev_id).map(|m| m.arm)
}

/// Deterministic FNV-1a fold helper.
fn fnv(acc: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *acc ^= b as u64;
        *acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

/// Hash of the BEHAVIOURALLY-observable engine state at the current tick.
///
/// Excludes `event_id` values (non-load-bearing labels under Fix D). Includes
/// every field that determines the simulation's future trajectory: per-agent
/// position + `AgentState` + needs (bit-exact) + `BodyHealth`, the dead set,
/// the per-reason death split, and the resource-tile key-sets.
fn behavioural_fingerprint(e: &SimEngine) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;

    // Collect entities first (release the query borrow before per-component
    // `world.get` access), then build one row per agent.
    let ents: Vec<hecs::Entity> = e.world.query::<&Agent>().iter().map(|(ent, _)| ent).collect();
    let mut rows: Vec<(AgentId, [u8; 8], [u8; 8], String)> = Vec::new();
    for ent in ents {
        let Ok(agent) = e.world.get::<&Agent>(ent) else { continue };
        let pos_bits = e
            .world
            .get::<&Position>(ent)
            .map(|p| ((p.x as u64) << 32) | p.y as u64)
            .unwrap_or(0);
        let hunger = e
            .world
            .get::<&Hunger>(ent)
            .map(|h| (h.value as f64).to_bits())
            .unwrap_or(0);
        let thirst = e
            .world
            .get::<&Thirst>(ent)
            .map(|t| t.value.to_bits())
            .unwrap_or(0);
        let sleep = e
            .world
            .get::<&Sleep>(ent)
            .map(|s| s.fatigue.to_bits())
            .unwrap_or(0);
        let hp = e
            .world
            .get::<&BodyHealth>(ent)
            .map(|b| b.hp.to_bits())
            .unwrap_or(0);
        let state = e
            .world
            .get::<&AgentState>(ent)
            .map(|s| format!("{s:?}"))
            .unwrap_or_default();
        let need_mix = hunger
            .wrapping_mul(0x9E37_79B9)
            ^ thirst.rotate_left(17)
            ^ sleep.rotate_left(31)
            ^ hp.rotate_left(7);
        rows.push((agent.id, pos_bits.to_le_bytes(), need_mix.to_le_bytes(), state));
    }
    rows.sort_by_key(|(id, ..)| *id);
    for (id, pos_bits, need_mix, state) in &rows {
        fnv(&mut acc, &id.to_le_bytes());
        fnv(&mut acc, pos_bits);
        fnv(&mut acc, need_mix);
        fnv(&mut acc, state.as_bytes());
        fnv(&mut acc, b"|");
    }

    // Dead set + per-reason split (the victim-swap observables).
    let (mut starv, mut dehy, mut comb) = (0u32, 0u32, 0u32);
    let mut dead: BTreeSet<AgentId> = BTreeSet::new();
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, reason, .. } = ev {
                dead.insert(*agent);
                match reason {
                    DeathReason::Starvation => starv += 1,
                    DeathReason::Dehydration => dehy += 1,
                    DeathReason::Combat => comb += 1,
                }
            }
        }
    }
    fnv(&mut acc, b"#dead");
    for id in &dead {
        fnv(&mut acc, &id.to_le_bytes());
    }
    fnv(&mut acc, &starv.to_le_bytes());
    fnv(&mut acc, &dehy.to_le_bytes());
    fnv(&mut acc, &comb.to_le_bytes());

    // Resource-tile key-sets (depletion/regen surface).
    for (label, map) in [
        (b"#food".as_slice(), &e.resources.food_tiles),
        (b"#water".as_slice(), &e.resources.water_tiles),
        (b"#sleep".as_slice(), &e.resources.sleep_tiles),
    ] {
        fnv(&mut acc, label);
        let mut keys: Vec<(u32, u32)> = map.keys().copied().collect();
        keys.sort_unstable();
        for (x, y) in keys {
            fnv(&mut acc, &x.to_le_bytes());
            fnv(&mut acc, &y.to_le_bytes());
            // value too — regen amount must be deterministic.
            if let Some(v) = map.get(&(x, y)) {
                fnv(&mut acc, &[*v]);
            }
        }
    }

    acc
}

// ════════════════════════════════════════════════════════════════════════════
// A1 — encode-time arm tagging: MemorySystem stores the classified cascade arm.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_fix_d_a1_encode_stores_classified_arm() {
    // Type A (plan Assertion 2). Drive an encode of a representative event of
    // EACH of the six cascade classes and read back the stored
    // `MemoryEntry.arm`. Every stored arm is compared against a HARDCODED
    // literal expectation written into this test body — NOT a re-invocation of
    // the private `classify_event` (which would be circular: a wrong mapping
    // would match itself). A `classes_exercised` counter proves all six classes
    // actually fired, so a scenario that silently skipped, say, Combat cannot
    // pass vacuously. Finally, an anti-recursion event
    // (AgentDecision{MemoryReason}) and a non-actor event (BuildingPlaced) must
    // produce ZERO biased entries (classify → None → unencoded).
    let hunger = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
        id,
        parent: None,
        agent: aid,
        position: (10, 10),
        reason: DecisionReason::HungerThresholdBreach,
        tick,
    })
    .expect("A1: a Hunger-class event must encode for its actor");
    let thirst = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
        id,
        parent: None,
        agent: aid,
        position: (10, 10),
        reason: DecisionReason::ThirstThresholdBreach,
        tick,
    })
    .expect("A1: a Thirst-class event must encode for its actor");
    let fatigue = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
        id,
        parent: None,
        agent: aid,
        position: (10, 10),
        reason: DecisionReason::FatigueThresholdBreach,
        tick,
    })
    .expect("A1: a Fatigue-class event must encode for its actor");
    let construction = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
        id,
        parent: None,
        agent: aid,
        position: (10, 10),
        reason: DecisionReason::ConstructionReason,
        tick,
    })
    .expect("A1: a Construction-class event must encode for its actor");
    let social = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
        id,
        parent: None,
        agent: aid,
        position: (10, 10),
        reason: DecisionReason::SocialReason,
        tick,
    })
    .expect("A1: a Social-class event must encode for its actor");
    let combat = encode_event_arm(|id, aid, tick| CausalEvent::CombatCompleted {
        id,
        parent: None,
        attacker: aid,
        defender: aid.saturating_add(1),
        position: (10, 10),
        hp_after: 90.0,
        settlement_link: None,
        tick,
    })
    .expect("A1: a Combat-class event must encode for its actor");

    // HARDCODED expected (class → arm) table — literals, never `classify_event`.
    let table: [(&str, MemoryArm, MemoryArm); 6] = [
        ("Hunger", MemoryArm::Hunger, hunger),
        ("Thirst", MemoryArm::Thirst, thirst),
        ("Fatigue", MemoryArm::Fatigue, fatigue),
        ("Construction", MemoryArm::Construction, construction),
        ("Social", MemoryArm::Social, social),
        ("Combat", MemoryArm::Combat, combat),
    ];
    let mut classes_exercised = 0u32;
    let mut arm_mismatch_count = 0u32;
    for (label, expected, got) in table {
        classes_exercised += 1;
        if got != expected {
            arm_mismatch_count += 1;
        }
        assert_eq!(
            got, expected,
            "A1: a {label}-class event must encode with {expected:?} (got {got:?})"
        );
    }
    assert_eq!(arm_mismatch_count, 0, "A1: arm_mismatch_count must be 0");
    assert_eq!(
        classes_exercised, 6,
        "A1: all six cascade classes must be exercised (coverage_precondition); got {classes_exercised}"
    );

    // Anti-recursion (MemoryReason) + non-actor (BuildingPlaced) → no biased
    // entry (classify_event returns None → MemorySystem never encodes them).
    let anti_recursion = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
        id,
        parent: None,
        agent: aid,
        position: (10, 10),
        reason: DecisionReason::MemoryReason,
        tick,
    });
    assert_eq!(
        anti_recursion, None,
        "A1: an anti-recursion AgentDecision{{MemoryReason}} must produce ZERO biased entries"
    );
    let non_actor = encode_event_arm(|id, _aid, tick| CausalEvent::BuildingPlaced {
        id,
        parent: None,
        position: (10, 10),
        radius: 1,
        tick,
    });
    assert_eq!(
        non_actor, None,
        "A1: a non-actor BuildingPlaced event must produce ZERO biased entries"
    );

    println!(
        "[fix-d A1] 6/6 cascade classes encode the correct MemoryArm; anti-recursion + non-actor unencoded ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A2 — full production-scarcity scene is DETERMINISTIC under a per-tick
//      behavioural lockstep across two independent in-process runs.
//
// Two `finite_scene()` engines are built fresh (so their internal HashMaps draw
// independent `RandomState` seeds — the cross-run nondeterminism condition) and
// ticked in lockstep. After every tick both behavioural fingerprints must match.
// Cap 2200 spans every divergence tick the checkpoint observed (1802 / 2163 /
// 2491-ε); before Fix D the runs diverged at ~1802.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_fix_d_a2_scarcity_scene_lockstep_deterministic() {
    const CAP: u64 = 2200;

    let mut a = finite_scene();
    let mut b = finite_scene();

    // ── INVALID precondition (the load-bearing anti-gaming guard) ──────────
    // The 0-mismatch result below is only meaningful if the two engines were
    // built with DISTINCT HashMap `RandomState` seeds. We assert this DIRECTLY
    // off each engine's real `food_tiles` hasher (the construction path's
    // `RandomState::new()`), NOT by sampling iteration order. If the seeds
    // coincided, both engines would iterate every HashMap in the same order and
    // a HashMap-order-dependent bias bug would pass silently — proving nothing
    // about the exact failure mode Fix D exists to kill.
    assert!(
        engines_independently_seeded(&a, &b),
        "A2 INVALID: the two engines were built with the SAME RandomState seed, so a 0-mismatch \
         lockstep would be MEANINGLESS (an order-dependent bias bug would pass silently). The \
         determinism contract requires independently-seeded engines; this is a test-environment \
         fault, not a Fix D pass."
    );

    let mut first_divergence: Option<u64> = None;
    for t in 1..=CAP {
        a.tick();
        b.tick();
        let fa = behavioural_fingerprint(&a);
        let fb = behavioural_fingerprint(&b);
        if fa != fb {
            first_divergence = Some(t);
            break;
        }
    }

    if let Some(t) = first_divergence {
        // Dump the observable diff for diagnosis.
        let da = {
            let mut s: Vec<AgentId> = Vec::new();
            for (_t, log) in a.resources.causal_log.iter() {
                for ev in log.iter() {
                    if let CausalEvent::AgentDied { agent, .. } = ev {
                        s.push(*agent);
                    }
                }
            }
            s.sort_unstable();
            s
        };
        panic!(
            "A2: behavioural divergence at tick {t} (cap {CAP}) — Fix D did not \
             restore determinism. run A dead-so-far={da:?}. The memory-arm decoupling \
             is insufficient; another entropy source remains (instrument tick {t})."
        );
    }

    println!(
        "[fix-d A2] {CAP} ticks lockstep: behaviourally IDENTICAL across two \
         independent-RandomState runs — determinism restored ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A3 — the cascade decision is RELATIVE, not absolute (plan Assertion 13).
//
// A uniform additive constant applied to EVERY arm's stored bias must leave the
// flip-tick INVARIANT, because the production flip condition is the relative
// comparison `delta_target > BIAS_FLIP_THRESHOLD + delta_natural`
// (agent_decision.rs) — adding the same constant to `delta_target` and
// `delta_natural` cancels. The baseline flip-tick is additionally anchored to a
// hardcoded literal so a regression that reads bias as an absolute magnitude is
// caught.
//
// Scenario: a single Idle agent with Hunger AND Thirst both above threshold (50)
// — natural winner = Hunger (priority 0). Two synthetic Thirst-arm memory
// entries (valence 1.0, salience 1.0, encoded_tick 0) bias the cascade toward
// Thirst, with `delta_Thirst = 2 * recency_factor(0, t)`. Driving ONLY
// `AgentDecisionSystem` (no MemorySystem) keeps salience pinned at 1.0, so only
// the linear recency factor decays with the current tick:
//
//   flip while  2 * (1 - t / MAX_RECENCY_TICKS) > 1.0   ⇔   t < MAX/2 = 2190.
//   flip-tick (first revert from Seeking{Water} to natural Seeking{Food}) = 2190.
//
// The uniform-shift run adds one entry per arm (identical valence/salience/
// encoded_tick), so every arm's delta gains the SAME +0.5*recency each tick —
// which cancels in the relative comparison, leaving the flip-tick unchanged.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_fix_d_a3_cascade_flip_is_relative_uniform_shift_invariant() {
    // Locked expected baseline flip-tick, measured against THIS commit at the
    // locked levers: MAX_RECENCY_TICKS (4380) / 2 == 2190. Pinned to the
    // INTEGER flip-tick (never the f64 bias sum) to avoid float `==` fragility.
    const EXPECTED_FLIP_TICK: u64 = MAX_RECENCY_TICKS / 2;
    // Scan horizon: the bias hits 0 at MAX_RECENCY_TICKS, well past the flip.
    const HORIZON: u64 = MAX_RECENCY_TICKS;

    // Drive a controlled single-agent cascade and return the integer tick at
    // which the chosen Seeking arm flips from the memory-biased target (Water)
    // back to the natural winner (Food). `uniform_shift` adds one identical
    // bias entry per arm (the uniform additive shift over every arm).
    fn flip_tick(uniform_shift: bool) -> u64 {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        let ent = e.spawn_agent(20, 20);
        e.world.insert_one(ent, Memory::new()).expect("attach Memory");
        {
            let mut mem = e.world.get::<&mut Memory>(ent).expect("memory present");
            // Two Thirst-arm bias entries → delta_Thirst = 2 * recency.
            mem.insert(MemoryEntry::new(1, 0, 1.0, 1.0, MemoryArm::Thirst));
            mem.insert(MemoryEntry::new(2, 0, 1.0, 1.0, MemoryArm::Thirst));
            if uniform_shift {
                // One entry per arm, identical (valence, salience, encoded_tick)
                // → each arm's delta gains the SAME +0.5*recency each tick.
                for (i, arm) in [
                    MemoryArm::Hunger,
                    MemoryArm::Thirst,
                    MemoryArm::Fatigue,
                    MemoryArm::Construction,
                    MemoryArm::Social,
                    MemoryArm::Combat,
                ]
                .into_iter()
                .enumerate()
                {
                    mem.insert(MemoryEntry::new(100 + i as EventId, 0, 0.5, 1.0, arm));
                }
            }
        }
        let mut sys = AgentDecisionSystem::new();
        let mut flip: Option<u64> = None;
        for t in 0..HORIZON {
            e.resources.current_tick = t;
            // Clean per-tick cascade evaluation: reset to Idle with both needs
            // pinned above threshold (Hunger natural winner, Thirst eligible
            // flip candidate). Memory persists (not in this component bag).
            e.world
                .insert(
                    ent,
                    (
                        AgentState::Idle,
                        Hunger::new(80.0, 0.0),
                        Thirst::new(80.0, 0.0),
                    ),
                )
                .expect("repin agent state + needs");
            sys.tick(&mut e.world, &mut e.resources);
            let st = e
                .world
                .get::<&AgentState>(ent)
                .map(|s| *s)
                .unwrap_or(AgentState::Idle);
            // Flipped ⇒ Seeking{Water}; reverted to natural ⇒ Seeking{Food}.
            if matches!(st, AgentState::Seeking { target: TargetKind::Food }) {
                flip = Some(t);
                break;
            }
        }
        flip.expect("A3: the cascade must eventually revert to the natural Food arm")
    }

    let baseline = flip_tick(false);
    let shifted = flip_tick(true);
    println!(
        "[fix-d A3] flip_tick baseline={baseline} uniform_shift={shifted} expected={EXPECTED_FLIP_TICK}"
    );
    assert_eq!(
        shifted, baseline,
        "A3: a uniform additive bias shift across every arm must leave the flip-tick INVARIANT \
         (the cascade compares biases relatively); baseline={baseline} shifted={shifted}"
    );
    assert_eq!(
        baseline, EXPECTED_FLIP_TICK,
        "A3: baseline flip-tick must equal the locked expected {EXPECTED_FLIP_TICK} \
         (MAX_RECENCY_TICKS/2)"
    );
}
