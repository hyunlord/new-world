//! Actual piece placement and aggregate preview use the same current person occupancy.
mod spine_support;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::content_state::CompletedLayer;
use sim_core::colony::{Command, RejectReason};
use sim_core::components::{Agent, Position};
use sim_systems::commands::submit;
use sim_systems::runtime::building_templates::commands::preview;
use spine_support::*;

#[test]
fn spine_person_on_planned_wall_rejects_preview_build_and_physical_piece_equally() {
    // Given a valid template envelope and a real person standing on its planned wall.
    let mut engine = scene();
    let anchor = anchor(&engine, "log_house");
    let catalog = installed_or_bundled();
    let wall = catalog
        .building_templates()
        .get("log_house")
        .expect("house")
        .expand(catalog, None, 0)
        .expect("authored plan")
        .into_iter()
        .find(|piece| {
            sim_systems::runtime::jobs::space::completion_layer(piece.kind) == CompletedLayer::Wall
        })
        .expect("ring wall");
    let tile = (anchor.0 + wall.offset.0, anchor.1 + wall.offset.1);
    let entity = engine
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .next()
        .map(|(entity, _)| entity)
        .expect("seed0 person");
    {
        let mut position = engine
            .world
            .get::<&mut Position>(entity)
            .expect("person position");
        position.x = tile.0;
        position.y = tile.1;
    }
    let before = engine.resources.colony.buildings.clone();
    let occupied = engine.resources.colony.occupied.clone();
    let physical = sim_systems::runtime::jobs::items::colony_sites(&engine.world).len();
    // When all three authoritative placement entry points inspect this same paused world.
    assert_eq!(
        preview(
            &engine.world,
            &engine.resources,
            "log_house",
            anchor,
            0,
            None
        ),
        Err(RejectReason::Person)
    );
    let direct = submit(
        &mut engine,
        Command::PlaceBlueprint {
            kind: wall.kind,
            x: tile.0,
            y: tile.1,
        },
    );
    let template = submit(
        &mut engine,
        Command::BuildTemplate {
            id: "log_house".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    // Then parity holds and neither command allocates construction or template state.
    assert_eq!(direct.reason, Some(RejectReason::Person));
    assert_eq!(template.reason, direct.reason);
    assert_eq!(engine.resources.colony.buildings, before);
    assert_eq!(engine.resources.colony.occupied, occupied);
    assert_eq!(
        sim_systems::runtime::jobs::items::colony_sites(&engine.world).len(),
        physical
    );
}
