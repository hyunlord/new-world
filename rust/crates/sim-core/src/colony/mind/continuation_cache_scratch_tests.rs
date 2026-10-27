//! Warm replay must not consume or mutate another agent's stale scan scratch.
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
    values: Vec<Result<GuardRead, MindEvalError>>,
    reads: Vec<FactId>,
}
impl Reader {
    fn new(policy: &CompiledMind, agent: u64, tick: u64) -> Self {
        Self {
            revision: policy.revision(),
            agent,
            tick,
            values: policy
                .facts()
                .iter()
                .map(|fact| Ok(GuardRead::Value(i64::from(fact.name == "available"))))
                .collect(),
            reads: Vec::new(),
        }
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
        self.agent
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
fn fixture(extra: usize, empty: bool) -> CompiledMind {
    let mut registry = schema();
    for index in 0..extra {
        registry.facts.push(FactSpec {
            name: format!("aaa_{index}"),
            kind: FactKind::Signed,
        });
    }
    // Compilation drops unused registry facts; score-only references retain slots
    // without adding guard reads or changing the tested successful predicate path.
    let terms = std::iter::once("(fact: \"other\", weight_permille: 0)".to_owned())
        .chain((0..extra).map(|index| format!("(fact: \"aaa_{index}\", weight_permille: 0)")))
        .collect::<Vec<_>>()
        .join(",");
    let custom = |name, band, predicates: &str| {
        node(name, band, 0, &terms, 0).replace("(fact: \"available\", op: \"has\")", predicates)
    };
    let nodes = if empty {
        vec![custom("owner", "need", "")]
    } else {
        vec![
        custom("urgent", "critical", "(fact:\"available\",op:\"has\"),(fact:\"cold\",op:\"has\"),(fact:\"hearth_lit\",op:\"has\")"),
        custom("shared", "critical", "(fact:\"available\",op:\"has\"),(fact:\"cold\",op:\"has\"),(fact:\"need\",op:\">\",value:Some(5))"),
        custom("owner", "need", "(fact:\"need\",op:\">=\",value:Some(0)),(fact:\"available\",op:\"has\")"),
    ]
    };
    compile(&policy(&nodes), &registry).unwrap()
}
fn run(
    policy: &CompiledMind,
    mut reader: Reader,
    cache: &mut GuardCache,
    scratch: &mut GuardScratch,
) -> (Result<GuardOutcome, MindEvalError>, bool) {
    let plan = ContinuationPlan::new(policy);
    let running = Some(RunningChoice {
        revision: policy.revision(),
        node: policy.node_id("owner").unwrap(),
        finished: false,
    });
    let key = EvalKey {
        world_seed: 5,
        agent: reader.agent,
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
    assert_eq!(reader.reads, ordinary.reads);
    assert_eq!(reader.reads, frozen.reads);
    (outcome, hit)
}
fn snapshot(scratch: &GuardScratch) -> (Vec<Option<i64>>, Vec<u8>, Vec<usize>, [usize; 3]) {
    (
        scratch.facts.clone(),
        scratch.predicates.clone(),
        scratch.read_facts.clone(),
        [
            scratch.facts.capacity(),
            scratch.predicates.capacity(),
            scratch.read_facts.capacity(),
        ],
    )
}
fn seed(policy: &CompiledMind, agent: u64, scratch: &mut GuardScratch) -> GuardCache {
    let mut cache = GuardCache::default();
    let (outcome, _) = run(policy, Reader::new(policy, agent, 0), &mut cache, scratch);
    assert!(matches!(outcome, Ok(GuardOutcome::ContinueOwned { .. })));
    cache
}

#[test]
fn exact_hits_preserve_scratch_across_interleaved_agents_and_policy_slot_counts() {
    let small = fixture(0, false);
    let large = fixture(33, false);
    assert!(large.facts().len() > small.facts().len());
    let mut scratch = GuardScratch::default();
    let mut first = seed(&small, 7, &mut scratch);
    let mut second = seed(&large, 15, &mut scratch);
    let stale = snapshot(&scratch);
    assert!(run(&small, Reader::new(&small, 7, 1), &mut first, &mut scratch).1);
    assert_eq!(snapshot(&scratch), stale);
    let mut changed = Reader::new(&large, 15, 1);
    changed.set(&large, "need", Ok(GuardRead::Value(1)));
    assert!(!run(&large, changed, &mut second, &mut scratch).1);
    let stale = snapshot(&scratch);
    assert!(run(&small, Reader::new(&small, 7, 2), &mut first, &mut scratch).1);
    assert_eq!(snapshot(&scratch), stale);
    // Reusing one cache for another agent/program must reset before consuming old slots.
    assert!(
        !run(
            &small,
            Reader::new(&small, 23, 3),
            &mut second,
            &mut scratch
        )
        .1
    );
    assert!(
        run(
            &small,
            Reader::new(&small, 23, 4),
            &mut second,
            &mut scratch
        )
        .1
    );
}

#[test]
fn changed_prefixes_and_errors_match_oracles_with_unrelated_stale_predicates() {
    let policy = fixture(0, false);
    let other = fixture(33, false);
    for name in ["available", "cold", "need"] {
        for value in [
            Ok(GuardRead::Value(-1)),
            Ok(GuardRead::Value(0)),
            Ok(GuardRead::Value(1)),
            Ok(GuardRead::Value(2)),
            Ok(GuardRead::Unsupported),
            Err(MindEvalError::InvalidIndex("reused scratch provider error")),
        ] {
            let mut scratch = GuardScratch::default();
            let mut cache = seed(&policy, 7, &mut scratch);
            let _other_cache = seed(&other, 15, &mut scratch);
            let stale = snapshot(&scratch);
            let mut reader = Reader::new(&policy, 7, 1);
            reader.set(&policy, name, value.clone());
            let (outcome, hit) = run(&policy, reader, &mut cache, &mut scratch);
            if hit || outcome.is_err() || value == Ok(GuardRead::Unsupported) {
                assert_eq!(snapshot(&scratch), stale);
            }
            // An error/full outcome must not leak scratch values or predicates into recovery.
            let _ = run(
                &policy,
                Reader::new(&policy, 7, 2),
                &mut cache,
                &mut scratch,
            );
        }
    }
}

#[test]
fn mismatch_opens_new_suffix_without_prefetch_or_duplicate_reads() {
    let policy = fixture(0, false);
    let other = fixture(33, false);
    for suffix in [
        Ok(GuardRead::Value(0)),
        Ok(GuardRead::Value(1)),
        Ok(GuardRead::Value(2)),
        Ok(GuardRead::Unsupported),
        Err(MindEvalError::InvalidIndex("new suffix error")),
    ] {
        let mut scratch = GuardScratch::default();
        let mut cache = seed(&policy, 7, &mut scratch);
        let _other_cache = seed(&other, 15, &mut scratch);
        let mut reader = Reader::new(&policy, 7, 1);
        reader.set(&policy, "cold", Ok(GuardRead::Value(1)));
        reader.set(&policy, "hearth_lit", suffix);
        assert!(!run(&policy, reader, &mut cache, &mut scratch).1);
        let _ = run(
            &policy,
            Reader::new(&policy, 7, 2),
            &mut cache,
            &mut scratch,
        );
    }
}

#[test]
fn empty_success_preserves_unrelated_scratch_then_cold_scan_clears_it() {
    let empty = fixture(0, true);
    let other = fixture(33, false);
    let mut scratch = GuardScratch::default();
    let mut cache = seed(&empty, 7, &mut scratch);
    let _other_cache = seed(&other, 15, &mut scratch);
    let stale = snapshot(&scratch);
    assert!(run(&empty, Reader::new(&empty, 7, 1), &mut cache, &mut scratch).1);
    assert_eq!(snapshot(&scratch), stale);
    cache.invalidate();
    assert!(!run(&empty, Reader::new(&empty, 7, 2), &mut cache, &mut scratch).1);
    assert!(scratch.read_facts.is_empty());
    assert!(scratch.facts.iter().all(Option::is_none));
    assert!(scratch.predicates.iter().all(|value| *value == 0));
}
