//! Observable hunt-to-haul transitions; disappearance never implies delivery or cancellation.
use serde::Serialize;
use sim_core::colony::{Carry, ItemKind, Job, JobKind};
use sim_core::components::Agent;
use sim_engine::SimEngine;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HaulStart {
    pub agent_id: u64,
    pub executed_tick: u64,
    pub source: (u32, u32),
    pub destination: (u32, u32),
    pub item_code: u32,
    pub destination_class: &'static str,
    pub observation: &'static str,
    pub taken_at: u64,
}
#[derive(Debug, Serialize)]
pub struct HaulTerminal {
    pub agent_id: u64,
    pub taken_at: u64,
    pub executed_tick: u64,
    pub outcome: &'static str,
    pub predicate_id: &'static str,
    pub source: (u32, u32),
    pub destination: (u32, u32),
    pub alive_after_tick: bool,
    pub job_present_after_tick: bool,
    pub carry_present_after_tick: bool,
}

pub fn jobs(e: &SimEngine) -> BTreeMap<u64, Job> {
    e.world
        .query::<(&Agent, &Job)>()
        .iter()
        .map(|(_, (a, j))| (a.id, *j))
        .collect()
}

pub fn starts(e: &SimEngine, before: &BTreeMap<u64, Job>) -> Vec<HaulStart> {
    let hearths = sim_systems::runtime::jobs::hearth::hearth_tiles(&e.resources);
    let mut rows = Vec::new();
    for (id, job) in jobs(e) {
        let Some(old) = before.get(&id) else { continue };
        let (JobKind::Hunt { .. }, JobKind::Haul { from, to }) = (old.kind, job.kind) else {
            continue;
        };
        if job.item != ItemKind::role("grazer_meat") || job.taken_at != e.resources.current_tick {
            continue;
        }
        let class = if e.resources.colony.zone_of_tile.contains_key(&to) {
            "stockpile"
        } else if hearths
            .iter()
            .any(|h| h.0.abs_diff(to.0).max(h.1.abs_diff(to.1)) <= 2)
        {
            "hearth"
        } else {
            "other"
        };
        rows.push(HaulStart {
            agent_id: id,
            executed_tick: e.resources.current_tick,
            source: from,
            destination: to,
            item_code: job.item.code(),
            destination_class: class,
            observation: "hunt_to_haul",
            taken_at: job.taken_at,
        });
    }
    rows
}

pub fn terminals(e: &SimEngine, active: &mut BTreeMap<u64, HaulStart>) -> Vec<HaulTerminal> {
    let now = jobs(e);
    let mut ended = Vec::new();
    let entities: BTreeMap<_, _> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, a)| (a.id, entity))
        .collect();
    for (&id, start) in active.iter() {
        let continuing = now.get(&id).is_some_and(|j| j.taken_at == start.taken_at && j.item.code() == start.item_code
            && matches!(j.kind, JobKind::Haul { from, to } if from == start.source && to == start.destination));
        if continuing {
            continue;
        }
        let entity = entities.get(&id);
        ended.push(HaulTerminal {
            agent_id: id,
            taken_at: start.taken_at,
            executed_tick: e.resources.current_tick,
            outcome: "UNRESOLVED_TERMINAL",
            predicate_id: "job_disappeared_or_replaced_no_unique_terminal_proof",
            source: start.source,
            destination: start.destination,
            alive_after_tick: entity.is_some(),
            job_present_after_tick: now.contains_key(&id),
            carry_present_after_tick: entity
                .is_some_and(|entity| e.world.get::<&Carry>(*entity).is_ok()),
        });
    }
    for row in &ended {
        active.remove(&row.agent_id);
    }
    ended
}
