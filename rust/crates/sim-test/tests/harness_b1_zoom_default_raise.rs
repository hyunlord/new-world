//! V7 B-1 — Bigger agents via ZOOM_DEFAULT raise (SPRITE_SCALE untouched).
//!
//! feature: b1-zoom-default-raise
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Pure static file-inspection harness (G Phase A / H Phase A precedent).
//! No simulation is run (`ticks: 0`), no ECS component is queried — a camera
//! zoom constant is a GDScript / Camera2D effect the headless harness cannot
//! exercise (no viewport, and the per-sprite size delta is sub-resolution for
//! the whole-scene VLM — see CLAUDE.md "VLM Visual Verification — Known
//! Limitation"). Every assertion reads source text via the established
//! helpers. The authoritative behavioural gate is the windowed-Godot review
//! (prompt Section 7).
//!
//! Feature: raise the DEFAULT camera zoom only — `ZOOM_DEFAULT` → 5.0 — so
//! agents (16×24 frame × SPRITE_SCALE 0.25) render at 20×30 px at the default
//! view. SPRITE_SCALE (Phase 4-γ) is UNTOUCHED; we scale the view, not the
//! sprite. ZOOM_MIN (0.5) / ZOOM_MAX (8.0) unchanged.
//!
//! Assertions (6):
//!   a1 — ZOOM_DEFAULT == Vector2(5.0,5.0)                     (Type A)
//!   a2 — ZOOM_MIN preserved == Vector2(0.5,0.5)               (Type D)
//!   a3 — ZOOM_MAX preserved == Vector2(8.0,8.0)               (Type D)
//!   a4 — default x within [min x, max x]: 0.5 <= 5.0 <= 8.0   (Type A)
//!   a5 — agent_renderer SPRITE_SCALE untouched == 0.25        (Type D)
//!   a6 — ZOOM_DEFAULT x (5.0) >= Phase 14-ζ CLOSE_MIN 2.0     (Type A)
//!
//! (Plan Assertion 7 — "every ZOOM_DEFAULT guard expects the new value AND
//!  cargo test --workspace is green" — is the cross-file workspace guard
//!  proven by the 10 mechanical guard swaps, not a test in THIS file. This
//!  harness asserts only the new value and never spells the old parenthesised
//!  zoom literal, so the contract "b1 omits the old literal" holds.)
//!
//! Run:
//!   cargo test -p sim-test --test harness_b1_zoom_default_raise -- --nocapture

use std::fs;
use std::path::PathBuf;

// ── helpers (G / H Phase A precedent) ────────────────────────────────────────

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

/// Strip `#` line comments while respecting string literals.
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

/// Find every `const`/`var` RHS declaration of `ident`.
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

/// Parse a leading float from an RHS string (tolerates a leading sign).
fn parse_float_rhs(rhs: &str) -> Option<f64> {
    let trimmed = rhs.trim();
    let mut end = 0usize;
    let mut saw_digit = false;
    let mut saw_dot = false;
    for (i, c) in trimmed.char_indices() {
        if i == 0 && (c == '-' || c == '+') {
            end = i + c.len_utf8();
            continue;
        }
        if c.is_ascii_digit() {
            end = i + c.len_utf8();
            saw_digit = true;
        } else if c == '.' && !saw_dot {
            end = i + c.len_utf8();
            saw_dot = true;
        } else {
            break;
        }
    }
    if !saw_digit {
        return None;
    }
    trimmed[..end].parse::<f64>().ok()
}

/// Extract the first (x) numeric component from a `Vector2(x, y)` RHS by
/// parsing the float immediately after the opening parenthesis.
fn vec2_first_component(rhs: &str) -> Option<f64> {
    let paren = rhs.find('(')?;
    parse_float_rhs(rhs[paren + 1..].trim_start())
}

// ════════════════════════════════════════════════════════════════════════════
// scripts/ui/camera_controller.gd  +  scripts/ui/agent_renderer.gd
// ════════════════════════════════════════════════════════════════════════════

// ─── a1: ZOOM_DEFAULT raised to 5.0 ───────────────────────────────────────────
#[test]
fn harness_b1_a1_zoom_default_equals_five() {
    // Type A — the entire feature IS this single literal swap to the 5.0
    // default. Any other value = feature not implemented. Exact
    // (whitespace-collapsed) match is required — a range or `> 0` would pass
    // for a broken or untouched constant.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let def_rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "a1.1");
    assert_eq!(
        no_ws(&def_rhs),
        "Vector2(5.0,5.0)",
        "a1.2: ZOOM_DEFAULT must equal Vector2(5.0,5.0) (raised to 5.0 in B-1); got `{def_rhs}`"
    );
    println!("[B1 a1] ZOOM_DEFAULT = Vector2(5.0, 5.0) ✓");
}

// ─── a2: ZOOM_MIN preserved ───────────────────────────────────────────────────
#[test]
fn harness_b1_a2_zoom_min_preserved() {
    // Type D — regression guard. ZOOM_MIN (Phase 12-α overview floor) must NOT
    // change; catches an over-eager edit that also touches the floor.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let min_rhs = unique_decl_rhs(&stripped, "ZOOM_MIN", "a2.1");
    assert_eq!(
        no_ws(&min_rhs),
        "Vector2(0.5,0.5)",
        "a2.2: ZOOM_MIN must equal Vector2(0.5,0.5) (Phase 12-α invariant); got `{min_rhs}`"
    );
    println!("[B1 a2] ZOOM_MIN = Vector2(0.5, 0.5) preserved ✓");
}

// ─── a3: ZOOM_MAX preserved ───────────────────────────────────────────────────
#[test]
fn harness_b1_a3_zoom_max_preserved() {
    // Type D — regression guard. ZOOM_MAX (G Phase A zoom-in ceiling 8.0) must
    // NOT change; catches an edit that raises the ceiling along with the default.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let max_rhs = unique_decl_rhs(&stripped, "ZOOM_MAX", "a3.1");
    assert_eq!(
        no_ws(&max_rhs),
        "Vector2(8.0,8.0)",
        "a3.2: ZOOM_MAX must equal Vector2(8.0,8.0) (G Phase A invariant); got `{max_rhs}`"
    );
    println!("[B1 a3] ZOOM_MAX = Vector2(8.0, 8.0) preserved ✓");
}

// ─── a4: default within [min, max] bounds ─────────────────────────────────────
#[test]
fn harness_b1_a4_default_within_bounds() {
    // Type A — mathematical invariant. `_ready()` sets zoom = ZOOM_DEFAULT and
    // `_apply_zoom_delta` clamps to [MIN, MAX]; a default outside the bounds
    // would be silently clamped on first wheel notch. Independent of the 5.0
    // choice, so it survives a future 4.0/6.0 follow-up while catching a typo
    // (50.0 / 0.05).
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let min_x = vec2_first_component(&unique_decl_rhs(&stripped, "ZOOM_MIN", "a4.1"))
        .expect("a4.2: ZOOM_MIN x-component must parse as float");
    let def_x = vec2_first_component(&unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "a4.3"))
        .expect("a4.4: ZOOM_DEFAULT x-component must parse as float");
    let max_x = vec2_first_component(&unique_decl_rhs(&stripped, "ZOOM_MAX", "a4.5"))
        .expect("a4.6: ZOOM_MAX x-component must parse as float");
    assert!(
        min_x <= def_x,
        "a4.7: ZOOM_MIN x ({min_x}) must be <= ZOOM_DEFAULT x ({def_x})"
    );
    assert!(
        def_x <= max_x,
        "a4.8: ZOOM_DEFAULT x ({def_x}) must be <= ZOOM_MAX x ({max_x})"
    );
    println!("[B1 a4] {min_x} <= {def_x} <= {max_x} (default within bounds) ✓");
}

// ─── a5: SPRITE_SCALE untouched ───────────────────────────────────────────────
#[test]
fn harness_b1_a5_sprite_scale_untouched() {
    // Type D — cross-file regression guard. The feature's premise is "scale the
    // view, not the sprite". SPRITE_SCALE (Phase 4-γ invariant, locked in 14
    // harnesses) must remain 0.25 — proves B-1 did not take the forbidden
    // shortcut of touching the sprite.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "a5.1");
    assert_eq!(
        no_ws(&rhs),
        "0.25",
        "a5.2: SPRITE_SCALE must equal 0.25 (Phase 4-γ invariant, untouched by B-1); got `{rhs}`"
    );
    println!("[B1 a5] agent_renderer SPRITE_SCALE = 0.25 untouched ✓");
}

// ─── a6: default still in Phase 14-ζ CLOSE tier ───────────────────────────────
#[test]
fn harness_b1_a6_zoom_lod_close_tier() {
    // Type A — behavioural invariant. Phase 14-ζ classifies CLOSE tier as zoom
    // >= 2.0 (FAR < 1.0). The 5.0 default must land in CLOSE tier so the
    // trail/overview LOD behaviour is unchanged. The 2.0
    // literal is the documented Phase 14-ζ CLOSE_MIN (zoom_lod_controller); this
    // assertion reads only the camera_controller default — zoom_lod_controller
    // itself stays untouched.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let def_x = vec2_first_component(&unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "a6.1"))
        .expect("a6.2: ZOOM_DEFAULT x-component must parse as float");
    assert!(
        def_x >= 2.0,
        "a6.3: ZOOM_DEFAULT x ({def_x}) must be >= Phase 14-ζ CLOSE_MIN 2.0 \
         (default view stays CLOSE tier — trails on, overview off)"
    );
    println!("[B1 a6] ZOOM_DEFAULT x = {def_x} >= 2.0 (CLOSE tier) ✓");
}
