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
            mode: BillMode::Maintain(1)
        }),
        "four raw units leave one cooked slot within the five-unit reserve; commands={commands:?}"
    );
}

#[path = "cooking_procurement_tests.rs"]
mod procurement_tests;

#[test]
fn reserved_seed_cooking_is_finite_and_does_not_reopen_after_completion() {
    let (mut world, mut res, mut policy, _) = fixture(true);
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
