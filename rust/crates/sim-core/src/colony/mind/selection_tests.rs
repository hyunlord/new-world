//! Selector boundaries: band order, exact hysteresis, witnesses and overflow.

use super::*;

fn run(
    compiled: &CompiledMind,
    table: &PersonFacts,
    running: Option<RunningChoice>,
) -> (Selection, EvalScratch) {
    let mut scratch = EvalScratch::default();
    let selection = choose(
        compiled,
        table.row(7).expect("row"),
        running,
        key(table.tick()),
        &mut scratch,
    )
    .expect("choice");
    (selection, scratch)
}
fn current(policy: &CompiledMind, name: &str) -> RunningChoice {
    RunningChoice {
        revision: policy.revision(),
        node: policy.node_id(name).expect("node"),
        finished: false,
    }
}
#[test]
fn first_eligible_band_preempts_arbitrary_lower_scores_and_authored_ties() {
    let p = compile(
        &policy(&[
            node("first", "critical", -99, "", 0),
            node("second", "critical", -99, "", 0),
            node("work", "work", i64::MAX, "", 0),
        ]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, 0, 0);
    let (choice, _) = run(&p, &table, Some(current(&p, "work")));
    assert_eq!(choice.selected.node, p.node_id("first").expect("first"));
    assert_eq!(choice.top3.len(), 2);
    assert_eq!(choice.transition, Transition::Switch);
}

#[test]
fn hysteresis_is_strict_and_uses_running_margin_with_current_score() {
    let p = compile(
        &policy(&[
            node(
                "challenger",
                "need",
                0,
                "(fact: \"need\", weight_permille: 1000)",
                0,
            ),
            node(
                "running",
                "need",
                0,
                "(fact: \"other\", weight_permille: 1000)",
                10,
            ),
        ]),
        &schema(),
    )
    .expect("policy");
    for (challenger, expected) in [(109, "running"), (110, "running"), (111, "challenger")] {
        let table = facts(&p, 0, challenger, 100);
        let (choice, scratch) = run(&p, &table, Some(current(&p, "running")));
        assert_eq!(choice.selected.node, p.node_id(expected).expect("expected"));
        let incumbent = scratch
            .witnesses()
            .iter()
            .find(|w| w.fact == p.fact_id("other").expect("fact"))
            .expect("witness");
        assert_eq!(incumbent.value, 100);
        assert!(incumbent.has_role(WitnessRole::IncumbentScore));
    }
}

#[test]
fn zero_margin_tie_retains_running_but_no_incumbent_uses_data_order() {
    let p = compile(
        &policy(&[
            node("first", "need", 5, "", 0),
            node("second", "need", 5, "", 0),
        ]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, 0, 0);
    assert_eq!(
        run(&p, &table, None).0.selected.node,
        p.node_id("first").expect("first")
    );
    assert_eq!(
        run(&p, &table, Some(current(&p, "second"))).0.selected.node,
        p.node_id("second").expect("second")
    );
}

#[test]
fn ineligible_or_finished_incumbent_cannot_be_pinned_by_margin() {
    let running = node("running", "need", 100, "", i64::MAX).replace(
        "fact: \"available\", op: \"has\"",
        "fact: \"cold\", op: \"has\"",
    );
    let p = compile(
        &policy(&[running, node("other_node", "need", 200, "", 0)]),
        &schema(),
    )
    .expect("policy");
    let mut table = facts(&p, 0, 0, 0);
    table
        .set(p.fact_id("cold").expect("fact"), 0, 0)
        .expect("not cold");
    assert_eq!(
        run(&p, &table, Some(current(&p, "running")))
            .0
            .selected
            .node,
        p.node_id("other_node").expect("node")
    );
    table
        .set(p.fact_id("cold").expect("fact"), 0, 1)
        .expect("cold");
    let mut finished = current(&p, "running");
    finished.finished = true;
    assert_eq!(
        run(&p, &table, Some(finished)).0.selected.node,
        p.node_id("other_node").expect("node")
    );
}

#[test]
fn retained_fourth_candidate_is_separate_from_true_raw_top_three() {
    let p = compile(
        &policy(&[
            node("a", "need", 40, "", 0),
            node("b", "need", 30, "", 0),
            node("c", "need", 20, "", 0),
            node("d", "need", 10, "", 40),
        ]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, 0, 0);
    let (choice, _) = run(&p, &table, Some(current(&p, "d")));
    assert_eq!(choice.selected.node, p.node_id("d").expect("d"));
    assert!(choice.retained_by_hysteresis);
    assert_eq!(
        choice.top3.iter().map(|c| c.score).collect::<Vec<_>>(),
        vec![40, 30, 20]
    );
}

#[test]
fn score_rounding_is_per_term_toward_zero_and_overflow_is_not_saturated() {
    let p = compile(
        &policy(&[node(
            "a",
            "need",
            0,
            "(fact: \"need\", weight_permille: 500), (fact: \"other\", weight_permille: 500)",
            0,
        )]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, -1, -1);
    assert_eq!(run(&p, &table, None).0.selected.score, 0);
    for (constant, weight, value) in [(0, i64::MAX, i64::MAX), (i64::MAX, 1000, 1)] {
        let p = compile(
            &policy(&[node(
                "a",
                "need",
                constant,
                &format!("(fact: \"need\", weight_permille: {weight})"),
                0,
            )]),
            &schema(),
        )
        .expect("policy");
        let table = facts(&p, 0, value, 0);
        assert!(matches!(
            choose(
                &p,
                table.row(7).expect("row"),
                None,
                key(0),
                &mut EvalScratch::default()
            ),
            Err(MindEvalError::ScoreOverflow { .. })
        ));
    }
}

#[test]
fn cadence_never_skips_running_revalidation_and_stage_is_not_a_magic_fact() {
    let slow = node("slow", "leisure", 10, "", 0)
        .replace(
            "name_key: \"NODE_slow\"",
            "name_key: \"NODE_slow\", every_ticks: 5",
        )
        .replace(
            "fact: \"available\", op: \"has\"",
            "fact: \"cold\", op: \"has\"",
        )
        .replace("[\"adult\", \"child\", \"elder\"]", "[\"adult\"]");
    let p = compile(&policy(&[slow]), &schema()).expect("policy");
    for tick in 0..10 {
        let mut table = facts(&p, tick, 0, 0);
        assert_eq!(
            run(&p, &table, Some(current(&p, "slow"))).0.selected.node,
            p.node_id("slow").expect("slow")
        );
        table
            .set(p.fact_id("cold").expect("cold"), 0, 0)
            .expect("not cold");
        assert_eq!(
            run(&p, &table, Some(current(&p, "slow"))).0.selected.node,
            p.node_id("idle").expect("idle")
        );
        table
            .set(p.fact_id("cold").expect("cold"), 0, 1)
            .expect("cold");
        table.set_life_stage(0, LifeStage::Child).expect("stage");
        assert_eq!(
            run(&p, &table, Some(current(&p, "slow"))).0.selected.node,
            p.node_id("idle").expect("idle")
        );
    }
}

#[test]
fn pure_repeated_selection_matches_and_wrong_revision_or_key_fails() {
    let p = compile(&policy(&[node("a", "need", 1, "", 0)]), &schema()).expect("policy");
    let table = facts(&p, 0, 0, 0);
    let (first, first_scratch) = run(&p, &table, None);
    let (second, second_scratch) = run(&p, &table, None);
    assert_eq!(first, second);
    assert_eq!(first_scratch.witnesses(), second_scratch.witnesses());
    let other = compile(&policy(&[node("b", "need", 1, "", 0)]), &schema()).expect("other");
    assert!(matches!(
        choose(
            &other,
            table.row(7).expect("row"),
            None,
            key(0),
            &mut EvalScratch::default()
        ),
        Err(MindEvalError::PolicyMismatch)
    ));
    assert!(matches!(
        choose(
            &p,
            table.row(7).expect("row"),
            None,
            key(1),
            &mut EvalScratch::default()
        ),
        Err(MindEvalError::KeyMismatch)
    ));
}
