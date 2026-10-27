//! Stateless eight-tick continuation for unit-cadence policies; never scores a selection.
use super::{
    model::{MindNode, Op, Predicate},
    CompiledMind, EvalKey, FactId, FactKind, LifeStage, MindEvalError, MindRevision, NodeId,
    RunningChoice,
};

/// One cheap direct observation, or an explicit request for the full provider path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardRead {
    /// Exact provider units; booleans must be zero or one.
    Value(i64),
    /// This provider requires preparation not available to the continuation guard.
    Unsupported,
}

/// Read-only direct observations for one person at one decision boundary.
///
/// Values and life stage must remain stable for the entire guard call. Missing or
/// malformed observations are errors; unsupported preparation is not a missing value.
pub trait ContinuationReader {
    /// Provider errors preserve ordinary selector input errors.
    type Error: From<MindEvalError>;
    /// Policy owning these fact slots.
    fn revision(&self) -> MindRevision;
    /// Person whose live observations are supplied.
    fn agent(&self) -> u64;
    /// Tick of the observations.
    fn tick(&self) -> u64;
    /// Current structural life stage.
    fn life_stage(&self) -> LifeStage;
    /// Read a direct fact without target discovery, effects or random draws.
    fn get(&mut self, fact: FactId) -> Result<GuardRead, Self::Error>;
}

/// Deterministic urgent-node inventory compiled once from an immutable policy.
#[derive(Debug, Clone)]
pub struct ContinuationPlan {
    instance: std::sync::Arc<()>,
    revision: MindRevision,
    urgent: Vec<NodeId>,
    next_after_failure: Vec<Vec<usize>>,
    prefix_after_failure: Vec<Vec<usize>>,
    adjacent_prefix: Vec<usize>,
    has_authored_cadence: bool,
}
impl ContinuationPlan {
    /// Whether this plan belongs to this immutable compiled instance (or its clone).
    pub fn is_for(&self, policy: &CompiledMind) -> bool {
        std::sync::Arc::ptr_eq(&self.instance, &policy.instance)
    }

    /// Include every critical node and every authored finish node, in band order.
    pub fn new(policy: &CompiledMind) -> Self {
        let urgent: Vec<NodeId> = policy
            .bands
            .iter()
            .flatten()
            .copied()
            .filter(|id| {
                let node = &policy.nodes[id.index()];
                node.band.index() == 0 || node.finish.is_some()
            })
            .collect();
        let mut next_after_failure: Vec<Vec<usize>> = vec![Vec::new(); urgent.len()];
        for index in (0..urgent.len()).rev() {
            let predicates = &policy.nodes[urgent[index].index()].predicates;
            let shared = urgent.get(index + 1).map_or(0, |next| {
                predicates
                    .iter()
                    .zip(&policy.nodes[next.index()].predicates)
                    .take_while(|(left, right)| left.cache_index == right.cache_index)
                    .count()
            });
            next_after_failure[index] = (0..predicates.len())
                .map(|predicate| {
                    if predicate < shared {
                        next_after_failure[index + 1][predicate]
                    } else {
                        index + 1
                    }
                })
                .collect();
        }
        let common_prefix = |index: usize, next: usize| {
            urgent.get(next).map_or(0, |next| {
                policy.nodes[urgent[index].index()]
                    .predicates
                    .iter()
                    .zip(&policy.nodes[next.index()].predicates)
                    .take_while(|(left, right)| left.cache_index == right.cache_index)
                    .count()
            })
        };
        let adjacent_prefix = (0..urgent.len())
            .map(|index| common_prefix(index, index + 1))
            .collect();
        let prefix_after_failure = next_after_failure
            .iter()
            .enumerate()
            .map(|(index, destinations)| {
                destinations
                    .iter()
                    .map(|&next| common_prefix(index, next))
                    .collect()
            })
            .collect();
        Self {
            instance: std::sync::Arc::clone(&policy.instance),
            revision: policy.revision(),
            urgent,
            next_after_failure,
            prefix_after_failure,
            adjacent_prefix,
            has_authored_cadence: policy.nodes.iter().any(|node| node.every_ticks != 1),
        }
    }
}

/// Why the unchanged selector must run; these are diagnostics, never new policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardWake {
    /// There is no prior choice to continue.
    NoRunning,
    /// The previous lifecycle ended.
    Finished,
    /// The fixed eight-tick phase or authored cadence requires full reconsideration.
    Cadence,
    /// Critical owners always receive full evaluation.
    CriticalIncumbent,
    /// A finish node cannot own a continuing lifecycle.
    FinishIncumbent,
    /// The running node no longer meets its stage or predicates.
    IncumbentIneligible,
    /// An authored critical or finish node is eligible now.
    UrgentEligible,
    /// An encountered predicate requires the full preparation path.
    UnsupportedFact,
}

/// Admission only: retained ticks have no fabricated ranking, score or witnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardOutcome {
    /// Run the original evaluator with fresh full observations.
    FullEvaluation {
        /// Structural reason for reconsideration.
        reason: GuardWake,
    },
    /// The caller may continue only after separately validating lifecycle ownership.
    ContinueOwned {
        /// The unchanged unfinished incumbent.
        node: NodeId,
    },
}

/// Reusable scan-local memoization; reset before a scan may consume any entries.
#[derive(Debug, Default)]
pub struct GuardScratch {
    facts: Vec<Option<i64>>,
    predicates: Vec<u8>,
    read_facts: Vec<usize>,
}
impl GuardScratch {
    fn reset(&mut self, policy: &CompiledMind) {
        for index in self.read_facts.drain(..) {
            self.facts[index] = None;
        }
        self.facts.resize(policy.facts.len(), None);
        self.predicates.resize(policy.predicate_count, 0);
        self.predicates.fill(0);
    }
}

#[derive(Clone, Copy)]
enum Eligibility {
    Eligible,
    Ineligible { failed_predicate: Option<usize> },
    Unsupported,
}

fn matches_predicate(value: i64, predicate: &Predicate) -> bool {
    match predicate.op {
        Op::Lt => value < predicate.value,
        Op::Le => value <= predicate.value,
        Op::Gt => value > predicate.value,
        Op::Ge => value >= predicate.value,
        Op::Eq => value == predicate.value,
        Op::Has => value == 1,
        Op::Not => value == 0,
    }
}

fn eligible<R: ContinuationReader>(
    policy: &CompiledMind,
    node: &MindNode,
    reader: &mut R,
    key: EvalKey,
    incumbent: bool,
    true_prefix: usize,
    scratch: &mut GuardScratch,
) -> Result<Eligibility, R::Error> {
    if node.stages & reader.life_stage().bit() == 0 {
        return Ok(Eligibility::Ineligible {
            failed_predicate: None,
        });
    }
    let period = u64::from(node.every_ticks);
    if !incumbent && period != 1 {
        let phase = (node.random_key % period + key.agent % period) % period;
        if key.tick % period != phase {
            return Ok(Eligibility::Ineligible {
                failed_predicate: None,
            });
        }
    }
    for (index, predicate) in node.predicates.iter().enumerate().skip(true_prefix) {
        let cached = scratch.predicates[predicate.cache_index];
        let matches = if cached != 0 {
            cached == 2
        } else {
            let fact = predicate.fact;
            let value = if let Some(value) = scratch.facts[fact.index()] {
                value
            } else {
                let value = match reader.get(fact)? {
                    GuardRead::Value(value) => value,
                    GuardRead::Unsupported => return Ok(Eligibility::Unsupported),
                };
                if policy.facts[fact.index()].kind == FactKind::Bool && !matches!(value, 0 | 1) {
                    return Err(MindEvalError::InvalidBoolean(fact).into());
                }
                scratch.facts[fact.index()] = Some(value);
                scratch.read_facts.push(fact.index());
                value
            };
            let matches = matches_predicate(value, predicate);
            scratch.predicates[predicate.cache_index] = if matches { 2 } else { 1 };
            matches
        };
        if !matches {
            return Ok(Eligibility::Ineligible {
                failed_predicate: Some(index),
            });
        }
    }
    Ok(Eligibility::Eligible)
}

/// Check generic eligibility interrupts and the fixed absolute phase before continuing.
///
/// The caller additionally owns history validity, replay exclusion, registered
/// continuation capability and live FSM/target ownership checks. No score terms
/// are read, and this function cannot choose an alternative node or emit evidence.
pub fn guard_continuation<R: ContinuationReader>(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut R,
    running: Option<RunningChoice>,
    key: EvalKey,
    scratch: &mut GuardScratch,
) -> Result<GuardOutcome, R::Error> {
    match preflight(policy, plan, reader, running, key)? {
        Preflight::Complete(outcome) => Ok(outcome),
        Preflight::Scan(running) => {
            scratch.reset(policy);
            scan(policy, plan, reader, running, key, scratch)
        }
    }
}

enum Preflight {
    Complete(GuardOutcome),
    Scan(RunningChoice),
}

fn preflight<R: ContinuationReader>(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &R,
    running: Option<RunningChoice>,
    key: EvalKey,
) -> Result<Preflight, R::Error> {
    if plan.revision != policy.revision()
        || reader.revision() != policy.revision()
        || running.is_some_and(|old| old.revision != policy.revision())
    {
        return Err(MindEvalError::PolicyMismatch.into());
    }
    if reader.agent() != key.agent || reader.tick() != key.tick {
        return Err(MindEvalError::KeyMismatch.into());
    }
    let full = |reason| GuardOutcome::FullEvaluation { reason };
    let Some(running) = running else {
        return Ok(Preflight::Complete(full(GuardWake::NoRunning)));
    };
    let incumbent = policy
        .node(running.node)
        .ok_or(MindEvalError::InvalidIndex("running node"))?;
    if running.finished {
        return Ok(Preflight::Complete(full(GuardWake::Finished)));
    }
    // Full evaluation preserves authored admission phases that may never intersect ours.
    if plan.has_authored_cadence || key.tick % 8 == key.agent % 8 {
        return Ok(Preflight::Complete(full(GuardWake::Cadence)));
    }
    if incumbent.band.index() == 0 {
        return Ok(Preflight::Complete(full(GuardWake::CriticalIncumbent)));
    }
    if incumbent.finish.is_some() {
        return Ok(Preflight::Complete(full(GuardWake::FinishIncumbent)));
    }
    Ok(Preflight::Scan(running))
}

fn scan<R: ContinuationReader>(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut R,
    running: RunningChoice,
    key: EvalKey,
    scratch: &mut GuardScratch,
) -> Result<GuardOutcome, R::Error> {
    let incumbent = policy
        .node(running.node)
        .ok_or(MindEvalError::InvalidIndex("running node"))?;
    let full = |reason| GuardOutcome::FullEvaluation { reason };
    let mut index = 0;
    let mut true_prefix = 0;
    while index < plan.urgent.len() {
        let node = policy
            .node(plan.urgent[index])
            .ok_or(MindEvalError::InvalidIndex("guard node"))?;
        match eligible(policy, node, reader, key, false, true_prefix, scratch)? {
            Eligibility::Eligible => return Ok(full(GuardWake::UrgentEligible)),
            Eligibility::Unsupported => return Ok(full(GuardWake::UnsupportedFact)),
            Eligibility::Ineligible {
                failed_predicate: Some(predicate),
            } => {
                // Every skipped node has this exact root-through-failure path.
                // Its earlier predicates and false terminal are already cached;
                // stage/cadence exclusions cannot add reads or make it eligible.
                // Only predicates strictly before the failure are known true.
                true_prefix = predicate.min(plan.prefix_after_failure[index][predicate]);
                index = plan.next_after_failure[index][predicate];
            }
            Eligibility::Ineligible {
                failed_predicate: None,
            } => {
                // Structural exclusion proves no new predicate, even on a shared path.
                true_prefix = true_prefix.min(plan.adjacent_prefix[index]);
                index += 1;
            }
        }
    }
    match eligible(policy, incumbent, reader, key, true, 0, scratch)? {
        Eligibility::Eligible => Ok(GuardOutcome::ContinueOwned { node: running.node }),
        Eligibility::Ineligible { .. } => Ok(full(GuardWake::IncumbentIneligible)),
        Eligibility::Unsupported => Ok(full(GuardWake::UnsupportedFact)),
    }
}

/// Exercise the optimized eligibility algorithm against its frozen oracle only.
/// Mixed-cadence fixtures bypass the new public fallback so their structural and
/// ordered-read assertions remain nonvacuous; public behavior is tested separately.
#[cfg(test)]
fn guard_continuation_algorithm_only<R: ContinuationReader>(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut R,
    running: Option<RunningChoice>,
    key: EvalKey,
    scratch: &mut GuardScratch,
) -> Result<GuardOutcome, R::Error> {
    let mut algorithm_plan = plan.clone();
    algorithm_plan.has_authored_cadence = false;
    guard_continuation(policy, &algorithm_plan, reader, running, key, scratch)
}

#[cfg(test)]
#[path = "continuation_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "continuation_reference.rs"]
mod reference;

#[cfg(test)]
#[path = "continuation_carry_tests.rs"]
mod carry_tests;

#[cfg(test)]
#[path = "continuation_cadence_tests.rs"]
mod cadence_tests;

#[path = "continuation_cache.rs"]
mod cache;
pub use cache::{guard_continuation_cached, GuardCache};
