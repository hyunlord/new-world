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
    /// Wild regions placed after every other step (Pass 11); zero places none.
    pub wild_regions: u32,
    /// Bushes asked for in each wild region (Pass 11).
    pub region_bushes: u32,
    /// Deadwood spots asked for in each wild region (Pass 11).
    pub region_deadwood: u32,
    /// Pass 12 (`game_v18`): a region short of its bushes or deadwood is
    /// filled from looser candidate tiers (`camp::regions`). False = v17.
    pub region_fill: bool,
    /// Pass 12 (`game_v18`): a region centre candidate needs a water tile
    /// within this Chebyshev distance. None = v17 (no water rule).
    pub region_water_max: Option<u32>,
}
impl WorldGenConfig {
    /// The live game's generator inputs for world `seed`: one flat literal
    /// equal to `game_v26` at `v7-final-art` (ADR 0011), checked against
    /// `rust/crates/sim-core/tests/data/v7_final_worldgen_seed*.txt`
    /// (`tests/baseline_worldgen_matches_v7_final.rs`). Founders, bushes,
    /// deadwood, forest and wild regions are the world's hidden parameters
    /// (`WorldVariety::draw`); everything else is fixed.
    pub const fn baseline(seed: u64) -> Self {
        let variety = sim_core::variety::WorldVariety::draw(seed);
        Self {
            seed,
            width: 64,
            height: 64,
            starting_agents: variety.starting_agents,
            berry_bushes: variety.berry_bushes,
            berry_capacity: variety.berry_capacity,
            forest_coverage: variety.forest_permille as f64 / 1000.0,
            hill_coverage: 0.1,
            mountain_coverage: 0.04,
            deadwood_spots: variety.deadwood_spots,
            deadwood_capacity: variety.deadwood_capacity,
            deadwood_min_radius: 5,
            deadwood_max_radius: 18,
            wild_regions: variety.wild_regions as u32,
            region_bushes: variety.region_bushes,
            region_deadwood: variety.region_deadwood,
            region_fill: true,
            region_water_max: Some(12),
        }
    }

    /// The benchmark stress world: [`Self::baseline`]`(seed)` with `agents`
    /// founders on a `side × side` map (run it under `WorldRules::baseline`).
    /// Used by `worldsim_perf` (`quick`, `stress`) and the bridge's
    /// `reset_world_stress`; never a game scene.
    pub const fn stress(seed: u64, agents: u32, side: u32) -> Self {
        Self {
            starting_agents: agents,
            width: side,
            height: side,
            ..Self::baseline(seed)
        }
    }
}
/// Construct a complete seeded world at tick zero under `rules`.
///
/// Coverage fractions share the non-river tile count denominator and round to
/// the nearest whole tile. Requires dimensions at least 32 and valid fractions.
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
    // Pass 11: wild regions come after everything else, from their own keyed
    // streams, so the rest of a v17 world is identical to the v16 world.
    if cfg.wild_regions > 0 {
        camp::regions(cfg, &mut engine.resources);
    }
    engine
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_world_rules_are_retained_without_enabling_future_state() {
        let config = WorldGenConfig::baseline(7);
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
    fn stress_world_starts_with_the_requested_founders() {
        let engine = build_world_with(
            &WorldGenConfig::stress(15100, 500, 64),
            WorldRules::baseline(15100),
        );
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(
            engine
                .world
                .query::<&sim_core::components::Agent>()
                .iter()
                .count(),
            500
        );
    }
}
