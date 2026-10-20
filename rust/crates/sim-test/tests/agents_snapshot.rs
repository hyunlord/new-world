//! Merged in V8-0 (8.4) from `harness_p4_gamma_rendering.rs`,
//! `harness_p11_alpha_agent_renderer.rs`, `harness_carry_viz_2_5b.rs`; each
//! source is one module below, unchanged.

// From `harness_p4_gamma_rendering.rs`.
mod rendering {
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
        println!(
            "[γ-D8] β movement reflected post-tick at tick {} ✓",
            moved_at.unwrap()
        );
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
            assert_eq!(
                states.len(),
                n as usize,
                "states length must equal n = {}",
                n
            );

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
}

// From `harness_p11_alpha_agent_renderer.rs`.
mod renderer_substrate {
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
    //! The V7 static file-content checks of `agent_renderer.gd` and
    //! `palette_swap.gdshader` (A12–A22) were deleted in V8-0 with the V7
    //! renderer tints they pinned; only the collector tests above remain.
    //!
    //! Run: `cargo test -p sim-test --test agents_snapshot renderer_substrate`

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
}

// From `harness_carry_viz_2_5b.rs`.
mod carried_food {
    //! V7 Direction-2 slice 2-5b — carry viz: `AgentSnapshotRow.carried_food` collector.
    //!
    //! Bridge + GDScript only (no sim-core/systems/engine change), so the sole
    //! Rust-side coverage is this pure-collector test on
    //! `sim_bridge::ffi::collect_agent_snapshot` — mirroring the
    //! `harness_p4_gamma_rendering` pattern (build a `SimEngine` world, spawn
    //! agents, assert on the returned rows; no Godot runtime needed).
    //!
    //! The `carried_foods` `PackedInt32Array` in `agent_rows_to_dict` is built by
    //! `carried_foods[i] = rows[i].carried_food` with `resize(rows.len())` —
    //! identical in shape to the proven `seek_kinds`/`hungers` additive arrays — so
    //! the FFI-marshalling length invariant (`carried_foods.len() == rows.len()`)
    //! is structural; this test pins the `carried_food` ROW value the array copies.
    //!
    //! Assertions track the locked plan (stockpile-viz-2-5b):
    //!   A1: Food=7 → carried_food == 7 (sub-capacity exact, not a default/`> 0`).
    //!   A2: Water=4, Food=0 → carried_food == 0 (anti-gaming: a `total()`-based
    //!       bug would return 4; proves the field reads `get(Food)` specifically).
    //!   A3: no Inventory → carried_food == 0 (the `Option<&Inventory>` None branch).
    //!   A4: rows.len() == count of (Agent, Position) entities (parallel-array
    //!       contract — `Option` must NOT filter rows out).
    //!   A5: live Inventory mutation — 7 → 0 on deposit (reads live ECS per call).
    //!
    //! Run: `cargo test -p sim-test --test harness_carry_viz_2_5b -- --nocapture`

    use sim_bridge::ffi::collect_agent_snapshot;
    use sim_core::components::{Agent, Inventory, Position, ResourceKind};
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    const W: u32 = 64;
    const H: u32 = 64;

    fn fresh_engine() -> SimEngine {
        SimEngine::new(W, H, MaterialRegistry::new())
    }

    /// Look up the `carried_food` for a specific entity in the snapshot rows.
    fn carried_food_for(rows: &[sim_bridge::ffi::AgentSnapshotRow], ent: hecs::Entity) -> i32 {
        rows.iter()
            .find(|r| r.entity_bits == ent.to_bits().get())
            .unwrap_or_else(|| panic!("snapshot row for {ent:?} missing"))
            .carried_food
    }

    #[test]
    fn harness_carry_viz_agent_snapshot_carried_food() {
        let mut e = fresh_engine();

        // Agent A — carries Food = 7 (plan A1: non-trivial, < INVENTORY_CAPACITY=10
        // so add() does not clamp; defeats any default / `> 0` shortcut).
        let a = e.spawn_agent(5, 5);
        let mut inv_a = Inventory::default();
        inv_a.add(ResourceKind::Food, 7);
        let _ = e.world.insert_one(a, inv_a);

        // Agent B — Inventory holds Water = 4 but Food = 0 (plan A2 anti-gaming:
        // an implementation that read inv.total() or the wrong kind returns 4 here).
        let b = e.spawn_agent(6, 6);
        let mut inv_b = Inventory::default();
        inv_b.add(ResourceKind::Water, 4);
        let _ = e.world.insert_one(b, inv_b);

        // Agent C — NO Inventory component at all (plan A3: the None branch → 0).
        let c = e.spawn_agent(7, 7);

        let rows = collect_agent_snapshot(&e.world);

        // Plan A4 — parallel-array contract: one row per (Agent, Position) entity.
        // The Option<&Inventory> in the query tuple must NOT drop the no-inventory
        // agent. Compare against the independently-counted (Agent, Position) set.
        let agent_pos_count = e.world.query::<(&Agent, &Position)>().iter().count();
        // Type A: rows.len() == count of (Agent, Position) entities (exact).
        assert_eq!(
            rows.len(),
            agent_pos_count,
            "rows.len() must equal the (Agent, Position) entity count (Option non-filtering)"
        );
        // Type A: with three spawned agents, that count is exactly 3.
        assert_eq!(
            rows.len(),
            3,
            "exactly one snapshot row per spawned agent (3)"
        );

        // Type A: agent carrying Food=7 reports carried_food == 7 (exact).
        assert_eq!(
            carried_food_for(&rows, a),
            7,
            "Food=7 → carried_food=7 (plan A1)"
        );
        // Type A: Water=4 / Food=0 reports carried_food == 0 (anti-total() guard).
        assert_eq!(
            carried_food_for(&rows, b),
            0,
            "Water=4, Food=0 → carried_food=0 (plan A2: reads get(Food), not total())"
        );
        // Type A: agent with no Inventory reports carried_food == 0 (None branch).
        assert_eq!(
            carried_food_for(&rows, c),
            0,
            "no Inventory → carried_food=0 (plan A3)"
        );

        println!(
            "[2-5b] carried_food: A(Food=7)=7, B(Water=4,Food=0)=0, C(no-inv)=0; rows={} ✓",
            rows.len()
        );
    }

    #[test]
    fn harness_carry_viz_carried_food_tracks_inventory_change() {
        // Plan A5 — carried_food reflects the LIVE Inventory each call: it drops to
        // 0 when a deposit drains Food. A constant-return stub cannot satisfy 7 → 0.
        let mut e = fresh_engine();
        let a = e.spawn_agent(4, 4);
        let mut inv = Inventory::default();
        inv.add(ResourceKind::Food, 7);
        let _ = e.world.insert_one(a, inv);

        let before = collect_agent_snapshot(&e.world);
        // Type A: first collection carried_food == 7.
        assert_eq!(
            before[0].carried_food, 7,
            "before deposit: carried_food=7 (plan A5)"
        );

        // Simulate the real deposit event (2-3a drains Inventory Food → stockpile).
        if let Ok(mut inv) = e.world.get::<&mut Inventory>(a) {
            let n = inv.get(ResourceKind::Food);
            inv.remove(ResourceKind::Food, n);
        }

        let after = collect_agent_snapshot(&e.world);
        // Type A: second collection carried_food == 0.
        assert_eq!(
            after[0].carried_food, 0,
            "after deposit: carried_food drops to 0 (plan A5)"
        );
        println!("[2-5b] carried_food tracks live Inventory: 7 → 0 on deposit ✓");
    }
}
