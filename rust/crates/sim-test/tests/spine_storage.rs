//! Storage templates use real construction and the existing stockpile identity pipeline.
mod spine_support;
use sim_core::colony::building_site::BuildingStatus;
use sim_core::colony::Command;
use sim_systems::runtime::building_templates::{self, storage};
use spine_support::*;
use std::collections::BTreeSet;

#[test]
fn spine_storehouse_marks_only_interior_and_preserves_existing_stockpile_identity() {
    let mut engine = scene();
    harvest_logs(&mut engine);
    let anchor = anchor(&engine, "storehouse");
    let covered = (anchor.0 + 1, anchor.1 + 1);
    command(
        &mut engine,
        Command::AddStockpile {
            x0: covered.0,
            y0: covered.1,
            x1: covered.0,
            y1: covered.1,
        },
    );
    let old_zone = engine.resources.colony.zone_of_tile[&covered];
    let before = engine.resources.colony.zone_of_tile.clone();
    let id = engine.resources.colony.buildings.next_site_id;
    command(
        &mut engine,
        Command::BuildTemplate {
            id: "storehouse".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    let commands = engine.resources.colony.command_log.len();
    until(&mut engine, "storehouse physically completed", |engine| {
        engine.resources.colony.buildings.sites[&id].status == BuildingStatus::Complete
    });
    let site = engine.resources.colony.buildings.sites[&id].clone();
    let interior = BTreeSet::from([
        (anchor.0 + 1, anchor.1 + 1),
        (anchor.0 + 2, anchor.1 + 1),
        (anchor.0 + 1, anchor.1 + 2),
        (anchor.0 + 2, anchor.1 + 2),
    ]);
    assert!(interior
        .iter()
        .all(|tile| engine.resources.colony.zone_of_tile.contains_key(tile)));
    for (tile, zone) in &before {
        assert_eq!(engine.resources.colony.zone_of_tile.get(tile), Some(zone));
    }
    let added: BTreeSet<_> = engine
        .resources
        .colony
        .zone_of_tile
        .keys()
        .filter(|tile| !before.contains_key(tile))
        .copied()
        .collect();
    assert_eq!(
        added,
        interior
            .difference(&BTreeSet::from([covered]))
            .copied()
            .collect()
    );
    assert_eq!(engine.resources.colony.zone_of_tile[&covered], old_zone);
    assert_eq!(
        engine.resources.colony.command_log.len(),
        commands,
        "completion retains parent provenance without invented AddStockpile command"
    );
    assert!(
        engine
            .resources
            .colony
            .rooms
            .values()
            .any(|room| { room.dwelling_kind.as_deref() == Some("storehouse") }),
        "actual detected storage room carries its template identity"
    );
    let next = engine.resources.colony.next_zone_id;
    storage::complete(&engine.world, &mut engine.resources, &site)
        .expect("covered cells are idempotent");
    assert_eq!(engine.resources.colony.next_zone_id, next);
    let mut invalid = site.clone();
    invalid.anchor.0 = u32::MAX;
    assert!(storage::complete(&engine.world, &mut engine.resources, &invalid).is_err());
    assert_eq!(
        engine.resources.colony.next_zone_id, next,
        "failed preflight cannot allocate"
    );
    let zones = engine.resources.colony.zone_of_tile.clone();
    let saved = sim_systems::save::save(&mut engine).expect("storage save");
    let mut loaded = sim_systems::save::load(&saved).expect("storage load");
    for _ in 0..120 {
        loaded.tick();
    }
    assert_eq!(loaded.resources.colony.zone_of_tile, zones);
    assert_eq!(loaded.resources.colony.next_zone_id, next);
    assert_eq!(
        loaded.resources.colony.buildings.sites[&id].completion_entry,
        site.completion_entry
    );
    building_templates::validate_state(&loaded.world, &loaded.resources)
        .expect("valid completed storage");
    assert!(loaded.resources.colony.rooms.values().any(|room| {
        room.dwelling_kind.as_deref() == Some("storehouse")
            && room.role == sim_core::rooms::RoomRole::Storehouse
    }));
    let wall = site
        .pieces
        .iter()
        .find(|piece| {
            sim_systems::runtime::jobs::space::completion_layer(piece.kind)
                == sim_core::colony::content_state::CompletedLayer::Wall
        })
        .expect("storehouse wall")
        .tile;
    command(
        &mut loaded,
        Command::DesignateDeconstruct {
            x0: wall.0,
            y0: wall.1,
            x1: wall.0,
            y1: wall.1,
        },
    );
    until(
        &mut loaded,
        "real demolition reconciles aggregate",
        |engine| engine.resources.colony.buildings.sites[&id].status == BuildingStatus::Cancelled,
    );
    assert!(!building_templates::is_intact(
        &loaded.resources,
        &loaded.resources.colony.buildings.sites[&id]
    ));
    assert_eq!(
        loaded.resources.colony.buildings.sites[&id].completion_entry,
        site.completion_entry
    );
    building_templates::validate_state(&loaded.world, &loaded.resources)
        .expect("legitimate demolition retains historical completion");
    let demolished = sim_systems::save::save(&mut loaded).expect("demolished template save");
    sim_systems::save::load(&demolished).expect("demolished template load");
}
