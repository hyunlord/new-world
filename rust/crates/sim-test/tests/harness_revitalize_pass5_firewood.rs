//! Pass 5 Part 3: deadwood, firewood trips, the hearth fire and cold nights.
use hecs::Entity;
use sim_bridge::ffi::collect_resource_snapshot;
use sim_core::components::{
    Agent, AgentState, Cold, Hunger, Inventory, Position, ResourceKind, SeekTarget, Settlement, Sleep,
    Social, TargetKind, Thirst, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_core::TerrainType;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::needs::HungerDecaySystem;
use sim_systems::runtime::resource_regen::ResourceRegenSystem;
use sim_systems::runtime::settlement::firewood::{wood_in_hand, wood_target};
use sim_systems::runtime::settlement::{FireSystem, StockpileDepositSystem};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v10_scene, game_v11_scene};

const HOME: (u32, u32) = (20, 20);

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// Bare v11 world: one settlement at `HOME` holding `wood` in store, `adults`
/// idle satisfied members on the formation tile, and full deadwood spots.
fn wood_fixture(adults: usize, spots: &[(u32, u32)], per_spot: u32, tick: u64, wood: u32) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    let mut rules = WorldRules::game_v11();
    if let Some(firewood) = rules.society.firewood.as_mut() {
        firewood.max_gatherers_per_spot = per_spot;
    }
    engine.resources.rules = rules;
    engine.resources.current_tick = tick;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.store(ResourceKind::Wood, wood);
    let mut entities = Vec::new();
    for index in 0..adults {
        let entity = engine.spawn_agent(HOME.0, HOME.1);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.05),
                    Thirst::new(0.0, 0.08),
                    Sleep::new(0.0, 0.03),
                    Social::new(0.0, 0.04),
                    Inventory::default(),
                    MovementRng::new(index as u64 + 1),
                ),
            )
            .expect("components");
        settlement.add_member_agent(id);
        entities.push(entity);
    }
    engine.resources.settlements.insert(1, settlement);
    for &spot in spots {
        engine.resources.wood_tiles.insert(spot, 6);
        engine.resources.wood_source_max.insert(spot, 6);
    }
    (engine, entities)
}

fn wood_store(engine: &SimEngine) -> u32 {
    engine.resources.settlements[&1].stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0)
}

fn wood_targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
    entities
        .iter()
        .map(|e| {
            let state = *engine.world.get::<&AgentState>(*e).expect("state");
            matches!(state, AgentState::Seeking { target: TargetKind::GatherWood })
                .then(|| engine.world.get::<&SeekTarget>(*e).ok().map(|s| s.tile))
                .flatten()
        })
        .collect()
}

fn hearth(engine: &SimEngine) -> (u32, u32) {
    *engine
        .resources
        .building_kinds
        .iter()
        .find(|(_, kind)| **kind == sim_core::components::BuildingKind::Hearth)
        .expect("hearth")
        .0
}

#[test]
fn harness_deadwood_placement() {
    let chebyshev = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
    for seed in 0..4u64 {
        let v11 = game_v11_scene(seed);
        let r = &v11.resources;
        let center = hearth(&v11);
        assert_eq!(r.wood_source_max.len(), 20, "seed {seed}");
        assert_eq!(r.wood_tiles, r.wood_source_max, "every spot starts full");
        let spots: Vec<_> = r.wood_source_max.keys().copied().collect();
        for (i, &spot) in spots.iter().enumerate() {
            assert!((5..=18).contains(&chebyshev(spot, center)), "seed {seed} spot {spot:?}");
            assert_eq!(r.wood_source_max[&spot], 6);
            let terrain = r.tile_grid.terrain_type[r.tile_grid.idx(spot.0, spot.1)];
            assert!(matches!(terrain, TerrainType::Forest | TerrainType::Plain), "{terrain:?}");
            assert!(!r.water_tiles.contains_key(&spot));
            assert!(!r.building_kinds.contains_key(&spot));
            assert!(!r.food_source_max.contains_key(&spot));
            for &other in &spots[i + 1..] {
                assert!(chebyshev(spot, other) >= 2, "{spot:?} {other:?}");
            }
        }
        // Deterministic per seed; a different seed places differently.
        assert_eq!(game_v11_scene(seed).resources.wood_source_max, r.wood_source_max);
        // The v10 part of the world is untouched.
        let v10 = game_v10_scene(seed);
        assert_eq!(v10.resources.food_tiles, r.food_tiles);
        assert_eq!(v10.resources.food_source_max, r.food_source_max);
        assert_eq!(v10.resources.water_tiles, r.water_tiles);
        assert_eq!(v10.resources.building_kinds, r.building_kinds);
        assert_eq!(v10.resources.tile_grid.terrain_type, r.tile_grid.terrain_type);
        assert!(v10.resources.wood_tiles.is_empty() && v10.resources.wood_source_max.is_empty());
    }
    assert_ne!(game_v11_scene(0).resources.wood_source_max, game_v11_scene(1).resources.wood_source_max);
}

#[test]
fn harness_wood_regen_schedule() {
    let (mut engine, _) = wood_fixture(0, &[(30, 20), (31, 25)], 1, 0, 0);
    engine.resources.wood_tiles.remove(&(30, 20));
    let mut regen = ResourceRegenSystem::new();
    let mut seen = Vec::new();
    for _ in 0..12 {
        regen.tick(&mut engine.world, &mut engine.resources);
        assert!(engine.resources.wood_tiles.values().all(|v| *v > 0), "no zero-valued tile");
        seen.push(engine.resources.wood_tiles.get(&(30, 20)).copied().unwrap_or(0));
    }
    // 500 permille per 120-tick pulse: one unit every second pulse, six a day.
    assert_eq!(seen, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
    assert_eq!(engine.resources.wood_tiles[&(31, 25)], 6, "a full spot stays at its ceiling");
    // Without the firewood rule the wood channel never moves.
    engine.resources.rules = WorldRules::game_v10();
    engine.resources.wood_tiles.clear();
    for _ in 0..4 {
        regen.tick(&mut engine.world, &mut engine.resources);
    }
    assert!(engine.resources.wood_tiles.is_empty());
}

#[test]
fn harness_firewood_trip_and_deposit() {
    let (mut engine, entities) = wood_fixture(1, &[(26, 20)], 1, tick_at(1, 600), 0);
    let gatherer = entities[0];
    let mut movement = AgentMovementSystem::new();
    let mut decision = AgentDecisionSystem::new();
    let mut deposit = StockpileDepositSystem::new();
    let start = engine.resources.current_tick;
    decision.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        *engine.world.get::<&AgentState>(gatherer).expect("state"),
        AgentState::Seeking { target: TargetKind::GatherWood }
    );
    assert_eq!(engine.world.get::<&SeekTarget>(gatherer).expect("target").tile, (26, 20));
    assert_eq!(engine.resources.tallies.wood_trips, 1);
    let mut picked = false;
    for tick in start + 1..start + 400 {
        engine.resources.current_tick = tick;
        movement.tick(&mut engine.world, &mut engine.resources);
        decision.tick(&mut engine.world, &mut engine.resources);
        if tick % 10 == 0 {
            deposit.tick(&mut engine.world, &mut engine.resources);
        }
        let carried = engine.world.get::<&Inventory>(gatherer).expect("inv").get(ResourceKind::Wood);
        picked |= carried == 2;
        if picked && wood_store(&engine) == 2 {
            break;
        }
    }
    assert!(picked, "the gatherer never picked up a wood load");
    assert_eq!(wood_store(&engine), 2, "the load was not deposited");
    assert_eq!(engine.resources.tallies.wood_gathered, 2);
    assert_eq!(engine.resources.wood_tiles[&(26, 20)], 4);
    assert_eq!(engine.world.get::<&Inventory>(gatherer).expect("inv").get(ResourceKind::Wood), 0);
    assert_eq!(engine.resources.tallies.haul_deposits, 1, "a walked-home load counts as a haul");
    let pos = *engine.world.get::<&Position>(gatherer).expect("pos");
    assert!(pos.x.abs_diff(HOME.0).max(pos.y.abs_diff(HOME.1)) <= SETTLEMENT_PROXIMITY_RADIUS);
}

#[test]
fn harness_firewood_stops_at_target() {
    let mut engine = game_v11_scene(2);
    while engine.resources.settlements.is_empty() {
        engine.tick();
    }
    // Move to 10:00 of display day 1 (inside the firewood window).
    while engine.current_tick() < tick_at(1, 600) {
        engine.tick();
    }
    let sid = *engine.resources.settlements.keys().next().expect("village");
    let target = wood_target(&engine.resources).expect("v11 wood target");
    assert!(target > 0.0);
    let surplus = target.ceil() as u32 + 30;
    engine.resources.settlements.get_mut(&sid).expect("village").store(ResourceKind::Wood, surplus);
    let gathering: Vec<_> = engine
        .world
        .query::<&AgentState>()
        .iter()
        .filter(|(_, s)| matches!(s, AgentState::Seeking { target: TargetKind::GatherWood } | AgentState::Consuming { target: TargetKind::GatherWood }))
        .map(|(e, _)| e)
        .collect();
    for entity in gathering {
        engine.world.insert_one(entity, AgentState::Idle).expect("idle");
        let _ = engine.world.remove_one::<SeekTarget>(entity);
    }
    let trips = engine.resources.tallies.wood_trips;
    while engine.current_tick() < tick_at(1, 1020) {
        engine.tick();
        let target = wood_target(&engine.resources).unwrap_or(0.0);
        assert!(wood_in_hand(&engine.world, &engine.resources, sid) as f64 >= target);
    }
    assert_eq!(engine.resources.tallies.wood_trips, trips, "firewood arm fired above the target");
    // Below the target the arm fires again.
    let (mut engine, entities) = wood_fixture(1, &[(26, 20)], 1, tick_at(1, 600), 0);
    let target = wood_target(&engine.resources).expect("target");
    engine.resources.settlements.get_mut(&1).expect("village").store(ResourceKind::Wood, target.ceil() as u32);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![None]);
    engine.resources.settlements.get_mut(&1).expect("village").withdraw(ResourceKind::Wood, 1);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![Some((26, 20))]);
    // Outside the window (09:00) nobody goes.
    let (mut engine, entities) = wood_fixture(1, &[(26, 20)], 1, tick_at(1, 540), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_targets(&engine, &entities), vec![None]);
}

#[test]
fn harness_firewood_claims_cap() {
    let spots = [(30, 20), (20, 31)];
    let (mut engine, entities) = wood_fixture(3, &spots, 1, tick_at(1, 600), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let chosen = wood_targets(&engine, &entities);
    assert_eq!(chosen.iter().filter(|t| t.is_some()).count(), 2, "{chosen:?}");
    for spot in spots {
        assert_eq!(chosen.iter().filter(|t| **t == Some(spot)).count(), 1, "{chosen:?}");
    }
    assert_eq!(engine.resources.tallies.wood_trips, 2);

    let (mut engine, entities) = wood_fixture(4, &spots, 2, tick_at(1, 600), 0);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let chosen = wood_targets(&engine, &entities);
    for spot in spots {
        assert_eq!(chosen.iter().filter(|t| **t == Some(spot)).count(), 2, "{chosen:?}");
    }
}

#[test]
fn harness_firewood_retargets_empty_spot() {
    let spots = [(32, 20), (20, 33)];
    let (mut engine, entities) = wood_fixture(1, &spots, 1, tick_at(1, 600), 0);
    let gatherer = entities[0];
    let mut movement = AgentMovementSystem::new();
    let mut decision = AgentDecisionSystem::new();
    let start = engine.resources.current_tick;
    let mut phase = 0;
    for tick in start..start + 400 {
        engine.resources.current_tick = tick;
        movement.tick(&mut engine.world, &mut engine.resources);
        decision.tick(&mut engine.world, &mut engine.resources);
        let state = *engine.world.get::<&AgentState>(gatherer).expect("state");
        let seek = engine.world.get::<&SeekTarget>(gatherer).ok().map(|s| s.tile);
        if matches!(state, AgentState::Seeking { target: TargetKind::GatherWood }) {
            let target = seek.unwrap_or_else(|| panic!("wood gatherer without a target at tick {tick}"));
            assert!(engine.resources.wood_tiles.contains_key(&target), "seeking empty spot {target:?} at tick {tick}");
        }
        match phase {
            0 if seek == Some(spots[0]) && tick > start + 16 => {
                engine.resources.wood_tiles.remove(&spots[0]);
                phase = 1;
            }
            1 => {
                assert_eq!(engine.world.get::<&SeekTarget>(gatherer).ok().map(|s| s.tile), Some(spots[1]));
                engine.resources.wood_tiles.remove(&spots[1]);
                phase = 2;
            }
            2 => {
                assert_eq!(state, AgentState::Idle, "gatherer must idle when no spot qualifies");
                assert!(engine.world.get::<&SeekTarget>(gatherer).is_err());
                return;
            }
            _ => {}
        }
    }
    panic!("scenario did not complete (phase {phase})");
}

#[test]
fn harness_fire_burns_and_goes_out() {
    // 18:00 on spring day 1: the fire burns two units and is lit.
    let (mut engine, _) = wood_fixture(1, &[], 1, tick_at(1, 1080), 5);
    let mut fire = FireSystem::new();
    fire.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_store(&engine), 3);
    assert!(engine.resources.fires[&1].lit);
    // 02:00 with one unit left: the hour cannot be paid, the fire is out.
    engine.resources.current_tick = tick_at(2, 120);
    engine.resources.settlements.get_mut(&1).expect("village").withdraw(ResourceKind::Wood, 2);
    fire.tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.fires[&1].lit);
    // An empty store at 02:00 stays out.
    fire.tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.fires[&1].lit);
    assert_eq!(wood_store(&engine), 0);
    // Outside the burn window nothing burns and the fire is out.
    let (mut engine, _) = wood_fixture(1, &[], 1, tick_at(1, 720), 5);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_store(&engine), 5);
    assert!(!engine.resources.fires[&1].lit);
    // Over a full spring night the fire burns 2 x 13 = 26 units.
    let (mut engine, _) = wood_fixture(0, &[], 1, tick_at(1, 1080), 100);
    for tick in tick_at(1, 1080)..tick_at(2, 1080) {
        engine.resources.current_tick = tick;
        if tick % 60 == 0 {
            FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        }
    }
    assert_eq!(wood_store(&engine), 74);
}

#[test]
fn harness_fire_cold_raises_hunger() {
    // Display day 21 is the first day of winter (seven-day seasons).
    let winter_night = tick_at(21, 1380);
    let growth = |engine: &mut SimEngine, entity: Entity| {
        let before = engine.world.get::<&Hunger>(entity).expect("hunger").value;
        HungerDecaySystem::new().tick(&mut engine.world, &mut engine.resources);
        engine.world.get::<&Hunger>(entity).expect("hunger").value - before
    };
    // Fire out (no wood) on a winter night: members are cold, hunger x1.5.
    let (mut engine, entities) = wood_fixture(2, &[], 1, winter_night, 0);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.fires[&1].lit);
    for &entity in &entities {
        assert!(engine.world.get::<&Cold>(entity).is_ok());
    }
    assert_eq!(engine.resources.tallies.cold_member_hours, 2);
    let cold = growth(&mut engine, entities[0]);
    assert!((cold - 0.075).abs() < 1e-6, "cold growth {cold}");
    // Fire lit: no marker, growth x1.0.
    let (mut engine, entities) = wood_fixture(2, &[], 1, winter_night, 10);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(engine.resources.fires[&1].lit);
    assert!(engine.world.get::<&Cold>(entities[0]).is_err());
    let warm = growth(&mut engine, entities[0]);
    assert!((warm - 0.05).abs() < 1e-6, "warm growth {warm}");
    // The marker leaves once the fire is lit again or the cold window ends.
    let (mut engine, entities) = wood_fixture(1, &[], 1, winter_night, 0);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Cold>(entities[0]).is_ok());
    engine.resources.current_tick = tick_at(22, 420);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Cold>(entities[0]).is_err());
    // A spring night with the fire out is not cold.
    let (mut engine, entities) = wood_fixture(1, &[], 1, tick_at(1, 1380), 0);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Cold>(entities[0]).is_err());
}

#[test]
fn harness_firewood_ffi_rows() {
    let engine = game_v11_scene(1);
    let deadwood: Vec<_> = collect_resource_snapshot(&engine.resources).into_iter().filter(|r| r.kind == 3).collect();
    assert_eq!(deadwood.len(), 20);
    assert!(deadwood.iter().all(|r| r.amount == 6 && r.max == 6));
    let v10 = game_v10_scene(1);
    assert!(collect_resource_snapshot(&v10.resources).iter().all(|r| r.kind != 3));
}

#[test]
fn harness_firewood_v10_unchanged() {
    for (seed, expected) in ["0e38264bd3abe6ba", "3b0590ec71558a58", "d265695444b8b314", "48120c365d543729"]
        .into_iter()
        .enumerate()
    {
        let mut engine = game_v10_scene(seed as u64);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(fingerprint(&engine), expected, "seed {seed}");
        assert_eq!(engine.resources.tallies.wood_trips, 0);
        assert!(engine.resources.fires.is_empty());
        assert_eq!(engine.world.query::<&Cold>().iter().count(), 0);
    }
}
