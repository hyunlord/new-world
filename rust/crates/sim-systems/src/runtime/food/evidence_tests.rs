//! Observation bounds and chronology do not alter physical food decisions.
use super::*;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{CancelReason, ItemKind, ItemSpec};
use sim_core::components::{Agent, Hunger};
use sim_core::rules::WorldRules;

fn scene() -> SimResources {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res
}
fn failure(tick: u64, reason: AcquisitionReason) -> AcquisitionReceipt {
    AcquisitionReceipt {
        tick,
        target: Some((2, 2)),
        job_started: Some(0),
        reason,
    }
}
#[test]
fn cancellation_does_not_erase_specific_same_job_failure() {
    let mut res = scene();
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::OutputPlacementFull),
    );
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::Cancelled(CancelReason::TargetGone)),
    );
    assert_eq!(
        res.food.evidence.active[&7].last.reason,
        AcquisitionReason::OutputPlacementFull
    );
    res.current_tick = 2;
    res.food.tallies.harvested_units = 4;
    res.food.tallies.eaten_units = 2;
    let index = death_parent(&mut res, 7).unwrap();
    let OperationEvidence::Acquisition { flow, receipt } = res.food.evidence.operations[&index]
    else {
        panic!("receipt")
    };
    assert_eq!((flow.from_tick, flow.to_tick), (0, 2));
    assert_eq!(flow.units.0[0], 4);
    assert_eq!(flow.units.0[3], 2);
    assert_eq!(receipt.reason, AcquisitionReason::OutputPlacementFull);
    assert!(!res.food.evidence.active.contains_key(&7));
}
#[test]
fn successful_meal_invalidates_old_failed_acquisition() {
    let mut res = scene();
    let mut world = hecs::World::new();
    let entity = world.spawn((Agent { id: 7 }, Hunger::new(90.0, 0.05)));
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::TargetUnavailable),
    );
    let item = ItemKind::from_name("roots").unwrap();
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(item, None),
        1,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .unwrap();
    crate::runtime::food::meals::consume(
        &mut world,
        &mut res,
        entity,
        FoodOwner::Ground((2, 2)),
        0,
    )
    .unwrap();
    assert!(death_parent(&mut res, 7).is_none());
}
#[test]
fn daily_flow_history_is_bounded_and_duplicate_boundary_is_inert() {
    let mut res = scene();
    observe_day(&mut res);
    for tick in 1..=7 {
        res.current_tick = tick;
        res.food.tallies.harvested_units += 3;
        res.food.tallies.spoiled_units += 1;
        observe_day(&mut res);
        observe_day(&mut res);
    }
    assert_eq!(res.food.evidence.recent.len(), 4);
    for row in &res.food.evidence.recent {
        assert_eq!(row.to_tick - row.from_tick, 1);
        assert_eq!(row.units.0[0], 3);
        assert_eq!(row.units.0[4], 1);
    }
}
#[test]
fn baseline_observers_are_inert() {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::OutputPlacementFull),
    );
    observe_day(&mut res);
    assert_eq!(res.food.evidence, Default::default());
}
