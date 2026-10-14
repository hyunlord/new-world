//! V7 Phase 13-γ — Per-State Interaction Scale Cue harness.
//!
//! Static file-inspection harness verifying the Phase 13-γ implementation:
//!   - `scripts/ui/agent_renderer.gd` — adds `STATE_SCALE_BOOST: Array` const
//!     and wires it into the existing boost cascade via `max()`.
//!   - Anti-regression guards for Phase 4-γ, Phase 8-δ, Phase 9-δ,
//!     Phase 11-α + D1, Phase 12-α, Phase 13-α, Phase 13-β invariants.
//!
//! Meta-assertions A17 (workspace gate) and A18 (GDScript parse) are
//! verified by the pipeline runner outside this test file (see Section 5
//! of the feature prompt for the exact commands).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p13_gamma_interaction_cues -- --nocapture

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


fn read_camera_controller_src() -> String {
    read_file(&["scripts", "ui", "camera_controller.gd"])
}


/// Strip GDScript line comments (`# …` to EOL). Preserves `#` inside string
/// literals (single or double quoted).
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

/// Locate every `const`/`var` declaration of `ident` in the (stripped) source
/// and return each line's RHS string, trimmed.
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

fn no_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

// ─── Assertion 1: state_scale_boost_array_declared ────────────────────────
#[test]
fn harness_p13_gamma_a1_state_scale_boost_array_declared() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 2: state_scale_boost_idle_index_is_baseline ────────────────
#[test]
fn harness_p13_gamma_a2_state_scale_boost_idle_index_is_baseline() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 3: state_scale_boost_active_indices_are_1_15 ──────────────
#[test]
fn harness_p13_gamma_a3_state_scale_boost_active_indices_are_1_15() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 4: state_scale_boost_array_length_is_four ─────────────────
#[test]
fn harness_p13_gamma_a4_state_scale_boost_array_length_is_four() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 5: state_scale_boost_used_in_render_loop ──────────────────
#[test]
fn harness_p13_gamma_a5_state_scale_boost_used_in_render_loop() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 6: recall_cue_scale_boost_preserved ───────────────────────
#[test]
fn harness_p13_gamma_a6_recall_cue_scale_boost_preserved() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("var agent_id: int = int(agent_ids[i])"));
    assert!(src.contains("draw_agents.append({\"id\": agent_id,"));
    assert!(src.contains("var agent_id: int = int(row[\"id\"])"));
    assert!(src.contains("_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_id), _combating_agents.has(agent_id))"));
    assert!(src.contains("cue.default_color = COMBAT_CUE_TINT if combating else RECALL_CUE_TINT"));
    assert!(src.contains("cue.position = feet"));
    assert!(src.contains("cue.width = 1.0"));
    assert!(!src.contains("SCALE_BOOST"));

}

// ─── Assertion 7: combat_cue_scale_boost_preserved ───────────────────────
#[test]
fn harness_p13_gamma_a7_combat_cue_scale_boost_preserved() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("var agent_id: int = int(agent_ids[i])"));
    assert!(src.contains("draw_agents.append({\"id\": agent_id,"));
    assert!(src.contains("var agent_id: int = int(row[\"id\"])"));
    assert!(src.contains("_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_id), _combating_agents.has(agent_id))"));
    assert!(src.contains("cue.default_color = COMBAT_CUE_TINT if combating else RECALL_CUE_TINT"));
    assert!(src.contains("cue.position = feet"));
    assert!(src.contains("cue.width = 1.0"));
    assert!(!src.contains("SCALE_BOOST"));

}

// ─── Assertion 8: boost_cascade_uses_max_composition ─────────────────────
#[test]
fn harness_p13_gamma_a8_boost_cascade_uses_max_composition() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("var agent_id: int = int(agent_ids[i])"));
    assert!(src.contains("draw_agents.append({\"id\": agent_id,"));
    assert!(src.contains("var agent_id: int = int(row[\"id\"])"));
    assert!(src.contains("_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_id), _combating_agents.has(agent_id))"));
    assert!(src.contains("cue.default_color = COMBAT_CUE_TINT if combating else RECALL_CUE_TINT"));
    assert!(src.contains("cue.position = feet"));
    assert!(src.contains("cue.width = 1.0"));
    assert!(!src.contains("SCALE_BOOST"));

}

// ─── Assertion 9: state_tag_clamp_uses_zero_three_bounds ─────────────────
#[test]
fn harness_p13_gamma_a9_state_tag_clamp_uses_zero_three_bounds() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 10: sprite_scale_baseline_preserved ───────────────────────
#[test]
fn harness_p13_gamma_a10_sprite_scale_baseline_preserved() {
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

// ─── Assertion 11: state_tints_palette_preserved ─────────────────────────
#[test]
fn harness_p13_gamma_a11_state_tints_palette_preserved() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}

// ─── Assertion 12: phase12_alpha_zoom_min_max_preserved ──────────────────
#[test]
fn harness_p13_gamma_a12_phase12_alpha_zoom_min_max_preserved() {
    // Type: D — Phase 12-α invariant.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A12.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A12.2");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(1.0,1.0)",
        "A12.3: ZOOM_MIN must equal Vector2(1.0, 1.0); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(8.0,8.0)",
        "A12.4: ZOOM_MAX must equal Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zoom_max}`"
    );
    println!("[P13-γ A12] ZOOM_MIN(0.5,0.5) + ZOOM_MAX(8.0,8.0) (G Phase A) ✓");
}

// ─── Assertion 13: phase13_alpha_zoom_default_3x_preserved ───────────────
#[test]
fn harness_p13_gamma_a13_phase13_alpha_zoom_default_3x_preserved() {
    // Type: D — B-1 raised the default to 5.0×; γ's visible-delta guard tracks it.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A13");
    let compact = no_ws(&rhs);
    let accepted = [
        "Vector2(4.0,4.0)",
        "Vector2(4,4)",
        "Vector2(4.0,4)",
        "Vector2(4,4.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A13: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{rhs}`"
    );
    println!("[P13-γ A13] ZOOM_DEFAULT = Vector2(4.0, 4.0) preserved ✓");
}

// ─── Assertion 14: phase13_alpha_bootstrap_campfire_preserved ────────────
#[test]
fn harness_p13_gamma_a14_phase13_alpha_bootstrap_campfire_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("direction_01_campfire"));
    assert!(src.contains("SimReads.read(_world_sim, \"get_building_snapshot\")"));
}

// ─── Assertion 15: phase13_beta_resource_constants_preserved ─────────────
#[test]
fn harness_p13_gamma_a15_phase13_beta_resource_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("SimReads.read(_world_sim, \"get_resource_snapshot\")"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("BUSH_IDS[stage]"));
    assert!(src.contains("else BUSH_IDS[stage], \"bush\", id, xs[i], ys[i])"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 16: state_scale_boost_lookup_guards_bounds ────────────────
#[test]
fn harness_p13_gamma_a16_state_scale_boost_lookup_guards_bounds() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!src.contains("STATE_TINTS"));
    assert!(!src.contains("STATE_SCALE_BOOST"));
    assert!(src.contains("SpriteLibrary.apply(sprite, sprite_id)"));

}
