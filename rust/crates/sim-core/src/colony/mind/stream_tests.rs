//! Retained pre-streaming algorithm compared with production selection.
use super::*;
use crate::colony::mind::tests::{facts, key, node, policy, schema};
use crate::colony::mind::{compile, FactId, LifeStage, MindRevision};

fn retained_vector_oracle<R: FactReader>(
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
    let mut eligible_candidates = Vec::new();
    for band in &policy.bands {
        for id in band {
            let node = &policy.nodes[id.index()];
            let incumbent = running.is_some_and(|old| old.node == *id && !old.finished);
            if eligible(node, row, key, incumbent, scratch)? {
                let score = score(node, row, incumbent, scratch)?;
                eligible_candidates.push(Candidate {
                    node: *id,
                    band: node.band,
                    behaviour: node.behaviour,
                    score,
                });
            }
        }
        if eligible_candidates.is_empty() {
            continue;
        }
        let mut top3: ArrayVec<Candidate, 3> = ArrayVec::new();
        for candidate in &eligible_candidates {
            let position = top3
                .iter()
                .position(|other| outranks(candidate, other))
                .unwrap_or(top3.len());
            if position < 3 {
                if top3.is_full() {
                    top3.pop();
                }
                top3.insert(position, *candidate);
            }
        }
        let best = *top3.first().ok_or(MindEvalError::NoEligibleNode)?;
        let mut selected = best;
        let mut retained = false;
        if let Some(old) = running.filter(|old| !old.finished) {
            if let Some(current) = eligible_candidates
                .iter()
                .find(|candidate| candidate.node == old.node)
            {
                let margin = policy.nodes[old.node.index()].hysteresis;
                if i128::from(best.score) - i128::from(current.score) <= i128::from(margin) {
                    selected = *current;
                    retained = best.node != current.node;
                }
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

struct TraceReader<'a, const STABLE: bool> {
    row: FactRow<'a>,
    reads: Vec<FactId>,
    fail_fact: Option<FactId>,
}
impl<const STABLE: bool> FactReader for TraceReader<'_, STABLE> {
    const STABLE_READS: bool = STABLE;
    type Error = MindEvalError;
    fn agent(&self) -> crate::components::AgentId {
        self.row.agent()
    }
    fn tick(&self) -> u64 {
        self.row.tick()
    }
    fn revision(&self) -> MindRevision {
        self.row.revision()
    }
    fn life_stage(&self) -> LifeStage {
        self.row.life_stage()
    }
    fn get(&mut self, fact: FactId) -> Result<i64, Self::Error> {
        self.reads.push(fact);
        if self.fail_fact == Some(fact) {
            return Err(MindEvalError::InvalidIndex("late provider"));
        }
        self.row.get(fact)
    }
}
fn compare<const STABLE: bool>(
    p: &CompiledMind,
    row: FactRow<'_>,
    running: Option<RunningChoice>,
    fail_fact: Option<FactId>,
) -> Result<Selection, MindEvalError> {
    let eval_key = key(row.tick());
    let mut actual_reader = TraceReader::<STABLE> {
        row,
        reads: Vec::new(),
        fail_fact,
    };
    let mut oracle_reader = TraceReader::<STABLE> {
        row,
        reads: Vec::new(),
        fail_fact,
    };
    let mut actual_scratch = EvalScratch::default();
    let mut oracle_scratch = EvalScratch::default();
    let actual = choose_with_reader(
        p,
        &mut actual_reader,
        running,
        eval_key,
        &mut actual_scratch,
    );
    let expected = retained_vector_oracle(
        p,
        &mut oracle_reader,
        running,
        eval_key,
        &mut oracle_scratch,
    );
    assert_eq!(actual, expected);
    assert_eq!(actual_reader.reads, oracle_reader.reads);
    assert_eq!(actual_scratch.witnesses(), oracle_scratch.witnesses());
    actual
}

#[test]
fn streaming_matches_vector_oracle_for_ties_all_incumbents_and_finished_states() {
    for scores in [
        [0; 6],
        [6, 5, 4, 3, 2, 1],
        [-8, -4, -6, -2, -7, -1],
        [i64::MIN, 0, i64::MAX, 0, -1, 1],
    ] {
        for margin in [0, 3, i64::MAX] {
            let nodes: Vec<_> = scores
                .iter()
                .enumerate()
                .map(|(i, score)| node(&format!("n{i}"), "need", *score, "", margin))
                .collect();
            let p = compile(&policy(&nodes), &schema()).expect("policy");
            let table = facts(&p, 0, 0, 0);
            for old in std::iter::once(None).chain(p.nodes().iter().flat_map(|n| {
                [false, true].map(|finished| {
                    Some(RunningChoice {
                        revision: p.revision(),
                        node: n.id,
                        finished,
                    })
                })
            })) {
                compare::<true>(&p, table.row(7).expect("row"), old, None).expect("choice");
                compare::<false>(&p, table.row(7).expect("row"), old, None).expect("choice");
            }
        }
    }
}

#[test]
fn streaming_retains_incumbent_outside_top3_without_inserting_it_into_top3() {
    let p = compile(
        &policy(&[
            node("a", "need", 10, "", 0),
            node("b", "need", 9, "", 0),
            node("c", "need", 8, "", 0),
            node("d", "need", 7, "", 3),
        ]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, 0, 0);
    let id = p.node_id("d").expect("d");
    let chosen = compare::<true>(
        &p,
        table.row(7).expect("row"),
        Some(RunningChoice {
            revision: p.revision(),
            node: id,
            finished: false,
        }),
        None,
    )
    .expect("choice");
    assert_eq!(chosen.selected.node, id);
    assert!(chosen.retained_by_hysteresis);
    assert!(!chosen.top3.iter().any(|candidate| candidate.node == id));
}

#[test]
fn streaming_still_reads_and_errors_after_top3_is_full() {
    for (constant, weight, fail) in [
        (0, 1000, true),
        (i64::MAX, 1000, false),
        (0, i64::MAX, false),
    ] {
        let p = compile(
            &policy(&[
                node("a", "need", 10, "", 0),
                node("b", "need", 9, "", 0),
                node("c", "need", 8, "", 0),
                node(
                    "late",
                    "need",
                    constant,
                    &format!("(fact: \"need\", weight_permille: {weight})"),
                    0,
                ),
            ]),
            &schema(),
        )
        .expect("policy");
        let table = facts(&p, 0, i64::MAX, 0);
        let fact = p.fact_id("need").expect("need");
        let result = compare::<true>(&p, table.row(7).expect("row"), None, fail.then_some(fact));
        assert_eq!(
            result,
            if fail {
                Err(MindEvalError::InvalidIndex("late provider"))
            } else {
                Err(MindEvalError::ScoreOverflow {
                    node: p.node_id("late").expect("late"),
                    fact,
                })
            }
        );
        compare::<false>(&p, table.row(7).expect("row"), None, fail.then_some(fact))
            .expect_err("same late error");
    }
}

#[test]
fn streaming_preserves_rejected_bands_and_suppresses_lower_band_errors() {
    let rejected = node("rejected", "critical", 0, "", 0).replace(
        "fact: \"available\", op: \"has\"",
        "fact: \"need\", op: \">\", value: Some(90)",
    );
    let p = compile(
        &policy(&[
            rejected,
            node("winner", "need", 2, "", 0),
            node(
                "unreached",
                "work",
                0,
                "(fact: \"other\", weight_permille: 1000)",
                0,
            ),
        ]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, 0, 0);
    let chosen =
        compare::<true>(&p, table.row(7).expect("row"), None, p.fact_id("other")).expect("choice");
    assert_eq!(chosen.selected.node, p.node_id("winner").expect("winner"));
}
