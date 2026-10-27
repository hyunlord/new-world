//! Frozen pre-optimization guard algorithm for differential tests.
use super::*;
/// Reusable ephemeral memoization; no entries survive into the next guard evaluation.
#[derive(Debug, Default)]
pub struct GuardScratch {
    facts: Vec<Option<i64>>,
    predicates: Vec<Option<bool>>,
    read_facts: Vec<usize>,
    read_predicates: Vec<usize>,
}
impl GuardScratch {
    fn reset(&mut self, policy: &CompiledMind) {
        for index in self.read_facts.drain(..) {
            self.facts[index] = None;
        }
        for index in self.read_predicates.drain(..) {
            self.predicates[index] = None;
        }
        self.facts.resize(policy.facts.len(), None);
        self.predicates.resize(policy.predicate_count, None);
    }
}

#[derive(Clone, Copy)]
enum Eligibility {
    Eligible,
    Ineligible,
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
    scratch: &mut GuardScratch,
) -> Result<Eligibility, R::Error> {
    if node.stages & reader.life_stage().bit() == 0 {
        return Ok(Eligibility::Ineligible);
    }
    let period = u64::from(node.every_ticks);
    if !incumbent && period != 1 {
        let phase = (node.random_key % period + key.agent % period) % period;
        if key.tick % period != phase {
            return Ok(Eligibility::Ineligible);
        }
    }
    for predicate in &node.predicates {
        let matches = if let Some(value) = scratch.predicates[predicate.cache_index] {
            value
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
            scratch.predicates[predicate.cache_index] = Some(matches);
            scratch.read_predicates.push(predicate.cache_index);
            matches
        };
        if !matches {
            return Ok(Eligibility::Ineligible);
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
        return Ok(full(GuardWake::NoRunning));
    };
    let incumbent = policy
        .node(running.node)
        .ok_or(MindEvalError::InvalidIndex("running node"))?;
    if running.finished {
        return Ok(full(GuardWake::Finished));
    }
    if key.tick % 8 == key.agent % 8 {
        return Ok(full(GuardWake::Cadence));
    }
    if incumbent.band.index() == 0 {
        return Ok(full(GuardWake::CriticalIncumbent));
    }
    if incumbent.finish.is_some() {
        return Ok(full(GuardWake::FinishIncumbent));
    }
    scratch.reset(policy);
    for id in &plan.urgent {
        let node = policy
            .node(*id)
            .ok_or(MindEvalError::InvalidIndex("guard node"))?;
        match eligible(policy, node, reader, key, false, scratch)? {
            Eligibility::Eligible => return Ok(full(GuardWake::UrgentEligible)),
            Eligibility::Unsupported => return Ok(full(GuardWake::UnsupportedFact)),
            Eligibility::Ineligible => {}
        }
    }
    match eligible(policy, incumbent, reader, key, true, scratch)? {
        Eligibility::Eligible => Ok(GuardOutcome::ContinueOwned { node: running.node }),
        Eligibility::Ineligible => Ok(full(GuardWake::IncumbentIneligible)),
        Eligibility::Unsupported => Ok(full(GuardWake::UnsupportedFact)),
    }
}
