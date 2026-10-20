//! Pass 12 Part 2: regions that deliver, daughters near water with a winter
//! fall of their own, and rain that burns wet wood (PASS12_GOAL.md 7.2–7.4).
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Position, Settlement, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_core::variety::WorldVariety;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use sim_test::scenes::game_v18_nofission_scene;

fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

fn v18_world(seed: u64) -> SimEngine {
    build_world_with(&WorldGenConfig::game_v18(seed), WorldRules::game_v18(seed))
}

#[test]
fn harness_region_fill_delivers_in_v18_only() {
    // v17 worlds never fill (their digests are pinned in the frozen test).
    for seed in 0..16 {
        let v17 = build_world_with(&WorldGenConfig::game_v17(seed), WorldRules::game_v17(seed));
        let report = v17.resources.worldgen_report;
        assert_eq!((report.region_fill_bushes, report.region_fill_deadwood), ([0; 4], 0), "seed {seed}");
        assert!(!WorldGenConfig::game_v17(seed).region_fill && WorldGenConfig::game_v17(seed).region_water_max.is_none());
    }
    // v18 over seeds 0..128: realized bushes and deadwood >= 0.95 of requested.
    let reports: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8u64)
            .map(|part| scope.spawn(move || (part * 16..part * 16 + 16).map(|s| v18_world(s).resources.worldgen_report).collect::<Vec<_>>()))
            .collect();
        handles.into_iter().flat_map(|h| h.join().expect("worldgen thread")).collect()
    });
    let sum = |f: fn(&sim_engine::WorldGenReport) -> u32| reports.iter().map(f).sum::<u32>();
    let (bushes, bushes_req) = (sum(|r| r.region_bushes_placed), sum(|r| r.region_bushes_requested));
    let (wood, wood_req) = (sum(|r| r.region_deadwood_placed), sum(|r| r.region_deadwood_requested));
    println!("v18 seeds 0..128: bushes {bushes}/{bushes_req}, deadwood {wood}/{wood_req}");
    assert!(f64::from(bushes) >= 0.95 * f64::from(bushes_req));
    assert!(f64::from(wood) >= 0.95 * f64::from(wood_req));
}

#[test]
fn harness_region_fill_never_touches_terrain_or_water() {
    for seed in [0u64, 5, 24, 77] {
        let variety = WorldVariety::draw_v18(seed);
        let filled = WorldGenConfig::game_v18_with(seed, &variety);
        let unfilled = WorldGenConfig { region_fill: false, ..filled };
        let (a, b) = (build_world_with(&filled, WorldRules::game_v18(seed)), build_world_with(&unfilled, WorldRules::game_v18(seed)));
        assert_eq!(a.resources.tile_grid.terrain_type, b.resources.tile_grid.terrain_type, "terrain never edited");
        assert_eq!(a.resources.water_tiles, b.resources.water_tiles);
        assert_eq!(a.resources.building_kinds, b.resources.building_kinds);
        for tile in a.resources.food_source_max.keys() {
            assert!(!a.resources.water_tiles.contains_key(tile) && !a.resources.building_kinds.contains_key(tile));
            assert!(!a.resources.wood_source_max.contains_key(tile), "a bush on a deadwood spot");
        }
        let fill = a.resources.worldgen_report;
        let extra = fill.region_fill_bushes.iter().sum::<u32>();
        assert_eq!(a.resources.worldgen_report.region_bushes_placed, b.resources.worldgen_report.region_bushes_placed + extra);
    }
}

#[test]
fn harness_region_water_rule() {
    for seed in 0..64 {
        let engine = v18_world(seed);
        for region in &engine.resources.wild_regions {
            assert!(region.water_distance <= 12, "seed {seed}: centre {:?} water at {}", region.centre, region.water_distance);
        }
    }
}

/// Wood on the spots of area `area`.
fn area_wood(engine: &SimEngine, area: u8) -> u32 {
    engine
        .resources
        .wood_spot_area
        .iter()
        .filter(|(_, a)| **a == area)
        .map(|(tile, _)| u32::from(engine.resources.wood_tiles.get(tile).copied().unwrap_or(0)))
        .sum()
}

#[test]
fn harness_area_fall_floor() {
    // A v18 world with a settlement (one member) on region 0's centre, run
    // through six winter days of regrowth with every spot emptied after each
    // pulse (as gatherers would; the snow cap of 1 per spot never binds), with
    // the floor and without it. The floor binds where the region has fewer
    // spots than floor x the camp's: the first seed whose region 0 has fewer
    // than 0.6 x the camp's spots is used, with the start value 600 and 1000.
    let spots = |e: &SimEngine, area: u8| e.resources.wood_spot_area.values().filter(|a| **a == area).count() as u64;
    let seed = (0..400u64)
        .find(|&s| {
            let e = game_v18_nofission_scene(s);
            spots(&e, 1) * 1000 < 600 * spots(&e, 0)
        })
        .expect("a world whose region 0 has few spots");
    let day = tick_at(21, 600);
    // Returns (fall placed per area 0..=3 over the six days, the engine).
    let regrow = |floor: u32| {
        let mut engine = game_v18_nofission_scene(seed);
        if let Some(mut firewood) = engine.resources.rules.society.firewood {
            firewood.area_fall_floor_permille = floor;
            engine.resources.rules.society.firewood = Some(firewood);
        }
        let centre = engine.resources.wild_regions[0].centre;
        let mut daughter = Settlement::new_with_id(9, 0);
        daughter.formation_tile = centre;
        daughter.add_member_agent(9_999);
        engine.resources.settlements.insert(9, daughter);
        let all: Vec<_> = engine.resources.wood_source_max.keys().copied().collect();
        let mut placed = [0u32; 4];
        let mut system = ResourceRegenSystem;
        let mut tick = day;
        while tick < day + 6 * 1440 {
            for tile in &all {
                engine.resources.wood_tiles.insert(*tile, 0);
            }
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
            for (area, total) in placed.iter_mut().enumerate() {
                *total += area_wood(&engine, area as u8);
            }
            tick += REGEN_INTERVAL;
        }
        (placed, engine)
    };
    let (without, plain) = regrow(0);
    assert_eq!(plain.resources.tallies.area_floor_wood, 0);
    for floor in [600, 1000] {
        let (with, engine) = regrow(floor);
        assert_eq!(with[0], without[0], "the camp's fall is never reduced (floor {floor})");
        assert!(with[1] > without[1], "the settled region gets more wood (floor {floor}): {with:?} vs {without:?}");
        assert_eq!(with[2..], without[2..], "unsettled regions keep their share");
        assert!(engine.resources.tallies.area_floor_wood > 0);
        println!("floor {floor} (seed {seed}, region spots {} vs camp {}): fall per area {with:?} vs {without:?} without", spots(&engine, 1), spots(&engine, 0));
    }
    // At floor 1000 the settled region receives at least the camp's daily fall
    // less one unit of rounding per day.
    let (full, _) = regrow(1000);
    assert!(full[1] + 6 >= full[0], "{full:?}");
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
fn hungry_founder(rules: WorldRules, local_bush: bool, founder: bool) -> (AgentState, Option<(u32, u32)>) {
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
            .insert_one(entity, sim_core::components::Founding { site: (50, 50), fission_id: 1, leader: id, since_tick: 0 })
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
    let target = engine.world.get::<&sim_core::components::SeekTarget>(entity).ok().map(|t| t.tile);
    (state, target)
}

#[test]
fn harness_local_food_availability_only_in_v18() {
    let seeking_food = AgentState::Seeking { target: TargetKind::Food };
    // v17: a full bush anywhere makes the hunger actionable.
    assert_eq!(hungry(WorldRules::game_v17_nofission(0), false), seeking_food);
    // v18: only a bush inside the member's catchment does.
    assert_ne!(hungry(WorldRules::game_v18_nofission(0), false), seeking_food);
    assert_eq!(hungry(WorldRules::game_v18_nofission(0), true), seeking_food);
    // The meal target: v17 the nearest bush anywhere, v18 the nearest in the catchment.
    let (_, v17_target) = hungry_with_target(WorldRules::game_v17_nofission(0), true);
    let (_, v18_target) = hungry_with_target(WorldRules::game_v18_nofission(0), true);
    assert_eq!(v17_target, Some((37, 20)), "v17 eats at the nearest bush anywhere");
    assert_eq!(v18_target, Some((30, 22)), "v18 eats inside its catchment");
    // Review round 1, M1: a founder still walking keeps the world-wide rule.
    let (state, target) = hungry_founder(WorldRules::game_v18(0), false, true);
    assert_eq!(state, seeking_food);
    assert_eq!(target, Some((37, 20)));
    assert!(!WorldRules::game_v17(0).agent.local_food_availability && WorldRules::game_v18(0).agent.local_food_availability);
}

#[test]
fn harness_local_gather_stays_in_the_catchment() {
    use sim_systems::runtime::decision::agent_decision::{nearest_food_tile_in_radius, nearest_gather_tile};
    let mut tiles = std::collections::HashMap::new();
    tiles.insert((37u32, 20u32), 4u8); // 17 from the hearth (20, 20): outside the catchment of 16
    tiles.insert((30, 24), 2); // inside
    let pos = Position::new(35, 20);
    assert_eq!(nearest_food_tile_in_radius(&pos, &tiles, 8), Some((37, 20)));
    assert_eq!(nearest_gather_tile(&pos, &tiles, 8, None, &[]), Some((37, 20)), "None = v17");
    assert_eq!(nearest_gather_tile(&pos, &tiles, 8, Some(((20, 20), 16)), &[]), Some((30, 24)));
    tiles.remove(&(30, 24));
    assert_eq!(nearest_gather_tile(&pos, &tiles, 8, Some(((20, 20), 16)), &[]), None);
}
