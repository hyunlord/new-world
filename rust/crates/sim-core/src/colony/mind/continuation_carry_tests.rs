//! Carried truth may bypass only an already validated exact root prefix.
use super::*;
use crate::colony::mind::{
    compile,
    tests::{node, policy, schema},
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
    fn set(&mut self, policy: &CompiledMind, name: &str, value: GuardRead) {
        self.values[policy.fact_id(name).expect("fact").index()] = Ok(value);
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
fn custom(name: &str, band: &str, predicates: &str, extra: &str) -> String {
    node(name, band, 10, "(fact: \"other\", weight_permille: 0)", 0)
        .replace("(fact: \"available\", op: \"has\")", predicates)
        .replace("hysteresis: 0,", &format!("hysteresis: 0, {extra}"))
}
fn compile_nodes(mut nodes: Vec<String>) -> CompiledMind {
    nodes.push(node(
        "owner",
        "need",
        20,
        "(fact: \"other\", weight_permille: 1000)",
        0,
    ));
    compile(&policy(&nodes), &schema()).expect("policy")
}
fn running(policy: &CompiledMind, name: &str) -> RunningChoice {
    RunningChoice {
        revision: policy.revision(),
        node: policy.node_id(name).expect("node"),
        finished: false,
    }
}

#[test]
fn carried_prefix_matches_reference_across_structural_skips_and_first_errors() {
    let a = "(fact: \"available\", op: \"has\")";
    let b = "(fact: \"need\", op: \">=\", value: Some(0))";
    let c = "(fact: \"cold\", op: \"has\")";
    let d = "(fact: \"hearth_lit\", op: \"has\")";
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for tail in [format!("{a},{b},{d}"), a.into(), format!("{b},{a},{d}")] {
        let excluded = custom(
            "excluded",
            "leisure",
            &format!("{a},{b},{d}"),
            "finish: Some(\"completed\"), every_ticks: 3,",
        )
        .replace("[\"adult\", \"child\", \"elder\"]", "[\"adult\"]");
        let policy = compile_nodes(vec![
            custom("first", "critical", &format!("{a},{b},{c}"), ""),
            custom("jumped", "critical", &format!("{a},{b},{c},{d}"), ""),
            excluded,
            custom("boundary", "leisure", &tail, "finish: Some(\"completed\"),"),
        ]);
        let plan = ContinuationPlan::new(&policy);
        for stage in [LifeStage::Adult, LifeStage::Child, LifeStage::Elder] {
            for tick in 0..24 {
                for cold in [0, 1] {
                    for observed in [
                        Ok(GuardRead::Value(0)),
                        Ok(GuardRead::Value(1)),
                        Ok(GuardRead::Value(2)),
                        Ok(GuardRead::Unsupported),
                        Err(MindEvalError::InvalidIndex("first unread failure")),
                    ] {
                        let mut reader = Reader::new(&policy, tick);
                        reader.stage = stage;
                        reader.set(&policy, "cold", GuardRead::Value(cold));
                        reader.values[policy.fact_id("hearth_lit").unwrap().index()] = observed;
                        let mut old = Reader::new(&policy, tick);
                        old.stage = stage;
                        old.values.clone_from(&reader.values);
                        let key = reader.key();
                        let incumbent = Some(running(&policy, "owner"));
                        let got = guard_continuation_algorithm_only(
                            &policy,
                            &plan,
                            &mut reader,
                            incumbent,
                            key,
                            &mut scratch,
                        );
                        let expected = reference::guard_continuation(
                            &policy,
                            &plan,
                            &mut old,
                            incumbent,
                            key,
                            &mut original,
                        );
                        assert_eq!(got, expected, "tick {tick} stage {stage:?}");
                        assert_eq!(reader.reads, old.reads, "actual get/error order");
                    }
                }
            }
        }
    }
}

#[test]
fn transition_metadata_is_exact_lcp_including_end_and_shorter_nodes() {
    let a = "(fact: \"available\", op: \"has\")";
    let b = "(fact: \"need\", op: \">=\", value: Some(0))";
    let c = "(fact: \"cold\", op: \"has\")";
    let policy = compile_nodes(vec![
        custom("first", "critical", &format!("{a},{b},{c}"), ""),
        custom("duplicate", "critical", &format!("{a},{b},{c}"), ""),
        custom("shorter", "critical", &format!("{a},{b}"), ""),
        custom("reordered", "critical", &format!("{b},{a},{c}"), ""),
    ]);
    let plan = ContinuationPlan::new(&policy);
    for index in 0..plan.urgent.len() {
        let lcp = |next: usize| {
            let Some(id) = plan.urgent.get(next) else {
                return 0;
            };
            let left = &policy.node(plan.urgent[index]).unwrap().predicates;
            let right = &policy.node(*id).unwrap().predicates;
            let mut length = 0;
            while length < left.len().min(right.len())
                && left[length].cache_index == right[length].cache_index
            {
                length += 1;
            }
            length
        };
        assert_eq!(plan.adjacent_prefix[index], lcp(index + 1));
        for (predicate, &next) in plan.next_after_failure[index].iter().enumerate() {
            assert_eq!(plan.prefix_after_failure[index][predicate], lcp(next));
        }
    }
}

#[test]
fn incumbent_never_inherits_an_urgent_true_prefix() {
    let policy = compile(
        &policy(&[
            custom(
                "urgent",
                "critical",
                "(fact: \"available\", op: \"has\"), (fact: \"cold\", op: \"has\")",
                "",
            ),
            custom(
                "owner",
                "need",
                "(fact: \"other\", op: \">=\", value: Some(0))",
                "",
            ),
        ]),
        &schema(),
    )
    .unwrap();
    let plan = ContinuationPlan::new(&policy);
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for observed in [
        Ok(GuardRead::Unsupported),
        Err(MindEvalError::InvalidIndex("incumbent input")),
        Ok(GuardRead::Value(-1)),
        Ok(GuardRead::Value(0)),
    ] {
        let mut reader = Reader::new(&policy, 0);
        reader.values[policy.fact_id("other").unwrap().index()] = observed;
        let mut old = Reader::new(&policy, 0);
        old.values.clone_from(&reader.values);
        let key = reader.key();
        let incumbent = Some(running(&policy, "owner"));
        assert_eq!(
            guard_continuation(&policy, &plan, &mut reader, incumbent, key, &mut scratch),
            reference::guard_continuation(&policy, &plan, &mut old, incumbent, key, &mut original)
        );
        assert_eq!(reader.reads, old.reads);
        assert_eq!(reader.reads.last(), policy.fact_id("other").as_ref());
    }
}
