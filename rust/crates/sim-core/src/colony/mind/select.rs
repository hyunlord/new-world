//! Pure ordered-band selection; checked integer scores and incumbent hysteresis.

use super::{
    model::*, BlockedCandidate, Candidate, EvalKey, FactReader, FactRow, MindEvalError,
    RunningChoice, Selection, Transition,
};
use arrayvec::ArrayVec;

/// Why a factual value was read during the decisive band walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessRole {
    /// Value participated in a satisfied eligibility predicate.
    Predicate,
    /// Value ruled out an evaluated node.
    RejectedPredicate,
    /// Value contributed to an eligible candidate's score.
    Score,
    /// Value revalidated the running node's eligibility.
    IncumbentPredicate,
    /// Value determined the running score used for hysteresis.
    IncumbentScore,
}
impl WitnessRole {
    const fn bit(self) -> u8 {
        match self {
            Self::Predicate => 1,
            Self::RejectedPredicate => 2,
            Self::Score => 4,
            Self::IncumbentPredicate => 8,
            Self::IncumbentScore => 16,
        }
    }
}
/// One exact factual value; repeated reads share the same witness and role bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactWitness {
    /// Registered column under the selection's policy revision.
    pub fact: FactId,
    /// Exact i64 value at the evaluation boundary; Bool uses zero/one.
    pub value: i64,
    roles: u8,
}
impl FactWitness {
    /// Whether this value participated in the specified explanatory role.
    pub const fn has_role(self, role: WitnessRole) -> bool {
        self.roles & role.bit() != 0
    }
}
/// Internal record of a rejected node, before filtering to BlockedCandidate.
#[derive(Debug, Clone, Copy)]
struct RejectedEntry {
    node: NodeId,
    band: BandId,
    depth: usize,
    fact: FactId,
    op_str: &'static str,
    threshold: i64,
    actual: i64,
}
/// Caller-owned reusable storage; no simulation state or causal IDs are touched.
#[derive(Debug, Default)]
pub struct EvalScratch {
    witnesses: Vec<FactWitness>,
    slots: Vec<Option<usize>>,
    predicates: Vec<u8>,
    /// Rejected nodes captured during band evaluation (presentation only).
    rejected: Vec<RejectedEntry>,
}
// Low five bits retain WitnessRole flags; the upper bits describe cache state.
const PREDICATE_VALID: u8 = 1 << 7;
const PREDICATE_MATCH: u8 = 1 << 6;
/// Maximum rejected entries buffered during one evaluation.
const REJECTED_CAP: usize = 16;
impl EvalScratch {
    /// All actual predicate and score inputs read by the last completed evaluation.
    pub fn witnesses(&self) -> &[FactWitness] {
        &self.witnesses
    }
    /// Up to 2 blocked candidates from the most recent evaluation, filtered to
    /// nodes in bands above the winning band. Presentation only.
    pub fn blocked_candidates(&self, winning_band: BandId) -> ArrayVec<BlockedCandidate, 2> {
        let mut out = ArrayVec::new();
        for entry in &self.rejected {
            if entry.band.index() < winning_band.index() && out.len() < 2 {
                out.push(BlockedCandidate {
                    node: entry.node,
                    band: entry.band,
                    predicate_depth: entry.depth,
                    fact: entry.fact,
                    op: entry.op_str,
                    threshold: entry.threshold,
                    actual: entry.actual,
                });
            }
        }
        out
    }
    fn clear(&mut self, columns: usize, predicates: usize) {
        self.witnesses.clear();
        self.slots.resize(columns, None);
        self.slots.fill(None);
        self.predicates.resize(predicates, 0);
        self.predicates.fill(0);
        self.rejected.clear();
    }
    #[inline]
    fn record(&mut self, fact: FactId, value: i64, roles: u8) {
        if let Some(index) = self.slots[fact.index()] {
            self.witnesses[index].roles |= roles;
        } else {
            self.slots[fact.index()] = Some(self.witnesses.len());
            self.witnesses.push(FactWitness { fact, value, roles });
        }
    }
}
fn predicate_matches(value: i64, predicate: &Predicate) -> bool {
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
#[inline]
const fn predicate_roles(matches: bool, incumbent: bool) -> u8 {
    (if matches {
        WitnessRole::Predicate.bit()
    } else {
        WitnessRole::RejectedPredicate.bit()
    }) | if incumbent {
        WitnessRole::IncumbentPredicate.bit()
    } else {
        0
    }
}
#[derive(Clone, Copy)]
enum Eligibility {
    Eligible,
    Structural,
    Rejected(usize),
}
fn eligibility<R: FactReader>(
    node: &MindNode,
    row: &mut R,
    key: EvalKey,
    incumbent: bool,
    scratch: &mut EvalScratch,
) -> Result<Eligibility, R::Error> {
    if node.stages & row.life_stage().bit() == 0 {
        return Ok(Eligibility::Structural);
    }
    let period = u64::from(node.every_ticks);
    // A fixed arithmetic phase is scheduling, not a random choice. It is stable
    // under unrelated node insertion; running nodes always revalidate now.
    if period != 1 {
        let phase = (node.random_key % period + key.agent % period) % period;
        if !incumbent && key.tick % period != phase {
            return Ok(Eligibility::Structural);
        }
    }
    for (depth, predicate) in node.predicates.iter().enumerate() {
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
            return Ok(Eligibility::Rejected(depth));
        }
    }
    Ok(Eligibility::Eligible)
}
#[cfg(test)]
fn eligible<R: FactReader>(
    node: &MindNode,
    row: &mut R,
    key: EvalKey,
    incumbent: bool,
    scratch: &mut EvalScratch,
) -> Result<bool, R::Error> {
    Ok(matches!(
        eligibility(node, row, key, incumbent, scratch)?,
        Eligibility::Eligible
    ))
}
fn score<R: FactReader>(
    node: &MindNode,
    row: &mut R,
    incumbent: bool,
    scratch: &mut EvalScratch,
) -> Result<i64, R::Error> {
    let mut total = node.constant;
    for term in &node.terms {
        let value = row.get(term.fact)?;
        let roles = WitnessRole::Score.bit()
            | if incumbent {
                WitnessRole::IncumbentScore.bit()
            } else {
                0
            };
        scratch.record(term.fact, value, roles);
        let overflow = || MindEvalError::ScoreOverflow {
            node: node.id,
            fact: term.fact,
        };
        let contribution = if term.weight == 1000 {
            value
        } else {
            let product = i128::from(value) * i128::from(term.weight) / 1000;
            i64::try_from(product).map_err(|_| overflow())?
        };
        total = total.checked_add(contribution).ok_or_else(overflow)?;
    }
    Ok(total)
}
fn outranks(left: &Candidate, right: &Candidate) -> bool {
    left.score > right.score
        || (left.score == right.score && left.node.index() < right.node.index())
}

/// Choose from the first eligible band, retaining an eligible running node unless
/// a same-band challenger strictly exceeds its margin. Scratch is the only write.
pub fn choose(
    policy: &CompiledMind,
    mut row: FactRow<'_>,
    running: Option<RunningChoice>,
    key: EvalKey,
    scratch: &mut EvalScratch,
) -> Result<Selection, MindEvalError> {
    choose_with_reader(policy, &mut row, running, key, scratch)
}

/// Choose with on-demand provider reads in exactly the eager selector's witness order.
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
        let incumbent_position = running
            .filter(|old| !old.finished)
            .and_then(|old| band.binary_search(&old.node).ok());
        let mut position_in_band = 0;
        while position_in_band < band.len() {
            let id = band[position_in_band];
            let node = &policy.nodes[id.index()];
            let incumbent = incumbent_position == Some(position_in_band);
            let eligibility = eligibility(node, row, key, incumbent, scratch)?;
            let mut next = position_in_band + 1;
            if R::STABLE_READS {
                if let Eligibility::Rejected(depth) = eligibility {
                    next =
                        policy.false_prefix_end[policy.node_predicate_offsets[id.index()] + depth];
                    if let Some(current) =
                        incumbent_position.filter(|current| *current > position_in_band)
                    {
                        next = next.min(current);
                    }
                    // STABLE_READS permits omitted get calls, not omitted structural observations.
                    for _ in position_in_band + 1..next {
                        let _ = row.life_stage();
                    }
                }
            }
            position_in_band = next;
            // Record rejected non-structural nodes for why-not presentation.
            // This is a pure side-effect on scratch; it never affects selection.
            if let Eligibility::Rejected(depth) = eligibility {
                if scratch.rejected.len() < REJECTED_CAP {
                    let pred = &node.predicates[depth];
                    let actual = scratch.slots[pred.fact.index()]
                        .map(|i| scratch.witnesses[i].value)
                        .unwrap_or(0);
                    let op_str = match pred.op {
                        Op::Lt => "<",
                        Op::Le => "<=",
                        Op::Gt => ">",
                        Op::Ge => ">=",
                        Op::Eq => "==",
                        Op::Has => "has",
                        Op::Not => "not",
                    };
                    scratch.rejected.push(RejectedEntry {
                        node: id,
                        band: node.band,
                        depth,
                        fact: pred.fact,
                        op_str,
                        threshold: pred.value,
                        actual,
                    });
                }
            }
            if matches!(eligibility, Eligibility::Eligible) {
                let score = score(node, row, incumbent, scratch)?;
                let candidate = Candidate {
                    node: id,
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

#[cfg(test)]
#[path = "stream_tests.rs"]
mod stream_tests;

#[cfg(test)]
#[path = "selector_jump_tests.rs"]
mod jump_tests;
