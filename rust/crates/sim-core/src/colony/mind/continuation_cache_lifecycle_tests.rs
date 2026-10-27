use super::*;

#[test]
fn keys_structural_exits_and_program_instances_never_reuse_stale_success() {
    let policy = fixture();
    let plan = ContinuationPlan::new(&policy);
    for case in 0..12 {
        let mut cache = seed(&policy, &plan);
        let mut reader = Reader::new(&policy, 1);
        let mut prior = Some(running(&policy));
        match case {
            0 => reader.tick = 0,
            1 => reader.tick = 3,
            2 => reader.tick = 7,
            3 => reader.stage = LifeStage::Child,
            4 => reader.stage = LifeStage::Elder,
            5 => reader.agent = 8,
            6 => prior = None,
            7 => prior.as_mut().unwrap().finished = true,
            8 => prior.as_mut().unwrap().node = policy.node_id("urgent").unwrap(),
            9 => prior.as_mut().unwrap().revision = MindRevision(0),
            10 => reader.revision = MindRevision(0),
            11 => cache.invalidate(),
            _ => unreachable!(),
        }
        assert!(!compare(&policy, &plan, &mut reader, prior, &mut cache).1);
    }
    let mut cache = seed(&policy, &plan);
    let independent = fixture();
    assert_eq!(policy.revision(), independent.revision());
    assert!(!plan.is_for(&independent));
    assert!(
        !compare(
            &independent,
            &ContinuationPlan::new(&independent),
            &mut Reader::new(&independent, 1),
            Some(running(&independent)),
            &mut cache
        )
        .1
    );
    let mut cache = seed(&policy, &plan);
    let moved_clone = policy.clone();
    assert!(plan.is_for(&moved_clone));
    assert!(
        compare(
            &moved_clone,
            &plan,
            &mut Reader::new(&moved_clone, 1),
            Some(running(&moved_clone)),
            &mut cache
        )
        .1
    );
    cache.success.as_mut().unwrap().tick = u64::MAX;
    assert!(
        !compare(
            &policy,
            &plan,
            &mut Reader::new(&policy, 0),
            Some(running(&policy)),
            &mut cache
        )
        .1
    );
}

#[test]
fn empty_paths_and_fact_cap_affect_only_cost() {
    for count in [0, 32, 33] {
        let mut registry = schema();
        for index in 0..count {
            registry.facts.push(FactSpec {
                name: format!("input_{index}"),
                kind: FactKind::Signed,
            });
        }
        let predicates = (0..count)
            .map(|index| format!("(fact:\"input_{index}\",op:\">=\",value:Some(0))"))
            .collect::<Vec<_>>()
            .join(",");
        let source = policy(&[node(
            "owner",
            "need",
            0,
            "(fact: \"other\", weight_permille: 0)",
            0,
        )
        .replace("(fact: \"available\", op: \"has\")", &predicates)]);
        let policy = compile(&source, &registry).unwrap();
        let plan = ContinuationPlan::new(&policy);
        let mut cache = seed(&policy, &plan);
        assert_eq!(cache.success.is_some(), count <= MAX_FACTS);
        assert!(cache.facts.capacity() <= MAX_FACTS);
        assert_eq!(
            compare(
                &policy,
                &plan,
                &mut Reader::new(&policy, 1),
                Some(running(&policy)),
                &mut cache
            )
            .1,
            count <= MAX_FACTS
        );
    }
}

#[test]
fn custom_cadence_remains_full_and_never_populates_cache() {
    let source = policy(&[node("owner", "leisure", 0, "", 0)
        .replace("hysteresis: 0,", "hysteresis: 0, every_ticks: 2,")]);
    let policy = compile(&source, &schema()).unwrap();
    let plan = ContinuationPlan::new(&policy);
    let mut cache = GuardCache::default();
    for tick in 0..16 {
        let (outcome, hit) = compare(
            &policy,
            &plan,
            &mut Reader::new(&policy, tick),
            Some(running(&policy)),
            &mut cache,
        );
        assert_eq!(
            outcome,
            Ok(GuardOutcome::FullEvaluation {
                reason: GuardWake::Cadence
            })
        );
        assert!(!hit);
        assert!(cache.success.is_none());
    }
}

#[test]
fn preflight_key_errors_and_invalid_nodes_do_not_read_or_retain_success() {
    let policy = fixture();
    let plan = ContinuationPlan::new(&policy);
    for invalid in 0..3 {
        let mut cache = seed(&policy, &plan);
        let mut reader = Reader::new(&policy, 1);
        let mut key = reader.key();
        let mut prior = running(&policy);
        match invalid {
            0 => key.agent += 1,
            1 => key.tick += 1,
            2 => prior.node = NodeId(usize::MAX),
            _ => unreachable!(),
        }
        let mut original = reader.clone();
        let expected = guard_continuation(
            &policy,
            &plan,
            &mut original,
            Some(prior),
            key,
            &mut GuardScratch::default(),
        );
        let actual = guard_continuation_cached(
            &policy,
            &plan,
            &mut reader,
            Some(prior),
            key,
            &mut GuardScratch::default(),
            &mut cache,
        )
        .map(|(outcome, _)| outcome);
        assert_eq!(actual, expected);
        assert!(actual.is_err());
        assert!(reader.reads.is_empty());
        assert!(cache.success.is_none());
    }
}

#[test]
fn stage_exclusion_and_finish_incumbent_keep_structural_precedence() {
    let owner =
        node("owner", "need", 0, "", 0).replace("[\"adult\", \"child\", \"elder\"]", "[\"adult\"]");
    let finish = node("finish", "leisure", 0, "", 0)
        .replace(
            "hysteresis: 0,",
            "hysteresis: 0, finish: Some(\"completed\"),",
        )
        .replace(
            "(fact: \"available\", op: \"has\")",
            "(fact: \"cold\", op: \"has\")",
        );
    let policy = compile(&policy(&[owner, finish]), &schema()).unwrap();
    let plan = ContinuationPlan::new(&policy);
    for stage in [LifeStage::Child, LifeStage::Elder] {
        let mut cache = seed(&policy, &plan);
        let mut reader = Reader::new(&policy, 1);
        reader.stage = stage;
        let (outcome, hit) = compare(
            &policy,
            &plan,
            &mut reader,
            Some(running(&policy)),
            &mut cache,
        );
        assert_eq!(
            outcome,
            Ok(GuardOutcome::FullEvaluation {
                reason: GuardWake::IncumbentIneligible
            })
        );
        assert!(!hit);
    }
    let mut cache = seed(&policy, &plan);
    let mut prior = running(&policy);
    prior.node = policy.node_id("finish").unwrap();
    let mut reader = Reader::new(&policy, 1);
    let (outcome, hit) = compare(&policy, &plan, &mut reader, Some(prior), &mut cache);
    assert_eq!(
        outcome,
        Ok(GuardOutcome::FullEvaluation {
            reason: GuardWake::FinishIncumbent
        })
    );
    assert!(!hit);
    assert!(reader.reads.is_empty());
}
