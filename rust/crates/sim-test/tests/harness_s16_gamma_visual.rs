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

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
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

/// Find the RHS values of every `const IDENT := …` declaration over stripped
/// GDScript source. Returns each RHS string verbatim (trimmed).
fn find_const_rhs(stripped: &str, ident: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in stripped.lines() {
        let t = line.trim_start();
        let Some(after) = t.strip_prefix("const ") else {
            continue;
        };
        if !after.starts_with(ident) {
            continue;
        }
        let next = after.as_bytes().get(ident.len()).copied().unwrap_or(b' ') as char;
        if next.is_ascii_alphanumeric() || next == '_' {
            continue; // prefix collision (e.g. Z_RESOURCE vs Z_RESOURCE_SOURCE)
        }
        if let Some(p) = line.find(":=") {
            out.push(line[p + 2..].trim().to_string());
        } else if let Some(p) = line.find('=') {
            out.push(line[p + 1..].trim().to_string());
        }
    }
    out
}

/// Parse the leading signed integer from a GDScript const RHS.
fn parse_int_rhs(rhs: &str) -> Option<i64> {
    let trimmed = rhs.trim();
    let mut end = 0usize;
    let mut saw_digit = false;
    for (i, c) in trimmed.char_indices() {
        if i == 0 && (c == '-' || c == '+') {
            end = i + c.len_utf8();
            continue;
        }
        if c.is_ascii_digit() {
            end = i + c.len_utf8();
            saw_digit = true;
        } else {
            break;
        }
    }
    if !saw_digit {
        return None;
    }
    trimmed[..end].parse::<i64>().ok()
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
    // Type A — spec-mandated (Section 2 row 1 / T1). Slice the
    // _render_resource_sources body ONLY — RESOURCE_SPRITE_PATH/Sprite2D
    // legitimately remain in the decorative scatter elsewhere in the file.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let (s, e) = find_gd_func_body(&stripped, "_render_resource_sources")
        .expect("a4.0: _render_resource_sources must exist");
    let body = &stripped[s..e];
    // (a) Polygon2D instantiated for the marker.
    assert!(
        body.contains("Polygon2D"),
        "a4.1: marker must be a Polygon2D. Body:\n{body}"
    );
    // (b) marker color assigned from SOURCE_KIND_COLORS.
    assert!(
        body.contains("SOURCE_KIND_COLORS"),
        "a4.2: marker color must come from SOURCE_KIND_COLORS. Body:\n{body}"
    );
    // (c) marker z_index is set.
    assert!(
        body.contains("z_index"),
        "a4.3: marker must set z_index. Body:\n{body}"
    );
    // (d) the per-source Sprite2D path from α0 is REPLACED, not kept alongside.
    assert!(
        !body.contains("Sprite2D"),
        "a4.4: _render_resource_sources must NOT construct a Sprite2D marker \
         (the α0 tinted-Sprite2D path is replaced, not augmented). Body:\n{body}"
    );
    println!("[S16-γ a4] marker = Polygon2D + SOURCE_KIND_COLORS + z_index, no Sprite2D ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 5 — z-order above the decorative layer (Type A)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a5_z_order_above_decorative() {
    // Type A — ordering invariant. The marker must render strictly above the
    // decorative Z_RESOURCE=3 scatter or it is occluded.
    let stripped = strip_gd_comments(&read_world_renderer_src());

    let src_rhss = find_const_rhs(&stripped, "Z_RESOURCE_SOURCE");
    assert_eq!(
        src_rhss.len(),
        1,
        "a5.1: expected exactly 1 Z_RESOURCE_SOURCE declaration; got {src_rhss:?}"
    );
    let dec_rhss = find_const_rhs(&stripped, "Z_RESOURCE");
    assert_eq!(
        dec_rhss.len(),
        1,
        "a5.2: expected exactly 1 Z_RESOURCE declaration; got {dec_rhss:?}"
    );
    let z_source = parse_int_rhs(&src_rhss[0])
        .unwrap_or_else(|| panic!("a5.3: Z_RESOURCE_SOURCE RHS must parse as int; got `{}`", src_rhss[0]));
    let z_decor = parse_int_rhs(&dec_rhss[0])
        .unwrap_or_else(|| panic!("a5.4: Z_RESOURCE RHS must parse as int; got `{}`", dec_rhss[0]));
    assert!(
        z_source > z_decor,
        "a5.5: Z_RESOURCE_SOURCE ({z_source}) must be strictly greater than Z_RESOURCE ({z_decor})"
    );
    println!("[S16-γ a5] Z_RESOURCE_SOURCE ({z_source}) > Z_RESOURCE ({z_decor}) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 6 — decorative invariant preserved (Type D)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a6_decorative_invariant_preserved() {
    // Type D — p13-β decorative-scatter regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    assert!(
        stripped.contains("RESOURCE_SEED"),
        "a6.1: world_renderer.gd must keep RESOURCE_SEED (p13-β decorative layer)"
    );
    let rhss = find_const_rhs(&stripped, "RESOURCE_COUNT");
    assert_eq!(
        rhss.len(),
        1,
        "a6.2: expected exactly 1 RESOURCE_COUNT declaration; got {rhss:?}"
    );
    let count = parse_int_rhs(&rhss[0])
        .unwrap_or_else(|| panic!("a6.3: RESOURCE_COUNT RHS must parse as int; got `{}`", rhss[0]));
    assert_eq!(count, 20, "a6.4: RESOURCE_COUNT must remain 20; got {count}");
    println!("[S16-γ a6] RESOURCE_SEED present + RESOURCE_COUNT == 20 ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 7 — backend-truth positions preserved (Type D)
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_s16_gamma_a7_backend_truth_positions_preserved() {
    // Type D — α0 substrate regression guard. Marker positions must keep
    // reading the backend FFI snapshot, not be hardcoded client-side.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let (s, e) = find_gd_func_body(&stripped, "_render_resource_sources")
        .expect("a7.0: _render_resource_sources must exist");
    let body = &stripped[s..e];
    assert!(
        body.contains("get_resource_snapshot"),
        "a7.1: _render_resource_sources must keep calling get_resource_snapshot \
         (backend-truth positions). Body:\n{body}"
    );
    println!("[S16-γ a7] _render_resource_sources reads get_resource_snapshot ✓");
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
