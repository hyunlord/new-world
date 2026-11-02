//! Native observer contracts; no Godot runtime needed for these collector tests.

use super::*;
use sim_core::chronicle::{Chronicle, ChronicleEntry};
use sim_core::material::MaterialRegistry;

fn engine() -> SimEngine {
    let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
    engine.world.spawn((Agent { id: u64::MAX },));
    engine.world.spawn((Agent { id: 0 },));
    engine
}

#[test]
fn cursor_retains_identical_losses_and_optional_unsigned_holders() {
    let mut engine = engine();
    let mut observer = GateObserver::new(&engine);
    let entry = ChronicleEntry::new(0, ChronicleKind::KnowledgeLost).value(-1);
    engine.resources.chronicle.push(entry);
    engine.resources.chronicle.push(entry);
    engine.resources.chronicle.push(entry.subject(u64::MAX));
    observer.observe(&engine);
    observer.observe(&engine);
    let instance = &observer.instances[0];
    assert_eq!(instance.losses.len(), 3);
    assert_eq!(instance.losses[0].holder, None);
    assert_eq!(instance.losses[2].holder, Some(u64::MAX));
    assert_eq!(instance.history_gap, Some("unknown_knowledge_node"));
    assert!(!instance.history_complete);
    assert_eq!(instance.initial.agents, vec![0, u64::MAX]);
}

#[test]
fn reset_retains_bootstrap_and_load_never_claims_complete_history() {
    let mut first = engine();
    let second = engine();
    let mut observer = GateObserver::new(&first);
    first.tick();
    observer.observe(&first);
    observer.replace(&first, &second, "reset", false);
    observer.replace(&second, &first, "load", true);
    assert_eq!(observer.instances.len(), 3);
    assert_eq!(observer.instances[0].initial.tick, 0);
    assert_eq!(observer.instances[0].terminal.as_ref().unwrap().tick, 1);
    assert!(observer.instances[1].history_complete);
    assert_eq!(observer.instances[2].initial.tick, 1);
    assert!(!observer.instances[2].history_complete);
    assert_eq!(
        observer.instances[2].history_gap,
        Some("initial_history_unauthenticated")
    );
}

#[test]
fn shrinking_history_is_not_silently_treated_as_empty() {
    let mut engine = engine();
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(0, ChronicleKind::Born));
    let mut observer = GateObserver::new(&engine);
    engine.resources.chronicle = Chronicle::new();
    observer.observe(&engine);
    assert!(!observer.instances[0].history_complete);
    assert_eq!(observer.instances[0].history_gap, Some("chronicle_shrank"));
}

#[test]
fn disabled_knowledge_loss_cannot_claim_complete_null_history() {
    let mut engine = engine();
    let mut observer = GateObserver::new(&engine);
    assert!(!observer.instances[0].knowledge_enabled);
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(0, ChronicleKind::KnowledgeLost).value(0));
    observer.observe(&engine);
    assert!(!observer.instances[0].history_complete);
    assert_eq!(observer.instances[0].losses.len(), 1);
    assert!(observer.instances[0].history_gap.is_some());
}

#[test]
fn cached_capability_disagreement_fails_closed() {
    let engine = engine();
    let mut observer = GateObserver::new(&engine);
    observer.instances[0].knowledge_enabled = true;
    observer.observe(&engine);
    assert!(!observer.instances[0].history_complete);
    assert_eq!(
        observer.instances[0].history_gap,
        Some("knowledge_capability_changed")
    );
}

#[test]
fn future_and_reordered_chronicle_entries_invalidate_history() {
    let mut engine = engine();
    let mut observer = GateObserver::new(&engine);
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(2, ChronicleKind::Born));
    observer.observe(&engine);
    assert!(!observer.instances[0].history_complete);
    assert_eq!(
        observer.instances[0].history_gap,
        Some("invalid_chronicle_tick_order")
    );
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(0, ChronicleKind::Born));
    observer.observe(&engine);
    assert_eq!(observer.instances[0].cursor, 2);
    assert!(!observer.instances[0].history_complete);
}

#[test]
fn repeated_native_observation_preserves_accessed_simulation_state() {
    let engine = engine();
    let census = Census::read(&engine);
    let resources = ron::ser::to_string(&engine.resources).unwrap();
    let mut observer = GateObserver::new(&engine);
    for _ in 0..3 {
        observer.observe(&engine);
        assert_eq!(Census::read(&engine), census);
        assert_eq!(ron::ser::to_string(&engine.resources).unwrap(), resources);
    }
}

#[test]
fn manual_advance_observes_every_completed_tick_and_preserves_return_contract() {
    let mut engine = engine();
    let mut ticks = Vec::new();
    let end =
        crate::ffi::engine_build::advance_engine_ticks_observed(&mut engine, 1_000, |engine| {
            ticks.push(engine.current_tick());
        });
    assert_eq!(end, 1_000);
    assert_eq!(ticks, (1..=1_000).collect::<Vec<_>>());
    crate::ffi::engine_build::advance_engine_ticks_observed(&mut engine, -1, |_| {
        panic!("negative advance must not tick")
    });
    assert_eq!(engine.current_tick(), 1_000);
}

#[test]
fn terminal_census_tracks_live_agents_and_freezes_at_replacement() {
    let mut first = engine();
    let second = engine();
    let mut observer = GateObserver::new(&first);
    let entity = first
        .world
        .query::<&Agent>()
        .iter()
        .find_map(|(entity, agent)| (agent.id == u64::MAX).then_some(entity))
        .unwrap();
    first.world.despawn(entity).unwrap();
    first.world.spawn((Agent { id: 7 },));
    let current = Census::read(&first);
    assert_eq!(current.agents, vec![0, 7]);
    assert_eq!(
        current.agents.len(),
        first.world.query::<&Agent>().iter().count()
    );
    observer.replace(&first, &second, "reset", false);
    first.world.spawn((Agent { id: 9 },));
    assert_eq!(observer.instances[0].terminal.as_ref().unwrap(), &current);
    assert_eq!(observer.instances[0].initial.agents, vec![0, u64::MAX]);
    assert_eq!(Census::read(&second).agents, vec![0, u64::MAX]);
}
