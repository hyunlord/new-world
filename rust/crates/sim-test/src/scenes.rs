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

/// Pass 8 fixture: the v13 world under the v14 rules (nights and days that
/// vary within a winter, windfall on the snow).
pub fn game_v14_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v14(seed),
        sim_core::rules::WorldRules::game_v14(),
    )
}

/// Pass 9 fixture: the v14 world under the v15 rules (keyed random draws,
/// lineage keys, decisions in AgentId order; same balance as v14).
pub fn game_v15_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v15(seed),
        sim_core::rules::WorldRules::game_v15(),
    )
}

/// Pass 10 fixture: the v15 world under the v16_neutral rules (v15 plus
/// exposure: a cold night wounds and health can reach zero).
pub fn game_v16_neutral_scene(seed: u64) -> SimEngine {
    sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v15(seed),
        sim_core::rules::WorldRules::game_v16_neutral(),
    )
}

/// Pass 10 fixture: world `seed` with its hidden parameters
/// (`WorldVariety::draw(seed)`) under the v16 rules.
pub fn game_v16_scene(seed: u64) -> SimEngine {
    game_v16_scene_with(seed, &sim_core::variety::WorldVariety::draw(seed))
}

/// [`game_v16_scene`] for given hidden parameters (recorded in
/// `SimResources::world_variety`, measurement only).
pub fn game_v16_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v16_with(seed, variety),
        sim_core::rules::WorldRules::game_v16_with(variety),
    );
    // Pass 11: v16 has no wild regions; it records slots 0..7 only.
    engine.resources.world_variety = Some(variety.without_regions());
    engine
}

/// Pass 11 fixture: world `seed` with its hidden parameters, wild regions
/// included, under the v17 rules without fission (exposure v2, soaking rain).
pub fn game_v17_nofission_scene(seed: u64) -> SimEngine {
    game_v17_nofission_scene_with(seed, &sim_core::variety::WorldVariety::draw(seed))
}

/// [`game_v17_nofission_scene`] for given hidden parameters.
pub fn game_v17_nofission_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v17_with(seed, variety),
        sim_core::rules::WorldRules::game_v17_nofission_with(variety),
    );
    engine.resources.world_variety = Some(*variety);
    engine
}

/// Pass 11 fixture: world `seed` with its hidden parameters, wild regions
/// included, under the v17 rules (exposure v2, soaking rain, fission).
pub fn game_v17_scene(seed: u64) -> SimEngine {
    game_v17_scene_with(seed, &sim_core::variety::WorldVariety::draw(seed))
}

/// [`game_v17_scene`] for given hidden parameters.
pub fn game_v17_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v17_with(seed, variety),
        sim_core::rules::WorldRules::game_v17_with(variety),
    );
    engine.resources.world_variety = Some(*variety);
    engine
}

/// Pass 12 fixture: world `seed` with its v18 hidden parameters
/// (`WorldVariety::draw_v18`), regions that deliver and centres near water,
/// under the v18 rules without fission (rain burns wet wood, the settled-region
/// winter-fall floor).
pub fn game_v18_nofission_scene(seed: u64) -> SimEngine {
    game_v18_nofission_scene_with(seed, &sim_core::variety::WorldVariety::draw_v18(seed))
}

/// [`game_v18_nofission_scene`] for given hidden parameters.
pub fn game_v18_nofission_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v18_with(seed, variety),
        sim_core::rules::WorldRules::game_v18_nofission_with(variety),
    );
    engine.resources.world_variety = Some(*variety);
    engine
}

/// Pass 12 fixture: world `seed` under the v18 rules (fission as a daily
/// hazard, autumn departures, rain that burns wet wood).
pub fn game_v18_scene(seed: u64) -> SimEngine {
    game_v18_scene_with(seed, &sim_core::variety::WorldVariety::draw_v18(seed))
}

/// [`game_v18_scene`] for given hidden parameters.
pub fn game_v18_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v18_with(seed, variety),
        sim_core::rules::WorldRules::game_v18_with(variety),
    );
    engine.resources.world_variety = Some(*variety);
    engine
}

/// Pass 13 fixture: world `seed` with its v18 hidden parameters under the v19
/// rules without fission (the settled-area floor in its own accumulator).
pub fn game_v19_nofission_scene(seed: u64) -> SimEngine {
    game_v19_nofission_scene_with(seed, &sim_core::variety::WorldVariety::draw_v18(seed))
}

/// [`game_v19_nofission_scene`] for given hidden parameters.
pub fn game_v19_nofission_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v19_with(seed, variety),
        sim_core::rules::WorldRules::game_v19_nofission_with(variety),
    );
    engine.resources.world_variety = Some(*variety);
    engine
}

/// Pass 13 fixture: world `seed` under the v19 rules (fission by accumulated
/// pressure against a hidden threshold).
pub fn game_v19_scene(seed: u64) -> SimEngine {
    game_v19_scene_with(seed, &sim_core::variety::WorldVariety::draw_v18(seed))
}

/// [`game_v19_scene`] for given hidden parameters.
pub fn game_v19_scene_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> SimEngine {
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::game_v19_with(seed, variety),
        sim_core::rules::WorldRules::game_v19_with(variety),
    );
    engine.resources.world_variety = Some(*variety);
    engine
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
