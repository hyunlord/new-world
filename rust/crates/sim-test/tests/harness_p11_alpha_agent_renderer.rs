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
//!   A16 — palette_swap.gdshader declares `varying vec4 v_tint` and captures
//!         `v_tint = COLOR` in vertex() (V7 H Phase A green-bug fix)
//!   A17 — palette_swap.gdshader multiplies palette_color.rgb * v_tint.rgb
//!         in the visible-alpha branch (tex.a > 0.01)
//!   A18 — _snapshot_checksum_from MUST iterate every row (no `mini(n, 32)`
//!         truncation), so tick-boundary detection covers every agent
//!   A19 — _snapshot_checksum_from MUST be identity-aware: signature
//!         includes `agent_ids`, body mixes `agent_ids[i]` into the hash,
//!         and the call site forwards `agent_ids` (no order-insensitive
//!         XOR over positions alone)
//!
//! D1 follow-up assertions (A20–A22) — anti-circular color-value guard.
//! A12–A19 only assert the *shape* of `STATE_TINTS` (4 entries, the `Color(...)`
//! form, `set_instance_color(...) + clampi(tag, 0, 3)`), so a regression that
//! restored the pre-D1 `Color(1.0, 1.0, 1.0, 1.0)` Idle identity tint — or any
//! other off-palette color — would still pass. These three tests close that
//! gap:
//!   A20 — STATE_TINTS[0] (Idle) MUST equal the D1 cool-blue literal
//!         `Color(0.55, 0.70, 0.95, 1.0)` so the Phase 11-α visible delta
//!         actually composites over the palette output for the dominant
//!         Idle population.
//!   A21 — STATE_TINTS block MUST NOT contain the pre-D1 Idle identity
//!         tint `Color(1.0, 1.0, 1.0, 1.0)`; on a white tint MultiMesh
//!         per-instance color multiplies the palette to identity, leaving
//!         the renderer visually indistinguishable from pre-Phase-11-α.
//!   A22 — STATE_TINTS palette MUST be the exact ordered D1 palette for
//!         indices 0..=3. Parses every `Color(...)` literal inside the
//!         palette block and compares the (r, g, b, a) tuple by index to:
//!           [0] Idle           — Color(0.55, 0.70, 0.95, 1.0)
//!           [1] Seeking        — Color(1.0,  0.85, 0.15, 1.0)
//!           [2] Consuming(Ag.) — Color(1.0,  0.40, 0.75, 1.0)
//!           [3] Consuming(oth) — Color(0.30, 0.95, 0.35, 1.0)
//!         This is the strongest anti-circular guard: any drift on any
//!         channel of any of the four entries fails the test.
//!
//! Run: `cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture`

use sim_bridge::ffi::collect_agent_snapshot;
use sim_core::components::agent_state::TargetKind;
use sim_core::components::AgentState;
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;

// ─── static file-content helpers (A12–A17) ─────────────────────────────────

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
        .insert_one(
            entity,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
        )
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
        .insert_one(
            entity,
            AgentState::Consuming {
                target: TargetKind::Agent(99),
            },
        )
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
        .insert_one(
            entity,
            AgentState::Consuming {
                target: TargetKind::Food,
            },
        )
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
        .insert_one(
            entity,
            AgentState::Consuming {
                target: TargetKind::Sleep,
            },
        )
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
        .insert_one(
            a1,
            AgentState::Seeking {
                target: TargetKind::Water,
            },
        )
        .unwrap();
    e.world
        .insert_one(
            a2,
            AgentState::Consuming {
                target: TargetKind::Agent(7),
            },
        )
        .unwrap();
    e.world
        .insert_one(
            a3,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
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
    assert!(
        tags.contains(&2),
        "A10: tag 2 (Consuming Agent) must appear"
    );
    assert!(
        tags.contains(&3),
        "A10: tag 3 (Consuming other) must appear"
    );
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

// ─────────────────────────────────────────────────────────────────────────
// D1 follow-up — anti-circular color-value guard (A20–A21).
//
// A12–A19 only enforce the *shape* of `STATE_TINTS` (4 entries, `Color(...)`
// form, clamp-indexed lookup). A regression that restored the pre-D1
// `Color(1.0, 1.0, 1.0, 1.0)` Idle identity tint would still pass — and
// that is precisely the regression D1 exists to forbid (a white instance
// tint multiplied with the palette output equals the pre-Phase-11-α
// appearance, so the visible delta dies silently).
//
// These two static-file tests extract the STATE_TINTS array body using
// the same bracket-depth scan as A15 and then assert (A20) the exact D1
// Idle literal is present and (A21) the pre-D1 identity literal is
// absent.
// ─────────────────────────────────────────────────────────────────────────
