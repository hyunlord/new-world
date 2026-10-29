//! Job seams must preserve actual food ownership and preparation accounting.
use crate::runtime::food::{self, items};
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::{BlueprintKind, Carry, ItemKind, ItemSpec, JobKind, Workshop};
use sim_core::components::{Agent, Position};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use sim_engine::SimResources;

fn scene() -> (World, SimResources) {
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    (World::new(), res)
}

#[test]
fn job_pickup_shelf_delivery_and_cancellation_keep_original_birth_day() {
    let (mut world, mut res) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    let spec = ItemSpec::raw(roots, None);
    items::create_ground(&mut res, (2, 2), spec, 4, 0, items::FoodSource::Harvest).unwrap();
    res.current_tick = res.ticks_per_day * 2;
    let person = world.spawn((Agent { id: 1 }, Position::new(2, 2)));
    assert_eq!(
        super::food_transfers::pickup(&mut world, &mut res, person, (2, 2), roots, 2),
        (2, spec)
    );
    assert_eq!(res.food.ages[&FoodOwner::Carried(1)].born_day, 0);
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (4, 4),
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    super::food_transfers::arrive(
        &mut world,
        &mut res,
        person,
        JobKind::Supply {
            from: (2, 2),
            workshop: 1,
            to: (4, 5),
        },
        (4, 5),
        Carry::of(spec, 2),
    );
    assert!(world.get::<&Carry>(person).is_err());
    let shelf = FoodOwner::Shelf {
        workshop: 1,
        key: FoodStackKey::from(spec),
    };
    assert_eq!(res.food.ages[&shelf].born_day, 0);
    assert_eq!(
        super::food_transfers::pickup(&mut world, &mut res, person, (2, 2), roots, 2).0,
        2
    );
    super::cancel::drop_carry(&mut world, &mut res, person);
    assert_eq!(res.food.ages[&FoodOwner::Ground((2, 2))].born_day, 0);
    food::validate(&world, &res).unwrap();
}

#[test]
fn expired_carried_food_is_booked_as_spoilage_on_cancel() {
    let (mut world, mut res) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    let spec = ItemSpec::raw(roots, None);
    items::create_ground(&mut res, (2, 2), spec, 2, 0, items::FoodSource::Harvest).unwrap();
    let person = world.spawn((Agent { id: 1 }, Position::new(2, 2)));
    assert_eq!(
        super::food_transfers::pickup(&mut world, &mut res, person, (2, 2), roots, 2).0,
        2
    );
    res.current_tick = res.ticks_per_day * 6;
    super::cancel::drop_carry(&mut world, &mut res, person);
    assert!(world.get::<&Carry>(person).is_err());
    assert_eq!(res.food.tallies.spoiled_units, 2);
    assert_eq!(res.colony.item_ledger.consumed_of(roots), 2);
    food::validate(&world, &res).unwrap();
}

#[test]
fn cooking_consumes_fresh_shelf_inputs_once_and_passive_output_waits_for_due_tick() {
    let (mut world, mut res) = scene();
    let recipe = sim_core::colony::RecipeKind::from_name("roasted_roots")
        .unwrap()
        .recipe()
        .unwrap();
    let roots = recipe.inputs[0].0;
    let spec = ItemSpec::raw(roots, None);
    items::create_ground(&mut res, (2, 2), spec, 2, 0, items::FoodSource::Harvest).unwrap();
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (4, 4),
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    let owner = FoodOwner::Shelf {
        workshop: 1,
        key: FoodStackKey::from(spec),
    };
    items::transfer(&mut world, &mut res, FoodOwner::Ground((2, 2)), owner, 2, 1).unwrap();
    res.current_tick = res.ticks_per_day;
    super::craft_food::consume(&mut world, &mut res, 1, recipe).unwrap();
    res.colony.item_ledger.craft_start(&recipe.inputs);
    assert_eq!(res.food.tallies.craft_input_units, 2);
    assert!(!res.food.ages.contains_key(&owner));
    assert!(res.colony.workshops[&1].shelf.is_empty());
    let due_tick = res.current_tick + res.ticks_per_day;
    res.food
        .pending_drying
        .push(sim_core::colony::food::DryingBatch {
            workshop: 1,
            bill: 1,
            tile: (4, 5),
            output: ItemSpec::raw(recipe.output.0, None),
            count: recipe.output.1,
            due_tick,
        });
    food::drying::advance(&mut res).unwrap();
    assert_eq!(res.food.tallies.craft_output_units, 0);
    assert_eq!(res.food.pending_drying.len(), 1);
    food::validate(&world, &res).unwrap();
    res.current_tick = due_tick;
    food::drying::advance(&mut res).unwrap();
    assert_eq!(res.food.tallies.craft_output_units, 3);
    assert_eq!(res.colony.item_ledger.created_of(recipe.output.0), 3);
    assert!(res.food.pending_drying.is_empty());
    food::drying::advance(&mut res).unwrap();
    assert_eq!(res.colony.item_ledger.created_of(recipe.output.0), 3);
    food::validate(&world, &res).unwrap();
}

#[test]
fn expired_cooking_input_does_not_partially_consume_the_recipe() {
    let (mut world, mut res) = scene();
    let recipe = sim_core::colony::RecipeKind::from_name("roasted_roots")
        .unwrap()
        .recipe()
        .unwrap();
    let roots = recipe.inputs[0].0;
    let spec = ItemSpec::raw(roots, None);
    items::create_ground(&mut res, (2, 2), spec, 2, 0, items::FoodSource::Harvest).unwrap();
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (4, 4),
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    let owner = FoodOwner::Shelf {
        workshop: 1,
        key: FoodStackKey::from(spec),
    };
    items::transfer(&mut world, &mut res, FoodOwner::Ground((2, 2)), owner, 2, 1).unwrap();
    res.current_tick = res.ticks_per_day * 6;
    assert!(super::craft_food::consume(&mut world, &mut res, 1, recipe).is_none());
    assert_eq!(res.colony.workshops[&1].shelf[0].count, 2);
    assert_eq!(res.food.tallies.craft_input_units, 0);
}

#[test]
fn full_map_cancellation_retains_food_but_death_records_explicit_loss() {
    let (mut world, mut res) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    let spec = ItemSpec::raw(roots, None);
    items::create_ground(&mut res, (2, 2), spec, 2, 0, items::FoodSource::Harvest).unwrap();
    let person = world.spawn((Agent { id: 1 }, Position::new(2, 2)));
    assert_eq!(
        super::food_transfers::pickup(&mut world, &mut res, person, (2, 2), roots, 2).0,
        2
    );
    for y in 0..8 {
        for x in 0..8 {
            res.colony.ground_items.insert(
                (x, y),
                sim_core::colony::ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 1),
            );
            res.colony.item_ledger.create(ItemKind::Log, 1);
        }
    }
    super::cancel::drop_carry(&mut world, &mut res, person);
    assert_eq!(world.get::<&Carry>(person).unwrap().count, 2);
    assert_eq!(res.food.tallies.lost_units, 0);
    super::cancel::on_death(&mut world, &mut res, person);
    assert!(world.get::<&Carry>(person).is_err());
    assert!(!res.food.ages.contains_key(&FoodOwner::Carried(1)));
    assert_eq!(res.food.tallies.lost_units, 2);
    assert_eq!(res.food.tallies.spoiled_units, 0);
    assert_eq!(res.food.tallies.eaten_units, 0);
    assert_eq!(res.colony.item_ledger.consumed_of(roots), 2);
    food::validate(&world, &res).unwrap();
    super::invariants::check(&world, &res).unwrap();
}

#[test]
fn blocked_relocation_keeps_all_food_and_birth_records() {
    let (mut world, mut res) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    let spec = ItemSpec::raw(roots, None);
    items::create_ground(&mut res, (2, 2), spec, 2, 0, items::FoodSource::Harvest).unwrap();
    for y in 0..8 {
        for x in 0..8 {
            res.colony
                .occupied
                .insert((x, y), sim_core::colony::Occupant::Site((2, 2)));
        }
    }
    assert_eq!(
        items::relocate_ground(&mut world, &mut res, &[(2, 2)], (2, 2), 1),
        Err(items::FoodError::NoCapacity)
    );
    assert_eq!(res.colony.ground_items[&(2, 2)].count, 2);
    assert_eq!(res.food.ages[&FoodOwner::Ground((2, 2))].born_day, 0);
    res.colony.occupied.remove(&(3, 2));
    items::relocate_ground(&mut world, &mut res, &[(2, 2)], (2, 2), 1).unwrap();
    assert!(!res.colony.ground_items.contains_key(&(2, 2)));
    assert!(!res.food.ages.contains_key(&FoodOwner::Ground((2, 2))));
    assert_eq!(res.food.ages[&FoodOwner::Ground((3, 2))].born_day, 0);
    food::validate(&world, &res).unwrap();
}

#[test]
fn debug_food_is_tracked_as_initial_supply_only() {
    let (mut world, mut res) = scene();
    let roots = ItemKind::from_name("roots").unwrap();
    let result = crate::commands::apply_command(
        &mut world,
        &mut res,
        sim_core::colony::Command::DebugGive {
            kind: roots,
            count: 2,
            x: 2,
            y: 2,
        },
    );
    assert!(result.accepted);
    assert_eq!(res.food.ages[&FoodOwner::Ground((2, 2))].born_day, 0);
    assert_eq!(
        res.food.tallies.initial_units,
        u64::from(roots.definition().food.as_ref().unwrap().units) * 2
    );
    assert_eq!(res.food.tallies.harvested_units, 0);
    assert!(res.food.tallies.migrated.is_empty());
    food::validate(&world, &res).unwrap();
}
