//! Restore boundary rejects forged aggregate metadata while accepting real queued work.
mod spine_support;
use sim_core::colony::building_site::{BuildingStatus, PiecePhase};
use sim_core::colony::Command;
use sim_systems::runtime::building_templates::validate_state;
use spine_support::*;

#[test]
fn spine_saved_building_semantics_reject_empty_overflow_and_forged_completion() {
    let mut engine = scene();
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
    validate_state(&engine.world, &engine.resources)
        .expect("real queued construction without stock");
    let saved = sim_systems::save::save(&mut engine).expect("valid partial save");
    let loaded = sim_systems::save::load(&saved).expect("valid partial load");
    validate_state(&loaded.world, &loaded.resources).expect("restored physical ownership");
    let original = engine.resources.colony.buildings.sites[&id].clone();
    {
        let site = engine
            .resources
            .colony
            .buildings
            .sites
            .get_mut(&id)
            .expect("site");
        site.pieces.clear();
        site.status = BuildingStatus::Complete;
    }
    assert!(validate_state(&engine.world, &engine.resources).is_err());
    assert!(
        sim_systems::save::save(&mut engine).is_err(),
        "save uses same semantic gate"
    );
    engine
        .resources
        .colony
        .buildings
        .sites
        .insert(id, original.clone());
    engine
        .resources
        .colony
        .buildings
        .sites
        .get_mut(&id)
        .expect("site")
        .anchor
        .0 = u32::MAX;
    assert!(validate_state(&engine.world, &engine.resources).is_err());
    engine
        .resources
        .colony
        .buildings
        .sites
        .insert(id, original.clone());
    {
        let site = engine
            .resources
            .colony
            .buildings
            .sites
            .get_mut(&id)
            .expect("site");
        site.status = BuildingStatus::Complete;
        site.completion_entry = None;
        for piece in &mut site.pieces {
            piece.phase = PiecePhase::Complete;
        }
    }
    assert!(
        validate_state(&engine.world, &engine.resources).is_err(),
        "allComplete cannot invent history"
    );
    engine
        .resources
        .colony
        .buildings
        .sites
        .insert(id, original.clone());
    {
        let site = engine
            .resources
            .colony
            .buildings
            .sites
            .get_mut(&id)
            .expect("site");
        site.source_record = u32::MAX;
    }
    assert!(
        validate_state(&engine.world, &engine.resources).is_err(),
        "missing command provenance"
    );
    engine.resources.colony.buildings.sites.insert(id, original);
    validate_state(&engine.world, &engine.resources).expect("restored original remains valid");
}
