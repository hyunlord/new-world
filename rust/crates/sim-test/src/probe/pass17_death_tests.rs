use super::*;
use hecs::{Entity, World};
use sim_core::components::{ActivityKind, Life, RoutineOutcome, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimResources};
use sim_systems::runtime::agent::AgingSystem;

#[derive(Clone, Copy)]
enum Case {
    Start,
    StartAndEnd,
    NoPlace,
    Complete,
    Consuming,
}

/// Emits the production observation shapes at decision priority; real aging kills later.
struct DecisionEvents {
    entity: Entity,
    case: Case,
}

fn event(tick: u64, outcome: RoutineOutcome) -> RoutineObservation {
    RoutineObservation {
        tick,
        kind: Some(ActivityKind::Craft),
        outcome,
    }
}

impl RuntimeSystem for DecisionEvents {
    fn name(&self) -> &str {
        "RoutineDeathFixture"
    }
    fn priority(&self) -> u32 {
        125
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let tick = resources.current_tick;
        if !world.contains(self.entity) {
            return;
        }
        if tick == 0 {
            match self.case {
                Case::NoPlace => {
                    world
                        .insert_one(
                            self.entity,
                            event(tick, RoutineOutcome::NoPlace { redrawn: true }),
                        )
                        .expect("failure event");
                }
                Case::StartAndEnd => {
                    let ended = event(tick, RoutineOutcome::Ended { completed: false });
                    world
                        .insert(
                            self.entity,
                            (
                                ended,
                                RoutineObservationBatch {
                                    tick,
                                    events: vec![
                                        event(tick, RoutineOutcome::Started { redrawn: true }),
                                        ended,
                                    ],
                                },
                            ),
                        )
                        .expect("transient events");
                }
                Case::Start | Case::Complete | Case::Consuming => {
                    world
                        .insert(
                            self.entity,
                            (
                                Activity {
                                    kind: ActivityKind::Craft,
                                    settlement: 1,
                                    place: (1, 1),
                                    chosen_tick: 0,
                                    arrived_tick: None,
                                    until_tick: None,
                                },
                                AgentState::Seeking {
                                    target: TargetKind::Activity,
                                },
                                event(tick, RoutineOutcome::Started { redrawn: true }),
                            ),
                        )
                        .expect("start");
                }
            }
        } else if tick == 1 && matches!(self.case, Case::Complete | Case::Consuming) {
            let until = if matches!(self.case, Case::Complete) {
                60
            } else {
                120
            };
            world
                .insert(
                    self.entity,
                    (
                        Activity {
                            kind: ActivityKind::Craft,
                            settlement: 1,
                            place: (1, 1),
                            chosen_tick: 0,
                            arrived_tick: Some(1),
                            until_tick: Some(until),
                        },
                        AgentState::Consuming {
                            target: TargetKind::Activity,
                        },
                    ),
                )
                .expect("arrival");
        } else if tick == 60 && matches!(self.case, Case::Complete) {
            world
                .remove_one::<Activity>(self.entity)
                .expect("end activity");
            world
                .insert(
                    self.entity,
                    (
                        AgentState::Idle,
                        event(tick, RoutineOutcome::Ended { completed: true }),
                    ),
                )
                .expect("completion");
        }
    }
}

struct Harness {
    engine: SimEngine,
    trace: RoutineTracer,
    id: AgentId,
    observed: super::super::routine_observer::SharedRoutine,
}

impl Harness {
    fn new(case: Case, lifespan: u64) -> Self {
        let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
        engine.resources.rules = WorldRules::baseline(7);
        let entity = engine.spawn_agent(1, 1);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Life {
                        born_tick: 0,
                        lifespan_ticks: lifespan,
                    },
                ),
            )
            .expect("life");
        engine.register_system(Box::new(DecisionEvents { entity, case }));
        engine.register_system(Box::new(AgingSystem));
        let trace = RoutineTracer::start(&engine);
        let observed =
            super::super::routine_observer::install(&mut engine).expect("routine enabled");
        Self {
            engine,
            trace,
            id,
            observed,
        }
    }

    fn run(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.engine.tick();
            let snapshot = super::super::routine_observer::take(
                &self.observed,
                self.engine.resources.current_tick,
            )
            .expect("snapshot");
            self.trace.recover_disappeared(&self.engine, snapshot);
            self.trace.after_tick(&self.engine);
        }
    }

    fn counts(&self) -> Value {
        self.trace.person(self.id, true)
    }
}

#[test]
fn start_and_redraw_survive_same_tick_old_age_despawn() {
    let mut h = Harness::new(Case::Start, 0);
    h.run(1);
    assert_eq!(h.engine.resources.tallies.deaths_old_age, 1);
    assert_eq!(h.counts()["activities_started"][1], 1);
    assert_eq!(h.counts()["no_place_redraws"], 1);
    assert_eq!(h.counts()["activity_minutes"][1], 0);
    assert_eq!(h.counts()["completed_activities"], 0);
    assert_eq!(h.trace.hours[0]["states"]["adult"], json!([0, 0, 0, 0, 0]));
    assert_eq!(h.trace.meta(&h.engine)["observation_misses"], 0);
    let started = h
        .trace
        .events
        .iter()
        .find(|row| row["event"] == "started")
        .expect("start");
    assert_eq!(started["observation_boundary"], "pre_lethal");
    assert_eq!(
        started["current_state"],
        json!(AgentState::Seeking {
            target: TargetKind::Activity
        })
    );
}

#[test]
fn start_end_batch_survives_same_tick_old_age_despawn() {
    let mut h = Harness::new(Case::StartAndEnd, 0);
    h.run(1);
    assert_eq!(h.counts()["activities_started"][1], 1);
    assert_eq!(h.counts()["no_place_redraws"], 1);
    let ended = h
        .trace
        .events
        .iter()
        .find(|row| row["event"] == "ended")
        .expect("end");
    assert_eq!(ended["completed"], false);
    assert_eq!(ended["consuming_minutes"], 0);
    assert!(ended["arrived_tick"].is_null());
    assert_eq!(h.trace.meta(&h.engine)["observation_misses"], 0);
    assert_eq!(
        h.trace
            .events
            .iter()
            .filter(|row| row["event"] == "end_observation")
            .count(),
        1
    );
}

#[test]
fn no_place_and_redraw_survive_same_tick_old_age_despawn() {
    let mut h = Harness::new(Case::NoPlace, 0);
    h.run(1);
    assert_eq!(h.counts()["no_place_redraws"], 1);
    assert_eq!(h.counts()["no_place_failures"], 1);
    assert_eq!(
        h.counts()["activities_started"],
        json!([0, 0, 0, 0, 0, 0, 0])
    );
    assert_eq!(h.trace.meta(&h.engine)["observation_misses"], 0);
}

#[test]
fn completed_episode_survives_later_same_tick_despawn_with_exact_prior_minutes() {
    let mut h = Harness::new(Case::Complete, 60);
    h.run(61);
    assert_eq!(h.engine.resources.tallies.deaths_old_age, 1);
    assert_eq!(h.counts()["completed_activities"], 1);
    assert_eq!(h.counts()["completed_minutes"], 59);
    assert_eq!(h.counts()["activity_minutes"][1], 59);
    let ended = h
        .trace
        .events
        .iter()
        .find(|row| row["event"] == "ended")
        .expect("end");
    assert_eq!(ended["completed"], true);
    assert_eq!(ended["tick"], 60);
    assert_eq!(ended["consuming_minutes"], 59);
    assert_eq!(h.trace.meta(&h.engine)["observation_misses"], 0);
}

#[test]
fn death_tick_does_not_add_a_consuming_minute_or_completion() {
    let mut h = Harness::new(Case::Consuming, 60);
    h.run(61);
    assert_eq!(h.engine.resources.tallies.deaths_old_age, 1);
    assert_eq!(h.counts()["activity_minutes"][1], 59);
    assert_eq!(h.counts()["completed_activities"], 0);
    let interval = h
        .trace
        .events
        .iter()
        .find(|row| row["event"] == "consuming_interval")
        .expect("interval");
    assert_eq!(interval["start_tick"], 1);
    assert_eq!(interval["end_tick"], 60);
    assert_eq!(interval["minutes"], 59);
    let ended = h
        .trace
        .events
        .iter()
        .find(|row| row["event"] == "ended")
        .expect("end");
    assert_eq!(
        ended["current_state"],
        json!(AgentState::Consuming {
            target: TargetKind::Activity
        })
    );
    assert_eq!(ended["observation_boundary"], "pre_lethal");
}

#[test]
fn surviving_holder_is_not_counted_twice() {
    let mut h = Harness::new(Case::Complete, 120);
    h.run(61);
    assert_eq!(h.engine.resources.tallies.deaths_old_age, 0);
    assert_eq!(h.counts()["activities_started"][1], 1);
    assert_eq!(h.counts()["no_place_redraws"], 1);
    assert_eq!(h.counts()["completed_activities"], 1);
    assert_eq!(h.counts()["completed_minutes"], 59);
    assert_eq!(h.counts()["activity_minutes"][1], 59);
    assert_eq!(h.trace.meta(&h.engine)["observation_misses"], 0);
    assert!(h
        .trace
        .events
        .iter()
        .all(|row| row["observation_boundary"].is_null()));
}

#[test]
fn observer_is_absent_from_old_worlds_and_checks_tick_and_single_drain() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    assert!(super::super::routine_observer::install(&mut engine).is_none());
    assert_eq!(engine.system_count(), 0);
    engine.resources.rules.agent.routine = WorldRules::baseline(7).agent.routine;
    let observed = super::super::routine_observer::install(&mut engine).expect("enabled");
    engine.tick();
    assert!(super::super::routine_observer::take(&observed, 1).is_err());
    assert!(super::super::routine_observer::take(&observed, 0).is_err());
    engine.tick();
    assert!(super::super::routine_observer::take(&observed, 1).is_ok());
    assert!(super::super::routine_observer::take(&observed, 1).is_err());
}
