//! A permitted outer steward cancellation reconciles before another command can reuse its tile.
mod spine_support;
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::building_site::{BuildingStatus, PiecePhase};
use sim_core::colony::steward::{rules, Action, StewardScope};
use sim_core::colony::Command;
use sim_systems::runtime::jobs::space::{completed_at, completion_layer};
use spine_support::*;

#[test]
fn spine_wrapped_cancel_replacement_does_not_adopt_old_template_provenance() {
    // Given a real queued template and an allowed single steward envelope.
    let mut engine = scene();
    harvest_logs(&mut engine);
    let anchor = anchor(&engine, "log_house");
    let id = engine.resources.colony.buildings.next_site_id;
    command(
        &mut engine,
        Command::BuildTemplate {
            id: "log_house".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    let original = engine.resources.colony.buildings.sites[&id].clone();
    let piece = original
        .pieces
        .iter()
        .find(|piece| piece.phase == PiecePhase::Active)
        .expect("actual physical work")
        .clone();
    let rule = rules()
        .expect("validated rules")
        .rules
        .iter()
        .find(|rule| {
            rule.scope == StewardScope::Housing && matches!(rule.action, Action::House { .. })
        })
        .expect("real housing rule")
        .id;
    // When cancellation and same-kind independent replacement are submitted without a tick.
    command(
        &mut engine,
        Command::Steward {
            rule,
            command: Box::new(Command::CancelArea {
                x0: piece.tile.0,
                y0: piece.tile.1,
                x1: piece.tile.0,
                y1: piece.tile.1,
            }),
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.sites[&id].status,
        BuildingStatus::Cancelled,
        "outer accepted cancellation reconciles before returning"
    );
    command(
        &mut engine,
        Command::PlaceBlueprint {
            kind: piece.kind,
            x: piece.tile.0,
            y: piece.tile.1,
        },
    );
    let saved =
        sim_systems::save::save(&mut engine).expect("paused wrapped cancel and replacement save");
    let mut loaded = sim_systems::save::load(&saved).expect("paused replacement load");
    // Then the independent physical completion cannot revive or complete the old aggregate.
    until(
        &mut loaded,
        "independent replacement piece complete",
        |engine| {
            completed_at(&engine.resources, piece.tile, completion_layer(piece.kind))
                == Some(piece.kind)
        },
    );
    let old = &loaded.resources.colony.buildings.sites[&id];
    assert_eq!(old.status, BuildingStatus::Cancelled);
    assert_eq!(old.source_record, original.source_record);
    assert_eq!(old.cause, original.cause);
    assert!(old.completion_entry.is_none());
    assert!(!loaded
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|entry| entry.kind == ChronicleKind::BuildingComplete));
    let completed = sim_systems::save::save(&mut loaded).expect("independent completion save");
    sim_systems::save::load(&completed).expect("independent completion load");
}
