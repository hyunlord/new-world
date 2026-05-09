//! Harness tests for Phase 2 (T7.6) influence systems.
//!
//! Cross-crate integration: sim-core + sim-engine + sim-systems.
//! Run via `cargo test -p sim-test harness_ -- --nocapture`.

use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_phase2_systems;

fn empty_registry() -> MaterialRegistry {
    MaterialRegistry::new()
}

#[test]
fn harness_phase2_systems_registered() {
    let mut engine = SimEngine::new(64, 64, empty_registry());
    register_phase2_systems(&mut engine);

    assert_eq!(engine.system_count(), 4, "Phase 2 must register 4 systems");

    let names = engine.system_names();
    assert_eq!(
        names,
        vec![
            "BuildingStampSystem",
            "InfluenceUpdateSystem",
            "AgentInfluenceSampleSystem",
            "InfluenceVisualizationSystem",
        ],
        "Priority order must be 90 → 100 → 110 → 1000"
    );
}

#[test]
fn harness_phase2_tick_loop_stable() {
    let mut engine = SimEngine::new(64, 64, empty_registry());
    register_phase2_systems(&mut engine);

    for _ in 0..50 {
        engine.tick();
    }

    assert_eq!(engine.current_tick(), 50);
    assert_eq!(engine.system_count(), 4);
}
