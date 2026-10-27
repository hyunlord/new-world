//! Resolve authored policy against actual provider and lifecycle registrations once.

use std::fmt;
use std::sync::Arc;

use sim_core::colony::mind::{self, CompiledMind, MindRegistrySchema, NodeId};

use super::context::{ContextError, RunningMetadata};
use sim_core::colony::mind::{MindRevision, RunningChoice};
use sim_core::components::AgentId;

use super::behaviours::{self, BehaviourEntry};
use super::facts::{ProviderPlan, ProviderRegistry, BUILTIN_PROVIDERS};

/// Policy preparation fails explicitly before an executor can choose or mutate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindProgramError(pub String);

impl fmt::Display for MindProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MindProgramError {}

/// Immutable resolved policy and its bound fact/effect callbacks.
pub struct MindProgram {
    source: String,
    /// Resolved immutable bands, nodes and registry identities.
    pub policy: CompiledMind,
    /// Bound factual readers for this exact policy revision.
    pub providers: ProviderPlan,
    effects: Vec<&'static BehaviourEntry>,
    running_revision: MindRevision,
    running_metadata: Vec<[Arc<RunningMetadata>; 2]>,
}

impl MindProgram {
    /// Compile a local policy without mutating a world or installing global data.
    pub fn load(source: &str) -> Result<Self, MindProgramError> {
        let registry = ProviderRegistry::new(BUILTIN_PROVIDERS)
            .map_err(|error| MindProgramError(error.to_string()))?;
        let schema = MindRegistrySchema {
            facts: registry.schema(),
            behaviours: behaviours::registry_schema(),
        };
        let policy =
            mind::compile(source, &schema).map_err(|error| MindProgramError(error.to_string()))?;
        let providers = registry
            .bind(&policy)
            .map_err(|error| MindProgramError(error.to_string()))?;
        let effects = policy
            .nodes()
            .iter()
            .map(|node| {
                let registered = policy.behaviour(node.behaviour).ok_or_else(|| {
                    MindProgramError(format!("mind node {} has no resolved behaviour", node.name))
                })?;
                behaviours::lookup(&registered.name).ok_or_else(|| {
                    MindProgramError(format!(
                        "mind behaviour {} has no lifecycle",
                        registered.name
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let running_metadata = policy
            .nodes()
            .iter()
            .map(|node| {
                let behaviour = policy.behaviour(node.behaviour).ok_or_else(|| {
                    MindProgramError(format!("mind node {} has no resolved behaviour", node.name))
                })?;
                Ok(std::array::from_fn(|finished| {
                    Arc::new(RunningMetadata {
                        behaviour: behaviour.name.clone(),
                        target: node.target.clone(),
                        finished: finished != 0,
                    })
                }))
            })
            .collect::<Result<Vec<_>, MindProgramError>>()?;
        let running_revision = policy.revision();
        Ok(Self {
            source: source.to_owned(),
            policy,
            providers,
            effects,
            running_revision,
            running_metadata,
        })
    }

    pub(super) fn running_metadata(
        &self,
        agent: AgentId,
        choice: RunningChoice,
    ) -> Result<Arc<RunningMetadata>, ContextError> {
        if choice.revision != self.policy.revision()
            || self.running_revision != self.policy.revision()
        {
            return Err(ContextError::InvalidRunningChoice(agent));
        }
        self.running_metadata
            .get(choice.node.index())
            .map(|variants| Arc::clone(&variants[usize::from(choice.finished)]))
            .ok_or(ContextError::InvalidRunningChoice(agent))
    }

    /// Exact authored bytes retained for save/reload policy identity.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Checked dictionary-local lookup; no authored string dispatch in the tick.
    pub(super) fn effect(&self, node: NodeId) -> Result<&'static BehaviourEntry, MindProgramError> {
        self.effects.get(node.index()).copied().ok_or_else(|| {
            MindProgramError(format!("mind node slot {} has no lifecycle", node.index()))
        })
    }
}
