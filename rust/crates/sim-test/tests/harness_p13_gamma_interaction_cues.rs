//! V7 Phase 13-γ — Per-State Interaction Scale Cue harness.
//!
//! Static file-inspection harness verifying the Phase 13-γ implementation:
//!   - `scripts/ui/agent_renderer.gd` — adds `STATE_SCALE_BOOST: Array` const
//!     and wires it into the existing boost cascade via `max()`.
//!   - Anti-regression guards for Phase 4-γ, Phase 8-δ, Phase 9-δ,
//!     Phase 11-α + D1, Phase 12-α, Phase 13-α, Phase 13-β invariants.
//!
//! Meta-assertions A17 (workspace gate) and A18 (GDScript parse) are
//! verified by the pipeline runner outside this test file (see Section 5
//! of the feature prompt for the exact commands).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p13_gamma_interaction_cues -- --nocapture

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

/// Locate the file-level `STATE_SCALE_BOOST` declaration line, verify it is
/// outside any `func`/`class_name` body, and return the bracketed-literal
/// content (between `[` and the matching `]`).
fn extract_state_scale_boost_literal(stripped: &str) -> String {
    // Find the declaration line.
    let mut decl_line_idx: Option<usize> = None;
    let mut in_func = false;
    let lines: Vec<&str> = stripped.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        // Track func boundaries — any non-blank, non-indented line that starts
        // with `func ` opens a function body; the body extends until the next
        // non-blank, non-indented line that does NOT continue that body.
        // Simpler heuristic: a file-level `const` is at indent 0 with no
        // leading whitespace and starts with `const `.
        if line.starts_with("func ") {
            in_func = true;
        }
        // A line at indent 0 that's not a continuation indicates re-entry to
        // file scope. `const NAME` at indent 0 is what we look for.
        let starts_at_col0 = !line.starts_with(' ') && !line.starts_with('\t');
        if starts_at_col0 && !line.starts_with("func ") && !line.is_empty() {
            // We're back at file scope (or a class header etc.).
            if !line.starts_with("func ") {
                in_func = false;
            }
        }
        if starts_at_col0 && t.starts_with("const STATE_SCALE_BOOST") {
            // Word-boundary check: next char after the identifier must not be
            // alphanumeric/_ .
            let after = &t["const STATE_SCALE_BOOST".len()..];
            let bnd = after.chars().next().unwrap_or(' ');
            if !bnd.is_ascii_alphanumeric() && bnd != '_' {
                assert!(
                    !in_func,
                    "STATE_SCALE_BOOST must be declared at file scope, not inside a func body"
                );
                decl_line_idx = Some(idx);
                break;
            }
        }
    }
    let idx = decl_line_idx
        .expect("STATE_SCALE_BOOST: file-level `const` declaration not found in agent_renderer.gd");
    let line = lines[idx];
    // Verify the type annotation is `Array`.
    let compact = no_ws(line);
    assert!(
        compact.contains("STATE_SCALE_BOOST:Array=["),
        "STATE_SCALE_BOOST: declaration must annotate type as `Array` with bracketed literal; \
         compact line=`{compact}`"
    );
    // Extract bracketed literal — may extend across lines if the declaration
    // is multi-line. Walk forward tracking bracket depth.
    let open_rel = line
        .find('[')
        .expect("STATE_SCALE_BOOST: opening `[` must exist on declaration line");
    let mut buf = String::new();
    buf.push_str(&line[open_rel + 1..]);
    buf.push('\n');
    let mut depth: i32 = 1;
    // Count brackets in initial slice after `[`.
    for c in line[open_rel + 1..].chars() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
    }
    let mut j = idx + 1;
    while depth > 0 && j < lines.len() {
        for c in lines[j].chars() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                _ => {}
            }
        }
        buf.push_str(lines[j]);
        buf.push('\n');
        j += 1;
    }
    assert!(
        depth <= 0,
        "STATE_SCALE_BOOST: array literal must terminate with matching `]`"
    );
    // Trim everything after the matching `]`.
    let close_pos = buf.rfind(']').expect("STATE_SCALE_BOOST: `]` must be present");
    buf[..close_pos].to_string()
}

/// Parse the comma-separated entries of a `STATE_SCALE_BOOST` literal body
/// (i.e. everything between `[` and `]`, exclusive). Returns each trimmed
/// entry as a string. Strips trailing empty entries from a final trailing
/// comma if present.
fn parse_array_entries(body: &str) -> Vec<String> {
    let cleaned: String = body.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    let mut entries: Vec<String> = cleaned
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    while entries.last().is_some_and(|s| s.is_empty()) {
        entries.pop();
    }
    entries
}

// ─── Assertion 1: state_scale_boost_array_declared ────────────────────────
#[test]
fn harness_p13_gamma_a1_state_scale_boost_array_declared() {
    // Type: A — Spec mandates a new file-level `const STATE_SCALE_BOOST:
    // Array = [...]` declaration. Strict-shape match prevents `var` /
    // function-scoped placements.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_state_scale_boost_literal(&stripped);
    // Body extraction itself enforces the declaration is file-scope `const`
    // with type `Array` and bracketed literal.
    let entries = parse_array_entries(&body);
    assert!(
        !entries.is_empty(),
        "A1: STATE_SCALE_BOOST literal must contain at least one entry; body=`{body}`"
    );
    println!(
        "[P13-γ A1] file-level `const STATE_SCALE_BOOST: Array = [...]` declared ({n} entries) ✓",
        n = entries.len()
    );
}

// ─── Assertion 2: state_scale_boost_idle_index_is_baseline ────────────────
#[test]
fn harness_p13_gamma_a2_state_scale_boost_idle_index_is_baseline() {
    // Type: A — Idle (state_tag 0) renders at baseline 1.0 (Phase 4-γ
    // SPRITE_SCALE tile-fit invariant preserved).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_state_scale_boost_literal(&stripped);
    let entries = parse_array_entries(&body);
    assert!(
        !entries.is_empty(),
        "A2: STATE_SCALE_BOOST has no entries; body=`{body}`"
    );
    let v: f32 = entries[0]
        .parse()
        .unwrap_or_else(|_| panic!("A2: index 0 must parse as f32; got `{}`", entries[0]));
    assert!(
        (v - 1.0).abs() < f32::EPSILON,
        "A2: STATE_SCALE_BOOST[0] (Idle) must be EXACTLY 1.0; got {v}"
    );
    println!("[P13-γ A2] STATE_SCALE_BOOST[0] = 1.0 (Idle baseline) ✓");
}

// ─── Assertion 3: state_scale_boost_active_indices_are_1_15 ──────────────
#[test]
fn harness_p13_gamma_a3_state_scale_boost_active_indices_are_1_15() {
    // Type: A — Indices 1 (Seeking), 2 (Consuming(Agent)), 3 (Consuming
    // other) all == 1.15. Exact equality prevents rounding drift.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_state_scale_boost_literal(&stripped);
    let entries = parse_array_entries(&body);
    assert!(
        entries.len() >= 4,
        "A3: STATE_SCALE_BOOST must have ≥4 entries; got {n}",
        n = entries.len()
    );
    for (i, entry) in entries.iter().enumerate().take(4).skip(1) {
        let v: f32 = entry
            .parse()
            .unwrap_or_else(|_| panic!("A3: index {i} must parse as f32; got `{entry}`"));
        assert!(
            (v - 1.15).abs() < 1e-6,
            "A3: STATE_SCALE_BOOST[{i}] must be EXACTLY 1.15; got {v}"
        );
    }
    println!("[P13-γ A3] STATE_SCALE_BOOST[1,2,3] all = 1.15 ✓");
}

// ─── Assertion 4: state_scale_boost_array_length_is_four ─────────────────
#[test]
fn harness_p13_gamma_a4_state_scale_boost_array_length_is_four() {
    // Type: A — state_tag domain is {0,1,2,3} per FFI. Must be exactly 4
    // entries (fewer → out-of-bounds; more → unused tail / wrong indexing).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_state_scale_boost_literal(&stripped);
    let entries = parse_array_entries(&body);
    assert_eq!(
        entries.len(),
        4,
        "A4: STATE_SCALE_BOOST must have EXACTLY 4 entries (state_tag domain); got {n}: {entries:?}",
        n = entries.len()
    );
    println!("[P13-γ A4] STATE_SCALE_BOOST length = 4 ✓");
}

// ─── Assertion 5: state_scale_boost_used_in_render_loop ──────────────────
#[test]
fn harness_p13_gamma_a5_state_scale_boost_used_in_render_loop() {
    // Type: A — Constant must be referenced in the boost cascade. Catches
    // declaration without wiring. Two substrings:
    //   1. `STATE_SCALE_BOOST[` (indexed access)
    //   2. `max(boost,` containing the lookup (composition into cascade)
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let compact = no_ws(&stripped);
    assert!(
        compact.contains("STATE_SCALE_BOOST["),
        "A5.1: stripped source must contain `STATE_SCALE_BOOST[` indexed access"
    );
    // Find a line that contains both `max(boost,` and `STATE_SCALE_BOOST[`.
    let mut found = false;
    for line in stripped.lines() {
        let line_compact = no_ws(line);
        if line_compact.contains("max(boost,")
            && line_compact.contains("STATE_SCALE_BOOST[")
        {
            found = true;
            break;
        }
        // Allow split: assignment line `boost = max(boost, float(STATE_SCALE_BOOST[...]))`
        // is already on a single line in the spec.
    }
    assert!(
        found,
        "A5.2: stripped source must contain a `max(boost, …)` call whose argument indexes \
         `STATE_SCALE_BOOST[…]` on the same line (boost-cascade composition)"
    );
    println!("[P13-γ A5] STATE_SCALE_BOOST referenced in boost cascade via max() ✓");
}

// ─── Assertion 6: recall_cue_scale_boost_preserved ───────────────────────
#[test]
fn harness_p13_gamma_a6_recall_cue_scale_boost_preserved() {
    // Type: D — Phase 8-δ regression guard. Event-driven cue must still win.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "RECALL_CUE_SCALE_BOOST", "A6");
    let v: f32 = rhs
        .parse()
        .unwrap_or_else(|_| panic!("A6: RECALL_CUE_SCALE_BOOST must parse as f32; got `{rhs}`"));
    assert!(
        (v - 1.25).abs() < f32::EPSILON,
        "A6: RECALL_CUE_SCALE_BOOST must be EXACTLY 1.25 (Phase 8-δ invariant); got {v}"
    );
    println!("[P13-γ A6] RECALL_CUE_SCALE_BOOST = 1.25 preserved ✓");
}

// ─── Assertion 7: combat_cue_scale_boost_preserved ───────────────────────
#[test]
fn harness_p13_gamma_a7_combat_cue_scale_boost_preserved() {
    // Type: D — Phase 9-δ regression guard.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "COMBAT_CUE_SCALE_BOOST", "A7");
    let v: f32 = rhs
        .parse()
        .unwrap_or_else(|_| panic!("A7: COMBAT_CUE_SCALE_BOOST must parse as f32; got `{rhs}`"));
    assert!(
        (v - 1.3).abs() < 1e-6,
        "A7: COMBAT_CUE_SCALE_BOOST must be EXACTLY 1.3 (Phase 9-δ invariant); got {v}"
    );
    println!("[P13-γ A7] COMBAT_CUE_SCALE_BOOST = 1.3 preserved ✓");
}

// ─── Assertion 8: boost_cascade_uses_max_composition ─────────────────────
#[test]
fn harness_p13_gamma_a8_boost_cascade_uses_max_composition() {
    // Type: A — Spec mandates max() composition for state + recall + combat
    // (≥3 occurrences). Fewer means one cue path is missing or replaced with
    // assignment (which would let γ's modest 1.15 override the larger event
    // cues — wrong direction).
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);
    // Count occurrences of the canonical pattern `boost = max(boost,`.
    // Whitespace-tolerant count: collapse spaces around `=` and after `(`.
    let needle = "boost=max(boost,";
    let mut count = 0usize;
    for line in stripped.lines() {
        let line_compact = no_ws(line);
        // Each line is one statement in GDScript, count one occurrence per line.
        if line_compact.contains(needle) {
            count += 1;
        }
    }
    assert!(
        count >= 3,
        "A8: stripped agent_renderer.gd must contain ≥3 `boost = max(boost, …)` lines \
         (state + recall + combat); got {count}"
    );
    println!("[P13-γ A8] `boost = max(boost, …)` occurrences = {count} (≥3) ✓");
}

// ─── Assertion 9: state_tag_clamp_uses_zero_three_bounds ─────────────────
#[test]
fn harness_p13_gamma_a9_state_tag_clamp_uses_zero_three_bounds() {
    // Type: A — Exactly one line in the source must contain a clampi(…, 0, 3)
    // call AND be the line that produces the boost-lookup index (either the
    // line that assigns `state_tag_for_boost`, or the line that directly
    // indexes `STATE_SCALE_BOOST[`). Pre-existing STATE_TINTS clamps are
    // excluded from the count because they reference neither
    // `state_tag_for_boost` nor `STATE_SCALE_BOOST`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let mut matches: Vec<String> = Vec::new();
    for line in stripped.lines() {
        let compact = no_ws(line);
        if !(compact.contains("clampi(") && compact.contains(",0,3)")) {
            continue;
        }
        // Must be the boost-lookup line (not the STATE_TINTS lookup).
        if compact.contains("state_tag_for_boost") || compact.contains("STATE_SCALE_BOOST[") {
            matches.push(line.to_string());
        }
    }
    assert_eq!(
        matches.len(),
        1,
        "A9: must have EXACTLY 1 boost-index clamp line containing `clampi(…, 0, 3)` AND \
         referencing `state_tag_for_boost` or `STATE_SCALE_BOOST[`; got {n}: {matches:?}",
        n = matches.len()
    );
    println!("[P13-γ A9] clampi(…, 0, 3) bounds applied to state-boost index (1 line) ✓");
}

// ─── Assertion 10: sprite_scale_baseline_preserved ───────────────────────
#[test]
fn harness_p13_gamma_a10_sprite_scale_baseline_preserved() {
    // Type: D — Phase 4-γ tile-fit invariant. γ must not retune baseline.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "A10");
    let accepted = ["0.25", ".25"];
    assert!(
        accepted.contains(&rhs.as_str()),
        "A10: SPRITE_SCALE must be EXACTLY `0.25` or `.25` (Phase 4-γ invariant); got `{rhs}`"
    );
    println!("[P13-γ A10] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 11: state_tints_palette_preserved ─────────────────────────
#[test]
fn harness_p13_gamma_a11_state_tints_palette_preserved() {
    // Type: D — Phase 11-α + D1 invariant. γ does not own the palette.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_TINTS"),
        "A11.1: agent_renderer.gd must contain `STATE_TINTS` identifier"
    );
    let decl_pos = stripped
        .find("STATE_TINTS")
        .expect("A11.2: STATE_TINTS identifier located");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A11.3: STATE_TINTS array must open with `[`");
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
        "A11.4: STATE_TINTS array must terminate with a matching `]`"
    );
    let block = &stripped[open_abs..k];
    // Count `Color(` literals inside the block.
    let color_count = block.matches("Color(").count();
    assert_eq!(
        color_count, 4,
        "A11.5: STATE_TINTS block must contain EXACTLY 4 `Color(...)` literals (D1 palette); got {color_count}"
    );
    println!("[P13-γ A11] STATE_TINTS palette preserved (4 Color literals) ✓");
}

// ─── Assertion 12: phase12_alpha_zoom_min_max_preserved ──────────────────
#[test]
fn harness_p13_gamma_a12_phase12_alpha_zoom_min_max_preserved() {
    // Type: D — Phase 12-α invariant.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A12.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A12.2");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(0.5,0.5)",
        "A12.3: ZOOM_MIN must equal Vector2(0.5, 0.5); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(8.0,8.0)",
        "A12.4: ZOOM_MAX must equal Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zoom_max}`"
    );
    println!("[P13-γ A12] ZOOM_MIN(0.5,0.5) + ZOOM_MAX(8.0,8.0) (G Phase A) ✓");
}

// ─── Assertion 13: phase13_alpha_zoom_default_3x_preserved ───────────────
#[test]
fn harness_p13_gamma_a13_phase13_alpha_zoom_default_3x_preserved() {
    // Type: D — Phase 13-α invariant. γ's expected visible delta assumes 3.0×.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A13");
    let compact = no_ws(&rhs);
    let accepted = [
        "Vector2(3.0,3.0)",
        "Vector2(3,3)",
        "Vector2(3.0,3)",
        "Vector2(3,3.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A13: ZOOM_DEFAULT must equal Vector2(3.0, 3.0) (Phase 13-α invariant); got `{rhs}`"
    );
    println!("[P13-γ A13] ZOOM_DEFAULT = Vector2(3.0, 3.0) preserved ✓");
}

// ─── Assertion 14: phase13_alpha_bootstrap_campfire_preserved ────────────
#[test]
fn harness_p13_gamma_a14_phase13_alpha_bootstrap_campfire_preserved() {
    // Type: D — Phase 13-α cross-phase regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "BUILDING_SPRITE_PATH", "A14");
    assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A14: BUILDING_SPRITE_PATH must remain the campfire literal (Phase 13-α invariant); got `{rhs}`"
    );
    println!("[P13-γ A14] BUILDING_SPRITE_PATH → campfire/1.png preserved ✓");
}

// ─── Assertion 15: phase13_beta_resource_constants_preserved ─────────────
#[test]
fn harness_p13_gamma_a15_phase13_beta_resource_constants_preserved() {
    // Type: D — Phase 13-β cross-phase regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());

    let resource_path = unique_decl_rhs(&stripped, "RESOURCE_SPRITE_PATH", "A15.1");
    assert_eq!(
        resource_path, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A15.2: RESOURCE_SPRITE_PATH must remain the storage_pit literal; got `{resource_path}`"
    );

    let z_resource = unique_decl_rhs(&stripped, "Z_RESOURCE", "A15.3");
    assert_eq!(z_resource, "3", "A15.4: Z_RESOURCE must equal 3; got `{z_resource}`");

    let resource_count = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A15.5");
    assert_eq!(
        resource_count, "20",
        "A15.6: RESOURCE_COUNT must equal 20; got `{resource_count}`"
    );

    let resource_seed = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A15.7");
    assert_eq!(
        resource_seed, "88675123",
        "A15.8: RESOURCE_SEED must equal 88675123; got `{resource_seed}`"
    );
    println!("[P13-γ A15] Phase 13-β resource constants preserved (path/z/count/seed) ✓");
}

// ─── Assertion 16: state_scale_boost_lookup_guards_bounds ────────────────
#[test]
fn harness_p13_gamma_a16_state_scale_boost_lookup_guards_bounds() {
    // Type: A — The boost-lookup line must guard against `i >= states.size()`
    // by defaulting to 0 (Idle baseline). Without the guard the renderer
    // crashes; with the wrong fallback (e.g. 1=Seeking) every uninitialised
    // agent renders boosted.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let mut hit = false;
    for line in stripped.lines() {
        // Scope: only the boost-lookup line (the one that assigns
        // `state_tag_for_boost` or directly indexes `STATE_SCALE_BOOST[`).
        // The pre-existing STATE_TINTS lookup line has its own
        // `if i < states.size() else 0` guard and must not be allowed to
        // satisfy A16 on its own.
        if !(line.contains("state_tag_for_boost") || line.contains("STATE_SCALE_BOOST[")) {
            continue;
        }
        let compact = no_ws(line);
        if compact.contains("i<states.size()")
            && compact.contains("else0")
            && compact.contains("states[i]")
        {
            hit = true;
            break;
        }
    }
    assert!(
        hit,
        "A16: state-boost lookup line must guard `states[i]` with \
         `if i < states.size() else 0` (fallback to Idle baseline) — \
         pre-existing STATE_TINTS lookup line does not count"
    );
    println!("[P13-γ A16] index-guard pattern on boost-lookup line ✓");
}
