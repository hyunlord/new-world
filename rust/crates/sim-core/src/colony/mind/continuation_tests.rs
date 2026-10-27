//! Pure continuation admission boundaries; the original selector remains the oracle for predicates.
use super::*;
use crate::colony::mind::{
    choose, compile,
    tests::{node, policy, schema},
    EvalScratch, PersonFacts,
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
fn full(reason: GuardWake) -> GuardOutcome {
    GuardOutcome::FullEvaluation { reason }
}
fn run(
    policy: &CompiledMind,
    reader: &mut Reader,
    old: Option<RunningChoice>,
    scratch: &mut GuardScratch,
) -> Result<GuardOutcome, MindEvalError> {
    let key = reader.key();
    guard_continuation(
        policy,
        &ContinuationPlan::new(policy),
        reader,
        old,
        key,
        scratch,
    )
}

#[test]
fn all_eight_phases_and_large_keys_are_stateless_and_never_read_scores() {
    let p = compile_nodes(vec![]);
    let prior = running(&p, "owner");
    let mut scratch = GuardScratch::default();
    for agent in (0..8).chain([u64::MAX, u64::MAX - 1]) {
        for tick in (0..16).chain([u64::MAX, u64::MAX - 7]) {
            let mut reader = Reader::new(&p, tick);
            reader.agent = agent;
            reader.set(&p, "other", GuardRead::Unsupported);
            let expected = if tick % 8 == agent % 8 {
                full(GuardWake::Cadence)
            } else {
                GuardOutcome::ContinueOwned { node: prior.node }
            };
            assert_eq!(
                run(&p, &mut reader, Some(prior), &mut scratch).unwrap(),
                expected
            );
            assert!(!reader.reads.contains(&p.fact_id("other").unwrap()));
            if tick % 8 == agent % 8 {
                assert!(reader.reads.is_empty());
            }
        }
    }
}

#[test]
fn structural_boundaries_and_identity_errors_precede_provider_reads() {
    let p = compile_nodes(vec![
        custom("urgent", "critical", "", ""),
        custom("finish", "leisure", "", "finish: Some(\"completed\"),"),
    ]);
    let mut reader = Reader::new(&p, 0);
    let mut scratch = GuardScratch::default();
    assert_eq!(
        run(&p, &mut reader, None, &mut scratch).unwrap(),
        full(GuardWake::NoRunning)
    );
    let mut old = running(&p, "owner");
    old.finished = true;
    assert_eq!(
        run(&p, &mut reader, Some(old), &mut scratch).unwrap(),
        full(GuardWake::Finished)
    );
    for (name, reason) in [
        ("urgent", GuardWake::CriticalIncumbent),
        ("finish", GuardWake::FinishIncumbent),
    ] {
        assert_eq!(
            run(&p, &mut reader, Some(running(&p, name)), &mut scratch).unwrap(),
            full(reason)
        );
    }
    old.node = NodeId(usize::MAX);
    assert_eq!(
        run(&p, &mut reader, Some(old), &mut scratch),
        Err(MindEvalError::InvalidIndex("running node"))
    );
    old = running(&p, "owner");
    old.revision = MindRevision(p.revision().code().wrapping_add(1));
    assert_eq!(
        run(&p, &mut reader, Some(old), &mut scratch),
        Err(MindEvalError::PolicyMismatch)
    );
    let mut key = reader.key();
    key.agent += 1;
    assert_eq!(
        guard_continuation(
            &p,
            &ContinuationPlan::new(&p),
            &mut reader,
            None,
            key,
            &mut scratch
        ),
        Err(MindEvalError::KeyMismatch)
    );
    let other = compile_nodes(vec![custom("different", "critical", "", "")]);
    key = reader.key();
    assert_eq!(
        guard_continuation(
            &p,
            &ContinuationPlan::new(&other),
            &mut reader,
            None,
            key,
            &mut scratch
        ),
        Err(MindEvalError::PolicyMismatch)
    );
    reader.revision = other.revision();
    assert_eq!(
        run(&p, &mut reader, None, &mut scratch),
        Err(MindEvalError::PolicyMismatch)
    );
    assert!(reader.reads.is_empty());
}

#[test]
fn custom_critical_and_finish_nodes_wake_at_every_phase_without_scoring() {
    for band in ["critical", "duty", "need", "work", "leisure", "idle"] {
        let finish = if band == "critical" {
            ""
        } else {
            "finish: Some(\"interrupted\"),"
        };
        let p = compile_nodes(vec![custom(
            "custom_interrupt",
            band,
            "(fact: \"need\", op: \">=\", value: Some(750))",
            finish,
        )]);
        for tick in 0..8 {
            let mut reader = Reader::new(&p, tick);
            reader.set(&p, "need", GuardRead::Value(750));
            let expected = if tick == 7 {
                GuardWake::Cadence
            } else {
                GuardWake::UrgentEligible
            };
            assert_eq!(
                run(
                    &p,
                    &mut reader,
                    Some(running(&p, "owner")),
                    &mut GuardScratch::default()
                )
                .unwrap(),
                full(expected)
            );
        }
    }
}

#[test]
fn integer_and_boolean_operators_match_full_selector_at_boundaries() {
    for (op, fact, threshold, values) in [
        ("<", "need", Some(0), vec![i64::MIN, -1, 0, 1, i64::MAX]),
        ("<=", "need", Some(0), vec![-1, 0, 1]),
        (">", "need", Some(0), vec![-1, 0, 1]),
        (">=", "need", Some(0), vec![-1, 0, 1]),
        ("==", "need", Some(0), vec![-1, 0, 1]),
        ("has", "cold", None, vec![0, 1]),
        ("not", "cold", None, vec![0, 1]),
    ] {
        let threshold = threshold
            .map(|value| format!(", value: Some({value})"))
            .unwrap_or_default();
        let predicates = format!("(fact: {fact:?}, op: {op:?}{threshold})");
        let p = compile_nodes(vec![custom("critical", "critical", &predicates, "")]);
        for value in values {
            let mut reader = Reader::new(&p, 0);
            reader.set(&p, fact, GuardRead::Value(value));
            let mut table = PersonFacts::new(&p, vec![7], 0).unwrap();
            table.set_life_stage(0, LifeStage::Adult).unwrap();
            for (index, value) in reader.values.iter().enumerate() {
                let Ok(GuardRead::Value(value)) = value else {
                    panic!("value")
                };
                table.set(FactId(index), 0, *value).unwrap();
            }
            let selected = choose(
                &p,
                table.row(7).unwrap(),
                Some(running(&p, "owner")),
                reader.key(),
                &mut EvalScratch::default(),
            )
            .unwrap();
            let expected = if selected.selected.node == p.node_id("critical").unwrap() {
                full(GuardWake::UrgentEligible)
            } else {
                GuardOutcome::ContinueOwned {
                    node: p.node_id("owner").unwrap(),
                }
            };
            assert_eq!(
                run(
                    &p,
                    &mut reader,
                    Some(running(&p, "owner")),
                    &mut GuardScratch::default()
                )
                .unwrap(),
                expected,
                "{op} {value}"
            );
        }
    }
}

#[test]
fn unsupported_short_circuits_only_when_reached_and_invalid_facts_propagate() {
    let p = compile_nodes(vec![custom(
        "critical",
        "critical",
        "(fact: \"need\", op: \">\", value: Some(0)), (fact: \"cold\", op: \"has\")",
        "",
    )]);
    let mut reader = Reader::new(&p, 0);
    let mut scratch = GuardScratch::default();
    reader.set(&p, "cold", GuardRead::Unsupported);
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch).unwrap(),
        GuardOutcome::ContinueOwned {
            node: p.node_id("owner").unwrap()
        }
    );
    assert!(!reader.reads.contains(&p.fact_id("cold").unwrap()));
    reader.set(&p, "need", GuardRead::Value(1));
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch).unwrap(),
        full(GuardWake::UnsupportedFact)
    );
    reader.set(&p, "cold", GuardRead::Value(2));
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch),
        Err(MindEvalError::InvalidBoolean(p.fact_id("cold").unwrap()))
    );
    let missing = MindEvalError::UnwrittenFact {
        fact: p.fact_id("cold").unwrap(),
        row: 0,
    };
    reader.values[p.fact_id("cold").unwrap().index()] = Err(missing.clone());
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch),
        Err(missing)
    );
    reader.set(&p, "need", GuardRead::Value(0));
    reader.set(&p, "available", GuardRead::Unsupported);
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch).unwrap(),
        full(GuardWake::UnsupportedFact)
    );
    reader.set(&p, "available", GuardRead::Value(0));
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch).unwrap(),
        full(GuardWake::IncumbentIneligible)
    );
}

#[test]
fn stage_and_authored_cadence_filter_urgent_nodes_but_incumbent_is_exempt() {
    let urgent = custom(
        "finish",
        "leisure",
        "(fact: \"cold\", op: \"has\")",
        "finish: Some(\"completed\"), every_ticks: 3,",
    )
    .replace("[\"adult\", \"child\", \"elder\"]", "[\"child\"]");
    let owner = custom(
        "owner",
        "leisure",
        "(fact: \"available\", op: \"has\")",
        "every_ticks: 5,",
    );
    let p = compile(&policy(&[urgent, owner]), &schema()).unwrap();
    let finish = p.node(p.node_id("finish").unwrap()).unwrap();
    for tick in 0..32 {
        if tick % 8 == 7 {
            continue;
        }
        let mut reader = Reader::new(&p, tick);
        reader.set(&p, "cold", GuardRead::Unsupported);
        let key = reader.key();
        assert_eq!(
            guard_continuation_algorithm_only(
                &p,
                &ContinuationPlan::new(&p),
                &mut reader,
                Some(running(&p, "owner")),
                key,
                &mut GuardScratch::default()
            )
            .unwrap(),
            GuardOutcome::ContinueOwned {
                node: p.node_id("owner").unwrap()
            }
        );
        assert!(!reader.reads.contains(&p.fact_id("cold").unwrap()));
        reader.stage = LifeStage::Child;
        reader.set(&p, "cold", GuardRead::Value(1));
        let active = tick % 3 == (finish.random_key % 3 + 7 % 3) % 3;
        let expected = if active {
            full(GuardWake::UrgentEligible)
        } else {
            GuardOutcome::ContinueOwned {
                node: p.node_id("owner").unwrap(),
            }
        };
        assert_eq!(
            guard_continuation_algorithm_only(
                &p,
                &ContinuationPlan::new(&p),
                &mut reader,
                Some(running(&p, "owner")),
                key,
                &mut GuardScratch::default()
            )
            .unwrap(),
            expected
        );
    }
    let adult_only = custom("owner", "leisure", "", "")
        .replace("[\"adult\", \"child\", \"elder\"]", "[\"adult\"]");
    let p = compile(&policy(&[adult_only]), &schema()).unwrap();
    let mut reader = Reader::new(&p, 0);
    reader.stage = LifeStage::Child;
    assert_eq!(
        run(
            &p,
            &mut reader,
            Some(running(&p, "owner")),
            &mut GuardScratch::default()
        )
        .unwrap(),
        full(GuardWake::IncumbentIneligible)
    );
}

#[test]
fn facts_and_shared_predicates_are_memoized_only_within_one_call() {
    let predicates = "(fact: \"need\", op: \">=\", value: Some(10))";
    let p = compile_nodes(vec![
        custom("critical", "critical", predicates, ""),
        custom(
            "finish",
            "leisure",
            predicates,
            "finish: Some(\"completed\"),",
        ),
    ]);
    let mut reader = Reader::new(&p, 0);
    reader.set(&p, "need", GuardRead::Value(5));
    let mut scratch = GuardScratch::default();
    assert!(matches!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch).unwrap(),
        GuardOutcome::ContinueOwned { .. }
    ));
    let need = p.fact_id("need").unwrap();
    assert_eq!(reader.reads.iter().filter(|fact| **fact == need).count(), 1);
    reader.reads.clear();
    reader.set(&p, "need", GuardRead::Value(10));
    assert_eq!(
        run(&p, &mut reader, Some(running(&p, "owner")), &mut scratch).unwrap(),
        full(GuardWake::UrgentEligible)
    );
    assert_eq!(reader.reads, vec![need]);
    let smaller = compile_nodes(vec![]);
    let mut reader = Reader::new(&smaller, 0);
    assert!(matches!(
        run(
            &smaller,
            &mut reader,
            Some(running(&smaller, "owner")),
            &mut scratch
        )
        .unwrap(),
        GuardOutcome::ContinueOwned { .. }
    ));
}

#[test]
fn ordinary_competitor_waits_until_fixed_phase_then_full_selection_is_current() {
    let p = compile_nodes(vec![node("ordinary_competitor", "duty", 100, "", 0)]);
    let prior = running(&p, "owner");
    let mut scratch = GuardScratch::default();
    for tick in 0..=7 {
        let mut reader = Reader::new(&p, tick);
        let result = run(&p, &mut reader, Some(prior), &mut scratch).unwrap();
        if tick < 7 {
            assert_eq!(result, GuardOutcome::ContinueOwned { node: prior.node });
        } else {
            assert_eq!(result, full(GuardWake::Cadence));
            let mut table = PersonFacts::new(&p, vec![7], tick).unwrap();
            table.set_life_stage(0, LifeStage::Adult).unwrap();
            for (index, value) in reader.values.iter().enumerate() {
                let Ok(GuardRead::Value(value)) = value else {
                    panic!("value")
                };
                table.set(FactId(index), 0, *value).unwrap();
            }
            let selected = choose(
                &p,
                table.row(7).unwrap(),
                Some(prior),
                reader.key(),
                &mut EvalScratch::default(),
            )
            .unwrap();
            assert_eq!(
                selected.selected.node,
                p.node_id("ordinary_competitor").unwrap()
            );
        }
    }
}

#[test]
fn compact_cache_matches_original_outcomes_and_ordered_reads() {
    let p = compile_nodes(vec![
        custom(
            "urgent_a",
            "critical",
            "(fact: \"available\", op: \"has\"), (fact: \"other\", op: \">\", value: Some(3))",
            "",
        ),
        custom(
            "urgent_b",
            "critical",
            "(fact: \"other\", op: \">\", value: Some(3)), (fact: \"available\", op: \"not\")",
            "",
        ),
        custom(
            "terminal",
            "leisure",
            "(fact: \"other\", op: \"<\", value: Some(-1))",
            "finish: Some(\"completed\"),",
        ),
    ]);
    let prior = running(&p, "owner");
    let plan = ContinuationPlan::new(&p);
    let mut current = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for tick in 0..16 {
        for available in [0, 1, 2] {
            for other in [
                Ok(GuardRead::Value(-2)),
                Ok(GuardRead::Value(0)),
                Ok(GuardRead::Value(4)),
                Ok(GuardRead::Unsupported),
                Err(MindEvalError::InvalidIndex("injected")),
            ] {
                let mut a = Reader::new(&p, tick);
                a.set(&p, "available", GuardRead::Value(available));
                a.values[p.fact_id("other").unwrap().index()] = other.clone();
                let mut b = Reader::new(&p, tick);
                b.values.clone_from(&a.values);
                let key = a.key();
                let got = guard_continuation(&p, &plan, &mut a, Some(prior), key, &mut current);
                let expected = reference::guard_continuation(
                    &p,
                    &plan,
                    &mut b,
                    Some(prior),
                    key,
                    &mut original,
                );
                assert_eq!(got, expected);
                assert_eq!(a.reads, b.reads);
            }
        }
    }
}

#[path = "continuation_prefix_tests.rs"]
mod prefix_tests;

#[path = "continuation_jump_tests.rs"]
mod jump_tests;
