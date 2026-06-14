//! V7 Phase 10-γ — settlement-migration pathing harness (plan_attempt 2).
//!
//! Stage 1 (`9dce85e1`) made the migration cascade arm a NO-OP (record the
//! `SettlementReason` intent, stay `Idle`) to stop the windowed mass-freeze.
//! Stage 2 / P10-γ RESTORES the FSM transition: a non-member that wants to
//! join a settlement transitions to `Seeking { Agent(member) }`, acquires a
//! persistent `SettlementMigrant` marker, receives a `SeekTarget` at the
//! NEAREST member's CURRENT tile (re-resolved every tick — the ζ post-pass),
//! walks there via the UNCHANGED β `movement.rs` directed step, is
//! auto-admitted by the existing proximity-join refresh, and then exits
//! `Seeking` (marker removed).
//!
//! NON-CIRCULAR rule (plan A2/A17): every targeting assertion compares the
//! production-chosen `SeekTarget` against a member tile computed by an
//! INDEPENDENT oracle [`nearest_member`] that reads member `Position`s out of
//! the world and re-derives the Manhattan-nearest member with the documented
//! `(x, y)` lexicographic tie-break — NEVER a value read back from
//! `SeekTarget`. A wrong production picker (e.g. lowest-id) yields a
//! `SeekTarget` ≠ the oracle's choice → the assertion fails.
//!
//! LOCKED β model (plan Section 3 / A3): `movement.rs` applies
//! `pos += (signum(dx), signum(dy))` on BOTH axes. The only single statement
//! true for both geometries is **Chebyshev distance −1 per directed tick**:
//!   • ON-AXIS  (shares one coord) → single-axis step → Manhattan −1 = Chebyshev −1.
//!   • OFF-AXIS (differs both)     → DIAGONAL step → Manhattan −2 = Chebyshev −1.
//! A "single-axis Manhattan −1" assertion for an off-axis migrant describes
//! behaviour that does NOT exist in locked β and is forbidden by the plan.
//!
//! Topology tags: [ISO] = deterministic hand-seeded; [PROD] = the mass-freeze
//! reproduction config (64 bootstrap agents + 3 startup buildings).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p10_gamma_settlement_migration -- --nocapture

use std::collections::{HashMap, HashSet};

use hecs::Entity;
use sim_bridge::ffi::world_node::{
    bootstrap_spawn_agents, collect_agent_snapshot, enqueue_building_placed, AgentSnapshotRow,
};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Memory, Position, SeekTarget, Settlement, SettlementMigrant,
    Sleep, Social, TargetKind, Thirst, SETTLEMENT_MAX_POP,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::{AgentDecisionSystem, SOCIAL_THRESHOLD};
use sim_systems::runtime::influence::BuildingStampSystem;
use sim_systems::runtime::settlement::BIRTH_COOLDOWN_TICKS;

const W: u32 = 64;
const H: u32 = 64;
const BOOTSTRAP_COUNT: u64 = 64;

// ── basic readers ────────────────────────────────────────────────────────────

/// Fresh 64×64 engine, no runtime systems (direct-tick unit tests).
fn engine64() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Fresh 64×64 engine with the full default runtime (settlement tests).
fn fresh_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Production bootstrap path (= 64 agents, ids 0..63) + default runtime.
fn bootstrapped() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    e
}

fn seek_tile(e: &SimEngine, ent: Entity) -> Option<(u32, u32)> {
    e.world.get::<&SeekTarget>(ent).ok().map(|s| s.tile)
}

fn agent_state(e: &SimEngine, ent: Entity) -> AgentState {
    *e.world.get::<&AgentState>(ent).expect("AgentState present")
}

fn agent_pos(e: &SimEngine, ent: Entity) -> (u32, u32) {
    let p = e.world.get::<&Position>(ent).expect("Position present");
    (p.x, p.y)
}

fn agent_id(e: &SimEngine, ent: Entity) -> AgentId {
    e.world.get::<&Agent>(ent).expect("Agent present").id
}

/// `true` iff `ent` carries the `SettlementMigrant` marker.
fn has_marker(e: &SimEngine, ent: Entity) -> bool {
    e.world.get::<&SettlementMigrant>(ent).is_ok()
}

fn manhattan(a: (u32, u32), b: (u32, u32)) -> i64 {
    (a.0 as i64 - b.0 as i64).abs() + (a.1 as i64 - b.1 as i64).abs()
}

/// Chebyshev distance `max(|dx|,|dy|)` between two tiles.
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> i64 {
    (a.0 as i64 - b.0 as i64).abs().max((a.1 as i64 - b.1 as i64).abs())
}

/// INDEPENDENT nearest-member oracle (plan A2/A17 non-circular requirement).
/// Reads every live member `Position` out of the world and re-derives the
/// Manhattan-nearest member of any capacity-bearing, non-empty settlement,
/// tie-broken by the member tile's `(x, y)` then the member `AgentId` — exactly
/// the production filter+tie-break, but computed from member Positions, NEVER
/// from `SeekTarget`. Returns `(member_id, member_tile)`.
fn nearest_member(e: &SimEngine, from: (u32, u32)) -> (AgentId, (u32, u32)) {
    let positions: HashMap<AgentId, (u32, u32)> = e
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (a, p))| (a.id, (p.x, p.y)))
        .collect();
    e.resources
        .settlements
        .values()
        .filter(|s| s.population_stats.current < SETTLEMENT_MAX_POP && !s.member_agents.is_empty())
        .flat_map(|s| s.member_agents.iter().copied())
        .filter_map(|mid| positions.get(&mid).map(|t| (mid, *t)))
        .min_by_key(|(mid, (mx, my))| {
            let dx = (*mx as i64 - from.0 as i64).abs();
            let dy = (*my as i64 - from.1 as i64).abs();
            (dx + dy, *mx, *my, *mid)
        })
        .expect("oracle: at least one live capacity-bearing member must exist")
}

fn entity_by_id(e: &SimEngine, aid: AgentId) -> Entity {
    for (ent, a) in e.world.query::<&Agent>().iter() {
        if a.id == aid {
            return ent;
        }
    }
    panic!("no entity with agent id {aid}");
}

fn is_member(e: &SimEngine, aid: AgentId) -> bool {
    e.resources
        .settlements
        .values()
        .any(|s| s.member_agents.contains(&aid))
}

/// Total member count across every settlement (ISO topologies hold a single
/// settlement, so this equals that settlement's roster size).
fn total_member_count(e: &SimEngine) -> usize {
    e.resources
        .settlements
        .values()
        .map(|s| s.member_agents.len())
        .sum()
}

/// `true` iff `state` is the specific migration target seek `Seeking{Agent(pid)}`.
fn is_seeking_agent_at(state: AgentState, pid: AgentId) -> bool {
    matches!(state, AgentState::Seeking { target: TargetKind::Agent(t) } if t == pid)
}

/// `true` iff `state` is any `Seeking{Agent(_)}`.
fn is_seeking_any_agent(state: AgentState) -> bool {
    matches!(state, AgentState::Seeking { target: TargetKind::Agent(_) })
}

fn count_settlement_reason(e: &SimEngine, aid: AgentId) -> usize {
    use sim_core::causal::event::{CausalEvent, DecisionReason};
    let mut n = 0;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDecision {
                reason: DecisionReason::SettlementReason,
                agent,
                ..
            } = ev
            {
                if *agent == aid {
                    n += 1;
                }
            }
        }
    }
    n
}

fn count_agent_born(e: &SimEngine) -> usize {
    use sim_core::causal::event::CausalEvent;
    let mut n = 0;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if matches!(ev, CausalEvent::AgentBorn { .. }) {
                n += 1;
            }
        }
    }
    n
}

// ── topology builders ─────────────────────────────────────────────────────────

/// Place `count` buildings near `(cx, cy)` via the FFI queue + drain.
fn place_buildings(engine: &mut SimEngine, cx: u32, cy: u32, count: u32) {
    for i in 0..count {
        let dx = i % 3;
        let dy = i / 3;
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (cx + dx, cy + dy + 3),
                radius: 1,
            });
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut engine.world, &mut engine.resources);
}

/// Place the 3 real startup buildings at the scene coordinates
/// (32,32),(24,32),(40,32) (mirrors `world_renderer.gd::_ready()`).
fn place_startup_buildings(e: &mut SimEngine) {
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
}

/// [ISO] Form a settlement with 3 STATIONARY founders (no `MovementRng`, so the
/// movement system never iterates them) at `(cx,cy),(cx+1,cy),(cx+2,cy)` plus 2
/// buildings. Founders being stationary makes the member tiles a fixed, known
/// goal frame for the directed-step / convergence assertions. Returns the
/// founder member tiles. The settlement is asserted VERIFIED-FORMED.
fn form_settlement_stationary(e: &mut SimEngine, cx: u32, cy: u32) -> Vec<(AgentId, (u32, u32))> {
    let mut founders = Vec::new();
    for i in 0..3u32 {
        let ent = e.spawn_agent(cx + i, cy);
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
                    // NO MovementRng → AgentMovementSystem skips → stationary.
                ),
            )
            .expect("seed stationary founder");
        founders.push((agent_id(e, ent), (cx + i, cy)));
    }
    place_buildings(e, cx, cy, 2);
    e.tick(); // formation runs after decision (138 > 125) → founders stay Idle, then form
    assert!(
        !e.resources.settlements.is_empty(),
        "precondition: a settlement must form from 3 founders + 2 buildings"
    );
    assert!(
        e.resources
            .settlements
            .values()
            .any(|s| !s.member_agents.is_empty()),
        "precondition: the formed settlement must have ≥1 member (VERIFIED-FORMED)"
    );
    founders
}

/// Seed a non-member outsider at `(x, y)` with the given needs / loneliness.
/// Growth rates are 0 so a need held sub-threshold stays sub-threshold for the
/// whole measured window (plan convergence-window edge case).
#[allow(clippy::too_many_arguments)] // test seed helper — explicit needs tuple is clearer than a struct
fn seed_outsider(
    e: &mut SimEngine,
    x: u32,
    y: u32,
    hunger: f64,
    thirst: f64,
    fatigue: f64,
    loneliness: f64,
    rng: u64,
) -> Entity {
    let ent = e.spawn_agent(x, y);
    e.world
        .insert(
            ent,
            (
                AgentState::Idle,
                Hunger::new(hunger as f32, 0.0),
                Thirst::new(thirst, 0.0),
                Sleep::new(fatigue, 0.0),
                Social::new(loneliness, 0.0),
                Memory::new(),
                MovementRng::new(rng),
            ),
        )
        .expect("seed outsider");
    ent
}

/// Convenience: a calm outsider (all needs 0, loneliness strictly sub-threshold)
/// — the canonical migration candidate for the convergence assertions.
fn seed_calm_outsider(e: &mut SimEngine, x: u32, y: u32, rng: u64) -> Entity {
    seed_outsider(e, x, y, 0.0, 0.0, 0.0, SOCIAL_THRESHOLD - 1.0, rng)
}

/// [ISO] Inject a settlement with members hand-placed at `member_tiles` for
/// decision-only target-selection tests (A17/A19). Members are spawned
/// stationary agents (Idle, no needs); the settlement roster lists their ids so
/// production treats them as members (they never migrate) and the nearest-member
/// selection has live positions to read. Returns the member ids in tile order.
fn inject_settlement(e: &mut SimEngine, member_tiles: &[(u32, u32)]) -> Vec<AgentId> {
    let sid = e.resources.issue_settlement_id();
    let mut s = Settlement::new_with_id(sid, 0);
    let mut ids = Vec::new();
    for &(x, y) in member_tiles {
        let ent = e.spawn_agent(x, y);
        e.world
            .insert(ent, (AgentState::Idle, Social::new(0.0, 0.0)))
            .expect("seed injected member");
        let aid = agent_id(e, ent);
        s.add_member_agent(aid);
        ids.push(aid);
    }
    s.population_stats.current = ids.len() as u32;
    e.resources.settlements.insert(sid, s);
    ids
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 1 [ISO] — transition + marker + intent event (end-of-tick triple).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a1_transition_marker_intent() {
    // Type A. After ONE decision tick on a calm far non-member (settlement
    // formed), the END-OF-TICK conjunction must hold: (a) Seeking{Agent(member)},
    // (b) SettlementMigrant marker, (c) ≥1 of the migrant's OWN SettlementReason
    // events (anti-vacuity: proves the SETTLEMENT arm fired, not the social arm).
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);

    let migrant = seed_calm_outsider(&mut e, 50, 50, 901);
    let migrant_id = agent_id(&e, migrant);
    let (expected_member, _) = nearest_member(&e, agent_pos(&e, migrant));

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);

    assert!(
        is_seeking_agent_at(agent_state(&e, migrant), expected_member),
        "A1(a): outsider must transition to Seeking{{Agent(nearest member {expected_member})}}; got {:?}",
        agent_state(&e, migrant)
    );
    assert!(
        has_marker(&e, migrant),
        "A1(b): migrant must carry the SettlementMigrant marker"
    );
    assert!(
        count_settlement_reason(&e, migrant_id) >= 1,
        "A1(c): migrant's own SettlementReason intent event must be present (settlement-arm proof)"
    );
    println!("[p10-γ A1] Idle → Seeking{{Agent({expected_member})}} + marker + intent event ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 2 [ISO] — SeekTarget == nearest member's CURRENT tile (non-circular).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a2_seektarget_nearest_member_tile() {
    // Type A. SeekTarget must equal the INDEPENDENTLY-computed nearest member
    // tile (oracle reads member Positions, not SeekTarget) AND differ from the
    // migrant's own tile (seeded far).
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);

    let migrant = seed_calm_outsider(&mut e, 50, 50, 902);
    let own = agent_pos(&e, migrant);
    let (_, expected_tile) = nearest_member(&e, own);

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);

    let st = seek_tile(&e, migrant).expect("A2: migrant must carry a SeekTarget");
    assert_eq!(
        st, expected_tile,
        "A2: SeekTarget must equal the independently-computed nearest member tile"
    );
    assert_ne!(st, own, "A2: SeekTarget must NOT be the migrant's own tile (seeded far)");
    println!("[p10-γ A2] SeekTarget {st:?} == oracle nearest, != own {own:?} ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 3 [ISO] — directed step obeys LOCKED β (Chebyshev −1), frozen-frame.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a3_directed_step_chebyshev_minus_one() {
    // Type A. Member tiles are STATIONARY (founders without MovementRng) so the
    // frozen-SeekTarget frame and the live-member frame coincide. Drive
    // `decision` (attach SeekTarget), then ONE `movement` step, manually.

    // ── 3a ON-AXIS: migrant (40,20) shares row y=20 with the founder cluster ──
    {
        let mut e = fresh_engine();
        form_settlement_stationary(&mut e, 20, 20);
        let migrant = seed_calm_outsider(&mut e, 40, 20, 9031);
        let (_, goal) = nearest_member(&e, (40, 20)); // (22,20): on-axis nearest

        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            seek_tile(&e, migrant),
            Some(goal),
            "3a setup: SeekTarget must be the nearest (on-axis) member tile"
        );
        let before = agent_pos(&e, migrant);

        let mut mov = AgentMovementSystem::new();
        mov.tick(&mut e.world, &mut e.resources);
        let after = agent_pos(&e, migrant);

        assert_eq!(
            chebyshev(after, goal),
            chebyshev(before, goal) - 1,
            "3a (LOCKED β): on-axis step closes Chebyshev distance by exactly 1; {before:?}→{after:?} toward {goal:?}"
        );
        assert_eq!(
            manhattan(after, goal),
            manhattan(before, goal) - 1,
            "3a (LOCKED β): on-axis step is single-axis ⇒ Manhattan −1"
        );
        println!("[p10-γ A3a] on-axis {before:?}→{after:?}, Chebyshev −1 toward {goal:?} ✓");
    }

    // ── 3b OFF-AXIS: migrant (40,40) differs on BOTH axes from the cluster ──
    {
        let mut e = fresh_engine();
        form_settlement_stationary(&mut e, 20, 20);
        let migrant = seed_calm_outsider(&mut e, 40, 40, 9032);
        let (_, goal) = nearest_member(&e, (40, 40)); // (22,20): off-axis nearest

        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            seek_tile(&e, migrant),
            Some(goal),
            "3b setup: SeekTarget must be the nearest (off-axis) member tile"
        );
        let before = agent_pos(&e, migrant);
        assert_ne!(before.0, goal.0, "3b setup: off-axis ⇒ x differs");
        assert_ne!(before.1, goal.1, "3b setup: off-axis ⇒ y differs");

        let mut mov = AgentMovementSystem::new();
        mov.tick(&mut e.world, &mut e.resources);
        let after = agent_pos(&e, migrant);

        assert_eq!(
            chebyshev(after, goal),
            chebyshev(before, goal) - 1,
            "3b (LOCKED β): off-axis step must close Chebyshev distance by EXACTLY 1; {before:?}→{after:?} toward {goal:?}"
        );
        let dx = after.0 as i64 - before.0 as i64;
        let dy = after.1 as i64 - before.1 as i64;
        let want_dx = (goal.0 as i64 - before.0 as i64).signum();
        let want_dy = (goal.1 as i64 - before.1 as i64).signum();
        assert!(
            dx == want_dx && dy == want_dy,
            "3b (LOCKED β): off-axis step must move BOTH axes in the correct signum direction; got ({dx},{dy}), want ({want_dx},{want_dy})"
        );
        println!("[p10-γ A3b] off-axis {before:?}→{after:?}, Chebyshev −1, diagonal toward {goal:?} ✓");
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 4 [ISO] — radius-independent convergence (Chebyshev −1 WHILE
// migrating, then exits). Asserted ONLY for in-FSM ticks; arrival via join.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a4_radius_independent_convergence() {
    // Type A. Stationary target cluster at (20,20); ONE calm migrant at (40,40).
    // For every tick the agent is STILL in the migration FSM (marker present AND
    // Seeking{Agent}) the Chebyshev distance to the SeekTarget-tile-used-that-tick
    // must drop by EXACTLY 1; the invariant is NOT evaluated at/after the exit
    // (join) tick. The agent MUST exit the FSM within initial-Chebyshev + margin.
    const MARGIN: u64 = 15;
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);
    let migrant = seed_calm_outsider(&mut e, 40, 40, 904);
    let (_, goal0) = nearest_member(&e, (40, 40));
    let initial = chebyshev((40, 40), goal0);
    let ceiling = initial as u64 + MARGIN;

    let mut prev_in_fsm: Option<i64> = None;
    let mut exited_at: Option<u64> = None;
    for t in 1..=ceiling {
        e.tick();
        let st = agent_state(&e, migrant);
        let in_fsm = has_marker(&e, migrant) && is_seeking_any_agent(st);
        if in_fsm {
            let goal = seek_tile(&e, migrant).expect("A4: in-FSM migrant must carry a SeekTarget");
            let d = chebyshev(agent_pos(&e, migrant), goal);
            if let Some(prev) = prev_in_fsm {
                assert_eq!(
                    d,
                    prev - 1,
                    "A4: while migrating, Chebyshev distance to the SeekTarget-used-that-tick must drop by exactly 1 (tick {t}: {prev}→{d})"
                );
            }
            prev_in_fsm = Some(d);
        } else if prev_in_fsm.is_some() {
            exited_at = Some(t);
            break;
        }
    }
    assert!(
        prev_in_fsm.is_some(),
        "A4: migrant must actually enter the migration FSM"
    );
    assert!(
        exited_at.is_some(),
        "A4(b): migrant must EXIT the FSM (join) within initial-Chebyshev+margin ({ceiling}) ticks; never exited"
    );
    println!(
        "[p10-γ A4] in-FSM Chebyshev −1 monotone; FSM exit (join) at tick {} ≤ {ceiling} ✓",
        exited_at.unwrap()
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 5 [ISO] — proximity arrival → auto-join; member count strictly grows.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a5_arrival_autojoin() {
    // Type A. Run the convergence scenario until the migrant is admitted. Assert
    // migrant ∈ member_agents AND the roster count strictly exceeds the
    // pre-migration founder count (no births in ISO ⇒ growth is migration-driven).
    const BUDGET: u64 = 80;
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);
    let pre_count = total_member_count(&e); // founders only
    let migrant = seed_calm_outsider(&mut e, 40, 40, 905);
    let migrant_id = agent_id(&e, migrant);

    let mut joined = false;
    for _ in 1..=BUDGET {
        e.tick();
        if is_member(&e, migrant_id) {
            joined = true;
            break;
        }
    }
    assert!(joined, "A5: migrant must join (∈ member_agents) within {BUDGET} ticks");
    assert!(
        total_member_count(&e) > pre_count,
        "A5: member count ({}) must strictly exceed the pre-migration founder count ({pre_count})",
        total_member_count(&e)
    );
    println!(
        "[p10-γ A5] migrant joined; roster {} > pre {pre_count} ✓",
        total_member_count(&e)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 6 [ISO] — post-join exit within ≤1 decision tick (negative invariant).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a6_post_join_exit_one_tick() {
    // Type A. After the join is observed, within ≤1 decision tick: (a) marker
    // absent AND (b) state != Seeking{Agent(former_target)}. Negative invariant
    // (not strict Idle) — a need may legitimately re-route the agent the same tick.
    const BUDGET: u64 = 80;
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);
    let migrant = seed_calm_outsider(&mut e, 40, 40, 906);
    let migrant_id = agent_id(&e, migrant);
    // Capture the target member BEFORE join, while still migrating, to name the
    // "former target" for the stale-seek check.
    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    let former_target = match agent_state(&e, migrant) {
        AgentState::Seeking { target: TargetKind::Agent(t) } => t,
        other => panic!("A6 setup: migrant must be Seeking{{Agent}} after first decision; got {other:?}"),
    };

    let mut joined = false;
    for _ in 1..=BUDGET {
        e.tick();
        if is_member(&e, migrant_id) {
            joined = true;
            break;
        }
    }
    assert!(joined, "A6 precondition: migrant must join within {BUDGET} ticks");

    // ≤1 decision tick window: exit may already hold at the join observation, or
    // after exactly one more tick (the acknowledged join→clear decision lag).
    let exited = |e: &SimEngine| {
        !is_seeking_agent_at(agent_state(e, migrant), former_target) && !has_marker(e, migrant)
    };
    if !exited(&e) {
        e.tick();
    }
    assert!(
        exited(&e),
        "A6: within ≤1 decision tick of joining, migrant must stop Seeking{{Agent(former {former_target})}} and drop the marker; state {:?}, marker {}",
        agent_state(&e, migrant),
        has_marker(&e, migrant)
    );
    println!("[p10-γ A6] post-join exit within ≤1 decision tick (no stale seek, no marker) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 7 [ISO] — social Seeking{Agent} is NOT misclassified as a migrant.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a7_social_not_migrant() {
    // Type A. Two social seekers on DISTINCT tiles (loneliness > threshold, each
    // carrying a SeekTarget, no settlement). Neither may acquire the marker; each
    // keeps partner-tile routing (≠ own tile).
    let mut e = engine64();
    let a = e.spawn_agent(5, 5);
    let b = e.spawn_agent(8, 5);
    let aid = agent_id(&e, a);
    let bid = agent_id(&e, b);
    e.world
        .insert(
            a,
            (
                AgentState::Seeking { target: TargetKind::Agent(bid) },
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0),
                SeekTarget::new((0, 0)),
            ),
        )
        .expect("seed A");
    e.world
        .insert(
            b,
            (
                AgentState::Seeking { target: TargetKind::Agent(aid) },
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0),
                SeekTarget::new((0, 0)),
            ),
        )
        .expect("seed B");

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);

    assert!(!has_marker(&e, a), "A7: social seeker A must NOT carry the marker");
    assert!(!has_marker(&e, b), "A7: social seeker B must NOT carry the marker");
    assert_eq!(seek_tile(&e, a), Some((8, 5)), "A7: A → partner B tile (8,5)");
    assert_eq!(seek_tile(&e, b), Some((5, 5)), "A7: B → partner A tile (5,5)");
    assert_ne!(seek_tile(&e, a), Some((5, 5)), "A7: A SeekTarget != own tile");
    assert_ne!(seek_tile(&e, b), Some((8, 5)), "A7: B SeekTarget != own tile");
    println!("[p10-γ A7] social seekers keep partner routing, no marker leaked ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 8 [ISO] — settlement dissolve (roster emptied) → abort, no freeze.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a8_dissolve_abort() {
    // Type A. Drive the migrant through the REAL entry arm, then empty every
    // settlement's member_agents (roster dissolve). On the next decision tick:
    // marker removed, no longer Seeking the former target, and no target-less
    // Seeking{Agent} at any subsequent sample.
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);
    let migrant = seed_calm_outsider(&mut e, 50, 50, 9081);

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    let former = match agent_state(&e, migrant) {
        AgentState::Seeking { target: TargetKind::Agent(t) } => t,
        other => panic!("8 setup: migrant must be Seeking{{Agent}}; got {other:?}"),
    };
    assert!(has_marker(&e, migrant), "8 setup: migrant must carry the marker");

    for s in e.resources.settlements.values_mut() {
        s.member_agents.clear();
        s.population_stats.current = 0;
    }

    dec.tick(&mut e.world, &mut e.resources); // abort (decision-only: no proximity rebuild)
    assert!(!has_marker(&e, migrant), "A8: marker removed on dissolve-abort");
    assert!(
        !is_seeking_agent_at(agent_state(&e, migrant), former),
        "A8: dissolved → migrant must stop Seeking the former member; got {:?}",
        agent_state(&e, migrant)
    );
    // No target-less Seeking{Agent} at any subsequent sample.
    for _ in 0..5u64 {
        let st = agent_state(&e, migrant);
        if is_seeking_any_agent(st) {
            assert!(
                seek_tile(&e, migrant).is_some(),
                "A8: a target-less Seeking{{Agent}} (the freeze signature) must never appear after abort"
            );
        }
        dec.tick(&mut e.world, &mut e.resources);
    }
    println!("[p10-γ A8] dissolve → abort, marker removed, no target-less seek ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 9 [ISO] — target ENTITY despawn while id lingers in roster → abort.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a9_target_despawn_abort() {
    // Type A. Drive into the FSM, then despawn the target member ENTITY while
    // leaving its id in member_agents (stale roster). The two-part target_alive
    // check (roster AND live entity) must abort: no panic, marker removed within
    // ≤3 decision ticks, migration state exited, no target-less Seeking{Agent}.
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);
    let migrant = seed_calm_outsider(&mut e, 50, 50, 9091);

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    let target = match agent_state(&e, migrant) {
        AgentState::Seeking { target: TargetKind::Agent(t) } => t,
        other => panic!("9 setup: migrant must be Seeking{{Agent}}; got {other:?}"),
    };
    assert!(has_marker(&e, migrant), "9 setup: migrant must carry the marker");

    // Despawn the target ENTITY but leave its id in member_agents (stale roster).
    let target_ent = entity_by_id(&e, target);
    e.world.despawn(target_ent).expect("despawn target member entity");
    assert!(
        is_member(&e, target),
        "9 setup: despawned member id must still LINGER in member_agents (stale roster)"
    );

    let mut exited_within = false;
    for _ in 0..3u64 {
        dec.tick(&mut e.world, &mut e.resources); // no panic on the ghost target
        if !has_marker(&e, migrant) && !is_seeking_agent_at(agent_state(&e, migrant), target) {
            exited_within = true;
            break;
        }
    }
    assert!(
        exited_within,
        "A9: despawned target (stale roster) → migrant must abort within ≤3 decision ticks; state {:?}, marker {}",
        agent_state(&e, migrant),
        has_marker(&e, migrant)
    );
    // No target-less Seeking{Agent} at any subsequent sample.
    for _ in 0..5u64 {
        if is_seeking_any_agent(agent_state(&e, migrant)) {
            assert!(
                seek_tile(&e, migrant).is_some(),
                "A9: target-less Seeking{{Agent}} freeze signature must never appear"
            );
        }
        dec.tick(&mut e.world, &mut e.resources);
    }
    println!("[p10-γ A9] despawned target (stale roster) → abort, no ghost-chase freeze ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 10 [PROD] — long-run freeze guard (per-conjunct typed, births-proof).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a10_prod_long_run_guard() {
    // 3000 ticks, sampled every 50. (a) Type A: target-less Seeking{Agent} == 0
    // at every sample. (b) Type D: worst per-agent "position unchanged WHILE in a
    // DIRECTED motile state (Seeking{_} carrying a SeekTarget)" streak < 50 —
    // Idle/Consuming/rest are EXCLUDED (Idle Brownian may legitimately stall).
    // (c) Type A: ≥1 specific bootstrap NON-founder (id<64) observed as a
    // NON-member on an EARLIER tick that later appears as a member — proving the
    // join flowed through the migration walk, not birth growth. Tracked PER-TICK
    // (not at the 50-tick (a) grid): the belonging model joins on contact and all
    // non-founders are members before tick 50, so a sample-gated capture would
    // miss every edge (measured in zzz_diag_a10 — sample-grid capture = 0).
    const TICKS: u64 = 3000;
    const SAMPLE: u64 = 50;
    let mut e = bootstrapped();
    place_startup_buildings(&mut e);

    let mut founders: Option<HashSet<AgentId>> = None;
    let mut max_targetless = 0usize;
    let mut prev_pos: HashMap<AgentId, (u32, u32)> = HashMap::new();
    let mut directed_streak: HashMap<AgentId, u32> = HashMap::new();
    let mut worst_directed = 0u32;
    let mut seen_nonmember: HashSet<AgentId> = HashSet::new(); // non-founder ids observed as NON-member at a PRIOR sample
    let mut nonmember_to_member_transitions = 0usize; // genuine non-member→member transitions observed
    let mut transition_example: Option<AgentId> = None;

    for t in 1..=TICKS {
        e.tick();

        // Non-vacuity: a settlement must form (else the freeze cannot reproduce).
        if founders.is_none() && !e.resources.settlements.is_empty() {
            let set: HashSet<AgentId> = e
                .resources
                .settlements
                .values()
                .flat_map(|s| s.member_agents.iter().copied())
                .collect();
            if !set.is_empty() {
                founders = Some(set);
            }
        }

        // (b) directed-motile frozen streak, every tick (bootstrap ids only).
        let mut live: HashSet<AgentId> = HashSet::new();
        for (_ent, (a, state, pos, seek)) in e
            .world
            .query::<(&Agent, &AgentState, &Position, Option<&SeekTarget>)>()
            .iter()
        {
            if a.id >= BOOTSTRAP_COUNT {
                continue; // newborns (id ≥ 64) excluded — separate Stage-1.5 concern
            }
            live.insert(a.id);
            let here = (pos.x, pos.y);
            let moved = prev_pos.get(&a.id).map(|p| *p != here).unwrap_or(true);
            let directed_motile =
                matches!(state, AgentState::Seeking { .. }) && seek.is_some();
            let s = directed_streak.entry(a.id).or_insert(0);
            if directed_motile && !moved {
                *s += 1;
                worst_directed = worst_directed.max(*s);
            } else {
                *s = 0;
            }
            prev_pos.insert(a.id, here);
        }
        directed_streak.retain(|k, _| live.contains(k));
        prev_pos.retain(|k, _| live.contains(k));

        // (c) genuine NON-member→member transition (membership-model-reform),
        // tracked EVERY TICK — NOT at the 50-tick (a) sample grid. Measurement
        // (zzz_diag_a10) shows the belonging model joins on radius contact and
        // PERSISTS, and every non-founder bootstrap agent joins within ~40 ticks
        // of formation (founders captured @≈tick 8; all 58 non-founders are
        // members by tick 50). A sample-gated capture would therefore ALWAYS
        // first observe them as already-members and never witness the edge —
        // which is why the prior sample-based capture read 0. Per-tick observation
        // captures the real non-member→member transition: a non-founder bootstrap
        // agent (id<64, not a founder) seen as a NON-member on an EARLIER tick
        // that LATER is a member. The ONLY way a non-founder gains membership is
        // by entering a settlement's proximity radius (the migration walk), so a
        // non-member→member transition proves the join flowed through migration,
        // not birth growth. Each agent is counted at most once (removed from the
        // tracking set on the tick it transitions). member_ids is collected first
        // (avoids holding the settlements borrow across the world query).
        if let Some(ref fset) = founders {
            let member_ids: HashSet<AgentId> = e
                .resources
                .settlements
                .values()
                .flat_map(|s| s.member_agents.iter().copied())
                .collect();
            let nonfounder_bootstrap: Vec<AgentId> = e
                .world
                .query::<&Agent>()
                .iter()
                .map(|(_, a)| a.id)
                .filter(|id| *id < BOOTSTRAP_COUNT && !fset.contains(id))
                .collect();
            for id in nonfounder_bootstrap {
                if member_ids.contains(&id) {
                    // Member now — was it a non-member on an earlier tick? If so,
                    // this is a real transition (counted once via the remove).
                    if seen_nonmember.remove(&id) {
                        nonmember_to_member_transitions += 1;
                        if transition_example.is_none() {
                            transition_example = Some(id);
                        }
                    }
                } else {
                    // Non-member on this tick — record for a FUTURE comparison.
                    seen_nonmember.insert(id);
                }
            }
        }

        if t == 10 {
            assert!(
                !e.resources.settlements.is_empty(),
                "A10 non-vacuity: a settlement must form by tick 10"
            );
        }

        if t % SAMPLE == 0 {
            // (a) target-less Seeking{Agent} count.
            let mut targetless = 0usize;
            for (_ent, (_a, state, seek)) in e
                .world
                .query::<(&Agent, &AgentState, Option<&SeekTarget>)>()
                .iter()
            {
                if matches!(state, AgentState::Seeking { target: TargetKind::Agent(_) })
                    && seek.is_none()
                {
                    targetless += 1;
                }
            }
            max_targetless = max_targetless.max(targetless);
        }
    }

    println!(
        "[p10-γ A10] max target-less Seeking{{Agent}} = {max_targetless}; worst directed frozen streak = {worst_directed}; non-member→member transitions = {nonmember_to_member_transitions} (example id = {transition_example:?})"
    );
    assert_eq!(
        max_targetless, 0,
        "A10(a): target-less Seeking{{Agent}} freeze signature must be 0 at every sample; got {max_targetless}"
    );
    assert!(
        worst_directed < 50,
        "A10(b): worst DIRECTED-motile frozen streak must be < 50 ticks; got {worst_directed}"
    );
    assert!(
        nonmember_to_member_transitions >= 1,
        "A10(c): ≥1 genuine NON-member→member transition (a non-founder bootstrap agent \
         that was a NON-member at an earlier sample and a member at a later sample) must \
         occur — proves migration→join under the belonging model; got {nonmember_to_member_transitions}"
    );
    println!("[p10-γ A10] PROD long-run: 0 target-less, directed freeze {worst_directed}<50, {nonmember_to_member_transitions} non-member→member transition(s) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 11 [ISO] — migrant already ON the member tile still enters FSM,
// then joins & exits (Stage-1 idle/no-transition path must NOT satisfy this).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a11_on_member_tile() {
    // Type A. Migrant seeded ON the formation centre (20,20) where a founder
    // sits. nearest member = that founder (distance 0). Decision-only tick MUST
    // produce the Stage-2 entry triple (a vacuous Idle pass is a FAILURE); β
    // takes a zero step; then proximity join + explicit exit.
    const BUDGET: u64 = 40;
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);
    let migrant = seed_calm_outsider(&mut e, 20, 20, 911);
    let migrant_id = agent_id(&e, migrant);
    let (member, member_tile) = nearest_member(&e, (20, 20));

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    assert!(
        is_seeking_agent_at(agent_state(&e, migrant), member),
        "A11: on-tile migrant MUST transition to Seeking{{Agent}} (Stage-2), not stay Idle (Stage-1); got {:?}",
        agent_state(&e, migrant)
    );
    assert!(has_marker(&e, migrant), "A11: on-tile migrant MUST carry the marker");
    assert_eq!(
        seek_tile(&e, migrant),
        Some(member_tile),
        "A11: on-tile migrant SeekTarget == member tile (degenerate, allowed)"
    );
    // Zero-length directed step: one movement tick must not move it / crash.
    let mut mov = AgentMovementSystem::new();
    mov.tick(&mut e.world, &mut e.resources);
    assert_eq!(agent_pos(&e, migrant), (20, 20), "A11: zero-length step must not move the on-tile migrant");

    // Full runtime: proximity join + explicit exit (≤5 ticks after join).
    let mut joined = false;
    for _ in 1..=BUDGET {
        e.tick();
        if is_member(&e, migrant_id) {
            joined = true;
            break;
        }
    }
    assert!(joined, "A11: on-tile migrant must join within ≤{BUDGET} ticks");
    let mut exited = false;
    for _ in 0..5u64 {
        if !is_seeking_agent_at(agent_state(&e, migrant), member) && !has_marker(&e, migrant) {
            exited = true;
            break;
        }
        e.tick();
    }
    assert!(exited, "A11: on-tile migrant must exit migration (marker removed) within ≤5 ticks of joining");
    println!("[p10-γ A11] on-member-tile: zero-length step, joined, exited ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 12 [ISO] — concurrent migrants: live per-tick re-resolution,
// including the same-target case (join-order-independent, neither starved).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a12_concurrent_migrants() {
    // (a) DIFFERENT targets: two migrants whose nearest members differ; after the
    //     targets move, each re-resolves to its own target's NEW tile.
    {
        let mut e = engine64();
        // Decision-only world: inject a settlement with two members at distinct
        // tiles; membership is hand-controlled (no SettlementSystem rebuild).
        let members = inject_settlement(&mut e, &[(10, 10), (30, 30)]);
        let (m_a, m_b) = (members[0], members[1]);
        let mig1 = seed_calm_outsider(&mut e, 5, 5, 9121); // nearest → (10,10)
        let mig2 = seed_calm_outsider(&mut e, 35, 35, 9122); // nearest → (30,30)

        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert!(is_seeking_agent_at(agent_state(&e, mig1), m_a), "A12a: mig1 → member A");
        assert!(is_seeking_agent_at(agent_state(&e, mig2), m_b), "A12a: mig2 → member B");
        assert_eq!(seek_tile(&e, mig1), Some((10, 10)), "A12a: mig1 SeekTarget == A tile");
        assert_eq!(seek_tile(&e, mig2), Some((30, 30)), "A12a: mig2 SeekTarget == B tile");

        // Move both target members; decision-only ⇒ migrants stay put.
        e.world.insert_one(entity_by_id(&e, m_a), Position { x: 12, y: 12 }).expect("move A");
        e.world.insert_one(entity_by_id(&e, m_b), Position { x: 28, y: 28 }).expect("move B");
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(seek_tile(&e, mig1), Some((12, 12)), "A12a: mig1 re-resolves to A's NEW tile");
        assert_eq!(seek_tile(&e, mig2), Some((28, 28)), "A12a: mig2 re-resolves to B's NEW tile");
        println!("[p10-γ A12a] different-target migrants re-resolve independently ✓");
    }

    // (b1) SAME target re-resolution: two migrants, ONE member; both re-resolve.
    {
        let mut e = engine64();
        let members = inject_settlement(&mut e, &[(10, 10)]);
        let m = members[0];
        let mig1 = seed_calm_outsider(&mut e, 5, 5, 9123);
        let mig2 = seed_calm_outsider(&mut e, 5, 15, 9124);

        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(seek_tile(&e, mig1), Some((10, 10)), "A12b1: mig1 → shared member tile");
        assert_eq!(seek_tile(&e, mig2), Some((10, 10)), "A12b1: mig2 → shared member tile");

        e.world.insert_one(entity_by_id(&e, m), Position { x: 13, y: 13 }).expect("move shared member");
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(seek_tile(&e, mig1), Some((13, 13)), "A12b1: mig1 re-resolves to NEW shared tile");
        assert_eq!(seek_tile(&e, mig2), Some((13, 13)), "A12b1: mig2 re-resolves to NEW shared tile");
        println!("[p10-γ A12b1] same-target migrants both re-resolve to the moving member ✓");
    }

    // (b2) SAME target join: two migrants to the SAME real settlement BOTH join
    //      (join-order-independent — neither starved).
    {
        const BUDGET: u64 = 90;
        let mut e = fresh_engine();
        form_settlement_stationary(&mut e, 20, 20);
        let mig1 = seed_calm_outsider(&mut e, 50, 50, 9125);
        let mig2 = seed_calm_outsider(&mut e, 10, 48, 9126);
        let id1 = agent_id(&e, mig1);
        let id2 = agent_id(&e, mig2);

        let mut both = false;
        for _ in 1..=BUDGET {
            e.tick();
            if is_member(&e, id1) && is_member(&e, id2) {
                both = true;
                break;
            }
        }
        assert!(both, "A12b2: BOTH same-target migrants must join within {BUDGET} ticks (neither starved)");
        println!("[p10-γ A12b2] both same-target migrants joined (neither starved) ✓");
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 13 [PROD/ISO] — cross-feature regression: births, gather, determinism.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a13_cross_feature_regression() {
    // (a) Type D — settlement births still fire after the migration change.
    {
        let mut e = fresh_engine();
        for i in 0..3u32 {
            let ent = e.spawn_agent(26 + i, 26);
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
                        MovementRng::new(700 + i as u64),
                    ),
                )
                .expect("seed founder");
        }
        place_buildings(&mut e, 26, 26, 2);
        for _ in 0..(BIRTH_COOLDOWN_TICKS + 2) {
            e.tick();
        }
        let births = count_agent_born(&e);
        assert!(births >= 1, "A13(a): settlement births must still fire; got {births}");
        println!("[p10-γ A13a] {births} AgentBorn after cooldown — births preserved ✓");
    }

    // (b) Type A — resource gather loop preserved: Idle→Seeking{Food}→Consuming.
    {
        let mut e = engine64();
        let a = e.spawn_agent(10, 10);
        e.world
            .insert(a, (AgentState::Idle, Hunger::new(51.0, 0.0)))
            .expect("seed hungry");
        e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);
        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            agent_state(&e, a),
            AgentState::Seeking { target: TargetKind::Food },
            "A13(b): Idle → Seeking{{Food}}"
        );
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            agent_state(&e, a),
            AgentState::Consuming { target: TargetKind::Food },
            "A13(b): Seeking{{Food}} → Consuming{{Food}}"
        );
        println!("[p10-γ A13b] resource gather loop preserved ✓");
    }

    // (c) Type A — determinism: two independent runs ⇒ identical maps.
    {
        type SnapMap = HashMap<AgentId, ((u32, u32), AgentState, bool, Option<(u32, u32)>)>;
        fn run() -> SnapMap {
            let mut e = bootstrapped();
            place_startup_buildings(&mut e);
            for _ in 0..200u64 {
                e.tick();
            }
            let mut rows: Vec<(Entity, AgentId, (u32, u32), AgentState)> = Vec::new();
            for (ent, (a, p, s)) in e.world.query::<(&Agent, &Position, &AgentState)>().iter() {
                rows.push((ent, a.id, (p.x, p.y), *s));
            }
            let mut out = SnapMap::new();
            for (ent, id, pos, st) in rows {
                let marker = e.world.get::<&SettlementMigrant>(ent).is_ok();
                let seek = e.world.get::<&SeekTarget>(ent).ok().map(|s| s.tile);
                out.insert(id, (pos, st, marker, seek));
            }
            out
        }
        let m1 = run();
        let m2 = run();
        assert!(!m1.is_empty(), "A13(c): bootstrap must spawn agents");
        assert_eq!(m1, m2, "A13(c): two 200-tick runs must yield identical {{id→(pos,state,marker,seek)}} maps");
        println!("[p10-γ A13c] determinism preserved across two runs ({} agents) ✓", m1.len());
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 14 [ISO+PROD] — members/founders never carry the marker
// (with the ≤1-decision-tick join→clear tolerance).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a14_marker_hygiene() {
    // Type A. The proximity refresh sets membership; the decision arm clears the
    // marker on the NEXT decision tick, so a just-joined agent is legitimately
    // member+marker for ≤1 tick. A violation is a member that carries the marker
    // AND was ALSO a member at the previous sample (i.e. NOT within that ≤1-tick
    // lag). Founders (members at formation) — tested in the ISO topology where
    // they are STATIONARY and never leave — must NEVER carry the marker.
    //
    // The founder-never-marker invariant is deliberately checked ONLY in ISO:
    // in PROD a bootstrap agent that was a member at the first formation tick can
    // legitimately wander out (Brownian) and later re-migrate, carrying the
    // marker — so "founder" is not a permanent label there. The PROD run instead
    // enforces the stuck-marker invariant, which covers every current member
    // (founders included) without that false positive.

    // ── ISO: stationary founders + a single joining migrant, sampled every tick ──
    {
        const BUDGET: u64 = 80;
        let mut e = fresh_engine();
        let founders = form_settlement_stationary(&mut e, 20, 20);
        let founder_ids: HashSet<AgentId> = founders.iter().map(|(id, _)| *id).collect();
        let migrant = seed_calm_outsider(&mut e, 40, 40, 9140);
        let migrant_id = agent_id(&e, migrant);

        let mut prev_member: HashSet<AgentId> = HashSet::new();
        let mut joined = false;
        for _ in 1..=BUDGET {
            e.tick();
            let mut now_member: HashSet<AgentId> = HashSet::new();
            for (ent, a) in e.world.query::<&Agent>().iter() {
                let member = is_member(&e, a.id);
                let marker = has_marker(&e, ent);
                if member {
                    now_member.insert(a.id);
                }
                // Stationary founders are permanent members → never migrants.
                if founder_ids.contains(&a.id) {
                    assert!(!marker, "A14(ISO): stationary founder {} must NEVER carry the marker", a.id);
                }
                // No agent may stay member+marker across two consecutive samples.
                if member && marker && prev_member.contains(&a.id) {
                    panic!(
                        "A14(ISO): agent {} member+marker for ≥2 consecutive ticks — stuck marker, not the ≤1-tick join lag",
                        a.id
                    );
                }
            }
            prev_member = now_member;
            if is_member(&e, migrant_id) {
                joined = true;
            }
        }
        assert!(joined, "A14(ISO) non-vacuity: the migrant must actually join within {BUDGET} ticks");
    }

    // ── PROD: stuck-marker invariant across a long wandering run ──
    // Sampled EVERY tick: the marker is cleared on the decision tick AFTER the
    // proximity refresh sets membership, so the legitimate member+marker window
    // is exactly 1 tick. A boundary-flickering agent (Brownian in/out of the
    // radius) is member+marker for ≤1 tick on EACH re-entry — a 25-tick sampling
    // gap would false-flag two unrelated re-entries as "stuck", so the tolerance
    // must be per-tick. Violation = member+marker for ≥2 CONSECUTIVE ticks.
    {
        const TICKS: u64 = 1500;
        let mut e = bootstrapped();
        place_startup_buildings(&mut e);
        let mut consec_member_marker: HashMap<AgentId, u32> = HashMap::new();
        let mut worst = 0u32;
        for t in 1..=TICKS {
            e.tick();
            let mut live: HashSet<AgentId> = HashSet::new();
            for (ent, a) in e.world.query::<&Agent>().iter() {
                live.insert(a.id);
                let stuck = is_member(&e, a.id) && has_marker(&e, ent);
                let c = consec_member_marker.entry(a.id).or_insert(0);
                if stuck {
                    *c += 1;
                    worst = worst.max(*c);
                } else {
                    *c = 0;
                }
                assert!(
                    *c < 2,
                    "A14(PROD): agent {} member+marker for ≥2 consecutive ticks (tick {t}) — stuck marker, not the ≤1-tick join lag",
                    a.id
                );
            }
            consec_member_marker.retain(|k, _| live.contains(k));
        }
        println!("[p10-γ A14] PROD worst consecutive member+marker streak = {worst} (≤1 tolerated)");
    }
    println!("[p10-γ A14] marker hygiene: stationary founders never marked, no stuck member+marker (≤1-tick join lag tolerated) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 15 [ISO] — need-preemption blocks/aborts migration (entry + mid-walk),
// parametrized over hunger / thirst / sleep.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a15_need_preemption() {
    // Type A. (a) ENTRY: a non-member with ONE need above threshold (others sub,
    // loneliness sub, settlement present) does NOT enter migration — no marker and
    // the resulting Seeking is the resource/rest arm. (b) MID-WALK: a migrant
    // driven into the FSM then pushed above threshold on one need EXITS migration
    // (marker cleared) and re-routes to the resource/rest arm with no stale marker.
    // Need index: 0=hunger→Food, 1=thirst→Water, 2=sleep→Sleep.
    let resource = |need: usize| match need {
        0 => TargetKind::Food,
        1 => TargetKind::Water,
        _ => TargetKind::Sleep,
    };
    // Build a Hunger/Thirst/Sleep triple with exactly `need` breached at 51.
    let needs = |need: usize| -> (f64, f64, f64) {
        match need {
            0 => (51.0, 0.0, 0.0),
            1 => (0.0, 51.0, 0.0),
            _ => (0.0, 0.0, 51.0),
        }
    };

    for need in 0..3usize {
        // ── (a) ENTRY ──
        {
            let mut e = fresh_engine();
            form_settlement_stationary(&mut e, 20, 20);
            let (h, t, f) = needs(need);
            let migrant = seed_outsider(&mut e, 50, 50, h, t, f, SOCIAL_THRESHOLD - 1.0, 9150 + need as u64);
            let mut dec = AgentDecisionSystem::new();
            dec.tick(&mut e.world, &mut e.resources);
            assert!(
                !has_marker(&e, migrant),
                "A15a(need {need}): a breached-need non-member must NOT acquire the migrant marker"
            );
            assert_eq!(
                agent_state(&e, migrant),
                AgentState::Seeking { target: resource(need) },
                "A15a(need {need}): must take the resource/rest arm, not the migration arm"
            );
            assert!(
                !is_seeking_any_agent(agent_state(&e, migrant)),
                "A15a(need {need}): resulting Seeking must NOT be Seeking{{Agent}} (migration)"
            );
        }
        // ── (b) MID-WALK ──
        {
            let mut e = fresh_engine();
            form_settlement_stationary(&mut e, 20, 20);
            let migrant = seed_calm_outsider(&mut e, 50, 50, 9160 + need as u64);
            let mut dec = AgentDecisionSystem::new();
            dec.tick(&mut e.world, &mut e.resources); // → migration FSM
            assert!(
                is_seeking_any_agent(agent_state(&e, migrant)) && has_marker(&e, migrant),
                "A15b(need {need}) setup: migrant must be in the migration FSM"
            );
            // Breach the need mid-walk.
            match need {
                0 => e.world.insert_one(migrant, Hunger::new(51.0, 0.0)).expect("breach hunger"),
                1 => e.world.insert_one(migrant, Thirst::new(51.0, 0.0)).expect("breach thirst"),
                _ => e.world.insert_one(migrant, Sleep::new(51.0, 0.0)).expect("breach fatigue"),
            };
            dec.tick(&mut e.world, &mut e.resources); // preemption
            assert!(
                !has_marker(&e, migrant),
                "A15b(need {need}): the marker must be cleared within ≤1 decision tick of the breach"
            );
            assert_eq!(
                agent_state(&e, migrant),
                AgentState::Seeking { target: resource(need) },
                "A15b(need {need}): preempted migrant must re-route to the resource/rest arm"
            );
        }
    }
    println!("[p10-γ A15] need-preemption (entry + mid-walk) for hunger/thirst/sleep ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 16 [ISO] — SettlementMigrant survives a serde round-trip (Day-1).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a16_serde_round_trip() {
    // Type A. A new component that does not round-trip breaks save/load.
    let original = SettlementMigrant;
    let encoded = ron::to_string(&original).expect("SettlementMigrant must Serialize");
    let decoded: SettlementMigrant =
        ron::from_str(&encoded).expect("SettlementMigrant must Deserialize");
    assert_eq!(original, decoded, "A16: marker must be byte-identical after a serde round-trip");
    println!("[p10-γ A16] SettlementMigrant serde round-trip ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 17 [ISO] — multi-member nearest selection + (x,y) tie-break + determinism.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a17_nearest_selection_determinism() {
    // Type A. (i) DISTINCT distances → the Manhattan-nearest member is chosen.
    {
        let mut e = engine64();
        let _members = inject_settlement(&mut e, &[(10, 10), (10, 20)]);
        // migrant (10,40): to (10,20)=20 < (10,10)=30 → nearest (10,20).
        let migrant = seed_calm_outsider(&mut e, 10, 40, 9171);
        let (_, expected) = nearest_member(&e, (10, 40));
        assert_eq!(expected, (10, 20), "A17(i) oracle sanity: nearest must be (10,20)");
        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(seek_tile(&e, migrant), Some((10, 20)), "A17(i): SeekTarget == nearest member tile");
    }

    // (ii) EQUAL distance → the documented (x,y) lexicographic tie-break decides,
    //      and two identical runs select the SAME member.
    let run_tie = || -> (u32, u32) {
        let mut e = engine64();
        // members (10,10) and (20,10); migrant (15,40): both Manhattan 35 (tie).
        let _members = inject_settlement(&mut e, &[(10, 10), (20, 10)]);
        let migrant = seed_calm_outsider(&mut e, 15, 40, 9172);
        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        seek_tile(&e, migrant).expect("A17(ii): migrant must carry a SeekTarget")
    };
    assert_eq!(manhattan((15, 40), (10, 10)), manhattan((15, 40), (20, 10)), "A17(ii) sanity: tie");
    let first = run_tie();
    let second = run_tie();
    assert_eq!(
        first, (10, 10),
        "A17(ii): the (x,y) lexicographic tie-break must pick the lower-x member (10,10); got {first:?}"
    );
    assert_eq!(first, second, "A17(ii): two identical runs must select the same member (determinism)");
    println!("[p10-γ A17] nearest selection + (x,y) tie-break + determinism ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 18 [ISO] — SettlementMigrant is absent from every FFI snapshot surface.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a18_no_ffi_exposure() {
    // Type A. Programmatic negative check: the marker must have ZERO footprint on
    // the FFI agent-snapshot surface. Snapshot the SAME entity with the marker,
    // then after removing it — identical AgentSnapshotRow ⇒ the marker leaks into
    // no snapshot key/array (the snapshot only carries state_tag / seek_kind /
    // target tile, all derived from AgentState + SeekTarget, never the marker).
    let mut e = engine64();
    let migrant = e.spawn_agent(40, 40);
    let member = e.spawn_agent(20, 20);
    let member_id = agent_id(&e, member);
    e.world
        .insert(
            migrant,
            (
                AgentState::Seeking { target: TargetKind::Agent(member_id) },
                SeekTarget::new((20, 20)),
                SettlementMigrant,
            ),
        )
        .expect("seed migrant with marker");

    let with_marker = collect_agent_snapshot(&e.world);
    e.world.remove_one::<SettlementMigrant>(migrant).expect("remove marker");
    let without_marker = collect_agent_snapshot(&e.world);

    assert_eq!(
        with_marker, without_marker,
        "A18: the SettlementMigrant marker must not influence ANY AgentSnapshotRow field (no FFI exposure)"
    );
    // Sanity: the migrant row exists and exposes only the in-scope ε surface
    // (Agent seek ⇒ seek_kind 0; SeekTarget still surfaced via target_x/y).
    let row: &AgentSnapshotRow = with_marker
        .iter()
        .find(|r| r.agent_id == agent_id(&e, migrant))
        .expect("A18: migrant must appear in the snapshot");
    assert_eq!(row.seek_kind, 0, "A18: Seeking{{Agent}} carries seek_kind 0 (not a resource trip)");
    println!("[p10-γ A18] marker has zero FFI snapshot footprint ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 19 [ISO] — no settlement / empty roster → stays Idle, no marker,
// no garbage target, no panic.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a19_no_settlement_empty_roster() {
    // Type A. (a) ZERO settlements; (b) ONE settlement with an EMPTY roster.
    // In both: stays Idle, no marker, no migration-assigned SeekTarget, no panic.

    // ── (a) zero settlements ──
    {
        let mut e = engine64();
        let outsider = seed_calm_outsider(&mut e, 30, 30, 9191);
        assert!(e.resources.settlements.is_empty(), "19a setup: no settlements");
        let mut dec = AgentDecisionSystem::new();
        for _ in 0..5u64 {
            dec.tick(&mut e.world, &mut e.resources); // must not panic
        }
        assert_eq!(agent_state(&e, outsider), AgentState::Idle, "A19a: stays Idle with no settlement");
        assert!(!has_marker(&e, outsider), "A19a: no marker with no settlement");
        assert!(seek_tile(&e, outsider).is_none(), "A19a: no migration-assigned SeekTarget");
    }

    // ── (b) one settlement, empty roster ──
    {
        let mut e = engine64();
        let sid = e.resources.issue_settlement_id();
        let s = Settlement::new_with_id(sid, 0); // member_agents empty, current 0
        e.resources.settlements.insert(sid, s);
        let outsider = seed_calm_outsider(&mut e, 30, 30, 9192);
        let mut dec = AgentDecisionSystem::new();
        for _ in 0..5u64 {
            dec.tick(&mut e.world, &mut e.resources); // empty roster ⇒ no candidate, no panic
        }
        assert_eq!(agent_state(&e, outsider), AgentState::Idle, "A19b: empty roster ⇒ stays Idle");
        assert!(!has_marker(&e, outsider), "A19b: empty roster ⇒ no marker");
        assert!(seek_tile(&e, outsider).is_none(), "A19b: empty roster ⇒ no migration SeekTarget");
    }
    println!("[p10-γ A19] no-settlement / empty-roster → Idle, no marker, no garbage target, no panic ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 2b [ISO] — plan A2 anti-gaming: a NON-member decoy placed strictly
// CLOSER than any member must NOT be selected (the headline `nearest_agent()`
// freeze vector). The picker iterates member_agents only, so the migrant targets
// a genuine roster member even though the decoy is in the live position map.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a2b_decoy_closer_non_member_not_selected() {
    // Type A. A `target = nearest_agent()` implementation passes the rigged
    // "member is nearest" topology (A2) but freezes in production by walking to a
    // non-member. Placing a non-member decoy STRICTLY CLOSER (Manhattan) than any
    // member forces the implementation to select a roster member specifically:
    // self-targeting and decoy-targeting both fail.
    let mut e = fresh_engine();
    form_settlement_stationary(&mut e, 20, 20);

    // Decoy: a non-member far from the formation tile (Chebyshev ≫ 5 of (20,20),
    // so the proximity refresh never admits it), placed much closer to the
    // migrant than any founder member. It IS in `agent_positions`, so a
    // nearest-ANY-agent picker would choose it.
    let decoy_tile = (45u32, 50u32);
    let _decoy = seed_calm_outsider(&mut e, decoy_tile.0, decoy_tile.1, 9210);
    let migrant = seed_calm_outsider(&mut e, 50, 50, 9211);
    let own = agent_pos(&e, migrant);
    let (member_id, member_tile) = nearest_member(&e, own);

    // Sanity: the decoy is STRICTLY closer than the chosen member, so a
    // nearest-agent bug would pick the decoy.
    assert!(
        manhattan(own, decoy_tile) < manhattan(own, member_tile),
        "A2b sanity: decoy {decoy_tile:?} must be strictly closer than member tile {member_tile:?}"
    );
    assert!(
        !is_member(&e, agent_id(&e, _decoy)),
        "A2b sanity: the decoy must be a NON-member"
    );

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);

    let st = seek_tile(&e, migrant).expect("A2b: migrant must carry a SeekTarget");
    assert_eq!(
        st, member_tile,
        "A2b: SeekTarget must be the nearest MEMBER tile, not the closer non-member decoy"
    );
    assert_ne!(st, decoy_tile, "A2b: SeekTarget must NOT be the closer non-member decoy tile");
    assert_ne!(st, own, "A2b: SeekTarget must NOT be the migrant's own tile");
    assert!(
        is_member(&e, member_id),
        "A2b: the chosen target's AgentId must be in member_agents"
    );
    println!("[p10-γ A2b] closer non-member decoy NOT selected; SeekTarget == member tile {member_tile:?} ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 20 [ISO] — plan A15: a FULL settlement (current == SETTLEMENT_MAX_POP)
// is never targeted, so a nearby non-member never enters a permanent
// targetless/standstill freeze. Decision-only engine so the injected full roster
// is not recomputed by the proximity refresh (which would reset `current`).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_gamma_a20_full_settlement_no_permanent_freeze() {
    // Type D liveness guard. MAX_POP is reachable in production (births grow
    // settlements). With proximity-join rejecting admission at MAX_POP, a migrant
    // that TARGETED a full settlement would reach proximity, never join, and stay
    // Seeking{Agent(member)} forever — the precise freeze this stage eliminates.
    // The migration arm filters `population_stats.current < MAX_POP`, so a full
    // settlement is never selected: the migrant stays Idle and keeps Brownian
    // motion (no marker, no targetless Seeking{Agent}, no permanent standstill).
    const TICKS: u64 = 400;
    const WINDOW: u32 = 100;
    let mut e = engine64();

    // Inject a FULL settlement: one LIVE member near (20,20) (so a buggy picker
    // has a live tile to target) with `current` pinned at MAX_POP.
    let sid = e.resources.issue_settlement_id();
    let mut s = Settlement::new_with_id(sid, 0);
    let member_ent = e.spawn_agent(20, 20);
    e.world
        .insert(member_ent, (AgentState::Idle, Social::new(0.0, 0.0)))
        .expect("seed full-settlement member");
    s.add_member_agent(agent_id(&e, member_ent));
    s.population_stats.current = SETTLEMENT_MAX_POP; // FULL → join impossible
    e.resources.settlements.insert(sid, s);
    assert_eq!(
        e.resources.settlements.values().next().unwrap().population_stats.current,
        SETTLEMENT_MAX_POP,
        "A20 setup: the injected settlement must be at MAX_POP"
    );

    // Migrant within the proximity disk of (20,20) (Chebyshev ≤ 5).
    let migrant = seed_calm_outsider(&mut e, 23, 22, 9200);

    let mut dec = AgentDecisionSystem::new();
    let mut mov = AgentMovementSystem::new();
    let mut worst_targetless = 0u32;
    let mut targetless_streak = 0u32;
    let mut standstill = 0u32;
    let mut worst_standstill = 0u32;
    let mut prev = agent_pos(&e, migrant);

    for _ in 0..TICKS {
        dec.tick(&mut e.world, &mut e.resources);
        mov.tick(&mut e.world, &mut e.resources);

        // A full settlement must never be selected → no marker, ever.
        assert!(
            !has_marker(&e, migrant),
            "A20: full settlement must never be targeted → migrant must NOT carry the marker"
        );

        let st = agent_state(&e, migrant);
        let targetless = is_seeking_any_agent(st) && seek_tile(&e, migrant).is_none();
        if targetless {
            targetless_streak += 1;
            worst_targetless = worst_targetless.max(targetless_streak);
        } else {
            targetless_streak = 0;
        }

        let here = agent_pos(&e, migrant);
        if here != prev {
            standstill = 0;
            prev = here;
        } else {
            standstill += 1;
            worst_standstill = worst_standstill.max(standstill);
        }
    }

    assert!(
        worst_targetless < WINDOW,
        "A20: worst targetless Seeking{{Agent}} streak {worst_targetless} must be < {WINDOW} (no permanent freeze)"
    );
    assert!(
        worst_standstill < WINDOW,
        "A20: migrant must move within every {WINDOW}-tick window; worst standstill {worst_standstill}"
    );
    println!(
        "[p10-γ A20] full-settlement (MAX_POP) never targeted; worst targetless {worst_targetless}<{WINDOW}, worst standstill {worst_standstill}<{WINDOW} ✓"
    );
}
