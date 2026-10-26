//! Validate saved policy bindings before registration, then restore shared payloads.

use crate::runtime::mind::MindProgram;
use sim_core::causal::{CausalEvent, CausalLogStorage};
use sim_core::chronicle::CauseRef;
use sim_core::colony::mind_records::MindFactSource;
use sim_engine::SimResources;
use std::collections::BTreeSet;

pub(super) fn validate(resources: &mut SimResources) -> Result<(), String> {
    let state = &resources.mind;
    let Some(source) = state.policy_source.as_deref() else {
        if !state.running.is_empty()
            || state.history.is_some()
            || !state.fact_origins.is_empty()
            || !state.fact_source_events.is_empty()
            || !state.order_origins.is_empty()
            || !state.job_origins.is_empty()
            || !state.draft_origins.is_empty()
            || resources.causal_log.iter().any(|(_, log)| {
                log.iter().any(|event| {
                    matches!(
                        event,
                        CausalEvent::MindChoice { .. } | CausalEvent::MindFactOrigin { .. }
                    )
                })
            })
        {
            return Err("mind state requires authored policy source".into());
        }
        return Ok(());
    };
    let next_id = resources
        .next_event_id
        .load(std::sync::atomic::Ordering::Relaxed);
    let mut bridge_ids = BTreeSet::new();
    for (&source, origin) in &state.fact_source_events {
        validate_source_at(resources, source, origin.tick)?;
        if origin.id >= next_id || !bridge_ids.insert(origin.id) {
            return Err("mind fact source event has unallocated or duplicate id".into());
        }
    }
    for &source in state.order_origins.values() {
        validate_source_at(resources, source, resources.current_tick)?;
    }
    for origin in state.job_origins.values() {
        if origin.taken_at > resources.current_tick {
            return Err("mind job assignment is in future".into());
        }
        // A command may retarget an existing job after its original taken_at.
        validate_source_at(resources, origin.source, resources.current_tick)?;
    }
    for origin in state.draft_origins.values() {
        if origin.since > resources.current_tick {
            return Err("mind draft identity is in future".into());
        }
        validate_source_at(resources, origin.source, resources.current_tick)?;
    }
    let program = MindProgram::load(source).map_err(|error| error.to_string())?;
    let policy = &program.policy;
    let history = state
        .history
        .as_ref()
        .ok_or("mind policy requires history dictionary")?;
    history
        .check_policy(policy)
        .map_err(|error| error.to_string())?;
    for choice in state.running.values() {
        if choice.revision != policy.revision() || policy.node(choice.node).is_none() {
            return Err("mind running choice does not match saved policy".into());
        }
    }
    for (_, fact) in state.fact_origins.keys() {
        if policy.fact_id_at(fact.index()).is_none() {
            return Err("mind fact origin uses unknown fact slot".into());
        }
    }
    // Rebuilding the sparse rings preserves their order and capacity while replacing
    // independently decoded Arcs with the history's identical payload where retained.
    let mut logs = CausalLogStorage::new();
    for (tile, log) in resources.causal_log.iter() {
        for event in log.iter() {
            let replacement = match event {
                CausalEvent::MindChoice { choice } => {
                    let record = choice
                        .decode(policy, history.dictionary())
                        .map_err(|error| error.to_string())?;
                    let choice = match history.find(record.agent, record.id) {
                        Some(saved) if saved.bytes() == choice.bytes() => saved.clone(),
                        Some(_) => return Err("mind tile/history payload mismatch".into()),
                        None => choice.clone(),
                    };
                    CausalEvent::MindChoice { choice }
                }
                CausalEvent::MindFactOrigin { id, source, tick } => {
                    validate_source_at(resources, *source, *tick)?;
                    if !state
                        .fact_source_events
                        .get(source)
                        .is_some_and(|record| record.id == *id && record.tick == *tick)
                    {
                        return Err("mind fact origin event does not match source cache".into());
                    }
                    event.clone()
                }
                other => other.clone(),
            };
            logs.push(*tile, replacement);
        }
    }
    resources.causal_log = logs;
    Ok(())
}

/// A source must already exist when its assignment or bridge was recorded.
fn validate_source_at(
    resources: &SimResources,
    source: MindFactSource,
    tick: u64,
) -> Result<(), String> {
    crate::runtime::mind::record::validate_source(resources, source)
        .map_err(|error| error.to_string())?;
    let source_tick = match source {
        MindFactSource::Command(index) => usize::try_from(index)
            .ok()
            .and_then(|index| resources.colony.command_log.get(index))
            .map(|record| record.tick),
        MindFactSource::Chronicle(
            CauseRef::Entry(index)
            | CauseRef::Pressure {
                entry: Some(index), ..
            }
            | CauseRef::Modifier {
                entry: Some(index), ..
            },
        ) => resources.chronicle.entry(index).map(|entry| entry.tick),
        MindFactSource::Chronicle(CauseRef::Sample { sample, .. }) => {
            resources.chronicle.sample(sample).map(|sample| sample.tick)
        }
        MindFactSource::Chronicle(
            CauseRef::Pressure { entry: None, .. } | CauseRef::Modifier { entry: None, .. },
        ) => None,
    };
    if tick > resources.current_tick || source_tick.is_none_or(|source_tick| source_tick > tick) {
        return Err("mind provenance predates its source or is in future".into());
    }
    Ok(())
}
