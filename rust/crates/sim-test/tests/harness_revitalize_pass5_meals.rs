//! Pass 5 Part 2: meals at the hearth served from the settlement store.
use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, ResourceKind, Settlement, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::StockpileConsumeSystem;
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v10_scene;

const HOME: (u32, u32) = (20, 20);

/// Tick at which the display clock (480-tick offset) reads `minute`.
fn tick_at(minute: u64) -> u64 {
    (minute + 1440 - 480) % 1440
}

/// Bare world: one v11 settlement at `HOME` with `store` food and one member
/// per `(position, hunger, thirst)`; a stocked bush four tiles from camp and
/// one far away, water on the river bank. Clock at `minute`.
fn meal_fixture(members: &[((u32, u32), f32, f64)], store: u32, minute: u64) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v11();
    engine.resources.current_tick = tick_at(minute);
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.store(ResourceKind::Food, store);
    let mut entities = Vec::new();
    for (index, &((x, y), hunger, thirst)) in members.iter().enumerate() {
        let entity = engine.spawn_agent(x, y);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(hunger, 0.05),
                    Thirst::new(thirst, 0.08),
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
    for (x, y) in [(24, 20), (50, 50)] {
        engine.resources.set_food_tile(x, y, 4);
        engine.resources.set_food_source_max(x, y, 4);
    }
    engine.resources.set_water_tile(20, 30, 255);
    (engine, entities)
}

fn store(engine: &SimEngine) -> u32 {
    engine.resources.settlements[&1].stockpile.get(&ResourceKind::Food).copied().unwrap_or(0)
}

fn state(engine: &SimEngine, entity: Entity) -> AgentState {
    *engine.world.get::<&AgentState>(entity).expect("state")
}

fn hunger(engine: &SimEngine, entity: Entity) -> f32 {
    engine.world.get::<&Hunger>(entity).expect("hunger").value
}

#[test]
fn harness_meal_breakfast_from_store() {
    let at_camp = [((20, 20), 30.0, 0.0), ((21, 19), 30.0, 0.0), ((26, 24), 30.0, 0.0)];
    let (mut engine, entities) = meal_fixture(&at_camp, 50, 390);
    let meals = engine.resources.tallies.meals_from_store;
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(store(&engine), 50 - 3, "one unit per member at breakfast");
    assert_eq!(engine.resources.tallies.meals_from_store, meals + 3);
    for &entity in &entities {
        assert_eq!(hunger(&engine, entity), 0.0);
    }
    for _ in 0..5 {
        engine.resources.current_tick += 1;
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        for &entity in &entities {
            assert_ne!(state(&engine, entity), AgentState::Seeking { target: TargetKind::Food });
        }
    }
    // Outside the meal windows hunger 30 is not served.
    let (mut engine, _) = meal_fixture(&at_camp, 50, 720);
    StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(store(&engine), 50);
    // Dinner serves like breakfast.
    let (mut engine, _) = meal_fixture(&at_camp, 50, 1140);
    StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(store(&engine), 47);
}

#[test]
fn harness_meal_hungry_at_camp_waits_for_store() {
    // Noon (outside the meal windows, inside the forage window); a bush four
    // tiles away. Thirst 60 is breached but not critical, so it must not send
    // the waiting member to the river either.
    // Start one tick after a serving pass (the system runs every 5 ticks), so
    // the member waits through four decision ticks before it is served.
    let (mut engine, entities) = meal_fixture(&[((21, 21), 55.0, 60.0)], 10, 721);
    let member = entities[0];
    let mut decision = AgentDecisionSystem::new();
    let mut consume = StockpileConsumeSystem::new();
    let start = engine.resources.current_tick;
    assert_eq!(start % 5, 1);
    let mut fed_at = None;
    for tick in start..start + 5 {
        engine.resources.current_tick = tick;
        decision.tick(&mut engine.world, &mut engine.resources);
        if fed_at.is_none() {
            assert_eq!(state(&engine, member), AgentState::Idle, "waiting member left at tick {tick}");
        }
        if tick % 5 == 0 {
            consume.tick(&mut engine.world, &mut engine.resources);
        }
        if fed_at.is_none() && hunger(&engine, member) < 50.0 {
            fed_at = Some(tick);
        }
    }
    let fed_at = fed_at.expect("member was not fed within five ticks");
    assert_eq!(fed_at - start, 4);
    assert_eq!(store(&engine), 9);
    assert_eq!(engine.resources.tallies.forage_trips, 0);

    // A critical water need is the one exception.
    let (mut engine, entities) = meal_fixture(&[((21, 21), 55.0, 80.0)], 10, 720);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(state(&engine, entities[0]), AgentState::Seeking { target: TargetKind::Water });
}

#[test]
fn harness_meal_away_eats_at_bush() {
    let (mut engine, entities) = meal_fixture(&[((32, 20), 55.0, 0.0)], 10, 720);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(state(&engine, entities[0]), AgentState::Seeking { target: TargetKind::Food });
    StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(store(&engine), 10, "a member twelve tiles out is not served");
}

#[test]
fn harness_meal_empty_store_falls_back() {
    let (mut engine, entities) = meal_fixture(&[((21, 21), 55.0, 0.0)], 0, 720);
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(state(&engine, entities[0]), AgentState::Seeking { target: TargetKind::Food });
}

#[test]
fn harness_meal_v10_unchanged() {
    for (seed, expected) in ["0e38264bd3abe6ba", "3b0590ec71558a58", "d265695444b8b314", "48120c365d543729"]
        .into_iter()
        .enumerate()
    {
        let mut engine = game_v10_scene(seed as u64);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(fingerprint(&engine), expected, "seed {seed}");
        assert_eq!(engine.resources.rules.society.meals, None);
    }
}
