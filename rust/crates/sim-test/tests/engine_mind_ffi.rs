//! The inspector reads retained reasons without advancing or rewriting the world.

use sim_bridge::ffi::collect_agent_mind;
use sim_core::components::Agent;
use sim_systems::runtime::mind::{ColonyMindSystem, MindProgram};

#[test]
fn retained_mind_read_is_repeatable_and_does_not_change_simulation() {
    let mut engine = sim_test::colony_engine(0);
    let entity = engine.world.query::<&Agent>().iter().next().unwrap().0;
    assert!(collect_agent_mind(&engine, entity.to_bits().get())
        .unwrap()
        .is_none());
    let source = include_str!("../../sim-core/data/colony/mind.ron");
    let mut mind = ColonyMindSystem::new(MindProgram::load(source).unwrap());
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .unwrap();
    let before = sim_test::probe::colony::colony_fingerprint(&engine);
    let history = ron::ser::to_string(&engine.resources.mind).unwrap();
    let event_next = engine
        .resources
        .next_event_id
        .load(std::sync::atomic::Ordering::Relaxed);
    let first = collect_agent_mind(&engine, entity.to_bits().get())
        .unwrap()
        .unwrap();
    assert!(!first.facts.is_empty());
    for _ in 0..10 {
        assert_eq!(
            Some(first.clone()),
            collect_agent_mind(&engine, entity.to_bits().get()).unwrap()
        );
    }
    let retained = engine.resources.mind.history.as_ref().unwrap();
    let raw = retained
        .latest(first.agent_id)
        .unwrap()
        .decode_display(retained.dictionary())
        .unwrap();
    assert_eq!(
        first.facts.len(),
        raw.facts
            .iter()
            .filter(|fact| raw.selected.labels.inputs.contains(&fact.fact))
            .count()
    );
    assert_eq!(before, sim_test::probe::colony::colony_fingerprint(&engine));
    assert_eq!(
        history,
        ron::ser::to_string(&engine.resources.mind).unwrap()
    );
    assert_eq!(
        event_next,
        engine
            .resources
            .next_event_id
            .load(std::sync::atomic::Ordering::Relaxed)
    );
}

fn reason_fixture(terms: &str) -> sim_bridge::ffi::AgentMindView {
    use sim_core::colony::mind::{
        choose, compile, BehaviourSpec, EvalKey, EvalScratch, FactKind, FactSpec, LifeStage,
        MindRegistrySchema, PersonFacts,
    };
    use sim_core::colony::mind_records::{
        ChoiceCandidate, ChoiceFact, MindChoiceRecord, MindHistory, PackedMindChoice,
    };
    let schema = MindRegistrySchema {
        facts: ["hunger", "has_job", "memory", "cost"]
            .into_iter()
            .map(|name| FactSpec {
                name: name.into(),
                kind: if name == "has_job" {
                    FactKind::Bool
                } else {
                    FactKind::Signed
                },
            })
            .collect(),
        behaviours: vec![BehaviourSpec {
            name: "rest".into(),
            target_bindings: vec![],
        }],
    };
    let source = format!(
        r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
        (id:"hold",band:"work",behaviour:"rest",when:[(fact:"hunger",op:"<",value:Some(750)),(fact:"has_job",op:"has")],
        score:(constant:0,terms:[{terms}]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"HOLD"),
        (id:"idle",band:"idle",behaviour:"rest",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,
        life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#
    );
    let policy = compile(&source, &schema).unwrap();
    let mut engine = sim_test::colony_engine(0);
    let (entity, agent) = engine
        .world
        .query::<&Agent>()
        .iter()
        .next()
        .map(|(e, a)| (e, a.id))
        .unwrap();
    let node = policy.node_id("hold").unwrap();
    let mut table = PersonFacts::new(&policy, vec![agent], 0).unwrap();
    table.set_life_stage(0, LifeStage::Adult).unwrap();
    for (name, value) in [
        ("hunger", 700),
        ("has_job", 1),
        ("memory", 1501),
        ("cost", 1501),
    ] {
        if let Some(fact) = policy.fact_id(name) {
            table.set(fact, 0, value).unwrap();
        }
    }
    let mut scratch = EvalScratch::default();
    let selection = choose(
        &policy,
        table.row(agent).unwrap(),
        None,
        EvalKey {
            world_seed: 0,
            agent,
            tick: 0,
            replicate: 0,
        },
        &mut scratch,
    )
    .unwrap();
    assert_eq!(selection.selected.node, node);
    let mut facts: Vec<_> = scratch
        .witnesses()
        .iter()
        .map(|witness| ChoiceFact {
            fact: witness.fact,
            value: witness.value,
            cause: None,
        })
        .collect();
    facts.sort_by_key(|fact| fact.fact.index());
    let record = MindChoiceRecord {
        revision: policy.revision(),
        id: 1,
        parent: None,
        agent,
        tick: 0,
        node,
        band: selection.selected.band,
        score: selection.selected.score,
        top3: selection
            .top3
            .iter()
            .map(|candidate| ChoiceCandidate {
                node: candidate.node,
                score: candidate.score,
            })
            .collect(),
        facts,
    };
    let mut history = MindHistory::new(&policy).unwrap();
    history
        .push(&policy, PackedMindChoice::new(&policy, &record).unwrap())
        .unwrap();
    engine.resources.mind.history = Some(history);
    let view = collect_agent_mind(&engine, entity.to_bits().get())
        .unwrap()
        .unwrap();
    let hunger = view
        .facts
        .iter()
        .find(|fact| fact.name == "hunger")
        .unwrap();
    assert_eq!(hunger.reason_kind, "low");
    assert_eq!(hunger.predicates, vec![("<".into(), Some(750))]);
    assert_eq!(
        view.facts.len(),
        policy
            .node(node)
            .unwrap()
            .fact_ids()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    view
}

#[test]
fn positive_job_condition_precedes_low_hunger_guard() {
    // Given a job hold whose hunger predicate is a negative eligibility guard.
    let view = reason_fixture("");
    let first = &view.facts[0];
    // Then the explanation names the positive job condition, not hunger as an unmet need.
    assert_eq!(first.name, "has_job");
    assert_eq!(first.reason_kind, "present");
}

#[test]
fn signed_memory_score_contributions_rank_before_boolean_conditions() {
    // Given distinct signed inputs: trunc(1501*1001/1000) and trunc(1501*-1/1000).
    let view =
        reason_fixture(r#"(fact:"memory",weight_permille:1001),(fact:"cost",weight_permille:-1)"#);
    let first = &view.facts[0];
    // Then the actual positive memory contribution outranks the job condition.
    assert_eq!(first.name, "memory");
    assert_eq!(first.reason_kind, "score");
    assert_eq!(first.score_terms, vec![(1001, 1502)]);
    assert_eq!(first.score_contribution, 1502);
    let cost = view.facts.iter().find(|fact| fact.name == "cost").unwrap();
    assert_eq!(cost.score_terms, vec![(-1, -1)]);
    assert_eq!(cost.score_contribution, -1);
    assert_eq!(cost.reason_kind, "score");
}
