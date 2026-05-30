//! V7 G Phase A — Agent single-frame fix + zoom-in headroom harness.
//!
//! feature: g-phase-a-agent-frame-zoom
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Pure static file-inspection harness (Phase 14-ε/ζ precedent). No simulation
//! is run (`ticks: 0`), no ECS component is queried — the two defects (agent
//! sprite UV region + camera zoom-in ceiling) are GDScript/visual and cannot be
//! observed headless (the camera cannot be driven; per-sprite UV changes are
//! sub-resolution for the VLM — see "VLM Visual Verification — Known
//! Limitation"). Every assertion reads source text via the established helpers.
//! The authoritative behavioural gate is the windowed-Godot review (Section 7),
//! not this harness.
//!
//! Run:
//!   cargo test -p sim-test --test harness_g_phase_a_agent_frame_zoom -- --nocapture

use std::fs;
use std::path::PathBuf;

// ── helpers (Phase 14-ε/ζ precedent) ────────────────────────────────────────

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

fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_camera_controller_src() -> String {
    read_file(&["scripts", "ui", "camera_controller.gd"])
}

fn read_palette_shader_src() -> String {
    read_file(&["shaders", "palette_swap.gdshader"])
}

fn read_zoom_lod_src() -> String {
    read_file(&["scripts", "ui", "zoom_lod_controller.gd"])
}

fn read_test_file(name: &str) -> String {
    read_file(&["rust", "crates", "sim-test", "tests", name])
}

/// Strip `#` line comments while respecting string literals.
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

// ════════════════════════════════════════════════════════════════════════════
// Group A — scripts/ui/agent_renderer.gd  (Defect 1 — FORM)
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 1: sheet_cols_equals_4 ─────────────────────────────────────
#[test]
fn harness_g_a1_sheet_cols_equals_4() {
    // Type A — physical invariant. agent_base.png is 64px wide = 4 cols of 16px.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SHEET_COLS", "a1");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("a1: SHEET_COLS RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 4, "a1: SHEET_COLS must equal 4 (64px / 16px); got {n}");
    println!("[G a1] SHEET_COLS = 4 ✓");
}

// ─── Assertion 2: sheet_rows_equals_3 ─────────────────────────────────────
#[test]
fn harness_g_a2_sheet_rows_equals_3() {
    // Type A — physical invariant. agent_base.png is 72px tall = 3 rows of 24px.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SHEET_ROWS", "a2");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("a2: SHEET_ROWS RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 3, "a2: SHEET_ROWS must equal 3 (72px / 24px); got {n}");
    println!("[G a2] SHEET_ROWS = 3 ✓");
}

// ─── Assertion 3: agent_frame_index_declared_and_int ──────────────────────
#[test]
fn harness_g_a3_agent_frame_index_declared_and_int() {
    // Type A — exactly one AGENT_FRAME_INDEX decl, RHS parses to 0 (top-left).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhss = find_decl_rhss(&stripped, "AGENT_FRAME_INDEX");
    assert_eq!(
        rhss.len(),
        1,
        "a3.1: expected exactly 1 declaration of `AGENT_FRAME_INDEX`; got {n}: {rhss:?}",
        n = rhss.len()
    );
    let rhs = &rhss[0];
    let n = parse_int_rhs(rhs)
        .unwrap_or_else(|| panic!("a3.2: AGENT_FRAME_INDEX RHS must parse as int; got `{rhs}`"));
    assert!(n >= 0, "a3.3: AGENT_FRAME_INDEX must be non-negative; got {n}");
    assert_eq!(n, 0, "a3.4: AGENT_FRAME_INDEX must equal 0 (frame 0 = top-left cell); got {n}");
    println!("[G a3] AGENT_FRAME_INDEX declared once, = 0 ✓");
}

// ─── Assertion 4: ready_reuses_quadmesh_arrays ────────────────────────────
#[test]
fn harness_g_a4_ready_reuses_quadmesh_arrays() {
    // Type A — _ready reuses the QuadMesh's OWN generated vertex+UV arrays
    // (flip-safe). Both tokens are logically required to reuse-and-remap.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let (s, e) = find_func_body(&stripped, "_ready").expect("a4.1: _ready must exist");
    let body = &stripped[s..e];
    assert!(
        body.contains("get_mesh_arrays"),
        "a4.2: _ready must call `get_mesh_arrays` (reuse QuadMesh's own arrays); body:\n{body}"
    );
    assert!(
        body.contains("Mesh.ARRAY_TEX_UV"),
        "a4.3: _ready must index `Mesh.ARRAY_TEX_UV` (remap UV array); body:\n{body}"
    );
    println!("[G a4] _ready reuses QuadMesh arrays (get_mesh_arrays + ARRAY_TEX_UV) ✓");
}

// ─── Assertion 5: ready_builds_frame_subrect_via_arraymesh ────────────────
#[test]
fn harness_g_a5_ready_builds_frame_subrect_via_arraymesh() {
    // Type A — the frame sub-rect divides UV span by the sheet grid and rebuilds
    // a mesh from the remapped arrays. All four tokens are necessary.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let (s, e) = find_func_body(&stripped, "_ready").expect("a5.1: _ready must exist");
    let body = &stripped[s..e];
    let needles = ["SHEET_COLS", "SHEET_ROWS", "ArrayMesh", "add_surface_from_arrays"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !body.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a5.2: _ready must scale UVs by sheet grid + rebuild via ArrayMesh; missing={missing:?}; \
         body:\n{body}"
    );
    println!("[G a5] _ready builds frame sub-rect via ArrayMesh ✓");
}

// ─── Assertion 6: multimesh_uses_frame_mesh_not_whole_quad ────────────────
#[test]
fn harness_g_a6_multimesh_uses_frame_mesh_not_whole_quad() {
    // Type A — discriminating negative check. The new frame-mesh assignment must
    // be present AND the old whole-sheet assignment must be GONE (replaced, not
    // supplemented). Runs on comment-stripped, whitespace-collapsed _ready body.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let (s, e) = find_func_body(&stripped, "_ready").expect("a6.1: _ready must exist");
    let compact = no_ws(&stripped[s..e]);
    assert!(
        compact.contains("multi_mesh.mesh=frame_mesh"),
        "a6.2: _ready must assign `multi_mesh.mesh = frame_mesh` (the single-frame mesh)"
    );
    assert!(
        !compact.contains("multi_mesh.mesh=quad"),
        "a6.3: _ready must NOT keep the old `multi_mesh.mesh = quad` (whole-sheet) assignment — \
         the fix must replace it, not supplement it"
    );
    println!("[G a6] multi_mesh uses frame_mesh, whole-quad assignment gone ✓");
}

// ─── Assertion 7: sprite_scale_preserved ──────────────────────────────────
#[test]
fn harness_g_a7_sprite_scale_preserved() {
    // Type D — Phase 4-γ tile-fit invariant (16/64 = 0.25). The form fix is
    // UV-only; SPRITE_SCALE must NOT be changed.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "a7.1");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("a7.2: SPRITE_SCALE RHS must parse as float; got `{rhs}`"));
    assert!(
        (f - 0.25).abs() < 1e-9,
        "a7.3: SPRITE_SCALE must equal 0.25 (Phase 4-γ invariant; form fixed via UV); got {f}"
    );
    println!("[G a7] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 8: agent_renderer_cue_invariants_intact ────────────────────
#[test]
fn harness_g_a8_agent_renderer_cue_invariants_intact() {
    // Type D — Phases 8-δ/9-δ/11-α/13-γ/14-α cue logic + 64×72 quad-size consts
    // must survive a UV-only edit.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let needles = [
        "ROLE_BUCKET_COUNT",
        "ICON_OFFSET_PX",
        "STATE_TINTS",
        "STATE_SCALE_BOOST",
        "SPRITE_W",
        "SPRITE_H",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a8: agent_renderer.gd must keep all 6 cue/sprite invariant tokens; missing={missing:?}"
    );
    println!("[G a8] agent_renderer.gd cue/sprite invariants intact ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group B — scripts/ui/camera_controller.gd  (Defect 2 — ZOOM)
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 9: zoom_max_equals_eight ───────────────────────────────────
#[test]
fn harness_g_a9_zoom_max_equals_eight() {
    // Type A — contractual new zoom-in ceiling. 4.0 → 8.0.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_MAX", "a9.1");
    let compact = no_ws(&rhs);
    assert_eq!(
        compact, "Vector2(8.0,8.0)",
        "a9.2: ZOOM_MAX RHS must equal Vector2(8.0,8.0) (whitespace-collapsed; raised from 4.0); \
         got `{rhs}`"
    );
    println!("[G a9] ZOOM_MAX = Vector2(8.0, 8.0) ✓");
}

// ─── Assertion 10: zoom_min_and_default_preserved ─────────────────────────
#[test]
fn harness_g_a10_zoom_min_and_default_preserved() {
    // Type D — Phase 12-α ZOOM_MIN (0.5) locked; ZOOM_DEFAULT was raised
    // 3.0 → 5.0 in B-1 (this guard tracks the new value).
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let min_rhs = unique_decl_rhs(&stripped, "ZOOM_MIN", "a10.1");
    let def_rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "a10.2");
    assert_eq!(
        no_ws(&min_rhs),
        "Vector2(0.5,0.5)",
        "a10.3: ZOOM_MIN must equal Vector2(0.5,0.5) (Phase 12-α invariant); got `{min_rhs}`"
    );
    assert_eq!(
        no_ws(&def_rhs),
        "Vector2(5.0,5.0)",
        "a10.4: ZOOM_DEFAULT must equal Vector2(5.0,5.0) (raised from 3.0 in B-1); got `{def_rhs}`"
    );
    println!("[G a10] ZOOM_MIN(0.5) + ZOOM_DEFAULT(5.0) preserved ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group C — Cross-phase regression-guard reference files
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 11: palette_shader_color_path ──────────────────────────────
#[test]
fn harness_g_a11_palette_shader_color_path() {
    // Type D — shader color-logic path tracker. V7 H Phase A fixed the
    // green-bug by moving the per-instance tint capture from fragment() to
    // vertex() (`v_tint = COLOR`) and multiplying `palette_color.rgb *
    // v_tint.rgb`. The old `vec4 modulate = COLOR` / `* modulate.rgb` literals
    // now survive only inside the explanatory header comment, so this presence
    // check tracks the NEW code literals. (No negative guard on the old
    // literals here: this assertion reads the shader WITHOUT comment-stripping,
    // and the header comment legitimately references them.)
    let src = read_palette_shader_src();
    let needles = [
        "texture(TEXTURE, UV)",
        "v_tint = COLOR",
        "palette_color.rgb * v_tint.rgb",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !src.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a11: palette_swap.gdshader must keep its (H Phase A) color-logic path; missing={missing:?}"
    );
    println!("[G a11] palette_swap.gdshader color path tracks v_tint fix ✓");
}

// ─── Assertion 12: zoom_lod_controller_thresholds_intact ──────────────────
#[test]
fn harness_g_a12_zoom_lod_controller_thresholds_intact() {
    // Type D — Phase 14-ζ LOD tiers are independent of ZOOM_MAX; zoom 8.0 is
    // still classified CLOSE (8.0 ≥ 2.0).
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    let needles = ["ZOOM_FAR_MAX", "ZOOM_CLOSE_MIN"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a12: zoom_lod_controller.gd must keep its LOD threshold tokens; missing={missing:?}"
    );
    println!("[G a12] zoom_lod_controller.gd thresholds intact ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group D — Existing-harness regression-contract updates (ZOOM_MAX 4.0 → 8.0)
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 13: existing_p12_alpha_a4_updated_to_eight ──────────────────
#[test]
fn harness_g_a13_existing_p12_alpha_a4_updated_to_eight() {
    // Type D — p12-α A4 accepted-literal list must now track 8.0, not 4.0.
    let src = read_test_file("harness_p12_alpha_camera_zoom.rs");
    let compact = no_ws(&src);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a13.1: harness_p12_alpha_camera_zoom.rs must accept Vector2(8.0,8.0) (raised in G Phase A)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a13.2: harness_p12_alpha_camera_zoom.rs must NOT still assert the old Vector2(4.0,4.0) \
         ZOOM_MAX literal"
    );
    println!("[G a13] p12-α A4 tracks ZOOM_MAX = 8.0 ✓");
}

// ─── Assertion 14: existing_p13_alpha_a3_updated_to_eight ──────────────────
#[test]
fn harness_g_a14_existing_p13_alpha_a3_updated_to_eight() {
    // Type D — p13-α A3 expected ZOOM_MAX literal must now equal 8.0.
    let src = read_test_file("harness_p13_alpha_camera_and_buildings.rs");
    let compact = no_ws(&src);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a14.1: harness_p13_alpha_camera_and_buildings.rs must expect Vector2(8.0,8.0)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a14.2: harness_p13_alpha_camera_and_buildings.rs must NOT still expect Vector2(4.0,4.0)"
    );
    println!("[G a14] p13-α A3 expects ZOOM_MAX = 8.0 ✓");
}

// ─── Assertion 15: existing_p14_zeta_a19_regression_literal_updated ────────
#[test]
fn harness_g_a15_existing_p14_zeta_regression_literal_updated() {
    // Type D — p14-ζ camera-controller regression literal list must track 8.0
    // while still locking the unchanged min(0.5); the default was raised
    // 3.0 → 5.0 in B-1, so p14-ζ now carries Vector2(5.0,5.0).
    let src = read_test_file("harness_p14_zeta_zoom_adaptive.rs");
    let compact = no_ws(&src);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a15.1: harness_p14_zeta_zoom_adaptive.rs must contain Vector2(8.0,8.0)"
    );
    assert!(
        compact.contains("Vector2(0.5,0.5)"),
        "a15.2: harness_p14_zeta_zoom_adaptive.rs must retain Vector2(0.5,0.5)"
    );
    assert!(
        compact.contains("Vector2(5.0,5.0)"),
        "a15.3: harness_p14_zeta_zoom_adaptive.rs must carry Vector2(5.0,5.0) \
         (ZOOM_DEFAULT raised from 3.0 in B-1)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a15.4: harness_p14_zeta_zoom_adaptive.rs must NOT still require Vector2(4.0,4.0)"
    );
    println!("[G a15] p14-ζ regression literal tracks 8.0 (+0.5/5.0 retained) ✓");
}
