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
use std::process::Command;

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

/// Extract the argument-list slice (everything between the opening `(` of
/// `on_building_placed(` and its matching `)`) for every textual call site
/// in the stripped source. Returns a Vec of trimmed arg-string slices, one
/// per call site, in source order.
fn extract_on_building_placed_args(stripped: &str) -> Vec<String> {
    let needle = "on_building_placed(";
    let mut out = Vec::new();
    let bytes = stripped.as_bytes();
    let mut cursor = 0usize;
    while let Some(rel) = stripped[cursor..].find(needle) {
        let open = cursor + rel + needle.len();
        // Walk forward tracking paren depth to find the matching `)`.
        let mut depth: i32 = 1;
        let mut j = open;
        while j < bytes.len() && depth > 0 {
            match bytes[j] as char {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                break;
            }
            j += 1;
        }
        assert!(
            j < bytes.len(),
            "on_building_placed( at byte {open} has no matching `)`"
        );
        let args = stripped[open..j].trim().to_string();
        out.push(args);
        cursor = j + 1;
    }
    out
}

/// Parse a comma-separated arg list (depth-aware, so nested `Vector2(a, b)`
/// is treated as one arg). Returns trimmed arg tokens.
fn split_top_level_args(args: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    for c in args.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                out.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let last = current.trim().to_string();
    if !last.is_empty() {
        out.push(last);
    }
    out
}

/// Extract the body lines of a `for extra_x in [BOOTSTRAP_X_LEFT, BOOTSTRAP_X_RIGHT]:`
/// loop in the stripped source. Returns the concatenated body (lines that
/// are more deeply indented than the `for` line), or `None` if the loop is
/// not present.
fn extract_extra_x_loop_body(stripped: &str) -> Option<String> {
    let lines: Vec<&str> = stripped.lines().collect();
    let mut for_idx: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate() {
        let compact = no_ws(line);
        // Accept the canonical literal forms; tolerate either order of constants.
        if compact.contains("forextra_xin[BOOTSTRAP_X_LEFT,BOOTSTRAP_X_RIGHT]:")
            || compact.contains("forextra_xin[BOOTSTRAP_X_RIGHT,BOOTSTRAP_X_LEFT]:")
        {
            for_idx = Some(idx);
            break;
        }
    }
    let idx = for_idx?;
    let for_line = lines[idx];
    // Indentation of the for header (count leading whitespace chars).
    let for_indent = for_line.len() - for_line.trim_start().len();
    let mut body = String::new();
    for body_line in &lines[idx + 1..] {
        // Stop at the first non-blank line whose indentation is ≤ for_indent.
        if body_line.trim().is_empty() {
            body.push_str(body_line);
            body.push('\n');
            continue;
        }
        let li = body_line.len() - body_line.trim_start().len();
        if li <= for_indent {
            break;
        }
        body.push_str(body_line);
        body.push('\n');
    }
    Some(body)
}

// ─── Assertion 1: bootstrap_x_left_constant_declared ─────────────────────
#[test]
fn harness_p13_epsilon_a1_bootstrap_x_left_constant_declared() {
    // Type: A — feature spec mandates literal value 24 for the left flank.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "BOOTSTRAP_X_LEFT", "A1");
    assert_eq!(
        rhs, "24",
        "A1: BOOTSTRAP_X_LEFT RHS must be EXACTLY `24`; got `{rhs}`"
    );
    println!("[P13-ε A1] BOOTSTRAP_X_LEFT = 24 ✓");
}

// ─── Assertion 2: bootstrap_x_right_constant_declared ────────────────────
#[test]
fn harness_p13_epsilon_a2_bootstrap_x_right_constant_declared() {
    // Type: A — feature spec mandates literal value 40 for the right flank.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "BOOTSTRAP_X_RIGHT", "A2");
    assert_eq!(
        rhs, "40",
        "A2: BOOTSTRAP_X_RIGHT RHS must be EXACTLY `40`; got `{rhs}`"
    );
    println!("[P13-ε A2] BOOTSTRAP_X_RIGHT = 40 ✓");
}

// ─── Assertion 3: three_on_building_placed_calls_total ───────────────────
#[test]
fn harness_p13_epsilon_a3_three_on_building_placed_calls_total() {
    // Type: A — exactly 3 call sites (centre + 2 flanks).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let calls = extract_on_building_placed_args(&stripped);
    assert_eq!(
        calls.len(),
        3,
        "A3: stripped world_renderer.gd must contain EXACTLY 3 `on_building_placed(` \
         call sites; got {n}: args={calls:?}",
        n = calls.len()
    );
    println!("[P13-ε A3] exactly 3 `on_building_placed(` call sites ✓");
}

// ─── Assertion 4: on_building_placed_arg_signatures ──────────────────────
#[test]
fn harness_p13_epsilon_a4_on_building_placed_arg_signatures() {
    // Type: A — the three call sites must use these exact arg tuples
    // (any order): centre/left/right with shared BOOTSTRAP_Y + BOOTSTRAP_RADIUS.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let calls = extract_on_building_placed_args(&stripped);
    assert_eq!(
        calls.len(),
        3,
        "A4.0: precondition — must have 3 calls (covered by A3)"
    );

    let want: Vec<Vec<&str>> = vec![
        vec!["BOOTSTRAP_X", "BOOTSTRAP_Y", "BOOTSTRAP_RADIUS"],
        vec!["BOOTSTRAP_X_LEFT", "BOOTSTRAP_Y", "BOOTSTRAP_RADIUS"],
        vec!["BOOTSTRAP_X_RIGHT", "BOOTSTRAP_Y", "BOOTSTRAP_RADIUS"],
    ];
    let mut matched = [false; 3];
    let mut observed: Vec<Vec<String>> = Vec::new();
    for args in calls.iter() {
        let toks = split_top_level_args(args);
        observed.push(toks.clone());
        if toks.len() != 3 {
            continue;
        }
        for (i, want_tuple) in want.iter().enumerate() {
            if matched[i] {
                continue;
            }
            if toks[0] == want_tuple[0]
                && toks[1] == want_tuple[1]
                && toks[2] == want_tuple[2]
            {
                matched[i] = true;
                break;
            }
        }
    }
    for (i, ok) in matched.iter().enumerate() {
        assert!(
            *ok,
            "A4: expected on_building_placed tuple {tuple:?} not found among observed call args: {observed:?}",
            tuple = want[i]
        );
    }
    println!("[P13-ε A4] all three on_building_placed(...) arg tuples present (centre + left + right) ✓");
}

// ─── Assertion 5: extra_sprite_block_reuses_building_tex ─────────────────
#[test]
fn harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex() {
    // Type: A — must (a) loop exactly over BOOTSTRAP_X_LEFT and
    // BOOTSTRAP_X_RIGHT, AND (b) assign `.texture = building_tex` inside.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = extract_extra_x_loop_body(&stripped).expect(
        "A5.1: stripped source must contain a `for extra_x in [BOOTSTRAP_X_LEFT, \
         BOOTSTRAP_X_RIGHT]:` loop (whitespace-tolerant)",
    );
    let compact = no_ws(&body);
    assert!(
        compact.contains(".texture=building_tex"),
        "A5.2: extra_x loop body must contain `.texture = building_tex` (reuse, no new \
         load); body=\n{body}"
    );
    // Reinforce: no new `load(` for a building/campfire texture inside the loop.
    assert!(
        !compact.contains("load(\"res://assets/sprites/buildings/campfire/"),
        "A5.3: extra_x loop body must NOT reload the campfire asset; reuse the \
         existing `building_tex` variable. body=\n{body}"
    );
    println!("[P13-ε A5] for-loop iterates flanks AND reuses building_tex ✓");
}

// ─── Assertion 6: extra_sprite_z_index_is_z_building ─────────────────────
#[test]
fn harness_p13_epsilon_a6_extra_sprite_z_index_is_z_building() {
    // Type: A — loop body must contain `.z_index = Z_BUILDING`.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = extract_extra_x_loop_body(&stripped).expect("A6.1: extra_x loop body must exist (covered by A5)");
    let compact = no_ws(&body);
    assert!(
        compact.contains(".z_index=Z_BUILDING"),
        "A6.2: extra_x loop body must contain `.z_index = Z_BUILDING` (Phase 12-β.1 \
         canonical building layer constant); body=\n{body}"
    );
    println!("[P13-ε A6] extra_sprite.z_index = Z_BUILDING ✓");
}

// ─── Assertion 7: extra_sprite_y_position_uses_bootstrap_y ───────────────
#[test]
fn harness_p13_epsilon_a7_extra_sprite_y_position_uses_bootstrap_y() {
    // Type: A — loop body's position expression must reference BOOTSTRAP_Y.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = extract_extra_x_loop_body(&stripped).expect("A7.1: extra_x loop body must exist (covered by A5)");
    assert!(
        body.contains("BOOTSTRAP_Y"),
        "A7.2: extra_x loop body must reference `BOOTSTRAP_Y` in the Vector2 y \
         component so all three buildings share the same horizontal row; body=\n{body}"
    );
    println!("[P13-ε A7] BOOTSTRAP_Y referenced in flank sprite Y position ✓");
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
        "Vector2(3.0,3.0)",
        "Vector2(3,3)",
        "Vector2(3.0,3)",
        "Vector2(3,3.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(3.0, 3.0); got `{rhs}`"
    );
    println!("[P13-ε A11] ZOOM_DEFAULT = Vector2(3.0, 3.0) preserved ✓");
}

// ─── Assertion 12: phase13_alpha_centre_bootstrap_preserved ──────────────
#[test]
fn harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved() {
    // Type: D — Phase 13-α centre bootstrap invariant. ε adds flanks at ±8
    // from this centre; any drift breaks cluster geometry.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let x = unique_decl_rhs(&stripped, "BOOTSTRAP_X", "A12.1");
    let y = unique_decl_rhs(&stripped, "BOOTSTRAP_Y", "A12.2");
    let r = unique_decl_rhs(&stripped, "BOOTSTRAP_RADIUS", "A12.3");
    assert_eq!(x, "32", "A12.4: BOOTSTRAP_X must be `32`; got `{x}`");
    assert_eq!(y, "32", "A12.5: BOOTSTRAP_Y must be `32`; got `{y}`");
    assert_eq!(r, "8", "A12.6: BOOTSTRAP_RADIUS must be `8`; got `{r}`");
    println!("[P13-ε A12] centre BOOTSTRAP_X/Y/RADIUS = 32/32/8 preserved ✓");
}

// ─── Assertion 13: phase13_beta_resource_constants_preserved ─────────────
#[test]
fn harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved() {
    // Type: D — Phase 13-β resource placeholder invariants.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let path = unique_decl_rhs(&stripped, "RESOURCE_SPRITE_PATH", "A13.1");
    assert_eq!(
        path, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A13.2: RESOURCE_SPRITE_PATH must remain storage_pit literal; got `{path}`"
    );
    let z = unique_decl_rhs(&stripped, "Z_RESOURCE", "A13.3");
    assert_eq!(z, "3", "A13.4: Z_RESOURCE must equal 3; got `{z}`");
    let count = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A13.5");
    assert_eq!(count, "20", "A13.6: RESOURCE_COUNT must equal 20; got `{count}`");
    let seed = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A13.7");
    assert_eq!(
        seed, "88675123",
        "A13.8: RESOURCE_SEED must equal 88675123; got `{seed}`"
    );
    println!("[P13-ε A13] Phase 13-β resource constants (path/z=3/count=20/seed) preserved ✓");
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

// ─── Assertion 16: no_rust_simulation_crate_modification ─────────────────
#[test]
fn harness_p13_epsilon_a16_no_rust_simulation_crate_modification() {
    // Type: A — lane-discipline guard. `--quick` lane forbids edits to
    // sim-core / sim-bridge / sim-systems / sim-engine. We check the local
    // working-tree state via `git status --porcelain` and assert no
    // tracked-or-untracked paths under these crates appear. The check is
    // best-effort: if `git` is unavailable in the test environment, we
    // print a notice and skip the assertion (pipeline runs this guard at
    // the pre-commit hook layer as well).
    //
    // V7 Phase 14-γ amendment (2026-05-26): when `HARNESS_LANE=full` is
    // set in the environment, the active pipeline run is a legitimate
    // `--full` lane feature that may touch sim-bridge (the only
    // simulation crate exposed to GDScript). Skip the guard in that
    // case — the lane choice itself authorises the sim-bridge edit, and
    // the pipeline's Evaluator step independently reviews the change.
    if std::env::var("HARNESS_LANE").as_deref() == Ok("full") {
        println!("[P13-ε A16] HARNESS_LANE=full active — lane-discipline guard skipped");
        return;
    }
    let root = project_root();
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output();
    let Ok(out) = out else {
        println!("[P13-ε A16] git unavailable in test env — skipped (pipeline still enforces)");
        return;
    };
    if !out.status.success() {
        println!("[P13-ε A16] git status failed (status={:?}) — skipped", out.status.code());
        return;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let forbidden_prefixes = [
        "rust/crates/sim-core/",
        "rust/crates/sim-bridge/",
        "rust/crates/sim-systems/",
        "rust/crates/sim-engine/",
    ];
    let mut offenders: Vec<String> = Vec::new();
    for line in stdout.lines() {
        // Porcelain v1 format: `XY <path>` (path begins at byte 3).
        if line.len() < 4 {
            continue;
        }
        let path = line[3..].trim();
        // Handle rename: `R  old -> new` form.
        let candidate = if let Some(arrow) = path.find(" -> ") {
            &path[arrow + 4..]
        } else {
            path
        };
        for prefix in forbidden_prefixes.iter() {
            if candidate.starts_with(prefix) {
                offenders.push(line.to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "A16: --quick lane forbids modifications under sim-core/sim-bridge/sim-systems/sim-engine; \
         offending git status entries:\n  {}",
        offenders.join("\n  ")
    );
    println!("[P13-ε A16] no rust simulation crate modifications in working tree ✓");
}
