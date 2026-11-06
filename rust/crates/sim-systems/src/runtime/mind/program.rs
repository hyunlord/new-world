//! Resolve authored policy against actual provider and lifecycle registrations once.

use std::borrow::Cow;
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
    source: Cow<'static, str>,
    /// Resolved immutable bands, nodes and registry identities.
    pub policy: CompiledMind,
    /// Bound factual readers for this exact policy revision.
    pub providers: ProviderPlan,
    effects: Vec<&'static BehaviourEntry>,
    running_revision: MindRevision,
    registry_version: RegistryVersion,
    running_metadata: Vec<[Arc<RunningMetadata>; 2]>,
}

#[derive(Clone, Copy)]
enum RegistryVersion {
    Current,
    Format17,
    Format13,
}

impl MindProgram {
    /// Thermal activation follows the verified registry, never policy text matching.
    pub(crate) fn enables_numeric_thermal(&self) -> bool {
        match self.registry_version {
            RegistryVersion::Current => true,
            RegistryVersion::Format17 | RegistryVersion::Format13 => false,
        }
    }

    /// Compile policy supplied from static program data.
    pub fn load_static(source: &'static str) -> Result<Self, MindProgramError> {
        Self::compile(Cow::Borrowed(source))
    }

    /// Compile a local policy without mutating a world or installing global data.
    pub fn load(source: &str) -> Result<Self, MindProgramError> {
        Self::compile(Cow::Owned(source.to_owned()))
    }

    /// Restore only a schema whose complete dictionary and packed history verify.
    pub(crate) fn load_saved(
        source: &str,
        history: &sim_core::colony::mind_records::MindHistory,
    ) -> Result<Self, MindProgramError> {
        let current = Self::load(source)?;
        if history.check_policy(&current.policy).is_ok() {
            return Ok(current);
        }
        let predecessor =
            Self::compile_schema(Cow::Owned(source.to_owned()), RegistryVersion::Format17)?;
        if history.check_policy(&predecessor.policy).is_ok() {
            return Ok(predecessor);
        }
        let historical =
            Self::compile_schema(Cow::Owned(source.to_owned()), RegistryVersion::Format13)?;
        history
            .check_policy(&historical.policy)
            .map_err(|error| MindProgramError(error.to_string()))?;
        Ok(historical)
    }

    fn compile(source: Cow<'static, str>) -> Result<Self, MindProgramError> {
        Self::compile_schema(source, RegistryVersion::Current)
    }

    fn compile_schema(
        source: Cow<'static, str>,
        version: RegistryVersion,
    ) -> Result<Self, MindProgramError> {
        let registry = ProviderRegistry::new(BUILTIN_PROVIDERS)
            .map_err(|error| MindProgramError(error.to_string()))?;
        let mut facts = registry.schema();
        let mut behaviour_schema = behaviours::registry_schema();
        if !matches!(version, RegistryVersion::Current) {
            // Published Stage2 dictionaries predate these thermal registrations.
            // Filter the exact additions, then require the complete saved hash/history.
            facts.retain(|fact| {
                !matches!(
                    fact.name.as_str(),
                    "warm_refuge_available"
                        | "warm_refuge_active"
                        | "warm_refuge_here"
                        | "warm_refuge_owned"
                        | "recovery_needs_ready"
                        | "job_fuel_service"
                        | "health_full"
                )
            });
            behaviour_schema.retain(|behaviour| behaviour.name != "warm_refuge");
        }
        if matches!(version, RegistryVersion::Format13) {
            // dba478e54/e2ba280f3: these five registrations did not yet exist.
            // Revision hashing includes unused registrations; never rewrite a
            // saved dictionary or accept a hash without its complete history.
            facts.retain(|fact| {
                !matches!(
                    fact.name.as_str(),
                    "harvest_window"
                        | "job_food_transport"
                        | "harvest_storage_available"
                        | "winter_stock_duty"
                        | "water_target_available"
                )
            });
        }
        let schema = MindRegistrySchema {
            facts,
            behaviours: behaviour_schema,
        };
        let policy =
            mind::compile(&source, &schema).map_err(|error| MindProgramError(error.to_string()))?;
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
            source,
            policy,
            providers,
            effects,
            running_revision,
            registry_version: version,
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

#[cfg(test)]
#[path = "program_source_tests.rs"]
mod source_tests;
