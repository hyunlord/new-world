//! M2 Part 2: workshops, bills and recipes (goal 3.3–3.5, §6).

mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, run_until, tick_checked, worker};
use sim_core::colony::materials::FLINT;
use sim_core::colony::{
    BillMode, BlueprintKind, CancelReason, Command, ItemKind, ItemSpec, Job, JobKind, JobStage,
    Occupant, RecipeKind, RejectReason, ZoneKind,
};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::bills::stockpiled;
use sim_systems::runtime::jobs::invariants::ledger_of;

fn flint(n: u16) -> (ItemSpec, u16) {
    (ItemSpec::raw(ItemKind::Flint, Some(FLINT)), n)
}

fn knapping_world() -> (SimEngine, u32) {
    let mut engine = job_world_m2(32, 32);
    for x in [3, 5, 7] {
        worker(&mut engine, x, 3);
    }
    let shop = build_workshop(&mut engine, BlueprintKind::KnappingSpot, (12, 12), (4, 4));
    (engine, shop)
}

#[test]
fn a_knapping_spot_is_built_from_four_stone() {
    let (engine, shop) = knapping_world();
    let c = &engine.resources.colony;
    let found = &c.workshops[&shop];
    assert_eq!(
        (found.kind, found.size, found.work_tile()),
        (BlueprintKind::KnappingSpot, (1, 1), (12, 13))
    );
    assert_eq!(c.occupied.get(&(12, 12)), Some(&Occupant::Workshop(shop)));
    let stone = ledger_of(&engine.world, &engine.resources, ItemKind::Stone);
    assert_eq!((stone.created, stone.consumed, stone.ground), (4, 4, 0));
    assert_eq!(c.tallies.workshops_built, 1);
    assert!(
        !engine.resources.building_kinds.contains_key(&(12, 12)),
        "a workshop is not a dwelling"
    );
}

#[test]
fn a_knap_bill_of_5_makes_5_blades_from_5_flint() {
    let (mut engine, shop) = knapping_world();
    let (spec, n) = flint(8);
    drop_items(&mut engine, (20, 20), spec, n);
    let added = submit(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::KnapBlade,
            mode: BillMode::Make(5),
        },
    );
    assert!(added.accepted);
    let used = run_until(&mut engine, 4_000, |e| {
        e.resources.colony.tallies.crafts_done >= 5
    });
    assert!(used < 4_000, "5 blades within 4,000 ticks");
    // Let the bill system see the bill done and the jobs settle.
    for _ in 0..300 {
        tick_checked(&mut engine);
    }
    let blades = ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade);
    let fl = ledger_of(&engine.world, &engine.resources, ItemKind::Flint);
    assert_eq!(
        (blades.created, fl.consumed),
        (5, 5),
        "exactly five crafts, one flint each"
    );
    let c = &engine.resources.colony;
    assert_eq!(c.item_ledger.crafted_of(ItemKind::FlintBlade), 5);
    assert_eq!(c.item_ledger.transforms, 5);
    assert_eq!(c.workshops[&shop].bills[0].done, 5);
    assert_eq!(
        c.workshops[&shop].active, None,
        "a finished bill is not active"
    );
    for stack in c
        .ground_items
        .values()
        .filter(|s| s.kind == ItemKind::FlintBlade)
    {
        assert_eq!(stack.material, Some(FLINT), "a blade keeps its flint");
        // Level 0–1 crafters: 30 + 3·level + bonus (−10…15) + flint's +4.
        assert!(
            (24..=52).contains(&stack.quality),
            "quality {}",
            stack.quality
        );
    }
}

#[test]
fn maintain_10_firewood_keeps_the_stock_between_10_and_10_plus_a_carry() {
    let mut engine = job_world_m2(40, 40);
    for x in [3, 5] {
        worker(&mut engine, x, 3);
    }
    let shop = build_workshop(&mut engine, BlueprintKind::WoodBench, (12, 12), (4, 4));
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 20,
                y0: 20,
                x1: 23,
                y1: 23
            }
        )
        .accepted
    );
    drop_items(
        &mut engine,
        (30, 30),
        ItemSpec::raw(ItemKind::Log, None),
        25,
    );
    drop_items(
        &mut engine,
        (31, 30),
        ItemSpec::raw(ItemKind::Log, None),
        25,
    );
    let bill = Command::AddBill {
        workshop: shop,
        recipe: RecipeKind::SplitFirewood,
        mode: BillMode::Maintain(10),
    };
    assert!(submit(&mut engine, bill).accepted);
    let reached = run_until(&mut engine, 4_000, |e| {
        stockpiled(&e.resources, ItemKind::Firewood) >= 10
    });
    assert!(reached < 4_000, "10 firewood stocked");
    let (mut low, mut high) = (u32::MAX, 0);
    for tick in 0..10 * 1_440u64 {
        tick_checked(&mut engine);
        if tick % 60 == 0 {
            let n = stockpiled(&engine.resources, ItemKind::Firewood);
            low = low.min(n);
            high = high.max(n);
        }
        // Burn 4 firewood from the stockpile every day at noon, as the hearth will.
        if tick % 1_440 == 720 {
            let c = &engine.resources.colony;
            let tile = c
                .ground_items
                .iter()
                .find(|(t, s)| s.kind == ItemKind::Firewood && c.zone_of_tile.contains_key(t))
                .map(|(t, _)| *t);
            if let Some(tile) = tile {
                let (taken, _) = sim_systems::runtime::jobs::items::take_items(
                    &mut engine.resources,
                    tile,
                    ItemKind::Firewood,
                    4,
                );
                engine
                    .resources
                    .colony
                    .item_ledger
                    .burn(ItemKind::Firewood, u64::from(taken));
            }
        }
    }
    // 10 ± a firewood carry (10): never below 6 after a burn of 4, never over 20.
    assert!(low >= 6 && high <= 20, "stock ranged {low}–{high}");
    let wood = ledger_of(&engine.world, &engine.resources, ItemKind::Firewood);
    assert!(wood.burned_of_check(), "firewood burned");
    assert!(
        engine
            .resources
            .colony
            .item_ledger
            .crafted_of(ItemKind::Firewood)
            >= 40
    );
    let zones = engine
        .resources
        .colony
        .zones
        .values()
        .filter(|z| z.kind == ZoneKind::Stockpile)
        .count();
    assert_eq!(zones, 1);
}

trait Burned {
    fn burned_of_check(&self) -> bool;
}

impl Burned for sim_systems::runtime::jobs::invariants::LedgerTerms {
    fn burned_of_check(&self) -> bool {
        self.consumed > 0 && self.balanced()
    }
}

#[test]
fn a_craft_cancelled_after_it_started_loses_its_inputs() {
    let (mut engine, shop) = knapping_world();
    let (spec, n) = flint(3);
    drop_items(&mut engine, (20, 20), spec, n);
    let bill_id = engine.resources.colony.next_bill_id.max(1);
    submit(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::KnapBlade,
            mode: BillMode::Make(3),
        },
    );
    let crafting = |e: &SimEngine| {
        e.world.query::<&Job>().iter().any(|(_, j)| {
            matches!(j.kind, JobKind::Craft { .. }) && matches!(j.stage, JobStage::Working { .. })
        })
    };
    let used = run_until(&mut engine, 3_000, crafting);
    assert!(used < 3_000, "a craft started");
    let consumed_before = ledger_of(&engine.world, &engine.resources, ItemKind::Flint).consumed;
    assert_eq!(
        consumed_before, 1,
        "the input is consumed when the craft starts"
    );
    let removed = submit(
        &mut engine,
        Command::RemoveBill {
            workshop: shop,
            bill: bill_id,
        },
    );
    assert!(removed.accepted);
    let c = &engine.resources.colony;
    assert_eq!(c.tallies.crafts_lost, 1);
    assert_eq!(c.tallies.cancels_of(CancelReason::Command), 1);
    assert!(!c
        .reservations
        .keys()
        .any(|k| matches!(k, sim_core::colony::ReserveKey::Bench(_))));
    for _ in 0..600 {
        tick_checked(&mut engine);
    }
    let fl = ledger_of(&engine.world, &engine.resources, ItemKind::Flint);
    let blades = ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade);
    assert_eq!(
        (fl.consumed, blades.created),
        (1, 0),
        "the lost flint is logged consumed, no blade"
    );
    assert_eq!(
        fl.ground + fl.carried + fl.shelved,
        2,
        "the other two flint are still in the world"
    );
    assert_eq!(fl.shelved, 0, "a removed bill's shelf items are put down");
}

#[test]
fn bill_commands_check_their_workshop_and_bill_and_are_logged() {
    let (mut engine, shop) = knapping_world();
    let wrong = submit(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::HaftAxe,
            mode: BillMode::Make(1),
        },
    );
    assert_eq!(wrong.reason, Some(RejectReason::WrongWorkshop));
    let unknown = submit(
        &mut engine,
        Command::AddBill {
            workshop: 99,
            recipe: RecipeKind::KnapBlade,
            mode: BillMode::Make(1),
        },
    );
    assert_eq!(unknown.reason, Some(RejectReason::Unknown));
    for n in [2u16, 3] {
        let ok = submit(
            &mut engine,
            Command::AddBill {
                workshop: shop,
                recipe: RecipeKind::KnapBlade,
                mode: BillMode::Make(n),
            },
        );
        assert!(ok.accepted);
    }
    let ids: Vec<u32> = engine.resources.colony.workshops[&shop]
        .bills
        .iter()
        .map(|b| b.id)
        .collect();
    assert_eq!(ids.len(), 2);
    assert!(
        submit(
            &mut engine,
            Command::MoveBill {
                workshop: shop,
                bill: ids[1],
                up: true
            }
        )
        .accepted
    );
    let order: Vec<u32> = engine.resources.colony.workshops[&shop]
        .bills
        .iter()
        .map(|b| b.id)
        .collect();
    assert_eq!(order, vec![ids[1], ids[0]]);
    let top = submit(
        &mut engine,
        Command::MoveBill {
            workshop: shop,
            bill: ids[1],
            up: true,
        },
    );
    assert_eq!(top.reason, Some(RejectReason::Nothing));
    let suspend = Command::SetBillSuspended {
        workshop: shop,
        bill: ids[1],
        suspended: true,
    };
    assert!(submit(&mut engine, suspend).accepted);
    let mode = Command::SetBillMode {
        workshop: shop,
        bill: ids[0],
        mode: BillMode::Maintain(4),
    };
    assert!(submit(&mut engine, mode).accepted);
    let b0 = engine.resources.colony.workshops[&shop]
        .bill(ids[0])
        .copied();
    assert_eq!(
        b0.map(|b| (b.mode, b.done)),
        Some((BillMode::Maintain(4), 0))
    );
    let next = engine.current_tick();
    let log = &engine.resources.colony.command_log;
    assert!(
        log.iter().rev().take(6).all(|r| r.tick == next),
        "logged with the next tick"
    );
    assert_eq!(log.last().map(|r| r.command.name()), Some("set_bill_mode"));
}
