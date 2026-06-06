//! V7 Visualization C — Fix Inspector Panel Overlap harness.
//!
//! feature: fix-inspector-overlap
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Static GDScript text-parse regression test for the top-right overlap
//! between `agent_inspector_panel.gd` (right-anchored, full-height) and
//! `hud_status_panel.gd` (top-right, fixed 320×180). The fix pushes the
//! inspector's top edge to `INSPECTOR_TOP_OFFSET = 204.0` so it begins
//! below the status panel's bottom edge (HUD_MARGIN + PANEL_HEIGHT = 192).
//!
//! No Godot engine launch — these assertions parse file text only.
//!
//! Structural rigor: A2/A4 inspect ONLY the bare panel-rect assignments
//! inside `_ready()` (identifier with no `obj.` prefix), so a child/background
//! assignment such as `_vbox.offset_top = 8.0` or `bg.anchor_right = 1.0`
//! can neither satisfy nor falsely break the guard.
//!
//! Run:
//!   cargo test -p sim-test --test harness_fix_inspector_overlap -- --nocapture

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

fn read_inspector_src() -> String {
    read_file(&["scripts", "ui", "panels", "agent_inspector_panel.gd"])
}

fn read_status_panel_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_status_panel.gd"])
}

/// True when `after` (the slice immediately following an identifier) ends that
/// identifier — i.e. the next char is not alphanumeric/underscore, so we did
/// not match a longer identifier with `name` as a prefix.
fn identifier_terminated(after: &str) -> bool {
    match after.chars().next() {
        None => true,
        Some(c) => !(c.is_ascii_alphanumeric() || c == '_'),
    }
}

/// Count module-level (zero-indentation) `const NAME` declarations.
///
/// Module-level means the line begins at column 0 (GDScript top-level consts
/// carry no indentation; a same-named local would be indented inside a func).
fn count_module_const(src: &str, name: &str) -> usize {
    src.lines()
        .filter(|line| {
            // Reject indented (local / nested) declarations.
            if line.starts_with([' ', '\t']) {
                return false;
            }
            let Some(rest) = line.strip_prefix("const ") else {
                return false;
            };
            let rest = rest.trim_start();
            rest.strip_prefix(name)
                .map(identifier_terminated)
                .unwrap_or(false)
        })
        .count()
}

/// Parse the numeric literal assigned to a `const NAME` declaration.
///
/// Handles both `const NAME := 204.0` and `const NAME: float = 180.0`
/// forms by taking the substring after the final `=` on the declaring line
/// and parsing the leading numeric token to f64.
fn parse_const_float(src: &str, name: &str) -> Option<f64> {
    for line in src.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("const ") else {
            continue;
        };
        let rest = rest.trim_start();
        if !rest.starts_with(name) {
            continue;
        }
        if !identifier_terminated(&rest[name.len()..]) {
            continue;
        }
        let Some(eq_idx) = line.rfind('=') else {
            continue;
        };
        let rhs = line[eq_idx + 1..].trim();
        let token: String = rhs
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
            .collect();
        if let Ok(v) = token.parse::<f64>() {
            return Some(v);
        }
    }
    None
}

/// Extract the `_ready()` function body (lines between `func _ready(` and the
/// next top-level `func ` declaration). Used to scope structural assertions to
/// the panel's own layout block.
fn ready_body(src: &str) -> String {
    let mut out = String::new();
    let mut in_ready = false;
    for line in src.lines() {
        if line.trim_start().starts_with("func _ready(") {
            in_ready = true;
            continue;
        }
        if in_ready {
            // A new top-level `func` (column 0) ends `_ready`.
            if line.starts_with("func ") {
                break;
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Does `_ready()` contain a *bare* assignment `target = <rhs>` — i.e. the
/// assignment target is `target` with no `obj.` prefix, so child/background
/// node assignments (`_vbox.offset_top`, `bg.anchor_right`) are excluded?
/// `rhs` comparison is whitespace-insensitive.
fn ready_has_bare_assign(src: &str, target: &str, expected_rhs: &str) -> bool {
    ready_body(src).lines().any(|line| {
        let t = line.trim();
        // A bare `target` cannot be preceded by `obj.` because trimming the
        // leading whitespace leaves the prefix `obj.` in place; such lines do
        // not start with `target`.
        let Some(after) = t.strip_prefix(target) else {
            return false;
        };
        // Reject longer identifiers like `offset_top_extra`: the char right
        // after `target` must not continue the identifier.
        if !identifier_terminated(after) {
            return false;
        }
        let Some(rest) = after.trim_start().strip_prefix('=') else {
            return false;
        };
        rest.trim() == expected_rhs
    })
}

// ── A1: inspector_top_offset_const_exists_exactly_once_and_value ────────────

#[test]
fn harness_inspector_top_offset_const_exists_and_value() {
    let src = read_inspector_src();

    // Structural: exactly one module-level declaration (no shadowing / dupes).
    let decls = count_module_const(&src, "INSPECTOR_TOP_OFFSET");
    assert_eq!(
        decls, 1,
        "A1: agent_inspector_panel.gd must declare `const INSPECTOR_TOP_OFFSET` \
         exactly once at module level, found {decls}"
    );

    let value = parse_const_float(&src, "INSPECTOR_TOP_OFFSET")
        .expect("A1: INSPECTOR_TOP_OFFSET must parse to a numeric literal");
    // Type A: exact equality on a spec-derived layout constant
    // (204.0 = PANEL_HEIGHT 180 + 2×HUD_MARGIN 12).
    assert_eq!(
        value, 204.0,
        "A1: INSPECTOR_TOP_OFFSET must equal exactly 204.0, got {value}"
    );
}

// ── A2: ready_applies_offset_to_panel_rect ─────────────────────────────────

#[test]
fn harness_inspector_ready_applies_offset_to_panel_rect() {
    let src = read_inspector_src();

    // The panel rect's own `offset_top` is bound to the constant (bare
    // assignment inside `_ready`, excluding `_vbox.offset_top`).
    assert!(
        ready_has_bare_assign(&src, "offset_top", "INSPECTOR_TOP_OFFSET"),
        "A2: `_ready` must contain bare panel `offset_top = INSPECTOR_TOP_OFFSET`"
    );
    // The buggy panel-rect literal `offset_top = 0.0` must be gone, regardless
    // of spacing, and without false-matching the child `_vbox.offset_top`.
    assert!(
        !ready_has_bare_assign(&src, "offset_top", "0.0"),
        "A2: bare panel-rect `offset_top = 0.0` must be removed from `_ready`"
    );
}

// ── A3: no_overlap_invariant_cross_file ────────────────────────────────────

#[test]
fn harness_inspector_no_overlap_invariant_cross_file() {
    let inspector = read_inspector_src();
    let status = read_status_panel_src();

    let inspector_top = parse_const_float(&inspector, "INSPECTOR_TOP_OFFSET")
        .expect("A3: INSPECTOR_TOP_OFFSET must parse");
    let hud_margin =
        parse_const_float(&status, "HUD_MARGIN").expect("A3: HUD_MARGIN must parse");
    let panel_height =
        parse_const_float(&status, "PANEL_HEIGHT").expect("A3: PANEL_HEIGHT must parse");

    let status_bottom = hud_margin + panel_height;

    // Type D: the inspector's top edge must sit at or below the status panel
    // bottom edge. `>=` keeps the guard valid under any compliant future
    // offset; it re-flags overlap if a status-panel resize pushes the bottom
    // edge past the inspector top.
    assert!(
        inspector_top >= status_bottom,
        "A3: INSPECTOR_TOP_OFFSET ({inspector_top}) must be >= status-panel bottom \
         (HUD_MARGIN {hud_margin} + PANEL_HEIGHT {panel_height} = {status_bottom})"
    );
}

// ── A4: anchors_and_width_untouched_regression ─────────────────────────────

#[test]
fn harness_inspector_anchors_and_width_untouched_regression() {
    let src = read_inspector_src();

    // Width unchanged (layout-position-only fix; scope containment).
    let width = parse_const_float(&src, "PANEL_WIDTH")
        .expect("A4: `const PANEL_WIDTH` must parse");
    assert_eq!(
        width, 280.0,
        "A4: PANEL_WIDTH must remain exactly 280.0, got {width}"
    );
    // The panel rect's own right anchor unchanged — bare assignment in
    // `_ready`, NOT `bg.anchor_right` / `_vbox.anchor_right`.
    assert!(
        ready_has_bare_assign(&src, "anchor_right", "1.0"),
        "A4: `_ready` must keep bare panel `anchor_right = 1.0`"
    );
}
