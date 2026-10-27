//! Stateless P3d continuation admission and optional aggregate coverage diagnostics.
use super::{
    behaviours::LifecycleBinding,
    context::{PersonSnapshot, TickContext},
    facts::{FactError, ProviderPlan},
    MindProgram,
};
use hecs::World;
use sim_core::colony::mind::{
    guard_continuation, CompiledMind, ContinuationPlan, ContinuationReader, EvalKey, FactId,
    GuardOutcome, GuardRead, GuardScratch, LifeStage, MindRevision, NodeId, RunningChoice,
};
use sim_core::components::{Activity, AgentId, AgentState, SeekTarget, TargetKind};
use sim_engine::SimResources;

#[path = "scheduling_counts.rs"]
mod counts;
use counts::{wake_index, Counts};

pub(super) struct Diagnostics {
    plan: ContinuationPlan,
    scratch: GuardScratch,
    counts: Counts,
}
impl Diagnostics {
    pub(super) fn new(policy: &CompiledMind) -> Self {
        let behaviour_count = policy
            .nodes()
            .iter()
            .map(|node| node.behaviour.index())
            .max()
            .map_or(0, |index| index + 1);
        Self {
            plan: ContinuationPlan::new(policy),
            scratch: GuardScratch::default(),
            counts: Counts {
                continued_behaviours: vec![0; behaviour_count],
                visited_behaviours: vec![0; behaviour_count],
                unsupported_owner_behaviours: vec![0; behaviour_count],
                critical_incumbent_behaviours: vec![0; behaviour_count],
                unsupported_facts: vec![0; policy.facts().len()],
                ..Counts::default()
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn observe(
        &mut self,
        program: &MindProgram,
        world: &World,
        resources: &SimResources,
        context: &TickContext,
        agent: AgentId,
        prior: Option<RunningChoice>,
        force_full: bool,
    ) -> Option<NodeId> {
        self.counts.visits += 1;
        let bound_node = prior.and_then(|choice| program.policy.node(choice.node));
        if let Some(node) = bound_node {
            self.counts.visited_behaviours[node.behaviour.index()] += 1;
        } else {
            self.counts.visits_without_bound_behaviour += 1;
        }
        if force_full {
            self.counts.replay_or_invitation += 1;
            return None;
        }
        let Some(prior) = prior else {
            self.counts.full_reasons[0] += 1;
            return None;
        };
        if let Some(node) = bound_node.filter(|node| node.band.index() == 0) {
            self.counts.critical_incumbents += 1;
            self.counts.critical_incumbent_behaviours[node.behaviour.index()] += 1;
        }
        let Some(history) = resources.mind.history.as_ref() else {
            self.counts.missing_history += 1;
            return None;
        };
        let Some(latest) = history.latest(agent) else {
            self.counts.missing_history += 1;
            return None;
        };
        // Diagnostic-only strict decode: do not infer causal ownership from nonempty history.
        match latest.decode(&program.policy, history.dictionary()) {
            Ok(record)
                if record.agent == agent
                    && record.node == prior.node
                    && record.revision == prior.revision => {}
            Ok(_) => {
                self.counts.mismatched_history += 1;
                return None;
            }
            Err(_) => {
                self.counts.guard_errors += 1;
                return None;
            }
        }
        let Ok(effect) = program.effect(prior.node) else {
            self.counts.guard_errors += 1;
            return None;
        };
        if !effect.diagnostic_continuation() {
            self.counts.unsupported_owner += 1;
            if let Some(node) = bound_node {
                self.counts.unsupported_owner_behaviours[node.behaviour.index()] += 1;
            }
            return None;
        }
        let Ok(person) = context.row(agent) else {
            self.counts.guard_errors += 1;
            return None;
        };
        let seek = world
            .get::<&SeekTarget>(person.entity)
            .ok()
            .map(|target| *target);
        if !owner_matches(effect.lifecycle, person.state, person.activity, seek) {
            self.counts.owner_mismatch += 1;
            return None;
        }
        let mut reader = DirectReader {
            providers: &program.providers,
            context,
            resources,
            person,
            revision: program.policy.revision(),
            reads: 0,
            unsupported: None,
            trace: None,
        };
        let result = guard_continuation(
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
        );
        self.counts.reads += reader.reads;
        if let Some(fact) = reader.unsupported {
            self.counts.unsupported_facts[fact.index()] += 1;
        }
        match result {
            Ok(GuardOutcome::ContinueOwned { node }) => {
                self.counts.would_continue += 1;
                if let Some(bound) = program.policy.node(node) {
                    self.counts.continued_behaviours[bound.behaviour.index()] += 1;
                }
                Some(node)
            }
            Ok(GuardOutcome::FullEvaluation { reason }) => {
                self.counts.full_reasons[wake_index(reason)] += 1;
                None
            }
            // Diagnostics must not turn an unrequested guard input into a new runtime failure.
            // The unchanged full selector below remains authoritative for its own errors.
            Err(_) => {
                self.counts.guard_errors += 1;
                None
            }
        }
    }

    pub(super) fn compare(
        &mut self,
        policy: &CompiledMind,
        proposed: Option<NodeId>,
        actual: NodeId,
    ) {
        if let Some(proposed) = proposed {
            if proposed == actual {
                self.counts.same_node += 1;
            } else {
                self.counts.different_node += 1;
            }
            if policy.node(proposed).map(|n| n.behaviour)
                != policy.node(actual).map(|n| n.behaviour)
            {
                self.counts.different_behaviour += 1;
            }
        }
    }
}
impl Drop for Diagnostics {
    fn drop(&mut self) {
        // One process/system-lifetime summary, never per-person logging or serialized state.
        eprintln!("mind_continuation_diagnostic {:?}", self.counts);
    }
}

fn owner_matches(
    lifecycle: LifecycleBinding,
    state: AgentState,
    activity: Option<Activity>,
    seek: Option<SeekTarget>,
) -> bool {
    match lifecycle {
        LifecycleBinding::Resource(kind) => match state {
            AgentState::Seeking { target } => target == kind && seek.is_some(),
            AgentState::Consuming { target } => target == kind,
            AgentState::Idle => false,
        },
        LifecycleBinding::Routine(kind) => {
            activity.is_some_and(|active| active.kind == kind)
                && matches!(
                    state,
                    AgentState::Seeking {
                        target: TargetKind::Activity
                    } | AgentState::Consuming {
                        target: TargetKind::Activity
                    }
                )
        }
        LifecycleBinding::Idle
        | LifecycleBinding::Social
        | LifecycleBinding::Rest
        | LifecycleBinding::External => false,
    }
}

pub(in crate::runtime) type GuardTrace = Vec<(FactId, Result<GuardRead, String>)>;

struct DirectReader<'a> {
    providers: &'a ProviderPlan,
    context: &'a TickContext,
    resources: &'a SimResources,
    person: &'a PersonSnapshot,
    revision: MindRevision,
    reads: u64,
    unsupported: Option<FactId>,
    trace: Option<&'a mut GuardTrace>,
}
impl ContinuationReader for DirectReader<'_> {
    type Error = FactError;
    fn revision(&self) -> MindRevision {
        self.revision
    }
    fn agent(&self) -> AgentId {
        self.person.agent
    }
    fn tick(&self) -> u64 {
        self.context.tick
    }
    fn life_stage(&self) -> LifeStage {
        self.person.life_stage
    }
    fn get(&mut self, fact: FactId) -> Result<GuardRead, FactError> {
        self.reads += 1;
        let result =
            self.providers
                .continuation_read(fact, self.person, self.context, self.resources);
        if let Some(trace) = self.trace.as_deref_mut() {
            trace.push((fact, result.as_ref().copied().map_err(ToString::to_string)));
        }
        let value = result?;
        if matches!(value, GuardRead::Unsupported) {
            self.unsupported = Some(fact);
        }
        Ok(value)
    }
}

#[cfg(test)]
#[path = "scheduling_tests.rs"]
mod tests;

#[path = "continuation_runtime.rs"]
pub(in crate::runtime) mod runtime;
pub(in crate::runtime) use runtime::{owned_target, ContinuationEvidence, ContinuationGate};
