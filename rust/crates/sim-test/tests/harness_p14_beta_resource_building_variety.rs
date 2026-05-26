//! V7 Phase 14-β — Resource + Building Variety harness.
//!
//! Static file-inspection harness verifying the Phase 14-β implementation:
//!   - `scripts/ui/world_renderer.gd` — 5 resource type paths +
//!     RESOURCE_WATER_TINT + 4 village fixture paths + 4 village fixture
//!     positions + Z_VILLAGE_FIXTURE + updated resource scatter loop +
//!     new village fixture placement loop.
//!   - Anti-regression guards for Phase 13-β, Phase 13-α/ε, Phase 12-β.2,
//!     Phase 12-γ, and Phase 14-α invariants.
//!   - Substrate verification: new sprite assets exist on disk + PNG magic.
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_beta_resource_building_variety -- --nocapture

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

/// Find the array-literal body of a multi-line `const NAME: Array = [...]`
/// declaration in the (stripped) source. Returns the content between `[`
/// and the matching `]` (NOT including the brackets themselves). Handles
/// nested brackets and parenthesised entries like `Vector2i(28, 28)` so the
/// returned span is exactly the contiguous literal body.
fn find_const_array_body<'a>(stripped: &'a str, ident: &str) -> Option<&'a str> {
    let needle_a = format!("const {ident}:");
    let needle_b = format!("const {ident} :");
    let needle_c = format!("const {ident}=");
    let needle_d = format!("const {ident} =");
    let mut decl_pos: Option<usize> = None;
    for n in [&needle_a, &needle_b, &needle_c, &needle_d] {
        if let Some(p) = stripped.find(n.as_str()) {
            decl_pos = Some(p);
            break;
        }
    }
    let decl_pos = decl_pos?;
    let after = &stripped[decl_pos..];
    let open_rel = after.find('[')?;
    let open_abs = decl_pos + open_rel;
    let bytes = stripped.as_bytes();
    let mut depth_sq = 1i32;
    let mut depth_paren = 0i32;
    let mut k = open_abs + 1;
    while k < bytes.len() && depth_sq > 0 {
        match bytes[k] as char {
            '[' => depth_sq += 1,
            ']' => depth_sq -= 1,
            '(' => depth_paren += 1,
            ')' => depth_paren -= 1,
            _ => {}
        }
        if depth_sq == 0 {
            break;
        }
        k += 1;
    }
    if depth_sq != 0 || depth_paren != 0 {
        return None;
    }
    Some(&stripped[open_abs + 1..k])
}

/// Split an array literal body on top-level commas (ignoring commas inside
/// parens / brackets). Returns trimmed non-empty entries.
fn split_top_level_commas(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut depth_paren = 0i32;
    let mut depth_sq = 0i32;
    let mut in_str: Option<char> = None;
    for c in body.chars() {
        match in_str {
            Some(q) => {
                buf.push(c);
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '"' | '\'' => {
                    in_str = Some(c);
                    buf.push(c);
                }
                '(' => {
                    depth_paren += 1;
                    buf.push(c);
                }
                ')' => {
                    depth_paren -= 1;
                    buf.push(c);
                }
                '[' => {
                    depth_sq += 1;
                    buf.push(c);
                }
                ']' => {
                    depth_sq -= 1;
                    buf.push(c);
                }
                ',' if depth_paren == 0 && depth_sq == 0 => {
                    let trimmed = buf.trim();
                    if !trimmed.is_empty() {
                        out.push(trimmed.to_string());
                    }
                    buf.clear();
                }
                _ => buf.push(c),
            },
        }
    }
    let trimmed = buf.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
    out
}

/// Verify a PNG file exists, is non-empty, and starts with the PNG magic.
fn assert_png_at(path: &PathBuf, label: &str) {
    assert!(path.is_file(), "{label}: file must exist at {path:?}");
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("{label}: read {path:?}: {e}"));
    assert!(bytes.len() >= 8, "{label}: too short for PNG magic; got {} bytes", bytes.len());
    let png_magic: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(
        &bytes[..8],
        &png_magic,
        "{label}: must start with PNG magic; got first 8 bytes = {:02X?}",
        &bytes[..8]
    );
}

/// Extract the resource scatter block — from the `for i in RESOURCE_COUNT`
/// (or `for _i in RESOURCE_COUNT`) loop header to the first
/// `add_child(res_sprite)` after it (inclusive of that call).
fn extract_resource_loop_block(stripped: &str) -> Option<&str> {
    // Try indexed form first per spec; fall back to `for _i` so the assertion
    // can produce a meaningful failure message before the loop is updated.
    let header_pos = stripped
        .find("for i in RESOURCE_COUNT")
        .or_else(|| stripped.find("for _i in RESOURCE_COUNT"))?;
    let after = &stripped[header_pos..];
    let close_rel = after.find("add_child(res_sprite)")?;
    let close_abs = header_pos + close_rel + "add_child(res_sprite)".len();
    Some(&stripped[header_pos..close_abs])
}

/// Compute the leading-whitespace prefix (tabs+spaces) of the line containing
/// absolute byte offset `pos`. Returns the slice between the line start and
/// the first non-whitespace byte.
fn line_indent_at(stripped: &str, pos: usize) -> &str {
    let bytes = stripped.as_bytes();
    let mut start = pos;
    while start > 0 && bytes[start - 1] != b'\n' {
        start -= 1;
    }
    let mut end = start;
    while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
        end += 1;
    }
    &stripped[start..end]
}

/// Extract the contiguous block of lines starting at the header line at
/// `header_pos` and continuing while subsequent lines are blank OR have
/// strictly greater leading whitespace than the header line. Returns the
/// substring from the header line start through the last included line's
/// terminating newline.
fn extract_indented_block(stripped: &str, header_pos: usize) -> &str {
    let bytes = stripped.as_bytes();
    // Move to start of header line.
    let mut line_start = header_pos;
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    let header_indent = line_indent_at(stripped, header_pos);
    let header_indent_len = header_indent.len();
    // Find end of header line.
    let mut cursor = header_pos;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        cursor += 1;
    }
    if cursor < bytes.len() {
        cursor += 1; // include the newline
    }
    // Now iterate subsequent lines.
    let mut block_end = cursor;
    while cursor < bytes.len() {
        let next_line_start = cursor;
        // Find next newline / end.
        let mut next_nl = cursor;
        while next_nl < bytes.len() && bytes[next_nl] != b'\n' {
            next_nl += 1;
        }
        // Get this line's indent + first non-ws byte.
        let mut first_non_ws = next_line_start;
        while first_non_ws < next_nl
            && (bytes[first_non_ws] == b' ' || bytes[first_non_ws] == b'\t')
        {
            first_non_ws += 1;
        }
        let is_blank = first_non_ws == next_nl;
        let this_indent_len = first_non_ws - next_line_start;
        if !is_blank && this_indent_len <= header_indent_len {
            break;
        }
        // Include this line.
        cursor = if next_nl < bytes.len() { next_nl + 1 } else { next_nl };
        block_end = cursor;
    }
    &stripped[line_start..block_end]
}

// ─── Assertion 1: resource_type_paths_array_declared ─────────────────────
#[test]
fn harness_p14_beta_a1_resource_type_paths_array_declared() {
    // Type: A — Static structural invariant. Plan threshold: declaration must
    // exist AND parse successfully (== 1 match for the declaration token).
    // We count occurrences of any of the canonical declaration-token forms in
    // the stripped (comment-stripped) source and require exactly one. This
    // defends against future accidental duplicates (e.g. dev shadow definition
    // somewhere lower in the file) that would still individually parse.
    let stripped = strip_gd_comments(&read_world_renderer_src());

    // Count declaration-token occurrences. ONLY the typed-annotation form
    // `: Array = [` (with whitespace variants around the colon and equals
    // sign) is accepted. The inferred-type walrus form `:= [` is REJECTED
    // because the plan threshold mandates the explicit `Array` type tag
    // (this protects against accidental typing of the array to a different
    // base or wholesale removal of the type annotation).
    let decl_tokens = [
        "const RESOURCE_TYPE_PATHS: Array = [",
        "const RESOURCE_TYPE_PATHS:Array=[",
        "const RESOURCE_TYPE_PATHS: Array=[",
        "const RESOURCE_TYPE_PATHS:Array = [",
        "const RESOURCE_TYPE_PATHS :Array= [",
        "const RESOURCE_TYPE_PATHS : Array = [",
        "const RESOURCE_TYPE_PATHS : Array=[",
        "const RESOURCE_TYPE_PATHS :Array=[",
        "const RESOURCE_TYPE_PATHS : Array =[",
        "const RESOURCE_TYPE_PATHS:Array =[",
    ];
    let total_token_hits: usize = decl_tokens
        .iter()
        .map(|tok| stripped.matches(tok).count())
        .sum();
    assert_eq!(
        total_token_hits, 1,
        "A1.1: exactly ONE typed `const RESOURCE_TYPE_PATHS: Array = [` declaration token \
         must exist in the stripped source; the inferred-type `:=` form is REJECTED. \
         Found {total_token_hits} matches across the canonical typed declaration forms \
         {decl_tokens:?}"
    );

    // A1.1b: explicitly reject the walrus / inferred-type forms so the
    // failure message is clear if a future edit drops the `: Array` tag.
    let walrus_forms = [
        "const RESOURCE_TYPE_PATHS := [",
        "const RESOURCE_TYPE_PATHS:=[",
        "const RESOURCE_TYPE_PATHS :=[",
        "const RESOURCE_TYPE_PATHS:= [",
    ];
    let walrus_hits: usize = walrus_forms
        .iter()
        .map(|tok| stripped.matches(tok).count())
        .sum();
    assert_eq!(
        walrus_hits, 0,
        "A1.1b: the inferred-type `const RESOURCE_TYPE_PATHS := [` form is FORBIDDEN; \
         the plan requires the explicit `: Array =` type annotation. Found {walrus_hits} \
         walrus matches across forms {walrus_forms:?}"
    );

    // Cross-check via the existing decl-RHS extractor.
    let rhss = find_decl_rhss(&stripped, "RESOURCE_TYPE_PATHS");
    assert_eq!(
        rhss.len(),
        1,
        "A1.2: cross-check — find_decl_rhss must also yield exactly 1 \
         RESOURCE_TYPE_PATHS declaration; got {n}: {rhss:?}",
        n = rhss.len()
    );

    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A1.3: `const RESOURCE_TYPE_PATHS: Array = [...]` must parse successfully");
    let entries = split_top_level_commas(body);
    assert!(
        !entries.is_empty(),
        "A1.4: RESOURCE_TYPE_PATHS array literal must contain at least one entry; got 0"
    );
    println!(
        "[P14-β A1] RESOURCE_TYPE_PATHS declared exactly once with {} entries ✓",
        entries.len()
    );
}

// ─── Assertion 2: resource_type_paths_has_exactly_five_entries ───────────
#[test]
fn harness_p14_beta_a2_resource_type_paths_has_exactly_five_entries() {
    // Type: A — Spec mandates exactly 5 resource types.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A2.1: RESOURCE_TYPE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert_eq!(
        entries.len(),
        5,
        "A2.2: RESOURCE_TYPE_PATHS must contain EXACTLY 5 entries; got {n}: {entries:?}",
        n = entries.len()
    );
    println!("[P14-β A2] RESOURCE_TYPE_PATHS has exactly 5 entries ✓");
}

// ─── Assertion 3: resource_type_paths_index_0_references_legacy_constant ─
#[test]
fn harness_p14_beta_a3_resource_type_paths_index_0_references_legacy_constant() {
    // Type: A — Spec requires Berry slot to reuse RESOURCE_SPRITE_PATH by
    // reference (NOT inlined as a string literal).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A3.1: RESOURCE_TYPE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert!(
        !entries.is_empty(),
        "A3.2: RESOURCE_TYPE_PATHS must have at least 1 entry"
    );
    let entry0 = &entries[0];
    assert_eq!(
        entry0, "RESOURCE_SPRITE_PATH",
        "A3.3: RESOURCE_TYPE_PATHS[0] must be the bare identifier `RESOURCE_SPRITE_PATH` \
         (NOT the inlined string literal); got `{entry0}`"
    );
    println!("[P14-β A3] RESOURCE_TYPE_PATHS[0] = RESOURCE_SPRITE_PATH (by reference) ✓");
}

// ─── Assertion 4: resource_type_paths_index_1_wood_literal ───────────────
#[test]
fn harness_p14_beta_a4_resource_type_paths_index_1_wood_literal() {
    // Type: A — Spec locks Wood to workbench/1.png exact literal.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A4.1: RESOURCE_TYPE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert!(entries.len() >= 2, "A4.2: need entry at index 1");
    let entry1 = &entries[1];
    assert_eq!(
        entry1, "\"res://assets/sprites/furniture/workbench/1.png\"",
        "A4.3: RESOURCE_TYPE_PATHS[1] must equal exact Wood literal; got `{entry1}`"
    );
    println!("[P14-β A4] RESOURCE_TYPE_PATHS[1] = workbench/1.png ✓");
}

// ─── Assertion 5: resource_type_paths_index_2_stone_literal ──────────────
#[test]
fn harness_p14_beta_a5_resource_type_paths_index_2_stone_literal() {
    // Type: A — Spec locks Stone to walls/limestone/1.png.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A5.1: RESOURCE_TYPE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert!(entries.len() >= 3, "A5.2: need entry at index 2");
    let entry2 = &entries[2];
    assert_eq!(
        entry2, "\"res://assets/sprites/walls/limestone/1.png\"",
        "A5.3: RESOURCE_TYPE_PATHS[2] must equal exact Stone literal; got `{entry2}`"
    );
    println!("[P14-β A5] RESOURCE_TYPE_PATHS[2] = walls/limestone/1.png ✓");
}

// ─── Assertion 6: resource_type_paths_index_3_water_literal ──────────────
#[test]
fn harness_p14_beta_a6_resource_type_paths_index_3_water_literal() {
    // Type: A — Spec locks Water to floors/stone_slab/1.png.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A6.1: RESOURCE_TYPE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert!(entries.len() >= 4, "A6.2: need entry at index 3");
    let entry3 = &entries[3];
    assert_eq!(
        entry3, "\"res://assets/sprites/floors/stone_slab/1.png\"",
        "A6.3: RESOURCE_TYPE_PATHS[3] must equal exact Water literal; got `{entry3}`"
    );
    println!("[P14-β A6] RESOURCE_TYPE_PATHS[3] = floors/stone_slab/1.png ✓");
}

// ─── Assertion 7: resource_type_paths_index_4_food_literal ───────────────
#[test]
fn harness_p14_beta_a7_resource_type_paths_index_4_food_literal() {
    // Type: A — Spec locks Food to hearth/2.png (variant 2).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "RESOURCE_TYPE_PATHS")
        .expect("A7.1: RESOURCE_TYPE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert!(entries.len() >= 5, "A7.2: need entry at index 4");
    let entry4 = &entries[4];
    assert_eq!(
        entry4, "\"res://assets/sprites/furniture/hearth/2.png\"",
        "A7.3: RESOURCE_TYPE_PATHS[4] must equal exact Food literal; got `{entry4}`"
    );
    println!("[P14-β A7] RESOURCE_TYPE_PATHS[4] = furniture/hearth/2.png ✓");
}

// ─── Assertion 8: resource_water_tint_constant_value ─────────────────────
#[test]
fn harness_p14_beta_a8_resource_water_tint_constant_value() {
    // Type: A — Spec locks Color(0.45, 0.65, 1.0, 1.0).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "RESOURCE_WATER_TINT", "A8.1");
    let compact = no_ws(&rhs);
    // Accept the canonical literal exactly. Float-component equality is
    // string-level here (since we are comparing source text).
    let accepted = [
        "Color(0.45,0.65,1.0,1.0)",
        "Color(0.45,0.65,1,1)",
        "Color(0.45,0.65,1.0,1)",
        "Color(0.45,0.65,1,1.0)",
    ];
    assert!(
        accepted.contains(&compact.as_str()),
        "A8.2: RESOURCE_WATER_TINT must equal Color(0.45, 0.65, 1.0, 1.0); got `{rhs}`"
    );
    println!("[P14-β A8] RESOURCE_WATER_TINT = Color(0.45, 0.65, 1.0, 1.0) ✓");
}

// ─── Assertion 9: resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator
#[test]
fn harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator() {
    // Type: A — Loop header must be `for i in RESOURCE_COUNT` (indexed) AND
    // the body must contain `i % RESOURCE_TYPE_PATHS.size()` or `i % 5`.
    // Sub-assertion A9.4: the loop body must include the `if rtex == null:`
    // / `continue` null-guard BEFORE `add_child(res_sprite)` — otherwise a
    // failed `load()` would silently produce a null-texture sprite (or crash).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let block = extract_resource_loop_block(&stripped)
        .expect("A9.1: resource scatter loop block must exist");
    assert!(
        block.starts_with("for i in RESOURCE_COUNT"),
        "A9.2: loop header must be `for i in RESOURCE_COUNT` (NOT `for _i in ...`); \
         block prefix=`{}`",
        &block[..block.len().min(40)]
    );
    let compact = no_ws(block);
    let has_size_form = compact.contains("i%RESOURCE_TYPE_PATHS.size()");
    let has_literal_form = compact.contains("i%5");
    assert!(
        has_size_form || has_literal_form,
        "A9.3: loop body must contain `i % RESOURCE_TYPE_PATHS.size()` or `i % 5`; block=\n{block}"
    );

    // A9.4: null guard. Find positions of `if rtex == null:`, the following
    // `continue`, and `add_child(res_sprite)`. The three must appear in that
    // order so the null-texture iteration is skipped before the sprite is
    // added to the scene tree.
    let null_check_pos = compact.find("ifrtex==null:").unwrap_or_else(|| {
        panic!(
            "A9.4: loop body must contain `if rtex == null:` null-guard; block=\n{block}"
        )
    });
    let after_null = &compact[null_check_pos..];
    let continue_rel = after_null.find("continue").unwrap_or_else(|| {
        panic!(
            "A9.5: a `continue` must appear after `if rtex == null:` in the loop body; \
             block=\n{block}"
        )
    });
    let continue_abs = null_check_pos + continue_rel;
    let add_child_abs = compact.find("add_child(res_sprite)").unwrap_or_else(|| {
        panic!(
            "A9.6: `add_child(res_sprite)` must appear in the loop body; block=\n{block}"
        )
    });
    assert!(
        continue_abs < add_child_abs,
        "A9.7: the `continue` (offset {continue_abs}) must come BEFORE \
         `add_child(res_sprite)` (offset {add_child_abs}) so null textures are \
         skipped; block=\n{block}"
    );
    println!(
        "[P14-β A9] `for i in RESOURCE_COUNT` + modulo cycle + null-guard `if rtex == null: continue` ✓"
    );
}

// ─── Assertion 10: water_modulate_branch_present ─────────────────────────
#[test]
fn harness_p14_beta_a10_water_modulate_branch_present() {
    // Type: A — Water (index 3) must receive RESOURCE_WATER_TINT modulate.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let block = extract_resource_loop_block(&stripped)
        .expect("A10.1: resource scatter loop block must exist");
    let compact = no_ws(block);

    let branch_forms = [
        "type_idx==3",
        "i%5==3",
        "i%RESOURCE_TYPE_PATHS.size()==3",
    ];
    let branch_pos = branch_forms
        .iter()
        .filter_map(|n| compact.find(n).map(|p| (p, *n)))
        .min_by_key(|t| t.0);
    let (branch_pos, branch_form) = branch_pos.unwrap_or_else(|| {
        panic!(
            "A10.2: loop body must contain one of `type_idx == 3` / `i % 5 == 3` / \
             `i % RESOURCE_TYPE_PATHS.size() == 3`; block=\n{block}"
        )
    });
    let after = &compact[branch_pos..];
    assert!(
        after.contains("RESOURCE_WATER_TINT"),
        "A10.3: `RESOURCE_WATER_TINT` must appear AFTER the `{branch_form}` branch; \
         compact-after=`{after}`"
    );
    println!("[P14-β A10] water modulate branch (`{branch_form}` → RESOURCE_WATER_TINT) ✓");
}

// ─── Assertion 11: village_fixture_paths_has_exactly_four_literals ───────
#[test]
fn harness_p14_beta_a11_village_fixture_paths_has_exactly_four_literals() {
    // Type: A — Spec locks each fixture path and its index.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "VILLAGE_FIXTURE_PATHS")
        .expect("A11.1: VILLAGE_FIXTURE_PATHS array must parse");
    let entries = split_top_level_commas(body);
    assert_eq!(
        entries.len(),
        4,
        "A11.2: VILLAGE_FIXTURE_PATHS must contain EXACTLY 4 entries; got {n}: {entries:?}",
        n = entries.len()
    );
    let want = [
        "\"res://assets/sprites/furniture/workbench/2.png\"",
        "\"res://assets/sprites/furniture/drying_rack/1.png\"",
        "\"res://assets/sprites/furniture/lean_to/1.png\"",
        "\"res://assets/sprites/furniture/storage_pit/2.png\"",
    ];
    for (i, want_v) in want.iter().enumerate() {
        assert_eq!(
            &entries[i], want_v,
            "A11.3: VILLAGE_FIXTURE_PATHS[{i}] must equal `{want_v}`; got `{e}`",
            e = entries[i]
        );
    }
    println!("[P14-β A11] VILLAGE_FIXTURE_PATHS = 4 exact fixture literals ✓");
}

// ─── Assertion 12: village_fixture_positions_match_diamond_layout ────────
#[test]
fn harness_p14_beta_a12_village_fixture_positions_match_diamond_layout() {
    // Type: A — Spec locks the diamond positions around (32, 32).
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let body = find_const_array_body(&stripped, "VILLAGE_FIXTURE_POSITIONS")
        .expect("A12.1: VILLAGE_FIXTURE_POSITIONS array must parse");
    let entries = split_top_level_commas(body);
    assert_eq!(
        entries.len(),
        4,
        "A12.2: VILLAGE_FIXTURE_POSITIONS must contain EXACTLY 4 entries; got {n}: {entries:?}",
        n = entries.len()
    );
    let want = [
        "Vector2i(28,28)",
        "Vector2i(36,28)",
        "Vector2i(28,36)",
        "Vector2i(36,36)",
    ];
    for (i, want_v) in want.iter().enumerate() {
        let got = no_ws(&entries[i]);
        assert_eq!(
            got, *want_v,
            "A12.3: VILLAGE_FIXTURE_POSITIONS[{i}] must equal `{want_v}`; got `{e}`",
            e = entries[i]
        );
    }
    println!("[P14-β A12] VILLAGE_FIXTURE_POSITIONS = 4 exact diamond entries ✓");
}

// ─── Assertion 13: z_village_fixture_equals_five ─────────────────────────
#[test]
fn harness_p14_beta_a13_z_village_fixture_equals_five() {
    // Type: A — Spec mandates Z=5 to share the ConstructionSite plane.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "Z_VILLAGE_FIXTURE", "A13.1");
    assert_eq!(
        rhs, "5",
        "A13.2: Z_VILLAGE_FIXTURE must equal `5`; got `{rhs}`"
    );
    println!("[P14-β A13] Z_VILLAGE_FIXTURE = 5 ✓");
}

// ─── Assertion 14: village_fixture_placement_loop_present ────────────────
#[test]
fn harness_p14_beta_a14_village_fixture_placement_loop_present() {
    // Type: A — Single placement pass referencing BOTH arrays with the same
    // index variable and calling `add_child(...)` in the body. The loop body
    // is bounded by indentation (not "rest of the file") so we don't pick up
    // references from unrelated later code.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let header_candidates = [
        "for fi in VILLAGE_FIXTURE_PATHS.size()",
        "for fi in 4",
    ];
    // A14.1: count placement-loop headers in the whole stripped source.
    // Exactly one is required.
    let total_hits: usize = header_candidates
        .iter()
        .map(|n| stripped.matches(n).count())
        .sum();
    assert_eq!(
        total_hits, 1,
        "A14.1: exactly ONE village fixture placement loop header must exist \
         (`for fi in VILLAGE_FIXTURE_PATHS.size()` or `for fi in 4`); found {total_hits} \
         total matches"
    );
    let (start, header) = header_candidates
        .iter()
        .filter_map(|n| stripped.find(n).map(|p| (p, *n)))
        .min_by_key(|t| t.0)
        .unwrap_or_else(|| {
            panic!(
                "A14.2: village fixture placement loop must use header \
                 `for fi in VILLAGE_FIXTURE_PATHS.size()` or `for fi in 4`; not found"
            )
        });

    // A14.3: bound the loop body by indentation. Take only lines strictly
    // more indented than the header line (plus blank lines). This prevents
    // references from later code (e.g. `_update_settlement_furniture`) from
    // falsely satisfying the body-content checks.
    let bounded = extract_indented_block(&stripped, start);

    // Strip the header line itself so the assertions check the BODY only,
    // not the header (which already mentions `VILLAGE_FIXTURE_PATHS` in its
    // `.size()` form).
    let header_nl = bounded.find('\n').unwrap_or(bounded.len());
    let body_only = if header_nl < bounded.len() {
        &bounded[header_nl + 1..]
    } else {
        ""
    };

    assert!(
        body_only.contains("VILLAGE_FIXTURE_PATHS[fi]")
            || body_only.contains("VILLAGE_FIXTURE_PATHS [fi]"),
        "A14.4: loop body must reference `VILLAGE_FIXTURE_PATHS[fi]`; header=`{header}`; \
         body=\n{body_only}"
    );
    assert!(
        body_only.contains("VILLAGE_FIXTURE_POSITIONS[fi]")
            || body_only.contains("VILLAGE_FIXTURE_POSITIONS [fi]"),
        "A14.5: loop body must reference `VILLAGE_FIXTURE_POSITIONS[fi]`; header=`{header}`; \
         body=\n{body_only}"
    );
    assert!(
        body_only.contains("add_child(fixture_sprite)"),
        "A14.6: loop body must call `add_child(fixture_sprite)`; header=`{header}`; \
         body=\n{body_only}"
    );
    println!("[P14-β A14] exactly one village fixture placement loop (`{header}`) ✓");
}

// ─── Assertion 15: phase_13_beta_resource_constants_preserved ────────────
#[test]
fn harness_p14_beta_a15_phase_13_beta_resource_constants_preserved() {
    // Type: D — Regression guard against Phase 13-β harness.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let path = unique_decl_rhs(&stripped, "RESOURCE_SPRITE_PATH", "A15.1");
    assert_eq!(
        path, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A15.2: RESOURCE_SPRITE_PATH must remain exactly `storage_pit/1.png`; got `{path}`"
    );
    let count = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A15.3");
    assert_eq!(count, "20", "A15.4: RESOURCE_COUNT must be 20; got `{count}`");
    let seed = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A15.5");
    assert_eq!(
        seed, "88675123",
        "A15.6: RESOURCE_SEED must be 88675123; got `{seed}`"
    );
    let z = unique_decl_rhs(&stripped, "Z_RESOURCE", "A15.7");
    assert_eq!(z, "3", "A15.8: Z_RESOURCE must be 3; got `{z}`");
    println!("[P14-β A15] Phase 13-β resource constants preserved ✓");
}

// ─── Assertion 16: phase_13_alpha_bootstrap_constants_preserved ──────────
#[test]
fn harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved() {
    // Type: D — Regression guard for Phase 13-α + 13-ε bootstrap row.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let path = unique_decl_rhs(&stripped, "BUILDING_SPRITE_PATH", "A16.1");
    assert_eq!(
        path, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A16.2: BUILDING_SPRITE_PATH must remain exact campfire literal; got `{path}`"
    );
    let bx = unique_decl_rhs(&stripped, "BOOTSTRAP_X", "A16.3");
    let by = unique_decl_rhs(&stripped, "BOOTSTRAP_Y", "A16.4");
    let bxl = unique_decl_rhs(&stripped, "BOOTSTRAP_X_LEFT", "A16.5");
    let bxr = unique_decl_rhs(&stripped, "BOOTSTRAP_X_RIGHT", "A16.6");
    assert_eq!(bx, "32", "A16.7: BOOTSTRAP_X must be 32; got `{bx}`");
    assert_eq!(by, "32", "A16.8: BOOTSTRAP_Y must be 32; got `{by}`");
    assert_eq!(bxl, "24", "A16.9: BOOTSTRAP_X_LEFT must be 24; got `{bxl}`");
    assert_eq!(
        bxr, "40",
        "A16.10: BOOTSTRAP_X_RIGHT must be 40; got `{bxr}`"
    );
    println!("[P14-β A16] Phase 13-α/ε bootstrap constants preserved ✓");
}

// ─── Assertion 17: phase_12_construction_constants_preserved ─────────────
#[test]
fn harness_p14_beta_a17_phase_12_construction_constants_preserved() {
    // Type: D — Regression guard for Phase 12-β.2 A3. Exact literal — NOT
    // .contains(); the `buildings/` directory is the canonical path.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let path = unique_decl_rhs(&stripped, "CONSTRUCTION_SPRITE_PATH", "A17.1");
    assert_eq!(
        path, "\"res://assets/sprites/buildings/cairn/1.png\"",
        "A17.2: CONSTRUCTION_SPRITE_PATH must equal EXACT `buildings/cairn/1.png` literal; \
         got `{path}`"
    );
    let z = unique_decl_rhs(&stripped, "Z_CONSTRUCTION", "A17.3");
    assert_eq!(z, "5", "A17.4: Z_CONSTRUCTION must be 5; got `{z}`");
    println!("[P14-β A17] Phase 12 construction constants preserved (exact literal) ✓");
}

// ─── Assertion 18: phase_12_gamma_furniture_constants_preserved ──────────
#[test]
fn harness_p14_beta_a18_phase_12_gamma_furniture_constants_preserved() {
    // Type: D — Regression guard for Phase 12-γ. hearth/1.png and hearth/2.png
    // are distinct variants — must remain so.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let path = unique_decl_rhs(&stripped, "FURNITURE_SPRITE_PATH", "A18.1");
    assert_eq!(
        path, "\"res://assets/sprites/furniture/hearth/1.png\"",
        "A18.2: FURNITURE_SPRITE_PATH must equal exact `furniture/hearth/1.png`; got `{path}`"
    );
    let z = unique_decl_rhs(&stripped, "Z_FURNITURE", "A18.3");
    assert_eq!(z, "4", "A18.4: Z_FURNITURE must be 4; got `{z}`");
    println!("[P14-β A18] Phase 12-γ furniture constants preserved ✓");
}

// ─── Assertion 19: phase_14_alpha_agent_renderer_constants_preserved ─────
#[test]
fn harness_p14_beta_a19_phase_14_alpha_agent_renderer_constants_preserved() {
    // Type: D — Phase 14-α scope-discipline guard. β must not touch
    // agent_renderer.gd at all.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let bucket = unique_decl_rhs(&stripped, "ROLE_BUCKET_COUNT", "A19.1");
    assert_eq!(
        bucket, "4",
        "A19.2: ROLE_BUCKET_COUNT must remain 4; got `{bucket}`"
    );
    let icon = unique_decl_rhs(&stripped, "ICON_OFFSET_PX", "A19.3");
    assert_eq!(
        no_ws(&icon),
        "Vector2(0,-12)",
        "A19.4: ICON_OFFSET_PX must remain Vector2(0, -12); got `{icon}`"
    );
    println!("[P14-β A19] Phase 14-α agent_renderer constants preserved ✓");
}

// ─── Assertion 20: new_resource_sprite_assets_exist_on_disk ──────────────
#[test]
fn harness_p14_beta_a20_new_resource_sprite_assets_exist_on_disk() {
    // Type: A — Substrate verification: 4 new resource sprite assets +
    // PNG magic.
    let root = project_root();
    let cases: [(&str, &[&str]); 4] = [
        ("Wood", &["assets", "sprites", "furniture", "workbench", "1.png"]),
        ("Stone", &["assets", "sprites", "walls", "limestone", "1.png"]),
        ("Water", &["assets", "sprites", "floors", "stone_slab", "1.png"]),
        ("Food", &["assets", "sprites", "furniture", "hearth", "2.png"]),
    ];
    for (label, segs) in cases.iter() {
        let mut path = root.clone();
        for s in *segs {
            path.push(s);
        }
        assert_png_at(&path, &format!("A20[{label}]"));
    }
    println!("[P14-β A20] all 4 new resource sprite assets exist + PNG magic OK ✓");
}

// ─── Assertion 21: new_village_fixture_assets_exist_on_disk ──────────────
#[test]
fn harness_p14_beta_a21_new_village_fixture_assets_exist_on_disk() {
    // Type: A — Substrate verification: 4 new village fixture assets +
    // PNG magic.
    let root = project_root();
    let cases: [(&str, &[&str]); 4] = [
        ("Workshop", &["assets", "sprites", "furniture", "workbench", "2.png"]),
        ("Drying", &["assets", "sprites", "furniture", "drying_rack", "1.png"]),
        ("Shelter", &["assets", "sprites", "furniture", "lean_to", "1.png"]),
        ("Storage", &["assets", "sprites", "furniture", "storage_pit", "2.png"]),
    ];
    for (label, segs) in cases.iter() {
        let mut path = root.clone();
        for s in *segs {
            path.push(s);
        }
        assert_png_at(&path, &format!("A21[{label}]"));
    }
    println!("[P14-β A21] all 4 new village fixture assets exist + PNG magic OK ✓");
}

// ─── Assertion 22: no_rust_crate_modifications ───────────────────────────
#[test]
fn harness_p14_beta_a22_no_rust_crate_modifications() {
    // Type: A — `--quick` lane scope guard. Zero `.rs` changes inside
    // sim-core / sim-systems / sim-engine / sim-bridge / sim-data,
    // INCLUDING untracked new files (a Generator dropping a fresh
    // `rust/crates/sim-core/src/foo.rs` would be invisible to
    // `git diff HEAD` but is still a lane violation).
    //
    // The new harness file under `rust/crates/sim-test/tests/` is NOT
    // counted (sim-test is permitted).
    //
    // Detection sources, unioned:
    //   1. `git diff --name-only HEAD --`            → tracked modifications
    //   2. `git ls-files --others --exclude-standard --` → untracked new files
    //
    // V7 Phase 14-γ amendment (2026-05-26): `--full` lane features
    // (e.g. Phase 14-γ collect_agent_detail FFI) legitimately touch
    // sim-bridge. When `HARNESS_LANE=full` is set in the environment,
    // skip this guard — the lane choice authorises the change and the
    // pipeline's Evaluator independently reviews it.
    if std::env::var("HARNESS_LANE").as_deref() == Ok("full") {
        println!("[P14-β A22] HARNESS_LANE=full active — lane-discipline guard skipped");
        return;
    }
    let root = project_root();
    let forbidden_prefixes = [
        "rust/crates/sim-core/",
        "rust/crates/sim-systems/",
        "rust/crates/sim-engine/",
        "rust/crates/sim-bridge/",
        "rust/crates/sim-data/",
    ];

    let run_git = |args: &[&str]| -> Option<String> {
        match Command::new("git").args(args).current_dir(&root).output() {
            Ok(o) => Some(String::from_utf8_lossy(&o.stdout).into_owned()),
            Err(e) => {
                eprintln!("A22: git unavailable for `git {args:?}` ({e}); skipping that source");
                None
            }
        }
    };

    let mut any_source_ran = false;
    let mut candidate_paths: Vec<String> = Vec::new();

    // Source 1: tracked modifications vs HEAD.
    if let Some(diff_out) = run_git(&["diff", "--name-only", "HEAD", "--"]) {
        any_source_ran = true;
        for raw in diff_out.lines() {
            let line = raw.trim();
            if !line.is_empty() {
                candidate_paths.push(line.to_string());
            }
        }
    }

    // Source 2: untracked, non-gitignored new files.
    if let Some(untracked_out) = run_git(&["ls-files", "--others", "--exclude-standard", "--"]) {
        any_source_ran = true;
        for raw in untracked_out.lines() {
            let line = raw.trim();
            if !line.is_empty() {
                candidate_paths.push(line.to_string());
            }
        }
    }

    if !any_source_ran {
        // No git data at all — downgrade to a skip-with-warning so the
        // harness still runs from a tarball checkout.
        eprintln!("A22: no git sources available; skipping");
        return;
    }

    // Dedupe and filter to forbidden `.rs` paths.
    candidate_paths.sort();
    candidate_paths.dedup();
    let mut offenders: Vec<String> = Vec::new();
    for line in candidate_paths.iter() {
        if !line.ends_with(".rs") {
            continue;
        }
        for prefix in forbidden_prefixes.iter() {
            if line.starts_with(prefix) {
                offenders.push(line.clone());
                break;
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "A22: Phase 14-β is `--quick` lane; zero `.rs` changes allowed in \
         sim-core/sim-systems/sim-engine/sim-bridge/sim-data (tracked OR untracked), \
         but found: {offenders:?}"
    );
    println!(
        "[P14-β A22] zero Rust crate `.rs` modifications in --quick scope \
         (tracked + untracked checked) ✓"
    );
}
