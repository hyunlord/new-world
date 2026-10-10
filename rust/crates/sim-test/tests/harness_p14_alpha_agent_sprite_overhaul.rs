//! V7 Phase 14-α — Agent Sprite Overhaul (Per-Role HUE + Head-Icon Mount Point) harness.
//!
//! Static file-inspection harness verifying the Phase 14-α implementation:
//!   - `scripts/ui/agent_renderer.gd` — adds ROLE_BUCKET_COUNT, ICON_OFFSET_PX,
//!     ICON_SIZE_PX constants, a `_role_bucket()` helper, extends
//!     `_palette_for_id` signature to (eid, agent_id), updates the
//!     `set_instance_custom_data` call site, adds `head_icon_position()` helper.
//!   - Anti-regression guards for Phase 4-γ, Phase 8-δ, Phase 9-δ,
//!     Phase 11-α + D1, Phase 12-α, Phase 13-α/β/γ/ε invariants.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_alpha_agent_sprite_overhaul -- --nocapture

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

fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
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

/// Extract the body of a top-level `func <name>(...)` in `stripped` source.
/// Returns the concatenated body lines (all lines more indented than the
/// `func` header line), or `None` if not found.
fn extract_func_body(stripped: &str, func_signature_prefix: &str) -> Option<String> {
    let lines: Vec<&str> = stripped.lines().collect();
    let mut header_idx: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with(func_signature_prefix) {
            header_idx = Some(idx);
            break;
        }
    }
    let idx = header_idx?;
    let header = lines[idx];
    let header_indent = header.len() - header.trim_start().len();
    let mut body = String::new();
    for body_line in &lines[idx + 1..] {
        if body_line.trim().is_empty() {
            body.push_str(body_line);
            body.push('\n');
            continue;
        }
        let li = body_line.len() - body_line.trim_start().len();
        if li <= header_indent {
            break;
        }
        body.push_str(body_line);
        body.push('\n');
    }
    Some(body)
}

// ─── Assertion 1: role_bucket_count_constant_declared ─────────────────────
#[test]
fn harness_p14_alpha_a1_role_bucket_count_declared() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 2: role_bucket_function_signature_present ──────────────────
#[test]
fn harness_p14_alpha_a2_role_bucket_function_signature_present() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 3: role_bucket_body_uses_agent_id_knuth_hash ───────────────
#[test]
fn harness_p14_alpha_a3_role_bucket_body_uses_agent_id_knuth_hash() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 4: palette_for_id_signature_extended ───────────────────────
#[test]
fn harness_p14_alpha_a4_palette_for_id_signature_extended() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 5: palette_body_channel_keyed_off_role_bucket ──────────────
#[test]
fn harness_p14_alpha_a5_palette_body_channel_keyed_off_role_bucket() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 6: palette_hair_and_skin_preserve_entity_bits_hashing ──────
#[test]
fn harness_p14_alpha_a6_palette_hair_and_skin_preserve_entity_bits_hashing() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 7: palette_call_site_passes_both_ids ───────────────────────
#[test]
fn harness_p14_alpha_a7_palette_call_site_passes_both_ids() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = include_str!("../../../../scripts/ui/art_hash.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ArtHashScript.look(agent_id)"));
    assert!(hash.contains("mix(agent_id)"));
    assert!(hash.contains("\"skin\": posmod(h >> 9, 4)"));
    assert!(hash.contains("\"hair\": posmod(h >> 11, 5)"));
    assert!(hash.contains("\"clothing\": posmod(h >> 15, 6)"));

}

// ─── Assertion 8: icon_offset_px_constant_declared ────────────────────────
#[test]
fn harness_p14_alpha_a8_icon_offset_px_declared() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("ICON_OFFSET_PX := Vector2(0, -30)"));
    assert!(src.contains("return world_pos + ICON_OFFSET_PX"));

}

// ─── Assertion 9: icon_size_px_constant_declared ──────────────────────────
#[test]
fn harness_p14_alpha_a9_icon_size_px_declared() {
    // Type: A — must be declared exactly once as Vector2(16, 16).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "ICON_SIZE_PX", "A9");
    let compact = no_ws(&rhs);
    assert_eq!(
        compact, "Vector2(16,16)",
        "A9: ICON_SIZE_PX must equal Vector2(16, 16); got `{rhs}`"
    );
    println!("[P14-α A9] ICON_SIZE_PX = Vector2(16, 16) ✓");
}

// ─── Assertion 10: head_icon_position_helper_declared ─────────────────────
#[test]
fn harness_p14_alpha_a10_head_icon_position_helper_declared() {
    // Type: A — must contain the signature AND body must return
    // `world_pos + ICON_OFFSET_PX`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let sig = "func head_icon_position(world_pos: Vector2) -> Vector2";
    let sig_count = stripped.matches(sig).count();
    assert_eq!(
        sig_count, 1,
        "A10.1: signature `{sig}` must appear exactly once; got {sig_count}"
    );
    let body = extract_func_body(&stripped, "func head_icon_position(")
        .expect("A10.2: head_icon_position body must exist");
    let compact = no_ws(&body);
    assert!(
        compact.contains("returnworld_pos+ICON_OFFSET_PX"),
        "A10.3: head_icon_position body must `return world_pos + ICON_OFFSET_PX`; body=\n{body}"
    );
    println!("[P14-α A10] head_icon_position(world_pos) -> Vector2 declared with correct body ✓");
}

// ─── Assertion 11: sprite_scale_invariant_preserved ───────────────────────
#[test]
fn harness_p14_alpha_a11_sprite_scale_preserved() {
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

// ─── Assertion 12: state_tints_palette_preserved ──────────────────────────
#[test]
fn harness_p14_alpha_a12_state_tints_palette_preserved() {
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

// ─── Assertion 13: state_scale_boost_preserved ────────────────────────────
#[test]
fn harness_p14_alpha_a13_state_scale_boost_preserved() {
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

// ─── Assertion 14: recall_and_combat_cue_constants_preserved ──────────────
#[test]
fn harness_p14_alpha_a14_recall_and_combat_cue_constants_preserved() {
    // Art production Part 2: see docs/art/TEST_LEDGER.md.
    let src = include_str!("../../../../scripts/ui/agent_renderer.gd")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(src.contains("_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_ids[i]), _combating_agents.has(agent_ids[i]))"));
    assert!(src.contains("cue.default_color = COMBAT_CUE_TINT if combating else RECALL_CUE_TINT"));
    assert!(src.contains("cue.position = feet"));
    assert!(src.contains("cue.width = 1.0"));
    assert!(!src.contains("SCALE_BOOST"));

}

// ─── Assertion 15: camera_zoom_invariants_preserved ───────────────────────
#[test]
fn harness_p14_alpha_a15_camera_zoom_invariants_preserved() {
    // Type: D — Phase 12-α + Phase 13-α invariants. Scope discipline guard.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A15.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A15.2");
    let zoom_default = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A15.3");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(1.0,1.0)",
        "A15.4: ZOOM_MIN must equal Vector2(1.0, 1.0); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(8.0,8.0)",
        "A15.5: ZOOM_MAX must equal Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zoom_max}`"
    );
    let default_compact = no_ws(&zoom_default);
    let accepted = [
        "Vector2(4.0,4.0)",
        "Vector2(4,4)",
        "Vector2(4.0,4)",
        "Vector2(4,4.0)",
    ];
    assert!(
        accepted.contains(&default_compact.as_str()),
        "A15.6: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{zoom_default}`"
    );
    println!("[P14-α A15] camera ZOOM_MIN/MAX/DEFAULT invariants preserved ✓");
}

// ─── Assertion 16: phase13_bootstrap_three_campfires_preserved ────────────
#[test]
fn harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("hearth_frame(_hour(), _elapsed)"));
    assert!(src.contains("sprite.position = _tile_center(x, y)"));
    assert!(src.contains("_configure(sprite, desired, \"building\", ids[i], xs[i], ys[i])"));
}

// ─── Assertion 17: phase13_beta_resource_constants_preserved ──────────────
#[test]
fn harness_p14_alpha_a17_phase13_beta_resource_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("BUSH_IDS[stage]"));
    assert!(src.contains("else BUSH_IDS[stage], \"bush\", id, xs[i], ys[i])"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 18: agent_base_png_asset_exists ────────────────────────────
#[test]
fn harness_p14_alpha_a18_agent_base_png_asset_exists() {
    // Type: A — substrate verification: file exists and size > 0.
    let path = project_root()
        .join("assets")
        .join("sprites")
        .join("agent_base.png");
    assert!(
        path.is_file(),
        "A18.1: agent_base.png must exist at {path:?}"
    );
    let meta = fs::metadata(&path)
        .unwrap_or_else(|e| panic!("A18.2: metadata for {path:?}: {e}"));
    assert!(
        meta.len() > 0,
        "A18.3: agent_base.png must be non-empty; got 0 bytes"
    );
    println!("[P14-α A18] agent_base.png exists, {} bytes ✓", meta.len());
}

// ─── Assertion 19: palette_lut_png_asset_exists ───────────────────────────
#[test]
fn harness_p14_alpha_a19_palette_lut_png_asset_exists() {
    // Type: A — substrate verification: file exists and size > 0.
    let path = project_root()
        .join("assets")
        .join("sprites")
        .join("palette_lut.png");
    assert!(
        path.is_file(),
        "A19.1: palette_lut.png must exist at {path:?}"
    );
    let meta = fs::metadata(&path)
        .unwrap_or_else(|e| panic!("A19.2: metadata for {path:?}: {e}"));
    assert!(
        meta.len() > 0,
        "A19.3: palette_lut.png must be non-empty; got 0 bytes"
    );
    println!("[P14-α A19] palette_lut.png exists, {} bytes ✓", meta.len());
}
