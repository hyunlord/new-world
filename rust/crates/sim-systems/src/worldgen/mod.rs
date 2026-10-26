//! Seeded worlds: the frozen `baseline` (V7) world and the M1 `colony` world
//! (ADR 0014); legacy fixtures never call this module.
mod camp;
pub mod colony;
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
    /// M1: the colony world's extra steps (`worldgen::colony`). None = baseline.
    pub colony: Option<ColonyGen>,
}

/// The M1 colony world's extra worldgen inputs (goal 8.1). Counts are the
/// baseline means scaled by area (192² / 64² = 9); see `WorldGenConfig::colony`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColonyGen {
    /// Bushes placed beyond the camp ring (the camp ring keeps `berry_bushes`).
    pub outer_bushes: u32,
    /// Deadwood spots placed beyond the camp band (the band keeps `deadwood_spots`).
    pub outer_deadwood: u32,
    /// Chebyshev radius around the hearth with no trees.
    pub clearing_radius: u32,
    /// Outermost Chebyshev distance of a wild-region centre from the hearth
    /// (baseline: `camp::REGION_MAX_DISTANCE`, 28).
    pub region_max_distance: u32,
    /// Founders that must be adults (goal 8.1: 7 of 10).
    pub min_adults: u32,
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
            colony: None,
        }
    }

    /// The M1 colony world for `seed` (goal 8.1, ADR 0014): 192 × 192 (a
    /// multiple of the 32-tile geology chunk), exactly 10 founders, forest,
    /// hill and mountain at the baseline's mean variety values (fixed, not
    /// drawn), bushes and deadwood at the baseline means × 9 (area) — the camp
    /// ring and band keep the baseline mean (12 bushes, 20 deadwood spots)
    /// and the rest lies beyond them — and the baseline mean of 2 wild
    /// regions × 9 = 18 regions of 10 bushes and 15 deadwood spots, their
    /// centres 22–88 tiles from the hearth. Run it under `WorldRules::colony`.
    pub const fn colony(seed: u64) -> Self {
        Self {
            seed,
            width: 192,
            height: 192,
            starting_agents: 10,
            berry_bushes: 12,
            berry_capacity: 4,
            forest_coverage: 0.3,
            hill_coverage: 0.1,
            mountain_coverage: 0.04,
            deadwood_spots: 20,
            deadwood_capacity: 6,
            deadwood_min_radius: 5,
            deadwood_max_radius: 18,
            wild_regions: 18,
            region_bushes: 10,
            region_deadwood: 15,
            region_fill: true,
            region_water_max: Some(12),
            colony: Some(ColonyGen {
                outer_bushes: 96,
                outer_deadwood: 160,
                clearing_radius: 6,
                region_max_distance: 88,
                min_adults: 7,
            }),
        }
    }

    /// The standing benchmark's colony world (M1.5 goal 6.1, ADR 0016 C3):
    /// [`Self::colony`]`(seed)` on a `side × side` map with `agents` founders,
    /// the bushes and deadwood beyond the camp scaled with the founders
    /// (`outer × agents / 10`; worldgen places as many as the map has
    /// candidates for) so nobody starves inside a benchmark run. Run it under
    /// `WorldRules::colony`; measurement only, never a game scene.
    pub const fn colony_stress(seed: u64, agents: u32, side: u32) -> Self {
        let base = Self::colony(seed);
        let scale = if agents > 10 { agents } else { 10 };
        Self {
            starting_agents: agents,
            width: side,
            height: side,
            colony: Some(ColonyGen {
                outer_bushes: 96 * scale / 10,
                outer_deadwood: 160 * scale / 10,
                clearing_radius: 6,
                region_max_distance: 88,
                min_adults: 7,
            }),
            ..base
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
    build_world_with_registrar(cfg, rules, |engine| {
        #[cfg(feature = "mind_candidate")]
        if engine.resources.rules.colony.is_some() {
            crate::register_colony_mind_runtime_systems(engine);
            return;
        }
        crate::register_default_runtime_systems(engine);
    })
}

/// Production generation with an explicitly legacy decision stack for shadow comparison.
/// Ordinary world selection may change; this observer-only factory keeps the old brain.
#[cfg(feature = "mind_shadow")]
pub fn build_shadow_world_with(cfg: &WorldGenConfig, rules: WorldRules) -> SimEngine {
    build_world_with_registrar(cfg, rules, crate::register_shadow_runtime_systems)
}

fn build_world_with_registrar(
    cfg: &WorldGenConfig,
    rules: WorldRules,
    register: fn(&mut SimEngine),
) -> SimEngine {
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
    engine.resources.world_seed = cfg.seed;
    engine.resources.rules = rules;
    register(&mut engine);
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
    // M1: the colony world's extra steps, last, from their own keyed streams.
    if let Some(gen) = cfg.colony {
        colony::generate(cfg, &gen, &mut engine);
    }
    engine
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_colony_registration_matches_candidate_feature_before_first_tick() {
        let engine = build_world_with(&WorldGenConfig::colony(7), WorldRules::colony(7));
        let names = engine.system_names();
        let candidate = cfg!(feature = "mind_candidate");
        assert_eq!(
            names
                .iter()
                .filter(|name| **name == "ColonyMindSystem")
                .count(),
            usize::from(candidate)
        );
        assert_eq!(
            names
                .iter()
                .filter(|name| **name == "AgentDecisionSystem")
                .count(),
            usize::from(!candidate)
        );
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(engine.resources.world_seed, 7);
        assert_eq!(engine.resources.mind.policy_source.is_some(), candidate);
        assert_eq!(engine.resources.mind.history.is_some(), candidate);
        assert!(engine.resources.mind.running.is_empty());
        if candidate {
            let source = include_str!("../../../sim-core/data/colony/mind.ron");
            assert_eq!(engine.resources.mind.policy_source.as_deref(), Some(source));
            let program = crate::runtime::mind::MindProgram::load(source).unwrap();
            engine
                .resources
                .mind
                .history
                .as_ref()
                .unwrap()
                .check_policy(&program.policy)
                .unwrap();
        }
    }

    #[test]
    fn candidate_feature_does_not_install_mind_in_baseline() {
        let engine = build_world_with(&WorldGenConfig::baseline(7), WorldRules::baseline(7));
        assert!(engine.system_names().contains(&"AgentDecisionSystem"));
        assert!(!engine.system_names().contains(&"ColonyMindSystem"));
        assert!(engine.resources.mind.policy_source.is_none());
        assert!(engine.resources.mind.history.is_none());
    }

    #[test]
    fn registrar_observes_requested_rules_and_seed() {
        let rules = WorldRules::colony(7);
        let engine = build_world_with_registrar(&WorldGenConfig::colony(7), rules, |engine| {
            assert_eq!(engine.resources.rules, WorldRules::colony(7));
            assert_eq!(engine.resources.world_seed, 7);
        });
        assert_eq!(engine.resources.rules, rules);
    }

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
