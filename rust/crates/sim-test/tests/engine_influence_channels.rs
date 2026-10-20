//! Merged in V8-0 (8.4) from `harness_t7_10_a_warmth_wiring.rs`,
//! `harness_t7_10_b_light_shadowcast_wiring.rs`,
//! `harness_t7_10_b1_light_viz_toggle.rs`,
//! `harness_t7_10_c_noise_linear_wiring.rs`,
//! `harness_t7_10_d_danger_linear_wiring.rs`,
//! `harness_t7_10_e_spiritual_bfs_wiring.rs`,
//! `harness_t7_10_f_beauty_bfs_wiring.rs`; each source is one module below,
//! unchanged.

// From `harness_t7_10_a_warmth_wiring.rs`.
mod warmth {
    //! T7.10.A — Warmth channel BFS wiring (single-channel Phase 2 escape).
    //!
    //! After T7.6 dispatch shell + T7.9.A scaffold + T7.9.B render, the
    //! Warmth channel is the first to escape the shell. A BuildingPlacedEvent
    //! at (32, 32) with radius 12 must produce a radial warmth disc in
    //! `current[Warmth]` centered at (32, 32) after one tick.
    //!
    //! Setup note: 64×64 engine with empty MaterialRegistry (open field,
    //! no wall blocking). All thresholds are LOCKED from plan_final.md —
    //! do NOT modify them.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    fn place_warmth_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    // ── Plan A1: source center reaches initial intensity ─────────────────────────

    /// Type A: influence_grid.sample(SX, SY, Warmth) == 200 after 1 tick post-event.
    ///
    /// Physical invariant. The BFS source tile must receive full initial_intensity
    /// (200) via apply_agg at the start of propagation. Any value ≠ 200 means
    /// propagation never ran or the source seeding is broken.
    #[test]
    fn harness_t7_10_a_source_center_lit() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();
        // Type A: threshold == 200
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            v, 200,
            "source center must equal WARMTH_INITIAL_INTENSITY (200); got {v}"
        );
    }

    // ── Plan A2: radial decay one step ───────────────────────────────────────────

    /// Type A: influence_grid.sample(SX, SY+1, Warmth) ∈ [170, 174] after 1 tick.
    ///
    /// Mathematical invariant. Warmth decay k=0.15, WARMTH_DECAY_PER_STEP =
    /// exp(-0.15) ≈ 0.8607. Chain step: floor(200.0 * 0.8607) = 172.
    /// Range [170, 174] adds ±2 tolerance for f32 rounding.
    #[test]
    fn harness_t7_10_a_radial_decay_one_step() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [170, 174]
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY + 1, InfluenceChannel::Warmth);
        assert!(
            (170..=174).contains(&v),
            "1-step neighbor must decay to ~172 (200*exp(-0.15)); got {v}. \
         Below 170 = k wrong; above 174 = no decay applied."
        );
    }

    // ── Plan A3: gradient monotonically decreasing along cardinal axis ────────────

    /// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for all d in 0..=11.
    ///
    /// Mathematical invariant. Exponential decay with k=0.15 > 0 is strictly
    /// monotonically decreasing. Violation at any step indicates BFS ordering
    /// error or wrong decay direction.
    #[test]
    fn harness_t7_10_a_gradient_monotone() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();
        for d in 0u32..=11 {
            let v_near = e
                .resources
                .influence_grid
                .sample(SX + d, SY, InfluenceChannel::Warmth);
            let v_far = e
                .resources
                .influence_grid
                .sample(SX + d + 1, SY, InfluenceChannel::Warmth);
            // Type A: strict monotone decrease for each pair (d, d+1)
            assert!(
                v_near > v_far,
                "gradient must be strictly decreasing: \
             sample({},{})={} must > sample({},{})={}; \
             violation at d={d} means BFS decay is broken",
                SX + d,
                SY,
                v_near,
                SX + d + 1,
                SY,
                v_far,
                d = d
            );
        }
    }

    // ── Plan A4: boundary tile at max_radius is nonzero ──────────────────────────

    /// Type A: influence_grid.sample(44, 32, Warmth) ∈ [26, 34] after 1 tick.
    ///
    /// Mathematical invariant. The BFS must reach max_radius=12 tiles.
    /// Chain-computed value at d=12: floor(36.0 * 0.8607) = 30.
    /// Range [26, 34] gives ±4 margin for accumulated f32 truncation.
    /// Value == 0 means propagation stopped prematurely.
    #[test]
    fn harness_t7_10_a_boundary_at_max_radius() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();
        // Tile (44, 32) is exactly Manhattan distance 12 from source (32, 32).
        // Type A: threshold ∈ [26, 34]
        let v = e
            .resources
            .influence_grid
            .sample(44, 32, InfluenceChannel::Warmth);
        assert!(
            (26..=34).contains(&v),
            "tile at max_radius=12 must be in [26,34] (chain end ≈30); got {v}; \
         value==0 means propagation stopped before max_radius"
        );
    }

    // ── Plan A5: beyond max_radius is zero ───────────────────────────────────────

    /// Type A: influence_grid.sample(SX+13, SY, Warmth) == 0 after 1 tick.
    ///
    /// Hard cutoff invariant. max_radius=12 is enforced by BFS termination.
    /// Tile at Manhattan distance 13 must NEVER be written.
    #[test]
    fn harness_t7_10_a_max_radius_cutoff() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();
        // Type A: threshold == 0 (distance 13 from source, one beyond max_radius=12)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 13, SY, InfluenceChannel::Warmth);
        assert_eq!(v, 0, "tile beyond max_radius=12 must be 0; got {v}");
    }

    // ── Plan A6: cold-tier persistence across 10 event-less ticks ────────────────

    /// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
    ///
    /// Cold-tier event-driven semantics. Warmth is UpdateTier::Cold — it must
    /// persist indefinitely between placement events. If the persistence branch
    /// (copy current→pending before swap) is missing, the disc flickers to 0
    /// on every event-less tick.
    #[test]
    fn harness_t7_10_a_persistence_across_ticks() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick(); // tick 1: stamp + propagation
        for _ in 0..10 {
            e.tick(); // ticks 2–11: event-less persistence branch
        }
        // Type A: threshold == 200 (source persists through all 10 idle ticks)
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            v, 200,
            "source must persist across 10 event-less ticks (Cold-tier semantics); got {v}"
        );
    }

    // ── Plan A7+A8: non-Warmth channels remain zero at source ────────────────────

    /// Type A: sample(SX, SY, ch) == 0 for Spiritual/Beauty/Light (A7) and
    ///         Noise/FoodAroma/Danger/Social (A8) after 1 tick with event.
    ///
    /// T7.10.A wires the Warmth channel; T7.10.B then wired Light. Spiritual/Beauty
    /// have BSS dirty_regions populated but IUS must NOT propagate them yet (T7.10.C..F).
    /// Noise/FoodAroma/Danger/Social are not stamped at all. These 6 channels must
    /// remain zero. Light is verified separately (it now propagates to 200 at source).
    #[test]
    fn harness_t7_10_a_other_channels_remain_zero() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();

        // T7.10.B regression guard: Light now propagates at source center.
        let light = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            light, 200,
            "T7.10.B: Light at source must be 200 (shadowcast propagation); got {light}"
        );

        // T7.10.C regression guard: Noise now propagates at source center.
        let noise = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            noise, 200,
            "T7.10.C: Noise at source must be 200 (linear-decay propagation); got {noise}"
        );

        // T7.10.D regression guard: Danger now propagates at source center.
        let danger = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            danger, 200,
            "T7.10.D: Danger at source must be 200 (linear-decay+cap propagation); got {danger}"
        );

        // T7.10.E regression guard: Spiritual now propagates at source center.
        let spiritual = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
        spiritual, 200,
        "T7.10.E: Spiritual at source must be 200 (BFS exp k=0.08 propagation); got {spiritual}"
    );

        // T7.10.F regression guard: Beauty now propagates at source center.
        let beauty = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            beauty, 200,
            "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
        );

        for ch in [
            // Unstamped channels (BSS never marks dirty) — only 2 remain post-T7.10.F.
            InfluenceChannel::FoodAroma,
            InfluenceChannel::Social,
        ] {
            // Type A: threshold == 0 for the 2 unstamped channels
            let v = e.resources.influence_grid.sample(SX, SY, ch);
            assert_eq!(v, 0, "{ch:?} must remain zero post-T7.10.F (all 6 stamped channels wired; only FoodAroma/Social unstamped); got {v}");
        }
    }

    // ── Plan A9: no warmth without events ────────────────────────────────────────

    /// Type A: Warmth == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
    ///
    /// Without any BuildingPlacedEvent, dirty_regions[Warmth] stays empty every
    /// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
    /// Any nonzero value means uninitialized buffer state leaking through.
    #[test]
    fn harness_t7_10_a_no_event_no_warmth() {
        let mut e = fresh_engine();
        for _ in 0..5 {
            e.tick();
        }
        for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
            // Type A: threshold == 0 at all three positions with no events
            let v = e
                .resources
                .influence_grid
                .sample(x, y, InfluenceChannel::Warmth);
            assert_eq!(v, 0, "no events ⇒ Warmth stays zero at ({x},{y}); got {v}");
        }
    }

    // ── Plan A10: dirty_regions for Warmth are drained after tick ────────────────

    /// Type A: dirty_regions[Warmth].len() == 0 after 1 tick processing a Warmth event.
    ///
    /// Correctness invariant. IUS must drain dirty_regions[Warmth] during the tick
    /// that processes them (via std::mem::take, not just read). If dirty_regions
    /// are not drained, tick N+1 re-runs BFS from stale regions, breaking Cold-tier
    /// event-driven semantics and causing incorrect additive accumulation.
    #[test]
    fn harness_t7_10_a_dirty_regions_drained() {
        let mut e = fresh_engine();
        place_warmth_source(&mut e);
        e.tick();
        // Type A: threshold == 0 (IUS drains via std::mem::take)
        let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[Warmth] must be drained (len==0) after IUS processes them; got {len}"
        );
    }
}

// From `harness_t7_10_b_light_shadowcast_wiring.rs`.
mod light {
    //! T7.10.B — Light channel shadowcast wiring (second-channel Phase 2 escape).
    //!
    //! After T7.10.A wired Warmth via BFS, the Light channel is the second to
    //! escape the dispatch shell. A BuildingPlacedEvent at (32, 32) with
    //! radius 12 must produce a Light field in `current[Light]` centered at
    //! (32, 32) after one tick, computed by recursive symmetric shadowcasting
    //! with `intensity / (1 + 0.1 * d)` Euclidean falloff (propagate.rs:249).
    //!
    //! Setup note: 64×64 engine with empty MaterialRegistry (open field,
    //! no wall blocking → all-open shadowcast = octagonal/circular shape).
    //! All thresholds are LOCKED from plan — do NOT modify them.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    fn place_light_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    // ── Plan B1: source center reaches initial intensity ─────────────────────────

    /// Type A: influence_grid.sample(SX, SY, Light) == 200 after 1 tick post-event.
    ///
    /// Physical invariant. The shadowcast source tile must receive full
    /// LIGHT_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
    /// propagation (propagate.rs:161). Any value ≠ 200 means propagation
    /// never ran or the source seeding is broken.
    #[test]
    fn harness_t7_10_b_source_center_lit() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();
        // Type A: threshold == 200
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            v, 200,
            "source center must equal LIGHT_INITIAL_INTENSITY (200); got {v}"
        );
    }

    // ── Plan B2: shadowcast falloff one cardinal step ────────────────────────────

    /// Type A: influence_grid.sample(SX+1, SY, Light) ∈ [178, 184] after 1 tick.
    ///
    /// Mathematical invariant. Shadowcast falloff `intensity / (1 + 0.1 * d)`
    /// (propagate.rs:249). At d=1: 200 / 1.1 = 181.8 → truncates to 181.
    /// Range [178, 184] adds ±3 tolerance for f32 rounding across octants.
    #[test]
    fn harness_t7_10_b_falloff_one_step() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [178, 184]
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY, InfluenceChannel::Light);
        assert!(
            (178..=184).contains(&v),
            "1-step neighbor must decay to ~181 (200/(1+0.1)); got {v}. \
         Below 178 = falloff wrong; above 184 = no decay applied."
        );
    }

    // ── Plan B3: diagonal tile uses Euclidean distance (discriminator) ──────────

    /// Type A: influence_grid.sample(SX+1, SY+1, Light) ∈ [172, 179] after 1 tick.
    ///
    /// Euclidean discriminator invariant. Diagonal distance = sqrt(2) ≈ 1.4142.
    /// Shadowcast falloff: 200 / (1 + 0.1 * 1.4142) = 200 / 1.14142 ≈ 175.
    /// Range [172, 179] adds tolerance for f32 sqrt rounding across platforms.
    ///
    /// A BFS-Manhattan implementation would use d=2 for (SX+1, SY+1),
    /// yielding floor(200 / 1.2) = 166 — OUTSIDE [172, 179]. Any
    /// implementation that produces a value in [172, 179] provably uses
    /// Euclidean distance, not Manhattan distance.
    #[test]
    fn harness_t7_10_b_falloff_diagonal_euclidean_discriminator() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [172, 179]  (Euclidean d≈1.414 → 200/1.1414≈175)
        // BFS-Manhattan gives 166 which falls OUTSIDE this range — discriminates impls.
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY + 1, InfluenceChannel::Light);
        assert!(
            (172..=179).contains(&v),
            "diagonal tile (SX+1,SY+1) must be in [172,179] \
         (Euclidean d≈1.414 → ≈175); got {v}. \
         Value 166 = BFS-Manhattan (d=2 wrongly used). \
         Outside [172,179] = wrong falloff formula or distance metric."
        );
    }

    // ── Plan B4: gradient monotonically strictly decreasing along cardinal axis ──

    /// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
    ///
    /// Mathematical invariant. Shadowcast falloff `intensity / (1 + 0.1 * d)` with
    /// k=0.1 > 0 is strictly monotone decreasing in continuous form. After u8
    /// truncation, the minimum inter-step gap along this cardinal axis is 3
    /// (at d=14→15: floor(200/2.4)=83 > floor(200/2.5)=80), so strict `>` is safe
    /// for every adjacent pair — no truncation ties possible in [0,14]. Violation
    /// at any step means the Euclidean distance computation or falloff is broken.
    #[test]
    fn harness_t7_10_b_gradient_monotone() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();

        // Strict pairwise decrease for every adjacent pair along cardinal axis.
        // Min gap ≥ 3 (at d=14→15: floor(200/2.4)=83 > floor(200/2.5)=80),
        // so strict `>` is safe — no u8 truncation ties possible.
        for d in 0u32..=14 {
            let v_near = e
                .resources
                .influence_grid
                .sample(SX + d, SY, InfluenceChannel::Light);
            let v_far = e
                .resources
                .influence_grid
                .sample(SX + d + 1, SY, InfluenceChannel::Light);
            // Type A: strictly decreasing for every pair (d, d+1) on [0,14]
            assert!(
                v_near > v_far,
                "gradient must be strictly decreasing: \
             sample({},{})={} must > sample({},{})={}; \
             violation at d={d} means shadowcast falloff is broken",
                SX + d,
                SY,
                v_near,
                SX + d + 1,
                SY,
                v_far,
            );
        }
    }

    // ── Plan B5: boundary tile at max_radius is nonzero ──────────────────────────

    /// Type A: influence_grid.sample(SX+15, SY, Light) ∈ [76, 84] after 1 tick.
    ///
    /// Mathematical invariant. The shadowcast must reach max_radius=15 along
    /// the cardinal axis (Euclidean dist = 15 exactly). Expected:
    /// 200 / (1 + 0.1*15) = 200 / 2.5 = 80.
    /// Range [76, 84] gives ±4 margin for f32 sqrt and clamp truncation.
    /// Value == 0 means propagation stopped prematurely.
    #[test]
    fn harness_t7_10_b_boundary_at_max_radius() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();
        // Tile (SX+15, SY) is Euclidean distance exactly 15 from source.
        // Type A: threshold ∈ [76, 84]
        let v = e
            .resources
            .influence_grid
            .sample(SX + 15, SY, InfluenceChannel::Light);
        assert!(
            (76..=84).contains(&v),
            "tile at max_radius=15 must be in [76,84] (chain end ≈80); got {v}; \
         value==0 means shadowcast stopped before max_radius"
        );
    }

    // ── Plan B6: beyond max_radius is zero ───────────────────────────────────────

    /// Type A: influence_grid.sample(SX+16, SY, Light) == 0 after 1 tick.
    ///
    /// Hard cutoff invariant. max_radius=15 is enforced by shadowcast
    /// `dist_sq <= radius²` check (propagate.rs:247). At (SX+16, SY)
    /// the Euclidean distance squared = 256 > 225 → never written.
    #[test]
    fn harness_t7_10_b_max_radius_cutoff() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();
        // Type A: threshold == 0 (Euclidean distance 16 > max_radius 15)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 16, SY, InfluenceChannel::Light);
        assert_eq!(v, 0, "tile beyond max_radius=15 must be 0; got {v}");
    }

    // ── Plan B7: Warm-tier persistence across 10 event-less ticks ────────────────

    /// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
    ///
    /// Warm-tier persistence (T7.10.B). Light must survive event-less ticks
    /// just like Warmth does for Cold-tier. If the persistence branch (copy
    /// current[Light] → pending[Light] before swap) is missing, the field
    /// flickers to 0 on every event-less tick.
    #[test]
    fn harness_t7_10_b_persistence_across_ticks() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick(); // tick 1: stamp + shadowcast
        for _ in 0..10 {
            e.tick(); // ticks 2–11: event-less persistence branch
        }
        // Type A: threshold == 200 (source persists through all 10 idle ticks)
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            v, 200,
            "source must persist across 10 event-less ticks (Warm-tier persistence); got {v}"
        );
    }

    // ── Plan B8+B9: non-Light channels behavior at source ────────────────────────

    /// Type A: at source after 1 tick with event:
    ///   - Warmth = 200 (T7.10.A regression guard — must remain wired)
    ///   - Spiritual/Beauty = 0 (BSS marks dirty but IUS dispatch-shell)
    ///   - Noise/FoodAroma/Danger/Social = 0 (not stamped)
    ///
    /// T7.10.B wires the Light channel. The T7.10.A Warmth wiring must NOT
    /// regress (Warmth still propagates to 200 at source). Spiritual/Beauty
    /// have BSS dirty_regions populated but IUS does not propagate them yet
    /// (dispatch-shell clears pending → swap → 0). Unstamped channels stay 0.
    #[test]
    fn harness_t7_10_b_other_channels_behavior() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();

        // T7.10.A regression guard: Warmth must still propagate to 200 at source.
        let warmth = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            warmth, 200,
            "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
        );

        // T7.10.C regression guard: Noise must propagate to 200 at source.
        let noise = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            noise, 200,
            "T7.10.C: Noise at source must be 200 (linear-decay propagation); got {noise}"
        );

        // T7.10.D regression guard: Danger must propagate to 200 at source.
        let danger = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            danger, 200,
            "T7.10.D: Danger at source must be 200 (linear-decay+cap propagation); got {danger}"
        );

        // T7.10.E regression guard: Spiritual must propagate to 200 at source.
        let spiritual = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
        spiritual, 200,
        "T7.10.E: Spiritual at source must be 200 (BFS exp k=0.08 propagation); got {spiritual}"
    );

        // T7.10.F regression guard: Beauty now propagates at source center.
        let beauty = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            beauty, 200,
            "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
        );

        // Unstamped channels (BSS never marks dirty) — only 2 remain post-T7.10.F.
        for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
            let v = e.resources.influence_grid.sample(SX, SY, ch);
            assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
        }
    }

    // ── Plan B10: no event = no light ────────────────────────────────────────────

    /// Type A: Light == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
    ///
    /// Without any BuildingPlacedEvent, dirty_regions[Light] stays empty every
    /// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
    /// Any nonzero value means uninitialized buffer state leaking through.
    #[test]
    fn harness_t7_10_b_no_event_no_light() {
        let mut e = fresh_engine();
        for _ in 0..5 {
            e.tick();
        }
        for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
            // Type A: threshold == 0 at all three positions with no events
            let v = e
                .resources
                .influence_grid
                .sample(x, y, InfluenceChannel::Light);
            assert_eq!(v, 0, "no events ⇒ Light stays zero at ({x},{y}); got {v}");
        }
    }

    // ── Plan B11: dirty_regions for Light are drained after tick ─────────────────

    /// Type A: dirty_regions[Light].len() == 0 after 1 tick processing a Light event.
    ///
    /// Correctness invariant. IUS must drain dirty_regions[Light] during the tick
    /// that processes them (via std::mem::take, not just read). If dirty_regions
    /// are not drained, tick N+1 re-runs shadowcast from stale regions, breaking
    /// Warm-tier persistence semantics and wasting CPU on redundant propagation.
    #[test]
    fn harness_t7_10_b_dirty_regions_drained() {
        let mut e = fresh_engine();
        place_light_source(&mut e);
        e.tick();
        // Type A: threshold == 0 (IUS drains via std::mem::take)
        let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Light as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[Light] must be drained (len==0) after IUS processes them; got {len}"
        );
    }
}

// From `harness_t7_10_b1_light_viz_toggle.rs`.
mod light_viz_toggle {
    //! T7.10.B1 — Light visualization toggle: concurrent channel validity.
    //!
    //! These tests verify the PRECONDITIONS for the SPACE-key toggle in
    //! `world_renderer.gd`: both the Warmth channel (T7.10.A) and the Light
    //! channel (T7.10.B) must remain fully populated in the SAME engine run.
    //!
    //! The toggle only works correctly if:
    //!   1. Warmth channel has a genuine non-trivial disc when Light co-exists.
    //!   2. Light channel has a genuine non-trivial disc when Warmth co-exists.
    //!   3. The Light disc extends beyond the Warmth disc (toggle has visual value).
    //!   4. The Light source tile holds the global channel maximum.
    //!   5. Light is zero beyond spec radius (no bleed past d=16).
    //!
    //! T7.10.A and T7.10.B each test one channel in isolation. B1 tests cross-
    //! channel concurrent validity — the exact invariant the toggle depends on.
    //!
    //! Setup: 64×64 engine, empty MaterialRegistry (all-open field), single
    //! BuildingPlacedEvent at (32,32) radius=12 consumed in tick 1; ticks 2–10
    //! exercise Cold/Warm-tier persistence. All thresholds LOCKED from plan.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_b1_light_viz_toggle -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    /// Fresh 64×64 engine with all Phase 2 systems registered.
    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    /// Push a single BuildingPlacedEvent at (32,32) r=12.
    /// Same event triggers both Warmth BFS (T7.10.A) and Light shadowcast (T7.10.B).
    fn place_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    /// Run `n` ticks. Event (if any) is consumed in the first tick.
    fn run_ticks(engine: &mut SimEngine, n: u32) {
        for _ in 0..n {
            engine.tick();
        }
    }

    // ── B1-1: Warmth disc is non-trivial while Light co-exists ───────────────────

    /// Type A: Warmth non-zero tile count >= 300 after 10 ticks.
    ///
    /// Warmth BFS radius ~12 tiles → disc area π×12² ≈ 452 tiles in a 64×64 grid.
    /// Threshold 300 = 66% of the theoretical disc, conservative for BFS boundary
    /// discretisation. This test runs in the SAME engine instance as B1-2 (Light)
    /// to confirm the Warmth channel remains fully populated while the Light channel
    /// co-exists in the same SimResources. T7.10.A verifies Warmth in isolation;
    /// B1-1 asserts it survives concurrent channel presence.
    #[test]
    fn harness_t7_10_b1_warmth_channel_non_trivial_concurrent() {
        let mut e = fresh_engine();
        place_source(&mut e);
        run_ticks(&mut e, 10);

        let warmth_buf = e
            .resources
            .influence_grid
            .current_buf(InfluenceChannel::Warmth);
        // Type A: threshold >= 300 non-zero positions
        let count = warmth_buf.iter().filter(|&&v| v > 0).count();
        assert!(
            count >= 300,
            "Warmth disc must have >= 300 non-zero tiles when Light co-exists \
         (BFS radius ~12, disc area ~452); got {count}. \
         T7.10.A regression: Warmth channel not surviving concurrent Light presence."
        );
        println!("[B1-1] Warmth non-zero tiles: {count} (threshold >= 300)");
    }

    // ── B1-2: Light disc is non-trivial while Warmth co-exists ───────────────────

    /// Type A: Light non-zero tile count >= 450 after 10 ticks.
    ///
    /// Light shadowcast radius 15 tiles → disc area π×15² ≈ 707 tiles. Octagonal
    /// shadowcast clips corners; threshold 450 = 64% of full-circle area accounts
    /// for clipping. This test runs in the SAME engine instance as B1-1 (Warmth)
    /// to confirm the Light channel is fully populated when Warmth co-exists.
    /// T7.10.B verifies Light in isolation; B1-2 asserts it survives concurrent
    /// channel presence. If Light has < 450 non-zero tiles the toggled overlay is
    /// visually empty — the toggle feature is broken regardless of GDScript state.
    #[test]
    fn harness_t7_10_b1_light_channel_non_trivial_concurrent() {
        let mut e = fresh_engine();
        place_source(&mut e);
        run_ticks(&mut e, 10);

        let light_buf = e
            .resources
            .influence_grid
            .current_buf(InfluenceChannel::Light);
        // Type A: threshold >= 450 non-zero positions
        let count = light_buf.iter().filter(|&&v| v > 0).count();
        assert!(
            count >= 450,
            "Light disc must have >= 450 non-zero tiles when Warmth co-exists \
         (shadowcast radius 15, disc area ~707, threshold=64% for octagonal clipping); \
         got {count}. T7.10.B regression: Light channel not surviving concurrent Warmth."
        );
        println!("[B1-2] Light non-zero tiles: {count} (threshold >= 450)");
    }

    // ── B1-3: Light disc extends beyond the Warmth disc ──────────────────────────

    /// Type A: annular zone tile count (Light substantially > Warmth) >= 150.
    ///
    /// Warmth radius ~12, Light radius 15. Annular zone between radii 12 and 15:
    /// π×(15²−12²) ≈ 255 tiles where Light > 0 and Warmth ≈ 0. Threshold 150 = 59%
    /// of the annular area, conservative for BFS boundary overlap. Margin = 1% of
    /// Light's global maximum (≈ 200 → margin ≈ 2). Any tile where
    /// Light − Warmth > margin is an "annular" tile.
    ///
    /// This is the sine qua non of the toggle's visual utility. If the Light disc
    /// does not extend beyond Warmth, pressing SPACE reveals nothing new.
    #[test]
    fn harness_t7_10_b1_light_disc_extends_beyond_warmth() {
        let mut e = fresh_engine();
        place_source(&mut e);
        run_ticks(&mut e, 10);

        let light_buf = e
            .resources
            .influence_grid
            .current_buf(InfluenceChannel::Light);
        let warmth_buf = e
            .resources
            .influence_grid
            .current_buf(InfluenceChannel::Warmth);

        let max1 = light_buf.iter().copied().max().unwrap_or(0);
        let margin_f = max1 as f64 * 0.01; // 1% of Light peak, spec = 200 → margin ≈ 2.0

        // Type A: threshold >= 150 positions where (Light - Warmth) > margin
        let count = light_buf
            .iter()
            .copied()
            .zip(warmth_buf.iter().copied())
            .filter(|&(l, w)| (l as f64 - w as f64) > margin_f)
            .count();

        assert!(
            count >= 150,
            "Light disc must extend >= 150 tiles beyond Warmth disc (annular zone r=12..15, \
         π×(225−144)≈255 tiles; threshold 59%); got {count}. \
         max_light={max1}, margin_1pct={margin_f:.2}. \
         If count < 150 the SPACE toggle reveals no new information."
        );
        println!("[B1-3] Annular zone (Light > Warmth + margin) tiles: {count} (threshold >= 150), margin={margin_f:.2}");
    }

    // ── B1-4: Light source tile holds the global channel maximum ─────────────────

    /// Type A: channel_1[32][32] / max(channel_1) >= 0.99.
    ///
    /// Spec: source intensity = 200, falloff 200/(1+0.1×d) strictly decreasing.
    /// At d=0 the value is exactly 200 — the unique global maximum. Ratio >= 0.99
    /// tolerates floating-point rounding while requiring source tile to be at peak.
    /// Failure means the source was not planted at (32,32) or some tile received
    /// a higher value — a wiring defect that would make the toggled view confusing.
    #[test]
    fn harness_t7_10_b1_light_source_tile_is_maximum() {
        let mut e = fresh_engine();
        place_source(&mut e);
        run_ticks(&mut e, 10);

        let light_buf = e
            .resources
            .influence_grid
            .current_buf(InfluenceChannel::Light);
        let max_val = light_buf.iter().copied().max().unwrap_or(0);
        let source_val = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);

        // Type A: threshold ratio >= 0.99
        // Integer form: source_val * 100 >= max_val * 99
        assert!(
            (source_val as u32) * 100 >= (max_val as u32) * 99,
            "Light source tile ({SX},{SY}) must be >= 99% of global max: \
         source={source_val}, global_max={max_val}, ratio={:.4}. \
         Failure means source not planted at (32,32) or falloff assigns higher \
         values to non-source tiles.",
            source_val as f64 / max_val.max(1) as f64
        );
        println!(
            "[B1-4] source={source_val}, max={max_val}, ratio={:.4} (threshold >= 0.99)",
            source_val as f64 / max_val.max(1) as f64
        );
    }

    // ── B1-6: GDScript toggle handler is present (Type S anti-circular guard) ────

    // ── B1-5: Light is zero beyond spec radius boundary ──────────────────────────

    /// Type A: max Light value at Euclidean distance > 16.0 <= 0.001 (u8: == 0).
    ///
    /// Spec: shadowcast radius = 15 tiles. Distance threshold 16.0 (one full tile
    /// past spec radius) eliminates boundary-cell ambiguity. Any value > 0 at d>16
    /// means the shadowcast propagates further than specified, making the Light
    /// disc visually larger than the T7.10.B design. u8 values are integers so
    /// the 0.001 threshold is operationally equivalent to == 0.
    #[test]
    fn harness_t7_10_b1_light_zero_beyond_radius() {
        let mut e = fresh_engine();
        place_source(&mut e);
        run_ticks(&mut e, 10);

        let w = e.resources.influence_grid.width;
        let h = e.resources.influence_grid.height;
        let mut max_beyond: u8 = 0;
        let mut worst_pos = (0u32, 0u32);

        for y in 0..h {
            for x in 0..w {
                let dx = x as f64 - SX as f64;
                let dy = y as f64 - SY as f64;
                if (dx * dx + dy * dy).sqrt() > 16.0 {
                    let v = e
                        .resources
                        .influence_grid
                        .sample(x, y, InfluenceChannel::Light);
                    if v > max_beyond {
                        max_beyond = v;
                        worst_pos = (x, y);
                    }
                }
            }
        }

        // Type A: threshold <= 0.001 (u8 integer must be 0)
        assert_eq!(
            max_beyond, 0,
            "max Light value at Euclidean distance > 16 must be 0 (spec radius=15); \
         got max={max_beyond} at tile {:?}. \
         Any nonzero value means shadowcast propagates past spec radius.",
            worst_pos
        );
        println!("[B1-5] max Light beyond d=16: {max_beyond} (threshold == 0)");
    }
}

// From `harness_t7_10_c_noise_linear_wiring.rs`.
mod noise {
    //! T7.10.C — Noise channel linear-decay wiring (third-channel Phase 2 escape).
    //!
    //! After T7.10.A (Warmth BFS) and T7.10.B (Light shadowcast), the Noise
    //! channel is the third to escape the dispatch shell. A BuildingPlacedEvent
    //! at (32, 32) with radius 12 must produce a Noise field in `current[Noise]`
    //! centered at (32, 32) after one tick, computed by linear-decay BFS with
    //! alpha=15 per step (Songs of Syx 2-tile ISSUE 2 v0.1.1 fix via
    //! `propagate_noise`, wraps `propagate_linear` with u32::MAX max_radius —
    //! natural radius via the `intensity < 5` cutoff).
    //!
    //! Setup note: 64×64 engine with empty MaterialRegistry (open field, no
    //! wall blocking → pure 4-neighbor linear decay). All thresholds are
    //! LOCKED from plan — do NOT modify them.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_c_noise_linear_wiring -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    fn place_noise_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    // ── Plan C1: source center reaches initial intensity ─────────────────────────

    /// Type A: influence_grid.sample(SX, SY, Noise) == 200 after 1 tick post-event.
    ///
    /// Physical invariant. The linear-decay source tile must receive full
    /// NOISE_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
    /// propagation (propagate.rs:358). Any value ≠ 200 means propagation never
    /// ran or the source seeding is broken.
    #[test]
    fn harness_t7_10_c_source_center_lit() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // Type A: threshold == 200
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            v, 200,
            "source center must equal NOISE_INITIAL_INTENSITY (200); got {v}"
        );
    }

    // ── Plan C2: linear decay one cardinal step (alpha=15 discriminator) ─────────

    /// Type A: influence_grid.sample(SX+1, SY, Noise) == 185 after 1 tick.
    ///
    /// Mathematical invariant. Linear decay alpha=15 per BFS step
    /// (propagate.rs:385: `intensity.saturating_sub(alpha)`). At BFS d=1:
    /// 200 - 15 = 185 exact. No tolerance — integer arithmetic, no f32.
    ///
    /// A shadowcast implementation (200/(1+0.1*1)=181) would FAIL this — proves
    /// linear-decay path is wired, not Light's falloff function.
    #[test]
    fn harness_t7_10_c_linear_one_step_alpha_discriminator() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // Type A: threshold == 185 (exact)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY, InfluenceChannel::Noise);
        assert_eq!(
            v, 185,
            "1-step cardinal neighbor must equal 200-15=185 (linear alpha=15); got {v}. \
         Value 181 = shadowcast falloff (wrong primitive). \
         Other value = wrong alpha or propagation never ran."
        );
    }

    // ── Plan C3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

    /// Type A: influence_grid.sample(SX+1, SY+1, Noise) == 170 after 1 tick.
    ///
    /// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
    /// via 4-neighbor BFS: 200 - 2*15 = 170. NO Euclidean shortcut — BFS uses
    /// Manhattan-like distance under 4-neighborhood expansion.
    ///
    /// A Euclidean implementation (200 - 15*sqrt(2) ≈ 178.8) would FAIL — proves
    /// linear-decay uses the BFS frontier, not Euclidean distance like Light.
    #[test]
    fn harness_t7_10_c_bfs_distance_manhattan_discriminator() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // Type A: threshold == 170 (200 - 2*15)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY + 1, InfluenceChannel::Noise);
        assert_eq!(
            v, 170,
            "diagonal (SX+1,SY+1) must equal 170 (BFS d=2, 200-2*15); got {v}. \
         Value ≈179 = Euclidean (wrong distance metric). \
         Other value = wrong alpha or BFS broken."
        );
    }

    // ── Plan C4: gradient monotonically strictly decreasing along cardinal axis ──

    /// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=11 (strict).
    ///
    /// Mathematical invariant. Linear decay alpha=15 yields constant 15-unit step
    /// drops: 200, 185, 170, ..., 20, 5. Adjacent pairs always differ by exactly
    /// 15 (no truncation ties) so strict `>` is safe across d=0..=11. Violation
    /// at any step means BFS frontier or alpha decrement is broken.
    #[test]
    fn harness_t7_10_c_gradient_monotone() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        for d in 0..=11u32 {
            let v1 = e
                .resources
                .influence_grid
                .sample(SX + d, SY, InfluenceChannel::Noise);
            let v2 = e
                .resources
                .influence_grid
                .sample(SX + d + 1, SY, InfluenceChannel::Noise);
            // Type A: threshold strict `>` for every adjacent pair
            assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Linear alpha=15 must yield exact 15-unit drops.",
            d + 1
        );
        }
    }

    // ── Plan C5: natural radius boundary (intensity<5 cutoff) ────────────────────

    /// Type A: sample(SX+13, SY, Noise) == 5 AND sample(SX+14, SY, Noise) == 0.
    ///
    /// Natural cutoff invariant. propagate_linear (propagate.rs:363) exits when
    /// `intensity < 5`. At BFS d=13: 200 - 13*15 = 5 → still queued (5 not < 5),
    /// so the tile is written with intensity=5. At d=14: decayed = 5-15 →
    /// saturating_sub = 0, `after_block > 0` is false → tile NOT written.
    ///
    /// Discriminates: a Generator who uses `<=` cutoff or wrong alpha would
    /// produce a different boundary tile.
    #[test]
    fn harness_t7_10_c_natural_radius_boundary() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // Type A: threshold == 5 at d=13
        let v13 = e
            .resources
            .influence_grid
            .sample(SX + 13, SY, InfluenceChannel::Noise);
        assert_eq!(
            v13, 5,
            "Noise at d=13 must equal 200-13*15=5 (intensity<5 cutoff still queues 5); got {v13}"
        );
        // Type A: threshold == 0 at d=14
        let v14 = e
            .resources
            .influence_grid
            .sample(SX + 14, SY, InfluenceChannel::Noise);
        assert_eq!(
        v14, 0,
        "Noise at d=14 must be 0 (decayed=saturating_sub(5,15)=0, after_block>0 false); got {v14}"
    );
    }

    // ── Plan C6: persistence — Noise survives 10 event-less ticks ────────────────

    /// Type A: sample(SX, SY, Noise) == 200 after the single event tick + 9 more
    /// event-less ticks (total 10 ticks).
    ///
    /// Persistence invariant (T7.10.C update.rs:217-254). On event-less ticks
    /// the Hot-tier branch copies `current[Noise]` into `pending[Noise]` so the
    /// swap is a no-op, preventing flicker. A Generator who omits persistence
    /// would see Noise zero out after tick 2.
    #[test]
    fn harness_t7_10_c_persistence_ten_ticks() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        for _ in 0..10 {
            e.tick();
        }
        // Type A: threshold == 200 at source after 10 ticks
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            v, 200,
            "source Noise must persist == 200 after 10 ticks (1 event + 9 event-less); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
        );
    }

    // ── Plan C7: no event = no noise ─────────────────────────────────────────────

    /// Type A: Noise == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
    ///
    /// Without any BuildingPlacedEvent, dirty_regions[Noise] stays empty every
    /// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
    /// Any nonzero value means uninitialized buffer state leaking through.
    #[test]
    fn harness_t7_10_c_no_event_no_noise() {
        let mut e = fresh_engine();
        for _ in 0..5 {
            e.tick();
        }
        for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
            // Type A: threshold == 0 at all three positions with no events
            let v = e
                .resources
                .influence_grid
                .sample(x, y, InfluenceChannel::Noise);
            assert_eq!(v, 0, "Noise at ({x},{y}) must be 0 with no events; got {v}");
        }
    }

    // ── Plan C8: T7.10.A/B regression — Warmth+Light survive Noise wiring ────────

    /// Type A: Warmth(SX,SY) == 200 AND Light(SX,SY) == 200 after 1 tick post-event.
    ///
    /// Cross-channel regression guard. T7.10.C adds the Noise branch to IUS;
    /// the Warmth (T7.10.A) and Light (T7.10.B) branches must still produce
    /// their respective discs. A Generator who accidentally re-orders or
    /// short-circuits Warmth/Light when adding Noise would FAIL this guard.
    #[test]
    fn harness_t7_10_c_warmth_and_light_regression_guard() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // Type A: threshold == 200 for Warmth source center
        let warmth = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            warmth, 200,
            "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
        );
        // Type A: threshold == 200 for Light source center
        let light = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            light, 200,
            "T7.10.B regression: Light at source must remain 200; got {light}"
        );
    }

    // ── Plan C9: dirty_regions[Noise] drained by IUS via std::mem::take ─────────

    /// Type A: dirty_regions[Noise].len() == 0 after one full engine.tick() with
    /// a BuildingPlacedEvent.
    ///
    /// IUS drain invariant. T7.10.C uses `std::mem::take` on the Noise dirty
    /// regions, matching T7.10.A/B semantics. A Generator who reads via
    /// `iter()` or `clone()` without draining would leave the entries in place.
    #[test]
    fn harness_t7_10_c_dirty_regions_drained() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // Type A: threshold == 0
        let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Noise as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[Noise] must be drained by IUS via std::mem::take; got len={len}"
        );
    }

    // ── Plan C10: other-channels behavior (Spiritual/Beauty stay 0, FoodAroma/Social stay 0)

    /// Type A: T7.10.D regression — Danger sample == 200 at source;
    /// dispatch-shell stamped channels Spiritual/Beauty sample to 0;
    /// unstamped channels FoodAroma/Social sample to 0.
    ///
    /// T7.10.C wired Warmth+Light+Noise. T7.10.D additionally wires Danger.
    /// The remaining stamped channels (Spiritual, Beauty) have BSS dirty_regions
    /// but IUS does not yet propagate them. Unstamped channels (FoodAroma, Social)
    /// stay completely cold.
    #[test]
    fn harness_t7_10_c_other_channels_remain_zero() {
        let mut e = fresh_engine();
        place_noise_source(&mut e);
        e.tick();
        // T7.10.D regression guard: Danger now propagates at source center.
        let danger = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            danger, 200,
            "T7.10.D: Danger at source must be 200 (linear-decay+cap propagation); got {danger}"
        );
        // T7.10.E regression guard: Spiritual now propagates at source center.
        let spiritual = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
        spiritual, 200,
        "T7.10.E: Spiritual at source must be 200 (BFS exp k=0.08 propagation); got {spiritual}"
    );
        // T7.10.F regression guard: Beauty now propagates at source center.
        let beauty = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            beauty, 200,
            "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
        );
        // Unstamped channels (BSS never marks dirty) — only 2 remain post-T7.10.F.
        for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
            let v = e.resources.influence_grid.sample(SX, SY, ch);
            assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
        }
    }
}

// From `harness_t7_10_d_danger_linear_wiring.rs`.
mod danger {
    //! T7.10.D — Danger channel linear-decay+cap wiring (fourth-channel Phase 2 escape).
    //!
    //! After T7.10.A (Warmth BFS), T7.10.B (Light shadowcast), and T7.10.C (Noise
    //! linear-decay), the Danger channel is the fourth to escape the dispatch
    //! shell. A BuildingPlacedEvent at (32, 32) with radius 12 must produce a
    //! Danger field in `current[Danger]` centered at (32, 32) after one tick,
    //! computed by linear-decay BFS with alpha=5 per step AND a hard sight-radius
    //! cap of 15 tiles (Phase 0 ISSUE 3 fix via `propagate_danger`, wraps
    //! `propagate_linear` with max_radius=15 and blocking_cache=None — Danger
    //! pierces walls).
    //!
    //! Setup note: 64×64 engine with empty MaterialRegistry (open field, no
    //! wall blocking would apply anyway since Danger ignores walls). All
    //! thresholds are LOCKED from plan — do NOT modify them.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_d_danger_linear_wiring -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    fn place_danger_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    // ── Plan D1: source center reaches initial intensity ─────────────────────────

    /// Type A: influence_grid.sample(SX, SY, Danger) == 200 after 1 tick post-event.
    ///
    /// Physical invariant. The linear-decay source tile must receive full
    /// DANGER_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
    /// propagation (propagate.rs:358). Any value ≠ 200 means propagation never
    /// ran or the source seeding is broken.
    #[test]
    fn harness_t7_10_d_source_center_lit() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // Type A: threshold == 200
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            v, 200,
            "source center must equal DANGER_INITIAL_INTENSITY (200); got {v}"
        );
    }

    // ── Plan D2: linear decay one cardinal step (alpha=5 discriminator) ──────────

    /// Type A: influence_grid.sample(SX+1, SY, Danger) == 195 after 1 tick.
    ///
    /// Mathematical invariant. Linear decay alpha=5 per BFS step (Phase 0
    /// ISSUE 3 fix locked in `propagate_danger`). At BFS d=1: 200 - 5 = 195 exact.
    /// No tolerance — integer arithmetic, no f32.
    ///
    /// Discriminates against Noise's alpha=15 (would give 185) and against
    /// shadowcast falloff 200/(1+0.1*1)=181. Proves linear-decay alpha=5 path
    /// is wired, not Noise's or Light's primitive.
    #[test]
    fn harness_t7_10_d_linear_one_step_alpha_discriminator() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // Type A: threshold == 195 (exact)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY, InfluenceChannel::Danger);
        assert_eq!(
            v, 195,
            "1-step cardinal neighbor must equal 200-5=195 (linear alpha=5); got {v}. \
         Value 185 = Noise alpha=15 (wrong primitive). \
         Value 181 = shadowcast falloff (wrong primitive). \
         Other value = wrong alpha or propagation never ran."
        );
    }

    // ── Plan D3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

    /// Type A: influence_grid.sample(SX+1, SY+1, Danger) == 190 after 1 tick.
    ///
    /// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
    /// via 4-neighbor BFS: 200 - 2*5 = 190. NO Euclidean shortcut — BFS uses
    /// Manhattan-like distance under 4-neighborhood expansion.
    ///
    /// Discriminates against Euclidean 200 - 5*sqrt(2) ≈ 192.9 (would round to
    /// 193). Proves linear-decay uses BFS frontier, not Euclidean distance.
    #[test]
    fn harness_t7_10_d_bfs_distance_manhattan_discriminator() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // Type A: threshold == 190 (200 - 2*5)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY + 1, InfluenceChannel::Danger);
        assert_eq!(
            v, 190,
            "diagonal (SX+1,SY+1) must equal 190 (BFS d=2, 200-2*5); got {v}. \
         Value ≈193 = Euclidean (wrong distance metric). \
         Other value = wrong alpha or BFS broken."
        );
    }

    // ── Plan D4: gradient monotonically strictly decreasing along cardinal axis ──

    /// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
    ///
    /// Mathematical invariant. Linear decay alpha=5 yields constant 5-unit step
    /// drops across the cap radius: 200, 195, 190, ..., 130, 125. Adjacent pairs
    /// always differ by exactly 5 (no truncation ties) so strict `>` is safe
    /// across d=0..=14. Violation at any step means BFS frontier or alpha
    /// decrement is broken.
    #[test]
    fn harness_t7_10_d_gradient_monotone() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        for d in 0..=14u32 {
            let v1 = e
                .resources
                .influence_grid
                .sample(SX + d, SY, InfluenceChannel::Danger);
            let v2 = e
                .resources
                .influence_grid
                .sample(SX + d + 1, SY, InfluenceChannel::Danger);
            // Type A: threshold strict `>` for every adjacent pair
            assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Linear alpha=5 must yield exact 5-unit drops up to the cap.",
            d + 1
        );
        }
    }

    // ── Plan D5: sight-radius cap boundary (Phase 0 ISSUE 3 fix discriminator) ───

    /// Type A: sample(SX+15, SY, Danger) == 125 AND sample(SX+16, SY, Danger) == 0.
    ///
    /// Cap invariant. propagate_danger passes max_radius=15 to propagate_linear.
    /// propagate.rs:380 caps BFS expansion at `dist >= max_radius` (does not
    /// queue further). At BFS d=15: 200 - 15*5 = 125 — the cap allows this tile
    /// to be written but blocks d=16. At d=16: NOT reached, tile remains 0.
    ///
    /// Discriminates: Noise (no cap) at d=16 would give 200-16*15=neg→saturating
    /// then NOT pass intensity<5 check at d=14 (200-14*15=−10→0), so Noise would
    /// be 0 at d=14 already. A Generator who omits the cap (uses propagate_linear
    /// with u32::MAX) would write d=16=120 instead of 0. A Generator who uses
    /// the wrong cap (e.g. 14 or 16) would shift the boundary by one tile.
    #[test]
    fn harness_t7_10_d_sight_radius_cap_boundary() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // Type A: threshold == 125 at d=15 (cap allows this tile)
        let v15 = e
            .resources
            .influence_grid
            .sample(SX + 15, SY, InfluenceChannel::Danger);
        assert_eq!(
            v15, 125,
            "Danger at d=15 must equal 200-15*5=125 (cap=15 allows this tile); got {v15}. \
         Value 0 = cap too tight (max_radius<15). \
         Value ≠125 = wrong alpha."
        );
        // Type A: threshold == 0 at d=16 (cap stops propagation)
        let v16 = e
            .resources
            .influence_grid
            .sample(SX + 16, SY, InfluenceChannel::Danger);
        assert_eq!(
            v16, 0,
            "Danger at d=16 must be 0 (cap=15 blocks expansion past d=15); got {v16}. \
         Value 120 = no cap (would be 200-16*5=120 without cap). \
         Other nonzero value = cap implemented but wrong threshold."
        );
    }

    // ── Plan D6: persistence — Danger survives 10 event-less ticks ───────────────

    /// Type A: sample(SX, SY, Danger) == 200 after the single event tick + 9 more
    /// event-less ticks (total 10 ticks).
    ///
    /// Persistence invariant (T7.10.D update.rs). On event-less ticks the
    /// Hot-tier branch copies `current[Danger]` into `pending[Danger]` so the
    /// swap is a no-op, preventing flicker. A Generator who omits persistence
    /// would see Danger zero out after tick 2.
    #[test]
    fn harness_t7_10_d_persistence_ten_ticks() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        for _ in 0..10 {
            e.tick();
        }
        // Type A: threshold == 200 at source after 10 ticks
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            v, 200,
            "source Danger must persist == 200 after 10 ticks (1 event + 9 event-less); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
        );
    }

    // ── Plan D7: no event = no danger ────────────────────────────────────────────

    /// Type A: Danger == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
    ///
    /// Without any BuildingPlacedEvent, dirty_regions[Danger] stays empty every
    /// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
    /// Any nonzero value means uninitialized buffer state leaking through.
    #[test]
    fn harness_t7_10_d_no_event_no_danger() {
        let mut e = fresh_engine();
        for _ in 0..5 {
            e.tick();
        }
        for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
            // Type A: threshold == 0 at all three positions with no events
            let v = e
                .resources
                .influence_grid
                .sample(x, y, InfluenceChannel::Danger);
            assert_eq!(
                v, 0,
                "Danger at ({x},{y}) must be 0 with no events; got {v}"
            );
        }
    }

    // ── Plan D8: T7.10.A/B/C regression — Warmth+Light+Noise survive Danger wiring

    /// Type A: Warmth(SX,SY) == 200 AND Light(SX,SY) == 200 AND Noise(SX,SY) == 200
    /// after 1 tick post-event.
    ///
    /// Cross-channel regression guard. T7.10.D adds the Danger branch to IUS;
    /// the Warmth (T7.10.A), Light (T7.10.B), and Noise (T7.10.C) branches must
    /// still produce their respective discs. A Generator who accidentally
    /// re-orders or short-circuits Warmth/Light/Noise when adding Danger would
    /// FAIL this guard.
    #[test]
    fn harness_t7_10_d_warmth_light_noise_regression_guard() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // Type A: threshold == 200 for Warmth source center
        let warmth = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            warmth, 200,
            "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
        );
        // Type A: threshold == 200 for Light source center
        let light = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            light, 200,
            "T7.10.B regression: Light at source must remain 200; got {light}"
        );
        // Type A: threshold == 200 for Noise source center
        let noise = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            noise, 200,
            "T7.10.C regression: Noise at source must remain 200; got {noise}"
        );
    }

    // ── Plan D9: dirty_regions[Danger] drained by IUS via std::mem::take ────────

    /// Type A: dirty_regions[Danger].len() == 0 after one full engine.tick() with
    /// a BuildingPlacedEvent.
    ///
    /// IUS drain invariant. T7.10.D uses `std::mem::take` on the Danger dirty
    /// regions, matching T7.10.A/B/C semantics. A Generator who reads via
    /// `iter()` or `clone()` without draining would leave the entries in place.
    #[test]
    fn harness_t7_10_d_dirty_regions_drained() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // Type A: threshold == 0
        let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Danger as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[Danger] must be drained by IUS via std::mem::take; got len={len}"
        );
    }

    // ── Plan D10: other-channels behavior (Beauty stays 0, FoodAroma/Social stay 0)

    /// Type A: T7.10.E regression — Spiritual sample == 200 at source;
    /// dispatch-shell stamped channel Beauty samples to 0;
    /// unstamped channels FoodAroma/Social sample to 0.
    ///
    /// T7.10.D wired Warmth+Light+Noise+Danger. T7.10.E additionally wires
    /// Spiritual via BFS exp k=0.08. The remaining stamped channel (Beauty) has
    /// BSS dirty_regions but IUS does not yet propagate it. Unstamped channels
    /// (FoodAroma, Social) stay completely cold.
    #[test]
    fn harness_t7_10_d_other_channels_remain_zero() {
        let mut e = fresh_engine();
        place_danger_source(&mut e);
        e.tick();
        // T7.10.E regression guard: Spiritual now propagates at source center.
        let spiritual = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
        spiritual, 200,
        "T7.10.E: Spiritual at source must be 200 (BFS exp k=0.08 propagation); got {spiritual}"
    );
        // T7.10.F regression guard: Beauty now propagates at source center.
        let beauty = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            beauty, 200,
            "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
        );
        // Unstamped channels (BSS never marks dirty) — only 2 remain post-T7.10.F.
        for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
            let v = e.resources.influence_grid.sample(SX, SY, ch);
            assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
        }
    }
}

// From `harness_t7_10_e_spiritual_bfs_wiring.rs`.
mod spiritual {
    //! T7.10.E — Spiritual channel BFS exponential-decay wiring (fifth-channel Phase 2 escape).
    //!
    //! After T7.10.A (Warmth BFS), T7.10.B (Light shadowcast), T7.10.C (Noise
    //! linear-decay), and T7.10.D (Danger linear-decay+cap), the Spiritual channel
    //! is the fifth to escape the dispatch shell. A BuildingPlacedEvent at (32, 32)
    //! with radius 12 must produce a Spiritual field in `current[Spiritual]`
    //! centered at (32, 32) after one tick, computed by BFS with exponential decay
    //! k=0.08 (channel.rs:69 Phase 0 spec — gentler than Warmth's k=0.15) and
    //! max_radius=15 (longer reach than Warmth's 12, matching the ritual-influence
    //! archetype).
    //!
    //! Setup note: 64×64 engine with empty MaterialRegistry (open field, no wall
    //! blocking). All thresholds are LOCKED from plan — do NOT modify them.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_e_spiritual_bfs_wiring -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    fn place_spiritual_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    // ── Plan E1: source center reaches initial intensity ─────────────────────────

    /// Type A: influence_grid.sample(SX, SY, Spiritual) == 200 after 1 tick post-event.
    ///
    /// Physical invariant. The BFS source tile must receive full
    /// SPIRITUAL_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
    /// propagation. Any value ≠ 200 means propagation never ran or the source
    /// seeding is broken.
    #[test]
    fn harness_t7_10_e_source_center_lit() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold == 200
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
            v, 200,
            "source center must equal SPIRITUAL_INITIAL_INTENSITY (200); got {v}"
        );
    }

    // ── Plan E2: exp decay one cardinal step (k=0.08 discriminator vs Warmth k=0.15)

    /// Type A: influence_grid.sample(SX+1, SY, Spiritual) ∈ [182, 186] after 1 tick.
    ///
    /// Mathematical invariant. Spiritual decay k=0.08 (channel.rs:69 Phase 0 spec),
    /// SPIRITUAL_DECAY_PER_STEP = exp(-0.08) ≈ 0.923116. Chain step:
    /// floor(200.0 * 0.923116) = 184. Range [182, 186] adds ±2 tolerance for f32
    /// rounding.
    ///
    /// Discriminates against Warmth's k=0.15 (would give floor(200*0.8607)=172 —
    /// OUTSIDE [182,186]) and the prior k=0.10 drift (would give floor(200*0.9048)
    /// =180 — OUTSIDE [182,186]). Also discriminates against Light's shadowcast
    /// falloff 200/(1+0.1)=181 — Spiritual uses BFS (Manhattan distance) not
    /// Euclidean, so Plan E3 nails the remaining difference.
    #[test]
    fn harness_t7_10_e_exp_decay_one_step_discriminator() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [182, 186]
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY, InfluenceChannel::Spiritual);
        assert!(
            (182..=186).contains(&v),
            "1-step cardinal neighbor must decay to ~184 (200*exp(-0.08)); got {v}. \
         Value ~180 = stale k=0.10 (T7.10.E pre-fix drift). \
         Value ~172 = Warmth k=0.15 (wrong decay constant). \
         Below 182 or above 186 = wrong primitive or no decay applied."
        );
    }

    // ── Plan E3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

    /// Type A: influence_grid.sample(SX+1, SY+1, Spiritual) ∈ [168, 172] after 1 tick.
    ///
    /// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
    /// via 4-neighbor BFS: chain through (SX+1, SY) → (SX+1, SY+1).
    /// 200 * 0.923116 ≈ 184.6 → 184.6 * 0.923116 ≈ 170.5. Range [168, 172] gives
    /// ±2 tolerance for accumulated f32 truncation across the two-step chain.
    ///
    /// Discriminates against Euclidean (Light) at d=sqrt(2): 200/(1+0.1*1.414) ≈
    /// 175 — OUTSIDE [168,172]. Proves BFS frontier, not Euclidean distance. Also
    /// discriminates against stale k=0.10 chain ≈ 163 — OUTSIDE [168,172].
    #[test]
    fn harness_t7_10_e_bfs_distance_manhattan_discriminator() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [168, 172] (two-step chain ≈170)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY + 1, InfluenceChannel::Spiritual);
        assert!(
            (168..=172).contains(&v),
            "diagonal (SX+1,SY+1) must be in [168,172] (BFS d=2 chain ≈170 with k=0.08); got {v}. \
         Value ~163 = stale k=0.10 (T7.10.E pre-fix drift). \
         Value ~175 = Euclidean (wrong distance metric). \
         Outside [168,172] = wrong decay or BFS broken."
        );
    }

    // ── Plan E4: gradient monotonically strictly decreasing along cardinal axis ──

    /// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
    ///
    /// Mathematical invariant. Exponential decay with k=0.08 > 0 is strictly
    /// monotone decreasing in continuous form. After u8 truncation, the minimum
    /// inter-step gap (multiplicative chain by 0.923116) remains ≥ 1 across
    /// d=0..=14 (the smallest expected sample near d=15 is ~200*0.9231^15 ≈ 60,
    /// still gives ~5-unit drops). Strict `>` is safe for every adjacent pair.
    /// Violation at any step means BFS frontier or decay function is broken.
    #[test]
    fn harness_t7_10_e_gradient_monotone() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        for d in 0..=14u32 {
            let v1 = e
                .resources
                .influence_grid
                .sample(SX + d, SY, InfluenceChannel::Spiritual);
            let v2 = e
                .resources
                .influence_grid
                .sample(SX + d + 1, SY, InfluenceChannel::Spiritual);
            // Type A: threshold strict `>` for every adjacent pair
            assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Spiritual k=0.08 must yield strictly decreasing chain across d=0..=14.",
            d + 1
        );
        }
    }

    // ── Plan E5: boundary tile at max_radius is nonzero ──────────────────────────

    /// Type A: influence_grid.sample(SX+15, SY, Spiritual) ∈ [55, 65] after 1 tick.
    ///
    /// Mathematical invariant. The BFS must reach max_radius=15 along the cardinal
    /// axis. Chain end at d=15: 200 * 0.9231^15 ≈ 200 * 0.3012 ≈ 60.2. After
    /// u8 truncation across 15 chained f32 multiplications: range [55, 65] gives
    /// ±5 margin for accumulated rounding.
    ///
    /// Value == 0 means BFS stopped before max_radius (wrong max_radius constant).
    /// Value ~45 indicates stale k=0.10 (T7.10.E pre-fix drift; 200*0.9048^15≈44.6).
    /// Value <55 may indicate Warmth's k=0.15 (would give ≈21) or stale k=0.10.
    #[test]
    fn harness_t7_10_e_boundary_at_max_radius() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [55, 65]
        let v = e
            .resources
            .influence_grid
            .sample(SX + 15, SY, InfluenceChannel::Spiritual);
        assert!(
            (55..=65).contains(&v),
            "tile at max_radius=15 must be in [55,65] (chain end ≈60 with k=0.08); got {v}; \
         value==0 means propagation stopped before max_radius=15. \
         Value ~45 = stale k=0.10 (T7.10.E pre-fix drift). \
         Value <55 may indicate Warmth's k=0.15 or stale k=0.10."
        );
    }

    // ── Plan E6: beyond max_radius is zero ───────────────────────────────────────

    /// Type A: influence_grid.sample(SX+16, SY, Spiritual) == 0 after 1 tick.
    ///
    /// Hard cutoff invariant. max_radius=15 is enforced by BFS termination
    /// (propagate_bfs at propagate.rs:75 uses `dist >= max_radius` cap). Tile at
    /// Manhattan distance 16 must NEVER be written.
    #[test]
    fn harness_t7_10_e_max_radius_cutoff() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold == 0 (distance 16 from source, one beyond max_radius=15)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 16, SY, InfluenceChannel::Spiritual);
        assert_eq!(v, 0, "tile beyond max_radius=15 must be 0; got {v}");
    }

    // ── Plan E7: cold-tier persistence across 10 event-less ticks ────────────────

    /// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
    ///
    /// Cold-tier event-driven semantics. Spiritual is UpdateTier::Cold — it must
    /// persist indefinitely between placement events. If the persistence branch
    /// (copy current→pending before swap) is missing, the disc flickers to 0
    /// on every event-less tick.
    #[test]
    fn harness_t7_10_e_persistence_ten_ticks() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick(); // tick 1: stamp + propagation
        for _ in 0..10 {
            e.tick(); // ticks 2–11: event-less persistence branch
        }
        // Type A: threshold == 200 (source persists through all 10 idle ticks)
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
            v, 200,
            "source must persist across 10 event-less ticks (Cold-tier semantics); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
        );
    }

    // ── Plan E8: no event = no spiritual ─────────────────────────────────────────

    /// Type A: Spiritual == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
    ///
    /// Without any BuildingPlacedEvent, dirty_regions[Spiritual] stays empty every
    /// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
    /// Any nonzero value means uninitialized buffer state leaking through.
    #[test]
    fn harness_t7_10_e_no_event_no_spiritual() {
        let mut e = fresh_engine();
        for _ in 0..5 {
            e.tick();
        }
        for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
            // Type A: threshold == 0 at all three positions with no events
            let v = e
                .resources
                .influence_grid
                .sample(x, y, InfluenceChannel::Spiritual);
            assert_eq!(
                v, 0,
                "Spiritual at ({x},{y}) must be 0 with no events; got {v}"
            );
        }
    }

    // ── Plan E9: T7.10.A/B/C/D regression — prior channels survive Spiritual wiring

    /// Type A: Warmth/Light/Noise/Danger all == 200 at source after 1 tick post-event.
    ///
    /// Cross-channel regression guard. T7.10.E adds the Spiritual branch to IUS;
    /// the Warmth (T7.10.A), Light (T7.10.B), Noise (T7.10.C), and Danger (T7.10.D)
    /// branches must still produce their respective discs. A Generator who
    /// accidentally re-orders or short-circuits prior channels when adding
    /// Spiritual would FAIL this guard.
    #[test]
    fn harness_t7_10_e_prior_channels_regression_guard() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold == 200 for Warmth source center
        let warmth = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            warmth, 200,
            "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
        );
        // Type A: threshold == 200 for Light source center
        let light = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            light, 200,
            "T7.10.B regression: Light at source must remain 200; got {light}"
        );
        // Type A: threshold == 200 for Noise source center
        let noise = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            noise, 200,
            "T7.10.C regression: Noise at source must remain 200; got {noise}"
        );
        // Type A: threshold == 200 for Danger source center
        let danger = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            danger, 200,
            "T7.10.D regression: Danger at source must remain 200; got {danger}"
        );
    }

    // ── Plan E10: dirty_regions[Spiritual] drained by IUS via std::mem::take ─────

    /// Type A: dirty_regions[Spiritual].len() == 0 after one full engine.tick() with
    /// a BuildingPlacedEvent.
    ///
    /// IUS drain invariant. T7.10.E uses `std::mem::take` on the Spiritual dirty
    /// regions, matching T7.10.A/B/C/D semantics. A Generator who reads via
    /// `iter()` or `clone()` without draining would leave the entries in place.
    #[test]
    fn harness_t7_10_e_dirty_regions_drained() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // Type A: threshold == 0
        let len =
            e.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[Spiritual] must be drained by IUS via std::mem::take; got len={len}"
        );
    }

    // ── Plan E11: other-channels behavior (Beauty propagates post-T7.10.F, FoodAroma/Social stay 0)

    /// Type A: Beauty propagates at source (T7.10.F regression guard); unstamped
    /// channels FoodAroma/Social sample to 0.
    #[test]
    fn harness_t7_10_e_other_channels_remain_zero() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        // T7.10.F regression guard: Beauty now propagates at source center.
        let beauty = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            beauty, 200,
            "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
        );
        // Unstamped channels (BSS never marks dirty).
        for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
            let v = e.resources.influence_grid.sample(SX, SY, ch);
            assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
        }
    }

    // ── Plan E12: SPIRITUAL_DECAY_PER_STEP literal must match exp(-0.08) spec ─────

    // ── Plan E13: Spiritual-vs-Warmth ratio at d=10 (k=0.08 vs k=0.15 discriminator)

    /// Type A: ratio `Spiritual(d=10) / Warmth(d=10)` ∈ [1.9, 2.2] after 1 tick.
    ///
    /// Cross-channel decay-rate invariant. At d=10 from a shared source:
    /// - Spiritual k=0.08 chain (200 * 0.923116^10 with u8 truncation) ≈ 86
    /// - Warmth k=0.15 chain (200 * 0.860708^10 with u8 truncation) ≈ 42
    /// - Canonical ratio ≈ 86 / 42 ≈ 2.05.
    ///
    /// Stale k=0.10 chain would give Spiritual(d=10) ≈ 69 → ratio ≈ 1.64 — OUTSIDE
    /// [1.9, 2.2]. The window deliberately excludes the pre-fix drift while
    /// permitting f32 rounding around the canonical 2.05. Warmth's max_radius=12
    /// covers d=10 so it propagates here.
    #[test]
    fn harness_t7_10_e_spiritual_warmth_ratio_at_d10() {
        let mut e = fresh_engine();
        place_spiritual_source(&mut e);
        e.tick();
        let spiritual = e
            .resources
            .influence_grid
            .sample(SX + 10, SY, InfluenceChannel::Spiritual);
        let warmth = e
            .resources
            .influence_grid
            .sample(SX + 10, SY, InfluenceChannel::Warmth);

        // Type A: Warmth must propagate to d=10 (within Warmth's max_radius=12).
        assert!(
            warmth > 0,
            "Warmth must propagate to d=10 (within max_radius=12); got {warmth}. \
         If 0, the shared BuildingPlacedEvent did not stamp Warmth."
        );

        // Type A: ratio threshold ∈ [1.9, 2.2] (k=0.08 vs k=0.15 expected ≈2.05).
        let ratio = f32::from(spiritual) / f32::from(warmth);
        assert!(
            (1.9..=2.2).contains(&ratio),
            "Spiritual(d=10)/Warmth(d=10) ratio must be ∈ [1.9, 2.2] (k=0.08 vs k=0.15); \
         got ratio = {ratio:.3} (spiritual={spiritual}, warmth={warmth}). \
         Ratio ~1.64 = stale Spiritual k=0.10 drift. \
         Ratio outside [1.9, 2.2] = wrong decay constant on Spiritual or Warmth side."
        );
    }

    // ── Plan E14: stale-constant regression guard (scan production + renderer) ───
}

// From `harness_t7_10_f_beauty_bfs_wiring.rs`.
mod beauty {
    //! T7.10.F — Beauty channel BFS exponential-decay wiring (sixth-channel Phase 2 escape).
    //!
    //! T7.10.F is the SIXTH and FINAL stamped channel to escape the dispatch shell.
    //! After T7.10.A (Warmth BFS k=0.15), T7.10.B (Light shadowcast), T7.10.C
    //! (Noise linear-decay), T7.10.D (Danger linear-decay+cap), and T7.10.E
    //! (Spiritual BFS k=0.08), the Beauty channel completes the 6/6 stamped-channel
    //! wiring milestone. A BuildingPlacedEvent at (32, 32) with radius 12 must
    //! produce a Beauty field in `current[Beauty]` centered at (32, 32) after one
    //! tick, computed by BFS with exponential decay k=0.12 (Phase 0 channel.rs:74
    //! lock — between Warmth's 0.15 and Spiritual's 0.08) and max_radius=15
    //! (Spiritual parity, Cold-tier reach archetype).
    //!
    //! Setup note: 64×64 engine with empty MaterialRegistry (open field, no wall
    //! blocking). All thresholds are LOCKED from plan — do NOT modify them.
    //!
    //! Run: `cargo test -p sim-test --test harness_t7_10_f_beauty_bfs_wiring -- --nocapture`

    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_phase2_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    const SX: u32 = 32;
    const SY: u32 = 32;

    fn fresh_engine() -> SimEngine {
        let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut engine);
        engine
    }

    fn place_beauty_source(engine: &mut SimEngine) {
        engine
            .resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (SX, SY),
                radius: 12,
            });
    }

    // ── Plan F1: source center reaches initial intensity ─────────────────────────

    /// Type A: influence_grid.sample(SX, SY, Beauty) == 200 after 1 tick post-event.
    ///
    /// Physical invariant. The BFS source tile must receive full
    /// BEAUTY_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
    /// propagation. Any value ≠ 200 means propagation never ran or the source
    /// seeding is broken.
    #[test]
    fn harness_t7_10_f_source_center_lit() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold == 200
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            v, 200,
            "source center must equal BEAUTY_INITIAL_INTENSITY (200); got {v}"
        );
    }

    // ── Plan F2: exp decay one cardinal step (k=0.12 discriminator) ──────────────

    /// Type A: influence_grid.sample(SX+1, SY, Beauty) ∈ [175, 180] after 1 tick.
    ///
    /// Mathematical invariant. Beauty decay k=0.12, BEAUTY_DECAY_PER_STEP =
    /// exp(-0.12) ≈ 0.886920. Chain step: floor(200.0 * 0.886920) = 177.
    /// Range [175, 180] adds ±3 tolerance for f32 rounding.
    ///
    /// Discriminates against:
    ///   - Warmth k=0.15 → floor(200*0.8607)=172 — OUTSIDE [175,180]
    ///   - Spiritual k=0.08 → floor(200*0.9231)=184 — OUTSIDE [175,180]; F3
    ///     confirms via Manhattan d=2 chain ≈157 (vs Spiritual 170).
    #[test]
    fn harness_t7_10_f_exp_decay_one_step_discriminator() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [175, 180]
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY, InfluenceChannel::Beauty);
        assert!(
            (175..=180).contains(&v),
            "1-step cardinal neighbor must decay to ~177 (200*exp(-0.12)); got {v}. \
         Value ~172 = Warmth k=0.15 (wrong decay constant). \
         Value ~184 with d=2 chain ≈170 = Spiritual k=0.08 mis-wiring. \
         Below 175 or above 180 = wrong primitive or no decay applied."
        );
    }

    // ── Plan F3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

    /// Type A: influence_grid.sample(SX+1, SY+1, Beauty) ∈ [155, 160] after 1 tick.
    ///
    /// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
    /// via 4-neighbor BFS: chain through (SX+1, SY) → (SX+1, SY+1).
    /// 200 * 0.886920 = 177 → 177 * 0.886920 ≈ 157. Range [155, 160] gives ±3
    /// tolerance for accumulated f32 truncation across the two-step chain.
    ///
    /// Discriminates against:
    ///   - Euclidean (Light shadowcast) at d=sqrt(2): 200/(1+0.1*1.414) ≈ 175 —
    ///     OUTSIDE [155,160]. Proves BFS frontier.
    ///   - Spiritual k=0.08 two-step chain ≈ 170 — OUTSIDE [155,160]. Proves
    ///     Beauty's k=0.12 constant.
    #[test]
    fn harness_t7_10_f_bfs_distance_manhattan_discriminator() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [155, 160] (two-step chain ≈157)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 1, SY + 1, InfluenceChannel::Beauty);
        assert!(
            (155..=160).contains(&v),
            "diagonal (SX+1,SY+1) must be in [155,160] (BFS d=2 chain ≈157); got {v}. \
         Value ~175 = Euclidean (wrong distance metric). \
         Value ~170 = Spiritual k=0.08 (wrong decay constant). \
         Outside [155,160] = wrong decay or BFS broken."
        );
    }

    // ── Plan F4: gradient monotonically strictly decreasing along cardinal axis ──

    /// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
    ///
    /// Mathematical invariant. Exponential decay with k=0.12 > 0 is strictly
    /// monotone decreasing in continuous form. After u8 truncation, the minimum
    /// inter-step gap (multiplicative chain by 0.886920) remains ≥ 1 across
    /// d=0..=14 (the smallest expected sample near d=15 is ~200*0.8869^15 ≈ 33,
    /// still gives ~4-unit drops). Strict `>` is safe for every adjacent pair.
    /// Violation at any step means BFS frontier or decay function is broken.
    #[test]
    fn harness_t7_10_f_gradient_monotone() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        for d in 0..=14u32 {
            let v1 = e
                .resources
                .influence_grid
                .sample(SX + d, SY, InfluenceChannel::Beauty);
            let v2 = e
                .resources
                .influence_grid
                .sample(SX + d + 1, SY, InfluenceChannel::Beauty);
            // Type A: threshold strict `>` for every adjacent pair
            assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Beauty k=0.12 must yield strictly decreasing chain across d=0..=14.",
            d + 1
        );
        }
    }

    // ── Plan F5: boundary tile at max_radius is nonzero ──────────────────────────

    /// Type A: influence_grid.sample(SX+15, SY, Beauty) ∈ [28, 38] after 1 tick.
    ///
    /// Mathematical invariant. The BFS must reach max_radius=15 along the cardinal
    /// axis. Chain end at d=15: 200 * 0.8869^15 ≈ 200 * 0.1653 ≈ 33.1. After
    /// u8 truncation across 15 chained f32 multiplications: range [28, 38] gives
    /// ±5 margin for accumulated rounding.
    ///
    /// Value == 0 means BFS stopped before max_radius (wrong max_radius constant).
    /// Value > 38 means wrong decay constant (Spiritual's k=0.08 would give ≈60 —
    /// OUTSIDE [28,38]).
    #[test]
    fn harness_t7_10_f_boundary_at_max_radius() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold ∈ [28, 38]
        let v = e
            .resources
            .influence_grid
            .sample(SX + 15, SY, InfluenceChannel::Beauty);
        assert!(
            (28..=38).contains(&v),
            "tile at max_radius=15 must be in [28,38] (chain end ≈33); got {v}; \
         value==0 means propagation stopped before max_radius=15. \
         Value ~60 = Spiritual k=0.08 (wrong decay constant). \
         Value ~21 = Warmth k=0.15 (wrong decay constant)."
        );
    }

    // ── Plan F6: beyond max_radius is zero ───────────────────────────────────────

    /// Type A: influence_grid.sample(SX+16, SY, Beauty) == 0 after 1 tick.
    ///
    /// Hard cutoff invariant. max_radius=15 is enforced by BFS termination
    /// (propagate_bfs at propagate.rs:75 uses `dist >= max_radius` cap). Tile at
    /// Manhattan distance 16 must NEVER be written.
    #[test]
    fn harness_t7_10_f_max_radius_cutoff() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold == 0 (distance 16 from source, one beyond max_radius=15)
        let v = e
            .resources
            .influence_grid
            .sample(SX + 16, SY, InfluenceChannel::Beauty);
        assert_eq!(v, 0, "tile beyond max_radius=15 must be 0; got {v}");
    }

    // ── Plan F7: cold-tier persistence across 10 event-less ticks ────────────────

    /// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
    ///
    /// Cold-tier event-driven semantics. Beauty is UpdateTier::Cold — it must
    /// persist indefinitely between placement events. If the persistence branch
    /// (copy current→pending before swap) is missing, the disc flickers to 0
    /// on every event-less tick.
    #[test]
    fn harness_t7_10_f_persistence_ten_ticks() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick(); // tick 1: stamp + propagation
        for _ in 0..10 {
            e.tick(); // ticks 2–11: event-less persistence branch
        }
        // Type A: threshold == 200 (source persists through all 10 idle ticks)
        let v = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Beauty);
        assert_eq!(
            v, 200,
            "source must persist across 10 event-less ticks (Cold-tier semantics); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
        );
    }

    // ── Plan F8: no event = no beauty ────────────────────────────────────────────

    /// Type A: Beauty == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
    ///
    /// Without any BuildingPlacedEvent, dirty_regions[Beauty] stays empty every
    /// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
    /// Any nonzero value means uninitialized buffer state leaking through.
    #[test]
    fn harness_t7_10_f_no_event_no_beauty() {
        let mut e = fresh_engine();
        for _ in 0..5 {
            e.tick();
        }
        for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
            // Type A: threshold == 0 at all three positions with no events
            let v = e
                .resources
                .influence_grid
                .sample(x, y, InfluenceChannel::Beauty);
            assert_eq!(
                v, 0,
                "Beauty at ({x},{y}) must be 0 with no events; got {v}"
            );
        }
    }

    // ── Plan F9: T7.10.A/B/C/D/E regression — prior channels survive Beauty wiring

    /// Type A: Warmth/Light/Noise/Danger/Spiritual all == 200 at source after 1 tick.
    ///
    /// Cross-channel regression guard. T7.10.F adds the Beauty branch to IUS;
    /// the Warmth (T7.10.A), Light (T7.10.B), Noise (T7.10.C), Danger (T7.10.D),
    /// and Spiritual (T7.10.E) branches must still produce their respective
    /// discs. A Generator who accidentally re-orders or short-circuits prior
    /// channels when adding Beauty would FAIL this guard.
    #[test]
    fn harness_t7_10_f_prior_channels_regression_guard() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold == 200 for all 5 previously-wired channels' source centers
        let warmth = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Warmth);
        assert_eq!(
            warmth, 200,
            "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
        );
        let light = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Light);
        assert_eq!(
            light, 200,
            "T7.10.B regression: Light at source must remain 200; got {light}"
        );
        let noise = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Noise);
        assert_eq!(
            noise, 200,
            "T7.10.C regression: Noise at source must remain 200; got {noise}"
        );
        let danger = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Danger);
        assert_eq!(
            danger, 200,
            "T7.10.D regression: Danger at source must remain 200; got {danger}"
        );
        let spiritual = e
            .resources
            .influence_grid
            .sample(SX, SY, InfluenceChannel::Spiritual);
        assert_eq!(
            spiritual, 200,
            "T7.10.E regression: Spiritual at source must remain 200; got {spiritual}"
        );
    }

    // ── Plan F10: dirty_regions[Beauty] drained by IUS via std::mem::take ────────

    /// Type A: dirty_regions[Beauty].len() == 0 after one full engine.tick() with
    /// a BuildingPlacedEvent.
    ///
    /// IUS drain invariant. T7.10.F uses `std::mem::take` on the Beauty dirty
    /// regions, matching T7.10.A/B/C/D/E semantics. A Generator who reads via
    /// `iter()` or `clone()` without draining would leave the entries in place.
    #[test]
    fn harness_t7_10_f_dirty_regions_drained() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Type A: threshold == 0
        let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Beauty as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[Beauty] must be drained by IUS via std::mem::take; got len={len}"
        );
    }

    // ── Plan F11: unstamped channels (FoodAroma/Social) stay zero ────────────────

    /// Type A: FoodAroma and Social sample to 0 at source after 1 tick.
    ///
    /// T7.10.F wires the 6th and final stamped channel (Beauty). The remaining
    /// 2 unstamped channels (FoodAroma, Social) have NO BSS dirty marking,
    /// so they stay completely cold. This is the final-state invariant: post-
    /// T7.10.F there are NO stamped-but-dispatch-shell channels — only the
    /// 2 unstamped channels remain dispatch-shell.
    #[test]
    fn harness_t7_10_f_unstamped_channels_remain_zero() {
        let mut e = fresh_engine();
        place_beauty_source(&mut e);
        e.tick();
        // Unstamped channels (BSS never marks dirty).
        for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
            let v = e.resources.influence_grid.sample(SX, SY, ch);
            assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
        }
    }
}
