//! Emit complete switch reasons; critical band entries alone enter the durable chronicle.
//! Writes only mind state, causal records and chronicle; never chooses behaviour or reads timing.
mod draft;
mod memory;
pub use draft::draft_origins;
mod provenance;
pub use memory::memory_origins;
mod types;
pub use types::{CausePriority, FactCause, FactOrigin, RecordError, RecordPerson};
#[cfg(test)]
mod tests;
pub use provenance::{
    assigned_job_origins, automatic_origins, remember_completed_meal, source_cause, validate_source,
};

use sim_core::causal::{CausalEvent, EventId};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::mind::{CompiledMind, FactWitness, RunningChoice, Selection};
use sim_core::colony::mind_records::{
    ChoiceCandidate, ChoiceFact, MindChoiceRecord, MindFactOriginRecord, MindFactSource,
};
use sim_engine::SimResources;

/// Exact continuation needs no provenance work; the recorder still checks world/revision validity.
pub(super) fn is_unfinished_continuation(
    policy: &CompiledMind,
    prior: Option<RunningChoice>,
    selection: &Selection,
) -> bool {
    prior.is_some_and(|running| {
        running.revision == policy.revision()
            && !running.finished
            && running.node == selection.selected.node
    })
}

/// Record before effects and before replacing running state; continuing one unfinished node is a no-op.
pub fn record_selection(
    resources: &mut SimResources,
    policy: &CompiledMind,
    person: RecordPerson,
    selection: &Selection,
    witnesses: &[FactWitness],
    origins: &[FactOrigin],
) -> Result<Option<EventId>, RecordError> {
    if resources.rules.colony.is_none() {
        return Err(RecordError::Invalid("mind record outside colony"));
    }
    let prior = resources.mind.running.get(&person.agent).copied();
    if prior.is_some_and(|running| running.revision != policy.revision()) {
        return Err(RecordError::Invalid("running mind policy mismatch"));
    }
    resources
        .mind
        .history
        .as_mut()
        .ok_or(RecordError::Invalid("mind history not initialized"))?
        .check_initialized_policy(policy)?;
    if is_unfinished_continuation(policy, prior, selection) {
        return Ok(None);
    }
    let old_band = prior
        .and_then(|running| policy.node(running.node))
        .map(|node| node.band.index());
    let node = policy
        .node(selection.selected.node)
        .ok_or(RecordError::Invalid("selected node missing"))?;
    if node.band != selection.selected.band || node.behaviour != selection.selected.behaviour {
        return Err(RecordError::Invalid("selected node metadata mismatch"));
    }
    let node_code = u32::try_from(node.id.index())
        .map_err(|_| RecordError::Invalid("chronicle node code overflow"))?;
    let tile = tile_index(resources, person.position)?;
    let mut facts: Vec<_> = witnesses
        .iter()
        .map(|witness| ChoiceFact {
            fact: witness.fact,
            value: witness.value,
            cause: None,
        })
        .collect();
    facts.sort_by_key(|fact| fact.fact);
    if facts.windows(2).any(|w| w[0].fact == w[1].fact) {
        return Err(RecordError::Invalid("duplicate deciding fact"));
    }
    let next_id = resources
        .next_event_id
        .load(std::sync::atomic::Ordering::Relaxed);
    for origin in origins {
        if !witnesses.iter().any(|w| w.fact == origin.fact) {
            continue;
        }
        match origin.source {
            FactCause::Event(id) if id >= next_id => {
                return Err(RecordError::Invalid("fact cause was not issued"))
            }
            FactCause::Event(_) => {}
            FactCause::Source(source) => validate_source(resources, source)?,
        }
    }
    let mut parents = Vec::new();
    for fact in &mut facts {
        let origin = origins
            .iter()
            .filter(|o| o.fact == fact.fact)
            .min_by_key(|o| (o.priority, o.source));
        let cause = match origin {
            Some(origin) => {
                let id = match origin.source {
                    FactCause::Event(id) => id,
                    FactCause::Source(source) => materialize_source(resources, source, tile)?,
                };
                if node.uses_fact(fact.fact) {
                    parents.push((origin.priority, fact.fact, id, origin.source));
                }
                Some(id)
            }
            None => None,
        };
        fact.cause = cause;
    }
    parents.sort_by_key(|(priority, fact, id, _)| (*priority, *fact, *id));
    let parent = parents.first().map(|p| p.2);
    let durable_cause = parents.first().and_then(|p| source_cause(resources, p.3));
    let id = resources.issue_event_id();
    let record = MindChoiceRecord {
        revision: policy.revision(),
        id,
        parent,
        agent: person.agent,
        tick: resources.current_tick,
        node: selection.selected.node,
        band: selection.selected.band,
        score: selection.selected.score,
        top3: selection
            .top3
            .iter()
            .map(|c| ChoiceCandidate {
                node: c.node,
                score: c.score,
            })
            .collect(),
        facts,
    };
    let packed = resources
        .mind
        .history
        .as_mut()
        .ok_or(RecordError::Invalid("mind history missing"))?
        .push_record_for_initialized_policy(policy, &record)?;
    resources
        .causal_log
        .push(tile, CausalEvent::MindChoice { choice: packed });
    if selection.selected.band.index() == 0 && old_band != Some(0) {
        resources.chronicle.push_caused(
            ChronicleEntry::new(resources.current_tick, ChronicleKind::MindChoice)
                .subject(person.agent)
                .settlement(person.settlement)
                .cause(node_code)
                .value(selection.selected.score),
            durable_cause,
        );
    }
    resources.mind.running.insert(
        person.agent,
        RunningChoice {
            revision: policy.revision(),
            node: selection.selected.node,
            finished: false,
        },
    );
    Ok(Some(id))
}
fn tile_index(resources: &SimResources, (x, y): (u32, u32)) -> Result<u32, RecordError> {
    if x >= resources.tile_grid.width || y >= resources.tile_grid.height {
        return Err(RecordError::Invalid("mind record position outside world"));
    }
    y.checked_mul(resources.tile_grid.width)
        .and_then(|base| base.checked_add(x))
        .ok_or(RecordError::Invalid("mind record tile overflow"))
}
fn materialize_source(
    resources: &mut SimResources,
    source: MindFactSource,
    tile: u32,
) -> Result<EventId, RecordError> {
    validate_source(resources, source)?;
    if let Some(record) = resources.mind.fact_source_events.get(&source) {
        if record.tick > resources.current_tick
            || record.id
                >= resources
                    .next_event_id
                    .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(RecordError::Invalid("invalid cached fact source event"));
        }
        return Ok(record.id);
    }
    let id = resources.issue_event_id();
    let tick = resources.current_tick;
    resources
        .mind
        .fact_source_events
        .insert(source, MindFactOriginRecord { id, tick });
    resources
        .causal_log
        .push(tile, CausalEvent::MindFactOrigin { id, source, tick });
    Ok(id)
}
