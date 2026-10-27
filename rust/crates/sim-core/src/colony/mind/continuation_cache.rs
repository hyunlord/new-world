//! Exact current-read replay of a previously successful guard path.
use super::*;
use std::sync::Arc;

const MAX_FACTS: usize = 32;

#[derive(Debug)]
struct Success {
    instance: Arc<()>,
    revision: MindRevision,
    agent: u64,
    node: NodeId,
    stage: LifeStage,
    tick: u64,
}

/// Derived, nonserialized success proof. Clearing it changes only evaluation cost.
#[derive(Debug, Default)]
pub struct GuardCache {
    success: Option<Success>,
    facts: Vec<(FactId, i64)>,
}
impl GuardCache {
    /// Allocated observation payload bytes, excluding this entry and allocator metadata.
    pub fn allocated_fact_bytes(&self) -> usize {
        self.facts.capacity() * std::mem::size_of::<(FactId, i64)>()
    }

    /// Discard validity while retaining the bounded observation buffer.
    pub fn invalidate(&mut self) {
        self.success = None;
    }
}

/// Re-read an exact successful path, or resume the ordinary guard from fresh facts.
///
/// The boolean reports an actual exact-value hit, never a selection or witness.
/// Callers must separately admit current lifecycle ownership and entity identity.
/// Public custom readers can continue using the stateless `guard_continuation`.
#[allow(clippy::too_many_arguments)]
pub fn guard_continuation_cached<R: ContinuationReader>(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut R,
    running: Option<RunningChoice>,
    key: EvalKey,
    scratch: &mut GuardScratch,
    cache: &mut GuardCache,
) -> Result<(GuardOutcome, bool), R::Error> {
    let result = evaluate(policy, plan, reader, running, key, scratch, cache);
    if !matches!(result, Ok((GuardOutcome::ContinueOwned { .. }, _))) {
        cache.invalidate();
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn evaluate<R: ContinuationReader>(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut R,
    running: Option<RunningChoice>,
    key: EvalKey,
    scratch: &mut GuardScratch,
    cache: &mut GuardCache,
) -> Result<(GuardOutcome, bool), R::Error> {
    let running = match preflight(policy, plan, reader, running, key)? {
        Preflight::Complete(outcome) => return Ok((outcome, false)),
        Preflight::Scan(running) => running,
    };
    let stage = reader.life_stage();
    let reusable = cache.success.as_ref().is_some_and(|old| {
        plan.is_for(policy)
            && Arc::ptr_eq(&old.instance, &policy.instance)
            && old.revision == policy.revision()
            && old.agent == key.agent
            && old.node == running.node
            && old.stage == stage
            && old.tick.checked_add(1) == Some(key.tick)
    });
    let mut changed = None;
    if reusable {
        for (index, &(fact, previous)) in cache.facts.iter().enumerate() {
            let value = match reader.get(fact)? {
                GuardRead::Value(value) => value,
                GuardRead::Unsupported => {
                    return Ok((
                        GuardOutcome::FullEvaluation {
                            reason: GuardWake::UnsupportedFact,
                        },
                        false,
                    ));
                }
            };
            if policy.facts[fact.index()].kind == FactKind::Bool && !matches!(value, 0 | 1) {
                return Err(MindEvalError::InvalidBoolean(fact).into());
            }
            if value != previous {
                changed = Some((index, fact, value));
                break;
            }
        }
        if changed.is_none() {
            if let Some(success) = cache.success.as_mut() {
                success.tick = key.tick;
            }
            return Ok((GuardOutcome::ContinueOwned { node: running.node }, true));
        }
    }
    scratch.reset(policy);
    if let Some((index, changed_fact, changed_value)) = changed {
        // Each stored prefix value has just been re-read and proved exactly equal.
        for &(fact, value) in &cache.facts[..index] {
            scratch.facts[fact.index()] = Some(value);
            scratch.read_facts.push(fact.index());
        }
        scratch.facts[changed_fact.index()] = Some(changed_value);
        scratch.read_facts.push(changed_fact.index());
    }
    cache.invalidate();
    let outcome = scan(policy, plan, reader, running, key, scratch)?;
    if matches!(outcome, GuardOutcome::ContinueOwned { .. })
        && plan.is_for(policy)
        && scratch.read_facts.len() <= MAX_FACTS
    {
        cache.facts.clear();
        // Exact reservation avoids geometric growth beyond the documented cap.
        if cache.facts.capacity() < scratch.read_facts.len() {
            cache.facts.reserve_exact(scratch.read_facts.len());
        }
        for &index in &scratch.read_facts {
            if let Some(value) = scratch.facts[index] {
                cache.facts.push((FactId(index), value));
            }
        }
        cache.success = Some(Success {
            instance: Arc::clone(&policy.instance),
            revision: policy.revision(),
            agent: key.agent,
            node: running.node,
            stage,
            tick: key.tick,
        });
    }
    Ok((outcome, false))
}

#[cfg(test)]
#[path = "continuation_cache_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "continuation_cache_scratch_tests.rs"]
mod scratch_tests;
