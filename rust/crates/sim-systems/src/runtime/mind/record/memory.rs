//! Actual memory contributors supplied by the immutable person snapshot.
use super::{CausePriority, FactCause, FactOrigin};
use sim_core::causal::EventId;
use sim_core::colony::mind::{CompiledMind, FactWitness, Selection};

/// Preserve the actual contributor of each memory fact on a memory-driven switch.
/// The array order is the snapshot's hunger/thirst/fatigue/construction/social/combat order.
pub fn memory_origins(
    policy: &CompiledMind,
    selection: &Selection,
    causes: &[Option<EventId>; 6],
    witnesses: &[FactWitness],
) -> Vec<FactOrigin> {
    if !policy
        .node(selection.selected.node)
        .is_some_and(|node| node.tags.iter().any(|tag| tag == "memory"))
    {
        return Vec::new();
    }
    let names = [
        "memory_hunger",
        "memory_thirst",
        "memory_fatigue",
        "memory_construction",
        "memory_social",
        "memory_combat",
    ];
    witnesses
        .iter()
        .filter_map(|witness| {
            let name = &policy.facts().get(witness.fact.index())?.name;
            let index = names.iter().position(|candidate| candidate == name)?;
            Some(FactOrigin {
                fact: witness.fact,
                source: FactCause::Event(causes[index]?),
                priority: CausePriority::Other,
            })
        })
        .collect()
}
