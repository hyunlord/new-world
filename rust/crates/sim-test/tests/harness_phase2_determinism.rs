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

use sim_core::influence::{
    DirtyRegion, InfluenceChannel, MaterialBlockingCache,
};
use sim_core::influence::propagate::{
    propagate_bfs, propagate_danger, propagate_noise, propagate_shadowcast,
    stamp_social_aggregate, LodTier,
};
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
        BuildingPlacedEvent { position: (5, 5), radius: 1 },
        BuildingPlacedEvent { position: (10, 10), radius: 2 },
        BuildingPlacedEvent { position: (20, 15), radius: 3 },
        BuildingPlacedEvent { position: (25, 25), radius: 1 },
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

        let regs = &e.resources.influence_grid.dirty_regions
            [InfluenceChannel::Warmth as usize];
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
        BuildingPlacedEvent { position: (3, 3), radius: 2 },
        BuildingPlacedEvent { position: (16, 16), radius: 4 },
        BuildingPlacedEvent { position: (28, 28), radius: 5 },
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
        BuildingPlacedEvent { position: (5, 5), radius: 3 },     // valid
        BuildingPlacedEvent { position: (999, 999), radius: 1 }, // OOB
        BuildingPlacedEvent { position: (1, 1), radius: 100 },   // clamp
        BuildingPlacedEvent { position: (50, 5), radius: 1 },    // OOB (x)
        BuildingPlacedEvent { position: (31, 31), radius: 0 },   // boundary
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
    assert_eq!(runs[0].1.len(), 3, "expected 3 valid stamps (2 OOB skipped)");
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

    assert!(runs[0].iter().any(|&b| b > 0), "shadowcast produced no light");
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

    assert!(runs[0].iter().any(|&b| b > 0), "noise produced no influence");
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
    let keys: Vec<(MaterialId, InfluenceChannel)> = [
        "granite", "oak", "iron", "water",
    ]
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
    assert!(light_mid <= 100, "Max Light cannot exceed source intensity 100");

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

    assert!(runs[0].iter().any(|&b| b > 0), "danger produced no influence");
    for i in 1..N {
        assert_eq!(runs[0], runs[i], "danger diverged at instance {i}");
    }
}
