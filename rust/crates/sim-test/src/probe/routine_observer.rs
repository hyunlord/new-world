use super::pass17_activity::Observation;
use hecs::World;
use sim_core::components::{
    Activity, Agent, AgentId, AgentState, Child, Elder, RoutineObservation, RoutineObservationBatch,
};
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub(super) struct CapturedRoutine {
    pub(super) tick: u64,
    pub(super) people: BTreeMap<AgentId, Observation>,
    pub(super) elapsed: Duration,
}

pub(super) type SharedRoutine = Arc<Mutex<Option<CapturedRoutine>>>;

struct Observer {
    shared: SharedRoutine,
}

impl RuntimeSystem for Observer {
    fn name(&self) -> &str {
        "ProbeRoutineObserver"
    }
    fn priority(&self) -> u32 {
        136
    }
    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let started = Instant::now();
        let tick = resources.current_tick;
        let people = world
            .query::<(
                &Agent,
                Option<&AgentState>,
                Option<&Child>,
                Option<&Elder>,
                Option<&Activity>,
                Option<&RoutineObservation>,
                Option<&RoutineObservationBatch>,
            )>()
            .iter()
            .filter_map(
                |(_, (agent, state, child, elder, activity, event, batch))| {
                    let events: Vec<_> = batch.filter(|batch| batch.tick == tick).map_or_else(
                        || {
                            event
                                .filter(|event| event.tick == tick)
                                .copied()
                                .into_iter()
                                .collect()
                        },
                        |batch| batch.events.clone(),
                    );
                    if activity.is_none() && events.is_empty() {
                        return None;
                    }
                    Some((
                        agent.id,
                        Observation {
                            tick,
                            stage: if child.is_some() {
                                "child"
                            } else if elder.is_some() {
                                "elder"
                            } else {
                                "adult"
                            },
                            state: state.copied().unwrap_or_default(),
                            activity: activity.copied(),
                            events,
                        },
                    ))
                },
            )
            .collect();
        if let Ok(mut shared) = self.shared.lock() {
            *shared = Some(CapturedRoutine {
                tick,
                people,
                elapsed: started.elapsed(),
            });
        }
    }
}

pub(super) fn install(engine: &mut SimEngine) -> Option<SharedRoutine> {
    engine.resources.rules.agent.routine?;
    let shared = Arc::new(Mutex::new(None));
    engine.register_system(Box::new(Observer {
        shared: Arc::clone(&shared),
    }));
    Some(shared)
}

pub(super) fn take(shared: &SharedRoutine, tick: u64) -> Result<CapturedRoutine, String> {
    let snapshot = shared
        .lock()
        .map_err(|_| "routine observer lock poisoned")?
        .take()
        .ok_or("routine observer did not run")?;
    if snapshot.tick != tick {
        return Err(format!(
            "routine observer tick mismatch: captured {}, expected {tick}",
            snapshot.tick
        ));
    }
    Ok(snapshot)
}
