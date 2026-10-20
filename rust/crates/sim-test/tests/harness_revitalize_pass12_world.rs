//! Pass 12 Part 2: regions that deliver, daughters near water, and food
//! availability local to a member's catchment (PASS12_GOAL.md 7.2-7.4).
//!
//! V8-0: rebased from the `game_v17` / `game_v18` worlds on the baseline
//! (`sim_test::baseline_engine`, = `game_v26`, whose worldgen keeps v18's
//! region fill and water rule) and `WorldRules::baseline`; the v17 arms are
//! gone.
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Position, Settlement, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use sim_test::baseline_engine;

#[test]
fn harness_region_fill_delivers() {
    let cfg = WorldGenConfig::baseline(0);
    assert!(cfg.region_fill && cfg.region_water_max == Some(12));
    // Baseline worlds over seeds 0..128: realized bushes and deadwood >= 0.95
    // of requested, the fill supplying part of it.
    let reports: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8u64)
            .map(|part| {
                scope.spawn(move || {
                    (part * 16..part * 16 + 16)
                        .map(|s| baseline_engine(s).resources.worldgen_report)
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("worldgen thread"))
            .collect()
    });
    let sum = |f: fn(&sim_engine::WorldGenReport) -> u32| reports.iter().map(f).sum::<u32>();
    let (bushes, bushes_req) = (
        sum(|r| r.region_bushes_placed),
        sum(|r| r.region_bushes_requested),
    );
    let (wood, wood_req) = (
        sum(|r| r.region_deadwood_placed),
        sum(|r| r.region_deadwood_requested),
    );
    let fill_bushes = sum(|r| r.region_fill_bushes.iter().sum());
    let fill_wood = sum(|r| r.region_fill_deadwood);
    println!(
        "baseline seeds 0..128: bushes {bushes}/{bushes_req} (fill {fill_bushes}), \
         deadwood {wood}/{wood_req} (fill {fill_wood})"
    );
    assert!(f64::from(bushes) >= 0.95 * f64::from(bushes_req));
    assert!(f64::from(wood) >= 0.95 * f64::from(wood_req));
    // Measured: bushes 2519/2519 with 940 from the fill, deadwood 3692/3716
    // with 129 from the fill; without the fill the bushes would fall to 63 %
    // of requested. The bars are half the measured fill.
    assert!(fill_bushes >= 470, "fill bushes {fill_bushes}");
    assert!(fill_wood >= 64, "fill deadwood {fill_wood}");
}

#[test]
fn harness_region_fill_never_touches_terrain_or_water() {
    // The regions (and their fill) are placed after everything else: the same
    // world without wild regions (a count, 0) has the same terrain, water and
    // buildings, and differs exactly by the regions' bushes and deadwood.
    for seed in [0u64, 5, 24, 77] {
        let full = WorldGenConfig::baseline(seed);
        let bare = WorldGenConfig {
            wild_regions: 0,
            ..full
        };
        let (a, b) = (
            build_world_with(&full, WorldRules::baseline(seed)),
            build_world_with(&bare, WorldRules::baseline(seed)),
        );
        assert_eq!(
            a.resources.tile_grid.terrain_type, b.resources.tile_grid.terrain_type,
            "terrain never edited"
        );
        assert_eq!(a.resources.water_tiles, b.resources.water_tiles);
        assert_eq!(a.resources.building_kinds, b.resources.building_kinds);
        for tile in a.resources.food_source_max.keys() {
            assert!(
                !a.resources.water_tiles.contains_key(tile)
                    && !a.resources.building_kinds.contains_key(tile)
            );
            assert!(
                !a.resources.wood_source_max.contains_key(tile),
                "a bush on a deadwood spot"
            );
        }
        let report = a.resources.worldgen_report;
        assert!(report.regions_placed >= 1, "seed {seed}");
        assert_eq!(
            a.resources.food_source_max.len(),
            b.resources.food_source_max.len() + report.region_bushes_placed as usize,
            "seed {seed}"
        );
        assert_eq!(
            a.resources.wood_source_max.len(),
            b.resources.wood_source_max.len() + report.region_deadwood_placed as usize,
            "seed {seed}"
        );
        assert!(report.region_fill_bushes.iter().sum::<u32>() <= report.region_bushes_placed);
    }
}

#[test]
fn harness_region_water_rule() {
    for seed in 0..64 {
        let engine = baseline_engine(seed);
        for region in &engine.resources.wild_regions {
            assert!(
                region.water_distance <= 12,
                "seed {seed}: centre {:?} water at {}",
                region.centre,
                region.water_distance
            );
        }
    }
}

/// One hungry member (hunger 70) of a settlement at (20, 20) with an empty
/// store, standing 15 tiles out at (35, 20), Idle at 12:00; full bushes at
/// (50, 50) and (37, 20), both outside the catchment of 16 (Chebyshev 30 and
/// 17 from the hearth), the second two tiles from the member; with
/// `local_bush` also a bush at (30, 22) inside the catchment (7 tiles away).
fn hungry(rules: WorldRules, local_bush: bool) -> AgentState {
    hungry_with_target(rules, local_bush).0
}

/// [`hungry`] with the member's meal target (its `SeekTarget`, if any).
fn hungry_with_target(rules: WorldRules, local_bush: bool) -> (AgentState, Option<(u32, u32)>) {
    hungry_founder(rules, local_bush, false)
}

/// [`hungry_with_target`], optionally with the member walking with a fission group (`Founding`).
fn hungry_founder(
    rules: WorldRules,
    local_bush: bool,
    founder: bool,
) -> (AgentState, Option<(u32, u32)>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.current_tick = 1440 + 720 - 480;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    let entity = engine.spawn_agent(35, 20);
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(70.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Inventory::default(),
                MovementRng::new(1),
            ),
        )
        .expect("components");
    settlement.add_member_agent(id);
    if founder {
        engine
            .world
            .insert_one(
                entity,
                sim_core::components::Founding {
                    site: (50, 50),
                    fission_id: 1,
                    leader: id,
                    since_tick: 0,
                },
            )
            .expect("founding");
    }
    engine.resources.settlements.insert(1, settlement);
    engine.resources.set_food_tile(50, 50, 4);
    engine.resources.set_food_source_max(50, 50, 4);
    if local_bush {
        engine.resources.set_food_tile(30, 22, 2);
        engine.resources.set_food_source_max(30, 22, 4);
    }
    engine.resources.set_water_tile(35, 30, 255);
    engine.resources.set_food_tile(37, 20, 4);
    engine.resources.set_food_source_max(37, 20, 4);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let state = *engine.world.get::<&AgentState>(entity).expect("state");
    let target = engine
        .world
        .get::<&sim_core::components::SeekTarget>(entity)
        .ok()
        .map(|t| t.tile);
    (state, target)
}

#[test]
fn harness_local_food_availability() {
    let seeking_food = AgentState::Seeking {
        target: TargetKind::Food,
    };
    let rules = WorldRules::baseline(0);
    assert!(rules.agent.local_food_availability);
    // Only a bush inside the member's catchment makes the hunger actionable.
    assert_ne!(hungry(rules, false), seeking_food);
    assert_eq!(hungry(rules, true), seeking_food);
    // The meal target is the nearest bush in the catchment, not the nearer
    // one two tiles away outside it.
    let (_, target) = hungry_with_target(rules, true);
    assert_eq!(target, Some((30, 22)), "eats inside its catchment");
    // Review round 1, M1: a founder still walking keeps the world-wide rule.
    let (state, target) = hungry_founder(rules, false, true);
    assert_eq!(state, seeking_food);
    assert_eq!(target, Some((37, 20)));
}

#[test]
fn harness_local_gather_stays_in_the_catchment() {
    use sim_systems::runtime::decision::agent_decision::{
        nearest_food_tile_in_radius, nearest_gather_tile,
    };
    let mut tiles = std::collections::HashMap::new();
    tiles.insert((37u32, 20u32), 4u8); // 17 from the hearth (20, 20): outside the catchment of 16
    tiles.insert((30, 24), 2); // inside
    let pos = Position::new(35, 20);
    assert_eq!(nearest_food_tile_in_radius(&pos, &tiles, 8), Some((37, 20)));
    assert_eq!(
        nearest_gather_tile(&pos, &tiles, 8, None, &[]),
        Some((37, 20)),
        "no catchment: the nearest in range"
    );
    assert_eq!(
        nearest_gather_tile(&pos, &tiles, 8, Some(((20, 20), 16)), &[]),
        Some((30, 24))
    );
    tiles.remove(&(30, 24));
    assert_eq!(
        nearest_gather_tile(&pos, &tiles, 8, Some(((20, 20), 16)), &[]),
        None
    );
}
