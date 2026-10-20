use super::family_trace::{FamilyTracer, Person};
use serde_json::{json, Value};
use sim_core::components::{Agent, AgentId, Elder, FamilyEvent};
use sim_engine::{FissionRecord, SimEngine};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn observe(trace: &mut FamilyTracer, engine: &SimEngine, event: FamilyEvent) {
    let r = &engine.resources;
    let tick = r.current_tick;
    let tpd = r.ticks_per_day.max(1);
    let day = tick / tpd + 1;
    let mut row = match event {
        FamilyEvent::Paired { mother, father } => {
            for (id, partner) in [(mother, father), (father, mother)] {
                if let Some(person) = trace.people.get_mut(&id) {
                    person.partner = Some(partner);
                    person.pairing_tick = Some(tick);
                } else {
                    trace.misses += 1;
                }
            }
            json!({"event":"paired","mother":mother,"father":father})
        }
        FamilyEvent::Born {
            child,
            mother,
            father,
            fallback,
            lineage_key,
            female,
            temperament,
        } => {
            let mut person = Person::new(child);
            person.female = Some(female);
            person.mother = mother;
            person.father = father;
            person.lineage_key = Some(lineage_key);
            person.temperament = Some(temperament);
            person.born_tick = Some(tick);
            if trace.people.insert(child, person).is_some() {
                trace.misses += 1;
            }
            let counts = trace.births_by_day.entry(day).or_default();
            if fallback {
                counts.1 += 1;
            } else {
                counts.0 += 1;
            }
            json!({"event":"born","child":child,"mother":mother,"father":father,"fallback":fallback,
                "lineage_key":lineage_key,"female":female,"temperament":temperament})
        }
        FamilyEvent::Departed {
            fission_id,
            parent,
            leader,
            members,
            households,
            couples,
            couples_total,
            couples_split,
            households_split,
        } => {
            let selected: BTreeSet<_> = members.iter().copied().collect();
            let split_couples = couples
                .iter()
                .filter(|pair| selected.contains(&pair[0]) != selected.contains(&pair[1]))
                .count();
            let split_households = households
                .iter()
                .filter(|group| {
                    group.iter().any(|id| selected.contains(id))
                        && group.iter().any(|id| !selected.contains(id))
                })
                .count();
            if couples_total as usize != couples.len()
                || couples_split as usize != split_couples
                || households_split as usize != split_households
            {
                trace.misses += 1;
            }
            let mut row = json!({"event":"departed","fission_id":fission_id,"parent":parent,"leader":leader,"depart_tick":tick,
                "members":members,"households":households,"couples":couples,"couples_total":couples_total,
                "couples_split":couples_split,"households_split":households_split,"households_total":households.len(),
                "households_departed_whole":households.iter().filter(|group| !group.is_empty() && group.iter().all(|id| selected.contains(id))).count()});
            // Pass 18: elders who left and adult–elder couples (exactly one
            // partner an elder) split, from the members, the couples and the
            // `Elder` component as observed after the departure tick.
            let elders: BTreeSet<AgentId> = engine
                .world
                .query::<(&Agent, &Elder)>()
                .iter()
                .map(|(_, (agent, _))| agent.id)
                .collect();
            let adult_elder: Vec<[AgentId; 2]> = couples
                .iter()
                .filter(|pair| elders.contains(&pair[0]) != elders.contains(&pair[1]))
                .map(|pair| {
                    if elders.contains(&pair[0]) {
                        [pair[1], pair[0]]
                    } else {
                        *pair
                    }
                })
                .collect();
            let split_pairs: Vec<Value> = adult_elder
                .iter()
                .filter(|[adult, elder]| selected.contains(adult) != selected.contains(elder))
                .map(|[adult, elder]| json!({"adult": adult, "elder": elder, "adult_departed": selected.contains(adult)}))
                .collect();
            row["elders_departed"] = json!(members.iter().filter(|id| elders.contains(id)).count());
            row["adult_elder_couples"] = json!(adult_elder.len());
            row["adult_elder_couples_split"] = json!(split_pairs.len());
            row["adult_elder_split_pairs"] = json!(split_pairs);
            if trace.departures.insert(fission_id, row.clone()).is_some() {
                trace.misses += 1;
            }
            row
        }
    };
    row["tick"] = json!(tick);
    row["simulation_day"] = json!(day);
    row["display_day"] = json!(tick.saturating_add(r.rules.clock_offset_ticks) / tpd + 1);
    row["minute"] = json!(tick.saturating_add(r.rules.clock_offset_ticks) % tpd);
    trace.events.push(row);
}

pub(super) fn fissions(
    departures: &BTreeMap<u32, Value>,
    records: &[FissionRecord],
) -> (Vec<Value>, u64) {
    let mut misses = 0;
    let mut rows = Vec::new();
    let ids: BTreeSet<_> = records.iter().map(|record| record.fission_id).collect();
    misses += departures.keys().filter(|id| !ids.contains(id)).count() as u64;
    for record in records {
        let mut row = departures.get(&record.fission_id).cloned().unwrap_or_else(|| {
            misses += 1;
            json!({"fission_id":record.fission_id,"parent":record.parent,"depart_tick":record.depart_tick,"members":record.members})
        });
        if row["parent"] != record.parent
            || row["depart_tick"] != record.depart_tick
            || row["members"] != json!(record.members)
        {
            misses += 1;
        }
        row["status"] = json!(if record.cancelled_tick.is_some() {
            "cancelled"
        } else if record.founded_tick.is_some() {
            "completed"
        } else {
            "pending"
        });
        row["daughter"] = json!(record.daughter);
        row["founded_tick"] = json!(record.founded_tick);
        row["cancelled_tick"] = json!(record.cancelled_tick);
        rows.push(row);
    }
    (rows, misses)
}
