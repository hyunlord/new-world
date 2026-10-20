//! Unit tests of routine observation batches (compiled as the `batch_tests` module of `routine_trace.rs`).
use super::*;
use sim_core::components::{ActivityKind, RoutineOutcome, TargetKind};
use sim_core::material::MaterialRegistry;

fn event(tick: u64, outcome: RoutineOutcome) -> RoutineObservation {
    RoutineObservation {
        tick,
        kind: Some(ActivityKind::Craft),
        outcome,
    }
}

#[test]
fn transient_start_and_end_preserve_redraw_without_fabricating_consumption() {
    let mut person = Person::default();
    let rows = person.observe(Observation {
        tick: 7,
        stage: "adult",
        state: AgentState::Consuming {
            target: TargetKind::Agent(2),
        },
        activity: None,
        events: vec![
            event(7, RoutineOutcome::Started { redrawn: true }),
            event(7, RoutineOutcome::Ended { completed: false }),
        ],
    });
    assert_eq!(person.adult.json()["activities_started"][1], 1);
    assert_eq!(person.adult.json()["no_place_redraws"], 1);
    assert_eq!(
        person.adult.json()["activity_minutes"],
        json!([0, 0, 0, 0, 0, 0, 0])
    );
    assert_eq!(person.adult.json()["completed_activities"], 0);
    assert_eq!(person.misses, 0);
    let started = rows
        .iter()
        .find(|row| row["event"] == "started")
        .expect("start");
    assert!(started["activity"].is_null());
    let ended = rows
        .iter()
        .find(|row| row["event"] == "ended")
        .expect("end");
    assert_eq!(ended["chosen_tick"], 7);
    assert_eq!(ended["consuming_minutes"], 0);
    assert!(ended["arrived_tick"].is_null());
    assert!(ended["until_tick"].is_null());
    assert_eq!(ended["completed"], false);
    assert!(!rows.iter().any(|row| row["event"] == "consuming_interval"));
}

#[test]
fn batch_wins_over_latest_once_then_stale_batch_yields_to_current_event() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    let entity = engine.spawn_agent(1, 1);
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    let mut trace = RoutineTracer::start(&engine);
    engine.tick();
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                event(0, RoutineOutcome::Ended { completed: false }),
                RoutineObservationBatch {
                    tick: 0,
                    events: vec![
                        event(0, RoutineOutcome::Started { redrawn: true }),
                        event(0, RoutineOutcome::Ended { completed: false }),
                    ],
                },
            ),
        )
        .expect("events");
    trace.after_tick(&engine);
    engine.tick();
    engine
        .world
        .insert_one(entity, event(1, RoutineOutcome::NoPlace { redrawn: true }))
        .expect("latest");
    trace.after_tick(&engine);
    assert_eq!(trace.person(id, true)["activities_started"][1], 1);
    assert_eq!(trace.person(id, true)["no_place_redraws"], 2);
    assert_eq!(trace.person(id, true)["no_place_failures"], 1);
    assert_eq!(trace.meta(&engine)["observation_misses"], 0);
}

#[test]
fn same_tick_completion_and_new_start_keep_separate_episode_totals() {
    let mut person = Person::default();
    let mut activity = Activity {
        kind: ActivityKind::Craft,
        settlement: 1,
        place: (1, 1),
        chosen_tick: 0,
        arrived_tick: None,
        until_tick: None,
    };
    person.observe(Observation {
        tick: 0,
        stage: "adult",
        state: AgentState::Seeking {
            target: TargetKind::Activity,
        },
        activity: Some(activity),
        events: vec![event(0, RoutineOutcome::Started { redrawn: false })],
    });
    activity.arrived_tick = Some(1);
    activity.until_tick = Some(3);
    for tick in 1..3 {
        person.observe(Observation {
            tick,
            stage: "adult",
            state: AgentState::Consuming {
                target: TargetKind::Activity,
            },
            activity: Some(activity),
            events: Vec::new(),
        });
    }
    let next = Activity {
        kind: ActivityKind::Explore,
        chosen_tick: 3,
        arrived_tick: None,
        until_tick: None,
        ..activity
    };
    person.observe(Observation {
        tick: 3,
        stage: "adult",
        state: AgentState::Seeking {
            target: TargetKind::Activity,
        },
        activity: Some(next),
        events: vec![
            event(3, RoutineOutcome::Ended { completed: true }),
            RoutineObservation {
                tick: 3,
                kind: Some(ActivityKind::Explore),
                outcome: RoutineOutcome::Started { redrawn: false },
            },
        ],
    });
    assert_eq!(person.adult.json()["completed_activities"], 1);
    assert_eq!(person.adult.json()["completed_minutes"], 2);
    assert_eq!(
        person.adult.json()["activity_minutes"],
        json!([0, 2, 0, 0, 0, 0, 0])
    );
    assert_eq!(
        person.adult.json()["activities_started"],
        json!([0, 1, 0, 0, 1, 0, 0])
    );
    assert_eq!(person.misses, 0);
}

#[test]
fn twin_comparison_detects_batch_difference_with_same_latest_event() {
    let mut first = SimEngine::new(8, 8, MaterialRegistry::new());
    let mut second = SimEngine::new(8, 8, MaterialRegistry::new());
    let a = first.spawn_agent(1, 1);
    let b = second.spawn_agent(1, 1);
    let latest = event(0, RoutineOutcome::Ended { completed: false });
    first.world.insert_one(a, latest).expect("first latest");
    second.world.insert_one(b, latest).expect("second latest");
    assert!(twins_match(&first, &second));
    first
        .world
        .insert_one(
            a,
            RoutineObservationBatch {
                tick: 0,
                events: vec![event(0, RoutineOutcome::Started { redrawn: true }), latest],
            },
        )
        .expect("batch");
    assert!(!twins_match(&first, &second));
}
