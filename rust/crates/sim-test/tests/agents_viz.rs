//! Merged in V8-0 (8.4) from `harness_s16_epsilon_state_viz.rs`,
//! `harness_show_agent_needs_bars.rs`, `harness_s16_gamma_visual.rs`; each
//! source is one module below, unchanged.

// From `harness_s16_epsilon_state_viz.rs`.
mod state_markers {
    //! V7 Section 16-ε — state head-markers + goal-lines FFI harness.
    //!
    //! feature: s16-epsilon-state-viz
    //! seed: 42
    //! agent_count: 20 (struct-fixture assertions) / 64 (live bootstrap, A7)
    //! lane: --quick
    //!
    //! ε is a read-only visualization extension. It adds three additive parallel
    //! fields to `AgentSnapshotRow` (`seek_kind: u8`, `target_x: i32`,
    //! `target_y: i32`) carried by NEW parallel snapshot arrays — leaving the
    //! locked `state_tag` (P11-α A22) and the `agent_rows_split` 4-tuple
    //! (P4-γ) intact. No simulation behavior change.
    //!
    //! Non-circular rule: the struct-fixture assertions (1-6) build agents in
    //! KNOWN `AgentState` / `SeekTarget` configurations and read the snapshot
    //! with NO simulation step, so every expected value is hand-determined from
    //! the controlled fixture, never re-derived from production code. A7 is the
    //! complementary live-engine range/parity check on real bootstrap output.
    //!
    //! Run:
    //!   cargo test -p sim-test --test harness_s16_epsilon_state_viz -- --nocapture

    use sim_bridge::ffi::bootstrap_spawn_agents;
    use sim_bridge::ffi::{collect_agent_snapshot, AgentSnapshotRow};
    use sim_core::components::{Agent, AgentState, Hunger, SeekTarget, TargetKind};
    use sim_core::material::MaterialRegistry;
    use sim_engine::{SimEngine, RESOURCE_SOURCE_INFINITE};
    use sim_systems::register_default_runtime_systems;
    use sim_systems::runtime::agent::MovementRng;

    const W: u32 = 64;
    const H: u32 = 64;

    // ── helpers ─────────────────────────────────────────────────────────────────

    /// Fresh 64×64 engine (no systems registered — struct fixtures only).
    fn engine() -> SimEngine {
        SimEngine::new(W, H, MaterialRegistry::new())
    }

    /// Spawn an agent at `(x,y)`, attach `state`, and optionally a `SeekTarget`.
    fn spawn_with(
        e: &mut SimEngine,
        x: u32,
        y: u32,
        state: AgentState,
        seek: Option<SeekTarget>,
    ) -> hecs::Entity {
        let ent = e.spawn_agent(x, y);
        e.world.insert_one(ent, state).expect("insert AgentState");
        if let Some(st) = seek {
            e.world.insert_one(ent, st).expect("insert SeekTarget");
        }
        ent
    }

    /// The snapshot row whose `entity_bits` matches `ent`. Panics if absent.
    fn row_for(rows: &[AgentSnapshotRow], ent: hecs::Entity) -> AgentSnapshotRow {
        let bits = ent.to_bits().get();
        *rows
            .iter()
            .find(|r| r.entity_bits == bits)
            .expect("snapshot row for entity")
    }

    // ─── Assertion 1: seek_kind maps Food→1, Water→2, Sleep→3 ──────────────────
    #[test]
    fn harness_s16_epsilon_a1_seek_kind_resource_mapping() {
        // Type A — enum→code invariant (1=Food, 2=Water, 3=Sleep), exact equality.
        let mut e = engine();
        let food = spawn_with(
            &mut e,
            5,
            5,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            None,
        );
        let water = spawn_with(
            &mut e,
            6,
            6,
            AgentState::Seeking {
                target: TargetKind::Water,
            },
            None,
        );
        let sleep = spawn_with(
            &mut e,
            7,
            7,
            AgentState::Seeking {
                target: TargetKind::Sleep,
            },
            None,
        );

        let rows = collect_agent_snapshot(&e.world);
        assert_eq!(
            row_for(&rows, food).seek_kind,
            1,
            "A1: Seeking{{Food}} → seek_kind 1"
        );
        assert_eq!(
            row_for(&rows, water).seek_kind,
            2,
            "A1: Seeking{{Water}} → seek_kind 2"
        );
        assert_eq!(
            row_for(&rows, sleep).seek_kind,
            3,
            "A1: Seeking{{Sleep}} → seek_kind 3"
        );
        println!("[S16-ε A1] seek_kind Food=1, Water=2, Sleep=3 ✓");
    }

    // ─── Assertion 2: target_x / target_y echo the SeekTarget tile exactly ─────
    #[test]
    fn harness_s16_epsilon_a2_target_coords_echo_seek_tile() {
        // Type A — direct u32→i32 cast of SeekTarget.tile.0/.1. x≠y (10≠20)
        // makes an axis swap detectable; both must hold exactly.
        let mut e = engine();
        let a = spawn_with(
            &mut e,
            3,
            4,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((10, 20))),
        );
        let rows = collect_agent_snapshot(&e.world);
        let row = row_for(&rows, a);
        assert_eq!(
            row.target_x, 10,
            "A2: target_x must equal SeekTarget.tile.0 (10)"
        );
        assert_eq!(
            row.target_y, 20,
            "A2: target_y must equal SeekTarget.tile.1 (20)"
        );
        println!("[S16-ε A2] SeekTarget(10,20) → target_x=10, target_y=20 (no axis swap) ✓");
    }

    // ─── Assertion 3: absent SeekTarget → −1 sentinel + seek_kind 0 ────────────
    #[test]
    fn harness_s16_epsilon_a3_absent_seek_target_sentinel() {
        // Type A — Idle agent with NO SeekTarget: seek_kind 0, target (-1,-1).
        // The renderer's goal-line guard is `target_x >= 0`, so the sentinel
        // MUST be -1 (0,0 would draw a spurious line to the grid corner).
        let mut e = engine();
        let idle = spawn_with(&mut e, 8, 9, AgentState::Idle, None);
        let rows = collect_agent_snapshot(&e.world);
        let row = row_for(&rows, idle);
        assert_eq!(
            row.seek_kind, 0,
            "A3: Idle (no resource trip) → seek_kind 0"
        );
        assert_eq!(row.target_x, -1, "A3: no SeekTarget → target_x sentinel -1");
        assert_eq!(row.target_y, -1, "A3: no SeekTarget → target_y sentinel -1");
        println!("[S16-ε A3] Idle + no SeekTarget → seek_kind 0, target (-1,-1) ✓");
    }

    // ─── Assertion 4: state_tag mapping UNCHANGED on extended row (A22) ────────
    #[test]
    fn harness_s16_epsilon_a4_state_tag_table_unchanged() {
        // Type D — regression guard for the locked P11-α A22 state_tag table.
        // Adding seek_kind/target fields must not perturb state_tag.
        let mut e = engine();
        let idle = spawn_with(&mut e, 1, 1, AgentState::Idle, None);
        let seeking = spawn_with(
            &mut e,
            2,
            2,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            None,
        );
        let consuming_agent = spawn_with(
            &mut e,
            3,
            3,
            AgentState::Consuming {
                target: TargetKind::Agent(777),
            },
            None,
        );
        let consuming_food = spawn_with(
            &mut e,
            4,
            4,
            AgentState::Consuming {
                target: TargetKind::Food,
            },
            None,
        );

        let rows = collect_agent_snapshot(&e.world);
        assert_eq!(row_for(&rows, idle).state_tag, 0, "A4: Idle → state_tag 0");
        assert_eq!(
            row_for(&rows, seeking).state_tag,
            1,
            "A4: Seeking{{..}} → state_tag 1"
        );
        assert_eq!(
            row_for(&rows, consuming_agent).state_tag,
            2,
            "A4: Consuming{{Agent(_)}} → state_tag 2"
        );
        assert_eq!(
            row_for(&rows, consuming_food).state_tag,
            3,
            "A4: Consuming{{Food}} → state_tag 3"
        );
        println!("[S16-ε A4] state_tag table intact: 0/1/2/3 ✓");
    }

    // ─── Assertion 5: non-resource Seeking targets report seek_kind 0 ──────────
    #[test]
    fn harness_s16_epsilon_a5_non_resource_seek_kind_zero() {
        // Type A — seek_kind encodes only resource trips. ConstructionSite and
        // Agent are not resource trips → 0. (state_tag stays 1 — orthogonal.)
        let mut e = engine();
        let site = spawn_with(
            &mut e,
            10,
            10,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite,
            },
            None,
        );
        let partner = spawn_with(
            &mut e,
            11,
            11,
            AgentState::Seeking {
                target: TargetKind::Agent(424242),
            },
            None,
        );
        let rows = collect_agent_snapshot(&e.world);
        let site_row = row_for(&rows, site);
        let partner_row = row_for(&rows, partner);
        assert_eq!(
            site_row.seek_kind, 0,
            "A5: Seeking{{ConstructionSite}} → seek_kind 0"
        );
        assert_eq!(
            partner_row.seek_kind, 0,
            "A5: Seeking{{Agent(_)}} → seek_kind 0"
        );
        // Precondition: both are still Seeking (state_tag 1) — not vacuous.
        assert_eq!(
            site_row.state_tag, 1,
            "A5 precondition: ConstructionSite seek is state_tag 1"
        );
        assert_eq!(
            partner_row.state_tag, 1,
            "A5 precondition: Agent seek is state_tag 1"
        );
        println!("[S16-ε A5] ConstructionSite/Agent seeks → seek_kind 0 (state_tag still 1) ✓");
    }

    // ─── Assertion 6: pre-existing row fields byte-for-byte intact (P4-γ A5) ───
    #[test]
    fn harness_s16_epsilon_a6_existing_fields_intact() {
        // Type D — the struct extension is additive; the five original fields
        // (entity_bits, x, y, state_tag, agent_id) must be untouched.
        let mut e = engine();
        let ent = spawn_with(
            &mut e,
            13,
            21,
            AgentState::Consuming {
                target: TargetKind::Water,
            },
            Some(SeekTarget::new((1, 2))),
        );
        let expected_agent_id = e.world.get::<&Agent>(ent).expect("Agent").id;

        let rows = collect_agent_snapshot(&e.world);
        let row = row_for(&rows, ent);
        assert_eq!(
            row.entity_bits,
            ent.to_bits().get(),
            "A6: entity_bits must equal Entity::to_bits().get()"
        );
        assert_eq!(row.x, 13, "A6: x must equal inserted Position.x");
        assert_eq!(row.y, 21, "A6: y must equal inserted Position.y");
        // Consuming{Water} (non-Agent) → state_tag 3 per the A22 table.
        assert_eq!(
            row.state_tag, 3,
            "A6: state_tag must follow the A22 table (Consuming{{Water}}=3)"
        );
        assert_eq!(
            row.agent_id, expected_agent_id,
            "A6: agent_id must equal Agent.id"
        );
        println!("[S16-ε A6] entity_bits/x/y/state_tag/agent_id intact ✓");
    }

    // ─── Assertion 7: live snapshot well-formedness (range + parity) ───────────
    #[test]
    fn harness_s16_epsilon_a7_live_snapshot_well_formed() {
        // Type A — range/parity invariant on real bootstrap output after a short
        // run. Catches mis-mappings that only manifest under live Seeking
        // transitions. Grid is 64×64 so valid tile coords are [0,63].
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        register_default_runtime_systems(&mut e);
        bootstrap_spawn_agents(&mut e);
        for _ in 0..2000 {
            e.tick();
        }

        let rows = collect_agent_snapshot(&e.world);
        assert_eq!(
            rows.len(),
            64,
            "A7: row count must equal the bootstrap agent count (64)"
        );
        for (i, row) in rows.iter().enumerate() {
            // (a) seek_kind in the locked domain {0,1,2,3}.
            assert!(
                row.seek_kind <= 3,
                "A7(a): row {i} seek_kind {} must be in {{0,1,2,3}}",
                row.seek_kind
            );
            // (b) if seek_kind != 0 the target must be a valid in-grid tile.
            if row.seek_kind != 0 {
                assert!(
                    row.target_x >= 0 && row.target_y >= 0,
                    "A7(b): row {i} resource trip must carry a non-sentinel target ({},{})",
                    row.target_x,
                    row.target_y
                );
                assert!(
                    row.target_x <= 63 && row.target_y <= 63,
                    "A7(b): row {i} target ({},{}) must lie within the 64×64 grid",
                    row.target_x,
                    row.target_y
                );
            }
            // (c) sentinel parity: seek_kind 0 with target_x == -1 ⇒ target_y == -1.
            if row.seek_kind == 0 && row.target_x == -1 {
                assert_eq!(
                    row.target_y, -1,
                    "A7(c): row {i} half-set sentinel — target_x -1 requires target_y -1"
                );
            }
        }
        let seeking = rows.iter().filter(|r| r.seek_kind != 0).count();
        println!("[S16-ε A7] 64 rows well-formed; {seeking} resource-trip rows ✓");
    }

    // ─── Assertion 8: gathering loop preserved — Seeking→Consuming ─────────────
    #[test]
    fn harness_s16_epsilon_a8_gathering_loop_preserved() {
        // Type D — the additive FFI change must not disturb the α/β/δ gathering
        // loop. A forced-hungry agent co-located with a food source must reach
        // Consuming within 50 ticks and not stay stuck Idle.
        let mut e = engine();
        register_default_runtime_systems(&mut e);
        e.resources.set_food_tile(20, 20, RESOURCE_SOURCE_INFINITE);

        let agent = e.spawn_agent(20, 20);
        e.world
            .insert(
                agent,
                (
                    AgentState::Idle,
                    Hunger::new(51.0, 0.0),
                    MovementRng::new(42),
                ),
            )
            .expect("seed forced-hungry agent");

        let mut reached_consuming = false;
        let mut left_idle = false;
        for _ in 0..50 {
            e.tick();
            let st = *e
                .world
                .get::<&AgentState>(agent)
                .expect("AgentState present");
            if matches!(st, AgentState::Consuming { .. }) {
                reached_consuming = true;
            }
            if !matches!(st, AgentState::Idle) {
                left_idle = true;
            }
        }
        assert!(
            left_idle,
            "A8: forced-hungry agent must not remain stuck Idle"
        );
        assert!(
            reached_consuming,
            "A8: agent must reach Consuming{{..}} within the 50-tick window"
        );
        println!("[S16-ε A8] forced-hungry agent Seeking→Consuming within 50 ticks ✓");
    }
}

// From `harness_show_agent_needs_bars.rs`.
mod need_surface {
    //! V7 viz-B `show-agent-needs-bars` — per-agent need surface (FFI) + the
    //! head need-bar overlay (`need_bar_renderer.gd`).
    //!
    //! viz-A made resources visibly deplete; viz-B closes the causal loop by
    //! showing WHO is starving/dehydrating/exhausted. The agent snapshot gains
    //! `hunger` / `thirst` / `sleep` (the live need values, `[0, 100]`), and a
    //! standalone overlay draws a yellow→red bar above each at-risk agent
    //! (`danger = max(need)/100 ≥ 0.5`). Agents render via a single
    //! MultiMeshInstance2D, so the overlay re-derives positions from the snapshot
    //! each frame (the `seek_viz_renderer.gd` pattern).
    //!
    //! Assertion map (locked plan, plan_attempt 3):
    //!   A1 need arrays mirror live components; length + order alignment ....... Type A
    //!   A2 source fields not swapped (sleep from `.fatigue`); non-vacuity floors Type A/C
    //!   A3 danger = max(need)/100 ≥ 0 ; at-risk path reachable ................ Type A/C
    //!   A4 existing snapshot fields + locked `agent_rows_split` 4-tuple ....... Type D
    //!   A5 renderer source + DANGER_THRESHOLD==0.5 + bridge key/field mapping . Type D
    //!   A6 main.tscn additive wiring (+1 load_steps, +1 ext_resource) ......... Type D
    //!
    //! Assertion 7 (the cross-phase `load_steps` retarget) is NOT in this file's
    //! 6-assertion budget — it is verified by the two EXPLICITLY-authorised locked
    //! tests (`harness_p14_zeta_zoom_adaptive::a17`,
    //! `harness_p14_epsilon_activity_trails::a12`, both now `load_steps == 12`) and
    //! `harness_p14_delta` A11 (`load_steps >= 8`, unaffected).
    //!
    //! ─── Type C floor measurement (REQUIRED by plan; re-measure on change) ───
    //! Measured at seed 42, 20 agents, 2000 ticks, on 2026-06-05 against the
    //! working tree atop commit 6dfa7f7c. This file's `make_stage1_engine` uses
    //! EXPLICIT moderate sub-saturation growth rates (NOT the production
    //! BOOTSTRAP_* constants): production rates (Thirst 0.08, Hunger 0.05) drive
    //! BOTH hunger and thirst to the `SATURATION == 100` clamp well before tick
    //! 2000, which would (a) make hunger == thirst, collapsing the A2(c)
    //! mutually-distinct floor to 0, and (b) trip the `>= SATURATION` starvation
    //! damage path and despawn agents mid-run. The fixture instead places the
    //! three needs in non-overlapping sub-100 bands by tick 2000
    //! (sleep≈[16,18] < hunger≈[44,51] < thirst≈[80,84]) so every need is
    //! distinct AND the worst need is in the at-risk band (≥ 50 ⟺ danger ≥ 0.5).
    //! If the need-growth model OR these fixture rates change, RE-MEASURE the
    //! eprintln'd counts and reset the floors below to max(1, floor(k×observed)).
    //!   A2(b) `fatigue != growth_rate` subset .. observed 20 → floor max(1, ⌊0.5·20⌋) = 10
    //!   A2(c) mutually-distinct-need agents .... observed 20 → floor max(1, ⌊0.5·20⌋) = 10
    //!   A3(c) peak at-risk (max ≥ 50) count .... observed 20 → floor max(1, ⌊0.3·20⌋) =  6
    //! An observed count of 0 is a REPORTABLE FINDING (vacuous guard), not a pass.

    use std::collections::HashMap;

    use sim_bridge::ffi::{agent_rows_split, collect_agent_snapshot, AgentSnapshotRow};
    use sim_core::components::{Agent, Hunger, Position, SeekTarget, Sleep, Thirst};
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;
    use sim_systems::register_default_runtime_systems;
    use sim_systems::runtime::agent::MovementRng;

    const W: u32 = 64;
    const H: u32 = 64;
    /// Locked tick horizon for the FFI assertions (plan: ticks 2000).
    const TICKS: u32 = 2000;
    /// Need value at full saturation — mirrors `Hunger::SATURATION` and the
    /// renderer's `SATURATION` constant; the danger ratio is `value / 100`.
    const SATURATION: f32 = 100.0;
    /// Number of agents the fixture spawns (plan: agent_count 20).
    const AGENTS: u32 = 20;

    // ── Committed Type C floors (see header measurement block) ──────────────────
    const FLOOR_A2B_FATIGUE_NE_GROWTH: usize = 10;
    const FLOOR_A2C_DISTINCT_NEEDS: usize = 10;
    const FLOOR_A3C_AT_RISK_PEAK: usize = 6;

    /// Advance the engine `n` ticks. `SimEngine` exposes only single-tick
    /// `tick()`, so this is the canonical sim-test pattern.
    fn run_ticks(engine: &mut SimEngine, n: u32) {
        for _ in 0..n {
            engine.tick();
        }
    }

    /// Build a stage-1 engine with `AGENTS` agents on a deterministic lattice,
    /// each carrying (Agent, Position, MovementRng, Hunger, Thirst, Sleep,
    /// AgentState). Need growth uses EXPLICIT moderate sub-saturation rates (see
    /// the header block) so the three needs stay distinct AND climb into the
    /// at-risk band by tick 2000 — production rates would saturate hunger==thirst
    /// and risk starvation despawn. No startup buildings / resource sources, so
    /// needs climb monotonically (no consumption) and there is no
    /// settlement-migration freeze.
    fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
        use sim_core::components::AgentState;
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_default_runtime_systems(&mut engine);
        // Rates chosen so by tick 2000 (× rate): sleep ≈ +16, hunger ≈ +44,
        // thirst ≈ +80 — non-overlapping bands, all < 100.
        const HUNGER_RATE: f32 = 0.022;
        const THIRST_RATE: f64 = 0.040;
        const SLEEP_RATE: f64 = 0.008;
        for i in 0..agent_count {
            let x = 16 + (i % 4);
            let y = 16 + (i / 4);
            let entity = engine.spawn_agent(x, y);
            // Small per-agent staggered starts (deterministic, not RNG) so agents
            // diverge cross-row; the within-agent distinctness comes from the
            // distinct bands above.
            let h0 = (i % 8) as f32;
            let t0 = (i % 5) as f64;
            let sl0 = (i % 3) as f64;
            engine
                .world
                .insert(
                    entity,
                    (
                        MovementRng::new(seed.wrapping_add(i as u64)),
                        Hunger::new(h0, HUNGER_RATE),
                        Thirst::new(t0, THIRST_RATE),
                        Sleep::new(sl0, SLEEP_RATE),
                        AgentState::Idle,
                    ),
                )
                .expect("freshly spawned agent must still exist");
        }
        engine
    }

    fn engine() -> SimEngine {
        make_stage1_engine(42, AGENTS)
    }

    fn danger(r: &AgentSnapshotRow) -> f32 {
        r.hunger.max(r.thirst).max(r.sleep) / SATURATION
    }

    /// Live per-entity mirror of the need/position values, keyed by
    /// `Entity::to_bits().get()` (the value the row's `entity_bits` and the
    /// split's `ids` array carry). Need values use the SAME `unwrap_or(0.0)` +
    /// cast rules as `collect_agent_snapshot` so the comparison is bit-exact.
    struct LiveAgent {
        x: u32,
        y: u32,
        hunger: f32,
        thirst: f32,
        sleep: f32,
        agent_id: u64,
        has_seek: bool,
    }

    fn live_map(e: &SimEngine) -> HashMap<u64, LiveAgent> {
        let mut map = HashMap::new();
        for (ent, (agent, pos, mh, mt, ms, mseek)) in e
            .world
            .query::<(
                &Agent,
                &Position,
                Option<&Hunger>,
                Option<&Thirst>,
                Option<&Sleep>,
                Option<&SeekTarget>,
            )>()
            .iter()
        {
            map.insert(
                ent.to_bits().get(),
                LiveAgent {
                    x: pos.x,
                    y: pos.y,
                    hunger: mh.map(|h| h.value).unwrap_or(0.0),
                    thirst: mt.map(|t| t.value as f32).unwrap_or(0.0),
                    sleep: ms.map(|s| s.fatigue as f32).unwrap_or(0.0),
                    agent_id: agent.id,
                    has_seek: mseek.is_some(),
                },
            );
        }
        map
    }

    // ─── A1: need arrays mirror live components — value, length, order ──────────
    #[test]
    fn harness_needs_a1_arrays_mirror_components_length_and_order() {
        // Type A — the snapshot is a pure read of live components and the FFI need
        // arrays ARE the row f32 fields verbatim, so this is an exact marshalling
        // invariant (not a tuning value). The renderer paints the bar at
        // (xs[i], ys[i]); a row-order scramble would draw the wrong agent's needs,
        // so order alignment (c) is load-bearing.
        let mut e = engine();
        run_ticks(&mut e, TICKS);
        let rows = collect_agent_snapshot(&e.world);
        let live = live_map(&e);

        // (b) Length — rows.len() == live (Agent,Position) count.
        assert_eq!(
            rows.len(),
            live.len(),
            "A1(b): rows.len() must equal live (Agent,Position) count"
        );
        assert!(
            !rows.is_empty(),
            "A1(b): seed-42 fixture must produce agents"
        );

        let mut value_devs = 0usize;
        let mut pos_misaligns = 0usize;
        for (i, r) in rows.iter().enumerate() {
            let la = live
                .get(&r.entity_bits)
                .unwrap_or_else(|| panic!("A1: row {i} entity_bits has no live entity"));
            // (a) Value mirror — exact (same component, same casts).
            if (r.hunger - la.hunger).abs() > f32::EPSILON {
                value_devs += 1;
            }
            if (r.thirst - la.thirst).abs() > f32::EPSILON {
                value_devs += 1;
            }
            if (r.sleep - la.sleep).abs() > f32::EPSILON {
                value_devs += 1;
            }
            // (c) Order alignment — the need value at index i belongs to the same
            // agent whose position sits at index i.
            if r.x != la.x || r.y != la.y {
                pos_misaligns += 1;
            }
        }
        assert_eq!(value_devs, 0, "A1(a): need value deviations must be 0");
        assert_eq!(
            pos_misaligns, 0,
            "A1(c): row-order/position misalignments must be 0"
        );
        println!(
            "[viz-B A1] {} rows mirror live needs; 0 value devs, 0 order misaligns ✓",
            rows.len()
        );
    }

    // ─── A2: source fields not swapped; non-vacuity floors ──────────────────────
    #[test]
    fn harness_needs_a2_source_fields_not_swapped_with_floors() {
        let mut e = engine();
        run_ticks(&mut e, TICKS);
        let rows = collect_agent_snapshot(&e.world);

        // Re-read the raw live components to drive the source-correctness check
        // (sleep MUST come from `.fatigue`, never `.growth_rate`; hunger/thirst
        // not transposed).
        let mut comp: HashMap<u64, (f32, f32, f32, f32)> = HashMap::new(); // h, t, fatigue, growth_rate
        for (ent, (_a, h, t, s)) in e.world.query::<(&Agent, &Hunger, &Thirst, &Sleep)>().iter() {
            comp.insert(
                ent.to_bits().get(),
                (
                    h.value,
                    t.value as f32,
                    s.fatigue as f32,
                    s.growth_rate as f32,
                ),
            );
        }

        let mut source_violations = 0usize;
        let mut fatigue_ne_growth = 0usize;
        let mut distinct_needs = 0usize;
        for r in &rows {
            let (h, t, fatigue, growth) = comp[&r.entity_bits];
            // (a) Source correctness — no transposition.
            if (r.hunger - h).abs() > f32::EPSILON {
                source_violations += 1; // hunger not from Hunger.value
            }
            if (r.thirst - t).abs() > f32::EPSILON {
                source_violations += 1; // thirst not from Thirst.value
            }
            if (r.sleep - fatigue).abs() > f32::EPSILON {
                source_violations += 1; // sleep not from Sleep.fatigue
            }
            // The `!= growth_rate` clause only bites where they differ.
            if (fatigue - growth).abs() > f32::EPSILON {
                fatigue_ne_growth += 1;
                if (r.sleep - growth).abs() <= f32::EPSILON {
                    source_violations += 1; // sleep wrongly sourced from growth_rate
                }
            }
            // (c) Cross-need divergence — all three mutually distinct.
            if (r.hunger - r.thirst).abs() > f32::EPSILON
                && (r.thirst - r.sleep).abs() > f32::EPSILON
                && (r.hunger - r.sleep).abs() > f32::EPSILON
            {
                distinct_needs += 1;
            }
        }
        eprintln!(
        "[viz-B A2] observed: fatigue!=growth={fatigue_ne_growth}, distinct-needs={distinct_needs} \
         (floors {FLOOR_A2B_FATIGUE_NE_GROWTH} / {FLOOR_A2C_DISTINCT_NEEDS})"
    );
        // (a) Type A — exact mapping invariant.
        assert_eq!(
            source_violations, 0,
            "A2(a): field-source violations must be 0"
        );
        // (b)/(c) Type C — non-vacuity floors (a 0 observation is a finding, not a pass).
        assert!(
        fatigue_ne_growth >= FLOOR_A2B_FATIGUE_NE_GROWTH,
        "A2(b): fatigue!=growth_rate subset {fatigue_ne_growth} must be >= floor {FLOOR_A2B_FATIGUE_NE_GROWTH} \
         (else the `!= growth_rate` swap guard is vacuous — re-measure)"
    );
        assert!(
        distinct_needs >= FLOOR_A2C_DISTINCT_NEEDS,
        "A2(c): mutually-distinct-need agents {distinct_needs} must be >= floor {FLOOR_A2C_DISTINCT_NEEDS} \
         (else a hunger<->thirst transposition is unobservable — re-measure)"
    );
        println!("[viz-B A2] sleep from .fatigue, no transposition; floors met ✓");
    }

    // ─── A3: danger ratio non-negative; at-risk path reachable ──────────────────
    #[test]
    fn harness_needs_a3_danger_ratio_and_at_risk_reachable() {
        // (a) Non-negativity — Type A invariant. Run once and check every row.
        let mut e = engine();
        let mut max_danger = 0.0f32;
        let mut peak_at_risk = 0usize;
        // (c) Reachability — sample at 100-tick checkpoints, record the PEAK count
        // of at-risk rows (max(h,t,s) >= 50.0 ⟺ danger >= 0.5).
        let checkpoints = TICKS / 100;
        for _ in 0..checkpoints {
            run_ticks(&mut e, 100);
            let rows = collect_agent_snapshot(&e.world);
            let mut at_risk = 0usize;
            for r in &rows {
                let d = danger(r);
                assert!(
                    d >= 0.0,
                    "A3(a): danger must be >= 0 (needs are never negative); got {d}"
                );
                if d > max_danger {
                    max_danger = d;
                }
                if r.hunger.max(r.thirst).max(r.sleep) >= 50.0 {
                    at_risk += 1;
                }
            }
            if at_risk > peak_at_risk {
                peak_at_risk = at_risk;
            }
        }
        // (b) Upper bound — reported softly (no hard <= 1.0; the [0,100] clamp lives
        // in another, churning system).
        eprintln!(
            "[viz-B A3] observed max danger = {max_danger:.4} (soft, no upper-bound assert); \
         peak at-risk = {peak_at_risk} (floor {FLOOR_A3C_AT_RISK_PEAK})"
        );
        assert!(
            peak_at_risk >= FLOOR_A3C_AT_RISK_PEAK,
            "A3(c): peak at-risk count {peak_at_risk} must be >= floor {FLOOR_A3C_AT_RISK_PEAK} \
         (else the renderer's bar-draw branch is unreachable by real data — re-measure)"
        );
        println!("[viz-B A3] danger >= 0 for all rows; at-risk path reachable ✓");
    }

    // ─── A4: existing snapshot fields + locked 4-tuple split preserved ──────────
    #[test]
    fn harness_needs_a4_existing_fields_and_split_preserved() {
        // Type D regression guard — viz-B is strictly additive. The
        // `agent_rows_split` 4-tuple is locked by `harness_p4_gamma_rendering`,
        // and the Section 16-ε head-dot/goal-line fields must be untouched.
        let mut e = engine();
        run_ticks(&mut e, TICKS);
        let rows = collect_agent_snapshot(&e.world);
        let live = live_map(&e);
        assert!(!rows.is_empty(), "A4: fixture must produce rows");

        for r in &rows {
            let la = live
                .get(&r.entity_bits)
                .expect("A4: every row entity_bits maps to a live entity");
            assert_eq!(r.x, la.x, "A4: x preserved");
            assert_eq!(r.y, la.y, "A4: y preserved");
            assert!(
                r.state_tag <= 3,
                "A4: state_tag must be in {{0,1,2,3}}; got {}",
                r.state_tag
            );
            assert!(
                r.seek_kind <= 3,
                "A4: seek_kind must be in {{0,1,2,3}}; got {}",
                r.seek_kind
            );
            assert_eq!(r.agent_id, la.agent_id, "A4: agent_id mirrors Agent.id");
            if !la.has_seek {
                assert_eq!(
                    r.target_x, -1,
                    "A4: absent SeekTarget → target_x sentinel -1"
                );
                assert_eq!(
                    r.target_y, -1,
                    "A4: absent SeekTarget → target_y sentinel -1"
                );
            }
        }

        // Locked 4-tuple split: arity 4 (destructuring is a compile-time arity
        // check), all lengths == rows.len(), and the exact value mappings.
        let (ids, xs, ys, states) = agent_rows_split(&rows);
        assert_eq!(ids.len(), rows.len(), "A4: ids length == row count");
        assert_eq!(xs.len(), rows.len(), "A4: xs length == row count");
        assert_eq!(ys.len(), rows.len(), "A4: ys length == row count");
        assert_eq!(states.len(), rows.len(), "A4: states length == row count");
        for (i, r) in rows.iter().enumerate() {
            assert_eq!(
                ids[i], r.entity_bits as i64,
                "A4: ids[i] == entity_bits as i64"
            );
            assert_eq!(xs[i], r.x as i32, "A4: xs[i] == x as i32");
            assert_eq!(ys[i], r.y as i32, "A4: ys[i] == y as i32");
            assert_eq!(states[i], r.state_tag, "A4: states[i] == state_tag");
        }
        println!("[viz-B A4] existing fields + locked agent_rows_split 4-tuple preserved ✓");
    }
}

// From `harness_s16_gamma_visual.rs`.
mod resource_markers {
    //! V7 Section 16-γ — observability: visible resource markers + speed control.
    //!
    //! feature: s16-gamma-visual
    //! seed: 42
    //! agent_count: 20
    //! lane: --quick
    //!
    //! γ is PURE observability scaffolding over the α0+α+β gathering loop
    //! (53075aff / 335ca145 / fa5e350b). It changes NO simulation behavior:
    //!   T1 — world_renderer source markers: tinted Sprite2D → bright Polygon2D.
    //!   T2 — world_node.rs: `sim_speed` + `clamp_sim_speed` + accumulator scale.
    //!   T3 — camera_controller.gd: KEY_1..KEY_4 → set_sim_speed.
    //!
    //! Assertion map (11; plan s16-gamma-visual plan_attempt 1):
    //!   a1  — clamp_sim_speed pure correctness (hand-written expected) (Type A)
    //!   a2  — world_node sim_speed field + #[func] set_sim_speed + scale (Type A)
    //!   a3  — accumulator integrity: FIXED_DT + MAX_ITERS_PER_FRAME kept (Type D)
    //!   a4  — marker is Polygon2D + SOURCE_KIND_COLORS + z_index, no Sprite2D (Type A)
    //!   a5  — Z_RESOURCE_SOURCE > Z_RESOURCE                            (Type A)
    //!   a6  — decorative invariant: RESOURCE_SEED + RESOURCE_COUNT==20  (Type D)
    //!   a7  — backend-truth: get_resource_snapshot kept in marker fn    (Type D)
    //!   a8  — camera KEY_1..KEY_4 + set_sim_speed in _unhandled_input   (Type A)
    //!   a9  — camera invariants: extends Camera2D / KEY_P / handler; no KEY_SPACE (Type D)
    //!   a10 — α0 substrate intact: each kind >= 4 source tiles          (Type C)
    //!   a11 — gathering FSM intact: hungry agent Seeking → Consuming    (Type D)
    //!
    //! Run:
    //!   cargo test -p sim-test --test harness_s16_gamma_visual -- --nocapture

    use sim_bridge::ffi::bootstrap_spawn_agents;
    use sim_bridge::ffi::clamp_sim_speed;
    use sim_core::components::{AgentState, Hunger, TargetKind};
    use sim_core::material::MaterialRegistry;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::baseline::decision::AgentDecisionSystem;

    // ── constants mirrored from the production bootstrap (read-only) ────────────
    const DEFAULT_W: u32 = 64;
    const DEFAULT_H: u32 = 64;

    // ── helpers (h-phase-a / s16-α0 precedent) ──────────────────────────────────

    fn bootstrapped_engine() -> SimEngine {
        let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
        bootstrap_spawn_agents(&mut engine);
        engine
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Assertion 1 — clamp_sim_speed pure correctness (Type A, non-circular)
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_s16_gamma_a1_clamp_sim_speed_pure_correctness() {
        // Type A — mathematical invariant. Expected values are HAND-WRITTEN
        // literals, NOT a re-derivation of `speed.clamp(0.0, 4.0)`.
        assert_eq!(
            clamp_sim_speed(0.25),
            0.25,
            "a1.1: 0.25 in-range passthrough"
        );
        assert_eq!(clamp_sim_speed(0.5), 0.5, "a1.2: 0.5 in-range passthrough");
        assert_eq!(clamp_sim_speed(1.0), 1.0, "a1.3: 1.0 in-range passthrough");
        assert_eq!(clamp_sim_speed(2.0), 2.0, "a1.4: 2.0 in-range passthrough");
        assert_eq!(
            clamp_sim_speed(10.0),
            4.0,
            "a1.5: 10.0 clamps to upper bound 4.0"
        );
        assert_eq!(
            clamp_sim_speed(-1.0),
            0.0,
            "a1.6: -1.0 clamps to lower bound 0.0"
        );
        // Inclusive boundary inputs pass through unchanged.
        assert_eq!(clamp_sim_speed(0.0), 0.0, "a1.7: 0.0 inclusive lower bound");
        assert_eq!(clamp_sim_speed(4.0), 4.0, "a1.8: 4.0 inclusive upper bound");
        println!("[S16-γ a1] clamp_sim_speed correct for all 8 hand-written cases ✓");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Assertion 10 — α0 resource-source substrate intact (Type C)
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_s16_gamma_a10_resource_source_substrate_seeding_intact() {
        // Type C — α0 substrate regression guard. γ changes no backend; the fixed
        // 4-tile lattices per kind must remain.
        let engine = bootstrapped_engine();
        let food = engine.resources.food_tiles.len();
        let water = engine.resources.water_tiles.len();
        let sleep = engine.resources.sleep_tiles.len();
        assert!(
            food >= 4,
            "a10.1: food_tiles must have >= 4 source tiles; got {food}"
        );
        assert!(
            water >= 4,
            "a10.2: water_tiles must have >= 4 source tiles; got {water}"
        );
        assert!(
            sleep >= 4,
            "a10.3: sleep_tiles must have >= 4 source tiles; got {sleep}"
        );
        println!("[S16-γ a10] source tiles food={food} water={water} sleep={sleep} (each >= 4) ✓");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Assertion 11 — gathering-loop FSM not regressed (Type D)
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_s16_gamma_a11_gathering_loop_fsm_not_regressed() {
        // Type D — α/β gathering-loop regression guard. A hungry agent co-located
        // on a real bootstrap food tile must still progress Seeking → Consuming.
        let mut engine = bootstrapped_engine();
        let mut food_coords: Vec<(u32, u32)> =
            engine.resources.food_tiles.keys().copied().collect();
        food_coords.sort();
        let (fx, fy) = *food_coords
            .first()
            .expect("a11.0: bootstrap must produce >= 1 food tile");

        let entity = engine.spawn_agent(fx, fy);
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Seeking {
                        target: TargetKind::Food,
                    },
                    Hunger::new(80.0, 0.0),
                ),
            )
            .expect("a11.1: seed hungry agent on the food tile");

        let mut sys = AgentDecisionSystem::new();
        // Tick 1: a co-located Seeking{Food} agent transitions to Consuming{Food}.
        sys.tick(&mut engine.world, &mut engine.resources);
        let state = *engine
            .world
            .get::<&AgentState>(entity)
            .expect("a11.2: state present");
        assert_eq!(
            state,
            AgentState::Consuming {
                target: TargetKind::Food
            },
            "a11.3: a hungry agent on a food source must reach Consuming{{Food}}; got {state:?}"
        );
        println!("[S16-γ a11] hungry agent on food tile ({fx},{fy}) → Consuming{{Food}} ✓");
    }
}
