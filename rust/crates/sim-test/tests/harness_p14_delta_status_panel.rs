//! V7 Phase 14-δ — HUD Status Panel harness.
//!
//! feature: p14-delta-status-panel
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Static file-inspection harness verifying the Phase 14-δ implementation:
//!   - `scripts/ui/panels/hud_status_panel.gd` — new Control panel, top-right
//!     anchored, polls existing 3 SimBridge snapshots (no new FFI), surfaces
//!     Day/Year header, 5 resource cells, notification feed.
//!   - `scenes/main.tscn` — registers HudStatusPanel under UI CanvasLayer.
//!   - Regression guards: hud_topbar.gd UNCHANGED (Phase 13-δ A4/A5/A6),
//!     agent_renderer.gd ROLE_BUCKET_COUNT/ICON_OFFSET_PX preserved,
//!     world_renderer.gd RESOURCE_TYPE_PATHS/RESOURCE_COUNT/RESOURCE_SEED/
//!     VILLAGE_FIXTURE_* preserved, agent_inspector_panel.gd present.
//!   - Strict full-file hash equality on hud_topbar.gd (Assertion 17).
//!
//! Run:
//!   cargo test -p sim-test --test harness_p14_delta_status_panel -- --nocapture

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

fn read_hud_status_panel_src() -> String {
    read_file(&["scripts", "ui", "panels", "hud_status_panel.gd"])
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

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
}

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

/// Parse the integer literal of a `const NAME: int = N` (or `:= N`) RHS,
/// tolerating trailing comments (already stripped) and whitespace.
fn parse_int_rhs(rhs: &str) -> Option<i64> {
    let trimmed = rhs.trim();
    let mut end = 0usize;
    let mut saw_digit = false;
    for (i, c) in trimmed.char_indices() {
        if i == 0 && (c == '-' || c == '+') {
            end = i + c.len_utf8();
            continue;
        }
        if c.is_ascii_digit() {
            end = i + c.len_utf8();
            saw_digit = true;
        } else {
            break;
        }
    }
    if !saw_digit {
        return None;
    }
    trimmed[..end].parse::<i64>().ok()
}

fn sha256_hex(bytes: &[u8]) -> String {
    // Minimal-dependency SHA-256 implementation (RFC 6234). Sim-test cannot
    // pull in new crates without rewriting the workspace; this routine is
    // verified against the openssl reference hash in CI.
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut msg: Vec<u8> = bytes.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 =
                w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 =
                w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut s = String::with_capacity(64);
    for v in h.iter() {
        s.push_str(&format!("{v:08x}"));
    }
    s
}

// ─── A1: file exists + extends Control ────────────────────────────────────
#[test]
fn harness_p14_delta_a1_hud_status_panel_file_exists_and_extends_control() {
    // Type A — file present at exact path; first non-comment, non-blank
    // line is literally `extends Control`.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("hud_status_panel.gd");
    assert!(
        path.is_file(),
        "A1.1: hud_status_panel.gd must exist at {path:?}"
    );
    let src = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("A1.2: cannot read {path:?}: {e}"));
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
    let head = first.expect("A1.3: hud_status_panel.gd has no non-blank, non-comment content");
    assert_eq!(
        head, "extends Control",
        "A1.4: first non-comment, non-blank line must be `extends Control`; got `{head}`"
    );
    println!("[P14-δ A1] hud_status_panel.gd exists + `extends Control` ✓");
}

// ─── A2: TICKS_PER_DAY == 100 ─────────────────────────────────────────────
#[test]
fn harness_p14_delta_a2_ticks_per_day_constant_present() {
    // Type A — `const TICKS_PER_DAY ... = 100`.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let rhs = unique_decl_rhs(&stripped, "TICKS_PER_DAY", "A2");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("A2: TICKS_PER_DAY RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 100, "A2: TICKS_PER_DAY must equal 100; got {n}");
    println!("[P14-δ A2] TICKS_PER_DAY = 100 ✓");
}

// ─── A3: DAYS_PER_YEAR == 30 ──────────────────────────────────────────────
#[test]
fn harness_p14_delta_a3_days_per_year_constant_present() {
    // Type A — `const DAYS_PER_YEAR ... = 30`.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let rhs = unique_decl_rhs(&stripped, "DAYS_PER_YEAR", "A3");
    let n = parse_int_rhs(&rhs)
        .unwrap_or_else(|| panic!("A3: DAYS_PER_YEAR RHS must parse as int; got `{rhs}`"));
    assert_eq!(n, 30, "A3: DAYS_PER_YEAR must equal 30; got {n}");
    println!("[P14-δ A3] DAYS_PER_YEAR = 30 ✓");
}

// ─── A4: RESOURCE_TYPES_COUNT * RESOURCE_PER_TYPE == 20 ───────────────────
#[test]
fn harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta() {
    // Type D — guards Phase 13-β RESOURCE_COUNT = 20 alignment.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let types_rhs = unique_decl_rhs(&stripped, "RESOURCE_TYPES_COUNT", "A4.types");
    let per_rhs = unique_decl_rhs(&stripped, "RESOURCE_PER_TYPE", "A4.per");
    let types = parse_int_rhs(&types_rhs).unwrap_or_else(|| {
        panic!("A4: RESOURCE_TYPES_COUNT RHS must parse as int; got `{types_rhs}`")
    });
    let per = parse_int_rhs(&per_rhs).unwrap_or_else(|| {
        panic!("A4: RESOURCE_PER_TYPE RHS must parse as int; got `{per_rhs}`")
    });
    assert_eq!(types, 5, "A4: RESOURCE_TYPES_COUNT must equal 5; got {types}");
    assert_eq!(per, 4, "A4: RESOURCE_PER_TYPE must equal 4; got {per}");
    assert_eq!(
        types * per,
        20,
        "A4: RESOURCE_TYPES_COUNT * RESOURCE_PER_TYPE must equal 20; got {}",
        types * per
    );
    println!("[P14-δ A4] RESOURCE_TYPES_COUNT(5) * RESOURCE_PER_TYPE(4) = 20 ✓");
}

// ─── A5: RESOURCE_LABELS exact order ──────────────────────────────────────
#[test]
fn harness_p14_delta_a5_resource_labels_array_exact_order() {
    // Type A — exactly ["Berry", "Wood", "Stone", "Water", "Food"].
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let rhs = unique_decl_rhs(&stripped, "RESOURCE_LABELS", "A5");
    let compact = no_ws(&rhs);
    let want = "[\"Berry\",\"Wood\",\"Stone\",\"Water\",\"Food\"]";
    assert_eq!(
        compact, want,
        "A5: RESOURCE_LABELS must equal {want} (whitespace-collapsed); got `{compact}`"
    );
    println!("[P14-δ A5] RESOURCE_LABELS = [Berry, Wood, Stone, Water, Food] ✓");
}

// ─── A6: top-right anchor + MOUSE_FILTER_IGNORE ───────────────────────────
#[test]
fn harness_p14_delta_a6_anchored_top_right_with_mouse_filter_ignore() {
    // Type A — `anchor_left = 1.0` + `anchor_right = 1.0` +
    // `mouse_filter = Control.MOUSE_FILTER_IGNORE`.
    let src = read_hud_status_panel_src();
    let needles = [
        "anchor_left = 1.0",
        "anchor_right = 1.0",
        "mouse_filter = Control.MOUSE_FILTER_IGNORE",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !src.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A6: hud_status_panel.gd must contain the literal triple {needles:?}; \
         missing={missing:?}"
    );
    println!("[P14-δ A6] top-right anchors + MOUSE_FILTER_IGNORE present ✓");
}

// ─── A7: polls 3 existing snapshots ───────────────────────────────────────
#[test]
fn harness_p14_delta_a7_polls_all_three_existing_snapshots() {
    // Type A — `get_agent_snapshot`, `get_settlement_snapshot`,
    // `get_construction_snapshot` literals present.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
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
        "A7: hud_status_panel.gd must reference all 3 snapshot FFI names; missing={missing:?}"
    );
    println!("[P14-δ A7] 3 snapshot FFI literals present ✓");
}

// ─── A8: Variant-safe pattern matches hud_topbar idiom ────────────────────
#[test]
fn harness_p14_delta_a8_variant_safe_pattern_matches_hud_topbar_idiom() {
    // Type D — `is Dictionary` + `is PackedInt64Array` regression guard.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    assert!(
        stripped.contains("is Dictionary"),
        "A8.1: hud_status_panel.gd must contain `is Dictionary` type guard"
    );
    assert!(
        stripped.contains("is PackedInt64Array"),
        "A8.2: hud_status_panel.gd must contain `is PackedInt64Array` type guard"
    );
    println!("[P14-δ A8] Variant-safe pattern present ✓");
}

// ─── A9: notification feed constants ──────────────────────────────────────
#[test]
fn harness_p14_delta_a9_notification_feed_constants_present() {
    // Type A — MAX_VISIBLE_NOTIFICATIONS == 5 AND NOTIF_FADE_FRAMES == 480.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let max_rhs = unique_decl_rhs(&stripped, "MAX_VISIBLE_NOTIFICATIONS", "A9.max");
    let fade_rhs = unique_decl_rhs(&stripped, "NOTIF_FADE_FRAMES", "A9.fade");
    let max_n = parse_int_rhs(&max_rhs).unwrap_or_else(|| {
        panic!("A9: MAX_VISIBLE_NOTIFICATIONS RHS must parse as int; got `{max_rhs}`")
    });
    let fade_n = parse_int_rhs(&fade_rhs).unwrap_or_else(|| {
        panic!("A9: NOTIF_FADE_FRAMES RHS must parse as int; got `{fade_rhs}`")
    });
    assert_eq!(
        max_n, 5,
        "A9: MAX_VISIBLE_NOTIFICATIONS must equal 5; got {max_n}"
    );
    assert_eq!(
        fade_n, 480,
        "A9: NOTIF_FADE_FRAMES must equal 480; got {fade_n}"
    );
    println!("[P14-δ A9] MAX_VISIBLE_NOTIFICATIONS=5 + NOTIF_FADE_FRAMES=480 ✓");
}

// ─── A10: main.tscn registers HudStatusPanel under UI ─────────────────────
#[test]
fn harness_p14_delta_a10_main_tscn_registers_hud_status_panel() {
    // Type A — main.tscn references hud_status_panel.gd AND has a
    // [node name="HudStatusPanel" type="Control" parent="UI"] header line.
    let src = read_main_tscn_src();
    assert!(
        src.contains("hud_status_panel.gd"),
        "A10.1: main.tscn must reference `hud_status_panel.gd`"
    );
    let mut found = false;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[node ") {
            continue;
        }
        if t.contains("name=\"HudStatusPanel\"")
            && t.contains("type=\"Control\"")
            && t.contains("parent=\"UI\"")
        {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "A10.2: main.tscn must contain `[node name=\"HudStatusPanel\" type=\"Control\" ... parent=\"UI\" ...]` \
         line; source:\n{src}"
    );
    println!("[P14-δ A10] HudStatusPanel registered under UI ✓");
}

// ─── A11: load_steps = 8 ──────────────────────────────────────────────────
#[test]
fn harness_p14_delta_a11_main_tscn_load_steps_updated_from_7_to_8() {
    // Type A — `load_steps=8` in the [gd_scene ...] header, tolerating
    // optional whitespace around `=`.
    let src = read_main_tscn_src();
    let mut found_n: Option<i64> = None;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[gd_scene") {
            continue;
        }
        // Find `load_steps` then skip whitespace, optional `=`, whitespace,
        // then collect digits.
        if let Some(pos) = t.find("load_steps") {
            let after = &t[pos + "load_steps".len()..];
            let after = after.trim_start();
            let after = after.strip_prefix('=').unwrap_or(after);
            let after = after.trim_start();
            let mut digits = String::new();
            for c in after.chars() {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                found_n = Some(digits.parse::<i64>().expect("digit parse"));
            }
        }
        break;
    }
    let n = found_n.expect("A11.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 8, "A11.2: load_steps must equal 8; got {n}");
    println!("[P14-δ A11] load_steps = 8 ✓");
}

// ─── A12: hud_topbar.gd A4/A5/A6 invariants preserved ────────────────────
#[test]
fn harness_p14_delta_a12_phase_13_delta_a4_a5_a6_invariants_preserved_in_hud_topbar() {
    // Type D — substring presence check on the 6 canonical literals.
    let stripped = strip_gd_comments(&read_hud_topbar_src());
    let needles = [
        "get_agent_snapshot",
        "get_settlement_snapshot",
        "get_construction_snapshot",
        "is Dictionary",
        "is PackedInt64Array",
        "MOUSE_FILTER_IGNORE",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A12: hud_topbar.gd must contain all 6 invariant literals (Phase 13-δ A4/A5/A6); \
         missing={missing:?}"
    );
    println!("[P14-δ A12] hud_topbar.gd A4/A5/A6 literals all present ✓");
}

// ─── A13: agent_renderer.gd ROLE_BUCKET_COUNT + ICON_OFFSET_PX preserved ─
#[test]
fn harness_p14_delta_a13_phase_14_alpha_role_bucket_invariants_preserved() {
    // Type D — ROLE_BUCKET_COUNT == 4 AND ICON_OFFSET_PX present.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let bucket = unique_decl_rhs(&stripped, "ROLE_BUCKET_COUNT", "A13.bucket");
    assert_eq!(
        bucket.trim(),
        "4",
        "A13.1: ROLE_BUCKET_COUNT must equal 4; got `{bucket}`"
    );
    // ICON_OFFSET_PX declaration must exist (RHS form is not constrained
    // beyond its prior Phase 14-α/γ value, which other harnesses lock).
    let icon_rhss = find_decl_rhss(&stripped, "ICON_OFFSET_PX");
    assert!(
        !icon_rhss.is_empty(),
        "A13.2: agent_renderer.gd must declare `ICON_OFFSET_PX`; got 0 declarations"
    );
    println!("[P14-δ A13] ROLE_BUCKET_COUNT=4 + ICON_OFFSET_PX present ✓");
}

// ─── A14: world_renderer.gd resource registry preserved ──────────────────
#[test]
fn harness_p14_delta_a14_phase_14_beta_resource_registry_preserved() {
    // Type D — RESOURCE_TYPE_PATHS 5-entry, RESOURCE_COUNT == 20,
    // RESOURCE_SEED == 88675123.
    let stripped = strip_gd_comments(&read_world_renderer_src());

    let count = unique_decl_rhs(&stripped, "RESOURCE_COUNT", "A14.count");
    assert_eq!(
        count.trim(),
        "20",
        "A14.1: RESOURCE_COUNT must equal 20; got `{count}`"
    );

    let seed = unique_decl_rhs(&stripped, "RESOURCE_SEED", "A14.seed");
    assert_eq!(
        seed.trim(),
        "88675123",
        "A14.2: RESOURCE_SEED must equal 88675123; got `{seed}`"
    );

    // Count entries of RESOURCE_TYPE_PATHS array (5 res:// paths).
    let decl_pos = stripped
        .find("RESOURCE_TYPE_PATHS")
        .expect("A14.3: RESOURCE_TYPE_PATHS symbol must exist");
    let open_rel = stripped[decl_pos..]
        .find('[')
        .expect("A14.4: RESOURCE_TYPE_PATHS array must open with `[`");
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
        "A14.5: RESOURCE_TYPE_PATHS array must terminate with `]`"
    );
    let body = &stripped[open_abs + 1..k - 1];
    // Count comma-separated entries (tolerates trailing comma + identifier
    // aliases like RESOURCE_SPRITE_PATH; not just `res://` literals).
    let entries: Vec<&str> = body
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(
        entries.len(),
        5,
        "A14.6: RESOURCE_TYPE_PATHS must contain exactly 5 entries; got {} ({entries:?})",
        entries.len()
    );
    println!(
        "[P14-δ A14] RESOURCE_TYPE_PATHS(5 entries) + RESOURCE_COUNT=20 + RESOURCE_SEED=88675123 ✓"
    );
}

// ─── A15: agent_inspector_panel.gd present (Phase 14-γ) ──────────────────
#[test]
fn harness_p14_delta_a15_phase_14_gamma_agent_inspector_panel_present() {
    // Type D — file exists at exact path.
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("agent_inspector_panel.gd");
    assert!(
        path.is_file(),
        "A15: agent_inspector_panel.gd must exist at {path:?} (Phase 14-γ regression guard)"
    );
    println!("[P14-δ A15] agent_inspector_panel.gd present ✓");
}

// ─── A16: VILLAGE_FIXTURE_* prefix preserved in world_renderer.gd ────────
#[test]
fn harness_p14_delta_a16_village_fixture_constants_preserved_in_world_renderer() {
    // Type D — at least one `VILLAGE_FIXTURE_` prefixed identifier.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    let n = stripped.matches("VILLAGE_FIXTURE_").count();
    assert!(
        n >= 1,
        "A16: world_renderer.gd must keep at least one `VILLAGE_FIXTURE_` identifier; got {n}"
    );
    println!("[P14-δ A16] VILLAGE_FIXTURE_ prefix preserved ({n} occurrences) ✓");
}

// ─── A17: hud_topbar.gd byte hash equals the pre-δ HEAD snapshot ─────────
#[test]
fn harness_p14_delta_a17_hud_topbar_file_byte_or_line_count_unchanged_against_pre_phase_14_delta_head(
) {
    // Type D (strict) — full-file SHA-256 must match the recorded
    // pre-implementation hash. Captured from HEAD at planning time
    // (2026-05-29) via:
    //   shasum -a 256 scripts/ui/panels/hud_topbar.gd
    //   wc -l scripts/ui/panels/hud_topbar.gd
    // Expected: 103 lines and the hash below.
    const EXPECTED_HASH: &str =
        "e52990e3b9cf8074a198af56153f5d6fe57e0bf8cd369c90d75ce0c2c459cca5";
    const EXPECTED_LINES: usize = 103;

    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("hud_topbar.gd");
    let bytes = fs::read(&path)
        .unwrap_or_else(|e| panic!("A17.1: cannot read {path:?}: {e}"));
    let got_hash = sha256_hex(&bytes);
    let got_lines = String::from_utf8_lossy(&bytes).lines().count();
    assert_eq!(
        got_lines, EXPECTED_LINES,
        "A17.2: hud_topbar.gd line count must equal pre-δ HEAD ({EXPECTED_LINES}); got {got_lines}"
    );
    assert_eq!(
        got_hash, EXPECTED_HASH,
        "A17.3: hud_topbar.gd SHA-256 must equal pre-δ HEAD `{EXPECTED_HASH}`; got `{got_hash}`. \
         Phase 13-δ requires hud_topbar.gd to be absolutely untouched."
    );
    println!(
        "[P14-δ A17] hud_topbar.gd hash + line count match pre-δ HEAD (sha256={got_hash}) ✓"
    );
}
