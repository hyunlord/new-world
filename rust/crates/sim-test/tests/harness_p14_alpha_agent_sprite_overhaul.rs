//! V7 Phase 14-α — Agent Sprite Overhaul (Per-Role HUE + Head-Icon Mount Point) harness.
//!
//! Static file-inspection harness verifying the Phase 14-α implementation:
//!   - `scripts/ui/agent_renderer.gd` — adds ROLE_BUCKET_COUNT, ICON_OFFSET_PX,
//!     ICON_SIZE_PX constants, a `_role_bucket()` helper, extends
//!     `_palette_for_id` signature to (eid, agent_id), updates the
//!     `set_instance_custom_data` call site, adds `head_icon_position()` helper.
//!   - Anti-regression guards for Phase 4-γ, Phase 8-δ, Phase 9-δ,
//!     Phase 11-α + D1, Phase 12-α, Phase 13-α/β/γ/ε invariants.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_alpha_agent_sprite_overhaul -- --nocapture

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

/// Extract the body of a top-level `func <name>(...)` in `stripped` source.
/// Returns the concatenated body lines (all lines more indented than the
/// `func` header line), or `None` if not found.
fn extract_func_body(stripped: &str, func_signature_prefix: &str) -> Option<String> {
    let lines: Vec<&str> = stripped.lines().collect();
    let mut header_idx: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with(func_signature_prefix) {
            header_idx = Some(idx);
            break;
        }
    }
    let idx = header_idx?;
    let header = lines[idx];
    let header_indent = header.len() - header.trim_start().len();
    let mut body = String::new();
    for body_line in &lines[idx + 1..] {
        if body_line.trim().is_empty() {
            body.push_str(body_line);
            body.push('\n');
            continue;
        }
        let li = body_line.len() - body_line.trim_start().len();
        if li <= header_indent {
            break;
        }
        body.push_str(body_line);
        body.push('\n');
    }
    Some(body)
}

// ─── Assertion 1: role_bucket_count_constant_declared ─────────────────────
#[test]
fn harness_p14_alpha_a1_role_bucket_count_declared() {
    // Type: A — Spec mandates ROLE_BUCKET_COUNT = 4 (matches PALETTE_BODY_COLS).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "ROLE_BUCKET_COUNT", "A1");
    assert_eq!(
        rhs, "4",
        "A1: ROLE_BUCKET_COUNT must be declared exactly once with value 4; got `{rhs}`"
    );
    println!("[P14-α A1] ROLE_BUCKET_COUNT = 4 ✓");
}

// ─── Assertion 2: role_bucket_function_signature_present ──────────────────
#[test]
fn harness_p14_alpha_a2_role_bucket_function_signature_present() {
    // Type: A — Spec mandates `func _role_bucket(agent_id: int) -> int`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let needle = "func _role_bucket(agent_id: int) -> int";
    let count = stripped.matches(needle).count();
    assert_eq!(
        count, 1,
        "A2: signature `{needle}` must appear exactly once in stripped source; got {count}"
    );
    println!("[P14-α A2] `func _role_bucket(agent_id: int) -> int` declared ✓");
}

// ─── Assertion 3: role_bucket_body_uses_agent_id_knuth_hash ───────────────
#[test]
fn harness_p14_alpha_a3_role_bucket_body_uses_agent_id_knuth_hash() {
    // Type: A — Body must contain `absi(agent_id * 2654435761) % ROLE_BUCKET_COUNT`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_func_body(&stripped, "func _role_bucket(")
        .expect("A3.1: _role_bucket function body must exist");
    let compact = no_ws(&body);
    let needle = no_ws("absi(agent_id * 2654435761) % ROLE_BUCKET_COUNT");
    assert!(
        compact.contains(&needle),
        "A3.2: _role_bucket body must contain `absi(agent_id * 2654435761) % ROLE_BUCKET_COUNT`; \
         body=\n{body}"
    );
    println!("[P14-α A3] _role_bucket uses Knuth hash on agent_id ✓");
}

// ─── Assertion 4: palette_for_id_signature_extended ───────────────────────
#[test]
fn harness_p14_alpha_a4_palette_for_id_signature_extended() {
    // Type: A — Signature must be `_palette_for_id(eid: int, agent_id: int) -> Color`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let new_sig = "func _palette_for_id(eid: int, agent_id: int) -> Color";
    let new_count = stripped.matches(new_sig).count();
    assert_eq!(
        new_count, 1,
        "A4.1: new signature `{new_sig}` must appear exactly once; got {new_count}"
    );
    // The old single-arg form must NOT remain.
    let old_sig = "func _palette_for_id(eid: int) -> Color";
    let old_count = stripped.matches(old_sig).count();
    assert_eq!(
        old_count, 0,
        "A4.2: prior single-arg signature `{old_sig}` must NOT remain in source; got {old_count}"
    );
    println!("[P14-α A4] `_palette_for_id` signature extended to (eid, agent_id) ✓");
}

// ─── Assertion 5: palette_body_channel_keyed_off_role_bucket ──────────────
#[test]
fn harness_p14_alpha_a5_palette_body_channel_keyed_off_role_bucket() {
    // Type: A — body must invoke `_role_bucket(agent_id)` to derive the body
    // palette index `b`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_func_body(&stripped, "func _palette_for_id(")
        .expect("A5.1: _palette_for_id body must exist");
    assert!(
        body.contains("_role_bucket(agent_id)"),
        "A5.2: _palette_for_id body must invoke `_role_bucket(agent_id)`; body=\n{body}"
    );
    println!("[P14-α A5] body channel uses _role_bucket(agent_id) ✓");
}

// ─── Assertion 6: palette_hair_and_skin_preserve_entity_bits_hashing ──────
#[test]
fn harness_p14_alpha_a6_palette_hair_and_skin_preserve_entity_bits_hashing() {
    // Type: D — Phase 4-γ A5 invariant. Hair + skin still entity_bits.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let body = extract_func_body(&stripped, "func _palette_for_id(")
        .expect("A6.1: _palette_for_id body must exist");
    let compact = no_ws(&body);
    let hair_needle = no_ws("absi(eid * 2654435761) % PALETTE_HAIR_COLS");
    let skin_needle = no_ws("absi(eid * 2246822519) % PALETTE_SKIN_COLS");
    assert!(
        compact.contains(&hair_needle),
        "A6.2: hair channel must use `absi(eid * 2654435761) % PALETTE_HAIR_COLS`; body=\n{body}"
    );
    assert!(
        compact.contains(&skin_needle),
        "A6.3: skin channel must use `absi(eid * 2246822519) % PALETTE_SKIN_COLS`; body=\n{body}"
    );
    println!("[P14-α A6] hair + skin channels preserve entity_bits hashing ✓");
}

// ─── Assertion 7: palette_call_site_passes_both_ids ───────────────────────
#[test]
fn harness_p14_alpha_a7_palette_call_site_passes_both_ids() {
    // Type: A — call site must be `_palette_for_id(ids[i], int(agent_ids[i]))`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let compact = no_ws(&stripped);
    let new_call = no_ws("_palette_for_id(ids[i], int(agent_ids[i]))");
    assert!(
        compact.contains(&new_call),
        "A7.1: call site must use `_palette_for_id(ids[i], int(agent_ids[i]))`"
    );
    // The old single-arg call form must NOT remain.
    let old_call = no_ws("_palette_for_id(ids[i])");
    // The new call contains the old as a substring; we need a stricter check.
    // Count occurrences of `_palette_for_id(ids[i])` that are NOT followed by `,`.
    let mut leftover_old = 0usize;
    let needle = "_palette_for_id(ids[i])";
    let mut start = 0usize;
    while let Some(pos) = stripped[start..].find(needle) {
        let abs = start + pos;
        // Check the character after the match. The new form has `,` after
        // `ids[i]`, while the old form has `)` directly.
        // Specifically: old = `..._palette_for_id(ids[i])` ends with `)`.
        // The new form is `..._palette_for_id(ids[i], int(...))` — at the
        // position of `_palette_for_id(ids[i]` (without trailing `)`), the
        // next char is `,`. So we look at the byte at `abs + len(needle) - 1`
        // which is the `)` in the matched substring — but in the new form,
        // the substring `_palette_for_id(ids[i])` does not appear because
        // after `ids[i]` comes `,` not `)`. So any match of the old call
        // exactly indicates a leftover.
        let _ = old_call; // suppress unused warning if any
        leftover_old += 1;
        start = abs + needle.len();
    }
    assert_eq!(
        leftover_old, 0,
        "A7.2: prior single-arg call form `_palette_for_id(ids[i])` must NOT remain; \
         got {leftover_old} occurrence(s)"
    );
    println!("[P14-α A7] call site updated to `_palette_for_id(ids[i], int(agent_ids[i]))` ✓");
}

// ─── Assertion 8: icon_offset_px_constant_declared ────────────────────────
#[test]
fn harness_p14_alpha_a8_icon_offset_px_declared() {
    // Type: A — must be declared exactly once as Vector2(0, -12).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "ICON_OFFSET_PX", "A8");
    let compact = no_ws(&rhs);
    assert_eq!(
        compact, "Vector2(0,-12)",
        "A8: ICON_OFFSET_PX must equal Vector2(0, -12); got `{rhs}`"
    );
    println!("[P14-α A8] ICON_OFFSET_PX = Vector2(0, -12) ✓");
}

// ─── Assertion 9: icon_size_px_constant_declared ──────────────────────────
#[test]
fn harness_p14_alpha_a9_icon_size_px_declared() {
    // Type: A — must be declared exactly once as Vector2(16, 16).
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "ICON_SIZE_PX", "A9");
    let compact = no_ws(&rhs);
    assert_eq!(
        compact, "Vector2(16,16)",
        "A9: ICON_SIZE_PX must equal Vector2(16, 16); got `{rhs}`"
    );
    println!("[P14-α A9] ICON_SIZE_PX = Vector2(16, 16) ✓");
}

// ─── Assertion 10: head_icon_position_helper_declared ─────────────────────
#[test]
fn harness_p14_alpha_a10_head_icon_position_helper_declared() {
    // Type: A — must contain the signature AND body must return
    // `world_pos + ICON_OFFSET_PX`.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let sig = "func head_icon_position(world_pos: Vector2) -> Vector2";
    let sig_count = stripped.matches(sig).count();
    assert_eq!(
        sig_count, 1,
        "A10.1: signature `{sig}` must appear exactly once; got {sig_count}"
    );
    let body = extract_func_body(&stripped, "func head_icon_position(")
        .expect("A10.2: head_icon_position body must exist");
    let compact = no_ws(&body);
    assert!(
        compact.contains("returnworld_pos+ICON_OFFSET_PX"),
        "A10.3: head_icon_position body must `return world_pos + ICON_OFFSET_PX`; body=\n{body}"
    );
    println!("[P14-α A10] head_icon_position(world_pos) -> Vector2 declared with correct body ✓");
}

// ─── Assertion 11: sprite_scale_invariant_preserved ───────────────────────
#[test]
fn harness_p14_alpha_a11_sprite_scale_preserved() {
    // Type: D — Phase 4-γ invariant.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "A11");
    let accepted = ["0.25", ".25"];
    assert!(
        accepted.contains(&rhs.as_str()),
        "A11: SPRITE_SCALE must be EXACTLY `0.25` or `.25`; got `{rhs}`"
    );
    println!("[P14-α A11] SPRITE_SCALE = 0.25 preserved ✓");
}

// ─── Assertion 12: state_tints_palette_preserved ──────────────────────────
#[test]
fn harness_p14_alpha_a12_state_tints_palette_preserved() {
    // Type: D — Phase 11-α + D1 invariant.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_TINTS"),
        "A12.1: STATE_TINTS identifier must be present"
    );
    let decl_pos = stripped.find("STATE_TINTS").expect("A12.2");
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
        "A12.5: STATE_TINTS block must contain EXACTLY 4 Color literals; got {color_count}"
    );
    println!("[P14-α A12] STATE_TINTS palette = 4 entries preserved ✓");
}

// ─── Assertion 13: state_scale_boost_preserved ────────────────────────────
#[test]
fn harness_p14_alpha_a13_state_scale_boost_preserved() {
    // Type: D — Phase 13-γ invariant. Must be [1.0, 1.15, 1.15, 1.15].
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    assert!(
        stripped.contains("STATE_SCALE_BOOST"),
        "A13.1: STATE_SCALE_BOOST identifier must be present"
    );
    let decl_pos = stripped.find("STATE_SCALE_BOOST").expect("A13.2");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A13.3: STATE_SCALE_BOOST array must open with `[`");
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
    assert_eq!(depth, 0, "A13.4: STATE_SCALE_BOOST must terminate with `]`");
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
        "A13.5: STATE_SCALE_BOOST must have 4 entries; got {n}: {entries:?}",
        n = entries.len()
    );
    let want: [f32; 4] = [1.0, 1.15, 1.15, 1.15];
    for (i, want_v) in want.iter().enumerate() {
        let got: f32 = entries[i]
            .parse()
            .unwrap_or_else(|_| panic!("A13.6: entry {i} parse failed: `{}`", entries[i]));
        assert!(
            (got - want_v).abs() < 1e-6,
            "A13.7: STATE_SCALE_BOOST[{i}] must equal {want_v}; got {got}"
        );
    }
    println!("[P14-α A13] STATE_SCALE_BOOST = [1.0, 1.15, 1.15, 1.15] preserved ✓");
}

// ─── Assertion 14: recall_and_combat_cue_constants_preserved ──────────────
#[test]
fn harness_p14_alpha_a14_recall_and_combat_cue_constants_preserved() {
    // Type: D — Phase 8-δ and Phase 9-δ invariants.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let recall = unique_decl_rhs(&stripped, "RECALL_CUE_SCALE_BOOST", "A14.1");
    let combat = unique_decl_rhs(&stripped, "COMBAT_CUE_SCALE_BOOST", "A14.2");
    let recall_v: f32 = recall
        .parse()
        .unwrap_or_else(|_| panic!("A14.3: RECALL_CUE_SCALE_BOOST parse failed: `{recall}`"));
    let combat_v: f32 = combat
        .parse()
        .unwrap_or_else(|_| panic!("A14.4: COMBAT_CUE_SCALE_BOOST parse failed: `{combat}`"));
    assert!(
        (recall_v - 1.25).abs() < f32::EPSILON,
        "A14.5: RECALL_CUE_SCALE_BOOST must be EXACTLY 1.25; got {recall_v}"
    );
    assert!(
        (combat_v - 1.3).abs() < 1e-6,
        "A14.6: COMBAT_CUE_SCALE_BOOST must be EXACTLY 1.3; got {combat_v}"
    );
    println!("[P14-α A14] RECALL_CUE_SCALE_BOOST=1.25 + COMBAT_CUE_SCALE_BOOST=1.3 preserved ✓");
}

// ─── Assertion 15: camera_zoom_invariants_preserved ───────────────────────
#[test]
fn harness_p14_alpha_a15_camera_zoom_invariants_preserved() {
    // Type: D — Phase 12-α + Phase 13-α invariants. Scope discipline guard.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let zoom_min = unique_decl_rhs(&stripped, "ZOOM_MIN", "A15.1");
    let zoom_max = unique_decl_rhs(&stripped, "ZOOM_MAX", "A15.2");
    let zoom_default = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "A15.3");
    assert_eq!(
        no_ws(&zoom_min),
        "Vector2(0.5,0.5)",
        "A15.4: ZOOM_MIN must equal Vector2(0.5, 0.5); got `{zoom_min}`"
    );
    assert_eq!(
        no_ws(&zoom_max),
        "Vector2(4.0,4.0)",
        "A15.5: ZOOM_MAX must equal Vector2(4.0, 4.0); got `{zoom_max}`"
    );
    let default_compact = no_ws(&zoom_default);
    let accepted = [
        "Vector2(3.0,3.0)",
        "Vector2(3,3)",
        "Vector2(3.0,3)",
        "Vector2(3,3.0)",
    ];
    assert!(
        accepted.contains(&default_compact.as_str()),
        "A15.6: ZOOM_DEFAULT must equal Vector2(3.0, 3.0); got `{zoom_default}`"
    );
    println!("[P14-α A15] camera ZOOM_MIN/MAX/DEFAULT invariants preserved ✓");
}

// ─── Assertion 16: phase13_bootstrap_three_campfires_preserved ────────────
#[test]
fn harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved() {
    // Type: D — Phase 13-α + 13-ε invariants. Cross-file regression guard.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let x = unique_decl_rhs(&stripped, "BOOTSTRAP_X", "A16.1");
    let y = unique_decl_rhs(&stripped, "BOOTSTRAP_Y", "A16.2");
    let xl = unique_decl_rhs(&stripped, "BOOTSTRAP_X_LEFT", "A16.3");
    let xr = unique_decl_rhs(&stripped, "BOOTSTRAP_X_RIGHT", "A16.4");
    let path = unique_decl_rhs(&stripped, "BUILDING_SPRITE_PATH", "A16.5");
    assert_eq!(x, "32", "A16.6: BOOTSTRAP_X must be `32`; got `{x}`");
    assert_eq!(y, "32", "A16.7: BOOTSTRAP_Y must be `32`; got `{y}`");
    assert_eq!(xl, "24", "A16.8: BOOTSTRAP_X_LEFT must be `24`; got `{xl}`");
    assert_eq!(xr, "40", "A16.9: BOOTSTRAP_X_RIGHT must be `40`; got `{xr}`");
    assert!(
        path.contains("campfire/1.png"),
        "A16.10: BUILDING_SPRITE_PATH must reference campfire/1.png; got `{path}`"
    );
    println!("[P14-α A16] three-campfire bootstrap geometry preserved ✓");
}

// ─── Assertion 17: phase13_beta_resource_constants_preserved ──────────────
#[test]
fn harness_p14_alpha_a17_phase13_beta_resource_constants_preserved() {
    // Type: D — Phase 13-β invariant.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let path = unique_decl_rhs(&stripped, "RESOURCE_SPRITE_PATH", "A17.1");
    assert!(
        path.contains("storage_pit/1.png"),
        "A17.2: RESOURCE_SPRITE_PATH must reference storage_pit/1.png; got `{path}`"
    );
    let z = unique_decl_rhs(&stripped, "Z_RESOURCE", "A17.3");
    assert_eq!(z, "3", "A17.4: Z_RESOURCE must equal 3; got `{z}`");
    let count = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A17.5");
    assert_eq!(count, "20", "A17.6: RESOURCE_COUNT must equal 20; got `{count}`");
    let seed = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A17.7");
    assert_eq!(
        seed, "88675123",
        "A17.8: RESOURCE_SEED must equal 88675123; got `{seed}`"
    );
    println!("[P14-α A17] Phase 13-β resource constants preserved ✓");
}

// ─── Assertion 18: agent_base_png_asset_exists ────────────────────────────
#[test]
fn harness_p14_alpha_a18_agent_base_png_asset_exists() {
    // Type: A — substrate verification: file exists and size > 0.
    let path = project_root()
        .join("assets")
        .join("sprites")
        .join("agent_base.png");
    assert!(
        path.is_file(),
        "A18.1: agent_base.png must exist at {path:?}"
    );
    let meta = fs::metadata(&path)
        .unwrap_or_else(|e| panic!("A18.2: metadata for {path:?}: {e}"));
    assert!(
        meta.len() > 0,
        "A18.3: agent_base.png must be non-empty; got 0 bytes"
    );
    println!("[P14-α A18] agent_base.png exists, {} bytes ✓", meta.len());
}

// ─── Assertion 19: palette_lut_png_asset_exists ───────────────────────────
#[test]
fn harness_p14_alpha_a19_palette_lut_png_asset_exists() {
    // Type: A — substrate verification: file exists and size > 0.
    let path = project_root()
        .join("assets")
        .join("sprites")
        .join("palette_lut.png");
    assert!(
        path.is_file(),
        "A19.1: palette_lut.png must exist at {path:?}"
    );
    let meta = fs::metadata(&path)
        .unwrap_or_else(|e| panic!("A19.2: metadata for {path:?}: {e}"));
    assert!(
        meta.len() > 0,
        "A19.3: palette_lut.png must be non-empty; got 0 bytes"
    );
    println!("[P14-α A19] palette_lut.png exists, {} bytes ✓", meta.len());
}
