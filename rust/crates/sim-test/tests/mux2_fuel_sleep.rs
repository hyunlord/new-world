//! M-UX2: an adult already due to sleep must not steal the sole fuel job.
mod colony_support;

use colony_support::{bedroll, job_world_m2, tick_checked, worker};
use sim_core::colony::{CancelReason, Command, Job, JobKind, ReserveKey};
use sim_core::components::{Agent, BuildingKind, Sleep};
use sim_core::rules::{ColonyRule, WorldRules};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::decision::AgentDecisionSystem;

fn fixture(fatigue: f64) -> (SimEngine, hecs::Entity, hecs::Entity) {
    let mut engine = job_world_m2(32, 32);
    let mut colony = ColonyRule::m5_5();
    colony.knowledge = None;
    engine.resources.rules.colony = Some(colony);
    engine.resources.rules.society.firewood = WorldRules::baseline(1).society.firewood;
    engine
        .resources
        .building_kinds
        .insert((8, 8), BuildingKind::Hearth);
    engine.resources.wood_tiles.insert((14, 10), 10);
    let exhausted = worker(&mut engine, 10, 10);
    let rested = worker(&mut engine, 11, 10);
    engine
        .world
        .insert_one(exhausted, Sleep::new(fatigue, 0.0))
        .expect("sleep");
    engine
        .world
        .insert_one(rested, Sleep::new(0.0, 0.0))
        .expect("sleep");
    bedroll(&mut engine, (9, 10));
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 20,
                y0: 20,
                x1: 20,
                y1: 20,
            }
        )
        .accepted
    );
    engine.register_system(Box::new(AgentDecisionSystem::new()));
    (engine, exhausted, rested)
}

#[test]
fn exhausted_adult_does_not_claim_and_cancel_the_rested_workers_fuel_job() {
    let (mut engine, exhausted, rested) = fixture(100.0);
    tick_checked(&mut engine);
    let rested_job = engine.world.get::<&Job>(rested).ok().map(|job| *job);
    let exhausted_job = engine.world.get::<&Job>(exhausted).ok().map(|job| *job);
    let cancellations = engine
        .resources
        .colony
        .tallies
        .cancels_of(CancelReason::Needs);
    println!("rested={rested_job:?} exhausted={exhausted_job:?} needs_cancels={cancellations} releases={:?}", engine.resources.colony.release_counts);
    assert_eq!(
        cancellations, 0,
        "known sleep need should not cause a fuel-job cancellation"
    );
    assert!(matches!(
        rested_job.map(|job| job.kind),
        Some(JobKind::Gather { spot: (14, 10), .. })
    ));
    let rested_id = engine.world.get::<&Agent>(rested).expect("rested adult").id;
    assert_eq!(
        engine
            .resources
            .colony
            .reservations
            .get(&ReserveKey::Wood((14, 10))),
        Some(&rested_id)
    );
    assert!(exhausted_job.is_none());
}

#[test]
fn an_adult_below_the_sleep_threshold_can_still_take_fuel_work() {
    let (mut engine, first, _) = fixture(49.0);
    tick_checked(&mut engine);
    assert!(engine
        .world
        .get::<&Job>(first)
        .is_ok_and(|job| matches!(job.kind, JobKind::Gather { .. })));
    assert_eq!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Needs),
        0
    );
}

#[test]
fn exhausted_adult_actually_sleeps_and_recovers_while_rested_worker_delivers() {
    let (mut engine, exhausted, rested) = fixture(100.0);
    tick_checked(&mut engine);
    assert_eq!(
        colony_support::state_of(&engine, exhausted).target(),
        Some(sim_core::components::TargetKind::Sleep)
    );
    assert!(engine.world.get::<&Job>(rested).is_ok());
    for _ in 0..600 {
        tick_checked(&mut engine);
    }
    let fatigue = engine
        .world
        .get::<&Sleep>(exhausted)
        .expect("sleep need")
        .fatigue;
    assert!(fatigue < 50.0, "actual bed recovery: fatigue={fatigue}");
    assert!(
        sim_systems::runtime::jobs::hearth::feedable_firewood(&engine.resources) > 0,
        "rested worker still gathers and delivers fuel"
    );
    assert!(
        engine
            .resources
            .colony
            .tallies
            .cancels_of(CancelReason::Needs)
            < 3,
        "no repeated claim/cancel loop"
    );
}

#[test]
fn without_a_bed_fatigue_does_not_withhold_available_work() {
    let (mut engine, first, _) = fixture(100.0);
    engine.resources.sleep_tiles.clear();
    engine.resources.colony.beds.clear();
    engine.resources.rules.agent.need_fallthrough = true;
    tick_checked(&mut engine);
    assert!(engine.world.get::<&Job>(first).is_ok());
}

#[test]
fn exact_day_and_night_thresholds_follow_the_sleep_decision() {
    for (minute, fatigue, due) in [
        (720, 50.0, false),
        (720, 51.0, true),
        (0, 0.0, false),
        (0, 1.0, true),
    ] {
        let (mut engine, first, second) = fixture(fatigue);
        engine.resources.rules.clock_offset_ticks = minute;
        engine.resources.rules.agent.night_rest = WorldRules::baseline(1).agent.night_rest;
        tick_checked(&mut engine);
        assert_eq!(
            engine.world.get::<&Job>(first).is_ok(),
            !due,
            "minute={minute} fatigue={fatigue}"
        );
        assert_eq!(
            engine.world.get::<&Job>(second).is_ok(),
            due,
            "minute={minute} fatigue={fatigue}"
        );
    }
}

#[test]
fn rested_workers_explicit_priorities_remain_authoritative() {
    use sim_core::colony::{WorkPriorities, WorkType};
    let (mut engine, first, second) = fixture(0.0);
    let mut priorities = WorkPriorities::defaults(false);
    priorities.levels[WorkType::Split.code() as usize] = 0;
    engine
        .world
        .insert_one(first, priorities)
        .expect("priorities");
    tick_checked(&mut engine);
    assert!(engine.world.get::<&Job>(first).is_err());
    assert!(engine
        .world
        .get::<&Job>(second)
        .is_ok_and(|job| matches!(job.kind, JobKind::Gather { .. })));
}
