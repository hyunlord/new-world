//! V7 Visualization C — Fix Inspector Panel Overlap harness.
//!
//! feature: fix-inspector-overlap
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Static source checks for the production inspector, topbar and news geometry.
//! The inspector sits below the topbar at the right; news is bottom-left.
//! Both dimensions are read from the actual panel assignments, excluding children.
//! No Godot launch or simulation behavior is involved.
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

/// Resolve a numeric layout argument or its module constant.
fn layout_number(src: &str, value: &str) -> f64 {
    value.trim().parse().ok().or_else(|| parse_const_float(src, value.trim()))
        .unwrap_or_else(|| panic!("layout argument must be numeric or a constant: {value}"))
}

/// Parse only a bare panel Vector2 assignment in _ready, excluding child nodes.
fn ready_vector(src: &str, target: &str) -> [f64; 2] {
    let body = ready_body(src);
    let prefix = format!("{target} = Vector2(");
    let rhs = body.lines().find_map(|line| line.trim().strip_prefix(&prefix))
        .expect("bare panel Vector2 assignment");
    let (x, y) = rhs.strip_suffix(')').expect("Vector2 closes").split_once(',')
        .expect("Vector2 has two arguments");
    [layout_number(src, x), layout_number(src, y)]
}

/// Read the actual ArtUI panel rectangle, rather than unused legacy constants.
fn hud_rect(src: &str) -> [f64; 4] {
    let body = ready_body(src);
    let args = body.split_once("ArtUI.panel(self, Rect2(").expect("HUD panel call").1
        .split_once(')').expect("Rect2 closes").0;
    let values: Vec<f64> = args.split(',').map(|value| layout_number(src, value)).collect();
    values.try_into().expect("Rect2 has four arguments")
}

fn inspector_rect(src: &str) -> [f64; 4] {
    let [x, y] = ready_vector(src, "position");
    let [width, height] = ready_vector(src, "size");
    [x, y, width, height]
}

fn disjoint(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] + a[2] <= b[0] || b[0] + b[2] <= a[0]
        || a[1] + a[3] <= b[1] || b[1] + b[3] <= a[1]
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
    // 124 = topbar y12 + height100 + gap12.
    assert_eq!(
        value, 124.0,
        "A1: INSPECTOR_TOP_OFFSET must equal exactly 124.0, got {value}"
    );
}

// ── A2: ready_applies_offset_to_panel_rect ─────────────────────────────────

#[test]
fn harness_inspector_ready_applies_offset_to_panel_rect() {
    let src = read_inspector_src();

    // Inspect the bare panel position, excluding any child assignments.
    assert!(
        ready_has_bare_assign(&src, "position", "Vector2(1548, INSPECTOR_TOP_OFFSET)"),
        "A2: `_ready` must contain bare panel position bound to INSPECTOR_TOP_OFFSET"
    );
    // The size assignment supplies the dimensions used by the overlap check.
    assert!(
        ready_has_bare_assign(&src, "size", "Vector2(PANEL_WIDTH, 620)"),
        "A2: bare panel size must use PANEL_WIDTH and native layout height"
    );
}

// ── A3: no_overlap_invariant_cross_file ────────────────────────────────────

#[test]
fn harness_inspector_no_overlap_invariant_cross_file() {
    let inspector = read_inspector_src();
    let status = read_status_panel_src();

    let topbar = read_file(&["scripts", "ui", "panels", "hud_topbar.gd"]);
    let inspector = inspector_rect(&inspector);
    let news = hud_rect(&status);
    let topbar = hud_rect(&topbar);
    assert!(disjoint(inspector, news), "inspector must not overlap the news panel");
    assert!(disjoint(inspector, topbar), "inspector must not overlap the topbar");

}

// ── A4: anchors_and_width_untouched_regression ─────────────────────────────

#[test]
fn harness_inspector_anchors_and_width_untouched_regression() {
    let src = read_inspector_src();

    // Pin the production inspector width.
    let width = parse_const_float(&src, "PANEL_WIDTH")
        .expect("A4: `const PANEL_WIDTH` must parse");
    assert_eq!(
        width, 360.0,
        "A4: PANEL_WIDTH must remain exactly 360.0, got {width}"
    );
    // Preserve right alignment using actual panel rectangles.
    assert!(
        {
            let inspector = inspector_rect(&src);
            let topbar = hud_rect(&read_file(&["scripts", "ui", "panels", "hud_topbar.gd"]));
            inspector[0] + inspector[2] == topbar[0] + topbar[2]
        },
        "A4: inspector and topbar right edges must align"
    );
}
