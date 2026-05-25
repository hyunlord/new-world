//! V7 Phase 13-δ — Basic Top-Bar HUD harness.
//!
//! Static file-inspection harness verifying the Phase 13-δ implementation:
//!   - `scripts/ui/panels/hud_topbar.gd` — new Control node providing four
//!     read-only counters polled from existing snapshot FFI.
//!   - `scenes/main.tscn` — new node + ext_resource for the HUD, load_steps
//!     bumped accordingly.
//!   - Regression guards for Phase 4-γ, D1, Phase 12-α, Phase 13-α/β/γ
//!     invariants — δ must not touch the agent/world/camera layers.
//!
//! Meta-assertions A18 (GDScript parse), A19 (cargo test --workspace),
//! and A20 (cargo clippy) are verified by the pipeline runner outside this
//! test file.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p13_delta_basic_hud -- --nocapture

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

fn read_hud_topbar_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_topbar.gd"])
}

fn read_main_tscn_src() -> String {
    read_file(&["scenes", "main.tscn"])
}

fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_camera_controller_src() -> String {
    read_file(&["scripts", "ui", "camera_controller.gd"])
}

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
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

fn no_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
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

// ─── Assertion 1: hud_topbar_file_exists ──────────────────────────────────
#[test]
fn harness_p13_delta_a1_hud_topbar_file_exists() {
    // Type: A — physical existence and non-emptiness invariant.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("hud_topbar.gd");
    assert!(
        path.is_file(),
        "A1.1: hud_topbar.gd must exist at {path:?}"
    );
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("A1.2: read {path:?}: {e}"));
    assert!(
        !bytes.is_empty(),
        "A1.3: hud_topbar.gd must be non-empty; got 0 bytes"
    );
    println!(
        "[P13-δ A1] hud_topbar.gd exists, {} bytes ✓",
        bytes.len()
    );
}

// ─── Assertion 2: hud_topbar_extends_control ──────────────────────────────
#[test]
fn harness_p13_delta_a2_hud_topbar_extends_control() {
    // Type: A — first non-comment, non-blank line must be `extends Control`.
    let src = read_hud_topbar_src();
    let stripped = strip_gd_comments(&src);
    let mut first: Option<String> = None;
    for line in stripped.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        first = Some(t.to_string());
        break;
    }
    let head = first.expect("A2.1: hud_topbar.gd has no non-blank, non-comment content");
    assert_eq!(
        head, "extends Control",
        "A2.2: first non-comment, non-blank line must be `extends Control`; got `{head}`"
    );
    println!("[P13-δ A2] hud_topbar.gd extends Control (top of file) ✓");
}

// ─── Assertion 3: hud_topbar_declares_four_label_members ──────────────────
#[test]
fn harness_p13_delta_a3_hud_topbar_declares_four_label_members() {
    // Type: A — must declare four Label-typed members corresponding to
    // tick, agents, settlements, sites (case-insensitive substring).
    let src = read_hud_topbar_src();
    let stripped = strip_gd_comments(&src);

    // Collect every `var <ident> ... : Label` (or `:= Label.new()`) line.
    let mut label_idents: Vec<String> = Vec::new();
    for line in stripped.lines() {
        let t = line.trim_start();
        if !t.starts_with("var ") {
            continue;
        }
        let compact = no_ws(line);
        // Match `var IDENT:Label` or `var IDENT:Label=...` patterns.
        if !compact.contains(":Label") {
            continue;
        }
        // Make sure this is the Label type, not e.g. LabelSettings.
        // Look for `:Label` followed by end-of-string or a non-identifier
        // character. `chars().next().unwrap_or(' ')` collapses the empty
        // case to a space (which is not alphanumeric and not `_`), so a
        // single bounds check is sufficient.
        let mut is_label_typed = false;
        if let Some(pos) = compact.find(":Label") {
            let after = &compact[pos + ":Label".len()..];
            let next = after.chars().next().unwrap_or(' ');
            if !next.is_ascii_alphanumeric() && next != '_' {
                is_label_typed = true;
            }
        }
        if !is_label_typed {
            continue;
        }
        // Extract identifier: `var ` then identifier up to `:` or whitespace.
        let rest = &t[4..]; // strip "var "
        let mut ident = String::new();
        for c in rest.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c);
            } else {
                break;
            }
        }
        if !ident.is_empty() {
            label_idents.push(ident);
        }
    }

    let want = ["tick", "agent", "settlement", "site"];
    let mut missing: Vec<&str> = Vec::new();
    for w in want.iter() {
        let found = label_idents
            .iter()
            .any(|id| id.to_ascii_lowercase().contains(w));
        if !found {
            missing.push(w);
        }
    }
    assert!(
        missing.is_empty(),
        "A3: hud_topbar.gd must declare four Label members containing each of tick/agent/settlement/site \
         (case-insensitive); missing={missing:?}; found Label idents={label_idents:?}"
    );
    println!(
        "[P13-δ A3] four Label members present (idents={label_idents:?}) ✓"
    );
}

// ─── Assertion 4: hud_topbar_polls_three_snapshot_ffi_methods ─────────────
#[test]
fn harness_p13_delta_a4_hud_topbar_polls_three_snapshot_ffi_methods() {
    // Type: A — all three FFI string literals must be present.
    let stripped = strip_gd_comments(&read_hud_topbar_src());
    let needles = [
        "get_agent_snapshot",
        "get_settlement_snapshot",
        "get_construction_snapshot",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A4: hud_topbar.gd must reference all three snapshot FFI names; missing={missing:?}"
    );
    println!("[P13-δ A4] all three snapshot FFI names referenced ✓");
}

// ─── Assertion 5: hud_topbar_uses_variant_safe_ffi_pattern ────────────────
#[test]
fn harness_p13_delta_a5_hud_topbar_uses_variant_safe_ffi_pattern() {
    // Type: D — D Phase A regression guard. Untyped Variant access from
    // FFI causes warnings under treat_warnings_as_errors. Source must
    // contain both type-guard tokens at least once.
    let stripped = strip_gd_comments(&read_hud_topbar_src());
    assert!(
        stripped.contains("is Dictionary"),
        "A5.1: hud_topbar.gd must contain `is Dictionary` type guard"
    );
    assert!(
        stripped.contains("is PackedInt64Array"),
        "A5.2: hud_topbar.gd must contain `is PackedInt64Array` type guard"
    );
    println!("[P13-δ A5] Variant-safe type guards present ✓");
}

// ─── Assertion 6: hud_topbar_does_not_intercept_mouse ─────────────────────
#[test]
fn harness_p13_delta_a6_hud_topbar_does_not_intercept_mouse() {
    // Type: D — regression guard against breaking the tile-click flow.
    let stripped = strip_gd_comments(&read_hud_topbar_src());
    assert!(
        stripped.contains("MOUSE_FILTER_IGNORE"),
        "A6: hud_topbar.gd must contain `MOUSE_FILTER_IGNORE` so the overlay does not steal clicks"
    );
    println!("[P13-δ A6] MOUSE_FILTER_IGNORE present ✓");
}

// ─── Assertion 7: hud_topbar_does_not_mutate_sim_state ────────────────────
#[test]
fn harness_p13_delta_a7_hud_topbar_does_not_mutate_sim_state() {
    // Type: A — HUD is read-only. Zero mutation-style FFI calls allowed.
    // Forbidden tokens (followed by `(` after stripping whitespace):
    //   set_, spawn, command, apply_, mutate, tick
    let stripped = strip_gd_comments(&read_hud_topbar_src());

    // Allowed: theme constant API like `add_theme_constant_override(...)`.
    // We restrict the search to method-call patterns invoked on the FFI
    // node `_world_sim` or via `.call(...)`. The simplest robust check is
    // to scan for the forbidden token immediately followed by `(`, AND
    // exclude common UI-API names that legitimately use these substrings.
    let allowlist = [
        "add_theme_constant_override",
        "add_theme_color_override",
        "add_theme_font_override",
        "add_theme_font_size_override",
        "add_theme_stylebox_override",
        "set_anchors_preset",
    ];

    let forbidden = ["set_", "spawn", "command", "apply_", "mutate", "tick"];
    let mut violations: Vec<String> = Vec::new();
    for line in stripped.lines() {
        let compact_full = no_ws(line);
        for tok in forbidden.iter() {
            let needle = format!("{tok}(");
            if compact_full.contains(&needle) {
                // Filter out allowlisted call patterns.
                let mut allowed = false;
                for allow in allowlist.iter() {
                    if compact_full.contains(&format!("{allow}(")) {
                        allowed = true;
                        break;
                    }
                }
                // Also allow the internal `_tick_count += 1` (no `(`) — it
                // is identifier `_tick_count`, not `tick(`. The `tick(`
                // token requires an open-paren immediately after, so
                // `_tick_count` ≠ violation.
                if !allowed {
                    violations.push(format!("token=`{tok}(`  line=`{}`", line.trim()));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "A7: hud_topbar.gd must contain zero mutation-style FFI calls; violations:\n  {}",
        violations.join("\n  ")
    );
    println!("[P13-δ A7] no mutation-style FFI calls (HUD is read-only) ✓");
}

// ─── Assertion 8: main_tscn_registers_hud_topbar_node ─────────────────────
#[test]
fn harness_p13_delta_a8_main_tscn_registers_hud_topbar_node() {
    // Type: A — main.tscn must contain exactly one node block named
    // `HudTopbar` whose parent attribute resolves to the `UI` CanvasLayer.
    let src = read_main_tscn_src();
    let mut count = 0usize;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[node ") {
            continue;
        }
        // Look for both name="HudTopbar" and parent="UI" on the same node line.
        if t.contains("name=\"HudTopbar\"") && t.contains("parent=\"UI\"") {
            count += 1;
        }
    }
    assert_eq!(
        count, 1,
        "A8: main.tscn must contain exactly one [node name=\"HudTopbar\" ... parent=\"UI\"]; got {count}"
    );
    println!("[P13-δ A8] HudTopbar node registered under UI ✓");
}

// ─── Assertion 9: main_tscn_ext_resource_points_to_hud_script ─────────────
#[test]
fn harness_p13_delta_a9_main_tscn_ext_resource_points_to_hud_script() {
    // Type: A — main.tscn must contain exactly one ext_resource entry of
    // type Script pointing at the new hud_topbar.gd file.
    let src = read_main_tscn_src();
    let needle_path = "path=\"res://scripts/ui/panels/hud_topbar.gd\"";
    let needle_type = "type=\"Script\"";
    let mut count = 0usize;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[ext_resource") {
            continue;
        }
        if t.contains(needle_path) && t.contains(needle_type) {
            count += 1;
        }
    }
    assert_eq!(
        count, 1,
        "A9: main.tscn must contain exactly one [ext_resource type=\"Script\" path=\"res://scripts/ui/panels/hud_topbar.gd\" ...]; got {count}"
    );
    println!("[P13-δ A9] HUD ext_resource registered ✓");
}

// ─── Assertion 10: main_tscn_load_steps_bumped ────────────────────────────
#[test]
fn harness_p13_delta_a10_main_tscn_load_steps_bumped() {
    // Type: A — load_steps must be at least pre-δ baseline (5) + 1 = 6
    // since δ adds one ext_resource.
    let src = read_main_tscn_src();
    // Find the [gd_scene ... load_steps=N ...] header.
    let mut found: Option<i64> = None;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[gd_scene") {
            continue;
        }
        // Extract the load_steps integer.
        if let Some(pos) = t.find("load_steps=") {
            let after = &t[pos + "load_steps=".len()..];
            let mut digits = String::new();
            for c in after.chars() {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                found = Some(
                    digits
                        .parse::<i64>()
                        .expect("A10.1: load_steps must parse as integer"),
                );
            }
        }
        break;
    }
    let n = found.expect("A10.2: main.tscn must contain `[gd_scene ... load_steps=N ...]` header");
    const PRE_DELTA_LOAD_STEPS: i64 = 5;
    assert!(
        n > PRE_DELTA_LOAD_STEPS,
        "A10.3: load_steps must be > pre-δ ({PRE_DELTA_LOAD_STEPS}); got {n}"
    );
    println!("[P13-δ A10] load_steps = {n} (> {PRE_DELTA_LOAD_STEPS}) ✓");
}

// ─── Assertion 11: invariant_phase4_gamma_sprite_scale_unchanged ──────────
#[test]
fn harness_p13_delta_a11_phase4_gamma_sprite_scale_unchanged() {
    // Type: D — Phase 4-γ invariant. δ must not touch agent_renderer.gd.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "A11");
    let accepted = ["0.25", ".25"];
    assert!(
        accepted.contains(&rhs.as_str()),
        "A11: SPRITE_SCALE must be EXACTLY `0.25` (Phase 4-γ invariant); got `{rhs}`"
    );
    println!("[P13-δ A11] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 12: invariant_d1_state_tints_palette_unchanged ─────────────
#[test]
fn harness_p13_delta_a12_d1_state_tints_palette_unchanged() {
    // Type: D — Phase 11-α + D1 STATE_TINTS palette invariant. δ must not
    // change the four exact D1 Color literals (in order):
    //   0: Idle           — Color(0.55, 0.70, 0.95, 1.0)
    //   1: Seeking        — Color(1.0,  0.85, 0.15, 1.0)
    //   2: Consuming(Agt) — Color(1.0,  0.40, 0.75, 1.0)
    //   3: Consuming(oth) — Color(0.30, 0.95, 0.35, 1.0)
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_TINTS"),
        "A12.1: agent_renderer.gd must contain `STATE_TINTS` identifier"
    );
    let decl_pos = stripped
        .find("STATE_TINTS")
        .expect("A12.2: STATE_TINTS identifier located");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A12.3: STATE_TINTS array must open with `[`");
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
    assert_eq!(depth, 0, "A12.4: STATE_TINTS array must terminate with `]`");
    let block = &stripped[open_abs..k];
    let color_count = block.matches("Color(").count();
    assert_eq!(
        color_count, 4,
        "A12.5: STATE_TINTS block must contain EXACTLY 4 Color literals (D1 palette); got {color_count}"
    );

    // Extract each Color(...) literal in order and verify the 4 floats match
    // the D1 palette exactly (≤ 1e-6 tolerance).
    let want: [[f32; 4]; 4] = [
        [0.55, 0.70, 0.95, 1.0],
        [1.0, 0.85, 0.15, 1.0],
        [1.0, 0.40, 0.75, 1.0],
        [0.30, 0.95, 0.35, 1.0],
    ];
    let block_bytes = block.as_bytes();
    let mut cursor = 0usize;
    let mut colors: Vec<[f32; 4]> = Vec::new();
    while let Some(rel) = block[cursor..].find("Color(") {
        let open = cursor + rel + "Color(".len();
        // Find matching `)`.
        let mut close: Option<usize> = None;
        let mut j = open;
        while j < block_bytes.len() {
            if block_bytes[j] == b')' {
                close = Some(j);
                break;
            }
            j += 1;
        }
        let close_idx = close.unwrap_or_else(|| {
            panic!("A12.6: Color( at byte {open} has no matching `)` in STATE_TINTS block")
        });
        let inner = &block[open..close_idx];
        let parts: Vec<f32> = inner
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<f32>()
                    .unwrap_or_else(|_| panic!("A12.7: cannot parse Color component `{}` as f32", s.trim()))
            })
            .collect();
        assert_eq!(
            parts.len(),
            4,
            "A12.8: Color literal must have 4 components; got {n}: `{inner}`",
            n = parts.len()
        );
        colors.push([parts[0], parts[1], parts[2], parts[3]]);
        cursor = close_idx + 1;
    }
    assert_eq!(
        colors.len(),
        4,
        "A12.9: extracted {n} Color literals; expected 4",
        n = colors.len()
    );
    for (i, want_c) in want.iter().enumerate() {
        for (j, want_v) in want_c.iter().enumerate() {
            let got_v = colors[i][j];
            assert!(
                (got_v - want_v).abs() < 1e-6,
                "A12.10: STATE_TINTS[{i}] component {j} must equal {want_v}; got {got_v} \
                 (full color got=Color({a}, {b}, {c}, {d}); expected=Color({wa}, {wb}, {wc}, {wd}))",
                a = colors[i][0], b = colors[i][1], c = colors[i][2], d = colors[i][3],
                wa = want_c[0], wb = want_c[1], wc = want_c[2], wd = want_c[3],
            );
        }
    }
    println!(
        "[P13-δ A12] STATE_TINTS palette preserved (exact D1 4-color literals in order) ✓"
    );
}

// ─── Assertion 13: invariant_phase12_alpha_camera_zoom_bounds_unchanged ───
#[test]
fn harness_p13_delta_a13_phase12_alpha_camera_zoom_bounds_unchanged() {
    // Type: D — Phase 12-α invariant. δ must not touch camera_controller.gd.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A13.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A13.2");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(0.5,0.5)",
        "A13.3: ZOOM_MIN must equal Vector2(0.5, 0.5); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(4.0,4.0)",
        "A13.4: ZOOM_MAX must equal Vector2(4.0, 4.0); got `{zoom_max}`"
    );
    println!("[P13-δ A13] ZOOM_MIN(0.5,0.5) + ZOOM_MAX(4.0,4.0) preserved ✓");
}

// ─── Assertion 14: invariant_phase13_alpha_default_zoom_3x_unchanged ──────
#[test]
fn harness_p13_delta_a14_phase13_alpha_default_zoom_3x_unchanged() {
    // Type: D — Phase 13-α invariant.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A14");
    let compact = no_ws(&rhs);
    let accepted = [
        "Vector2(3.0,3.0)",
        "Vector2(3,3)",
        "Vector2(3.0,3)",
        "Vector2(3,3.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A14: ZOOM_DEFAULT must equal Vector2(3.0, 3.0) (Phase 13-α invariant); got `{rhs}`"
    );
    println!("[P13-δ A14] ZOOM_DEFAULT = Vector2(3.0, 3.0) preserved ✓");
}

// ─── Assertion 15: invariant_phase13_alpha_bootstrap_campfire_unchanged ───
#[test]
fn harness_p13_delta_a15_phase13_alpha_bootstrap_campfire_unchanged() {
    // Type: D — Phase 13-α bootstrap invariant.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "BUILDING_SPRITE_PATH", "A15");
    assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A15: BUILDING_SPRITE_PATH must remain the campfire literal (Phase 13-α invariant); got `{rhs}`"
    );
    println!("[P13-δ A15] BUILDING_SPRITE_PATH → campfire/1.png preserved ✓");
}

// ─── Assertion 16: invariant_phase13_beta_resource_constants_unchanged ────
#[test]
fn harness_p13_delta_a16_phase13_beta_resource_constants_unchanged() {
    // Type: D — Phase 13-β resource placeholder invariants.
    let stripped = strip_gd_comments(&read_world_renderer_src());

    let resource_path = unique_decl_rhs(&stripped, "RESOURCE_SPRITE_PATH", "A16.1");
    assert!(
        resource_path.contains("res://"),
        "A16.2: RESOURCE_SPRITE_PATH must be a res:// path; got `{resource_path}`"
    );

    let z_resource = unique_decl_rhs(&stripped, "Z_RESOURCE", "A16.3");
    assert_eq!(
        z_resource, "3",
        "A16.4: Z_RESOURCE must equal 3; got `{z_resource}`"
    );

    let resource_count = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A16.5");
    assert_eq!(
        resource_count, "20",
        "A16.6: RESOURCE_COUNT must equal 20; got `{resource_count}`"
    );
    println!("[P13-δ A16] Phase 13-β resource constants preserved (path/z=3/count=20) ✓");
}

// ─── Assertion 18: hud_topbar_silent_missing_world_sim ────────────────────
#[test]
fn harness_p13_delta_a18_hud_topbar_silent_missing_world_sim() {
    // Type: D — edge-case regression guard. Per the prompt: HUD instantiated
    // before WorldSim FFI registration must "short-circuit `_process` cleanly;
    // no error spam acceptable." A `push_error` or `push_warning` call in the
    // `_world_sim == null` path violates this requirement because Godot will
    // emit the message on every scene load when the autoload ordering races.
    //
    // We enforce zero `push_error(...)` / `push_warning(...)` / `printerr(...)`
    // calls anywhere in hud_topbar.gd. The HUD is a passive read-only overlay;
    // it has no legitimate need to log errors. If a future maintainer wants to
    // log something they must surface it via a non-spammy channel and update
    // this harness deliberately.
    let stripped = strip_gd_comments(&read_hud_topbar_src());
    let forbidden = ["push_error(", "push_warning(", "printerr("];
    let mut violations: Vec<String> = Vec::new();
    for line in stripped.lines() {
        let compact = no_ws(line);
        for tok in forbidden.iter() {
            if compact.contains(tok) {
                violations.push(format!("token=`{tok}`  line=`{}`", line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "A18: hud_topbar.gd must NOT call push_error/push_warning/printerr — \
         missing-WorldSim path must be silent (no log spam). violations:\n  {}",
        violations.join("\n  ")
    );
    println!("[P13-δ A18] no push_error/push_warning/printerr in hud_topbar.gd ✓");
}

// ─── Assertion 17: invariant_phase13_gamma_state_scale_boost_unchanged ────
#[test]
fn harness_p13_delta_a17_phase13_gamma_state_scale_boost_unchanged() {
    // Type: D — Phase 13-γ invariant. The four literals must remain in
    // order: 1.0, 1.15, 1.15, 1.15.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_SCALE_BOOST"),
        "A17.1: agent_renderer.gd must contain STATE_SCALE_BOOST identifier"
    );
    // Extract array body between `[` and matching `]` after the declaration.
    let decl_pos = stripped
        .find("STATE_SCALE_BOOST")
        .expect("A17.2: STATE_SCALE_BOOST located");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A17.3: STATE_SCALE_BOOST array must open with `[`");
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
        "A17.4: STATE_SCALE_BOOST array must terminate with `]`"
    );
    // Body is everything between `[` (open_abs) and the matching `]` at k-1.
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
        "A17.5: STATE_SCALE_BOOST must have exactly 4 entries; got {n}: {entries:?}",
        n = entries.len()
    );
    let want: [f32; 4] = [1.0, 1.15, 1.15, 1.15];
    for (i, want_v) in want.iter().enumerate() {
        let got: f32 = entries[i]
            .parse()
            .unwrap_or_else(|_| panic!("A17.6: entry {i} must parse as f32; got `{}`", entries[i]));
        assert!(
            (got - want_v).abs() < 1e-6,
            "A17.7: STATE_SCALE_BOOST[{i}] must equal {want_v}; got {got}"
        );
    }
    println!("[P13-δ A17] STATE_SCALE_BOOST = [1.0, 1.15, 1.15, 1.15] preserved ✓");
}
