//! D Phase A — GDScript Runtime Warnings Fix.
//!
//! Static file-inspection harness for the parse-warning cleanup and FFI
//! source-side stability checks defined in plan d-phase-a-runtime-warnings-fix.
//! All tests are pure file content checks; no simulation tick is run, no
//! new workspace dependency is added (regex is hand-rolled via str ops to
//! stay within the plan's "no Rust source change" scope constraint —
//! `sim-test` test files are the only sanctioned surface for the harness
//! generator, and adding a workspace-level `regex` dep would violate
//! scope assertion A16. The hand-rolled matchers are tightly scoped to
//! the exact patterns enumerated by the plan).
//!
//! Assertion → test mapping (locked thresholds):
//!   A3 — no `float(... TILE_SIZE / 2 ...)` pattern in agent_renderer.gd
//!   A4 — no `float(... TILE_SIZE / 2 ...)` pattern in world_renderer.gd
//!   A5 — `_ingest_memory_recalls(_ids: PackedInt64Array …)` exists exactly once
//!   A6 — `_ingest_combat_events(_ids: …, _agent_ids: …)` exists exactly once
//!   A7 — active params (xs/ys/n) NOT underscored inside the two ingest fns
//!   A8 — `fn get_construction_snapshot` survives in sim-bridge/src/ffi/
//!   A9 — `fn get_settlement_snapshot` survives in sim-bridge/src/ffi/
//!
//! Assertions A1, A2 (Godot --check-only parse) are verified via the
//! harness pipeline shell layer (see plan §5), not via cargo. A10–A14
//! are existing regression harnesses (P11-α, P12-α, P12-β, P12-β2, P12-γ).
//! A15 (workspace gate) and A16 (git scope) are pipeline-level checks.
//!
//! Run:
//!   cargo test -p sim-test --test harness_d_phase_a_runtime_warnings -- --nocapture

use std::fs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root above sim-test crate")
}

fn read_script(rel: &[&str]) -> String {
    let mut path = project_root();
    for seg in rel {
        path.push(seg);
    }
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Strip GDScript `#` line comments (string-quote aware) so contract checks
/// cannot be satisfied by commentary text alone.
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

fn read_agent_renderer_src() -> String {
    read_script(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_world_renderer_src() -> String {
    read_script(&["scripts", "ui", "world_renderer.gd"])
}

/// Read every `.rs` file under `rust/crates/sim-bridge/src/ffi/` and
/// concatenate. The plan permits the function to live in `world_node.rs`
/// or any submodule under `ffi/`, so we scan the whole directory.
fn read_ffi_sources() -> String {
    let dir = project_root()
        .join("rust")
        .join("crates")
        .join("sim-bridge")
        .join("src")
        .join("ffi");
    let mut combined = String::new();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {dir:?}: {e}"));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let txt = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path:?}: {e}"));
            combined.push_str(&txt);
            combined.push('\n');
        }
    }
    combined
}

// ── Hand-rolled matchers (no regex dep) ────────────────────────────────────

/// Count occurrences of `float(<body> TILE_SIZE / 2)` where `<body>` does
/// not contain a `(` or `)` (i.e. the integer division is the outer
/// expression directly inside the `float()` cast). Whitespace around `/`
/// is tolerated. Mirrors the plan's regex
/// `float\([^)]*TILE_SIZE\s*/\s*2\s*\)`.
fn count_float_tile_size_div_2(src: &str) -> usize {
    let needle = "float(";
    let mut count = 0usize;
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while let Some(found) = src[i..].find(needle) {
        let start = i + found + needle.len();
        // Walk to the matching `)` on the SAME nesting level (no `(` allowed
        // inside, matching `[^)]*` in the plan regex).
        let mut end = start;
        let mut ok = false;
        while end < bytes.len() {
            let c = bytes[end] as char;
            if c == '(' {
                break;
            }
            if c == ')' {
                ok = true;
                break;
            }
            end += 1;
        }
        if ok {
            let inner = &src[start..end];
            // Look for `TILE_SIZE` followed (after optional whitespace) by
            // `/ 2` ending at end-of-inner (i.e. the division is the LAST
            // term — the offending tail).
            if let Some(ts_pos) = inner.rfind("TILE_SIZE") {
                let tail = &inner[ts_pos + "TILE_SIZE".len()..];
                let trimmed = tail.trim_start();
                if let Some(rest) = trimmed.strip_prefix('/') {
                    let rest = rest.trim_start();
                    if let Some(after_two) = rest.strip_prefix('2') {
                        if after_two.trim().is_empty() {
                            count += 1;
                        }
                    }
                }
            }
            i = end + 1;
        } else {
            i = start;
        }
    }
    count
}

/// Find function definitions whose name matches `name` and return their
/// parameter-list strings (the text between the outermost `(` and `)`).
fn extract_param_lists(src: &str, name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let needle = format!("func {name}");
    let mut i = 0usize;
    let bytes = src.as_bytes();
    while let Some(found) = src[i..].find(&needle) {
        let abs = i + found;
        // Must be word-boundary on the left (start of line or whitespace)
        // and either whitespace or `(` on the right side of name.
        let left_ok = abs == 0
            || matches!(bytes[abs - 1] as char, ' ' | '\t' | '\n' | '\r');
        let after = abs + needle.len();
        let right_ok = after < bytes.len()
            && matches!(bytes[after] as char, '(' | ' ' | '\t');
        if !left_ok || !right_ok {
            i = abs + needle.len();
            continue;
        }
        // Walk to the first `(` then to its matching `)` (depth-tracked).
        let mut j = after;
        while j < bytes.len() && bytes[j] as char != '(' {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }
        let open = j;
        let mut depth = 1i32;
        let mut k = open + 1;
        while k < bytes.len() && depth > 0 {
            match bytes[k] as char {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            k += 1;
        }
        if depth == 0 {
            out.push(src[open + 1..k - 1].to_string());
        }
        i = k;
    }
    out
}

/// Check that the first parameter is named `param_name` with type `type_name`.
fn first_param_matches(params: &str, param_name: &str, type_name: &str) -> bool {
    // Take the first comma-separated chunk and normalize whitespace.
    let first = params.split(',').next().unwrap_or("").trim();
    // Expected shape: "<param_name>: <type_name>"
    let mut parts = first.splitn(2, ':');
    let lhs = parts.next().map(str::trim).unwrap_or("");
    let rhs = parts.next().map(str::trim).unwrap_or("");
    lhs == param_name && rhs == type_name
}

/// Check that the parameter list's first TWO parameters match the expected
/// names and types.
fn first_two_params_match(
    params: &str,
    n1: &str,
    t1: &str,
    n2: &str,
    t2: &str,
) -> bool {
    let mut it = params.split(',');
    let first = it.next().unwrap_or("").trim();
    let second = it.next().unwrap_or("").trim();
    let mut p1 = first.splitn(2, ':');
    let mut p2 = second.splitn(2, ':');
    let n1a = p1.next().map(str::trim).unwrap_or("");
    let t1a = p1.next().map(str::trim).unwrap_or("");
    let n2a = p2.next().map(str::trim).unwrap_or("");
    let t2a = p2.next().map(str::trim).unwrap_or("");
    n1a == n1 && t1a == t1 && n2a == n2 && t2a == t2
}

/// Returns true if any param name in the list is one of `_xs`, `_ys`, `_n`.
fn any_param_over_prefixed(params: &str) -> bool {
    for chunk in params.split(',') {
        let nm = chunk.trim().split(':').next().unwrap_or("").trim();
        if nm == "_xs" || nm == "_ys" || nm == "_n" {
            return true;
        }
    }
    false
}

/// Count standalone `fn <name>(` occurrences (Rust function definitions),
/// requiring whitespace or start-of-line on the left.
fn count_rust_fn_defs(src: &str, name: &str) -> usize {
    let needle = format!("fn {name}");
    let mut count = 0usize;
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while let Some(found) = src[i..].find(&needle) {
        let abs = i + found;
        let left_ok = abs == 0
            || matches!(
                bytes[abs - 1] as char,
                ' ' | '\t' | '\n' | '\r' | '\u{c}'
            );
        let after = abs + needle.len();
        // Right side: `(` (function def) or whitespace then `(`.
        let mut j = after;
        while j < bytes.len() && matches!(bytes[j] as char, ' ' | '\t') {
            j += 1;
        }
        let right_ok = j < bytes.len() && bytes[j] as char == '(';
        if left_ok && right_ok {
            count += 1;
        }
        i = abs + needle.len();
    }
    count
}

// ── A3 — no integer-division pattern in agent_renderer.gd ─────────────────
#[test]
fn harness_d_phase_a_no_integer_division_in_agent_renderer() {
    // Type: D — regression guard against the broken pattern
    let body = strip_gd_comments(&read_agent_renderer_src());
    let count = count_float_tile_size_div_2(&body);
    assert_eq!(
        count, 0,
        "A3: `float(... TILE_SIZE / 2)` integer-division pattern must not \
         appear in scripts/ui/agent_renderer.gd; found {count} occurrence(s)"
    );
    println!("[D-A A3] agent_renderer.gd integer-division pattern absent ✓");
}

// ── A4 — no integer-division pattern in world_renderer.gd ─────────────────
#[test]
fn harness_d_phase_a_no_integer_division_in_world_renderer() {
    // Type: D — regression guard
    let body = strip_gd_comments(&read_world_renderer_src());
    let count = count_float_tile_size_div_2(&body);
    assert_eq!(
        count, 0,
        "A4: `float(... TILE_SIZE / 2)` integer-division pattern must not \
         appear in scripts/ui/world_renderer.gd; found {count} occurrence(s)"
    );
    println!("[D-A A4] world_renderer.gd integer-division pattern absent ✓");
}

// ── A5 — memory_recall signature has leading-underscore _ids ──────────────
#[test]
fn harness_d_phase_a_memory_recall_signature_underscored() {
    // Type: D
    let body = strip_gd_comments(&read_agent_renderer_src());
    let lists = extract_param_lists(&body, "_ingest_memory_recalls");
    assert_eq!(
        lists.len(),
        1,
        "A5: must find exactly one `func _ingest_memory_recalls(...)` \
         definition; found {}",
        lists.len()
    );
    let ok = first_param_matches(&lists[0], "_ids", "PackedInt64Array");
    assert!(
        ok,
        "A5: first param of _ingest_memory_recalls must be \
         `_ids: PackedInt64Array`; got `{}`",
        lists[0]
    );
    println!("[D-A A5] _ingest_memory_recalls(_ids: PackedInt64Array, …) ✓");
}

// ── A6 — combat_event signature has _ids AND _agent_ids ───────────────────
#[test]
fn harness_d_phase_a_combat_event_signature_underscored() {
    // Type: D
    let body = strip_gd_comments(&read_agent_renderer_src());
    let lists = extract_param_lists(&body, "_ingest_combat_events");
    assert_eq!(
        lists.len(),
        1,
        "A6: must find exactly one `func _ingest_combat_events(...)` \
         definition; found {}",
        lists.len()
    );
    let ok = first_two_params_match(
        &lists[0],
        "_ids",
        "PackedInt64Array",
        "_agent_ids",
        "PackedInt64Array",
    );
    assert!(
        ok,
        "A6: first two params of _ingest_combat_events must be \
         `_ids: PackedInt64Array, _agent_ids: PackedInt64Array`; got `{}`",
        lists[0]
    );
    println!(
        "[D-A A6] _ingest_combat_events(_ids, _agent_ids, …) ✓"
    );
}

// ── A7 — active params (xs/ys/n) MUST NOT be underscored ──────────────────
#[test]
fn harness_d_phase_a_active_params_not_underscored() {
    // Type: A — over-correction guard
    let body = strip_gd_comments(&read_agent_renderer_src());
    let mut offenders: Vec<String> = Vec::new();
    for name in ["_ingest_memory_recalls", "_ingest_combat_events"] {
        for params in extract_param_lists(&body, name) {
            if any_param_over_prefixed(&params) {
                offenders.push(format!("{name}({params})"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "A7: active params (xs, ys, n) must remain unprefixed; found \
         over-prefixed in: {offenders:?}"
    );
    println!("[D-A A7] active params (xs, ys, n) not over-prefixed ✓");
}

// ── A8 — FFI get_construction_snapshot survives in sim-bridge ────────────
#[test]
fn harness_d_phase_a_ffi_get_construction_snapshot_present() {
    // Type: A
    let combined = read_ffi_sources();
    let count = count_rust_fn_defs(&combined, "get_construction_snapshot");
    assert!(
        count >= 1,
        "A8: expected ≥1 `fn get_construction_snapshot` definition in \
         rust/crates/sim-bridge/src/ffi/*.rs; found {count}"
    );
    println!("[D-A A8] get_construction_snapshot FFI source present ✓");
}

// ── A9 — FFI get_settlement_snapshot survives in sim-bridge ──────────────
#[test]
fn harness_d_phase_a_ffi_get_settlement_snapshot_present() {
    // Type: A
    let combined = read_ffi_sources();
    let count = count_rust_fn_defs(&combined, "get_settlement_snapshot");
    assert!(
        count >= 1,
        "A9: expected ≥1 `fn get_settlement_snapshot` definition in \
         rust/crates/sim-bridge/src/ffi/*.rs; found {count}"
    );
    println!("[D-A A9] get_settlement_snapshot FFI source present ✓");
}
