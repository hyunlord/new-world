//! V7 Section 16-ε — state head-markers + goal-lines FFI harness.
//!
//! feature: s16-epsilon-state-viz
//! seed: 42
//! agent_count: 20 (struct-fixture assertions) / 64 (live bootstrap, A7)
//! lane: --quick
//!
//! ε is a read-only visualization extension. It adds three additive parallel
//! fields to `AgentSnapshotRow` (`seek_kind: u8`, `target_x: i32`,
//! `target_y: i32`) carried by NEW parallel snapshot arrays — leaving the
//! locked `state_tag` (P11-α A22) and the `agent_rows_split` 4-tuple
//! (P4-γ) intact. No simulation behavior change.
//!
//! Non-circular rule: the struct-fixture assertions (1-6) build agents in
//! KNOWN `AgentState` / `SeekTarget` configurations and read the snapshot
//! with NO simulation step, so every expected value is hand-determined from
//! the controlled fixture, never re-derived from production code. A7 is the
//! complementary live-engine range/parity check on real bootstrap output.
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_epsilon_state_viz -- --nocapture

use std::fs;
use std::path::PathBuf;

use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_bridge::ffi::{collect_agent_snapshot, AgentSnapshotRow};
use sim_core::components::{Agent, AgentState, Hunger, SeekTarget, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_engine::{SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

const W: u32 = 64;
const H: u32 = 64;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Fresh 64×64 engine (no systems registered — struct fixtures only).
fn engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Spawn an agent at `(x,y)`, attach `state`, and optionally a `SeekTarget`.
fn spawn_with(
    e: &mut SimEngine,
    x: u32,
    y: u32,
    state: AgentState,
    seek: Option<SeekTarget>,
) -> hecs::Entity {
    let ent = e.spawn_agent(x, y);
    e.world.insert_one(ent, state).expect("insert AgentState");
    if let Some(st) = seek {
        e.world.insert_one(ent, st).expect("insert SeekTarget");
    }
    ent
}

/// The snapshot row whose `entity_bits` matches `ent`. Panics if absent.
fn row_for(rows: &[AgentSnapshotRow], ent: hecs::Entity) -> AgentSnapshotRow {
    let bits = ent.to_bits().get();
    *rows
        .iter()
        .find(|r| r.entity_bits == bits)
        .expect("snapshot row for entity")
}

fn project_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root above sim-test crate")
}

fn read_file(rel: &[&str]) -> String {
    let mut path = project_root();
    for seg in rel {
        path.push(seg);
    }
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Whitespace-stripped copy of `s` so literal checks ignore formatting.
fn no_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

// ─── Assertion 1: seek_kind maps Food→1, Water→2, Sleep→3 ──────────────────
#[test]
fn harness_s16_epsilon_a1_seek_kind_resource_mapping() {
    // Type A — enum→code invariant (1=Food, 2=Water, 3=Sleep), exact equality.
    let mut e = engine();
    let food = spawn_with(&mut e, 5, 5, AgentState::Seeking { target: TargetKind::Food }, None);
    let water = spawn_with(&mut e, 6, 6, AgentState::Seeking { target: TargetKind::Water }, None);
    let sleep = spawn_with(&mut e, 7, 7, AgentState::Seeking { target: TargetKind::Sleep }, None);

    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(row_for(&rows, food).seek_kind, 1, "A1: Seeking{{Food}} → seek_kind 1");
    assert_eq!(row_for(&rows, water).seek_kind, 2, "A1: Seeking{{Water}} → seek_kind 2");
    assert_eq!(row_for(&rows, sleep).seek_kind, 3, "A1: Seeking{{Sleep}} → seek_kind 3");
    println!("[S16-ε A1] seek_kind Food=1, Water=2, Sleep=3 ✓");
}

// ─── Assertion 2: target_x / target_y echo the SeekTarget tile exactly ─────
#[test]
fn harness_s16_epsilon_a2_target_coords_echo_seek_tile() {
    // Type A — direct u32→i32 cast of SeekTarget.tile.0/.1. x≠y (10≠20)
    // makes an axis swap detectable; both must hold exactly.
    let mut e = engine();
    let a = spawn_with(
        &mut e,
        3,
        4,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((10, 20))),
    );
    let rows = collect_agent_snapshot(&e.world);
    let row = row_for(&rows, a);
    assert_eq!(row.target_x, 10, "A2: target_x must equal SeekTarget.tile.0 (10)");
    assert_eq!(row.target_y, 20, "A2: target_y must equal SeekTarget.tile.1 (20)");
    println!("[S16-ε A2] SeekTarget(10,20) → target_x=10, target_y=20 (no axis swap) ✓");
}

// ─── Assertion 3: absent SeekTarget → −1 sentinel + seek_kind 0 ────────────
#[test]
fn harness_s16_epsilon_a3_absent_seek_target_sentinel() {
    // Type A — Idle agent with NO SeekTarget: seek_kind 0, target (-1,-1).
    // The renderer's goal-line guard is `target_x >= 0`, so the sentinel
    // MUST be -1 (0,0 would draw a spurious line to the grid corner).
    let mut e = engine();
    let idle = spawn_with(&mut e, 8, 9, AgentState::Idle, None);
    let rows = collect_agent_snapshot(&e.world);
    let row = row_for(&rows, idle);
    assert_eq!(row.seek_kind, 0, "A3: Idle (no resource trip) → seek_kind 0");
    assert_eq!(row.target_x, -1, "A3: no SeekTarget → target_x sentinel -1");
    assert_eq!(row.target_y, -1, "A3: no SeekTarget → target_y sentinel -1");
    println!("[S16-ε A3] Idle + no SeekTarget → seek_kind 0, target (-1,-1) ✓");
}

// ─── Assertion 4: state_tag mapping UNCHANGED on extended row (A22) ────────
#[test]
fn harness_s16_epsilon_a4_state_tag_table_unchanged() {
    // Type D — regression guard for the locked P11-α A22 state_tag table.
    // Adding seek_kind/target fields must not perturb state_tag.
    let mut e = engine();
    let idle = spawn_with(&mut e, 1, 1, AgentState::Idle, None);
    let seeking = spawn_with(&mut e, 2, 2, AgentState::Seeking { target: TargetKind::Food }, None);
    let consuming_agent = spawn_with(
        &mut e,
        3,
        3,
        AgentState::Consuming { target: TargetKind::Agent(777) },
        None,
    );
    let consuming_food = spawn_with(
        &mut e,
        4,
        4,
        AgentState::Consuming { target: TargetKind::Food },
        None,
    );

    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(row_for(&rows, idle).state_tag, 0, "A4: Idle → state_tag 0");
    assert_eq!(row_for(&rows, seeking).state_tag, 1, "A4: Seeking{{..}} → state_tag 1");
    assert_eq!(
        row_for(&rows, consuming_agent).state_tag,
        2,
        "A4: Consuming{{Agent(_)}} → state_tag 2"
    );
    assert_eq!(
        row_for(&rows, consuming_food).state_tag,
        3,
        "A4: Consuming{{Food}} → state_tag 3"
    );
    println!("[S16-ε A4] state_tag table intact: 0/1/2/3 ✓");
}

// ─── Assertion 5: non-resource Seeking targets report seek_kind 0 ──────────
#[test]
fn harness_s16_epsilon_a5_non_resource_seek_kind_zero() {
    // Type A — seek_kind encodes only resource trips. ConstructionSite and
    // Agent are not resource trips → 0. (state_tag stays 1 — orthogonal.)
    let mut e = engine();
    let site = spawn_with(
        &mut e,
        10,
        10,
        AgentState::Seeking { target: TargetKind::ConstructionSite },
        None,
    );
    let partner = spawn_with(
        &mut e,
        11,
        11,
        AgentState::Seeking { target: TargetKind::Agent(424242) },
        None,
    );
    let rows = collect_agent_snapshot(&e.world);
    let site_row = row_for(&rows, site);
    let partner_row = row_for(&rows, partner);
    assert_eq!(site_row.seek_kind, 0, "A5: Seeking{{ConstructionSite}} → seek_kind 0");
    assert_eq!(partner_row.seek_kind, 0, "A5: Seeking{{Agent(_)}} → seek_kind 0");
    // Precondition: both are still Seeking (state_tag 1) — not vacuous.
    assert_eq!(site_row.state_tag, 1, "A5 precondition: ConstructionSite seek is state_tag 1");
    assert_eq!(partner_row.state_tag, 1, "A5 precondition: Agent seek is state_tag 1");
    println!("[S16-ε A5] ConstructionSite/Agent seeks → seek_kind 0 (state_tag still 1) ✓");
}

// ─── Assertion 6: pre-existing row fields byte-for-byte intact (P4-γ A5) ───
#[test]
fn harness_s16_epsilon_a6_existing_fields_intact() {
    // Type D — the struct extension is additive; the five original fields
    // (entity_bits, x, y, state_tag, agent_id) must be untouched.
    let mut e = engine();
    let ent = spawn_with(
        &mut e,
        13,
        21,
        AgentState::Consuming { target: TargetKind::Water },
        Some(SeekTarget::new((1, 2))),
    );
    let expected_agent_id = e.world.get::<&Agent>(ent).expect("Agent").id;

    let rows = collect_agent_snapshot(&e.world);
    let row = row_for(&rows, ent);
    assert_eq!(
        row.entity_bits,
        ent.to_bits().get(),
        "A6: entity_bits must equal Entity::to_bits().get()"
    );
    assert_eq!(row.x, 13, "A6: x must equal inserted Position.x");
    assert_eq!(row.y, 21, "A6: y must equal inserted Position.y");
    // Consuming{Water} (non-Agent) → state_tag 3 per the A22 table.
    assert_eq!(row.state_tag, 3, "A6: state_tag must follow the A22 table (Consuming{{Water}}=3)");
    assert_eq!(row.agent_id, expected_agent_id, "A6: agent_id must equal Agent.id");
    println!("[S16-ε A6] entity_bits/x/y/state_tag/agent_id intact ✓");
}

// ─── Assertion 7: live snapshot well-formedness (range + parity) ───────────
#[test]
fn harness_s16_epsilon_a7_live_snapshot_well_formed() {
    // Type A — range/parity invariant on real bootstrap output after a short
    // run. Catches mis-mappings that only manifest under live Seeking
    // transitions. Grid is 64×64 so valid tile coords are [0,63].
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for _ in 0..2000 {
        e.tick();
    }

    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 64, "A7: row count must equal the bootstrap agent count (64)");
    for (i, row) in rows.iter().enumerate() {
        // (a) seek_kind in the locked domain {0,1,2,3}.
        assert!(
            row.seek_kind <= 3,
            "A7(a): row {i} seek_kind {} must be in {{0,1,2,3}}",
            row.seek_kind
        );
        // (b) if seek_kind != 0 the target must be a valid in-grid tile.
        if row.seek_kind != 0 {
            assert!(
                row.target_x >= 0 && row.target_y >= 0,
                "A7(b): row {i} resource trip must carry a non-sentinel target ({},{})",
                row.target_x,
                row.target_y
            );
            assert!(
                row.target_x <= 63 && row.target_y <= 63,
                "A7(b): row {i} target ({},{}) must lie within the 64×64 grid",
                row.target_x,
                row.target_y
            );
        }
        // (c) sentinel parity: seek_kind 0 with target_x == -1 ⇒ target_y == -1.
        if row.seek_kind == 0 && row.target_x == -1 {
            assert_eq!(
                row.target_y, -1,
                "A7(c): row {i} half-set sentinel — target_x -1 requires target_y -1"
            );
        }
    }
    let seeking = rows.iter().filter(|r| r.seek_kind != 0).count();
    println!("[S16-ε A7] 64 rows well-formed; {seeking} resource-trip rows ✓");
}

// ─── Assertion 8: gathering loop preserved — Seeking→Consuming ─────────────
#[test]
fn harness_s16_epsilon_a8_gathering_loop_preserved() {
    // Type D — the additive FFI change must not disturb the α/β/δ gathering
    // loop. A forced-hungry agent co-located with a food source must reach
    // Consuming within 50 ticks and not stay stuck Idle.
    let mut e = engine();
    register_default_runtime_systems(&mut e);
    e.resources.set_food_tile(20, 20, RESOURCE_SOURCE_INFINITE);

    let agent = e.spawn_agent(20, 20);
    e.world
        .insert(
            agent,
            (
                AgentState::Idle,
                Hunger::new(51.0, 0.0),
                MovementRng::new(42),
            ),
        )
        .expect("seed forced-hungry agent");

    let mut reached_consuming = false;
    let mut left_idle = false;
    for _ in 0..50 {
        e.tick();
        let st = *e.world.get::<&AgentState>(agent).expect("AgentState present");
        if matches!(st, AgentState::Consuming { .. }) {
            reached_consuming = true;
        }
        if !matches!(st, AgentState::Idle) {
            left_idle = true;
        }
    }
    assert!(left_idle, "A8: forced-hungry agent must not remain stuck Idle");
    assert!(
        reached_consuming,
        "A8: agent must reach Consuming{{..}} within the 50-tick window"
    );
    println!("[S16-ε A8] forced-hungry agent Seeking→Consuming within 50 ticks ✓");
}

// ─── Assertion 9: renderer + scene static structure ────────────────────────
#[test]
fn harness_s16_epsilon_a9_renderer_and_scene_static_structure() {
    // Type D — the renderer is sub-resolution for the VLM, so its draw calls
    // + locked colours + scene wiring are the only automatable evidence.
    let renderer = read_file(&["scripts", "ui", "seek_viz_renderer.gd"]);
    let renderer_ws = no_ws(&renderer);
    assert!(
        renderer.contains("get_agent_snapshot"),
        "A9.1: seek_viz_renderer.gd must call get_agent_snapshot"
    );
    assert!(
        renderer.contains("draw_line"),
        "A9.2: seek_viz_renderer.gd must contain at least one draw_line"
    );
    assert!(
        renderer.contains("draw_circle"),
        "A9.3: seek_viz_renderer.gd must contain at least one draw_circle"
    );
    assert!(
        renderer_ws.contains("Color(0.9,0.2,0.2)"),
        "A9.4: seek_viz_renderer.gd must define the locked Food colour Color(0.9,0.2,0.2)"
    );
    assert!(
        renderer_ws.contains("Color(0.2,0.5,1.0)"),
        "A9.5: seek_viz_renderer.gd must define the locked Water colour Color(0.2,0.5,1.0)"
    );
    assert!(
        renderer_ws.contains("Color(0.95,0.75,0.2)"),
        "A9.6: seek_viz_renderer.gd must define the locked Sleep colour Color(0.95,0.75,0.2)"
    );

    let scene = read_file(&["scenes", "main.tscn"]);
    assert!(
        scene.contains("seek_viz_renderer.gd"),
        "A9.7: main.tscn must reference seek_viz_renderer.gd"
    );
    let mut found_node = false;
    for line in scene.lines() {
        let t = line.trim();
        if !t.starts_with("[node ") {
            continue;
        }
        if t.contains("name=\"SeekVizRenderer\"") && t.contains("type=\"Node2D\"") {
            found_node = true;
            break;
        }
    }
    assert!(
        found_node,
        "A9.8: main.tscn must contain [node name=\"SeekVizRenderer\" type=\"Node2D\" ...]"
    );
    // Scene-preservation regression: existing nodes/resources untouched.
    assert!(
        scene.contains("name=\"AgentRenderer\""),
        "A9.9: main.tscn must still contain the AgentRenderer node"
    );
    assert!(
        scene.contains("camera_controller.gd"),
        "A9.10: main.tscn must still contain the camera_controller.gd ext_resource"
    );
    println!("[S16-ε A9] renderer draw calls + 3 locked colours + scene wiring intact ✓");
}
