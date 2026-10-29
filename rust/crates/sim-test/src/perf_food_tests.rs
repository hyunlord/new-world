//! Benchmark gifts preserve nominal supply without inventing cooked/group history.
use super::gift_food;
use sim_core::colony::{food::FoodOwner, ItemKind};
use sim_core::components::{Agent, Hunger, ResourceKind, Settlement};
use sim_systems::runtime::food;

fn settlement(engine: &mut sim_engine::SimEngine, tile: (u32, u32)) -> u32 {
    let mut store = Settlement::new_with_id(1, 0);
    store.formation_tile = tile;
    engine.resources.settlements.insert(1, store);
    1
}

#[test]
fn baseline_gift_retains_counter_semantics() {
    let mut engine = crate::baseline_engine(0);
    let id = settlement(&mut engine, (3, 3));
    let before = engine.resources.food.clone();
    assert_eq!(gift_food(&mut engine, id, 70).unwrap(), 70);
    assert_eq!(
        engine.resources.settlements[&id].stockpile[&ResourceKind::Food],
        70
    );
    assert_eq!(engine.resources.food, before);
    assert_eq!(gift_food(&mut engine, 999, 70).unwrap(), 0);
}

#[test]
fn colony_gift_is_real_neutral_food_with_exact_accounting() {
    let mut engine = crate::colony_engine(0);
    let tile = sim_systems::worldgen::colony::hearth(&engine.resources).unwrap();
    let id = settlement(&mut engine, tile);
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let initial = engine.resources.food.tallies.initial_units;
    let harvested = engine.resources.food.tallies.harvested_units;
    assert_eq!(gift_food(&mut engine, id, 70).unwrap(), 70);
    let kind = ItemKind::role("legacy_food");
    let count: u64 = engine
        .resources
        .colony
        .ground_items
        .values()
        .filter(|stack| stack.kind == kind)
        .map(|stack| u64::from(stack.count))
        .sum();
    assert_eq!(count, 70);
    assert_eq!(engine.resources.food.tallies.initial_units, initial + 70);
    assert_eq!(engine.resources.food.tallies.harvested_units, harvested);
    assert!(engine.resources.food.tallies.migrated.is_empty());
    assert_eq!(
        engine.resources.settlements[&id]
            .stockpile
            .get(&ResourceKind::Food),
        None
    );
    let owner = engine
        .resources
        .colony
        .ground_items
        .iter()
        .find(|(_, stack)| stack.kind == kind)
        .map(|(tile, _)| FoodOwner::Ground(*tile))
        .unwrap();
    let person = engine
        .world
        .query::<(&Agent, &Hunger)>()
        .iter()
        .min_by_key(|(_, (agent, _))| agent.id)
        .map(|(entity, _)| entity)
        .unwrap();
    engine.world.get::<&mut Hunger>(person).unwrap().value = 80.0;
    let day = food::day(&engine.resources);
    let meal =
        food::meals::consume(&mut engine.world, &mut engine.resources, person, owner, day).unwrap();
    assert_eq!(meal.units, 1);
    assert_eq!(meal.relief_tenths, 300);
    assert!(!meal.cooked);
    assert!(meal.groups.is_empty());
    assert_eq!(engine.world.get::<&Hunger>(person).unwrap().value, 50.0);
    assert_eq!(engine.resources.colony.item_ledger.consumed_of(kind), 1);
    food::validate(&engine.world, &engine.resources).unwrap();
    assert!(food::meals::consume(
        &mut engine.world,
        &mut engine.resources,
        person,
        owner,
        day + 1
    )
    .is_err());
}

#[test]
fn missing_colony_settlement_rejects_without_supply() {
    let mut engine = crate::colony_engine(0);
    let before = engine.resources.food.clone();
    assert!(gift_food(&mut engine, u32::MAX, 70).is_err());
    assert_eq!(engine.resources.food, before);
}
