//! P4-γ — Agent snapshot FFI surface (V7 Phase 4-γ, Sprite Rendering).
//!
//! Builds on P4-α (`Position`, `Agent`, `SimEngine::spawn_agent`) and
//! P4-β (`AgentMovementSystem`, `MovementRng`). Adds the pure-Rust
//! collector `sim_bridge::ffi::collect_agent_snapshot` backing the new
//! `WorldSimNode::get_agent_snapshot` FFI method.
//!
//! Per planning §2.3 (`.harness/plans/phase4.md` lines 156-188):
//!   - FFI returns 3 parallel PackedArrays (ids / xs / ys).
//!   - Renderer is a separate GDScript file (visual layer, not in this
//!     harness — the Godot harness covers it).
//!   - `--quick` pipeline lane (no sim-core/engine/systems change).
//!   - Performance gate 1K@60FPS (the perf tripwire here is a 5 ms
//!     budget on 1024-agent collection, well inside the 16.6 ms frame).
//!
//! Threshold types follow `.harness/policy/evaluation_criteria.md`:
//!   - Type A — structural invariants (compile-time / shape / equality)
//!   - Type D — regression guards (pre-existing runtime paths stay green)
//!
//! Assertions (11):
//!   1.  Type A — `sim_bridge::ffi::collect_agent_snapshot` symbol resolves
//!       at the expected re-export path.
//!   2.  Type A — empty world → empty `Vec<AgentSnapshotRow>`.
//!   3.  Type A — single agent spawned at (7, 11) produces one row whose
//!       `x == 7` and `y == 11`.
//!   4.  Type A — three agents at (1,2), (3,4), (5,6) produce a 3-row
//!       result whose `(x, y)` multiset equals the input multiset.
//!   5.  Type A — the `entity_bits` field equals `Entity::to_bits().get()`
//!       for the spawned entity (FFI id identity).
//!   6.  Type A — two consecutive calls on an unchanged world produce
//!       byte-identical `Vec<AgentSnapshotRow>` (hecs archetype order
//!       determinism).
//!   7.  Type A — boundary positions (0, 0) and (W-1, H-1) survive a
//!       snapshot round-trip with no drift (u32 boundary integrity).
//!   8.  Type D — after `register_phase2_systems + register_agent_systems`
//!       and one `engine.tick()`, the snapshot reflects the post-β-move
//!       position (β integration regression guard).
//!   9.  Type A — an entity carrying `Position` but **no** `Agent` marker
//!       is excluded from the snapshot (query filter enforcement).
//!  10.  Type A — over spawn sizes {0, 1, 64} the three parallel arrays
//!       produced by `agent_rows_split` (the pure-Rust core of
//!       `agent_rows_to_dict`) all have length `n` and values matching the
//!       row fields. This proves the marshalling invariant relied on by
//!       the Godot-side PackedArray contract.
//!  11.  Type D — performance smoke: collecting 1024 agents completes in
//!       under 5 ms on the test runner (regression tripwire, not a hard
//!       guarantee).
//!
//! Run: `cargo test -p sim-test --test harness_p4_gamma_rendering -- --nocapture`
//! Discovery: all test fns use the `harness_p4_gamma_*` prefix so the
//! repo-wide `cargo test -p sim-test harness_ -- --nocapture` selector
//! picks them up.

use sim_bridge::ffi::{agent_rows_split, collect_agent_snapshot, AgentSnapshotRow};
use sim_core::components::Position;
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::{register_agent_systems, register_phase2_systems};
use std::time::Instant;

const W: u32 = 64;
const H: u32 = 64;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

// ── Assertion 1 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a1_symbol_resolves() {
    // Type A — referencing the function and the row type compiles, which
    // is the structural proof that the FFI surface is wired correctly.
    let f: fn(&hecs::World) -> Vec<AgentSnapshotRow> = collect_agent_snapshot;
    let _ = f;
    println!("[γ-A1] collect_agent_snapshot symbol resolves ✓");
}

// ── Assertion 2 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a2_empty_world_is_empty_vec() {
    let e = fresh_engine();
    let rows = collect_agent_snapshot(&e.world);
    assert!(rows.is_empty(), "empty world must yield empty snapshot");
    println!("[γ-A2] empty world → empty Vec ✓");
}

// ── Assertion 3 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a3_single_agent_position_preserved() {
    let mut e = fresh_engine();
    e.spawn_agent(7, 11);
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1, "exactly one row for one agent");
    assert_eq!(rows[0].x, 7);
    assert_eq!(rows[0].y, 11);
    println!("[γ-A3] single agent (7,11) survives snapshot ✓");
}

// ── Assertion 4 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a4_three_agents_multiset_match() {
    let mut e = fresh_engine();
    e.spawn_agent(1, 2);
    e.spawn_agent(3, 4);
    e.spawn_agent(5, 6);
    let rows = collect_agent_snapshot(&e.world);
    let mut actual: Vec<(u32, u32)> = rows.iter().map(|r| (r.x, r.y)).collect();
    actual.sort();
    let mut expected = vec![(1, 2), (3, 4), (5, 6)];
    expected.sort();
    assert_eq!(actual, expected, "(x,y) multiset must match input");
    println!("[γ-A4] 3-agent multiset matches input ✓");
}

// ── Assertion 5 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a5_entity_bits_identity() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(2, 2);
    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].entity_bits,
        entity.to_bits().get(),
        "snapshot entity_bits must equal Entity::to_bits().get()"
    );
    println!("[γ-A5] entity_bits == Entity::to_bits().get() ✓");
}

// ── Assertion 6 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a6_order_stable_across_calls() {
    let mut e = fresh_engine();
    for i in 0..16u32 {
        e.spawn_agent(i, i);
    }
    let first = collect_agent_snapshot(&e.world);
    let second = collect_agent_snapshot(&e.world);
    assert_eq!(
        first, second,
        "two consecutive snapshots on unchanged world must be identical"
    );
    println!("[γ-A6] consecutive snapshots byte-identical ✓");
}

// ── Assertion 7 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a7_boundary_positions_preserved() {
    let mut e = fresh_engine();
    e.spawn_agent(0, 0);
    e.spawn_agent(W - 1, H - 1);
    let rows = collect_agent_snapshot(&e.world);
    let mut coords: Vec<(u32, u32)> = rows.iter().map(|r| (r.x, r.y)).collect();
    coords.sort();
    assert_eq!(coords, vec![(0, 0), (W - 1, H - 1)]);
    println!("[γ-A7] boundary positions (0,0), (W-1,H-1) preserved ✓");
}

// ── Assertion 8 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_d8_post_tick_reflects_beta_motion() {
    let mut e = fresh_engine();
    register_phase2_systems(&mut e);
    register_agent_systems(&mut e);
    let entity = e.spawn_agent(32, 32);
    e.world
        .insert_one(entity, MovementRng::new(0xDEAD_BEEF))
        .expect("entity just spawned, must accept insert_one");

    let pre = collect_agent_snapshot(&e.world);
    assert_eq!(pre.len(), 1);
    let (px, py) = (pre[0].x, pre[0].y);
    assert_eq!((px, py), (32, 32));

    // Brownian step is in {-1, 0, +1} per axis, so at least one of the
    // first few ticks must produce a position different from (32, 32);
    // we iterate up to 8 ticks (probability of zero motion on every
    // axis = (1/3)^16 ≈ 2e-8 — safely deterministic to fail on stall).
    let mut moved_at: Option<u32> = None;
    for t in 0..8 {
        e.tick();
        let post = collect_agent_snapshot(&e.world);
        if post[0].x != 32 || post[0].y != 32 {
            moved_at = Some(t);
            break;
        }
    }
    assert!(
        moved_at.is_some(),
        "β movement must shift agent away from (32,32) within 8 ticks"
    );
    println!("[γ-D8] β movement reflected post-tick at tick {} ✓", moved_at.unwrap());
}

// ── Assertion 9 ────────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a9_position_without_agent_excluded() {
    let mut e = fresh_engine();
    // Spawn one full agent (with the marker) and one bare Position.
    e.spawn_agent(10, 10);
    e.world.spawn((Position::new(20, 20),));

    let rows = collect_agent_snapshot(&e.world);
    assert_eq!(rows.len(), 1, "only the `Agent`-tagged entity is included");
    assert_eq!((rows[0].x, rows[0].y), (10, 10));

    // Sanity — both entities are reachable when we drop the Agent filter.
    let total_with_pos = e.world.query::<&Position>().iter().count();
    assert_eq!(total_with_pos, 2);
    println!("[γ-A9] Position-only entity excluded ✓");
}

// ── Assertion 10 ───────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_a10_three_parallel_arrays_length_and_values() {
    // Exercises `agent_rows_split` — the pure-Rust core of
    // `agent_rows_to_dict`. The Godot-side PackedArray contract derives
    // length and per-index values directly from this function.
    for n in [0u32, 1, 64] {
        let mut e = fresh_engine();
        for i in 0..n {
            e.spawn_agent(i % W, i % H);
        }
        let rows = collect_agent_snapshot(&e.world);
        let (ids, xs, ys, states) = agent_rows_split(&rows);

        // Length invariant on all four arrays.
        assert_eq!(rows.len(), n as usize, "row count must equal n = {}", n);
        assert_eq!(ids.len(), n as usize, "ids length must equal n = {}", n);
        assert_eq!(xs.len(), n as usize, "xs length must equal n = {}", n);
        assert_eq!(ys.len(), n as usize, "ys length must equal n = {}", n);
        assert_eq!(states.len(), n as usize, "states length must equal n = {}", n);

        // Per-index value identity: ids/xs/ys must mirror row fields.
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(ids[i], row.entity_bits as i64, "ids[{}] identity", i);
            assert_eq!(xs[i], row.x as i32, "xs[{}] identity", i);
            assert_eq!(ys[i], row.y as i32, "ys[{}] identity", i);
        }
    }
    println!("[γ-A10] ids/xs/ys length+value invariants hold for n ∈ {{0,1,64}} ✓");
}

// ── Assertion 11 ───────────────────────────────────────────────────────
#[test]
fn harness_p4_gamma_d11_perf_smoke_1024_agents_under_5ms() {
    let mut e = fresh_engine();
    // 1024 agents on a deterministic 32×32 sub-grid.
    for j in 0..32u32 {
        for i in 0..32u32 {
            e.spawn_agent(i, j);
        }
    }
    // Warm-up — first hecs query iteration may touch fresh cache lines.
    let _ = collect_agent_snapshot(&e.world);

    let t0 = Instant::now();
    let rows = collect_agent_snapshot(&e.world);
    let dt = t0.elapsed();
    assert_eq!(rows.len(), 1024);
    assert!(
        dt.as_millis() < 5,
        "1024-agent snapshot must finish in <5ms (got {} ms)",
        dt.as_millis()
    );
    println!("[γ-D11] 1024-agent collect {:?} (<5ms) ✓", dt);
}
