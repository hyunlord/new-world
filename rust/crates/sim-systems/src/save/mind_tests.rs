//! Mind save migration, validation and shared-history round trips.
use super::*;
use crate::runtime::mind::MindProgram;
use sim_core::causal::CausalEvent;
use sim_core::colony::mind::RunningChoice;
use sim_core::colony::mind_records::{
    ChoiceCandidate, ChoiceFact, MindChoiceRecord, MindHistory, PackedMindChoice,
};

const SOURCE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn engine_with_history() -> SimEngine {
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    let program = MindProgram::load(SOURCE).expect("actual registries");
    let policy = &program.policy;
    let node = policy.node_id("idle").expect("node");
    let mut history = MindHistory::new(policy).expect("history");
    for id in 1..=20 {
        let choice = PackedMindChoice::new(
            policy,
            &MindChoiceRecord {
                revision: policy.revision(),
                id,
                parent: None,
                agent: 7,
                tick: id,
                node,
                band: policy.node(node).expect("node").band,
                score: 0,
                top3: vec![ChoiceCandidate { node, score: 0 }],
                facts: vec![ChoiceFact {
                    fact: policy.fact_id("hunger").expect("fact"),
                    value: 0,
                    cause: None,
                }],
            },
        )
        .expect("record");
        history.push(policy, choice.clone()).expect("push");
        engine
            .resources
            .causal_log
            .push(0, CausalEvent::MindChoice { choice });
    }
    engine.resources.mind.policy_source = Some(SOURCE.into());
    engine.resources.mind.running.insert(
        7,
        RunningChoice {
            revision: policy.revision(),
            node,
            finished: false,
        },
    );
    engine.resources.mind.history = Some(history);
    engine
}

#[test]
fn saturated_history_round_trip_relinks_payload_and_preserves_bytes() {
    let mut engine = engine_with_history();
    let bytes = save(&mut engine).expect("save");
    let mut loaded = load_with(&bytes, |_| {}).expect("load");
    assert_eq!(save(&mut loaded).expect("resave"), bytes);
    let history = loaded.resources.mind.history.as_ref().expect("history");
    assert_eq!(history.records(7).expect("person").len(), 16);
    let latest = history.latest(7).expect("latest");
    let CausalEvent::MindChoice { choice } = loaded
        .resources
        .causal_log
        .lookup(latest.id())
        .expect("tile event")
    else {
        panic!("wrong event")
    };
    assert!(choice.shares_payload(latest));
    assert_eq!(
        encode(&CausalEvent::MindChoice {
            choice: choice.clone()
        })
        .expect("event")[0],
        16
    );
}

#[test]
fn saved_policy_and_running_dictionary_mismatches_are_rejected() {
    let mut engine = engine_with_history();
    engine.resources.mind.policy_source = Some(SOURCE.replace("constant:0", "constant:1"));
    assert!(save(&mut engine).is_err());
    engine.resources.mind.policy_source = None;
    assert!(save(&mut engine).is_err());
    let mut engine = engine_with_history();
    let other = MindProgram::load(&SOURCE.replace("constant:0", "constant:1")).expect("other");
    engine
        .resources
        .mind
        .running
        .get_mut(&7)
        .expect("running")
        .revision = other.policy.revision();
    assert!(save(&mut engine).is_err());
}

#[test]
fn format_eleven_migrates_empty_mind_and_registration_sees_saved_rules() {
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    engine.resources.rules.clock_offset_ticks = 123;
    let old = LegacyBodyV11 {
        current_tick: 0,
        world: world::capture(&engine.world).expect("world"),
        systems: vec![],
        steward: Default::default(),
        content_state: Default::default(),
        resources: engine.resources,
    };
    let envelope = Envelope {
        revision: sim_core::colony::content::installed()
            .expect("catalog")
            .revision()
            .to_owned(),
        payload: encode(&old).expect("old body"),
    };
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&11u32.to_le_bytes());
    bytes.extend(encode(&envelope).expect("envelope"));
    let loaded = load_with(&bytes, |engine| {
        assert_eq!(engine.resources.rules.clock_offset_ticks, 123)
    })
    .expect("legacy load");
    assert!(loaded.resources.mind.policy_source.is_none());
    assert!(loaded.resources.mind.running.is_empty());
}

#[test]
fn invalid_saved_policy_is_rejected_before_registration() {
    let mut engine = engine_with_history();
    let bytes = save(&mut engine).expect("valid save");
    let mut envelope: Envelope = decode(read_header(&bytes).expect("header")).expect("envelope");
    let mut body: Body = decode(&envelope.payload).expect("body");
    body.mind.policy_source = Some(SOURCE.replace("hunger", "unknown_fact"));
    envelope.payload = encode(&BodyRef {
        current_tick: body.current_tick,
        resources: &body.resources,
        world: &body.world,
        systems: &body.systems,
        steward: &body.steward,
        content_state: &body.content_state,
        mind: &body.mind,
    })
    .expect("malformed fixture");
    let mut corrupt = MAGIC.to_vec();
    corrupt.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    corrupt.extend(encode(&envelope).expect("envelope"));
    let mut registered = false;
    let result = load_with(&corrupt, |_| registered = true);
    assert!(matches!(result, Err(SaveError::Decode(_))));
    assert!(!registered);
}

#[test]
fn typed_fact_origin_event_appends_wire_discriminant_seventeen() {
    let event = CausalEvent::MindFactOrigin {
        id: 3,
        source: sim_core::colony::mind_records::MindFactSource::Command(0),
        tick: 5,
    };
    assert_eq!(encode(&event).expect("origin")[0], 17);
}

fn provenance_engine() -> SimEngine {
    let mut engine = engine_with_history();
    engine.resources.current_tick = 20;
    engine
        .resources
        .next_event_id
        .store(23, std::sync::atomic::Ordering::Relaxed);
    engine
        .resources
        .colony
        .command_log
        .push(sim_core::colony::CommandRecord {
            tick: 10,
            command: sim_core::colony::Command::SetStewardDefence { on: true },
            result: sim_core::colony::CommandResult::ok(1),
        });
    engine
}

#[test]
fn source_less_order_job_draft_and_bridge_are_rejected_independently() {
    use sim_core::colony::mind_job_origins::{MindDraftOrigin, MindJobOrigin, MindOrderKey};
    use sim_core::colony::mind_records::MindFactSource;
    let source = MindFactSource::Command(0);
    for case in 0..4 {
        let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
        match case {
            0 => {
                engine
                    .resources
                    .mind
                    .order_origins
                    .insert(MindOrderKey::Dress, source);
            }
            1 => {
                engine.resources.mind.job_origins.insert(
                    7,
                    MindJobOrigin {
                        kind: sim_core::colony::JobKind::Chop { tree: (1, 1) },
                        taken_at: 0,
                        source,
                    },
                );
            }
            2 => {
                engine.resources.mind.draft_origins.insert(
                    7,
                    MindDraftOrigin {
                        since: 0,
                        order: sim_core::colony::people::DraftOrder::Hold,
                        source,
                    },
                );
            }
            _ => engine.resources.causal_log.push(
                0,
                CausalEvent::MindFactOrigin {
                    id: 1,
                    source,
                    tick: 0,
                },
            ),
        }
        assert!(
            mind_validation::validate(&mut engine.resources).is_err(),
            "case {case}"
        );
    }
}

#[test]
fn saved_order_source_requires_actual_accepted_nonfuture_command() {
    use sim_core::colony::mind_job_origins::MindOrderKey;
    use sim_core::colony::mind_records::MindFactSource;
    let mut engine = provenance_engine();
    engine
        .resources
        .mind
        .order_origins
        .insert(MindOrderKey::Dress, MindFactSource::Command(0));
    assert!(mind_validation::validate(&mut engine.resources).is_ok());
    engine
        .resources
        .mind
        .order_origins
        .insert(MindOrderKey::Dress, MindFactSource::Command(1));
    assert!(mind_validation::validate(&mut engine.resources).is_err());
    engine
        .resources
        .mind
        .order_origins
        .insert(MindOrderKey::Dress, MindFactSource::Command(0));
    engine.resources.colony.command_log[0].result.accepted = false;
    assert!(mind_validation::validate(&mut engine.resources).is_err());
    engine.resources.colony.command_log[0].result.accepted = true;
    engine.resources.colony.command_log[0].tick = 21;
    assert!(mind_validation::validate(&mut engine.resources).is_err());
}

#[test]
fn bridge_must_follow_its_source_and_match_cached_header() {
    use sim_core::colony::mind_records::{MindFactOriginRecord, MindFactSource};
    let mut engine = provenance_engine();
    let source = MindFactSource::Command(0);
    engine
        .resources
        .mind
        .fact_source_events
        .insert(source, MindFactOriginRecord { id: 21, tick: 9 });
    assert!(mind_validation::validate(&mut engine.resources).is_err());
    engine
        .resources
        .mind
        .fact_source_events
        .insert(source, MindFactOriginRecord { id: 21, tick: 11 });
    engine.resources.causal_log.push(
        0,
        CausalEvent::MindFactOrigin {
            id: 22,
            source,
            tick: 11,
        },
    );
    assert!(mind_validation::validate(&mut engine.resources).is_err());
}

#[test]
fn chronicle_source_requires_real_entry_and_matching_pressure_kind() {
    use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
    use sim_core::colony::mind_job_origins::MindOrderKey;
    use sim_core::colony::mind_records::MindFactSource;
    let mut engine = provenance_engine();
    let entry = engine.resources.chronicle.push_caused(
        ChronicleEntry::new(5, ChronicleKind::PressureBegan).cause(3),
        None,
    );
    let valid = MindFactSource::Chronicle(CauseRef::Pressure {
        kind: 3,
        entry: Some(entry),
    });
    engine
        .resources
        .mind
        .order_origins
        .insert(MindOrderKey::Dress, valid);
    assert!(mind_validation::validate(&mut engine.resources).is_ok());
    engine.resources.mind.order_origins.insert(
        MindOrderKey::Dress,
        MindFactSource::Chronicle(CauseRef::Pressure {
            kind: 4,
            entry: Some(entry),
        }),
    );
    assert!(mind_validation::validate(&mut engine.resources).is_err());
    engine.resources.mind.order_origins.insert(
        MindOrderKey::Dress,
        MindFactSource::Chronicle(CauseRef::Entry(entry + 1)),
    );
    assert!(mind_validation::validate(&mut engine.resources).is_err());
}

#[test]
fn site_origin_entity_bits_survive_save_restore() {
    use sim_core::colony::mind_job_origins::MindOrderKey;
    use sim_core::colony::mind_records::MindFactSource;
    use sim_core::colony::{BlueprintKind, ColonySite};
    let mut engine = provenance_engine();
    let site = ColonySite {
        kind: BlueprintKind::Hut,
        top_left: (1, 1),
        size: (1, 1),
    };
    let entity = engine.world.spawn((site,));
    let key = MindOrderKey::Site {
        entity: entity.to_bits().get(),
        centre: site.centre(),
    };
    engine
        .resources
        .mind
        .order_origins
        .insert(key, MindFactSource::Command(0));
    let bytes = save(&mut engine).expect("save site origin");
    let loaded = load_with(&bytes, |_| {}).expect("reload site origin");
    assert_eq!(
        *loaded
            .world
            .get::<&ColonySite>(entity)
            .expect("same entity identity"),
        site
    );
    assert_eq!(
        loaded.resources.mind.order_origins[&key],
        MindFactSource::Command(0)
    );
}

// Production-switch regressions. These use the real registrars, not mocked lists.
fn switch_fixture(register: fn(&mut SimEngine)) -> SimEngine {
    let generated = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(7),
        sim_core::rules::WorldRules::colony(7),
    );
    let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
    engine.world = generated.world;
    engine.resources = generated.resources;
    engine.resources.mind = Default::default();
    register(&mut engine);
    engine
}

fn switch_v11_bytes(engine: &mut SimEngine) -> Vec<u8> {
    let resources = std::mem::replace(
        &mut engine.resources,
        SimEngine::new(1, 1, MaterialRegistry::new()).resources,
    );
    let mut old = LegacyBodyV11 {
        current_tick: engine.current_tick(),
        world: world::capture(&engine.world).unwrap(),
        systems: engine.system_states().unwrap(),
        steward: Default::default(),
        content_state: Default::default(),
        resources,
    };
    old.steward = std::mem::take(&mut old.resources.steward);
    old.content_state = std::mem::take(&mut old.resources.content_state);
    let envelope = Envelope {
        revision: sim_core::colony::content::installed()
            .unwrap()
            .revision()
            .to_owned(),
        payload: encode(&old).unwrap(),
    };
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&11u32.to_le_bytes());
    bytes.extend(encode(&envelope).unwrap());
    bytes
}

#[test]
fn switch_load_callback_sees_complete_saved_mind_resources() {
    let mut engine = engine_with_history();
    engine.resources.world_seed = 314;
    let bytes = save(&mut engine).unwrap();
    let loaded = load_with(&bytes, |candidate| {
        assert_eq!(candidate.resources.world_seed, 314);
        assert_eq!(
            candidate.resources.mind.policy_source.as_deref(),
            Some(SOURCE)
        );
        assert_eq!(
            candidate
                .resources
                .mind
                .history
                .as_ref()
                .unwrap()
                .records(7)
                .unwrap()
                .len(),
            16
        );
        assert!(candidate.resources.mind.running.contains_key(&7));
    })
    .unwrap();
    assert_eq!(loaded.resources.world_seed, 314);
}

#[test]
fn switch_v11_colony_migrates_only_the_legacy_decision_slot() {
    let mut engine = switch_fixture(crate::register_default_runtime_systems);
    for _ in 0..3 {
        engine.tick();
    }
    let tick = engine.current_tick();
    let bytes = switch_v11_bytes(&mut engine);
    let mut loaded = load(&bytes).unwrap();
    assert_eq!(loaded.current_tick(), tick);
    assert!(loaded.system_names().contains(&"ColonyMindSystem"));
    assert!(!loaded.system_names().contains(&"AgentDecisionSystem"));
    assert!(loaded.resources.mind.policy_source.is_some());
    assert!(loaded.resources.mind.history.is_some());
    loaded.tick();
    assert_eq!(loaded.current_tick(), tick + 1);
}

#[test]
fn switch_v11_rejects_nonempty_or_misnamed_legacy_decision_state() {
    let mut engine = switch_fixture(crate::register_default_runtime_systems);
    let bytes = switch_v11_bytes(&mut engine);
    for wrong_name in [false, true] {
        let mut envelope: Envelope = decode(read_header(&bytes).unwrap()).unwrap();
        let mut body: LegacyBodyV11 = decode(&envelope.payload).unwrap();
        let entry = body
            .systems
            .iter_mut()
            .find(|entry| entry.0 == "AgentDecisionSystem")
            .unwrap();
        if wrong_name {
            entry.0 = "UnknownDecisionSystem".into();
        } else {
            entry.1.push(1);
        }
        envelope.payload = encode(&body).unwrap();
        let mut malformed = MAGIC.to_vec();
        malformed.extend_from_slice(&11u32.to_le_bytes());
        malformed.extend(encode(&envelope).unwrap());
        assert!(matches!(load(&malformed), Err(SaveError::Systems(_))));
    }
}

#[test]
fn switch_v12_public_reload_keeps_custom_policy_and_nonzero_tick_continuation() {
    let mut engine = switch_fixture(|engine| {
        let source = SOURCE.replace("id:\"idle\"", "id:\"custom_idle\"");
        let program = MindProgram::load(&source).unwrap();
        engine.resources.mind.history = Some(MindHistory::new(&program.policy).unwrap());
        engine.resources.mind.policy_source = Some(source);
        crate::register_colony_mind_runtime_systems(engine);
    });
    for _ in 0..4 {
        engine.tick();
    }
    let bytes = save(&mut engine).unwrap();
    let mut loaded = load(&bytes).unwrap();
    assert_eq!(loaded.current_tick(), 4);
    assert_eq!(
        loaded.resources.mind.policy_source,
        engine.resources.mind.policy_source
    );
    for _ in 0..3 {
        engine.tick();
        loaded.tick();
    }
    let policy = MindProgram::load(loaded.resources.mind.policy_source.as_deref().unwrap())
        .unwrap()
        .policy;
    let custom = policy.node_id("custom_idle").unwrap();
    assert!(!loaded.resources.mind.running.is_empty());
    assert!(loaded
        .resources
        .mind
        .running
        .values()
        .all(|choice| choice.node == custom));
    assert_eq!(save(&mut loaded).unwrap(), save(&mut engine).unwrap());
}

#[test]
fn switch_v12_public_reload_preserves_saturated_ring_and_shared_payload() {
    let mut engine = engine_with_history();
    engine.resources.rules = sim_core::rules::WorldRules::colony(7);
    crate::register_colony_mind_runtime_systems(&mut engine);
    let bytes = save(&mut engine).unwrap();
    let mut loaded = load(&bytes).unwrap();
    assert_eq!(save(&mut loaded).unwrap(), bytes);
    let history = loaded.resources.mind.history.as_ref().unwrap();
    assert_eq!(history.records(7).unwrap().len(), 16);
    let latest = history.latest(7).unwrap();
    let CausalEvent::MindChoice { choice } =
        loaded.resources.causal_log.lookup(latest.id()).unwrap()
    else {
        panic!("choice")
    };
    assert!(choice.shares_payload(latest));
}

#[test]
fn switch_baseline_public_reload_keeps_old_registration_and_future_bytes() {
    let mut engine = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::baseline(7),
        sim_core::rules::WorldRules::baseline(7),
    );
    for _ in 0..3 {
        engine.tick();
    }
    let bytes = save(&mut engine).unwrap();
    let mut loaded = load(&bytes).unwrap();
    assert!(loaded.system_names().contains(&"AgentDecisionSystem"));
    assert!(!loaded.system_names().contains(&"ColonyMindSystem"));
    assert!(loaded.resources.mind.policy_source.is_none());
    for _ in 0..3 {
        engine.tick();
        loaded.tick();
    }
    assert_eq!(save(&mut loaded).unwrap(), save(&mut engine).unwrap());
}

#[test]
fn switch_preswitch_v12_colony_with_empty_mind_migrates_legacy_slot() {
    let mut engine = switch_fixture(crate::register_default_runtime_systems);
    for _ in 0..3 {
        engine.tick();
    }
    assert!(engine.resources.mind.policy_source.is_none());
    let bytes = save(&mut engine).unwrap();
    let mut loaded = load(&bytes).unwrap();
    assert_eq!(loaded.current_tick(), 3);
    assert!(loaded.system_names().contains(&"ColonyMindSystem"));
    assert!(!loaded.system_names().contains(&"AgentDecisionSystem"));
    assert!(loaded.resources.mind.history.is_some());
    loaded.tick();
    assert_eq!(loaded.current_tick(), 4);
}
