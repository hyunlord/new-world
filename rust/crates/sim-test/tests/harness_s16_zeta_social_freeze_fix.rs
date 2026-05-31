//! V7 Section 16-ζ — Social `Seeking{Agent}` freeze-trap fix harness.
//!
//! ζ gives SOCIAL `Seeking{Agent}` seekers a goal (the partner's CURRENT
//! tile, re-resolved every tick) so the existing β directed-step
//! (`movement.rs`, unchanged) walks them toward their partner instead of
//! freezing them in place forever. Settlement-migration `Seeking{Agent}`
//! seekers (the decision `else` branch) are deliberately NOT tagged and
//! receive NO `SeekTarget`, so they stay frozen by design (P10-γ owns
//! their pathing; the p10 birth tests lock that behavior).
//!
//! The distinction is made at the DECISION SOURCE: a `Seeking{Agent}`
//! transition through the social/needs/combat arm tags the entity; the
//! settlement `else` does not. The post-decision pass attaches a partner
//! `SeekTarget` only for tagged seeks (or seeks that already carry one
//! from a prior tick — the persistent tag, since only social seeks ever
//! receive a `SeekTarget`).
//!
//! Non-circular rule: every targeting assertion compares against a
//! HAND-PLACED coordinate derived from the known agent positions, never a
//! value re-read from the production pass. Distinct source/target tiles
//! rule out the degenerate "target == own tile" gaming vector.
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_zeta_social_freeze_fix -- --nocapture

use std::collections::{HashMap, HashSet};

use hecs::Entity;
use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_core::causal::event::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Memory, Position, SeekTarget, Sleep, Social, TargetKind,
    Thirst,
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

// ── helpers ─────────────────────────────────────────────────────────────────

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
/// → `bootstrap_spawn_agents` (= 64 agents). Mirrors `harness_s16_delta`.
fn bootstrapped() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    e
}

/// `Some(tile)` if `e` carries a `SeekTarget`, else `None`. Doubles as the
/// component-count check (`Some` ⇒ count 1, `None` ⇒ count 0).
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

fn loneliness(e: &SimEngine, ent: Entity) -> f64 {
    e.world.get::<&Social>(ent).expect("Social present").loneliness
}

/// Spawn `count` agents in a tight cluster near `(cx, cy)`, needs clamped
/// to 0 so no need-driven cascade arm fires. Mirrors `harness_p10_beta`.
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

/// Place `count` buildings near `(cx, cy)` via the FFI queue + drain.
/// Mirrors `harness_p10_beta`.
fn place_buildings(engine: &mut SimEngine, cx: u32, cy: u32, count: u32) {
    for i in 0..count {
        let dx = i % 3;
        let dy = i / 3;
        engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
            position: (cx + dx, cy + dy + 3),
            radius: 1,
        });
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut engine.world, &mut engine.resources);
}

/// Count `AgentBorn` causal events across all tile ring buffers.
fn count_agent_born(e: &SimEngine) -> usize {
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

// ─── Assertion 1: social pair on DISTINCT tiles → SeekTarget = PARTNER tile ─
#[test]
fn harness_s16_zeta_a1_social_pair_targets_partner_tile() {
    // Type A — existence + non-degenerate targeting. Two social Seeking{Agent}
    // seekers on DISTINCT tiles A@(5,5), B@(8,5), each already carrying a
    // SeekTarget (the persistent-social marker — only social seeks ever hold
    // one). One decision tick re-resolves each goal to the PARTNER's CURRENT
    // tile, which is NOT the agent's own tile. Hand-placed coords; the
    // distinct tiles rule out a self/own-tile-targeting impl.
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
                SeekTarget::new((0, 0)), // stale marker — persistent social tag
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

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    assert_eq!(
        agent_state(&e, a),
        AgentState::Seeking { target: TargetKind::Agent(bid) },
        "A1: A must remain Seeking{{Agent(B)}}"
    );
    assert_eq!(
        agent_state(&e, b),
        AgentState::Seeking { target: TargetKind::Agent(aid) },
        "A1: B must remain Seeking{{Agent(A)}}"
    );
    let at = seek_tile(&e, a).expect("A1: A must carry a SeekTarget");
    let bt = seek_tile(&e, b).expect("A1: B must carry a SeekTarget");
    assert_eq!(at, (8, 5), "A1: A's SeekTarget must be partner B's tile (8,5)");
    assert_eq!(bt, (5, 5), "A1: B's SeekTarget must be partner A's tile (5,5)");
    assert_ne!(at, (5, 5), "A1: A's SeekTarget must NOT be its own tile");
    assert_ne!(bt, (8, 5), "A1: B's SeekTarget must NOT be its own tile");
    println!("[S16-ζ A1] social pair → partner tiles (A→(8,5), B→(5,5)), not own ✓");
}

// ─── Assertion 2: directed step toward partner (movement.rs reused) ────────
#[test]
fn harness_s16_zeta_a2_directed_step_toward_partner() {
    // Type A — hand-computed signum: (5,5)→partner(10,5) ⇒ dx=+1, dy=0 ⇒
    // (6,5). The decision pass attaches the partner tile; the UNCHANGED
    // movement.rs consumes it for the one-tile directed step.
    let mut e = engine64();
    let seeker = e.spawn_agent(5, 5);
    let partner = e.spawn_agent(10, 5);
    let pid = agent_id(&e, partner);
    e.world
        .insert(
            seeker,
            (
                MovementRng::new(7),
                AgentState::Seeking { target: TargetKind::Agent(pid) },
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0),
                SeekTarget::new((0, 0)), // marker → decision re-resolves to partner tile
            ),
        )
        .expect("seed seeker");

    // Decision attach: re-resolve to partner's current tile (10,5).
    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        seek_tile(&e, seeker),
        Some((10, 5)),
        "A2 pre: decision must re-resolve SeekTarget to partner tile (10,5)"
    );

    // Movement consume: one signum step.
    let mut mov = AgentMovementSystem::new();
    mov.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        agent_pos(&e, seeker),
        (6, 5),
        "A2: one directed signum step toward (10,5) → (6,5)"
    );
    println!("[S16-ζ A2] decision attaches (10,5); movement steps (5,5)→(6,5) (movement.rs unchanged) ✓");
}

// ─── Assertion 3: partner SeekTarget RE-RESOLVES every tick ────────────────
#[test]
fn harness_s16_zeta_a3_partner_seektarget_reresolves() {
    // Type A — the defining social-vs-resource difference: a mobile goal,
    // re-resolved each tick. Partner moves (10,5)→(10,8); the next decision
    // tick must update the seeker's SeekTarget to (10,8).
    let mut e = engine64();
    let seeker = e.spawn_agent(5, 5);
    let partner = e.spawn_agent(10, 5);
    let pid = agent_id(&e, partner);
    e.world
        .insert(
            seeker,
            (
                AgentState::Seeking { target: TargetKind::Agent(pid) },
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0),
                SeekTarget::new((10, 5)), // already at partner's current tile
            ),
        )
        .expect("seed seeker");

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        seek_tile(&e, seeker),
        Some((10, 5)),
        "A3 pre: SeekTarget at partner's initial tile (10,5)"
    );

    // Move the partner.
    e.world
        .insert_one(partner, Position::new(10, 8))
        .expect("relocate partner");
    dec.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        seek_tile(&e, seeker),
        Some((10, 8)),
        "A3: SeekTarget must re-resolve to the partner's NEW tile (10,8)"
    );
    println!("[S16-ζ A3] partner moved (10,5)→(10,8); seeker SeekTarget re-resolved ✓");
}

// ─── Assertion 4: settlement migrant (loneliness < thr) → NO SeekTarget ────
#[test]
fn harness_s16_zeta_a4_settlement_migrant_no_target_regression_guard() {
    // Type D — THE regression guard. A non-member migrant entering
    // Seeking{Agent(member)} via the Settlement-migration arm, loneliness
    // pinned strictly BELOW threshold, must receive NO SeekTarget (it was not
    // tagged social). Isolates the SettlementReason arm as the sole cause.
    let mut e = fresh_engine();
    let cx = 20u32;
    let cy = 20u32;
    let _founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // form settlement

    let migrant = e.spawn_agent(50, 50);
    e.world
        .insert(
            migrant,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(SOCIAL_THRESHOLD - 1.0, 0.0), // BELOW threshold
                Memory::new(),
                MovementRng::new(123),
            ),
        )
        .expect("seed migrant");
    assert!(
        loneliness(&e, migrant) < SOCIAL_THRESHOLD,
        "A4 precondition: migrant loneliness must be below the social breach constant"
    );

    e.tick(); // decision: SettlementReason → Seeking{Agent(member)}
    assert!(
        matches!(
            agent_state(&e, migrant),
            AgentState::Seeking { target: TargetKind::Agent(_) }
        ),
        "A4: migrant must be Seeking{{Agent}} via the settlement arm"
    );
    assert!(
        seek_tile(&e, migrant).is_none(),
        "A4: settlement migrant must NOT receive a SeekTarget (count == 0)"
    );
    println!("[S16-ζ A4] settlement migrant (loneliness<thr) → Seeking{{Agent}}, no SeekTarget ✓");
}

// ─── Assertion 5: manually-placed untagged Seeking{Agent} → no target ──────
#[test]
fn harness_s16_zeta_a5_untagged_manual_seek_no_target() {
    // Type A — proves the SOURCE (the social tag), not the bare Seeking{Agent}
    // state, gates the attach. Manually set state (no decision transition), no
    // prior SeekTarget, loneliness BELOW threshold ⇒ no target attached.
    let mut e = engine64();
    let a = e.spawn_agent(20, 20);
    let other = e.spawn_agent(40, 40);
    let oid = agent_id(&e, other);
    e.world
        .insert(
            a,
            (
                AgentState::Seeking { target: TargetKind::Agent(oid) },
                Social::new(SOCIAL_THRESHOLD - 1.0, 0.0), // BELOW threshold
            ),
        )
        .expect("seed a");
    assert!(
        loneliness(&e, a) < SOCIAL_THRESHOLD,
        "A5 precondition: loneliness must be below the social breach constant"
    );

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    assert!(
        seek_tile(&e, a).is_none(),
        "A5: untagged manually-placed Seeking{{Agent}} must get NO SeekTarget"
    );
    println!("[S16-ζ A5] untagged manual Seeking{{Agent}} (no prior target) → no target ✓");
}

// ─── Assertion 6: SeekTarget cleared on exit from social seek ──────────────
#[test]
fn harness_s16_zeta_a6_cleared_on_exit() {
    // Type A — lifecycle invariant: a stale goal must not persist once the
    // agent stops social-seeking. Two sub-cases: →Idle and →Consuming{Agent}.
    {
        // Sub-case (a): forced to Idle.
        let mut e = engine64();
        let a = e.spawn_agent(10, 10);
        e.world
            .insert(a, (AgentState::Idle, SeekTarget::new((15, 10))))
            .expect("seed idle-with-stale-target");
        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert!(
            seek_tile(&e, a).is_none(),
            "A6a: Idle agent's stale SeekTarget must be cleared"
        );
    }
    {
        // Sub-case (b): forced to Consuming{Agent}.
        let mut e = engine64();
        let a = e.spawn_agent(10, 10);
        let p = e.spawn_agent(15, 10);
        let pid = agent_id(&e, p);
        e.world
            .insert(
                a,
                (
                    AgentState::Consuming { target: TargetKind::Agent(pid) },
                    SeekTarget::new((15, 10)),
                ),
            )
            .expect("seed consuming-with-stale-target");
        let mut dec = AgentDecisionSystem::new();
        dec.tick(&mut e.world, &mut e.resources);
        assert!(
            seek_tile(&e, a).is_none(),
            "A6b: Consuming{{Agent}} agent's stale SeekTarget must be cleared"
        );
    }
    println!("[S16-ζ A6] SeekTarget cleared on exit to Idle AND Consuming{{Agent}} ✓");
}

// ─── Assertion 7: resource set-once UNCHANGED across boundary crossing ─────
#[test]
fn harness_s16_zeta_a7_resource_set_once_unchanged() {
    // Type D — α stability guard. Two food tiles (10,10) & (20,10). Agent at
    // (12,10): nearest food (10,10) (dist 2 < 8). After moving to (16,10) a
    // DIFFERENT tile (20,10) is now nearer (dist 4 < 6), yet the resource
    // SeekTarget is set-once and must stay (10,10).
    let mut e = engine64();
    let a = e.spawn_agent(12, 10);
    e.world
        .insert(a, (AgentState::Idle, Hunger::new(51.0, 0.0)))
        .expect("seed hungry");
    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);
    e.resources.set_food_tile(20, 10, RESOURCE_SOURCE_INFINITE);

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        agent_state(&e, a),
        AgentState::Seeking { target: TargetKind::Food },
        "A7: Idle→Seeking{{Food}}"
    );
    assert_eq!(
        seek_tile(&e, a),
        Some((10, 10)),
        "A7: nearest food at start is (10,10)"
    );

    // Cross the nearest-tile boundary: (16,10)→(10,10)=6 > (16,10)→(20,10)=4.
    e.world
        .insert_one(a, Position::new(16, 10))
        .expect("move agent across boundary");
    dec.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        seek_tile(&e, a),
        Some((10, 10)),
        "A7: resource SeekTarget is set-once — must stay (10,10) despite (20,10) now nearer"
    );
    println!("[S16-ζ A7] resource set-once preserved across nearest-tile boundary crossing ✓");
}

// ─── Assertion 8: long-run social freeze eliminated (THE actual bug) ───────
#[test]
fn harness_s16_zeta_a8_long_run_social_freeze_eliminated() {
    // Type C — empirical. 64-agent bootstrap, 8000 ticks. Per agent, track the
    // max consecutive run of: (a) Seeking{Agent}, (b) ≥2 Manhattan tiles from
    // partner, AND (c) not moving (pos unchanged from prior tick). Pre-fix:
    // worst streak 5678 ticks, 5/64 trapped at tick 8000. Threshold: worst <
    // 100 AND permanently-trapped == 0.
    let mut e = bootstrapped();
    let mut prev_pos: HashMap<Entity, (u32, u32)> = HashMap::new();
    let mut cur_streak: HashMap<Entity, u32> = HashMap::new();
    let mut max_streak: u32 = 0;

    const TICKS: u64 = 8000;
    for _ in 0..TICKS {
        e.tick();

        // agent_id → current position snapshot (partner lookup surface).
        let mut pos_by_id: HashMap<AgentId, (u32, u32)> = HashMap::new();
        for (_, (a, p)) in e.world.query::<(&Agent, &Position)>().iter() {
            pos_by_id.insert(a.id, (p.x, p.y));
        }

        let mut seen: HashSet<Entity> = HashSet::new();
        for (ent, (_a, state, p)) in e.world.query::<(&Agent, &AgentState, &Position)>().iter() {
            seen.insert(ent);
            let here = (p.x, p.y);
            let moved = prev_pos.get(&ent).map(|prev| *prev != here).unwrap_or(true);

            let mut frozen_far = false;
            if let AgentState::Seeking { target: TargetKind::Agent(pid) } = *state {
                if let Some(partner) = pos_by_id.get(&pid) {
                    let dist = (here.0 as i64 - partner.0 as i64).abs()
                        + (here.1 as i64 - partner.1 as i64).abs();
                    if dist >= 2 && !moved {
                        frozen_far = true;
                    }
                }
            }

            let s = cur_streak.entry(ent).or_insert(0);
            if frozen_far {
                *s += 1;
            } else {
                *s = 0;
            }
            if *s > max_streak {
                max_streak = *s;
            }
            prev_pos.insert(ent, here);
        }

        // Forget vanished entities (no deaths expected in bootstrap; defensive).
        cur_streak.retain(|k, _| seen.contains(k));
        prev_pos.retain(|k, _| seen.contains(k));
    }

    // Permanently-trapped: a freeze streak still ongoing AND already long
    // (≥100) at the final tick — the 5/64 stuck-at-8000 symptom.
    let trapped = cur_streak.values().filter(|s| **s >= 100).count();
    println!(
        "[S16-ζ A8] worst far-from-partner non-moving Seeking{{Agent}} streak = {max_streak}, permanently-trapped = {trapped}"
    );
    assert!(
        max_streak < 100,
        "A8: worst far-from-partner non-moving Seeking{{Agent}} streak must be < 100; got {max_streak}"
    );
    assert_eq!(
        trapped, 0,
        "A8: no agent may be permanently trapped at tick {TICKS}; got {trapped}"
    );
    println!("[S16-ζ A8] long-run social freeze eliminated (worst {max_streak} < 100, 0 trapped) ✓");
}

// ─── Assertion 9: settlement births still fire (cross-feature regression) ──
#[test]
fn harness_s16_zeta_a9_settlement_births_still_fire() {
    // Type D — the exact dynamics the naive fix broke. 3-founder cluster + 2
    // buildings, run past BIRTH_COOLDOWN_TICKS, assert ≥1 AgentBorn.
    let mut e = fresh_engine();
    let cx = 26u32;
    let cy = 26u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize + 1) {
        e.tick();
    }
    let births = count_agent_born(&e);
    assert!(
        births >= 1,
        "A9: settlement births must still fire after the ζ social fix; got {births}"
    );
    println!("[S16-ζ A9] {births} AgentBorn after cooldown — settlement dynamics preserved ✓");
}

// ─── Assertion 10: determinism preserved ───────────────────────────────────
#[test]
fn harness_s16_zeta_a10_determinism_preserved() {
    // Type A — seeded splitmix64 (no wall-clock). Two independent 1500-tick
    // bootstraps must yield byte-identical {Agent.id → (pos, SeekTarget)} maps.
    type SnapMap = HashMap<AgentId, ((u32, u32), Option<(u32, u32)>)>;
    fn run() -> SnapMap {
        let mut e = bootstrapped();
        for _ in 0..1500 {
            e.tick();
        }
        // Collect entities first (avoid nested get during the query borrow).
        let mut rows: Vec<(Entity, AgentId, (u32, u32))> = Vec::new();
        for (ent, (a, p)) in e.world.query::<(&Agent, &Position)>().iter() {
            rows.push((ent, a.id, (p.x, p.y)));
        }
        let mut out = HashMap::new();
        for (ent, id, pos) in rows {
            let st = e.world.get::<&SeekTarget>(ent).ok().map(|s| s.tile);
            out.insert(id, (pos, st));
        }
        out
    }
    let m1 = run();
    let m2 = run();
    assert!(!m1.is_empty(), "A10: non-empty floor — bootstrap must spawn agents");
    assert_eq!(
        m1, m2,
        "A10: two independent bootstraps must yield identical {{id→(pos,seek)}} maps"
    );
    println!("[S16-ζ A10] determinism preserved across two 1500-tick bootstraps ({} agents) ✓", m1.len());
}

// ─── Assertion 11: gathering loop + α0 substrate preserved ─────────────────
#[test]
fn harness_s16_zeta_a11_gathering_loop_and_substrate_preserved() {
    // Type D — cross-phase regression guard. ζ touches only the social branch.
    {
        // (a) gathering loop Idle → Seeking{Food} → Consuming{Food}.
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
            "A11a: Idle→Seeking{{Food}}"
        );
        dec.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            agent_state(&e, a),
            AgentState::Consuming { target: TargetKind::Food },
            "A11a: Seeking{{Food}}→Consuming{{Food}}"
        );
    }
    {
        // (b) α0 substrate: 12 source tiles, all u8::MAX, after bootstrap.
        let e = bootstrapped();
        let nf = e.resources.food_tiles.len();
        let nw = e.resources.water_tiles.len();
        let ns = e.resources.sleep_tiles.len();
        assert_eq!(nf + nw + ns, 12, "A11b: combined source-tile count must be 12");
        for map in [
            &e.resources.food_tiles,
            &e.resources.water_tiles,
            &e.resources.sleep_tiles,
        ] {
            for ((x, y), v) in map.iter() {
                assert_eq!(*v, RESOURCE_SOURCE_INFINITE, "A11b: source ({x},{y}) must be u8::MAX");
            }
        }
    }
    println!("[S16-ζ A11] gathering loop intact + α0 substrate (12 sources @u8::MAX) preserved ✓");
}

// ─── Assertion 12: self-seek guard — Seeking{Agent(own_id)} → no target ────
#[test]
fn harness_s16_zeta_a12_self_seek_guard() {
    // Type A — an agent chasing ITSELF must attach no SeekTarget, must not
    // panic, and must not take a self-directed movement step. Hand-set id
    // equality (partner id == own id).
    let mut e = engine64();
    let a = e.spawn_agent(10, 10);
    let aid = agent_id(&e, a);
    e.world
        .insert(
            a,
            (
                MovementRng::new(7),
                AgentState::Seeking { target: TargetKind::Agent(aid) }, // chasing SELF
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0),
                SeekTarget::new((20, 20)), // stale marker — self-guard must clear it
            ),
        )
        .expect("seed self-seeker");

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources); // must not panic
    assert!(
        seek_tile(&e, a).is_none(),
        "A12: self-seek (pid == own id) must attach NO SeekTarget"
    );

    // No SeekTarget + a non-Idle state → no directed self-step on movement.
    let before = agent_pos(&e, a);
    let mut mov = AgentMovementSystem::new();
    mov.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        agent_pos(&e, a),
        before,
        "A12: self-seeker must not take a self-directed step (position unchanged)"
    );
    println!("[S16-ζ A12] self-seek guard: no target, no panic, no self-step ✓");
}

// ─── Assertion 13: partner-despawn clearing — stale goal removed, no panic ─
#[test]
fn harness_s16_zeta_a13_partner_despawn_clearing() {
    // Type A — a SeekTarget/partner lookup against a despawned id is a
    // stale-goal + panic risk. Despawn the partner, run one decision tick:
    // the seeker's SeekTarget must be cleared and the tick must not panic.
    let mut e = engine64();
    let seeker = e.spawn_agent(5, 5);
    let partner = e.spawn_agent(10, 5);
    let pid = agent_id(&e, partner);
    e.world
        .insert(
            seeker,
            (
                AgentState::Seeking { target: TargetKind::Agent(pid) },
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0),
                SeekTarget::new((10, 5)),
            ),
        )
        .expect("seed seeker");

    e.world.despawn(partner).expect("despawn partner");

    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources); // must not panic on missing-id lookup
    assert!(
        seek_tile(&e, seeker).is_none(),
        "A13: SeekTarget pointing at a despawned partner must be cleared"
    );
    println!("[S16-ζ A13] partner-despawn → stale SeekTarget cleared, no lookup panic ✓");
}

// ─── Assertion 14: lonely settlement migrant → settlement arm wins ─────────
#[test]
fn harness_s16_zeta_a14_lonely_settlement_migrant_no_target() {
    // Type D — the boundary the naive fix lived on: a migrant that is BOTH a
    // Settlement migrant AND lonely (loneliness > thr). The settlement source
    // governs → no partner SeekTarget; it stays frozen by design (P10-γ).
    let mut e = fresh_engine();
    let cx = 20u32;
    let cy = 20u32;
    let _founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // form settlement

    let migrant = e.spawn_agent(50, 50);
    e.world
        .insert(
            migrant,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(SOCIAL_THRESHOLD + 1.0, 0.0), // ABOVE threshold
                Memory::new(),
                MovementRng::new(321),
            ),
        )
        .expect("seed lonely migrant");
    assert!(
        loneliness(&e, migrant) > SOCIAL_THRESHOLD,
        "A14 precondition: migrant loneliness must be above the social breach constant"
    );

    e.tick(); // decision: settlement arm (no co-located social peer at (50,50))
    assert!(
        matches!(
            agent_state(&e, migrant),
            AgentState::Seeking { target: TargetKind::Agent(_) }
        ),
        "A14: migrant must be Seeking{{Agent}} (settlement arm)"
    );
    assert!(
        seek_tile(&e, migrant).is_none(),
        "A14: lonely settlement migrant must NOT get a social SeekTarget (settlement source wins)"
    );
    println!("[S16-ζ A14] lonely settlement migrant → settlement arm wins, no SeekTarget ✓");
}
