//! Housing index subtraction independent of stockpile storage.
mod spine_support;
use sim_core::colony::Command;
use spine_support::*;

#[test]
fn spine_remove_zone_area_subtracts_housing_without_stockpile_alias() {
    let mut engine = scene();
    harvest_logs(&mut engine);
    let anchor = anchor(&engine, "log_house");
    let zone = engine.resources.colony.next_zone_id;
    command(
        &mut engine,
        Command::AddHousingZone {
            x0: anchor.0,
            y0: anchor.1,
            x1: anchor.0 + 3,
            y1: anchor.1 + 3,
        },
    );
    command(
        &mut engine,
        Command::RemoveZoneArea {
            x0: anchor.0,
            y0: anchor.1,
            x1: anchor.0 + 1,
            y1: anchor.1 + 3,
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.housing_zones[&zone].len(),
        8
    );
    assert!(engine.resources.colony.buildings.housing_zones[&zone]
        .iter()
        .all(|tile| tile.0 >= anchor.0 + 2));
}

#[test]
fn spine_remove_zone_area_counts_overlapping_housing_and_stockpile_once() {
    let mut engine = scene();
    let tile = anchor(&engine, "log_house");
    command(
        &mut engine,
        Command::AddStockpile {
            x0: tile.0,
            y0: tile.1,
            x1: tile.0,
            y1: tile.1,
        },
    );
    let stockpile = engine.resources.colony.zone_of_tile[&tile];
    let housing = engine.resources.colony.next_zone_id;
    command(
        &mut engine,
        Command::AddHousingZone {
            x0: tile.0,
            y0: tile.1,
            x1: tile.0,
            y1: tile.1,
        },
    );
    assert_ne!(
        stockpile, housing,
        "separate maps share one monotonic identity allocator"
    );
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::RemoveZoneArea {
            x0: tile.0,
            y0: tile.1,
            x1: tile.0,
            y1: tile.1,
        },
    );
    assert!(result.accepted);
    assert_eq!(
        result.count, 1,
        "union counts the tile, not both memberships"
    );
    assert!(!engine.resources.colony.zone_of_tile.contains_key(&tile));
    assert!(!engine.resources.colony.zones.contains_key(&stockpile));
    assert!(!engine
        .resources
        .colony
        .buildings
        .housing_zones
        .contains_key(&housing));
}

#[test]
fn spine_manual_zone_trim_preserves_only_complete_building_membership_and_save() {
    for remove_all in [false, true] {
        let mut engine = scene();
        let anchor = anchor(&engine, "log_house");
        let zone = engine.resources.colony.next_zone_id;
        command(
            &mut engine,
            Command::AddHousingZone {
                x0: anchor.0,
                y0: anchor.1,
                x1: anchor.0 + 4,
                y1: anchor.1 + 3,
            },
        );
        let site = engine.resources.colony.buildings.next_site_id;
        command(
            &mut engine,
            Command::BuildTemplate {
                id: "log_house".into(),
                anchor,
                rotation: 0,
                station: None,
            },
        );
        let original_site = engine.resources.colony.buildings.sites[&site].clone();
        assert_eq!(
            engine.resources.colony.buildings.site_zones.get(&site),
            Some(&zone)
        );
        assert!(!engine
            .resources
            .colony
            .buildings
            .zone_causes
            .contains_key(&zone));

        // An unused strip can disappear while the building remains fully inside.
        command(
            &mut engine,
            Command::RemoveZoneArea {
                x0: anchor.0 + 4,
                y0: anchor.1,
                x1: anchor.0 + 4,
                y1: anchor.1 + 3,
            },
        );
        assert_eq!(
            engine.resources.colony.buildings.site_zones.get(&site),
            Some(&zone)
        );
        let bytes = sim_systems::save::save(&mut engine).expect("unrelated edge trim saves");
        let restored = sim_systems::save::load(&bytes).expect("unrelated edge trim loads");
        assert_eq!(
            restored.resources.colony.buildings.site_zones.get(&site),
            Some(&zone)
        );

        command(
            &mut engine,
            Command::RemoveZoneArea {
                x0: anchor.0,
                y0: anchor.1,
                x1: anchor.0 + if remove_all { 3 } else { 0 },
                y1: anchor.1 + if remove_all { 3 } else { 0 },
            },
        );
        let buildings = &engine.resources.colony.buildings;
        assert!(
            !buildings.site_zones.contains_key(&site),
            "partial/full trim clears stale membership"
        );
        assert_eq!(buildings.housing_zones.contains_key(&zone), !remove_all);
        assert_eq!(
            buildings.sites[&site], original_site,
            "physical plan and historical provenance stay intact"
        );
        let bytes = sim_systems::save::save(&mut engine).expect("trimmed manual zone saves");
        let restored = sim_systems::save::load(&bytes).expect("trimmed manual zone loads");
        assert!(!restored
            .resources
            .colony
            .buildings
            .site_zones
            .contains_key(&site));
        assert_eq!(
            restored.resources.colony.buildings.sites[&site],
            original_site
        );
    }
}
