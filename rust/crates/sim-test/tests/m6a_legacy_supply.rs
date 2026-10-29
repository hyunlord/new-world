//! Colony supply replacement must preserve baseline resource regeneration.
use sim_engine::RuntimeSystem;
use sim_systems::runtime::resource_regen::ResourceRegenSystem;

#[test]
fn colony_worldgen_replaces_the_legacy_supply_layer() {
    let engine = sim_test::colony_engine(0);
    assert!(!engine.resources.food.plants.is_empty());
    assert!(engine.resources.food_tiles.is_empty());
    assert!(engine.resources.food_source_max.is_empty());
    assert_eq!(engine.resources.food_regen_acc, 0);
}

#[test]
fn legacy_regeneration_cannot_recreate_colony_food() {
    let mut engine = sim_test::colony_engine(0);
    engine.resources.set_food_source_max(3, 3, 80);
    engine.resources.food_regen_acc = 777;
    let mut system = ResourceRegenSystem::new();
    system.tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.food_tiles.contains_key(&(3, 3)));
    assert_eq!(engine.resources.food_regen_acc, 777);
}

#[test]
fn baseline_regeneration_still_refills_its_food_source() {
    let mut engine = sim_test::baseline_engine(0);
    // A seasonal pulse may accrue less than one unit: isolate the fixed rate.
    engine.resources.rules.seasons = None;
    engine.resources.set_food_source_max(3, 3, 80);
    engine.resources.food_tiles.remove(&(3, 3));
    let mut system = ResourceRegenSystem::new();
    system.tick(&mut engine.world, &mut engine.resources);
    assert!(
        engine
            .resources
            .food_tiles
            .get(&(3, 3))
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert!(engine.resources.food.plants.is_empty());
}
