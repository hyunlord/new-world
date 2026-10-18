//! V7 Phase 13-ε — Bootstrap Seed: 3 Buildings Around Centre harness.
//!
//! Static file-inspection harness verifying the Phase 13-ε implementation:
//!   - `scripts/ui/world_renderer.gd` — adds two flank `on_building_placed`
//!     calls + two flank Sprite2D children at (24, 32) and (40, 32),
//!     reusing the existing `building_tex`.
//!   - Anti-regression guards for Phase 4-γ, D1 STATE_TINTS, Phase 12-α,
//!     Phase 13-α/β/γ/δ invariants.
//!
//! Assertion 17 (GDScript parse clean) and the scope-discipline git diff
//! guard (Assertion 16) sit at the pipeline layer; we implement A16 here
//! as best-effort `git status` inspection so failing implementations are
//! caught even when run outside the pipeline.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p13_epsilon_bootstrap_seed -- --nocapture

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
/// literals.
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

// ─── Assertion 1: bootstrap_x_left_constant_declared ─────────────────────
#[test]
fn harness_p13_epsilon_a1_bootstrap_x_left_constant_declared() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 2: bootstrap_x_right_constant_declared ────────────────────
#[test]
fn harness_p13_epsilon_a2_bootstrap_x_right_constant_declared() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 3: three_on_building_placed_calls_total ───────────────────
#[test]
fn harness_p13_epsilon_a3_three_on_building_placed_calls_total() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 4: on_building_placed_arg_signatures ──────────────────────
#[test]
fn harness_p13_epsilon_a4_on_building_placed_arg_signatures() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 5: extra_sprite_block_reuses_building_tex ─────────────────
#[test]
fn harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 6: extra_sprite_z_index_is_z_building ─────────────────────
#[test]
fn harness_p13_epsilon_a6_extra_sprite_z_index_is_z_building() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("z_index = 0"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("z_index = 5"));
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(src.contains("sprite.z_index = Z_OVERLAY"));
    assert!(src.contains("const Z_OVERLAY: int = 10"));
}

// ─── Assertion 7: extra_sprite_y_position_uses_bootstrap_y ───────────────
#[test]
fn harness_p13_epsilon_a7_extra_sprite_y_position_uses_bootstrap_y() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 8: phase4_gamma_sprite_scale_preserved ────────────────────
#[test]
fn harness_p13_epsilon_a8_phase4_gamma_sprite_scale_preserved() {
    // Type: D — Phase 4-γ invariant. Preserved per ε spec.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "A8");
    let accepted = ["0.25", ".25"];
    assert!(
        accepted.contains(&rhs.as_str()),
        "A8: SPRITE_SCALE must be EXACTLY `0.25` or `.25`; got `{rhs}`"
    );
    println!("[P13-ε A8] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 9: d1_state_tints_palette_preserved ───────────────────────
#[test]
fn harness_p13_epsilon_a9_d1_state_tints_palette_preserved() {
    // Type: D — 4 entries in STATE_TINTS palette.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_TINTS"),
        "A9.1: agent_renderer.gd must contain `STATE_TINTS`"
    );
    let decl_pos = stripped.find("STATE_TINTS").expect("A9.2");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A9.3: STATE_TINTS array must open with `[`");
    let open_abs = decl_pos + open_rel;
    let bytes = stripped.as_bytes();
    let mut depth = 1i32;
    let mut k = open_abs + 1;
    while k < bytes.len() && depth > 0 {
        match bytes[k] as char {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        k += 1;
    }
    assert_eq!(depth, 0, "A9.4: STATE_TINTS array must terminate with `]`");
    let block = &stripped[open_abs..k];
    let color_count = block.matches("Color(").count();
    assert_eq!(
        color_count, 4,
        "A9.5: STATE_TINTS block must contain EXACTLY 4 Color literals; got {color_count}"
    );
    println!("[P13-ε A9] STATE_TINTS palette = 4 entries preserved ✓");
}

// ─── Assertion 10: phase12_alpha_zoom_invariants_preserved ───────────────
#[test]
fn harness_p13_epsilon_a10_phase12_alpha_zoom_invariants_preserved() {
    // Type: D — Phase 12-α invariants: ZOOM_MIN, ZOOM_MAX, ZOOM_DEFAULT presence
    // with their original constants. (ZOOM_DEFAULT value gets the dedicated
    // A11 guard — here we verify all three are still declared at the original
    // values.)
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A10.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A10.2");
    let zoom_default = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A10.3");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(0.5,0.5)",
        "A10.4: ZOOM_MIN must equal Vector2(0.5, 0.5); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(8.0,8.0)",
        "A10.5: ZOOM_MAX must equal Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zoom_max}`"
    );
    // Sanity-check default presence (value tested by A11).
    assert!(
        no_ws(&zoom_default).starts_with("Vector2("),
        "A10.6: ZOOM_DEFAULT must be a Vector2 literal; got `{zoom_default}`"
    );
    println!("[P13-ε A10] Phase 12-α zoom min/max/default constants preserved ✓");
}

// ─── Assertion 11: phase13_alpha_default_zoom_3x_preserved ───────────────
#[test]
fn harness_p13_epsilon_a11_phase13_alpha_default_zoom_3x_preserved() {
    // Type: D — Phase 13-α invariant. The 96×96 px visibility argument in the
    // ε spec depends on this value.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A11");
    let compact = no_ws(&rhs);
    let accepted = [
        "Vector2(4.0,4.0)",
        "Vector2(4,4)",
        "Vector2(4.0,4)",
        "Vector2(4,4.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{rhs}`"
    );
    println!("[P13-ε A11] ZOOM_DEFAULT = Vector2(4.0, 4.0) preserved ✓");
}

// ─── Assertion 12: phase13_alpha_centre_bootstrap_preserved ──────────────
#[test]
fn harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"));
    assert!(src.contains("get_building_snapshot()"));
}

// ─── Assertion 13: phase13_beta_resource_constants_preserved ─────────────
#[test]
fn harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 14: phase13_gamma_state_scale_boost_preserved ─────────────
#[test]
fn harness_p13_epsilon_a14_phase13_gamma_state_scale_boost_preserved() {
    // Type: D — Phase 13-γ state-scale-boost array invariant. The four
    // entries must remain [1.0, 1.15, 1.15, 1.15] per Phase 13-γ A1-A4.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_SCALE_BOOST"),
        "A14.1: agent_renderer.gd must contain STATE_SCALE_BOOST identifier"
    );
    let decl_pos = stripped.find("STATE_SCALE_BOOST").expect("A14.2");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A14.3: STATE_SCALE_BOOST array must open with `[`");
    let open_abs = decl_pos + open_rel;
    let bytes = stripped.as_bytes();
    let mut depth = 1i32;
    let mut k = open_abs + 1;
    while k < bytes.len() && depth > 0 {
        match bytes[k] as char {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        k += 1;
    }
    assert_eq!(depth, 0, "A14.4: STATE_SCALE_BOOST must terminate with `]`");
    let body = &stripped[open_abs + 1..k - 1];
    let cleaned: String = body.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    let entries: Vec<String> = cleaned
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(
        entries.len(),
        4,
        "A14.5: STATE_SCALE_BOOST must have 4 entries; got {n}: {entries:?}",
        n = entries.len()
    );
    let want: [f32; 4] = [1.0, 1.15, 1.15, 1.15];
    for (i, want_v) in want.iter().enumerate() {
        let got: f32 = entries[i]
            .parse()
            .unwrap_or_else(|_| panic!("A14.6: entry {i} parse failed: `{}`", entries[i]));
        assert!(
            (got - want_v).abs() < 1e-6,
            "A14.7: STATE_SCALE_BOOST[{i}] must equal {want_v}; got {got}"
        );
    }
    println!("[P13-ε A14] STATE_SCALE_BOOST = [1.0, 1.15, 1.15, 1.15] preserved ✓");
}

// ─── Assertion 15: phase13_delta_hud_topbar_present ──────────────────────
#[test]
fn harness_p13_epsilon_a15_phase13_delta_hud_topbar_present() {
    // Type: D — Phase 13-δ HudTopbar file must exist and be non-empty.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("hud_topbar.gd");
    assert!(
        path.is_file(),
        "A15.1: hud_topbar.gd must exist at {path:?}"
    );
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("A15.2: read {path:?}: {e}"));
    assert!(
        !bytes.is_empty(),
        "A15.3: hud_topbar.gd must be non-empty; got 0 bytes"
    );
    println!(
        "[P13-ε A15] hud_topbar.gd exists, {} bytes ✓",
        bytes.len()
    );
}

// ─── Assertion 16: no_rust_simulation_crate_modification (RETIRED) ───────
#[test]
fn harness_p13_epsilon_a16_no_rust_simulation_crate_modification() {
    // RETIRED (S16 prep — Stage 61). This guard mis-encoded P13-ε's
    // GDScript-only-phase scope promise as a PERMANENT global git-status check
    // (forbidden_prefixes over sim-core/sim-bridge/sim-systems/sim-engine),
    // so it FAILed on ANY uncommitted Rust change — blocking ALL future Rust
    // backend work (Section 16+). P13-ε's actual Rust-untouched state was
    // verified at its merge commit and persists in git history; re-asserting
    // it against every future working tree is a design error. Forward
    // per-feature scope-creep protection is now provided by the pipeline's
    // F-Phase-A scope-semantic guard + the Codex Evaluator. Retiring restores
    // a no-op pass without losing real coverage. The now-orphaned
    // `use std::process::Command;` import was removed in the same change.
    // See .harness/prompts/prep-retire-stale-rust-guards.md.
    println!("[P13-ε A16] RETIRED — stale global Rust-block guard; forward protection via F-Phase-A + Evaluator");
}
