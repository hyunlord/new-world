//! Focused thermal dwelling regression cases.
use super::*;

#[test]
fn occupied_hearth_floor_exclusion_is_order_independent_and_real_floor_required() {
    let (mut engine, tiles) = room();
    let bed = tiles[0];
    bedroll(&mut engine, bed);
    engine
        .resources
        .building_kinds
        .insert(tiles[4], sim_core::components::BuildingKind::Hearth);
    for &tile in &tiles {
        let index = engine.resources.tile_grid.idx(tile.0, tile.1);
        engine.resources.tile_grid.floor_material[index] = Some(sim_core::colony::materials::OAK);
    }
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    let before = thermal::dwelling_view(&engine.world, &engine.resources, bed).unwrap();
    let index = engine.resources.tile_grid.idx(tiles[4].0, tiles[4].1);
    engine.resources.tile_grid.floor_material[index] = None;
    let after = thermal::dwelling_view(&engine.world, &engine.resources, bed).unwrap();
    assert_eq!(before.floor_coverage_permille, 1000);
    assert_eq!(after.floor_coverage_permille, 1000);
    assert_eq!(before.candidate, after.candidate);
    let index = engine.resources.tile_grid.idx(bed.0, bed.1);
    engine.resources.tile_grid.floor_material[index] = None;
    assert!(
        thermal::dwelling_view(&engine.world, &engine.resources, bed)
            .unwrap()
            .floor_coverage_permille
            < 1000
    );
}

#[test]
fn deficiency_waits_elapsed_time_and_recovery_clears_it() {
    let (mut engine, tiles) = room();
    let bed = tiles[0];
    bedroll(&mut engine, bed);
    for &tile in &tiles {
        let i = engine.resources.tile_grid.idx(tile.0, tile.1);
        engine.resources.tile_grid.floor_material[i] = Some(sim_core::colony::materials::OAK);
    }
    engine.resources.current_tick = 1439;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    let earned = thermal::dwelling_view(&engine.world, &engine.resources, bed)
        .unwrap()
        .stage;
    let index = engine.resources.tile_grid.idx(bed.0, bed.1);
    engine.resources.tile_grid.floor_material[index] = None;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    let deadline = thermal::dwelling_view(&engine.world, &engine.resources, bed)
        .unwrap()
        .deficiency_deadline
        .unwrap();
    assert_eq!(deadline, 1439 + 2880);
    engine.resources.current_tick = 1440;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert_eq!(
        thermal::dwelling_view(&engine.world, &engine.resources, bed)
            .unwrap()
            .stage,
        earned
    );
    engine.resources.tile_grid.floor_material[index] = Some(sim_core::colony::materials::OAK);
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert_eq!(
        thermal::dwelling_view(&engine.world, &engine.resources, bed)
            .unwrap()
            .deficiency_deadline,
        None
    );
    engine.resources.tile_grid.floor_material[index] = None;
    engine.resources.current_tick = 1441;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    engine.resources.current_tick = 1441 + 2880;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert_ne!(
        thermal::dwelling_view(&engine.world, &engine.resources, bed)
            .unwrap()
            .stage,
        earned
    );
}

#[test]
fn newly_enclosed_room_needs_its_own_hourly_sample_for_warm_stage() {
    let (mut engine, tiles) = room();
    let bed = tiles[0];
    bedroll(&mut engine, bed);
    let (_, person) = adults(&engine)[0];
    {
        let mut pos = engine.world.get::<&mut Position>(person).unwrap();
        pos.x = bed.0;
        pos.y = bed.1;
    }
    let sid = *engine.resources.settlements.keys().min().unwrap();
    engine.resources.current_tick = 11;
    thermal::record_paid_heat(
        &mut engine.resources,
        sid,
        PaidHeat {
            tick: 0,
            tile: tiles[4],
            requested_wood: 3,
            debited_wood: 3,
        },
    );
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert!(
        !thermal::dwelling_view(&engine.world, &engine.resources, bed)
            .unwrap()
            .warm_occupants,
        "a sample before this room existed is not evidence of its warmth"
    );
    engine.resources.current_tick = 60;
    thermal::record_paid_heat(
        &mut engine.resources,
        sid,
        PaidHeat {
            tick: 60,
            tile: tiles[4],
            requested_wood: 3,
            debited_wood: 3,
        },
    );
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert!(
        thermal::dwelling_view(&engine.world, &engine.resources, bed)
            .unwrap()
            .warm_occupants
    );
}
