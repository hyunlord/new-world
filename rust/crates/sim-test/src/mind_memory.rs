//! Explicit synthetic full-ring capacity companion; never part of timed simulation.
use serde::Serialize;
use serde_json::{json, Value};
use sim_core::causal::CausalEvent;
use sim_core::components::{Agent, Position};
use sim_engine::SimEngine;
use sim_systems::runtime::mind::MindProgram;

const CAPACITY: usize = 16;
#[derive(Serialize)]
struct Checkpoint {
    allocator_live_bytes: Option<u64>,
    records: usize,
    min_records: usize,
    max_records: usize,
}
fn checkpoint(engine: &SimEngine) -> Result<(usize, Checkpoint), String> {
    let history = engine
        .resources
        .mind
        .history
        .as_ref()
        .ok_or("mind history absent")?;
    let mut people = 0;
    let mut records = 0;
    let mut min_records = usize::MAX;
    let mut max_records = 0;
    for (_, agent) in engine.world.query::<&Agent>().iter() {
        let count = history.records(agent.id).map_or(0, |ring| ring.len());
        people += 1;
        records += count;
        min_records = min_records.min(count);
        max_records = max_records.max(count);
    }
    Ok((
        people,
        Checkpoint {
            allocator_live_bytes: crate::bench::alloc_bytes(),
            records,
            min_records: if people == 0 { 0 } else { min_records },
            max_records,
        },
    ))
}
/// Fill every live person's ring with sixteen full copies of their largest retained record.
/// Header IDs are freshly issued and ticks are the final observed tick. Fact values, scores,
/// candidates and real cause IDs are unchanged. No simulation ticks may follow this operation.
/// Temporary compiled policy and decoded templates are dropped before the final allocator read.
pub fn saturate(engine: &mut SimEngine) -> Result<Value, String> {
    let (people, before) = checkpoint(engine)?;
    if people == 0 {
        return Err("mind saturation requires living people".into());
    }
    {
        let source = engine
            .resources
            .mind
            .policy_source
            .as_deref()
            .ok_or("mind policy absent")?;
        let program =
            MindProgram::load(source).map_err(|error| format!("mind policy: {error:?}"))?;
        let history = engine
            .resources
            .mind
            .history
            .as_ref()
            .ok_or("mind history absent")?;
        history
            .check_policy(&program.policy)
            .map_err(|error| error.to_string())?;
        let mut templates = Vec::with_capacity(people);
        // Validate every person before mutating any history or issuing any event IDs.
        for (_, (agent, position)) in engine.world.query::<(&Agent, Option<&Position>)>().iter() {
            let entity = position.ok_or_else(|| format!("person {} lacks position", agent.id))?;
            if entity.x >= engine.resources.tile_grid.width
                || entity.y >= engine.resources.tile_grid.height
            {
                return Err(format!("person {} position outside grid", agent.id));
            }
            let packed = history
                .records(agent.id)
                .and_then(|ring| ring.iter().max_by_key(|record| record.bytes().len()))
                .ok_or_else(|| format!("person {} lacks retained mind record", agent.id))?;
            let mut record = packed
                .decode(&program.policy, history.dictionary())
                .map_err(|error| error.to_string())?;
            record.tick = engine.resources.current_tick;
            let tile = entity
                .y
                .checked_mul(engine.resources.tile_grid.width)
                .and_then(|base| base.checked_add(entity.x))
                .ok_or("tile index overflow")?;
            templates.push((agent.id, tile, record));
        }
        templates.sort_by_key(|(agent, _, _)| *agent);
        for (_, tile, mut record) in templates {
            for _ in 0..CAPACITY {
                record.id = engine.resources.issue_event_id();
                let packed = engine
                    .resources
                    .mind
                    .history
                    .as_mut()
                    .ok_or("mind history absent")?
                    .push_record(&program.policy, &record)
                    .map_err(|error| error.to_string())?;
                engine
                    .resources
                    .causal_log
                    .push(tile, CausalEvent::MindChoice { choice: packed });
            }
        }
    }
    let (after_people, after) = checkpoint(engine)?;
    if after_people != people || after.min_records != CAPACITY || after.max_records != CAPACITY {
        return Err("mind saturation did not fill every live ring to sixteen".into());
    }
    let delta = before
        .allocator_live_bytes
        .zip(after.allocator_live_bytes)
        .map(|(before, after)| i128::from(after) - i128::from(before));
    Ok(json!({
        "method": "synthetic capacity: replace each live person's ring with sixteen copies of their largest actually retained complete record; fresh event IDs and final tick headers; unchanged fact values/scores/candidates/causes; shared tile/history Arc; no ticks after saturation; not natural runtime occupancy",
        "capacity": CAPACITY, "people": people, "saturated_people": after_people,
        "measured_after_saturation": true, "allocator_live_bytes": after.allocator_live_bytes,
        "allocator_delta_bytes": delta, "before": before, "after": after,
    }))
}
/// Only the exact explicit opt-in enables this destructive terminal measurement companion.
pub fn requested(value: Option<&str>) -> bool {
    value == Some("1")
}
/// Disabled companion returns without reading or changing simulation state.
pub fn companion(engine: &mut SimEngine, option: Option<&str>) -> Result<Option<Value>, String> {
    if requested(option) {
        saturate(engine).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[path = "mind_memory_tests.rs"]
mod tests;
