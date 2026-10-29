//! Numeric replay may avoid a scan only with unchanged predicate truth and read order.
use super::*;
use crate::colony::mind::{
    compile,
    tests::{node, policy, schema},
};

#[derive(Clone)]
struct Reader {
    revision: MindRevision,
    tick: u64,
    values: Vec<Result<GuardRead, MindEvalError>>,
    reads: Vec<FactId>,
}
impl Reader {
    fn new(policy: &CompiledMind, tick: u64, need: i64) -> Self {
        let mut reader = Self {
            revision: policy.revision(),
            tick,
            values: vec![Ok(GuardRead::Value(0)); policy.facts().len()],
            reads: Vec::new(),
        };
        reader.set(policy, "need", Ok(GuardRead::Value(need)));
        reader
    }
    fn set(&mut self, policy: &CompiledMind, name: &str, value: Result<GuardRead, MindEvalError>) {
        self.values[policy.fact_id(name).unwrap().index()] = value;
    }
}
impl ContinuationReader for Reader {
    type Error = MindEvalError;
    fn revision(&self) -> MindRevision {
        self.revision
    }
    fn agent(&self) -> u64 {
        7
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn life_stage(&self) -> LifeStage {
        LifeStage::Adult
    }
    fn get(&mut self, fact: FactId) -> Result<GuardRead, Self::Error> {
        self.reads.push(fact);
        self.values[fact.index()].clone()
    }
}
fn fixture() -> CompiledMind {
    let custom = |name, band, predicates: &str| {
        node(name, band, 0, "", 0).replace("(fact: \"available\", op: \"has\")", predicates)
    };
    compile(&policy(&[
        custom("urgent", "critical", "(fact:\"need\",op:\">=\",value:Some(0)),(fact:\"cold\",op:\"has\"),(fact:\"hearth_lit\",op:\"has\")"),
        custom("owner", "need", "(fact:\"need\",op:\"<\",value:Some(10))"),
    ]), &schema()).unwrap()
}
fn compare(
    policy: &CompiledMind,
    mut reader: Reader,
    scratch: &mut GuardScratch,
    cache: &mut GuardCache,
) -> (Result<GuardOutcome, MindEvalError>, bool) {
    let plan = ContinuationPlan::new(policy);
    let running = Some(RunningChoice {
        revision: policy.revision(),
        node: policy.node_id("owner").unwrap(),
        finished: false,
    });
    let key = EvalKey {
        world_seed: 5,
        agent: 7,
        tick: reader.tick,
        replicate: 0,
    };
    let mut ordinary = reader.clone();
    let mut frozen = reader.clone();
    let actual =
        guard_continuation_cached(policy, &plan, &mut reader, running, key, scratch, cache);
    let expected = guard_continuation(
        policy,
        &plan,
        &mut ordinary,
        running,
        key,
        &mut GuardScratch::default(),
    );
    let reference = super::super::reference::guard_continuation(
        policy,
        &plan,
        &mut frozen,
        running,
        key,
        &mut super::super::reference::GuardScratch::default(),
    );
    let (outcome, hit) = match actual {
        Ok((outcome, hit)) => (Ok(outcome), hit),
        Err(error) => (Err(error), false),
    };
    assert_eq!(outcome, expected);
    assert_eq!(outcome, reference);
    assert_eq!(reader.reads, ordinary.reads, "stateless ordered reads");
    assert_eq!(reader.reads, frozen.reads, "frozen ordered reads");
    (outcome, hit)
}
fn seed(policy: &CompiledMind, scratch: &mut GuardScratch) -> GuardCache {
    let mut cache = GuardCache::default();
    let (outcome, hit) = compare(policy, Reader::new(policy, 0, 1), scratch, &mut cache);
    assert!(matches!(outcome, Ok(GuardOutcome::ContinueOwned { .. })));
    assert!(!hit);
    cache
}

#[test]
fn equivalent_numeric_replay_hits_without_touching_scratch_and_keeps_current_values() {
    let policy = fixture();
    let mut scratch = GuardScratch::default();
    let mut cache = seed(&policy, &mut scratch);
    let before = (
        scratch.facts.clone(),
        scratch.predicates.clone(),
        scratch.read_facts.clone(),
    );
    let pointer = cache.facts.as_ptr();
    let capacity = cache.facts.capacity();
    for (tick, value) in [(1, 2), (2, 9), (3, 0)] {
        let (outcome, hit) = compare(
            &policy,
            Reader::new(&policy, tick, value),
            &mut scratch,
            &mut cache,
        );
        assert!(matches!(outcome, Ok(GuardOutcome::ContinueOwned { .. })));
        assert!(
            hit,
            "changed need {value} stays in [0, 10); replay should hit"
        );
        assert_eq!(
            cache
                .facts
                .iter()
                .find(|(id, _)| *id == policy.fact_id("need").unwrap())
                .unwrap()
                .1,
            value
        );
        assert_eq!(
            (
                scratch.facts.clone(),
                scratch.predicates.clone(),
                scratch.read_facts.clone()
            ),
            before
        );
        assert_eq!(cache.facts.as_ptr(), pointer);
        assert_eq!(cache.facts.capacity(), capacity);
    }
}

#[test]
fn equivalent_prefix_then_changed_fact_preserves_current_scratch_and_suffix_errors() {
    let policy = fixture();
    for suffix in [
        Ok(GuardRead::Value(0)),
        Ok(GuardRead::Value(1)),
        Ok(GuardRead::Value(2)),
        Ok(GuardRead::Unsupported),
        Err(MindEvalError::InvalidIndex("new suffix failure")),
    ] {
        let mut scratch = GuardScratch::default();
        let mut cache = seed(&policy, &mut scratch);
        let mut reader = Reader::new(&policy, 1, 2);
        reader.set(&policy, "cold", Ok(GuardRead::Value(1)));
        reader.set(&policy, "hearth_lit", suffix);
        let (_, hit) = compare(&policy, reader, &mut scratch, &mut cache);
        assert!(!hit, "changed cold must resume the ordinary scan");
        assert_eq!(
            scratch.facts[policy.fact_id("need").unwrap().index()],
            Some(2)
        );
        assert_eq!(
            scratch.facts[policy.fact_id("cold").unwrap().index()],
            Some(1)
        );
    }
}

#[test]
fn numeric_threshold_crossing_keeps_incumbent_ineligible_fallback() {
    let policy = fixture();
    for value in [10, i64::MAX] {
        let mut scratch = GuardScratch::default();
        let mut cache = seed(&policy, &mut scratch);
        let (outcome, hit) = compare(
            &policy,
            Reader::new(&policy, 1, value),
            &mut scratch,
            &mut cache,
        );
        assert_eq!(
            outcome,
            Ok(GuardOutcome::FullEvaluation {
                reason: GuardWake::IncumbentIneligible
            })
        );
        assert!(!hit);
        assert!(cache.success.is_none());
    }
}
