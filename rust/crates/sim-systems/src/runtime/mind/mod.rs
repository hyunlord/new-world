//! Colony mind: registered facts, pure data selection and explicit lifecycle effects.
//! Registration remains on the legacy decision until the measured switch commit.

mod behaviours;
pub mod context;
pub mod facts;
mod food_targets;
mod harvest_food;
pub(crate) mod job_origins;
mod program;
pub(crate) mod record;
pub(in crate::runtime) mod scheduling;
#[cfg(feature = "mind_shadow")]
pub mod shadow;
mod system;
#[cfg(test)]
mod system_tests;

pub use program::{MindProgram, MindProgramError};
pub use system::{ColonyMindSystem, MindTickError};

#[cfg(test)]
mod harvest_food_tests;

#[cfg(test)]
mod harvest_food_cache_tests;

#[cfg(test)]
mod meal_admission_tests;

#[cfg(test)]
mod harvest_stock_tests;

#[cfg(test)]
mod harvest_stock_boundary_tests;

#[cfg(test)]
mod harvest_admission_tests;
#[cfg(test)]
mod winter_stock_duty_tests;

/// Read a running choice's lifecycle binding against the target already committed
/// by its effect. Projection never resolves a fresh target or runs a policy.
pub(in crate::runtime) fn visible_action_target(
    res: &sim_engine::SimResources,
    agent: sim_core::components::AgentId,
    state: sim_core::components::AgentState,
    activity: Option<sim_core::components::Activity>,
    candidate: context::TargetCandidate,
) -> Option<behaviours::ActionTarget> {
    let choice = res
        .mind
        .running
        .get(&agent)
        .filter(|choice| !choice.finished)?;
    let dictionary = res.mind.history.as_ref()?.dictionary();
    if choice.revision.code() != dictionary.revision() {
        return None;
    }
    let binding = behaviours::lookup(dictionary.node_label(choice.node.index())?.behaviour)?;
    if binding.externally_owned {
        return None;
    }
    let target = binding.action_target(Some(candidate)).ok()?;
    binding.adopts(state, activity, target).then_some(target)
}

pub(in crate::runtime) use behaviours::ActionTarget;
