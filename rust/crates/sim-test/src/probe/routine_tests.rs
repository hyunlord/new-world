use super::super::routine_accounting::Observation;
use super::*;
use sim_core::components::{ActivityKind, RoutineOutcome, TargetKind};
use sim_core::material::MaterialRegistry;

fn activity(chosen_tick: u64) -> Activity {
    Activity {
        kind: ActivityKind::Craft,
        settlement: 1,
        place: (4, 4),
        chosen_tick,
        arrived_tick: None,
        until_tick: None,
    }
}

fn observe(
    person: &mut Person,
    tick: u64,
    state: AgentState,
    activity: Option<Activity>,
    outcome: Option<RoutineOutcome>,
) -> Vec<Value> {
    person.observe(Observation {
        tick,
        stage: "adult",
        state,
        activity,
        events: outcome
            .map(|outcome| RoutineObservation {
                tick,
                kind: Some(ActivityKind::Craft),
                outcome,
            })
            .into_iter()
            .collect(),
    })
}

#[test]
fn completed_minutes_equal_consuming_intervals_when_walking_precedes_arrival() {
    let mut person = Person::default();
    let mut a = activity(4);
    let seeking = AgentState::Seeking {
        target: TargetKind::Activity,
    };
    let consuming = AgentState::Consuming {
        target: TargetKind::Activity,
    };
    let mut rows = observe(
        &mut person,
        4,
        seeking,
        Some(a),
        Some(RoutineOutcome::Started { redrawn: true }),
    );
    rows.extend(observe(&mut person, 5, seeking, Some(a), None));
    a.arrived_tick = Some(6);
    a.until_tick = Some(9);
    for tick in 6..9 {
        rows.extend(observe(&mut person, tick, consuming, Some(a), None));
    }
    rows.extend(observe(
        &mut person,
        9,
        AgentState::Idle,
        None,
        Some(RoutineOutcome::Ended { completed: true }),
    ));
    let counts = person.adult.json();
    assert_eq!(counts["activities_started"], json!([0, 1, 0, 0, 0, 0, 0]));
    assert_eq!(counts["activity_minutes"], json!([0, 3, 0, 0, 0, 0, 0]));
    assert_eq!(counts["completed_activities"], 1);
    assert_eq!(counts["completed_minutes"], 3);
    assert_eq!(counts["no_place_redraws"], 1);
    assert_eq!(person.misses, 0);
    let spans: Vec<_> = rows
        .iter()
        .filter(|r| r["event"] == "consuming_interval")
        .collect();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0]["start_tick"], 6);
    assert_eq!(spans[0]["end_tick"], 9);
    assert_eq!(rows.iter().filter(|r| r["event"] == "arrived").count(), 1);
    let ended = rows
        .iter()
        .find(|r| r["event"] == "ended")
        .expect("end row");
    assert_eq!(ended["consuming_minutes"], 3);
    assert_eq!(ended["completed"], true);
}

#[test]
fn interrupted_and_open_activities_never_enter_completed_mean() {
    let mut person = Person::default();
    let seeking = AgentState::Seeking {
        target: TargetKind::Activity,
    };
    let consuming = AgentState::Consuming {
        target: TargetKind::Activity,
    };
    let mut a = activity(0);
    observe(
        &mut person,
        0,
        seeking,
        Some(a),
        Some(RoutineOutcome::Started { redrawn: false }),
    );
    a.arrived_tick = Some(1);
    a.until_tick = Some(91);
    observe(&mut person, 1, consuming, Some(a), None);
    observe(
        &mut person,
        2,
        AgentState::Idle,
        None,
        Some(RoutineOutcome::Ended { completed: false }),
    );
    let mut b = activity(3);
    observe(
        &mut person,
        3,
        seeking,
        Some(b),
        Some(RoutineOutcome::Started { redrawn: false }),
    );
    b.arrived_tick = Some(4);
    b.until_tick = Some(94);
    observe(&mut person, 4, consuming, Some(b), None);
    assert_eq!(person.adult.json()["completed_activities"], 0);
    assert_eq!(person.adult.json()["completed_minutes"], 0);
    assert_eq!(person.adult.json()["activity_minutes"][1], 2);
    assert_eq!(person.final_interval().expect("open span")["minutes"], 1);
}

#[test]
fn life_stage_change_splits_intervals_and_excludes_elder_minutes() {
    let mut person = Person::default();
    let mut a = activity(0);
    observe(
        &mut person,
        0,
        AgentState::Seeking {
            target: TargetKind::Activity,
        },
        Some(a),
        Some(RoutineOutcome::Started { redrawn: false }),
    );
    a.arrived_tick = Some(1);
    a.until_tick = Some(4);
    observe(
        &mut person,
        1,
        AgentState::Consuming {
            target: TargetKind::Activity,
        },
        Some(a),
        None,
    );
    let rows = person.observe(Observation {
        tick: 2,
        stage: "elder",
        state: AgentState::Consuming {
            target: TargetKind::Activity,
        },
        activity: Some(a),
        events: Vec::new(),
    });
    person.observe(Observation {
        tick: 3,
        stage: "elder",
        state: AgentState::Consuming {
            target: TargetKind::Activity,
        },
        activity: Some(a),
        events: Vec::new(),
    });
    person.observe(Observation {
        tick: 4,
        stage: "elder",
        state: AgentState::Idle,
        activity: None,
        events: vec![RoutineObservation {
            tick: 4,
            kind: Some(ActivityKind::Craft),
            outcome: RoutineOutcome::Ended { completed: true },
        }],
    });
    assert_eq!(rows[0]["life_stage"], "adult");
    assert_eq!(rows[0]["minutes"], 1);
    assert_eq!(person.all.json()["activity_minutes"][1], 3);
    assert_eq!(person.adult.json()["activity_minutes"][1], 1);
    assert_eq!(person.adult.json()["completed_minutes"], 1);
}

#[test]
fn no_place_failures_are_counted_once_on_the_event_tick() {
    let mut person = Person::default();
    let event = RoutineObservation {
        tick: 2,
        kind: None,
        outcome: RoutineOutcome::NoPlace { redrawn: true },
    };
    for tick in 2..5 {
        person.observe(Observation {
            tick,
            stage: "adult",
            state: AgentState::Idle,
            activity: None,
            events: vec![event],
        });
    }
    assert_eq!(person.all.json()["no_place_redraws"], 1);
    assert_eq!(person.all.json()["no_place_failures"], 1);
    assert_eq!(person.all.json()["completed_activities"], 0);
    assert_eq!(person.misses, 0);
}

#[test]
fn hourly_rows_use_display_clock_and_exact_stage_state_counts_and_stores() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules.clock_offset_ticks = 720;
    let adult = engine.spawn_agent(1, 1);
    let child = engine.spawn_agent(2, 1);
    let elder = engine.spawn_agent(3, 1);
    engine
        .world
        .insert(adult, (AgentState::Idle,))
        .expect("adult state");
    engine
        .world
        .insert(
            child,
            (
                Child { born_tick: 0 },
                AgentState::Seeking {
                    target: TargetKind::Water,
                },
            ),
        )
        .expect("child");
    engine
        .world
        .insert(
            elder,
            (
                Elder,
                AgentState::Consuming {
                    target: TargetKind::Sleep,
                },
            ),
        )
        .expect("elder");
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.stockpile.insert(ResourceKind::Food, 17);
    settlement.stockpile.insert(ResourceKind::Wood, 23);
    engine.resources.settlements.insert(1, settlement);
    let mut trace = RoutineTracer::start(&engine);
    engine.tick();
    trace.after_tick(&engine);
    let row = &trace.hours[0];
    assert_eq!(row["minute"], 720);
    assert_eq!(row["day"], 1);
    assert_eq!(row["states"]["adult"], json!([1, 0, 0, 0, 0]));
    assert_eq!(row["states"]["child"], json!([0, 1, 0, 0, 0]));
    assert_eq!(row["states"]["elder"], json!([0, 0, 0, 1, 0]));
    assert_eq!(row["stockpile_food"], 17);
    assert_eq!(row["stockpile_wood"], 23);
    assert_eq!(trace.meta(&engine)["observation_misses"], 0);
}

#[test]
fn new_state_tag_preserves_existing_codes() {
    use super::super::fingerprint::state_tag;
    assert_eq!(state_tag(Some(&AgentState::Idle)), 0);
    assert_eq!(
        state_tag(Some(&AgentState::Seeking {
            target: TargetKind::Food
        })),
        1
    );
    assert_eq!(
        state_tag(Some(&AgentState::Consuming {
            target: TargetKind::Agent(2)
        })),
        2
    );
    assert_eq!(
        state_tag(Some(&AgentState::Consuming {
            target: TargetKind::Sleep
        })),
        3
    );
    assert_eq!(
        state_tag(Some(&AgentState::Seeking {
            target: TargetKind::Activity
        })),
        4
    );
    assert_eq!(
        state_tag(Some(&AgentState::Consuming {
            target: TargetKind::Activity
        })),
        4
    );
}

#[test]
fn hourly_cadence_covers_exactly_one_day_without_initial_double_count() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules.clock_offset_ticks = 480;
    let mut trace = RoutineTracer::start(&engine);
    assert!(trace.hours.is_empty());
    for _ in 0..1440 {
        engine.tick();
        trace.after_tick(&engine);
    }
    assert_eq!(trace.hours.len(), 24);
    assert_eq!(trace.hours[0]["tick"], 0);
    assert_eq!(trace.hours[0]["minute"], 480);
    assert_eq!(
        trace
            .hours
            .iter()
            .filter(|row| row["minute"] == 720)
            .count(),
        1
    );
    assert_eq!(
        trace
            .hours
            .iter()
            .filter(|row| (480..=1020).contains(&row["minute"].as_u64().expect("minute")))
            .count(),
        10
    );
}

#[test]
fn shelter_completions_exclude_initial_shelters() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine
        .resources
        .building_kinds
        .insert((1, 1), sim_core::components::BuildingKind::Shelter);
    let trace = RoutineTracer::start(&engine);
    assert_eq!(trace.meta(&engine)["shelters_completed"], 0);
    engine
        .resources
        .building_kinds
        .insert((2, 1), sim_core::components::BuildingKind::Shelter);
    assert_eq!(trace.meta(&engine)["shelters_completed"], 1);
}

#[test]
fn twin_comparison_detects_routine_timers_without_changing_fingerprint() {
    let mut first = SimEngine::new(8, 8, MaterialRegistry::new());
    let mut second = SimEngine::new(8, 8, MaterialRegistry::new());
    let a = first.spawn_agent(1, 1);
    let b = second.spawn_agent(1, 1);
    first
        .world
        .insert(a, (activity(0), ActivityRest { until_tick: 4 }))
        .expect("first");
    second
        .world
        .insert(b, (activity(0), ActivityRest { until_tick: 4 }))
        .expect("second");
    assert!(twins_match(&first, &second));
    second
        .world
        .get::<&mut ActivityRest>(b)
        .expect("rest")
        .until_tick = 5;
    assert!(!twins_match(&first, &second));
    assert_eq!(
        super::super::fingerprint::fingerprint(&first),
        super::super::fingerprint::fingerprint(&second)
    );
}
