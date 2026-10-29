//! Reject malformed observation chronology without requiring historical owners to survive.
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::food_evidence::{AcquisitionReceipt, FlowCounters, OperationEvidence};
use sim_core::colony::ItemKind;
use sim_core::components::AgentId;
use sim_engine::SimResources;
use std::collections::BTreeSet;

fn receipt_valid(receipt: AcquisitionReceipt, now: u64, res: &SimResources) -> bool {
    receipt.tick <= now
        && receipt.job_started.is_none_or(|tick| tick <= receipt.tick)
        && receipt
            .target
            .is_none_or(|(x, y)| x < res.tile_grid.width && y < res.tile_grid.height)
}
fn owner_valid(owner: FoodOwner, item: ItemKind, res: &SimResources) -> bool {
    match owner {
        FoodOwner::Ground((x, y)) => x < res.tile_grid.width && y < res.tile_grid.height,
        FoodOwner::Carried(_) => true,
        FoodOwner::Shelf { key, .. } => key.kind == item,
        FoodOwner::Pot { id, key } => id > 0 && key.kind == item,
    }
}
fn recorded_day(res: &SimResources, tick: u64) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index_at(res, tick)).unwrap_or(u32::MAX)
}

pub(super) fn validate(agents: &BTreeSet<AgentId>, res: &SimResources) -> Result<(), String> {
    let evidence = &res.food.evidence;
    if evidence.recent.len() > 4 {
        return Err("food flow history exceeds bound".into());
    }
    let counters = FlowCounters::capture(&res.food.tallies);
    for (&agent, episode) in &evidence.active {
        if !agents.contains(&agent)
            || episode.since_tick > episode.last.tick
            || !receipt_valid(episode.last, res.current_tick, res)
            || counters.since(episode.counters).is_none()
        {
            return Err(format!("invalid acquisition episode for {agent}"));
        }
    }
    if evidence.cursor.is_some_and(|(tick, previous)| {
        tick > res.current_tick || counters.since(previous).is_none()
    }) {
        return Err("invalid food flow cursor".into());
    }
    let mut previous = None;
    for row in &evidence.recent {
        if row.from_tick >= row.to_tick
            || row.to_tick > res.current_tick
            || previous.is_some_and(|end| end != row.from_tick)
            || counters.since(row.units).is_none()
        {
            return Err("invalid food flow interval".into());
        }
        previous = Some(row.to_tick);
    }
    if previous.is_some_and(|end| evidence.cursor.is_none_or(|(tick, _)| tick != end)) {
        return Err("food flow cursor does not end recent history".into());
    }
    for (index, entry) in res.chronicle.entries().iter().enumerate() {
        if entry.kind == ChronicleKind::FoodOperation
            && u32::try_from(index)
                .ok()
                .is_none_or(|index| !evidence.operations.contains_key(&index))
        {
            return Err("food operation entry lacks payload".into());
        }
    }
    for (&index, operation) in &evidence.operations {
        let entry = res
            .chronicle
            .entry(index)
            .ok_or("missing food operation entry")?;
        if entry.kind != ChronicleKind::FoodOperation || entry.tick > res.current_tick {
            return Err("invalid food operation entry".into());
        }
        let day = recorded_day(res, entry.tick);
        let code = match *operation {
            OperationEvidence::Acquisition { receipt, flow } => {
                if entry.subject.is_none()
                    || !receipt_valid(receipt, entry.tick, res)
                    || flow.from_tick > receipt.tick
                    || receipt.tick > flow.to_tick
                    || flow.to_tick != entry.tick
                    || counters.since(flow.units).is_none()
                {
                    return Err("invalid acquisition chronology".into());
                }
                0
            }
            OperationEvidence::Intake {
                owner,
                item,
                born_day,
                lifetime_days,
            } => {
                if lifetime_days == 0
                    || entry.subject.is_none()
                    || born_day > day
                    || day - born_day >= lifetime_days
                    || !owner_valid(owner, item, res)
                    || !item
                        .definition()
                        .food
                        .as_ref()
                        .is_some_and(|food| food.cooked)
                {
                    return Err("invalid intake evidence".into());
                }
                1
            }
            OperationEvidence::Stored {
                owner,
                item,
                pieces,
                born_day,
            } => {
                if pieces == 0
                    || born_day > day
                    || !owner_valid(owner, item, res)
                    || item.definition().food.is_none()
                    || !item.definition().tags.iter().any(|tag| tag == "preserved")
                {
                    return Err("invalid storage operation".into());
                }
                2
            }
        };
        if entry.cause != code {
            return Err("food operation tag mismatch".into());
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "evidence_validation_tests.rs"]
mod tests;
