//! Seeded Revitalize worlds; legacy fixtures never call this module.
mod camp;
mod noise;
mod terrain;
use sim_core::{rules::WorldRules, MaterialRegistry};
use sim_engine::SimEngine;
/// Seed used by the live game unless reset explicitly.
pub const DEFAULT_GAME_SEED: u64 = 20_261_010;
/// Deterministic world generation inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldGenConfig {
    /// Seed for independent purpose-salted streams.
    pub seed: u64,
    /// Width in tiles; the live game uses 64.
    pub width: u32,
    /// Height in tiles; the live game uses 64.
    pub height: u32,
    /// Initial residents.
    pub starting_agents: u32,
    /// Number of finite, renewable food sources.
    pub berry_bushes: u32,
    /// Maximum food at each bush.
    pub berry_capacity: u8,
    /// Forest fraction of ALL non-river tiles (not just lowland).
    pub forest_coverage: f64,
    /// Hill fraction of all non-river tiles.
    pub hill_coverage: f64,
    /// Mountain fraction of all non-river tiles.
    pub mountain_coverage: f64,
    /// Deadwood spots placed after every other step (Pass 5); zero places none.
    pub deadwood_spots: u32,
    /// Maximum wood at each deadwood spot.
    pub deadwood_capacity: u8,
    /// Innermost Chebyshev distance of a deadwood spot from the hearth.
    pub deadwood_min_radius: u32,
    /// Outermost Chebyshev distance of a deadwood spot from the hearth.
    pub deadwood_max_radius: u32,
}
impl WorldGenConfig {
    /// Pass 5 generator inputs: v10 plus twenty capacity-six deadwood spots
    /// 5..=18 tiles from the hearth, drawn from their own stream.
    pub const fn game_v11(seed: u64) -> Self {
        Self {
            deadwood_spots: 20,
            deadwood_capacity: 6,
            deadwood_min_radius: 5,
            deadwood_max_radius: 18,
            ..Self::game_v10(seed)
        }
    }

    /// Pass 4 generator inputs: v9 with twelve renewable food sources.
    pub const fn game_v10(seed: u64) -> Self {
        Self { berry_bushes: 12, ..Self::game_v9(seed) }
    }

    /// Versioned generator inputs with ten renewable food sources.
    pub const fn game_v9(seed: u64) -> Self {
        Self { berry_bushes: 10, ..Self::game_default(seed) }
    }

    /// 64×64, 24 residents, 8 capacity-four bushes, 30/10/4 percent terrain.
    pub const fn game_default(seed: u64) -> Self {
        Self {
            seed,
            width: 64,
            height: 64,
            starting_agents: 24,
            berry_bushes: 8,
            berry_capacity: 4,
            forest_coverage: 0.30,
            hill_coverage: 0.10,
            mountain_coverage: 0.04,
            deadwood_spots: 0,
            deadwood_capacity: 0,
            deadwood_min_radius: 0,
            deadwood_max_radius: 0,
        }
    }
}
/// Construct a complete seeded world at tick zero.
///
/// Coverage fractions share the non-river tile count denominator and round to
/// the nearest whole tile. Requires dimensions at least 32 and valid fractions.
pub fn build_world(cfg: &WorldGenConfig) -> SimEngine {
    build_world_with(cfg, WorldRules::game_v8())
}

/// Construct a seeded world with an explicitly selected rules version.
pub fn build_world_with(cfg: &WorldGenConfig, rules: WorldRules) -> SimEngine {
    assert!(
        cfg.width >= 32 && cfg.height >= 32,
        "world must have room for the camp and food rings"
    );
    let fractions = [
        cfg.forest_coverage,
        cfg.hill_coverage,
        cfg.mountain_coverage,
    ];
    assert!(
        fractions.iter().all(|&f| (0.0..=1.0).contains(&f)) && fractions.iter().sum::<f64>() <= 1.0,
        "terrain fractions must fit non-river land"
    );
    assert!(
        cfg.berry_capacity > 0 && cfg.berry_capacity < u8::MAX,
        "berries require a finite positive capacity"
    );
    let mut engine = SimEngine::new(cfg.width, cfg.height, MaterialRegistry::new());
    crate::register_default_runtime_systems(&mut engine);
    engine.resources.world_seed = cfg.seed;
    engine.resources.rules = rules;
    engine.resources.time_of_day = (engine.resources.rules.clock_offset_ticks
        % engine.resources.ticks_per_day) as f64
        / engine.resources.ticks_per_day as f64
        * 24.0;
    terrain::generate(cfg, &mut engine.resources.tile_grid);
    camp::shore_water(&mut engine.resources);
    camp::populate(cfg, &mut engine);
    // Pass 5: deadwood comes last, from its own stream, so the rest of a v11
    // world is identical to the v10 world of the same seed.
    if cfg.deadwood_spots > 0 {
        assert!(
            cfg.deadwood_capacity > 0 && cfg.deadwood_min_radius <= cfg.deadwood_max_radius,
            "deadwood requires a positive capacity and an ordered radius band"
        );
        camp::deadwood(cfg, &mut engine.resources);
    }
    engine
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_world_rules_are_retained_without_enabling_future_state() {
        let config = WorldGenConfig::game_default(7);
        let rules = WorldRules::default();
        let engine = build_world_with(&config, rules);
        assert_eq!(engine.resources.rules, rules);
        assert!(engine.resources.building_owner.is_empty());
        assert!(engine.resources.bed_of.is_empty());
        assert_eq!(
            engine
                .world
                .query::<&sim_core::components::Child>()
                .iter()
                .count(),
            0
        );
    }
    #[test]
    fn game_v11_adds_only_deadwood_to_v10() {
        let v11 = WorldGenConfig::game_v11(5);
        assert_eq!(
            (v11.deadwood_spots, v11.deadwood_capacity, v11.deadwood_min_radius, v11.deadwood_max_radius),
            (20, 6, 5, 18)
        );
        let plain = WorldGenConfig { deadwood_spots: 0, deadwood_capacity: 0, deadwood_min_radius: 0, deadwood_max_radius: 0, ..v11 };
        assert_eq!(plain, WorldGenConfig::game_v10(5));
        assert_eq!(WorldGenConfig::game_v10(5).deadwood_spots, 0);
    }
    #[test]
    fn default_configuration_is_explicit() {
        let cfg = WorldGenConfig::game_default(DEFAULT_GAME_SEED);
        assert_eq!(
            (
                cfg.width,
                cfg.height,
                cfg.starting_agents,
                cfg.berry_bushes,
                cfg.berry_capacity
            ),
            (64, 64, 24, 8, 4)
        );
    }
}
