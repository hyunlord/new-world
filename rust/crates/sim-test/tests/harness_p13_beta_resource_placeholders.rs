//! V7 Phase 13-β — Resource Node Placeholder Layer harness.
//!
//! Static file-inspection harness verifying the Phase 13-β implementation:
//!   - `scripts/ui/world_renderer.gd` — adds resource placeholder layer with
//!     20 storage_pit sprites scattered deterministically (RESOURCE_SEED).
//!   - Anti-regression guards for Phase 4-γ, D1 STATE_TINTS palette, Phase
//!     12-α/β.1/β.2/γ, and Phase 13-α invariants.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p13_beta_resource_placeholders -- --nocapture

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

// ─── Assertion 1: resource_sprite_path_constant_declared ──────────────────
#[test]
fn harness_p13_beta_a1_resource_sprite_path_constant_declared() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 2: z_resource_constant_equals_3 ────────────────────────────
#[test]
fn harness_p13_beta_a2_z_resource_constant_equals_3() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("z_index = 0"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("z_index = 5"));
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(src.contains("sprite.z_index = Z_OVERLAY"));
    assert!(src.contains("const Z_OVERLAY: int = 10"));
}

// ─── Assertion 3: resource_count_equals_20 ────────────────────────────────
#[test]
fn harness_p13_beta_a3_resource_count_equals_20() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 4: resource_seed_constant_declared ─────────────────────────
#[test]
fn harness_p13_beta_a4_resource_seed_constant_declared() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 5: resource_placement_uses_seeded_rng ──────────────────────
#[test]
fn harness_p13_beta_a5_resource_placement_uses_seeded_rng() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 6: storage_pit_asset_file_exists ───────────────────────────
#[test]
fn harness_p13_beta_a6_storage_pit_asset_file_exists() {
    // Type: A — Invariant: referenced asset must exist (else load() returns null).
    let path = project_root()
        .join("assets")
        .join("sprites")
        .join("furniture")
        .join("storage_pit")
        .join("1.png");
    assert!(
        path.is_file(),
        "A6.1: assets/sprites/furniture/storage_pit/1.png must exist; not found at {path:?}"
    );
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("A6.2: read {path:?}: {e}"));
    assert!(
        !bytes.is_empty(),
        "A6.3: storage_pit/1.png must be non-empty; got 0 bytes"
    );
    assert!(
        bytes.len() >= 8,
        "A6.4: storage_pit/1.png too short for PNG magic; got {} bytes",
        bytes.len()
    );
    let png_magic: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(
        &bytes[..8],
        &png_magic,
        "A6.5: storage_pit/1.png must start with PNG magic; got first 8 bytes = {:02X?}",
        &bytes[..8]
    );
    println!("[P13-β A6] storage_pit/1.png exists, non-empty, PNG magic OK ✓");
}

// ─── Assertion 7: z_order_chain_invariant ─────────────────────────────────
#[test]
fn harness_p13_beta_a7_z_order_chain_invariant() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("z_index = 0"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("z_index = 5"));
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(src.contains("sprite.z_index = Z_OVERLAY"));
    assert!(src.contains("const Z_OVERLAY: int = 10"));
}

// ─── Assertion 8: phase4_gamma_sprite_scale_preserved ─────────────────────
#[test]
fn harness_p13_beta_a8_phase4_gamma_sprite_scale_preserved() {
    // Type: D — Phase 4-γ regression guard.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "A8");
    let accepted = ["0.25", ".25"];
    assert!(
        accepted.contains(&rhs.as_str()),
        "A8: SPRITE_SCALE must be EXACTLY `0.25` or `.25` (Phase 4-γ invariant); got `{rhs}`"
    );
    println!("[P13-β A8] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 9: d1_state_tints_palette_preserved ────────────────────────
#[test]
fn harness_p13_beta_a9_d1_state_tints_palette_preserved() {
    // Type: D — D1 STATE_TINTS palette regression guard.
    // Source-of-truth: harness_p12_alpha_camera_zoom.rs A16.
    let p12_alpha_src = read_file(&[
        "rust",
        "crates",
        "sim-test",
        "tests",
        "harness_p12_alpha_camera_zoom.rs",
    ]);

    let mut golden: Vec<String> = Vec::new();
    for line in p12_alpha_src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("\"Color(") {
            if let Some(end) = rest.find(")\"") {
                let inner = &rest[..end];
                let lit = format!("Color({inner})");
                if !golden.contains(&lit) {
                    golden.push(lit);
                }
            }
        }
    }
    assert_eq!(
        golden.len(),
        4,
        "A9.1: expected 4 D1 STATE_TINTS Color literals from P12-α A16; got {n}: {golden:?}",
        n = golden.len()
    );

    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_TINTS"),
        "A9.2: agent_renderer.gd must contain `STATE_TINTS`"
    );
    let decl_pos = stripped.find("STATE_TINTS").expect("A9.3");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A9.4: STATE_TINTS array must open with `[`");
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
    assert_eq!(depth, 0, "A9.5: STATE_TINTS array must terminate with `]`");
    let block = &stripped[decl_pos..k];

    for needle in golden.iter() {
        assert!(
            block.contains(needle),
            "A9.6: STATE_TINTS block must contain D1 palette entry `{needle}`; block:\n{block}"
        );
    }
    println!("[P13-β A9] all 4 D1 STATE_TINTS literals preserved ✓");
}

// ─── Assertion 10: phase12_alpha_zoom_bounds_preserved ────────────────────
#[test]
fn harness_p13_beta_a10_phase12_alpha_zoom_bounds_preserved() {
    // Type: D — Phase 12-α regression guard.
    let stripped = strip_gd_comments(&read_camera_controller_src());

    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A10.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A10.2");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(0.5,0.5)",
        "A10.3: ZOOM_MIN must equal Vector2(0.5, 0.5); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(8.0,8.0)",
        "A10.4: ZOOM_MAX must equal Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zoom_max}`"
    );
    println!("[P13-β A10] ZOOM_MIN(0.5,0.5) + ZOOM_MAX(8.0,8.0) (G Phase A) ✓");
}

// ─── Assertion 11: phase13_alpha_zoom_default_3x_preserved ────────────────
#[test]
fn harness_p13_beta_a11_phase13_alpha_zoom_default_3x_preserved() {
    // Type: D — Phase 13-α regression guard.
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
    println!("[P13-β A11] ZOOM_DEFAULT = Vector2(4.0, 4.0) preserved ✓");
}

// ─── Assertion 12: phase13_alpha_bootstrap_campfire_preserved ─────────────
#[test]
fn harness_p13_beta_a12_phase13_alpha_bootstrap_campfire_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"));
    assert!(src.contains("get_building_snapshot()"));
}

// ─── Assertion 13: phase12_beta1_terrain_tileset_preserved ────────────────
#[test]
fn harness_p13_beta_a13_phase12_beta1_terrain_tileset_preserved() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("get_terrain_snapshot()"));
    assert!(src.contains("TileSetAtlasSource.new()"));
    assert!(src.contains("_layer.set_cell"));
    assert!(!src.contains("RandomNumberGenerator"));
    assert!(!src.contains("world_terrain.tres"));
}

// ─── Assertion 14: phase12_beta2_construction_z_5_preserved ───────────────
#[test]
fn harness_p13_beta_a14_phase12_beta2_construction_z_5_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"));
    assert!(src.contains("get_construction_snapshot()"));
    assert!(src.contains("sprite.modulate.a = 0.35"));
}

// ─── Assertion 15: phase12_gamma_furniture_z_4_preserved ──────────────────
#[test]
fn harness_p13_beta_a15_phase12_gamma_furniture_z_4_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_settlement_snapshot()"));
    assert!(src.contains("\"formation_xs\""));
    assert!(src.contains("\"formation_ys\""));
    assert!(src.contains("_tile_center(nearest.x, nearest.y)"));
    assert!(src.contains("_reap(_labels, seen)"));
}
