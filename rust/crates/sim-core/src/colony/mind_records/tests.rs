//! Record boundary tests; runtime/UI integration is owned by the mind coordinator.
use super::*;
use crate::causal::CausalEvent;
use crate::colony::mind::{compile, BehaviourSpec, FactKind, FactSpec, MindRegistrySchema};

fn policy() -> CompiledMind {
    let schema = MindRegistrySchema {
        facts: vec![
            FactSpec {
                name: "a".into(),
                kind: FactKind::Signed,
            },
            FactSpec {
                name: "b".into(),
                kind: FactKind::Signed,
            },
        ],
        behaviours: vec![BehaviourSpec {
            name: "rest".into(),
            target_bindings: vec![],
        }],
    };
    compile(
        r#"(
        schema_version:1, bands:["critical","duty","need","work","leisure","idle"],
        nodes:[
            (id:"first",band:"need",behaviour:"rest",when:[],
             score:(constant:0,terms:[(fact:"a",weight_permille:1),(fact:"b",weight_permille:1)]),
             hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"FIRST"),
            (id:"second",band:"need",behaviour:"rest",when:[],score:(constant:0,terms:[(fact:"a",weight_permille:0)]),
             hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"SECOND"),
            (id:"fallback",band:"idle",behaviour:"rest",when:[],score:(constant:0,terms:[(fact:"a",weight_permille:0)]),
             hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")
        ])"#,
        &schema,
    )
    .expect("fixture policy")
}
fn record(policy: &CompiledMind) -> MindChoiceRecord {
    let node = policy.node_id("first").expect("first");
    MindChoiceRecord {
        revision: policy.revision(),
        id: u64::MAX,
        parent: Some(u64::MAX - 1),
        agent: u64::MAX,
        tick: u64::MAX,
        node,
        band: policy.node(node).expect("node").band,
        score: i64::MAX,
        top3: vec![
            ChoiceCandidate {
                node,
                score: i64::MAX,
            },
            ChoiceCandidate {
                node: policy.node_id("second").expect("second"),
                score: i64::MIN,
            },
        ],
        facts: vec![
            ChoiceFact {
                fact: policy.fact_id("a").expect("a"),
                value: i64::MIN,
                cause: Some(u64::MAX - 1),
            },
            ChoiceFact {
                fact: policy.fact_id("b").expect("b"),
                value: i64::MAX,
                cause: None,
            },
        ],
    }
}
#[test]
fn display_decode_retains_labels_extrema_and_provenance_without_compilation() {
    let policy = policy();
    let input = record(&policy);
    let dictionary = MindRecordDictionary::from_policy(&policy).expect("dictionary");
    let packed = PackedMindChoice::new(&policy, &input).expect("pack");
    let display = packed.decode_display(&dictionary).expect("display");
    assert_eq!(
        (display.id, display.agent, display.tick),
        (input.id, input.agent, input.tick)
    );
    assert_eq!(display.parent, input.parent);
    assert_eq!(display.selected.labels.name, "first");
    assert_eq!(display.selected.labels.name_key, "FIRST");
    assert_eq!(display.selected.labels.behaviour, "rest");
    assert_eq!(display.band, "need");
    assert_eq!(display.selected.score, i64::MAX);
    assert_eq!(display.top3[1].score, i64::MIN);
    assert_eq!(display.facts[0].name, "a");
    assert_eq!(display.facts[0].value, i64::MIN);
    assert_eq!(display.facts[0].cause, input.parent);
    let mut changed = serde_json::to_value(&dictionary).expect("serialize");
    changed["revision"] = serde_json::json!(dictionary.revision().wrapping_add(1));
    let changed = serde_json::from_value(changed).expect("dictionary");
    assert!(matches!(
        packed.decode_display(&changed),
        Err(MindRecordError::PolicyMismatch)
    ));
}
#[test]
fn all_integer_extrema_and_missing_causes_round_trip() {
    let policy = policy();
    let dictionary = MindRecordDictionary::from_policy(&policy).expect("dictionary");
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        let mut input = record(&policy);
        input.score = value;
        input.top3[0].score = value;
        let packed = PackedMindChoice::new(&policy, &input).expect("pack");
        assert_eq!(packed.decode(&policy, &dictionary).expect("decode"), input);
        let bytes = serde_json::to_vec(&packed).expect("serde write");
        let restored: PackedMindChoice = serde_json::from_slice(&bytes).expect("serde read");
        assert_eq!(restored, packed);
    }
}
#[test]
fn truncated_overflowing_and_trailing_payloads_are_rejected() {
    let policy = policy();
    let packed = PackedMindChoice::new(&policy, &record(&policy)).expect("pack");
    for length in 0..packed.bytes().len() {
        assert!(
            PackedMindChoice::from_bytes(&packed.bytes()[..length]).is_err(),
            "prefix {length}"
        );
    }
    let mut trailing = packed.bytes().to_vec();
    trailing.push(0);
    assert!(PackedMindChoice::from_bytes(&trailing).is_err());
    let mut overflow = vec![1];
    overflow.extend_from_slice(&[255; 11]);
    assert!(PackedMindChoice::from_bytes(&overflow).is_err());
    assert!(PackedMindChoice::from_bytes(&[1, 128, 0]).is_err());
}
#[test]
fn history_keeps_exactly_sixteen_and_shares_event_payload() {
    let policy = policy();
    let mut history = MindHistory::new(&policy).expect("history");
    let mut input = record(&policy);
    input.parent = None;
    input.facts[0].cause = None;
    for id in 1..=20 {
        input.id = id;
        input.tick = id;
        let packed = PackedMindChoice::new(&policy, &input).expect("pack");
        let event = CausalEvent::MindChoice {
            choice: packed.clone(),
        };
        history.push(&policy, packed.clone()).expect("push");
        assert!(history
            .latest(input.agent)
            .expect("latest")
            .shares_payload(&packed));
        assert_eq!(event.id(), id);
        assert_eq!(event.tick(), id);
        assert_eq!(event.parent(), None);
        assert_eq!(event.channel(), None);
    }
    let records = history.records(input.agent).expect("records");
    assert_eq!(records.len(), 16);
    assert_eq!(records.front().expect("first retained").id(), 5);
    for entry in records {
        assert_eq!(
            entry
                .decode(&policy, history.dictionary())
                .expect("decode")
                .tick,
            entry.id()
        );
    }
    let restored: MindHistory =
        ron::from_str(&ron::to_string(&history).expect("serialize")).expect("restore");
    restored.check_policy(&policy).expect("policy");
    assert_eq!(restored, history);
    assert_eq!(
        std::mem::size_of::<PackedMindChoice>(),
        std::mem::size_of::<usize>()
    );
}
#[test]
fn wrong_dictionary_unknown_slots_and_duplicate_facts_fail() {
    let policy = policy();
    let mut input = record(&policy);
    input.facts.push(input.facts[0]);
    assert!(PackedMindChoice::new(&policy, &input).is_err());
    input = record(&policy);
    let packed = PackedMindChoice::new(&policy, &input).expect("pack");
    let dictionary = MindRecordDictionary::from_policy(&policy).expect("dictionary");
    let mut json = serde_json::to_value(&dictionary).expect("serialize");
    json["facts"][0][0] = serde_json::json!("different_name");
    let different = serde_json::from_value(json).expect("dictionary parse");
    assert_eq!(
        packed.decode(&policy, &different),
        Err(MindRecordError::PolicyMismatch)
    );
    let mut reader = super::reader::Reader {
        bytes: packed.bytes(),
        at: 0,
    };
    reader.byte().expect("version");
    for _ in 0..3 {
        reader.unsigned().expect("revision/id/tick");
    }
    reader.optional().expect("parent");
    reader.unsigned().expect("agent");
    let mut unknown = packed.bytes().to_vec();
    unknown[reader.at] = 127;
    let unknown = PackedMindChoice::from_bytes(&unknown).expect("structurally valid raw slot");
    assert_eq!(
        unknown.decode(&policy, &dictionary),
        Err(MindRecordError::UnknownSlot("node"))
    );
    input.band = policy.band_id_at(0).expect("critical");
    assert!(PackedMindChoice::new(&policy, &input).is_err());
}
#[test]
fn oversized_or_misowned_saved_history_is_rejected() {
    let policy = policy();
    let mut input = record(&policy);
    input.agent = 1;
    let packed = PackedMindChoice::new(&policy, &input).expect("pack");
    let mut history = MindHistory::new(&policy).expect("history");
    history.push(&policy, packed).expect("push");
    let mut json = serde_json::to_value(&history).expect("serialize");
    let one = json["people"]["1"][0].clone();
    json["people"]["1"] = serde_json::Value::Array(vec![one; 17]);
    assert!(serde_json::from_value::<MindHistory>(json).is_err());
    let mut json = serde_json::to_value(&history).expect("serialize");
    let records = json["people"]
        .as_object_mut()
        .expect("object")
        .remove("1")
        .expect("person");
    json["people"]["2"] = records;
    assert!(serde_json::from_value::<MindHistory>(json).is_err());
}

#[test]
fn dictionary_rejects_changed_score_metadata_even_with_same_revision() {
    let policy = policy();
    let dictionary = MindRecordDictionary::from_policy(&policy).unwrap();
    let mut value = serde_json::to_value(&dictionary).unwrap();
    value["nodes"][0]["score_terms"][0][1] = serde_json::json!(999);
    let altered: MindRecordDictionary = serde_json::from_value(value).unwrap();
    assert_eq!(altered.check(&policy), Err(MindRecordError::PolicyMismatch));
}

#[test]
fn initialized_insertion_preserves_bytes_history_and_shared_payload() {
    let policy = policy();
    let mut strict = MindHistory::new(&policy).expect("strict history");
    let mut initialized = MindHistory::new(&policy).expect("initialized history");
    let mut input = record(&policy);
    input.parent = None;
    input.facts[0].cause = None;
    for id in 1..=20 {
        input.id = id;
        input.tick = id;
        let expected = strict.push_record(&policy, &input).expect("strict push");
        let actual = initialized
            .push_record_for_initialized_policy(&policy, &input)
            .expect("initialized push");
        assert_eq!(actual.bytes(), expected.bytes());
        assert!(initialized
            .latest(input.agent)
            .expect("latest")
            .shares_payload(&actual));
    }
    assert_eq!(initialized, strict);
    assert_eq!(
        serde_json::to_vec(&initialized).unwrap(),
        serde_json::to_vec(&strict).unwrap()
    );
    initialized
        .check_policy(&policy)
        .expect("strict validation");
}

#[test]
fn initialized_insertion_retains_record_and_order_checks() {
    let policy = policy();
    let mut history = MindHistory::new(&policy).expect("history");
    let input = record(&policy);
    let mut malformed = input.clone();
    malformed.facts.push(malformed.facts[0]);
    assert!(history
        .push_record_for_initialized_policy(&policy, &malformed)
        .is_err());
    assert!(history.latest(input.agent).is_none());
    history
        .push_record_for_initialized_policy(&policy, &input)
        .expect("valid record");
    assert!(history
        .push_record_for_initialized_policy(&policy, &input)
        .is_err());
    assert_eq!(history.records(input.agent).unwrap().len(), 1);
}

#[test]
fn strict_history_boundaries_reject_same_revision_dictionary_corruption() {
    let policy = policy();
    let history = MindHistory::new(&policy).expect("history");
    let mut wire = serde_json::to_value(&history).unwrap();
    wire["dictionary"]["nodes"][0]["score_terms"][0][1] = serde_json::json!(999);
    let mut corrupt: MindHistory = serde_json::from_value(wire).unwrap();
    let input = record(&policy);
    assert_eq!(
        corrupt.check_policy(&policy),
        Err(MindRecordError::PolicyMismatch)
    );
    assert_eq!(
        corrupt.push_record(&policy, &input),
        Err(MindRecordError::PolicyMismatch)
    );
    let packed = PackedMindChoice::new(&policy, &input).unwrap();
    assert_eq!(
        corrupt.push(&policy, packed),
        Err(MindRecordError::PolicyMismatch)
    );
    assert!(corrupt.latest(input.agent).is_none());
}

#[test]
fn initialized_insertion_rejects_revision_mismatch_before_mutation() {
    let policy = policy();
    let history = MindHistory::new(&policy).expect("history");
    let mut wire = serde_json::to_value(&history).unwrap();
    wire["dictionary"]["revision"] = serde_json::json!(policy.revision().code().wrapping_add(1));
    let mut mismatched: MindHistory = serde_json::from_value(wire).unwrap();
    let input = record(&policy);
    assert_eq!(
        mismatched.push_record_for_initialized_policy(&policy, &input),
        Err(MindRecordError::PolicyMismatch)
    );
    assert!(mismatched.latest(input.agent).is_none());
}

#[test]
fn initialized_insertion_rejects_same_revision_corruption_without_panicking() {
    let policy = policy();
    let history = MindHistory::new(&policy).expect("history");
    let mut wire = serde_json::to_value(&history).unwrap();
    wire["dictionary"]["nodes"][0]["score_terms"][0][1] = serde_json::json!(999);
    let mut corrupt: MindHistory = serde_json::from_value(wire).unwrap();
    let before = serde_json::to_vec(&corrupt).unwrap();
    assert_eq!(
        corrupt.push_record_for_initialized_policy(&policy, &record(&policy)),
        Err(MindRecordError::PolicyMismatch)
    );
    assert_eq!(serde_json::to_vec(&corrupt).unwrap(), before);
}

#[test]
fn validation_cache_preserves_roundtrip_equality_and_wire_bytes() {
    let policy = policy();
    let mut history = MindHistory::new(&policy).unwrap();
    history.push_record(&policy, &record(&policy)).unwrap();
    let bytes = serde_json::to_vec(&history).unwrap();
    let mut restored: MindHistory = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, history);
    restored.check_initialized_policy(&policy).unwrap();
    restored.check_initialized_policy(&policy).unwrap();
    assert_eq!(restored, history);
    assert_eq!(restored.clone(), history);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    restored.check_policy(&policy).unwrap();
}

#[test]
fn cached_validation_failure_does_not_mark_a_corrupt_history_valid() {
    let policy = policy();
    let mut wire = serde_json::to_value(MindHistory::new(&policy).unwrap()).unwrap();
    wire["dictionary"]["nodes"][0]["score_terms"][0][1] = serde_json::json!(999);
    let mut corrupt: MindHistory = serde_json::from_value(wire).unwrap();
    for _ in 0..2 {
        assert_eq!(
            corrupt.check_initialized_policy(&policy),
            Err(MindRecordError::PolicyMismatch)
        );
        assert_eq!(
            corrupt.push_record_for_initialized_policy(&policy, &record(&policy)),
            Err(MindRecordError::PolicyMismatch)
        );
    }
}

#[test]
fn immutable_identity_matches_only_the_actual_choice_after_round_trip() {
    let policy = policy();
    let record = record(&policy);
    let packed = PackedMindChoice::new(&policy, &record).expect("packed");
    let restored: PackedMindChoice =
        serde_json::from_slice(&serde_json::to_vec(&packed).expect("serialize")).expect("restore");
    let running = crate::colony::mind::RunningChoice {
        revision: policy.revision(),
        node: record.node,
        finished: false,
    };
    for value in [&packed, &restored] {
        assert!(value
            .matches_running(record.agent, running)
            .expect("identity"));
        assert!(!value
            .matches_running(record.agent - 1, running)
            .expect("wrong agent"));
        assert!(!value
            .matches_running(
                record.agent,
                crate::colony::mind::RunningChoice {
                    node: policy.node_id("second").expect("second"),
                    ..running
                }
            )
            .expect("wrong node"));
    }
    assert_eq!(packed.bytes(), restored.bytes());
}

fn assert_cached_identity_matches_prefix(packed: &PackedMindChoice, policy: &CompiledMind) {
    let (revision, agent, node) = codec::identity(packed.bytes()).expect("validated prefix");
    for query_revision in [0, u64::MAX, policy.revision().code()] {
        for query_agent in [0, u64::MAX, agent] {
            for query_node in ["first", "second", "fallback"] {
                for finished in [false, true] {
                    let running = crate::colony::mind::RunningChoice {
                        revision: serde_json::from_value(serde_json::json!(query_revision))
                            .expect("revision"),
                        node: policy.node_id(query_node).expect("node"),
                        finished,
                    };
                    assert_eq!(
                        packed.matches_running(query_agent, running),
                        Ok(revision == query_revision
                            && agent == query_agent
                            && node == running.node.index()),
                    );
                }
            }
        }
    }
}

#[test]
fn cached_identity_preserves_checked_construction_wire_and_shared_ownership() {
    let policy = policy();
    for extreme in [0, u64::MAX] {
        for name in ["first", "second", "fallback"] {
            let mut input = record(&policy);
            input.id = extreme;
            input.tick = extreme;
            input.agent = extreme;
            input.parent = None;
            input.facts.clear();
            input.node = policy.node_id(name).expect("node");
            input.band = policy.node(input.node).expect("node definition").band;
            input.top3 = vec![ChoiceCandidate {
                node: input.node,
                score: input.score,
            }];
            let expected = codec::encode(&policy, &input).expect("old wire");
            let packed = PackedMindChoice::new(&policy, &input).expect("checked record");
            let clone = packed.clone();
            let parsed = PackedMindChoice::from_bytes(&expected).expect("checked wire");
            let restored: PackedMindChoice =
                serde_json::from_slice(&serde_json::to_vec(&packed).expect("serialize"))
                    .expect("restore");
            assert!(packed.shares_payload(&clone));
            assert!(!packed.shares_payload(&parsed));
            for value in [&packed, &clone, &parsed, &restored] {
                assert_eq!(value, &packed);
                assert_eq!(value.bytes(), expected);
                assert_cached_identity_matches_prefix(value, &policy);
            }
        }
    }
}

#[test]
fn cached_standalone_identity_preserves_extreme_unresolved_slots_and_revisions() {
    fn unsigned(bytes: &mut Vec<u8>, mut value: u64) {
        while value >= 128 {
            bytes.push((value as u8 & 127) | 128);
            value >>= 7;
        }
        bytes.push(value as u8);
    }
    let policy = policy();
    for revision in [0, u64::MAX, policy.revision().code()] {
        for node in [0, usize::MAX] {
            let mut bytes = vec![1];
            for value in [revision, u64::MAX, u64::MAX] {
                unsigned(&mut bytes, value);
            }
            bytes.push(0); // No parent.
            unsigned(&mut bytes, u64::MAX);
            unsigned(&mut bytes, u64::try_from(node).expect("wire slot"));
            bytes.extend_from_slice(&[0, 0, 1, 0, 0, 0]); // Band, score, one candidate, no facts.
            let packed = PackedMindChoice::from_bytes(&bytes).expect("structurally valid");
            assert_eq!(packed.bytes(), bytes);
            assert_cached_identity_matches_prefix(&packed, &policy);
            for end in 0..bytes.len() {
                assert!(PackedMindChoice::from_bytes(&bytes[..end]).is_err());
            }
            bytes.push(0);
            assert!(PackedMindChoice::from_bytes(&bytes).is_err());
        }
    }
    for malformed in [vec![1, 128, 0], vec![1; 1], vec![255; 11]] {
        assert!(PackedMindChoice::from_bytes(&malformed).is_err());
    }
}

#[test]
fn cached_identity_memory_cost_is_explicit() {
    // Existing header/bytes total 48 bytes on 64-bit hosts; the cache adds 24.
    let original_fields = 2 * std::mem::size_of::<u64>()
        + std::mem::size_of::<Option<EventId>>()
        + std::mem::size_of::<Box<[u8]>>();
    let cache_fields = 2 * std::mem::size_of::<u64>() + std::mem::size_of::<usize>();
    assert_eq!(
        std::mem::size_of::<Payload>(),
        original_fields + cache_fields
    );
    assert_eq!(
        std::mem::size_of::<PackedMindChoice>(),
        std::mem::size_of::<usize>()
    );
}
