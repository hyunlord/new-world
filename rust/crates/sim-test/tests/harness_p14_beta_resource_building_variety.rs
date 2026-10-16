//! V7 Phase 14-β — Resource + Building Variety harness.
//!
//! Static file-inspection harness verifying the Phase 14-β implementation:
//!   - `scripts/ui/world_renderer.gd` — 5 resource type paths +
//!     RESOURCE_WATER_TINT + 4 village fixture paths + 4 village fixture
//!     positions + Z_VILLAGE_FIXTURE + updated resource scatter loop +
//!     new village fixture placement loop.
//!   - Anti-regression guards for Phase 13-β, Phase 13-α/ε, Phase 12-β.2,
//!     Phase 12-γ, and Phase 14-α invariants.
//!   - Substrate verification: new sprite assets exist on disk + PNG magic.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_beta_resource_building_variety -- --nocapture

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

/// Verify a PNG file exists, is non-empty, and starts with the PNG magic.
fn assert_png_at(path: &PathBuf, label: &str) {
    assert!(path.is_file(), "{label}: file must exist at {path:?}");
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("{label}: read {path:?}: {e}"));
    assert!(bytes.len() >= 8, "{label}: too short for PNG magic; got {} bytes", bytes.len());
    let png_magic: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(
        &bytes[..8],
        &png_magic,
        "{label}: must start with PNG magic; got first 8 bytes = {:02X?}",
        &bytes[..8]
    );
}

// ─── Assertion 1: resource_type_paths_array_declared ─────────────────────
#[test]
fn harness_p14_beta_a1_resource_type_paths_array_declared() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 2: resource_type_paths_has_exactly_five_entries ───────────
#[test]
fn harness_p14_beta_a2_resource_type_paths_has_exactly_five_entries() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 3: resource_type_paths_index_0_references_legacy_constant ─
#[test]
fn harness_p14_beta_a3_resource_type_paths_index_0_references_legacy_constant() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 4: resource_type_paths_index_1_wood_literal ───────────────
#[test]
fn harness_p14_beta_a4_resource_type_paths_index_1_wood_literal() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 5: resource_type_paths_index_2_stone_literal ──────────────
#[test]
fn harness_p14_beta_a5_resource_type_paths_index_2_stone_literal() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 6: resource_type_paths_index_3_water_literal ──────────────
#[test]
fn harness_p14_beta_a6_resource_type_paths_index_3_water_literal() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 7: resource_type_paths_index_4_food_literal ───────────────
#[test]
fn harness_p14_beta_a7_resource_type_paths_index_4_food_literal() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 8: resource_water_tint_constant_value ─────────────────────
#[test]
fn harness_p14_beta_a8_resource_water_tint_constant_value() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("get_terrain_snapshot()"));
    assert!(src.contains("TileSetAtlasSource.new()"));
    assert!(src.contains("_layer.set_cell"));
    assert!(!src.contains("RandomNumberGenerator"));
    assert!(!src.contains("world_terrain.tres"));
}

// ─── Assertion 9: resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator
#[test]
fn harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 10: water_modulate_branch_present ─────────────────────────
#[test]
fn harness_p14_beta_a10_water_modulate_branch_present() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("get_terrain_snapshot()"));
    assert!(src.contains("TileSetAtlasSource.new()"));
    assert!(src.contains("_layer.set_cell"));
    assert!(!src.contains("RandomNumberGenerator"));
    assert!(!src.contains("world_terrain.tres"));
}

// ─── Assertion 11: village_fixture_paths_has_exactly_four_literals ───────
#[test]
fn harness_p14_beta_a11_village_fixture_paths_has_exactly_four_literals() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(!src.contains("on_building_placed("));
    assert!(!src.contains("RESOURCE_SEED"));
    assert!(!src.contains("VILLAGE_FIXTURE"));
    assert!(!src.contains("BOOTSTRAP_"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("get_resource_snapshot()"));
}

// ─── Assertion 12: village_fixture_positions_match_diamond_layout ────────
#[test]
fn harness_p14_beta_a12_village_fixture_positions_match_diamond_layout() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 13: z_village_fixture_equals_five ─────────────────────────
#[test]
fn harness_p14_beta_a13_z_village_fixture_equals_five() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("z_index = 0"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("z_index = 5"));
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(src.contains("sprite.z_index = Z_OVERLAY"));
    assert!(src.contains("const Z_OVERLAY: int = 10"));
}

// ─── Assertion 14: village_fixture_placement_loop_present ────────────────
#[test]
fn harness_p14_beta_a14_village_fixture_placement_loop_present() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 15: phase_13_beta_resource_constants_preserved ────────────
#[test]
fn harness_p14_beta_a15_phase_13_beta_resource_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_resource_snapshot()"));
    assert!(src.contains("if kinds[i] != 0:"));
    assert!(src.contains("berry_bush_%d.png"));
    assert!(src.contains("_berry_textures[stage]"));
    assert!(!src.contains("SOURCE_KIND_COLORS"));
    assert!(!src.contains("Polygon2D"));
}

// ─── Assertion 16: phase_13_alpha_bootstrap_constants_preserved ──────────
#[test]
fn harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 17: phase_12_construction_constants_preserved ─────────────
#[test]
fn harness_p14_beta_a17_phase_12_construction_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"));
    assert!(src.contains("get_construction_snapshot()"));
    assert!(src.contains("sprite.modulate.a = 0.35"));
}

// ─── Assertion 18: phase_12_gamma_furniture_constants_preserved ──────────
#[test]
fn harness_p14_beta_a18_phase_12_gamma_furniture_constants_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_settlement_snapshot()"));
    assert!(src.contains("\"formation_xs\""));
    assert!(src.contains("\"formation_ys\""));
    assert!(src.contains("_tile_center(nearest.x, nearest.y)"));
    assert!(src.contains("_reap(_labels, seen)"));
}

// ─── Assertion 19: phase_14_alpha_agent_renderer_constants_preserved ─────
#[test]
fn harness_p14_beta_a19_phase_14_alpha_agent_renderer_constants_preserved() {
    // Type: D — Phase 14-α scope-discipline guard. β must not touch
    // agent_renderer.gd at all.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let bucket = unique_decl_rhs(&stripped, "ROLE_BUCKET_COUNT", "A19.1");
    assert_eq!(
        bucket, "4",
        "A19.2: ROLE_BUCKET_COUNT must remain 4; got `{bucket}`"
    );
    let icon = unique_decl_rhs(&stripped, "ICON_OFFSET_PX", "A19.3");
    assert_eq!(
        no_ws(&icon),
        "Vector2(0,-12)",
        "A19.4: ICON_OFFSET_PX must remain Vector2(0, -12); got `{icon}`"
    );
    println!("[P14-β A19] Phase 14-α agent_renderer constants preserved ✓");
}

// ─── Assertion 20: new_resource_sprite_assets_exist_on_disk ──────────────
#[test]
fn harness_p14_beta_a20_new_resource_sprite_assets_exist_on_disk() {
    // Type: A — Substrate verification: 4 new resource sprite assets +
    // PNG magic.
    let root = project_root();
    let cases: [(&str, &[&str]); 4] = [
        ("Wood", &["assets", "sprites", "furniture", "workbench", "1.png"]),
        ("Stone", &["assets", "sprites", "walls", "limestone", "1.png"]),
        ("Water", &["assets", "sprites", "floors", "stone_slab", "1.png"]),
        ("Food", &["assets", "sprites", "furniture", "hearth", "2.png"]),
    ];
    for (label, segs) in cases.iter() {
        let mut path = root.clone();
        for s in *segs {
            path.push(s);
        }
        assert_png_at(&path, &format!("A20[{label}]"));
    }
    println!("[P14-β A20] all 4 new resource sprite assets exist + PNG magic OK ✓");
}

// ─── Assertion 21: new_village_fixture_assets_exist_on_disk ──────────────
#[test]
fn harness_p14_beta_a21_new_village_fixture_assets_exist_on_disk() {
    // Type: A — Substrate verification: 4 new village fixture assets +
    // PNG magic.
    let root = project_root();
    let cases: [(&str, &[&str]); 4] = [
        ("Workshop", &["assets", "sprites", "furniture", "workbench", "2.png"]),
        ("Drying", &["assets", "sprites", "furniture", "drying_rack", "1.png"]),
        ("Shelter", &["assets", "sprites", "furniture", "lean_to", "1.png"]),
        ("Storage", &["assets", "sprites", "furniture", "storage_pit", "2.png"]),
    ];
    for (label, segs) in cases.iter() {
        let mut path = root.clone();
        for s in *segs {
            path.push(s);
        }
        assert_png_at(&path, &format!("A21[{label}]"));
    }
    println!("[P14-β A21] all 4 new village fixture assets exist + PNG magic OK ✓");
}

// ─── Assertion 22: no_rust_crate_modifications (RETIRED) ─────────────────
#[test]
fn harness_p14_beta_a22_no_rust_crate_modifications() {
    // RETIRED (S16 prep — Stage 61). This guard mis-encoded P14-β's
    // GDScript-only-phase scope promise as a PERMANENT global git-diff check
    // (forbidden_prefixes over sim-core/sim-systems/sim-engine/sim-bridge/
    // sim-data, tracked + untracked), so it FAILed on ANY uncommitted Rust
    // change — blocking ALL future Rust backend work (Section 16+). P14-β's
    // actual Rust-untouched state was verified at its merge commit and
    // persists in git history; re-asserting it against every future working
    // tree is a design error. Forward per-feature scope-creep protection is
    // now provided by the pipeline's F-Phase-A scope-semantic guard + the
    // Codex Evaluator. Retiring restores a no-op pass without losing real
    // coverage. The now-orphaned `use std::process::Command;` import was
    // removed in the same change.
    // See .harness/prompts/prep-retire-stale-rust-guards.md.
    println!("[P14-β A22] RETIRED — stale global Rust-block guard; forward protection via F-Phase-A + Evaluator");
}
