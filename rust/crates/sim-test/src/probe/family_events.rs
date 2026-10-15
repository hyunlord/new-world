use super::family_trace::{FamilyTracer, Person};
use serde_json::{json, Value};
use sim_core::components::FamilyEvent;
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
            let row = json!({"event":"departed","fission_id":fission_id,"parent":parent,"leader":leader,"depart_tick":tick,
                "members":members,"households":households,"couples":couples,"couples_total":couples_total,
                "couples_split":couples_split,"households_split":households_split,"households_total":households.len(),
                "households_departed_whole":households.iter().filter(|group| !group.is_empty() && group.iter().all(|id| selected.contains(id))).count()});
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
