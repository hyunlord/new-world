//! V7 Phase 12-β — TileMapLayer Floor Terrain + Bootstrap Building Sprite harness.
//!
//! Static file-inspection harness verifying the Phase 12-β implementation:
//!   - `assets/tilesets/world_terrain.tres` — Godot TileSet resource with
//!     9 floor tile atlas sources (3 materials × 3 variants).
//!   - `scripts/ui/world_renderer.gd` (modified) — loads TileSet, creates
//!     TileMapLayer, populates 64×64 grid deterministically, instantiates
//!     bootstrap building sprite, sets z-order, applies overlay alpha.
//!   - `scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd` —
//!     runtime harness that validates the scene tree, writes pipeline
//!     artefacts, and exits non-zero on assertion failure.
//!   - Regression guards for Phase 4-γ, Phase 11-α + D1, Phase 12-α invariants.
//!   - Scope guard against Rust crate modifications.
//!
//! Run: `cargo test -p sim-test --test harness_p12_beta_terrain -- --nocapture`
//!
//! Plan locked thresholds: see `.harness/plans/p12-beta-terrain-and-buildings/plan_final.md`.

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

fn read_file(rel: &str) -> String {
    let path = project_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Strip GDScript line comments (`# …` to EOL). Preserves `#` inside string
/// literals (single or double quoted).
fn strip_gd_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut in_str: Option<char> = None;
        let mut keep_end = line.len();
        let bytes = line.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let c = b as char;
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

const TILESET_PATH: &str = "assets/tilesets/world_terrain.tres";
const WORLD_RENDERER_PATH: &str = "scripts/ui/world_renderer.gd";
const RUNTIME_HARNESS_PATH: &str =
    "scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd";
const AGENT_RENDERER_PATH: &str = "scripts/ui/agent_renderer.gd";
const MAIN_TSCN_PATH: &str = "scenes/main.tscn";

// ─── Assertion 1: tileset_resource_exists_and_nonempty ───────────────────
#[test]
fn harness_p12_beta_a1_tileset_resource_exists_and_nonempty() {
    // Type: A — file MUST exist with > 256 bytes (rejects trivially empty stubs).
    let path = project_root().join(TILESET_PATH);
    assert!(path.exists(), "A1: {TILESET_PATH} must exist at {path:?}");
    let meta = fs::metadata(&path).expect("file metadata");
    let len = meta.len();
    assert!(
        len > 256,
        "A1: {TILESET_PATH} must be > 256 bytes; got {len}"
    );
    println!("[P12-β A1] {TILESET_PATH} exists, {len} bytes ✓");
}

// ─── Assertion 2: tileset_tile_size_is_16 ─────────────────────────────────
#[test]
fn harness_p12_beta_a2_tileset_tile_size_is_16() {
    // Type: A — exactly one `tile_size = Vector2i(16, 16)` declaration.
    let src = read_file(TILESET_PATH);
    let count = src.matches("tile_size = Vector2i(16, 16)").count();
    assert_eq!(
        count, 1,
        "A2: must have exactly 1 `tile_size = Vector2i(16, 16)` declaration; found {count}"
    );
    println!("[P12-β A2] tile_size = Vector2i(16, 16) (exactly 1) ✓");
}

// ─── Assertion 3: tileset_has_nine_to_twelve_atlas_sources ───────────────
#[test]
fn harness_p12_beta_a3_tileset_has_nine_atlas_sources() {
    // Type: A — count of `[sub_resource type="TileSetAtlasSource"` in [9, 12].
    let src = read_file(TILESET_PATH);
    let count = src.matches("[sub_resource type=\"TileSetAtlasSource\"").count();
    assert!(
        (9..=12).contains(&count),
        "A3: must have 9..=12 TileSetAtlasSource sub_resources; found {count}"
    );
    println!("[P12-β A3] TileSetAtlasSource count = {count} (in [9,12]) ✓");
}

// ─── Assertion 4: tileset_references_all_three_floor_materials ───────────
#[test]
fn harness_p12_beta_a4_tileset_references_all_three_floor_materials() {
    // Type: A — all three material directory substrings present.
    let src = read_file(TILESET_PATH);
    for needle in ["floors/packed_earth/", "floors/stone_slab/", "floors/wood_plank/"] {
        assert!(
            src.contains(needle),
            "A4: tileset must reference `{needle}` — missing"
        );
    }
    println!("[P12-β A4] all 3 floor material paths present ✓");
}

// ─── Assertion 5: tileset_references_three_variants_per_material ─────────
#[test]
fn harness_p12_beta_a5_tileset_references_three_variants_per_material() {
    // Type: A — each of 9 distinct `<material>/<n>.png` substrings present.
    let src = read_file(TILESET_PATH);
    let materials = ["packed_earth", "stone_slab", "wood_plank"];
    for mat in materials.iter() {
        for n in 1..=3 {
            let needle = format!("{mat}/{n}.png");
            assert!(
                src.contains(&needle),
                "A5: tileset must reference variant `{needle}` — missing"
            );
        }
    }
    println!("[P12-β A5] all 9 variant filenames present ✓");
}

// ─── Assertion 6: world_renderer_loads_terrain_tileset ───────────────────
#[test]
fn harness_p12_beta_a6_world_renderer_loads_terrain_tileset() {
    // Type: A — constant declared pointing at the .tres AND a load(constant)
    // call appears in non-comment code.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Find a const line that mentions the tileset .tres path.
    let const_line = stripped
        .lines()
        .find(|l| {
            l.contains("assets/tilesets/world_terrain.tres") && l.trim_start().starts_with("const ")
        })
        .unwrap_or_else(|| {
            panic!(
                "A6.1: world_renderer.gd must declare a `const` referencing \
                 `assets/tilesets/world_terrain.tres`"
            )
        });

    // Extract the constant name (between `const ` and ` := ` / ` =` / `:`).
    let after_const = const_line.trim_start().trim_start_matches("const ").trim_start();
    let name_end = after_const
        .find([' ', ':', '='])
        .unwrap_or(after_const.len());
    let const_name = &after_const[..name_end];
    assert!(
        !const_name.is_empty(),
        "A6.2: could not parse constant name from line `{const_line}`"
    );

    // Verify a `load(<const_name>)` call appears in non-comment code.
    let load_pattern = format!("load({const_name})");
    assert!(
        stripped.contains(&load_pattern),
        "A6.3: world_renderer.gd must contain `{load_pattern}` in non-comment \
         code; constant `{const_name}` was declared but never loaded"
    );
    println!("[P12-β A6] const {const_name} → load({const_name}) ✓");
}

// ─── Assertion 7: world_renderer_instantiates_tilemaplayer_with_tileset ──
#[test]
fn harness_p12_beta_a7_world_renderer_instantiates_tilemaplayer_with_tileset() {
    // Type: A — TileMapLayer.new() + assignment of .tile_set + add_child on
    // the same variable.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Find the line creating TileMapLayer.new() and extract the variable name.
    let new_line = stripped
        .lines()
        .find(|l| l.contains("TileMapLayer.new()"))
        .unwrap_or_else(|| {
            panic!("A7.1: world_renderer.gd must contain `TileMapLayer.new()`")
        });

    // Parse `var <name> := TileMapLayer.new()` or `<name> = TileMapLayer.new()`.
    let trimmed = new_line.trim_start();
    let after_var = trimmed.trim_start_matches("var ").trim_start();
    let name_end = after_var
        .find([' ', ':', '='])
        .unwrap_or(after_var.len());
    let var_name = &after_var[..name_end];
    assert!(
        !var_name.is_empty(),
        "A7.2: could not parse TileMapLayer variable name from `{new_line}`"
    );

    let tile_set_assign = format!("{var_name}.tile_set =");
    assert!(
        stripped.contains(&tile_set_assign),
        "A7.3: must assign `{var_name}.tile_set = ...`; missing"
    );

    let add_child_call = format!("add_child({var_name})");
    assert!(
        stripped.contains(&add_child_call),
        "A7.4: must call `add_child({var_name})`; missing"
    );
    println!("[P12-β A7] TileMapLayer({var_name}) created, tile_set assigned, add_child'd ✓");
}

// ─── Assertion 8: world_renderer_populates_all_grid_cells ────────────────
#[test]
fn harness_p12_beta_a8_world_renderer_populates_all_grid_cells() {
    // Type: A — nested loop over GRID_W and GRID_H containing a set_cell(.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Locate any `for ... GRID_W` line and within its scope find a nested
    // `for ... GRID_H` whose body contains `set_cell(`. We approximate by
    // searching from the GRID_W `for` line to the end of file for both
    // GRID_H reference (inside a for loop) and set_cell(.
    let lines: Vec<&str> = stripped.lines().collect();
    let mut found = false;
    for (i, line) in lines.iter().enumerate() {
        let is_outer_for = line.trim_start().starts_with("for ") && line.contains("GRID_W");
        if !is_outer_for {
            continue;
        }
        // Inspect the next ~40 lines for a `for ... GRID_H` AND a `set_cell(`.
        let hi = (i + 40).min(lines.len());
        let window = lines[i..hi].join("\n");
        let has_inner = window.lines().any(|l| {
            l.trim_start().starts_with("for ") && l.contains("GRID_H")
        });
        let has_set_cell = window.contains("set_cell(");
        if has_inner && has_set_cell {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "A8: world_renderer.gd must contain a nested loop over GRID_W and \
         GRID_H whose body calls `set_cell(`"
    );
    println!("[P12-β A8] nested GRID_W × GRID_H loop with set_cell( ✓");
}

// ─── Assertion 9: terrain_placement_is_deterministic ─────────────────────
#[test]
fn harness_p12_beta_a9_terrain_placement_is_deterministic() {
    // Type: A — a numeric seed constant declared AND assigned to
    // RandomNumberGenerator instance's seed field BEFORE the set_cell loop.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Find a const declaration whose value is a numeric integer literal and
    // whose name suggests a seed (contains "SEED").
    let seed_const_line = stripped
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("const ") && t.contains("SEED")
        })
        .unwrap_or_else(|| {
            panic!("A9.1: world_renderer.gd must declare a `const ...SEED... = <int>`")
        });
    let after_const = seed_const_line.trim_start().trim_start_matches("const ").trim_start();
    let name_end = after_const
        .find([' ', ':', '='])
        .unwrap_or(after_const.len());
    let seed_name = &after_const[..name_end];
    assert!(
        !seed_name.is_empty(),
        "A9.2: could not parse seed constant name from `{seed_const_line}`"
    );

    // Verify `<rng_var>.seed = <seed_name>` (or equivalent assignment).
    let seed_assign_needle = format!(".seed = {seed_name}");
    assert!(
        stripped.contains(&seed_assign_needle),
        "A9.3: must assign `<rng>.seed = {seed_name}`; missing"
    );

    // Verify the seed assignment occurs in source order BEFORE the set_cell call.
    let assign_idx = stripped
        .find(&seed_assign_needle)
        .expect("A9.4: seed assign location");
    let set_cell_idx = stripped
        .find("set_cell(")
        .expect("A9.5: set_cell( location");
    assert!(
        assign_idx < set_cell_idx,
        "A9.6: seed assignment must come before set_cell loop \
         (assign_idx={assign_idx}, set_cell_idx={set_cell_idx})"
    );
    println!("[P12-β A9] const {seed_name} → rng.seed assigned before set_cell loop ✓");
}

// ─── Assertion 10: bootstrap_building_sprite_loaded ──────────────────────
#[test]
fn harness_p12_beta_a10_bootstrap_building_sprite_loaded() {
    // Type: A — constant pointing at `assets/sprites/buildings/cairn/1.png`,
    // load(constant), assignment to a Sprite2D.texture, and add_child.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Find the const that references the cairn sprite path.
    let const_line = stripped
        .lines()
        .find(|l| {
            l.trim_start().starts_with("const ")
                && l.contains("assets/sprites/buildings/cairn/1.png")
        })
        .unwrap_or_else(|| {
            panic!(
                "A10.1: world_renderer.gd must declare a `const` referencing \
                 `assets/sprites/buildings/cairn/1.png`"
            )
        });
    let after_const = const_line.trim_start().trim_start_matches("const ").trim_start();
    let name_end = after_const
        .find([' ', ':', '='])
        .unwrap_or(after_const.len());
    let bld_const = &after_const[..name_end];

    let load_call = format!("load({bld_const})");
    assert!(
        stripped.contains(&load_call),
        "A10.2: must call `{load_call}` in non-comment code"
    );

    // Find the Sprite2D creation line. Look for `<var> := Sprite2D.new()` or
    // `<var> = Sprite2D.new()`. We pick the line that follows the bootstrap
    // building loader area — generally there is more than one Sprite2D in
    // this file (overlay is also Sprite2D), so we need the one whose
    // .texture is assigned the loaded building texture.
    // Approach: locate every `<var>.texture = ` line in proximity to the
    // building texture load call and verify at least one .texture =
    // building texture variable exists AND that var has add_child.
    // Simpler approach: find any line `<var>.texture = building_tex` where
    // `building_tex` (or similar) appears post-load assignment.

    // Locate the line containing the load() expression and parse the
    // left-hand variable name from the START of that line.
    let load_line = stripped
        .lines()
        .find(|l| l.contains(&load_call))
        .unwrap_or_else(|| panic!("A10.3: line containing `{load_call}` must be present"));
    let trimmed = load_line.trim_start();
    let after_var = trimmed.trim_start_matches("var ").trim_start();
    let tex_name_end = after_var
        .find([' ', ':', '='])
        .unwrap_or(after_var.len());
    let tex_var = after_var[..tex_name_end].trim();
    assert!(
        !tex_var.is_empty(),
        "A10.4: could not parse texture variable name from `{load_line}`"
    );

    // Find a `<sprite_var>.texture = <tex_var>` assignment in the file.
    let texture_assign_pat = format!(".texture = {tex_var}");
    let assign_pos = stripped
        .find(&texture_assign_pat)
        .unwrap_or_else(|| panic!("A10.5: must assign `<sprite>.texture = {tex_var}`; missing"));

    // Parse the sprite variable name from the same line.
    let assign_line = stripped[..assign_pos + texture_assign_pat.len()]
        .lines()
        .last()
        .expect("A10.6: assign line");
    let sprite_var = assign_line
        .trim_start()
        .split('.')
        .next()
        .unwrap_or("")
        .trim();
    assert!(
        !sprite_var.is_empty(),
        "A10.7: could not parse sprite variable name from `{assign_line}`"
    );

    // Verify Sprite2D.new() exists somewhere AND add_child(<sprite_var>).
    assert!(
        stripped.contains("Sprite2D.new()"),
        "A10.8: must contain a `Sprite2D.new()` for the bootstrap building"
    );

    let add_child_call = format!("add_child({sprite_var})");
    assert!(
        stripped.contains(&add_child_call),
        "A10.9: must call `add_child({sprite_var})`; missing"
    );
    println!(
        "[P12-β A10] const {bld_const} → load → Sprite2D.texture → add_child({sprite_var}) ✓"
    );
}

// ─── Assertion 11: bootstrap_sprite_positioned_at_bootstrap_tile ─────────
#[test]
fn harness_p12_beta_a11_bootstrap_sprite_positioned_at_bootstrap_tile() {
    // Type: A — position assignment references BOOTSTRAP_X, BOOTSTRAP_Y,
    // SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y, AND TILE_SIZE.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Find lines containing `.position =` AND the bootstrap identifiers.
    // The expression may span multiple lines (Vector2(...) call). We scan for
    // the `.position =` line and concatenate up to the matching closing paren
    // depth-zero.
    let position_idx = stripped
        .find(".position = ")
        .or_else(|| stripped.find(".position =\n"))
        .unwrap_or_else(|| panic!("A11.1: must contain a `.position =` assignment"));

    // Conservatively, grab a 400-char window from .position = forward.
    let window_end = (position_idx + 800).min(stripped.len());
    // Find the next .position = after this one to limit the window, but for
    // the first match the window is fine.
    let window = &stripped[position_idx..window_end];

    // Find the closing paren matching the first Vector2( within the window.
    // We just scan the window for all five identifiers — the Godot Vector2
    // expression is a single statement so consecutive identifiers in
    // proximity is sufficient.
    let required = [
        "BOOTSTRAP_X",
        "BOOTSTRAP_Y",
        "SPRITE_ORIGIN_X",
        "SPRITE_ORIGIN_Y",
        "TILE_SIZE",
    ];

    // Try multiple `.position = ` occurrences if needed.
    let mut found_all = false;
    let mut search_from = 0usize;
    while let Some(rel) = stripped[search_from..].find(".position = ") {
        let abs = search_from + rel;
        let end = (abs + 800).min(stripped.len());
        let w = &stripped[abs..end];
        // Stop at the next .position = if any.
        let stop = w[12..]
            .find(".position = ")
            .map(|i| i + 12)
            .unwrap_or(w.len());
        let scoped = &w[..stop];
        if required.iter().all(|r| scoped.contains(r)) {
            found_all = true;
            break;
        }
        search_from = abs + 12;
    }
    let _ = window; // explicit no-warn
    assert!(
        found_all,
        "A11.2: a `.position = ` assignment must reference ALL of \
         BOOTSTRAP_X, BOOTSTRAP_Y, SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y, TILE_SIZE \
         within a single expression"
    );
    println!("[P12-β A11] bootstrap sprite .position uses 5 grid constants ✓");
}

// ─── Assertion 12: z_order_constants_match_spec ──────────────────────────
#[test]
fn harness_p12_beta_a12_z_order_constants_match_spec() {
    // Type: A — Z_TERRAIN = 0, Z_BUILDING = 5, Z_OVERLAY = 10.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    fn const_value_line<'a>(stripped: &'a str, name: &str) -> &'a str {
        stripped
            .lines()
            .find(|l| {
                let t = l.trim_start();
                t.starts_with("const ") && t.contains(name)
            })
            .unwrap_or_else(|| panic!("A12: const {name} not declared in world_renderer.gd"))
    }

    fn line_has_int_value(line: &str, target: i64) -> bool {
        // Extract substring after '=' or ':=' and check that the parsed
        // integer matches.
        let after_eq = match line.rfind('=') {
            Some(i) => &line[i + 1..],
            None => return false,
        };
        let trimmed = after_eq.trim();
        // Allow trailing comment-stripped whitespace.
        trimmed.parse::<i64>().ok() == Some(target)
    }

    let z_terrain_line = const_value_line(&stripped, "Z_TERRAIN");
    assert!(
        line_has_int_value(z_terrain_line, 0),
        "A12.1: Z_TERRAIN must equal exactly 0. Line: `{z_terrain_line}`"
    );

    let z_building_line = const_value_line(&stripped, "Z_BUILDING");
    assert!(
        line_has_int_value(z_building_line, 5),
        "A12.2: Z_BUILDING must equal exactly 5. Line: `{z_building_line}`"
    );

    let z_overlay_line = const_value_line(&stripped, "Z_OVERLAY");
    assert!(
        line_has_int_value(z_overlay_line, 10),
        "A12.3: Z_OVERLAY must equal exactly 10. Line: `{z_overlay_line}`"
    );
    println!("[P12-β A12] Z_TERRAIN=0, Z_BUILDING=5, Z_OVERLAY=10 ✓");
}

// ─── Assertion 13: z_order_constants_are_assigned_to_nodes ───────────────
#[test]
fn harness_p12_beta_a13_z_order_constants_are_assigned_to_nodes() {
    // Type: A — `.z_index = Z_TERRAIN`, `.z_index = Z_BUILDING`,
    // `.z_index = Z_OVERLAY` each appear in non-comment code.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    for needle in [".z_index = Z_TERRAIN", ".z_index = Z_BUILDING", ".z_index = Z_OVERLAY"] {
        assert!(
            stripped.contains(needle),
            "A13: must contain `{needle}` assignment; missing"
        );
    }
    println!("[P12-β A13] all 3 z_index assignments present ✓");
}

// ─── Assertion 14: overlay_alpha_in_valid_translucent_range ──────────────
#[test]
fn harness_p12_beta_a14_overlay_alpha_in_valid_translucent_range() {
    // Type: A — OVERLAY_ALPHA in open range (0.30, 0.90), referenced inside
    // a Color(...) expression assigned to <overlay_sprite>.modulate.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    // Find const OVERLAY_ALPHA = <float>.
    let alpha_line = stripped
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("const ") && t.contains("OVERLAY_ALPHA")
        })
        .unwrap_or_else(|| panic!("A14.1: const OVERLAY_ALPHA not declared"));
    let after_eq = alpha_line
        .rfind('=')
        .map(|i| &alpha_line[i + 1..])
        .unwrap_or("");
    let val: f64 = after_eq
        .trim()
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("A14.2: OVERLAY_ALPHA must parse as float; line=`{alpha_line}`"));
    assert!(
        val > 0.30 && val < 0.90,
        "A14.3: OVERLAY_ALPHA must be in open range (0.30, 0.90); got {val}"
    );

    // Verify OVERLAY_ALPHA appears inside a Color(...) expression on a
    // .modulate assignment line. We search for `.modulate = Color(`
    // occurrences and check each one for OVERLAY_ALPHA in its window.
    let mut found = false;
    let mut search_from = 0usize;
    while let Some(rel) = stripped[search_from..].find(".modulate = Color(") {
        let abs = search_from + rel;
        let end = (abs + 200).min(stripped.len());
        // Grab from `.modulate = Color(` until the closing `)` (best effort).
        let window = &stripped[abs..end];
        let close = window.find(')').map(|i| i + 1).unwrap_or(window.len());
        let scoped = &window[..close];
        if scoped.contains("OVERLAY_ALPHA") {
            found = true;
            break;
        }
        search_from = abs + 1;
    }
    assert!(
        found,
        "A14.4: OVERLAY_ALPHA must be referenced inside a \
         `<overlay>.modulate = Color(...)` expression; not found"
    );
    println!("[P12-β A14] OVERLAY_ALPHA = {val} ∈ (0.30, 0.90), used in .modulate Color() ✓");
}

// ─── Assertion 15: runtime_harness_script_present_and_extends_scenetree ──
#[test]
fn harness_p12_beta_a15_runtime_harness_script_present_and_extends_scenetree() {
    // Type: A — file exists AND non-comment content contains `extends SceneTree`.
    let path = project_root().join(RUNTIME_HARNESS_PATH);
    assert!(
        path.exists(),
        "A15.1: runtime harness must exist at {path:?}"
    );
    let src = read_file(RUNTIME_HARNESS_PATH);
    let stripped = strip_gd_comments(&src);
    assert!(
        stripped.contains("extends SceneTree"),
        "A15.2: runtime harness must contain `extends SceneTree` in non-comment code"
    );
    println!("[P12-β A15] runtime harness file present, extends SceneTree ✓");
}

// ─── Assertion 16: runtime_harness_emits_pipeline_artefacts ──────────────
#[test]
fn harness_p12_beta_a16_runtime_harness_emits_pipeline_artefacts() {
    // Type: A — all three artefact filenames appear in file-write expressions.
    let src = read_file(RUNTIME_HARNESS_PATH);
    let stripped = strip_gd_comments(&src);

    for needle in ["interactive_results.txt", "assertion_log.txt", "console_log.txt"] {
        assert!(
            stripped.contains(needle),
            "A16: runtime harness must reference `{needle}` for pipeline artefacts"
        );
    }
    println!("[P12-β A16] all 3 pipeline artefact filenames referenced ✓");
}

// ─── Assertion 17: runtime_harness_exit_code_nonzero_on_failure ──────────
#[test]
fn harness_p12_beta_a17_runtime_harness_exit_code_nonzero_on_failure() {
    // Type: D — at least one `quit(<nonzero>)` call must exist (per C-1 lesson).
    let src = read_file(RUNTIME_HARNESS_PATH);
    let stripped = strip_gd_comments(&src);

    // Look for `quit(N)` where N is a nonzero integer literal.
    // Pattern matches quit(1) quit(2) … quit(99). Reject quit(0) only.
    let mut found_nonzero = false;
    let mut search_from = 0usize;
    while let Some(rel) = stripped[search_from..].find("quit(") {
        let abs = search_from + rel + "quit(".len();
        let close = stripped[abs..]
            .find(')')
            .map(|i| abs + i)
            .unwrap_or(stripped.len());
        let arg = stripped[abs..close].trim();
        // Try parsing as int.
        if let Ok(v) = arg.parse::<i64>() {
            if v != 0 {
                found_nonzero = true;
                break;
            }
        } else if !arg.is_empty() && arg != "0" {
            // Variable name → conservative: accept as potentially nonzero only
            // if the function param is observably set to a nonzero value.
            // To keep this assertion strict, we require a LITERAL nonzero arg.
        }
        search_from = abs;
    }
    assert!(
        found_nonzero,
        "A17: runtime harness must include at least one `quit(<nonzero>)` \
         call for hard-fail exit (C-1 lesson). Did you accidentally only \
         use quit(0)?"
    );
    println!("[P12-β A17] runtime harness has quit(<nonzero>) hard-fail path ✓");
}

// ─── Assertion 18: runtime_harness_verifies_tilemaplayer_present_and_loaded
#[test]
fn harness_p12_beta_a18_runtime_harness_verifies_tilemaplayer_present_and_loaded() {
    // Type: A — locates TileMapLayer child AND asserts its tile_set is non-null.
    let src = read_file(RUNTIME_HARNESS_PATH);
    let stripped = strip_gd_comments(&src);

    // Look for TileMapLayer reference (either as type cast `as TileMapLayer`,
    // class name match, or get_children iteration mentioning it).
    assert!(
        stripped.contains("TileMapLayer"),
        "A18.1: runtime harness must reference TileMapLayer class to locate \
         the terrain layer node"
    );

    // Look for a tile_set non-null check.
    let has_tile_set_check = stripped.contains("tile_set")
        && (stripped.contains("tile_set != null")
            || stripped.contains("tile_set == null")
            || stripped.contains("tile_set"));
    assert!(
        has_tile_set_check,
        "A18.2: runtime harness must inspect the layer's `tile_set` field \
         (non-null check)"
    );
    // Stronger check: tile_set explicitly compared to null AND a record/quit
    // path triggered on failure.
    let stricter = stripped.contains("tile_set != null")
        || stripped.contains("tile_set == null");
    assert!(
        stricter,
        "A18.3: runtime harness must explicitly check `tile_set != null` or \
         `tile_set == null` to make the assertion observable"
    );
    println!("[P12-β A18] runtime harness checks TileMapLayer + tile_set != null ✓");
}

// ─── Assertion 19: runtime_harness_verifies_bootstrap_building_sprite_distinct
#[test]
fn harness_p12_beta_a19_runtime_harness_verifies_bootstrap_building_sprite_distinct() {
    // Type: A — locates a Sprite2D whose texture resource path contains
    // `buildings/cairn/`, distinguishing from the overlay sprite.
    let src = read_file(RUNTIME_HARNESS_PATH);
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("buildings/cairn/"),
        "A19.1: runtime harness must reference the substring \
         `buildings/cairn/` to distinguish bootstrap building from overlay"
    );
    // Should also be probing a Sprite2D child.
    assert!(
        stripped.contains("Sprite2D"),
        "A19.2: runtime harness must reference Sprite2D class to locate \
         the building sprite child"
    );
    println!("[P12-β A19] runtime harness locates Sprite2D with `buildings/cairn/` path ✓");
}

// ─── Assertion 20: runtime_harness_verifies_overlay_z_and_alpha ──────────
#[test]
fn harness_p12_beta_a20_runtime_harness_verifies_overlay_z_and_alpha() {
    // Type: A — asserts overlay.z_index == 10 AND overlay.modulate.a < 1.0.
    let src = read_file(RUNTIME_HARNESS_PATH);
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("z_index"),
        "A20.1: runtime harness must reference `z_index` to check overlay z-order"
    );
    // Check for literal `10` comparison or `Z_OVERLAY` literal usage with z_index.
    let z_check = stripped.contains("z_index == 10")
        || stripped.contains("z_index != 10")
        || stripped.contains("z_index >= 10");
    assert!(
        z_check,
        "A20.2: runtime harness must compare z_index against 10 \
         (overlay z-order invariant)"
    );

    assert!(
        stripped.contains("modulate.a"),
        "A20.3: runtime harness must reference `modulate.a` to check overlay alpha"
    );
    let alpha_check = stripped.contains("modulate.a < 1.0")
        || stripped.contains("modulate.a < 1")
        || stripped.contains("modulate.a >= 1.0");
    assert!(
        alpha_check,
        "A20.4: runtime harness must compare `modulate.a < 1.0` \
         (overlay translucency invariant)"
    );
    println!("[P12-β A20] runtime harness asserts z_index == 10 AND modulate.a < 1.0 ✓");
}

// ─── Assertion 21: phase_4_gamma_sprite_scale_invariant_preserved ────────
#[test]
fn harness_p12_beta_a21_phase4_gamma_sprite_scale_invariant_preserved() {
    // Type: D — regression guard. SPRITE_SCALE = 0.25 in agent_renderer.gd.
    let src = read_file(AGENT_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    let accepted = [
        "SPRITE_SCALE := 0.25",
        "SPRITE_SCALE: float = 0.25",
        "SPRITE_SCALE :float = 0.25",
        "SPRITE_SCALE: float= 0.25",
        "SPRITE_SCALE = 0.25",
    ];
    let matched = accepted.iter().any(|c| stripped.contains(c));
    assert!(
        matched,
        "A21: agent_renderer.gd must preserve SPRITE_SCALE = 0.25 \
         (Phase 4-γ tile-fit invariant)"
    );
    println!("[P12-β A21] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 22: d1_state_tints_palette_preserved ──────────────────────
#[test]
fn harness_p12_beta_a22_d1_state_tints_palette_preserved() {
    // Type: D — regression guard. All 4 D1 STATE_TINTS Color literals preserved.
    let src = read_file(AGENT_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    let required = [
        "Color(0.55, 0.70, 0.95, 1.0)",
        "Color(1.0, 0.85, 0.15, 1.0)",
        "Color(1.0, 0.40, 0.75, 1.0)",
        "Color(0.30, 0.95, 0.35, 1.0)",
    ];
    for needle in required.iter() {
        assert!(
            stripped.contains(needle),
            "A22: agent_renderer.gd MUST preserve D1 STATE_TINTS literal `{needle}`"
        );
    }
    println!("[P12-β A22] all 4 D1 STATE_TINTS Color literals preserved ✓");
}

// ─── Assertion 23: phase_12_alpha_camera_zoom_invariant_preserved ────────
#[test]
fn harness_p12_beta_a23_phase12_alpha_camera_zoom_invariant_preserved() {
    // Type: D — regression guard. Camera2D zoom=Vector2(2,2) AND
    // camera_controller.gd script ExtResource attachment.
    let tscn = read_file(MAIN_TSCN_PATH);

    let has_zoom_2x = tscn.contains("zoom = Vector2(2, 2)")
        || tscn.contains("zoom = Vector2(2.0, 2.0)");
    assert!(
        has_zoom_2x,
        "A23.1: main.tscn must preserve Camera2D `zoom = Vector2(2, 2)` (Phase 12-α)"
    );

    // Check for camera_controller.gd ExtResource attachment.
    let has_camera_script = tscn.contains("camera_controller.gd")
        && tscn.contains("script = ExtResource(");
    assert!(
        has_camera_script,
        "A23.2: main.tscn must preserve camera_controller.gd ExtResource \
         + `script = ExtResource(...)` attachment (Phase 12-α)"
    );
    println!("[P12-β A23] Camera2D zoom=Vector2(2,2) + camera_controller.gd attached ✓");
}

// ─── Assertion 24: no_rust_crate_modifications_in_scope_paths (RETIRED) ──
#[test]
fn harness_p12_beta_a24_no_rust_crate_modifications_in_scope_paths() {
    // RETIRED (S16 prep — Stage 61). This guard mis-encoded P12-β's
    // GDScript-only-phase scope promise as a PERMANENT global git-diff check
    // (forbidden_prefixes over sim-core/sim-systems/sim-engine/sim-data),
    // so it FAILed on ANY uncommitted Rust change — blocking ALL future Rust
    // backend work (Section 16+). P12-β's actual Rust-untouched state was
    // verified at its merge commit and persists in git history; re-asserting
    // it against every future working tree is a design error. Forward
    // per-feature scope-creep protection is now provided by the pipeline's
    // F-Phase-A scope-semantic guard + the Codex Evaluator. Retiring restores
    // a no-op pass without losing real coverage.
    // See .harness/prompts/prep-retire-stale-rust-guards.md.
    println!("[P12-β A24] RETIRED — stale global Rust-block guard; forward protection via F-Phase-A + Evaluator");
}

// ─── Assertion 25: gdscript_parses_cleanly_in_godot_headless ─────────────
#[test]
fn harness_p12_beta_a25_gdscript_parses_cleanly_in_godot_headless() {
    // Type: A — Godot --headless --check-only --script on both .gd files
    // must exit 0 with no `SCRIPT ERROR` or `Parse Error` in stderr.
    //
    // Implementation note: the previous attempt crashed in the evaluator
    // environment because Godot defaults its log writes to
    // `user://logs/godot.log` (which resolves to
    // `~/Library/Application Support/Godot/app_userdata/<project>/logs/`).
    // In a sandboxed evaluator that directory is not writable, so Godot
    // bails before parsing. We now pass `--log-file <tempfile>` to redirect
    // the log to a writable temp path, and `--path <project_root>` so the
    // project resolves correctly regardless of cwd. If Godot is absent on
    // the host, the test SKIPS — the pipeline Visual Verify stage will
    // run the same check.
    fn find_godot_binary() -> Option<String> {
        // Honour GODOT env var first, matching tools/harness/harness_pipeline.sh.
        if let Ok(p) = std::env::var("GODOT") {
            if !p.is_empty() && std::path::Path::new(&p).exists() {
                return Some(p);
            }
        }
        let home = std::env::var("HOME").unwrap_or_default();
        let candidates = [
            "/Applications/Godot.app/Contents/MacOS/Godot".to_string(),
            format!("{home}/Downloads/Godot.app/Contents/MacOS/Godot"),
            format!("{home}/Applications/Godot.app/Contents/MacOS/Godot"),
        ];
        for c in candidates.iter() {
            if std::path::Path::new(c).exists() {
                return Some(c.clone());
            }
        }
        // Fall back to PATH lookup (`which godot`).
        if let Ok(o) = std::process::Command::new("which").arg("godot").output() {
            if o.status.success() {
                let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !p.is_empty() && std::path::Path::new(&p).exists() {
                    return Some(p);
                }
            }
        }
        None
    }

    let godot_path = match find_godot_binary() {
        Some(p) => p,
        None => {
            println!(
                "[P12-β A25] SKIP — Godot binary not located via GODOT env, \
                 standard install paths, or PATH (Visual Verify stage will run this check)"
            );
            return;
        }
    };

    let root = project_root();
    let scripts = [
        "scripts/ui/world_renderer.gd",
        "scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd",
    ];

    // Use a temp dir under the OS temp so logging cannot crash on a
    // non-writable user:// path. Each script gets its own log file so the
    // assertion error message can name the failing one.
    let tmp_dir = std::env::temp_dir().join("p12_beta_a25_godot_logs");
    std::fs::create_dir_all(&tmp_dir).ok();

    for script in scripts.iter() {
        let log_file = tmp_dir.join(format!(
            "{}.log",
            script.replace(['/', '.'], "_")
        ));

        let output = std::process::Command::new(&godot_path)
            .arg("--headless")
            .arg("--path")
            .arg(&root)
            .arg("--log-file")
            .arg(&log_file)
            .arg("--check-only")
            .arg("--script")
            .arg(script)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|e| panic!("A25: failed to spawn Godot: {e}"));

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let log_contents = std::fs::read_to_string(&log_file).unwrap_or_default();

        // SCRIPT ERROR / Parse Error scan across stdout, stderr, AND the
        // redirected log file (Godot writes most errors to the log when
        // --log-file is set rather than stderr).
        let has_script_error = stderr.contains("SCRIPT ERROR")
            || stderr.contains("Parse Error")
            || stdout.contains("SCRIPT ERROR")
            || stdout.contains("Parse Error")
            || log_contents.contains("SCRIPT ERROR")
            || log_contents.contains("Parse Error");

        assert!(
            output.status.success() && !has_script_error,
            "A25: Godot --check-only failed for {script}.\n\
             godot_path={godot_path}\n\
             exit_status={:?}\n\
             log_file={log_file:?}\n\
             stderr=\n{stderr}\n\
             stdout=\n{stdout}\n\
             log=\n{log_contents}",
            output.status
        );
    }
    println!("[P12-β A25] both .gd files parse cleanly under Godot --check-only ✓");
}

// Note on Assertion 26 (visual_verification_terrain_visible_under_overlay):
// soft VLM check exercised by the pipeline's Visual Verify stage, not
// embeddable as a cargo test (requires VLM model inference on a runtime
// screenshot). The runtime harness (`harness_terrain.gd`) produces the
// screenshot artefact that A26 consumes.
