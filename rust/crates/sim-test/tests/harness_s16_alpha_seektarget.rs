//! V7 Section 16-α — SeekTarget (nearest-resource targeting) harness.
//!
//! Verifies the α targeting layer on top of the α0 substrate
//! (`53075aff`): the `AgentDecisionSystem` post-decision pass attaches a
//! `SeekTarget` (the nearest matching resource tile, by Manhattan
//! distance with an `(x, y)` tie-break) to every agent now
//! `Seeking{Food/Water/Sleep}` that lacks one, and clears it from any
//! agent no longer seeking a resource. α produces NO movement and NO
//! visible change — `SeekTarget` is internal state β consumes.
//!
//! Non-circular rule (α0 lesson): every `SeekTarget.tile` /
//! `nearest_resource_tile` assertion compares against a HAND-WRITTEN
//! expected coordinate derived from the known tile positions, never a
//! value re-derived from the same map. The independent min-distance
//! checks defeat the "return first-iterated tile" cheat.
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_alpha_seektarget -- --nocapture

use std::collections::HashMap;

use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_core::components::{
    Agent, AgentState, Hunger, Position, SeekTarget, Sleep, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::decision::{nearest_resource_tile, AgentDecisionSystem};

// ── helpers ─────────────────────────────────────────────────────────────────

/// Fresh 64×64 engine (large enough for every hand-placed tile below).
fn engine64() -> SimEngine {
    SimEngine::new(64, 64, MaterialRegistry::new())
}

/// `Some(tile)` if `e` carries a `SeekTarget`, else `None`. Doubles as the
/// "component count" check: `Some` ⇒ count 1, `None` ⇒ count 0.
fn seek_tile(engine: &SimEngine, e: hecs::Entity) -> Option<(u32, u32)> {
    engine.world.get::<&SeekTarget>(e).ok().map(|st| st.tile)
}

fn agent_state(engine: &SimEngine, e: hecs::Entity) -> AgentState {
    *engine.world.get::<&AgentState>(e).expect("AgentState present")
}

fn manhattan(a: (u32, u32), b: (u32, u32)) -> i64 {
    (a.0 as i64 - b.0 as i64).abs() + (a.1 as i64 - b.1 as i64).abs()
}

/// Minimum Manhattan distance from `pos` to any tile in `tiles`.
/// Computed independently of the production routine (non-circular).
fn min_manhattan_over(pos: (u32, u32), tiles: &HashMap<(u32, u32), u8>) -> i64 {
    tiles
        .keys()
        .map(|t| manhattan(pos, *t))
        .min()
        .expect("non-empty tile map")
}

// ─── Assertion 1: SeekTarget assigned on a hunger-driven seek ──────────────
#[test]
fn harness_seektarget_assigned_on_hunger_seek() {
    // Type: A — existence invariant. Idle + Hunger>THRESHOLD + food present
    // ⇒ exactly one SeekTarget after one decision tick.
    let mut e = engine64();
    let agent = e.spawn_agent(12, 12);
    e.world
        .insert(agent, (AgentState::Idle, Hunger::new(51.0, 0.0)))
        .expect("seed agent");
    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    assert_eq!(
        agent_state(&e, agent),
        AgentState::Seeking { target: TargetKind::Food },
        "A1: agent must transition Idle→Seeking{{Food}}"
    );
    assert!(
        seek_tile(&e, agent).is_some(),
        "A1: a Seeking{{Food}} agent with non-empty food_tiles MUST carry a SeekTarget"
    );
    println!("[S16-α A1] Idle+Hunger51 → Seeking{{Food}} + SeekTarget present ✓");
}

// ─── Assertion 2: nearest food tile, min-distance verified (non-circular) ──
#[test]
fn harness_nearest_food_tile_min_distance_verified() {
    // Type: A — hand-computed: dist (12,12)→(10,10)=4 < →(50,50)=76.
    let mut e = engine64();
    let agent = e.spawn_agent(12, 12);
    e.world
        .insert(agent, (AgentState::Idle, Hunger::new(51.0, 0.0)))
        .expect("seed agent");
    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);
    e.resources.set_food_tile(50, 50, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    let chosen = seek_tile(&e, agent).expect("A2: SeekTarget must be present");
    assert_eq!(chosen, (10, 10), "A2: nearest food tile must be the hand-computed (10,10)");
    // Independent strict-minimum check (defeats return-first cheat).
    let chosen_d = manhattan((12, 12), chosen);
    assert_eq!(
        chosen_d,
        min_manhattan_over((12, 12), &e.resources.food_tiles),
        "A2: chosen tile's distance must be the strict minimum over food_tiles"
    );
    assert!(
        manhattan((12, 12), (10, 10)) < manhattan((12, 12), (50, 50)),
        "A2: 4 < 76 must hold (hand-computed)"
    );
    println!("[S16-α A2] nearest food = (10,10), dist 4 = min over map ✓");
}

// ─── Assertion 3: nearest water with a closer confounding food tile ────────
#[test]
fn harness_nearest_water_tile_routing_with_confounder() {
    // Type: A — water (5,5) dist 3, (40,40) dist 67; food confounder (6,6)
    // dist 2 is strictly closer but the WRONG kind and must be ignored.
    let mut e = engine64();
    let agent = e.spawn_agent(6, 7);
    e.world
        .insert(agent, (AgentState::Seeking { target: TargetKind::Water },))
        .expect("seed agent");
    e.resources.set_water_tile(40, 40, RESOURCE_SOURCE_INFINITE);
    e.resources.set_water_tile(5, 5, RESOURCE_SOURCE_INFINITE);
    e.resources.set_food_tile(6, 6, RESOURCE_SOURCE_INFINITE); // confounder, dist 2

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    let chosen = seek_tile(&e, agent).expect("A3: SeekTarget must be present");
    assert_eq!(chosen, (5, 5), "A3: nearest WATER tile must be (5,5)");
    assert_ne!(chosen, (6, 6), "A3: must NOT pick the closer food confounder (6,6)");
    assert_eq!(
        manhattan((6, 7), chosen),
        min_manhattan_over((6, 7), &e.resources.water_tiles),
        "A3: chosen tile must be the min over water_tiles specifically"
    );
    println!("[S16-α A3] Seeking{{Water}} → (5,5), ignores closer food (6,6) ✓");
}

// ─── Assertion 4: nearest sleep with closer food + water confounders ───────
#[test]
fn harness_nearest_sleep_tile_routing_with_confounder() {
    // Type: A — sleep (20,20) dist 4, (60,4) dist 52; food (21,18) dist 1 and
    // water (23,17) dist 2 are closer but wrong-kind and must be ignored.
    let mut e = engine64();
    let agent = e.spawn_agent(22, 18);
    e.world
        .insert(agent, (AgentState::Seeking { target: TargetKind::Sleep },))
        .expect("seed agent");
    e.resources.set_sleep_tile(20, 20, RESOURCE_SOURCE_INFINITE);
    e.resources.set_sleep_tile(60, 4, RESOURCE_SOURCE_INFINITE);
    e.resources.set_food_tile(21, 18, RESOURCE_SOURCE_INFINITE); // dist 1
    e.resources.set_water_tile(23, 17, RESOURCE_SOURCE_INFINITE); // dist 2

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    let chosen = seek_tile(&e, agent).expect("A4: SeekTarget must be present");
    assert_eq!(chosen, (20, 20), "A4: nearest SLEEP tile must be (20,20)");
    assert!(
        chosen != (21, 18) && chosen != (23, 17),
        "A4: must reject the food/water confounders"
    );
    assert_eq!(
        manhattan((22, 18), chosen),
        min_manhattan_over((22, 18), &e.resources.sleep_tiles),
        "A4: chosen tile must be the min over sleep_tiles specifically"
    );
    println!("[S16-α A4] Seeking{{Sleep}} → (20,20), ignores closer food/water ✓");
}

// ─── Assertion 5: (x,y)-min tie-break, deterministic over 8 fresh builds ───
#[test]
fn harness_tie_break_xy_min_deterministic_repeated() {
    // Type: A — food (8,10) & (12,10) both dist 2 from (10,10); the (dist,x,y)
    // tie-break must pick (8,10) on EVERY independent fresh build.
    for rep in 0..8 {
        let mut e = engine64();
        let agent = e.spawn_agent(10, 10);
        e.world
            .insert(agent, (AgentState::Seeking { target: TargetKind::Food },))
            .expect("seed agent");
        e.resources.set_food_tile(8, 10, RESOURCE_SOURCE_INFINITE);
        e.resources.set_food_tile(12, 10, RESOURCE_SOURCE_INFINITE);

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let chosen = seek_tile(&e, agent).expect("A5: SeekTarget must be present");
        assert_eq!(
            chosen, (8, 10),
            "A5: rep {rep}: equidistant tie must resolve to (x,y)-min (8,10); got {chosen:?}"
        );
    }
    println!("[S16-α A5] (8,10) chosen on all 8 fresh builds (tie-break stable) ✓");
}

// ─── Assertion 6: cross-engine seed-identical SeekTargets, non-empty floor ─
#[test]
fn harness_cross_engine_identical_targets_nonempty() {
    // Type: A — deterministic sim: identical setup ⇒ identical SeekTargets,
    // keyed by Agent.id (stable identity, not query order). len >= 1 floor.
    fn build_and_collect() -> HashMap<u64, (u32, u32)> {
        let mut e = engine64();
        // Three hungry agents at distinct positions; two food tiles.
        let positions = [(12, 12), (30, 30), (45, 5)];
        let mut ents = Vec::new();
        for (x, y) in positions {
            let a = e.spawn_agent(x, y);
            e.world
                .insert(a, (AgentState::Idle, Hunger::new(51.0, 0.0)))
                .expect("seed agent");
            ents.push(a);
        }
        e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);
        e.resources.set_food_tile(50, 50, RESOURCE_SOURCE_INFINITE);

        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let mut out = HashMap::new();
        for a in ents {
            let id = e.world.get::<&Agent>(a).expect("Agent id").id;
            if let Some(tile) = seek_tile(&e, a) {
                out.insert(id, tile);
            }
        }
        out
    }

    let a = build_and_collect();
    let b = build_and_collect();
    assert!(
        !a.is_empty(),
        "A6: non-empty floor — engine_a must produce >= 1 SeekTarget"
    );
    assert_eq!(a, b, "A6: identical seed setup must yield identical SeekTargets by id");
    println!("[S16-α A6] cross-engine SeekTargets identical ({} entries) ✓", a.len());
}

// ─── Assertion 7: cleared when transitioning to Consuming ──────────────────
#[test]
fn harness_cleared_when_no_longer_seeking_resource() {
    // Type: A — agent ON its tile transitions Seeking{Food}→Consuming{Food}
    // this tick; the pre-attached SeekTarget MUST be removed.
    let mut e = engine64();
    let agent = e.spawn_agent(10, 10);
    e.world
        .insert(
            agent,
            (
                AgentState::Seeking { target: TargetKind::Food },
                Hunger::new(80.0, 0.0),
                SeekTarget::new((99, 99)), // stale goal that must disappear
            ),
        )
        .expect("seed agent");
    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    assert_eq!(
        agent_state(&e, agent),
        AgentState::Consuming { target: TargetKind::Food },
        "A7: agent must reach Consuming{{Food}}"
    );
    assert!(
        seek_tile(&e, agent).is_none(),
        "A7: SeekTarget must be cleared once the agent stops Seeking a resource"
    );
    println!("[S16-α A7] Seeking→Consuming clears the stale SeekTarget ✓");
}

// ─── Assertion 8: no target for Construction / Agent seeks (precondition) ──
#[test]
fn harness_no_target_for_construction_or_agent_seek() {
    // Type: A — scope boundary: ConstructionSite/Agent targets are co-located,
    // not resource tiles, and must never get a resource SeekTarget. State
    // precondition guards against a vacuous pass.
    let mut e = engine64();
    let a = e.spawn_agent(30, 30);
    e.world
        .insert(a, (AgentState::Seeking { target: TargetKind::ConstructionSite },))
        .expect("seed agent a");
    let b = e.spawn_agent(40, 40);
    e.world
        .insert(b, (AgentState::Seeking { target: TargetKind::Agent(999_999) },))
        .expect("seed agent b");
    // Resource tiles of every kind exist — yet neither must get a target.
    e.resources.set_food_tile(31, 30, RESOURCE_SOURCE_INFINITE);
    e.resources.set_water_tile(41, 40, RESOURCE_SOURCE_INFINITE);
    e.resources.set_sleep_tile(20, 20, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    // PRECONDITION: both agents are actually in the intended Seeking variant.
    assert_eq!(
        agent_state(&e, a),
        AgentState::Seeking { target: TargetKind::ConstructionSite },
        "A8 precondition: agent a must be Seeking{{ConstructionSite}}"
    );
    assert_eq!(
        agent_state(&e, b),
        AgentState::Seeking { target: TargetKind::Agent(999_999) },
        "A8 precondition: agent b must be Seeking{{Agent(_)}}"
    );
    assert!(seek_tile(&e, a).is_none(), "A8: ConstructionSite seek must get NO SeekTarget");
    assert!(seek_tile(&e, b).is_none(), "A8: Agent seek must get NO SeekTarget");
    println!("[S16-α A8] Construction/Agent seeks receive no resource SeekTarget ✓");
}

// ─── Assertion 9: nearest_resource_tile unit (non-circular) + empty=None ───
#[test]
fn harness_nearest_resource_tile_unit_direct() {
    // Type: A — direct pure-fn call; hand-computed (4,2)→(3,3)=2 is the min
    // over {(3,3)=2,(9,1)=6,(40,40)=74}. Empty map ⇒ None (sole None case).
    let mut tiles: HashMap<(u32, u32), u8> = HashMap::new();
    tiles.insert((3, 3), RESOURCE_SOURCE_INFINITE);
    tiles.insert((9, 1), RESOURCE_SOURCE_INFINITE);
    tiles.insert((40, 40), RESOURCE_SOURCE_INFINITE);
    let pos = Position::new(4, 2);

    let got = nearest_resource_tile(&pos, &tiles);
    assert_eq!(got, Some((3, 3)), "A9: nearest must be the hand-computed (3,3)");
    assert_eq!(
        manhattan((4, 2), got.unwrap()),
        min_manhattan_over((4, 2), &tiles),
        "A9: returned tile must achieve the independent min distance"
    );

    let empty: HashMap<(u32, u32), u8> = HashMap::new();
    assert_eq!(
        nearest_resource_tile(&pos, &empty),
        None,
        "A9: empty map must return None (documented contract)"
    );
    println!("[S16-α A9] nearest_resource_tile((4,2)) = Some((3,3)); empty = None ✓");
}

// ─── Assertion 10: SeekTarget serde round-trip ─────────────────────────────
#[test]
fn harness_seektarget_serde_roundtrip() {
    // Type: A — all sim-core components derive serde for save/load parity.
    let st = SeekTarget::new((7, 42));
    let encoded = ron::to_string(&st).expect("SeekTarget must Serialize");
    let decoded: SeekTarget = ron::from_str(&encoded).expect("SeekTarget must Deserialize");
    assert_eq!(st, decoded, "A10: RON round-trip must be exact");
    println!("[S16-α A10] SeekTarget((7,42)) RON round-trip exact ✓");
}

// ─── Assertion 11: α0 substrate preserved (cross-phase regression guard) ───
#[test]
fn harness_alpha0_substrate_preserved() {
    // Type: D — bootstrap must still seed 12 non-depleting source tiles
    // (4 food + 4 water + 4 sleep), every value == u8::MAX. α reads these.
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    bootstrap_spawn_agents(&mut engine);

    let nf = engine.resources.food_tiles.len();
    let nw = engine.resources.water_tiles.len();
    let ns = engine.resources.sleep_tiles.len();
    assert!(nf >= 1 && nw >= 1 && ns >= 1, "A11: every resource map must be non-empty");
    for map in [
        &engine.resources.food_tiles,
        &engine.resources.water_tiles,
        &engine.resources.sleep_tiles,
    ] {
        for ((x, y), v) in map.iter() {
            assert_eq!(*v, RESOURCE_SOURCE_INFINITE, "A11: source ({x},{y}) must be u8::MAX");
        }
    }
    // Exact-count guard (brittle by design; relax to >= 12 if α0 grows).
    assert_eq!(nf + nw + ns, 12, "A11: combined source-tile count must be 12 (4+4+4)");
    println!("[S16-α A11] bootstrap substrate intact: {nf}+{nw}+{ns}=12 sources at u8::MAX ✓");
}

// ─── Assertion 12: below-threshold gets no target; FSM intact ──────────────
#[test]
fn harness_idle_below_thresholds_no_target_and_fsm_intact() {
    // Type: D — regression guard for the added pass.
    // (a) Idle agent below all thresholds → no SeekTarget, stays Idle.
    let mut e = engine64();
    let calm = e.spawn_agent(5, 5);
    e.world
        .insert(
            calm,
            (
                AgentState::Idle,
                Hunger::new(10.0, 0.0),
                Thirst::new(10.0, 0.0),
                Sleep::new(10.0, 0.0),
            ),
        )
        .expect("seed calm agent");
    e.resources.set_food_tile(40, 40, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);
    assert_eq!(agent_state(&e, calm), AgentState::Idle, "A12a: calm agent must stay Idle");
    assert!(seek_tile(&e, calm).is_none(), "A12a: below-threshold agent gets no SeekTarget");

    // (b) Co-located breached agent completes Idle→Seeking→Consuming and
    // ends with no SeekTarget (existing FSM unbroken by the pass).
    let mut e2 = engine64();
    let hungry = e2.spawn_agent(10, 10);
    e2.world
        .insert(hungry, (AgentState::Idle, Hunger::new(51.0, 0.0)))
        .expect("seed hungry agent");
    e2.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);

    let mut sys2 = AgentDecisionSystem::new();
    // Tick 1: Idle → Seeking{Food} (pass attaches SeekTarget at (10,10)).
    sys2.tick(&mut e2.world, &mut e2.resources);
    assert_eq!(
        agent_state(&e2, hungry),
        AgentState::Seeking { target: TargetKind::Food },
        "A12b: tick 1 must reach Seeking{{Food}}"
    );
    assert_eq!(
        seek_tile(&e2, hungry),
        Some((10, 10)),
        "A12b: co-located Seeking agent targets its own tile (10,10)"
    );
    // Tick 2: Seeking{Food} on tile → Consuming{Food}; pass clears SeekTarget.
    sys2.tick(&mut e2.world, &mut e2.resources);
    assert_eq!(
        agent_state(&e2, hungry),
        AgentState::Consuming { target: TargetKind::Food },
        "A12b: tick 2 must reach Consuming{{Food}}"
    );
    assert!(
        seek_tile(&e2, hungry).is_none(),
        "A12b: agent ends with no SeekTarget after reaching Consuming"
    );
    println!("[S16-α A12] below-threshold no target; Idle→Seeking→Consuming intact ✓");
}

// ─── Assertion 13: threshold boundary exactly 50.0 (`>` predicate) ─────────
#[test]
fn harness_threshold_boundary_exact_50() {
    // Type: B — locked α0 predicate is strict `>` HUNGER_THRESHOLD(50.0).
    // X at exactly 50.0 must NOT seek; Y at 50.001 must seek.
    let mut e = engine64();
    let x = e.spawn_agent(12, 12);
    e.world
        .insert(x, (AgentState::Idle, Hunger::new(50.0, 0.0)))
        .expect("seed X");
    let y = e.spawn_agent(20, 20);
    e.world
        .insert(y, (AgentState::Idle, Hunger::new(50.001, 0.0)))
        .expect("seed Y");
    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    // Y (strictly above): seeks + target.
    assert_eq!(
        agent_state(&e, y),
        AgentState::Seeking { target: TargetKind::Food },
        "A13: Y (50.001 > 50.0) must seek"
    );
    assert!(seek_tile(&e, y).is_some(), "A13: Y must carry a SeekTarget");
    // X (exactly at boundary): stays Idle, no target.
    assert_eq!(agent_state(&e, x), AgentState::Idle, "A13: X (==50.0) must NOT seek (`>` predicate)");
    assert!(seek_tile(&e, x).is_none(), "A13: X must have no SeekTarget");
    println!("[S16-α A13] boundary `>`: X@50.0 idle, Y@50.001 seeks ✓");
}

// ─── Assertion 14: three distinct-kind agents in a single tick ─────────────
#[test]
fn harness_multi_agent_single_tick_distinct_kinds() {
    // Type: A — one Hunger / one Thirst / one Fatigue agent in ONE tick;
    // each must get the hand-computed nearest tile of ITS OWN kind, with no
    // cross-kind contamination.
    let mut e = engine64();
    let a_food = e.spawn_agent(12, 12);
    e.world
        .insert(a_food, (AgentState::Idle, Hunger::new(51.0, 0.0)))
        .expect("seed food agent");
    let a_water = e.spawn_agent(6, 7);
    e.world
        .insert(
            a_water,
            (AgentState::Idle, Hunger::new(0.0, 0.0), Thirst::new(51.0, 0.0)),
        )
        .expect("seed water agent");
    let a_sleep = e.spawn_agent(22, 18);
    e.world
        .insert(
            a_sleep,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(51.0, 0.0),
            ),
        )
        .expect("seed sleep agent");

    e.resources.set_food_tile(10, 10, RESOURCE_SOURCE_INFINITE);
    e.resources.set_food_tile(50, 50, RESOURCE_SOURCE_INFINITE);
    e.resources.set_water_tile(5, 5, RESOURCE_SOURCE_INFINITE);
    e.resources.set_water_tile(40, 40, RESOURCE_SOURCE_INFINITE);
    e.resources.set_sleep_tile(20, 20, RESOURCE_SOURCE_INFINITE);
    e.resources.set_sleep_tile(60, 4, RESOURCE_SOURCE_INFINITE);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    let food_t = seek_tile(&e, a_food).expect("A14: food agent must have a target");
    let water_t = seek_tile(&e, a_water).expect("A14: water agent must have a target");
    let sleep_t = seek_tile(&e, a_sleep).expect("A14: sleep agent must have a target");

    assert_eq!(food_t, (10, 10), "A14: food agent → nearest food (10,10)");
    assert_eq!(water_t, (5, 5), "A14: water agent → nearest water (5,5)");
    assert_eq!(sleep_t, (20, 20), "A14: sleep agent → nearest sleep (20,20)");

    // Independent min over each agent's OWN-kind map (non-circular).
    assert_eq!(manhattan((12, 12), food_t), min_manhattan_over((12, 12), &e.resources.food_tiles));
    assert_eq!(manhattan((6, 7), water_t), min_manhattan_over((6, 7), &e.resources.water_tiles));
    assert_eq!(manhattan((22, 18), sleep_t), min_manhattan_over((22, 18), &e.resources.sleep_tiles));

    // No cross-kind contamination: each chosen tile belongs to its own map.
    assert!(e.resources.food_tiles.contains_key(&food_t), "A14: food target in food_tiles");
    assert!(e.resources.water_tiles.contains_key(&water_t), "A14: water target in water_tiles");
    assert!(e.resources.sleep_tiles.contains_key(&sleep_t), "A14: sleep target in sleep_tiles");
    println!("[S16-α A14] 3 agents, 3 kinds, correct per-entity targets, zero contamination ✓");
}

// ─── Assertion 15: Seeking{Food} with an EMPTY food map (null path) ────────
#[test]
fn harness_seeking_with_empty_map_null_path() {
    // Type: A — nearest_resource_tile returns None on empty; the pass skips
    // the attach without panicking and the agent stays Seeking{Food}.
    let mut e = engine64();
    let agent = e.spawn_agent(12, 12);
    e.world
        .insert(
            agent,
            (AgentState::Seeking { target: TargetKind::Food }, Hunger::new(80.0, 0.0)),
        )
        .expect("seed agent");
    // food_tiles intentionally left EMPTY.

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources); // must not panic

    assert!(
        seek_tile(&e, agent).is_none(),
        "A15: empty food map → no SeekTarget attached"
    );
    assert_eq!(
        agent_state(&e, agent),
        AgentState::Seeking { target: TargetKind::Food },
        "A15: agent must remain Seeking{{Food}} (no spurious transition)"
    );
    println!("[S16-α A15] empty food map → no target, stays Seeking{{Food}}, no panic ✓");
}
