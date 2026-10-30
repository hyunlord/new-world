//! Stock harvesting respects restored owners and physical transport boundaries.
use super::harvest_stock_tests::{fixture, mind};
use sim_core::colony::{Job, JobKind};
use sim_core::components::{Activity, ActivityKind, AgentState, SeekTarget, TargetKind};

#[test]
fn restored_routine_yields_to_proactive_stock_harvest() {
    let (mut world, mut res, entity) = fixture();
    world
        .insert(
            entity,
            (
                Activity {
                    kind: ActivityKind::SitByFire,
                    settlement: 1,
                    place: (2, 2),
                    chosen_tick: 710,
                    arrived_tick: Some(710),
                    until_tick: Some(900),
                },
                AgentState::Consuming {
                    target: TargetKind::Activity,
                },
                SeekTarget::new((2, 2)),
            ),
        )
        .expect("restored routine");
    mind()
        .try_tick(&mut world, &mut res)
        .expect("stock decision");
    assert!(world
        .get::<&Job>(entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. })));
    assert!(world.get::<&Activity>(entity).is_err());
}

#[test]
fn reserved_storage_is_not_stolen_by_harvest_handoff() {
    use sim_core::colony::ReserveKey;
    use sim_engine::RuntimeSystem;
    let (mut world, mut res, entity) = fixture();
    use crate::runtime::food::items::{self, FoodSource};
    use sim_core::colony::{ItemKind, ItemSpec, JobStage};
    use sim_core::components::{Agent, Position};
    let roots = ItemKind::from_name("roots").expect("roots");
    items::create_ground(
        &mut res,
        (0, 0),
        ItemSpec::raw(roots, None),
        1,
        0,
        FoodSource::Debug,
    )
    .expect("other haul source");
    let other_job = Job {
        kind: JobKind::Haul {
            from: (0, 0),
            to: (6, 6),
        },
        stage: JobStage::ToSource,
        taken_at: 720,
        amount: 1,
        item: roots,
        material: None,
        fetch: None,
        best_distance: 7,
        progress_tick: 720,
    };
    world.spawn((
        Agent { id: 99 },
        Position::new(7, 0),
        other_job,
        AgentState::Seeking {
            target: TargetKind::Job,
        },
        SeekTarget::new((0, 0)),
    ));
    for key in other_job.keys() {
        res.colony.reservations.insert(key, 99);
    }
    crate::runtime::jobs::invariants::check(&world, &res).expect("valid competing haul");
    super::harvest_stock_tests::ready_harvest(&mut world, &mut res, entity);
    crate::runtime::jobs::JobExecuteSystem::new().tick(&mut world, &mut res);
    assert!(world.get::<&Job>(entity).is_err());
    assert_eq!(res.colony.reservations[&ReserveKey::Cell((6, 6))], 99);
    assert_eq!(res.colony.ground_items[&(2, 2)].count, 5);
    assert_eq!(res.food.tallies.harvested_units, 5);
}

#[test]
fn occupied_plant_tile_hands_off_the_actual_spill_placement() {
    use crate::runtime::food::items::{self, FoodSource};
    use sim_core::colony::{ItemKind, ItemSpec};
    use sim_engine::RuntimeSystem;
    let (mut world, mut res, entity) = fixture();
    let log = ItemSpec::raw(ItemKind::from_name("log").expect("log"), None);
    items::create_ground(&mut res, (2, 2), log, 1, 0, FoodSource::Debug).expect("occupied tile");
    super::harvest_stock_tests::ready_harvest(&mut world, &mut res, entity);
    crate::runtime::jobs::JobExecuteSystem::new().tick(&mut world, &mut res);
    let job = *world.get::<&Job>(entity).expect("haul actual output");
    let JobKind::Haul { from, to } = job.kind else {
        panic!("haul expected")
    };
    assert_ne!(from, (2, 2));
    assert_eq!(to, (6, 6));
    assert_eq!(res.colony.ground_items[&from].kind, job.item);
    assert_eq!(res.colony.ground_items[&from].count, 5);
    assert_eq!(res.colony.ground_items[&(2, 2)].kind, log.kind);
    crate::runtime::jobs::invariants::check(&world, &res).expect("spill conservation");
}

#[test]
fn authored_hoard_cap_stops_proactive_harvest() {
    use crate::runtime::food::items::{self, FoodSource};
    use sim_core::colony::{ItemKind, ItemSpec};
    let (mut world, mut res, entity) = fixture();
    let target = crate::runtime::settlement::food_economy::stock_target(&world, &res, 1)
        .expect("authored target");
    let cap = target
        * res
            .rules
            .society
            .stock_target
            .expect("stock rule")
            .hoard_cap_factor;
    let total = cap.ceil() as u16;
    assert!(total > 1);
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    items::create_ground(&mut res, (6, 6), roots, total - 1, 0, FoodSource::Debug)
        .expect("fill to cap");
    assert!(crate::runtime::food::reserves::units(&world, &res, 1, false) as f64 >= cap);
    mind()
        .try_tick(&mut world, &mut res)
        .expect("stock decision");
    assert!(world.get::<&Job>(entity).is_err());
    assert_eq!(res.food.tallies.harvested_units, 0);
}
