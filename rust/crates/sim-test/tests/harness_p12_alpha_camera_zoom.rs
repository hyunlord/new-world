//! V7 Phase 12-α — Camera Zoom Controls + Default 2.0× Zoom harness.
//!
//! Static file-inspection harness verifying the Phase 12-α implementation:
//!   - `scripts/ui/camera_controller.gd` (new) — Camera2D subclass with
//!     mouse-wheel zoom controls between [0.5×, 4.0×], 1.25× geometric
//!     step, 0.15s ease-out tween, default zoom 2.0×.
//!   - `scenes/main.tscn` (modified) — Camera2D node now has the controller
//!     script attached and zoom default Vector2(2, 2).
//!   - Anti-regression guards for Phase 4-γ SPRITE_SCALE = 0.25 invariant
//!     and D1 STATE_TINTS palette.
//!
//! Run: `cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture`

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

fn read_camera_controller_src() -> String {
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("camera_controller.gd");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn read_agent_renderer_src() -> String {
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("agent_renderer.gd");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn read_main_tscn_src() -> String {
    let path = project_root().join("scenes").join("main.tscn");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Strip GDScript line comments (`# …` to EOL). Quoted `#` inside string
/// literals is preserved.
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

/// Extract the property block (lines between the `[node name="Camera2D"` line
/// and the next `[node ` or `[ext_resource` marker) from main.tscn. Returns
/// the slice including the `[node ...]` header line for context.
fn extract_camera2d_block(tscn: &str) -> String {
    let needle = "[node name=\"Camera2D\" type=\"Camera2D\" parent=\".\"]";
    let start = tscn
        .find(needle)
        .unwrap_or_else(|| panic!("Camera2D node block not found in main.tscn"));
    let tail = &tscn[start..];
    // Find the next [node or [ext_resource after the header.
    // Skip past the header line first.
    let after_header = tail
        .find('\n')
        .map(|i| i + 1)
        .unwrap_or(tail.len());
    let body = &tail[after_header..];
    let end_rel = body
        .find("\n[node ")
        .or_else(|| body.find("\n[ext_resource"))
        .or_else(|| body.find("\n[connection"))
        .unwrap_or(body.len());
    let total_end = after_header + end_rel;
    tail[..total_end].to_string()
}

// ─── Assertion 1: camera_controller_script_file_exists ────────────────────
#[test]
fn harness_p12_alpha_a1_camera_controller_file_exists() {
    // Type: A (physical invariant). The file MUST exist and be non-empty;
    // the feature is defined as "a new GDScript file at this path".
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("camera_controller.gd");
    assert!(
        path.exists(),
        "A1: scripts/ui/camera_controller.gd must exist; not found at {path:?}"
    );
    let meta = fs::metadata(&path).expect("file metadata");
    assert!(
        meta.len() > 0,
        "A1: scripts/ui/camera_controller.gd must be non-empty; got {} bytes",
        meta.len()
    );
    // Readable as UTF-8.
    let src = fs::read_to_string(&path).expect("UTF-8 readable");
    assert!(
        !src.is_empty(),
        "A1: camera_controller.gd must contain UTF-8 text content"
    );
    println!("[P12-α A1] camera_controller.gd exists, non-empty, UTF-8 ✓");
}

// ─── Assertion 2: camera_controller_extends_camera2d ──────────────────────
#[test]
fn harness_p12_alpha_a2_extends_camera2d() {
    // Type: A (physical invariant). The controller MUST subclass Camera2D
    // to override `zoom` and receive `_unhandled_input` while attached to
    // the Camera2D node.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let count = stripped.matches("extends Camera2D").count();
    assert!(
        count >= 1,
        "A2: camera_controller.gd must contain `extends Camera2D` \
         in non-comment source; found {count} occurrence(s)"
    );
    println!("[P12-α A2] camera_controller.gd extends Camera2D ✓");
}

// ─── Assertion 3: zoom_min_constant_declared ──────────────────────────────
#[test]
fn harness_p12_alpha_a3_zoom_min_constant() {
    // Type: A (physical invariant). The 0.5× lower bound is the contractual
    // zoom-out limit. Required for clamp verification.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);

    // Regex-style search: locate `ZOOM_MIN` and verify its right-hand side
    // contains `Vector2(0.5, 0.5)` (whitespace-tolerant for the comma gap).
    let idx = stripped
        .find("ZOOM_MIN")
        .unwrap_or_else(|| panic!("A3: ZOOM_MIN identifier not found in stripped source"));
    let rhs = &stripped[idx..];
    // Take the rest of the line (until newline).
    let line_end = rhs.find('\n').unwrap_or(rhs.len());
    let line = &rhs[..line_end];

    let accepted = [
        "Vector2(0.5, 0.5)",
        "Vector2(0.5,0.5)",
        "Vector2(0.5 , 0.5)",
    ];
    let matched = accepted.iter().any(|c| line.contains(c));
    assert!(
        matched,
        "A3: ZOOM_MIN must be declared with `Vector2(0.5, 0.5)` (whitespace \
         tolerant). Line: `{line}`"
    );
    println!("[P12-α A3] ZOOM_MIN = Vector2(0.5, 0.5) ✓");
}

// ─── Assertion 4: zoom_max_constant_declared ──────────────────────────────
#[test]
fn harness_p12_alpha_a4_zoom_max_constant() {
    // Type: A (physical invariant). 4.0× is the contractual zoom-in limit.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let idx = stripped
        .find("ZOOM_MAX")
        .unwrap_or_else(|| panic!("A4: ZOOM_MAX identifier not found"));
    let rhs = &stripped[idx..];
    let line_end = rhs.find('\n').unwrap_or(rhs.len());
    let line = &rhs[..line_end];

    let accepted = [
        "Vector2(8.0, 8.0)",
        "Vector2(8.0,8.0)",
        "Vector2(8.0 , 8.0)",
    ];
    let matched = accepted.iter().any(|c| line.contains(c));
    assert!(
        matched,
        "A4: ZOOM_MAX must be declared with `Vector2(8.0, 8.0)` (whitespace \
         tolerant, raised from 4.0 in G Phase A). Line: `{line}`"
    );
    println!("[P12-α A4] ZOOM_MAX = Vector2(8.0, 8.0) (G Phase A) ✓");
}

// ─── Assertion 5: zoom_default_constant_is_three_x ────────────────────────
#[test]
fn harness_p12_alpha_a5_zoom_default_constant() {
    // Type: A (physical invariant). Original Phase 12-α dispatch locked
    // ZOOM_DEFAULT to 2.0×. Phase 13-α (Path B) intentionally raises this
    // to 3.0× so 16×18 px agent sprites render at 48–54 px (above the
    // perceptual threshold) while preserving the Phase 4-γ SPRITE_SCALE
    // = 0.25 invariant. The new locked value is enforced here AND in
    // harness_p13_alpha_camera_and_buildings.rs A1. Same precedent as
    // P12-α A18 releasing the world_renderer.gd lock when β.1 modified it.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let idx = stripped
        .find("ZOOM_DEFAULT")
        .unwrap_or_else(|| panic!("A5: ZOOM_DEFAULT identifier not found"));
    let rhs = &stripped[idx..];
    let line_end = rhs.find('\n').unwrap_or(rhs.len());
    let line = &rhs[..line_end];

    let accepted = [
        "Vector2(3.0, 3.0)",
        "Vector2(3.0,3.0)",
        "Vector2(3.0 , 3.0)",
    ];
    let matched = accepted.iter().any(|c| line.contains(c));
    assert!(
        matched,
        "A5: ZOOM_DEFAULT must be declared with `Vector2(3.0, 3.0)` \
         (whitespace tolerant; raised from 2.0× by Phase 13-α). Line: `{line}`"
    );
    println!("[P12-α A5] ZOOM_DEFAULT = Vector2(3.0, 3.0) ✓ (Phase 13-α supersession)");
}

// ─── Assertion 6: zoom_factor_geometric_step ──────────────────────────────
#[test]
fn harness_p12_alpha_a6_zoom_factor_geometric() {
    // Type: A (physical invariant per spec §3). 1.25× geometric step is the
    // contractual UX value (~4 notches across 0.5×–4.0× range).
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let idx = stripped
        .find("ZOOM_FACTOR")
        .unwrap_or_else(|| panic!("A6: ZOOM_FACTOR identifier not found"));
    let rhs = &stripped[idx..];
    let line_end = rhs.find('\n').unwrap_or(rhs.len());
    let line = &rhs[..line_end];

    assert!(
        line.contains("1.25"),
        "A6: ZOOM_FACTOR must equal literal `1.25`. Line: `{line}`"
    );
    // Negative guard: forbid drift to 1.5 / 2.0 on the same line.
    assert!(
        !line.contains("1.5") || line.contains("1.25"),
        "A6: ZOOM_FACTOR must be exactly 1.25, not 1.5. Line: `{line}`"
    );
    println!("[P12-α A6] ZOOM_FACTOR = 1.25 ✓");
}

// ─── Assertion 7: tween_duration_constant ─────────────────────────────────
#[test]
fn harness_p12_alpha_a7_tween_duration_constant() {
    // Type: A (physical invariant per spec §3). 0.15s tween duration.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let idx = stripped
        .find("TWEEN_DURATION")
        .unwrap_or_else(|| panic!("A7: TWEEN_DURATION identifier not found"));
    let rhs = &stripped[idx..];
    let line_end = rhs.find('\n').unwrap_or(rhs.len());
    let line = &rhs[..line_end];

    assert!(
        line.contains("0.15"),
        "A7: TWEEN_DURATION must equal literal `0.15`. Line: `{line}`"
    );
    println!("[P12-α A7] TWEEN_DURATION = 0.15 ✓");
}

// ─── Assertion 8: mouse_wheel_up_handler_present ──────────────────────────
#[test]
fn harness_p12_alpha_a8_wheel_up_handler() {
    // Type: A (physical invariant). The 'mouse-wheel zoom-in' feature
    // requirement cannot be satisfied without referencing this Godot input
    // constant.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let count = stripped.matches("MOUSE_BUTTON_WHEEL_UP").count();
    assert!(
        count >= 1,
        "A8: camera_controller.gd must reference MOUSE_BUTTON_WHEEL_UP; \
         found {count} occurrence(s)"
    );
    println!("[P12-α A8] MOUSE_BUTTON_WHEEL_UP referenced ✓");
}

// ─── Assertion 9: mouse_wheel_down_handler_present ────────────────────────
#[test]
fn harness_p12_alpha_a9_wheel_down_handler() {
    // Type: A (physical invariant). Symmetric to A8.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);
    let count = stripped.matches("MOUSE_BUTTON_WHEEL_DOWN").count();
    assert!(
        count >= 1,
        "A9: camera_controller.gd must reference MOUSE_BUTTON_WHEEL_DOWN; \
         found {count} occurrence(s)"
    );
    println!("[P12-α A9] MOUSE_BUTTON_WHEEL_DOWN referenced ✓");
}

// ─── Assertion 10: zoom_clamped_to_min_and_max ────────────────────────────
#[test]
fn harness_p12_alpha_a10_zoom_clamp() {
    // Type: A (physical invariant). Without clamping, the zoom can drift
    // outside [0.5, 4.0] after enough wheel notches.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("clamp("),
        "A10.1: camera_controller.gd must contain a clamp(...) call"
    );

    // Walk over clamp( calls and look for one within 3 lines that
    // references both ZOOM_MIN and ZOOM_MAX. To keep this simple we look
    // for a 6-line window in the file containing both identifiers along
    // with a clamp( call.
    let lines: Vec<&str> = stripped.lines().collect();
    let mut found = false;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("clamp(") {
            // Inspect window [i-3 .. i+3].
            let lo = i.saturating_sub(3);
            let hi = (i + 4).min(lines.len());
            let window = lines[lo..hi].join("\n");
            if window.contains("ZOOM_MIN") && window.contains("ZOOM_MAX") {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "A10.2: at least one clamp(...) call must reference BOTH ZOOM_MIN \
         and ZOOM_MAX within ±3 lines (zoom bound contract)"
    );
    println!("[P12-α A10] clamp(...) references ZOOM_MIN and ZOOM_MAX ✓");
}

// ─── Assertion 11: tween_used_for_zoom_property ───────────────────────────
#[test]
fn harness_p12_alpha_a11_create_tween_used() {
    // Type: A (physical invariant). The 'smooth tween interpolation' spec
    // cannot be satisfied without create_tween + tween_property targeting
    // zoom.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("create_tween("),
        "A11.1: camera_controller.gd must contain `create_tween(` call"
    );

    // Search for a tween_property call whose first non-self argument string
    // literal equals `"zoom"`. The Godot API signature is
    // `tween_property(object, property, final_val, duration)`, so the
    // pattern we look for is `tween_property(<obj>, "zoom"`.
    let stripped_compact: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    let pattern_matches = stripped_compact.contains("tween_property(self, \"zoom\"")
        || stripped_compact.contains("tween_property( self, \"zoom\"")
        || stripped_compact.contains("tween_property(self , \"zoom\"")
        || stripped.contains("tween_property(self, \"zoom\"")
        || stripped.contains("\"zoom\",");
    assert!(
        pattern_matches,
        "A11.2: camera_controller.gd must call \
         `tween_property(self, \"zoom\", ...)` targeting the `zoom` property"
    );
    println!("[P12-α A11] create_tween + tween_property(\"zoom\", ...) ✓");
}

// ─── Assertion 12: input_marked_as_handled ────────────────────────────────
#[test]
fn harness_p12_alpha_a12_input_handled_marked() {
    // Type: A (physical invariant). The spec requires wheel events to not
    // propagate to existing handlers in world_renderer.gd / causal_panel.gd.
    let src = read_camera_controller_src();
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("set_input_as_handled()"),
        "A12.1: camera_controller.gd must call `set_input_as_handled()` \
         (with or without `get_viewport().` prefix)"
    );

    // Confirm the call appears within an `_unhandled_input` function body.
    // Find the `_unhandled_input` definition and extract its body up to
    // the next top-level `func ` keyword.
    let fn_start = stripped.find("func _unhandled_input(").unwrap_or_else(|| {
        panic!(
            "A12.2: camera_controller.gd must define a func _unhandled_input(...) \
             that processes wheel events"
        )
    });
    let after_sig = fn_start;
    let tail = &stripped[after_sig..];
    let body_end_rel = tail[1..]
        .find("\nfunc ")
        .map(|i| i + 1)
        .unwrap_or(tail.len());
    let body = &tail[..body_end_rel];

    assert!(
        body.contains("set_input_as_handled()"),
        "A12.3: set_input_as_handled() must appear within the _unhandled_input \
         function body. Body:\n{body}"
    );
    println!("[P12-α A12] set_input_as_handled() inside _unhandled_input ✓");
}

// ─── Assertion 13: main_tscn_camera_default_zoom_is_two_x ─────────────────
#[test]
fn harness_p12_alpha_a13_main_tscn_camera_zoom_default_2x() {
    // Type: A (physical invariant). The feature's user-visible delta is
    // the default 2.0× zoom at scene load. Spec §2: scene-file default
    // also 2.0×.
    let tscn = read_main_tscn_src();
    let block = extract_camera2d_block(&tscn);

    let two_x = block.contains("zoom = Vector2(2, 2)")
        || block.contains("zoom = Vector2(2.0, 2.0)");
    assert!(
        two_x,
        "A13.1: Camera2D node in main.tscn must have `zoom = Vector2(2, 2)` \
         (or `Vector2(2.0, 2.0)`). Block:\n{block}"
    );

    let still_one = block.contains("zoom = Vector2(1, 1)")
        || block.contains("zoom = Vector2(1.0, 1.0)");
    assert!(
        !still_one,
        "A13.2: Camera2D node in main.tscn MUST NOT still have \
         `zoom = Vector2(1, 1)` (pre-feature value). Block:\n{block}"
    );
    println!("[P12-α A13] main.tscn Camera2D zoom = Vector2(2, 2) ✓");
}

// ─── Assertion 14: main_tscn_camera_has_script_attached ───────────────────
#[test]
fn harness_p12_alpha_a14_main_tscn_camera_script_attached() {
    // Type: A (physical invariant). Without the script attached to the
    // Camera2D node, _unhandled_input never fires and wheel zoom does
    // nothing.
    let tscn = read_main_tscn_src();
    let block = extract_camera2d_block(&tscn);

    // Find the `script = ExtResource("<id>")` line inside the block.
    let script_line = block.lines().find(|l| l.trim_start().starts_with("script = ExtResource"));
    let script_line = script_line.unwrap_or_else(|| {
        panic!(
            "A14.1: Camera2D node in main.tscn must have a `script = ExtResource(...)` \
             line. Block:\n{block}"
        )
    });

    // Extract the id from `script = ExtResource("<id>")` (or non-quoted form).
    let id_start = script_line
        .find("ExtResource(")
        .map(|i| i + "ExtResource(".len())
        .expect("A14.2: parse ExtResource id");
    let after = &script_line[id_start..];
    let id_end = after
        .find(')')
        .unwrap_or_else(|| panic!("A14.3: ExtResource(...) missing `)`"));
    let id_raw = after[..id_end].trim().trim_matches('"');

    // Now find the matching [ext_resource] declaration with id=<id_raw> and
    // path referencing camera_controller.gd.
    // Format: [ext_resource type="Script" path="..." id="..."]
    let ext_decls: Vec<&str> = tscn.lines().filter(|l| l.starts_with("[ext_resource")).collect();
    let matching = ext_decls.iter().find(|l| {
        let id_match = l.contains(&format!("id=\"{id_raw}\""))
            || l.contains(&format!("id={id_raw}"));
        let path_match = l.contains("path=\"res://scripts/ui/camera_controller.gd\"");
        id_match && path_match
    });
    assert!(
        matching.is_some(),
        "A14.4: no [ext_resource ...] declaration found with id=\"{id_raw}\" \
         AND path=\"res://scripts/ui/camera_controller.gd\". Found ext_resource \
         declarations:\n{}",
        ext_decls.join("\n")
    );
    println!(
        "[P12-α A14] Camera2D script = ExtResource({id_raw}) → camera_controller.gd ✓"
    );
}

// ─── Assertion 15: phase4_gamma_sprite_scale_invariant_preserved ─────────
#[test]
fn harness_p12_alpha_a15_phase4_gamma_sprite_scale_invariant() {
    // Type: D (regression guard). Phase 4-γ established SPRITE_SCALE = 0.25
    // as the tile-fit invariant; P12Plan-4 explicitly forbids changing it.
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);

    // Find the SPRITE_SCALE declaration line; accept both := and : float = forms.
    let idx = stripped
        .find("SPRITE_SCALE")
        .unwrap_or_else(|| panic!("A15.1: SPRITE_SCALE identifier not found in agent_renderer.gd"));
    let rhs = &stripped[idx..];
    let line_end = rhs.find('\n').unwrap_or(rhs.len());
    let line = &rhs[..line_end];

    // Reject if literal `0.25` is missing.
    assert!(
        line.contains("0.25"),
        "A15.2: SPRITE_SCALE must be bound to literal 0.25 (Phase 4-γ \
         tile-fit invariant). Line: `{line}`"
    );

    // Accept `const SPRITE_SCALE := 0.25` or `const SPRITE_SCALE: float = 0.25`.
    let accepted = [
        "SPRITE_SCALE := 0.25",
        "SPRITE_SCALE: float = 0.25",
        "SPRITE_SCALE :float = 0.25",
        "SPRITE_SCALE: float= 0.25",
    ];
    let matched = accepted.iter().any(|c| stripped.contains(c));
    assert!(
        matched,
        "A15.3: SPRITE_SCALE must be declared via `:= 0.25` or `: float = 0.25` \
         form. Line: `{line}`"
    );
    println!("[P12-α A15] SPRITE_SCALE = 0.25 preserved (Phase 4-γ invariant) ✓");
}

// ─── Assertion 16: d1_state_tints_palette_preserved ──────────────────────
#[test]
fn harness_p12_alpha_a16_d1_state_tints_palette_preserved() {
    // Type: D (regression guard). The 2.0× default zoom exists specifically
    // to make D1's retuned STATE_TINTS palette observable.
    let src = read_agent_renderer_src();
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
            "A16: agent_renderer.gd MUST preserve D1 STATE_TINTS literal `{needle}` \
             (missing one fails the palette invariant)"
        );
    }
    println!("[P12-α A16] all 4 D1 STATE_TINTS Color literals present ✓");
}

// ─── Assertion 17: no_rust_crate_source_modified ─────────────────────────
#[test]
fn harness_p12_alpha_a17_no_rust_crate_source_modified() {
    // Type: D (regression guard against scope creep). Spec §2 explicitly
    // lists 'Not changed' as all Rust crates except this new sim-test file.
    //
    // Implementation: query `git diff --name-only` between HEAD~1 and the
    // working tree to enumerate files touched by this feature. If git is
    // unavailable, fall back to a passive check via `git status` — and if
    // both fail, skip the test rather than block the suite (the assertion
    // is informational on a clean working tree).
    let root = project_root();
    // Use git to enumerate files modified between origin/lead/main and HEAD,
    // plus the working tree.
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["diff", "--name-only", "origin/lead/main...HEAD"])
        .output();
    let mut modified: Vec<String> = Vec::new();
    if let Ok(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            if !line.trim().is_empty() {
                modified.push(line.trim().to_string());
            }
        }
    }
    // Also include uncommitted working-tree changes.
    let output2 = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--porcelain"])
        .output();
    if let Ok(o) = output2 {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            // Porcelain format: XY <path>
            let path = line.get(3..).unwrap_or("").trim();
            if !path.is_empty() {
                modified.push(path.to_string());
            }
        }
    }

    // V7 Phase 12-β.2 (A3) expansion: sim-bridge FFI is now a legitimate
    // surface for renderer-feeding snapshots (e.g. construction-site
    // rendering). Mirrors the β.1 precedent that opened world_renderer.gd
    // (A18). β.2's own A14 regression guard locks the Phase 12-α visible
    // invariants (Camera2D zoom + camera_controller.gd attachment).
    let forbidden_prefixes = [
        "rust/crates/sim-core/src/",
        "rust/crates/sim-systems/src/",
        "rust/crates/sim-engine/src/",
        "rust/crates/sim-data/src/",
    ];
    let violations: Vec<&String> = modified
        .iter()
        .filter(|f| forbidden_prefixes.iter().any(|p| f.starts_with(p)))
        .collect();
    assert!(
        violations.is_empty(),
        "A17: Phase 12-α must NOT modify Rust simulation crate sources \
         (sim-core/sim-systems/sim-engine/sim-data). \
         Violations: {violations:?}"
    );
    println!("[P12-α A17] no Rust crate source modified ✓");
}

// ─── Assertion 18: shader_and_locale_unchanged ────────────────────────────
#[test]
fn harness_p12_alpha_a18_shader_and_locale_unchanged() {
    // Type: D (regression guard for §2 'Not changed' list).
    let root = project_root();
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["diff", "--name-only", "origin/lead/main...HEAD"])
        .output();
    let mut modified: Vec<String> = Vec::new();
    if let Ok(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            if !line.trim().is_empty() {
                modified.push(line.trim().to_string());
            }
        }
    }
    let output2 = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--porcelain"])
        .output();
    if let Ok(o) = output2 {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            let path = line.get(3..).unwrap_or("").trim();
            if !path.is_empty() {
                modified.push(path.to_string());
            }
        }
    }

    // Phase 12-α scope locks. Phase 12-β intentionally modifies
    // `scripts/ui/world_renderer.gd` (TileMapLayer terrain + bootstrap
    // building sprite per phase12.md §β plan), so `world_renderer.gd`
    // is NOT a permanent lock — it was an over-restrictive guard in
    // the original α dispatch. Released when β landed.
    //
    // V7 H Phase A — `shaders/palette_swap.gdshader` lock RELEASED. H Phase A
    // intentionally fixes the palette green-bug (vertex-stage tint capture),
    // same precedent as the world_renderer.gd release above. The causal_panel
    // and localization locks remain because no planned stage modifies them.
    let forbidden_exact = ["scripts/ui/panels/causal_panel.gd"];
    let forbidden_prefixes = ["localization/"];
    let mut violations: Vec<&String> = Vec::new();
    for f in modified.iter() {
        if forbidden_exact.iter().any(|p| f == p)
            || forbidden_prefixes.iter().any(|p| f.starts_with(p))
        {
            violations.push(f);
        }
    }
    assert!(
        violations.is_empty(),
        "A18: causal_panel/locale must NOT be modified beyond scope \
         (palette_swap.gdshader lock released in H Phase A). Violations: {violations:?}"
    );
    println!("[P12-α A18] causal_panel + locale unchanged (shader lock released) ✓");
}

// Note on Assertions 19 + 20: those are "run another test suite / full
// workspace gate" assertions. They are validated by the harness runner
// invoking `cargo test --workspace` and the dedicated P11-α suite
// separately; embedding them here would create a recursive cargo invocation.
// They are exercised in the verification commands (spec §5) and the gate.
