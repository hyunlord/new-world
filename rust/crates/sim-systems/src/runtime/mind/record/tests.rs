//! Recorder scenarios exercise real selector witnesses and actual causal/chronicle stores.
use super::*;
use sim_core::chronicle::CauseRef;
use sim_core::colony::mind::{
    choose, compile, BehaviourSpec, EvalKey, EvalScratch, FactKind, FactSpec, LifeStage,
    MindRegistrySchema, PersonFacts,
};
use sim_core::colony::mind_records::MindHistory;
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use std::sync::atomic::Ordering;

fn fixture() -> (SimResources, CompiledMind) {
    let schema = MindRegistrySchema {
        facts: vec![
            FactSpec {
                name: "trigger".into(),
                kind: FactKind::Signed,
            },
            FactSpec {
                name: "hunger".into(),
                kind: FactKind::Signed,
            },
        ],
        behaviours: vec![BehaviourSpec {
            name: "eat".into(),
            target_bindings: vec![],
        }],
    };
    let policy=compile(r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
        (id:"critical",band:"critical",behaviour:"eat",when:[(fact:"trigger",op:">",value:Some(0))],
         score:(constant:1,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"CRITICAL"),
        (id:"need",band:"need",behaviour:"eat",when:[(fact:"hunger",op:">",value:Some(0))],
         score:(constant:1,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"NEED"),
        (id:"idle",band:"idle",behaviour:"eat",when:[],
         score:(constant:0,terms:[(fact:"trigger",weight_permille:0)]),hysteresis:0,
         life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#,&schema).expect("policy");
    let mut resources = SimResources::new(4, 4, MaterialRegistry::new());
    resources.rules = WorldRules::colony(0);
    resources.current_tick = 10;
    resources.mind.history = Some(MindHistory::new(&policy).expect("history"));
    (resources, policy)
}
fn select(
    resources: &SimResources,
    policy: &CompiledMind,
    trigger: i64,
    hunger: i64,
) -> (Selection, EvalScratch) {
    let mut table = PersonFacts::new(policy, vec![7], resources.current_tick).expect("facts");
    table.set_life_stage(0, LifeStage::Adult).expect("stage");
    table
        .set(policy.fact_id("trigger").expect("trigger"), 0, trigger)
        .expect("trigger value");
    table
        .set(policy.fact_id("hunger").expect("hunger"), 0, hunger)
        .expect("hunger value");
    let mut scratch = EvalScratch::default();
    let selection = choose(
        policy,
        table.row(7).expect("row"),
        resources.mind.running.get(&7).copied(),
        EvalKey {
            world_seed: 0,
            agent: 7,
            tick: resources.current_tick,
            replicate: 0,
        },
        &mut scratch,
    )
    .expect("choose");
    (selection, scratch)
}
fn person() -> RecordPerson {
    RecordPerson {
        agent: 7,
        position: (1, 1),
        settlement: None,
    }
}
#[test]
fn actual_source_is_bridged_once_and_only_critical_band_entries_are_chronicled() {
    let (mut resources, policy) = fixture();
    let entry = resources
        .chronicle
        .push_caused(ChronicleEntry::new(2, ChronicleKind::PressureBegan), None);
    let source = MindFactSource::Chronicle(CauseRef::Entry(entry));
    let origins = [FactOrigin {
        fact: policy.fact_id("trigger").expect("fact"),
        source: FactCause::Source(source),
        priority: CausePriority::Threat,
    }];
    let (selection, scratch) = select(&resources, &policy, 1, 0);
    let id = record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &origins,
    )
    .expect("record")
    .expect("switch");
    assert_eq!(id, 1);
    assert_eq!(resources.mind.fact_source_events.len(), 1);
    assert_eq!(resources.chronicle.entries().len(), 2);
    assert_eq!(
        resources.chronicle.cause_of(1),
        Some(CauseRef::Entry(entry))
    );
    let next = resources.next_event_id.load(Ordering::Relaxed);
    let (selection, scratch) = select(&resources, &policy, 1, 0);
    assert_eq!(
        record_selection(
            &mut resources,
            &policy,
            person(),
            &selection,
            scratch.witnesses(),
            &origins
        )
        .expect("continue"),
        None
    );
    assert_eq!(resources.next_event_id.load(Ordering::Relaxed), next);
    resources.current_tick += 1;
    let (selection, scratch) = select(&resources, &policy, 0, 1);
    record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &[],
    )
    .expect("need");
    assert_eq!(resources.chronicle.entries().len(), 2);
    resources.current_tick += 1;
    let (selection, scratch) = select(&resources, &policy, 1, 0);
    record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &origins,
    )
    .expect("critical again");
    assert_eq!(resources.mind.fact_source_events.len(), 1);
    assert_eq!(resources.chronicle.entries().len(), 3);
    let history = resources.mind.history.as_ref().expect("history");
    let packed = history.latest(7).expect("latest");
    assert_eq!(packed.parent(), Some(0));
    assert!(!packed
        .decode(&policy, history.dictionary())
        .expect("decode")
        .facts
        .is_empty());
    let tile = resources.causal_log.get(5).expect("tile");
    let event = tile
        .as_slice()
        .iter()
        .find(|event| event.id() == packed.id())
        .expect("event");
    match event {
        CausalEvent::MindChoice { choice } => assert!(choice.shares_payload(packed)),
        _ => panic!("wrong kind"),
    }
}
#[test]
fn nonexistent_source_and_baseline_are_rejected_without_issuing_events() {
    let (mut resources, policy) = fixture();
    let (selection, scratch) = select(&resources, &policy, 1, 0);
    let origins = [FactOrigin {
        fact: policy.fact_id("trigger").expect("fact"),
        source: FactCause::Source(MindFactSource::Command(999)),
        priority: CausePriority::Job,
    }];
    assert!(record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &origins
    )
    .is_err());
    assert_eq!(resources.next_event_id.load(Ordering::Relaxed), 0);
    resources.rules.colony = None;
    assert!(record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &[]
    )
    .is_err());
    assert_eq!(resources.next_event_id.load(Ordering::Relaxed), 0);
}
#[test]
fn confirmed_meal_origin_uses_its_actual_choice() {
    let (mut resources, policy) = fixture();
    let (selection, scratch) = select(&resources, &policy, 0, 1);
    let id = record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &[],
    )
    .expect("meal")
    .expect("switch");
    remember_completed_meal(&mut resources, &policy, 7, id).expect("actual completed meal");
    assert_eq!(
        resources
            .mind
            .fact_origins
            .get(&(7, policy.fact_id("hunger").expect("fact"))),
        Some(&id)
    );
    assert!(remember_completed_meal(&mut resources, &policy, 7, id + 1).is_err());
}

#[test]
fn losing_node_fact_retains_provenance_without_becoming_selected_parent() {
    let (mut resources, policy) = fixture();
    let entry = resources
        .chronicle
        .push_caused(ChronicleEntry::new(2, ChronicleKind::PressureBegan), None);
    let origins = [FactOrigin {
        fact: policy.fact_id("trigger").expect("fact"),
        source: FactCause::Source(MindFactSource::Chronicle(CauseRef::Entry(entry))),
        priority: CausePriority::Threat,
    }];
    let (selection, scratch) = select(&resources, &policy, 0, 1);
    record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &origins,
    )
    .expect("record");
    let history = resources.mind.history.as_ref().expect("history");
    let choice = history
        .latest(7)
        .expect("choice")
        .decode(&policy, history.dictionary())
        .expect("decode");
    assert_eq!(choice.parent, None);
    assert_eq!(
        choice
            .facts
            .iter()
            .find(|fact| fact.fact == origins[0].fact)
            .expect("rejected guard witness")
            .cause,
        Some(0)
    );
}

#[test]
fn origin_skip_matches_exact_continuation_and_preserves_record_state() {
    let (mut resources, policy) = fixture();
    let (selection, scratch) = select(&resources, &policy, 1, 0);
    assert!(!is_unfinished_continuation(&policy, None, &selection));
    let id = record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &[],
    )
    .unwrap()
    .unwrap();
    let prior = resources.mind.running.get(&7).copied();
    assert!(is_unfinished_continuation(&policy, prior, &selection));
    let before = bincode::serialize(&resources.mind).unwrap();
    let events = bincode::serialize(&resources.causal_log).unwrap();
    let chronicle = bincode::serialize(&resources.chronicle).unwrap();
    let next = resources.next_event_id.load(Ordering::Relaxed);
    // The old path ignores origins on this exact continuation, including invalid ones.
    let ignored = [FactOrigin {
        fact: policy.fact_id("trigger").unwrap(),
        source: FactCause::Source(MindFactSource::Command(999)),
        priority: CausePriority::Job,
    }];
    for origins in [&ignored[..], &[][..]] {
        assert_eq!(
            record_selection(
                &mut resources,
                &policy,
                person(),
                &selection,
                scratch.witnesses(),
                origins
            )
            .unwrap(),
            None
        );
        assert_eq!(bincode::serialize(&resources.mind).unwrap(), before);
        assert_eq!(bincode::serialize(&resources.causal_log).unwrap(), events);
        assert_eq!(bincode::serialize(&resources.chronicle).unwrap(), chronicle);
        assert_eq!(resources.next_event_id.load(Ordering::Relaxed), next);
        assert_eq!(
            resources
                .mind
                .history
                .as_ref()
                .unwrap()
                .latest(7)
                .unwrap()
                .id(),
            id
        );
    }
    let mut finished = prior.unwrap();
    finished.finished = true;
    assert!(!is_unfinished_continuation(
        &policy,
        Some(finished),
        &selection
    ));
    let (changed, _) = select(&resources, &policy, 0, 1);
    assert!(!is_unfinished_continuation(&policy, prior, &changed));
    let mut wrong_revision = prior.unwrap();
    wrong_revision.revision = bincode::deserialize(
        &bincode::serialize(&policy.revision().code().wrapping_add(1)).unwrap(),
    )
    .unwrap();
    assert!(!is_unfinished_continuation(
        &policy,
        Some(wrong_revision),
        &selection
    ));
    resources.mind.running.insert(7, wrong_revision);
    assert!(record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &[]
    )
    .is_err());
    resources.mind.running.insert(7, prior.unwrap());
    resources.rules.colony = None;
    assert!(record_selection(
        &mut resources,
        &policy,
        person(),
        &selection,
        scratch.witnesses(),
        &[]
    )
    .is_err());
    assert_eq!(resources.next_event_id.load(Ordering::Relaxed), next);
}

#[test]
fn replaced_history_is_rejected_before_recording_or_continuation() {
    for continuation in [false, true] {
        let (mut resources, policy) = fixture();
        let (selection, scratch) = select(&resources, &policy, 1, 0);
        record_selection(
            &mut resources,
            &policy,
            person(),
            &selection,
            scratch.witnesses(),
            &[],
        )
        .unwrap();
        if !continuation {
            resources.mind.running.clear();
        }
        let history = resources.mind.history.as_ref().unwrap();
        let name = history.dictionary().node_label(0).unwrap().name.as_bytes();
        let mut wire = bincode::serialize(history).unwrap();
        // Fixed-width bincode prefix: revision, node count, then first name length.
        assert_eq!(&wire[24..24 + name.len()], name);
        wire[24] ^= 1; // Same-length valid ASCII name corruption; revision stays intact.
        resources.mind.history = Some(bincode::deserialize(&wire).unwrap());
        let before = bincode::serialize(&resources.mind).unwrap();
        let events = bincode::serialize(&resources.causal_log).unwrap();
        let chronicle = bincode::serialize(&resources.chronicle).unwrap();
        let next = resources.next_event_id.load(Ordering::Relaxed);
        let result = record_selection(
            &mut resources,
            &policy,
            person(),
            &selection,
            scratch.witnesses(),
            &[],
        );
        assert!(matches!(
            result,
            Err(RecordError::Payload(
                sim_core::colony::mind_records::MindRecordError::PolicyMismatch
            ))
        ));
        assert_eq!(bincode::serialize(&resources.mind).unwrap(), before);
        assert_eq!(bincode::serialize(&resources.causal_log).unwrap(), events);
        assert_eq!(bincode::serialize(&resources.chronicle).unwrap(), chronicle);
        assert_eq!(resources.next_event_id.load(Ordering::Relaxed), next);
    }
}

#[test]
fn initialized_history_validation_preserves_bincode_roundtrip() {
    let (resources, policy) = fixture();
    let history = resources.mind.history.as_ref().unwrap();
    let bytes = bincode::serialize(history).unwrap();
    let mut restored: MindHistory = bincode::deserialize(&bytes).unwrap();
    assert_eq!(&restored, history);
    restored.check_initialized_policy(&policy).unwrap();
    assert_eq!(&restored, history);
    assert_eq!(bincode::serialize(&restored).unwrap(), bytes);
}
