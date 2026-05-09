//! T7.6 Harness Integration Tests — Phase 2 influence RuntimeSystems.
//!
//! 15 assertions, all Type A (physical/mathematical invariants).
//! Written TEST-FIRST (RED) before implementation.

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::{
    AgentInfluenceSampleSystem, BuildingStampSystem, InfluenceUpdateSystem,
    InfluenceVisualizationSystem,
};
use sim_systems::runtime::influence::agent_sample::{InfluenceSample, Position};

fn make_engine(w: u32, h: u32) -> SimEngine {
    SimEngine::new(w, h, MaterialRegistry::new())
}

// ─── A1: InfluenceUpdateSystem name ────────────────────────────────────────

/// Type A: system.name() == "InfluenceUpdateSystem"
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_influence_update_name() {
    // metric: system.name(), threshold: == "InfluenceUpdateSystem"
    let s = InfluenceUpdateSystem::new();
    assert_eq!(s.name(), "InfluenceUpdateSystem");
}

// ─── A2: InfluenceUpdateSystem priority ────────────────────────────────────

/// Type A: system.priority() == 100
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_influence_update_priority() {
    // metric: system.priority(), threshold: == 100
    let s = InfluenceUpdateSystem::new();
    assert_eq!(s.priority(), 100u32);
}

// ─── A3: InfluenceUpdateSystem tick_interval ───────────────────────────────

/// Type A: system.tick_interval() == 1
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_influence_update_tick_interval() {
    // metric: system.tick_interval(), threshold: == 1
    let s = InfluenceUpdateSystem::new();
    assert_eq!(s.tick_interval(), 1u64);
}

// ─── A4: BuildingStampSystem priority ──────────────────────────────────────

/// Type A: BuildingStampSystem.priority() == 90 (must be < 100 = InfluenceUpdateSystem)
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_building_stamp_priority() {
    // metric: system.priority(), threshold: == 90
    let s = BuildingStampSystem::new();
    assert_eq!(s.priority(), 90u32);
}

// ─── A5: InfluenceUpdateSystem clear-before-swap (anti-hollow) ─────────────

/// Type A: write pending=200, swap (→current=200), tick InfluenceUpdateSystem → sample==0
/// Hollow stub failure mode: empty tick leaves current==200, assertion fails.
/// ticks: 1 (manual) | components_read: InfluenceGrid current[Warmth]
#[test]
fn harness_influence_update_clear_before_swap() {
    // metric: sample(10,10,Warmth) after write→swap→tick
    // threshold: == 0  (system clears pending THEN swaps, so current gets the cleared buffer)
    let mut e = make_engine(32, 32);
    // Write into pending
    let buf = e.resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth);
    buf[10 * 32 + 10] = 200;
    // Swap: pending(200) → current, old current(0) → pending
    e.resources.influence_grid.swap();
    // Tick InfluenceUpdateSystem: clear_all_pending (pending→0), swap (current←0)
    let mut s = InfluenceUpdateSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        e.resources.influence_grid.sample(10, 10, InfluenceChannel::Warmth),
        0u8
    );
}

// ─── A6: BuildingStampSystem tick_interval ─────────────────────────────────

/// Type A: BuildingStampSystem.tick_interval() == 1
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_building_stamp_tick_interval() {
    // metric: system.tick_interval(), threshold: == 1
    let s = BuildingStampSystem::new();
    assert_eq!(s.tick_interval(), 1u64);
}

// ─── A7: BuildingStampSystem phantom-stamp isolation (anti-hollow) ──────────

/// Type A: write pending=200, tick BuildingStampSystem ONLY, swap → current==200
/// Hollow stub failure mode: if stub erroneously clears pending, current==0 after swap.
/// ticks: 1 (manual BuildingStampSystem) | components_read: InfluenceGrid current[Warmth]
#[test]
fn harness_building_stamp_isolation() {
    // metric: sample(5,5,Warmth) after write→tick(Stamp only)→swap
    // threshold: == 200  (shell must NOT clear or modify pending)
    let mut e = make_engine(32, 32);
    let buf = e.resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth);
    buf[5 * 32 + 5] = 200;
    // Tick BuildingStampSystem only — must not touch pending
    let mut s = BuildingStampSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    // Manual swap: pending(200) → current
    e.resources.influence_grid.swap();
    assert_eq!(
        e.resources.influence_grid.sample(5, 5, InfluenceChannel::Warmth),
        200u8
    );
}

// ─── A8: AgentInfluenceSampleSystem reads current (anti-hollow) ────────────

/// Type A: write pending=123, swap (→current=123), tick sampler → agent.warmth==123
/// Hollow stub failure mode: empty tick leaves warmth==0 (sampler never read grid).
/// ticks: 1 (manual) | components_read: Position, InfluenceSample, InfluenceGrid current
#[test]
fn harness_agent_sample_reads_current() {
    // metric: InfluenceSample.warmth after write→swap→tick(AgentSampler)
    // threshold: == 123
    let mut e = make_engine(32, 32);
    let buf = e.resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth);
    buf[5 * 32 + 5] = 123;
    // Swap: value now in current
    e.resources.influence_grid.swap();
    // Spawn agent at (5,5)
    let id = e.world.spawn((Position { x: 5, y: 5 }, InfluenceSample::default()));
    // Tick sampler
    let mut s = AgentInfluenceSampleSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    // metric: agent.warmth
    let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
    assert_eq!(sample.warmth, 123u8);
}

// ─── A9: AgentInfluenceSampleSystem OOB position ignored ───────────────────

/// Type A: OOB position (9999,9999) must not panic and must leave sample unchanged.
/// ticks: 1 (manual) | components_read: Position, InfluenceSample
#[test]
fn harness_agent_sample_oob_ignored() {
    // metric: InfluenceSample unchanged after tick with OOB position
    // threshold: warmth==42, danger==7 (values set before tick, must be preserved)
    let mut e = make_engine(32, 32);
    let id = e.world.spawn((
        Position { x: 9999, y: 9999 },
        InfluenceSample { warmth: 42, danger: 7 },
    ));
    let mut s = AgentInfluenceSampleSystem::new();
    s.tick(&mut e.world, &mut e.resources);
    let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
    assert_eq!(sample.warmth, 42u8);
    assert_eq!(sample.danger, 7u8);
}

// ─── A10: InfluenceVisualizationSystem priority ─────────────────────────────

/// Type A: system.priority() == 1000
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_viz_priority() {
    // metric: system.priority(), threshold: == 1000
    let s = InfluenceVisualizationSystem::new();
    assert_eq!(s.priority(), 1000u32);
}

// ─── A11: InfluenceVisualizationSystem exact aggregation (anti-hollow) ──────

/// Type A: Warmth pending[0]=50,[1]=70 → warmth_total==120; Danger pending[0]=200,[10]=100 → danger_peak==200
/// Anti-swap: if channels are swapped, danger_peak=max(50,70)=70 ≠ 200 — caught.
/// Hollow stub failure mode: empty tick leaves warmth_total==0, danger_peak==0.
/// ticks: 1 (manual direct instantiation) | components_read: InfluenceGrid current[Warmth,Danger]
#[test]
fn harness_viz_warmth_total_danger_peak() {
    // metric: last_digest().warmth_total, threshold: == 120
    // metric: last_digest().danger_peak,  threshold: == 200
    let mut e = make_engine(32, 32);
    // Write Warmth: [0]=50, [1]=70 → sum=120
    {
        let buf = e.resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth);
        buf[0] = 50;
        buf[1] = 70;
    }
    // Write Danger: [0]=200, [10]=100 → peak=200
    {
        let buf = e.resources.influence_grid.pending_buf_mut(InfluenceChannel::Danger);
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

// ─── A12: InfluenceVisualizationSystem tick_interval ───────────────────────

/// Type A: system.tick_interval() == 6
/// ticks: 0 | components_read: RuntimeSystem metadata
#[test]
fn harness_viz_tick_interval() {
    // metric: system.tick_interval(), threshold: == 6
    let s = InfluenceVisualizationSystem::new();
    assert_eq!(s.tick_interval(), 6u64);
}

// ─── A13: register_phase2_systems → 4 systems ──────────────────────────────

/// Type A: register_phase2_systems registers exactly 4 systems.
/// ticks: 0 | components_read: SimEngine::system_count()
#[test]
fn harness_register_phase2_count() {
    // metric: engine.system_count(), threshold: == 4
    let mut engine = make_engine(64, 64);
    register_phase2_systems(&mut engine);
    assert_eq!(engine.system_count(), 4);
}

// ─── A14: register_phase2_systems → correct priority order ─────────────────

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

// ─── A15: 10 ticks → all channels zero (baseline) ──────────────────────────

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
