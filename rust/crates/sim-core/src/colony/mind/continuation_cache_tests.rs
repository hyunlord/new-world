use super::*;
use crate::colony::mind::{
    compile,
    tests::{node, policy, schema},
    FactSpec,
};

#[derive(Clone)]
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
            world_seed: 5,
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
        self.values[fact.index()].clone()
    }
}
fn fixture() -> CompiledMind {
    let custom = |name, band, predicates: &str| {
        node(name, band, 10, "", 0).replace("(fact: \"available\", op: \"has\")", predicates)
    };
    compile(&policy(&[
        custom("urgent", "critical", "(fact:\"available\",op:\"has\"),(fact:\"cold\",op:\"has\"),(fact:\"hearth_lit\",op:\"has\")"),
        custom("jumped", "critical", "(fact:\"available\",op:\"has\"),(fact:\"cold\",op:\"has\"),(fact:\"need\",op:\">\",value:Some(5))"),
        custom("owner", "need", "(fact:\"need\",op:\">=\",value:Some(0)),(fact:\"available\",op:\"has\")"),
    ]), &schema()).unwrap()
}
fn running(policy: &CompiledMind) -> RunningChoice {
    RunningChoice {
        revision: policy.revision(),
        node: policy.node_id("owner").unwrap(),
        finished: false,
    }
}
fn compare(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut Reader,
    prior: Option<RunningChoice>,
    cache: &mut GuardCache,
) -> (Result<GuardOutcome, MindEvalError>, bool) {
    let mut old = reader.clone();
    let mut frozen = reader.clone();
    let key = reader.key();
    let actual = guard_continuation_cached(
        policy,
        plan,
        reader,
        prior,
        key,
        &mut GuardScratch::default(),
        cache,
    );
    let expected = guard_continuation(
        policy,
        plan,
        &mut old,
        prior,
        key,
        &mut GuardScratch::default(),
    );
    let (outcome, hit) = match actual {
        Ok((result, hit)) => (Ok(result), hit),
        Err(error) => (Err(error), false),
    };
    assert_eq!(outcome, expected);
    assert_eq!(reader.reads, old.reads, "stateless unique get/error order");
    if !plan.has_authored_cadence {
        let reference = super::super::reference::guard_continuation(
            policy,
            plan,
            &mut frozen,
            prior,
            key,
            &mut super::super::reference::GuardScratch::default(),
        );
        assert_eq!(outcome, reference);
        assert_eq!(reader.reads, frozen.reads, "frozen unique get/error order");
    }
    (outcome, hit)
}
fn seed(policy: &CompiledMind, plan: &ContinuationPlan) -> GuardCache {
    let mut cache = GuardCache::default();
    let (result, hit) = compare(
        policy,
        plan,
        &mut Reader::new(policy, 0),
        Some(running(policy)),
        &mut cache,
    );
    assert!(matches!(result, Ok(GuardOutcome::ContinueOwned { .. })));
    assert!(!hit);
    cache
}

#[test]
fn warm_path_matches_both_oracles_on_changed_values_and_each_reached_error() {
    let policy = fixture();
    let plan = ContinuationPlan::new(&policy);
    let mut cache = seed(&policy, &plan);
    let allocation = cache.facts.as_ptr();
    let saved = cache.facts.clone();
    assert!(
        compare(
            &policy,
            &plan,
            &mut Reader::new(&policy, 1),
            Some(running(&policy)),
            &mut cache
        )
        .1
    );
    assert_eq!(cache.facts.as_ptr(), allocation);
    assert_eq!(cache.facts, saved);
    for name in ["available", "cold", "need"] {
        for observation in [
            Ok(GuardRead::Value(-1)),
            Ok(GuardRead::Value(0)),
            Ok(GuardRead::Value(1)),
            Ok(GuardRead::Value(2)),
            Ok(GuardRead::Unsupported),
            Err(MindEvalError::InvalidIndex("ordered provider failure")),
        ] {
            let mut cache = seed(&policy, &plan);
            let mut reader = Reader::new(&policy, 1);
            reader.values[policy.fact_id(name).unwrap().index()] = observation;
            let _ = compare(
                &policy,
                &plan,
                &mut reader,
                Some(running(&policy)),
                &mut cache,
            );
            let mut unique = reader.reads.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), reader.reads.len());
        }
    }
    // A changed second fact opens a previously unread suffix; its errors must be observed.
    for observation in [
        Ok(GuardRead::Unsupported),
        Err(MindEvalError::InvalidIndex("new suffix")),
        Ok(GuardRead::Value(2)),
    ] {
        let mut cache = seed(&policy, &plan);
        let mut reader = Reader::new(&policy, 1);
        reader.values[policy.fact_id("cold").unwrap().index()] = Ok(GuardRead::Value(1));
        reader.values[policy.fact_id("hearth_lit").unwrap().index()] = observation;
        assert!(
            !compare(
                &policy,
                &plan,
                &mut reader,
                Some(running(&policy)),
                &mut cache
            )
            .1
        );
        assert_eq!(reader.reads.last(), policy.fact_id("hearth_lit").as_ref());
    }
}

#[path = "continuation_cache_lifecycle_tests.rs"]
mod lifecycle;
