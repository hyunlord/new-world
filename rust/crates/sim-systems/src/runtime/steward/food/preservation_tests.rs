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
        Some(216),
        "preservation must not be capped to one day24 or require already harvested inputs"
    );
    let facts = super::summary(&world, &res, &policy);
    let berries = ItemKind::from_name("berries").expect("berries");
    assert_eq!(
        super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
        Some(&216)
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
    crate::runtime::food::items::create_ground(
        &mut res,
        (7, 8),
        ItemSpec::raw(ItemKind::from_name("hazelnuts").expect("nuts"), None),
        216,
        14,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("actual full reserve");
    let mut fed = super::summary(&world, &res, &policy);
    assert_eq!(fed.usable_units, 216);
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
    assert!(
        !super::harvest::plan(&world, &res, &policy, &fed)
            .iter()
            .any(|command| matches!(
                command,
                Command::DesignateHarvest {
                    x0: 9,
                    y0: 8,
                    x1: 9,
                    y1: 8
                }
            )),
        "a genuine winter deficit cannot admit new food beyond the shared physical reserve"
    );
}

#[test]
fn pending_preservation_is_credited_once_without_becoming_actual_stores() {
    let (world, mut res, policy) = fixture();
    let output = ItemSpec::raw(ItemKind::from_name("dried_berries").expect("output"), None);
    let batch = DryingBatch {
        workshop: 1,
        bill: 1,
        tile: (8, 8),
        output,
        count: 20,
        due_tick: 16 * res.ticks_per_day,
    };
    crate::runtime::food::origin::push_drying(&mut res, batch, Default::default());
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.stored_units, 0);
    assert_eq!(facts.winter_units, 0);
    assert_eq!(
        target(&world, &res, &policy),
        Some(216),
        "20 existing promises plus196 outstanding, not236 or196"
    );
    let berries = ItemKind::from_name("berries").expect("berries");
    assert_eq!(
        super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
        Some(&196)
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
        assert_eq!(target(&world, &res, &policy), Some(216));
        assert_eq!(
            super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
            Some(&196)
        );
    }
}

#[test]
fn late_or_already_expired_pending_output_does_not_reduce_winter_deficit() {
    for due_day in [0, 19] {
        let (world, mut res, policy) = fixture();
        let output = ItemSpec::raw(ItemKind::from_name("dried_berries").expect("output"), None);
        let batch = DryingBatch {
            workshop: 1,
            bill: 1,
            tile: (8, 8),
            output,
            count: 20,
            due_tick: due_day * res.ticks_per_day,
        };
        crate::runtime::food::origin::push_drying(&mut res, batch, Default::default());
        let facts = super::summary(&world, &res, &policy);
        let berries = ItemKind::from_name("berries").expect("input");
        assert_eq!(
            target(&world, &res, &policy),
            Some(if due_day == 0 { 216 } else { 236 }),
            "overdue accepted20 retain physical capacity until cleared; future20 are excluded only by emergency admission"
        );
        assert_eq!(
            super::dependencies::ingredient_deficits(&world, &res, &policy, &facts).get(&berries),
            Some(if due_day == 0 { &196 } else { &216 })
        );
        assert_eq!(
            super::harvest::targets(&policy, &facts).1,
            216,
            "the durable goal is unchanged even when admission is bounded"
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
        super::cooking::priority_targets(&world, &res, &policy, &facts, &Default::default(), false)
            .preservation
            .targets
            .get(&recipe),
        Some(&216)
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    assert!(commands.contains(&Command::AddBill { workshop: 2, recipe, mode: BillMode::Maintain(216) }),
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
    let (mut world, mut res, mut policy) = fixture();
    // Isolate paused preservation racks from the independently tested seed bootstrap.
    policy.cultivation = None;
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
    assert!(super::cooking::priority_targets(
        &world,
        &res,
        &policy,
        &facts,
        &Default::default(),
        false
    )
    .preservation
    .targets
    .is_empty());
    assert!(
        super::cooking::plan(&world, &res, &policy, &facts).is_empty(),
        "player controls cannot be bypassed by building a replacement rack"
    );
}

#[test]
fn harvest_budget_counts_only_executable_fresh_output_once() {
    use sim_core::colony::{Job, JobKind, JobStage};
    let (mut world, mut res, policy) = fixture();
    let recipe = policy.recipes[0];
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::AddBill {
                workshop: 1,
                recipe,
                mode: BillMode::Maintain(20),
            }
        )
        .accepted
    );
    let bill = res.colony.workshops[&1].bills[0].id;
    let definition = recipe.recipe().expect("recipe");
    let job = Job {
        kind: JobKind::Craft {
            workshop: 1,
            bill,
            at: (8, 8),
        },
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount: 0,
        item: definition.output.0,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: res.current_tick,
    };
    let entity = world.spawn((job,));
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(
        super::harvest::pending_output(&world, &res, &policy, &facts),
        (0, 0)
    );
    world.get::<&mut Job>(entity).expect("job").stage = JobStage::Working { ticks_left: 1 };
    let units = u64::from(definition.output.1)
        * u64::from(
            definition
                .output
                .0
                .definition()
                .food
                .as_ref()
                .expect("food")
                .units,
        );
    assert_eq!(
        super::harvest::pending_output(&world, &res, &policy, &facts),
        (units, units)
    );
    world
        .remove_one::<Job>(entity)
        .expect("cancelled work removed");
    assert_eq!(
        super::harvest::pending_output(&world, &res, &policy, &facts),
        (0, 0)
    );
    let due_tick = 16 * res.ticks_per_day;
    crate::runtime::food::origin::push_drying(
        &mut res,
        DryingBatch {
            workshop: 1,
            bill,
            tile: (8, 8),
            output: ItemSpec::raw(definition.output.0, None),
            count: definition.output.1,
            due_tick,
        },
        Default::default(),
    );
    assert_eq!(
        super::harvest::pending_output(&world, &res, &policy, &facts),
        (units, units)
    );
    res.colony.workshops.remove(&1);
    assert_eq!(
        super::harvest::pending_output(&world, &res, &policy, &facts),
        (0, 0)
    );
}

#[test]
fn preservation_admission_keeps_physical_grain_cooking_room_before_real_drying() {
    use crate::runtime::food::items::{self, FoodSource};
    use sim_core::components::AgentState;
    use sim_engine::RuntimeSystem;
    let (mut world, mut res, mut policy) = fixture();
    policy.cultivation = None;
    let porridge = RecipeKind::from_name("porridge").expect("porridge");
    policy.recipes.insert(0, porridge);
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: porridge.recipe().expect("recipe").workshop,
            top_left: (8, 8),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let pot = ItemSpec::raw(ItemKind::from_name("clay_pot").expect("pot"), None);
    items::create_ground(&mut res, (5, 6), pot, 1, 14, FoodSource::Debug).expect("pot");
    let pot = res.colony.ground_items.remove(&(5, 6)).expect("pot");
    res.colony
        .workshops
        .get_mut(&2)
        .expect("hearth")
        .shelf
        .push(pot);
    for (tile, name, count) in [((6, 6), "hazelnuts", 48), ((6, 7), "wild_grain", 16)] {
        items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(ItemKind::from_name(name).expect("food"), None),
            count,
            14,
            FoodSource::Harvest,
        )
        .expect("food");
    }
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 48);
    assert_eq!(
        super::cooking::priority_targets(&world, &res, &policy, &facts, &Default::default(), false)
            .targets
            .get(&porridge)
            .copied(),
        Some(24)
    );
    assert!(res.food.pending_drying.is_empty());
    let berries = ItemKind::from_name("berries").expect("berries");
    let demand = super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    let admitted = *demand.get(&berries).expect("real winter demand");
    assert_eq!(
        admitted, 144,
        "216 reserve minus48 current minus24 executable physical grain; the winter goal remains216"
    );
    let commands = super::cooking::plan(&world, &res, &policy, &facts);
    let rack = commands
        .into_iter()
        .find(|command| matches!(command, Command::AddBill { workshop: 1, .. }))
        .expect("rack bill");
    assert!(matches!(
        rack,
        Command::AddBill {
            mode: BillMode::Maintain(144),
            ..
        }
    ));
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 8,
                command: Box::new(rack)
            },
            0
        )
        .accepted
    );
    items::create_ground(
        &mut res,
        (9, 8),
        ItemSpec::raw(berries, None),
        u16::try_from(admitted).expect("bounded input"),
        14,
        FoodSource::Harvest,
    )
    .expect("procured berries");
    let worker = world.query::<&Agent>().iter().next().expect("worker").0;
    world
        .insert(
            worker,
            (AgentState::Idle, crate::runtime::agent::MovementRng::new(1)),
        )
        .expect("worker state");
    let mut assign = crate::runtime::jobs::JobAssignSystem::new();
    let mut execute = crate::runtime::jobs::JobExecuteSystem::new();
    let mut movement = crate::runtime::agent::AgentMovementSystem::new();
    let start = res.current_tick;
    for tick in start..start + 600 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        crate::runtime::jobs::bills::schedule(&mut world, &mut res);
        assign.tick(&mut world, &mut res);
        execute.tick(&mut world, &mut res);
        if !res.food.pending_drying.is_empty() {
            break;
        }
    }
    assert!(
        !res.food.pending_drying.is_empty(),
        "normal Supply/Craft must create the passive commitment"
    );
    let after = super::summary(&world, &res, &policy);
    assert_eq!(super::cooking::priority_targets(&world, &res, &policy, &after, &Default::default(), false).targets.get(&porridge).copied(), Some(24), "accepted drying and its consumed raw input count once, retaining executable grain capacity");
    crate::runtime::food::validate(&world, &res).expect("food conservation");
    crate::runtime::jobs::invariants::check(&world, &res).expect("reservation conservation");
}

#[test]
fn emergency_procurement_and_preservation_bills_share_the_urgent_reservation() {
    let (world, mut res, mut policy) = fixture();
    policy.cultivation = None;
    let recipe = RecipeKind::from_name("roasted_roots").expect("recipe");
    policy.recipes.insert(0, recipe);
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: recipe.recipe().expect("recipe").workshop,
            top_left: (8, 8),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    res.fires.insert(1, sim_engine::FireState { lit: true });
    let facts = super::summary(&world, &res, &policy);
    let physical =
        super::cooking::priority_targets(&world, &res, &policy, &facts, &Default::default(), false);
    assert_eq!(
        physical.targets.get(&recipe).copied().unwrap_or(0),
        0,
        "missing roots are not a physical cooking bill"
    );
    let drying = RecipeKind::from_name("dry_berries").expect("drying");
    assert_eq!(
        physical.preservation.targets.get(&drying),
        Some(&192),
        "reserve24 urgent food before192 future food under the216 cap"
    );
    let demand = super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    assert_eq!(
        demand.get(&ItemKind::from_name("roots").expect("roots")),
        Some(&16)
    );
    assert_eq!(
        demand.get(&ItemKind::from_name("berries").expect("berries")),
        Some(&192)
    );
}

#[test]
fn full_food_cap_allows_physical_net_zero_drying_but_not_missing_raw_inputs() {
    for (name, expected) in [("berries", 216), ("hazelnuts", 0)] {
        let (world, mut res, policy) = fixture();
        crate::runtime::food::items::create_ground(
            &mut res,
            (9, 8),
            ItemSpec::raw(ItemKind::from_name(name).expect("food"), None),
            216,
            14,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("physical edible reserve");
        let facts = super::summary(&world, &res, &policy);
        assert_eq!(facts.usable_units, 216);
        let allocation = super::cooking::priority_targets(
            &world,
            &res,
            &policy,
            &facts,
            &Default::default(),
            false,
        );
        assert_eq!(
            allocation.preservation.targets.get(&policy.recipes[0]),
            Some(&expected)
        );
        let demand = super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
        assert!(demand.values().all(|count| *count == 0), "physical conversion consumes only existing food; absent berries cannot gain fictional net-zero credit");
    }
}

#[test]
fn physical_cooking_and_preservation_cannot_spend_the_same_berry_stack() {
    let (world, mut res, mut policy) = fixture();
    policy.cultivation = None;
    let recipe = RecipeKind::from_name("fruit_porridge").expect("recipe");
    policy.recipes.insert(0, recipe);
    res.colony.workshops.insert(
        2,
        Workshop {
            id: 2,
            kind: recipe.recipe().expect("recipe").workshop,
            top_left: (8, 8),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    res.fires.insert(1, sim_engine::FireState { lit: true });
    for (tile, name, count) in [
        ((5, 5), "clay_pot", 1),
        ((6, 6), "wild_grain", 12),
        ((7, 7), "berries", 192),
    ] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(ItemKind::from_name(name).expect("item"), None),
            count,
            14,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("actual ingredients");
    }
    let pot = res.colony.ground_items.remove(&(5, 5)).expect("pot");
    res.colony
        .workshops
        .get_mut(&2)
        .expect("hearth")
        .shelf
        .push(pot);
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.usable_units, 192);
    let allocation =
        super::cooking::priority_targets(&world, &res, &policy, &facts, &Default::default(), false);
    assert_eq!(allocation.targets.get(&recipe), Some(&24));
    let berries = ItemKind::from_name("berries").expect("berries");
    let dry = RecipeKind::from_name("dry_berries").expect("dry");
    assert_eq!(
        allocation.preservation.targets.get(&dry),
        Some(&192),
        "six berries spent cooking leave186 physical berries and6 net-new capacity"
    );
    assert_eq!(allocation.preservation.missing.get(&berries), Some(&6));
    let demand = super::dependencies::ingredient_deficits(&world, &res, &policy, &facts);
    assert_eq!(demand.get(&berries), Some(&6));
}

#[test]
fn winter_forecast_uses_cold_demand_before_people_are_cold() {
    let (mut world, res, policy) = fixture();
    for id in 10..12 {
        world.spawn((
            Agent { id },
            Position::new(8, 8),
            sim_core::components::Child { born_tick: 0 },
        ));
    }
    let people: Vec<_> = world
        .query::<&Agent>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for cold in [false, true] {
        if cold {
            for &person in &people {
                world
                    .insert_one(person, sim_core::components::Cold)
                    .expect("cold person");
            }
        }
        let mut facts = super::summary(&world, &res, &policy);
        assert_eq!(facts.adults, 10);
        assert_eq!(facts.warm_daily_milli, 26880);
        assert_eq!(facts.daily_milli, if cold { 40320 } else { 26880 });
        assert!(facts.preparing);
        // (10 * 2.4 + 2 * 1.44) * 1.5 * 6 = 241.92, rounded up once.
        let (immediate, reserve) = super::harvest::targets(&policy, &facts);
        assert_eq!(
            reserve, 242,
            "winter forecast must not depend on current Cold"
        );
        assert_eq!(policy.reserve_days, 2);
        assert_eq!(immediate, if cold { 81 } else { 54 });
        let mut inputs = super::cooking::free_inputs(&world, &res);
        let mut headroom = 1000;
        let budget =
            super::preservation::budget(&world, &res, &policy, &facts, &mut inputs, &mut headroom);
        assert_eq!(budget.targets.get(&policy.recipes[0]), Some(&242));
        facts.preparing = false;
        assert_eq!(super::harvest::targets(&policy, &facts), (immediate, 0));
        assert!(super::preservation::budget(
            &world,
            &res,
            &policy,
            &facts,
            &mut inputs,
            &mut headroom
        )
        .targets
        .is_empty());
    }
}
