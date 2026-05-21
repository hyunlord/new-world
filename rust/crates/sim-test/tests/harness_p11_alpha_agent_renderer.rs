//! V7 Phase 11-α — Agent Renderer Rust substrate harness.
//!
//! Tests the `collect_agent_snapshot` FFI surface that feeds the GDScript
//! AgentRenderer's position interpolation and state_tag tint. All tests
//! operate on a bare `hecs::World` (via `SimEngine`) without running
//! simulation ticks — they validate the data contract, not system execution.
//!
//! Tag table (locked, §2-A-1 from `AgentSnapshotRow` type doc):
//!   - `0` = `AgentState::Idle` (or absent AgentState component)
//!   - `1` = `AgentState::Seeking { .. }` (any `TargetKind`)
//!   - `2` = `AgentState::Consuming { target: TargetKind::Agent(_) }`
//!   - `3` = `AgentState::Consuming { .. }` (any non-`Agent` `TargetKind`)
//!
//! Substrate assertions (A1–A11):
//!   A1  — Idle agent yields state_tag == 0
//!   A2  — Seeking agent yields state_tag == 1
//!   A3  — Consuming(Agent) agent yields state_tag == 2
//!   A4  — Consuming(Food) agent yields state_tag == 3
//!   A5  — Consuming(Sleep) agent yields state_tag == 3
//!   A6  — snapshot x matches ECS Position.x
//!   A7  — snapshot y matches ECS Position.y
//!   A8  — snapshot agent_id matches Agent.id component
//!   A9  — 5 agents → 5 snapshot rows
//!   A10 — all state_tag values in 0..=3
//!   A11 — entity without AgentState component defaults to tag 0
//!
//! Static file-content assertions (A12–A19) — code-attempt 2/3 additions.
//! Per the attempt-2 review, the substrate tests above only exercise
//! `collect_agent_snapshot`; the actual P11-α feature lives in GDScript +
//! shader code that those Rust tests cannot reach. The following tests
//! inspect the actual modified files to prove the new feature code
//! paths are present:
//!   A12 — agent_renderer.gd enables MultiMesh.use_colors = true
//!   A13 — agent_renderer.gd reads `states: PackedByteArray` from snapshot
//!   A14 — agent_renderer.gd builds _prev_positions / _curr_positions
//!         and interpolates with a lerp_t (Gaffer accumulator)
//!   A15 — agent_renderer.gd calls multi_mesh.set_instance_color(i,
//!         STATE_TINTS[...]) with a 4-entry STATE_TINTS palette
//!   A16 — palette_swap.gdshader captures `vec4 modulate = COLOR`
//!   A17 — palette_swap.gdshader multiplies palette_color.rgb * modulate.rgb
//!         in the visible-alpha branch (tex.a > 0.01)
//!   A18 — _snapshot_checksum_from MUST iterate every row (no `mini(n, 32)`
//!         truncation), so tick-boundary detection covers every agent
//!   A19 — _snapshot_checksum_from MUST be identity-aware: signature
//!         includes `agent_ids`, body mixes `agent_ids[i]` into the hash,
//!         and the call site forwards `agent_ids` (no order-insensitive
//!         XOR over positions alone)
//!
//! Run: `cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture`

use std::fs;
use std::path::PathBuf;

use sim_bridge::ffi::collect_agent_snapshot;
use sim_core::components::agent_state::TargetKind;
use sim_core::components::AgentState;
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;

// ─── static file-content helpers (A12–A17) ─────────────────────────────────

fn project_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root above sim-test crate")
}

fn read_agent_renderer_src() -> String {
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("agent_renderer.gd");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn read_palette_shader_src() -> String {
    let path = project_root()
        .join("shaders")
        .join("palette_swap.gdshader");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Strip GDScript line comments (`# …` to EOL) so a contract check against
/// the file body cannot be satisfied by commentary alone. Quoted `#` (e.g.
/// inside a Color literal) does not occur in this file, but we still keep
/// the logic conservative: only strip `#` when not inside a string literal.
fn strip_gd_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut in_str: Option<char> = None;
        let mut keep_end = line.len();
        let bytes = line.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let c = b as char;
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

/// Strip GLSL/GDShader line comments (`// …` to EOL).
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

const W: u32 = 64;
const H: u32 = 64;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

// ── A1 — Idle state_tag == 0 ───────────────────────────────────────────────
#[test]
fn harness_p11_alpha_idle_state_tag_is_zero() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(3, 5);
    // Explicitly insert Idle state (same as the default path).
    e.world
        .insert_one(entity, AgentState::Idle)
        .expect("entity just spawned");
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state_tag, 0,
        "A1: AgentState::Idle must yield state_tag == 0, got {}",
        rows[0].state_tag
    );
    println!("[P11-α A1] Idle → state_tag 0 ✓");
}

// ── A2 — Seeking state_tag == 1 ────────────────────────────────────────────
#[test]
fn harness_p11_alpha_seeking_state_tag_is_one() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(1, 2);
    e.world
        .insert_one(entity, AgentState::Seeking { target: TargetKind::Food })
        .expect("entity just spawned");
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state_tag, 1,
        "A2: AgentState::Seeking{{Food}} must yield state_tag == 1, got {}",
        rows[0].state_tag
    );
    println!("[P11-α A2] Seeking(Food) → state_tag 1 ✓");
}

// ── A3 — Consuming(Agent) state_tag == 2 ───────────────────────────────────
#[test]
fn harness_p11_alpha_consuming_agent_state_tag_is_two() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(4, 6);
    e.world
        .insert_one(entity, AgentState::Consuming { target: TargetKind::Agent(99) })
        .expect("entity just spawned");
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state_tag, 2,
        "A3: AgentState::Consuming{{Agent(_)}} must yield state_tag == 2, got {}",
        rows[0].state_tag
    );
    println!("[P11-α A3] Consuming(Agent(99)) → state_tag 2 ✓");
}

// ── A4 — Consuming(Food) state_tag == 3 ────────────────────────────────────
#[test]
fn harness_p11_alpha_consuming_food_state_tag_is_three() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(7, 8);
    e.world
        .insert_one(entity, AgentState::Consuming { target: TargetKind::Food })
        .expect("entity just spawned");
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state_tag, 3,
        "A4: AgentState::Consuming{{Food}} must yield state_tag == 3, got {}",
        rows[0].state_tag
    );
    println!("[P11-α A4] Consuming(Food) → state_tag 3 ✓");
}

// ── A5 — Consuming(Sleep) state_tag == 3 ───────────────────────────────────
#[test]
fn harness_p11_alpha_consuming_sleep_state_tag_is_three() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(2, 9);
    e.world
        .insert_one(entity, AgentState::Consuming { target: TargetKind::Sleep })
        .expect("entity just spawned");
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state_tag, 3,
        "A5: AgentState::Consuming{{Sleep}} must yield state_tag == 3, got {}",
        rows[0].state_tag
    );
    println!("[P11-α A5] Consuming(Sleep) → state_tag 3 ✓");
}

// ── A6 — snapshot x matches ECS Position.x ─────────────────────────────────
#[test]
fn harness_p11_alpha_position_x_matches_ecs() {
    let mut e = fresh_engine();
    e.spawn_agent(17, 3);
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].x, 17,
        "A6: snapshot x must equal spawn x=17, got {}",
        rows[0].x
    );
    println!("[P11-α A6] snapshot x == ECS Position.x (17) ✓");
}

// ── A7 — snapshot y matches ECS Position.y ─────────────────────────────────
#[test]
fn harness_p11_alpha_position_y_matches_ecs() {
    let mut e = fresh_engine();
    e.spawn_agent(4, 29);
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].y, 29,
        "A7: snapshot y must equal spawn y=29, got {}",
        rows[0].y
    );
    println!("[P11-α A7] snapshot y == ECS Position.y (29) ✓");
}

// ── A8 — snapshot agent_id matches Agent.id ────────────────────────────────
#[test]
fn harness_p11_alpha_agent_id_matches_component() {
    let mut e = fresh_engine();
    // AgentIds are minted sequentially; first agent gets id 0.
    e.spawn_agent(10, 10);
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    // The AgentId is a u64 minted by SimResources::issue_agent_id.
    // We verify it is a valid u64 (not some garbage) and consistent with
    // the sequential mint: first agent issued → id 0.
    assert_eq!(
        rows[0].agent_id, 0,
        "A8: first spawned agent_id must equal 0 (sequential mint), got {}",
        rows[0].agent_id
    );
    println!("[P11-α A8] agent_id matches Agent.id component (0) ✓");
}

// ── A9 — 5 agents → 5 snapshot rows ───────────────────────────────────────
#[test]
fn harness_p11_alpha_snapshot_count_matches_agent_count() {
    let mut e = fresh_engine();
    for i in 0..5u32 {
        e.spawn_agent(i * 3, i * 2);
    }
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(
        rows.len(),
        5,
        "A9: 5 agents must produce exactly 5 snapshot rows, got {}",
        rows.len()
    );
    println!("[P11-α A9] 5 agents → 5 rows ✓");
}

// ── A10 — all state_tag values in 0..=3 ────────────────────────────────────
#[test]
fn harness_p11_alpha_state_tag_in_valid_range() {
    let mut e = fresh_engine();
    // Spawn agents covering every reachable tag value.
    let a0 = e.spawn_agent(0, 0);
    let a1 = e.spawn_agent(1, 0);
    let a2 = e.spawn_agent(2, 0);
    let a3 = e.spawn_agent(3, 0);
    let a4 = e.spawn_agent(4, 0); // no AgentState → tag 0

    e.world.insert_one(a0, AgentState::Idle).unwrap();
    e.world
        .insert_one(a1, AgentState::Seeking { target: TargetKind::Water })
        .unwrap();
    e.world
        .insert_one(a2, AgentState::Consuming { target: TargetKind::Agent(7) })
        .unwrap();
    e.world
        .insert_one(a3, AgentState::Consuming { target: TargetKind::ConstructionSite })
        .unwrap();
    // a4 left without AgentState to exercise the None → 0 branch.
    let _ = a4;

    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 5);
    for row in &rows {
        assert!(
            row.state_tag <= 3,
            "A10: state_tag must be in 0..=3, got {}",
            row.state_tag
        );
    }
    // Verify all four tag values are present.
    let mut tags: Vec<u8> = rows.iter().map(|r| r.state_tag).collect();
    tags.sort();
    assert!(tags.contains(&0), "A10: tag 0 (Idle) must appear");
    assert!(tags.contains(&1), "A10: tag 1 (Seeking) must appear");
    assert!(tags.contains(&2), "A10: tag 2 (Consuming Agent) must appear");
    assert!(tags.contains(&3), "A10: tag 3 (Consuming other) must appear");
    println!("[P11-α A10] all state_tag values in 0..=3, all four present ✓");
}

// ── A11 — entity without AgentState component defaults to tag 0 ─────────────
#[test]
fn harness_p11_alpha_no_state_component_defaults_idle() {
    let mut e = fresh_engine();
    // spawn_agent only adds (Position, Agent{id}) — no AgentState.
    e.spawn_agent(5, 5);
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state_tag, 0,
        "A11: missing AgentState component must yield state_tag == 0, got {}",
        rows[0].state_tag
    );
    println!("[P11-α A11] no AgentState → state_tag 0 (Idle default) ✓");
}

// ─────────────────────────────────────────────────────────────────────────
// Static file-content assertions (A12–A17).
//
// The substrate tests above only prove that the FFI bridge surfaces the
// state_tag / x / y / agent_id fields needed by the GDScript renderer.
// They cannot prove that the GDScript renderer actually consumes those
// fields, that the MultiMesh is configured for per-instance color, or
// that the shader composites the tint over the palette output. The
// following six tests close that gap by scanning the modified source
// files for the contract markers required by the plan.
// ─────────────────────────────────────────────────────────────────────────

// ── A12 — agent_renderer.gd enables MultiMesh.use_colors = true ─────────────
#[test]
fn harness_p11_alpha_agent_renderer_enables_use_colors() {
    // Type: Type C invariant (static file substring). The MultiMesh must
    // be configured with use_colors = true so set_instance_color() takes
    // effect; if this flag is missing or false the entire tint feature is
    // a no-op.
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);
    let needle = "multi_mesh.use_colors = true";
    assert!(
        stripped.contains(needle),
        "A12: scripts/ui/agent_renderer.gd must enable `{needle}` \
         (state_tint requires per-instance color)"
    );
    // Negative check: the previous value `use_colors = false` must not
    // also be present (would indicate a re-toggle bug).
    assert!(
        !stripped.contains("multi_mesh.use_colors = false"),
        "A12: agent_renderer.gd must NOT also set `multi_mesh.use_colors = false`"
    );
    println!("[P11-α A12] agent_renderer.gd enables multi_mesh.use_colors = true ✓");
}

// ── A13 — agent_renderer.gd reads `states: PackedByteArray` from snapshot ───
#[test]
fn harness_p11_alpha_agent_renderer_reads_states_array() {
    // Type: Type C invariant. The renderer must pull the `states` key
    // from the FFI dict and store it in a `PackedByteArray` so the tag
    // bytes can be indexed per-agent.
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);
    // Tolerate single-vs-double quotes and minor spacing variations.
    let candidates = [
        "states: PackedByteArray = snap.get(\"states\", PackedByteArray())",
        "states: PackedByteArray = snap.get('states', PackedByteArray())",
    ];
    let matched = candidates.iter().any(|c| stripped.contains(c));
    assert!(
        matched,
        "A13: agent_renderer.gd must read `states: PackedByteArray = snap.get(\"states\", PackedByteArray())`. \
         None of the accepted forms found in code-body (comments stripped)."
    );
    println!("[P11-α A13] agent_renderer.gd reads states: PackedByteArray ✓");
}

// ── A14 — _prev_positions / _curr_positions interpolated with lerp_t ────────
#[test]
fn harness_p11_alpha_agent_renderer_interpolates_positions() {
    // Type: Type C invariant (multi-marker file substring). The Gaffer
    // accumulator pattern requires four code elements to be present:
    //   1. _prev_positions declared/used
    //   2. _curr_positions declared/used
    //   3. a `lerp_t` (clamped accumulator / SIM_TICK_DURATION)
    //   4. an actual `.lerp(…, lerp_t)` call so prev→curr is interpolated
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("_prev_positions"),
        "A14.1: agent_renderer.gd must reference `_prev_positions` for interpolation"
    );
    assert!(
        stripped.contains("_curr_positions"),
        "A14.2: agent_renderer.gd must reference `_curr_positions` for interpolation"
    );
    assert!(
        stripped.contains("lerp_t"),
        "A14.3: agent_renderer.gd must define `lerp_t` (clamped Gaffer accumulator)"
    );
    // The actual interpolation call: e.g. `prev_pos.lerp(curr_pos, lerp_t)`.
    assert!(
        stripped.contains(".lerp(") && stripped.contains("lerp_t)"),
        "A14.4: agent_renderer.gd must call `<vec>.lerp(<curr>, lerp_t)` so \
         the rendered pixel position is interpolated between ticks"
    );
    // The accumulator must be advanced and divided by SIM_TICK_DURATION
    // somewhere — that's what makes lerp_t a temporal blend rather than
    // a constant.
    assert!(
        stripped.contains("_lerp_accumulator")
            && stripped.contains("SIM_TICK_DURATION"),
        "A14.5: agent_renderer.gd must advance `_lerp_accumulator` and divide \
         by `SIM_TICK_DURATION` (Gaffer accumulator)"
    );
    println!(
        "[P11-α A14] agent_renderer.gd interpolates _prev_positions/_curr_positions with lerp_t ✓"
    );
}

// ── A15 — multi_mesh.set_instance_color(i, STATE_TINTS[...]) ────────────────
#[test]
fn harness_p11_alpha_agent_renderer_applies_state_tints() {
    // Type: Type C invariant. The renderer must:
    //   1. declare a STATE_TINTS palette covering 4 entries (Idle/Seeking/
    //      Consuming(Agent)/Consuming(other)),
    //   2. call multi_mesh.set_instance_color(i, STATE_TINTS[<tag>]) so
    //      each agent's instance color reflects its state_tag.
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("STATE_TINTS"),
        "A15.1: agent_renderer.gd must define a STATE_TINTS palette"
    );
    // Extract the STATE_TINTS array literal body — the slice between the
    // opening `[` after `STATE_TINTS: Array = ` and its matching `]`. We
    // use bracket-depth tracking so any future nested-array entry does not
    // confuse the boundary scan. Color( occurrences in RECALL_CUE_TINT,
    // COMBAT_CUE_TINT, and `_palette_for_id()` elsewhere in the file are
    // therefore excluded — only the palette block itself is inspected.
    let start_marker = "STATE_TINTS: Array = [";
    let block_open = stripped.find(start_marker).unwrap_or_else(|| {
        panic!(
            "A15.2: STATE_TINTS palette must be declared as \
             `STATE_TINTS: Array = [...]` (form not found in stripped source)"
        )
    });
    let after_open = block_open + start_marker.len();
    let tail = &stripped[after_open..];
    let mut depth: i32 = 1;
    let mut close_rel: Option<usize> = None;
    for (idx, ch) in tail.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    close_rel = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_rel = close_rel.unwrap_or_else(|| {
        panic!("A15.3: STATE_TINTS array must be terminated with a matching `]`")
    });
    let tints_block = &tail[..close_rel];

    // Exactly four `Color(...)` entries — one per locked state_tag value
    // (0=Idle, 1=Seeking, 2=Consuming(Agent), 3=Consuming(other)). With
    // exactly 4 entries and an index clamped to 0..=3 (asserted below),
    // every state_tag is guaranteed to land on an in-bounds palette color.
    let color_count_in_tints = tints_block.matches("Color(").count();
    assert_eq!(
        color_count_in_tints, 4,
        "A15.4: STATE_TINTS palette must contain EXACTLY 4 Color(...) entries \
         (one per state_tag 0..=3), found {color_count_in_tints}. Extracted block:\n{tints_block}"
    );

    // Each Color literal must be closed — guard against a malformed
    // `Color(1.0, 1.0, 1.0, 1.0` (missing `)`) producing a false positive.
    let close_paren_count = tints_block.matches(')').count();
    assert!(
        close_paren_count >= 4,
        "A15.5: STATE_TINTS block must contain >= 4 `)` (one per Color literal), \
         found {close_paren_count}. Extracted block:\n{tints_block}"
    );

    // set_instance_color call indexing STATE_TINTS by tag.
    assert!(
        stripped.contains("set_instance_color(") && stripped.contains("STATE_TINTS["),
        "A15.6: agent_renderer.gd must call \
         multi_mesh.set_instance_color(i, STATE_TINTS[<tag>])"
    );
    // The index expression must clamp the tag into 0..=3 so a stale or
    // out-of-range byte cannot crash the renderer. Combined with A15.4
    // (exactly 4 entries), this proves tags 0, 1, 2, and 3 each map to a
    // defined, in-bounds palette color.
    assert!(
        stripped.contains("clampi(") && stripped.contains(", 0, 3)"),
        "A15.7: state_tag → STATE_TINTS lookup must clamp into 0..=3 via clampi(..., 0, 3)"
    );
    println!(
        "[P11-α A15] STATE_TINTS palette has exactly 4 Color(...) entries; \
         set_instance_color(i, STATE_TINTS[clampi(tag, 0, 3)]) ✓"
    );
}

// ── A16 — palette_swap.gdshader captures `vec4 modulate = COLOR` ────────────
#[test]
fn harness_p11_alpha_shader_captures_instance_color() {
    // Type: Type C invariant. Without `vec4 modulate = COLOR` at the top
    // of fragment(), the per-instance tint written by set_instance_color()
    // is overwritten before it can be applied — the visual feature dies
    // silently in the shader.
    let src = read_palette_shader_src();
    let stripped = strip_glsl_line_comments(&src);

    // Locate the fragment() body and verify the capture happens inside it.
    let frag_start = stripped
        .find("void fragment()")
        .expect("A16: palette_swap.gdshader must define void fragment()");
    let frag_body = &stripped[frag_start..];
    assert!(
        frag_body.contains("vec4 modulate = COLOR"),
        "A16: palette_swap.gdshader fragment() must capture `vec4 modulate = COLOR` \
         before the palette swap overwrites COLOR"
    );
    println!("[P11-α A16] palette_swap.gdshader captures vec4 modulate = COLOR ✓");
}

// ── A17 — shader multiplies palette_color.rgb * modulate.rgb (visible α) ────
#[test]
fn harness_p11_alpha_shader_modulates_palette_output() {
    // Type: Type C invariant. The tint must multiply the palette output
    // in the visible-alpha branch (tex.a > 0.01); the transparent branch
    // is irrelevant because it never contributes to a rendered pixel. We
    // verify both the multiplication string and that it lives after the
    // alpha guard so transparent fragments are not also tinted.
    let src = read_palette_shader_src();
    let stripped = strip_glsl_line_comments(&src);

    // Must contain the multiplication itself (any whitespace variation).
    let multiplies = stripped.contains("palette_color.rgb * modulate.rgb")
        || stripped.contains("modulate.rgb * palette_color.rgb");
    assert!(
        multiplies,
        "A17.1: palette_swap.gdshader must multiply `palette_color.rgb * modulate.rgb` \
         so the per-instance state tint composites over the palette output"
    );

    // Must live after the `tex.a <= 0.01` early-out (i.e. in the visible
    // branch). We find the early-out position and confirm the multiply
    // appears after it.
    let alpha_guard_pos = stripped
        .find("tex.a <= 0.01")
        .expect("A17.2: shader must contain the `tex.a <= 0.01` alpha guard");
    let multiply_pos = stripped
        .find("palette_color.rgb * modulate.rgb")
        .or_else(|| stripped.find("modulate.rgb * palette_color.rgb"))
        .expect("A17.3: multiply expression must be findable for ordering check");
    assert!(
        multiply_pos > alpha_guard_pos,
        "A17.4: the palette_color.rgb * modulate.rgb multiply must appear \
         after the `tex.a <= 0.01` alpha guard so transparent fragments \
         are not tinted (multiply_pos={multiply_pos}, alpha_guard_pos={alpha_guard_pos})"
    );
    println!(
        "[P11-α A17] palette_swap.gdshader multiplies palette_color.rgb * modulate.rgb in visible-α path ✓"
    );
}

// ── A18 — checksum helper must iterate every row, not `mini(n, 32)` ─────────
#[test]
fn harness_p11_alpha_checksum_covers_all_rows() {
    // Type: Type C invariant (static-body extraction). The previous
    // attempt truncated the checksum hash to the first 32 rows
    // (`for i in mini(n, 32):`). With 10K agents that means 99.68% of
    // rows are ignored — a position change on agent #33+ does not flip
    // the checksum, the tick-boundary detector never fires, and
    // interpolation freezes on the first 32 agents. This test extracts
    // the body of `_snapshot_checksum_from(...)` and forbids the
    // truncation pattern while requiring a full-range iteration.
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);

    // Locate the function definition line.
    let fn_start = stripped
        .find("func _snapshot_checksum_from(")
        .expect("A18: agent_renderer.gd must define _snapshot_checksum_from(...)");
    // Conservatively bound the body: the next top-level `func ` keyword
    // (or EOF) terminates the helper's body.
    let after_sig = fn_start + "func _snapshot_checksum_from(".len();
    let tail = &stripped[after_sig..];
    let body_end_rel = tail.find("\nfunc ").unwrap_or(tail.len());
    let body = &tail[..body_end_rel];

    // Negative guard: the truncation pattern is FORBIDDEN.
    assert!(
        !body.contains("mini(n, 32)") && !body.contains("mini(n,32)"),
        "A18.1: _snapshot_checksum_from MUST NOT truncate to `mini(n, 32)` — \
         tick-boundary detection must cover every rendered agent, not just \
         the first 32. Found truncation pattern in body:\n{body}"
    );
    // Generic guard: any `mini(n, <const>)` constant cap is forbidden.
    // The signature passes `n` explicitly so the helper must honour it.
    let mini_n_pos = body.find("mini(n,");
    assert!(
        mini_n_pos.is_none(),
        "A18.2: _snapshot_checksum_from MUST NOT cap iteration via \
         `mini(n, <k>)` — must iterate every row. Found at offset {:?} in body:\n{body}",
        mini_n_pos
    );

    // Positive guard: the body must iterate over the full `n` range.
    // Accept either `for i in n:` or `for i in range(n):` as GDScript-
    // idiomatic full-range loops.
    let iterates_all = body.contains("for i in n:")
        || body.contains("for i in range(n):")
        || body.contains("for i in range(0, n):");
    assert!(
        iterates_all,
        "A18.3: _snapshot_checksum_from must iterate every row \
         (e.g. `for i in n:` or `for i in range(n):`). Body:\n{body}"
    );
    println!(
        "[P11-α A18] _snapshot_checksum_from iterates every row (no mini(n, 32) truncation) ✓"
    );
}

// ── A19 — checksum must be identity-aware (agent_ids mixed in) ──────────────
#[test]
fn harness_p11_alpha_checksum_is_identity_aware() {
    // Type: Type C invariant (static signature + body + call-site check).
    // The previous helper XOR-folded only positions (`xs`, `ys`) and was
    // therefore order-insensitive: swapping two agents whose positions
    // were unchanged silently produced the same hash, masking a tick
    // boundary. The fix requires:
    //   1. The helper signature includes `agent_ids` as a parameter.
    //   2. The body references `agent_ids[i]` so identity is folded in.
    //   3. The call site forwards `agent_ids` (not just `xs, ys, n`).
    //   4. The body mixes index `i` (or uses a non-commutative
    //      accumulator step) so a pure XOR over per-row values cannot
    //      cancel out.
    let src = read_agent_renderer_src();
    let stripped = strip_gd_comments(&src);

    // 1. Signature must include agent_ids.
    let sig_with_ids = stripped.contains("func _snapshot_checksum_from(agent_ids:")
        || stripped.contains("func _snapshot_checksum_from(\n\tagent_ids:")
        || stripped
            .contains("func _snapshot_checksum_from(agent_ids: PackedInt64Array");
    assert!(
        sig_with_ids,
        "A19.1: _snapshot_checksum_from signature MUST include `agent_ids` \
         as a parameter (identity-aware hashing). Current source does not match \
         any accepted form."
    );

    // Extract body again for finer-grained body checks.
    let fn_start = stripped
        .find("func _snapshot_checksum_from(")
        .expect("A19: _snapshot_checksum_from must be defined");
    let after_sig = fn_start + "func _snapshot_checksum_from(".len();
    let tail = &stripped[after_sig..];
    let body_end_rel = tail.find("\nfunc ").unwrap_or(tail.len());
    let body = &tail[..body_end_rel];

    // 2. Body must mix agent_ids[i] (identity).
    assert!(
        body.contains("agent_ids[i]"),
        "A19.2: _snapshot_checksum_from body MUST mix `agent_ids[i]` into \
         the hash so two snapshots that differ only in identity are \
         distinguishable. Body:\n{body}"
    );

    // 3. Body must mix the index `i` (order awareness) — a pure XOR over
    // commutative per-row values can cancel; combining `i` (or a
    // multiplicative accumulator step) prevents that.
    let mixes_index = body.contains("* i")
        || body.contains("(i *")
        || body.contains("i * ")
        || body.contains("i)")
            && (body.contains("h * ") || body.contains("h *= "));
    assert!(
        mixes_index,
        "A19.3: _snapshot_checksum_from body MUST mix the index `i` \
         (or use a non-commutative accumulator) so XOR over per-row \
         values cannot cancel. Body:\n{body}"
    );

    // 4. Call site must forward `agent_ids` to the helper.
    let call_with_ids = stripped.contains("_snapshot_checksum_from(agent_ids,")
        || stripped.contains("_snapshot_checksum_from(\n\t\tagent_ids,");
    assert!(
        call_with_ids,
        "A19.4: the `_snapshot_checksum_from` call site MUST forward \
         `agent_ids` as the first argument so identity participates in \
         tick-boundary detection."
    );
    println!(
        "[P11-α A19] _snapshot_checksum_from is identity-aware (signature + body + call site) ✓"
    );
}
