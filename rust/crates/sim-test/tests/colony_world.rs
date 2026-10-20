//! M1 Part 3: the colony world (goal 8.1, 8.5; ADR 0014) — its shape, its
//! viability without orders, and determinism with the U-script.

mod colony_support;

use colony_support::tick_checked;
use sim_core::colony::Command;
use sim_core::components::{Agent, Child, Elder};
use sim_core::TerrainType;
use sim_engine::SimEngine;
use sim_systems::worldgen::colony::hearth;

#[test]
fn colony_world_shape() {
    for seed in [0u64, 5] {
        let engine = sim_test::colony_engine(seed);
        let r = &engine.resources;
        assert_eq!((r.tile_grid.width, r.tile_grid.height), (192, 192));
        assert_eq!(192 % 32, 0, "a multiple of the geology chunk");
        let founders: Vec<_> = engine
            .world
            .query::<(&Agent, Option<&Child>, Option<&Elder>)>()
            .iter()
            .map(|(_, (_, c, e))| c.is_none() && e.is_none())
            .collect();
        assert_eq!(founders.len(), 10);
        assert!(founders.iter().filter(|adult| **adult).count() >= 7);
        let camp = hearth(r).expect("hearth");
        let forest = r
            .tile_grid
            .terrain_type
            .iter()
            .filter(|t| **t == TerrainType::Forest)
            .count();
        // About 30 % of the 36,480 non-river tiles are forest; nearly all carry a tree.
        assert!(
            r.colony.trees.len() > 9_000,
            "{} trees on {forest} forest tiles",
            r.colony.trees.len()
        );
        assert!(r.colony.trees.len() <= forest);
        for (tile, tree) in &r.colony.trees {
            let d = tile.0.abs_diff(camp.0).max(tile.1.abs_diff(camp.1));
            assert!(d > 6, "tree {tile:?} inside the clearing");
            assert!((3..=5).contains(&tree.logs));
            let i = r.tile_grid.idx(tile.0, tile.1);
            assert_eq!(r.tile_grid.terrain_type[i], TerrainType::Forest);
            assert!(!r.food_source_max.contains_key(tile) && !r.wood_source_max.contains_key(tile));
        }
        // Area-scaled sources: 12 + 96 camp-config bushes and 18 regions' worth.
        assert!(
            r.food_source_max.len() >= 108,
            "{} bushes",
            r.food_source_max.len()
        );
        assert!(
            r.wood_source_max.len() >= 180,
            "{} deadwood spots",
            r.wood_source_max.len()
        );
        // 18 asked; the water rule (a centre within 12 of water) keeps centres
        // in the river band: 11–14 placed on seeds 0–7 (M1 DECISIONS).
        assert!(
            r.wild_regions.len() >= 10,
            "{} wild regions",
            r.wild_regions.len()
        );
        assert_eq!(r.worldgen_report.regions_requested, 18);
        assert_eq!(r.rules.agent.move_interval_ticks, 3);
        assert!(r.rules.society.shelter_planning.is_none() && r.rules.society.fission.is_none());
        assert!(r.colony.designations.is_empty() && r.colony.ground_items.is_empty());
        assert_eq!(r.colony.tallies.founder_fixes, 0);
    }
}

#[test]
fn colony_founder_fix_makes_adults_when_short() {
    // Synthetic: mark six of the ten founders as children; the fix must bring
    // the adults back to seven by redrawing the youngest.
    let cfg = sim_systems::worldgen::WorldGenConfig::colony(3);
    let mut engine = sim_test::colony_engine(3);
    let mut ids: Vec<(u64, hecs::Entity)> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, a)| (a.id, e))
        .collect();
    ids.sort();
    for (_, e) in ids.iter().take(6) {
        let _ = engine.world.insert_one(*e, Child { born_tick: 0 });
    }
    let gen = cfg.colony.expect("colony gen");
    let fixed = sim_systems::worldgen::colony::ensure_adults(&cfg, &gen, &mut engine);
    assert_eq!(fixed, 3);
    let children = engine.world.query::<&Child>().iter().count();
    assert_eq!(children, 3);
}

fn deaths(engine: &SimEngine) -> u64 {
    let t = &engine.resources.tallies;
    t.deaths_starvation + t.deaths_dehydration + t.deaths_exposure + t.deaths_combat
}

#[test]
fn colony_viability_no_orders() {
    for seed in 0u64..8 {
        let mut engine = sim_test::colony_engine(seed);
        for _ in 0..10 * 1_440 {
            engine.tick();
        }
        assert_eq!(
            deaths(&engine),
            0,
            "seed {seed}: deaths in 10 days without orders"
        );
    }
}

/// The U-script's three orders (goal 10.1), resolved for this world.
pub fn u_script(engine: &SimEngine) -> Vec<Command> {
    sim_test::probe::colony::u_script_commands(engine)
}

#[test]
fn colony_determinism_with_the_u_script() {
    let run = |seed: u64| {
        let mut engine = sim_test::colony_engine(seed);
        let mut prints = Vec::new();
        for tick in 0..2 * 1_440u64 {
            if tick == 360 {
                for cmd in u_script(&engine) {
                    sim_systems::commands::submit(&mut engine, cmd);
                }
            }
            tick_checked(&mut engine);
            if tick % 240 == 0 {
                prints.push(sim_test::probe::colony::colony_fingerprint(&engine));
            }
        }
        prints
    };
    for seed in [0u64, 4] {
        let a = run(seed);
        assert_eq!(a, run(seed), "seed {seed}");
    }
}
