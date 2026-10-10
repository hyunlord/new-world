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
}
impl WorldGenConfig {
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
        }
    }
}
/// Construct a complete seeded world at tick zero.
///
/// Coverage fractions share the non-river tile count denominator and round to
/// the nearest whole tile. Requires dimensions at least 32 and valid fractions.
pub fn build_world(cfg: &WorldGenConfig) -> SimEngine {
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
    engine.resources.rules = WorldRules::game_v8();
    engine.resources.time_of_day =
        (engine.resources.rules.clock_offset_ticks % engine.resources.ticks_per_day) as f64
            / engine.resources.ticks_per_day as f64 * 24.0;
    terrain::generate(cfg, &mut engine.resources.tile_grid);
    camp::shore_water(&mut engine.resources);
    camp::populate(cfg, &mut engine);
    engine
}
#[cfg(test)]
mod tests {
    use super::*;
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
