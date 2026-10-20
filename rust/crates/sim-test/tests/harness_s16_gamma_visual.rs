//! V7 Section 16-γ — observability: visible resource markers + speed control.
//!
//! feature: s16-gamma-visual
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! γ is PURE observability scaffolding over the α0+α+β gathering loop
//! (53075aff / 335ca145 / fa5e350b). It changes NO simulation behavior:
//!   T1 — world_renderer source markers: tinted Sprite2D → bright Polygon2D.
//!   T2 — world_node.rs: `sim_speed` + `clamp_sim_speed` + accumulator scale.
//!   T3 — camera_controller.gd: KEY_1..KEY_4 → set_sim_speed.
//!
//! Assertion map (11; plan s16-gamma-visual plan_attempt 1):
//!   a1  — clamp_sim_speed pure correctness (hand-written expected) (Type A)
//!   a2  — world_node sim_speed field + #[func] set_sim_speed + scale (Type A)
//!   a3  — accumulator integrity: FIXED_DT + MAX_ITERS_PER_FRAME kept (Type D)
//!   a4  — marker is Polygon2D + SOURCE_KIND_COLORS + z_index, no Sprite2D (Type A)
//!   a5  — Z_RESOURCE_SOURCE > Z_RESOURCE                            (Type A)
//!   a6  — decorative invariant: RESOURCE_SEED + RESOURCE_COUNT==20  (Type D)
//!   a7  — backend-truth: get_resource_snapshot kept in marker fn    (Type D)
//!   a8  — camera KEY_1..KEY_4 + set_sim_speed in _unhandled_input   (Type A)
//!   a9  — camera invariants: extends Camera2D / KEY_P / handler; no KEY_SPACE (Type D)
//!   a10 — α0 substrate intact: each kind >= 4 source tiles          (Type C)
//!   a11 — gathering FSM intact: hungry agent Seeking → Consuming    (Type D)
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_gamma_visual -- --nocapture

use sim_bridge::ffi::clamp_sim_speed;
use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_core::components::{AgentState, Hunger, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::decision::AgentDecisionSystem;

// ── constants mirrored from the production bootstrap (read-only) ────────────
const DEFAULT_W: u32 = 64;
const DEFAULT_H: u32 = 64;

// ── helpers (h-phase-a / s16-α0 precedent) ──────────────────────────────────

fn bootstrapped_engine() -> SimEngine {
    let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    bootstrap_spawn_agents(&mut engine);
    engine
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 1 — clamp_sim_speed pure correctness (Type A, non-circular)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a1_clamp_sim_speed_pure_correctness() {
    // Type A — mathematical invariant. Expected values are HAND-WRITTEN
    // literals, NOT a re-derivation of `speed.clamp(0.0, 4.0)`.
    assert_eq!(
        clamp_sim_speed(0.25),
        0.25,
        "a1.1: 0.25 in-range passthrough"
    );
    assert_eq!(clamp_sim_speed(0.5), 0.5, "a1.2: 0.5 in-range passthrough");
    assert_eq!(clamp_sim_speed(1.0), 1.0, "a1.3: 1.0 in-range passthrough");
    assert_eq!(clamp_sim_speed(2.0), 2.0, "a1.4: 2.0 in-range passthrough");
    assert_eq!(
        clamp_sim_speed(10.0),
        4.0,
        "a1.5: 10.0 clamps to upper bound 4.0"
    );
    assert_eq!(
        clamp_sim_speed(-1.0),
        0.0,
        "a1.6: -1.0 clamps to lower bound 0.0"
    );
    // Inclusive boundary inputs pass through unchanged.
    assert_eq!(clamp_sim_speed(0.0), 0.0, "a1.7: 0.0 inclusive lower bound");
    assert_eq!(clamp_sim_speed(4.0), 4.0, "a1.8: 4.0 inclusive upper bound");
    println!("[S16-γ a1] clamp_sim_speed correct for all 8 hand-written cases ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 10 — α0 resource-source substrate intact (Type C)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a10_resource_source_substrate_seeding_intact() {
    // Type C — α0 substrate regression guard. γ changes no backend; the fixed
    // 4-tile lattices per kind must remain.
    let engine = bootstrapped_engine();
    let food = engine.resources.food_tiles.len();
    let water = engine.resources.water_tiles.len();
    let sleep = engine.resources.sleep_tiles.len();
    assert!(
        food >= 4,
        "a10.1: food_tiles must have >= 4 source tiles; got {food}"
    );
    assert!(
        water >= 4,
        "a10.2: water_tiles must have >= 4 source tiles; got {water}"
    );
    assert!(
        sleep >= 4,
        "a10.3: sleep_tiles must have >= 4 source tiles; got {sleep}"
    );
    println!("[S16-γ a10] source tiles food={food} water={water} sleep={sleep} (each >= 4) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 11 — gathering-loop FSM not regressed (Type D)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a11_gathering_loop_fsm_not_regressed() {
    // Type D — α/β gathering-loop regression guard. A hungry agent co-located
    // on a real bootstrap food tile must still progress Seeking → Consuming.
    let mut engine = bootstrapped_engine();
    let mut food_coords: Vec<(u32, u32)> = engine.resources.food_tiles.keys().copied().collect();
    food_coords.sort();
    let (fx, fy) = *food_coords
        .first()
        .expect("a11.0: bootstrap must produce >= 1 food tile");

    let entity = engine.spawn_agent(fx, fy);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Seeking {
                    target: TargetKind::Food,
                },
                Hunger::new(80.0, 0.0),
            ),
        )
        .expect("a11.1: seed hungry agent on the food tile");

    let mut sys = AgentDecisionSystem::new();
    // Tick 1: a co-located Seeking{Food} agent transitions to Consuming{Food}.
    sys.tick(&mut engine.world, &mut engine.resources);
    let state = *engine
        .world
        .get::<&AgentState>(entity)
        .expect("a11.2: state present");
    assert_eq!(
        state,
        AgentState::Consuming {
            target: TargetKind::Food
        },
        "a11.3: a hungry agent on a food source must reach Consuming{{Food}}; got {state:?}"
    );
    println!("[S16-γ a11] hungry agent on food tile ({fx},{fy}) → Consuming{{Food}} ✓");
}
