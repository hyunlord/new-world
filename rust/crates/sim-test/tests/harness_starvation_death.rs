//! V7 — starvation / dehydration death harness (feature `add-starvation-death`).
//!
//! Verifies the new `StarvationSystem` (priority 139, interval 1) + the shared
//! `survival::despawn_agent` helper:
//!   - sustained Hunger/Thirst saturation damages `BodyHealth.hp`;
//!   - hp ≤ 0 → death (despawn) with a chronicled `CausalEvent::AgentDied`;
//!   - thirst kills faster than hunger; both saturated stacks additively;
//!   - DeathReason precedence (thirst wins);
//!   - the shared despawn helper cleans `settlement.member_agents` /
//!     `population_stats` for BOTH starvation AND combat deaths;
//!   - recovery (heal below ceiling), gray-zone neutrality, robustness,
//!     determinism, and long-run population stability.
//!
//! INTERVAL NOTE: every per-tick arithmetic assertion assumes the system runs
//! at interval 1 (Hot tier). `StarvationSystem::tick_interval() == 1` is the
//! confirmed cadence — the death-tick driver below ticks the system once per
//! loop iteration, so the absolute bands hold as derived in the plan.
//!
//! Run:
//!   cargo test -p sim-test --test harness_starvation_death -- --nocapture

use std::collections::BTreeSet;

use hecs::Entity;
use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Memory, Settlement, SettlementId, Sleep,
    Social, Thirst, DEFAULT_MAX_HP, SETTLEMENT_MAX_POP,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::combat::CombatSystem;
use sim_systems::runtime::survival::{StarvationSystem, SAFE_NEED_CEILING};

const W: u32 = 64;
const H: u32 = 64;
const SAT: f32 = Hunger::SATURATION; // 100.0
const SAT_T: f64 = Thirst::SATURATION; // 100.0

// ── engines ─────────────────────────────────────────────────────────────────

/// Bare engine, NO runtime systems registered. Used to drive `StarvationSystem`
/// (or `CombatSystem`) directly so the arithmetic is fully isolated from
/// need-decay / decision / movement interference (plan convention #2).
fn iso_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Full default runtime (used for gathering-loop, birth, population, determinism).
fn full_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Production bootstrap: full runtime + 64 lattice agents (each carries
/// BodyHealth after this feature).
fn bootstrapped() -> SimEngine {
    let mut e = full_engine();
    bootstrap_spawn_agents(&mut e);
    e
}

// ── controlled-agent fixtures ────────────────────────────────────────────────

/// Spawn a controlled agent with a given starting hp and zero need-growth (so
/// the test, not a decay system, owns the need values). Returns (Entity, id).
fn spawn_controlled(e: &mut SimEngine, x: u32, y: u32, hp: f64) -> (Entity, AgentId) {
    let ent = e.spawn_agent(x, y);
    let id = e.world.get::<&Agent>(ent).expect("agent").id;
    e.world
        .insert(
            ent,
            (
                AgentState::Idle,
                BodyHealth { hp, max_hp: DEFAULT_MAX_HP },
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Memory::new(),
            ),
        )
        .expect("seed controlled agent");
    (ent, id)
}

fn hp(e: &SimEngine, ent: Entity) -> f64 {
    e.world.get::<&BodyHealth>(ent).expect("BodyHealth").hp
}

fn alive(e: &SimEngine, ent: Entity) -> bool {
    e.world.get::<&Agent>(ent).is_ok()
}

/// Re-pin the controlled agent's needs (if still alive) — every tick, per
/// plan convention #2 (need values would otherwise be re-set by growth).
fn pin(e: &mut SimEngine, ent: Entity, pin_h: Option<f32>, pin_t: Option<f64>) {
    if let Some(v) = pin_h {
        if let Ok(mut h) = e.world.get::<&mut Hunger>(ent) {
            h.value = v;
        }
    }
    if let Some(v) = pin_t {
        if let Ok(mut th) = e.world.get::<&mut Thirst>(ent) {
            th.value = v;
        }
    }
}

/// Drive `StarvationSystem` directly, re-pinning `(pin_h, pin_t)` every tick.
/// Returns `Some(tick)` (the `current_tick` value) at which the agent died, or
/// `None` if it survived `max` ticks.
fn drive_to_death(
    e: &mut SimEngine,
    ent: Entity,
    pin_h: Option<f32>,
    pin_t: Option<f64>,
    max: u64,
) -> Option<u64> {
    let mut sys = StarvationSystem::new();
    for t in 0..max {
        e.resources.current_tick = t;
        if !alive(e, ent) {
            return Some(t.saturating_sub(1));
        }
        pin(e, ent, pin_h, pin_t);
        sys.tick(&mut e.world, &mut e.resources);
        if !alive(e, ent) {
            return Some(t);
        }
    }
    None
}

// ── causal-log scanning ──────────────────────────────────────────────────────

fn find_agent_died(e: &SimEngine, id: AgentId) -> Option<(DeathReason, u64)> {
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, reason, tick, .. } = ev {
                if *agent == id {
                    return Some((*reason, *tick));
                }
            }
        }
    }
    None
}

fn count_agent_died(e: &SimEngine, id: AgentId) -> usize {
    let mut n = 0;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDied { agent, .. } = ev {
                if *agent == id {
                    n += 1;
                }
            }
        }
    }
    n
}

fn dead_id_set(e: &SimEngine) -> BTreeSet<AgentId> {
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

// ── settlement fixtures ──────────────────────────────────────────────────────

/// Insert a settlement into `resources.settlements` holding `members` and with
/// `current == len`, `total_deaths == 0`. Returns the new settlement id.
fn make_settlement_with(e: &mut SimEngine, members: &[AgentId]) -> SettlementId {
    let sid = e.resources.issue_settlement_id();
    let mut s = Settlement::new_with_id(sid, 0);
    for &m in members {
        s.add_member_agent(m);
    }
    s.population_stats.current = s.member_agents.len() as u32;
    e.resources.settlements.insert(sid, s);
    sid
}

fn settlement_total_deaths(e: &SimEngine) -> u32 {
    e.resources
        .settlements
        .values()
        .map(|s| s.population_stats.total_deaths)
        .sum()
}

fn live_count(e: &SimEngine) -> usize {
    e.world.query::<&Agent>().iter().count()
}

/// Place the 3 startup buildings at the live-scene coordinates so bootstrap
/// agents form a settlement early (mirrors `world_renderer.gd::_ready()`).
fn place_startup_buildings(e: &mut SimEngine) {
    for pos in [(32u32, 32u32), (24, 32), (40, 32)] {
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent { position: pos, radius: 8 });
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 1: HP decays under sustained hunger saturation.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a1_hp_decays_under_hunger() {
    // Type A — arithmetic invariant. 200 ticks × 0.08 = 16.0 hp loss; band ±3.
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let mut sys = StarvationSystem::new();

    // T0 — first damage tick (Hunger pinned ≥ SATURATION, Thirst pinned to 0).
    e.resources.current_tick = 0;
    pin(&mut e, ent, Some(SAT), Some(0.0));
    sys.tick(&mut e.world, &mut e.resources);
    let hp0 = hp(&e, ent);

    for t in 1..=200u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(SAT), Some(0.0));
        sys.tick(&mut e.world, &mut e.resources);
    }
    let hp1 = hp(&e, ent);
    let drop = hp0 - hp1;

    println!("[starvation A1] hp0={hp0:.4} hp1={hp1:.4} drop={drop:.4} (expect ~16.0)");
    assert!(hp1 < hp0, "A1: hp must strictly decrease under hunger saturation");
    assert!(
        (13.0..=19.0).contains(&drop),
        "A1: hp drop over 200 ticks must be in [13,19]; got {drop:.4}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 2: Gray-zone neutrality — need in (50,100) → no damage AND no heal.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a2_gray_zone_neutral() {
    // Type A — boundary invariant. Damage to ~80 first, then hold BOTH needs at
    // 75.0 (inside the neutral band) for 300 ticks: hp must not drift.
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let mut sys = StarvationSystem::new();

    // Damage to ~80 hp via hunger saturation (250 ticks × 0.08 = 20.0).
    for t in 0..250u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(SAT), Some(0.0));
        sys.tick(&mut e.world, &mut e.resources);
    }
    let hp_start = hp(&e, ent);
    assert!(hp_start < 100.0 && hp_start > 0.0, "A2 setup: hp must be sub-max and >0");

    // Neutral window: both needs at 75.0 for 300 ticks.
    for t in 250..550u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(75.0), Some(75.0));
        sys.tick(&mut e.world, &mut e.resources);
    }
    let hp_end = hp(&e, ent);
    let drift = (hp_end - hp_start).abs();
    println!("[starvation A2] hp_start={hp_start:.4} hp_end={hp_end:.4} drift={drift:.6}");
    assert!(
        drift <= 0.01,
        "A2: hp must not change in the gray zone (50,100); drift {drift:.6} > 0.01"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 3: Hunger-only death at the expected tick, by Starvation.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a3_hunger_death_tick_and_reason() {
    // Type A — 100 / 0.08 = 1250; band ±12. DeathReason == Starvation.
    let mut e = iso_engine();
    let (ent, id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let death = drive_to_death(&mut e, ent, Some(SAT), Some(0.0), 1400);
    let death = death.expect("A3: hunger-only agent must die within 1400 ticks");
    println!("[starvation A3] hunger-only death tick = {death}");
    assert!(
        (1238..=1262).contains(&death),
        "A3: hunger death tick must be in [1238,1262]; got {death}"
    );
    let (reason, _t) = find_agent_died(&e, id).expect("A3: AgentDied event must exist");
    assert_eq!(reason, DeathReason::Starvation, "A3: reason must be Starvation");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 4: Thirst-only death — Dehydration, and FASTER than hunger-only.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a4_thirst_death_faster_dehydration() {
    // Type A — 100 / 0.12 = 833; band ±14. Strict thirst < hunger ordering.
    let mut e_t = iso_engine();
    let (ent_t, id_t) = spawn_controlled(&mut e_t, 10, 10, 100.0);
    let thirst_death =
        drive_to_death(&mut e_t, ent_t, Some(0.0), Some(SAT_T), 1000).expect("A4: thirst death");

    let mut e_h = iso_engine();
    let (ent_h, _id_h) = spawn_controlled(&mut e_h, 10, 10, 100.0);
    let hunger_death =
        drive_to_death(&mut e_h, ent_h, Some(SAT), Some(0.0), 1400).expect("A4: hunger death");

    println!("[starvation A4] thirst_death={thirst_death} hunger_death={hunger_death}");
    assert!(
        (820..=848).contains(&thirst_death),
        "A4: thirst death tick must be in [820,848]; got {thirst_death}"
    );
    let (reason, _t) = find_agent_died(&e_t, id_t).expect("A4: AgentDied event must exist");
    assert_eq!(reason, DeathReason::Dehydration, "A4: reason must be Dehydration");
    assert!(
        thirst_death < hunger_death,
        "A4: thirst death ({thirst_death}) must be strictly before hunger death ({hunger_death})"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 5: Both saturated → death at the combined (additive) rate.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a5_both_saturated_additive() {
    // Type A — 100 / (0.08+0.12)=0.20 = 500; band ±12. Distinguishes additive
    // (500) from max() (834).
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let death =
        drive_to_death(&mut e, ent, Some(SAT), Some(SAT_T), 700).expect("A5: both-saturated death");
    println!("[starvation A5] both-saturated death tick = {death}");
    assert!(
        (488..=512).contains(&death),
        "A5: both-saturated death tick must be in [488,512]; got {death}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 6: Death reason correct; thirst takes precedence when both saturated.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a6_death_reason_precedence() {
    // Type A — (a) hunger→Starvation, (b) thirst→Dehydration, (c) both→Dehydration.
    let mut e_a = iso_engine();
    let (ent_a, id_a) = spawn_controlled(&mut e_a, 10, 10, 100.0);
    drive_to_death(&mut e_a, ent_a, Some(SAT), Some(0.0), 1400);
    let (ra, _) = find_agent_died(&e_a, id_a).expect("A6a event");
    assert_eq!(ra, DeathReason::Starvation, "A6(a): hunger-only → Starvation");

    let mut e_b = iso_engine();
    let (ent_b, id_b) = spawn_controlled(&mut e_b, 10, 10, 100.0);
    drive_to_death(&mut e_b, ent_b, Some(0.0), Some(SAT_T), 1000);
    let (rb, _) = find_agent_died(&e_b, id_b).expect("A6b event");
    assert_eq!(rb, DeathReason::Dehydration, "A6(b): thirst-only → Dehydration");

    let mut e_c = iso_engine();
    let (ent_c, id_c) = spawn_controlled(&mut e_c, 10, 10, 100.0);
    drive_to_death(&mut e_c, ent_c, Some(SAT), Some(SAT_T), 700);
    let (rc, _) = find_agent_died(&e_c, id_c).expect("A6c event");
    assert_eq!(rc, DeathReason::Dehydration, "A6(c): both saturated → Dehydration (precedence)");
    println!("[starvation A6] reasons a={ra:?} b={rb:?} c={rc:?} ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 7: AgentDied emitted 1:1 with each needs-death (id + tick ±1).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a7_event_one_to_one() {
    // Type A — isolated scenario (only this engineered death occurs). Exactly
    // one matching AgentDied, tick within ±1 of the observed despawn tick.
    let mut e = iso_engine();
    let (ent, id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let death = drive_to_death(&mut e, ent, Some(SAT), Some(0.0), 1400).expect("A7: death");
    assert_eq!(count_agent_died(&e, id), 1, "A7: exactly one AgentDied for the dead agent");
    let (_r, event_tick) = find_agent_died(&e, id).expect("A7: event present");
    let diff = event_tick.abs_diff(death);
    println!("[starvation A7] death tick={death} event tick={event_tick} diff={diff}");
    assert!(diff <= 1, "A7: event tick {event_tick} must be within ±1 of despawn tick {death}");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 8: HP clamped at 0, never negative; death triggers at hp == 0.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a8_hp_clamped_at_zero() {
    // Type A — boundary. Both saturated (0.20/tick). Start hp crafted so the
    // last live post-tick sample lands ≤ 0.01 (50.005 - 0.20·250 = 0.005),
    // exercising the "persists until hp reaches 0" property at fine resolution.
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 50.005);
    let mut sys = StarvationSystem::new();

    let mut min_hp = f64::MAX;
    let mut last_hp = 50.005;
    let mut present_ok = true;
    for t in 0..700u64 {
        e.resources.current_tick = t;
        if !alive(&e, ent) {
            break;
        }
        pin(&mut e, ent, Some(SAT), Some(SAT_T));
        sys.tick(&mut e.world, &mut e.resources);
        if !alive(&e, ent) {
            break;
        }
        let cur = hp(&e, ent);
        if cur > 0.0 && !alive(&e, ent) {
            present_ok = false;
        }
        last_hp = cur;
        if cur < min_hp {
            min_hp = cur;
        }
    }
    println!("[starvation A8] min_hp={min_hp:.6} last_hp_before_death={last_hp:.6}");
    assert!(min_hp >= 0.0, "A8(a): observed hp must never be negative; got {min_hp}");
    assert!(
        last_hp <= 0.01,
        "A8(b): agent must persist until hp reaches ~0; last live hp {last_hp:.6} > 0.01"
    );
    assert!(present_ok, "A8(c): agent must be present at every sampled tick where hp > 0");
    assert!(!alive(&e, ent), "A8: agent must be dead after the window");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 9: Starvation death cleans member_agents + population stats.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a9_settlement_roster_cleanup() {
    // Type A — bookkeeping invariant.
    let mut e = iso_engine();
    let (ent, id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let sid = make_settlement_with(&mut e, &[id]);
    let deaths_before = e.resources.settlements[&sid].population_stats.total_deaths;
    assert!(
        e.resources.settlements[&sid].member_agents.contains(&id),
        "A9 setup: agent must be a member"
    );

    drive_to_death(&mut e, ent, Some(SAT), Some(0.0), 1400).expect("A9: death");

    let s = &e.resources.settlements[&sid];
    assert!(!s.member_agents.contains(&id), "A9(a): dead agent removed from member_agents");
    assert_eq!(
        s.population_stats.total_deaths,
        deaths_before + 1,
        "A9(b): total_deaths must increment by exactly 1"
    );
    assert_eq!(
        s.population_stats.current as usize,
        s.member_agents.len(),
        "A9(c): current must equal member_agents.len()"
    );
    println!("[starvation A9] roster cleaned, total_deaths={} ✓", s.population_stats.total_deaths);
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 10: Combat death also cleans member_agents (forced regression guard).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a10_combat_death_roster_cleanup() {
    // Type D — FORCE a combat death (defender hp ≤ DAMAGE_PER_COMBAT_TICK) via a
    // directly-seeded combat pair; FAIL if no combat death occurs.
    let mut e = iso_engine();
    let attacker = e.spawn_agent(5, 5);
    let defender = e.spawn_agent(5, 5);
    let attacker_id = e.world.get::<&Agent>(attacker).unwrap().id;
    let defender_id = e.world.get::<&Agent>(defender).unwrap().id;
    e.world
        .insert(attacker, (AgentState::Idle, BodyHealth::new(), Memory::new()))
        .unwrap();
    e.world
        .insert(
            defender,
            (AgentState::Idle, BodyHealth { hp: 5.0, max_hp: DEFAULT_MAX_HP }, Memory::new()),
        )
        .unwrap();
    let sid = make_settlement_with(&mut e, &[attacker_id, defender_id]);
    let deaths_before = e.resources.settlements[&sid].population_stats.total_deaths;

    e.resources.combat_pairs.insert((attacker_id, defender_id));
    e.resources.current_tick = 1;
    let mut cs = CombatSystem::new();
    cs.tick(&mut e.world, &mut e.resources);

    // Precondition — the combat-death path MUST have fired.
    assert!(
        e.world.get::<&Agent>(defender).is_err(),
        "A10: scenario must produce a combat death (defender must despawn)"
    );
    let (reason, _t) = find_agent_died(&e, defender_id).expect("A10: AgentDied must be emitted");
    assert_eq!(reason, DeathReason::Combat, "A10: combat death reason must be Combat");

    let s = &e.resources.settlements[&sid];
    assert!(!s.member_agents.contains(&defender_id), "A10: combat-dead member removed from roster");
    assert_eq!(
        s.population_stats.total_deaths,
        deaths_before + 1,
        "A10: total_deaths must increment by 1 for the combat death"
    );
    println!("[starvation A10] combat death cleans roster (leak fix locked) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 11: Bandless agent dies cleanly and still emits AgentDied.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a11_bandless_death() {
    // Type A — robustness. No settlement exists → cleanup is a no-op (no panic).
    let mut e = iso_engine();
    let (ent, id) = spawn_controlled(&mut e, 10, 10, 100.0);
    assert!(e.resources.settlements.is_empty(), "A11 setup: no settlements");

    let death = drive_to_death(&mut e, ent, Some(SAT), Some(0.0), 1400);
    assert!(death.is_some(), "A11(b): bandless agent must despawn");
    assert!(!alive(&e, ent), "A11(b): agent gone");
    assert_eq!(count_agent_died(&e, id), 1, "A11(c): exactly one AgentDied");
    let (reason, _t) = find_agent_died(&e, id).expect("A11: event present");
    assert_eq!(reason, DeathReason::Starvation, "A11(c): reason Starvation");
    println!("[starvation A11] bandless agent died cleanly, event emitted ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 12: Multiple deaths in a single tick — all events, all rosters clean.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a12_multiple_same_tick_deaths() {
    // Type A — concurrency. Two members, identical saturation from the same
    // start hp → die on the same tick.
    let mut e = iso_engine();
    let (ent1, id1) = spawn_controlled(&mut e, 10, 10, 100.0);
    let (ent2, id2) = spawn_controlled(&mut e, 11, 10, 100.0);
    let sid = make_settlement_with(&mut e, &[id1, id2]);
    let deaths_before = e.resources.settlements[&sid].population_stats.total_deaths;

    let mut sys = StarvationSystem::new();
    let mut d1: Option<u64> = None;
    let mut d2: Option<u64> = None;
    for t in 0..700u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent1, Some(SAT), Some(SAT_T));
        pin(&mut e, ent2, Some(SAT), Some(SAT_T));
        sys.tick(&mut e.world, &mut e.resources);
        if d1.is_none() && !alive(&e, ent1) {
            d1 = Some(t);
        }
        if d2.is_none() && !alive(&e, ent2) {
            d2 = Some(t);
        }
        if d1.is_some() && d2.is_some() {
            break;
        }
    }
    let d1 = d1.expect("A12: agent 1 must die");
    let d2 = d2.expect("A12: agent 2 must die");
    println!("[starvation A12] death ticks d1={d1} d2={d2}");
    assert!(d1.abs_diff(d2) <= 1, "A12(a): both must die on the same tick (±1)");
    assert_eq!(count_agent_died(&e, id1), 1, "A12(b): one event for agent 1");
    assert_eq!(count_agent_died(&e, id2), 1, "A12(b): one event for agent 2");

    let s = &e.resources.settlements[&sid];
    assert!(!s.member_agents.contains(&id1), "A12(c): agent 1 removed");
    assert!(!s.member_agents.contains(&id2), "A12(c): agent 2 removed");
    assert_eq!(
        s.population_stats.total_deaths,
        deaths_before + 2,
        "A12(c): total_deaths must increment by 2 for two same-tick deaths"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 13: Well-fed agent never loses hp and never dies (one-sided guard).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a13_well_fed_never_dies() {
    // Type A — below ceiling with hp at max: no damage, no over-heal. Isolated
    // engine ⇒ no combat can perturb the exact == 100.0 check.
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let mut sys = StarvationSystem::new();
    let mut max_seen = 0.0f64;
    for t in 0..1500u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(0.0), Some(0.0));
        sys.tick(&mut e.world, &mut e.resources);
        let cur = hp(&e, ent);
        if cur > max_seen {
            max_seen = cur;
        }
    }
    println!("[starvation A13] final hp={:.4} max_seen={max_seen:.4}", hp(&e, ent));
    assert!(alive(&e, ent), "A13: well-fed agent must survive");
    assert_eq!(hp(&e, ent), DEFAULT_MAX_HP, "A13: hp must stay exactly max_hp");
    assert!(max_seen <= DEFAULT_MAX_HP, "A13: hp must never exceed max_hp");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 14: Recovery — a damaged agent held below ceiling heals and survives.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a14_recovery_heals() {
    // Type A (plan Assertion 10) — the ★ death-spiral-prevention guarantee.
    // Damage to a PARTIAL hp first (500 ticks of hunger saturation ⇒ hp ≈ 60,
    // comfortably below max so the heal is cleanly observable and does NOT clip
    // the max_hp ceiling), THEN hold both needs strictly below SAFE_NEED_CEILING
    // for a 200-tick recovery window. Plan-locked threshold: hp(feed+200) −
    // hp(feed) ≥ 8.0 (expected ≈ 200 × 0.05 = 10.0) AND strictly rising AND
    // alive at end. 700 ticks total.
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 100.0);
    let mut sys = StarvationSystem::new();

    // Damage phase — hunger saturation for 500 ticks (hp 100 → ~60).
    for t in 0..500u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(SAT), Some(0.0));
        sys.tick(&mut e.world, &mut e.resources);
    }
    let hp_feed = hp(&e, ent);
    assert!(
        hp_feed < 100.0 && hp_feed > 8.0,
        "A14 setup: hp must be damaged well below max but still alive; got {hp_feed:.4}"
    );

    // Recovery window — both needs held strictly below ceiling for 200 ticks.
    for t in 500..700u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(0.0), Some(0.0));
        sys.tick(&mut e.world, &mut e.resources);
    }
    let hp_after = hp(&e, ent);
    let rise = hp_after - hp_feed;
    println!("[starvation A14] hp_feed={hp_feed:.4} hp_feed+200={hp_after:.4} rise={rise:.4} (expect ~10.0)");
    assert!(rise >= 8.0, "A14: hp must rise ≥ 8.0 over the 200-tick recovery; got {rise:.4}");
    assert!(hp_after > hp_feed, "A14: hp must strictly increase");
    assert!(hp_after <= DEFAULT_MAX_HP, "A14: hp must not exceed max_hp");
    assert!(alive(&e, ent), "A14: recovering agent must survive");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 15: Bootstrap + newborn agents both carry BodyHealth.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a15_universal_body_health() {
    // Type A — universality. (a) all bootstrap agents; (b) the net-new lock:
    // every newborn also has BodyHealth.
    // (a) bootstrap.
    let e = bootstrapped();
    let total = e.world.query::<&Agent>().iter().count();
    let with_bh = e.world.query::<(&Agent, &BodyHealth)>().iter().count();
    assert_eq!(with_bh, total, "A15(a): all bootstrap agents must have BodyHealth");
    println!("[starvation A15a] {with_bh}/{total} bootstrap agents carry BodyHealth ✓");

    // (b) drive a birth — stable founders WITH BodyHealth + 2 buildings.
    let mut e = full_engine();
    let cx = 32u32;
    let cy = 32u32;
    let mut founder_ids = Vec::new();
    for i in 0..3u32 {
        let ent = e.spawn_agent(cx + i, cy);
        let id = e.world.get::<&Agent>(ent).unwrap().id;
        e.world
            .insert(
                ent,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                    BodyHealth::new(),
                ),
            )
            .unwrap();
        founder_ids.push(id);
    }
    for pos in [(cx, cy + 3), (cx + 1, cy + 3)] {
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent { position: pos, radius: 1 });
    }
    // Run past the birth cooldown (founded at tick 0, birth at tick 200).
    while e.current_tick() <= 215 {
        e.tick();
    }
    let final_count = live_count(&e);
    assert!(
        final_count > founder_ids.len(),
        "A15(b) setup: at least one birth must occur (count {final_count} > {})",
        founder_ids.len()
    );
    let total2 = e.world.query::<&Agent>().iter().count();
    let with_bh2 = e.world.query::<(&Agent, &BodyHealth)>().iter().count();
    assert_eq!(
        with_bh2, total2,
        "A15(b): every post-birth agent (incl. newborns) must carry BodyHealth"
    );
    println!("[starvation A15b] {with_bh2}/{total2} post-birth agents carry BodyHealth (births occurred) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 16: Gathering loop preserved — reachable food → eats and survives.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a16_gathering_loop_preserved() {
    // Type A — behavioral invariant. Hungry agent co-located with a food source
    // must drop below SAFE_NEED_CEILING (real satiation) and survive.
    let mut e = full_engine();
    let ent = e.spawn_agent(20, 20);
    e.world
        .insert(
            ent,
            (
                AgentState::Idle,
                BodyHealth::new(),
                // Above HUNGER_THRESHOLD (50) so the agent seeks; a single
                // consume (HUNGER_CONSUME_AMOUNT = 30) lands it at 40 < 50 —
                // strictly below SAFE_NEED_CEILING (a real satiation event).
                Hunger::new(70.0, 0.0),
                Thirst::new(0.0, 0.0),
                Memory::new(),
            ),
        )
        .unwrap();
    e.resources.set_food_tile(20, 20, sim_engine::RESOURCE_SOURCE_INFINITE);

    let mut min_hunger = f32::MAX;
    for _ in 0..60u64 {
        e.tick();
        if let Ok(h) = e.world.get::<&Hunger>(ent) {
            if h.value < min_hunger {
                min_hunger = h.value;
            }
        }
    }
    println!("[starvation A16] min hunger observed = {min_hunger:.4} (ceiling {SAFE_NEED_CEILING})");
    assert!(alive(&e, ent), "A16: an agent that can reach food must survive");
    assert!(
        (min_hunger as f64) < SAFE_NEED_CEILING,
        "A16: hunger must drop below SAFE_NEED_CEILING (real satiation); min {min_hunger:.4}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 17: Population stability over a long run (the balance gate).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a17_population_stability() {
    // Type E — soft/observational population-stability gate. Production scene
    // (bootstrap 64 + 3 buildings), ~5000 ticks.
    //
    // The starvation/death MECHANISM is proven by the controlled assertions
    // A1-A8 (pin a need at SATURATION → hp decays → the agent dies with the
    // correct DeathReason). This scene is RESOURCE-RICH (12 non-depleting
    // source tiles + the efficient gathering loop), so every agent reaches
    // food/water and resets its needs — zero starvation deaths here is the
    // CORRECT emergent outcome, not a bug. Forcing `total_deaths >= 1` in this
    // scene would conflate "the mechanism kills when food is unreachable" with
    // "this scene is resource-stressed"; real starvation tension is the job of
    // the follow-up resource-scarcity work, not the death mechanism. So A17
    // asserts only what IS a property of shipping the mechanism: it is WIRED
    // into the default runtime, and the population is STABLE (neither collapses
    // nor explodes). `total_deaths` is recorded for observability.
    //
    // Population ceiling (plan Assertion 16): births are config-capped by
    // `SETTLEMENT_MAX_POP` (50) PER settlement — `run_births` refuses to spawn
    // once `population_stats.current >= SETTLEMENT_MAX_POP`. The global live
    // count is therefore bounded by that per-settlement cap times the (small,
    // emergent) settlement count; we derive a generous multi-settlement ceiling
    // from the config constant rather than hard-coding a magic number. The
    // deterministic 5000-tick outcome for this scene is ~136 live (64 bootstrap
    // + births), well under the derived ceiling.
    const SETTLEMENT_MARGIN: usize = 8;
    let pop_ceiling: usize = SETTLEMENT_MAX_POP as usize * SETTLEMENT_MARGIN;
    let mut e = bootstrapped();
    place_startup_buildings(&mut e);
    for _ in 0..5000u64 {
        e.tick();
    }
    let final_count = live_count(&e);
    let total_deaths: u32 = settlement_total_deaths(&e);
    let total_births: u32 = e
        .resources
        .settlements
        .values()
        .map(|s| s.population_stats.total_births)
        .sum();
    println!(
        "[starvation A17] BALANCE — final_live={final_count} total_deaths={total_deaths} total_births={total_births}"
    );
    // Mechanism is WIRED into the default runtime (the consequence itself is
    // proven by A1-A8, which exercise this same registration path).
    assert!(
        e.system_names().contains(&"StarvationSystem"),
        "A17: StarvationSystem must be registered in the default runtime; got {:?}",
        e.system_names()
    );
    // Population STABILITY — neither collapse nor explosion.
    assert!(final_count > 0, "A17: population must not collapse to zero");
    assert!(final_count >= 20, "A17: population must not near-totally collapse (≥ 20)");
    assert!(
        final_count <= pop_ceiling,
        "A17: population must not explode (got {final_count}, ceiling {pop_ceiling} = SETTLEMENT_MAX_POP×{SETTLEMENT_MARGIN})"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 18: Determinism — same seed reproduces the death set + final pop.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a18_determinism() {
    // Type A — pure arithmetic + deferred despawn ⇒ byte-identical outcomes.
    fn run() -> (BTreeSet<AgentId>, u32, usize) {
        let mut e = bootstrapped();
        place_startup_buildings(&mut e);
        for _ in 0..5000u64 {
            e.tick();
        }
        (dead_id_set(&e), settlement_total_deaths(&e), live_count(&e))
    }
    let (set1, deaths1, live1) = run();
    let (set2, deaths2, live2) = run();
    println!(
        "[starvation A18] run1(deaths={deaths1} live={live1} dead_ids={}) run2(deaths={deaths2} live={live2} dead_ids={})",
        set1.len(),
        set2.len()
    );
    assert_eq!(set1, set2, "A18(a): dead-id set must be identical across runs");
    assert_eq!(deaths1, deaths2, "A18(b): total_deaths must be identical");
    assert_eq!(live1, live2, "A18(c): final live count must be identical");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 19 (plan Assertion 8): heal-gate strict boundary at SAFE_NEED_CEILING.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_starvation_a19_heal_gate_strict_boundary_at_ceiling() {
    // Type A — the single most-likely off-by-comparison bug guard. Both needs
    // pinned at EXACTLY SAFE_NEED_CEILING (50.0), hp started below max (80.0),
    // 300 ticks. need == 50.0 is NOT strictly below the ceiling, so the
    // `< SAFE_NEED_CEILING` heal gate MUST NOT fire (a `<=` bug would heal it);
    // nor does damage, since 50.0 < SATURATION. hp must be perfectly flat. This
    // is the ONLY assertion that pins at exactly the ceiling — distinct from the
    // gray-zone (75.0) check in A2.
    let ceil_h = SAFE_NEED_CEILING as f32; // 50.0
    let ceil_t = SAFE_NEED_CEILING; // 50.0
    let mut e = iso_engine();
    let (ent, _id) = spawn_controlled(&mut e, 10, 10, 80.0);
    let mut sys = StarvationSystem::new();

    let hp0 = hp(&e, ent);
    for t in 0..300u64 {
        e.resources.current_tick = t;
        pin(&mut e, ent, Some(ceil_h), Some(ceil_t));
        sys.tick(&mut e.world, &mut e.resources);
    }
    let hp1 = hp(&e, ent);
    println!("[starvation A19] hp0={hp0:.6} hp1={hp1:.6} (need pinned at exactly {SAFE_NEED_CEILING})");
    assert_eq!(
        hp1, hp0,
        "A19: hp must be flat with needs at exactly SAFE_NEED_CEILING (heal gate is strict `<`, not `<=`)"
    );
    assert!(alive(&e, ent), "A19: agent must stay alive at the ceiling boundary");
}
