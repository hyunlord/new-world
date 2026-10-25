//! M-UX2: recover split and dropped fuel through ordinary jobs without losing items.
mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, tick_checked, wall, worker};
use sim_core::colony::materials::OAK;
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, WorkPriorities, WorkType};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::bills::stockpiled;
use sim_systems::runtime::jobs::invariants::ledger_of;
use sim_systems::runtime::jobs::items::stack_max;

const STORE: (u32, u32) = (24, 24);
const DROP: (u32, u32) = (14, 16);
const DAY: u64 = 1_440;

fn fuel() -> ItemSpec {
    ItemSpec::raw(ItemKind::Firewood, Some(OAK))
}

fn fixture(split: bool) -> SimEngine {
    let mut engine = job_world_m2(32, 32);
    worker(&mut engine, 10, 10);
    let shop = split.then(|| {
        build_workshop(&mut engine, BlueprintKind::WoodBench, (12, 12), (10, 10))
    });
    assert!(submit(&mut engine, Command::AddStockpile {
        x0: STORE.0, y0: STORE.1, x1: STORE.0, y1: STORE.1,
    }).accepted);
    if let Some(shop) = shop {
        drop_items(&mut engine, (16, 12), ItemSpec::raw(ItemKind::Log, Some(OAK)), 100);
        assert!(submit(&mut engine, Command::AddBill {
            workshop: shop, recipe: sim_core::colony::RecipeKind::SplitFirewood,
            mode: BillMode::Make(100),
        }).accepted);
    }
    engine
}

fn wait_for_stock(engine: &mut SimEngine, dropped: bool) {
    for _ in 0..DAY {
        tick_checked(engine);
        if stockpiled(&engine.resources, ItemKind::Firewood) >= if dropped { 8 } else { 1 }
            && (!dropped || !engine.resources.colony.ground_items.contains_key(&DROP))
        {
            let ledger = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
            assert!(ledger.balanced());
            assert_eq!(ledger.consumed, 0);
            return;
        }
    }
    let ledger = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
    panic!("fuel did not reach available stockpile within one day: {ledger:?}");
}

#[test]
fn dropped_fuel_reaches_stockpile_while_split_bill_still_has_work() {
    let mut engine = fixture(true);
    drop_items(&mut engine, DROP, fuel(), 8);
    wait_for_stock(&mut engine, true);
    assert!(engine.resources.colony.tallies.crafts_done < 100);
}

#[test]
fn freshly_split_fuel_reaches_stockpile_before_finishing_large_bill() {
    let mut engine = fixture(true);
    wait_for_stock(&mut engine, false);
    assert!(engine.resources.colony.tallies.crafts_done > 0);
    assert!(engine.resources.colony.tallies.crafts_done < 100);
}

#[test]
fn dropped_fuel_without_competing_work_reaches_stockpile_exactly() {
    let mut engine = fixture(false);
    drop_items(&mut engine, DROP, fuel(), 8);
    for _ in 0..DAY {
        tick_checked(&mut engine);
    }
    assert_eq!(stockpiled(&engine.resources, ItemKind::Firewood), 8);
    let ledger = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
    assert_eq!((ledger.created, ledger.ground, ledger.carried, ledger.consumed), (8, 8, 0, 0));
    assert!(engine.resources.colony.reservations.is_empty());
}

#[test]
fn full_or_incompatible_stockpile_never_takes_dropped_fuel() {
    for occupied in [fuel(), ItemSpec::raw(ItemKind::Stone, None)] {
        let mut engine = fixture(false);
        let count = stack_max(&engine.resources, occupied.kind);
        drop_items(&mut engine, STORE, occupied, count);
        drop_items(&mut engine, DROP, fuel(), 8);
        for _ in 0..DAY {
            tick_checked(&mut engine);
        }
        let c = &engine.resources.colony;
        assert_eq!(c.ground_items[&DROP].count, 8);
        assert_eq!(c.ground_items[&STORE].spec(), occupied);
        assert_eq!(c.ground_items[&STORE].count, count);
        assert!(c.reservations.is_empty());
    }
}

#[test]
fn unreachable_stockpile_never_reserves_or_removes_dropped_fuel() {
    let mut engine = fixture(false);
    for y in 0..32 {
        wall(&mut engine, (20, y), false);
    }
    drop_items(&mut engine, DROP, fuel(), 8);
    for _ in 0..DAY {
        tick_checked(&mut engine);
    }
    assert_eq!(stockpiled(&engine.resources, ItemKind::Firewood), 0);
    assert_eq!(engine.resources.colony.ground_items[&DROP].count, 8);
    assert!(engine.resources.colony.reservations.is_empty());
}

#[test]
fn disabled_hauling_does_not_override_player_work_priorities() {
    let mut engine = fixture(false);
    let entities: Vec<_> = engine.world.query::<&sim_core::components::Agent>().iter().map(|(e, _)| e).collect();
    for entity in entities {
        let mut priorities = WorkPriorities::defaults(false);
        priorities.levels[WorkType::Haul.code() as usize] = 0;
        engine.world.insert_one(entity, priorities).expect("priorities");
    }
    drop_items(&mut engine, DROP, fuel(), 8);
    for _ in 0..DAY {
        tick_checked(&mut engine);
    }
    assert_eq!(stockpiled(&engine.resources, ItemKind::Firewood), 0);
    assert_eq!(engine.resources.colony.ground_items[&DROP].count, 8);
    assert!(engine.resources.colony.reservations.is_empty());
}
