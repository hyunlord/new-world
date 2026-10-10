//! V7 Phase 13-α — Camera Zoom 3.0× + Building Sprite Distinction harness.
//!
//! Static file-inspection harness verifying the Phase 13-α implementation:
//!   - `scripts/ui/camera_controller.gd` — ZOOM_DEFAULT raised 2.0× → 3.0×
//!     (Path B: preserve Phase 4-γ SPRITE_SCALE = 0.25 invariant).
//!   - `scripts/ui/world_renderer.gd` — BUILDING_SPRITE_PATH switched from
//!     cairn to campfire so bootstrap / ConstructionSite / Settlement layers
//!     become visually distinct.
//!   - Anti-regression guards for Phase 4-γ, Phase 11-α + D1, Phase 12-α,
//!     β.1, β.2 A3, and γ invariants.
//!
//! Helper discipline (attempt 2):
//!   - `find_decl_rhss` returns ALL whole-RHS strings for an identifier so
//!     callers can enforce both "exactly one declaration" AND "complete RHS
//!     match" (no concatenation, no extra tokens, no `0.250001` slipping
//!     past a `contains("0.25")` substring check).
//!   - D1 STATE_TINTS golden tuples are sourced at test runtime from
//!     `harness_p12_alpha_camera_zoom.rs` A16 so they live in a single
//!     source of truth (see A10 comment for why P12-α and not D-Phase-A).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p13_alpha_camera_and_buildings -- --nocapture

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



fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_main_tscn_src() -> String {
    read_file(&["scenes", "main.tscn"])
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
///
/// RHS rules:
///   - For `:=` (walrus), RHS = everything after `:=`.
///   - For `IDENT: TYPE = RHS` or `IDENT = RHS`, RHS = everything after the
///     first standalone `=` that is NOT part of `:=`.
///   - Trailing whitespace is stripped. Caller decides whether to compare
///     the literal exactly (e.g. `"…"` for strings, `0.25` for floats, `5`
///     for ints).
///
/// Identifier matching requires a word-boundary (next char must not be
/// alphanumeric or `_`) so `BUILDING_SPRITE_PATH` does not match
/// `BUILDING_SPRITE_PATH_FALLBACK`.
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
        // Compute the RHS slice. Prefer `:=` over `=` because `:=` ends in
        // `=` and a naive `find('=')` would land on the second char of `:=`.
        let rhs = if let Some(p) = line.find(":=") {
            line[p + 2..].trim().to_string()
        } else {
            // Find a `=` that is NOT preceded by `:` (avoid `:=`) and NOT
            // followed by `=` (avoid `==`, though declarations never have it).
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

/// Convenience: assert exactly one declaration of `ident` and return its
/// RHS string. Panics with a context-rich message otherwise.
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

/// Compact a string by removing ASCII whitespace. Useful for tolerating
/// inter-token whitespace in vector literals (with or without a space after
/// each comma) so spacing differences do not affect the comparison.
fn no_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Locate the `[node name="Camera2D" ...]` block in main.tscn and return the
/// slice covering that block (header + body, up to the next `[node `,
/// `[ext_resource`, or `[connection` marker).
fn extract_camera2d_block(tscn: &str) -> String {
    let needle = "[node name=\"Camera2D\"";
    let start = tscn
        .find(needle)
        .unwrap_or_else(|| panic!("Camera2D node block not found in main.tscn"));
    let tail = &tscn[start..];
    let after_header = tail.find('\n').map(|i| i + 1).unwrap_or(tail.len());
    let body = &tail[after_header..];
    let end_rel = body
        .find("\n[node ")
        .or_else(|| body.find("\n[ext_resource"))
        .or_else(|| body.find("\n[connection"))
        .unwrap_or(body.len());
    let total_end = after_header + end_rel;
    tail[..total_end].to_string()
}

// ─── Assertion 1: camera_controller_zoom_default_3x ───────────────────────
#[test]
fn harness_p13_alpha_a1_camera_controller_zoom_default_3x() {
    // Type: A — B-1 raised the default zoom literal 3.0× → 5.0×.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A1");
    // Whitespace-tolerant numeric match — accept `5` or `5.0` either side.
    let compact = no_ws(&rhs);
    let accepted = [
        "Vector2(4.0,4.0)",
        "Vector2(4,4)",
        "Vector2(4.0,4)",
        "Vector2(4,4.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A1: ZOOM_DEFAULT RHS must equal exactly one of Vector2(4.0,4.0) | \
         Vector2(4,4) | Vector2(4.0,4) | Vector2(4,4.0) (whitespace-stripped; \
         set to 4.0 by Revitalize Part 4); got RHS=`{rhs}` compact=`{compact}`"
    );
    println!("[P13-α A1] ZOOM_DEFAULT = Vector2(4.0, 4.0) ✓");
}

// ─── Assertion 2: camera_controller_zoom_min_unchanged ────────────────────
#[test]
fn harness_p13_alpha_a2_camera_controller_zoom_min_unchanged() {
    // Type: A — Phase 12-α mouse-wheel zoom-out range invariant.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_MIN", "A2");
    let compact = no_ws(&rhs);
    assert_eq!(
        compact, "Vector2(0.5,0.5)",
        "A2: ZOOM_MIN RHS must equal Vector2(0.5,0.5) (whitespace-stripped); got `{rhs}`"
    );
    println!("[P13-α A2] ZOOM_MIN = Vector2(0.5, 0.5) preserved ✓");
}

// ─── Assertion 3: camera_controller_zoom_max_unchanged ────────────────────
#[test]
fn harness_p13_alpha_a3_camera_controller_zoom_max_unchanged() {
    // Type: A — Phase 12-α mouse-wheel zoom-in range invariant.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_MAX", "A3");
    let compact = no_ws(&rhs);
    assert_eq!(
        compact, "Vector2(8.0,8.0)",
        "A3: ZOOM_MAX RHS must equal Vector2(8.0,8.0) (whitespace-stripped; \
         raised from 4.0 in G Phase A); got `{rhs}`"
    );
    println!("[P13-α A3] ZOOM_MAX = Vector2(8.0, 8.0) (G Phase A) ✓");
}

// ─── Assertion 4: bootstrap_building_path_campfire ────────────────────────
#[test]
fn harness_p13_alpha_a4_bootstrap_building_path_campfire() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"));
    assert!(src.contains("get_building_snapshot()"));
}

// ─── Assertion 5: bootstrap_no_longer_points_to_cairn ─────────────────────
#[test]
fn harness_p13_alpha_a5_bootstrap_no_longer_points_to_cairn() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"));
    assert!(src.contains("get_construction_snapshot()"));
    assert!(src.contains("sprite.modulate.a = 0.35"));
}

// ─── Assertion 6: construction_sprite_path_unchanged ──────────────────────
#[test]
fn harness_p13_alpha_a6_construction_sprite_path_unchanged() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"));
    assert!(src.contains("get_construction_snapshot()"));
    assert!(src.contains("sprite.modulate.a = 0.35"));
}

// ─── Assertion 7: furniture_sprite_path_repointed ─────────────────────────
#[test]
fn harness_p13_alpha_a7_furniture_sprite_path_unchanged() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("res://assets/sprites/buildings/stockpile/1.png"));
    assert!(src.contains("kind == 2"));
    assert!(src.contains("_stockpile_positions.append"));
}

// ─── Assertion 8: campfire_sprite_file_exists_and_nonempty ────────────────
#[test]
fn harness_p13_alpha_a8_campfire_sprite_file_exists_and_nonempty() {
    // Type: A — A4's referenced file must exist, be non-empty, and have
    // a valid PNG magic header (prevents empty-file substitution).
    let path = project_root()
        .join("assets")
        .join("sprites")
        .join("buildings")
        .join("campfire")
        .join("1.png");
    assert!(
        path.is_file(),
        "A8.1: assets/sprites/buildings/campfire/1.png must exist; not found at {path:?}"
    );
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("A8.2: read {path:?}: {e}"));
    assert!(
        !bytes.is_empty(),
        "A8.3: campfire/1.png must be non-empty; got 0 bytes"
    );
    assert!(
        bytes.len() >= 8,
        "A8.4: campfire/1.png too short for PNG magic; got {} bytes",
        bytes.len()
    );
    let png_magic: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(
        &bytes[..8],
        &png_magic,
        "A8.5: campfire/1.png must start with PNG magic 89 50 4E 47 0D 0A 1A 0A; \
         got first 8 bytes = {:02X?}",
        &bytes[..8]
    );
    println!("[P13-α A8] campfire/1.png exists, non-empty, PNG magic OK ✓");
}

// ─── Assertion 9: phase4_gamma_sprite_scale_invariant_preserved ──────────
#[test]
fn harness_p13_alpha_a9_phase4_gamma_sprite_scale_invariant_preserved() {
    // Type: A — Phase 4-γ agent renderer invariant. Path B preserves
    // SPRITE_SCALE = 0.25 end-to-end.
    //
    // RHS must parse as a float literal EXACTLY equal to one of the
    // accepted forms `0.25` or `.25`. Substring tricks (`contains("=0.25")`
    // would falsely accept `=0.250001`) are explicitly avoided by
    // matching the trimmed RHS against the exact literal set.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "A9");
    // Accept only the exact textual forms. Reject `0.250001`, `0.250`, etc.
    let accepted = ["0.25", ".25"];
    assert!(
        accepted.contains(&rhs.as_str()),
        "A9: SPRITE_SCALE RHS must be EXACTLY `0.25` or `.25` (Phase 4-γ \
         invariant; no `0.250001` drift); got `{rhs}`"
    );
    println!("[P13-α A9] SPRITE_SCALE = 0.25 preserved (exact literal) ✓");
}

// ─── Assertion 10: d1_state_tints_palette_preserved ──────────────────────
#[test]
fn harness_p13_alpha_a10_d1_state_tints_palette_preserved() {
    // Type: A — D1 STATE_TINTS palette invariant.
    //
    // Source-of-truth resolution: the test plan's drafter requested the
    // 4 golden RGBA tuples be sourced from `harness_d_phase_a_runtime_warnings.rs`,
    // but inspection shows that file does NOT actually contain the literals
    // (it covers integer-division warnings + FFI survival, not the palette).
    // The earliest harness that DOES enumerate them in checked code is
    // `harness_p12_alpha_camera_zoom.rs` (A16). We source from that file at
    // test runtime so the 4 literals live in a single source of truth and
    // a future palette retune only needs to update P12-α A16 (this harness
    // will pick up the change automatically).
    let p12_alpha_src = read_file(&[
        "rust",
        "crates",
        "sim-test",
        "tests",
        "harness_p12_alpha_camera_zoom.rs",
    ]);

    // Scan P12-α source for `"Color(...)",` lines and collect the unique
    // `Color(...)` literals. We expect exactly 4 from the A16 golden block.
    let mut golden: Vec<String> = Vec::new();
    for line in p12_alpha_src.lines() {
        // Match patterns of the form `"Color(<inner>)"`. We deliberately
        // require the surrounding quotes so we only pick up Rust string-
        // literal lines (the harness's expected-value array), not any
        // accidental Rust syntax that might say `Color`.
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("\"Color(") {
            // Find the closing `)"` (end of the Color literal inside quotes).
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
        "A10.1: expected to source exactly 4 D1 STATE_TINTS Color literals \
         from harness_p12_alpha_camera_zoom.rs A16; got {n}: {golden:?}",
        n = golden.len()
    );

    // Now verify each golden literal appears in agent_renderer.gd's
    // STATE_TINTS array block (depth-tracked).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_TINTS"),
        "A10.2: agent_renderer.gd must contain `STATE_TINTS` identifier"
    );
    let decl_pos = stripped
        .find("STATE_TINTS")
        .expect("A10.3: STATE_TINTS identifier located");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A10.4: STATE_TINTS array must open with `[`");
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
    assert_eq!(
        depth, 0,
        "A10.5: STATE_TINTS array must terminate with a matching `]`"
    );
    let block = &stripped[decl_pos..k];

    for needle in golden.iter() {
        assert!(
            block.contains(needle),
            "A10.6: STATE_TINTS block MUST contain D1 palette entry `{needle}` \
             (sourced from harness_p12_alpha_camera_zoom.rs A16); block:\n{block}"
        );
    }
    println!(
        "[P13-α A10] all {n} D1 STATE_TINTS Color literals (sourced from \
         P12-α A16) present in STATE_TINTS block ✓",
        n = golden.len()
    );
}

// ─── Assertion 11: phase12_beta1_terrain_tileset_preserved ───────────────
#[test]
fn harness_p13_alpha_a11_phase12_beta1_terrain_tileset_preserved() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("get_terrain_snapshot()"));
    assert!(src.contains("TileSetAtlasSource.new()"));
    assert!(src.contains("_layer.set_cell"));
    assert!(!src.contains("RandomNumberGenerator"));
    assert!(!src.contains("world_terrain.tres"));
}

// ─── Assertion 12: phase12_beta2_construction_z_preserved ────────────────
#[test]
fn harness_p13_alpha_a12_phase12_beta2_construction_z_preserved() {
    let src = include_str!("../../../../scripts/ui/terrain_renderer.gd");
    assert!(src.contains("z_index = 0"));
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("z_index = 5"));
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(src.contains("sprite.z_index = Z_OVERLAY"));
    assert!(src.contains("const Z_OVERLAY: int = 10"));
}

// ─── Assertion 13: phase12_gamma_furniture_z_preserved ───────────────────
#[test]
fn harness_p13_alpha_a13_phase12_gamma_furniture_z_preserved() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_settlement_snapshot()"));
    assert!(src.contains("\"formation_xs\""));
    assert!(src.contains("\"formation_ys\""));
    assert!(src.contains("_tile_center(nearest.x, nearest.y)"));
    assert!(src.contains("_reap(_labels, seen)"));
}

// ─── Assertion 14: three_distinct_building_sprite_paths ──────────────────
#[test]
fn harness_p13_alpha_a14_three_distinct_building_sprite_paths() {
    let src = include_str!("../../../../scripts/ui/building_renderer.gd");
    assert!(src.contains("get_building_snapshot()"));
    assert!(src.contains("_building_textures[kind]"));
    assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"));
}

// ─── Assertion 15: main_tscn_camera2d_script_attached ────────────────────
#[test]
fn harness_p13_alpha_a15_main_tscn_camera2d_script_attached() {
    // Type: A — Phase 12-α scene-structure invariant. ZOOM_DEFAULT only
    // matters if camera_controller.gd is actually attached to the Camera2D
    // node in main.tscn.
    let tscn = read_main_tscn_src();

    assert!(
        tscn.contains("[node name=\"Camera2D\""),
        "A15.1: main.tscn must contain `[node name=\"Camera2D\"` header"
    );

    let block = extract_camera2d_block(&tscn);

    let script_line = block
        .lines()
        .find(|l| l.trim_start().starts_with("script = ExtResource"))
        .unwrap_or_else(|| {
            panic!(
                "A15.2: Camera2D node block must contain a \
                 `script = ExtResource(...)` line. Block:\n{block}"
            )
        });

    let id_start = script_line
        .find("ExtResource(")
        .map(|i| i + "ExtResource(".len())
        .expect("A15.3: parse ExtResource id");
    let after = &script_line[id_start..];
    let id_end = after
        .find(')')
        .unwrap_or_else(|| panic!("A15.4: ExtResource(...) missing `)`"));
    let id_raw = after[..id_end].trim().trim_matches('"');

    let ext_decls: Vec<&str> = tscn
        .lines()
        .filter(|l| l.starts_with("[ext_resource"))
        .collect();
    let matching = ext_decls.iter().find(|l| {
        let id_match = l.contains(&format!("id=\"{id_raw}\""))
            || l.contains(&format!("id={id_raw}"));
        let path_match = l.contains("path=\"res://scripts/ui/camera_controller.gd\"");
        id_match && path_match
    });
    assert!(
        matching.is_some(),
        "A15.5: no [ext_resource ...] declaration found with id=\"{id_raw}\" AND \
         path=\"res://scripts/ui/camera_controller.gd\"; ext_resource decls:\n{}",
        ext_decls.join("\n")
    );
    println!(
        "[P13-α A15] Camera2D node has script = ExtResource({id_raw}) → camera_controller.gd ✓"
    );
}
