//! Public custom-cadence fallback prevents phase-disjoint opportunity starvation.
use super::*;
use crate::colony::mind::{
    choose, compile,
    tests::{facts, node, policy, schema},
    EvalScratch,
};

struct Reader {
    revision: MindRevision,
    agent: u64,
    tick: u64,
    stage: LifeStage,
    values: Vec<Result<GuardRead, MindEvalError>>,
    reads: Vec<FactId>,
}
impl Reader {
    fn new(policy: &CompiledMind, tick: u64) -> Self {
        Self {
            revision: policy.revision(),
            agent: 7,
            tick,
            stage: LifeStage::Adult,
            values: policy
                .facts()
                .iter()
                .map(|fact| Ok(GuardRead::Value(i64::from(fact.name == "available"))))
                .collect(),
            reads: Vec::new(),
        }
    }
    fn key(&self) -> EvalKey {
        EvalKey {
            world_seed: 13,
            agent: self.agent,
            tick: self.tick,
            replicate: 0,
        }
    }
}
impl ContinuationReader for Reader {
    type Error = MindEvalError;
    fn revision(&self) -> MindRevision {
        self.revision
    }
    fn agent(&self) -> u64 {
        self.agent
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn life_stage(&self) -> LifeStage {
        self.stage
    }
    fn get(&mut self, fact: FactId) -> Result<GuardRead, Self::Error> {
        self.reads.push(fact);
        self.values
            .get(fact.index())
            .ok_or(MindEvalError::InvalidIndex("guard test fact"))?
            .clone()
    }
}

fn custom_policy(period: u32) -> CompiledMind {
    // Find a genuine disjoint authored phase; do not assume a particular name hash.
    (0..128)
        .find_map(|suffix| {
            let name = format!("scheduled_{suffix}");
            let scheduled = node(&name, "leisure", 10, "", 0)
                .replace("tags: []", &format!("tags: [], every_ticks: {period}"));
            let compiled = compile(&policy(&[scheduled]), &schema()).unwrap();
            let scheduled = compiled.node_id(&name).unwrap();
            (period == 1
                || !compiled
                    .node(scheduled)
                    .unwrap()
                    .random_key
                    .is_multiple_of(u64::from(period)))
            .then_some(compiled)
        })
        .expect("disjoint fixture name")
}

fn owner(policy: &CompiledMind) -> RunningChoice {
    RunningChoice {
        revision: policy.revision(),
        node: policy.node_id("idle").unwrap(),
        finished: false,
    }
}

#[test]
fn phase_disjoint_opportunity_reaches_full_selector_instead_of_starving() {
    for period in [2, 4, 8] {
        let policy = custom_policy(period);
        let plan = ContinuationPlan::new(&policy);
        let scheduled = policy
            .nodes
            .iter()
            .find(|node| node.every_ticks == period)
            .unwrap();
        let target = scheduled.id;
        let phase =
            (scheduled.random_key % u64::from(period) + 7 % u64::from(period)) % u64::from(period);
        let mut visits = 0;
        for tick in 0..32 {
            if tick % u64::from(period) != phase {
                continue;
            }
            assert_ne!(
                tick % 8,
                7,
                "authored admission never intersects generic phase"
            );
            visits += 1;
            let mut reader = Reader::new(&policy, tick);
            let key = reader.key();
            let prior = Some(owner(&policy));
            let mut old = Reader::new(&policy, tick);
            assert_eq!(
                reference::guard_continuation(
                    &policy,
                    &plan,
                    &mut old,
                    prior,
                    key,
                    &mut reference::GuardScratch::default()
                )
                .unwrap(),
                GuardOutcome::ContinueOwned {
                    node: owner(&policy).node
                }
            );
            let outcome = guard_continuation(
                &policy,
                &plan,
                &mut reader,
                prior,
                key,
                &mut GuardScratch::default(),
            )
            .unwrap();
            assert!(
                reader.reads.is_empty(),
                "full selector owns observation validation"
            );
            match outcome {
                GuardOutcome::FullEvaluation {
                    reason: GuardWake::Cadence,
                } => {
                    let table = facts(&policy, tick, 0, 0);
                    let selected = choose(
                        &policy,
                        table.row(7).unwrap(),
                        prior,
                        key,
                        &mut EvalScratch::default(),
                    )
                    .unwrap();
                    assert_eq!(selected.selected.node, target);
                }
                other => panic!("custom cadence did not reach selector: {other:?}"),
            }
        }
        assert!(visits > 0);
    }
}

#[test]
fn unit_policy_keeps_public_guard_and_original_read_order() {
    let policy = custom_policy(1);
    let plan = ContinuationPlan::new(&policy);
    assert!(!plan.has_authored_cadence);
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for tick in 0..32 {
        let mut reader = Reader::new(&policy, tick);
        let mut old = Reader::new(&policy, tick);
        let key = reader.key();
        let prior = Some(owner(&policy));
        assert_eq!(
            guard_continuation(&policy, &plan, &mut reader, prior, key, &mut scratch),
            reference::guard_continuation(&policy, &plan, &mut old, prior, key, &mut original)
        );
        assert_eq!(reader.reads, old.reads);
    }
}

#[test]
fn authored_cadence_preserves_boundary_errors_and_requests_full_without_guard_reads() {
    let policy = custom_policy(2);
    let plan = ContinuationPlan::new(&policy);
    assert!(plan.has_authored_cadence);
    let mut reader = Reader::new(&policy, 0);
    let key = reader.key();
    let mut scratch = GuardScratch::default();
    assert_eq!(
        guard_continuation(&policy, &plan, &mut reader, None, key, &mut scratch),
        Ok(GuardOutcome::FullEvaluation {
            reason: GuardWake::NoRunning
        })
    );
    let mut prior = owner(&policy);
    prior.finished = true;
    assert_eq!(
        guard_continuation(&policy, &plan, &mut reader, Some(prior), key, &mut scratch),
        Ok(GuardOutcome::FullEvaluation {
            reason: GuardWake::Finished
        })
    );
    prior.finished = false;
    reader.agent = 8;
    assert_eq!(
        guard_continuation(&policy, &plan, &mut reader, Some(prior), key, &mut scratch),
        Err(MindEvalError::KeyMismatch)
    );
    reader.agent = 7;
    let other = custom_policy(1);
    let wrong_plan = ContinuationPlan::new(&other);
    assert_eq!(
        guard_continuation(
            &policy,
            &wrong_plan,
            &mut reader,
            Some(prior),
            key,
            &mut scratch
        ),
        Err(MindEvalError::PolicyMismatch)
    );
    for value in [
        Ok(GuardRead::Unsupported),
        Ok(GuardRead::Value(2)),
        Err(MindEvalError::InvalidIndex("full evaluator owns input")),
    ] {
        reader.values[policy.fact_id("available").unwrap().index()] = value;
        assert_eq!(
            guard_continuation(&policy, &plan, &mut reader, Some(prior), key, &mut scratch),
            Ok(GuardOutcome::FullEvaluation {
                reason: GuardWake::Cadence
            })
        );
    }
    assert!(reader.reads.is_empty());
}

#[test]
fn fallback_is_policy_wide_even_when_nonunit_node_excludes_current_stage() {
    let authored = node("child_only", "leisure", 10, "", 0)
        .replace("tags: []", "tags: [], every_ticks: 3")
        .replace("[\"adult\", \"child\", \"elder\"]", "[\"child\"]");
    let policy = compile(&policy(&[authored]), &schema()).unwrap();
    let plan = ContinuationPlan::new(&policy);
    let mut reader = Reader::new(&policy, 0);
    let key = reader.key();
    assert_eq!(
        guard_continuation(
            &policy,
            &plan,
            &mut reader,
            Some(owner(&policy)),
            key,
            &mut GuardScratch::default()
        ),
        Ok(GuardOutcome::FullEvaluation {
            reason: GuardWake::Cadence
        })
    );
    assert!(reader.reads.is_empty());
}
