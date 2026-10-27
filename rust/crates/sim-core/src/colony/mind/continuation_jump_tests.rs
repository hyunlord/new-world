//! A false predicate may skip only consecutive exact root-prefix matches.
use super::*;

fn compare(policy: &CompiledMind, reader: &mut Reader) -> Result<GuardOutcome, MindEvalError> {
    let plan = ContinuationPlan::new(policy);
    let mut original = Reader::new(policy, reader.tick);
    original.stage = reader.stage;
    original.values.clone_from(&reader.values);
    let key = reader.key();
    let old = Some(running(policy, "owner"));
    let evaluate = if policy.nodes.iter().any(|node| node.every_ticks != 1) {
        guard_continuation_algorithm_only
    } else {
        guard_continuation
    };
    let got = evaluate(
        policy,
        &plan,
        reader,
        old,
        key,
        &mut GuardScratch::default(),
    );
    let expected = reference::guard_continuation(
        policy,
        &plan,
        &mut original,
        old,
        key,
        &mut reference::GuardScratch::default(),
    );
    assert_eq!(got, expected);
    assert_eq!(reader.reads, original.reads);
    got
}

const PREFIX: &str = "(fact: \"available\", op: \"has\"), (fact: \"cold\", op: \"has\")";

#[test]
fn jump_stops_at_shorter_root_and_preserves_eligible_boundary() {
    let policy = compile_nodes(vec![
        custom("first", "critical", PREFIX, ""),
        custom(
            "skipped",
            "critical",
            &format!("{PREFIX}, (fact: \"need\", op: \">\", value: Some(0))"),
            "",
        ),
        custom(
            "shorter",
            "critical",
            "(fact: \"available\", op: \"has\")",
            "",
        ),
    ]);
    let plan = ContinuationPlan::new(&policy);
    assert_eq!(plan.next_after_failure[0][1], 2);
    let mut reader = Reader::new(&policy, 0);
    reader.set(&policy, "need", GuardRead::Unsupported);
    assert_eq!(
        compare(&policy, &mut reader),
        Ok(full(GuardWake::UrgentEligible))
    );
    assert_eq!(
        reader.reads,
        vec![
            policy.fact_id("available").unwrap(),
            policy.fact_id("cold").unwrap()
        ]
    );
}

#[test]
fn jump_stops_at_different_earlier_fact_and_preserves_first_failure() {
    let boundary = "(fact: \"hearth_lit\", op: \"has\"), (fact: \"cold\", op: \"has\")";
    let policy = compile_nodes(vec![
        custom("first", "critical", PREFIX, ""),
        custom("skipped", "critical", PREFIX, ""),
        custom("different", "critical", boundary, ""),
        custom("later_same", "critical", PREFIX, ""),
    ]);
    let plan = ContinuationPlan::new(&policy);
    assert_eq!(plan.next_after_failure[0][1], 2);
    for value in [
        Ok(GuardRead::Unsupported),
        Err(MindEvalError::InvalidIndex("boundary failure")),
        Ok(GuardRead::Value(2)),
        Ok(GuardRead::Value(0)),
    ] {
        let mut reader = Reader::new(&policy, 0);
        reader.values[policy.fact_id("hearth_lit").unwrap().index()] = value;
        let _ = compare(&policy, &mut reader);
        assert_eq!(
            reader.reads,
            vec![
                policy.fact_id("available").unwrap(),
                policy.fact_id("cold").unwrap(),
                policy.fact_id("hearth_lit").unwrap()
            ]
        );
    }
}

#[test]
fn stage_and_cadence_ineligibility_do_not_authorize_failure_jump() {
    let first = custom(
        "first",
        "leisure",
        PREFIX,
        "finish: Some(\"completed\"), every_ticks: 3,",
    )
    .replace("[\"adult\", \"child\", \"elder\"]", "[\"adult\"]");
    let second = custom("second", "leisure", PREFIX, "finish: Some(\"completed\"),");
    let policy = compile_nodes(vec![first, second]);
    let plan = ContinuationPlan::new(&policy);
    assert_eq!(plan.next_after_failure[0][1], 2);
    for stage in [LifeStage::Adult, LifeStage::Child] {
        for tick in 0..16 {
            if tick % 8 == 7 {
                continue;
            }
            let mut reader = Reader::new(&policy, tick);
            reader.stage = stage;
            reader.set(&policy, "cold", GuardRead::Value(1));
            assert_eq!(
                compare(&policy, &mut reader),
                Ok(full(GuardWake::UrgentEligible))
            );
            reader.reads.clear();
            reader.set(&policy, "cold", GuardRead::Value(0));
            assert_eq!(
                compare(&policy, &mut reader),
                Ok(GuardOutcome::ContinueOwned {
                    node: policy.node_id("owner").unwrap()
                })
            );
        }
    }
}

#[test]
fn reordered_prefix_never_coalesces_and_unsupported_never_jumps() {
    let policy = compile_nodes(vec![
        custom("first", "critical", PREFIX, ""),
        custom(
            "reordered",
            "critical",
            "(fact: \"cold\", op: \"has\"), (fact: \"available\", op: \"has\")",
            "",
        ),
    ]);
    let plan = ContinuationPlan::new(&policy);
    assert_eq!(plan.next_after_failure[0], vec![1, 1]);
    let mut reader = Reader::new(&policy, 0);
    reader.set(&policy, "cold", GuardRead::Unsupported);
    assert_eq!(
        compare(&policy, &mut reader),
        Ok(full(GuardWake::UnsupportedFact))
    );
}
