use super::*;
use sim_core::colony::mind::{
    compile, BehaviourSpec, CompiledMind, FactKind, FactSpec, MindRegistrySchema, RunningChoice,
};
use sim_core::colony::mind_job_origins::{MindDraftOrigin, MindJobOrigin, MindOrderKey};
use sim_core::colony::mind_records::{
    ChoiceCandidate, ChoiceFact, MindChoiceRecord, MindFactOriginRecord, MindFactSource,
    MindHistory,
};
use sim_core::MaterialRegistry;
use sim_engine::SimEngine;

fn fixture() -> (SimEngine, CompiledMind) {
    let source = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
        (id:"idle",band:"idle",behaviour:"rest",when:[],score:(constant:0,terms:[(fact:"a",weight_permille:1)]),
         hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let policy = compile(
        source,
        &MindRegistrySchema {
            facts: vec![FactSpec {
                name: "a".into(),
                kind: FactKind::Signed,
            }],
            behaviours: vec![BehaviourSpec {
                name: "rest".into(),
                target_bindings: vec![],
            }],
        },
    )
    .expect("policy");
    let mut engine = SimEngine::new(2, 2, MaterialRegistry::new());
    let node = policy.node_id("idle").expect("node");
    engine.resources.mind.policy_source = Some(source.into());
    engine.resources.mind.running.insert(
        7,
        RunningChoice {
            revision: policy.revision(),
            node,
            finished: false,
        },
    );
    let mut history = MindHistory::new(&policy).expect("history");
    history
        .push_record(
            &policy,
            &MindChoiceRecord {
                revision: policy.revision(),
                id: 1,
                parent: None,
                agent: 7,
                tick: 0,
                node,
                band: policy.node(node).expect("node").band,
                score: 3,
                top3: vec![ChoiceCandidate { node, score: 3 }],
                facts: vec![ChoiceFact {
                    fact: policy.fact_id("a").expect("fact"),
                    value: 3,
                    cause: None,
                }],
            },
        )
        .expect("record");
    engine.resources.mind.history = Some(history);
    engine.resources.next_event_id.store(2, Ordering::Relaxed);
    (engine, policy)
}
fn hashes(engine: &SimEngine) -> (String, String) {
    (
        crate::probe::fingerprint(engine),
        crate::probe::colony::colony_fingerprint(engine),
    )
}
#[test]
fn empty_state_preserves_exact_legacy_stream_even_when_event_allocator_differs() {
    let mut engine = SimEngine::new(2, 2, MaterialRegistry::new());
    let before = hashes(&engine);
    let mut hash = 0x12345678;
    append(&mut hash, &engine.resources);
    assert_eq!(hash, 0x12345678);
    engine.resources.mind = MindState::default();
    engine.resources.next_event_id.store(999, Ordering::Relaxed);
    assert_eq!(hashes(&engine), before);
}
#[test]
fn independent_same_policy_state_matches_and_running_source_or_ids_diverge() {
    let (left, _) = fixture();
    let (mut right, _) = fixture();
    assert_eq!(hashes(&left), hashes(&right));
    right
        .resources
        .mind
        .running
        .get_mut(&7)
        .expect("running")
        .finished = true;
    assert_ne!(hashes(&left).0, hashes(&right).0);
    assert_ne!(hashes(&left).1, hashes(&right).1);
    right.resources.mind = left.resources.mind.clone();
    right
        .resources
        .mind
        .policy_source
        .as_mut()
        .expect("source")
        .push(' ');
    assert_ne!(hashes(&left), hashes(&right));
    right.resources.mind = left.resources.mind.clone();
    right.resources.next_event_id.store(3, Ordering::Relaxed);
    assert_ne!(hashes(&left), hashes(&right));
}
#[test]
fn history_and_tuple_keyed_provenance_differences_are_observable() {
    let (left, policy) = fixture();
    let (mut right, _) = fixture();
    right
        .resources
        .mind
        .history
        .as_mut()
        .expect("history")
        .remove(7);
    assert_ne!(hashes(&left), hashes(&right));
    right.resources.mind = left.resources.mind.clone();
    right
        .resources
        .mind
        .fact_origins
        .insert((7, policy.fact_id("a").expect("fact")), 1);
    assert_ne!(hashes(&left), hashes(&right));
    right.resources.mind = left.resources.mind.clone();
    right
        .resources
        .mind
        .order_origins
        .insert(MindOrderKey::Dress, MindFactSource::Command(0));
    assert_ne!(hashes(&left), hashes(&right));
    right.resources.mind = left.resources.mind.clone();
    right.resources.mind.job_origins.insert(
        7,
        MindJobOrigin {
            kind: sim_core::colony::JobKind::Chop { tree: (1, 1) },
            taken_at: 0,
            source: MindFactSource::Command(0),
        },
    );
    assert_ne!(hashes(&left), hashes(&right));
    right.resources.mind = left.resources.mind.clone();
    right.resources.mind.draft_origins.insert(
        7,
        MindDraftOrigin {
            since: 0,
            order: sim_core::colony::people::DraftOrder::Hold,
            source: MindFactSource::Command(0),
        },
    );
    assert_ne!(hashes(&left), hashes(&right));
    right.resources.mind = left.resources.mind.clone();
    right.resources.mind.fact_source_events.insert(
        MindFactSource::Command(0),
        MindFactOriginRecord { id: 0, tick: 0 },
    );
    assert_ne!(hashes(&left), hashes(&right));
}
#[test]
fn map_insertion_order_does_not_change_streamed_fingerprint() {
    let (mut left, policy) = fixture();
    let (mut right, _) = fixture();
    let fact = policy.fact_id("a").expect("fact");
    for agent in [1, 2] {
        left.resources
            .mind
            .fact_origins
            .insert((agent, fact), agent);
    }
    for agent in [2, 1] {
        right
            .resources
            .mind
            .fact_origins
            .insert((agent, fact), agent);
    }
    assert_eq!(hashes(&left), hashes(&right));
}

#[test]
fn complete_record_fact_and_retained_dictionary_changes_are_observable() {
    let (left, policy) = fixture();
    let (mut right, _) = fixture();
    let original = left.resources.mind.history.as_ref().expect("history");
    let mut record = original
        .latest(7)
        .expect("record")
        .decode(&policy, original.dictionary())
        .expect("decode");
    record.facts[0].value = 4;
    let mut history = MindHistory::new(&policy).expect("history");
    history.push_record(&policy, &record).expect("record");
    right.resources.mind.history = Some(history);
    assert_ne!(hashes(&left), hashes(&right));
    let mut wire = serde_json::to_value(original).expect("serialize");
    wire["dictionary"]["nodes"][0]["name_key"] = serde_json::json!("ALTERED");
    right.resources.mind.history = Some(serde_json::from_value(wire).expect("history wire"));
    assert_ne!(hashes(&left), hashes(&right));
}

#[test]
fn actual_same_policy_coordinator_twins_match_each_tick() {
    use sim_systems::runtime::mind::{ColonyMindSystem, MindProgram};
    fn generated_mind_world() -> SimEngine {
        let generated = crate::scenes::colony_scene(42);
        let mut engine = SimEngine::new(0, 0, MaterialRegistry::new());
        engine.world = generated.world;
        engine.resources = generated.resources;
        engine.register_system(Box::new(
            sim_systems::runtime::influence::BuildingStampSystem::new(),
        ));
        engine.register_system(Box::new(
            sim_systems::runtime::settlement::SettlementSystem::new(),
        ));
        engine.register_system(Box::new(ColonyMindSystem::new(
            MindProgram::load(include_str!("../../../sim-core/data/colony/mind.ron"))
                .expect("real policy"),
        )));
        engine
    }
    let mut left = generated_mind_world();
    let mut right = generated_mind_world();
    assert_eq!(hashes(&left), hashes(&right));
    for _ in 0..8 {
        left.tick();
        right.tick();
        assert!(!left.resources.mind.running.is_empty());
        assert_eq!(hashes(&left), hashes(&right));
    }
}
