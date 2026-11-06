use super::*;

#[test]
fn warm_boundary_requests_heat_before_hourly_cooling() {
    let (mut engine, sid, person, tile) = cold_room();
    engine.resources.current_tick = 21 * 1440 + 540;
    let room = engine.resources.colony.room_of_tile[&tile];
    let state = engine.resources.thermal.as_mut().expect("thermal");
    let threshold = state.rules.cold_threshold_d10;
    state.rooms.get_mut(&room).expect("room").temperature_d10 = threshold;
    let _ = engine
        .world
        .remove_one::<sim_core::colony::people::Apparel>(person);
    assert!(firewood::room_heat_needed(
        &engine.world,
        &engine.resources,
        sid
    ));
    let before = wood(&engine, sid);
    let rate = firewood::daytime_service_rate(&engine.resources);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(before - wood(&engine, sid), rate);
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert!(
        thermal::felt_temperature(&engine.resources, None, tile).expect("temperature") >= threshold
    );
}

#[test]
fn preview_matches_commit_with_projected_door_intervals_and_paid_heat() {
    use sim_core::colony::thermal::DoorThermal;
    for (accounted, open_ticks, held_open) in [(719, 0, false), (700, 4, false), (660, 0, true)] {
        for paid in [false, true] {
            let (mut engine, sid, _, tile) = cold_room();
            let now = engine.resources.current_tick;
            let id = engine.resources.colony.room_of_tile[&tile];
            let room = &engine.resources.colony.rooms[&id];
            let max_y = room.tiles.iter().map(|t| t.1).max().expect("interior");
            let door = (tile.0, max_y + 1);
            assert!(engine
                .resources
                .tile_grid
                .is_door(engine.resources.tile_grid.idx(door.0, door.1)));
            engine
                .resources
                .thermal
                .as_mut()
                .expect("thermal")
                .doors
                .insert(
                    door,
                    DoorThermal {
                        accounted_until_tick: now - (720 - accounted),
                        open_until_tick: now - 2,
                        open_ticks_in_hour: open_ticks,
                        held_open,
                    },
                );
            if paid {
                FireSystem::new().tick(&mut engine.world, &mut engine.resources);
            }
            let before = engine.resources.thermal.clone();
            let predicted =
                thermal::preview_room_temperature(&engine.world, &engine.resources, id, None)
                    .expect("preview");
            assert_eq!(engine.resources.thermal, before, "preview must be pure");
            if paid {
                let without = thermal::preview_room_temperature(
                    &engine.world,
                    &engine.resources,
                    id,
                    Some(sid),
                )
                .expect("unpaid preview");
                assert!(predicted > without);
            }
            thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
            assert_eq!(
                thermal::place_temperature(&engine.resources, tile),
                Some(predicted)
            );
        }
    }
}

#[test]
fn warm_apparel_occupant_does_not_request_unneeded_room_heat() {
    use sim_core::colony::people::{Apparel, Top};
    let (mut engine, sid, person, tile) = cold_room();
    let id = engine.resources.colony.room_of_tile[&tile];
    engine
        .resources
        .thermal
        .as_mut()
        .expect("thermal")
        .rooms
        .get_mut(&id)
        .expect("room")
        .temperature_d10 = 50;
    let predicted =
        thermal::preview_room_temperature(&engine.world, &engine.resources, id, Some(sid))
            .expect("preview");
    assert!(predicted < 50);
    let apparel = Apparel {
        top: Top::FurCloak,
        since_day: 0,
        material: None,
        quality: 50,
        wear_remaining_hours: None,
    };
    engine.world.insert_one(person, apparel).expect("apparel");
    assert!(!firewood::room_heat_needed(
        &engine.world,
        &engine.resources,
        sid
    ));
}
