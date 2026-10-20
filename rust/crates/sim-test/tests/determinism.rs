//! Merged in V8-0 (8.4) from `harness_phase2_determinism.rs`,
//! `harness_a15_controlled_determinism.rs`, `harness_fix_d_memory_arm.rs`; each
//! source is one module below, unchanged.

// From `harness_phase2_determinism.rs`.
mod influence_primitives {
    //! NEXT-C Stage 1 — Phase 2 determinism harness (R1 pivot, 13 tests).
    //!
    //! Engine-level current surface (A-group, 5 tests) + sim-core primitive-level
    //! direct paths (B-group, 8 tests). Verifies byte-identical behaviour across
    //! N=5 fresh instances given identical inputs.
    //!
    //! Why R1: end-to-end propagation is NOT wired in Phase 2. BuildingStampSystem
    //! writes only to `dirty_regions`, InfluenceUpdateSystem is a clear+swap shell,
    //! and BFS/shadowcast/linear primitives are never called from `engine.tick()`.
    //! The v1 D1-D5 "engine.tick → influence current_buf bytes" plan was therefore
    //! vacuous and withdrawn. R1 tests pin (a) what the engine actually mutates
    //! today (queue drain, dirty_regions, clear+swap, viz digest at tick%6==0) and
    //! (b) the primitives that will be wired into IUS in Phase 3+.
    //!
    //! Run via: `cargo test -p sim-test --test harness_phase2_determinism -- --nocapture`

    use sim_core::influence::propagate::{
        propagate_bfs, propagate_danger, propagate_noise, propagate_shadowcast,
        stamp_social_aggregate, LodTier,
    };
    use sim_core::influence::{DirtyRegion, InfluenceChannel, MaterialBlockingCache};
    use sim_core::material::{MaterialId, MaterialRegistry};
    use sim_core::tile::TileGrid;
    use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
    use sim_systems::runtime::influence::visualization::VisualizationDigest;
    use sim_systems::runtime::influence::{
        BuildingStampSystem, InfluenceUpdateSystem, InfluenceVisualizationSystem,
    };

    const N: usize = 5;
    const W: u32 = 32;
    const H: u32 = 32;

    type RegionTuple = (u32, u32, u32, u32);
    type ChannelRegions = Vec<RegionTuple>;

    fn make_engine() -> SimEngine {
        SimEngine::new(W, H, MaterialRegistry::new())
    }

    fn region_tuple(r: &DirtyRegion) -> RegionTuple {
        (r.min_x, r.min_y, r.max_x, r.max_y)
    }

    // ─── A-group: engine-level current surface (5 tests) ─────────────────────────

    /// D1' — Drain order: 4 events queued in known order produce 4 DirtyRegion
    /// entries on every STAMPED channel in matching FIFO order across N instances.
    #[test]
    fn harness_det_a1_queue_fifo_drain_order() {
        let events = [
            BuildingPlacedEvent {
                position: (5, 5),
                radius: 1,
            },
            BuildingPlacedEvent {
                position: (10, 10),
                radius: 2,
            },
            BuildingPlacedEvent {
                position: (20, 15),
                radius: 3,
            },
            BuildingPlacedEvent {
                position: (25, 25),
                radius: 1,
            },
        ];

        // Type A invariant — exact Warmth dirty-region tuple list (plan threshold, locked).
        // Formula: (cx±r) clamped to [0, W-1]. W=32, H=32.
        //   (5,5,r=1) → (4,4,6,6)   (10,10,r=2) → (8,8,12,12)
        //   (20,15,r=3) → (17,12,23,18)  (25,25,r=1) → (24,24,26,26)
        const EXPECTED: &[RegionTuple] = &[
            (4, 4, 6, 6),
            (8, 8, 12, 12),
            (17, 12, 23, 18),
            (24, 24, 26, 26),
        ];

        let mut runs: Vec<Vec<(u32, u32, u32, u32)>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut e = make_engine();
            let mut bss = BuildingStampSystem::new();
            for ev in events.iter() {
                e.resources.building_event_queue.push_back(*ev);
            }
            bss.tick(&mut e.world, &mut e.resources);

            let regs = &e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
            runs.push(regs.iter().map(region_tuple).collect());
        }

        // Type A: exact tuple list matches plan threshold.
        assert_eq!(
            runs[0], EXPECTED,
            "Warmth FIFO order must match exact plan threshold"
        );
        for i in 1..N {
            assert_eq!(
                runs[0], runs[i],
                "Warmth FIFO drain order diverged between instance 0 and {i}"
            );
        }
    }

    /// D2' — Dirty region content: all 4 STAMPED channels receive the same
    /// DirtyRegion tuples (positions + clamped bounds) consistently across runs.
    #[test]
    fn harness_det_a2_dirty_regions_content_byte_identical() {
        let events = [
            BuildingPlacedEvent {
                position: (3, 3),
                radius: 2,
            },
            BuildingPlacedEvent {
                position: (16, 16),
                radius: 4,
            },
            BuildingPlacedEvent {
                position: (28, 28),
                radius: 5,
            },
        ];
        const CHANNELS: &[InfluenceChannel] = &[
            InfluenceChannel::Warmth,
            InfluenceChannel::Spiritual,
            InfluenceChannel::Beauty,
            InfluenceChannel::Light,
        ];

        let mut runs: Vec<Vec<ChannelRegions>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut e = make_engine();
            let mut bss = BuildingStampSystem::new();
            for ev in events.iter() {
                e.resources.building_event_queue.push_back(*ev);
            }
            bss.tick(&mut e.world, &mut e.resources);

            let mut per_channel = Vec::with_capacity(CHANNELS.len());
            for ch in CHANNELS {
                per_channel.push(
                    e.resources.influence_grid.dirty_regions[*ch as usize]
                        .iter()
                        .map(region_tuple)
                        .collect::<Vec<_>>(),
                );
            }
            runs.push(per_channel);
        }

        // Intra-run invariant: BSS stamps all 4 STAMPED_CHANNELS identically,
        // so each channel's dirty-region list must equal every other channel's
        // list within the same run instance.
        for ch_idx in 0..CHANNELS.len() {
            assert_eq!(runs[0][ch_idx].len(), 3);
            assert_eq!(
                runs[0][0], runs[0][ch_idx],
                "channel {:?} dirty-region content differs from Warmth within run 0 \
             (BSS must stamp all 4 channels identically)",
                CHANNELS[ch_idx]
            );
        }

        // Cross-run invariant: every instance produces byte-identical results.
        for ch_idx in 0..CHANNELS.len() {
            for i in 1..N {
                assert_eq!(
                    runs[0][ch_idx], runs[i][ch_idx],
                    "channel {:?} diverged between instance 0 and {i}",
                    CHANNELS[ch_idx]
                );
            }
        }
    }

    /// D3' — Clamp + OOB skip: mixed valid / OOB / huge-radius events produce
    /// deterministic queue drain (always empty) and DirtyRegion counts.
    #[test]
    fn harness_det_a3_bss_clamp_oob_deterministic() {
        let events = [
            BuildingPlacedEvent {
                position: (5, 5),
                radius: 3,
            }, // valid
            BuildingPlacedEvent {
                position: (999, 999),
                radius: 1,
            }, // OOB
            BuildingPlacedEvent {
                position: (1, 1),
                radius: 100,
            }, // clamp
            BuildingPlacedEvent {
                position: (50, 5),
                radius: 1,
            }, // OOB (x)
            BuildingPlacedEvent {
                position: (31, 31),
                radius: 0,
            }, // boundary
        ];

        let mut runs: Vec<(usize, ChannelRegions)> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut e = make_engine();
            let mut bss = BuildingStampSystem::new();
            for ev in events.iter() {
                e.resources.building_event_queue.push_back(*ev);
            }
            bss.tick(&mut e.world, &mut e.resources);

            let queue_len = e.resources.building_event_queue.len();
            let regs: Vec<_> = e.resources.influence_grid.dirty_regions
                [InfluenceChannel::Warmth as usize]
                .iter()
                .map(region_tuple)
                .collect();
            runs.push((queue_len, regs));
        }

        // Type A invariant — exact Warmth dirty-region tuple list (plan threshold, locked).
        // Formula: (cx±r) saturating-clamped to [0, W-1]. W=32, H=32.
        //   (5,5,r=3) → (2,2,8,8)       (999,999,r=1) → OOB skipped
        //   (1,1,r=100) → (0,0,31,31)   (50,5,r=1) → OOB skipped
        //   (31,31,r=0) → (31,31,31,31)
        const EXPECTED: &[RegionTuple] = &[(2, 2, 8, 8), (0, 0, 31, 31), (31, 31, 31, 31)];

        assert_eq!(runs[0].0, 0, "queue must be fully drained");
        assert_eq!(
            runs[0].1.len(),
            3,
            "expected 3 valid stamps (2 OOB skipped)"
        );
        // Type A: exact tuple list matches plan threshold.
        assert_eq!(
            runs[0].1, EXPECTED,
            "valid dirty-region list must match exact plan threshold"
        );
        for r in runs[0].1.iter() {
            assert!(r.2 < W && r.3 < H, "region {r:?} exceeds grid bounds");
        }
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "instance {i} diverged");
        }
    }

    /// D4' — IUS clear+swap idempotency: pending writes survive at most one tick.
    /// After 30 ticks with no source systems, every channel's current buffer is
    /// the all-zero baseline byte-identically across runs.
    #[test]
    fn harness_det_a4_ius_clear_swap_idempotent_30_ticks() {
        let mut runs: Vec<Vec<Vec<u8>>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut e = make_engine();
            // Seed pending buffer for Warmth so any non-clearing implementation
            // would leave residue we could detect.
            {
                let buf = e
                    .resources
                    .influence_grid
                    .pending_buf_mut(InfluenceChannel::Warmth);
                for (i, slot) in buf.iter_mut().take(100).enumerate() {
                    *slot = ((i as u32 * 7 + 3) % 256) as u8;
                }
            }
            e.register_system(Box::new(InfluenceUpdateSystem::new()));
            for _ in 0..30 {
                e.tick();
            }

            let mut snapshot = Vec::with_capacity(InfluenceChannel::COUNT);
            for ch in InfluenceChannel::all() {
                snapshot.push(e.resources.influence_grid.current_buf(*ch).to_vec());
            }
            runs.push(snapshot);
        }

        for (ch_idx, base_buf) in runs[0].iter().enumerate() {
            for &b in base_buf.iter() {
                assert_eq!(b, 0, "channel {ch_idx} current buf must be zeroed");
            }
            for (i, run) in runs.iter().enumerate().skip(1) {
                assert_eq!(
                    *base_buf, run[ch_idx],
                    "channel {ch_idx} diverged at instance {i}"
                );
            }
        }
    }

    /// D5' — Visualization digest fires at tick % 6 == 0 boundaries.
    /// After 13 manual loop iterations the last fire is at tick=12 and the digest
    /// captures the same warmth_total / danger_peak across N instances.
    #[test]
    fn harness_det_a5_viz_digest_tick_boundary_6() {
        let mut runs: Vec<VisualizationDigest> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut e = make_engine();
            // Seed current buffers with known values that no source system will
            // touch (we don't register BSS / IUS).
            {
                let buf = e
                    .resources
                    .influence_grid
                    .pending_buf_mut(InfluenceChannel::Warmth);
                buf[0] = 42;
                buf[1] = 58;
            }
            {
                let buf = e
                    .resources
                    .influence_grid
                    .pending_buf_mut(InfluenceChannel::Danger);
                buf[100] = 200;
            }
            e.resources.influence_grid.swap();

            let mut viz = InfluenceVisualizationSystem::new();
            for t in 0u64..13 {
                e.resources.current_tick = t;
                if t.is_multiple_of(6) {
                    viz.tick(&mut e.world, &mut e.resources);
                }
            }
            runs.push(*viz.last_digest());
        }

        assert_eq!(runs[0].tick, 12, "last fire must be at tick 12");
        assert_eq!(runs[0].warmth_total, 100, "42 + 58 = 100");
        assert_eq!(runs[0].danger_peak, 200);
        for i in 1..N {
            assert_eq!(runs[0].tick, runs[i].tick);
            assert_eq!(runs[0].warmth_total, runs[i].warmth_total);
            assert_eq!(runs[0].danger_peak, runs[i].danger_peak);
        }
    }

    // ─── B-group: primitive-level sim-core direct paths (8 tests) ────────────────

    /// P1 — propagate_bfs byte-identical across N=5 instances given identical
    /// inputs. Pins the FIFO BFS + visited bitmap determinism guarantee.
    #[test]
    fn harness_det_b1_propagate_bfs_byte_identical() {
        let grid = TileGrid::new(W, H);
        let cache = MaterialBlockingCache::empty();

        let mut runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut buf = vec![0u8; grid.len()];
            propagate_bfs(
                &grid,
                &cache,
                &mut buf,
                (10, 10),
                200,
                |i, _| i - 30.0,
                InfluenceChannel::Warmth,
                10,
            );
            runs.push(buf);
        }

        assert!(runs[0].iter().any(|&b| b > 0), "BFS produced no influence");
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "BFS diverged at instance {i}");
        }
    }

    /// P2 — propagate_shadowcast byte-identical across N=5 instances. Pins
    /// the 8-octant traversal + recursive symmetric shadowcast determinism.
    #[test]
    fn harness_det_b2_propagate_shadowcast_byte_identical() {
        let grid = TileGrid::new(W, H);

        let mut runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut buf = vec![0u8; grid.len()];
            propagate_shadowcast(&grid, &mut buf, (15, 15), 200, 10);
            runs.push(buf);
        }

        assert!(
            runs[0].iter().any(|&b| b > 0),
            "shadowcast produced no light"
        );
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "shadowcast diverged at instance {i}");
        }
    }

    /// P3 — propagate_noise (alpha=15, no radius cap) byte-identical across runs.
    #[test]
    fn harness_det_b3_propagate_noise_byte_identical() {
        let grid = TileGrid::new(W, H);
        let cache = MaterialBlockingCache::empty();

        let mut runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut buf = vec![0u8; grid.len()];
            propagate_noise(&grid, &cache, &mut buf, (10, 10), 200);
            runs.push(buf);
        }

        assert!(
            runs[0].iter().any(|&b| b > 0),
            "noise produced no influence"
        );
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "noise diverged at instance {i}");
        }
    }

    /// P4 — stamp_social_aggregate LOD filter: only Full/Medium agents contribute,
    /// Simplified/Dormant skip. Byte-identical across N=5.
    #[test]
    fn harness_det_b4_stamp_social_aggregate_lod_filter() {
        let agents = [
            ((4u32, 4u32), LodTier::Full),
            ((10u32, 10u32), LodTier::Simplified), // skipped
            ((12u32, 12u32), LodTier::Dormant),    // skipped
            ((18u32, 18u32), LodTier::Medium),
            ((24u32, 24u32), LodTier::Full),
        ];

        let mut runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut buf = vec![0u8; (W * H) as usize];
            stamp_social_aggregate(&mut buf, W, H, agents.iter().copied());
            runs.push(buf);
        }

        // Skipped agents leave their tiles at 0 (no other contributor nearby).
        assert_eq!(runs[0][(10 * W + 10) as usize], 0, "Simplified must skip");
        assert_eq!(runs[0][(12 * W + 12) as usize], 0, "Dormant must skip");
        // Active agents leave +1 at their own tile (diamond center).
        assert_eq!(runs[0][(4 * W + 4) as usize], 1);
        assert_eq!(runs[0][(18 * W + 18) as usize], 1);
        assert_eq!(runs[0][(24 * W + 24) as usize], 1);

        for i in 1..N {
            assert_eq!(runs[0], runs[i], "social stamp diverged at instance {i}");
        }
    }

    /// P5 — AHashMap blocking cache lookup determinism. Across N=5 fresh
    /// `MaterialBlockingCache::empty()` instances, the same key sequence returns
    /// byte-identical f32 bit patterns. This pins the "value is not seed-dependent"
    /// property of the lookup path used by propagate_bfs / propagate_noise.
    #[test]
    fn harness_det_b5_blocking_cache_lookup_byte_identical() {
        let keys: Vec<(MaterialId, InfluenceChannel)> = ["granite", "oak", "iron", "water"]
            .iter()
            .flat_map(|name| {
                let mid = MaterialId::from_str_hash(name);
                InfluenceChannel::all().iter().map(move |ch| (mid, *ch))
            })
            .collect();

        let mut runs: Vec<Vec<u32>> = Vec::with_capacity(N);
        for _ in 0..N {
            let cache = MaterialBlockingCache::empty();
            let bits: Vec<u32> = keys
                .iter()
                .map(|(mid, ch)| cache.get(*mid, *ch).to_bits())
                .collect();
            runs.push(bits);
        }

        // Empty cache returns 0.0 for every key — assert byte pattern matches.
        for &bits in runs[0].iter() {
            assert_eq!(bits, 0.0f32.to_bits());
        }
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "cache lookup bits diverged at {i}");
        }
    }

    /// P6 — EC-2 source self-blocking exempt. Even when a wall material is placed
    /// ON the source tile, the source receives the initial intensity (BFS distance
    /// 0 bypasses the blocking lookup). Byte-identical across N=5.
    #[test]
    fn harness_det_b6_ec2_source_self_blocking_exempt() {
        let mut runs: Vec<u8> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut grid = TileGrid::new(W, H);
            let mid = MaterialId::from_str_hash("opaque_wall");
            let src_idx = grid.idx(8, 8);
            grid.wall_material[src_idx] = Some(mid);
            let cache = MaterialBlockingCache::empty(); // 0.0 blocking either way

            let mut buf = vec![0u8; grid.len()];
            propagate_bfs(
                &grid,
                &cache,
                &mut buf,
                (8, 8),
                200,
                |i, _| i - 30.0,
                InfluenceChannel::Warmth,
                10,
            );
            runs.push(buf[src_idx]);
        }

        assert_eq!(runs[0], 200, "source tile must receive initial intensity");
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "source value diverged at instance {i}");
        }
    }

    /// P7 — EC-3 multi-source aggregation: same buffer, two sources, overlapping
    /// influence zones. Warmth (Additive) saturates; Light (Max) clamps. Byte-
    /// identical across N=5.
    #[test]
    fn harness_det_b7_ec3_multi_source_additive_vs_max() {
        let grid = TileGrid::new(W, H);
        let cache = MaterialBlockingCache::empty();

        let mut warmth_runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        let mut light_runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        for _ in 0..N {
            // Warmth: Additive aggregation — two BFS calls overlapping.
            let mut wbuf = vec![0u8; grid.len()];
            propagate_bfs(
                &grid,
                &cache,
                &mut wbuf,
                (10, 10),
                100,
                |i, _| i - 25.0,
                InfluenceChannel::Warmth,
                10,
            );
            propagate_bfs(
                &grid,
                &cache,
                &mut wbuf,
                (12, 10),
                100,
                |i, _| i - 25.0,
                InfluenceChannel::Warmth,
                10,
            );
            warmth_runs.push(wbuf);

            // Light: Max aggregation — two shadowcast calls overlapping.
            let mut lbuf = vec![0u8; grid.len()];
            propagate_shadowcast(&grid, &mut lbuf, (10, 10), 100, 10);
            propagate_shadowcast(&grid, &mut lbuf, (12, 10), 100, 10);
            light_runs.push(lbuf);
        }

        // Warmth midpoint (11,10) receives contributions from both BFS passes.
        // Type A invariant (plan threshold, locked):
        //   Each source decays: decay_fn(100, 1.0) = 100.0 - 25.0 = 75.0 → 75u8.
        //   apply_agg(Warmth=Additive, 0, 75) = 75  (first BFS).
        //   apply_agg(Warmth=Additive, 75, 75) = 75.saturating_add(75) = 150  (second BFS).
        //   warmth_mid == 150 exactly.
        let warmth_mid = warmth_runs[0][(10 * W + 11) as usize];
        assert_eq!(
            warmth_mid, 150,
            "Additive Warmth midpoint must equal 150 (75+75, plan threshold)"
        );

        // Light midpoint must equal max(single-source values), not their sum.
        let light_mid = light_runs[0][(10 * W + 11) as usize];
        assert!(
            light_mid <= 100,
            "Max Light cannot exceed source intensity 100"
        );

        for i in 1..N {
            assert_eq!(warmth_runs[0], warmth_runs[i], "Warmth diverged at {i}");
            assert_eq!(light_runs[0], light_runs[i], "Light diverged at {i}");
        }
    }

    /// P3b — propagate_danger (alpha=5, radius cap 15, no blocking) byte-identical
    /// across runs. Bundled into the B-group as a 12th sentinel test.
    #[test]
    fn harness_det_b8_propagate_danger_byte_identical() {
        let grid = TileGrid::new(W, H);

        let mut runs: Vec<Vec<u8>> = Vec::with_capacity(N);
        for _ in 0..N {
            let mut buf = vec![0u8; grid.len()];
            propagate_danger(&grid, &mut buf, (16, 16), 200);
            runs.push(buf);
        }

        assert!(
            runs[0].iter().any(|&b| b > 0),
            "danger produced no influence"
        );
        for i in 1..N {
            assert_eq!(runs[0], runs[i], "danger diverged at instance {i}");
        }
    }
}

// From `harness_a15_controlled_determinism.rs`.
mod controlled_divergence {
    //! a15-determinism-fix — CONTROLLED forced-divergence determinism suite
    //! (plan attempt 2, A1–A7).
    //!
    //! Root bug (Section 1): `AgentDecisionSystem` iterated the `settlement_migrant_tag`
    //! HashSet to apply a STRUCTURAL `world.insert_one(SettlementMigrant)`. A component
    //! insert moves the entity into the `SettlementMigrant` archetype whose dense-array
    //! order IS the insertion order, so the `RandomState`-seed-dependent HashSet
    //! iteration permuted hecs query order → a contended gather split diverged →
    //! a need value diverged (the a15 tick-2251 fingerprint divergence). The shipped
    //! fix sorts the migrant entities by the stable `AgentId` before inserting, so the
    //! structural insert order — and thus all downstream query iteration — is
    //! seed-independent.
    //!
    //! MEASURED reality (this harness's design driver): the existing process-seeded
    //! a15 pair is a COIN FLIP without the fix — it diverged on 2 of 3 fresh runs and
    //! converged by seed-luck on the 3rd. A SINGLE process-seeded pair therefore
    //! cannot reliably catch the bug (the Challenger's concern). This suite removes
    //! the luck two ways:
    //!   1. Assertion 2 establishes a DETERMINISTIC, reproducible-run-to-run different
    //!      INITIAL yielded iteration order (a controlled permutation of the agent
    //!      collection via a test-only marker insert+remove), and asserts that the two
    //!      engines genuinely start from different orders (anti-vacuous guard).
    //!   2. Assertion 5 runs MANY controlled-divergent pairs in one invocation; with
    //!      the fix every pair locksteps, and without the fix the probability that ALL
    //!      pairs converge by seed-luck is (≈1/3)^N → negligible, so the SUITE reliably
    //!      fails when the fix is absent.
    //!
    //! Assertion ↔ threshold-type map (per FINAL plan attempt 2):
    //!   A1 controlled forced-divergence reproducer — per-tick lockstep ... Type A
    //!   A2 divergent-ordering precondition observed & holds (anti-vacuous) Type A
    //!   A3 process-seeded full-horizon lockstep (existing contract) ...... Type A
    //!   A4 end-state observables identical across runs .................. Type A
    //!   A5 robustness across >= 3 controlled-divergent pairs ............ Type A
    //!   A6 comparison window non-vacuous (liveness + activity floors) ... Type C
    //!   A7 baseline registries mutually consistent, no stale-mask ....... Type A
    //!
    //! Run:
    //!   cargo test -p sim-test --test harness_a15_controlled_determinism -- --nocapture

    use std::collections::BTreeSet;

    use sim_bridge::ffi::{bootstrap_spawn_agents, seed_finite_resource_scarcity};
    use sim_core::causal::event::{CausalEvent, DeathReason};
    use sim_core::components::{
        Agent, AgentId, AgentState, BodyHealth, Hunger, Position, Sleep, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};
    use sim_systems::register_default_runtime_systems;

    const W: u32 = 64;
    const H: u32 = 64;
    /// Full-horizon length — matches the existing a15 horizon and spans the
    /// documented 9 → 489 → 2251 divergence chain.
    const RUN_TICKS: u64 = 5000;
    /// Reduced horizon for the additional A5 pairs — still strictly past the
    /// documented tick-2251 divergence point (plan: ">= 2300").
    const SHORT_TICKS: u64 = 2300;
    /// Number of distinct controlled-divergent pairs A5 exercises. The plan locks
    /// ">= 3"; this suite uses 8 to drive the no-fix false-pass probability to
    /// (≈1/3)^8 ≈ 1.5e-4 so the SUITE reliably fails when the fix is absent.
    const NUM_PAIRS: usize = 8;

    /// Test-only zero-sized marker used purely to migrate an agent out of and back
    /// into the base archetype, re-appending it in a chosen order. NEVER read by any
    /// production system, so it cannot perturb simulation VALUES — only the hecs
    /// dense-array (query iteration) order the decision system observes.
    struct OrderPerm;

    // ── scene / engine construction ───────────────────────────────────────────────

    /// Production-EQUIVALENT scene WITH finite scarcity (mirrors
    /// `harness_resource_scarcity::finite_scene`): default runtime systems +
    /// bootstrap agents + finite seeding + the 3 startup buildings.
    fn finite_scene() -> SimEngine {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        register_default_runtime_systems(&mut e);
        bootstrap_spawn_agents(&mut e);
        seed_finite_resource_scarcity(&mut e);
        for pos in [(32u32, 32u32), (24, 32), (40, 32)] {
            e.resources
                .building_event_queue
                .push_back(BuildingPlacedEvent {
                    position: pos,
                    radius: 8,
                });
        }
        e
    }

    // ── iteration-order control + observation ─────────────────────────────────────

    /// The ordered `AgentId` sequence the world currently YIELDS over the agent
    /// collection — the raw hecs query order the decision system processes.
    fn yielded_agent_order(e: &SimEngine) -> Vec<AgentId> {
        e.world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect()
    }

    /// Apply a deterministic permutation `perm` to the engine's base agent
    /// archetype so its yielded iteration order changes (membership unchanged).
    /// Reorders by insert+remove of the test-only `OrderPerm` marker: each insert
    /// migrates the entity to the `+OrderPerm` archetype and the remove migrates it
    /// back, RE-APPENDED at the dense-array tail. Processing the agents in `perm`
    /// order therefore leaves the base archetype yielding in `perm` order.
    fn permute_agent_order(e: &mut SimEngine, perm: impl Fn(&[hecs::Entity]) -> Vec<hecs::Entity>) {
        let cur: Vec<hecs::Entity> = e
            .world
            .query::<&Agent>()
            .iter()
            .map(|(ent, _)| ent)
            .collect();
        for ent in perm(&cur) {
            let _ = e.world.insert_one(ent, OrderPerm);
            let _ = e.world.remove_one::<OrderPerm>(ent);
        }
    }

    /// Reverse permutation (guaranteed != identity for n >= 2).
    fn perm_reverse(cur: &[hecs::Entity]) -> Vec<hecs::Entity> {
        let mut v = cur.to_vec();
        v.reverse();
        v
    }

    /// Rotate-left-by-`k` permutation (guaranteed != identity for 0 < k < n).
    fn perm_rotate(cur: &[hecs::Entity], k: usize) -> Vec<hecs::Entity> {
        let mut v = cur.to_vec();
        let n = v.len();
        if n > 0 {
            v.rotate_left(k % n);
        }
        v
    }

    // ── behavioural fingerprint (matches the existing a15 fingerprint) ────────────

    fn fnv(acc: &mut u64, bytes: &[u8]) {
        for &b in bytes {
            *acc ^= b as u64;
            *acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
        }
    }

    /// Behaviourally-observable fingerprint: per-agent position + `AgentState` +
    /// needs (bit-exact) + `BodyHealth`, the dead-agent set, the per-reason death
    /// split, and the resource-tile key-sets WITH values. EXCLUDES `event_id`.
    /// Rows are sorted by `AgentId`, so the fingerprint compares simulation STATE,
    /// not iteration order — it cannot pass vacuously by re-sorting.
    fn behavioural_fingerprint(e: &SimEngine) -> u64 {
        let mut acc: u64 = 0xcbf2_9ce4_8422_2325;

        let ents: Vec<hecs::Entity> = e
            .world
            .query::<&Agent>()
            .iter()
            .map(|(ent, _)| ent)
            .collect();
        let mut rows: Vec<(AgentId, u64, u64, String)> = Vec::new();
        for ent in ents {
            let Ok(agent) = e.world.get::<&Agent>(ent) else {
                continue;
            };
            let pos_bits = e
                .world
                .get::<&Position>(ent)
                .map(|p| ((p.x as u64) << 32) | p.y as u64)
                .unwrap_or(0);
            let hunger = e
                .world
                .get::<&Hunger>(ent)
                .map(|h| (h.value as f64).to_bits())
                .unwrap_or(0);
            let thirst = e
                .world
                .get::<&Thirst>(ent)
                .map(|t| t.value.to_bits())
                .unwrap_or(0);
            let sleep = e
                .world
                .get::<&Sleep>(ent)
                .map(|s| s.fatigue.to_bits())
                .unwrap_or(0);
            let hp = e
                .world
                .get::<&BodyHealth>(ent)
                .map(|b| b.hp.to_bits())
                .unwrap_or(0);
            let state = e
                .world
                .get::<&AgentState>(ent)
                .map(|s| format!("{s:?}"))
                .unwrap_or_default();
            let need_mix = hunger.wrapping_mul(0x9E37_79B9)
                ^ thirst.rotate_left(17)
                ^ sleep.rotate_left(31)
                ^ hp.rotate_left(7);
            rows.push((agent.id, pos_bits, need_mix, state));
        }
        rows.sort_by_key(|(id, ..)| *id);
        for (id, pos_bits, need_mix, state) in &rows {
            fnv(&mut acc, &id.to_le_bytes());
            fnv(&mut acc, &pos_bits.to_le_bytes());
            fnv(&mut acc, &need_mix.to_le_bytes());
            fnv(&mut acc, state.as_bytes());
            fnv(&mut acc, b"|");
        }

        let (starv, dehy, comb) = per_reason_split(e);
        let dead = dead_set(e);
        fnv(&mut acc, b"#dead");
        for id in &dead {
            fnv(&mut acc, &id.to_le_bytes());
        }
        fnv(&mut acc, &(starv as u64).to_le_bytes());
        fnv(&mut acc, &(dehy as u64).to_le_bytes());
        fnv(&mut acc, &(comb as u64).to_le_bytes());

        for (label, map) in [
            (b"#food".as_slice(), &e.resources.food_tiles),
            (b"#water".as_slice(), &e.resources.water_tiles),
            (b"#sleep".as_slice(), &e.resources.sleep_tiles),
        ] {
            fnv(&mut acc, label);
            let mut keys: Vec<(u32, u32)> = map.keys().copied().collect();
            keys.sort_unstable();
            for (x, y) in keys {
                fnv(&mut acc, &x.to_le_bytes());
                fnv(&mut acc, &y.to_le_bytes());
                if let Some(v) = map.get(&(x, y)) {
                    fnv(&mut acc, &[*v]);
                }
            }
        }

        acc
    }

    // ── observable scanners (read-only over the shipped causal log) ───────────────

    fn dead_set(e: &SimEngine) -> BTreeSet<AgentId> {
        let mut s = BTreeSet::new();
        for (_t, log) in e.resources.causal_log.iter() {
            for ev in log.iter() {
                if let CausalEvent::AgentDied { agent, .. } = ev {
                    s.insert(*agent);
                }
            }
        }
        s
    }

    fn scarcity_dead_set(e: &SimEngine) -> BTreeSet<AgentId> {
        let mut s = BTreeSet::new();
        for (_t, log) in e.resources.causal_log.iter() {
            for ev in log.iter() {
                if let CausalEvent::AgentDied { agent, reason, .. } = ev {
                    if matches!(reason, DeathReason::Starvation | DeathReason::Dehydration) {
                        s.insert(*agent);
                    }
                }
            }
        }
        s
    }

    /// `(starvation, dehydration, combat)` — `DeathReason` is exhaustively matched
    /// so a future variant fails to compile here.
    fn per_reason_split(e: &SimEngine) -> (usize, usize, usize) {
        let (mut starvation, mut dehydration, mut combat) = (0usize, 0usize, 0usize);
        for (_t, log) in e.resources.causal_log.iter() {
            for ev in log.iter() {
                if let CausalEvent::AgentDied { reason, .. } = ev {
                    match reason {
                        DeathReason::Starvation => starvation += 1,
                        DeathReason::Dehydration => dehydration += 1,
                        DeathReason::Combat => combat += 1,
                        // Pass 4: the legacy scarcity scene never ages anyone.
                        DeathReason::OldAge => {}
                        // Pass 10: legacy worlds have no exposure rule.
                        DeathReason::Exposure => {}
                    }
                }
            }
        }
        (starvation, dehydration, combat)
    }

    fn settlement_total_deaths(e: &SimEngine) -> u32 {
        e.resources
            .settlements
            .values()
            .map(|s| s.population_stats.total_deaths)
            .sum()
    }

    fn live_count(e: &SimEngine) -> usize {
        e.world.query::<&Agent>().iter().count()
    }

    /// End-state observable bundle for A4 equality checks.
    #[derive(Debug, PartialEq, Eq)]
    struct EndState {
        dead: BTreeSet<AgentId>,
        scarcity_dead: BTreeSet<AgentId>,
        per_reason: (usize, usize, usize),
        total_deaths: u32,
        live: usize,
    }

    fn end_state(e: &SimEngine) -> EndState {
        EndState {
            dead: dead_set(e),
            scarcity_dead: scarcity_dead_set(e),
            per_reason: per_reason_split(e),
            total_deaths: settlement_total_deaths(e),
            live: live_count(e),
        }
    }

    /// Run a controlled-divergent pair: A is un-permuted, B is permuted by `perm`.
    /// Asserts the A2 precondition (orders genuinely differ) up front, then ticks
    /// both `ticks` ticks in lockstep, returning the first behavioural divergence
    /// tick (None == full lockstep) plus the post-run end-states for A4.
    fn run_controlled_pair(
        label: &str,
        perm: impl Fn(&[hecs::Entity]) -> Vec<hecs::Entity>,
        ticks: u64,
    ) -> (Option<u64>, EndState, EndState) {
        let mut a = finite_scene();
        let mut b = finite_scene();
        permute_agent_order(&mut b, perm);

        // ── A2 precondition: the two captured initial orders MUST differ. ──
        let order_a = yielded_agent_order(&a);
        let order_b = yielded_agent_order(&b);
        assert!(
        order_a != order_b,
        "[{label}] A2 INVALID: the controlled permutation did NOT change the yielded iteration \
         order (both engines start identically ordered) — a 0-mismatch lockstep would be \
         VACUOUS. order_a.len()={}, order_b.len()={}",
        order_a.len(),
        order_b.len(),
    );
        assert_eq!(
            order_a.iter().copied().collect::<BTreeSet<_>>(),
            order_b.iter().copied().collect::<BTreeSet<_>>(),
            "[{label}] A2: permutation must preserve MEMBERSHIP (same agent set, different order)",
        );

        let mut first_divergence: Option<u64> = None;
        for t in 1..=ticks {
            a.tick();
            b.tick();
            if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
                first_divergence = Some(t);
                break;
            }
        }
        (first_divergence, end_state(&a), end_state(&b))
    }

    // ════════════════════════════════════════════════════════════════════════════
    // A1 + A2 + A4 + A6 — primary controlled forced-divergence pair (reverse order).
    // One 5000-tick run anchors per-tick lockstep (A1), the anti-vacuous order
    // precondition (A2), end-state equality (A4, this pair), and the non-vacuity
    // floors (A6). Sharing the run keeps the suite cheap.
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_a15_controlled_forced_divergence_lockstep() {
        // A6 non-vacuity is measured on the un-permuted twin A across the window.
        let mut a = finite_scene();
        let mut b = finite_scene();
        permute_agent_order(&mut b, perm_reverse);

        // ── A2: controlled-divergence precondition is OBSERVED and holds. ──
        let order_a = yielded_agent_order(&a);
        let order_b = yielded_agent_order(&b);
        assert!(
            order_a != order_b,
            "A2 INVALID: reverse permutation did not change the yielded iteration order — \
         a 0-mismatch lockstep would be VACUOUS"
        );
        assert_eq!(
            order_a.iter().copied().collect::<BTreeSet<_>>(),
            order_b.iter().copied().collect::<BTreeSet<_>>(),
            "A2: reverse permutation must preserve membership"
        );

        // A6 activity counters (on A, the un-permuted reference trajectory).
        let deaths_before = dead_set(&a).len();
        let mut prev_states: std::collections::HashMap<AgentId, String> = state_map(&a);
        let mut transitions: usize = 0;

        // ── A1: per-tick behavioural lockstep across the full horizon. ──
        let mut first_divergence: Option<u64> = None;
        for t in 1..=RUN_TICKS {
            a.tick();
            b.tick();
            // accumulate A6 state-transition count on A
            let now = state_map(&a);
            for (id, st) in &now {
                if prev_states.get(id).map(|p| p != st).unwrap_or(false) {
                    transitions += 1;
                }
            }
            prev_states = now;
            if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
                first_divergence = Some(t);
                break;
            }
        }
        assert!(
        first_divergence.is_none(),
        "A1: behavioural divergence at tick {first_divergence:?} of {RUN_TICKS} under a CONTROLLED \
         reverse-permuted starting order — the determinism fix did not restore full-horizon \
         lockstep (run A dead-so-far={:?})",
        dead_set(&a),
    );

        // ── A4 (this pair): end-state observables identical. ──
        let (es_a, es_b) = (end_state(&a), end_state(&b));
        assert_eq!(
            es_a, es_b,
            "A4: end-state observables must be identical across the controlled pair"
        );

        // ── A6: comparison window was non-vacuous. ──
        let deaths_in_window = dead_set(&a).len() - deaths_before;
        let live_final = live_count(&a);
        assert!(
            live_final >= 3,
            "A6(a): live_count at the final compared tick must be >= 3 (got {live_final}) — \
         a permutation needs >= 2 elements; >= 3 is the conservative non-vacuity floor"
        );
        assert!(
            deaths_in_window >= 1,
            "A6(b): >= 1 death must occur in the compared window (got {deaths_in_window}) — \
         the scarcity scene must actually exercise the order-sensitive death path"
        );
        assert!(
        transitions >= 1,
        "A6(c): >= 1 AgentState transition must occur in the compared window (got {transitions})"
    );

        println!(
        "[a15 A1/A2/A4/A6] reverse-permuted pair: full {RUN_TICKS}-tick lockstep (orders differ at \
         t0) — live_final={live_final} deaths_in_window={deaths_in_window} transitions={transitions} \
         end_state(dead={} scarcity={} per_reason={:?} td={} live={})",
        es_a.dead.len(),
        es_a.scarcity_dead.len(),
        es_a.per_reason,
        es_a.total_deaths,
        es_a.live,
    );
    }

    /// Per-agent `AgentId -> AgentState` debug-string map (A6 transition counter).
    fn state_map(e: &SimEngine) -> std::collections::HashMap<AgentId, String> {
        let mut m = std::collections::HashMap::new();
        for (_, (agent, state)) in e.world.query::<(&Agent, &AgentState)>().iter() {
            m.insert(agent.id, format!("{state:?}"));
        }
        m
    }

    // ════════════════════════════════════════════════════════════════════════════
    // A3 + A4 — process-seeded full-horizon lockstep (existing a15 contract).
    // Two finite scenes built the way the existing a15 test builds them (no
    // controlled permutation — independent process-random internal seeds).
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_a15_process_seeded_lockstep() {
        let mut a = finite_scene();
        let mut b = finite_scene();

        let mut first_divergence: Option<u64> = None;
        for t in 1..=RUN_TICKS {
            a.tick();
            b.tick();
            if behavioural_fingerprint(&a) != behavioural_fingerprint(&b) {
                first_divergence = Some(t);
                break;
            }
        }
        assert!(
        first_divergence.is_none(),
        "A3: process-seeded behavioural divergence at tick {first_divergence:?} of {RUN_TICKS} — \
         the fix did not hold under the existing a15 conditions (run A dead-so-far={:?})",
        dead_set(&a),
    );

        // ── A4 (process-seeded pair): end-state observables identical. ──
        let (es_a, es_b) = (end_state(&a), end_state(&b));
        assert_eq!(
            es_a, es_b,
            "A4: end-state observables must be identical across the process-seeded pair"
        );

        println!(
            "[a15 A3/A4] process-seeded pair: full {RUN_TICKS}-tick lockstep — \
         end_state(dead={} scarcity={} per_reason={:?} td={} live={})",
            es_a.dead.len(),
            es_a.scarcity_dead.len(),
            es_a.per_reason,
            es_a.total_deaths,
            es_a.live,
        );
    }

    // ════════════════════════════════════════════════════════════════════════════
    // A5 — robustness across MULTIPLE distinct controlled-divergent pairs.
    // Pair 0 runs the full horizon; pairs 1..NUM_PAIRS run >= 2300 ticks (past the
    // documented tick-2251 divergence). Each pair must lockstep AND pass the A2
    // precondition. With the fix every pair locksteps; without it, the chance ALL
    // pairs converge by seed-luck is (≈1/3)^NUM_PAIRS → negligible, so this is the
    // guard that makes the SUITE reliably fail when the fix is absent.
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_a15_multi_permutation_robustness() {
        // NUM_PAIRS distinct, deterministic permutations: reverse, then rotate-left
        // by 1..=NUM_PAIRS-1 (all non-identity, all distinct from each other for the
        // 64-agent bootstrap lattice).
        for i in 0..NUM_PAIRS {
            let ticks = if i == 0 { RUN_TICKS } else { SHORT_TICKS };
            let (label, div, es_a, es_b): (String, Option<u64>, EndState, EndState) = if i == 0 {
                let (d, a, b) = run_controlled_pair("pair0/reverse", perm_reverse, ticks);
                ("pair0/reverse".to_string(), d, a, b)
            } else {
                let k = i; // rotate-left by k (1..=NUM_PAIRS-1)
                let lbl = format!("pair{i}/rotate{k}");
                let (d, a, b) = run_controlled_pair(&lbl, move |cur| perm_rotate(cur, k), ticks);
                (lbl, d, a, b)
            };
            assert!(
            div.is_none(),
            "A5[{label}]: behavioural divergence at tick {div:?} of {ticks} under a controlled \
             divergent starting order — the fix did not restore lockstep for this permutation"
        );
            // Aggregate end-state must also match per pair (subset of A4 across pairs).
            assert_eq!(
                es_a, es_b,
                "A5[{label}]: end-state observables must match within the pair"
            );
            println!("[a15 A5] {label}: {ticks}-tick lockstep (orders differ at t0) ✓");
        }
        println!("[a15 A5] all {NUM_PAIRS} controlled-divergent pairs locksteppped ✓");
    }
}

// From `harness_fix_d_memory_arm.rs`.
mod memory_arm {
    //! V7 feature `add-resource-scarcity-regen` — Fix D (memory-arm determinism).
    //!
    //! # Why this harness exists
    //!
    //! The resource-scarcity scene exposed a determinism root cause (documented in
    //! the project memory `resource-scarcity-checkpoint`): the cascade-bias sum
    //! `memory_weight_delta` re-derived each memory entry's cascade arm at READ
    //! time via `event_id_matches_arm(entry.event_id, arm, causal_log)`, which did
    //! a `causal_log.lookup`. That per-tile ring buffer is an 8-slot FIFO, so an
    //! entry whose source event had been evicted resolved to "no arm" and silently
    //! stopped contributing. Because `event_id` allocation order (and therefore
    //! which event occupies which ring slot at a given tick) is not byte-stable
    //! across processes, the same logical memory could be load-bearing in one run
    //! and inert in another — flipping a cascade decision (`Idle` ↔ `Seeking`) from
    //! an otherwise identical agent state, producing a dehydration victim-swap
    //! (run A kills a different agent than run B).
    //!
    //! Fix D records the arm on the [`MemoryEntry`] at ENCODE time (from
    //! `classify_event`) and has `memory_weight_delta` read that stored tag. The
    //! delta becomes a pure function of each entry's own `(arm, valence, salience,
    //! encoded_tick)` — independent of `event_id` values AND ring-buffer eviction.
    //!
    //! # Assertions
    //!   A1  encode-time arm tagging — MemorySystem stores the classified arm ...... Type A
    //!   A2  full production-scarcity scene is DETERMINISTIC under a per-tick
    //!       behavioural lockstep (cap 2200, two independent in-process runs) ...... Type A
    //!
    //! Fingerprint scope (A2): the lockstep hash covers BEHAVIOURAL state only —
    //! agent position, `AgentState`, all needs (bit-exact), `BodyHealth`, the
    //! dead-agent set, the per-reason death split, and resource-tile key-sets. It
    //! deliberately EXCLUDES `event_id` values and memory `event_id` contents:
    //! Fix D leaves `event_id` a non-load-bearing label that MAY still differ
    //! across runs (it does not determinise the id counter). Determinism of the
    //! observable simulation outcome is the contract, not of internal labels.
    //!
    //! Run:
    //!   `cargo test -p sim-test --test harness_fix_d_memory_arm -- --nocapture`
    //!   (release strongly recommended for A2: `cargo test --release ...`)

    use std::collections::BTreeSet;
    use std::hash::BuildHasher;

    use sim_bridge::ffi::{bootstrap_spawn_agents, seed_finite_resource_scarcity};
    use sim_core::causal::event::{CausalEvent, DeathReason, DecisionReason, EventId};
    use sim_core::components::{
        Agent, AgentId, AgentState, BodyHealth, Hunger, Memory, MemoryArm, MemoryEntry, Position,
        Sleep, TargetKind, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
    use sim_systems::register_default_runtime_systems;
    use sim_systems::runtime::decision::AgentDecisionSystem;
    use sim_systems::runtime::memory::{MemorySystem, MAX_RECENCY_TICKS};

    const W: u32 = 64;
    const H: u32 = 64;

    /// Production-EQUIVALENT scene WITH finite scarcity — byte-for-byte the same
    /// construction `harness_resource_scarcity::finite_scene` uses (the scene that
    /// exhibited the nondeterminism): default runtime + bootstrap lattice + finite
    /// seeding + the 3 startup buildings that seed an early settlement.
    fn finite_scene() -> SimEngine {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        register_default_runtime_systems(&mut e);
        bootstrap_spawn_agents(&mut e);
        seed_finite_resource_scarcity(&mut e);
        for pos in [(32u32, 32u32), (24, 32), (40, 32)] {
            e.resources
                .building_event_queue
                .push_back(BuildingPlacedEvent {
                    position: pos,
                    radius: 8,
                });
        }
        e
    }

    /// Direct construction-path assertion of the distinct-seed precondition: the
    /// two engines' resource HashMaps were built with INDEPENDENT
    /// `RandomState::new()` seeds.
    ///
    /// `HashMap::hasher()` exposes each map's actual `RandomState` — the seed the
    /// construction path drew via `HashMap::new()` → `RandomState::new()`. Hashing
    /// the same fixed keys through both hashers yields a different value for at
    /// least one key iff the seeds are independent. This reads the REAL engine
    /// hashers directly — NOT an iteration-order sample of a small key set (which
    /// could coincide and falsely report INVALID). Two distinct 64-bit-seeded
    /// `RandomState`s disagree on at least one of these fixed keys with
    /// overwhelming probability.
    fn engines_independently_seeded(a: &SimEngine, b: &SimEngine) -> bool {
        let ha = a.resources.food_tiles.hasher();
        let hb = b.resources.food_tiles.hasher();
        [(0u32, 0u32), (1, 2), (3, 5), (7, 11), (13, 17)]
            .into_iter()
            .any(|k| ha.hash_one(k) != hb.hash_one(k))
    }

    /// Encode a single event for a fresh actor agent and return the stored
    /// [`MemoryArm`] on the resulting entry (`None` if the event was not encoded).
    ///
    /// `make` builds the [`CausalEvent`] from `(event_id, actor_agent_id,
    /// current_tick)`. `MemorySystem` only encodes events whose `tick() ==
    /// current_tick` AND whose `classify_event` agent list contains a live agent,
    /// so the event must be built at `current_tick` for the spawned actor.
    fn encode_event_arm(
        make: impl FnOnce(EventId, AgentId, u64) -> CausalEvent,
    ) -> Option<MemoryArm> {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        let ent = e.spawn_agent(10, 10);
        e.world
            .insert_one(ent, Memory::new())
            .expect("attach Memory");
        let aid = e.world.get::<&Agent>(ent).expect("agent present").id;
        let tick = e.resources.current_tick;
        let idx = 10 * W + 10;
        let ev_id = e.resources.issue_event_id();
        e.resources.causal_log.push(idx, make(ev_id, aid, tick));
        let mut sys = MemorySystem;
        sys.tick(&mut e.world, &mut e.resources);
        let mem = e.world.get::<&Memory>(ent).expect("memory present");
        mem.entries
            .iter()
            .find(|m| m.event_id == ev_id)
            .map(|m| m.arm)
    }

    /// Deterministic FNV-1a fold helper.
    fn fnv(acc: &mut u64, bytes: &[u8]) {
        for &b in bytes {
            *acc ^= b as u64;
            *acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
        }
    }

    /// Hash of the BEHAVIOURALLY-observable engine state at the current tick.
    ///
    /// Excludes `event_id` values (non-load-bearing labels under Fix D). Includes
    /// every field that determines the simulation's future trajectory: per-agent
    /// position + `AgentState` + needs (bit-exact) + `BodyHealth`, the dead set,
    /// the per-reason death split, and the resource-tile key-sets.
    fn behavioural_fingerprint(e: &SimEngine) -> u64 {
        let mut acc: u64 = 0xcbf2_9ce4_8422_2325;

        // Collect entities first (release the query borrow before per-component
        // `world.get` access), then build one row per agent.
        let ents: Vec<hecs::Entity> = e
            .world
            .query::<&Agent>()
            .iter()
            .map(|(ent, _)| ent)
            .collect();
        let mut rows: Vec<(AgentId, [u8; 8], [u8; 8], String)> = Vec::new();
        for ent in ents {
            let Ok(agent) = e.world.get::<&Agent>(ent) else {
                continue;
            };
            let pos_bits = e
                .world
                .get::<&Position>(ent)
                .map(|p| ((p.x as u64) << 32) | p.y as u64)
                .unwrap_or(0);
            let hunger = e
                .world
                .get::<&Hunger>(ent)
                .map(|h| (h.value as f64).to_bits())
                .unwrap_or(0);
            let thirst = e
                .world
                .get::<&Thirst>(ent)
                .map(|t| t.value.to_bits())
                .unwrap_or(0);
            let sleep = e
                .world
                .get::<&Sleep>(ent)
                .map(|s| s.fatigue.to_bits())
                .unwrap_or(0);
            let hp = e
                .world
                .get::<&BodyHealth>(ent)
                .map(|b| b.hp.to_bits())
                .unwrap_or(0);
            let state = e
                .world
                .get::<&AgentState>(ent)
                .map(|s| format!("{s:?}"))
                .unwrap_or_default();
            let need_mix = hunger.wrapping_mul(0x9E37_79B9)
                ^ thirst.rotate_left(17)
                ^ sleep.rotate_left(31)
                ^ hp.rotate_left(7);
            rows.push((
                agent.id,
                pos_bits.to_le_bytes(),
                need_mix.to_le_bytes(),
                state,
            ));
        }
        rows.sort_by_key(|(id, ..)| *id);
        for (id, pos_bits, need_mix, state) in &rows {
            fnv(&mut acc, &id.to_le_bytes());
            fnv(&mut acc, pos_bits);
            fnv(&mut acc, need_mix);
            fnv(&mut acc, state.as_bytes());
            fnv(&mut acc, b"|");
        }

        // Dead set + per-reason split (the victim-swap observables).
        let (mut starv, mut dehy, mut comb) = (0u32, 0u32, 0u32);
        let mut dead: BTreeSet<AgentId> = BTreeSet::new();
        for (_t, log) in e.resources.causal_log.iter() {
            for ev in log.iter() {
                if let CausalEvent::AgentDied { agent, reason, .. } = ev {
                    dead.insert(*agent);
                    match reason {
                        DeathReason::Starvation => starv += 1,
                        DeathReason::Dehydration => dehy += 1,
                        DeathReason::Combat => comb += 1,
                        // Pass 4: legacy worlds never age, so old age is never counted here.
                        DeathReason::OldAge => {}
                        // Pass 10: legacy worlds have no exposure rule.
                        DeathReason::Exposure => {}
                    }
                }
            }
        }
        fnv(&mut acc, b"#dead");
        for id in &dead {
            fnv(&mut acc, &id.to_le_bytes());
        }
        fnv(&mut acc, &starv.to_le_bytes());
        fnv(&mut acc, &dehy.to_le_bytes());
        fnv(&mut acc, &comb.to_le_bytes());

        // Resource-tile key-sets (depletion/regen surface).
        for (label, map) in [
            (b"#food".as_slice(), &e.resources.food_tiles),
            (b"#water".as_slice(), &e.resources.water_tiles),
            (b"#sleep".as_slice(), &e.resources.sleep_tiles),
        ] {
            fnv(&mut acc, label);
            let mut keys: Vec<(u32, u32)> = map.keys().copied().collect();
            keys.sort_unstable();
            for (x, y) in keys {
                fnv(&mut acc, &x.to_le_bytes());
                fnv(&mut acc, &y.to_le_bytes());
                // value too — regen amount must be deterministic.
                if let Some(v) = map.get(&(x, y)) {
                    fnv(&mut acc, &[*v]);
                }
            }
        }

        acc
    }

    // ════════════════════════════════════════════════════════════════════════════
    // A1 — encode-time arm tagging: MemorySystem stores the classified cascade arm.
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_fix_d_a1_encode_stores_classified_arm() {
        // Type A (plan Assertion 2). Drive an encode of a representative event of
        // EACH of the six cascade classes and read back the stored
        // `MemoryEntry.arm`. Every stored arm is compared against a HARDCODED
        // literal expectation written into this test body — NOT a re-invocation of
        // the private `classify_event` (which would be circular: a wrong mapping
        // would match itself). A `classes_exercised` counter proves all six classes
        // actually fired, so a scenario that silently skipped, say, Combat cannot
        // pass vacuously. Finally, an anti-recursion event
        // (AgentDecision{MemoryReason}) and a non-actor event (BuildingPlaced) must
        // produce ZERO biased entries (classify → None → unencoded).
        let hunger = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
            id,
            parent: None,
            agent: aid,
            position: (10, 10),
            reason: DecisionReason::HungerThresholdBreach,
            tick,
        })
        .expect("A1: a Hunger-class event must encode for its actor");
        let thirst = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
            id,
            parent: None,
            agent: aid,
            position: (10, 10),
            reason: DecisionReason::ThirstThresholdBreach,
            tick,
        })
        .expect("A1: a Thirst-class event must encode for its actor");
        let fatigue = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
            id,
            parent: None,
            agent: aid,
            position: (10, 10),
            reason: DecisionReason::FatigueThresholdBreach,
            tick,
        })
        .expect("A1: a Fatigue-class event must encode for its actor");
        let construction = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
            id,
            parent: None,
            agent: aid,
            position: (10, 10),
            reason: DecisionReason::ConstructionReason,
            tick,
        })
        .expect("A1: a Construction-class event must encode for its actor");
        let social = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
            id,
            parent: None,
            agent: aid,
            position: (10, 10),
            reason: DecisionReason::SocialReason,
            tick,
        })
        .expect("A1: a Social-class event must encode for its actor");
        let combat = encode_event_arm(|id, aid, tick| CausalEvent::CombatCompleted {
            id,
            parent: None,
            attacker: aid,
            defender: aid.saturating_add(1),
            position: (10, 10),
            hp_after: 90.0,
            settlement_link: None,
            tick,
        })
        .expect("A1: a Combat-class event must encode for its actor");

        // HARDCODED expected (class → arm) table — literals, never `classify_event`.
        let table: [(&str, MemoryArm, MemoryArm); 6] = [
            ("Hunger", MemoryArm::Hunger, hunger),
            ("Thirst", MemoryArm::Thirst, thirst),
            ("Fatigue", MemoryArm::Fatigue, fatigue),
            ("Construction", MemoryArm::Construction, construction),
            ("Social", MemoryArm::Social, social),
            ("Combat", MemoryArm::Combat, combat),
        ];
        let mut classes_exercised = 0u32;
        let mut arm_mismatch_count = 0u32;
        for (label, expected, got) in table {
            classes_exercised += 1;
            if got != expected {
                arm_mismatch_count += 1;
            }
            assert_eq!(
                got, expected,
                "A1: a {label}-class event must encode with {expected:?} (got {got:?})"
            );
        }
        assert_eq!(arm_mismatch_count, 0, "A1: arm_mismatch_count must be 0");
        assert_eq!(
        classes_exercised, 6,
        "A1: all six cascade classes must be exercised (coverage_precondition); got {classes_exercised}"
    );

        // Anti-recursion (MemoryReason) + non-actor (BuildingPlaced) → no biased
        // entry (classify_event returns None → MemorySystem never encodes them).
        let anti_recursion = encode_event_arm(|id, aid, tick| CausalEvent::AgentDecision {
            id,
            parent: None,
            agent: aid,
            position: (10, 10),
            reason: DecisionReason::MemoryReason,
            tick,
        });
        assert_eq!(
            anti_recursion, None,
            "A1: an anti-recursion AgentDecision{{MemoryReason}} must produce ZERO biased entries"
        );
        let non_actor = encode_event_arm(|id, _aid, tick| CausalEvent::BuildingPlaced {
            id,
            parent: None,
            position: (10, 10),
            radius: 1,
            tick,
        });
        assert_eq!(
            non_actor, None,
            "A1: a non-actor BuildingPlaced event must produce ZERO biased entries"
        );

        println!(
        "[fix-d A1] 6/6 cascade classes encode the correct MemoryArm; anti-recursion + non-actor unencoded ✓"
    );
    }

    // ════════════════════════════════════════════════════════════════════════════
    // A2 — full production-scarcity scene is DETERMINISTIC under a per-tick
    //      behavioural lockstep across two independent in-process runs.
    //
    // Two `finite_scene()` engines are built fresh (so their internal HashMaps draw
    // independent `RandomState` seeds — the cross-run nondeterminism condition) and
    // ticked in lockstep. After every tick both behavioural fingerprints must match.
    // Cap 2200 spans every divergence tick the checkpoint observed (1802 / 2163 /
    // 2491-ε); before Fix D the runs diverged at ~1802.
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_fix_d_a2_scarcity_scene_lockstep_deterministic() {
        const CAP: u64 = 2200;

        let mut a = finite_scene();
        let mut b = finite_scene();

        // ── INVALID precondition (the load-bearing anti-gaming guard) ──────────
        // The 0-mismatch result below is only meaningful if the two engines were
        // built with DISTINCT HashMap `RandomState` seeds. We assert this DIRECTLY
        // off each engine's real `food_tiles` hasher (the construction path's
        // `RandomState::new()`), NOT by sampling iteration order. If the seeds
        // coincided, both engines would iterate every HashMap in the same order and
        // a HashMap-order-dependent bias bug would pass silently — proving nothing
        // about the exact failure mode Fix D exists to kill.
        assert!(
        engines_independently_seeded(&a, &b),
        "A2 INVALID: the two engines were built with the SAME RandomState seed, so a 0-mismatch \
         lockstep would be MEANINGLESS (an order-dependent bias bug would pass silently). The \
         determinism contract requires independently-seeded engines; this is a test-environment \
         fault, not a Fix D pass."
    );

        let mut first_divergence: Option<u64> = None;
        for t in 1..=CAP {
            a.tick();
            b.tick();
            let fa = behavioural_fingerprint(&a);
            let fb = behavioural_fingerprint(&b);
            if fa != fb {
                first_divergence = Some(t);
                break;
            }
        }

        if let Some(t) = first_divergence {
            // Dump the observable diff for diagnosis.
            let da = {
                let mut s: Vec<AgentId> = Vec::new();
                for (_t, log) in a.resources.causal_log.iter() {
                    for ev in log.iter() {
                        if let CausalEvent::AgentDied { agent, .. } = ev {
                            s.push(*agent);
                        }
                    }
                }
                s.sort_unstable();
                s
            };
            panic!(
                "A2: behavioural divergence at tick {t} (cap {CAP}) — Fix D did not \
             restore determinism. run A dead-so-far={da:?}. The memory-arm decoupling \
             is insufficient; another entropy source remains (instrument tick {t})."
            );
        }

        println!(
            "[fix-d A2] {CAP} ticks lockstep: behaviourally IDENTICAL across two \
         independent-RandomState runs — determinism restored ✓"
        );
    }

    // ════════════════════════════════════════════════════════════════════════════
    // A3 — the cascade decision is RELATIVE, not absolute (plan Assertion 13).
    //
    // A uniform additive constant applied to EVERY arm's stored bias must leave the
    // flip-tick INVARIANT, because the production flip condition is the relative
    // comparison `delta_target > BIAS_FLIP_THRESHOLD + delta_natural`
    // (agent_decision.rs) — adding the same constant to `delta_target` and
    // `delta_natural` cancels. The baseline flip-tick is additionally anchored to a
    // hardcoded literal so a regression that reads bias as an absolute magnitude is
    // caught.
    //
    // Scenario: a single Idle agent with Hunger AND Thirst both above threshold (50)
    // — natural winner = Hunger (priority 0). Two synthetic Thirst-arm memory
    // entries (valence 1.0, salience 1.0, encoded_tick 0) bias the cascade toward
    // Thirst, with `delta_Thirst = 2 * recency_factor(0, t)`. Driving ONLY
    // `AgentDecisionSystem` (no MemorySystem) keeps salience pinned at 1.0, so only
    // the linear recency factor decays with the current tick:
    //
    //   flip while  2 * (1 - t / MAX_RECENCY_TICKS) > 1.0   ⇔   t < MAX/2 = 2190.
    //   flip-tick (first revert from Seeking{Water} to natural Seeking{Food}) = 2190.
    //
    // The uniform-shift run adds one entry per arm (identical valence/salience/
    // encoded_tick), so every arm's delta gains the SAME +0.5*recency each tick —
    // which cancels in the relative comparison, leaving the flip-tick unchanged.
    // ════════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_fix_d_a3_cascade_flip_is_relative_uniform_shift_invariant() {
        // Locked expected baseline flip-tick, measured against THIS commit at the
        // locked levers: MAX_RECENCY_TICKS (4380) / 2 == 2190. Pinned to the
        // INTEGER flip-tick (never the f64 bias sum) to avoid float `==` fragility.
        const EXPECTED_FLIP_TICK: u64 = MAX_RECENCY_TICKS / 2;
        // Scan horizon: the bias hits 0 at MAX_RECENCY_TICKS, well past the flip.
        const HORIZON: u64 = MAX_RECENCY_TICKS;

        // Drive a controlled single-agent cascade and return the integer tick at
        // which the chosen Seeking arm flips from the memory-biased target (Water)
        // back to the natural winner (Food). `uniform_shift` adds one identical
        // bias entry per arm (the uniform additive shift over every arm).
        fn flip_tick(uniform_shift: bool) -> u64 {
            let mut e = SimEngine::new(W, H, MaterialRegistry::new());
            let ent = e.spawn_agent(20, 20);
            e.world
                .insert_one(ent, Memory::new())
                .expect("attach Memory");
            {
                let mut mem = e.world.get::<&mut Memory>(ent).expect("memory present");
                // Two Thirst-arm bias entries → delta_Thirst = 2 * recency.
                mem.insert(MemoryEntry::new(1, 0, 1.0, 1.0, MemoryArm::Thirst));
                mem.insert(MemoryEntry::new(2, 0, 1.0, 1.0, MemoryArm::Thirst));
                if uniform_shift {
                    // One entry per arm, identical (valence, salience, encoded_tick)
                    // → each arm's delta gains the SAME +0.5*recency each tick.
                    for (i, arm) in [
                        MemoryArm::Hunger,
                        MemoryArm::Thirst,
                        MemoryArm::Fatigue,
                        MemoryArm::Construction,
                        MemoryArm::Social,
                        MemoryArm::Combat,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        mem.insert(MemoryEntry::new(100 + i as EventId, 0, 0.5, 1.0, arm));
                    }
                }
            }
            let mut sys = AgentDecisionSystem::new();
            let mut flip: Option<u64> = None;
            for t in 0..HORIZON {
                e.resources.current_tick = t;
                // Clean per-tick cascade evaluation: reset to Idle with both needs
                // pinned above threshold (Hunger natural winner, Thirst eligible
                // flip candidate). Memory persists (not in this component bag).
                e.world
                    .insert(
                        ent,
                        (
                            AgentState::Idle,
                            Hunger::new(80.0, 0.0),
                            Thirst::new(80.0, 0.0),
                        ),
                    )
                    .expect("repin agent state + needs");
                sys.tick(&mut e.world, &mut e.resources);
                let st = e
                    .world
                    .get::<&AgentState>(ent)
                    .map(|s| *s)
                    .unwrap_or(AgentState::Idle);
                // Flipped ⇒ Seeking{Water}; reverted to natural ⇒ Seeking{Food}.
                if matches!(
                    st,
                    AgentState::Seeking {
                        target: TargetKind::Food
                    }
                ) {
                    flip = Some(t);
                    break;
                }
            }
            flip.expect("A3: the cascade must eventually revert to the natural Food arm")
        }

        let baseline = flip_tick(false);
        let shifted = flip_tick(true);
        println!(
        "[fix-d A3] flip_tick baseline={baseline} uniform_shift={shifted} expected={EXPECTED_FLIP_TICK}"
    );
        assert_eq!(
        shifted, baseline,
        "A3: a uniform additive bias shift across every arm must leave the flip-tick INVARIANT \
         (the cascade compares biases relatively); baseline={baseline} shifted={shifted}"
    );
        assert_eq!(
            baseline, EXPECTED_FLIP_TICK,
            "A3: baseline flip-tick must equal the locked expected {EXPECTED_FLIP_TICK} \
         (MAX_RECENCY_TICKS/2)"
        );
    }
}

// Added in V8-0 (8.4): the live game's rules are deterministic.
mod baseline {
    //! Two engines built from `sim_test::baseline_engine(seed)` stay
    //! fingerprint-equal tick by tick, different seeds give different worlds,
    //! and the fingerprint is not vacuous (it changes as the world runs).
    use sim_test::baseline_engine;
    use sim_test::probe::fingerprint;

    /// Ticks per day of the baseline calendar (1 tick = 1 simulated minute).
    const DAY: u64 = 1_440;
    /// Fingerprints are compared every this many ticks.
    const STEP: u64 = 60;

    #[test]
    fn baseline_runs_are_identical_tick_for_tick_and_seeds_differ() {
        for seed in [0_u64, 1] {
            let mut a = baseline_engine(seed);
            let mut b = baseline_engine(seed);
            let start = fingerprint(&a);
            let mut changes = 0_u32;
            let mut last = start.clone();
            for hour in 0..DAY / STEP {
                for _ in 0..STEP {
                    a.tick();
                    b.tick();
                }
                let fa = fingerprint(&a);
                assert_eq!(
                    fa,
                    fingerprint(&b),
                    "seed {seed}: diverged by hour {}",
                    hour + 1
                );
                if fa != last {
                    changes += 1;
                }
                last = fa;
            }
            // The world moves in (almost) every hour of the first day; a
            // fingerprint frozen for more than a quarter of the day would mean
            // the comparison above checked nothing.
            assert!(
                changes >= 18,
                "seed {seed}: fingerprint changed in only {changes} of 24 hours"
            );
        }
        assert_ne!(
            fingerprint(&baseline_engine(0)),
            fingerprint(&baseline_engine(1)),
            "seeds 0 and 1 built the same world"
        );
    }
}
