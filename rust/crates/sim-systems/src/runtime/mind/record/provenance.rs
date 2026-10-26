//! Provenance adapters preserve source namespaces and require an actual recorded source.
use super::{CausePriority, FactCause, FactOrigin, RecordError};
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::mind::{CompiledMind, FactWitness};
use sim_core::colony::mind_records::MindFactSource;
use sim_core::colony::people::Fleeing;
use sim_core::colony::Job;
use sim_core::components::AgentId;
use sim_engine::SimResources;

/// Check the actual referenced durable record before issuing a causal bridge.
pub fn validate_source(
    resources: &SimResources,
    source: MindFactSource,
) -> Result<(), RecordError> {
    let tick = resources.current_tick;
    let valid = match source {
        MindFactSource::Command(index) => usize::try_from(index)
            .ok()
            .and_then(|i| resources.colony.command_log.get(i))
            .is_some_and(|record| record.result.accepted && record.tick <= tick),
        MindFactSource::Chronicle(cause) => match cause {
            CauseRef::Entry(index)
            | CauseRef::Modifier {
                entry: Some(index), ..
            } => resources
                .chronicle
                .entry(index)
                .is_some_and(|entry| entry.tick <= tick),
            CauseRef::Pressure {
                kind,
                entry: Some(index),
            } => resources.chronicle.entry(index).is_some_and(|entry| {
                entry.tick <= tick
                    && entry.kind == ChronicleKind::PressureBegan
                    && entry.cause == kind
            }),
            CauseRef::Sample { sample, .. } => resources
                .chronicle
                .sample(sample)
                .is_some_and(|sample| sample.tick <= tick),
            CauseRef::Pressure { entry: None, .. } | CauseRef::Modifier { entry: None, .. } => {
                false
            }
        },
    };
    if valid {
        Ok(())
    } else {
        Err(RecordError::Invalid(
            "mind fact source missing, rejected or in future",
        ))
    }
}
/// Durable chronicle ancestry behind a real source; command-only roots have no fabricated entry.
pub fn source_cause(resources: &SimResources, cause: FactCause) -> Option<CauseRef> {
    match cause {
        FactCause::Source(MindFactSource::Chronicle(cause)) => Some(cause),
        FactCause::Source(MindFactSource::Command(_)) => None,
        FactCause::Event(id) => {
            resources
                .mind
                .fact_source_events
                .iter()
                .find_map(|(source, record)| {
                    if record.id != id {
                        return None;
                    }
                    match source {
                        MindFactSource::Chronicle(cause) => Some(*cause),
                        MindFactSource::Command(_) => None,
                    }
                })
        }
    }
}
/// Actual flight owner identifies its animal and pack; no nearest-tile causal guess is made.
fn flight_source(resources: &SimResources, fleeing: Fleeing) -> Option<MindFactSource> {
    let animal = resources.colony.wildlife.animals.get(&fleeing.from)?;
    let day = animal.pack?;
    let pressure = resources
        .colony
        .story
        .active
        .iter()
        .chain(&resources.colony.story.history)
        .find(|pressure| pressure.start_day == day && pressure.kind.definition().pack.is_some())?;
    let source = MindFactSource::Chronicle(CauseRef::Pressure {
        kind: pressure.kind.code(),
        entry: pressure.began_entry,
    });
    validate_source(resources, source).ok()?;
    Some(source)
}
/// Gather only proven cached origins and the actual flight's pressure origin.
/// Job origins must be supplied by assignment provenance, not guessed from current targets.
pub fn automatic_origins(
    resources: &SimResources,
    policy: &CompiledMind,
    agent: AgentId,
    fleeing: Option<Fleeing>,
    witnesses: &[FactWitness],
) -> Vec<FactOrigin> {
    let threat = fleeing.and_then(|fleeing| flight_source(resources, fleeing));
    witnesses
        .iter()
        .filter_map(|witness| {
            let name = policy.facts().get(witness.fact.index())?.name.as_str();
            let is_threat = matches!(
                name,
                "fleeing" | "threat_distance" | "threat_present" | "reachable_shelter"
            );
            if is_threat {
                if let Some(source) = threat {
                    return Some(FactOrigin {
                        fact: witness.fact,
                        source: FactCause::Source(source),
                        priority: CausePriority::Threat,
                    });
                }
            }
            let id = resources
                .mind
                .fact_origins
                .get(&(agent, witness.fact))
                .copied()?;
            let priority = if matches!(
                name,
                "has_job" | "job_stage" | "craft_working" | "job_destination_distance"
            ) {
                CausePriority::Job
            } else if name.starts_with("hunger")
                || name.starts_with("thirst")
                || name.starts_with("fatigue")
            {
                CausePriority::Need
            } else {
                CausePriority::Other
            };
            Some(FactOrigin {
                fact: witness.fact,
                source: FactCause::Event(id),
                priority,
            })
        })
        .collect()
}
/// Bind assignment facts only to the exact job whose origin was captured at assignment.
pub fn assigned_job_origins(
    resources: &SimResources,
    policy: &CompiledMind,
    agent: AgentId,
    job: Option<Job>,
    witnesses: &[FactWitness],
) -> Vec<FactOrigin> {
    let Some(job) = job else { return Vec::new() };
    let Some(origin) = resources.mind.job_origins.get(&agent) else {
        return Vec::new();
    };
    if origin.kind != job.kind || origin.taken_at != job.taken_at {
        return Vec::new();
    }
    witnesses
        .iter()
        .filter_map(|witness| {
            let name = policy.facts().get(witness.fact.index())?.name.as_str();
            matches!(
                name,
                "has_job"
                    | "job_stage"
                    | "craft_working"
                    | "job_destination_distance"
                    | "job_owned_state"
                    | "job_working"
                    | "job_to_source"
                    | "job_to_dest"
                    | "job_distance"
            )
            .then_some(FactOrigin {
                fact: witness.fact,
                source: FactCause::Source(origin.source),
                priority: CausePriority::Job,
            })
        })
        .collect()
}
/// Call only after food consumption actually succeeded. The origin is that completed meal's
/// initiating eat choice, not an invented completion event or a merely attempted meal.
pub fn remember_completed_meal(
    resources: &mut SimResources,
    policy: &CompiledMind,
    agent: AgentId,
    choice_id: u64,
) -> Result<(), RecordError> {
    let history = resources
        .mind
        .history
        .as_ref()
        .ok_or(RecordError::Invalid("meal history missing"))?;
    let choice = history
        .find(agent, choice_id)
        .ok_or(RecordError::Invalid("meal choice missing"))?
        .decode(policy, history.dictionary())?;
    let node = policy
        .node(choice.node)
        .ok_or(RecordError::Invalid("meal node missing"))?;
    if policy.behaviour(node.behaviour).map(|b| b.name.as_str()) != Some("eat") {
        return Err(RecordError::Invalid("meal origin is not an eat choice"));
    }
    for name in [
        "hunger",
        "hunger_tenths",
        "hunger_excess_tenths",
        "hunger_actionable",
    ] {
        if let Some(fact) = policy.fact_id(name) {
            resources.mind.fact_origins.insert((agent, fact), choice_id);
        }
    }
    Ok(())
}
