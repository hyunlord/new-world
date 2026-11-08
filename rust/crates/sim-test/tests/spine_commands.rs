//! Command rejection and cancellation retain the existing physical ledger contracts.
mod spine_support;
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::building_site::{BuildingStatus, PiecePhase};
use sim_core::colony::{Command, RejectReason};
use sim_systems::commands::submit;
use spine_support::*;

#[test]
fn spine_invalid_and_inert_templates_do_not_allocate_or_place() {
    let mut engine = scene();
    let default = sim_systems::runtime::building_templates::commands::material_requirements(
        &engine.resources,
        "workshop_shed",
        None,
        0,
    )
    .expect("default station costs");
    let selected = sim_systems::runtime::building_templates::commands::material_requirements(
        &engine.resources,
        "workshop_shed",
        Some("knapping_spot"),
        0,
    )
    .expect("authored alternative costs");
    assert_ne!(
        default, selected,
        "selected physical station changes exact requirements"
    );

    for (id, anchor, rotation, station, expected) in [
        ("missing_template", (1, 1), 0, None, RejectReason::Unknown),
        ("well", (1, 1), 0, None, RejectReason::NotYet),
        ("log_house", (u32::MAX, 1), 0, None, RejectReason::OutOfMap),
        ("log_house", (1, 1), 4, None, RejectReason::Unknown),
        (
            "workshop_shed",
            (1, 1),
            0,
            Some("missing_station"),
            RejectReason::Unknown,
        ),
        (
            "log_house",
            (1, 1),
            0,
            Some("wood_bench"),
            RejectReason::Unknown,
        ),
    ] {
        let before =
            sim_systems::save::encode(&engine.resources.colony.buildings).expect("state bytes");
        let physical = sim_systems::runtime::jobs::items::colony_sites(&engine.world).len();
        let result = submit(
            &mut engine,
            Command::BuildTemplate {
                id: id.to_owned(),
                anchor,
                rotation,
                station: station.map(str::to_owned),
            },
        );
        assert_eq!(result.reason, Some(expected));
        assert_eq!(
            sim_systems::save::encode(&engine.resources.colony.buildings).expect("state bytes"),
            before
        );
        assert_eq!(
            sim_systems::runtime::jobs::items::colony_sites(&engine.world).len(),
            physical
        );
    }
}

#[test]
fn spine_cancel_partial_construction_keeps_ledger_and_never_completes() {
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
    until(
        &mut engine,
        "physical delivery before cancellation",
        |engine| {
            sim_systems::runtime::jobs::items::colony_sites(&engine.world)
                .values()
                .any(|site| site.needs.used().any(|line| line.delivered > 0))
        },
    );
    cancel_remaining(&mut engine, id);
    let site = &engine.resources.colony.buildings.sites[&id];
    assert_eq!(site.status, BuildingStatus::Cancelled);
    assert!(site
        .pieces
        .iter()
        .all(|piece| matches!(piece.phase, PiecePhase::Complete | PiecePhase::Cancelled)));
    assert!(site.completion_entry.is_none());
    for _ in 0..240 {
        engine.tick();
    }
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
        .expect("refund and carried supplies are conserved");
    assert!(!engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|entry| entry.kind == ChronicleKind::BuildingComplete));
    let saved = sim_systems::save::save(&mut engine).expect("cancelled save");
    let loaded = sim_systems::save::load(&saved).expect("cancelled load");
    assert_eq!(
        loaded.resources.colony.buildings.sites[&id].status,
        BuildingStatus::Cancelled
    );
}

#[test]
fn spine_missing_physical_site_is_cancelled_not_completed() {
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
    // An ordinary player area-cancel removes one active physical site. The aggregate
    // must detect its loss; disappearance is not an exact completion witness.
    let piece = engine.resources.colony.buildings.sites[&id]
        .pieces
        .iter()
        .find(|piece| piece.phase == PiecePhase::Active)
        .expect("physical site")
        .clone();
    command(
        &mut engine,
        Command::CancelArea {
            x0: piece.tile.0,
            y0: piece.tile.1,
            x1: piece.tile.0,
            y1: piece.tile.1,
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.sites[&id].status,
        BuildingStatus::Cancelled,
        "accepted top-level cancel reconciles before any tick"
    );
    let paused = sim_systems::save::save(&mut engine).expect("paused CancelArea save");
    sim_systems::save::load(&paused).expect("paused CancelArea load");
    engine.tick();
    assert_eq!(
        engine.resources.colony.buildings.sites[&id].status,
        BuildingStatus::Cancelled
    );
    assert!(engine.resources.colony.buildings.sites[&id]
        .completion_entry
        .is_none());
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
        .expect("physical cancellation retains invariants");
}
