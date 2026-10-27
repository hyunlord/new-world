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

#[test]
fn merged_witness_roles_keep_first_read_order_and_exact_values() {
    let rejected = node("rejected", "critical", 0, "", 0).replace(
        "fact: \"available\", op: \"has\"",
        "fact: \"need\", op: \">\", value: Some(90)",
    );
    let running = node(
        "running",
        "need",
        0,
        "(fact: \"need\", weight_permille: 1000)",
        0,
    )
    .replace(
        "when: [(fact: \"available\", op: \"has\")]",
        "when: [(fact: \"available\", op: \"has\"), (fact: \"need\", op: \">\", value: Some(0))]",
    );
    let other = node("other", "need", 100, "", 0).replace(
        "fact: \"available\", op: \"has\"",
        "fact: \"other\", op: \"<\", value: Some(0)",
    );
    let p = compile(&policy(&[rejected, running, other]), &schema()).expect("policy");
    let table = facts(&p, 3, 40, 17);
    let (choice, scratch) = run(&p, &table, Some(current(&p, "running")));
    assert_eq!(choice.selected.node, p.node_id("running").expect("running"));
    assert_eq!(choice.selected.score, 40);
    let observed: Vec<_> = scratch
        .witnesses()
        .iter()
        .map(|w| {
            (
                w.fact,
                w.value,
                [
                    w.has_role(WitnessRole::Predicate),
                    w.has_role(WitnessRole::RejectedPredicate),
                    w.has_role(WitnessRole::Score),
                    w.has_role(WitnessRole::IncumbentPredicate),
                    w.has_role(WitnessRole::IncumbentScore),
                ],
            )
        })
        .collect();
    assert_eq!(
        observed,
        vec![
            (
                p.fact_id("need").expect("need"),
                40,
                [true, true, true, true, true]
            ),
            (
                p.fact_id("available").expect("available"),
                1,
                [true, false, false, true, false]
            ),
            (
                p.fact_id("other").expect("other"),
                17,
                [false, true, false, false, false]
            ),
        ]
    );
}

#[test]
fn unit_period_and_nonunit_period_keep_authored_admission_phase() {
    for period in [1_u64, 2, 5] {
        let scheduled = node("scheduled", "leisure", 10, "", 0).replace(
            "name_key: \"NODE_scheduled\"",
            &format!("name_key: \"NODE_scheduled\", every_ticks: {period}"),
        );
        let p = compile(&policy(&[scheduled]), &schema()).expect("policy");
        let id = p.node_id("scheduled").expect("scheduled");
        let phase = (p.node(id).expect("node").random_key % period + 7 % period) % period;
        for tick in (0..15).chain([u64::MAX]) {
            let table = facts(&p, tick, 0, 0);
            let admitted = tick % period == phase;
            let (choice, _) = run(&p, &table, None);
            assert_eq!(
                choice.selected.node == id,
                admitted,
                "period={period} tick={tick}"
            );
            assert_eq!(
                run(&p, &table, Some(current(&p, "scheduled")))
                    .0
                    .selected
                    .node,
                id
            );
        }
    }
}

#[test]
fn unit_weight_matches_wide_arithmetic_at_signed_boundaries() {
    let p = compile(
        &policy(&[node(
            "unit",
            "need",
            0,
            "(fact: \"need\", weight_permille: 1000)",
            0,
        )]),
        &schema(),
    )
    .expect("policy");
    for value in [
        i64::MIN,
        i64::MIN + 1,
        -1001,
        -1,
        0,
        1,
        1001,
        i64::MAX - 1,
        i64::MAX,
    ] {
        let table = facts(&p, 0, value, 0);
        for running in [None, Some(current(&p, "unit"))] {
            let (selection, scratch) = run(&p, &table, running);
            let expected = i64::try_from(i128::from(value) * 1000 / 1000).unwrap();
            assert_eq!(selection.selected.score, expected);
            assert_eq!(selection.top3[0].score, expected);
            let witnesses = scratch.witnesses();
            assert_eq!(
                witnesses
                    .iter()
                    .map(|w| (w.fact, w.value))
                    .collect::<Vec<_>>(),
                vec![
                    (p.fact_id("available").unwrap(), 1),
                    (p.fact_id("need").unwrap(), value)
                ]
            );
            assert!(witnesses[1].has_role(WitnessRole::Score));
            assert_eq!(
                witnesses[1].has_role(WitnessRole::IncumbentScore),
                running.is_some()
            );
        }
    }
}

#[test]
fn unit_weight_mixed_terms_keep_authored_read_order_and_fractional_truncation() {
    let p = compile(
        &policy(&[node(
            "mixed",
            "need",
            4,
            "(fact: \"other\", weight_permille: 500), (fact: \"need\", weight_permille: 1000)",
            0,
        )]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, -7, -3);
    let (selection, scratch) = run(&p, &table, None);
    assert_eq!(selection.selected.score, -4); // 4 + trunc(-3/2) - 7.
    assert_eq!(
        scratch
            .witnesses()
            .iter()
            .map(|w| (w.fact, w.value))
            .collect::<Vec<_>>(),
        vec![
            (p.fact_id("available").unwrap(), 1),
            (p.fact_id("other").unwrap(), -3),
            (p.fact_id("need").unwrap(), -7)
        ]
    );
}

#[test]
fn unit_weight_total_overflow_retains_exact_error_and_failing_witness() {
    for (constant, value) in [(i64::MAX, 1), (i64::MIN, -1), (1, i64::MAX), (-1, i64::MIN)] {
        let p = compile(
            &policy(&[node(
                "unit",
                "need",
                constant,
                "(fact: \"need\", weight_permille: 1000), (fact: \"other\", weight_permille: 1000)",
                0,
            )]),
            &schema(),
        )
        .expect("policy");
        let table = facts(&p, 0, value, 0);
        let mut scratch = EvalScratch::default();
        assert_eq!(
            choose(&p, table.row(7).unwrap(), None, key(0), &mut scratch),
            Err(MindEvalError::ScoreOverflow {
                node: p.node_id("unit").unwrap(),
                fact: p.fact_id("need").unwrap(),
            })
        );
        assert_eq!(
            scratch
                .witnesses()
                .iter()
                .map(|w| (w.fact, w.value))
                .collect::<Vec<_>>(),
            vec![
                (p.fact_id("available").unwrap(), 1),
                (p.fact_id("need").unwrap(), value)
            ]
        );
        assert!(scratch.witnesses()[1].has_role(WitnessRole::Score));
    }
}

struct CacheTestReader<'a, const STABLE: bool> {
    row: FactRow<'a>,
    reads: usize,
    fail_at: Option<usize>,
    fail_fact: Option<FactId>,
    change_after_first: bool,
}
impl<const STABLE: bool> FactReader for CacheTestReader<'_, STABLE> {
    const STABLE_READS: bool = STABLE;
    type Error = MindEvalError;
    fn agent(&self) -> crate::components::AgentId {
        self.row.agent()
    }
    fn tick(&self) -> u64 {
        self.row.tick()
    }
    fn life_stage(&self) -> LifeStage {
        self.row.life_stage()
    }
    fn revision(&self) -> MindRevision {
        self.row.revision()
    }
    fn get(&mut self, fact: FactId) -> Result<i64, Self::Error> {
        self.reads += 1;
        if self.fail_at == Some(self.reads) || self.fail_fact == Some(fact) {
            return Err(MindEvalError::InvalidIndex("test later read"));
        }
        let value = self.row.get(fact)?;
        Ok(if self.change_after_first && self.reads > 1 {
            0
        } else {
            value
        })
    }
}

fn cache_reader<const STABLE: bool>(row: FactRow<'_>) -> CacheTestReader<'_, STABLE> {
    CacheTestReader {
        row,
        reads: 0,
        fail_at: None,
        fail_fact: None,
        change_after_first: false,
    }
}

#[test]
fn predicate_cache_matches_uncached_roles_order_and_resets_on_every_selection() {
    let rejected = node("rejected", "critical", 0, "", 0).replace(
        "fact: \"available\", op: \"has\"",
        "fact: \"need\", op: \">\", value: Some(90)",
    );
    let first = node(
        "first",
        "need",
        0,
        "(fact: \"need\", weight_permille: 1000)",
        0,
    );
    let second = node(
        "second",
        "need",
        10,
        "(fact: \"other\", weight_permille: 1000)",
        5,
    );
    let rejected_again = rejected.replace("rejected", "rejected_again");
    let p = compile(
        &policy(&[rejected, rejected_again, first, second]),
        &schema(),
    )
    .expect("policy");
    let mut cached_scratch = EvalScratch::default();
    for tick in [0, 0, 1, u64::MAX] {
        for (need, other) in [(0, 50), (95, -4), (40, 30)] {
            let table = facts(&p, tick, need, other);
            for running in [
                None,
                Some(current(&p, "rejected_again")),
                Some(current(&p, "first")),
                Some(current(&p, "second")),
                Some(RunningChoice {
                    finished: true,
                    ..current(&p, "second")
                }),
            ] {
                let mut reference = cache_reader::<false>(table.row(7).expect("row"));
                let mut cached = cache_reader::<true>(table.row(7).expect("row"));
                let mut reference_scratch = EvalScratch::default();
                let expected = choose_with_reader(
                    &p,
                    &mut reference,
                    running,
                    key(tick),
                    &mut reference_scratch,
                );
                let actual =
                    choose_with_reader(&p, &mut cached, running, key(tick), &mut cached_scratch);
                assert_eq!(actual, expected);
                assert_eq!(cached_scratch.witnesses(), reference_scratch.witnesses());
                assert!(cached.reads <= reference.reads);
                if need <= 90 {
                    assert!(cached.reads < reference.reads);
                }
            }
        }
    }
    // Reusing scratch for another person at the same tick must reread every value.
    let mut other_person = PersonFacts::new(&p, vec![8], 0).expect("table");
    other_person
        .set_life_stage(0, LifeStage::Child)
        .expect("stage");
    for (index, spec) in p.facts().iter().enumerate() {
        other_person
            .set(
                p.fact_id(&spec.name).expect("fact"),
                0,
                if spec.kind == FactKind::Bool {
                    1
                } else {
                    index as i64
                },
            )
            .expect("value");
    }
    let other_key = EvalKey { agent: 8, ..key(0) };
    let mut cached = cache_reader::<true>(other_person.row(8).expect("row"));
    let mut reference = cache_reader::<false>(other_person.row(8).expect("row"));
    let mut reference_scratch = EvalScratch::default();
    assert_eq!(
        choose_with_reader(&p, &mut cached, None, other_key, &mut cached_scratch),
        choose_with_reader(&p, &mut reference, None, other_key, &mut reference_scratch)
    );
    assert_eq!(cached_scratch.witnesses(), reference_scratch.witnesses());
    // The same scratch must discard predicate IDs/values when the policy changes.
    let other_policy = compile(
        &policy(&[node("different", "critical", 3, "", 0)]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&other_policy, 0, 0, 0);
    let mut cached = cache_reader::<true>(table.row(7).expect("row"));
    let actual = choose_with_reader(
        &other_policy,
        &mut cached,
        None,
        key(0),
        &mut cached_scratch,
    )
    .expect("choice");
    assert_eq!(actual, run(&other_policy, &table, None).0);
}

#[test]
fn predicate_cache_keeps_generic_later_failures_and_changed_values() {
    let p = compile(
        &policy(&[
            node("first", "need", 1, "", 0),
            node("second", "need", 2, "", 0),
        ]),
        &schema(),
    )
    .expect("policy");
    let table = facts(&p, 0, 0, 0);
    let mut stateful = cache_reader::<false>(table.row(7).expect("row"));
    stateful.fail_at = Some(2);
    assert_eq!(
        choose_with_reader(&p, &mut stateful, None, key(0), &mut EvalScratch::default()),
        Err(MindEvalError::InvalidIndex("test later read"))
    );
    assert_eq!(stateful.reads, 2);
    let mut stateful = cache_reader::<false>(table.row(7).expect("row"));
    stateful.change_after_first = true;
    let actual = choose_with_reader(&p, &mut stateful, None, key(0), &mut EvalScratch::default())
        .expect("choice");
    assert_eq!(actual.selected.node, p.node_id("first").expect("first"));
    assert_eq!(stateful.reads, 2);
}

#[test]
fn predicate_cache_keeps_first_errors_short_circuit_and_overflow_attribution() {
    let skipped = node("skipped", "critical", 0, "", 0).replace(
        "when: [(fact: \"available\", op: \"has\")]",
        "when: [(fact: \"need\", op: \"<\", value: Some(0)), (fact: \"other\", op: \">\", value: Some(0))]",
    );
    let p = compile(
        &policy(&[
            skipped,
            node("first", "need", 1, "", 0),
            node("second", "need", 2, "", 0),
        ]),
        &schema(),
    )
    .expect("policy");
    // Fail if the unneeded second predicate is read at all.
    let complete = facts(&p, 0, 0, 0);
    let mut cached = cache_reader::<true>(complete.row(7).expect("row"));
    cached.fail_fact = p.fact_id("other");
    let selected = choose_with_reader(&p, &mut cached, None, key(0), &mut EvalScratch::default())
        .expect("choice");
    assert_eq!(selected.selected.node, p.node_id("second").expect("second"));
    assert_eq!(cached.reads, 2);
    for fail_at in [1, 2] {
        let mut cached = cache_reader::<true>(complete.row(7).expect("row"));
        cached.fail_at = Some(fail_at);
        assert_eq!(
            choose_with_reader(&p, &mut cached, None, key(0), &mut EvalScratch::default()),
            Err(MindEvalError::InvalidIndex("test later read"))
        );
    }
    for (constant, weight) in [(i64::MAX, 1000_i64), (0, i64::MAX)] {
        let overflowing = node(
            "overflow",
            "need",
            constant,
            &format!("(fact: \"need\", weight_permille: {weight})"),
            0,
        );
        let p = compile(
            &policy(&[node("first", "need", 1, "", 0), overflowing]),
            &schema(),
        )
        .expect("policy");
        let table = facts(&p, 0, i64::MAX, 0);
        let mut cached = cache_reader::<true>(table.row(7).expect("row"));
        let mut uncached = cache_reader::<false>(table.row(7).expect("row"));
        let actual = choose_with_reader(&p, &mut cached, None, key(0), &mut EvalScratch::default());
        assert_eq!(
            actual,
            choose_with_reader(&p, &mut uncached, None, key(0), &mut EvalScratch::default())
        );
        assert_eq!(
            actual,
            Err(MindEvalError::ScoreOverflow {
                node: p.node_id("overflow").expect("node"),
                fact: p.fact_id("need").expect("fact")
            })
        );
    }
}

#[test]
fn predicate_cache_partial_error_does_not_publish_or_retain_stale_entries() {
    let critical = node("critical", "critical", 5, "", 0).replace(
        "fact: \"available\", op: \"has\"",
        "fact: \"need\", op: \">\", value: Some(90)",
    );
    let p = compile(
        &policy(&[
            critical,
            node(
                "need",
                "need",
                1,
                "(fact: \"other\", weight_permille: 1000)",
                0,
            ),
        ]),
        &schema(),
    )
    .expect("policy");
    let first = facts(&p, 0, 40, 0);
    let mut cached = cache_reader::<true>(first.row(7).expect("row"));
    cached.fail_fact = p.fact_id("other");
    let mut scratch = EvalScratch::default();
    assert_eq!(
        choose_with_reader(&p, &mut cached, None, key(0), &mut scratch),
        Err(MindEvalError::InvalidIndex("test later read")),
    );
    assert_eq!(scratch.witnesses().len(), 2);
    // Same person/tick, different evaluation values: cached rejection must clear.
    let second = facts(&p, 0, 95, 0);
    let mut cached = cache_reader::<true>(second.row(7).expect("row"));
    let actual = choose_with_reader(&p, &mut cached, None, key(0), &mut scratch).expect("choice");
    let mut reference = cache_reader::<false>(second.row(7).expect("row"));
    let mut reference_scratch = EvalScratch::default();
    let expected = choose_with_reader(&p, &mut reference, None, key(0), &mut reference_scratch)
        .expect("choice");
    assert_eq!(actual, expected);
    assert_eq!(scratch.witnesses(), reference_scratch.witnesses());
    assert_eq!(
        actual.selected.node,
        p.node_id("critical").expect("critical")
    );
}
