//! Independent choice-coverage observer tests and negative controls.
use super::*;
use sim_core::colony::mind::{compile, BehaviourSpec, FactKind, FactSpec, MindRegistrySchema};
use sim_core::colony::mind_records::{ChoiceCandidate, ChoiceFact, MindChoiceRecord, MindHistory};
use sim_core::MaterialRegistry;

fn fixture(problem: Option<&str>) -> SimResources {
    let source = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
        (id:"urgent",band:"critical",behaviour:"rest",when:[(fact:"value",op:">",value:Some(0))],
         score:(constant:1,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"URGENT"),
        (id:"idle",band:"idle",behaviour:"rest",when:[],score:(constant:0,terms:[(fact:"value",weight_permille:0)]),
         hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let policy = compile(
        source,
        &MindRegistrySchema {
            facts: vec![FactSpec {
                name: "value".into(),
                kind: FactKind::Signed,
            }],
            behaviours: vec![BehaviourSpec {
                name: "rest".into(),
                target_bindings: vec![],
            }],
        },
    )
    .expect("policy");
    let node = policy.node_id("urgent").expect("node");
    let mut resources = SimResources::new(2, 2, MaterialRegistry::new());
    resources.current_tick = 5;
    resources.mind.policy_source = Some(source.into());
    resources.mind.running.insert(
        7,
        RunningChoice {
            revision: policy.revision(),
            node,
            finished: false,
        },
    );
    let mut history = MindHistory::new(&policy).expect("history");
    if problem != Some("missing_record") {
        history
            .push_record(
                &policy,
                &MindChoiceRecord {
                    revision: policy.revision(),
                    id: 1,
                    parent: None,
                    agent: 7,
                    tick: if problem == Some("wrong_tick") { 4 } else { 5 },
                    node,
                    band: policy.node(node).expect("node").band,
                    score: 1,
                    top3: vec![ChoiceCandidate { node, score: 1 }],
                    facts: if problem == Some("empty_facts") {
                        vec![]
                    } else {
                        vec![ChoiceFact {
                            fact: policy.fact_id("value").expect("fact"),
                            value: 1,
                            cause: None,
                        }]
                    },
                },
            )
            .expect("record");
    }
    resources.mind.history = Some(history);
    resources
}
#[test]
fn detects_missing_empty_and_wrong_tick_records_against_independent_runtime_choice() {
    for problem in ["missing_record", "empty_facts", "wrong_tick"] {
        let resources = fixture(Some(problem));
        let mut observer = MindCoverage::default();
        observer.after_tick(&resources);
        let report = observer.report();
        assert_eq!(report["status"], "FAIL");
        assert_eq!(report["expected_records"], 1);
        assert_eq!(report["critical_expected"], 1);
        assert_eq!(report["critical_covered"], 0);
        assert_eq!(report["examples"][0]["problem"], problem);
    }
}
#[test]
fn valid_first_choice_counts_and_unchanged_runtime_does_not_inflate_denominator() {
    let resources = fixture(None);
    let mut observer = MindCoverage::default();
    observer.after_tick(&resources);
    observer.before_tick(&resources);
    observer.after_tick(&resources);
    let report = observer.report();
    assert_eq!(report["status"], "PASS");
    assert_eq!(report["first_choices"], 1);
    assert_eq!(report["expected_records"], 1);
    assert_eq!(report["critical_coverage"], 1.0);
}
#[test]
fn absent_mind_is_not_a_coverage_pass() {
    let resources = SimResources::new(2, 2, MaterialRegistry::new());
    let mut observer = MindCoverage::default();
    observer.before_tick(&resources);
    observer.after_tick(&resources);
    assert!(!observer.active());
    assert_eq!(observer.report()["status"], "N/A");
    assert!(observer.report()["coverage"].is_null());
}
#[test]
fn real_colony_tick_boundaries_have_complete_reason_records() {
    use sim_systems::runtime::mind::{ColonyMindSystem, MindProgram};
    let generated = crate::scenes::colony_scene(42);
    // Keep actual generated ECS/resources but not its intentionally legacy registry.
    let mut engine = sim_engine::SimEngine::new(0, 0, MaterialRegistry::new());
    engine.world = generated.world;
    engine.resources = generated.resources;
    engine.register_system(Box::new(
        sim_systems::runtime::influence::BuildingStampSystem::new(),
    ));
    engine.register_system(Box::new(
        sim_systems::runtime::settlement::SettlementSystem::new(),
    ));
    let source = include_str!("../../sim-core/data/colony/mind.ron");
    engine.register_system(Box::new(ColonyMindSystem::new(
        MindProgram::load(source).expect("real policy"),
    )));
    assert!(!engine.system_names().contains(&"AgentDecisionSystem"));
    assert!(engine.system_names().contains(&"ColonyMindSystem"));
    let mut observer = MindCoverage::default();
    for _ in 0..64 {
        observer.before_tick(&engine.resources);
        engine.tick();
        observer.after_tick(&engine.resources);
    }
    let report = observer.report();
    assert!(report["expected_records"].as_u64().expect("count") > 0);
    assert_eq!(report["status"], "PASS", "{report}");
}

#[test]
fn same_behaviour_restart_is_separate_from_behavior_coverage_denominator() {
    let mut resources = fixture(None);
    resources
        .mind
        .running
        .get_mut(&7)
        .expect("running")
        .finished = true;
    let mut observer = MindCoverage::default();
    observer.before_tick(&resources);
    resources
        .mind
        .running
        .get_mut(&7)
        .expect("running")
        .finished = false;
    observer.after_tick(&resources);
    let report = observer.report();
    assert_eq!(report["same_behaviour_restarts"], 1);
    assert_eq!(report["behaviour_expected"], 0);
    assert_eq!(report["node_expected"], 1);
    assert!(report["coverage"].is_null());
    assert_eq!(report["node_coverage"], 1.0);
    assert!(report["critical_coverage"].is_null());
}

#[test]
fn same_behaviour_critical_node_change_requires_a_record_in_strict_critical_gate() {
    let mut resources = fixture(Some("missing_record"));
    let policy = compile(
        resources.mind.policy_source.as_deref().expect("source"),
        &MindRegistrySchema {
            facts: vec![FactSpec {
                name: "value".into(),
                kind: FactKind::Signed,
            }],
            behaviours: vec![BehaviourSpec {
                name: "rest".into(),
                target_bindings: vec![],
            }],
        },
    )
    .expect("policy");
    let critical = *resources.mind.running.get(&7).expect("running");
    resources.mind.running.get_mut(&7).expect("running").node =
        policy.node_id("idle").expect("idle");
    let mut observer = MindCoverage::default();
    observer.before_tick(&resources);
    resources.mind.running.insert(7, critical);
    observer.after_tick(&resources);
    let report = observer.report();
    assert_eq!(report["status"], "FAIL");
    assert_eq!(report["behaviour_expected"], 0);
    assert_eq!(report["node_only_choices"], 1);
    assert_eq!(report["critical_expected"], 0);
    assert_eq!(report["critical_choice_expected"], 1);
    assert_eq!(report["critical_choice_covered"], 0);
    assert_eq!(report["critical_choice_coverage"], 0.0);
    assert_eq!(report["examples"][0]["problem"], "missing_record");
}
