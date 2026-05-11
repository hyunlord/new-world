//! T7.8 Phase 2 — Criterion benchmarks: assertions B1–B5 (plan_attempt 3).
//!
//! These benchmarks validate Hard Gate 6 thresholds (Phase 0 v0.1.1 §G2).
//! Run via `cargo bench -p sim-test --bench phase2_benchmarks`, NOT cargo test.
//!
//! Assertion mapping:
//!   B1 (A16): BSS-only, 1 event  — median < 1ms
//!   B2 (A17): BSS-only, 1000 events — median < 5ms
//!   B3 (A18): Full 4 systems, 20 agents, 10 events — median < 5ms
//!   B4 (A19): Full 4 systems, 20 agents, Viz aligned to fire — median < 5ms
//!   B5 (A20): Engine construction + 4 systems + 20 agents (no tick) — median < 50ms
//!
//! Fact 10 (isolation): Each benchmark uses `iter_batched` (B1–B4) or `b.iter`
//! (B5) so every Criterion iteration starts with a fresh SimEngine. This prevents
//! dirty_regions from accumulating across iterations (which would create
//! non-stationary measurements).
//!
//! Fact 12 / B4 tick alignment: Plan says "advance counter to 5" but the tick()
//! function uses `current_tick` BEFORE incrementing. To have the MEASURED tick
//! execute at `current_tick=6` (6%6==0, Viz fires), we run 6 setup ticks in the
//! setup closure. Running only 5 would leave current_tick=5 (5%6≠0, Viz silent).
//! This is a plan off-by-one; the correct implementation is 6 setup ticks.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::agent_sample::{InfluenceSample, Position};
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;

// ── B1 (Assertion 16): BSS single-event throughput — median < 1ms ────────────

/// Fresh BSS-only engine per iteration; 1 Warmth event at (32,32) r=3.
/// IUS/AIS/Viz are absent so this isolates raw BSS cost only.
///
/// `iter_batched` ensures each iteration gets a fresh engine:
///   dirty_regions start empty → O(1) append per event (non-stationary prevention).
///
/// Hard Gate 6: median < 1ms
fn bench_b1_bss_single_event(c: &mut Criterion) {
    c.bench_function("phase2/b1_bss_single_event_threshold_1ms", |b| {
        b.iter_batched(
            // Setup closure: fresh BSS-only engine with 1 event queued
            || {
                let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
                engine.register_system(Box::new(BuildingStampSystem::new()));
                engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
                    position: (32, 32),
                    radius: 3,
                });
                engine
            },
            // Measured closure: 1 tick — BSS drains 1 event, marks 4 channels dirty
            |mut engine| {
                engine.tick();
                black_box(engine)
            },
            BatchSize::SmallInput,
        );
    });
}

// ── B2 (Assertion 17): BSS 1000-event throughput — median < 5ms ───────────────

/// Fresh BSS-only engine per iteration; 1000 Warmth events at ((i*7)%64, (i*11)%64).
/// Isolates BSS linear-scan cost at maximum anticipated burst volume.
///
/// `iter_batched` gives a fresh dirty_regions each iteration — prevents O(n) BSS
/// cost from growing with accumulated dirty_region count across iterations.
///
/// Hard Gate 6: median < 5ms
fn bench_b2_bss_1000_events(c: &mut Criterion) {
    c.bench_function("phase2/b2_bss_1000_events_threshold_5ms", |b| {
        b.iter_batched(
            // Setup: BSS-only engine, 1000 events at plan-specified positions
            || {
                let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
                engine.register_system(Box::new(BuildingStampSystem::new()));
                for i in 0u32..1000 {
                    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
                        position: ((i * 7) % W, (i * 11) % H),
                        radius: 1,
                    });
                }
                engine
            },
            // Measured: 1 tick — BSS while-loop drains all 1000 events
            |mut engine| {
                engine.tick();
                black_box(engine)
            },
            BatchSize::SmallInput,
        );
    });
}

// ── B3 (Assertion 18): Full-pipeline 10-event tick throughput — median < 5ms ─

/// Fresh full-pipeline engine (all 4 systems) per iteration; 20 agents; 10 events.
/// Establishes Phase 2 steady-state tick cost for the full BSS→IUS→AIS→Viz chain.
///
/// Fresh engine per iteration (current_tick=0 at measured tick start). Note: on a
/// fresh engine, 0.is_multiple_of(6)==true so Viz fires on this first tick. This
/// is consistent with the plan threshold (<5ms) and the full-chain measurement intent.
///
/// Hard Gate 6: median < 5ms
fn bench_b3_full_pipeline_10_events(c: &mut Criterion) {
    c.bench_function("phase2/b3_full_pipeline_10_events_threshold_5ms", |b| {
        b.iter_batched(
            // Setup: all 4 systems + 20 agents + 10 events
            || {
                let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
                register_phase2_systems(&mut engine);
                for i in 0u32..20 {
                    let x = (i * 3) % W;
                    let y = (i * 5) % H;
                    engine.world.spawn((Position { x, y }, InfluenceSample::default()));
                }
                for i in 0u32..10 {
                    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
                        position: ((i * 6) % W, (i * 7 + 3) % H),
                        radius: 2,
                    });
                }
                engine
            },
            // Measured: 1 tick — full BSS→IUS→AIS→Viz chain
            |mut engine| {
                engine.tick();
                black_box(engine)
            },
            BatchSize::SmallInput,
        );
    });
}

// ── B4 (Assertion 19): Viz-aligned full-pipeline tick — median < 5ms ─────────

/// Fresh full-pipeline engine per iteration; 20 agents; Viz fires on measured tick.
///
/// Tick alignment (Fact 12, with off-by-one correction):
///   The plan says "advance counter to 5" but tick() uses current_tick BEFORE
///   incrementing. After 5 setup ticks, current_tick=5 (5%6≠0, Viz silent).
///   After 6 setup ticks, current_tick=6 (6%6==0, Viz fires). This benchmark
///   uses 6 setup ticks to satisfy the plan's intent that Viz fires on the measured tick.
///
/// The measured tick: current_tick=6, 6%6==0 → Viz performs full O(4096-tile) scan.
/// 1 BSS event is queued before the measured tick to exercise the dirty path.
///
/// Hard Gate 6: median < 5ms
fn bench_b4_viz_aligned_tick(c: &mut Criterion) {
    c.bench_function("phase2/b4_viz_aligned_tick_threshold_5ms", |b| {
        b.iter_batched(
            // Setup: all 4 systems, 20 agents, 6 ticks to align counter, then 1 event
            || {
                let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
                register_phase2_systems(&mut engine);
                for i in 0u32..20 {
                    let x = (i * 3) % W;
                    let y = (i * 5) % H;
                    engine.world.spawn((Position { x, y }, InfluenceSample::default()));
                }
                // Advance tick counter: after 6 calls, current_tick==6.
                // 6%6==0 → Viz will fire on the next (measured) tick.
                // No events queued here — prevents dirty_regions growth during setup.
                for _ in 0..6 {
                    engine.tick();
                }
                // Queue 1 event for the measured tick (exercises BSS→dirty path with Viz)
                engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
                    position: (32, 32),
                    radius: 3,
                });
                engine
            },
            // Measured: current_tick=6, Viz fires (6%6==0), BSS stamps 1 event
            |mut engine| {
                engine.tick();
                black_box(engine)
            },
            BatchSize::SmallInput,
        );
    });
}

// ── B5 (Assertion 20): Engine construction cost — median < 50ms ───────────────

/// Measures the cost of constructing a Phase 2 engine from scratch each iteration:
///   SimEngine::new + register_phase2_systems + 20 agents spawned.
/// No engine.tick() calls — pure construction cost only.
///
/// Uses b.iter (not iter_batched) because construction IS the measured operation;
/// there is no setup state to preserve across iterations.
///
/// Hard Gate 6 §P0-init: engine construction median < 50ms
fn bench_b5_engine_construction(c: &mut Criterion) {
    c.bench_function("phase2/b5_engine_construction_threshold_50ms", |b| {
        b.iter(|| {
            let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
            register_phase2_systems(&mut engine);
            for i in 0u32..20 {
                let x = (i * 3) % W;
                let y = (i * 5) % H;
                engine.world.spawn((Position { x, y }, InfluenceSample::default()));
            }
            black_box(engine)
        });
    });
}

criterion_group!(
    phase2_benches,
    bench_b1_bss_single_event,
    bench_b2_bss_1000_events,
    bench_b3_full_pipeline_10_events,
    bench_b4_viz_aligned_tick,
    bench_b5_engine_construction,
);
criterion_main!(phase2_benches);
