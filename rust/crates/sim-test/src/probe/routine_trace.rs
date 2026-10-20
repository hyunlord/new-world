//! Pass 17 trace-only routine, life-stage and noon-store observations.
use super::routine_accounting::{Counts, Observation, Person, KINDS};
use serde_json::{json, Value};
use sim_core::components::{
    Activity, ActivityRest, Agent, AgentId, AgentState, Child, Elder, ResourceKind,
    RoutineObservation, RoutineObservationBatch,
};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn twins_match(first: &SimEngine, second: &SimEngine) -> bool {
    let snapshot = |engine: &SimEngine| {
        let mut rows: Vec<_> = engine
            .world
            .query::<(
                &Agent,
                Option<&Activity>,
                Option<&ActivityRest>,
                Option<&RoutineObservation>,
                Option<&RoutineObservationBatch>,
            )>()
            .iter()
            .map(|(_, (agent, activity, rest, event, batch))| {
                (
                    agent.id,
                    activity.copied(),
                    rest.copied(),
                    event.copied(),
                    batch.cloned(),
                )
            })
            .collect();
        rows.sort_by_key(|row| row.0);
        rows
    };
    snapshot(first) == snapshot(second)
}

#[derive(Debug, Default)]
pub(super) struct RoutineTracer {
    people: BTreeMap<AgentId, Person>,
    events: Vec<Value>,
    hours: Vec<Value>,
    initial_shelters: usize,
}

impl RoutineTracer {
    pub(super) fn start(engine: &SimEngine) -> Self {
        Self {
            initial_shelters: super::metrics::shelter_count(engine),
            ..Self::default()
        }
    }

    pub(super) fn after_tick(&mut self, engine: &SimEngine) {
        let tick = engine.resources.current_tick;
        let mut seen = BTreeSet::new();
        let mut states = [[0u64; 5]; 3];
        let mut query = engine.world.query::<(
            &Agent,
            Option<&AgentState>,
            Option<&Child>,
            Option<&Elder>,
            Option<&Activity>,
            Option<&RoutineObservation>,
            Option<&RoutineObservationBatch>,
        )>();
        let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
        rows.sort_by_key(|row| row.0.id);
        for (agent, state, child, elder, activity, event, batch) in rows {
            seen.insert(agent.id);
            let (stage, stage_index) = if child.is_some() {
                ("child", 0)
            } else if elder.is_some() {
                ("elder", 2)
            } else {
                ("adult", 1)
            };
            let state = state.copied().unwrap_or_default();
            states[stage_index][usize::from(super::fingerprint::state_tag(Some(&state)))] += 1;
            let events = self
                .people
                .entry(agent.id)
                .or_default()
                .observe(Observation {
                    tick,
                    stage,
                    state,
                    activity: activity.copied(),
                    events: batch.filter(|batch| batch.tick == tick).map_or_else(
                        || {
                            event
                                .filter(|event| event.tick == tick)
                                .copied()
                                .into_iter()
                                .collect()
                        },
                        |batch| batch.events.clone(),
                    ),
                });
            for mut row in events {
                row["id"] = json!(agent.id);
                self.events.push(row);
            }
        }
        for (id, person) in &mut self.people {
            if !seen.contains(id) {
                for mut row in person.disappeared(tick) {
                    row["id"] = json!(id);
                    self.events.push(row);
                }
            }
        }
        let r = &engine.resources;
        if r.ticks_per_day == 1440 {
            let absolute = tick.saturating_add(r.rules.clock_offset_ticks);
            let minute = absolute % r.ticks_per_day;
            if minute.is_multiple_of(60) {
                let stock = |kind| {
                    r.settlements
                        .values()
                        .map(|s| u64::from(s.stockpile.get(&kind).copied().unwrap_or(0)))
                        .sum::<u64>()
                };
                self.hours.push(json!({"tick":tick,"day":absolute / r.ticks_per_day + 1,"minute":minute,
                    "states":{"child":states[0],"adult":states[1],"elder":states[2]},
                    "stockpile_food":stock(ResourceKind::Food),"stockpile_wood":stock(ResourceKind::Wood)}));
            }
        }
    }

    pub(super) fn recover_disappeared(
        &mut self,
        engine: &SimEngine,
        snapshot: super::routine_observer::CapturedRoutine,
    ) {
        let alive: BTreeSet<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, agent)| agent.id)
            .collect();
        for (id, observation) in snapshot.people {
            if alive.contains(&id) {
                continue;
            }
            for mut row in self
                .people
                .entry(id)
                .or_default()
                .observe_disappeared(observation)
            {
                row["id"] = json!(id);
                self.events.push(row);
            }
        }
    }

    pub(super) fn person(&self, id: AgentId, adult_only: bool) -> Value {
        self.people.get(&id).map_or_else(
            || Counts::default().json(),
            |p| {
                if adult_only {
                    p.adult.json()
                } else {
                    p.all.json()
                }
            },
        )
    }

    pub(super) fn meta(&self, engine: &SimEngine) -> Value {
        let sum = |key: &str| {
            self.people
                .values()
                .map(|p| p.all.json()[key].as_u64().unwrap_or(0))
                .sum::<u64>()
        };
        json!({"schema_version":17,"activity_kinds":KINDS,
            "shelters_completed":super::metrics::shelter_count(engine).saturating_sub(self.initial_shelters),
            "no_place_redraws":sum("no_place_redraws"),"no_place_failures":sum("no_place_failures"),
            "observation_misses":self.people.values().map(|p| p.misses).sum::<u64>(),
            "clock":"after system tick; day is one-based display day; minute includes clock_offset_ticks",
            "interval":"[start_tick,end_tick), each tick observed in Consuming{Activity}; walking excluded",
            "completed":"RoutineOutcome::Ended{completed:true} after arrival; interrupted/dead/open episodes excluded"})
    }

    pub(super) fn output(&self) -> [(&'static str, Vec<Value>); 2] {
        let mut events = self.events.clone();
        for (id, person) in &self.people {
            if let Some(mut row) = person.final_interval() {
                row["id"] = json!(id);
                events.push(row);
            }
        }
        [("routine", events), ("hours", self.hours.clone())]
    }
}

#[cfg(test)]
#[path = "routine_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "routine_batch_tests.rs"]
mod batch_tests;

#[cfg(test)]
#[path = "routine_death_tests.rs"]
mod death_tests;
