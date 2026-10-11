//! Shared, frozen simulation fixtures used by probes and integration tests.
use sim_engine::SimEngine;

/// The shipped legacy world, including the three former renderer boot buildings.
pub fn legacy_scene() -> SimEngine {
    let mut engine = sim_bridge::ffi::world_node::init_production_engine();
    for x in [32, 24, 40] {
        sim_bridge::ffi::world_node::enqueue_building_placed(&mut engine.resources, x, 32, 8);
    }
    engine
}

/// Seeded game fixture shared by instrumentation and integration tests.
pub fn game_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world(&sim_systems::worldgen::WorldGenConfig::game_default(seed))
}

/// Versioned game fixture for opt-in village, food, and rhythm rules.
pub fn game_v9_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v9(seed),
        sim_core::rules::WorldRules::game_v9(),
    )
}

/// Pass 4 fixture: twelve bushes with v10 foraging, aging and season rules.
pub fn game_v10_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v10(seed),
        sim_core::rules::WorldRules::game_v10(),
    )
}

/// Pass 5 fixture: v10 plus deadwood, meals, firewood and gathering preemption.
pub fn game_v11_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v11(seed),
        sim_core::rules::WorldRules::game_v11(),
    )
}

/// Pass 6 fixture: the v11 world under the v12 rules (autumn wood, lean winter).
pub fn game_v12_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v12(seed),
        sim_core::rules::WorldRules::game_v12(),
    )
}

/// Pass 7 fixture: the v12 world under the v13 rules (winters of drawn
/// severity, claims bounded by what a spot holds).
pub fn game_v13_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v13(seed),
        sim_core::rules::WorldRules::game_v13(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn game_scene_preserves_seed_and_starts_with_twenty_four_agents() {
        let engine = super::game_scene(7);
        assert_eq!(engine.resources.world_seed, 7);
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(
            engine
                .world
                .query::<&sim_core::components::Agent>()
                .iter()
                .count(),
            24
        );
    }

    #[test]
    fn legacy_boot_buildings_are_queued_without_advancing_time() {
        let engine = super::legacy_scene();
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(engine.resources.building_event_queue.len(), 3);
    }
}
