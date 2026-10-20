//! Family probe tracer: follows people, family events, daily births and fissions and assembles
//! the family trace files. Read-only.
use super::{family_events, family_samples};
use serde_json::{json, Value};
use sim_core::components::{
    Agent, AgentId, FamilyEvent, FamilyObservations, Kin, Lineage, Sex, Temperament,
};
use sim_engine::SimEngine;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct Person {
    pub(super) id: AgentId,
    pub(super) female: Option<bool>,
    pub(super) mother: Option<AgentId>,
    pub(super) father: Option<AgentId>,
    pub(super) temperament: Option<Temperament>,
    pub(super) lineage_key: Option<u64>,
    pub(super) born_tick: Option<u64>,
    pub(super) alive: bool,
    pub(super) partner: Option<AgentId>,
    pub(super) pairing_tick: Option<u64>,
    pub(super) household_key: AgentId,
    pub(super) shelter_nights: Vec<Value>,
}

impl Person {
    pub(super) fn new(id: AgentId) -> Self {
        Self {
            id,
            female: None,
            mother: None,
            father: None,
            temperament: None,
            lineage_key: None,
            born_tick: None,
            alive: true,
            partner: None,
            pairing_tick: None,
            household_key: id,
            shelter_nights: Vec::new(),
        }
    }

    fn json(&self) -> Value {
        json!({"id":self.id,"female":self.female,"mother":self.mother,"father":self.father,
            "temperament":self.temperament,"lineage_key":self.lineage_key,"born_tick":self.born_tick,
            "alive_end":self.alive,"partner":self.partner,"pairing_tick":self.pairing_tick,
            "household_key":self.household_key,"shelter_nights":self.shelter_nights})
    }
}

#[derive(Debug, Default)]
pub(super) struct FamilyTracer {
    pub(super) people: BTreeMap<AgentId, Person>,
    pub(super) events: Vec<Value>,
    pub(super) departures: BTreeMap<u32, Value>,
    pub(super) births_by_day: BTreeMap<u64, (u64, u64)>,
    hours: Vec<Value>,
    day60: Option<Value>,
    last_tick: Option<u64>,
    pub(super) misses: u64,
    initial_births: u64,
    initial_fissions: usize,
}

pub(super) struct FamilyOutput {
    pub(super) files: Vec<(&'static str, Vec<Value>)>,
    pub(super) meta: Value,
}

impl FamilyTracer {
    pub(super) fn start(engine: &SimEngine) -> Option<Self> {
        engine.resources.rules.society.family?;
        let mut tracer = Self {
            initial_births: engine.resources.tallies.births,
            initial_fissions: engine.resources.fission_log.len(),
            ..Self::default()
        };
        tracer.record_people(engine, true);
        Some(tracer)
    }

    fn record_people(&mut self, engine: &SimEngine, initial: bool) {
        for person in self.people.values_mut() {
            person.alive = false;
        }
        for (_, (agent, sex, kin, lineage, temperament)) in engine
            .world
            .query::<(
                &Agent,
                Option<&Sex>,
                Option<&Kin>,
                Option<&Lineage>,
                Option<&Temperament>,
            )>()
            .iter()
        {
            if !initial && !self.people.contains_key(&agent.id) {
                self.misses += 1;
            }
            let person = self
                .people
                .entry(agent.id)
                .or_insert_with(|| Person::new(agent.id));
            person.alive = true;
            person.female = sex.map(|sex| sex.female);
            person.lineage_key = lineage.map(|lineage| lineage.key);
            person.temperament = temperament.copied();
            if sex.is_none() || kin.is_none() || lineage.is_none() || temperament.is_none() {
                self.misses += 1;
            }
            if let Some(kin) = kin {
                person.mother = kin.mother;
                person.father = kin.father;
                if person.partner != kin.partner {
                    person.partner = kin.partner;
                    person.pairing_tick = None;
                    if !initial && kin.partner.is_some() {
                        self.misses += 1;
                    }
                }
            }
        }
    }

    pub(super) fn after_tick(&mut self, engine: &SimEngine) -> Vec<FamilyEvent> {
        let tick = engine.resources.current_tick;
        if self.last_tick == Some(tick) {
            return Vec::new();
        }
        if self.last_tick.is_some_and(|last| last + 1 != tick) {
            self.misses += 1;
        }
        self.last_tick = Some(tick);
        let mut batches: Vec<_> = engine
            .world
            .query::<(Option<&Agent>, &FamilyObservations)>()
            .iter()
            .map(|(entity, (agent, batch))| {
                (entity.to_bits().get(), agent.is_some(), batch.clone())
            })
            .collect();
        batches.sort_by_key(|row| row.0);
        if batches.len() > 1 {
            self.misses += 1;
        }
        let mut births = Vec::new();
        for (_, on_agent, batch) in batches {
            if on_agent {
                self.misses += 1;
            }
            if batch.tick != tick {
                continue;
            }
            for event in batch.events {
                if matches!(event, FamilyEvent::Born { .. }) {
                    births.push(event.clone());
                }
                family_events::observe(self, engine, event);
            }
        }
        self.record_people(engine, false);
        let r = &engine.resources;
        if r.ticks_per_day == 1440 {
            let minute = tick.saturating_add(r.rules.clock_offset_ticks) % 1440;
            if minute.is_multiple_of(60) {
                let sample = family_samples::capture(engine, &self.people);
                self.remember_sample(&sample, minute == 1200);
                self.hours.push(sample);
            }
            if engine.current_tick() == 60 * 1440 {
                let sample = family_samples::capture(engine, &self.people);
                self.remember_sample(&sample, false);
                self.day60 = Some(sample);
            }
        }
        births
    }

    fn remember_sample(&mut self, sample: &Value, night: bool) {
        for row in sample["people"].as_array().into_iter().flatten() {
            let Some(person) = row["id"].as_u64().and_then(|id| self.people.get_mut(&id)) else {
                continue;
            };
            person.household_key = row["household_key"].as_u64().unwrap_or(person.id);
            if night {
                person.shelter_nights.push(
                    json!({"tick":sample["tick"],"simulation_day":sample["simulation_day"],
                    "display_day":sample["display_day"],"bed":row["bed"],"partner":row["partner"],
                    "household_key":row["household_key"]}),
                );
            }
        }
    }

    pub(super) fn output(&self, engine: &SimEngine) -> FamilyOutput {
        let end = engine.current_tick();
        let tpd = engine.resources.ticks_per_day.max(1);
        let births_total: u64 = self
            .births_by_day
            .values()
            .map(|(couple, fallback)| couple + fallback)
            .sum();
        let source_births = engine
            .resources
            .tallies
            .births
            .saturating_sub(self.initial_births);
        let source_fissions = &engine.resources.fission_log[self.initial_fissions..];
        let (fissions, fission_misses) = family_events::fissions(&self.departures, source_fissions);
        let days: Vec<_> = (1..=end.div_ceil(tpd)).map(|day| {
            let (couple, fallback) = self.births_by_day.get(&day).copied().unwrap_or_default();
            json!({"day":day,"births":couple+fallback,"couple_births":couple,"fallback_births":fallback})
        }).collect();
        let final_sample = family_samples::capture(engine, &self.people);
        let households: BTreeMap<_, _> = final_sample["people"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|row| Some((row["id"].as_u64()?, row["household_key"].clone())))
            .collect();
        let people = self
            .people
            .iter()
            .map(|(id, person)| {
                let mut row = person.json();
                if let Some(key) = households.get(id) {
                    row["household_key"] = key.clone();
                }
                row
            })
            .collect();
        FamilyOutput {
            files: vec![
                ("family_people", people),
                ("family_hours", self.hours.clone()),
                ("family_events", self.events.clone()),
                ("family_fissions", fissions),
            ],
            meta: json!({"schema_version":17,"end_tick":end,"people":self.people.len(),"births_total":births_total,
                "source_births_total":source_births,"departures_total":self.departures.len(),"source_departures_total":source_fissions.len(),
                "observation_misses":self.misses + births_total.abs_diff(source_births) + fission_misses,
                "day60":self.day60,"births_by_day":days,
                "clock":"tick is processed system tick; elapsed_ticks is completed engine ticks; simulation_day=tick/1440+1; display_day includes clock offset",
                "household_key":"smallest id of the sampled household; dead people retain last observed household",
                "partner":"raw Kin partner; metric couples require both alive and mutual"}),
        }
    }
}

pub(super) fn twins_match(first: &SimEngine, second: &SimEngine) -> bool {
    let snapshot = |engine: &SimEngine| {
        let mut people: Vec<_> = engine
            .world
            .query::<(&Agent, Option<&Sex>, Option<&Kin>)>()
            .iter()
            .map(|(_, (agent, sex, kin))| (agent.id, sex.copied(), kin.copied()))
            .collect();
        people.sort_by_key(|row| row.0);
        let mut events: Vec<_> = engine
            .world
            .query::<&FamilyObservations>()
            .iter()
            .map(|(entity, events)| (entity.to_bits().get(), events.clone()))
            .collect();
        events.sort_by_key(|row| row.0);
        (people, events)
    };
    snapshot(first) == snapshot(second)
}

#[cfg(test)]
#[path = "family_trace_tests.rs"]
mod tests;
