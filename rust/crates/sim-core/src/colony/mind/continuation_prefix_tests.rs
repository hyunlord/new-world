//! Ordered-prefix cache must preserve the original guard's outcome and actual read trace.
use super::*;

fn differential(
    policy: &CompiledMind,
    plan: &ContinuationPlan,
    reader: &mut Reader,
    current: &mut GuardScratch,
    original: &mut reference::GuardScratch,
) -> Result<GuardOutcome, MindEvalError> {
    let mut old = Reader::new(policy, reader.tick);
    old.agent = reader.agent;
    old.stage = reader.stage;
    old.values.clone_from(&reader.values);
    let key = reader.key();
    let running = Some(running(policy, "owner"));
    let got = if policy.nodes.iter().any(|node| node.every_ticks != 1) {
        guard_continuation_algorithm_only(policy, plan, reader, running, key, current)
    } else {
        guard_continuation(policy, plan, reader, running, key, current)
    };
    let expected = reference::guard_continuation(policy, plan, &mut old, running, key, original);
    assert_eq!(
        got, expected,
        "tick {} stage {:?}",
        reader.tick, reader.stage
    );
    assert_eq!(reader.reads, old.reads, "actual provider read order");
    got
}

#[test]
fn prefix_lengths_divergence_and_policy_reset_match_original() {
    let terms = [
        "(fact: \"available\", op: \"has\")",
        "(fact: \"need\", op: \">=\", value: Some(0))",
        "(fact: \"other\", op: \"<=\", value: Some(4))",
        "(fact: \"hearth_lit\", op: \"not\")",
        "(fact: \"cold\", op: \"has\")",
    ];
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for length in 0..=terms.len() {
        let nodes = (0..6)
            .map(|offset| {
                let mut predicates = terms[..length].to_vec();
                predicates.rotate_left(if length == 0 { 0 } else { offset % length });
                custom(
                    &format!("urgent_{offset}"),
                    "critical",
                    &predicates.join(","),
                    "",
                )
            })
            .collect();
        let policy = compile_nodes(nodes);
        let plan = ContinuationPlan::new(&policy);
        for tick in 0..16 {
            for value in [0, 1, 2] {
                let mut reader = Reader::new(&policy, tick);
                for (name, observed) in [
                    ("cold", value),
                    ("need", if tick % 2 == 0 { 1 } else { -1 }),
                ] {
                    if let Some(fact) = policy.fact_id(name) {
                        reader.values[fact.index()] = Ok(GuardRead::Value(observed));
                    }
                }
                let _ = differential(&policy, &plan, &mut reader, &mut scratch, &mut original);
            }
        }
    }
}

#[test]
fn known_later_false_never_skips_earlier_unread_unsupported_error_or_bad_boolean() {
    let policy = compile_nodes(vec![
        custom(
            "cache_false",
            "critical",
            "(fact: \"need\", op: \">\", value: Some(9))",
            "",
        ),
        custom(
            "ordered",
            "critical",
            "(fact: \"cold\", op: \"has\"), (fact: \"need\", op: \">\", value: Some(9))",
            "",
        ),
    ]);
    let plan = ContinuationPlan::new(&policy);
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for observation in [
        Ok(GuardRead::Unsupported),
        Err(MindEvalError::InvalidIndex("first unread failure")),
        Ok(GuardRead::Value(2)),
        Ok(GuardRead::Value(0)),
        Ok(GuardRead::Value(1)),
    ] {
        let mut reader = Reader::new(&policy, 0);
        reader.values[policy.fact_id("cold").unwrap().index()] = observation;
        let _ = differential(&policy, &plan, &mut reader, &mut scratch, &mut original);
        assert_eq!(
            &reader.reads[..2],
            &[
                policy.fact_id("need").unwrap(),
                policy.fact_id("cold").unwrap()
            ]
        );
    }
}

#[test]
fn nested_shared_paths_have_exact_contiguous_failure_destinations() {
    let a = "(fact: \"available\", op: \"has\")";
    let b = "(fact: \"need\", op: \">=\", value: Some(0))";
    let c = "(fact: \"cold\", op: \"has\")";
    let d = "(fact: \"other\", op: \">\", value: Some(5))";
    let policy = compile_nodes(vec![
        custom("long_a", "critical", &format!("{a},{b},{c},{d}"), ""),
        custom("long_b", "critical", &format!("{a},{b},{c}"), ""),
        custom("short", "critical", &format!("{a},{b},{d}"), ""),
        custom("different", "critical", &format!("{b},{a},{c}"), ""),
    ]);
    let plan = ContinuationPlan::new(&policy);
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for (cold, need) in [(0, 0), (1, -1), (1, 0), (0, 0)] {
        let mut reader = Reader::new(&policy, 0);
        reader.set(&policy, "cold", GuardRead::Value(cold));
        reader.set(&policy, "need", GuardRead::Value(need));
        let _ = differential(&policy, &plan, &mut reader, &mut scratch, &mut original);
        for (position, id) in plan.urgent.iter().enumerate() {
            let predicates = &policy.node(*id).unwrap().predicates;
            for (predicate, destination) in plan.next_after_failure[position].iter().enumerate() {
                assert!(*destination > position && *destination <= plan.urgent.len());
                let prefix = &predicates[..=predicate];
                let shares = |id| {
                    let other = &policy.node(id).unwrap().predicates;
                    other.len() >= prefix.len()
                        && prefix
                            .iter()
                            .zip(other)
                            .all(|(a, b)| a.cache_index == b.cache_index)
                };
                assert!(plan.urgent[position + 1..*destination]
                    .iter()
                    .all(|id| shares(*id)));
                if let Some(next) = plan.urgent.get(*destination) {
                    assert!(!shares(*next));
                }
            }
        }
    }
}

#[test]
fn excluded_stage_and_cadence_never_seed_prefix_cache() {
    let predicates = "(fact: \"cold\", op: \"has\"), (fact: \"available\", op: \"has\")";
    let policy = compile_nodes(vec![
        custom(
            "child_finish",
            "leisure",
            predicates,
            "finish: Some(\"completed\"), every_ticks: 3,",
        )
        .replace("[\"adult\", \"child\", \"elder\"]", "[\"child\"]"),
        custom(
            "elder_finish",
            "leisure",
            predicates,
            "finish: Some(\"completed\"), every_ticks: 5,",
        )
        .replace("[\"adult\", \"child\", \"elder\"]", "[\"elder\"]"),
    ]);
    let plan = ContinuationPlan::new(&policy);
    let mut scratch = GuardScratch::default();
    let mut original = reference::GuardScratch::default();
    for tick in 0..40 {
        for stage in [LifeStage::Adult, LifeStage::Child, LifeStage::Elder] {
            let mut reader = Reader::new(&policy, tick);
            reader.stage = stage;
            reader.set(&policy, "cold", GuardRead::Unsupported);
            let _ = differential(&policy, &plan, &mut reader, &mut scratch, &mut original);
            if stage == LifeStage::Adult {
                assert!(!reader.reads.contains(&policy.fact_id("cold").unwrap()));
            }
        }
    }
}
