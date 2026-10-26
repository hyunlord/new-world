//! Command attribution for the current drafting episode.
use super::{CausePriority, FactCause, FactOrigin};
use sim_core::colony::mind::{CompiledMind, FactWitness};
use sim_core::colony::people::Drafted;
use sim_core::components::AgentId;
use sim_engine::SimResources;

/// The generic drafted fact persists through autonomous MoveTo -> Hold and arming changes.
/// Order-specific facts would additionally require matching the captured order.
pub fn draft_origins(
    resources: &SimResources,
    policy: &CompiledMind,
    agent: AgentId,
    drafted: Option<Drafted>,
    witnesses: &[FactWitness],
) -> Vec<FactOrigin> {
    let Some(drafted) = drafted else {
        return Vec::new();
    };
    let Some(origin) = resources.mind.draft_origins.get(&agent) else {
        return Vec::new();
    };
    if origin.since != drafted.since {
        return Vec::new();
    }
    witnesses
        .iter()
        .filter_map(|witness| {
            (policy.facts().get(witness.fact.index())?.name == "drafted").then_some(FactOrigin {
                fact: witness.fact,
                source: FactCause::Source(origin.source),
                priority: CausePriority::Job,
            })
        })
        .collect()
}
