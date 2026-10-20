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

/// The village food rules of the live game (`WorldRules::baseline`) copied
/// into `WorldRules::default()` (R1): the provisioned birth gate (utilization
/// 0.78, capacity-weighted supply), owned housing, hearth anchors, the relief
/// radius and strict consumption. Nothing else ticks in these fixtures.
fn food_rules() -> WorldRules {
    let live = WorldRules::baseline(7);
    let mut rules = WorldRules::default();
    rules.society.birth = live.society.birth;
    rules.society.shelter_planning = live.society.shelter_planning;
    rules.society.settlement_anchor = live.society.settlement_anchor;
    rules.society.relief_radius = live.society.relief_radius;
    rules.agent.strict_consume = live.agent.strict_consume;
    rules.agent.idle_tether_radius = live.agent.idle_tether_radius;
    rules
}

fn birth_fixture(members: u64, stock: u32) -> (SimEngine, SettlementSystem, u32) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = food_rules();
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
    // Baseline utilization 0.78 (v9 had 0.85 → 42): 0.78 × 120 / 2.4 = 39.
    let utilization = match engine.resources.rules.society.birth {
        sim_core::rules::BirthRule::Provisioned {
            sustain: Some(sustain),
            ..
        } => sustain.utilization,
        _ => panic!("baseline birth is provisioned with a sustain gate"),
    };
    assert_eq!(utilization, 0.78);
    assert_eq!(sustainable_adults(120.0, utilization, 1440), 39);
}

#[test]
fn harness_food_no_free_meals() {
    // Baseline `strict_consume`: a second eater on an emptied tile gets
    // nothing (the old v8 arm, where it ate for free, is gone with v8).
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = food_rules();
    assert!(engine.resources.rules.agent.strict_consume);
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
    assert_eq!(engine.world.get::<&Hunger>(b).expect("second").value, 80.0);
    assert_eq!(engine.resources.tallies.meals_without_food, 0);
}

#[test]
fn harness_food_birth_blocked_at_supply() {
    // 39 members: a 40th adult needs 96.0 > 0.78 × 120 = 93.6 (the first
    // refused size under the baseline gate; v9's was 42).
    let (mut engine, mut system, _) = birth_fixture(39, 500);
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
    // 38 members: the 39th adult needs 93.6 ≤ 93.6, the largest admitted size.
    let (mut engine, mut system, _) = birth_fixture(38, 500);
    tick_settlements(&mut engine, &mut system, 721);
    assert_eq!(engine.resources.tallies.births, 1);
}

#[test]
fn harness_food_relief_reaches_tether() {
    let live = WorldRules::baseline(7);
    assert_eq!(live.society.relief_radius, live.agent.idle_tether_radius);
    // A hungry member 7 tiles from the store (inside relief radius 8) eats
    // from it without walking home.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = food_rules();
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
        50.0
    );
}
