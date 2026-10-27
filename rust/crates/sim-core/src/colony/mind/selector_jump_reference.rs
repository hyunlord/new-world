//! Frozen candidate4 eligibility and full selector oracle.
use super::*;
fn eligible<R: FactReader>(
    node: &MindNode,
    row: &mut R,
    key: EvalKey,
    incumbent: bool,
    scratch: &mut EvalScratch,
) -> Result<bool, R::Error> {
    if node.stages & row.life_stage().bit() == 0 {
        return Ok(false);
    }
    let period = u64::from(node.every_ticks);
    // A fixed arithmetic phase is scheduling, not a random choice. It is stable
    // under unrelated node insertion; running nodes always revalidate now.
    if period != 1 {
        let phase = (node.random_key % period + key.agent % period) % period;
        if !incumbent && key.tick % period != phase {
            return Ok(false);
        }
    }
    for predicate in &node.predicates {
        let cached = if R::STABLE_READS {
            scratch.predicates[predicate.cache_index]
        } else {
            0
        };
        let matches = if cached & PREDICATE_VALID != 0 {
            let matches = cached & PREDICATE_MATCH != 0;
            let roles = predicate_roles(matches, incumbent);
            if cached & roles != roles {
                // VALID is published only after record() installs the witness slot.
                // Neither slots nor witness order change until clear() resets both.
                let index = scratch.slots[predicate.fact.index()]
                    .ok_or(MindEvalError::InvalidIndex("cached predicate witness"))?;
                scratch.witnesses[index].roles |= roles;
                scratch.predicates[predicate.cache_index] = cached | roles;
            }
            matches
        } else {
            let value = row.get(predicate.fact)?;
            let matches = predicate_matches(value, predicate);
            let roles = predicate_roles(matches, incumbent);
            scratch.record(predicate.fact, value, roles);
            if R::STABLE_READS {
                scratch.predicates[predicate.cache_index] =
                    PREDICATE_VALID | if matches { PREDICATE_MATCH } else { 0 } | roles;
            }
            matches
        };
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}
pub fn choose_with_reader<R: FactReader>(
    policy: &CompiledMind,
    row: &mut R,
    running: Option<RunningChoice>,
    key: EvalKey,
    scratch: &mut EvalScratch,
) -> Result<Selection, R::Error> {
    scratch.clear(
        policy.facts().len(),
        if R::STABLE_READS {
            policy.predicate_count
        } else {
            0
        },
    );
    if row.revision() != policy.revision()
        || running.is_some_and(|old| old.revision != policy.revision())
    {
        return Err(MindEvalError::PolicyMismatch.into());
    }
    if row.agent() != key.agent || row.tick() != key.tick {
        return Err(MindEvalError::KeyMismatch.into());
    }
    if running.is_some_and(|old| policy.node(old.node).is_none()) {
        return Err(MindEvalError::InvalidIndex("running node").into());
    }
    for band in &policy.bands {
        let mut top3: ArrayVec<Candidate, 3> = ArrayVec::new();
        let mut current = None;
        for id in band {
            let node = &policy.nodes[id.index()];
            let incumbent = running.is_some_and(|old| old.node == *id && !old.finished);
            if eligible(node, row, key, incumbent, scratch)? {
                let score = score(node, row, incumbent, scratch)?;
                let candidate = Candidate {
                    node: *id,
                    band: node.band,
                    behaviour: node.behaviour,
                    score,
                };
                if incumbent {
                    current = Some(candidate);
                }
                let position = top3
                    .iter()
                    .position(|other| outranks(&candidate, other))
                    .unwrap_or(top3.len());
                if position < 3 {
                    if top3.is_full() {
                        top3.pop();
                    }
                    top3.insert(position, candidate);
                }
            }
        }
        let Some(best) = top3.first().copied() else {
            continue;
        };
        let mut selected = best;
        let mut retained = false;
        if let Some(current) = current {
            let margin = policy.nodes[current.node.index()].hysteresis;
            if i128::from(best.score) - i128::from(current.score) <= i128::from(margin) {
                selected = current;
                retained = best.node != current.node;
            }
        }
        let transition = match running {
            None => Transition::Start,
            Some(old) if old.node != selected.node => Transition::Switch,
            Some(old) if old.finished => Transition::RestartAfterFinish,
            Some(_) => Transition::Continue,
        };
        return Ok(Selection {
            selected,
            top3,
            transition,
            retained_by_hysteresis: retained,
        });
    }
    Err(MindEvalError::NoEligibleNode.into())
}
