//! Malformed immutable evidence must fail while actual completed operations remain valid.
use super::*;
use crate::runtime::food::{evidence, items, meals};
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::food_evidence::{AcquisitionReason, FlowInterval};
use sim_core::colony::{ItemKind, ItemSpec};
use sim_core::components::{Agent, Hunger};
use sim_core::rules::WorldRules;

fn scene() -> (hecs::World, SimResources) {
    let world = hecs::World::new();
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.current_tick = 2 * res.ticks_per_day;
    (world, res)
}
fn valid(world: &hecs::World, res: &SimResources) -> Result<(), String> {
    crate::runtime::food::validate(world, res)
}
fn intake() -> (hecs::World, SimResources, u32) {
    let (mut world, mut res) = scene();
    let entity = world.spawn((Agent { id: 7 }, Hunger::new(90.0, 0.05)));
    let item = ItemKind::from_name("porridge").unwrap();
    items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(item, None),
        1,
        2,
        items::FoodSource::Debug,
    )
    .unwrap();
    meals::consume(&mut world, &mut res, entity, FoodOwner::Ground((2, 2)), 2).unwrap();
    let index = *res
        .food
        .evidence
        .operations
        .keys()
        .next()
        .expect("actual intake operation");
    valid(&world, &res).expect("actual successful intake admitted");
    (world, res, index)
}

#[test]
fn actual_intake_survives_owner_removal_and_later_expiry_date() {
    let (world, mut res, _) = intake();
    assert!(!res.colony.ground_items.contains_key(&(2, 2)));
    res.current_tick += 100 * res.ticks_per_day;
    valid(&world, &res).expect("validate historical event day, not current freshness");
}

#[test]
fn futureborn_and_already_expired_intake_are_rejected() {
    for (born_day, lifetime_days) in [(3, 10), (0, 2), (0, 1)] {
        let (world, mut res, index) = intake();
        let OperationEvidence::Intake { owner, item, .. } = res.food.evidence.operations[&index]
        else {
            panic!("intake");
        };
        res.food.evidence.operations.insert(
            index,
            OperationEvidence::Intake {
                owner,
                item,
                born_day,
                lifetime_days,
            },
        );
        assert!(
            valid(&world, &res).is_err(),
            "invalid age ({born_day}, {lifetime_days})"
        );
    }
}

#[test]
fn historical_pot_and_shelf_keys_must_match_payload_item_without_live_owner() {
    let (world, mut res, index) = intake();
    let OperationEvidence::Intake {
        item,
        born_day,
        lifetime_days,
        ..
    } = res.food.evidence.operations[&index]
    else {
        panic!("intake");
    };
    let correct = FoodStackKey::from(ItemSpec::raw(item, None));
    let wrong = FoodStackKey::from(ItemSpec::raw(ItemKind::from_name("stew").unwrap(), None));
    for owner in [
        FoodOwner::Pot {
            id: 900,
            key: correct,
        },
        FoodOwner::Shelf {
            workshop: 900,
            key: correct,
        },
    ] {
        res.food.evidence.operations.insert(
            index,
            OperationEvidence::Intake {
                owner,
                item,
                born_day,
                lifetime_days,
            },
        );
        valid(&world, &res).expect("removed historical owner remains legitimate");
    }
    for owner in [
        FoodOwner::Pot {
            id: 900,
            key: wrong,
        },
        FoodOwner::Shelf {
            workshop: 900,
            key: wrong,
        },
    ] {
        res.food.evidence.operations.insert(
            index,
            OperationEvidence::Intake {
                owner,
                item,
                born_day,
                lifetime_days,
            },
        );
        assert!(valid(&world, &res).is_err(), "owner item identity mismatch");
    }
}

#[test]
fn every_operation_entry_requires_its_payload() {
    let (world, mut res, index) = intake();
    res.food.evidence.operations.remove(&index);
    assert!(valid(&world, &res).is_err());
}

fn acquisition(
    res: &mut SimResources,
    target: (u32, u32),
    start: u64,
    receipt_tick: u64,
    end: u64,
) {
    evidence::operation(
        res,
        Some(7),
        OperationEvidence::Acquisition {
            receipt: AcquisitionReceipt {
                tick: receipt_tick,
                target: Some(target),
                job_started: Some(start),
                reason: AcquisitionReason::OutputPlacementFull,
            },
            flow: FlowInterval {
                from_tick: 0,
                to_tick: end,
                units: FlowCounters::default(),
            },
        },
    );
}
#[test]
fn acquisition_chronology_and_map_bounds_are_checked() {
    let (world, mut res) = scene();
    let tick = res.current_tick;
    acquisition(&mut res, (2, 2), 0, tick, tick);
    valid(&world, &res).expect("historical person need not still live");
    for (target, start, receipt_tick, end) in [
        ((8, 2), 0, tick, tick),
        ((2, 2), tick + 1, tick, tick),
        ((2, 2), 0, tick + 1, tick),
        ((2, 2), 0, tick, tick - 1),
    ] {
        let (world, mut res) = scene();
        acquisition(&mut res, target, start, receipt_tick, end);
        assert!(valid(&world, &res).is_err());
    }
}

#[test]
fn daily_observations_require_contiguous_history_and_matching_cursor() {
    let (world, mut res) = scene();
    evidence::observe_day(&mut res);
    res.current_tick += res.ticks_per_day;
    evidence::observe_day(&mut res);
    valid(&world, &res).expect("actual observation intervals admitted");
    let saved = res.food.evidence.clone();
    res.food.evidence.cursor = None;
    assert!(valid(&world, &res).is_err());
    res.food.evidence = saved.clone();
    res.food.evidence.recent.front_mut().unwrap().to_tick += 1;
    assert!(valid(&world, &res).is_err());
    res.food.evidence = saved;
    res.food.evidence.recent.front_mut().unwrap().units.0[0] = 1;
    assert!(
        valid(&world, &res).is_err(),
        "interval cannot exceed lifetime tally"
    );
}

#[test]
fn stored_payload_age_and_owner_identity_are_checked_after_owner_disappears() {
    let (world, mut res) = scene();
    let item = ItemKind::from_name("dried_meat").unwrap();
    let key = FoodStackKey::from(ItemSpec::raw(item, None));
    let owner = FoodOwner::Pot { id: 900, key };
    let index = evidence::operation(
        &mut res,
        None,
        OperationEvidence::Stored {
            owner,
            item,
            pieces: 1,
            born_day: 2,
        },
    );
    valid(&world, &res).expect("historical storage need not exist now");
    res.food.evidence.operations.insert(
        index,
        OperationEvidence::Stored {
            owner,
            item,
            pieces: 1,
            born_day: 3,
        },
    );
    assert!(valid(&world, &res).is_err());
    let wrong = FoodStackKey::from(ItemSpec::raw(
        ItemKind::from_name("dried_berries").unwrap(),
        None,
    ));
    res.food.evidence.operations.insert(
        index,
        OperationEvidence::Stored {
            owner: FoodOwner::Pot {
                id: 900,
                key: wrong,
            },
            item,
            pieces: 1,
            born_day: 2,
        },
    );
    assert!(valid(&world, &res).is_err());
}
