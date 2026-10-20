//! Pass 4 Part 2: purposeful foraging, hauling home and the store target.
use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Position, ResourceKind, SeekTarget, Settlement, Sleep,
    Social, TargetKind, Thirst, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::food_economy::{stock_in_hand, stock_target};
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v10_scene, game_v9_scene};

/// v9 plus only the Part 2 rules of v10 (forage and stock target).
fn v9_with_forage() -> WorldRules {
    let mut rules = WorldRules::game_v9();
    rules.agent.forage = WorldRules::game_v10().agent.forage;
    rules.society.stock_target = WorldRules::game_v10().society.stock_target;
    rules
}

fn food_stock(engine: &SimEngine) -> u32 {
    engine
        .resources
        .settlements
        .values()
        .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))
        .sum()
}

fn is_gathering(state: AgentState) -> bool {
    matches!(
        state,
        AgentState::Seeking {
            target: TargetKind::GatherFood
        } | AgentState::Consuming {
            target: TargetKind::GatherFood
        }
    )
}

/// Bare world: one settlement at (20, 20) whose idle, satisfied members stand on
/// the formation tile, and stocked bushes; clock at 08:00 inside the forage window.
fn forage_fixture(adults: usize, bushes: &[(u32, u32)], per_bush: u32) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    let mut rules = WorldRules::game_v10();
    if let Some(forage) = rules.agent.forage.as_mut() {
        forage.max_foragers_per_bush = per_bush;
    }
    engine.resources.rules = rules;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    let mut entities = Vec::new();
    for index in 0..adults {
        let entity = engine.spawn_agent(20, 20);
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
    for &(x, y) in bushes {
        engine.resources.set_food_tile(x, y, 4);
        engine.resources.set_food_source_max(x, y, 4);
    }
    (engine, entities)
}

fn targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
    entities
        .iter()
        .map(|e| {
            let state = *engine.world.get::<&AgentState>(*e).expect("state");
            matches!(
                state,
                AgentState::Seeking {
                    target: TargetKind::GatherFood
                }
            )
            .then(|| engine.world.get::<&SeekTarget>(*e).ok().map(|s| s.tile))
            .flatten()
        })
        .collect()
}

#[test]
fn harness_forage_seed8_no_longer_stagnates() {
    // The exact seed-8 map that stagnated for 45 days in Pass 3.
    let mut engine = build_world_with(&WorldGenConfig::game_v9(8), v9_with_forage());
    let (mut max_stock, mut first_birth) = (0, None);
    for tick in 1..=4 * 1440u64 {
        engine.tick();
        if tick <= 3 * 1440 {
            max_stock = max_stock.max(food_stock(&engine));
        }
        if first_birth.is_none() && engine.resources.tallies.births > 0 {
            first_birth = Some(tick);
        }
    }
    assert!(max_stock >= 39, "stock reached only {max_stock} by day 3");
    assert!(first_birth.is_some(), "no birth by day 4");
    assert!(engine.resources.tallies.forage_trips > 0);
    assert!(engine.resources.tallies.haul_deposits > 0);
}

#[test]
fn harness_forage_stops_at_target() {
    let mut engine = game_v10_scene(2);
    while engine.resources.settlements.is_empty() {
        engine.tick();
    }
    let sid = *engine.resources.settlements.keys().next().expect("village");
    let target = stock_target(&engine.world, &engine.resources, sid).expect("v10 target");
    assert!(target > 0.0);
    let surplus = (3.0 * target).ceil() as u32 + 200;
    engine
        .resources
        .settlements
        .get_mut(&sid)
        .expect("village")
        .store(ResourceKind::Food, surplus);
    // Drop trips already underway so only new decisions are observed.
    let gathering: Vec<_> = engine
        .world
        .query::<&AgentState>()
        .iter()
        .filter(|(_, s)| is_gathering(**s))
        .map(|(e, _)| e)
        .collect();
    for entity in gathering {
        engine
            .world
            .insert_one(entity, AgentState::Idle)
            .expect("idle");
        let _ = engine.world.remove_one::<SeekTarget>(entity);
    }
    let trips = engine.resources.tallies.forage_trips;
    for _ in 0..1440 {
        engine.tick();
        let target = stock_target(&engine.world, &engine.resources, sid).unwrap_or(0.0);
        let in_hand = stock_in_hand(&engine.world, &engine.resources, sid) as f64;
        if in_hand >= 2.0 * target {
            let gatherers = engine
                .world
                .query::<&AgentState>()
                .iter()
                .filter(|(_, s)| is_gathering(**s))
                .count();
            assert_eq!(
                gatherers,
                0,
                "gathering at tick {} with {in_hand} >= 2 x {target}",
                engine.current_tick()
            );
        }
    }
    assert_eq!(
        engine.resources.tallies.forage_trips, trips,
        "forage arm fired above the target"
    );
    assert!(stock_in_hand(&engine.world, &engine.resources, sid) as f64 >= 2.0 * target);
}

#[test]
fn harness_haul_home_deposits() {
    let mut engine = game_v10_scene(4);
    while engine.resources.settlements.is_empty() {
        engine.tick();
    }
    let (sid, home, member) = {
        let (sid, settlement) = engine.resources.settlements.iter().next().expect("village");
        let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
        members.sort_unstable();
        (*sid, settlement.formation_tile, members[0])
    };
    let entity = engine
        .world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == member)
        .map(|(e, _)| e)
        .expect("member entity");
    let start = (home.0 + 12, home.1);
    let mut inventory = Inventory::default();
    inventory.add(ResourceKind::Food, 6);
    engine
        .world
        .insert(
            entity,
            (
                Position::new(start.0, start.1),
                AgentState::Idle,
                inventory,
                Hunger::new(0.0, 0.05),
                Thirst::new(0.0, 0.08),
                Sleep::new(0.0, 0.03),
                Social::new(0.0, 0.04),
            ),
        )
        .expect("hauler");
    let _ = engine.world.remove_one::<SeekTarget>(entity);
    let before = food_stock(&engine);
    let deposits = engine.resources.tallies.haul_deposits;
    let limit = 16 * engine.resources.rules.agent.move_interval_ticks + 20;
    let mut arrived = None;
    for elapsed in 1..=limit {
        engine.tick();
        let pos = *engine.world.get::<&Position>(entity).expect("pos");
        if arrived.is_none()
            && pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1)) <= SETTLEMENT_PROXIMITY_RADIUS
        {
            arrived = Some(elapsed);
        }
        if engine
            .world
            .get::<&Inventory>(entity)
            .expect("inv")
            .get(ResourceKind::Food)
            == 0
        {
            break;
        }
    }
    assert!(arrived.is_some(), "hauler never reached the deposit radius");
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(entity)
            .expect("inv")
            .get(ResourceKind::Food),
        0
    );
    assert!(
        food_stock(&engine) >= before + 6,
        "stock {} did not gain the load over {before}",
        food_stock(&engine)
    );
    assert_eq!(engine.resources.tallies.haul_deposits, deposits + 1);
    assert!(engine.resources.settlements.contains_key(&sid));
}

#[test]
fn harness_forage_claims_cap() {
    let bushes = [(30, 20), (20, 31)];
    let (mut engine, entities) = forage_fixture(3, &bushes, 1);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let chosen = targets(&engine, &entities);
    assert_eq!(
        chosen.iter().filter(|t| t.is_some()).count(),
        2,
        "{chosen:?}"
    );
    for bush in bushes {
        assert_eq!(
            chosen.iter().filter(|t| **t == Some(bush)).count(),
            1,
            "{chosen:?}"
        );
    }
    assert_eq!(engine.resources.tallies.forage_trips, 2);

    let (mut engine, entities) = forage_fixture(4, &bushes, 2);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    let chosen = targets(&engine, &entities);
    for bush in bushes {
        assert_eq!(
            chosen.iter().filter(|t| **t == Some(bush)).count(),
            2,
            "{chosen:?}"
        );
    }
}

#[test]
fn harness_forage_retargets_when_bush_empties() {
    let bushes = [(32, 20), (20, 33)];
    let (mut engine, entities) = forage_fixture(1, &bushes, 1);
    let forager = entities[0];
    let mut movement = AgentMovementSystem::new();
    let mut decision = AgentDecisionSystem::new();
    let mut phase = 0;
    for tick in 0..400u64 {
        engine.resources.current_tick = tick;
        movement.tick(&mut engine.world, &mut engine.resources);
        decision.tick(&mut engine.world, &mut engine.resources);
        let state = *engine.world.get::<&AgentState>(forager).expect("state");
        let pos = *engine.world.get::<&Position>(forager).expect("pos");
        let seek = engine
            .world
            .get::<&SeekTarget>(forager)
            .ok()
            .map(|s| s.tile);
        // After every decision pass a gatherer heads to a tile that holds food:
        // a bush emptied since the last tick is replaced within that one tick.
        if matches!(
            state,
            AgentState::Seeking {
                target: TargetKind::GatherFood
            }
        ) {
            let target = seek
                .unwrap_or_else(|| panic!("forager without a target at tick {tick} at {pos:?}"));
            assert!(
                engine.resources.food_tiles.contains_key(&target),
                "seeking empty tile {target:?} at tick {tick}"
            );
        }
        match phase {
            0 if seek == Some(bushes[0]) && tick > 16 => {
                // Empty the claimed bush before the forager arrives.
                engine.resources.set_food_tile(32, 20, 0);
                phase = 1;
            }
            1 => {
                let retargeted = engine
                    .world
                    .get::<&SeekTarget>(forager)
                    .ok()
                    .map(|s| s.tile);
                assert_eq!(
                    retargeted,
                    Some(bushes[1]),
                    "no re-target after the bush emptied"
                );
                engine.resources.set_food_tile(20, 33, 0);
                phase = 2;
            }
            2 => {
                assert_eq!(
                    state,
                    AgentState::Idle,
                    "forager must idle when no bush qualifies"
                );
                assert!(engine.world.get::<&SeekTarget>(forager).is_err());
                return;
            }
            _ => {}
        }
    }
    panic!("scenario did not complete (phase {phase})");
}

#[test]
fn harness_forage_gatherer_whose_tile_vanished_idles_instead_of_freezing() {
    // A gatherer without a target and no food within reach (its tile was taken
    // earlier in the same tick) must not stay frozen in Seeking{GatherFood}.
    let (mut engine, entities) = forage_fixture(1, &[], 1);
    let gatherer = entities[0];
    engine
        .world
        .insert_one(
            gatherer,
            AgentState::Seeking {
                target: TargetKind::GatherFood,
            },
        )
        .expect("state");
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        *engine.world.get::<&AgentState>(gatherer).expect("state"),
        AgentState::Idle
    );
    assert!(engine.world.get::<&SeekTarget>(gatherer).is_err());
}

#[test]
fn harness_haul_home_after_the_forage_window() {
    // Outside the window even one carried unit is hauled home and deposited.
    let mut engine = game_v10_scene(4);
    let end = u64::from(
        WorldRules::game_v10()
            .agent
            .forage
            .expect("forage")
            .end_minute,
    );
    while engine.current_tick() + 480 < end + 10 {
        engine.tick();
    }
    let (home, member) = {
        let settlement = engine
            .resources
            .settlements
            .values()
            .next()
            .expect("village");
        let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
        members.sort_unstable();
        (settlement.formation_tile, members[members.len() - 1])
    };
    let entity = engine
        .world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == member)
        .map(|(e, _)| e)
        .expect("member");
    let mut inventory = Inventory::default();
    inventory.add(ResourceKind::Food, 1);
    engine
        .world
        .insert(
            entity,
            (
                Position::new(home.0, home.1 + 10),
                AgentState::Idle,
                inventory,
                Hunger::new(0.0, 0.05),
                Thirst::new(0.0, 0.08),
                Sleep::new(0.0, 0.03),
                Social::new(0.0, 0.04),
            ),
        )
        .expect("hauler");
    let _ = engine.world.remove_one::<SeekTarget>(entity);
    let (before, deposits) = (food_stock(&engine), engine.resources.tallies.haul_deposits);
    for _ in 0..(10 * engine.resources.rules.agent.move_interval_ticks + 20) {
        engine.tick();
        if engine
            .world
            .get::<&Inventory>(entity)
            .expect("inv")
            .get(ResourceKind::Food)
            == 0
        {
            break;
        }
    }
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(entity)
            .expect("inv")
            .get(ResourceKind::Food),
        0
    );
    assert!(food_stock(&engine) > before);
    assert_eq!(engine.resources.tallies.haul_deposits, deposits + 1);
}

#[test]
fn harness_forage_v9_unchanged() {
    for (seed, expected) in [
        "c022f6555945fcad",
        "5ca9b7e597238ee3",
        "d8afca881c554360",
        "48cf47e1341f5048",
    ]
    .into_iter()
    .enumerate()
    {
        let mut engine = game_v9_scene(seed as u64);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(fingerprint(&engine), expected, "seed {seed}");
        assert_eq!(engine.resources.tallies.forage_trips, 0);
        assert_eq!(engine.resources.tallies.haul_deposits, 0);
    }
}
