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
    /// Pass 10 generator inputs of world `seed`: v15's recipe with the
    /// founders, bushes, deadwood and forest of its hidden parameters
    /// (`WorldVariety::draw`). Never redrawn; worldgen accepts every range.
    pub const fn game_v16(seed: u64) -> Self {
        Self::game_v16_with(seed, &sim_core::variety::WorldVariety::draw(seed))
    }

    /// [`Self::game_v16`] for given hidden parameters; `WorldVariety::NEUTRAL`
    /// gives `game_v15(seed)` exactly.
    pub const fn game_v16_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> Self {
        Self {
            starting_agents: variety.starting_agents,
            berry_bushes: variety.berry_bushes,
            berry_capacity: variety.berry_capacity,
            deadwood_spots: variety.deadwood_spots,
            deadwood_capacity: variety.deadwood_capacity,
            forest_coverage: variety.forest_permille as f64 / 1000.0,
            ..Self::game_v15(seed)
        }
    }

    /// Pass 11 generator inputs of world `seed` (`game_v17` and
    /// `game_v17_nofission`): v16's world plus the wild regions of its hidden
    /// parameters.
    pub const fn game_v17(seed: u64) -> Self {
        Self::game_v17_with(seed, &sim_core::variety::WorldVariety::draw(seed))
    }

    /// [`Self::game_v17`] for given hidden parameters.
    pub const fn game_v17_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> Self {
        Self {
            wild_regions: variety.wild_regions as u32,
            region_bushes: variety.region_bushes,
            region_deadwood: variety.region_deadwood,
            ..Self::game_v16_with(seed, variety)
        }
    }

    /// Pass 12 generator inputs of world `seed` (`game_v18` and
    /// `game_v18_nofission`): v17's world from `WorldVariety::draw_v18`, with
    /// regions that deliver what they promise (`region_fill`) and centres
    /// near water (`region_water_max`).
    pub const fn game_v18(seed: u64) -> Self {
        Self::game_v18_with(seed, &sim_core::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v18`] for given hidden parameters.
    pub const fn game_v18_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> Self {
        // Section 7.3 (a), DAUGHTER_DIAGNOSIS.md (b): centres within 12 of water.
        Self { region_fill: true, region_water_max: Some(12), ..Self::game_v17_with(seed, variety) }
    }

    /// Pass 13 generator inputs of world `seed` (`game_v19` and
    /// `game_v19_nofission`): v18's world (no worldgen change).
    pub const fn game_v19(seed: u64) -> Self {
        Self::game_v18(seed)
    }

    /// [`Self::game_v19`] for given hidden parameters.
    pub const fn game_v19_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> Self {
        Self::game_v18_with(seed, variety)
    }

    /// Pass 16 generator inputs of world `seed` (`game_v20` and
    /// `game_v20_inert`): v19's world (no worldgen change).
    pub const fn game_v20(seed: u64) -> Self {
        Self::game_v19(seed)
    }

    /// [`Self::game_v20`] for given hidden parameters.
    pub const fn game_v20_with(seed: u64, variety: &sim_core::variety::WorldVariety) -> Self {
        Self::game_v19_with(seed, variety)
    }

    /// Pass 17 routine world: unchanged Pass 16 generation inputs.
    pub const fn game_v21(seed: u64) -> Self {
        Self::game_v20(seed)
    }

    /// Pass 17 family world: unchanged routine-world generation inputs.
    pub const fn game_v22(seed: u64) -> Self {
        Self::game_v21(seed)
    }

    /// Pass 18 family world: unchanged `game_v22` generation inputs.
    pub const fn game_v23(seed: u64) -> Self {
        Self::game_v22(seed)
    }

    /// Pass 19 staggered-day-end world: unchanged `game_v23` generation inputs.
    pub const fn game_v24(seed: u64) -> Self {
        Self::game_v23(seed)
    }

    /// Pass 20 relations world: unchanged `game_v24` generation inputs.
    pub const fn game_v25(seed: u64) -> Self {
        Self::game_v24(seed)
    }

    /// Revitalize Pass 15 stress world: [`Self::game_v19`]`(seed)` with
    /// `agents` founders on a `side × side` map (run it under
    /// `WorldRules::game_v19(seed)`). Benchmark-only; not a game scene; never
    /// used by the recorder or the probe's scenes.
    pub const fn stress_v19(seed: u64, agents: u32, side: u32) -> Self {
        Self { starting_agents: agents, width: side, height: side, ..Self::game_v19(seed) }
    }

    /// Pass 9 generator inputs: the same world as v14 (no worldgen change).
    pub const fn game_v15(seed: u64) -> Self {
        Self::game_v14(seed)
    }

    /// Pass 8 generator inputs: the same world as v13 (no worldgen change).
    pub const fn game_v14(seed: u64) -> Self {
        Self::game_v13(seed)
    }

    /// Pass 7 generator inputs: the same world as v12 (no worldgen change).
    pub const fn game_v13(seed: u64) -> Self {
        Self::game_v12(seed)
    }

    /// Pass 6 generator inputs: the same world as v11 (no worldgen change).
    pub const fn game_v12(seed: u64) -> Self {
        Self::game_v11(seed)
    }

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
            wild_regions: 0,
            region_bushes: 0,
            region_deadwood: 0,
            region_fill: false,
            region_water_max: None,
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
    fn game_v12_is_the_v11_world() {
        for seed in [0, 5, DEFAULT_GAME_SEED] {
            assert_eq!(WorldGenConfig::game_v12(seed), WorldGenConfig::game_v11(seed));
        }
    }
    #[test]
    fn game_v14_is_the_v13_world() {
        for seed in [0, 5, DEFAULT_GAME_SEED] {
            assert_eq!(WorldGenConfig::game_v14(seed), WorldGenConfig::game_v13(seed));
        }
    }
    #[test]
    fn game_v13_is_the_v12_world() {
        for seed in [0, 5, DEFAULT_GAME_SEED] {
            assert_eq!(WorldGenConfig::game_v13(seed), WorldGenConfig::game_v12(seed));
        }
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
    #[test]
    fn stress_v19_changes_only_the_founders_and_the_map() {
        // Pass 15 6.2.
        for seed in [0, 7, 15100] {
            let base = WorldGenConfig::game_v19(seed);
            let stress = WorldGenConfig::stress_v19(seed, 500, 64);
            assert_eq!(stress.starting_agents, 500);
            assert_eq!(WorldGenConfig { starting_agents: base.starting_agents, ..stress }, base);
            assert_eq!(WorldGenConfig::stress_v19(seed, base.starting_agents, 64), base);
            let wide = WorldGenConfig::stress_v19(seed, base.starting_agents, 128);
            assert_eq!(WorldGenConfig { width: 64, height: 64, ..wide }, base);
        }
    }
    #[test]
    fn stress_v19_world_starts_with_the_requested_founders() {
        let engine = build_world_with(&WorldGenConfig::stress_v19(15100, 500, 64), WorldRules::game_v19(15100));
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(engine.world.query::<&sim_core::components::Agent>().iter().count(), 500);
    }
}
