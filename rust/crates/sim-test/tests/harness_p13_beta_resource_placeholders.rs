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

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
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
    // Type: A — Invariant: exact placeholder sprite path required.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "RESOURCE_SPRITE_PATH", "A1");
    assert_eq!(
        rhs, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A1: RESOURCE_SPRITE_PATH RHS must be EXACTLY the storage_pit literal; got `{rhs}`"
    );
    println!("[P13-β A1] RESOURCE_SPRITE_PATH → \"…/storage_pit/1.png\" exactly ✓");
}

// ─── Assertion 2: z_resource_constant_equals_3 ────────────────────────────
#[test]
fn harness_p13_beta_a2_z_resource_constant_equals_3() {
    // Type: A — Invariant: z=3 is only free slot between Z_TERRAIN(0) and Z_FURNITURE(4).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "Z_RESOURCE", "A2");
    assert_eq!(
        rhs, "3",
        "A2: Z_RESOURCE RHS must be EXACTLY `3`; got `{rhs}`"
    );
    println!("[P13-β A2] Z_RESOURCE = 3 ✓");
}

// ─── Assertion 3: resource_count_equals_20 ────────────────────────────────
#[test]
fn harness_p13_beta_a3_resource_count_equals_20() {
    // Type: A — Invariant: feature spec mandates 20 placeholders (exact).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A3");
    assert_eq!(
        rhs, "20",
        "A3: RESOURCE_COUNT RHS must be EXACTLY `20`; got `{rhs}`"
    );
    println!("[P13-β A3] RESOURCE_COUNT = 20 ✓");
}

// ─── Assertion 4: resource_seed_constant_declared ─────────────────────────
#[test]
fn harness_p13_beta_a4_resource_seed_constant_declared() {
    // Type: A — Invariant: deterministic placement requires fixed seed.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A4");
    assert_eq!(
        rhs, "88675123",
        "A4: RESOURCE_SEED RHS must be EXACTLY `88675123`; got `{rhs}`"
    );
    println!("[P13-β A4] RESOURCE_SEED = 88675123 ✓");
}

// ─── Assertion 5: resource_placement_uses_seeded_rng ──────────────────────
#[test]
fn harness_p13_beta_a5_resource_placement_uses_seeded_rng() {
    // Type: A — Invariant: must use RandomNumberGenerator with .seed = RESOURCE_SEED
    // and randi_range(0, GRID_W - 1) (grid-bound pattern).
    let stripped = strip_gd_comments(&read_world_renderer_src());

    assert!(
        stripped.contains("RandomNumberGenerator.new()"),
        "A5.1: resource block must call `RandomNumberGenerator.new()`; not found in stripped source"
    );

    // Look for assignment of RESOURCE_SEED to a `.seed` field (whitespace-tolerant).
    let compact = no_ws(&stripped);
    assert!(
        compact.contains(".seed=RESOURCE_SEED"),
        "A5.2: resource block must contain an assignment `<rng>.seed = RESOURCE_SEED`; \
         not found in whitespace-stripped source"
    );

    // Look for randi_range(0, GRID_W - 1) — accept whitespace tolerance.
    assert!(
        compact.contains("randi_range(0,GRID_W-1)"),
        "A5.3: resource block must contain a `randi_range(0, GRID_W - 1)` call \
         (mirroring grid-bound pattern); not found in whitespace-stripped source"
    );
    println!(
        "[P13-β A5] RNG seeded with RESOURCE_SEED + randi_range(0, GRID_W - 1) present ✓"
    );
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
    // Type: A — Invariant: strict chain Z_TERRAIN(0) < Z_RESOURCE(3) <
    // Z_FURNITURE(4) < Z_CONSTRUCTION(5) < Z_OVERLAY(10).
    let stripped = strip_gd_comments(&read_world_renderer_src());

    let z_terrain: i64 = unique_decl_rhs(&stripped, "Z_TERRAIN", "A7.1")
        .parse()
        .expect("A7.1: Z_TERRAIN must parse as int");
    let z_resource: i64 = unique_decl_rhs(&stripped, "Z_RESOURCE", "A7.2")
        .parse()
        .expect("A7.2: Z_RESOURCE must parse as int");
    let z_furniture: i64 = unique_decl_rhs(&stripped, "Z_FURNITURE", "A7.3")
        .parse()
        .expect("A7.3: Z_FURNITURE must parse as int");
    let z_construction: i64 = unique_decl_rhs(&stripped, "Z_CONSTRUCTION", "A7.4")
        .parse()
        .expect("A7.4: Z_CONSTRUCTION must parse as int");
    let z_overlay: i64 = unique_decl_rhs(&stripped, "Z_OVERLAY", "A7.5")
        .parse()
        .expect("A7.5: Z_OVERLAY must parse as int");

    assert_eq!(z_terrain, 0, "A7.6: Z_TERRAIN must be 0; got {z_terrain}");
    assert_eq!(z_resource, 3, "A7.7: Z_RESOURCE must be 3; got {z_resource}");
    assert_eq!(z_furniture, 4, "A7.8: Z_FURNITURE must be 4; got {z_furniture}");
    assert_eq!(
        z_construction, 5,
        "A7.9: Z_CONSTRUCTION must be 5; got {z_construction}"
    );
    assert_eq!(z_overlay, 10, "A7.10: Z_OVERLAY must be 10; got {z_overlay}");

    assert!(
        z_terrain < z_resource
            && z_resource < z_furniture
            && z_furniture < z_construction
            && z_construction < z_overlay,
        "A7.11: z-order chain violated: terrain({z_terrain}) < resource({z_resource}) < \
         furniture({z_furniture}) < construction({z_construction}) < overlay({z_overlay})"
    );
    println!(
        "[P13-β A7] z-order chain: {z_terrain} < {z_resource} < {z_furniture} < \
         {z_construction} < {z_overlay} ✓"
    );
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
        "Vector2(3.0,3.0)",
        "Vector2(3,3)",
        "Vector2(3.0,3)",
        "Vector2(3,3.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(3.0, 3.0) (Phase 13-α invariant); got `{rhs}`"
    );
    println!("[P13-β A11] ZOOM_DEFAULT = Vector2(3.0, 3.0) preserved ✓");
}

// ─── Assertion 12: phase13_alpha_bootstrap_campfire_preserved ─────────────
#[test]
fn harness_p13_beta_a12_phase13_alpha_bootstrap_campfire_preserved() {
    // Type: D — Phase 13-α bootstrap campfire regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "BUILDING_SPRITE_PATH", "A12");
    assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A12: BUILDING_SPRITE_PATH must remain the campfire literal (Phase 13-α invariant); got `{rhs}`"
    );
    println!("[P13-β A12] BUILDING_SPRITE_PATH → campfire/1.png preserved ✓");
}

// ─── Assertion 13: phase12_beta1_terrain_tileset_preserved ────────────────
#[test]
fn harness_p13_beta_a13_phase12_beta1_terrain_tileset_preserved() {
    // Type: D — Phase 12-β.1 regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());

    let terrain_rhss = find_decl_rhss(&stripped, "TERRAIN_TILESET_PATH");
    assert_eq!(
        terrain_rhss.len(),
        1,
        "A13.1: TERRAIN_TILESET_PATH must be declared exactly once; got {n}: {terrain_rhss:?}",
        n = terrain_rhss.len()
    );

    let alpha_rhs = unique_decl_rhs(&stripped, "OVERLAY_ALPHA", "A13.2");
    let val: f64 = alpha_rhs
        .parse()
        .unwrap_or_else(|_| panic!("A13.3: OVERLAY_ALPHA must parse as float; RHS=`{alpha_rhs}`"));
    assert!(
        (val - 0.65).abs() < 1e-9,
        "A13.4: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    );
    println!("[P13-β A13] TERRAIN_TILESET_PATH + OVERLAY_ALPHA = 0.65 preserved ✓");
}

// ─── Assertion 14: phase12_beta2_construction_z_5_preserved ───────────────
#[test]
fn harness_p13_beta_a14_phase12_beta2_construction_z_5_preserved() {
    // Type: D — Phase 12-β.2 A3 regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "Z_CONSTRUCTION", "A14");
    assert_eq!(
        rhs, "5",
        "A14: Z_CONSTRUCTION must remain `5` (Phase 12-β.2 A3 invariant); got `{rhs}`"
    );
    println!("[P13-β A14] Z_CONSTRUCTION = 5 preserved ✓");
}

// ─── Assertion 15: phase12_gamma_furniture_z_4_preserved ──────────────────
#[test]
fn harness_p13_beta_a15_phase12_gamma_furniture_z_4_preserved() {
    // Type: D — Phase 12-γ regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "Z_FURNITURE", "A15");
    assert_eq!(
        rhs, "4",
        "A15: Z_FURNITURE must remain `4` (Phase 12-γ invariant); got `{rhs}`"
    );
    println!("[P13-β A15] Z_FURNITURE = 4 preserved ✓");
}
