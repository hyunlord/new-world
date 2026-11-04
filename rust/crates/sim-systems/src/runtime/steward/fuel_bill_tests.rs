//! Seasonal fuel reconciliation preserves ordinary player bill ownership.
use super::*;
use sim_core::colony::knowledge::{Knowledge, KnowledgeEntry, Source};
use sim_core::colony::Workshop;

fn fixture(owned: bool) -> (hecs::World, SimResources, Rule, u32) {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let mut engine = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(1),
        sim_core::rules::WorldRules::colony(1),
    );
    engine.resources.rules.clock_offset_ticks = 0;
    for (_, known) in engine.world.query::<&mut Knowledge>().iter() {
        *known = Knowledge::from_entries(
            knowledge::defs()
                .nodes
                .iter()
                .map(|node| KnowledgeEntry::new(node.id, Source::Oral, 1000, 0))
                .collect(),
        );
    }
    let rule = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find(|rule| rule.id == 9)
        .expect("fuel")
        .clone();
    let Action::Bills {
        winter_recipe,
        days,
        fuel,
        ..
    } = &rule.action
    else {
        panic!("fuel bills");
    };
    let target = fuel_target(&engine.resources, *winter_recipe, *days, fuel.as_ref());
    let recipe = *winter_recipe;
    engine.resources.colony.workshops.insert(
        99,
        Workshop {
            id: 99,
            kind: recipe.recipe().expect("recipe").workshop,
            top_left: (120, 120),
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        },
    );
    let add = Command::AddBill {
        workshop: 99,
        recipe,
        mode: BillMode::Maintain(u16::try_from(target).expect("bounded fuel target")),
    };
    let command = if owned {
        Command::Steward {
            rule: 9,
            command: Box::new(add),
        }
    } else {
        add
    };
    assert!(
        crate::commands::apply_and_log(&mut engine.world, &mut engine.resources, command, 0)
            .accepted
    );
    let bill = engine.resources.colony.workshops[&99].bills[0].id;
    assert!(!engine.resources.colony.workshops[&99].bills[0].auto);
    (engine.world, engine.resources, rule, bill)
}

#[test]
fn owned_fuel_bill_accepts_summer_target_without_duplicate_bill() {
    let (mut world, mut res, rule, bill) = fixture(true);
    let before = res.colony.workshops[&99].bills[0].mode;
    res.current_tick = 6 * res.ticks_per_day;
    let commands = plan(&world, &res, &rule);
    let update = commands
        .iter()
        .find(|command| {
            matches!(command,
        Command::SetBillMode { workshop: 99, bill: id, .. } if *id == bill)
        })
        .expect("warm bill must be reconciled in summer")
        .clone();
    let Command::SetBillMode { mode, .. } = update else {
        panic!("mode update");
    };
    assert_ne!(mode, before);
    assert_eq!(mode, BillMode::Maintain(296));
    assert!(!commands
        .iter()
        .any(|command| matches!(command, Command::AddBill { workshop: 99, .. })));
    let tick = res.current_tick;
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 9,
                command: Box::new(update)
            },
            tick
        )
        .accepted
    );
    assert_eq!(res.colony.workshops[&99].bills.len(), 1);
    assert_eq!(res.colony.workshops[&99].bills[0].mode, mode);
}

#[test]
fn player_created_edited_or_paused_fuel_bills_are_not_reconciled() {
    for variant in 0..3 {
        let (mut world, mut res, rule, bill) = fixture(variant != 0);
        if variant != 0 {
            let command = if variant == 1 {
                Command::SetBillMode {
                    workshop: 99,
                    bill,
                    mode: BillMode::Maintain(12),
                }
            } else {
                Command::SetBillSuspended {
                    workshop: 99,
                    bill,
                    suspended: true,
                }
            };
            assert!(crate::commands::apply_and_log(&mut world, &mut res, command, 1).accepted);
        }
        res.current_tick = 6 * res.ticks_per_day;
        assert!(!plan(&world, &res, &rule).iter().any(|command| matches!(
            command,
            Command::SetBillMode { workshop: 99, .. }
                | Command::SetBillSuspended { workshop: 99, .. }
                | Command::AddBill { workshop: 99, .. }
        )));
        supply_hearth_reserve(&mut res);
        assert!(!plan(&world, &res, &rule).iter().any(|command| matches!(
            command,
            Command::SetBillMode { workshop: 99, .. }
                | Command::SetBillSuspended { workshop: 99, .. }
                | Command::AddBill { workshop: 99, .. }
        )));
    }
}

#[test]
fn eligible_fuel_bill_is_updated_by_steward_without_grace() {
    use sim_engine::RuntimeSystem;

    let (mut world, mut res, rule, bill) = fixture(true);
    assert!(!rule.extended);
    assert!(rule.grace_ticks.is_none());
    res.steward.enabled = true;
    res.current_tick = 6 * res.ticks_per_day;
    let tick = res.current_tick;
    super::super::StewardSystem.tick(&mut world, &mut res);
    assert!(res.colony.command_log.iter().any(|record| {
        record.tick == tick
            && record.result.accepted
            && matches!(&record.command, Command::Steward { rule: 9, command }
                if matches!(**command, Command::SetBillMode {
                    workshop: 99, bill: id, mode: BillMode::Maintain(296)
                } if id == bill))
    }));
    assert_eq!(
        res.colony.workshops[&99].bills[0].mode,
        BillMode::Maintain(296)
    );
}

fn supply_hearth_reserve(res: &mut SimResources) {
    let hearth = hearth_tiles(res)[0];
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.formation_tile = hearth;
    settlement
        .stockpile
        .insert(sim_core::components::ResourceKind::Wood, 10_000);
    res.settlements.insert(1, settlement);
}

#[test]
fn sufficient_hearth_stock_reconciles_owned_fuel_bill_to_zero() {
    let (mut world, mut res, rule, bill) = fixture(true);
    supply_hearth_reserve(&mut res);
    let commands = plan(&world, &res, &rule);
    let update = commands
        .iter()
        .find(|command| {
            matches!(command,
        Command::SetBillMode { workshop: 99, bill: id, mode: BillMode::Maintain(0) }
            if *id == bill)
        })
        .expect("covered fuel demand must stop existing production")
        .clone();
    assert!(
        crate::commands::apply_and_log(
            &mut world,
            &mut res,
            Command::Steward {
                rule: 9,
                command: Box::new(update)
            },
            1
        )
        .accepted
    );
    assert_eq!(res.colony.workshops[&99].bills.len(), 1);
    assert_eq!(
        res.colony.workshops[&99].bills[0].mode,
        BillMode::Maintain(0)
    );
    assert!(!plan(&world, &res, &rule).iter().any(|command| matches!(
        command,
        Command::AddBill { .. } | Command::SetBillMode { .. } | Command::PlaceBlueprint { .. }
    )));
}

#[test]
fn zero_fuel_demand_creates_neither_bill_nor_workshop() {
    for existing_workshop in [true, false] {
        let (world, mut res, rule, _) = fixture(true);
        supply_hearth_reserve(&mut res);
        if existing_workshop {
            res.colony
                .workshops
                .get_mut(&99)
                .expect("workshop")
                .bills
                .clear();
        } else {
            res.colony.workshops.remove(&99);
        }
        assert!(!plan(&world, &res, &rule).iter().any(|command| matches!(
            command,
            Command::AddBill { .. } | Command::SetBillMode { .. } | Command::PlaceBlueprint { .. }
        )));
    }
}
