//! Meta-harness for `fix-count-guard-no-nested`.
//!
//! Verifies that `harness_p8_beta_a26_test_count_regression_guard`
//! (in `harness_p8_beta_memory_system.rs`) was rewritten from a nested
//! `cargo … --list` subprocess into a static source-tree `#[test]` count,
//! WITHOUT re-running the slow path. All assertions are static source
//! inspection or `std::fs` directory walks — NO subprocess is spawned here.
//!
//! Assertion numbering mirrors the locked plan (A1, A2, A4, A5, A6). Plan A3
//! ("the production guard itself executes and reports `ok` at HEAD") cannot be
//! checked from this file: integration-test files compile as separate crates,
//! so this file cannot cross-call the a26 fn. A3 is verified by the guard's own
//! result line in `cargo test -p sim-test --test harness_p8_beta_memory_system`.

use std::path::{Path, PathBuf};

/// Path to the rust workspace root (`…/rust`), derived from this crate's
/// manifest dir (`…/rust/crates/sim-test`).
fn rust_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // sim-test -> crates
        .expect("sim-test has a parent (crates)")
        .parent() // crates -> rust
        .expect("crates has a parent (rust)")
        .to_owned()
}

/// Absolute path to the file under test.
fn target_source_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/harness_p8_beta_memory_system.rs")
}

/// Full source text of the file under test.
fn read_target_source() -> String {
    let p = target_source_path();
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Recursively collect `*.rs` files under `dir`. Missing dirs contribute nothing.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Line-anchored `#[test]` count under one crate's `src` + `tests` dirs.
fn count_crate_tests(crate_dir: &Path) -> u32 {
    let mut files = Vec::new();
    collect_rs(&crate_dir.join("src"), &mut files);
    collect_rs(&crate_dir.join("tests"), &mut files);
    let mut total = 0u32;
    for f in files {
        if let Ok(text) = std::fs::read_to_string(&f) {
            total += text
                .lines()
                .filter(|l| l.trim_start().starts_with("#[test]"))
                .count() as u32;
        }
    }
    total
}

/// Workspace-wide line-anchored `#[test]` count across every `crates/<c>`.
fn count_workspace_tests(crates_dir: &Path) -> u32 {
    let Ok(entries) = std::fs::read_dir(crates_dir) else {
        panic!("crates dir not readable: {}", crates_dir.display());
    };
    let mut total = 0u32;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            total += count_crate_tests(&p);
        }
    }
    total
}

/// Isolate the body of `harness_p8_beta_a26_test_count_regression_guard`:
/// from its `fn` line to the next top-level (`column 0`) `fn`/`#[test]`/`#[…]`
/// item, or to EOF if it is the last function.
fn isolate_a26_body(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let start = lines
        .iter()
        .position(|l| {
            l.trim_start()
                .starts_with("fn harness_p8_beta_a26_test_count_regression_guard")
        })
        .expect("a26 fn signature not found");
    let mut end = lines.len();
    for (i, l) in lines.iter().enumerate().skip(start + 1) {
        // Top-level next item begins at column 0 (no indentation). Nested
        // helper fns inside a26 are indented and correctly excluded.
        if l.starts_with("fn ") || l.starts_with("#[test]") || l.starts_with("#[") {
            end = i;
            break;
        }
    }
    lines[start..end].join("\n")
}

/// Parse the `STATIC_TEST_FLOOR` const value from the source.
fn parse_static_test_floor(src: &str) -> u32 {
    let line = src
        .lines()
        .find(|l| l.trim_start().starts_with("const STATIC_TEST_FLOOR"))
        .expect("STATIC_TEST_FLOOR const declaration not found");
    let rhs = line
        .split('=')
        .nth(1)
        .expect("STATIC_TEST_FLOOR has an `=` rhs");
    rhs.trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("u32")
        .trim()
        .replace('_', "")
        .parse::<u32>()
        .unwrap_or_else(|e| panic!("parse STATIC_TEST_FLOOR rhs {rhs:?}: {e}"))
}

// ── A2 support: Rust-aware lexical stripper ──────────────────────────────────
//
// A2's whole point is that guard text (the `STATIC_TEST_FLOOR` identifier and a
// count-vs-floor comparison operator) must appear in EXECUTABLE code — not in a
// comment or a string literal. Attempt 2 only stripped `//` line comments and
// plain `"…"` strings; the reviewer correctly noted that a gutted body could
// still hide the guard text inside a `/* … */` block comment or a raw string
// (`r"…"`, `r#"…"#`). This scanner closes that vector: it removes
//   * `//` line comments              (dropped to end of line)
//   * `/* … */` block comments        (NESTING, as Rust allows)
//   * `"…"` normal strings            (with `\` escapes)
//   * `r"…"` / `r#"…"#` raw strings   (any number of `#`)
// replacing their contents with spaces and preserving newlines so line
// structure is intact. Char literals / lifetimes are left as-is — they cannot
// hold the multi-token guard text we search for, so ignoring them is safe.
// Byte strings (`b"…"`) are covered by the normal-string `"` path; byte raw
// strings (`br"…"`) by the raw-string path.
//
// Scanning is on bytes but UTF-8 safe: every delimiter compared is ASCII (and
// UTF-8 guarantees ASCII byte values never occur inside a multi-byte sequence),
// and the only place the `&str` is sliced (`src[i..]`) happens at positions that
// are always char boundaries (start of input, or just past an ASCII delimiter,
// or after copying a whole `char`).
fn strip_comments_and_strings(src: &str) -> String {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut out = String::with_capacity(n);
    let mut i = 0usize;
    while i < n {
        let c = bytes[i];

        // `//` line comment → drop to end of line (keep the newline).
        if c == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // `/* … */` block comment, with Rust-style nesting.
        if c == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            let mut depth = 1i32;
            i += 2;
            while i < n && depth > 0 {
                if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < n && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }

        // Raw string: optional `b`, then `r`, then `#*`, then `"`.
        if c == b'r' || (c == b'b' && i + 1 < n && bytes[i + 1] == b'r') {
            let r_pos = if c == b'b' { i + 1 } else { i };
            let mut j = r_pos + 1;
            let mut hashes = 0usize;
            while j < n && bytes[j] == b'#' {
                hashes += 1;
                j += 1;
            }
            if j < n && bytes[j] == b'"' {
                // Confirmed raw-string opening; content runs until a `"` followed
                // by exactly `hashes` `#`.
                i = j + 1;
                loop {
                    if i >= n {
                        break; // unterminated; bail gracefully
                    }
                    if bytes[i] == b'"' {
                        let mut k = i + 1;
                        let mut cnt = 0usize;
                        while k < n && cnt < hashes && bytes[k] == b'#' {
                            cnt += 1;
                            k += 1;
                        }
                        if cnt == hashes {
                            i = k; // past the closing delimiter
                            break;
                        }
                    }
                    out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
                continue;
            }
            // Not a raw string (e.g. identifier `read`/`body`): fall through.
        }

        // Normal string `"…"` (also consumes `b"…"` byte-string content).
        if c == b'"' {
            i += 1;
            while i < n {
                if bytes[i] == b'\\' {
                    out.push(' ');
                    if i + 1 < n {
                        out.push(' ');
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }

        // Default: copy one full UTF-8 char (i is on a char boundary here).
        let ch = src[i..]
            .chars()
            .next()
            .expect("i is on a char boundary in the default branch");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Extract the first-argument (predicate) text of every `assert!` /
/// `assert_eq!` / `assert_ne!` macro invocation in `exec` (which must already
/// be comment/string-stripped). The first argument runs from just after the
/// macro's opening `(` to the first top-level `,` (or the matching `)` for a
/// one-arg assert), tracking nested parens so an inner `,` does not truncate it.
fn extract_assert_predicates(exec: &str) -> Vec<String> {
    let macros: [&[u8]; 3] = [b"assert_eq!", b"assert_ne!", b"assert!"];
    let bytes = exec.as_bytes();
    let n = bytes.len();
    let mut preds = Vec::new();
    let mut i = 0usize;
    while i < n {
        // Match an assert-family macro name at i (byte-slice compare = UTF-8 safe).
        let mut macro_len = 0usize;
        for m in macros {
            if bytes[i..].starts_with(m) {
                macro_len = m.len();
                break;
            }
        }
        if macro_len == 0 {
            i += 1;
            continue;
        }
        // Skip whitespace between `!` and `(`.
        let mut j = i + macro_len;
        while j < n && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= n || bytes[j] != b'(' {
            i += macro_len;
            continue;
        }
        // Walk the argument list; capture the first arg.
        let arg_start = j + 1; // char boundary (just past ASCII '(')
        let mut depth = 1i32;
        let mut k = arg_start;
        let mut arg_end = arg_start;
        while k < n {
            match bytes[k] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        arg_end = k; // boundary (ASCII ')')
                        break;
                    }
                }
                b',' if depth == 1 => {
                    arg_end = k; // boundary (ASCII ',')
                    break;
                }
                _ => {}
            }
            k += 1;
            arg_end = k;
        }
        preds.push(exec[arg_start..arg_end.min(n)].to_string());
        i = arg_end.max(i + macro_len);
    }
    preds
}

/// The A2 verdict, factored out so the positive (real source) and negative
/// (synthetic spoof) tests share ONE implementation.
///
/// Returns `Ok(form)` iff `body` contains an EXECUTABLE count-vs-floor guard:
///   1. `STATIC_TEST_FLOOR` appears in executable (post-strip) code, AND
///   2. at least one `assert!`/`assert_eq!`/`assert_ne!` predicate itself
///      contains a count-vs-floor comparison.
///
/// This is the reviewer's hardened contract: the comparison must live INSIDE
/// the assert predicate. A bare `assert!(true)` plus a separate stray
/// `count >= STATIC_TEST_FLOOR` expression is rejected, and guard text that
/// survives only inside a comment or (raw) string is rejected.
fn a26_guard_verdict(body: &str) -> Result<String, String> {
    let exec = strip_comments_and_strings(body);

    if !exec.contains("STATIC_TEST_FLOOR") {
        return Err(
            "no executable STATIC_TEST_FLOOR reference (only comment/string mentions remain)"
                .to_string(),
        );
    }

    let preds = extract_assert_predicates(&exec);
    if preds.is_empty() {
        return Err("no assert!/assert_eq!/assert_ne! macro in executable body".to_string());
    }

    // Whitespace-insensitive comparison forms tying `count` to the floor.
    let forms = [
        "count>=STATIC_TEST_FLOOR",
        "count>STATIC_TEST_FLOOR",
        "STATIC_TEST_FLOOR<=count",
        "STATIC_TEST_FLOOR<count",
    ];
    for p in &preds {
        let norm: String = p.split_whitespace().collect();
        for f in forms {
            if norm.contains(f) {
                return Ok(f.to_string());
            }
        }
    }

    Err(format!(
        "no count-vs-floor comparison inside any assert predicate; predicates were {preds:?}"
    ))
}

// ── A1 ──────────────────────────────────────────────────────────────────────
// No nested-cargo subprocess anywhere in the modified file.
// Type A. threshold: each forbidden token occurs 0× across the WHOLE file.
#[test]
fn harness_count_guard_no_nested_cargo_subprocess() {
    let src = read_target_source();
    for tok in [
        "Command::new",
        "std::process::Command",
        "\"--list\"",
        "\"cargo\"",
    ] {
        let n = src.matches(tok).count();
        assert_eq!(
            n, 0,
            "A1: forbidden subprocess token {tok:?} found {n}× in \
             harness_p8_beta_memory_system.rs (must be 0 file-wide)"
        );
    }
}

// ── A2 ──────────────────────────────────────────────────────────────────────
// a26 still guards — the comparison lives INSIDE an assert predicate. Type A.
// threshold: STATIC_TEST_FLOOR present in executable code AND a count-vs-floor
// comparison present inside an assert predicate.
//
// Hardened (attempt 3): A2 now requires the comparison to be inside an actual
// assert predicate (via `a26_guard_verdict`), not merely "an assert somewhere"
// plus "a comparison somewhere". Comment/string/raw-string text is stripped
// first, so guard text hidden in a `/* … */` block comment, a `"…"` string, or
// an `r#"…"#` raw string cannot satisfy the check. The negative tests below
// pin both gaming vectors.
#[test]
fn harness_count_guard_a26_body_compares_against_floor() {
    let src = read_target_source();
    let body = isolate_a26_body(&src);
    match a26_guard_verdict(&body) {
        Ok(form) => eprintln!("A2: a26 executable guard predicate present: {form:?}"),
        Err(why) => panic!(
            "A2: a26 body lacks an executable count-vs-floor comparison inside an \
             assert predicate: {why}"
        ),
    }
}

// ── A2 negative #1 ───────────────────────────────────────────────────────────
// Guard text that survives ONLY inside a block comment / raw string must be
// REJECTED (the gut-the-body-to-comments vector).
#[test]
fn harness_count_guard_a2_rejects_comment_and_rawstring_spoof() {
    let spoof = r####"
fn harness_p8_beta_a26_test_count_regression_guard() {
    /* count >= STATIC_TEST_FLOOR — guard text only in a (nesting /* */) block comment */
    let _doc = r#"count >= STATIC_TEST_FLOOR lives only inside a raw string"#;
    let _ = 1 + 1;
}
"####;
    let verdict = a26_guard_verdict(spoof);
    assert!(
        verdict.is_err(),
        "A2 negative: a gutted body whose only guard text is in a block comment / \
         raw string must be REJECTED, got {verdict:?}"
    );
}

// ── A2 negative #2 ───────────────────────────────────────────────────────────
// `assert!(true)` plus a SEPARATE stray `count >= STATIC_TEST_FLOOR` expression
// must be REJECTED — the comparison must be INSIDE the assert predicate.
#[test]
fn harness_count_guard_a2_rejects_assert_true_plus_stray_comparison() {
    let spoof = r####"
fn harness_p8_beta_a26_test_count_regression_guard() {
    let count: u32 = 1700;
    assert!(true, "always passes: {count} vs {STATIC_TEST_FLOOR}");
    let _ = count >= STATIC_TEST_FLOOR; // stray comparison, NOT inside an assert
}
"####;
    let verdict = a26_guard_verdict(spoof);
    assert!(
        verdict.is_err(),
        "A2 negative: assert!(true) + a separate stray count-vs-floor expression \
         must be REJECTED (comparison must live inside the assert predicate), got {verdict:?}"
    );
}

// ── A2 positive control ──────────────────────────────────────────────────────
// A well-formed `assert!(count >= STATIC_TEST_FLOOR, …)` must be ACCEPTED —
// proves `a26_guard_verdict` is not vacuously always-Err.
#[test]
fn harness_count_guard_a2_accepts_wellformed_guard() {
    let good = r####"
fn harness_p8_beta_a26_test_count_regression_guard() {
    let count: u32 = walk_and_count();
    assert!(count >= STATIC_TEST_FLOOR, "silent removal: {count} < {STATIC_TEST_FLOOR}");
}
"####;
    let verdict = a26_guard_verdict(good);
    assert!(
        verdict.is_ok(),
        "A2 positive control: a well-formed assert!(count >= STATIC_TEST_FLOOR, …) \
         must be ACCEPTED, got {verdict:?}"
    );
}

// ── A4 ──────────────────────────────────────────────────────────────────────
// Independent #[test] count meets the floor, with per-crate traversal-integrity
// sub-checks. Type C. threshold: workspace >= 1600 AND sim-test >= 1000 AND
// sim-core >= 200.
#[test]
fn harness_count_guard_independent_count_meets_floor() {
    let crates = rust_root().join("crates");
    let total = count_workspace_tests(&crates);
    let sim_test = count_crate_tests(&crates.join("sim-test"));
    let sim_core = count_crate_tests(&crates.join("sim-core"));

    assert!(
        total >= 1600,
        "A4: workspace #[test] count {total} < floor 1600 (silent removal or broken walk)"
    );
    assert!(
        sim_test >= 1000,
        "A4: sim-test #[test] subtotal {sim_test} < 1000 (walk missed the sim-test crate)"
    );
    assert!(
        sim_core >= 200,
        "A4: sim-core #[test] subtotal {sim_core} < 200 (walk missed the sim-core crate)"
    );
}

// ── A5 ──────────────────────────────────────────────────────────────────────
// Floor const value is correct, against a fixed band (not coupled to live
// count). Type D. threshold: 1500 <= floor <= 1700 AND floor == 1600.
#[test]
fn harness_count_guard_floor_value_in_band() {
    let src = read_target_source();
    let floor = parse_static_test_floor(&src);
    assert!(
        (1500..=1700).contains(&floor),
        "A5: STATIC_TEST_FLOOR {floor} outside fixed band [1500, 1700]"
    );
    assert_eq!(
        floor, 1600,
        "A5: STATIC_TEST_FLOOR must equal the locked value 1600"
    );
}

// ── A6 ──────────────────────────────────────────────────────────────────────
// a26 function name preserved AND #[test] directly precedes its fn declaration.
// Type A. threshold: identifier present AND attribute directly attached.
#[test]
fn harness_count_guard_a26_name_and_attribute_preserved() {
    let src = read_target_source();
    assert!(
        src.contains("harness_p8_beta_a26_test_count_regression_guard"),
        "A6: a26 identifier missing (renamed or deleted)"
    );

    let lines: Vec<&str> = src.lines().collect();
    let fn_idx = lines
        .iter()
        .position(|l| {
            l.trim_start()
                .starts_with("fn harness_p8_beta_a26_test_count_regression_guard")
        })
        .expect("A6: a26 fn declaration not found");
    assert!(fn_idx >= 1, "A6: a26 fn cannot be the first line of the file");
    let prev = lines[fn_idx - 1].trim();
    assert_eq!(
        prev, "#[test]",
        "A6: line immediately preceding a26 fn must be `#[test]` (found {prev:?})"
    );
}
