//! V7 H Phase A — Trackpad zoom + pause + camera tracking + palette color fix.
//!
//! feature: h-phase-a-input-camera-color
//! seed: 42
//! agent_count: 20
//! lane: --quick
//!
//! Pure static file-inspection harness (G Phase A / Phase 14-ε/ζ precedent).
//! No simulation is run (`ticks: 0`), no ECS component is queried — the four
//! defects (trackpad zoom, pause, camera tracking, palette-swap color) are
//! GDScript-input / Camera2D / canvas-shader effects the headless harness
//! cannot exercise (no input device, the camera cannot be driven, and the
//! per-sprite color delta is sub-resolution for the VLM — see CLAUDE.md "VLM
//! Visual Verification — Known Limitation"). Every assertion reads source text
//! via the established helpers. The authoritative behavioural gate is the
//! windowed-Godot review (Section 7); the color fix is runtime-verified this
//! session via a standalone render-debug (vertex-tint → brown).
//!
//! Assertions (16):
//!   a1  — camera_controller handles InputEventPanGesture          (Type A)
//!   a2  — camera_controller handles InputEventMagnifyGesture      (Type A)
//!   a3  — mouse-wheel UP+DOWN path preserved                      (Type D)
//!   a4  — _unhandled_input binds KEY_P (pause)                    (Type A)
//!   a5  — pause toggles process_mode DISABLED/INHERIT             (Type A)
//!   a6  — _process tracks centroid (get_agent_snapshot + lerp)    (Type A)
//!   a7  — resolves /root/Main/WorldSim                            (Type A)
//!   a8  — tracking coordinate consts (TILE_SIZE 16, ORIGIN 448)   (Type A)
//!   a9  — ZOOM_MIN/MAX/DEFAULT invariants unchanged               (Type D)
//!   a10 — KEY_SPACE NOT repurposed in camera_controller           (Type D)
//!   a11 — shader captures v_tint = COLOR in vertex()              (Type A)
//!   a12 — shader multiplies v_tint.rgb; old modulate bug removed  (Type D)
//!   a13 — shader row-select/palette_uv/palette_lut preserved      (Type D)
//!   a14 — agent_renderer SPRITE_SCALE/STATE_TINTS/SHEET_COLS kept (Type D)
//!   a15 — world_renderer KEY_SPACE overlay intact                 (Type D)
//!   a16 — zoom_lod_controller LOD thresholds intact               (Type D)
//!
//! Run:
//!   cargo test -p sim-test --test harness_h_phase_a_input_camera_color -- --nocapture

use std::fs;
use std::path::PathBuf;

// ── helpers (G Phase A precedent) ────────────────────────────────────────────

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

fn read_palette_shader_src() -> String {
    read_file(&["shaders", "palette_swap.gdshader"])
}

fn read_agent_renderer_src() -> String {
    read_file(&["scripts", "ui", "agent_renderer.gd"])
}

fn read_world_renderer_src() -> String {
    read_file(&["scripts", "ui", "world_renderer.gd"])
}

fn read_zoom_lod_src() -> String {
    read_file(&["scripts", "ui", "zoom_lod_controller.gd"])
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

/// Strip GLSL/GDShader `// …` line comments.
fn strip_glsl_line_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        if let Some(idx) = line.find("//") {
            out.push_str(&line[..idx]);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn no_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Collapse runs of whitespace to a single space and trim, so multi-token
/// substring checks tolerate formatting differences while preserving token
/// boundaries (unlike `no_ws`).
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
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

/// Locate the byte-range of a top-level `func NAME(` body over stripped source.
fn find_func_body(stripped: &str, fname: &str) -> Option<(usize, usize)> {
    let needle = format!("func {fname}(");
    let start = stripped.find(&needle)?;
    let nl = stripped[start..].find('\n')?;
    let body_start = start + nl + 1;
    let next_func = stripped[body_start..].find("\nfunc ");
    let body_end = match next_func {
        Some(off) => body_start + off + 1,
        None => stripped.len(),
    };
    Some((body_start, body_end))
}

/// Extract a GLSL/GDShader `void NAME() { … }` function body from stripped
/// source. Returns the slice from the `void NAME(` header to the next
/// top-level `void ` declaration (or EOF).
fn shader_func_slice<'a>(stripped: &'a str, fname: &str) -> &'a str {
    let needle = format!("void {fname}(");
    let start = stripped
        .find(&needle)
        .unwrap_or_else(|| panic!("shader must define `void {fname}()`"));
    let after = &stripped[start..];
    let body_end = after[1..]
        .find("\nvoid ")
        .map(|i| i + 1)
        .unwrap_or(after.len());
    &after[..body_end]
}

// ════════════════════════════════════════════════════════════════════════════
// Group 1 — scripts/ui/camera_controller.gd  (Defects 1–3: trackpad/pause/track)
// ════════════════════════════════════════════════════════════════════════════

// ─── a1: trackpad pan gesture handled ─────────────────────────────────────────
#[test]
fn harness_h_a1_trackpad_pan_gesture_handled() {
    // Type A — physical invariant. Two-finger trackpad scroll arrives ONLY as
    // InputEventPanGesture; the trackpad-zoom requirement is unsatisfiable
    // without a branch referencing this class.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    assert!(
        stripped.contains("InputEventPanGesture"),
        "a1: camera_controller.gd must reference InputEventPanGesture \
         (trackpad two-finger scroll → zoom)"
    );
    println!("[H a1] InputEventPanGesture handled ✓");
}

// ─── a2: trackpad magnify gesture handled ─────────────────────────────────────
#[test]
fn harness_h_a2_trackpad_magnify_gesture_handled() {
    // Type A — physical invariant. Trackpad pinch arrives ONLY as
    // InputEventMagnifyGesture; pinch-zoom cannot work without it.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    assert!(
        stripped.contains("InputEventMagnifyGesture"),
        "a2: camera_controller.gd must reference InputEventMagnifyGesture \
         (trackpad pinch → zoom)"
    );
    println!("[H a2] InputEventMagnifyGesture handled ✓");
}

// ─── a3: mouse-wheel path preserved ───────────────────────────────────────────
#[test]
fn harness_h_a3_mouse_wheel_path_preserved() {
    // Type D — regression guard. The gesture branches must be ADDED, not
    // replace the existing wheel path (Section 2 forbids changing it).
    let stripped = strip_gd_comments(&read_camera_controller_src());
    assert!(
        stripped.contains("MOUSE_BUTTON_WHEEL_UP"),
        "a3.1: camera_controller.gd must keep MOUSE_BUTTON_WHEEL_UP (wheel zoom-in)"
    );
    assert!(
        stripped.contains("MOUSE_BUTTON_WHEEL_DOWN"),
        "a3.2: camera_controller.gd must keep MOUSE_BUTTON_WHEEL_DOWN (wheel zoom-out)"
    );
    println!("[H a3] mouse-wheel UP+DOWN path preserved ✓");
}

// ─── a4: pause bound to KEY_P inside _unhandled_input ──────────────────────────
#[test]
fn harness_h_a4_pause_key_p_in_input() {
    // Type A — physical invariant. Pause is bound to KEY_P inside
    // _unhandled_input; scoping to the body prevents a stray comment/const
    // elsewhere from satisfying it.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let (s, e) =
        find_func_body(&stripped, "_unhandled_input").expect("a4.1: _unhandled_input must exist");
    let body = &stripped[s..e];
    assert!(
        body.contains("KEY_P"),
        "a4.2: KEY_P must be referenced inside the _unhandled_input body \
         (pause trigger). Body:\n{body}"
    );
    println!("[H a4] KEY_P bound inside _unhandled_input ✓");
}

// ─── a5: pause toggles process_mode DISABLED/INHERIT ──────────────────────────
#[test]
fn harness_h_a5_pause_toggles_process_mode() {
    // Type A — physical invariant. The pause mechanism freezes the Rust tick
    // by gating WorldSimNode.process() via process_mode = DISABLED and resumes
    // via INHERIT. All three tokens are necessary for a working freeze/resume.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    for needle in ["process_mode", "PROCESS_MODE_DISABLED", "PROCESS_MODE_INHERIT"] {
        assert!(
            stripped.contains(needle),
            "a5: camera_controller.gd must reference `{needle}` \
             (pause freeze/resume mechanism)"
        );
    }
    println!("[H a5] pause toggles process_mode DISABLED/INHERIT ✓");
}

// ─── a6: _process tracks the swarm centroid ───────────────────────────────────
#[test]
fn harness_h_a6_camera_track_in_process() {
    // Type A — physical invariant. Centroid tracking requires reading the
    // agent snapshot every frame and easing position toward the mean via lerp.
    // Both tokens are scoped to the _process body so an unrelated snapshot read
    // cannot satisfy a missing tracking implementation.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let (s, e) = find_func_body(&stripped, "_process").expect("a6.1: _process must exist");
    let body = &stripped[s..e];
    assert!(
        body.contains("get_agent_snapshot"),
        "a6.2: _process body must read get_agent_snapshot (centroid source). Body:\n{body}"
    );
    assert!(
        body.contains("lerp"),
        "a6.3: _process body must ease position via lerp (gentle tracking). Body:\n{body}"
    );
    println!("[H a6] _process tracks centroid (get_agent_snapshot + lerp) ✓");
}

// ─── a7: resolves the WorldSim node ───────────────────────────────────────────
#[test]
fn harness_h_a7_resolves_world_sim_node() {
    // Type A — physical invariant. Both pause (a5) and tracking (a6) require a
    // reference to the WorldSim node; without resolving this path _world_sim
    // stays null and both features silently no-op.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    assert!(
        stripped.contains("/root/Main/WorldSim"),
        "a7: camera_controller.gd must resolve the WorldSim node path \
         `/root/Main/WorldSim`"
    );
    println!("[H a7] resolves /root/Main/WorldSim ✓");
}

// ─── a8: tracking coordinate constants ────────────────────────────────────────
#[test]
fn harness_h_a8_tracking_coordinate_constants() {
    // Type A — physical invariant. The tracked centroid must be converted to
    // world-pixels on the SAME SPRITE_ORIGIN + tile*TILE_SIZE basis the
    // renderers use, or the camera centres on the wrong point. Exact-match is
    // required (not >0) because any other value breaks alignment.
    let stripped = strip_gd_comments(&read_camera_controller_src());

    let track_rhs = unique_decl_rhs(&stripped, "CAMERA_TRACK_SPEED", "a8.1");
    let track = parse_float_rhs(&track_rhs).unwrap_or_else(|| {
        panic!("a8.2: CAMERA_TRACK_SPEED RHS must parse as float; got `{track_rhs}`")
    });
    assert!(
        track > 0.0,
        "a8.3: CAMERA_TRACK_SPEED must be a positive lerp rate; got {track}"
    );

    let tile_rhs = unique_decl_rhs(&stripped, "TILE_SIZE", "a8.4");
    let tile = parse_int_rhs(&tile_rhs)
        .unwrap_or_else(|| panic!("a8.5: TILE_SIZE RHS must parse as int; got `{tile_rhs}`"));
    assert_eq!(
        tile, 16,
        "a8.6: TILE_SIZE must equal 16 (renderer grid); got {tile}"
    );

    let ox_rhs = unique_decl_rhs(&stripped, "SPRITE_ORIGIN_X", "a8.7");
    let ox = parse_int_rhs(&ox_rhs)
        .unwrap_or_else(|| panic!("a8.8: SPRITE_ORIGIN_X RHS must parse as int; got `{ox_rhs}`"));
    assert_eq!(
        ox, 448,
        "a8.9: SPRITE_ORIGIN_X must equal 448 (renderer origin); got {ox}"
    );

    println!("[H a8] CAMERA_TRACK_SPEED float, TILE_SIZE=16, SPRITE_ORIGIN_X=448 ✓");
}

// ─── a9: zoom invariants unchanged ────────────────────────────────────────────
#[test]
fn harness_h_a9_zoom_invariants_unchanged() {
    // Type D — regression guard. Section 2 forbids changing ZOOM_MIN/MAX here.
    // These are the Phase 12-α (0.5) / G Phase A (8.0) locked values; ZOOM_DEFAULT
    // was raised 3.0 → 5.0 in B-1. Exact match catches any drift.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    let min_rhs = unique_decl_rhs(&stripped, "ZOOM_MIN", "a9.1");
    let max_rhs = unique_decl_rhs(&stripped, "ZOOM_MAX", "a9.2");
    let def_rhs = unique_decl_rhs(&stripped, "ZOOM_DEFAULT", "a9.3");
    assert_eq!(
        no_ws(&min_rhs),
        "Vector2(0.5,0.5)",
        "a9.4: ZOOM_MIN must equal Vector2(0.5,0.5); got `{min_rhs}`"
    );
    assert_eq!(
        no_ws(&max_rhs),
        "Vector2(8.0,8.0)",
        "a9.5: ZOOM_MAX must equal Vector2(8.0,8.0); got `{max_rhs}`"
    );
    assert_eq!(
        no_ws(&def_rhs),
        "Vector2(5.0,5.0)",
        "a9.6: ZOOM_DEFAULT must equal Vector2(5.0,5.0) (raised from 3.0 in B-1); got `{def_rhs}`"
    );
    println!("[H a9] ZOOM_MIN(0.5)/MAX(8.0)/DEFAULT(5.0) unchanged ✓");
}

// ─── a10: KEY_SPACE not repurposed in camera_controller ───────────────────────
#[test]
fn harness_h_a10_space_not_repurposed_in_camera() {
    // Type D — negative regression guard. KEY_SPACE is owned by
    // world_renderer.gd (overlay-channel cycle); pause must use KEY_P (a4),
    // not steal SPACE.
    let stripped = strip_gd_comments(&read_camera_controller_src());
    assert!(
        !stripped.contains("KEY_SPACE"),
        "a10: camera_controller.gd must NOT reference KEY_SPACE (pause is KEY_P; \
         SPACE belongs to world_renderer overlay cycle)"
    );
    println!("[H a10] KEY_SPACE not repurposed in camera_controller ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group 2 — shaders/palette_swap.gdshader  (Defect 4: green-bug color fix)
// ════════════════════════════════════════════════════════════════════════════

// ─── a11: vertex-stage tint capture ───────────────────────────────────────────
#[test]
fn harness_h_a11_shader_vertex_tint_capture() {
    // Type A — physical invariant / the fix. COLOR must be captured at the
    // VERTEX stage (pre-texture-multiply) to recover the true per-instance
    // tint. Scoping `v_tint = COLOR` to the vertex() body ensures the capture
    // is at the correct stage, not fragment.
    let stripped = strip_glsl_line_comments(&read_palette_shader_src());
    assert!(
        stripped.contains("varying vec4 v_tint"),
        "a11.1: palette_swap.gdshader must declare `varying vec4 v_tint`"
    );
    let vertex = collapse_ws(shader_func_slice(&stripped, "vertex"));
    assert!(
        vertex.contains("v_tint = COLOR"),
        "a11.2: vertex() body must capture `v_tint = COLOR` (pre-texture tint). \
         vertex():\n{vertex}"
    );
    println!("[H a11] shader captures v_tint = COLOR in vertex() ✓");
}

// ─── a12: multiply v_tint, old fragment-modulate bug removed ──────────────────
#[test]
fn harness_h_a12_shader_multiplies_v_tint_and_bug_removed() {
    // Type D — regression/bug-removal guard for the confirmed green-bug. The
    // positive `* v_tint.rgb` proves the fix; the two negative guards prove the
    // fragment-stage `modulate` capture+multiply (the bug) was REMOVED, not
    // left as dead code. MUST run on comment-stripped source — the shader
    // header comment legitimately mentions the old `vec4 modulate = COLOR`
    // phrasing while explaining the fix.
    let stripped = strip_glsl_line_comments(&read_palette_shader_src());
    let compact = collapse_ws(&stripped);

    assert!(
        compact.contains("palette_color.rgb * v_tint.rgb"),
        "a12.1: shader must multiply `palette_color.rgb * v_tint.rgb` (the fix)"
    );
    assert!(
        !stripped.contains("vec4 modulate = COLOR"),
        "a12.2: shader must NOT keep the fragment-stage `vec4 modulate = COLOR` \
         capture (the green-bug)"
    );
    assert!(
        !compact.contains("palette_color.rgb * modulate.rgb"),
        "a12.3: shader must NOT keep the `palette_color.rgb * modulate.rgb` \
         multiply (the green-bug)"
    );
    println!("[H a12] shader multiplies v_tint.rgb; fragment modulate bug removed ✓");
}

// ─── a13: row-selection / palette logic preserved ─────────────────────────────
#[test]
fn harness_h_a13_shader_rowselect_logic_preserved() {
    // Type D — regression guard. Section 2 requires the row-selection +
    // palette_uv + LUT-sample logic kept "byte-for-byte otherwise". The
    // tint-capture move must not disturb the Phase 11-α row-select path.
    let stripped = strip_glsl_line_comments(&read_palette_shader_src());
    for needle in ["row_selector = tex.g", "palette_uv", "palette_lut"] {
        assert!(
            stripped.contains(needle),
            "a13: palette_swap.gdshader must preserve `{needle}` (row-select path)"
        );
    }
    println!("[H a13] shader row-select / palette_uv / palette_lut preserved ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Group 3 — Cross-phase regression-guard reference files (READ-ONLY, "Not changed")
// ════════════════════════════════════════════════════════════════════════════

// ─── a14: agent_renderer invariants intact ────────────────────────────────────
#[test]
fn harness_h_a14_agent_renderer_untouched_invariants() {
    // Type D — regression guard. Section 2 lists agent_renderer.gd as "Not
    // changed". Phase 4-γ SPRITE_SCALE=0.25, Phase 11-α STATE_TINTS, and G
    // Phase A SHEET_COLS must all survive untouched.
    let stripped = strip_gd_comments(&read_agent_renderer_src());
    let rhs = unique_decl_rhs(&stripped, "SPRITE_SCALE", "a14.1");
    let f = parse_float_rhs(&rhs)
        .unwrap_or_else(|| panic!("a14.2: SPRITE_SCALE RHS must parse as float; got `{rhs}`"));
    assert!(
        (f - 0.25).abs() < 1e-9,
        "a14.3: SPRITE_SCALE must equal 0.25 (Phase 4-γ invariant); got {f}"
    );
    assert!(
        stripped.contains("STATE_TINTS"),
        "a14.4: agent_renderer.gd must keep STATE_TINTS (Phase 11-α)"
    );
    assert!(
        stripped.contains("SHEET_COLS"),
        "a14.5: agent_renderer.gd must keep SHEET_COLS (G Phase A)"
    );
    println!("[H a14] agent_renderer SPRITE_SCALE=0.25 + STATE_TINTS + SHEET_COLS intact ✓");
}

// ─── a15: world_renderer KEY_SPACE overlay intact ─────────────────────────────
#[test]
fn harness_h_a15_world_renderer_space_overlay_intact() {
    // Type D — regression guard. The KEY_SPACE overlay-channel cycle lives in
    // world_renderer.gd ("Not changed"). Paired with a10, this proves SPACE
    // ownership stayed in world_renderer while pause went to KEY_P.
    let stripped = strip_gd_comments(&read_world_renderer_src());
    assert!(
        stripped.contains("KEY_SPACE"),
        "a15: world_renderer.gd must keep KEY_SPACE (overlay-channel cycle)"
    );
    println!("[H a15] world_renderer KEY_SPACE overlay intact ✓");
}

// ─── a16: zoom_lod_controller thresholds intact ───────────────────────────────
#[test]
fn harness_h_a16_zoom_lod_thresholds_intact() {
    // Type D — regression guard. Section 2 lists zoom_lod_controller.gd as
    // "Not changed"; Phase 14-ζ LOD thresholds anchor the tier classification,
    // independent of the camera changes.
    let stripped = strip_gd_comments(&read_zoom_lod_src());
    for needle in ["ZOOM_FAR_MAX", "ZOOM_CLOSE_MIN"] {
        assert!(
            stripped.contains(needle),
            "a16: zoom_lod_controller.gd must keep `{needle}` (Phase 14-ζ LOD)"
        );
    }
    println!("[H a16] zoom_lod_controller LOD thresholds intact ✓");
}
