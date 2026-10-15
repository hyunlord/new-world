use super::family_trace::Person;
use serde_json::{json, Value};
use sim_core::components::{Agent, AgentId, Child, Elder, Kin};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn capture(engine: &SimEngine, people: &BTreeMap<AgentId, Person>) -> Value {
    let r = &engine.resources;
    let mut homes = BTreeMap::new();
    let mut keys = BTreeMap::new();
    let mut settlements: Vec<_> = r.settlements.iter().collect();
    settlements.sort_by_key(|(id, _)| **id);
    for (&sid, settlement) in settlements {
        let members = settlement.member_agents.iter().copied().collect();
        for group in sim_systems::runtime::agent::households::households(&engine.world, &members) {
            if let Some(&key) = group.first() {
                for id in group {
                    homes.entry(id).or_insert(sid);
                    keys.entry(id).or_insert(key);
                }
            }
        }
    }
    let unassigned: BTreeSet<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, agent)| agent.id)
        .filter(|id| !homes.contains_key(id))
        .collect();
    for group in sim_systems::runtime::agent::households::households(&engine.world, &unassigned) {
        if let Some(&key) = group.first() {
            for id in group {
                keys.insert(id, key);
            }
        }
    }
    let mut rows: Vec<_> = engine.world.query::<(&Agent, Option<&Kin>, Option<&Child>, Option<&Elder>)>().iter()
        .map(|(_, (agent, kin, child, elder))| {
            let id = agent.id;
            (id, json!({"id":id,"settlement":homes.get(&id),
                "life_stage":if child.is_some() {"child"} else if elder.is_some() {"elder"} else {"adult"},
                "partner":kin.and_then(|kin| kin.partner),"pairing_tick":people.get(&id).and_then(|p| p.pairing_tick),
                "household_key":keys.get(&id).copied().unwrap_or(id),"bed":r.bed_of.get(&id)}))
        }).collect();
    rows.sort_by_key(|row| row.0);
    let rows: Vec<_> = rows.into_iter().map(|(_, row)| row).collect();
    let beds = rows.iter().filter(|row| !row["bed"].is_null()).count() as u64;
    let capacity = super::metrics::housing_capacity(engine).unwrap_or(0);
    let tpd = r.ticks_per_day.max(1);
    let clock = r.current_tick.saturating_add(r.rules.clock_offset_ticks);
    json!({"tick":r.current_tick,"elapsed_ticks":engine.current_tick(),"simulation_day":r.current_tick/tpd+1,
        "display_day":clock/tpd+1,"minute":clock%tpd,"people":rows,
        "housing_capacity":capacity,"people_with_bed":beds,"bedless":rows.len() as u64-beds,
        "unused_slots":capacity.saturating_sub(beds)})
}
