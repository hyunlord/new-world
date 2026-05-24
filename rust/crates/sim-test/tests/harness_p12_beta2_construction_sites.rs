//! V7 Phase 12-β.2 (A3) — ConstructionSite rendering + progress alpha harness.
//!
//! Static-inspection + smoke-test harness verifying the Phase 12-β.2 A3
//! implementation:
//!   - `rust/crates/sim-bridge/src/ffi/world_node.rs` — adds
//!     `ConstructionSnapshotRow`, `collect_construction_snapshot`,
//!     `construction_rows_to_dict`, and a `#[func] get_construction_snapshot`
//!     method on `WorldSimNode`.
//!   - `rust/crates/sim-bridge/src/ffi/mod.rs` — re-exports the new symbols.
//!   - `scripts/ui/world_renderer.gd` (modified) — polls the new snapshot,
//!     maintains a keyed dictionary of Sprite2D nodes, reaps despawned sites.
//!   - Regression guards for Phase 4-γ, Phase 11-α + D1, Phase 12-α, and
//!     Phase 12-β.1 invariants.
//!
//! Plan locked thresholds: see `.harness/plans/p12-beta2-a3-construction-sites/plan_final.md`.

use std::fs;
use std::path::PathBuf;

use hecs::World;
use sim_bridge::ffi::{collect_construction_snapshot, ConstructionSnapshotRow};
use sim_core::components::{BuildingBlueprint, ConstructionSite, Position};

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

/// Strip Rust line comments (`// …` to EOL). Preserves `//` inside string
/// literals (single or double quoted).
fn strip_rs_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut in_str: Option<char> = None;
        let bytes = line.as_bytes();
        let mut keep_end = line.len();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;
            match in_str {
                Some(q) if c == q => in_str = None,
                None if c == '"' || c == '\'' => in_str = Some(c),
                None if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' => {
                    keep_end = i;
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        out.push_str(&line[..keep_end]);
        out.push('\n');
    }
    out
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

const WORLD_NODE_PATH: &str = "rust/crates/sim-bridge/src/ffi/world_node.rs";
const WORLD_RENDERER_PATH: &str = "scripts/ui/world_renderer.gd";
const AGENT_RENDERER_PATH: &str = "scripts/ui/agent_renderer.gd";
const MAIN_TSCN_PATH: &str = "scenes/main.tscn";

// ─── Assertion 1: collector_function_symbol_exists ────────────────────────
#[test]
fn harness_p12_beta2_a1_collector_function_symbol_exists() {
    // Type: A — exactly 1 `pub fn collect_construction_snapshot` symbol
    // accepting `&hecs::World` (or `&World`) in sim-bridge.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);
    let count = stripped
        .matches("pub fn collect_construction_snapshot")
        .count();
    assert_eq!(
        count, 1,
        "A1: must have exactly 1 `pub fn collect_construction_snapshot` declaration; found {count}"
    );
    println!("[P12-β.2 A1] collect_construction_snapshot symbol present ✓");
}

// ─── Assertion 2: snapshot_row_struct_has_five_required_fields ────────────
#[test]
fn harness_p12_beta2_a2_snapshot_row_struct_has_five_required_fields() {
    // Type: A — `pub struct ConstructionSnapshotRow` declared with all 5
    // field names: entity_bits, x, y, progress, required_progress.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    let struct_pos = stripped
        .find("pub struct ConstructionSnapshotRow")
        .expect("A2.1: `pub struct ConstructionSnapshotRow` must be declared");
    let end = stripped[struct_pos..]
        .find('}')
        .map(|i| struct_pos + i + 1)
        .unwrap_or(stripped.len());
    let body = &stripped[struct_pos..end];
    for field in [
        "entity_bits",
        "x",
        "y",
        "progress",
        "required_progress",
    ] {
        assert!(
            body.contains(field),
            "A2.2: ConstructionSnapshotRow must declare field `{field}`; body=\n{body}"
        );
    }
    println!("[P12-β.2 A2] all 5 ConstructionSnapshotRow fields present ✓");
}

// ─── Assertion 3: collector_queries_correct_component_tuple ───────────────
#[test]
fn harness_p12_beta2_a3_collector_queries_correct_component_tuple() {
    // Type: A — `query::<(&ConstructionSite, &Position)>` (or equivalent
    // stripped-comment form) inside `collect_construction_snapshot`.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    let fn_pos = stripped
        .find("pub fn collect_construction_snapshot")
        .expect("A3.1: collect_construction_snapshot must exist");
    // Scope: from fn declaration to the next `pub fn` or the end of file.
    let next_pub_fn = stripped[fn_pos + 1..]
        .find("\npub fn ")
        .map(|i| fn_pos + 1 + i)
        .unwrap_or(stripped.len());
    let body = &stripped[fn_pos..next_pub_fn];

    // Acceptable forms (whitespace-tolerant via simple substring check):
    let acceptable = [
        "query::<(&ConstructionSite, &Position)>",
        "query::<(&ConstructionSite,&Position)>",
    ];
    let matched = acceptable.iter().any(|p| body.contains(p));
    assert!(
        matched,
        "A3.2: collect_construction_snapshot body must query \
         `(&ConstructionSite, &Position)`; body=\n{body}"
    );
    println!("[P12-β.2 A3] collector queries (&ConstructionSite, &Position) ✓");
}

// ─── Assertion 4: ffi_method_exposed_with_godot_func_attribute ────────────
#[test]
fn harness_p12_beta2_a4_ffi_method_exposed_with_godot_func_attribute() {
    // Type: A — `#[func]` immediately preceding
    // `fn get_construction_snapshot(&self) -> VarDictionary`, AND the body
    // references `collect_construction_snapshot` with the engine world.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    // Find the fn signature.
    let sig = "fn get_construction_snapshot(&self) -> VarDictionary";
    let sig_pos = stripped
        .find(sig)
        .unwrap_or_else(|| panic!("A4.1: `{sig}` must be declared"));

    // Walk backward to the previous non-empty line; require `#[func]`.
    let prefix = &stripped[..sig_pos];
    let prev_lines: Vec<&str> = prefix.lines().collect();
    let mut last_nonempty: Option<&str> = None;
    for line in prev_lines.iter().rev() {
        if !line.trim().is_empty() {
            last_nonempty = Some(line.trim());
            break;
        }
    }
    let preceding = last_nonempty.unwrap_or("");
    assert_eq!(
        preceding, "#[func]",
        "A4.2: line immediately preceding `{sig}` must be `#[func]`; got `{preceding}`"
    );

    // Find the function body: from sig_pos forward to first '}' at brace depth 0.
    let after_sig = &stripped[sig_pos..];
    let open_brace = after_sig
        .find('{')
        .unwrap_or_else(|| panic!("A4.3: function body opening `{{` not found after signature"));
    let mut depth = 0i32;
    let mut close_idx = open_brace;
    for (i, c) in after_sig[open_brace..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = open_brace + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &after_sig[open_brace..=close_idx];
    assert!(
        body.contains("collect_construction_snapshot"),
        "A4.4: function body must call `collect_construction_snapshot`; body=\n{body}"
    );
    println!("[P12-β.2 A4] #[func] get_construction_snapshot forwards to collector ✓");
}

// ─── Assertion 5: dict_marshaller_sets_five_parallel_array_keys ───────────
#[test]
fn harness_p12_beta2_a5_dict_marshaller_sets_five_parallel_array_keys() {
    // Type: A — `dict.set` calls for all five string keys: "ids", "xs",
    // "ys", "progresses", "required_progresses".
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    let needles = [
        "dict.set(\"ids\"",
        "dict.set(\"xs\"",
        "dict.set(\"ys\"",
        "dict.set(\"progresses\"",
        "dict.set(\"required_progresses\"",
    ];
    for needle in needles.iter() {
        assert!(
            stripped.contains(needle),
            "A5: must contain `{needle}` somewhere in sim-bridge source"
        );
    }
    println!("[P12-β.2 A5] all 5 dict.set keys present ✓");
}

// ─── Assertion 6: collector_smoke_test_returns_matching_row ───────────────
#[test]
fn harness_p12_beta2_a6_collector_smoke_test_returns_matching_row() {
    // Type: A — spawn one (ConstructionSite, Position) entity with known
    // values; verify all 5 fields and Vec length on the returned row.
    let mut world = World::new();
    let blueprint = BuildingBlueprint::new(0, 1, 1, 100);
    let mut site = ConstructionSite::new(blueprint, Position { x: 7, y: 11 });
    site.progress = 42;
    let entity = world.spawn((site, Position { x: 7, y: 11 }));

    let rows: Vec<ConstructionSnapshotRow> = collect_construction_snapshot(&world);

    assert_eq!(rows.len(), 1, "A6.1: expected exactly 1 row; got {}", rows.len());
    let row = rows[0];
    assert_eq!(row.x, 7, "A6.2: row.x must equal 7; got {}", row.x);
    assert_eq!(row.y, 11, "A6.3: row.y must equal 11; got {}", row.y);
    assert_eq!(
        row.progress, 42,
        "A6.4: row.progress must equal 42; got {}",
        row.progress
    );
    assert_eq!(
        row.required_progress, 100,
        "A6.5: row.required_progress must equal 100; got {}",
        row.required_progress
    );
    assert_eq!(
        row.entity_bits,
        entity.to_bits().get(),
        "A6.6: row.entity_bits must equal entity.to_bits().get()"
    );
    println!("[P12-β.2 A6] smoke test — row pass-through invariants hold ✓");
}

// ─── Assertion 7: empty_world_returns_empty_vec ───────────────────────────
#[test]
fn harness_p12_beta2_a7_empty_world_returns_empty_vec() {
    // Type: A — empty world → Vec length == 0 (not panic).
    let world = World::new();
    let rows = collect_construction_snapshot(&world);
    assert_eq!(
        rows.len(),
        0,
        "A7: empty world must yield an empty Vec; got {}",
        rows.len()
    );
    println!("[P12-β.2 A7] empty world returns empty Vec ✓");
}

// ─── Assertion 8: renderer_declares_construction_sprite_path_constant ────
#[test]
fn harness_p12_beta2_a8_renderer_declares_construction_sprite_path_constant() {
    // Type: A — CONSTRUCTION_SPRITE_PATH constant in world_renderer.gd
    // referencing `assets/sprites/buildings/` and ending with `.png`.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    let line = stripped
        .lines()
        .find(|l| {
            l.trim_start().starts_with("const ") && l.contains("CONSTRUCTION_SPRITE_PATH")
        })
        .unwrap_or_else(|| {
            panic!("A8.1: world_renderer.gd must declare `const CONSTRUCTION_SPRITE_PATH`")
        });
    assert!(
        line.contains("assets/sprites/buildings/"),
        "A8.2: CONSTRUCTION_SPRITE_PATH must reference `assets/sprites/buildings/`; line=`{line}`"
    );
    assert!(
        line.contains(".png"),
        "A8.3: CONSTRUCTION_SPRITE_PATH must end with `.png`; line=`{line}`"
    );
    println!("[P12-β.2 A8] CONSTRUCTION_SPRITE_PATH declared with valid path ✓");
}

// ─── Assertion 9: alpha_remap_constants_have_exact_values ────────────────
#[test]
fn harness_p12_beta2_a9_alpha_remap_constants_have_exact_values() {
    // Type: A — CONSTRUCTION_ALPHA_MIN = 0.3, CONSTRUCTION_ALPHA_MAX = 1.0,
    // Z_CONSTRUCTION = 5.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    fn find_const_value(stripped: &str, name: &str) -> String {
        let line = stripped
            .lines()
            .find(|l| {
                let t = l.trim_start();
                t.starts_with("const ") && t.contains(name)
            })
            .unwrap_or_else(|| panic!("A9: const {name} not declared"));
        let after_eq = line.rfind('=').map(|i| &line[i + 1..]).unwrap_or("");
        after_eq.trim().to_string()
    }

    let min_val = find_const_value(&stripped, "CONSTRUCTION_ALPHA_MIN");
    let min_parsed: f64 = min_val
        .parse()
        .unwrap_or_else(|_| panic!("A9.1: CONSTRUCTION_ALPHA_MIN must parse as float; got `{min_val}`"));
    assert!(
        (min_parsed - 0.3).abs() < 1e-9,
        "A9.2: CONSTRUCTION_ALPHA_MIN must equal exactly 0.3; got {min_parsed}"
    );

    let max_val = find_const_value(&stripped, "CONSTRUCTION_ALPHA_MAX");
    let max_parsed: f64 = max_val
        .parse()
        .unwrap_or_else(|_| panic!("A9.3: CONSTRUCTION_ALPHA_MAX must parse as float; got `{max_val}`"));
    assert!(
        (max_parsed - 1.0).abs() < 1e-9,
        "A9.4: CONSTRUCTION_ALPHA_MAX must equal exactly 1.0; got {max_parsed}"
    );

    let z_val = find_const_value(&stripped, "Z_CONSTRUCTION");
    let z_parsed: i64 = z_val
        .parse()
        .unwrap_or_else(|_| panic!("A9.5: Z_CONSTRUCTION must parse as int; got `{z_val}`"));
    assert_eq!(
        z_parsed, 5,
        "A9.6: Z_CONSTRUCTION must equal exactly 5; got {z_parsed}"
    );
    println!("[P12-β.2 A9] alpha + z-order constants exact ✓");
}

// ─── Assertion 10: renderer_calls_ffi_snapshot_method ────────────────────
#[test]
fn harness_p12_beta2_a10_renderer_calls_ffi_snapshot_method() {
    // Type: A — `world_sim.get_construction_snapshot()` substring present.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);
    assert!(
        stripped.contains("world_sim.get_construction_snapshot()"),
        "A10: world_renderer.gd must call `world_sim.get_construction_snapshot()`"
    );
    println!("[P12-β.2 A10] renderer polls get_construction_snapshot() ✓");
}

// ─── Assertion 11: reaper_loop_frees_despawned_sprites ───────────────────
#[test]
fn harness_p12_beta2_a11_reaper_loop_frees_despawned_sprites() {
    // Type: A — iteration over `_construction_sprites.keys()` AND a
    // `queue_free()` call inside that iteration AND an `erase()` call on
    // the same dictionary.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("_construction_sprites.keys()"),
        "A11.1: must iterate over `_construction_sprites.keys()`"
    );
    assert!(
        stripped.contains("queue_free()"),
        "A11.2: must contain `queue_free()` call"
    );
    assert!(
        stripped.contains("_construction_sprites.erase"),
        "A11.3: must contain `_construction_sprites.erase` call"
    );
    println!("[P12-β.2 A11] reaper loop has keys() + queue_free() + erase() ✓");
}

// ─── Assertion 12: phase4_gamma_sprite_scale_invariant_preserved ─────────
#[test]
fn harness_p12_beta2_a12_phase4_gamma_sprite_scale_invariant_preserved() {
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
        "A12: agent_renderer.gd must preserve SPRITE_SCALE = 0.25 \
         (Phase 4-γ tile-fit invariant)"
    );
    println!("[P12-β.2 A12] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 13: d1_state_tints_palette_preserved ──────────────────────
#[test]
fn harness_p12_beta2_a13_d1_state_tints_palette_preserved() {
    // Type: D — regression guard. All 4 D1 STATE_TINTS Color literals.
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
            "A13: agent_renderer.gd MUST preserve D1 STATE_TINTS literal `{needle}`"
        );
    }
    println!("[P12-β.2 A13] all 4 D1 STATE_TINTS Color literals preserved ✓");
}

// ─── Assertion 14: phase12_alpha_and_beta1_invariants_preserved ──────────
#[test]
fn harness_p12_beta2_a14_phase12_alpha_and_beta1_invariants_preserved() {
    // Type: D — regression guard for Phase 12-α + 12-β.1:
    //   (a) main.tscn: Camera2D zoom = Vector2(2, 2) + camera_controller.gd attached
    //   (b) world_renderer.gd: TERRAIN_TILESET_PATH, BUILDING_SPRITE_PATH,
    //       OVERLAY_ALPHA literal value 0.65
    let tscn = read_file(MAIN_TSCN_PATH);
    let zoom_ok = tscn.contains("zoom = Vector2(2, 2)")
        || tscn.contains("zoom = Vector2(2.0, 2.0)");
    assert!(
        zoom_ok,
        "A14.1: main.tscn must preserve Camera2D `zoom = Vector2(2, 2)` (Phase 12-α)"
    );
    let camera_script_ok =
        tscn.contains("camera_controller.gd") && tscn.contains("script = ExtResource(");
    assert!(
        camera_script_ok,
        "A14.2: main.tscn must preserve camera_controller.gd ExtResource attachment (Phase 12-α)"
    );

    let renderer = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&renderer);
    assert!(
        stripped.contains("TERRAIN_TILESET_PATH"),
        "A14.3: world_renderer.gd must still declare TERRAIN_TILESET_PATH (Phase 12-β.1)"
    );
    assert!(
        stripped.contains("BUILDING_SPRITE_PATH"),
        "A14.4: world_renderer.gd must still declare BUILDING_SPRITE_PATH (Phase 12-β.1)"
    );

    // OVERLAY_ALPHA literal value 0.65.
    let alpha_line = stripped
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("const ") && t.contains("OVERLAY_ALPHA")
        })
        .unwrap_or_else(|| panic!("A14.5: const OVERLAY_ALPHA must remain declared"));
    let after_eq = alpha_line.rfind('=').map(|i| &alpha_line[i + 1..]).unwrap_or("");
    let val: f64 = after_eq
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("A14.6: OVERLAY_ALPHA must parse as float; line=`{alpha_line}`"));
    assert!(
        (val - 0.65).abs() < 1e-9,
        "A14.7: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    );
    println!("[P12-β.2 A14] Phase 12-α + 12-β.1 invariants preserved ✓");
}
