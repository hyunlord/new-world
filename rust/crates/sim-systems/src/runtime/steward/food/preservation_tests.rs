use hecs::World;
use sim_core::colony::food::{DryingBatch, PlantState};
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::steward::{Action, FoodPlan};
use sim_core::colony::{BillMode, Command, ItemKind, ItemSpec, RecipeKind, Workshop};
use sim_core::components::{Agent, Position, Settlement};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

fn fixture() -> (World, SimResources, FoodPlan) {
    let mut world = World::new();
    let mut res = SimResources::new(16, 16, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(1);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 14 * res.ticks_per_day;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (8, 8);
    res.settlements.insert(1, settlement);
    for id in 0..10 {
        let known = Knowledge::from_entries(
            knowledge::defs()
                .nodes
                .iter()
                .map(|n| KnowledgeEntry::new(n.id, Source::Oral, 1000, 0))
                .collect(),
        );
        world.spawn((Agent { id }, Position::new(8, 8), known));
    }
    let recipe = RecipeKind::from_name("dry_berries").expect("recipe");
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: recipe.recipe().expect("recipe").workshop,
            top_left: (8, 8),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    let mut policy = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(p) => Some(p.clone()),
            _ => None,
        })
        .expect("food plan");
    policy.recipes = vec![recipe];
    policy.vessel_recipes.clear();
    policy.winter_keep_days = 6;
    (world, res, policy)
}

fn target(world: &World, res: &SimResources, policy: &FoodPlan) -> Option<u16> {
    super::cooking::plan(world, res, policy, &super::summary(world, res, policy))
        .iter()
        .find_map(|c| match c {
            Command::AddBill {
                workshop: 1,
                mode: BillMode::Maintain(n),
                ..
            } => Some(*n),
            _ => None,
        })
}

#[test]
fn preservation_budget_covers_winter_deficit_without_current_inputs() {
    let (world, mut res, policy) = fixture();
    assert_eq!(
        target(&world, &res, &policy),
        Some(144),
        "preservation must not be capped to one day24 or require already harvested inputs"
    );
    let facts = super::summary(&world, &res, &policy);
    let berries = ItemKind::from_name("berries").expect("berries");
    assert_eq!(
        super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
        Some(&144)
    );
    let catalog = sim_core::colony::content::installed_or_bundled();
    let species = catalog.plant_by_name("berry_bush").expect("berries plant");
    res.food.plants.insert(
        (9, 8),
        PlantState {
            species,
            units_left: 4,
            next_regrow_day: u32::MAX,
        },
    );
    let mut fed = super::summary(&world, &res, &policy);
    fed.usable_units = 1000;
    fed.groups.extend(
        berries
            .definition()
            .food
            .as_ref()
            .expect("food")
            .groups
            .iter()
            .cloned(),
    );
    assert!(super::harvest::plan(&world, &res, &policy, &fed).iter().any(|command| matches!(command,
        Command::DesignateHarvest { x0: 9, y0: 8, x1: 9, y1: 8 })),
        "physical berries harvest must follow preservation deficit even without immediate hunger/diversity/winter-edible gap");
}

#[test]
fn pending_preservation_is_credited_once_without_becoming_actual_stores() {
    let (world, mut res, policy) = fixture();
    let output = ItemSpec::raw(ItemKind::from_name("dried_berries").expect("output"), None);
    res.food.pending_drying.push(DryingBatch {
        workshop: 1,
        bill: 1,
        tile: (8, 8),
        output,
        count: 20,
        due_tick: 16 * res.ticks_per_day,
    });
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.stored_units, 0);
    assert_eq!(facts.winter_units, 0);
    assert_eq!(
        target(&world, &res, &policy),
        Some(144),
        "20 existing promises plus124 outstanding, not164 or124"
    );
    let berries = ItemKind::from_name("berries").expect("berries");
    assert_eq!(
        super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
        Some(&124)
    );
}

#[test]
fn absent_preservation_workshop_never_creates_recipe_or_store_credit() {
    let (world, mut res, policy) = fixture();
    res.colony.workshops.clear();
    assert_eq!(target(&world, &res, &policy), None);
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.winter_units, 0);
    assert!(super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).is_empty());
}

#[test]
fn completed_preservation_keeps_one_credit_while_waiting_for_and_finishing_haul() {
    let (mut world, mut res, policy) = fixture();
    let output = ItemSpec::raw(ItemKind::from_name("dried_berries").expect("output"), None);
    crate::runtime::food::items::create_ground(
        &mut res,
        (5, 5),
        output,
        20,
        14,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("completed output");
    let berries = ItemKind::from_name("berries").expect("input");
    for stored in [false, true] {
        if stored {
            assert!(
                crate::commands::apply_command(
                    &mut world,
                    &mut res,
                    Command::AddStockpile {
                        x0: 6,
                        y0: 6,
                        x1: 6,
                        y1: 6
                    }
                )
                .accepted
            );
            crate::runtime::food::items::transfer(
                &mut world,
                &mut res,
                sim_core::colony::food::FoodOwner::Ground((5, 5)),
                sim_core::colony::food::FoodOwner::Ground((6, 6)),
                20,
                14,
            )
            .expect("normal haul transition");
        }
        let facts = super::summary(&world, &res, &policy);
        assert_eq!(facts.winter_units, if stored { 20 } else { 0 });
        assert_eq!(target(&world, &res, &policy), Some(144));
        assert_eq!(
            super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
            Some(&124)
        );
    }
}

#[test]
fn late_or_already_expired_pending_output_does_not_reduce_winter_deficit() {
    for due_day in [0, 19] {
        let (world, mut res, policy) = fixture();
        let output = ItemSpec::raw(ItemKind::from_name("dried_berries").expect("output"), None);
        res.food.pending_drying.push(DryingBatch {
            workshop: 1,
            bill: 1,
            tile: (8, 8),
            output,
            count: 20,
            due_tick: due_day * res.ticks_per_day,
        });
        let facts = super::summary(&world, &res, &policy);
        let berries = ItemKind::from_name("berries").expect("input");
        assert_eq!(
            target(&world, &res, &policy),
            Some(164),
            "unqualified existing promises cannot replace144 units needed on time"
        );
        assert_eq!(
            super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
            Some(&144)
        );
        assert_eq!(facts.stored_units, 0);
    }
}

#[test]
fn paused_player_first_rack_does_not_hide_eligible_second_rack() {
    let (mut world, mut res, policy) = fixture();
    let recipe = policy.recipes[0];
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: recipe.recipe().expect("recipe").workshop,
            top_left: (10, 8),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    let now = res.current_tick;
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::AddBill {
                workshop: 1,
                recipe,
                mode: BillMode::Maintain(7)
            },
            now,
        )
        .accepted
    );
    let player_bill = res.colony.workshops[&1].bills[0].id;
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::SetBillSuspended {
                workshop: 1,
                bill: player_bill,
                suspended: true
            },
            now,
        )
        .accepted
    );
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(
        super::preservation::budget(&world, &res, &policy, &facts)
            .targets
            .get(&recipe),
        Some(&144)
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(commands.contains(&Command::AddBill { workshop: 2, recipe, mode: BillMode::Maintain(144) }),
        "the same eligible rack that admitted the reserve budget must receive its bill: {commands:?}");
    assert!(
        !commands.iter().any(|command| matches!(
            command,
            Command::SetBillSuspended { workshop: 1, .. }
                | Command::SetBillMode { workshop: 1, .. }
                | Command::AddBill { workshop: 1, .. }
        )),
        "the player's paused first rack remains untouched"
    );
}

#[test]
fn all_player_paused_racks_do_not_create_an_extra_preservation_workshop() {
    let (mut world, mut res, policy) = fixture();
    let recipe = policy.recipes[0];
    let now = res.current_tick;
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::AddBill {
                workshop: 1,
                recipe,
                mode: BillMode::Maintain(7)
            },
            now
        )
        .accepted
    );
    let bill = res.colony.workshops[&1].bills[0].id;
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::SetBillSuspended {
                workshop: 1,
                bill,
                suspended: true
            },
            now
        )
        .accepted
    );
    let facts = super::summary(&world, &res, &policy);
    assert!(super::preservation::budget(&world, &res, &policy, &facts)
        .targets
        .is_empty());
    assert!(
        super::cooking::plan(&world, &res, &policy, &facts).is_empty(),
        "player controls cannot be bypassed by building a replacement rack"
    );
}
