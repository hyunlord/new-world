//! Observer boundary tests independent of production quality arithmetic.
use super::*;

fn candidate() -> CraftEvent {
    CraftEvent {
        seed: 0,
        agent_id: 7,
        executed_tick: 1439,
        workshop_id: 1,
        bill_id: 2,
        recipe_code: 0,
        recipe_identity: "knap_blade".into(),
        catalog_revision: "test".into(),
        output_item_code: 3,
        output_count: 1,
        at: (2, 3),
        job_stage: "working",
        ticks_left: 1,
        skill_code: 0,
        skill_present: true,
        xp_before: 29,
        level_before: 0,
        material_id: Some(1),
        material_hardness: Some(70),
        hardness_percent: Some(70),
        temperament_present: true,
        persistence: Some(599),
        knowledge_enabled: true,
        gate_mask: 1,
        gating_nodes: vec![GatingNode {
            node_id: 0,
            entry_present: true,
            proficiency: 300,
            last_used: Some(0),
            admission_threshold: 200,
        }],
        recipe_quality_add: 0,
        material_term: MaterialTerm::Hardness {
            offset: 50,
            divisor: 5,
        },
        tally_before_count: 2,
        tally_after_count: 2,
        tally_before_quality_sum: 70,
        tally_after_quality_sum: 70,
        completion_count_delta: 0,
        actual_quality: 0,
        alive_after_tick: true,
        arithmetic_manifest_sha256: "a".repeat(64),
    }
}

#[test]
fn completion_survives_death_and_preserves_pre_award_xp_and_knowledge() {
    let before = BTreeMap::from([((7, 0), (2, 70)), ((99, 8), (5, 200))]);
    let after = BTreeMap::from([((7, 0), (3, 115)), ((99, 8), (5, 200))]);
    let inputs = BTreeMap::from([(7, candidate())]);
    let (events, errors) = craft::reconcile(&before, &after, &inputs, &BTreeSet::new(), 1439);
    assert!(errors.is_empty());
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert!(!event.alive_after_tick);
    assert_eq!((event.xp_before, event.level_before), (29, 0));
    assert_eq!(event.gating_nodes[0].proficiency, 300);
    assert_eq!(
        (event.completion_count_delta, event.actual_quality),
        (1, 45)
    );
    assert_eq!(craft::rows(&after).len(), 2);
}

#[test]
fn cancelled_candidate_without_delta_is_not_completion() {
    let before = BTreeMap::from([((7, 0), (2, 70))]);
    let (events, errors) = craft::reconcile(
        &before,
        &before,
        &BTreeMap::from([(7, candidate())]),
        &BTreeSet::new(),
        1439,
    );
    assert!(events.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn missing_removed_regressed_and_multiple_deltas_are_invalid() {
    let before = BTreeMap::from([((7, 0), (2, 70))]);
    for (after, code) in [
        (BTreeMap::new(), "TALLY_REMOVED"),
        (BTreeMap::from([((7, 0), (1, 70))]), "TALLY_REGRESSED"),
        (BTreeMap::from([((7, 0), (4, 100))]), "TALLY_DELTA"),
        (BTreeMap::from([((7, 0), (2, 71))]), "TALLY_DELTA"),
        (
            BTreeMap::from([((7, 0), (3, 100))]),
            "MISSING_CRAFT_SNAPSHOT",
        ),
    ] {
        let (events, errors) =
            craft::reconcile(&before, &after, &BTreeMap::new(), &BTreeSet::new(), 1439);
        assert!(events.is_empty());
        assert_eq!(errors[0].code, code);
    }
}

#[test]
fn simultaneous_crafters_keep_distinct_inputs_and_zero_quality_completions() {
    let mut second = candidate();
    second.agent_id = 8;
    second.bill_id = 3;
    second.xp_before = 630;
    second.level_before = 6;
    let before = BTreeMap::from([((7, 0), (2, 70)), ((8, 0), (2, 70))]);
    let after = BTreeMap::from([((7, 0), (3, 70)), ((8, 0), (3, 125))]);
    let (events, errors) = craft::reconcile(
        &before,
        &after,
        &BTreeMap::from([(7, candidate()), (8, second)]),
        &BTreeSet::from([7, 8]),
        1439,
    );
    assert!(errors.is_empty());
    assert_eq!(
        events
            .iter()
            .map(|e| (e.agent_id, e.bill_id, e.actual_quality))
            .collect::<Vec<_>>(),
        vec![(7, 2, 0), (8, 3, 55)]
    );
}

#[test]
fn exact_day_boundaries_and_digest_syntax() {
    assert_eq!(daily::day_end(1438), None);
    assert_eq!(daily::day_end(1439), Some(1));
    assert_eq!(daily::day_end(17279), Some(12));
    assert_eq!(daily::day_end(51839), Some(36));
    assert_eq!(daily::day_end(69119), Some(48));
    assert_eq!(daily::day_end(u64::MAX), None);
    assert!(manifest::valid_hash(&"a".repeat(64)));
    assert!(!manifest::valid_hash(&"A".repeat(64)));
    assert!(!manifest::valid_hash(&"a".repeat(63)));
    assert_eq!(
        serde_json::to_string(&manifest::Policy::Old3_1).expect("policy"),
        "\"old_3_1\""
    );
}

#[test]
fn b4_arguments_require_explicit_complete_opt_in() {
    for args in [
        vec!["--b4-observe"],
        vec!["--b4-manifest", "missing"],
        vec!["--b4-observe", "--b4-observe"],
    ] {
        assert!(
            super::super::colony::ColonyArgs::parse(args.into_iter().map(String::from)).is_err()
        );
    }
    assert!(super::super::colony::ColonyArgs::parse(Vec::new())
        .expect("ordinary args")
        .b4
        .is_none());
}

#[path = "../../../tests/colony_support/mod.rs"]
mod colony_support;

#[test]
fn actual_paid_crafts_cross_xp_continue_and_survive_post_finish_decay_and_death() {
    use sim_core::colony::{
        knowledge::{self, Knowledge, Source},
        materials::FLINT,
        BillMode, BlueprintKind, ItemKind, ItemSpec, SkillKind, Skills,
    };
    use sim_core::components::{Position, Temperament};
    use sim_core::rules::ColonyRule;
    use sim_engine::RuntimeSystem;
    use sim_systems::runtime::{knowledge::KnowledgeDecaySystem, survival::despawn_agent};

    let mut engine = colony_support::job_world_m2(40, 40);
    let first = colony_support::worker(&mut engine, 3, 3);
    let second = colony_support::worker(&mut engine, 4, 3);
    let first_id = engine.world.get::<&Agent>(first).expect("first").id;
    let second_id = engine.world.get::<&Agent>(second).expect("second").id;
    let first_shop =
        colony_support::build_workshop(&mut engine, BlueprintKind::KnappingSpot, (12, 12), (4, 4));
    let second_shop =
        colony_support::build_workshop(&mut engine, BlueprintKind::KnappingSpot, (20, 12), (4, 4));
    // Advance the actual engine clock so an unused unrelated entry can decay.
    while engine.current_tick() < 1440 {
        engine.tick();
    }
    let mut rule = ColonyRule::m5_5();
    rule.knowledge.as_mut().expect("knowledge rule").unused_days = 0;
    engine.resources.rules.colony = Some(rule);
    let gate = knowledge::gates().recipe(RecipeKind::KnapBlade);
    let other = knowledge::defs()
        .nodes
        .iter()
        .find(|n| gate & knowledge::bit(n.id) == 0)
        .expect("unrelated node")
        .id;
    for entity in [first, second] {
        let mut known = Knowledge::default();
        for node in knowledge::ids_of(gate) {
            known.set(node, 1000, Source::Seeded, 0);
        }
        known.set(other, 1000, Source::Observed, 0);
        let mut skills = Skills::default();
        skills.train(SkillKind::Knap, 29);
        engine
            .world
            .insert(entity, (known, skills, Temperament::from_axes([500; 4])))
            .expect("inputs");
    }
    colony_support::drop_items(
        &mut engine,
        (10, 10),
        ItemSpec::raw(ItemKind::Flint, Some(FLINT)),
        8,
    );
    for workshop in [first_shop, second_shop] {
        assert!(
            sim_systems::commands::submit(
                &mut engine,
                Command::AddBill {
                    workshop,
                    recipe: RecipeKind::KnapBlade,
                    mode: BillMode::Make(3)
                }
            )
            .accepted
        );
    }
    let config = Config {
        manifest: manifest::Manifest {
            schema_version: 1,
            arm: manifest::Arm::B0,
            arithmetic_manifest_sha256: "a".repeat(64),
            catalog_revision: content::installed().expect("catalog").revision().into(),
            policy: manifest::Policy::Old3_1,
        },
        manifest_sha256: "b".repeat(64),
    };
    let mut watch = B4Watch::new(&engine, config, 69120).expect("observer");
    let mut killed = false;
    let mut saw_decay = false;
    let mut saw_paid = BTreeSet::new();
    for _ in 0..5000 {
        watch.before_tick(&engine);
        saw_paid.extend(watch.candidates.keys().copied());
        engine.tick();
        let completed = engine
            .resources
            .colony
            .tallies
            .craft_quality
            .get(&(first_id, 0))
            .is_some_and(|v| v.0 > 0);
        if completed && !killed {
            let before_xp = watch
                .candidates
                .get(&first_id)
                .expect("pre-finish input")
                .xp_before;
            assert_eq!(before_xp, 29);
            assert!(
                engine
                    .world
                    .get::<&Skills>(first)
                    .expect("trained")
                    .level(SkillKind::Knap)
                    > 0
            );
            let prior = engine
                .world
                .get::<&Knowledge>(first)
                .expect("knowledge")
                .proficiency(other);
            KnowledgeDecaySystem.tick(&mut engine.world, &mut engine.resources);
            saw_decay = engine
                .world
                .get::<&Knowledge>(first)
                .expect("knowledge")
                .proficiency(other)
                < prior;
            let pos = *engine.world.get::<&Position>(first).expect("position");
            let tick = engine.resources.current_tick;
            despawn_agent(
                &mut engine.world,
                &mut engine.resources,
                first,
                first_id,
                (pos.x, pos.y),
                sim_core::causal::event::DeathReason::Combat,
                tick,
            );
            killed = true;
        }
        watch.after_tick(&engine);
        assert!(
            watch.report.invalid_reasons.is_empty(),
            "{:?}",
            watch.report.invalid_reasons
        );
        if killed
            && watch
                .report
                .craft_events
                .iter()
                .filter(|event| event.agent_id == second_id)
                .count()
                >= 2
        {
            break;
        }
    }
    assert!(killed && saw_decay);
    assert!(saw_paid.contains(&first_id) && saw_paid.contains(&second_id));
    let dead = watch
        .report
        .craft_events
        .iter()
        .find(|event| event.agent_id == first_id)
        .expect("dead crafter event");
    assert!(!dead.alive_after_tick);
    assert_eq!((dead.xp_before, dead.level_before), (29, 0));
    assert!(dead
        .gating_nodes
        .iter()
        .all(|node| node.proficiency == 1000));
    assert!(engine
        .resources
        .colony
        .tallies
        .craft_quality
        .contains_key(&(first_id, 0)));
    let living: Vec<_> = watch
        .report
        .craft_events
        .iter()
        .filter(|event| event.agent_id == second_id)
        .collect();
    assert!(living.len() >= 2, "continuing bill completed twice");
    assert!(living[1].xp_before > living[0].xp_before);
    let mut cancelled_paid = false;
    for _ in 0..3000 {
        watch.before_tick(&engine);
        let paid = watch.candidates.contains_key(&second_id);
        let event_count = watch.report.craft_events.len();
        let tallies_before = engine.resources.colony.tallies.craft_quality.clone();
        if paid {
            sim_systems::runtime::jobs::cancel::cancel(
                &mut engine.world,
                &mut engine.resources,
                second,
                sim_core::colony::CancelReason::Command,
            );
        }
        engine.tick();
        watch.after_tick(&engine);
        assert!(watch.report.invalid_reasons.is_empty());
        if paid {
            assert_eq!(watch.report.craft_events.len(), event_count);
            assert_eq!(
                engine.resources.colony.tallies.craft_quality,
                tallies_before
            );
            cancelled_paid = true;
            break;
        }
    }
    assert!(cancelled_paid, "real paid job cancelled without completion");
    watch.check_primary(&engine);
    assert!(watch.report.invalid_reasons.is_empty());
}

#[test]
fn trader_policy_metadata_requires_exact_authored_policy() {
    use sim_core::{colony::knowledge::Era, rules::GoodsPolicy};
    let registered = GoodsPolicy::Scarce {
        maximum_practitioners: 2,
        scarce_weight: 30,
        staple_weight: 10,
        rare_weight: 1,
        next_era_cutoff: 750,
        rare_era: Era::Neolithic,
    };
    assert_eq!(
        trade::policy_weights(manifest::Policy::Old3_1, None, (3, 1)),
        Ok((3, 1))
    );
    assert!(trade::policy_weights(manifest::Policy::Old3_1, Some(&registered), (3, 1)).is_err());
    assert!(trade::policy_weights(manifest::Policy::Scarce30_10_1, None, (30, 10)).is_err());
    for legacy in [(30, 10), (3, 2), (2, 1)] {
        assert!(trade::policy_weights(manifest::Policy::Old3_1, None, legacy).is_err());
    }
    // Old-save rule fields do not override installed policy for future caravans.
    assert_eq!(
        trade::policy_weights(manifest::Policy::Scarce30_10_1, Some(&registered), (3, 1)),
        Ok((30, 10))
    );
    for (
        maximum_practitioners,
        scarce_weight,
        staple_weight,
        rare_weight,
        next_era_cutoff,
        rare_era,
    ) in [
        (3, 30, 10, 1, 750, Era::Neolithic),
        (2, 31, 10, 1, 750, Era::Neolithic),
        (2, 30, 11, 1, 750, Era::Neolithic),
        (2, 30, 10, 2, 750, Era::Neolithic),
        (2, 30, 10, 1, 751, Era::Neolithic),
        (2, 30, 10, 1, 750, Era::Paleolithic),
    ] {
        let changed = GoodsPolicy::Scarce {
            maximum_practitioners,
            scarce_weight,
            staple_weight,
            rare_weight,
            next_era_cutoff,
            rare_era,
        };
        assert!(
            trade::policy_weights(manifest::Policy::Scarce30_10_1, Some(&changed), (3, 1)).is_err(),
            "{changed:?}"
        );
    }
}
