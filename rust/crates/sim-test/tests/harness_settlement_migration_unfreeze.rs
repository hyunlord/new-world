//! V7 Settlement-migration unfreeze (Stage 1) — mass-freeze regression harness.
//!
//! Reproduces the REAL windowed scene topology that triggered the user-reported
//! "사람이 멈추는" (agents freeze en masse) bug: the production `bootstrap`
//! path (64 agents) PLUS the scene's 3 startup buildings, which form a
//! settlement early. Pre-fix, every non-member agent that reached the
//! lowest-priority settlement-migration arm transitioned to
//! `Seeking { Agent(member) }` with NO `SeekTarget`; `movement.rs` suppresses
//! Brownian motion for every `Seeking` state and steps only toward an attached
//! `SeekTarget`, so the migrant froze forever (tick 27 → 60/64 frozen,
//! tick 201 → 64/64).
//!
//! The Stage-1 fix records the migration INTENT (`SettlementReason` causal
//! event — preserves p10-β A16 + community-history routing) but removes the
//! `Seeking{Agent}` FSM transition. The non-member stays `Idle` → keeps
//! Brownian motion → no freeze. Full migration pathing (walk to settlement +
//! join) is Stage 2 / P10-γ.
//!
//! The bug's EXACT signature is `AgentState::Seeking{Agent(_)}` carrying NO
//! `SeekTarget` (the settlement arm attaches none; the legitimate ζ social arm
//! ALWAYS attaches the partner tile). The long-run guard (A5a) keys on that
//! targetless signature so it distinguishes the freeze bug from legitimate
//! social seeks.
//!
//! Run:
//!   cargo test -p sim-test --test harness_settlement_migration_unfreeze -- --nocapture

use std::collections::{HashMap, HashSet};

use hecs::Entity;
use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_core::causal::event::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Memory, Position, SeekTarget, Sleep, Social, TargetKind,
    Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::{AgentDecisionSystem, SOCIAL_THRESHOLD};
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;

// ── helpers (mirror harness_s16_zeta_social_freeze_fix.rs) ──────────────────

/// Fresh 64×64 engine, no systems registered (for direct-tick unit tests).
fn engine64() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Fresh 64×64 engine with the full default runtime (for settlement tests).
fn fresh_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Production bootstrap path: `new` → `register_default_runtime_systems`
/// → `bootstrap_spawn_agents` (= 64 agents, ids 0..63).
fn bootstrapped() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    e
}

/// `Some(tile)` if `e` carries a `SeekTarget`, else `None`.
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

/// Spawn `count` agents in a tight cluster near `(cx, cy)`, needs clamped to 0
/// so no need-driven cascade arm fires. Mirrors `harness_s16_zeta` / p10-β.
fn spawn_cluster(engine: &mut SimEngine, cx: u32, cy: u32, count: u32) -> Vec<AgentId> {
    let mut ids = Vec::new();
    for i in 0..count {
        let dx = i % 3;
        let dy = i / 3;
        let entity = engine.spawn_agent(cx + dx, cy + dy);
        let aid = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                    MovementRng::new(42u64.wrapping_add(i as u64)),
                ),
            )
            .expect("seed cluster agent");
        ids.push(aid);
    }
    ids.sort();
    ids
}

/// Place `count` buildings near `(cx, cy)` via the FFI queue + drain. Mirrors
/// `harness_s16_zeta` / p10-β — used to form a settlement around founders.
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

/// Place the 3 startup buildings at the REAL scene coordinates
/// (32,32),(24,32),(40,32) via the FFI queue + a `BuildingStampSystem` drain.
/// Mirrors `world_renderer.gd::_ready()` — this is what makes a settlement form
/// early and triggers the mass migration freeze in the pre-fix code.
fn place_startup_buildings(e: &mut SimEngine) {
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
}

/// Form a settlement around a 3-founder cluster + 2 buildings at (20,20)
/// (the proven `harness_s16_zeta` A4 formation), leaving the founders as
/// members. One tick runs the formation system.
fn form_settlement(e: &mut SimEngine) {
    let _founders = spawn_cluster(e, 20, 20, 3);
    place_buildings(e, 20, 20, 2);
    e.tick();
}

/// Lowest `AgentId` that is a member of any formed settlement.
fn any_member(e: &SimEngine) -> AgentId {
    e.resources
        .settlements
        .values()
        .flat_map(|s| s.member_agents.iter())
        .copied()
        .min()
        .expect("a settlement member must exist after formation")
}

/// Count `AgentDecision { reason: SettlementReason }` causal events keyed to
/// `aid` across all tile ring buffers (mirrors the ζ harness scan pattern).
fn count_settlement_reason(e: &SimEngine, aid: AgentId) -> usize {
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

/// Seed a non-member outsider at `(x, y)` with needs 0 and the given loneliness.
fn seed_outsider(e: &mut SimEngine, x: u32, y: u32, loneliness: f64, rng: u64) -> Entity {
    let ent = e.spawn_agent(x, y);
    e.world
        .insert(
            ent,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(loneliness, 0.0),
                Memory::new(),
                MovementRng::new(rng),
            ),
        )
        .expect("seed outsider");
    ent
}

// ─── Assertion 1: settlement forms from founders + startup buildings ────────
#[test]
fn harness_mig_unfreeze_a1_settlement_forms() {
    // Type C — vacuity precondition (setup-dependent observation, NOT a fix
    // invariant). 3-founder cluster near the scene buildings + the 3 startup
    // buildings at (32,32),(24,32),(40,32). If 0 settlements form, every
    // downstream Idle/event assertion would pass trivially.
    let mut e = fresh_engine();
    let _founders = spawn_cluster(&mut e, 32, 32, 3);
    place_startup_buildings(&mut e);
    for _ in 0..5 {
        e.tick();
    }
    let n = e.resources.settlements.len();
    assert!(
        n >= 1,
        "A1: a settlement must form from 3 founders + 3 startup buildings; got {n}"
    );
    println!("[mig-unfreeze A1] {n} settlement(s) formed from founders + startup buildings ✓");
}

// ─── Assertion 2: non-member outsider stays Idle, no SeekTarget ─────────────
#[test]
fn harness_mig_unfreeze_a2_outsider_stays_idle() {
    // Type A — direct FSM invariant of the fix: the settlement-migration arm no
    // longer performs `*state = Seeking{Agent}`, so an outsider that reaches the
    // arm must remain Idle. Conjunct (2) NO SeekTarget rules out the
    // out-of-scope "make migration work" alternative (which would attach one).
    let mut e = fresh_engine();
    form_settlement(&mut e);
    assert!(
        !e.resources.settlements.is_empty(),
        "A2 precondition: a settlement must exist"
    );

    let outsider = seed_outsider(&mut e, 50, 50, SOCIAL_THRESHOLD - 1.0, 901);
    e.tick(); // decision tick
    assert_eq!(
        agent_state(&e, outsider),
        AgentState::Idle,
        "A2: non-member outsider must stay Idle (NOT Seeking{{Agent}}) after the fix"
    );
    assert!(
        seek_tile(&e, outsider).is_none(),
        "A2: outsider must carry NO SeekTarget (migration movement is disabled in Stage 1)"
    );
    println!("[mig-unfreeze A2] non-member outsider stays Idle, no SeekTarget ✓");
}

// ─── Assertion 3: SettlementReason intent event still emitted for outsider ──
#[test]
fn harness_mig_unfreeze_a3_settlement_intent_emitted() {
    // Type D — regression guard (p10-β A16) + the positive arm-entry proof that
    // makes A2 non-vacuous. The fix KEEPS the SettlementReason event push while
    // removing only the FSM transition; filtered by the outsider's own id.
    let mut e = fresh_engine();
    form_settlement(&mut e);
    assert!(
        !e.resources.settlements.is_empty(),
        "A3 precondition: a settlement must exist"
    );

    let outsider = seed_outsider(&mut e, 50, 50, SOCIAL_THRESHOLD - 1.0, 902);
    let outsider_id = agent_id(&e, outsider);
    e.tick();
    assert!(
        count_settlement_reason(&e, outsider_id) >= 1,
        "A3: SettlementReason intent event must still fire for the outsider (p10-β A16)"
    );
    println!("[mig-unfreeze A3] SettlementReason intent event preserved for the outsider ✓");
}

// ─── Assertion 4: settlement member emits no migration intent ───────────────
#[test]
fn harness_mig_unfreeze_a4_member_no_migration() {
    // Type A — invariant (p10-β A17): only NON-members are pulled toward a
    // settlement. A member emitting SettlementReason for its own id is a logic
    // error regardless of tuning.
    let mut e = fresh_engine();
    form_settlement(&mut e);
    let member = any_member(&e);
    e.tick(); // give the member a decision opportunity
    assert_eq!(
        count_settlement_reason(&e, member),
        0,
        "A4: a settlement member must NOT emit a SettlementReason migration intent"
    );
    println!("[mig-unfreeze A4] settlement member emits no migration intent ✓");
}

// ─── Assertion 5(a): long-run, ZERO targetless Seeking{Agent} bug signature ─
#[test]
fn harness_mig_unfreeze_a5a_long_run_no_targetless_seek() {
    // Type D — THE unfreeze guard. Bootstrap (64) + 3 startup buildings, 300
    // ticks, sampled at {50,100,150,200,250,300}. The EXACT pre-fix bug
    // signature is `Seeking{Agent}` carrying NO SeekTarget (settlement arm
    // attaches none; the ζ social arm always does). Pre-fix this hit 60 by
    // tick 27 / 64 by tick 201; post-fix it must be 0 at every sample.
    let mut e = bootstrapped();
    place_startup_buildings(&mut e);

    let samples = [50u64, 100, 150, 200, 250, 300];
    let mut max_bug = 0usize;
    for t in 1..=300u64 {
        e.tick();
        // Non-vacuity guard: the settlement that triggers migration must exist
        // (else the freeze cannot reproduce and the guard would pass trivially).
        if t == 10 {
            assert!(
                !e.resources.settlements.is_empty(),
                "A5a precondition: a settlement must form from bootstrap + startup \
                 buildings by tick 10, else the migration freeze cannot reproduce"
            );
        }
        if samples.contains(&t) {
            let mut c = 0usize;
            for (_e, (_a, state, seek)) in e
                .world
                .query::<(&Agent, &AgentState, Option<&SeekTarget>)>()
                .iter()
            {
                if matches!(state, AgentState::Seeking { target: TargetKind::Agent(_) })
                    && seek.is_none()
                {
                    c += 1;
                }
            }
            if c > max_bug {
                max_bug = c;
            }
        }
    }
    println!("[mig-unfreeze A5a] max targetless Seeking{{Agent}} across samples = {max_bug}");
    assert_eq!(
        max_bug, 0,
        "A5a: targetless Seeking{{Agent}} bug signature must be 0 at every sampled tick; got {max_bug}"
    );
    println!("[mig-unfreeze A5a] long-run targetless-seek freeze eliminated (0 at all samples) ✓");
}

// ─── Assertion 5(b): long-run, no mass positional freeze (state-filtered) ───
#[test]
fn harness_mig_unfreeze_a5b_no_motile_freeze() {
    // Type D — behavioral freeze guard. Per BOOTSTRAP agent (id < 64), track the
    // max consecutive run of "position unchanged" counted ONLY while in a MOTILE
    // state (Idle, or Seeking{_} carrying a SeekTarget); a non-motile tick
    // (Consuming, or Seeking without a target — by-design suppressed) resets the
    // streak so legitimate stationarity is not conflated with the bug freeze.
    // Newborns (id >= 64) are EXCLUDED — births spawn without MovementRng so
    // they cannot move (a SEPARATE out-of-scope bug, Stage 1.5).
    let mut e = bootstrapped();
    place_startup_buildings(&mut e);

    let mut prev: HashMap<Entity, (u32, u32)> = HashMap::new();
    let mut streak: HashMap<Entity, u32> = HashMap::new();
    let mut worst = 0u32;
    for _ in 0..300u64 {
        e.tick();
        let mut seen: HashSet<Entity> = HashSet::new();
        for (ent, (a, state, pos, seek)) in e
            .world
            .query::<(&Agent, &AgentState, &Position, Option<&SeekTarget>)>()
            .iter()
        {
            if a.id >= 64 {
                continue; // exclude newborns (no MovementRng — separate bug)
            }
            seen.insert(ent);
            let here = (pos.x, pos.y);
            let moved = prev.get(&ent).map(|p| *p != here).unwrap_or(true);
            let motile = matches!(state, AgentState::Idle)
                || (matches!(state, AgentState::Seeking { .. }) && seek.is_some());
            let s = streak.entry(ent).or_insert(0);
            if motile && !moved {
                *s += 1;
                if *s > worst {
                    worst = *s;
                }
            } else {
                *s = 0;
            }
            prev.insert(ent, here);
        }
        streak.retain(|k, _| seen.contains(k));
        prev.retain(|k, _| seen.contains(k));
    }
    println!("[mig-unfreeze A5b] worst motile-state frozen streak (bootstrap agents) = {worst}");
    assert!(
        worst < 30,
        "A5b: worst motile-state frozen streak must be < 30 ticks; got {worst}"
    );
    println!("[mig-unfreeze A5b] no mass positional freeze (worst {worst} < 30) ✓");
}

// ─── Assertion 5(c): the specific hand-placed migrant is genuinely unfrozen ─
#[test]
fn harness_mig_unfreeze_a5c_specific_migrant_moves() {
    // Type D — positive behavioral proof for the exact migrant the regression is
    // about. M-A4 proves it STAYS Idle / no SeekTarget; this proves "Idle" means
    // "unfrozen" — the migrant resumes Brownian motion within a small bound.
    // Pre-fix it was Seeking{Agent} frozen (never moved); post-fix it wanders.
    let mut e = fresh_engine();
    form_settlement(&mut e);
    let migrant = seed_outsider(&mut e, 50, 50, SOCIAL_THRESHOLD - 1.0, 777);
    let start = agent_pos(&e, migrant);

    let mut moved = false;
    for _ in 0..20u64 {
        e.tick();
        if agent_pos(&e, migrant) != start {
            moved = true;
            break;
        }
    }
    assert!(
        moved,
        "A5c: hand-placed settlement migrant must resume motion (position changes) \
         within 20 ticks — it is unfrozen, not stuck in a targetless Seeking{{Agent}}"
    );
    println!("[mig-unfreeze A5c] specific migrant moved from {start:?} (unfrozen) ✓");
}

// ─── Assertion 6: gathering loop preserved ──────────────────────────────────
#[test]
fn harness_mig_unfreeze_a6_gathering_loop_preserved() {
    // Type D — cross-phase regression guard (Section 16-α/β resource loop). The
    // fix touches only the settlement-migration arm; the needs/resource arms
    // must be untouched. Mirrors ζ A11a.
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
        "A6: Idle → Seeking{{Food}}"
    );
    dec.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        agent_state(&e, a),
        AgentState::Consuming { target: TargetKind::Food },
        "A6: Seeking{{Food}} → Consuming{{Food}}"
    );
    println!("[mig-unfreeze A6] gathering loop Idle→Seeking{{Food}}→Consuming{{Food}} preserved ✓");
}

// ─── Assertion 7: Social ζ path preserved (partner SeekTargets attached) ────
#[test]
fn harness_mig_unfreeze_a7_social_path_preserved() {
    // Type A — invariant guarding the ζ social fix against collateral damage.
    // Two social Seeking{Agent} seekers on DISTINCT tiles A@(5,5), B@(8,5), each
    // carrying the persistent-social marker. One decision tick re-resolves each
    // goal to the PARTNER's CURRENT tile (NOT the agent's own tile). Hand-placed
    // coords rule out the degenerate "target == own tile" gaming vector. This is
    // also why A5a must count TARGETLESS seeks only — targeted social seeks are
    // EXPECTED and must NOT be counted as the freeze bug.
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
                SeekTarget::new((0, 0)), // persistent social marker
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

    let at = seek_tile(&e, a).expect("A7: A must carry a SeekTarget");
    let bt = seek_tile(&e, b).expect("A7: B must carry a SeekTarget");
    assert_eq!(at, (8, 5), "A7: A's SeekTarget must be partner B's tile (8,5)");
    assert_eq!(bt, (5, 5), "A7: B's SeekTarget must be partner A's tile (5,5)");
    assert_ne!(at, (5, 5), "A7: A's SeekTarget must NOT be its own tile");
    assert_ne!(bt, (8, 5), "A7: B's SeekTarget must NOT be its own tile");
    println!("[mig-unfreeze A7] Social ζ path preserved (partner SeekTargets attached, not own) ✓");
}

// ─── Assertion 8: determinism preserved ─────────────────────────────────────
#[test]
fn harness_mig_unfreeze_a8_determinism_preserved() {
    // Type A — Day-1 invariant: removing one FSM transition must not introduce
    // nondeterminism. Two independent bootstrap + startup-building engines, 200
    // ticks, must yield byte-identical {id → (pos, state)} maps.
    type SnapMap = HashMap<AgentId, ((u32, u32), AgentState)>;
    fn run() -> SnapMap {
        let mut e = bootstrapped();
        place_startup_buildings(&mut e);
        for _ in 0..200u64 {
            e.tick();
        }
        let mut rows: Vec<(AgentId, (u32, u32), AgentState)> = Vec::new();
        for (_e, (a, p, s)) in e.world.query::<(&Agent, &Position, &AgentState)>().iter() {
            rows.push((a.id, (p.x, p.y), *s));
        }
        let mut out = SnapMap::new();
        for (id, pos, st) in rows {
            out.insert(id, (pos, st));
        }
        out
    }
    let m1 = run();
    let m2 = run();
    assert!(!m1.is_empty(), "A8: non-empty floor — bootstrap must spawn agents");
    assert_eq!(
        m1, m2,
        "A8: two independent bootstrap+building runs must yield identical {{id→(pos,state)}} maps"
    );
    println!("[mig-unfreeze A8] determinism preserved across two 200-tick runs ({} agents) ✓", m1.len());
}
