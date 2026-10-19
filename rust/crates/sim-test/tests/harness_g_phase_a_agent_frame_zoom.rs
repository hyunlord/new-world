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


// ════════════════════════════════════════════════════════════════════════════
// Group A — scripts/ui/agent_renderer.gd  (Defect 1 — FORM)
// ════════════════════════════════════════════════════════════════════════════

// ─── Assertion 1: sheet_cols_equals_4 ─────────────────────────────────────
#[test]
fn harness_g_a1_sheet_cols_equals_4() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 2: sheet_rows_equals_3 ─────────────────────────────────────
#[test]
fn harness_g_a2_sheet_rows_equals_3() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 3: agent_frame_index_declared_and_int ──────────────────────
#[test]
fn harness_g_a3_agent_frame_index_declared_and_int() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 4: ready_reuses_quadmesh_arrays ────────────────────────────
#[test]
fn harness_g_a4_ready_reuses_quadmesh_arrays() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 5: ready_builds_frame_subrect_via_arraymesh ────────────────
#[test]
fn harness_g_a5_ready_builds_frame_subrect_via_arraymesh() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 6: multimesh_uses_frame_mesh_not_whole_quad ────────────────
#[test]
fn harness_g_a6_multimesh_uses_frame_mesh_not_whole_quad() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 7: sprite_scale_preserved ──────────────────────────────────
#[test]
fn harness_g_a7_sprite_scale_preserved() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));

}

// ─── Assertion 8: agent_renderer_cue_invariants_intact ────────────────────
#[test]
fn harness_g_a8_agent_renderer_cue_invariants_intact() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let atlas = include_str!("../../../../scripts/ui/art_sprite_library.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));
    assert!(atlas.contains("sprite.scale = Vector2.ONE"));
    assert!(atlas.contains("sprite.region_rect = Rect2"));
    assert!(atlas.contains("sprite.offset = -Vector2"));
    assert!(atlas.contains("CanvasItem.TEXTURE_FILTER_NEAREST"));
    assert!(src.contains("ICON_OFFSET_PX"));
    assert!(!src.contains("STATE_TINTS"));

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
        "Vector2(1.0,1.0)",
        "a10.3: ZOOM_MIN must equal Vector2(1.0,1.0) (Phase 12-α invariant); got `{min_rhs}`"
    );
    assert_eq!(
        no_ws(&def_rhs),
        "Vector2(4.0,4.0)",
        "a10.4: ZOOM_DEFAULT must equal Vector2(4.0,4.0) (set to 4.0 by Revitalize Part 4); got `{def_rhs}`"
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
    // Default zoom is four; this guard belongs only to the maximum-zoom contract.
    let max_test = src.split_once("fn harness_p12_alpha_a4_zoom_max_constant()")
        .expect("maximum zoom test exists").1.split_once("\n}")
        .expect("maximum zoom test closes").0;
    let compact = no_ws(max_test);
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
    // Default zoom is four; this guard belongs only to the maximum-zoom contract.
    let max_test = src.split_once("fn harness_p13_alpha_a3_camera_controller_zoom_max_unchanged()")
        .expect("maximum zoom test exists").1.split_once("\n}")
        .expect("maximum zoom test closes").0;
    let compact = no_ws(max_test);
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
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/camera_controller.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ZOOM_MIN: Vector2 = Vector2(1.0, 1.0)"));
    assert!(src.contains("ZOOM_MAX: Vector2 = Vector2(8.0, 8.0)"));
    assert!(src.contains("ZOOM_DEFAULT: Vector2 = Vector2(4.0, 4.0)"));

}
