//! Shared runtime/host admission and typed current-owner target binding.
use super::{owner_matches, DirectReader, GuardTrace};
use crate::runtime::mind::{
    behaviours::{ActionTarget, LifecycleBinding},
    context::{PersonSnapshot, TickContext},
    facts::FactError,
    MindProgram,
};
use sim_core::colony::mind::{
    guard_continuation_cached, CompiledMind, ContinuationPlan, EvalKey, GuardCache, GuardOutcome,
    GuardScratch, GuardWake, RunningChoice,
};
use sim_core::components::{Activity, AgentId, AgentState, SeekTarget, TargetKind};
use sim_engine::SimResources;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct ContinuationEvidence {
    pub state: AgentState,
    pub activity: Option<Activity>,
    pub seek: Option<SeekTarget>,
    pub history_matches: bool,
    pub replay_or_invitation: bool,
}

struct CachedPerson {
    entity: hecs::Entity,
    path: GuardCache,
}

#[derive(Default)]
struct CacheDiagnostics {
    hits: u64,
    misses: u64,
}

pub(in crate::runtime) struct ContinuationGate {
    plan: ContinuationPlan,
    scratch: GuardScratch,
    entries: BTreeMap<AgentId, CachedPerson>,
    tick: Option<u64>,
    diagnostics: Option<CacheDiagnostics>,
    #[cfg(test)]
    hits: u64,
}
impl ContinuationGate {
    pub(in crate::runtime) fn new(policy: &CompiledMind) -> Self {
        Self {
            plan: ContinuationPlan::new(policy),
            scratch: GuardScratch::default(),
            entries: BTreeMap::new(),
            tick: None,
            diagnostics: std::env::var_os("WORLDSIM_MIND_GUARD_CACHE_DIAGNOSTIC")
                .filter(|value| value == "1")
                .map(|_| CacheDiagnostics::default()),
            #[cfg(test)]
            hits: 0,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime) fn evaluate(
        &mut self,
        program: &MindProgram,
        context: &TickContext,
        resources: &SimResources,
        agent: AgentId,
        prior: Option<RunningChoice>,
        evidence: ContinuationEvidence,
    ) -> Result<GuardOutcome, FactError> {
        self.evaluate_traced(program, context, resources, agent, prior, evidence, None)
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime) fn evaluate_traced(
        &mut self,
        program: &MindProgram,
        context: &TickContext,
        resources: &SimResources,
        agent: AgentId,
        prior: Option<RunningChoice>,
        evidence: ContinuationEvidence,
        trace: Option<&mut GuardTrace>,
    ) -> Result<GuardOutcome, FactError> {
        if !self.plan.is_for(&program.policy) {
            // Preserve the existing plan mismatch/error behavior if a caller reuses a gate.
            self.entries.clear();
        }
        if self.tick != Some(resources.current_tick) {
            if self.tick.is_some_and(|tick| tick > resources.current_tick) {
                self.entries.clear();
            } else {
                let mut rows = context.rows().iter().peekable();
                self.entries.retain(|&id, cached| {
                    while rows.peek().is_some_and(|row| row.agent < id) {
                        rows.next();
                    }
                    rows.peek()
                        .is_some_and(|row| row.agent == id && row.entity == cached.entity)
                });
            }
            self.tick = Some(resources.current_tick);
        }
        let result =
            self.evaluate_admitted(program, context, resources, agent, prior, evidence, trace);
        if !matches!(result, Ok(GuardOutcome::ContinueOwned { .. })) {
            if let Some(cached) = self.entries.get_mut(&agent) {
                cached.path.invalidate();
            }
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_admitted(
        &mut self,
        program: &MindProgram,
        context: &TickContext,
        resources: &SimResources,
        agent: AgentId,
        prior: Option<RunningChoice>,
        evidence: ContinuationEvidence,
        trace: Option<&mut GuardTrace>,
    ) -> Result<GuardOutcome, FactError> {
        let full = GuardOutcome::FullEvaluation {
            reason: GuardWake::NoRunning,
        };
        if evidence.replay_or_invitation || !evidence.history_matches {
            return Ok(full);
        }
        let Some(prior) = prior else {
            return Ok(full);
        };
        let effect = match program.effect(prior.node) {
            Ok(effect) => effect,
            Err(_) => return Ok(full),
        };
        if !effect.scheduled_continuation() {
            return Ok(full);
        }
        let person = context.row(agent)?;
        if evidence.state != person.state || evidence.activity != person.activity {
            return Ok(full);
        }
        if !owner_admitted(
            effect.lifecycle,
            OwnerState {
                state: person.state,
                activity: person.activity,
                seek: evidence.seek,
                has_job: person.job.is_some(),
                fleeing: person.fleeing.is_some(),
                drafted: person.drafted,
            },
        ) {
            return Ok(full);
        }
        let mut reader = DirectReader {
            providers: &program.providers,
            context,
            resources,
            person,
            revision: program.policy.revision(),
            reads: 0,
            unsupported: None,
            trace,
        };
        let cached = self.entries.entry(agent).or_insert_with(|| CachedPerson {
            entity: person.entity,
            path: GuardCache::default(),
        });
        if cached.entity != person.entity {
            cached.entity = person.entity;
            cached.path.invalidate();
        }
        let result = guard_continuation_cached(
            &program.policy,
            &self.plan,
            &mut reader,
            Some(prior),
            EvalKey {
                world_seed: resources.world_seed,
                agent,
                tick: resources.current_tick,
                replicate: resources.noise_replicate_for(resources.current_tick),
            },
            &mut self.scratch,
            &mut cached.path,
        );
        if let Some(diagnostics) = self.diagnostics.as_mut() {
            if matches!(result, Ok((_, true))) {
                diagnostics.hits += 1;
            } else {
                diagnostics.misses += 1;
            }
        }
        let (outcome, hit) = result?;
        #[cfg(test)]
        if hit {
            self.hits += 1;
        }
        #[cfg(not(test))]
        let _ = hit;
        Ok(outcome)
    }
}

impl Drop for ContinuationGate {
    fn drop(&mut self) {
        if let Some(diagnostics) = &self.diagnostics {
            let vector_capacity_bytes: usize = self
                .entries
                .values()
                .map(|entry| entry.path.allocated_fact_bytes())
                .sum();
            let entry_bytes = std::mem::size_of::<(AgentId, CachedPerson)>();
            eprintln!(
                "mind_guard_cache_diagnostic hits={} misses={} entries={} entry_bytes={} vector_capacity_bytes={} map_allocator_overhead=unmeasured",
                diagnostics.hits, diagnostics.misses, self.entries.len(), entry_bytes, vector_capacity_bytes
            );
        }
    }
}

/// Bind the live lifecycle owner; no nearest-source/peer search is performed.
pub(in crate::runtime) fn owned_target(
    binding: LifecycleBinding,
    person: &PersonSnapshot,
    context: &TickContext,
    seek: Option<SeekTarget>,
) -> Option<ActionTarget> {
    match binding {
        LifecycleBinding::Resource(_) => Some(ActionTarget::Tile(
            seek.map_or((person.position.x, person.position.y), |s| s.tile),
        )),
        LifecycleBinding::Routine(kind) => person
            .activity
            .filter(|activity| activity.kind == kind)
            .map(|activity| ActionTarget::Routine {
                kind,
                tile: activity.place,
            }),
        LifecycleBinding::Social => match person.state.target() {
            Some(TargetKind::Agent(peer)) => {
                Some(context.row(peer).ok().map_or(ActionTarget::None, |other| {
                    ActionTarget::Agent {
                        agent: peer,
                        tile: (other.position.x, other.position.y),
                    }
                }))
            }
            _ => None,
        },
        LifecycleBinding::Idle => Some(ActionTarget::None),
        LifecycleBinding::Rest | LifecycleBinding::External => None,
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct OwnerState {
    pub state: AgentState,
    pub activity: Option<Activity>,
    pub seek: Option<SeekTarget>,
    pub has_job: bool,
    pub fleeing: bool,
    pub drafted: bool,
}

pub(in crate::runtime) fn owner_admitted(binding: LifecycleBinding, owner: OwnerState) -> bool {
    match binding {
        LifecycleBinding::Idle => {
            owner.state == AgentState::Idle
                && !owner.has_job
                && !owner.fleeing
                && !owner.drafted
                && owner.activity.is_none()
                && owner.seek.is_none()
        }
        LifecycleBinding::Social => matches!(
            owner.state,
            AgentState::Seeking {
                target: TargetKind::Agent(_)
            } | AgentState::Consuming {
                target: TargetKind::Agent(_)
            }
        ),
        _ => owner_matches(binding, owner.state, owner.activity, owner.seek),
    }
}

#[cfg(test)]
#[path = "continuation_runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "continuation_cache_runtime_tests.rs"]
mod cache_tests;
