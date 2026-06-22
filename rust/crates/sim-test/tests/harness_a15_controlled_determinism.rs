//! a15-determinism-fix — CONTROLLED forced-divergence determinism suite
//! (plan attempt 2, A1–A7).
//!
//! Root bug (Section 1): `AgentDecisionSystem` iterated the `settlement_migrant_tag`
//! HashSet to apply a STRUCTURAL `world.insert_one(SettlementMigrant)`. A component
//! insert moves the entity into the `SettlementMigrant` archetype whose dense-array
//! order IS the insertion order, so the `RandomState`-seed-dependent HashSet
//! iteration permuted hecs query order → a contended gather split diverged →
//! a need value diverged (the a15 tick-2251 fingerprint divergence). The shipped
//! fix sorts the migrant entities by the stable `AgentId` before inserting, so the
//! structural insert order — and thus all downstream query iteration — is
//! seed-independent.
//!
//! MEASURED reality (this harness's design driver): the existing process-seeded
//! a15 pair is a COIN FLIP without the fix — it diverged on 2 of 3 fresh runs and
//! converged by seed-luck on the 3rd. A SINGLE process-seeded pair therefore
//! cannot reliably catch the bug (the Challenger's concern). This suite removes
//! the luck two ways:
//!   1. Assertion 2 establishes a DETERMINISTIC, reproducible-run-to-run different
//!      INITIAL yielded iteration order (a controlled permutation of the agent
//!      collection via a test-only marker insert+remove), and asserts that the two
//!      engines genuinely start from different orders (anti-vacuous guard).
//!   2. Assertion 5 runs MANY controlled-divergent pairs in one invocation; with
//!      the fix every pair locksteps, and without the fix the probability that ALL
//!      pairs converge by seed-luck is (≈1/3)^N → negligible, so the SUITE reliably
//!      fails when the fix is absent.
//!
//! Assertion ↔ threshold-type map (per FINAL plan attempt 2):
//!   A1 controlled forced-divergence reproducer — per-tick lockstep ... Type A
//!   A2 divergent-ordering precondition observed & holds (anti-vacuous) Type A
//!   A3 process-seeded full-horizon lockstep (existing contract) ...... Type A
//!   A4 end-state observables identical across runs .................. Type A
//!   A5 robustness across >= 3 controlled-divergent pairs ............ Type A
//!   A6 comparison window non-vacuous (liveness + activity floors) ... Type C
//!   A7 baseline registries mutually consistent, no stale-mask ....... Type A
//!
//! Run:
//!   cargo test -p sim-test --test harness_a15_controlled_determinism -- --nocapture

use std::collections::BTreeSet;

use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, seed_finite_resource_scarcity};
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Position, Sleep, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_default_runtime_systems;

const W: u32 = 64;
const H: u32 = 64;
/// Full-horizon length — matches the existing a15 horizon and spans the
/// documented 9 → 489 → 2251 divergence chain.
const RUN_TICKS: u64 = 5000;
/// Reduced horizon for the additional A5 pairs — still strictly past the
/// documented tick-2251 divergence point (plan: ">= 2300").
const SHORT_TICKS: u64 = 2300;
/// Number of distinct controlled-divergent pairs A5 exercises. The plan locks
/// ">= 3"; this suite uses 8 to drive the no-fix false-pass probability to
/// (≈1/3)^8 ≈ 1.5e-4 so the SUITE reliably fails when the fix is absent.
const NUM_PAIRS: usize = 8;

/// Test-only zero-sized marker used purely to migrate an agent out of and back
/// into the base archetype, re-appending it in a chosen order. NEVER read by any
/// production system, so it cannot perturb simulation VALUES — only the hecs
/// dense-array (query iteration) order the decision system observes.
struct OrderPerm;

// ── scene / engine construction ───────────────────────────────────────────────

/// Production-EQUIVALENT scene WITH finite scarcity (mirrors
/// `harness_resource_scarcity::finite_scene`): default runtime systems +
/// bootstrap agents + finite seeding + the 3 startup buildings.
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

// ── iteration-order control + observation ─────────────────────────────────────

/// The ordered `AgentId` sequence the world currently YIELDS over the agent
/// collection — the raw hecs query order the decision system processes.
fn yielded_agent_order(e: &SimEngine) -> Vec<AgentId> {
    e.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect()
}

/// Apply a deterministic permutation `perm` to the engine's base agent
/// archetype so its yielded iteration order changes (membership unchanged).
/// Reorders by insert+remove of the test-only `OrderPerm` marker: each insert
/// migrates the entity to the `+OrderPerm` archetype and the remove migrates it
/// back, RE-APPENDED at the dense-array tail. Processing the agents in `perm`
/// order therefore leaves the base archetype yielding in `perm` order.
fn permute_agent_order(e: &mut SimEngine, perm: impl Fn(&[hecs::Entity]) -> Vec<hecs::Entity>) {
    let cur: Vec<hecs::Entity> = e.world.query::<&Agent>().iter().map(|(ent, _)| ent).collect();
    for ent in perm(&cur) {
        let _ = e.world.insert_one(ent, OrderPerm);
        let _ = e.world.remove_one::<OrderPerm>(ent);
    }
}

/// Reverse permutation (guaranteed != identity for n >= 2).
fn perm_reverse(cur: &[hecs::Entity]) -> Vec<hecs::Entity> {
    let mut v = cur.to_vec();
    v.reverse();
    v
}

/// Rotate-left-by-`k` permutation (guaranteed != identity for 0 < k < n).
fn perm_rotate(cur: &[hecs::Entity], k: usize) -> Vec<hecs::Entity> {
    let mut v = cur.to_vec();
    let n = v.len();
    if n > 0 {
        v.rotate_left(k % n);
    }
    v
}

// ── behavioural fingerprint (matches the existing a15 fingerprint) ────────────

fn fnv(acc: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *acc ^= b as u64;
        *acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

/// Behaviourally-observable fingerprint: per-agent position + `AgentState` +
/// needs (bit-exact) + `BodyHealth`, the dead-agent set, the per-reason death
/// split, and the resource-tile key-sets WITH values. EXCLUDES `event_id`.
/// Rows are sorted by `AgentId`, so the fingerprint compares simulation STATE,
/// not iteration order — it cannot pass vacuously by re-sorting.
fn behavioural_fingerprint(e: &SimEngine) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;

    let ents: Vec<hecs::Entity> = e.world.query::<&Agent>().iter().map(|(ent, _)| ent).collect();
    let mut rows: Vec<(AgentId, u64, u64, String)> = Vec::new();
    for ent in ents {
        let Ok(agent) = e.world.get::<&Agent>(ent) else { continue };
        let pos_bits = e
            .world
            .get::<&Position>(ent)
            .map(|p| ((p.x as u64) << 32) | p.y as u64)
            .unwrap_or(0);
        let hunger = e.world.get::<&Hunger>(ent).map(|h| (h.value as f64).to_bits()).unwrap_or(0);
        let thirst = e.world.get::<&Thirst>(ent).map(|t| t.value.to_bits()).unwrap_or(0);
        let sleep = e.world.get::<&Sleep>(ent).map(|s| s.fatigue.to_bits()).unwrap_or(0);
        let hp = e.world.get::<&BodyHealth>(ent).map(|b| b.hp.to_bits()).unwrap_or(0);
        let state = e.world.get::<&AgentState>(ent).map(|s| format!("{s:?}")).unwrap_or_default();
        let need_mix = hunger
            .wrapping_mul(0x9E37_79B9)
            ^ thirst.rotate_left(17)
            ^ sleep.rotate_left(31)
            ^ hp.rotate_left(7);
        rows.push((agent.id, pos_bits, need_mix, state));
    }
    rows.sort_by_key(|(id, ..)| *id);
    for (id, pos_bits, need_mix, state) in &rows {
        fnv(&mut acc, &id.to_le_bytes());
        fnv(&mut acc, &pos_bits.to_le_bytes());
        fnv(&mut acc, &need_mix.to_le_bytes());
        fnv(&mut acc, state.as_bytes());
        fnv(&mut acc, b"|");
    }

    let (starv, dehy, comb) = per_reason_split(e);
    let dead = dead_set(e);
    fnv(&mut acc, b"#dead");
    for id in &dead {
        fnv(&mut acc, &id.to_le_bytes());
    }
    fnv(&mut acc, &(starv as u64).to_le_bytes());
    fnv(&mut acc, &(dehy as u64).to_le_bytes());
    fnv(&mut acc, &(comb as u64).to_le_bytes());

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
            if let Some(v) = map.get(&(x, y)) {
                fnv(&mut acc, &[*v]);
            }
        }
    }

    acc
}

// ── observable scanners (read-only over the shipped causal log) ───────────────

fn dead_set(e: &SimEngine) -> BTreeSet<AgentId> {
    let mut s = BTreeSet::new();
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, .. } = ev {
                s.insert(*agent);
            }
        }
    }
    s
}

fn scarcity_dead_set(e: &SimEngine) -> BTreeSet<AgentId> {
    let mut s = BTreeSet::new();
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, reason, .. } = ev {
                if matches!(reason, DeathReason::Starvation | DeathReason::Dehydration) {
                    s.insert(*agent);
                }
            }
        }
    }
    s
}

/// `(starvation, dehydration, combat)` — `DeathReason` is exhaustively matched
/// so a future variant fails to compile here.
fn per_reason_split(e: &SimEngine) -> (usize, usize, usize) {
    let (mut starvation, mut dehydration, mut combat) = (0usize, 0usize, 0usize);
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { reason, .. } = ev {
                match reason {
                    DeathReason::Starvation => starvation += 1,
                    DeathReason::Dehydration => dehydration += 1,
                    DeathReason::Combat => combat += 1,
                }
            }
        }
    }
    (starvation, dehydration, combat)
}

fn settlement_total_deaths(e: &SimEngine) -> u32 {
    e.resources.settlements.values().map(|s| s.population_stats.total_deaths).sum()
}

fn live_count(e: &SimEngine) -> usize {
    e.world.query::<&Agent>().iter().count()
}

/// End-state observable bundle for A4 equality checks.
#[derive(Debug, PartialEq, Eq)]
struct EndState {
    dead: BTreeSet<AgentId>,
    scarcity_dead: BTreeSet<AgentId>,
    per_reason: (usize, usize, usize),
    total_deaths: u32,
    live: usize,
}

fn end_state(e: &SimEngine) -> EndState {
    EndState {
        dead: dead_set(e),
        scarcity_dead: scarcity_dead_set(e),
        per_reason: per_reason_split(e),
        total_deaths: settlement_total_deaths(e),
        live: live_count(e),
    }
}

/// Run a controlled-divergent pair: A is un-permuted, B is permuted by `perm`.
/// Asserts the A2 precondition (orders genuinely differ) up front, then ticks
/// both `ticks` ticks in lockstep, returning the first behavioural divergence
/// tick (None == full lockstep) plus the post-run end-states for A4.
fn run_controlled_pair(
    label: &str,
    perm: impl Fn(&[hecs::Entity]) -> Vec<hecs::Entity>,
    ticks: u64,
) -> (Option<u64>, EndState, EndState) {
    let mut a = finite_scene();
    let mut b = finite_scene();
    permute_agent_order(&mut b, perm);

    // ── A2 precondition: the two captured initial orders MUST differ. ──
    let order_a = yielded_agent_order(&a);
    let order_b = yielded_agent_order(&b);
    assert!(
        order_a != order_b,
        "[{label}] A2 INVALID: the controlled permutation did NOT change the yielded iteration \
         order (both engines start identically ordered) — a 0-mismatch lockstep would be \
         VACUOUS. order_a.len()={}, order_b.len()={}",
        order_a.len(),
        order_b.len(),
    );
    assert_eq!(
        order_a.iter().copied().collect::<BTreeSet<_>>(),
        order_b.iter().copied().collect::<BTreeSet<_>>(),
        "[{label}] A2: permutation must preserve MEMBERSHIP (same agent set, different order)",
    );

    let mut first_divergence: Option<u64> = None;
    for t in 1..=ticks {
        a.tick();
        b.tick();
        if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
            first_divergence = Some(t);
            break;
        }
    }
    (first_divergence, end_state(&a), end_state(&b))
}

// ════════════════════════════════════════════════════════════════════════════
// A1 + A2 + A4 + A6 — primary controlled forced-divergence pair (reverse order).
// One 5000-tick run anchors per-tick lockstep (A1), the anti-vacuous order
// precondition (A2), end-state equality (A4, this pair), and the non-vacuity
// floors (A6). Sharing the run keeps the suite cheap.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_a15_controlled_forced_divergence_lockstep() {
    // A6 non-vacuity is measured on the un-permuted twin A across the window.
    let mut a = finite_scene();
    let mut b = finite_scene();
    permute_agent_order(&mut b, perm_reverse);

    // ── A2: controlled-divergence precondition is OBSERVED and holds. ──
    let order_a = yielded_agent_order(&a);
    let order_b = yielded_agent_order(&b);
    assert!(
        order_a != order_b,
        "A2 INVALID: reverse permutation did not change the yielded iteration order — \
         a 0-mismatch lockstep would be VACUOUS"
    );
    assert_eq!(
        order_a.iter().copied().collect::<BTreeSet<_>>(),
        order_b.iter().copied().collect::<BTreeSet<_>>(),
        "A2: reverse permutation must preserve membership"
    );

    // A6 activity counters (on A, the un-permuted reference trajectory).
    let deaths_before = dead_set(&a).len();
    let mut prev_states: std::collections::HashMap<AgentId, String> = state_map(&a);
    let mut transitions: usize = 0;

    // ── A1: per-tick behavioural lockstep across the full horizon. ──
    let mut first_divergence: Option<u64> = None;
    for t in 1..=RUN_TICKS {
        a.tick();
        b.tick();
        // accumulate A6 state-transition count on A
        let now = state_map(&a);
        for (id, st) in &now {
            if prev_states.get(id).map(|p| p != st).unwrap_or(false) {
                transitions += 1;
            }
        }
        prev_states = now;
        if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
            first_divergence = Some(t);
            break;
        }
    }
    assert!(
        first_divergence.is_none(),
        "A1: behavioural divergence at tick {first_divergence:?} of {RUN_TICKS} under a CONTROLLED \
         reverse-permuted starting order — the determinism fix did not restore full-horizon \
         lockstep (run A dead-so-far={:?})",
        dead_set(&a),
    );

    // ── A4 (this pair): end-state observables identical. ──
    let (es_a, es_b) = (end_state(&a), end_state(&b));
    assert_eq!(es_a, es_b, "A4: end-state observables must be identical across the controlled pair");

    // ── A6: comparison window was non-vacuous. ──
    let deaths_in_window = dead_set(&a).len() - deaths_before;
    let live_final = live_count(&a);
    assert!(
        live_final >= 3,
        "A6(a): live_count at the final compared tick must be >= 3 (got {live_final}) — \
         a permutation needs >= 2 elements; >= 3 is the conservative non-vacuity floor"
    );
    assert!(
        deaths_in_window >= 1,
        "A6(b): >= 1 death must occur in the compared window (got {deaths_in_window}) — \
         the scarcity scene must actually exercise the order-sensitive death path"
    );
    assert!(
        transitions >= 1,
        "A6(c): >= 1 AgentState transition must occur in the compared window (got {transitions})"
    );

    println!(
        "[a15 A1/A2/A4/A6] reverse-permuted pair: full {RUN_TICKS}-tick lockstep (orders differ at \
         t0) — live_final={live_final} deaths_in_window={deaths_in_window} transitions={transitions} \
         end_state(dead={} scarcity={} per_reason={:?} td={} live={})",
        es_a.dead.len(),
        es_a.scarcity_dead.len(),
        es_a.per_reason,
        es_a.total_deaths,
        es_a.live,
    );
}

/// Per-agent `AgentId -> AgentState` debug-string map (A6 transition counter).
fn state_map(e: &SimEngine) -> std::collections::HashMap<AgentId, String> {
    let mut m = std::collections::HashMap::new();
    for (_, (agent, state)) in e.world.query::<(&Agent, &AgentState)>().iter() {
        m.insert(agent.id, format!("{state:?}"));
    }
    m
}

// ════════════════════════════════════════════════════════════════════════════
// A3 + A4 — process-seeded full-horizon lockstep (existing a15 contract).
// Two finite scenes built the way the existing a15 test builds them (no
// controlled permutation — independent process-random internal seeds).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_a15_process_seeded_lockstep() {
    let mut a = finite_scene();
    let mut b = finite_scene();

    let mut first_divergence: Option<u64> = None;
    for t in 1..=RUN_TICKS {
        a.tick();
        b.tick();
        if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
            first_divergence = Some(t);
            break;
        }
    }
    assert!(
        first_divergence.is_none(),
        "A3: process-seeded behavioural divergence at tick {first_divergence:?} of {RUN_TICKS} — \
         the fix did not hold under the existing a15 conditions (run A dead-so-far={:?})",
        dead_set(&a),
    );

    // ── A4 (process-seeded pair): end-state observables identical. ──
    let (es_a, es_b) = (end_state(&a), end_state(&b));
    assert_eq!(es_a, es_b, "A4: end-state observables must be identical across the process-seeded pair");

    println!(
        "[a15 A3/A4] process-seeded pair: full {RUN_TICKS}-tick lockstep — \
         end_state(dead={} scarcity={} per_reason={:?} td={} live={})",
        es_a.dead.len(),
        es_a.scarcity_dead.len(),
        es_a.per_reason,
        es_a.total_deaths,
        es_a.live,
    );
}

// ════════════════════════════════════════════════════════════════════════════
// A5 — robustness across MULTIPLE distinct controlled-divergent pairs.
// Pair 0 runs the full horizon; pairs 1..NUM_PAIRS run >= 2300 ticks (past the
// documented tick-2251 divergence). Each pair must lockstep AND pass the A2
// precondition. With the fix every pair locksteps; without it, the chance ALL
// pairs converge by seed-luck is (≈1/3)^NUM_PAIRS → negligible, so this is the
// guard that makes the SUITE reliably fail when the fix is absent.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_a15_multi_permutation_robustness() {
    // NUM_PAIRS distinct, deterministic permutations: reverse, then rotate-left
    // by 1..=NUM_PAIRS-1 (all non-identity, all distinct from each other for the
    // 64-agent bootstrap lattice).
    for i in 0..NUM_PAIRS {
        let ticks = if i == 0 { RUN_TICKS } else { SHORT_TICKS };
        let (label, div, es_a, es_b): (String, Option<u64>, EndState, EndState) = if i == 0 {
            let (d, a, b) = run_controlled_pair("pair0/reverse", perm_reverse, ticks);
            ("pair0/reverse".to_string(), d, a, b)
        } else {
            let k = i; // rotate-left by k (1..=NUM_PAIRS-1)
            let lbl = format!("pair{i}/rotate{k}");
            let (d, a, b) = run_controlled_pair(&lbl, move |cur| perm_rotate(cur, k), ticks);
            (lbl, d, a, b)
        };
        assert!(
            div.is_none(),
            "A5[{label}]: behavioural divergence at tick {div:?} of {ticks} under a controlled \
             divergent starting order — the fix did not restore lockstep for this permutation"
        );
        // Aggregate end-state must also match per pair (subset of A4 across pairs).
        assert_eq!(es_a, es_b, "A5[{label}]: end-state observables must match within the pair");
        println!("[a15 A5] {label}: {ticks}-tick lockstep (orders differ at t0) ✓");
    }
    println!("[a15 A5] all {NUM_PAIRS} controlled-divergent pairs locksteppped ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// A7 — baseline registries are mutually consistent and do not stale-mask a15.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_a15_baseline_registry_consistency() {
    // Repo root = three levels up from this crate (sim-test -> crates -> rust -> repo).
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let known_path = root.join(".harness/baseline/known_failures.txt");
    let count_path = root.join(".harness/baseline_test_failures.txt");

    let known = std::fs::read_to_string(&known_path)
        .unwrap_or_else(|e| panic!("A7: cannot read {}: {e}", known_path.display()));
    let count_raw = std::fs::read_to_string(&count_path)
        .unwrap_or_else(|e| panic!("A7: cannot read {}: {e}", count_path.display()));

    // Active entry = non-comment, non-blank line; first whitespace token is the
    // test path (the regression guard reads exactly that token).
    let active: Vec<&str> = known
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let active_count = active.len();

    let declared: usize = count_raw
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("A7: baseline_test_failures.txt is not an integer: {e}"));

    assert_eq!(
        declared, active_count,
        "A7: baseline_test_failures.txt ({declared}) must EQUAL the active-entry count of \
         known_failures.txt ({active_count}) — the regression guard set-differences against this \
         list, so a count mismatch corrupts it. Active entries: {active:?}"
    );

    let a15_present = active
        .iter()
        .any(|l| l.split_whitespace().next() == Some("harness_scarcity_a15_determinism_of_scarcity_run"));
    assert!(
        !a15_present,
        "A7: harness_scarcity_a15_determinism_of_scarcity_run must NOT be an active baseline entry \
         (this ticket makes it pass; leaving it registered would mask a future real regression)"
    );

    println!(
        "[a15 A7] baseline consistent: declared={declared} active={active_count}, a15 absent ✓"
    );
}
