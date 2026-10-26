//! Terminal memory companion capacity, sharing and opt-in tests.
use super::*;
use sim_core::MaterialRegistry;
use sim_systems::runtime::mind::ColonyMindSystem;
fn world() -> SimEngine {
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
        MindProgram::load(include_str!("../../sim-core/data/colony/mind.ron")).expect("policy"),
    )));
    engine.tick();
    engine
}
#[test]
fn disabled_companion_leaves_baseline_state_untouched() {
    let mut engine = crate::scenes::baseline_scene(42);
    let before = crate::probe::fingerprint(&engine);
    for option in [None, Some("0"), Some("true")] {
        assert!(companion(&mut engine, option).expect("disabled").is_none());
    }
    assert_eq!(crate::probe::fingerprint(&engine), before);
    assert!(engine.resources.mind.history.is_none());
}
#[test]
fn saturation_fills_sixteen_and_shares_log_payloads_without_ticking() {
    let mut engine = world();
    let tick = engine.current_tick();
    let report = companion(&mut engine, Some("1"))
        .expect("saturate")
        .expect("report");
    assert_eq!(engine.current_tick(), tick);
    assert_eq!(report["after"]["min_records"], 16);
    assert_eq!(report["after"]["max_records"], 16);
    assert_eq!(report["people"], report["saturated_people"]);
    let history = engine.resources.mind.history.as_ref().expect("history");
    let mut shared = 0;
    for (_, log) in engine.resources.causal_log.iter() {
        for event in log.iter() {
            if let CausalEvent::MindChoice { choice } = event {
                let record = choice.decode_display(history.dictionary()).expect("decode");
                if let Some(retained) = history.find(record.agent, record.id) {
                    assert!(choice.shares_payload(retained));
                    shared += 1;
                }
            }
        }
    }
    assert!(shared > 0);
}
#[test]
fn missing_person_record_fails_before_issuing_ids() {
    let mut engine = world();
    let agent = engine
        .world
        .query::<&Agent>()
        .iter()
        .next()
        .expect("person")
        .1
        .id;
    engine
        .resources
        .mind
        .history
        .as_mut()
        .expect("history")
        .remove(agent);
    let next = engine
        .resources
        .next_event_id
        .load(std::sync::atomic::Ordering::Relaxed);
    assert!(saturate(&mut engine)
        .expect_err("missing record")
        .contains("lacks retained"));
    assert_eq!(
        engine
            .resources
            .next_event_id
            .load(std::sync::atomic::Ordering::Relaxed),
        next
    );
}
