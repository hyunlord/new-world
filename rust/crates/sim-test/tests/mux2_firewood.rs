//! M-UX2: recover split and dropped fuel through ordinary jobs without losing items.
mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, tick_checked, wall, worker};
use sim_core::colony::materials::OAK;
use sim_core::colony::{
    BillMode, BlueprintKind, Command, ItemKind, ItemSpec, WorkPriorities, WorkType,
};
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
    let shop =
        split.then(|| build_workshop(&mut engine, BlueprintKind::WoodBench, (12, 12), (10, 10)));
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: STORE.0,
                y0: STORE.1,
                x1: STORE.0,
                y1: STORE.1,
            }
        )
        .accepted
    );
    if let Some(shop) = shop {
        drop_items(
            &mut engine,
            (16, 12),
            ItemSpec::raw(ItemKind::Log, Some(OAK)),
            100,
        );
        assert!(
            submit(
                &mut engine,
                Command::AddBill {
                    workshop: shop,
                    recipe: sim_core::colony::RecipeKind::SplitFirewood,
                    mode: BillMode::Make(100),
                }
            )
            .accepted
        );
    }
    engine
}

fn wait_for_stock(engine: &mut SimEngine, dropped: bool) {
    for _ in 0..DAY {
        tick_checked(engine);
        if stockpiled(&engine.resources, ItemKind::Firewood) >= (if dropped { 8 } else { 1 })
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
    assert_eq!(
        (
            ledger.created,
            ledger.ground,
            ledger.carried,
            ledger.consumed
        ),
        (8, 8, 0, 0)
    );
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
    let mut entities: Vec<_> = engine
        .world
        .query::<&sim_core::components::Agent>()
        .iter()
        .map(|(entity, agent)| (agent.id, entity))
        .collect();
    entities.sort_by_key(|row| row.0);
    for (_, entity) in entities {
        let mut priorities = WorkPriorities::defaults(false);
        priorities.levels[WorkType::Haul.code() as usize] = 0;
        engine
            .world
            .insert_one(entity, priorities)
            .expect("priorities");
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
fn explicit_split_precedence_and_disabled_haul_are_respected_during_production() {
    for haul in [0, 4] {
        let mut engine = fixture(true);
        let mut entities: Vec<_> = engine
            .world
            .query::<&sim_core::components::Agent>()
            .iter()
            .map(|(entity, agent)| (agent.id, entity))
            .collect();
        entities.sort_by_key(|row| row.0);
        for (_, entity) in entities {
            let mut priorities = WorkPriorities::defaults(false);
            priorities.levels[WorkType::Split.code() as usize] = 1;
            priorities.levels[WorkType::Haul.code() as usize] = haul;
            engine
                .world
                .insert_one(entity, priorities)
                .expect("priorities");
        }
        drop_items(&mut engine, DROP, fuel(), 8);
        for _ in 0..DAY {
            tick_checked(&mut engine);
        }
        assert!(engine.resources.colony.tallies.crafts_done > 0);
        assert_eq!(stockpiled(&engine.resources, ItemKind::Firewood), 0);
        assert_eq!(engine.resources.colony.ground_items[&DROP].count, 8);
    }
}

#[test]
fn an_interrupted_fuel_haul_drops_then_recovers_the_exact_load() {
    use sim_core::colony::{CancelReason, Carry};
    use sim_systems::runtime::jobs::cancel::cancel;
    let mut engine = fixture(false);
    drop_items(&mut engine, DROP, fuel(), 8);
    let mut carrier = None;
    for _ in 0..DAY {
        tick_checked(&mut engine);
        carrier = engine
            .world
            .query::<&Carry>()
            .iter()
            .find(|(_, carry)| carry.kind == ItemKind::Firewood)
            .map(|(entity, _)| entity);
        if carrier.is_some() {
            break;
        }
    }
    let carrier = carrier.expect("ordinary haul picked up fuel");
    cancel(
        &mut engine.world,
        &mut engine.resources,
        carrier,
        CancelReason::Needs,
    );
    assert!(engine.resources.colony.reservations.is_empty());
    assert_eq!(
        ledger_of(&engine.world, &engine.resources, ItemKind::Firewood).carried,
        0
    );
    for _ in 0..DAY {
        tick_checked(&mut engine);
    }
    assert_eq!(stockpiled(&engine.resources, ItemKind::Firewood), 8);
    let ledger = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
    assert_eq!(
        (
            ledger.created,
            ledger.ground,
            ledger.carried,
            ledger.consumed
        ),
        (8, 8, 0, 0)
    );
    assert!(engine.resources.colony.reservations.is_empty());
}

fn competing_world(work: WorkType, priority: u8) -> SimEngine {
    let mut engine = fixture(false);
    if work == WorkType::Hunt {
        let mut rule = sim_core::rules::ColonyRule::m5_5();
        rule.knowledge = None;
        engine.resources.rules.colony = Some(rule);
        engine.resources.colony.wildlife.seeded = true;
        let animal = sim_systems::runtime::wildlife::spawn_deer(&mut engine.resources, (18, 10))
            .expect("controlled hunt target");
        assert!(submit(&mut engine, Command::DesignateHunt { animal }).accepted);
    } else {
        let tiles: Vec<_> = (3..9).flat_map(|x| (16..22).map(move |y| (x, y))).collect();
        for &tile in &tiles {
            colony_support::plant(&mut engine, tile, 4);
        }
        assert!(submit(&mut engine, Command::DesignateChopTiles { tiles }).accepted);
    }
    let entity = engine
        .world
        .query::<&sim_core::components::Agent>()
        .iter()
        .next()
        .map(|(entity, _)| entity)
        .expect("one worker");
    let mut priorities = WorkPriorities::defaults(false);
    priorities.levels[work.code() as usize] = priority;
    engine
        .world
        .insert_one(entity, priorities)
        .expect("priorities");
    drop_items(&mut engine, DROP, fuel(), 8);
    engine
}

#[test]
fn fuel_haul_wins_equal_priority_hunt_and_chop_at_next_assignment() {
    for work in [WorkType::Hunt, WorkType::Chop] {
        let mut engine = competing_world(work, 3);
        tick_checked(&mut engine);
        let jobs: Vec<_> = engine
            .world
            .query::<&sim_core::colony::Job>()
            .iter()
            .map(|(_, job)| job.kind)
            .collect();
        assert_eq!(jobs.len(), 1);
        assert!(
            matches!(
                jobs[0],
                sim_core::colony::JobKind::Haul {
                    from: DROP,
                    to: STORE
                }
            ),
            "equal-priority {work:?} must not strand usable fuel: {jobs:?}"
        );
        wait_for_stock(&mut engine, true);
    }
}

#[test]
fn fuel_haul_does_not_override_explicit_hunt_or_chop_priority() {
    for work in [WorkType::Hunt, WorkType::Chop] {
        let mut engine = competing_world(work, 1);
        tick_checked(&mut engine);
        let jobs: Vec<_> = engine
            .world
            .query::<&sim_core::colony::Job>()
            .iter()
            .map(|(_, job)| job.kind)
            .collect();
        assert_eq!(jobs.len(), 1);
        assert!(
            matches!(
                (work, jobs[0]),
                (WorkType::Hunt, sim_core::colony::JobKind::Hunt { .. })
                    | (WorkType::Chop, sim_core::colony::JobKind::Chop { .. })
            ),
            "explicit higher priority remains authoritative: {jobs:?}"
        );
    }
}

#[test]
fn dropped_fuel_reaches_stockpile_despite_a_day_of_chop_backlog() {
    let mut engine = competing_world(WorkType::Chop, 3);
    wait_for_stock(&mut engine, true);
    assert!(!engine.resources.colony.designations.is_empty());
}
