//! Harness tests for Phase 2 (T7.6) influence systems.
//!
//! 15 plan assertions (all Type A) + 3 supplemental assertions:
//!   - harness_agent_sample_per_agent_tile  (plan A9:  per-agent tile indexing)
//!   - harness_viz_fire_count_20_ticks      (plan A11: 4 fires at 0,6,12,18)
//!   - harness_viz_tick_field               (plan A14: digest.tick captured correctly)
//!
//! Run via: `cargo test -p sim-test harness_ -- --nocapture`

use hecs::World;
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::{
    AgentInfluenceSampleSystem, BuildingStampSystem, InfluenceUpdateSystem,
    InfluenceVisualizationSystem,
};
use sim_systems::runtime::influence::agent_sample::{InfluenceSample, Position};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

fn make_engine(w: u32, h: u32) -> SimEngine {
    SimEngine::new(w, h, MaterialRegistry::new())
}

// ─── Test-only helper: fire counter for scheduling assertions ─────────────────
//
// Implements RuntimeSystem with tick_interval=6. The engine fires this at
// ticks where `current_tick % 6 == 0`. Over 20 ticks that is 0,6,12,18 = 4.
// Arc<AtomicU32> is Send+Sync, so FireCounter satisfies `Box<dyn RuntimeSystem + Send>`.

struct FireCounter(Arc<AtomicU32>);

impl RuntimeSystem for FireCounter {
    fn name(&self) -> &str {
        "FireCounter"
    }
    fn priority(&self) -> u32 {
        // Run alongside InfluenceVisualizationSystem (1000); offset so they
        // don't collide if both are registered in the same engine.
        1001
    }
    fn tick_interval(&self) -> u64 {
        6
    }
    fn tick(&mut self, _world: &mut World, _resources: &mut SimResources) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

// ─── A1: InfluenceUpdateSystem name ──────────────────────────────────────────

/// Type A: system.name() == "InfluenceUpdateSystem"
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_influence_update_name() {
    // metric: system.name(), threshold: == "InfluenceUpdateSystem"
    let s = InfluenceUpdateSystem::new();
    assert_eq!(s.name(), "InfluenceUpdateSystem");
}

// ─── A2: InfluenceUpdateSystem priority ──────────────────────────────────────

/// Type A: system.priority() == 100
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_influence_update_priority() {
    // metric: system.priority(), threshold: == 100
    let s = InfluenceUpdateSystem::new();
    assert_eq!(s.priority(), 100u32);
}

// ─── A3: InfluenceUpdateSystem tick_interval ─────────────────────────────────

/// Type A: system.tick_interval() == 1
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_influence_update_tick_interval() {
    // metric: system.tick_interval(), threshold: == 1
    let s = InfluenceUpdateSystem::new();
    assert_eq!(s.tick_interval(), 1u64);
}

// ─── A4: BuildingStampSystem priority ────────────────────────────────────────

/// Type A: BuildingStampSystem.priority() == 90 (must be < 100 = InfluenceUpdateSystem)
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_building_stamp_priority() {
    // metric: system.priority(), threshold: == 90
    let s = BuildingStampSystem::new();
    assert_eq!(s.priority(), 90u32);
}

// ─── A5: InfluenceUpdateSystem clear-before-swap (anti-hollow) ───────────────

/// Type A: write pending=200, swap (→current=200), tick InfluenceUpdateSystem → sample==0
/// Hollow stub failure mode: empty tick body leaves current==200, assertion fails.
/// ticks: 1 (manual) | components_read: InfluenceGrid current[Beauty]
///
/// T7.10.E relaxation: switched from Spiritual to Beauty.
/// Warmth/Light/Noise/Danger/Spiritual now have persistence semantics
/// (current→pending copy on idle tick) so sample==200 after a tick is CORRECT
/// T7.10.A..E behavior, not a hollow-stub signal.
/// Beauty remains on the Phase 2 dispatch shell: IUS calls clear_pending(Beauty)
/// then swap() → current[Beauty] receives the cleared buffer → sample==0. ✓
#[test]
fn harness_influence_update_clear_before_swap() {
    // metric: sample(10,10,Beauty) after write→swap→tick
    // threshold: == 0  (system clears pending THEN swaps; current receives the cleared buffer)
    let mut e = make_engine(32, 32);
    let buf = e
        .resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::Beauty);
    buf[10 * 32 + 10] = 200;
    // Swap: pending(200) → current, old current(0) → pending
    e.resources.influence_grid.swap();
    // Tick InfluenceUpdateSystem: clear_pending(Beauty)→0, swap (current[Beauty]←0)
    let mut s = InfluenceUpdateSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        e.resources
            .influence_grid
            .sample(10, 10, InfluenceChannel::Beauty),
        0u8
    );
}

// ─── A6: BuildingStampSystem tick_interval ───────────────────────────────────

/// Type A: BuildingStampSystem.tick_interval() == 1
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_building_stamp_tick_interval() {
    // metric: system.tick_interval(), threshold: == 1
    let s = BuildingStampSystem::new();
    assert_eq!(s.tick_interval(), 1u64);
}

// ─── A7: BuildingStampSystem phantom-stamp isolation (anti-hollow) ────────────

/// Type A: write pending=200, tick BuildingStampSystem ONLY, swap → current==200
/// Isolation rationale: if InfluenceUpdate also ran it would clear pending → masking
/// any phantom write. Testing Stamp alone makes phantom stamps detectable.
/// Hollow stub failure mode: stub erroneously clears pending → current==0 after swap.
/// ticks: 1 (manual BuildingStampSystem only) | components_read: InfluenceGrid current[Warmth]
#[test]
fn harness_building_stamp_isolation() {
    // metric: sample(5,5,Warmth) after write→tick(Stamp only)→swap
    // threshold: == 200  (shell MUST NOT clear or modify pending)
    let mut e = make_engine(32, 32);
    let buf = e
        .resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::Warmth);
    buf[5 * 32 + 5] = 200;
    // Tick BuildingStampSystem only — must not touch pending
    let mut s = BuildingStampSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    // Manual swap: pending(200) → current
    e.resources.influence_grid.swap();
    assert_eq!(
        e.resources
            .influence_grid
            .sample(5, 5, InfluenceChannel::Warmth),
        200u8
    );
}

// ─── A8: AgentInfluenceSampleSystem reads current buffer (anti-hollow) ────────

/// Type A: write pending=123, swap (→current=123), tick sampler → agent.warmth==123
/// Hollow stub failure mode: empty tick body leaves warmth==0 (sampler never read grid).
/// ticks: 1 (manual) | components_read: Position, InfluenceSample, InfluenceGrid current
#[test]
fn harness_agent_sample_reads_current() {
    // metric: InfluenceSample.warmth after write→swap→tick(AgentSampler)
    // threshold: == 123
    let mut e = make_engine(32, 32);
    let buf = e
        .resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::Warmth);
    buf[5 * 32 + 5] = 123;
    // Swap: value now in current
    e.resources.influence_grid.swap();
    let id = e
        .world
        .spawn((Position { x: 5, y: 5 }, InfluenceSample::default()));
    let mut s = AgentInfluenceSampleSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    // metric: agent.warmth
    let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
    assert_eq!(sample.warmth, 123u8);
}

// ─── A9: AgentInfluenceSampleSystem OOB position ignored ──────────────────────

/// Type A: OOB position (9999,9999) must not panic and must leave sample unchanged.
/// ticks: 1 (manual) | components_read: Position, InfluenceSample
#[test]
fn harness_agent_sample_oob_ignored() {
    // metric: InfluenceSample unchanged after tick with OOB position
    // threshold: warmth==42, danger==7 (values set before tick, must be preserved)
    let mut e = make_engine(32, 32);
    let id = e.world.spawn((
        Position { x: 9999, y: 9999 },
        InfluenceSample {
            warmth: 42,
            danger: 7,
        },
    ));
    let mut s = AgentInfluenceSampleSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
    assert_eq!(sample.warmth, 42u8);
    assert_eq!(sample.danger, 7u8);
}

// ─── Plan A9: AgentInfluenceSampleSystem per-agent tile indexing (anti-hollow) ─

/// Type A: 3 agents at distinct tiles must each read their OWN tile's value.
/// Hollow stub failure mode: all agents read the same tile (e.g. idx[0]) →
/// 2 of 3 asserts fail.
/// ticks: 1 (manual) | components_read: Position, InfluenceSample, InfluenceGrid Danger
#[test]
fn harness_agent_sample_per_agent_tile() {
    // metric: InfluenceSample.danger per agent at distinct tiles
    // threshold: agent_a.danger==10, agent_b.danger==20, agent_c.danger==30
    let mut e = make_engine(32, 32);
    {
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Danger);
        buf[32 + 1] = 10;
        buf[2 * 32 + 2] = 20;
        buf[3 * 32 + 3] = 30;
    }
    e.resources.influence_grid.swap();
    let a = e
        .world
        .spawn((Position { x: 1, y: 1 }, InfluenceSample::default()));
    let b = e
        .world
        .spawn((Position { x: 2, y: 2 }, InfluenceSample::default()));
    let c = e
        .world
        .spawn((Position { x: 3, y: 3 }, InfluenceSample::default()));
    let mut s = AgentInfluenceSampleSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    assert_eq!(e.world.get::<&InfluenceSample>(a).unwrap().danger, 10u8);
    assert_eq!(e.world.get::<&InfluenceSample>(b).unwrap().danger, 20u8);
    assert_eq!(e.world.get::<&InfluenceSample>(c).unwrap().danger, 30u8);
}

// ─── A10: InfluenceVisualizationSystem priority ───────────────────────────────

/// Type A: system.priority() == 1000
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_viz_priority() {
    // metric: system.priority(), threshold: == 1000
    let s = InfluenceVisualizationSystem::new();
    assert_eq!(s.priority(), 1000u32);
}

// ─── A11: InfluenceVisualizationSystem exact aggregation (anti-hollow) ─────────

/// Type A: Warmth pending[0]=50,[1]=70 → warmth_total==120;
///         Danger pending[0]=200,[10]=100 → danger_peak==200
/// Anti-channel-swap: if Warmth/Danger buffers transposed, danger_peak=max(50,70)=70 ≠ 200.
/// Hollow stub failure mode: empty tick leaves warmth_total==0, danger_peak==0.
/// ticks: 1 (manual, direct instantiation) | components_read: InfluenceGrid current[Warmth,Danger]
#[test]
fn harness_viz_warmth_total_danger_peak() {
    // metric: last_digest().warmth_total, threshold: == 120
    // metric: last_digest().danger_peak,  threshold: == 200
    let mut e = make_engine(32, 32);
    // Write Warmth: [0]=50, [1]=70 → sum=120
    {
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        buf[0] = 50;
        buf[1] = 70;
    }
    // Write Danger: [0]=200, [10]=100 → peak=200
    {
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Danger);
        buf[0] = 200;
        buf[10] = 100;
    }
    // Swap: values now in current
    e.resources.influence_grid.swap();
    // Direct instantiation (not boxed) so last_digest() is accessible
    let mut s = InfluenceVisualizationSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    assert_eq!(s.last_digest().warmth_total, 120u64);
    assert_eq!(s.last_digest().danger_peak, 200u8);
}

// ─── A12: InfluenceVisualizationSystem tick_interval ─────────────────────────

/// Type A: system.tick_interval() == 6
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_viz_tick_interval() {
    // metric: system.tick_interval(), threshold: == 6
    let s = InfluenceVisualizationSystem::new();
    assert_eq!(s.tick_interval(), 6u64);
}

// ─── Plan A11: InfluenceVisualizationSystem fire-count over 20 ticks ──────────
//
// Documents tick-0 semantics: is_multiple_of(6) is true at tick 0, so the
// system fires at 0, 6, 12, 18 — exactly 4 times across 20 engine ticks.

/// Type A: FireCounter (interval=6) fires exactly 4 times across 20 engine ticks.
/// ticks: 20 (engine) | components_read: AtomicU32 counter via FireCounter
#[test]
fn harness_viz_fire_count_20_ticks() {
    // metric: number of FireCounter ticks over 20 engine ticks
    // threshold: == 4  (fires at ticks 0, 6, 12, 18)
    let counter = Arc::new(AtomicU32::new(0));
    let mut engine = make_engine(32, 32);
    engine.register_system(Box::new(FireCounter(counter.clone())));
    for _ in 0..20 {
        engine.tick();
    }
    assert_eq!(
        counter.load(Ordering::SeqCst),
        4u32,
        "interval=6 must fire exactly 4 times in 20 ticks (at 0, 6, 12, 18)"
    );
}

// ─── A13: register_phase2_systems → 4 systems ────────────────────────────────

/// Type A: register_phase2_systems registers exactly 4 systems.
/// ticks: 0 | components_read: SimEngine::system_count()
#[test]
fn harness_register_phase2_count() {
    // metric: engine.system_count(), threshold: == 4
    let mut engine = make_engine(64, 64);
    register_phase2_systems(&mut engine);
    assert_eq!(engine.system_count(), 4);
}

// ─── A14: register_phase2_systems → correct priority order ───────────────────

/// Type A: system_names() must reflect priority sort: 90 < 100 < 110 < 1000
/// ticks: 0 | components_read: SimEngine::system_names()
#[test]
fn harness_register_phase2_order() {
    // metric: engine.system_names() in execution order
    // threshold: ["BuildingStampSystem","InfluenceUpdateSystem","AgentInfluenceSampleSystem","InfluenceVisualizationSystem"]
    let mut engine = make_engine(64, 64);
    register_phase2_systems(&mut engine);
    assert_eq!(
        engine.system_names(),
        vec![
            "BuildingStampSystem",
            "InfluenceUpdateSystem",
            "AgentInfluenceSampleSystem",
            "InfluenceVisualizationSystem",
        ]
    );
}

// ─── Plan A14: InfluenceVisualizationSystem tick field in digest ───────────────

/// Type A: last_digest().tick must equal resources.current_tick at the moment tick() fires.
/// Hollow stub failure mode: tick field left at 0 regardless of actual tick number.
/// ticks: 0 (manual, direct instantiation) | components_read: VisualizationDigest.tick
#[test]
fn harness_viz_tick_field() {
    // metric: last_digest().tick after manual tick at current_tick=6 then =12
    // threshold: digest.tick==6, then digest.tick==12
    let mut e = make_engine(32, 32);
    let mut s = InfluenceVisualizationSystem::new();

    // Simulate tick 6
    e.resources.current_tick = 6;
    s.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        s.last_digest().tick,
        6u64,
        "digest.tick must capture current_tick=6"
    );

    // Simulate tick 12
    e.resources.current_tick = 12;
    s.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        s.last_digest().tick,
        12u64,
        "digest.tick must capture current_tick=12"
    );
}

// ─── A15: 10 ticks → all channels zero (baseline) ────────────────────────────

/// Type A: no sources registered → all influence channels must remain zero after ticks.
/// ticks: 10 | components_read: InfluenceGrid current (all 8 channels)
#[test]
fn harness_phase2_baseline_zero() {
    // metric: sample(x,y,ch) for corners + center, all channels
    // threshold: == 0 for all
    let mut engine = make_engine(64, 64);
    register_phase2_systems(&mut engine);
    for _ in 0..10 {
        engine.tick();
    }
    for ch in InfluenceChannel::all() {
        assert_eq!(
            engine.resources.influence_grid.sample(0, 0, *ch),
            0u8,
            "channel {ch:?} at (0,0)"
        );
        assert_eq!(
            engine.resources.influence_grid.sample(32, 32, *ch),
            0u8,
            "channel {ch:?} at (32,32)"
        );
        assert_eq!(
            engine.resources.influence_grid.sample(63, 63, *ch),
            0u8,
            "channel {ch:?} at (63,63)"
        );
    }
}
