//! Old food saves preserve finite ownership and never invent historical freshness.
use sim_core::colony::food::{FoodOwner, FoodState};
use sim_core::colony::{ItemKind, ItemSpec, ItemStack};
use sim_core::components::{Agent, Inventory, Position, ResourceKind, Settlement};
use sim_systems::runtime::food::migration::migrate_legacy;

#[test]
fn baseline_migration_is_inert() {
    let mut engine = sim_test::baseline_engine(3);
    let before = engine.resources.food_tiles.clone();
    migrate_legacy(&mut engine.world, &mut engine.resources).unwrap();
    assert_eq!(engine.resources.food, FoodState::default());
    assert_eq!(engine.resources.food_tiles, before);
}

#[test]
fn finite_store_and_cargo_convert_without_claiming_harvest_or_meals() {
    let mut engine = sim_test::colony_engine(0);
    engine.resources.food = FoodState::default();
    for settlement in engine.resources.settlements.values_mut() {
        settlement.stockpile.remove(&ResourceKind::Food);
    }
    let settlement = 1;
    let mut village = Settlement::new_with_id(settlement, 0);
    village.formation_tile = sim_systems::worldgen::colony::hearth(&engine.resources).unwrap();
    engine.resources.settlements.insert(settlement, village);
    engine
        .resources
        .settlements
        .get_mut(&settlement)
        .unwrap()
        .store(ResourceKind::Food, 17);
    let person = engine
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .min_by_key(|(_, (agent, _))| agent.id)
        .map(|(entity, _)| entity)
        .unwrap();
    let mut inventory = Inventory::default();
    assert_eq!(inventory.add(ResourceKind::Food, 4), 0);
    engine.world.insert_one(person, inventory).unwrap();
    migrate_legacy(&mut engine.world, &mut engine.resources).unwrap();
    let legacy = ItemKind::role("legacy_food");
    assert_eq!(
        engine.resources.food.tallies.migrated.get(&legacy),
        Some(&21)
    );
    assert_eq!(engine.resources.food.tallies.harvested_units, 0);
    assert_eq!(engine.resources.food.tallies.meals, 0);
    assert!(engine.resources.food.meals.is_empty());
    assert!(engine.resources.food_tiles.is_empty());
    assert!(engine.resources.food_source_max.is_empty());
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(person)
            .unwrap()
            .get(ResourceKind::Food),
        0
    );
    assert!(engine
        .resources
        .settlements
        .values()
        .all(|s| !s.stockpile.contains_key(&ResourceKind::Food)));
}

#[test]
fn unmatched_old_meat_expires_while_known_pieces_keep_their_date() {
    let mut engine = sim_test::colony_engine(0);
    engine.resources.food = FoodState::default();
    let meat = ItemKind::role("grazer_meat");
    let tile = (3, 3);
    assert!(!engine.resources.colony.ground_items.contains_key(&tile));
    engine
        .resources
        .colony
        .ground_items
        .insert(tile, ItemStack::of(ItemSpec::raw(meat, None), 5));
    engine.resources.colony.item_ledger.create(meat, 5);
    engine.resources.colony.wildlife.meat_lots = vec![(0, 3)];
    migrate_legacy(&mut engine.world, &mut engine.resources).unwrap();
    assert_eq!(engine.resources.colony.ground_items[&tile].count, 3);
    assert_eq!(
        engine.resources.food.ages[&FoodOwner::Ground(tile)].born_day,
        0
    );
    assert_eq!(engine.resources.food.tallies.spoiled.get(&meat), Some(&2));
    assert_eq!(engine.resources.colony.item_ledger.consumed_of(meat), 2);
    assert!(engine.resources.colony.wildlife.meat_lots.is_empty());
}

#[test]
fn impossible_lot_quantity_is_rejected_without_fabricating_food() {
    let mut engine = sim_test::colony_engine(0);
    engine.resources.food = FoodState::default();
    engine.resources.colony.wildlife.meat_lots = vec![(0, 1)];
    let error = migrate_legacy(&mut engine.world, &mut engine.resources).unwrap_err();
    assert!(error.contains("legacy meat lots claim"), "{error}");
}
