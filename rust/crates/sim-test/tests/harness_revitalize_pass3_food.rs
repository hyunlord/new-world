use sim_core::components::{
    Agent, AgentState, BuildingKind, Hunger, ResourceKind, Settlement, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::{
    food_economy::{members_need_per_day, supply_per_day, sustainable_adults},
    SettlementSystem, StockpileConsumeSystem,
};

fn birth_fixture(members: u64, stock: u32) -> (SimEngine, SettlementSystem, u32) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v9();
    for _ in 0..members {
        let entity = engine.spawn_agent(20, 20);
        engine
            .world
            .insert(entity, (Hunger::new(0.0, 0.05), Thirst::new(0.0, 0.08)))
            .expect("needs");
    }
    engine.resources.building_registry.insert(100, (20, 20));
    engine
        .resources
        .building_kinds
        .insert((20, 20), BuildingKind::Hearth);
    for x in 21..27 {
        engine
            .resources
            .building_registry
            .insert(u64::from(100 + x), (x, 20));
        engine
            .resources
            .building_kinds
            .insert((x, 20), BuildingKind::Shelter);
    }
    for x in 20..30 {
        engine.resources.food_source_max.insert((x, 22), 4);
    }
    let mut settlements = SettlementSystem::new();
    settlements.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.resources.settlements.len(), 1);
    let sid = *engine.resources.settlements.keys().next().expect("village");
    engine
        .resources
        .settlements
        .get_mut(&sid)
        .expect("village")
        .store(ResourceKind::Food, stock);
    (engine, settlements, sid)
}

fn tick_settlements(engine: &mut SimEngine, system: &mut SettlementSystem, ticks: u64) {
    for tick in 1..=ticks {
        engine.resources.current_tick = tick;
        system.tick(&mut engine.world, &mut engine.resources);
    }
}

#[test]
fn harness_food_supply_and_need_formula() {
    let (engine, _, sid) = birth_fixture(24, 500);
    assert_eq!(supply_per_day(&engine.resources, sid, 16), 120.0);
    assert!((members_need_per_day(&engine.world, &engine.resources, sid) - 57.6).abs() < 1e-4);
    assert_eq!(sustainable_adults(120.0, 0.85, 1440), 42);
}

#[test]
fn harness_food_no_free_meals() {
    for v9 in [false, true] {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = if v9 {
            WorldRules::game_v9()
        } else {
            WorldRules::game_v8()
        };
        let a = engine.spawn_agent(10, 10);
        let b = engine.spawn_agent(10, 10);
        engine
            .world
            .insert(
                a,
                (
                    Hunger::new(80.0, 0.0),
                    AgentState::Consuming {
                        target: TargetKind::Food,
                    },
                ),
            )
            .expect("first eater");
        engine
            .world
            .insert_one(b, Hunger::new(80.0, 0.0))
            .expect("second eater");
        engine.resources.set_food_tile(10, 10, 1);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.world.get::<&Hunger>(a).expect("first").value, 50.0);
        engine
            .world
            .remove_one::<AgentState>(a)
            .expect("first finished");
        engine
            .world
            .insert_one(
                b,
                AgentState::Consuming {
                    target: TargetKind::Food,
                },
            )
            .expect("second commit");
        engine.resources.current_tick = 1;
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            engine.world.get::<&Hunger>(b).expect("second").value,
            if v9 { 80.0 } else { 50.0 }
        );
        assert_eq!(engine.resources.tallies.meals_without_food, u64::from(!v9));
    }
}

#[test]
fn harness_food_birth_blocked_at_supply() {
    let (mut engine, mut system, _) = birth_fixture(42, 500);
    tick_settlements(&mut engine, &mut system, 2880);
    assert_eq!(engine.resources.tallies.births, 0);
}

#[test]
fn harness_food_birth_blocked_without_reserve() {
    let (mut engine, mut system, _) = birth_fixture(24, 38);
    tick_settlements(&mut engine, &mut system, 2880);
    assert_eq!(engine.resources.tallies.births, 0);
}

#[test]
fn harness_food_birth_allowed_when_sustainable() {
    let (mut engine, mut system, _) = birth_fixture(24, 500);
    tick_settlements(&mut engine, &mut system, 721);
    assert_eq!(engine.resources.tallies.births, 1);
}

#[test]
fn harness_food_relief_reaches_tether() {
    assert_eq!(
        WorldRules::game_v9().society.relief_radius,
        WorldRules::game_v9().agent.idle_tether_radius
    );
    for v9 in [false, true] {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = if v9 {
            WorldRules::game_v9()
        } else {
            WorldRules::game_v8()
        };
        let entity = engine.spawn_agent(17, 10);
        engine
            .world
            .insert_one(entity, Hunger::new(80.0, 0.0))
            .expect("hunger");
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (10, 10);
        settlement.add_member_agent(id);
        settlement.store(ResourceKind::Food, 1);
        engine.resources.settlements.insert(1, settlement);
        StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            engine.world.get::<&Hunger>(entity).expect("hunger").value,
            if v9 { 50.0 } else { 80.0 }
        );
    }
}
