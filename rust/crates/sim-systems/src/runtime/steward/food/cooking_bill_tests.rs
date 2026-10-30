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
        "one adult needs ceil(2.4)=3 units, not the initial24; commands={commands:?}"
    );
}

#[path = "cooking_procurement_tests.rs"]
mod procurement_tests;
