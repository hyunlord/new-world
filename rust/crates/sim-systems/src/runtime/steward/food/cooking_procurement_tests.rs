//! Cooking procurement follows the existing daily bill budget.
use super::{
    fixture, Agent, BillMode, Command, FoodPlan, Position, RecipeKind, SimResources, World,
};

fn ten_adult_cooking_fixture(recipes: &[&str]) -> (World, SimResources, FoodPlan) {
    let (mut world, res, mut policy, _) = fixture(true);
    for id in 8..17 {
        world.spawn((Agent { id }, Position::new(4, 4)));
    }
    // This fixture isolates legacy daily cooking arithmetic; seed allocation has its own regressions.
    policy.cultivation = None;
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
    let (world, mut res, policy) = ten_adult_cooking_fixture(&["porridge", "fruit_porridge"]);
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(grain, None),
        5,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical shared ingredient");
    let facts = super::super::summary(&world, &res, &policy);
    // Physical priority cooking uses four grain for six portions; the residual18 needs twelve grain.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    // One physical grain remains, so eleven new grain fund the residual budget without berries.
    assert_eq!(demand.get(&grain), Some(&11));
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
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::Steward {
                    rule: 8,
                    command: Box::new(Command::AddBill {
                        workshop: 1,
                        recipe: policy.recipes[0],
                        mode: BillMode::Maintain(24),
                    })
                },
                0,
            )
            .accepted
        );
        let bill = res.colony.workshops[&1]
            .bills
            .iter()
            .find(|bill| bill.recipe == policy.recipes[0])
            .expect("owned porridge bill")
            .id;
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
        if physical_grain > 0 {
            use sim_core::colony::food::{FoodOwner, FoodStackKey};
            let spec = sim_core::colony::ItemSpec::raw(grain, None);
            crate::runtime::food::items::create_ground(
                &mut res,
                (2, 2),
                spec,
                physical_grain,
                0,
                crate::runtime::food::items::FoodSource::Harvest,
            )
            .expect("physical committed grain");
            crate::runtime::food::items::transfer(
                &mut world,
                &mut res,
                FoodOwner::Ground((2, 2)),
                FoodOwner::Shelf {
                    workshop: 1,
                    key: FoodStackKey::from(spec),
                },
                physical_grain,
                0,
            )
            .expect("actual pre-craft shelf");
        }
        let facts = super::super::summary(&world, &res, &policy);
        // When ingredients are requested for the remaining 21 portions.
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        // Then seven new batches require 14 new grain in either stage.
        assert_eq!(demand.get(&grain), Some(&14), "stage={stage:?}");
    }
}

#[test]
fn cooking_procurement_and_available_bills_share_the_same_daily_budget() {
    // Given two capable recipes, then enough current inputs to activate both.
    let (world, mut res, policy) = ten_adult_cooking_fixture(&["porridge", "fruit_porridge"]);
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    let berries = sim_core::colony::ItemKind::from_name("berries").expect("berries");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(grain, None),
        5,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical portable grain");
    let facts = super::super::summary(&world, &res, &policy);
    let before = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 3),
        sim_core::colony::ItemSpec::raw(berries, None),
        1,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical portable berries");
    let facts = super::super::summary(&world, &res, &policy);
    // When both recipes become available, procurement retains its 24-portion horizon.
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    let commands = super::super::cooking::plan(&world, &res, &policy, &facts);
    // Then procurement does not duplicate demand; current bills allocate the five
    // physical grain once, as two authored-priority porridge batches (six portions).
    assert_eq!(before.get(&grain), Some(&11));
    assert_eq!(demand.get(&grain), Some(&11));
    assert!(commands.contains(&Command::AddBill {
        workshop: 1,
        recipe: policy.recipes[0],
        mode: BillMode::Maintain(6),
    }));
    assert!(!commands.iter().any(|command| matches!(
        command,
        Command::AddBill { recipe, .. } if *recipe == policy.recipes[1]
    )));
}

#[test]
fn cooking_procurement_requests_fresh_grain_when_stored_grain_is_not_portable() {
    use crate::runtime::food::{self, items};
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::{ItemKind, ItemSpec};
    for born_day in [13, 0] {
        let (world, mut res, policy) = ten_adult_cooking_fixture(&["porridge"]);
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        res.current_tick = 13 * res.ticks_per_day;
        res.colony.rooms.insert(
            90,
            sim_core::rooms::Room {
                dwelling_kind: None,
                id: 90,
                tiles: std::collections::BTreeSet::from([(2, 2)]),
                doors: std::collections::BTreeSet::new(),
                roofed: true,
                role: sim_core::rooms::RoomRole::Storehouse,
                dug: false,
            },
        );
        res.colony.room_of_tile.insert((2, 2), 90);
        let grain = ItemKind::from_name("wild_grain").expect("grain");
        items::create_ground(
            &mut res,
            (2, 2),
            ItemSpec::raw(grain, None),
            16,
            born_day,
            items::FoodSource::Harvest,
        )
        .expect("physical grain fresh at storage owner");
        let owner = FoodOwner::Ground((2, 2));
        assert!(
            food::storage::remaining_days(&world, &res, owner, 13).expect("stored freshness") > 0
        );
        assert_eq!(
            food::storage::portable(&res, owner, res.colony.ground_items[&(2, 2)]),
            born_day == 13,
        );
        let facts = super::super::summary(&world, &res, &policy);
        assert_eq!(facts.pieces.get(&grain), Some(&16));
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        assert_eq!(
            demand.get(&grain),
            Some(&if born_day == 13 { 0 } else { 16 }),
            "unportable stored grain must not suppress the existing daily cooking request",
        );
        assert_eq!(res.colony.ground_items[&(2, 2)].count, 16);
        assert_eq!(facts.pieces.get(&grain), Some(&16));
    }
}

#[test]
fn cooking_procurement_uses_aged_local_shelf_only_at_a_compatible_workshop() {
    use crate::runtime::food::{self, items};
    use sim_core::colony::food::{FoodOwner, FoodStackKey};
    use sim_core::colony::{ItemKind, ItemSpec};
    for (compatible, same_kind_other_shop) in [(true, false), (false, false), (true, true)] {
        let (mut world, mut res, policy) = ten_adult_cooking_fixture(&["porridge"]);
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        let local = compatible && !same_kind_other_shop;
        let shop_id = if local { 1 } else { 2 };
        if !local {
            let mut other = res.colony.workshops[&1].clone();
            other.id = shop_id;
            if !compatible {
                other.kind = RecipeKind::from_name("dry_berries")
                    .expect("drying recipe")
                    .recipe()
                    .expect("definition")
                    .workshop;
            }
            other.top_left = (5, 5);
            other.bills.clear();
            other.active = None;
            assert_eq!(
                policy.recipes[0]
                    .recipe()
                    .expect("porridge")
                    .supports_workshop(other.kind),
                compatible
            );
            res.colony.workshops.insert(shop_id, other);
        }
        let tile = res.colony.workshops[&shop_id].work_tile();
        res.colony.rooms.insert(
            90,
            sim_core::rooms::Room {
                dwelling_kind: None,
                id: 90,
                tiles: std::collections::BTreeSet::from([tile]),
                doors: std::collections::BTreeSet::new(),
                roofed: true,
                role: sim_core::rooms::RoomRole::Storehouse,
                dug: false,
            },
        );
        res.colony.room_of_tile.insert(tile, 90);
        let grain = ItemKind::from_name("wild_grain").expect("grain");
        let spec = ItemSpec::raw(grain, None);
        items::create_ground(&mut res, (2, 2), spec, 16, 0, items::FoodSource::Harvest)
            .expect("physical grain");
        let owner = FoodOwner::Shelf {
            workshop: shop_id,
            key: FoodStackKey::from(spec),
        };
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 2)),
            owner,
            16,
            0,
        )
        .expect("actual fresh supply before storage");
        res.current_tick = 13 * res.ticks_per_day;
        let stack = items::stack(&world, &res, owner).expect("physical shelf");
        assert!(
            food::storage::remaining_days(&world, &res, owner, 13).expect("fresh in place") > 0
        );
        assert!(!food::storage::portable(&res, owner, stack));
        let facts = super::super::summary(&world, &res, &policy);
        assert_eq!(facts.pieces.get(&grain), Some(&16));
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        assert_eq!(
            demand.get(&grain),
            Some(&if local { 0 } else { 16 }),
            "only the actual selected shop shelf can supply the porridge recipe"
        );
        assert_eq!(
            items::stack(&world, &res, owner)
                .expect("retained stock")
                .count,
            16
        );
    }
}

#[test]
fn cooking_procurement_counts_other_accepted_food_without_blocking_preservation() {
    use crate::runtime::food::{items, origin};
    use sim_core::colony::food::{DryingBatch, FoodOwner, PlantState};
    use sim_core::colony::{ItemKind, ItemSpec, Workshop};

    let mut observed = Vec::new();
    for (pending, preserve) in [(0, false), (4, true), (4, false)] {
        let (mut world, mut res, mut policy) = ten_adult_cooking_fixture(&["fruit_porridge"]);
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        res.current_tick = 14 * res.ticks_per_day;
        policy.reserve_days = 2;
        policy.winter_keep_days = 6;
        res.fires.insert(1, sim_engine::FireState { lit: true });
        let fruit = policy.recipes[0];
        let drying = RecipeKind::from_name("dry_berries").expect("drying recipe");
        let berries = ItemKind::from_name("berries").expect("berries");
        let roots = ItemKind::from_name("roots").expect("roots");
        let grain = ItemKind::from_name("wild_grain").expect("grain");
        res.colony.workshops.insert(
            2,
            Workshop {
                id: 2,
                kind: drying.recipe().expect("recipe").workshop,
                top_left: (6, 3),
                size: (1, 1),
                bills: vec![],
                shelf: vec![],
                active: None,
            },
        );
        for (workshop, recipe) in [(1, fruit), (2, drying)] {
            let tick = res.current_tick;
            assert!(
                crate::commands::apply_and_log(
                    &mut world,
                    &mut res,
                    Command::Steward {
                        rule: 8,
                        command: Box::new(Command::AddBill {
                            workshop,
                            recipe,
                            mode: BillMode::Maintain(24),
                        }),
                    },
                    tick,
                )
                .accepted
            );
        }
        for (x, y, count) in [
            (1, 1, 28),
            (2, 1, 28),
            (3, 1, 28),
            (4, 1, 28),
            (5, 1, 28),
            (1, 2, 24),
            (2, 2, 24),
            (3, 2, 24),
        ] {
            items::create_ground(
                &mut res,
                (x, y),
                ItemSpec::raw(roots, None),
                count,
                13,
                items::FoodSource::Harvest,
            )
            .expect("212 actual fresh roots expiring before winter ends");
        }
        items::create_ground(
            &mut res,
            (6, 1),
            ItemSpec::raw(grain, None),
            4,
            14,
            items::FoodSource::Harvest,
        )
        .expect("four portable grain, not raw edible food");
        if pending > 0 {
            items::create_ground(
                &mut res,
                (6, 2),
                ItemSpec::raw(berries, None),
                pending,
                14,
                items::FoodSource::Harvest,
            )
            .expect("real drying inputs");
            items::take(&mut world, &mut res, FoodOwner::Ground((6, 2)), pending, 14)
                .expect("accepted drying has consumed its inputs");
            let shop = &res.colony.workshops[&2];
            let bill = shop
                .bills
                .iter()
                .find(|bill| bill.recipe == drying)
                .expect("owned drying bill")
                .id;
            let batch = DryingBatch {
                workshop: 2,
                bill,
                tile: shop.work_tile(),
                output: ItemSpec::raw(drying.recipe().expect("recipe").output.0, None),
                count: pending,
                due_tick: 16 * res.ticks_per_day,
            };
            origin::push_drying(&mut res, batch, Default::default());
        }
        if preserve {
            policy.recipes.push(drying);
        }
        let species = sim_core::colony::content::installed_or_bundled()
            .plant_by_name("berry_bush")
            .expect("berry plant");
        res.food.plants.insert(
            (6, 5),
            PlantState {
                species,
                units_left: 4,
                next_regrow_day: u32::MAX,
            },
        );
        assert!(crate::runtime::food::plants::harvestable(&res, (6, 5)));
        let facts = super::super::summary(&world, &res, &policy);
        assert_eq!(facts.usable_units, 212);
        assert_eq!(facts.cooked_units, 0);
        assert_eq!(facts.winter_units, 0);
        assert_eq!(super::super::harvest::targets(&policy, &facts), (48, 216));
        assert!(super::super::dependencies::seed_inputs(&world, &res, &policy, &facts).is_empty());
        assert_eq!(
            crate::runtime::jobs::bills::bill_count(
                &world,
                &res,
                fruit.recipe().expect("recipe").output.0,
            ),
            0
        );
        let preservation = super::super::cooking::priority_targets(
            &world,
            &res,
            &policy,
            &facts,
            &Default::default(),
            false,
        )
        .preservation;
        assert_eq!(preservation.missing.get(&berries).copied().unwrap_or(0), 0);
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        let commands = super::super::harvest::plan(&world, &res, &policy, &facts);
        observed.push((
            pending,
            preserve,
            demand.get(&berries).copied().unwrap_or(0),
            commands.contains(&Command::DesignateHarvest {
                x0: 6,
                y0: 5,
                x1: 6,
                y1: 5,
            }),
        ));
    }
    assert_eq!(
        observed,
        vec![
            (0, false, 1, true),
            (4, true, 0, false),
            (4, false, 0, false)
        ],
        "accepted other-output food closes new input admission while the real winter goal stays216"
    );
}

#[test]
fn cooking_procurement_funds_only_the_residual_authored_priority_budget() {
    use sim_core::colony::{ItemKind, ItemSpec};
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    let berries = ItemKind::from_name("berries").expect("berries");
    let roots = ItemKind::from_name("roots").expect("roots");
    let mut observed = Vec::new();
    for (grain_count, root_count) in [(16, 0), (5, 0), (0, 0), (0, 16)] {
        let recipes = if root_count > 0 {
            ["porridge", "roasted_roots"]
        } else {
            ["porridge", "fruit_porridge"]
        };
        let (world, mut res, policy) = ten_adult_cooking_fixture(&recipes);
        res.fires.insert(1, sim_engine::FireState { lit: true });
        for (tile, kind, count) in [((2, 2), grain, grain_count), ((2, 3), roots, root_count)] {
            if count > 0 {
                crate::runtime::food::items::create_ground(
                    &mut res,
                    tile,
                    ItemSpec::raw(kind, None),
                    count,
                    0,
                    crate::runtime::food::items::FoodSource::Harvest,
                )
                .expect("actual input");
            }
        }
        let facts = super::super::summary(&world, &res, &policy);
        let commands = super::super::cooking::plan(&world, &res, &policy, &facts);
        if grain_count == 16 {
            assert!(
                commands.contains(&Command::AddBill {
                    workshop: 1,
                    recipe: policy.recipes[0],
                    mode: BillMode::Maintain(24),
                }),
                "physical preferred recipe fills the entire cooked budget: {commands:?}"
            );
        }
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        observed.push((
            grain_count,
            root_count,
            demand.get(&grain).copied().unwrap_or(0),
            demand.get(&berries).copied().unwrap_or(0),
            demand.get(&roots).copied().unwrap_or(0),
        ));
    }
    assert_eq!(
        observed,
        vec![
            (16, 0, 0, 0, 0),
            (5, 0, 11, 0, 0),
            (0, 0, 16, 0, 0),
            (0, 16, 0, 0, 0)
        ],
        "fund supplied recipes first, then procure only the remaining authored budget"
    );
}

#[test]
fn future_preservation_does_not_block_emergency_cooking_or_its_missing_inputs() {
    use sim_core::colony::food::DryingBatch;
    use sim_core::colony::{Bill, ItemKind, ItemSpec, Workshop};
    for grain_count in [0, 16] {
        let (world, mut res, mut policy) = ten_adult_cooking_fixture(&["porridge"]);
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        res.current_tick = 14 * res.ticks_per_day;
        policy.reserve_days = 2;
        policy.winter_keep_days = 6;
        res.fires.insert(1, sim_engine::FireState { lit: true });
        let drying = RecipeKind::from_name("dry_berries").expect("drying");
        res.colony.workshops.insert(
            2,
            Workshop {
                id: 2,
                kind: drying.recipe().expect("recipe").workshop,
                top_left: (6, 3),
                size: (1, 1),
                bills: vec![Bill {
                    id: 1,
                    recipe: drying,
                    mode: BillMode::Make(108),
                    done: 108,
                    suspended: false,
                    auto: false,
                }],
                shelf: vec![],
                active: None,
            },
        );
        let due_tick = 16 * res.ticks_per_day;
        crate::runtime::food::origin::push_drying(
            &mut res,
            DryingBatch {
                workshop: 2,
                bill: 1,
                tile: (6, 4),
                output: ItemSpec::raw(drying.recipe().expect("recipe").output.0, None),
                count: 216,
                due_tick,
            },
            Default::default(),
        );
        let grain = ItemKind::from_name("wild_grain").expect("grain");
        if grain_count > 0 {
            crate::runtime::food::items::create_ground(
                &mut res,
                (2, 2),
                ItemSpec::raw(grain, None),
                grain_count,
                14,
                crate::runtime::food::items::FoodSource::Harvest,
            )
            .expect("actual emergency ingredient");
        }
        let facts = super::super::summary(&world, &res, &policy);
        assert_eq!(facts.usable_units, 0);
        assert_eq!(super::super::harvest::targets(&policy, &facts), (48, 216));
        let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        assert_eq!(
            demand.get(&grain).copied().unwrap_or(0),
            16 - u64::from(grain_count),
            "food due in two days cannot feed today's emergency"
        );
        if grain_count > 0 {
            let commands = super::super::cooking::plan(&world, &res, &policy, &facts);
            assert!(
                commands.contains(&Command::AddBill {
                    workshop: 1,
                    recipe: policy.recipes[0],
                    mode: BillMode::Maintain(24),
                }),
                "emergency inputs must become a real cooking bill: {commands:?}"
            );
        }
    }
}

#[test]
fn projected_missing_edible_inputs_spend_their_full_new_food_capacity() {
    use sim_core::colony::{ItemKind, ItemSpec};
    let (world, mut res, mut policy) = ten_adult_cooking_fixture(&["roasted_roots"]);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 14 * res.ticks_per_day;
    policy.reserve_days = 2;
    policy.winter_keep_days = 6;
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let nuts = ItemKind::from_name("hazelnuts").expect("nuts");
    let roots = ItemKind::from_name("roots").expect("roots");
    for (x, count) in [
        (1, 28),
        (2, 28),
        (3, 28),
        (4, 28),
        (5, 24),
        (6, 24),
        (7, 24),
        (8, 24),
    ] {
        crate::runtime::food::items::create_ground(
            &mut res,
            (x, 1),
            ItemSpec::raw(nuts, None),
            count,
            14,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("208 physical edible units");
    }
    let facts = super::super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 208);
    assert_eq!(super::super::harvest::targets(&policy, &facts), (48, 216));
    let demand = super::super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    assert_eq!(demand.get(&roots), Some(&4),
        "eight spare units fund two three-portion batches; absent raw roots are not existing food credit");
}
