//! Proposed regression: Bill.auto is legacy engine provenance, not steward ownership.
use hecs::World;
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::steward::{Action, FoodPlan};
use sim_core::colony::{BillMode, Command, RecipeKind, Workshop};
use sim_core::components::{Agent, Position, Settlement};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

fn fixture(steward: bool) -> (World, SimResources, FoodPlan, u32) {
    let mut world = World::new();
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(1);
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (4, 4);
    res.settlements.insert(1, settlement);
    let known = Knowledge::from_entries(
        knowledge::defs()
            .nodes
            .iter()
            .map(|node| KnowledgeEntry::new(node.id, Source::Oral, 1000, 0))
            .collect(),
    );
    world.spawn((Agent { id: 7 }, Position::new(4, 4), known));
    let recipe = RecipeKind::from_name("roasted_roots").expect("recipe");
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: recipe.recipe().expect("definition").workshop,
            top_left: (4, 4),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    let add = Command::AddBill {
        workshop: 1,
        recipe,
        mode: BillMode::Maintain(24),
    };
    let command = if steward {
        Command::Steward {
            rule: 8,
            command: Box::new(add),
        }
    } else {
        add
    };
    assert!(crate::commands::apply_and_log(&mut world, &mut res, command, 0).accepted);
    let bill = res.colony.workshops[&1].bills[0].id;
    assert!(
        !res.colony.workshops[&1].bills[0].auto,
        "ordinary AddBill does not create engine auto bills"
    );
    let mut policy = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy.clone()),
            _ => None,
        })
        .expect("food policy");
    policy.recipes = vec![recipe];
    policy.vessel_recipes.clear();
    (world, res, policy, bill)
}

#[test]
fn steward_created_food_bill_is_suspended_when_no_recipe_inputs_remain() {
    let (world, res, policy, bill) = fixture(true);
    let commands = super::plan(&world, &res, &policy);
    assert!(
        commands.contains(&Command::SetBillSuspended {
            workshop: 1,
            bill,
            suspended: true
        }),
        "steward must reconcile its own ordinary bill when target falls to zero"
    );
}

#[test]
fn directly_created_player_food_bill_is_not_reconciled() {
    let (world, res, policy, _) = fixture(false);
    assert!(!super::plan(&world, &res, &policy)
        .iter()
        .any(|command| matches!(
            command,
            Command::SetBillSuspended { workshop: 1, .. }
                | Command::SetBillMode { workshop: 1, .. }
        )));
}

#[test]
fn player_edit_takes_precedence_over_prior_steward_bill_ownership() {
    let (mut world, mut res, policy, bill) = fixture(true);
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::SetBillMode {
                workshop: 1,
                bill,
                mode: BillMode::Maintain(7)
            },
            1
        )
        .accepted
    );
    assert!(!super::plan(&world, &res, &policy)
        .iter()
        .any(|command| matches!(
            command,
            Command::SetBillSuspended { workshop: 1, .. }
                | Command::SetBillMode { workshop: 1, .. }
        )));
}

#[test]
fn steward_created_food_bill_updates_its_initial_target_from_current_demand() {
    let (world, mut res, policy, bill) = fixture(true);
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let roots = sim_core::colony::ItemKind::from_name("roots").expect("roots");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(roots, None),
        4,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("real inputs");
    let commands = super::plan(&world, &res, &policy);
    assert!(
        commands.contains(&Command::SetBillMode {
            workshop: 1,
            bill,
            mode: BillMode::Maintain(3)
        }),
        "consuming two raw roots for three meals leaves five edible units; commands={commands:?}"
    );
}

#[path = "cooking_procurement_tests.rs"]
mod procurement_tests;

#[test]
fn satisfied_seed_reserve_keeps_grain_cooking_replenishable() {
    use sim_core::colony::{ItemKind, ItemSpec, ItemStack};
    let (world, mut res, mut policy, _) = fixture(true);
    seed_reserve_meals(&mut res);
    let recipe = RecipeKind::from_name("porridge").expect("recipe");
    policy.recipes = vec![recipe];
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    let seed = ItemKind::from_name("emmer_seed").expect("durable seed");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(grain, None),
        6,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("fresh physical grain");
    res.colony
        .ground_items
        .insert((2, 3), ItemStack::of(ItemSpec::raw(seed, None), 4));
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(super::seed_reserve(&res, &policy, &facts), Some((seed, 4)));
    assert_eq!(super::dependencies::seed_count(&world, &res, seed), 4);
    assert!(super::dependencies::discovery_materials(&world, &policy).is_empty());
    assert_eq!(
        super::dependencies::seed_inputs(&world, &res, &policy, &facts)
            .get(&grain)
            .copied()
            .unwrap_or(0),
        0
    );
    assert!(facts.usable_units.saturating_mul(1000) >= facts.daily_milli);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::AddBill {
            workshop: 1,
            recipe,
            mode: BillMode::Maintain(2),
        }),
        "a satisfied physical seed reserve must not impose a finite cooking allowance: {commands:?}"
    );
}

#[test]
fn cooking_credit_matches_target_when_other_recipes_lack_inputs() {
    use sim_core::colony::{ItemKind, ItemSpec, ItemStack};
    let (mut world, mut res, mut policy, _) = fixture(true);
    let known = world
        .query::<&Knowledge>()
        .iter()
        .next()
        .expect("founder")
        .1
        .clone();
    for id in 8..17 {
        world.spawn((Agent { id }, Position::new(4, 4), known.clone()));
    }
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let porridge = RecipeKind::from_name("porridge").expect("porridge");
    let roots = RecipeKind::from_name("roasted_roots").expect("roasted roots");
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    let seed = ItemKind::from_name("emmer_seed").expect("durable seed");
    policy.recipes = vec![porridge, roots];
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(grain, None),
        16,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("fresh physical grain");
    res.colony
        .ground_items
        .insert((2, 3), ItemStack::of(ItemSpec::raw(seed, None), 34));
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.daily_milli, 24_000);
    assert_eq!(super::seed_reserve(&res, &policy, &facts), Some((seed, 34)));
    let add = Command::AddBill {
        workshop: 1,
        recipe: porridge,
        mode: BillMode::Maintain(24),
    };
    assert!(super::cooking::plan(&world, &res, &policy, &facts).contains(&add));
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(add),
            },
            0,
        )
        .accepted
    );
    let mut supplied_only = policy.clone();
    supplied_only.recipes = vec![porridge];
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &supplied_only, &facts),
        24
    );
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        24,
        "an unavailable recipe must not dilute the actual accepted cooking target"
    );
}

fn priority_cooking_fixture() -> (World, SimResources, FoodPlan, u32) {
    use sim_core::colony::{ItemKind, ItemSpec, ItemStack};
    let (mut world, mut res, mut policy, bill) = fixture(true);
    let known = world
        .query::<&Knowledge>()
        .iter()
        .next()
        .expect("founder")
        .1
        .clone();
    for id in 8..17 {
        world.spawn((Agent { id }, Position::new(4, 4), known.clone()));
    }
    res.fires.insert(1, sim_engine::FireState { lit: true });
    policy.recipes = ["porridge", "fruit_porridge", "roasted_roots"]
        .map(|name| RecipeKind::from_name(name).expect("recipe"))
        .to_vec();
    let seed = ItemKind::from_name("emmer_seed").expect("durable seed");
    res.colony
        .ground_items
        .insert((2, 3), ItemStack::of(ItemSpec::raw(seed, None), 34));
    (world, res, policy, bill)
}

#[test]
fn priority_cooking_retains_other_meals_inside_one_stock_budget() {
    use sim_core::colony::{ItemKind, ItemSpec};
    let (world, mut res, policy, _) = priority_cooking_fixture();
    for (tile, name, count) in [
        ((1, 1), "porridge", 10),
        ((1, 2), "fruit_porridge", 4),
        ((1, 3), "roasted_roots", 3),
        ((2, 1), "wild_grain", 16),
        ((2, 2), "berries", 8),
        ((3, 1), "roots", 8),
    ] {
        let item = ItemKind::from_name(name).expect("item");
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(item, None),
            count,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("fresh physical stock");
    }
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.daily_milli, 24_000);
    assert_eq!(facts.cooked_units, 17);
    assert!(super::dependencies::seed_inputs(&world, &res, &policy, &facts).is_empty());
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::AddBill {
            workshop: 1,
            recipe: policy.recipes[0],
            mode: BillMode::Maintain(16),
        }),
        "24 total minus 17 existing meals permits two preferred three-unit batches: {commands:?}"
    );
}

#[test]
fn priority_cooking_falls_back_when_preferred_grain_is_absent() {
    use sim_core::colony::{ItemKind, ItemSpec};
    let (mut world, mut res, policy, bill) = priority_cooking_fixture();
    for (tile, name, count) in [((1, 1), "roots", 20), ((1, 2), "roasted_roots", 3)] {
        let item = ItemKind::from_name(name).expect("item");
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(item, None),
            count,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("fresh physical stock");
    }
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::SetBillMode {
                    workshop: 1,
                    bill,
                    mode: BillMode::Maintain(3),
                }),
            },
        )
        .accepted
    );
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.daily_milli, 24_000);
    assert_eq!(facts.cooked_units, 3);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::SetBillMode {
            workshop: 1,
            bill,
            mode: BillMode::Maintain(24),
        }),
        "missing grain must leave the existing cooked budget available to roots: {commands:?}"
    );
    assert!(!commands.iter().any(|command| matches!(command,
        Command::AddBill { recipe, .. } if *recipe == policy.recipes[0] || *recipe == policy.recipes[1]
    )));
}

#[test]
fn priority_cooking_counts_in_flight_output_once_at_both_craft_stages() {
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::{ItemKind, ItemSpec, Job, JobKind, JobStage};
    for stage in [JobStage::ToSource, JobStage::Working { ticks_left: 30 }] {
        let (mut world, mut res, policy, _) = priority_cooking_fixture();
        for (tile, name, count) in [
            ((1, 1), "porridge", 10),
            ((1, 2), "fruit_porridge", 4),
            ((1, 3), "roasted_roots", 3),
            ((2, 1), "wild_grain", 16),
            ((2, 2), "berries", 8),
        ] {
            let item = ItemKind::from_name(name).expect("item");
            crate::runtime::food::items::create_ground(
                &mut res,
                tile,
                ItemSpec::raw(item, None),
                count,
                0,
                crate::runtime::food::items::FoodSource::Harvest,
            )
            .expect("fresh stock");
        }
        let fruit = policy.recipes[1];
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::Steward {
                    rule: 8,
                    command: Box::new(Command::AddBill {
                        workshop: 1,
                        recipe: fruit,
                        mode: BillMode::Maintain(8)
                    }),
                },
                0
            )
            .accepted
        );
        let bill = res.colony.workshops[&1]
            .bills
            .iter()
            .find(|bill| bill.recipe == fruit)
            .expect("fruit bill")
            .id;
        if matches!(stage, JobStage::Working { .. }) {
            for (tile, count) in [((2, 1), 2), ((2, 2), 1)] {
                crate::runtime::food::items::take(
                    &mut world,
                    &mut res,
                    FoodOwner::Ground(tile),
                    count,
                    0,
                )
                .expect("Working inputs already consumed");
            }
        } else {
            // ToSource reserves a real shelf batch, so free_inputs removes it once.
            for (tile, count) in [((2, 1), 2), ((2, 2), 1)] {
                let stack = crate::runtime::food::items::take(
                    &mut world,
                    &mut res,
                    FoodOwner::Ground(tile),
                    count,
                    0,
                )
                .expect("physical shelf input");
                let owner = FoodOwner::Shelf {
                    workshop: 1,
                    key: sim_core::colony::food::FoodStackKey::from(stack.spec),
                };
                crate::runtime::food::items::put(&mut world, &mut res, owner, stack, 0)
                    .expect("fresh shelf ownership and age");
            }
        }
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
                    item: fruit.recipe().expect("recipe").output.0,
                    material: None,
                    fetch: None,
                    best_distance: 0,
                    progress_tick: 0,
                },
            )
            .expect("accepted craft");
        let facts = super::summary(&world, &res, &policy);
        let commands = super::cooking::plan(&world, &res, &policy, &facts);
        assert!(
            commands.contains(&Command::AddBill {
                workshop: 1,
                recipe: policy.recipes[0],
                mode: BillMode::Maintain(13),
            }),
            "17 physical plus 4 assigned leaves one preferred batch at {stage:?}: {commands:?}"
        );
    }
}

#[test]
fn priority_cooking_does_not_allocate_to_player_owned_preferred_bill() {
    use sim_core::colony::{ItemKind, ItemSpec};
    let (mut world, mut res, policy, bill) = priority_cooking_fixture();
    for (tile, name) in [((1, 1), "wild_grain"), ((1, 2), "roots")] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(ItemKind::from_name(name).expect("item"), None),
            20,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("fresh stock");
    }
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::AddBill {
                workshop: 1,
                recipe: policy.recipes[0],
                mode: BillMode::Maintain(1),
            },
            0
        )
        .accepted
    );
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::SetBillMode {
                    workshop: 1,
                    bill,
                    mode: BillMode::Maintain(1)
                }),
            }
        )
        .accepted
    );
    let facts = super::summary(&world, &res, &policy);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::SetBillMode {
            workshop: 1,
            bill,
            mode: BillMode::Maintain(24),
        }),
        "player-owned preferred recipe cannot spend the automatic budget: {commands:?}"
    );
    let preferred = res.colony.workshops[&1]
        .bills
        .iter()
        .find(|entry| entry.recipe == policy.recipes[0])
        .expect("player bill")
        .id;
    assert!(!commands.iter().any(|command| matches!(command,
        Command::SetBillMode { bill, .. } | Command::SetBillSuspended { bill, .. } if *bill == preferred
    )));
}

#[test]
fn priority_cooking_counts_disjoint_accepted_food_horizons_once_each() {
    use sim_core::colony::food::{DryingBatch, FoodOwner};
    use sim_core::colony::{ItemKind, ItemSpec, Job, JobKind, JobStage};
    let (mut world, mut res, mut policy, _) = priority_cooking_fixture();
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 14 * res.ticks_per_day;
    policy.recipes.truncate(1);
    let roots = ItemKind::from_name("roots").expect("roots");
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
        crate::runtime::food::items::create_ground(
            &mut res,
            (x, y),
            ItemSpec::raw(roots, None),
            count,
            14,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("212 fresh edible units");
    }
    let grain = ItemKind::from_name("wild_grain").expect("grain");
    crate::runtime::food::items::create_ground(
        &mut res,
        (6, 1),
        ItemSpec::raw(grain, None),
        18,
        14,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("fresh grain");
    let porridge = policy.recipes[0];
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 1,
                    recipe: porridge,
                    mode: BillMode::Maintain(3),
                }),
            },
            14 * 1440
        )
        .accepted
    );
    let bill = res.colony.workshops[&1]
        .bills
        .iter()
        .find(|bill| bill.recipe == porridge)
        .expect("porridge bill")
        .id;
    crate::runtime::food::items::take(&mut world, &mut res, FoodOwner::Ground((6, 1)), 2, 14)
        .expect("working grain already consumed");
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
                stage: JobStage::Working { ticks_left: 30 },
                taken_at: res.current_tick,
                amount: 0,
                item: porridge.recipe().expect("recipe").output.0,
                material: None,
                fetch: None,
                best_distance: 0,
                progress_tick: res.current_tick,
            },
        )
        .expect("accepted immediate craft");
    crate::runtime::food::origin::push_drying(
        &mut res,
        DryingBatch {
            workshop: 1,
            bill,
            tile: (4, 5),
            output: ItemSpec::raw(
                ItemKind::from_name("dried_berries").expect("dried berries"),
                None,
            ),
            count: 2,
            due_tick: 17 * 1440,
        },
        Default::default(),
    );
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 212);
    assert_eq!(super::harvest::targets(&policy, &facts), (48, 216));
    assert_eq!(
        super::harvest::pending_output(&world, &res, &policy, &facts),
        (3, 2),
        "immediate porridge expires before winter; later dried berries miss immediate horizon"
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        !commands.iter().any(|command| matches!(command,
            Command::SetBillMode { bill: id, mode: BillMode::Maintain(target), .. }
                if *id == bill && *target > 3
        )),
        "212 usable plus disjoint accepted 3+2 already fills 216: {commands:?}"
    );
}

#[test]
fn reserved_seed_cooking_is_finite_and_does_not_reopen_after_completion() {
    let (mut world, mut res, mut policy, _) = fixture(true);
    seed_reserve_meals(&mut res);
    let recipe = RecipeKind::from_name("porridge").expect("recipe");
    policy.recipes = vec![recipe];
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 1,
                    recipe,
                    mode: BillMode::Maintain(24),
                }),
            },
            0
        )
        .accepted
    );
    let bill = res.colony.workshops[&1]
        .bills
        .iter()
        .find(|bill| bill.recipe == recipe)
        .expect("bill")
        .id;
    let seed = sim_core::colony::ItemKind::from_name("wild_grain").expect("seed");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(seed, None),
        6,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical seed");
    let facts = super::summary(&world, &res, &policy);
    let physical_seed = sim_core::colony::ItemKind::from_name("emmer_seed").expect("durable seed");
    assert_eq!(
        super::seed_reserve(&res, &policy, &facts),
        Some((physical_seed, 4))
    );
    assert_eq!(
        super::dependencies::seed_inputs(&world, &res, &policy, &facts).get(&seed),
        Some(&4)
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    let mode = Command::SetBillMode {
        workshop: 1,
        bill,
        mode: BillMode::Make(1),
    };
    assert!(
        commands.contains(&mode),
        "only two non-reserved grain fund one craft: {commands:?}"
    );
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(mode),
            }
        )
        .accepted
    );
    assert!(!res.steward.vetoed(
        &sim_core::colony::steward::Target::Bill(1, recipe.code()),
        res.current_tick
    ));
    let unchanged = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(!unchanged
        .iter()
        .any(|command| matches!(command, Command::SetBillMode { bill: id, .. } if *id == bill)));
    // Reproduce the physical input consumption and completed-bill counter boundary.
    let consumed = crate::runtime::food::items::take(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((2, 2)),
        2,
        0,
    )
    .expect("actual input removal");
    assert_eq!(consumed.count, 2);
    let worker = world.query::<&Agent>().iter().next().expect("worker").0;
    world
        .insert_one(
            worker,
            sim_core::colony::Job {
                kind: sim_core::colony::JobKind::Craft {
                    workshop: 1,
                    bill,
                    at: (4, 5),
                },
                stage: sim_core::colony::JobStage::Working { ticks_left: 30 },
                taken_at: 0,
                amount: 0,
                item: recipe.recipe().expect("recipe").output.0,
                material: None,
                fetch: None,
                best_distance: 0,
                progress_tick: 0,
            },
        )
        .expect("actual in-flight craft fixture");
    let working_facts = super::summary(&world, &res, &policy);
    let working = super::cooking::plan(&world, &res, &policy, &working_facts);
    assert!(
        !working.contains(&Command::SetBillSuspended {
            workshop: 1,
            bill,
            suspended: true
        }),
        "consumed inputs must not cancel the already working craft: {working:?}"
    );
    assert!(
        !working
            .iter()
            .any(|command| matches!(command, Command::SetBillMode { bill: id, .. } if *id == bill)),
        "the single assigned craft already exhausts the finite allowance"
    );
    world
        .remove_one::<sim_core::colony::Job>(worker)
        .expect("finished craft");
    res.colony
        .workshops
        .get_mut(&1)
        .expect("shop")
        .bills
        .iter_mut()
        .find(|entry| entry.id == bill)
        .expect("bill")
        .done = 1;
    let facts = super::summary(&world, &res, &policy);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(commands.contains(&Command::SetBillSuspended {
        workshop: 1,
        bill,
        suspended: true
    }));
    assert!(!commands
        .iter()
        .any(|command| matches!(command, Command::SetBillMode { bill: id, .. } if *id == bill)));
    assert_eq!(facts.pieces.get(&seed), Some(&4));
}

#[test]
fn seed_cooking_recipes_share_one_surplus_allocation() {
    let (world, mut res, mut policy, _) = fixture(true);
    seed_reserve_meals(&mut res);
    let fruit = RecipeKind::from_name("fruit_porridge").expect("fruit recipe");
    let plain = RecipeKind::from_name("porridge").expect("plain recipe");
    policy.recipes = vec![fruit, plain];
    for (tile, name, count) in [((2, 2), "wild_grain", 6), ((2, 3), "berries", 1)] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            sim_core::colony::ItemSpec::raw(
                sim_core::colony::ItemKind::from_name(name).expect("item"),
                None,
            ),
            count,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("physical input");
    }
    let facts = super::summary(&world, &res, &policy);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(commands.contains(&Command::AddBill {
        workshop: 1,
        recipe: fruit,
        mode: BillMode::Make(1)
    }));
    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, Command::AddBill { recipe, .. } if *recipe == plain)),
        "the same two surplus grain cannot fund both recipes: {commands:?}"
    );
}

#[test]
fn seed_bootstrap_has_a_finite_physical_input_budget_and_stops_at_reserve() {
    let (mut world, mut res, policy, _) = fixture(true);
    seed_reserve_meals(&mut res);
    let recipe = super::dependencies::seed_recipe(&policy).expect("seed conversion");
    let def = recipe.recipe().expect("recipe");
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: def.workshop,
            top_left: (2, 4),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    let grain = def.inputs[0].0;
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(grain, None),
        4,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("bootstrap input");
    let facts = super::summary(&world, &res, &policy);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::AddBill {
            workshop: 2,
            recipe,
            mode: BillMode::Make(2)
        }),
        "four physical grain fund exactly two seed crafts: {commands:?}"
    );
    let add = Command::Steward {
        rule: 8,
        command: Box::new(Command::AddBill {
            workshop: 2,
            recipe,
            mode: BillMode::Make(2),
        }),
    };
    assert!(crate::commands::apply_command(&mut world, &mut res, add).accepted);
    let bill = res.colony.workshops[&2].bills[0].id;
    res.colony.ground_items.insert(
        (2, 3),
        sim_core::colony::ItemStack::of(sim_core::colony::ItemSpec::raw(def.output.0, None), 4),
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::SetBillSuspended {
            workshop: 2,
            bill,
            suspended: true
        }),
        "physical durable seed reserve ends bootstrap: {commands:?}"
    );
    let seed_stack = res
        .colony
        .ground_items
        .remove(&(2, 3))
        .expect("move physical seed");
    let carrier = world.spawn((
        sim_core::colony::Carry::of(seed_stack.spec(), seed_stack.count),
        sim_core::colony::Job {
            kind: sim_core::colony::JobKind::SowField {
                tile: (3, 3),
                from: (2, 3),
            },
            stage: sim_core::colony::JobStage::Working { ticks_left: 30 },
            taken_at: 0,
            amount: 4,
            item: seed_stack.kind,
            material: None,
            fetch: None,
            best_distance: 0,
            progress_tick: 0,
        },
    ));
    assert_eq!(
        super::dependencies::seed_count(&world, &res, def.output.0),
        4
    );
    let carried = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        carried.contains(&Command::SetBillSuspended {
            workshop: 2,
            bill,
            suspended: true
        }),
        "seed carried for sowing is still physical stock until consumption"
    );
    world.despawn(carrier).expect("move carrier seed to shelf");
    res.colony
        .workshops
        .get_mut(&2)
        .expect("workshop")
        .shelf
        .push(seed_stack);
    assert_eq!(
        super::dependencies::seed_count(&world, &res, def.output.0),
        4
    );
    let shelved = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(shelved.contains(&Command::SetBillSuspended {
        workshop: 2,
        bill,
        suspended: true
    }));
}

fn seed_reserve_meals(res: &mut SimResources) {
    crate::runtime::food::items::create_ground(
        res,
        (3, 2),
        sim_core::colony::ItemSpec::raw(
            sim_core::colony::ItemKind::from_name("roots").expect("roots"),
            None,
        ),
        3,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("non-emergency physical meals");
}

#[test]
fn emergency_cooking_spends_bootstrap_grain_before_people_starve() {
    let (mut world, mut res, mut policy, _) = fixture(true);
    let cultivation = knowledge::id("cultivation").expect("authored cultivation");
    let known = Knowledge::from_entries(
        knowledge::defs()
            .nodes
            .iter()
            .filter(|node| node.id != cultivation)
            .map(|node| KnowledgeEntry::new(node.id, Source::Oral, 1000, 0))
            .collect(),
    );
    let first = world.query::<&Agent>().iter().next().expect("person").0;
    world
        .insert_one(first, known.clone())
        .expect("pending cultivation discovery");
    for id in 8..17 {
        world.spawn((Agent { id }, Position::new(4, 4), known.clone()));
    }
    let porridge = RecipeKind::from_name("porridge").expect("porridge");
    policy.recipes = vec![porridge];
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(grain, None),
        36,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("real grain stock");
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 0);
    assert_eq!(
        super::dependencies::seed_inputs(&world, &res, &policy, &facts).get(&grain),
        Some(&36)
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::AddBill {
            workshop: 1,
            recipe: porridge,
            mode: BillMode::Make(8)
        }),
        "one daily budget of24 portions must be cookable from reserved grain: {commands:?}"
    );
    let seed_recipe = super::dependencies::seed_recipe(&policy).expect("seed recipe");
    assert!(
        !commands.iter().any(
            |command| matches!(command, Command::AddBill { recipe, .. } if *recipe == seed_recipe)
        ),
        "new seed extraction cannot compete with emergency meals"
    );
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "a proposed bill is not executable food"
    );
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 1,
                    recipe: porridge,
                    mode: BillMode::Make(8),
                }),
            },
            0
        )
        .accepted
    );
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "an accepted bill without actual heat cannot displace forage"
    );
    res.fires.insert(1, sim_engine::FireState { lit: true });
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        24,
        "physical grain and the accepted finite bill displace twenty-four future forage units"
    );
}

#[test]
fn executable_cooking_credit_counts_only_net_food_and_respects_suspension() {
    let (world, mut res, policy, _) = fixture(true);
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let roots = sim_core::colony::ItemKind::from_name("roots").expect("roots");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(roots, None),
        2,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical ingredients");
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        1,
        "two raw units are already counted; converting them to three adds only one"
    );
    res.fires.get_mut(&1).expect("hearth").lit = false;
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "supplyable ingredients without actual heat are not executable food"
    );
    res.fires.get_mut(&1).expect("hearth").lit = true;
    res.colony.workshops.get_mut(&1).expect("shop").bills[0].suspended = true;
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "paused bills are not future food"
    );
    res.colony.workshops.get_mut(&1).expect("shop").bills[0].suspended = false;
    let duplicate = res.colony.workshops[&1].bills[0];
    res.colony
        .workshops
        .get_mut(&1)
        .expect("shop")
        .bills
        .push(duplicate);
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        1,
        "multiple bills cannot duplicate their shared physical ingredients"
    );
}

#[test]
fn executable_cooking_credit_waits_for_the_required_pot_to_arrive() {
    use sim_core::colony::{ItemKind, ItemSpec, ItemStack};
    let (mut world, mut res, mut policy, _) = fixture(true);
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let stew = RecipeKind::from_name("stew").expect("stew");
    policy.recipes = vec![stew];
    for (tile, name) in [((2, 2), "meat"), ((2, 3), "roots")] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(ItemKind::from_name(name).expect("input"), None),
            1,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("physical input");
    }
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 1,
                    recipe: stew,
                    mode: BillMode::Make(1)
                }),
            },
            0
        )
        .accepted
    );
    let pot = ItemKind::all()
        .find(|kind| kind.definition().pot_capacity > 0)
        .expect("vessel");
    res.colony
        .ground_items
        .insert((3, 3), ItemStack::of(ItemSpec::raw(pot, None), 1));
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "a reachable vessel is only supplyable, not yet executable"
    );
    let vessel = res
        .colony
        .ground_items
        .remove(&(3, 3))
        .expect("real vessel");
    res.colony
        .workshops
        .get_mut(&1)
        .expect("shop")
        .shelf
        .push(vessel);
    assert!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts) > 0,
        "the same ingredients become executable only with the local pot and lit hearth"
    );
}

#[test]
fn winter_grain_cooking_uses_combined_headroom_without_a_second_food_budget() {
    for (raw, expected_batches) in [(172, 8), (216, 0), (260, 0)] {
        let (mut world, mut res, mut policy, _) = fixture(true);
        let known = world
            .query::<&Knowledge>()
            .iter()
            .next()
            .expect("founder")
            .1
            .clone();
        for id in 8..17 {
            world.spawn((Agent { id }, Position::new(4, 4), known.clone()));
        }
        let porridge = RecipeKind::from_name("porridge").expect("porridge");
        policy.recipes = vec![porridge];
        res.current_tick = 8 * res.ticks_per_day;
        res.fires.insert(1, sim_engine::FireState { lit: true });
        for (tile, name, count) in [((2, 2), "hazelnuts", raw), ((2, 3), "wild_grain", 100)] {
            crate::runtime::food::items::create_ground(
                &mut res,
                tile,
                sim_core::colony::ItemSpec::raw(
                    sim_core::colony::ItemKind::from_name(name).expect("input"),
                    None,
                ),
                count,
                8,
                crate::runtime::food::items::FoodSource::Harvest,
            )
            .expect("real food");
        }
        let facts = super::summary(&world, &res, &policy);
        assert!(facts.preparing);
        assert_eq!(super::harvest::targets(&policy, &facts), (48, 216));
        assert_eq!(facts.usable_units, u64::from(raw));
        let commands = super::cooking::plan(&world, &res, &policy, &facts);
        let meal = commands.iter().find(|command| {
            matches!(command,
            Command::AddBill { recipe, .. } if *recipe == porridge)
        });
        if expected_batches == 0 {
            assert!(
                meal.is_none(),
                "raw stock at/above max(48,216) forbids net-new grain meals: {commands:?}"
            );
        } else {
            assert_eq!(
                meal,
                Some(&Command::AddBill {
                    workshop: 1,
                    recipe: porridge,
                    mode: BillMode::Make(expected_batches),
                })
            );
            assert!(u64::from(raw) + u64::from(expected_batches) * 3 <= 216);
        }
    }
}

#[test]
fn executable_cooking_credit_requires_one_practitioner_for_both_recipe_gates() {
    let (mut world, mut res, mut policy, _) = fixture(true);
    let porridge = RecipeKind::from_name("porridge").expect("porridge");
    policy.recipes = vec![porridge];
    policy.cultivation = None;
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let grain = sim_core::colony::ItemKind::from_name("wild_grain").expect("grain");
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        sim_core::colony::ItemSpec::raw(grain, None),
        12,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical grain");
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 1,
                    recipe: porridge,
                    mode: BillMode::Maintain(24),
                }),
            },
            0,
        )
        .accepted
    );
    let facts = super::summary(&world, &res, &policy);
    assert!(super::cooking::convertible_immediate(&world, &res, &policy, &facts) > 0);
    let primary = knowledge::defs().id_of("cooking").expect("cooking");
    let auxiliary = knowledge::defs()
        .id_of("grain_parching")
        .expect("grain parching");
    let cook = Knowledge::from_entries(vec![KnowledgeEntry::new(primary, Source::Oral, 1000, 0)]);
    let parcher =
        Knowledge::from_entries(vec![KnowledgeEntry::new(auxiliary, Source::Oral, 1000, 0)]);
    let person = world.query::<&Agent>().iter().next().expect("adult").0;
    world
        .insert_one(person, cook.clone())
        .expect("replace knowledge");
    world.spawn((Agent { id: 8 }, Position::new(4, 4), parcher.clone()));
    let recipe = porridge.recipe().expect("recipe");
    for knowledge in [&cook, &parcher] {
        assert!(
            !(knowledge::gates().recipe(porridge) & !knowledge.practice() == 0
                && crate::runtime::jobs::food_recipe_permitted(
                    &res,
                    &res.colony.workshops[&1],
                    recipe,
                    knowledge.practice(),
                )),
            "neither actual adult can execute this pot-free recipe"
        );
    }
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "separate primary and auxiliary practitioners cannot displace real food harvest"
    );
    let bill = res.colony.workshops[&1]
        .bills
        .iter()
        .find(|bill| bill.recipe == porridge)
        .expect("porridge bill")
        .id;
    assert!(
        super::cooking::plan(&world, &res, &policy, &facts).contains(&Command::SetBillSuspended {
            workshop: 1,
            bill,
            suspended: true
        })
    );
    let both = Knowledge::from_entries(vec![
        KnowledgeEntry::new(primary, Source::Oral, 1000, 0),
        KnowledgeEntry::new(auxiliary, Source::Oral, 1000, 0),
    ]);
    world.insert_one(person, both.clone()).expect("same person");
    let expected = super::cooking::convertible_immediate(&world, &res, &policy, &facts);
    assert!(expected > 0);
    world
        .insert_one(person, sim_core::components::Child { born_tick: 0 })
        .expect("child");
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0
    );
    world
        .remove_one::<sim_core::components::Child>(person)
        .expect("adult again");
    world
        .insert_one(person, cook.clone())
        .expect("pot-only cook");
    let pot = sim_core::colony::ItemSpec::raw(
        sim_core::colony::ItemKind::from_name("clay_pot").expect("pot"),
        None,
    );
    crate::runtime::food::items::create_ground(
        &mut res,
        (4, 5),
        pot,
        1,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("real pot");
    let pot_id = crate::runtime::food::storage::deploy_pot(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((4, 5)),
        (4, 5),
        1,
    )
    .expect("local pot");
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        expected
    );
    assert!(crate::runtime::knowledge::knowledge_of(&world, person)
        .get(auxiliary)
        .is_none());
    res.food.pots.get_mut(&pot_id).expect("pot").tile = (0, 0);
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0
    );
    res.food.pots.remove(&pot_id);
    // Procurement must use the same person's gates, but must not demand heat or inputs first.
    let _grain = crate::runtime::food::items::take(
        &mut world,
        &mut res,
        sim_core::colony::food::FoodOwner::Ground((2, 2)),
        12,
        0,
    )
    .expect("remove physical grain");
    res.fires.get_mut(&1).expect("fire").lit = false;
    let empty = super::summary(&world, &res, &policy);
    assert_eq!(
        super::dependencies::ingredient_deficits(&world, &res, &policy, &empty)
            .get(&grain)
            .copied()
            .unwrap_or(0),
        0
    );
    world.insert_one(person, both).expect("capable procurer");
    assert!(
        super::dependencies::ingredient_deficits(&world, &res, &policy, &empty)
            .get(&grain)
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &empty),
        0
    );
}

fn delayed_grain_bill(steward_roots: bool) -> (World, SimResources, FoodPlan, u32, u32) {
    let (mut world, mut res, mut policy, roots_bill) = fixture(steward_roots);
    let porridge = RecipeKind::from_name("porridge").expect("porridge");
    let roots = RecipeKind::from_name("roasted_roots").expect("roots recipe");
    policy.recipes = vec![porridge, roots];
    policy.cultivation = None;
    res.fires.insert(1, sim_engine::FireState { lit: true });
    for (name, tile) in [("roots", (2, 2)), ("wild_grain", (2, 3))] {
        let item = sim_core::colony::ItemKind::from_name(name).expect("input item");
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            sim_core::colony::ItemSpec::raw(item, None),
            4,
            0,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("physical recipe inputs");
    }
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 1,
                    recipe: porridge,
                    mode: BillMode::Make(3),
                }),
            },
            1,
        )
        .accepted
    );
    let bills = &res.colony.workshops[&1].bills;
    assert_eq!(
        bills.iter().map(|bill| bill.recipe).collect::<Vec<_>>(),
        vec![roots, porridge]
    );
    let porridge_bill = bills[1].id;
    (world, res, policy, roots_bill, porridge_bill)
}

#[test]
fn delayed_grain_food_bill_follows_authored_priority() {
    let (world, res, policy, _, porridge_bill) = delayed_grain_bill(true);
    let facts = super::summary(&world, &res, &policy);
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(
        commands.contains(&Command::MoveBill {
            workshop: 1,
            bill: porridge_bill,
            up: true,
        }),
        "late available grain must follow authored priority; commands={commands:?}"
    );
}

#[test]
fn delayed_grain_food_bill_preserves_player_bill_order() {
    let (world, res, policy, _, _) = delayed_grain_bill(false);
    let facts = super::summary(&world, &res, &policy);
    assert!(!super::cooking::plan(&world, &res, &policy, &facts)
        .iter()
        .any(|command| matches!(command, Command::MoveBill { .. })));
}

#[test]
fn delayed_grain_food_bill_preserves_player_edited_order() {
    let (mut world, mut res, policy, roots_bill, _) = delayed_grain_bill(true);
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::SetBillMode {
                workshop: 1,
                bill: roots_bill,
                mode: BillMode::Maintain(7)
            },
            2,
        )
        .accepted
    );
    let facts = super::summary(&world, &res, &policy);
    assert!(!super::cooking::plan(&world, &res, &policy, &facts)
        .iter()
        .any(|command| matches!(command, Command::MoveBill { .. })));
}

#[test]
fn executable_cooking_credit_excludes_store_grain_that_cannot_be_carried() {
    use crate::runtime::food::{self, items};
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::{ItemKind, ItemSpec};

    for born_day in [13, 0] {
        let (mut world, mut res, mut policy, _) = fixture(true);
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        res.current_tick = 13 * res.ticks_per_day;
        let day = food::day(&res);
        assert_eq!(day, 13);
        let porridge = RecipeKind::from_name("porridge").expect("porridge");
        let grain = ItemKind::from_name("wild_grain").expect("grain");
        policy.recipes = vec![porridge];
        policy.cultivation = None;
        res.fires.insert(1, sim_engine::FireState { lit: true });
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
        items::create_ground(
            &mut res,
            (2, 2),
            ItemSpec::raw(grain, None),
            4,
            born_day,
            items::FoodSource::Harvest,
        )
        .expect("fresh in actual storage class");
        assert!(
            crate::commands::apply_and_log(
                &mut world,
                &mut res,
                Command::Steward {
                    rule: 8,
                    command: Box::new(Command::AddBill {
                        workshop: 1,
                        recipe: porridge,
                        mode: BillMode::Make(2),
                    })
                },
                0,
            )
            .accepted
        );
        let bill = res.colony.workshops[&1]
            .bills
            .iter()
            .find(|bill| bill.recipe == porridge)
            .expect("accepted bill")
            .id;
        res.colony.workshops.get_mut(&1).expect("shop").active = Some(bill);
        let owner = FoodOwner::Ground((2, 2));
        assert!(
            food::storage::remaining_days(&world, &res, owner, day)
                .expect("stored grain freshness")
                > 0
        );
        let facts = super::summary(&world, &res, &policy);
        assert_eq!(
            facts.pieces.get(&grain),
            Some(&4),
            "physical stock is retained"
        );
        let credit = super::cooking::convertible_immediate(&world, &res, &policy, &facts);
        if born_day == day {
            assert!(credit > 0, "fresh transferable input backs real conversion");
            assert_eq!(
                items::transfer(&mut world, &mut res, owner, FoodOwner::Carried(7), 2, day),
                Ok(2)
            );
        } else {
            assert_eq!(
                items::transfer(&mut world, &mut res, owner, FoodOwner::Carried(7), 2, day),
                Err(items::FoodError::Expired)
            );
            assert_eq!(res.colony.ground_items[&(2, 2)].count, 4);
            assert_eq!(
                credit, 0,
                "stored but uncarryable grain cannot displace food harvest"
            );
        }
    }
}

#[test]
fn executable_cooking_credit_keeps_local_shelf_but_not_another_shops_shelf() {
    use crate::runtime::food::{self, items};
    use sim_core::colony::food::{FoodOwner, FoodStackKey};
    for local in [true, false] {
        let (mut world, mut res, mut policy, _, _) = delayed_grain_bill(true);
        policy.recipes = vec![RecipeKind::from_name("porridge").expect("porridge")];
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        let id = if local { 1 } else { 2 };
        if !local {
            let mut other = res.colony.workshops[&1].clone();
            other.id = id;
            other.top_left = (5, 5);
            other.bills.clear();
            other.active = None;
            res.colony.workshops.insert(id, other);
        }
        let tile = res.colony.workshops[&id].work_tile();
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
        let stack = res.colony.ground_items[&(2, 3)];
        let owner = FoodOwner::Shelf {
            workshop: id,
            key: FoodStackKey::from(stack.spec()),
        };
        items::transfer(&mut world, &mut res, FoodOwner::Ground((2, 3)), owner, 4, 0)
            .expect("actual fresh supply to shelf");
        res.current_tick = 13 * res.ticks_per_day;
        assert!(
            food::storage::remaining_days(&world, &res, owner, 13).expect("shelf remains fresh")
                > 0
        );
        assert!(!food::storage::portable(&res, owner, stack));
        let facts = super::summary(&world, &res, &policy);
        let credit = super::cooking::convertible_immediate(&world, &res, &policy, &facts);
        if local {
            assert!(credit > 0, "shelved inputs can be cooked without carrying");
        } else {
            assert_eq!(credit, 0, "another shop's shelf cannot feed this bill");
        }
    }
}

#[test]
fn executable_cooking_credit_debits_committed_inputs_from_the_actual_workshop() {
    use crate::runtime::food::items;
    use sim_core::colony::food::{FoodOwner, FoodStackKey};
    use sim_core::colony::{Job, JobKind, JobStage};
    let (mut world, mut res, mut policy, _, _) = delayed_grain_bill(true);
    let porridge = RecipeKind::from_name("porridge").expect("porridge");
    policy.recipes = vec![porridge];
    for id in 8..18 {
        world.spawn((Agent { id }, Position::new(4, 4)));
    }
    let mut second = res.colony.workshops[&1].clone();
    second.id = 2;
    second.top_left = (5, 5);
    second.bills.clear();
    second.active = None;
    res.colony.workshops.insert(2, second);
    // Only shop 2 has a porridge bill; shop 1's shelf is not a transferable pool.
    res.colony
        .workshops
        .get_mut(&1)
        .expect("first shop")
        .bills
        .retain(|bill| bill.recipe != porridge);
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(Command::AddBill {
                    workshop: 2,
                    recipe: porridge,
                    mode: BillMode::Maintain(24),
                })
            },
            2
        )
        .accepted
    );
    let bill = res.colony.workshops[&2].bills[0].id;
    let stack = res.colony.ground_items[&(2, 3)];
    for workshop in [1, 2] {
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 3)),
            FoodOwner::Shelf {
                workshop,
                key: FoodStackKey::from(stack.spec()),
            },
            2,
            0,
        )
        .expect("actual supplied grain");
    }
    res.fires.insert(2, sim_engine::FireState { lit: true });
    let worker = world
        .query::<&Agent>()
        .iter()
        .find(|(_, agent)| agent.id == 7)
        .expect("qualified worker")
        .0;
    world
        .insert_one(
            worker,
            Job {
                kind: JobKind::Craft {
                    workshop: 2,
                    bill,
                    at: res.colony.workshops[&2].work_tile(),
                },
                stage: JobStage::ToSource,
                taken_at: 0,
                amount: 0,
                item: porridge.recipe().expect("recipe").output.0,
                material: None,
                fetch: None,
                best_distance: 0,
                progress_tick: 0,
            },
        )
        .expect("actual assigned craft");
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(
        super::cooking::convertible_immediate(&world, &res, &policy, &facts),
        0,
        "shop 2's grain is already committed; shop 1 cannot replace that debit"
    );
}

#[test]
fn owned_vessel_target_counts_physical_pots_waiting_outside_store() {
    for recipe_name in ["form_pot", "fire_pot"] {
        for (outside, inside, expected) in [(2, 0, 0), (1, 4, 1)] {
            for player in [false, true] {
                let (mut world, mut res, mut policy, _) = fixture(true);
                let recipe = RecipeKind::from_name(recipe_name).expect("vessel recipe");
                let def = recipe.recipe().expect("definition");
                policy.recipes.clear();
                policy.cultivation = None;
                policy.vessel_recipes = vec![(recipe, 2)];
                res.colony.workshops.clear();
                res.colony.workshops.insert(
                    1,
                    Workshop {
                        id: 1,
                        kind: def.workshop,
                        top_left: (4, 4),
                        size: (1, 1),
                        bills: Vec::new(),
                        shelf: Vec::new(),
                        active: None,
                    },
                );
                let add = Command::AddBill {
                    workshop: 1,
                    recipe,
                    mode: BillMode::Maintain(2),
                };
                let command = if player {
                    add
                } else {
                    Command::Steward {
                        rule: 8,
                        command: Box::new(add),
                    }
                };
                assert!(crate::commands::apply_and_log(&mut world, &mut res, command, 1).accepted);
                let bill = res.colony.workshops[&1].bills[0].id;
                res.colony.rooms.insert(
                    90,
                    sim_core::rooms::Room {
                        dwelling_kind: None,
                        id: 90,
                        tiles: std::collections::BTreeSet::from([(4, 4)]),
                        doors: std::collections::BTreeSet::new(),
                        roofed: true,
                        role: sim_core::rooms::RoomRole::Storehouse,
                        dug: false,
                    },
                );
                res.colony.room_of_tile.insert((4, 4), 90);
                crate::runtime::food::items::create_ground(
                    &mut res,
                    (2, 2),
                    sim_core::colony::ItemSpec::raw(def.output.0, None),
                    outside,
                    0,
                    crate::runtime::food::items::FoodSource::Debug,
                )
                .expect("actual outside vessels");
                if inside > 0 {
                    crate::runtime::food::items::create_ground(
                        &mut res,
                        (4, 4),
                        sim_core::colony::ItemSpec::raw(def.output.0, None),
                        inside,
                        0,
                        crate::runtime::food::items::FoodSource::Debug,
                    )
                    .expect("actual inside vessels");
                }
                assert_eq!(
                    crate::runtime::jobs::bills::maintain_count(&world, &res, def.output.0),
                    u32::from(outside + inside)
                );
                assert_eq!(
                    crate::runtime::jobs::bills::bill_count(&world, &res, def.output.0),
                    u32::from(inside)
                );
                let facts = super::summary(&world, &res, &policy);
                let commands = super::cooking::plan(&world, &res, &policy, &facts);
                let stop = Command::SetBillSuspended {
                    workshop: 1,
                    bill,
                    suspended: true,
                };
                if player {
                    assert!(!commands.iter().any(|command| matches!(command,
                    Command::SetBillMode { bill: id, .. } | Command::SetBillSuspended { bill: id, .. }
                    if *id == bill)), "manual vessel bill stays unchanged");
                } else if expected == 0 {
                    assert!(
                        commands.contains(&stop),
                        "outside vessels fulfill {recipe_name} target: {commands:?}"
                    );
                } else {
                    assert!(
                        commands.contains(&Command::SetBillMode {
                            workshop: 1,
                            bill,
                            mode: BillMode::Maintain(expected),
                        }),
                        "mixed excess stock must not inflate the vessel target: {commands:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn discovery_grain_floor_survives_low_reserves_but_not_zero_food() {
    for (meals, grain_count, cook) in [(16, 2, false), (16, 36, true), (0, 2, true)] {
        let (mut world, mut res, mut policy, _) = fixture(true);
        let cultivation = knowledge::id("cultivation").expect("cultivation");
        let known = Knowledge::from_entries(
            knowledge::defs()
                .nodes
                .iter()
                .filter(|node| node.id != cultivation)
                .map(|node| KnowledgeEntry::new(node.id, Source::Oral, 1000, 0))
                .collect(),
        );
        let first = world.query::<&Agent>().iter().next().expect("person").0;
        world.insert_one(first, known.clone()).expect("knowledge");
        for id in 8..17 {
            world.spawn((Agent { id }, Position::new(4, 4), known.clone()));
        }
        let porridge = RecipeKind::from_name("porridge").expect("porridge");
        policy.recipes = vec![porridge];
        for (name, tile, count) in [
            ("wild_grain", (2, 2), grain_count),
            ("roots", (3, 2), meals),
        ] {
            if count == 0 {
                continue;
            }
            crate::runtime::food::items::create_ground(
                &mut res,
                tile,
                sim_core::colony::ItemSpec::raw(
                    sim_core::colony::ItemKind::from_name(name).expect("item"),
                    None,
                ),
                count,
                0,
                crate::runtime::food::items::FoodSource::Harvest,
            )
            .expect("physical food");
        }
        let facts = super::summary(&world, &res, &policy);
        assert_eq!(facts.usable_units, u64::from(meals));
        assert_eq!(facts.daily_milli, 24000);
        let commands = super::cooking::plan(&world, &res, &policy, &facts);
        let admitted = commands.iter().any(|command| {
            matches!(command,
            Command::AddBill { recipe, mode: BillMode::Make(count), .. }
            if *recipe == porridge && *count > 0)
        });
        assert_eq!(
            admitted, cook,
            "meals={meals}, grain={grain_count}: {commands:?}"
        );
    }
}
