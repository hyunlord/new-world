//! V7 Phase 14-ε — Activity Trails harness.
//!
//! feature: p14-epsilon-activity-trails
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Static file-inspection harness verifying the Phase 14-ε implementation:
//!   - `scripts/ui/activity_trail_renderer.gd` — new Node2D, draws faint
//!     polyline trails through recent agent positions for non-Idle agents.
//!   - `scenes/main.tscn` — registers ActivityTrailRenderer under Main.
//!   - Regression guards (Phase 14-α agent_renderer.gd, Phase 14-β
//!     world_renderer.gd, Phase 13-δ hud_topbar.gd, Phase 14-δ
//!     hud_status_panel.gd).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_epsilon_activity_trails -- --nocapture

use std::fs;
use std::path::PathBuf;

// ── helpers ───────────────────────────────────────────────────────────────

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

fn read_trail_renderer_src() -> String {
    read_file(&["scripts", "ui", "activity_trail_renderer.gd"])
}

fn read_main_tscn_src() -> String {
    read_file(&["scenes", "main.tscn"])
}

fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
}

fn read_hud_topbar_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_topbar.gd"])
}

fn read_hud_status_panel_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_status_panel.gd"])
}

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

fn no_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Find every `const`/`var` RHS declaration of `ident`.
fn find_decl_rhss(stripped: &str, ident: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in stripped.lines() {
        let t = line.trim_start();
        if !(t.starts_with("const ") || t.starts_with("var ")) {
            continue;
        }
        let after_kw = t
            .strip_prefix("const ")
            .or_else(|| t.strip_prefix("var "))
            .unwrap_or(t);
        if !after_kw.starts_with(ident) {
            continue;
        }
        let next = after_kw
            .as_bytes()
            .get(ident.len())
            .copied()
            .unwrap_or(b' ') as char;
        if next.is_ascii_alphanumeric() || next == '_' {
            continue;
        }
        let rhs = if let Some(p) = line.find(":=") {
            line[p + 2..].trim().to_string()
        } else {
            let bytes = line.as_bytes();
            let mut found: Option<usize> = None;
            let mut i = 0usize;
            while i < bytes.len() {
                if bytes[i] == b'=' {
                    let prev = if i == 0 { b' ' } else { bytes[i - 1] };
                    let next_b = bytes.get(i + 1).copied().unwrap_or(b' ');
                    if prev != b':' && prev != b'=' && next_b != b'=' {
                        found = Some(i);
                        break;
                    }
                }
                i += 1;
            }
            match found {
                Some(p) => line[p + 1..].trim().to_string(),
                None => continue,
            }
        };
        out.push(rhs);
    }
    out
}

fn unique_decl_rhs(stripped: &str, ident: &str, label: &str) -> String {
    let rhss = find_decl_rhss(stripped, ident);
    assert_eq!(
        rhss.len(),
        1,
        "{label}: expected exactly 1 declaration of `{ident}`; got {n}: {rhss:?}",
        n = rhss.len()
    );
    rhss.into_iter().next().unwrap()
}

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

fn parse_float_rhs(rhs: &str) -> Option<f64> {
    let trimmed = rhs.trim();
    let mut end = 0usize;
    let mut saw_digit = false;
    let mut saw_dot = false;
    for (i, c) in trimmed.char_indices() {
        if i == 0 && (c == '-' || c == '+') {
            end = i + c.len_utf8();
            continue;
        }
        if c.is_ascii_digit() {
            end = i + c.len_utf8();
            saw_digit = true;
        } else if c == '.' && !saw_dot {
            end = i + c.len_utf8();
            saw_dot = true;
        } else {
            break;
        }
    }
    if !saw_digit {
        return None;
    }
    trimmed[..end].parse::<f64>().ok()
}

/// Locate the byte-range of a top-level `func NAME(` body.
/// Returns (start_after_signature, end_exclusive) over the stripped source.
fn find_func_body(stripped: &str, fname: &str) -> Option<(usize, usize)> {
    let needle = format!("func {fname}(");
    let start = stripped.find(&needle)?;
    // Find end-of-line for the signature.
    let nl = stripped[start..].find('\n')?;
    let body_start = start + nl + 1;
    // Body ends at next top-level `func ` or end-of-file.
    let next_func = stripped[body_start..].find("\nfunc ");
    let body_end = match next_func {
        Some(off) => body_start + off + 1,
        None => stripped.len(),
    };
    Some((body_start, body_end))
}

// ─── A1: file exists + extends Node2D ─────────────────────────────────────
#[test]
fn harness_p14_epsilon_a1_activity_trail_renderer_file_exists_and_extends_node2d() {
    // Type A — file present; first non-blank, non-comment line == `extends Node2D`.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("activity_trail_renderer.gd");
    assert!(
        path.is_file(),
        "A1.1: activity_trail_renderer.gd must exist at {path:?}"
    );
    let src = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("A1.2: cannot read {path:?}: {e}"));
    let stripped = strip_gd_comments(&src);
    let mut first: Option<String> = None;
    for line in stripped.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        first = Some(t.to_string());
        break;
    }
    let head = first.expect("A1.3: activity_trail_renderer.gd has no non-blank, non-comment content");
    assert_eq!(
        head, "extends Node2D",
        "A1.4: first non-comment, non-blank line must be `extends Node2D`; got `{head}`"
    );
    println!("[P14-ε A1] activity_trail_renderer.gd exists + `extends Node2D` ✓");
}

// ─── A2: TRAIL_LENGTH == 16 ──────────────────────────────────────────────
#[test]
fn harness_p14_epsilon_a2_trail_length_constant_equals_16() {
    // Type A — exactly one declaration of TRAIL_LENGTH, RHS parses to 16.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "TRAIL_LENGTH", "A2");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("A2: TRAIL_LENGTH RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 16, "A2: TRAIL_LENGTH must equal 16; got {n}");
    println!("[P14-ε A2] TRAIL_LENGTH = 16 ✓");
}

// ─── A3: TRAIL_WIDTH == 2.0 ──────────────────────────────────────────────
#[test]
fn harness_p14_epsilon_a3_trail_width_constant_equals_2_0() {
    // Type A — exactly one declaration of TRAIL_WIDTH, RHS parses to 2.0.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "TRAIL_WIDTH", "A3");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("A3: TRAIL_WIDTH RHS must parse as float; got `{rhs}`"));
    assert!(
        (f - 2.0).abs() < 1e-9,
        "A3: TRAIL_WIDTH must equal 2.0; got {f}"
    );
    println!("[P14-ε A3] TRAIL_WIDTH = 2.0 ✓");
}

// ─── A4: TRAIL_ALPHA == 0.4 ──────────────────────────────────────────────
#[test]
fn harness_p14_epsilon_a4_trail_alpha_constant_equals_0_4() {
    // Type A — exactly one declaration of TRAIL_ALPHA, RHS parses to 0.4.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "TRAIL_ALPHA", "A4");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("A4: TRAIL_ALPHA RHS must parse as float; got `{rhs}`"));
    assert!(
        (f - 0.4).abs() < 1e-9,
        "A4: TRAIL_ALPHA must equal 0.4; got {f}"
    );
    println!("[P14-ε A4] TRAIL_ALPHA = 0.4 ✓");
}

// ─── A5: Z_TRAIL == 2 AND z_index = Z_TRAIL in _ready ─────────────────────
#[test]
fn harness_p14_epsilon_a5_z_index_set_to_two_in_ready() {
    // Type A — `const Z_TRAIL := 2` AND `z_index = Z_TRAIL` (whitespace-
    // collapsed) appears within `_ready` body window.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "Z_TRAIL", "A5.const");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("A5: Z_TRAIL RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 2, "A5.1: Z_TRAIL must equal 2; got {n}");

    let (s, e) = find_func_body(&stripped, "_ready")
        .expect("A5.2: _ready function must exist in activity_trail_renderer.gd");
    let body = &stripped[s..e];
    let body_compact = no_ws(body);
    assert!(
        body_compact.contains("z_index=Z_TRAIL"),
        "A5.3: `_ready` body must contain `z_index = Z_TRAIL` (whitespace-collapsed); body:\n{body}"
    );
    println!("[P14-ε A5] Z_TRAIL = 2 + `z_index = Z_TRAIL` in _ready ✓");
}

// ─── A6: three distinct trail colours per state_tag ──────────────────────
#[test]
fn harness_p14_epsilon_a6_three_distinct_state_tag_trail_colours_declared() {
    // Type A — three TRAIL_COLOR_* `const` declarations exist; no two RHSs
    // identical (whitespace-collapsed inequality).
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let seeking = unique_decl_rhs(&stripped, "TRAIL_COLOR_SEEKING", "A6.seeking");
    let consuming_agent =
        unique_decl_rhs(&stripped, "TRAIL_COLOR_CONSUMING_AGENT", "A6.consuming_agent");
    let consuming_other =
        unique_decl_rhs(&stripped, "TRAIL_COLOR_CONSUMING_OTHER", "A6.consuming_other");

    let s = no_ws(&seeking);
    let ca = no_ws(&consuming_agent);
    let co = no_ws(&consuming_other);
    assert_ne!(
        s, ca,
        "A6.1: TRAIL_COLOR_SEEKING and TRAIL_COLOR_CONSUMING_AGENT must differ; both = `{s}`"
    );
    assert_ne!(
        s, co,
        "A6.2: TRAIL_COLOR_SEEKING and TRAIL_COLOR_CONSUMING_OTHER must differ; both = `{s}`"
    );
    assert_ne!(
        ca, co,
        "A6.3: TRAIL_COLOR_CONSUMING_AGENT and TRAIL_COLOR_CONSUMING_OTHER must differ; both = `{ca}`"
    );
    println!("[P14-ε A6] 3 distinct TRAIL_COLOR_* declarations ✓");
}

// ─── A7: snapshot scope = agent only ─────────────────────────────────────
#[test]
fn harness_p14_epsilon_a7_snapshot_scope_limited_to_agent_only() {
    // Type A — `get_agent_snapshot` present; all 8 other snapshot/FFI
    // literals absent.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    assert!(
        stripped.contains("get_agent_snapshot"),
        "A7.1: activity_trail_renderer.gd must contain `get_agent_snapshot`"
    );
    let forbidden = [
        "get_settlement_snapshot",
        "get_construction_snapshot",
        "get_tile_detail",
        "get_agent_detail",
        "get_relationship_snapshot",
        "get_influence_overlay",
        "get_event_chain",
        "get_tile_causal_history",
    ];
    let mut found: Vec<&str> = Vec::new();
    for n in forbidden.iter() {
        if stripped.contains(n) {
            found.push(n);
        }
    }
    assert!(
        found.is_empty(),
        "A7.2: activity_trail_renderer.gd must reference only `get_agent_snapshot`; \
         found forbidden FFI literals: {found:?}"
    );
    println!("[P14-ε A7] snapshot scope limited to get_agent_snapshot ✓");
}

// ─── A8: Variant-safe pattern for 4 Packed* types + Dictionary ───────────
#[test]
fn harness_p14_epsilon_a8_variant_safe_pattern_present_for_four_packed_types() {
    // Type D — Phase 13-δ A8 regression precedent.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let needles = [
        "is Dictionary",
        "is PackedInt64Array",
        "is PackedByteArray",
        "is PackedInt32Array",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A8: activity_trail_renderer.gd must contain all 4 Variant-safe type guards; \
         missing={missing:?}"
    );
    println!("[P14-ε A8] Variant-safe pattern (4 Packed* types + Dictionary) present ✓");
}

// ─── A9: draw_polyline inside _draw ───────────────────────────────────────
#[test]
fn harness_p14_epsilon_a9_draw_polyline_present_inside_draw_function() {
    // Type A — `func _draw(` present AND `draw_polyline` appears inside body.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let (s, e) = find_func_body(&stripped, "_draw")
        .expect("A9.1: _draw function must exist in activity_trail_renderer.gd");
    let body = &stripped[s..e];
    assert!(
        body.contains("draw_polyline"),
        "A9.2: `_draw` body must contain `draw_polyline`; body:\n{body}"
    );
    println!("[P14-ε A9] `draw_polyline` inside `_draw` ✓");
}

// ─── A10: Idle agents (tag == 0) skipped in _draw ─────────────────────────
#[test]
fn harness_p14_epsilon_a10_idle_agents_skipped_in_draw() {
    // Type A — `_draw` body window contains `tag == 0` (ws tolerated) AND a
    // following `continue` statement.
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let (s, e) = find_func_body(&stripped, "_draw")
        .expect("A10.1: _draw function must exist in activity_trail_renderer.gd");
    let body = &stripped[s..e];
    let compact = no_ws(body);
    let tag_pos = compact
        .find("tag==0")
        .unwrap_or_else(|| panic!("A10.2: `_draw` body must contain `tag == 0`; body:\n{body}"));
    let continue_pos = compact[tag_pos..]
        .find("continue")
        .unwrap_or_else(|| panic!("A10.3: `_draw` body must have `continue` after `tag == 0`; body:\n{body}"));
    assert!(
        continue_pos > 0,
        "A10.4: `continue` must follow `tag == 0` in `_draw` body"
    );
    println!("[P14-ε A10] Idle skip (`tag == 0` → `continue`) inside `_draw` ✓");
}

// ─── A11: main.tscn registers ActivityTrailRenderer ──────────────────────
#[test]
fn harness_p14_epsilon_a11_main_tscn_registers_activity_trail_renderer_node() {
    // Type A — main.tscn references `activity_trail_renderer.gd` AND has a
    // [node name="ActivityTrailRenderer" type="Node2D" ...] line.
    let src = read_main_tscn_src();
    assert!(
        src.contains("activity_trail_renderer.gd"),
        "A11.1: main.tscn must reference `activity_trail_renderer.gd`"
    );
    let mut found = false;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[node ") {
            continue;
        }
        if t.contains("name=\"ActivityTrailRenderer\"") && t.contains("type=\"Node2D\"") {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "A11.2: main.tscn must contain `[node name=\"ActivityTrailRenderer\" type=\"Node2D\" ...]`; \
         source:\n{src}"
    );
    println!("[P14-ε A11] ActivityTrailRenderer node registered in main.tscn ✓");
}

// ─── A12: main.tscn load_steps == 12 ─────────────────────────────────────
#[test]
fn harness_p14_epsilon_a12_main_tscn_load_steps_equals_12() {
    // Type A — `[gd_scene ... load_steps=13 ...]`, whitespace around `=` tolerated.
    // Phase 14-ε registered ActivityTrailRenderer at load_steps=9; Phase 14-ζ
    // added SettlementOverviewRenderer + ZoomLodController (9 → 11); viz-B added
    // NeedBarRenderer (11 → 12); viz-D adds DeathVizRenderer (12 → 13). The scene
    // is shared, so this prior-phase invariant tracks the current ext_resource
    // count. ε's ActivityTrailRenderer registration itself is unchanged (A11).
    let src = read_main_tscn_src();
    let mut found_n: Option<i64> = None;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[gd_scene") {
            continue;
        }
        if let Some(pos) = t.find("load_steps") {
            let after = &t[pos + "load_steps".len()..];
            let after = after.trim_start();
            let after = after.strip_prefix('=').unwrap_or(after);
            let after = after.trim_start();
            let mut digits = String::new();
            for c in after.chars() {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                found_n = Some(digits.parse::<i64>().expect("digit parse"));
            }
        }
        break;
    }
    let n = found_n.expect("A12.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 13, "A12.2: load_steps must equal 13 (Phase 14-ζ 9 → 11, viz-B 11 → 12, viz-D 12 → 13); got {n}");
    println!("[P14-ε A12] load_steps = 13 (post viz-D DeathVizRenderer) ✓");
}

// ─── A13: agent_renderer.gd Phase 14-α/13-γ/11-α/4-γ invariants intact ───
#[test]
fn harness_p14_epsilon_a13_regression_guard_agent_renderer_phase_invariants_intact() {
    // Type D — five canonical literals all present.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let needles = [
        "ROLE_BUCKET_COUNT",
        "ICON_OFFSET_PX",
        "STATE_SCALE_BOOST",
        "STATE_TINTS",
        "SPRITE_SCALE",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A13: agent_renderer.gd must keep all 5 invariant literals; missing={missing:?}"
    );
    println!("[P14-ε A13] agent_renderer.gd 5-literal regression guard intact ✓");
}

// ─── A14: world_renderer.gd Phase 14-β invariants intact ─────────────────
#[test]
fn harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact() {
    // Type D — RESOURCE_TYPE_PATHS + VILLAGE_FIXTURE_ + RESOURCE_COUNT=20 +
    // RESOURCE_SEED=88675123.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    assert!(
        stripped.contains("RESOURCE_TYPE_PATHS"),
        "A14.1: world_renderer.gd must keep RESOURCE_TYPE_PATHS"
    );
    assert!(
        stripped.contains("VILLAGE_FIXTURE_"),
        "A14.2: world_renderer.gd must keep at least one VILLAGE_FIXTURE_ identifier"
    );

    let count_rhs = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A14.count");
    let count = parse_int_rhs(&count_rhs)
        .unwrap_or_else(|| panic!("A14.3: RESOURCE_COUNT RHS must parse as int; got `{count_rhs}`"));
    assert_eq!(count, 20, "A14.4: RESOURCE_COUNT must equal 20; got {count}");

    let seed_rhs = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A14.seed");
    let seed = parse_int_rhs(&seed_rhs)
        .unwrap_or_else(|| panic!("A14.5: RESOURCE_SEED RHS must parse as int; got `{seed_rhs}`"));
    assert_eq!(
        seed, 88675123,
        "A14.6: RESOURCE_SEED must equal 88675123; got {seed}"
    );
    println!("[P14-ε A14] world_renderer.gd Phase 14-β invariants intact ✓");
}

// ─── A15: hud_topbar.gd Phase 13-δ invariants intact ─────────────────────
#[test]
fn harness_p14_epsilon_a15_regression_guard_hud_topbar_phase_13_delta_intact() {
    // Type D — six canonical literals all present.
    let stripped = strip_gd_comments(&read_hud_topbar_src());
    let needles = [
        "get_agent_snapshot",
        "get_settlement_snapshot",
        "get_construction_snapshot",
        "is Dictionary",
        "is PackedInt64Array",
        "MOUSE_FILTER_IGNORE",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A15: hud_topbar.gd must keep all 6 invariant literals (Phase 13-δ A4/A5/A6); \
         missing={missing:?}"
    );
    println!("[P14-ε A15] hud_topbar.gd Phase 13-δ invariants intact ✓");
}

// ─── A16: hud_status_panel.gd Phase 14-δ invariants intact ───────────────
#[test]
fn harness_p14_epsilon_a16_regression_guard_hud_status_panel_phase_14_delta_intact() {
    // Type D — three canonical literals all present.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let needles = ["TICKS_PER_DAY", "RESOURCE_TYPES_COUNT", "RESOURCE_LABELS"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A16: hud_status_panel.gd must keep all 3 invariant literals (Phase 14-δ A2/A4/A5); \
         missing={missing:?}"
    );
    println!("[P14-ε A16] hud_status_panel.gd Phase 14-δ invariants intact ✓");
}
