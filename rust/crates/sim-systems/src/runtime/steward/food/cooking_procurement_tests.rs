//! Cooking procurement follows the existing daily bill budget.
use super::{
    fixture, Agent, BillMode, Command, FoodPlan, Position, RecipeKind, SimResources, World,
};

fn ten_adult_cooking_fixture(recipes: &[&str]) -> (World, SimResources, FoodPlan) {
    let (mut world, res, mut policy, _) = fixture(true);
    for id in 8..17 {
        world.spawn((Agent { id }, Position::new(4, 4)));
    }
    policy.recipes = recipes
        .iter()
        .map(|name| RecipeKind::from_name(name).expect("recipe"))
        .collect();
    (world, res, policy)
}

#[test]
fn cooking_procurement_covers_daily_target_before_any_inputs_exist() {
    // Given ten adults and the known grain recipe, but no grain to unlock a bill.
    let (world, res, policy) = ten_adult_cooking_fixture(&["porridge"]);
    let facts = super::super::summary(&world, &res, &policy);
    // When the planner requests ingredients for its existing one-day target.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    // Then 24 portions need eight batches of two grain, not one batch.
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    assert_eq!(demand.get(&grain), Some(&16));
}

#[test]
fn cooking_procurement_subtracts_food_and_ingredients_from_daily_target() {
    // Given 15 finished portions and five grain already physically available.
    let (world, mut res, policy) = ten_adult_cooking_fixture(&["porridge"]);
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    let porridge = sim_core::colony::ItemKind::from_name("porridge").expect("porridge");
    for (tile, kind, count) in [((2, 2), porridge, 15), ((2, 3), grain, 5)] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            sim_core::colony::ItemSpec::raw(kind, None),
            count,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("real food");
    }
    let facts = super::super::summary(&world, &res, &policy);
    // When the remaining nine portions are projected through three recipe batches.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    // Then six required grain minus five present leaves just one to procure.
    assert_eq!(demand.get(&grain), Some(&1));
}

#[test]
fn cooking_procurement_combines_shared_inputs_without_double_subtraction() {
    // Given two eligible recipes sharing grain and five grain already available.
    let (world, res, policy) = ten_adult_cooking_fixture(&["porridge", "fruit_porridge"]);
    let mut facts = super::super::summary(&world, &res, &policy);
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    facts.pieces.insert(grain, 5);
    // When each recipe receives 12 of the existing 24 daily portions.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    // Then porridge needs eight grain, fruit porridge six, less five already held.
    assert_eq!(demand.get(&grain), Some(&9));
}

#[test]
fn cooking_procurement_respects_zero_cooked_target() {
    // Given a disabled cooked inventory target.
    let (world, res, mut policy) = ten_adult_cooking_fixture(&["porridge"]);
    policy.cooked_days_milli = 0;
    let facts = super::super::summary(&world, &res, &policy);
    // When ingredients are projected.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    // Then no immediate recipe ingredients are requested.
    assert!(demand.values().all(|amount| *amount == 0));
}

#[test]
fn cooking_procurement_counts_pending_output_without_reusing_its_inputs() {
    use sim_core::colony::{ItemKind, Job, JobKind, JobStage};
    for (stage, physical_grain) in [
        (JobStage::ToSource, 2),
        (JobStage::Working { ticks_left: 30 }, 0),
    ] {
        // Given one assigned three-portion batch, before or after input consumption.
        let (mut world, mut res, policy) = ten_adult_cooking_fixture(&["porridge"]);
        let grain = ItemKind::from_name("wild_grain").expect("grain");
        let porridge = ItemKind::from_name("porridge").expect("porridge");
        let shop = res.colony.workshops.get_mut(&1).expect("hearth");
        shop.bills[0].recipe = policy.recipes[0];
        let bill = shop.bills[0].id;
        let worker = world.query::<&Agent>().iter().next().expect("worker").0;
        world
            .insert_one(
                worker,
                Job {
                    kind: JobKind::Craft {
                        workshop: 1,
                        bill,
                        at: (4, 5),
                    },
                    stage,
                    taken_at: 0,
                    amount: 0,
                    item: porridge,
                    material: None,
                    fetch: None,
                    best_distance: 0,
                    progress_tick: 0,
                },
            )
            .expect("owned craft");
        let mut facts = super::super::summary(&world, &res, &policy);
        facts.pieces.insert(grain, physical_grain);
        // When ingredients are requested for the remaining 21 portions.
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        // Then seven new batches require 14 new grain in either stage.
        assert_eq!(demand.get(&grain), Some(&14), "stage={stage:?}");
    }
}

#[test]
fn cooking_procurement_and_available_bills_share_the_same_daily_budget() {
    // Given two capable recipes, then enough current inputs to activate both.
    let (world, res, policy) = ten_adult_cooking_fixture(&["porridge", "fruit_porridge"]);
    let mut facts = super::super::summary(&world, &res, &policy);
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    let berries = sim_core::colony::ItemKind::from_name("berries").expect("berries");
    facts.pieces.insert(grain, 5);
    let before = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    facts.pieces.insert(berries, 1);
    // When both bills become available, procurement still shares the same 24 portions.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    let commands = super::super::cooking::plan(&world, &res, &policy, &facts);
    // Then new availability does not duplicate grain demand or raise the total target.
    assert_eq!(before.get(&grain), Some(&9));
    assert_eq!(demand.get(&grain), Some(&9));
    for recipe in policy.recipes {
        assert!(commands.contains(&Command::AddBill {
            workshop: 1,
            recipe,
            mode: BillMode::Maintain(12),
        }));
    }
}
