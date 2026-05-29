//! V7 Phase 14-ζ — Zoom-Adaptive Rendering harness.
//!
//! feature: p14-zeta-zoom-adaptive
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Static file-inspection harness (Phase 14-ε precedent). No simulation is
//! run (`ticks: 0`), no ECS component is queried — the camera cannot be driven
//! headless, so zoom-tier behaviour is verified structurally:
//!   - `scripts/ui/zoom_lod_controller.gd`        — new Node, 3-tier LOD logic.
//!   - `scripts/ui/settlement_overview_renderer.gd` — new Node2D, region discs.
//!   - `scenes/main.tscn`                          — registers both siblings.
//!   - Regression guards (Phase 12-α/13-α camera, 14-α agent, 14-β world,
//!     14-ε trails, 14-δ status panel, 13-δ topbar).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_zeta_zoom_adaptive -- --nocapture

use std::fs;
use std::path::PathBuf;

// ── helpers (Phase 14-ε precedent) ─────────────────────────────────────────

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

fn read_zoom_lod_src() -> String {
    read_file(&["scripts", "ui", "zoom_lod_controller.gd"])
}

fn read_overview_src() -> String {
    read_file(&["scripts", "ui", "settlement_overview_renderer.gd"])
}

fn read_main_tscn_src() -> String {
    read_file(&["scenes", "main.tscn"])
}

fn read_camera_controller_src() -> String {
    read_file(&["scripts", "ui", "camera_controller.gd"])
}

fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
}

fn read_trail_renderer_src() -> String {
    read_file(&["scripts", "ui", "activity_trail_renderer.gd"])
}

fn read_hud_status_panel_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_status_panel.gd"])
}

fn read_hud_topbar_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_topbar.gd"])
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

/// Locate the byte-range of a top-level `func NAME(` body over stripped source.
fn find_func_body(stripped: &str, fname: &str) -> Option<(usize, usize)> {
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

/// First non-blank, non-comment line of a stripped source.
fn first_code_line(stripped: &str) -> Option<String> {
    for line in stripped.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return Some(t.to_string());
    }
    None
}

/// Read header `load_steps=N` from a .tscn (whitespace around `=` tolerated).
fn tscn_load_steps(src: &str) -> Option<i64> {
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
                return digits.parse::<i64>().ok();
            }
        }
        break;
    }
    None
}

// ════════════════════════════════════════════════════════════════════════════
// Group A — scripts/ui/zoom_lod_controller.gd
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 1: controller exists + `extends Node` ──────────────────────
#[test]
fn harness_zoom_a1_controller_exists_and_extends_node() {
    // Type A — file present; first non-comment, non-blank line == `extends Node`.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("zoom_lod_controller.gd");
    assert!(
        path.is_file(),
        "a1.1: zoom_lod_controller.gd must exist at {path:?}"
    );
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("a1.2: cannot read {path:?}: {e}"));
    let stripped = strip_gd_comments(&src);
    let head = first_code_line(&stripped)
        .expect("a1.3: zoom_lod_controller.gd has no non-blank, non-comment content");
    assert_eq!(
        head, "extends Node",
        "a1.4: first non-comment, non-blank line must be `extends Node` (not Node2D/Control); got `{head}`"
    );
    println!("[P14-ζ a1] zoom_lod_controller.gd exists + `extends Node` ✓");
}

// ─── Assertion 2: ZOOM_FAR_MAX == 1.0 ─────────────────────────────────────
#[test]
fn harness_zoom_a2_zoom_far_max_equals_one() {
    // Type A — exactly one ZOOM_FAR_MAX decl, RHS parses to 1.0.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_FAR_MAX", "a2");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("a2: ZOOM_FAR_MAX RHS must parse as float; got `{rhs}`"));
    assert!((f - 1.0).abs() < 1e-9, "a2: ZOOM_FAR_MAX must equal 1.0; got {f}");
    println!("[P14-ζ a2] ZOOM_FAR_MAX = 1.0 ✓");
}

// ─── Assertion 3: ZOOM_CLOSE_MIN == 2.0 ───────────────────────────────────
#[test]
fn harness_zoom_a3_zoom_close_min_equals_two() {
    // Type A — exactly one ZOOM_CLOSE_MIN decl, RHS parses to 2.0.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_CLOSE_MIN", "a3");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("a3: ZOOM_CLOSE_MIN RHS must parse as float; got `{rhs}`"));
    assert!((f - 2.0).abs() < 1e-9, "a3: ZOOM_CLOSE_MIN must equal 2.0; got {f}");
    println!("[P14-ζ a3] ZOOM_CLOSE_MIN = 2.0 ✓");
}

// ─── Assertion 4: 3-tier enum (FAR/MEDIUM/CLOSE) ──────────────────────────
#[test]
fn harness_zoom_a4_tier_enum_has_three_named_tiers() {
    // Type A — `enum Tier` AND FAR AND MEDIUM AND CLOSE all present.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let needles = ["enum Tier", "FAR", "MEDIUM", "CLOSE"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a4: zoom_lod_controller.gd must define a 3-tier Tier enum; missing={missing:?}"
    );
    println!("[P14-ζ a4] enum Tier {{ FAR, MEDIUM, CLOSE }} ✓");
}

// ─── Assertion 5: _tier_for_zoom uses both thresholds + both terminal tiers ─
#[test]
fn harness_zoom_a5_tier_for_zoom_uses_both_thresholds() {
    // Type A — _tier_for_zoom body references ZOOM_FAR_MAX, ZOOM_CLOSE_MIN,
    // Tier.FAR, Tier.CLOSE.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let (s, e) = find_func_body(&stripped, "_tier_for_zoom")
        .expect("a5.1: _tier_for_zoom function must exist");
    let body = &stripped[s..e];
    let needles = ["ZOOM_FAR_MAX", "ZOOM_CLOSE_MIN", "Tier.FAR", "Tier.CLOSE"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !body.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a5.2: _tier_for_zoom body must reference both thresholds and both terminal tiers; \
         missing={missing:?}; body:\n{body}"
    );
    println!("[P14-ζ a5] _tier_for_zoom references both thresholds + FAR/CLOSE ✓");
}

// ─── Assertion 6: _apply_tier toggles both renderers ──────────────────────
#[test]
fn harness_zoom_a6_apply_tier_toggles_both_renderers() {
    // Type A — _apply_tier body (ws-collapsed) contains `_trail_renderer.visible`
    // AND `_overview_renderer.visible`.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let (s, e) =
        find_func_body(&stripped, "_apply_tier").expect("a6.1: _apply_tier function must exist");
    let compact = no_ws(&stripped[s..e]);
    assert!(
        compact.contains("_trail_renderer.visible"),
        "a6.2: _apply_tier body must contain `_trail_renderer.visible`"
    );
    assert!(
        compact.contains("_overview_renderer.visible"),
        "a6.3: _apply_tier body must contain `_overview_renderer.visible`"
    );
    println!("[P14-ζ a6] _apply_tier toggles both sibling renderers ✓");
}

// ─── Assertion 7: resolves three sibling node paths ───────────────────────
#[test]
fn harness_zoom_a7_resolves_three_sibling_node_paths() {
    // Type A — three exact absolute-path literals present.
    let src = read_zoom_lod_src();
    let needles = [
        "/root/Main/Camera2D",
        "/root/Main/ActivityTrailRenderer",
        "/root/Main/SettlementOverviewRenderer",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !src.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a7: zoom_lod_controller.gd must resolve all 3 sibling node paths; missing={missing:?}"
    );
    println!("[P14-ζ a7] resolves Camera2D + ActivityTrailRenderer + SettlementOverviewRenderer ✓");
}

// ─── Assertion 8: _process reads live Camera2D.zoom ───────────────────────
#[test]
fn harness_zoom_a8_process_reads_live_camera_zoom() {
    // Type A — _process body (ws-collapsed) contains `_camera.zoom`.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let (s, e) =
        find_func_body(&stripped, "_process").expect("a8.1: _process function must exist");
    let compact = no_ws(&stripped[s..e]);
    assert!(
        compact.contains("_camera.zoom"),
        "a8.2: _process body must read `_camera.zoom` (live tweened zoom, not a target)"
    );
    println!("[P14-ζ a8] _process reads live _camera.zoom ✓");
}

// ─── Assertion 9 (strengthening): _ready applies initial tier ─────────────
#[test]
fn harness_zoom_a9_ready_applies_initial_tier() {
    // Type A — _ready body calls _apply_tier (initial CLOSE before first _process).
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let (s, e) = find_func_body(&stripped, "_ready").expect("a9.1: _ready function must exist");
    let body = &stripped[s..e];
    assert!(
        body.contains("_apply_tier"),
        "a9.2: _ready body must call `_apply_tier` to set the initial tier; body:\n{body}"
    );
    println!("[P14-ζ a9] _ready applies initial tier ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group B — scripts/ui/settlement_overview_renderer.gd
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 10: overview exists + `extends Node2D` ─────────────────────
#[test]
fn harness_zoom_a10_overview_exists_and_extends_node2d() {
    // Type A — file present; first non-comment, non-blank line == `extends Node2D`.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("settlement_overview_renderer.gd");
    assert!(
        path.is_file(),
        "a10.1: settlement_overview_renderer.gd must exist at {path:?}"
    );
    let src =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("a10.2: cannot read {path:?}: {e}"));
    let stripped = strip_gd_comments(&src);
    let head = first_code_line(&stripped)
        .expect("a10.3: settlement_overview_renderer.gd has no non-blank, non-comment content");
    assert_eq!(
        head, "extends Node2D",
        "a10.4: first non-comment, non-blank line must be `extends Node2D`; got `{head}`"
    );
    println!("[P14-ζ a10] settlement_overview_renderer.gd exists + `extends Node2D` ✓");
}

// ─── Assertion 11: Z_OVERVIEW == 1 ────────────────────────────────────────
#[test]
fn harness_zoom_a11_z_overview_equals_one() {
    // Type A — exactly one Z_OVERVIEW decl, RHS parses to 1.
    let stripped = strip_gd_comments(&read_overview_src());
    let rhs = unique_decl_rhs(&stripped, "Z_OVERVIEW", "a11");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("a11: Z_OVERVIEW RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 1, "a11: Z_OVERVIEW must equal 1; got {n}");
    println!("[P14-ζ a11] Z_OVERVIEW = 1 ✓");
}

// ─── Assertion 12: OVERVIEW_ALPHA == 0.28 ─────────────────────────────────
#[test]
fn harness_zoom_a12_overview_alpha_equals_0_28() {
    // Type A — exactly one OVERVIEW_ALPHA decl, RHS parses to 0.28.
    let stripped = strip_gd_comments(&read_overview_src());
    let rhs = unique_decl_rhs(&stripped, "OVERVIEW_ALPHA", "a12");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("a12: OVERVIEW_ALPHA RHS must parse as float; got `{rhs}`"));
    assert!(
        (f - 0.28).abs() < 1e-9,
        "a12: OVERVIEW_ALPHA must equal 0.28; got {f}"
    );
    println!("[P14-ζ a12] OVERVIEW_ALPHA = 0.28 ✓");
}

// ─── Assertion 13: reads ONLY settlement snapshot ─────────────────────────
#[test]
fn harness_zoom_a13_reads_only_settlement_snapshot() {
    // Type A — `get_settlement_snapshot` present; all 8 other FFI literals
    // absent (comments stripped first per edge-case discipline).
    let stripped = strip_gd_comments(&read_overview_src());
    assert!(
        stripped.contains("get_settlement_snapshot"),
        "a13.1: settlement_overview_renderer.gd must contain `get_settlement_snapshot`"
    );
    let forbidden = [
        "get_agent_snapshot",
        "get_construction_snapshot",
        "get_agent_detail",
        "get_tile_detail",
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
        "a13.2: settlement_overview_renderer.gd must reference only `get_settlement_snapshot`; \
         found forbidden FFI literals: {found:?}"
    );
    println!("[P14-ζ a13] snapshot scope limited to get_settlement_snapshot ✓");
}

// ─── Assertion 14: starts hidden + guards when hidden ─────────────────────
#[test]
fn harness_zoom_a14_starts_hidden_and_guards_when_hidden() {
    // Type A — _ready body (ws-collapsed) contains `visible=false`; _process
    // body contains `not visible` (early-return perf guard).
    let stripped = strip_gd_comments(&read_overview_src());
    let (rs, re) = find_func_body(&stripped, "_ready").expect("a14.1: _ready must exist");
    let ready_compact = no_ws(&stripped[rs..re]);
    assert!(
        ready_compact.contains("visible=false"),
        "a14.2: _ready body must set `visible = false` (start hidden at CLOSE default)"
    );
    let (ps, pe) = find_func_body(&stripped, "_process").expect("a14.3: _process must exist");
    let process_body = &stripped[ps..pe];
    assert!(
        process_body.contains("not visible"),
        "a14.4: _process body must early-return on `not visible` (perf guard); body:\n{process_body}"
    );
    println!("[P14-ζ a14] starts hidden + `not visible` guard in _process ✓");
}

// ─── Assertion 15: draw_circle radius scales by member_count ──────────────
#[test]
fn harness_zoom_a15_draw_circle_radius_scales_by_member_count() {
    // Type A — _draw body contains `draw_circle`; file contains `member_counts`
    // AND `BASE_RADIUS_PX`.
    let stripped = strip_gd_comments(&read_overview_src());
    let (s, e) = find_func_body(&stripped, "_draw").expect("a15.1: _draw must exist");
    let body = &stripped[s..e];
    assert!(
        body.contains("draw_circle"),
        "a15.2: _draw body must contain `draw_circle`; body:\n{body}"
    );
    assert!(
        stripped.contains("member_counts"),
        "a15.3: settlement_overview_renderer.gd must reference `member_counts`"
    );
    assert!(
        stripped.contains("BASE_RADIUS_PX"),
        "a15.4: settlement_overview_renderer.gd must reference `BASE_RADIUS_PX`"
    );
    println!("[P14-ζ a15] draw_circle radius scales by member_count ✓");
}

// ─── Assertion 16 (strengthening): world_renderer coordinate basis ────────
#[test]
fn harness_zoom_a16_centroid_uses_world_renderer_coordinate_basis() {
    // Type A — TILE_SIZE + SPRITE_ORIGIN_X + SPRITE_ORIGIN_Y all present.
    let stripped = strip_gd_comments(&read_overview_src());
    let needles = ["TILE_SIZE", "SPRITE_ORIGIN_X", "SPRITE_ORIGIN_Y"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a16: discs must share world_renderer SPRITE_ORIGIN + TILE_SIZE basis; missing={missing:?}"
    );
    println!("[P14-ζ a16] centroid uses world_renderer coordinate basis ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group C — scenes/main.tscn
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 17: load_steps == 11 ───────────────────────────────────────
#[test]
fn harness_zoom_a17_main_tscn_load_steps_equals_11() {
    // Type A — `[gd_scene ... load_steps=11 ...]`, ws around `=` tolerated.
    let src = read_main_tscn_src();
    let n = tscn_load_steps(&src)
        .expect("a17.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 11, "a17.2: load_steps must equal 11; got {n}");
    println!("[P14-ζ a17] load_steps = 11 ✓");
}

// ─── Assertion 18: registers both new nodes ───────────────────────────────
#[test]
fn harness_zoom_a18_main_tscn_registers_both_new_nodes() {
    // Type A — both script refs present AND both node lines present.
    let src = read_main_tscn_src();
    assert!(
        src.contains("zoom_lod_controller.gd"),
        "a18.1: main.tscn must reference `zoom_lod_controller.gd`"
    );
    assert!(
        src.contains("settlement_overview_renderer.gd"),
        "a18.2: main.tscn must reference `settlement_overview_renderer.gd`"
    );
    let mut found_zoom = false;
    let mut found_overview = false;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[node ") {
            continue;
        }
        if t.contains("name=\"ZoomLodController\"") && t.contains("type=\"Node\"") {
            found_zoom = true;
        }
        if t.contains("name=\"SettlementOverviewRenderer\"") && t.contains("type=\"Node2D\"") {
            found_overview = true;
        }
    }
    assert!(
        found_zoom,
        "a18.3: main.tscn must contain `[node name=\"ZoomLodController\" type=\"Node\" ...]`"
    );
    assert!(
        found_overview,
        "a18.4: main.tscn must contain `[node name=\"SettlementOverviewRenderer\" type=\"Node2D\" ...]`"
    );
    println!("[P14-ζ a18] both new sibling nodes registered in main.tscn ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group D — Cross-phase regression guards (Type D)
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 19: Phase 12-α/13-α camera_controller invariants intact ────
#[test]
fn harness_zoom_a19_phase12a_13a_camera_controller_zoom_invariants_intact() {
    // Type D — ZOOM_MIN/MAX/DEFAULT tokens + 3 zoom Vector2 literals.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let tokens = ["ZOOM_MIN", "ZOOM_MAX", "ZOOM_DEFAULT"];
    let mut missing: Vec<&str> = Vec::new();
    for n in tokens.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a19.1: camera_controller.gd must keep ZOOM_MIN/MAX/DEFAULT; missing={missing:?}"
    );
    let compact = no_ws(&stripped);
    let lits = ["Vector2(0.5,0.5)", "Vector2(4.0,4.0)", "Vector2(3.0,3.0)"];
    let mut missing_lits: Vec<&str> = Vec::new();
    for l in lits.iter() {
        if !compact.contains(l) {
            missing_lits.push(l);
        }
    }
    assert!(
        missing_lits.is_empty(),
        "a19.2: camera_controller.gd must keep 0.5/4.0/3.0 zoom literals; missing={missing_lits:?}"
    );
    println!("[P14-ζ a19] camera_controller.gd Phase 12-α/13-α invariants intact ✓");
}

// ─── Assertion 20: Phase 14-α agent_renderer invariants intact ────────────
#[test]
fn harness_zoom_a20_phase14a_agent_renderer_invariants_intact() {
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
        "a20: agent_renderer.gd must keep all 5 invariant literals; missing={missing:?}"
    );
    println!("[P14-ζ a20] agent_renderer.gd invariants intact ✓");
}

// ─── Assertion 21: Phase 14-β world_renderer invariants intact ────────────
#[test]
fn harness_zoom_a21_phase14b_world_renderer_invariants_intact() {
    // Type D — RESOURCE_TYPE_PATHS + VILLAGE_FIXTURE_ + RESOURCE_COUNT=20 +
    // RESOURCE_SEED=88675123.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    assert!(
        stripped.contains("RESOURCE_TYPE_PATHS"),
        "a21.1: world_renderer.gd must keep RESOURCE_TYPE_PATHS"
    );
    assert!(
        stripped.contains("VILLAGE_FIXTURE_"),
        "a21.2: world_renderer.gd must keep at least one VILLAGE_FIXTURE_ identifier"
    );
    let count_rhs = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "a21.count");
    let count = parse_int_rhs(&count_rhs)
        .unwrap_or_else(|| panic!("a21.3: RESOURCE_COUNT RHS must parse as int; got `{count_rhs}`"));
    assert_eq!(count, 20, "a21.4: RESOURCE_COUNT must equal 20; got {count}");
    let seed_rhs = unique_decl_rhs(&stripped, "RESOURCE_SEED", "a21.seed");
    let seed = parse_int_rhs(&seed_rhs)
        .unwrap_or_else(|| panic!("a21.5: RESOURCE_SEED RHS must parse as int; got `{seed_rhs}`"));
    assert_eq!(seed, 88675123, "a21.6: RESOURCE_SEED must equal 88675123; got {seed}");
    println!("[P14-ζ a21] world_renderer.gd Phase 14-β invariants intact ✓");
}

// ─── Assertion 22: Phase 14-ε activity_trail_renderer intact ──────────────
#[test]
fn harness_zoom_a22_phase14e_activity_trail_renderer_intact() {
    // Type D — seven canonical trail tokens all present (ζ must NOT edit ε file).
    let stripped = strip_gd_comments(&read_trail_renderer_src());
    let needles = [
        "TRAIL_LENGTH",
        "TRAIL_WIDTH",
        "TRAIL_ALPHA",
        "Z_TRAIL",
        "TRAIL_COLOR_SEEKING",
        "TRAIL_COLOR_CONSUMING_AGENT",
        "TRAIL_COLOR_CONSUMING_OTHER",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a22: activity_trail_renderer.gd must keep all 7 Phase 14-ε tokens; missing={missing:?}"
    );
    println!("[P14-ζ a22] activity_trail_renderer.gd Phase 14-ε invariants intact ✓");
}

// ─── Assertion 23: Phase 14-δ hud_status_panel intact ─────────────────────
#[test]
fn harness_zoom_a23_phase14d_hud_status_panel_intact() {
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
        "a23: hud_status_panel.gd must keep all 3 invariant literals; missing={missing:?}"
    );
    println!("[P14-ζ a23] hud_status_panel.gd Phase 14-δ invariants intact ✓");
}

// ─── Assertion 24: Phase 13-δ hud_topbar intact ───────────────────────────
#[test]
fn harness_zoom_a24_phase13d_hud_topbar_intact() {
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
        "a24: hud_topbar.gd must keep all 6 invariant literals (Phase 13-δ A4/A5/A6); missing={missing:?}"
    );
    println!("[P14-ζ a24] hud_topbar.gd Phase 13-δ invariants intact ✓");
}
