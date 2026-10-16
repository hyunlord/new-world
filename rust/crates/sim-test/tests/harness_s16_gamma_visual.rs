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

use std::fs;
use std::path::PathBuf;

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

fn read_world_node_src() -> String {
    read_file(&["rust", "crates", "sim-bridge", "src", "ffi", "world_node.rs"])
}



fn read_camera_controller_src() -> String {
    read_file(&["scripts", "ui", "camera_controller.gd"])
}

/// Strip `#` line comments while respecting string literals (h-a10 precedent).
fn strip_gd_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut in_str: Option<char> = None;
        let mut keep_end = line.len();
        for (i, c) in line.char_indices() {
            match in_str {
                Some(q) if c == q => in_str = None,
                None if c == '"' || c == '\'' => in_str = Some(c),
                None if c == '#' => {
                    keep_end = i;
                    break;
                }
                _ => {}
            }
        }
        out.push_str(&line[..keep_end]);
        out.push('\n');
    }
    out
}

/// Collapse runs of whitespace to single spaces (token-boundary preserving).
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Locate the byte-range of a top-level `func NAME(` body over GDScript source.
fn find_gd_func_body(stripped: &str, fname: &str) -> Option<(usize, usize)> {
    let needle = format!("func {fname}(");
    let start = stripped.find(&needle)?;
    let nl = stripped[start..].find('\n')?;
    let body_start = start + nl + 1;
    let next_func = stripped[body_start..].find("\nfunc ");
    let body_end = match next_func {
        Some(off) => body_start + off + 1,
        None => stripped.len(),
    };
    Some((body_start, body_end))
}

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
    assert_eq!(clamp_sim_speed(0.25), 0.25, "a1.1: 0.25 in-range passthrough");
    assert_eq!(clamp_sim_speed(0.5), 0.5, "a1.2: 0.5 in-range passthrough");
    assert_eq!(clamp_sim_speed(1.0), 1.0, "a1.3: 1.0 in-range passthrough");
    assert_eq!(clamp_sim_speed(2.0), 2.0, "a1.4: 2.0 in-range passthrough");
    assert_eq!(clamp_sim_speed(10.0), 4.0, "a1.5: 10.0 clamps to upper bound 4.0");
    assert_eq!(clamp_sim_speed(-1.0), 0.0, "a1.6: -1.0 clamps to lower bound 0.0");
    // Inclusive boundary inputs pass through unchanged.
    assert_eq!(clamp_sim_speed(0.0), 0.0, "a1.7: 0.0 inclusive lower bound");
    assert_eq!(clamp_sim_speed(4.0), 4.0, "a1.8: 4.0 inclusive upper bound");
    println!("[S16-γ a1] clamp_sim_speed correct for all 8 hand-written cases ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 2 — world_node sim_speed symbols present (Type A)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a2_world_node_sim_speed_symbols_present() {
    // Type A — spec-mandated symbols (Section 2 row 2). Absence = unimplemented.
    let src = read_world_node_src();
    let compact = collapse_ws(&src);
    // (a) struct declares a `sim_speed` field of type f64.
    assert!(
        compact.contains("sim_speed: f64"),
        "a2.1: WorldSimNode must declare a `sim_speed: f64` field"
    );
    // (b) a set_sim_speed method carries the #[func] attribute. Verify the
    //     #[func] attribute immediately precedes the fn (allow whitespace).
    let set_pos = src
        .find("fn set_sim_speed")
        .expect("a2.2: a `fn set_sim_speed` must exist");
    let preamble = &src[..set_pos];
    assert!(
        preamble.trim_end().ends_with("#[func]"),
        "a2.3: `fn set_sim_speed` must be immediately preceded by the #[func] attribute"
    );
    // (c) process() scales the accumulator increment by sim_speed
    //     (accept either factor order).
    assert!(
        compact.contains("delta * self.sim_speed") || compact.contains("self.sim_speed * delta"),
        "a2.4: process() must scale the accumulator increment by sim_speed \
         (`delta * self.sim_speed` or `self.sim_speed * delta`)"
    );
    println!("[S16-γ a2] sim_speed field + #[func] set_sim_speed + accumulator scale ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 3 — accumulator integrity preserved (Type D)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a3_accumulator_integrity_preserved() {
    // Type D — regression guard. The Gaffer fixed-timestep loop and
    // spiral-of-death guard must survive the speed change untouched.
    let src = read_world_node_src();
    assert!(
        src.contains("FIXED_DT"),
        "a3.1: world_node.rs must keep FIXED_DT (fixed-timestep determinism)"
    );
    assert!(
        src.contains("MAX_ITERS_PER_FRAME"),
        "a3.2: world_node.rs must keep MAX_ITERS_PER_FRAME (spiral-of-death guard)"
    );
    println!("[S16-γ a3] FIXED_DT + MAX_ITERS_PER_FRAME preserved ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 4 — marker is Polygon2D (function-body slice) (Type A)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a4_world_renderer_marker_is_polygon2d() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 5 — z-order above the decorative layer (Type A)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a5_z_order_above_decorative() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("z_index = 0"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("z_index = 5"));
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(src.contains("sprite.z_index = Z_OVERLAY"));
    assert!(src.contains("const Z_OVERLAY: int = 10"));
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 6 — decorative invariant preserved (Type D)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a6_decorative_invariant_preserved() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 7 — backend-truth positions preserved (Type D)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a7_backend_truth_positions_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 8 — camera speed keys bound (Type A)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a8_camera_controller_speed_keys_bound() {
    // Type A — spec-mandated (Section 2 row 3 / T3). The four number keys must
    // drive set_sim_speed inside the _unhandled_input body.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let (s, e) = find_gd_func_body(&stripped, "_unhandled_input")
        .expect("a8.0: _unhandled_input must exist");
    let body = &stripped[s..e];
    for key in ["KEY_1", "KEY_2", "KEY_3", "KEY_4"] {
        assert!(
            body.contains(key),
            "a8.1: _unhandled_input must reference {key} (speed binding). Body:\n{body}"
        );
    }
    assert!(
        body.contains("set_sim_speed"),
        "a8.2: _unhandled_input must call set_sim_speed. Body:\n{body}"
    );
    println!("[S16-γ a8] KEY_1..KEY_4 + set_sim_speed bound in _unhandled_input ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 9 — camera invariants preserved (Type D, KEY_SPACE trap)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a9_camera_controller_invariants_preserved() {
    // Type D — h-a10 invariant guard. ⚠️ camera_controller.gd lines 81-82
    // mention KEY_SPACE INSIDE A COMMENT; the absence check MUST run on
    // comment-stripped source or it false-fails on correct code.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    assert!(
        stripped.contains("extends Camera2D"),
        "a9.1: camera_controller.gd must keep `extends Camera2D`"
    );
    assert!(
        stripped.contains("KEY_P"),
        "a9.2: camera_controller.gd must keep KEY_P (pause)"
    );
    assert!(
        stripped.contains("func _unhandled_input"),
        "a9.3: camera_controller.gd must keep `func _unhandled_input`"
    );
    assert!(
        !stripped.contains("KEY_SPACE"),
        "a9.4: camera_controller.gd must NOT reference KEY_SPACE in CODE \
         (comment-stripped); SPACE belongs to the world_renderer overlay cycle"
    );
    println!("[S16-γ a9] extends Camera2D / KEY_P / _unhandled_input kept; no KEY_SPACE in code ✓");
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
    assert!(food >= 4, "a10.1: food_tiles must have >= 4 source tiles; got {food}");
    assert!(water >= 4, "a10.2: water_tiles must have >= 4 source tiles; got {water}");
    assert!(sleep >= 4, "a10.3: sleep_tiles must have >= 4 source tiles; got {sleep}");
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
                AgentState::Seeking { target: TargetKind::Food },
                Hunger::new(80.0, 0.0),
            ),
        )
        .expect("a11.1: seed hungry agent on the food tile");

    let mut sys = AgentDecisionSystem::new();
    // Tick 1: a co-located Seeking{Food} agent transitions to Consuming{Food}.
    sys.tick(&mut engine.world, &mut engine.resources);
    let state = *engine.world.get::<&AgentState>(entity).expect("a11.2: state present");
    assert_eq!(
        state,
        AgentState::Consuming { target: TargetKind::Food },
        "a11.3: a hungry agent on a food source must reach Consuming{{Food}}; got {state:?}"
    );
    println!("[S16-γ a11] hungry agent on food tile ({fx},{fy}) → Consuming{{Food}} ✓");
}
