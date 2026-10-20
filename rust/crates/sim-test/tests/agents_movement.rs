//! Merged in V8-0 (8.4) from `harness_p4_beta_movement.rs`,
//! `harness_s16_beta_movement.rs`; each source is one module below, unchanged.

// From `harness_p4_beta_movement.rs`.
mod wander {
    //! P4-β — Agent Movement System + MovementRng plumbing (V7 Week 7-8).
    //!
    //! Builds on P4-α canonical components (`sim_core::components::{Position,
    //! Agent}` + `SimEngine::spawn_agent`). Adds priority-120 `AgentMovementSystem`
    //! and per-agent `MovementRng` so agents take deterministic Brownian steps
    //! on every tick.
    //!
    //! Per planning §2.2 (`.harness/plans/phase4.md` lines 120-153):
    //!   - Movement model: Brownian (deterministic seeded RNG)
    //!   - Priority: ~120, after AgentInfluenceSampleSystem (priority 110)
    //!   - tick_interval: 1
    //!   - AgentDecision causal variant: DEFERRED (planning permits default
    //!     deferral; AgentId infrastructure not yet in sim-core)
    //!
    //! Threshold types follow `.harness/policy/evaluation_criteria.md`:
    //!   - Type A — structural invariants (compile-time / shape / equality)
    //!   - Type D — regression guards (pre-existing runtime paths stay green)
    //!
    //! Assertions:
    //!   1.  Type A — `AgentMovementSystem` re-exports resolve at
    //!       `sim_systems::runtime::agent::AgentMovementSystem`.
    //!   2.  Type A — system metadata (name / priority=120 / tick_interval=1).
    //!   3.  Type A — `MovementRng` re-exports resolve at
    //!       `sim_systems::runtime::agent::MovementRng`.
    //!   4.  Type A — `MovementRng::new(seed)` produces a working PRNG that
    //!       escapes the all-zero state (splitmix64 invariant).
    //!   5.  Type A — single agent at (10,10) with seed=42 moves on the first
    //!       tick by at most 1 cell on each axis (Brownian neighborhood).
    //!   6.  Type A — full determinism: replaying with identical seeds produces
    //!       byte-identical trajectories across 16 ticks.
    //!   7.  Type A — distinct seeds produce distinct trajectories (within 32
    //!       ticks the two walks diverge at least once).
    //!   8.  Type D — multi-agent independence: agent with seed=7 produces the
    //!       same 16-tick trajectory solo and alongside a seed-13 co-tenant.
    //!   9.  Type A — boundary clamp: an agent spawned at (0,0) and another at
    //!       (W-1,H-1) survive 200 ticks without ever leaving the grid (u32
    //!       underflow / out-of-bounds protection).
    //!   10. Type D — `register_agent_systems(&mut engine)` registers the
    //!       movement system and 8 subsequent `engine.tick()` calls advance the
    //!       single spawned agent away from its initial position.
    //!   11. Type D — `register_phase2_systems` + `register_agent_systems`
    //!       coexist: after one full `engine.tick()` an agent that started at
    //!       a freshly-stamped warmth source reads `InfluenceSample::warmth > 0`
    //!       (IUS@100 propagates → AIS@110 samples → AMS@120 moves).
    //!
    //! Run: `cargo test -p sim-test --test harness_p4_beta_movement -- --nocapture`

    use sim_core::components::{Agent, Position};
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
    use sim_systems::runtime::influence::agent_sample::InfluenceSample;
    use sim_systems::{register_agent_systems, register_phase2_systems};

    const W: u32 = 32;
    const H: u32 = 32;

    fn fresh_engine() -> SimEngine {
        SimEngine::new(W, H, MaterialRegistry::new())
    }

    // ─── Assertion 1: AgentMovementSystem re-export ────────────────────────────

    #[test]
    fn harness_p4_beta_movement_system_export_resolves() {
        type _S = sim_systems::runtime::agent::AgentMovementSystem;
        let s = AgentMovementSystem::new();
        assert_eq!(s.name(), "AgentMovementSystem");
    }

    // ─── Assertion 2: system metadata ──────────────────────────────────────────

    #[test]
    fn harness_p4_beta_system_metadata() {
        let s = AgentMovementSystem::new();
        assert_eq!(s.name(), "AgentMovementSystem");
        assert_eq!(s.priority(), 120, "priority must be 120 per planning §2.2");
        assert_eq!(s.tick_interval(), 1, "tick_interval must be 1 (every tick)");
    }

    // ─── Assertion 3: MovementRng re-export ────────────────────────────────────

    #[test]
    fn harness_p4_beta_rng_export_resolves() {
        type _R = sim_systems::runtime::agent::MovementRng;
        let _r = MovementRng::new(0);
    }

    // ─── Assertion 4: splitmix64 escapes zero seed ─────────────────────────────

    #[test]
    fn harness_p4_beta_rng_escapes_zero_seed() {
        let mut r = MovementRng::new(0);
        // splitmix64 with a non-zero additive constant must produce a non-zero
        // first sample even when seeded with zero.
        let first = r.next_u64();
        assert_ne!(first, 0, "splitmix64 must escape the all-zero state");
    }

    // ─── Assertion 5: one tick → Brownian neighborhood ─────────────────────────

    #[test]
    fn harness_p4_beta_one_tick_brownian_step() {
        let mut e = fresh_engine();
        let id = e
            .world
            .spawn((Position::new(10, 10), Agent { id: 0 }, MovementRng::new(42)));
        let mut sys = AgentMovementSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let p = *e.world.get::<&Position>(id).unwrap();
        assert!(
            p.x.abs_diff(10) <= 1,
            "x must move by at most 1 cell after one tick, got {} (was 10)",
            p.x
        );
        assert!(
            p.y.abs_diff(10) <= 1,
            "y must move by at most 1 cell after one tick, got {} (was 10)",
            p.y
        );
    }

    // ─── Assertion 6: full determinism across 16 ticks ─────────────────────────

    #[test]
    fn harness_p4_beta_determinism_full_trajectory() {
        fn trajectory(seed: u64) -> Vec<(u32, u32)> {
            let mut e = fresh_engine();
            let id = e.world.spawn((
                Position::new(16, 16),
                Agent { id: 0 },
                MovementRng::new(seed),
            ));
            let mut sys = AgentMovementSystem::new();
            let mut out = Vec::with_capacity(16);
            for _ in 0..16 {
                sys.tick(&mut e.world, &mut e.resources);
                let p = *e.world.get::<&Position>(id).unwrap();
                out.push((p.x, p.y));
            }
            out
        }
        let a = trajectory(0xC0FFEE);
        let b = trajectory(0xC0FFEE);
        assert_eq!(a, b, "identical seed must produce identical trajectory");
        assert_eq!(a.len(), 16);
    }

    // ─── Assertion 7: distinct seeds → distinct trajectories ───────────────────

    #[test]
    fn harness_p4_beta_distinct_seeds_diverge() {
        let mut e = fresh_engine();
        let a = e
            .world
            .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(1)));
        let b = e
            .world
            .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(2)));
        let mut sys = AgentMovementSystem::new();
        let mut diverged = false;
        for _ in 0..32 {
            sys.tick(&mut e.world, &mut e.resources);
            let pa = *e.world.get::<&Position>(a).unwrap();
            let pb = *e.world.get::<&Position>(b).unwrap();
            if pa != pb {
                diverged = true;
                break;
            }
        }
        assert!(
            diverged,
            "two agents with distinct seeds must diverge within 32 ticks"
        );
    }

    // ─── Assertion 8: multi-agent stream independence ──────────────────────────
    //
    // Plan-locked specification (`.harness/prompts/p4-beta-movement-decision.md`
    // Section 3.5 row 8): "solo trajectory (seed 7, 16 ticks) == multi-tenant
    // trajectory of same seed alongside seed 13".

    #[test]
    fn harness_p4_beta_multi_agent_independence() {
        // Run agent A (seed 7) solo for 16 ticks → expected trajectory.
        let mut solo_engine = fresh_engine();
        let solo =
            solo_engine
                .world
                .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(7)));
        let mut solo_sys = AgentMovementSystem::new();
        let mut solo_traj = Vec::with_capacity(16);
        for _ in 0..16 {
            solo_sys.tick(&mut solo_engine.world, &mut solo_engine.resources);
            let p = *solo_engine.world.get::<&Position>(solo).unwrap();
            solo_traj.push((p.x, p.y));
        }

        // Run the same seed-7 agent alongside a seed-13 co-tenant → seed-7
        // trajectory must be byte-identical to the solo run.
        let mut multi_engine = fresh_engine();
        let a =
            multi_engine
                .world
                .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(7)));
        let _b =
            multi_engine
                .world
                .spawn((Position::new(5, 5), Agent { id: 0 }, MovementRng::new(13)));
        let mut multi_sys = AgentMovementSystem::new();
        let mut multi_traj = Vec::with_capacity(16);
        for _ in 0..16 {
            multi_sys.tick(&mut multi_engine.world, &mut multi_engine.resources);
            let p = *multi_engine.world.get::<&Position>(a).unwrap();
            multi_traj.push((p.x, p.y));
        }

        // Diagnostic: how many of the 16 sampled positions actually changed
        // between consecutive ticks (Brownian zero-step is allowed, so this
        // is a soft signal, not an assertion target).
        let agents_doing_movement = solo_traj.windows(2).filter(|w| w[0] != w[1]).count();
        eprintln!(
            "[p4-beta][multi_agent_independence] agents_doing_movement={} \
         (transitions where (x,y) changed across the 16-tick solo run)",
            agents_doing_movement
        );

        assert_eq!(
            solo_traj, multi_traj,
            "seed-7 trajectory must not depend on a co-tenant seed-13 agent"
        );
    }

    // ─── Assertion 9: boundary clamp at both corners ───────────────────────────

    #[test]
    fn harness_p4_beta_boundary_clamp() {
        let mut e = fresh_engine();
        let nw = e
            .world
            .spawn((Position::new(0, 0), Agent { id: 0 }, MovementRng::new(7)));
        let se = e.world.spawn((
            Position::new(W - 1, H - 1),
            Agent { id: 1 },
            MovementRng::new(13),
        ));
        let mut sys = AgentMovementSystem::new();
        for _ in 0..200 {
            sys.tick(&mut e.world, &mut e.resources);
            let p_nw = *e.world.get::<&Position>(nw).unwrap();
            let p_se = *e.world.get::<&Position>(se).unwrap();
            assert!(p_nw.x < W, "NW agent left grid on x: {}", p_nw.x);
            assert!(p_nw.y < H, "NW agent left grid on y: {}", p_nw.y);
            assert!(p_se.x < W, "SE agent left grid on x: {}", p_se.x);
            assert!(p_se.y < H, "SE agent left grid on y: {}", p_se.y);
        }
    }

    // ─── Assertion 10: register_agent_systems + tick progress ──────────────────
    //
    // Plan-locked specification (`.harness/prompts/p4-beta-movement-decision.md`
    // Section 3.5 row 10): "register_agent_systems(&mut engine) + 1 spawn +
    // 8 advance ticks → at least one position changed".

    #[test]
    fn harness_p4_beta_register_and_progress() {
        let mut e = fresh_engine();
        let initial = Position::new(8, 8);
        let id = e
            .world
            .spawn((initial, Agent { id: 0 }, MovementRng::new(0x1000)));

        register_agent_systems(&mut e);

        for _ in 0..8 {
            e.tick();
        }

        let after = *e.world.get::<&Position>(id).unwrap();
        let moved = (after.x, after.y) != (initial.x, initial.y);
        let agents_moved = if moved { 1 } else { 0 };
        eprintln!(
            "[p4-beta][register_and_progress] agents_moved={} after 8 engine.tick()s \
         (initial=({},{}) final=({},{}))",
            agents_moved, initial.x, initial.y, after.x, after.y
        );
        assert!(
            moved,
            "spawned agent must have moved from initial=({},{}) after 8 engine.tick()s, \
         got final=({},{})",
            initial.x, initial.y, after.x, after.y
        );
    }

    // ─── Assertion 11: AgentInfluenceSampleSystem coexistence (regression) ─────
    //
    // Plan-locked specification (`.harness/prompts/p4-beta-movement-decision.md`
    // Section 3.5 row 11): "stamp Warmth at agent's pre-move tile, register
    // Phase 2 + agent stacks, 1 tick → `InfluenceSample::warmth > 0` (IUS@110
    // still upstream of AMS@120)". We use the canonical BuildingStampSystem
    // path (queue a BuildingPlacedEvent for BSS@90 to drain), so the warmth
    // arrives via IUS propagation rather than a manual buffer poke. AIS@110
    // then samples the canonical Position before AMS@120 moves the agent.

    #[test]
    fn harness_p4_beta_influence_sampler_still_reads_canonical_position() {
        let mut e = fresh_engine();

        // Queue a building placement at the agent's spawn tile so BSS@90
        // marks Warmth dirty → IUS@100 propagates → AIS@110 samples non-zero
        // warmth into the agent's InfluenceSample → AMS@120 moves the agent.
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (12, 12),
                radius: 2,
            });

        let id = e.world.spawn((
            Position::new(12, 12),
            Agent { id: 0 },
            InfluenceSample::default(),
            MovementRng::new(99),
        ));

        register_phase2_systems(&mut e);
        register_agent_systems(&mut e);

        e.tick();

        let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
        let p_after = *e.world.get::<&Position>(id).unwrap();

        // Diagnostic: how many agents observed non-zero warmth this tick
        // (here N=1 by construction, but expose it for --nocapture audits).
        let agents_doing_movement = e.world.query::<(&Position, &MovementRng)>().iter().count();
        let agents_moved = if (p_after.x, p_after.y) != (12, 12) {
            1
        } else {
            0
        };
        eprintln!(
            "[p4-beta][sampler_coexist] agents_doing_movement={} agents_moved={} \
         warmth={} final=({},{})",
            agents_doing_movement, agents_moved, sample.warmth, p_after.x, p_after.y
        );

        assert!(
            sample.warmth > 0,
            "after one engine.tick() AIS@110 must have sampled warmth > 0 at \
         (12,12) — got {}",
            sample.warmth
        );
        // Movement system also ran (AMS@120) and must have kept the agent on-grid.
        assert!(
            p_after.x < W,
            "agent left grid on x after one engine.tick()"
        );
        assert!(
            p_after.y < H,
            "agent left grid on y after one engine.tick()"
        );
    }
}

// From `harness_s16_beta_movement.rs`.
mod directed {
    //! V7 Section 16-β — directed movement (Seeking → goal) harness.
    //!
    //! β closes the gathering loop laid by α0 (`53075aff`, resource substrate)
    //! and α (`335ca145`, `SeekTarget` nearest-tile targeting): a
    //! `Seeking{Food/Water/Sleep}` agent that carries a `SeekTarget` now takes a
    //! one-tile **directed signum step** toward `SeekTarget.tile` in
    //! `AgentMovementSystem`, instead of being frozen. The full path is:
    //!
    //!   breach → Seeking + SeekTarget (α) → directed walk (β) →
    //!   Seeking→Consuming (decision, on arrival) → satisfied → Idle.
    //!
    //! Non-circular rule: every trajectory assertion compares against a
    //! HAND-COMPUTED signum result derived from the start/target coordinates,
    //! never a value re-read from the production step. Directed math is
    //! `dx = signum(target.x - pos.x)`, `dy = signum(target.y - pos.y)` — pure
    //! coordinate arithmetic, RNG-free, so it is inherently deterministic.
    //!
    //! Run:
    //!   cargo test -p sim-test --test harness_s16_beta_movement -- --nocapture

    use sim_core::components::{AgentState, Hunger, Position, SeekTarget, TargetKind};
    use sim_core::material::MaterialRegistry;
    use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
    use sim_systems::register_default_runtime_systems;
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};

    // ── helpers ─────────────────────────────────────────────────────────────────

    /// Fresh 64×64 engine — large enough for every hand-placed tile below
    /// (clamp bounds are `max_x = max_y = 63`).
    fn engine() -> SimEngine {
        SimEngine::new(64, 64, MaterialRegistry::new())
    }

    /// Current `(x, y)` of `ent`.
    fn pos(e: &SimEngine, ent: hecs::Entity) -> (u32, u32) {
        let p = e.world.get::<&Position>(ent).expect("Position present");
        (p.x, p.y)
    }

    /// State of `ent`.
    fn state(e: &SimEngine, ent: hecs::Entity) -> AgentState {
        *e.world.get::<&AgentState>(ent).expect("AgentState present")
    }

    /// Spawn a movement-eligible probe: Position + MovementRng + AgentState
    /// (+ optional SeekTarget). `MovementRng` is REQUIRED — the movement query
    /// is `(&mut Position, &mut MovementRng, Option<&AgentState>, Option<&SeekTarget>)`,
    /// so an agent without it is never visited by the system.
    fn spawn_probe(
        e: &mut SimEngine,
        at: (u32, u32),
        seed: u64,
        st: AgentState,
        seek: Option<SeekTarget>,
    ) -> hecs::Entity {
        let ent = e.spawn_agent(at.0, at.1);
        e.world
            .insert(ent, (MovementRng::new(seed), st))
            .expect("insert rng + state");
        if let Some(s) = seek {
            e.world.insert_one(ent, s).expect("insert seek target");
        }
        ent
    }

    /// Run `n` movement-system ticks on `e`.
    fn run_movement(e: &mut SimEngine, n: usize) {
        let mut sys = AgentMovementSystem::new();
        for _ in 0..n {
            sys.tick(&mut e.world, &mut e.resources);
        }
    }

    // ── shared end-to-end gathering-loop run (Assertions 4, 9, 16) ──────────────

    /// Result of one full-engine gathering-loop run.
    struct LoopResult {
        final_hunger: f32,
        final_state: AgentState,
        reached_source: bool,
        source_value: Option<u8>,
    }

    /// Full `engine.tick()` run with all default runtime systems registered.
    /// Seeds ONE infinite food source at `(20,20)`, spawns ONE agent at
    /// `(16,16)` (DIAGONAL Chebyshev-4 path, both axes non-zero each step),
    /// `Idle` + `Hunger{value=51.0 (>THRESHOLD), growth=0}` + a `MovementRng`,
    /// then runs 30 ticks sampling the agent position each tick.
    fn run_gathering_loop() -> LoopResult {
        let mut e = engine();
        register_default_runtime_systems(&mut e);
        e.resources.set_food_tile(20, 20, RESOURCE_SOURCE_INFINITE);

        let agent = e.spawn_agent(16, 16);
        e.world
            .insert(
                agent,
                (
                    AgentState::Idle,
                    Hunger::new(51.0, 0.0),
                    MovementRng::new(42),
                ),
            )
            .expect("seed end-to-end agent");

        let mut reached = false;
        for _ in 0..30 {
            e.tick();
            if pos(&e, agent) == (20, 20) {
                reached = true;
            }
        }

        let final_hunger = e.world.get::<&Hunger>(agent).expect("hunger").value;
        let final_state = state(&e, agent);
        let source_value = e.resources.food_tiles.get(&(20, 20)).copied();
        LoopResult {
            final_hunger,
            final_state,
            reached_source: reached,
            source_value,
        }
    }

    // ─── Assertion 1: one directed signum step toward food target ──────────────
    #[test]
    fn harness_movement_directed_step_toward_food_target_one_tick() {
        // Type: A — hand-computed signum: dx=signum(15-10)=+1, dy=signum(10-10)=0
        // ⇒ exactly (11,10) after one movement tick. RNG-free.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((15, 10))),
        );
        run_movement(&mut e, 1);
        assert_eq!(pos(&e, a), (11, 10), "A1: one signum step → (11,10)");
        println!("[S16-β A1] (10,10)+target(15,10) → (11,10) after 1 tick ✓");
    }

    // ─── Assertion 2: directed path reaches then holds the target ──────────────
    #[test]
    fn harness_movement_directed_path_reaches_then_holds_target() {
        // Type: A — Chebyshev 5; one step/tick closes it in 5 ticks, then dx=0
        // holds for the remaining 3 ticks. x never overshoots past 15.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((15, 10))),
        );
        let mut sys = AgentMovementSystem::new();
        let mut prev_x = 10u32;
        for tick in 1..=8 {
            sys.tick(&mut e.world, &mut e.resources);
            let (x, y) = pos(&e, a);
            assert_eq!(y, 10, "A2: y must never drift (tick {tick})");
            assert!(
                x >= prev_x,
                "A2: x must be monotonically non-decreasing (tick {tick})"
            );
            assert!(
                x <= 15,
                "A2: x must never overshoot past 15 (tick {tick}, x={x})"
            );
            prev_x = x;
        }
        assert_eq!(
            pos(&e, a),
            (15, 10),
            "A2: final position must be the target (15,10)"
        );
        println!("[S16-β A2] (10,10)→(15,10) in 5 steps, holds through tick 8 ✓");
    }

    // ─── Assertion 3: directed movement deterministic AND non-frozen ───────────
    #[test]
    fn harness_movement_directed_movement_deterministic_and_nonfrozen() {
        // Type: A — two independent engines, identical setup, 6 ticks each:
        // (a) cross-engine equal final position, (b) BOTH actually displaced
        // toward goal (pos.x > 10) — a no-op β would freeze at x=10 and fail (b).
        fn build_and_run() -> (u32, u32) {
            let mut e = engine();
            let a = spawn_probe(
                &mut e,
                (10, 10),
                42,
                AgentState::Seeking {
                    target: TargetKind::Food,
                },
                Some(SeekTarget::new((15, 10))),
            );
            run_movement(&mut e, 6);
            pos(&e, a)
        }
        let pa = build_and_run();
        let pb = build_and_run();
        assert_eq!(
            pa, pb,
            "A3a: identical setup must yield identical final position"
        );
        assert!(
            pa.0 > 10,
            "A3b: engine_a agent must have moved toward goal (x>10)"
        );
        assert!(
            pb.0 > 10,
            "A3b: engine_b agent must have moved toward goal (x>10)"
        );
        println!("[S16-β A3] deterministic ({pa:?}=={pb:?}) AND non-frozen (x>10) ✓");
    }

    // ─── Assertion 4: end-to-end gathering loop satisfies hunger ───────────────
    #[test]
    fn harness_movement_end_to_end_gathering_loop_satisfies_hunger() {
        // Type: C — breach→Seeking(α)→directed-diagonal-walk(β)→arrive→Consuming
        // →satisfied→Idle fires end-to-end. Computed final Hunger 51-30=21.0 < 50.
        let r = run_gathering_loop();
        assert!(
            r.final_hunger < 50.0,
            "A4: final Hunger must drop below HUNGER_THRESHOLD(50.0); got {}",
            r.final_hunger
        );
        assert_eq!(
            r.final_state,
            AgentState::Idle,
            "A4: agent must return to Idle"
        );
        println!(
            "[S16-β A4] end-to-end: Hunger {} < 50.0, state Idle ✓",
            r.final_hunger
        );
    }

    // ─── Assertion 5: Seeking without a target freezes ─────────────────────────
    #[test]
    fn harness_movement_seeking_without_target_freezes() {
        // Type: A — Seeking with NO SeekTarget + empty food_tiles must freeze
        // (no Brownian, no directed step). Position drift = no-target guard bug.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            42,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            None,
        );
        run_movement(&mut e, 32);
        assert_eq!(
            pos(&e, a),
            (10, 10),
            "A5: no-target Seeking agent must stay frozen"
        );
        println!("[S16-β A5] Seeking + no SeekTarget → frozen at (10,10) ✓");
    }

    // ─── Assertion 6: Consuming state freezes ──────────────────────────────────
    #[test]
    fn harness_movement_consuming_state_freezes() {
        // Type: D — regression guard. Consuming must freeze so the 2-tick consume
        // commit reads the correct tile. Seed 1 produces a non-zero step when not
        // suppressed, so a regressed freeze would surface observably.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            1,
            AgentState::Consuming {
                target: TargetKind::Food,
            },
            None,
        );
        run_movement(&mut e, 32);
        assert_eq!(pos(&e, a), (10, 10), "A6: Consuming agent must stay frozen");
        // Sanity: seed 1 does produce movement when not suppressed.
        let mut probe = MovementRng::new(1);
        let dx = probe.next_step();
        let dy = probe.next_step();
        assert!(
            dx != 0 || dy != 0,
            "A6: seed 1 must yield a non-zero step (else vacuous)"
        );
        println!("[S16-β A6] Consuming{{Food}} → frozen at (10,10) ✓");
    }

    // ─── Assertion 7: Idle Brownian preserved, bounded ─────────────────────────
    #[test]
    fn harness_movement_idle_brownian_preserved_bounded() {
        // Type: A — Idle agents must still Brownian-walk: (a) net displacement
        // non-zero over 32 ticks (HARD assert — never vacuously passes), and
        // (b) every per-axis step ≤ 1. Seed 42 verified to move under current
        // code (matches the movement.rs `idle_state_still_moves` seed).
        let mut e = engine();
        let a = spawn_probe(&mut e, (16, 16), 42, AgentState::Idle, None);
        let mut sys = AgentMovementSystem::new();
        let mut prev = (16u32, 16u32);
        let mut moved = false;
        for tick in 1..=32 {
            sys.tick(&mut e.world, &mut e.resources);
            let (x, y) = pos(&e, a);
            assert!(
                x.abs_diff(prev.0) <= 1,
                "A7: |Δx| ≤ 1 per tick (tick {tick})"
            );
            assert!(
                y.abs_diff(prev.1) <= 1,
                "A7: |Δy| ≤ 1 per tick (tick {tick})"
            );
            if (x, y) != (16, 16) {
                moved = true;
            }
            prev = (x, y);
        }
        assert!(
            moved,
            "A7: Idle agent must execute Brownian motion (net displacement ≠ 0)"
        );
        println!("[S16-β A7] Idle Brownian preserved, |Δ|≤1/axis, moved=true ✓");
    }

    // ─── Assertion 8: Water + Sleep directed steps (kind-agnostic) ─────────────
    #[test]
    fn harness_movement_water_and_sleep_directed_steps() {
        // Type: A — directed branch keys on Seeking{..} regardless of TargetKind.
        // (a) Water target (10,15): dy=+1 → (10,11). (b) Sleep target (5,10):
        // dx=-1 → (9,10).
        let mut e = engine();
        let water = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Water,
            },
            Some(SeekTarget::new((10, 15))),
        );
        let sleep = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Sleep,
            },
            Some(SeekTarget::new((5, 10))),
        );
        run_movement(&mut e, 1);
        assert_eq!(pos(&e, water), (10, 11), "A8a: Water seek → (10,11)");
        assert_eq!(pos(&e, sleep), (9, 10), "A8b: Sleep seek → (9,10)");
        println!("[S16-β A8] Water→(10,11), Sleep→(9,10) — kind-agnostic ✓");
    }

    // ─── Assertion 9: infinite source not depleted by the loop ─────────────────
    #[test]
    fn harness_movement_infinite_source_not_depleted_by_loop() {
        // Type: D — validity-conjoined with A4 (Hunger dropped) + A16 (reached).
        // The u8::MAX source must remain u8::MAX after a real co-located consume.
        let r = run_gathering_loop();
        // Validity precondition: only meaningful if the agent actually consumed.
        assert!(
            r.final_hunger < 50.0,
            "A9 precondition (A4): Hunger must have dropped"
        );
        assert!(
            r.reached_source,
            "A9 precondition (A16): agent must have reached (20,20)"
        );
        assert_eq!(
            r.source_value,
            Some(RESOURCE_SOURCE_INFINITE),
            "A9: source tile (20,20) must remain u8::MAX after a genuine consume"
        );
        println!("[S16-β A9] source (20,20) still u8::MAX after consumed-from loop ✓");
    }

    // ─── Assertion 10: co-located target → signum-0 → no move ──────────────────
    #[test]
    fn harness_movement_colocated_target_signum_zero_no_move() {
        // Type: A — agent already ON its SeekTarget: dx=signum(0)=0, dy=signum(0)=0
        // ⇒ zero displacement.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (12, 12),
            7,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((12, 12))),
        );
        run_movement(&mut e, 1);
        assert_eq!(pos(&e, a), (12, 12), "A10: signum-0 on both axes → no move");
        println!("[S16-β A10] on-target agent holds station at (12,12) ✓");
    }

    // ─── Assertion 11: suppresses_movement() truth table unchanged ─────────────
    #[test]
    fn harness_movement_suppresses_movement_truth_table_unchanged() {
        // Type: D — cross-phase regression guard on the LOCKED predicate. β must
        // NOT alter it; 5 external harnesses depend on it.
        assert!(
            AgentState::Seeking {
                target: TargetKind::Food
            }
            .suppresses_movement(),
            "A11: Seeking{{Food}}.suppresses_movement() must remain true"
        );
        assert!(
            !AgentState::Idle.suppresses_movement(),
            "A11: Idle.suppresses_movement() must remain false"
        );
        println!("[S16-β A11] truth table intact: Seeking→true, Idle→false ✓");
    }

    // ─── Assertion 12: directed step does not consume RNG ──────────────────────
    #[test]
    fn harness_movement_directed_step_does_not_consume_rng() {
        // Type: A — Agent A runs 5 directed ticks (carrying SeekTarget) then goes
        // Idle; Agent B goes Idle immediately. Same seed S. If the directed step
        // consumed no RNG, A's stream is un-advanced, so A's first Idle Brownian
        // Δ must equal B's. Compare STEP VECTORS (cancels A's directed offset).
        const S: u64 = 12345;
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            S,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((15, 10))),
        );
        let b = spawn_probe(&mut e, (10, 10), S, AgentState::Idle, None);

        // Preferred direct check: after 5 directed ticks A's RNG state must be
        // byte-identical to a fresh seed-S RNG (MovementRng derives PartialEq).
        run_movement(&mut e, 5);
        {
            let rng_a = *e.world.get::<&MovementRng>(a).expect("A rng");
            assert_eq!(
                rng_a,
                MovementRng::new(S),
                "A12: directed steps must NOT advance the RNG (state must equal fresh seed S)"
            );
        }

        // Observable proxy: transition A → Idle, take one Brownian step each, and
        // compare per-axis displacement vectors.
        e.world.insert_one(a, AgentState::Idle).expect("A → Idle");
        let a_before = pos(&e, a);
        let b_before = pos(&e, b);
        run_movement(&mut e, 1);
        let a_after = pos(&e, a);
        let b_after = pos(&e, b);
        let da = (
            a_after.0 as i64 - a_before.0 as i64,
            a_after.1 as i64 - a_before.1 as i64,
        );
        let db = (
            b_after.0 as i64 - b_before.0 as i64,
            b_after.1 as i64 - b_before.1 as i64,
        );
        assert_eq!(
            da, db,
            "A12: A and B first-Idle Brownian Δ must match (RNG un-consumed)"
        );
        println!("[S16-β A12] directed step consumes no RNG (Δ {da:?}=={db:?}) ✓");
    }

    // ─── Assertion 13: simultaneous positive two-axis step in one tick ─────────
    #[test]
    fn harness_movement_diagonal_two_axis_step_one_tick() {
        // Type: A — dx=signum(15-10)=+1, dy=signum(15-10)=+1 ⇒ (11,11). Catches
        // the update-x-then-early-return-before-y bug class (would yield (11,10)).
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((15, 15))),
        );
        run_movement(&mut e, 1);
        assert_eq!(pos(&e, a), (11, 11), "A13: both axes step +1 → (11,11)");
        println!("[S16-β A13] diagonal +x/+y in one tick → (11,11) ✓");
    }

    // ─── Assertion 14: simultaneous negative two-axis step in one tick ─────────
    #[test]
    fn harness_movement_diagonal_negative_two_axis_step_one_tick() {
        // Type: A — dx=signum(5-10)=-1, dy=signum(5-10)=-1 ⇒ (9,9). Covers the
        // combined-negative direction entirely absent from prior plans.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((5, 5))),
        );
        run_movement(&mut e, 1);
        assert_eq!(pos(&e, a), (9, 9), "A14: both axes step -1 → (9,9)");
        println!("[S16-β A14] diagonal -x/-y in one tick → (9,9) ✓");
    }

    // ─── Assertion 15: staggered-axis arrival (one axis holds, other closes) ───
    #[test]
    fn harness_movement_diagonal_staggered_axis_arrival() {
        // Type: A — target (15,12): y closes 10→12 in 2 ticks then signum-0 holds;
        // x closes 10→15 in 5. A stop-when-EITHER-axis-arrives bug freezes x at 12.
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            7,
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            Some(SeekTarget::new((15, 12))),
        );
        let mut sys = AgentMovementSystem::new();
        let mut prev_x = 10u32;
        for tick in 1..=8 {
            sys.tick(&mut e.world, &mut e.resources);
            let (x, y) = pos(&e, a);
            if tick >= 2 {
                assert_eq!(
                    y, 12,
                    "A15: y must reach 12 by tick 2 and hold (tick {tick})"
                );
            }
            assert!(x >= prev_x, "A15: x must be non-decreasing (tick {tick})");
            assert!(
                x <= 15,
                "A15: x must never overshoot 15 (tick {tick}, x={x})"
            );
            prev_x = x;
        }
        assert_eq!(pos(&e, a), (15, 12), "A15: final position must be (15,12)");
        println!("[S16-β A15] staggered arrival: y holds at 12 while x closes to 15 ✓");
    }

    // ─── Assertion 16: agent reaches the source tile during the loop ───────────
    #[test]
    fn harness_movement_agent_reaches_source_tile_during_loop() {
        // Type: A — de-vacuums A9: the agent's Position must equal (20,20) on at
        // least one tick of the 30-tick run (walked the full diagonal path).
        let r = run_gathering_loop();
        assert!(
            r.reached_source,
            "A16: agent must reach source (20,20) on at least one tick"
        );
        // Anchor: the run is the same one A4 measured (Hunger dropped).
        assert!(
            r.final_hunger < 50.0,
            "A16: same run as A4 — Hunger must have dropped"
        );
        println!("[S16-β A16] agent reached (20,20) during the 30-tick loop ✓");
    }
}
