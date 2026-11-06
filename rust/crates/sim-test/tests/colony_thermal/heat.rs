//! Focused thermal heat regression cases.
use super::*;

#[test]
fn only_positive_complete_actual_payment_warms_its_physical_room() {
    for (requested, paid, heated) in [(0, 0, false), (3, 2, false), (3, 3, true)] {
        let (mut engine, tiles) = room();
        let sid = *engine.resources.settlements.keys().min().unwrap();
        let state = engine.resources.thermal.as_mut().unwrap();
        state.rules.adult_body_heat = 0;
        state.rules.child_body_heat = 0;
        state.rules.season_outdoor_d10 = [-50; 4];
        state.rules.hour_offset_d10 = [0; 24];
        engine.resources.current_tick = 60;
        thermal::record_paid_heat(
            &mut engine.resources,
            sid,
            PaidHeat {
                tick: 60,
                tile: tiles[4],
                requested_wood: requested,
                debited_wood: paid,
            },
        );
        thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
        let temperature = thermal::place_temperature(&engine.resources, tiles[4]).unwrap();
        assert_eq!(temperature > -50, heated);
        let ledger = engine.resources.colony.item_ledger.clone();
        let once = engine.resources.thermal.clone();
        thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
        assert_eq!(
            thermal::place_temperature(&engine.resources, tiles[4]),
            Some(temperature)
        );
        assert_eq!(engine.resources.colony.item_ledger, ledger);
        assert_eq!(
            engine
                .resources
                .thermal
                .as_ref()
                .unwrap()
                .last_exposure_tick,
            once.unwrap().last_exposure_tick
        );
    }
}

#[test]
fn fire_receipt_preserves_one_partial_buffer_debit_and_no_item_sink() {
    let (mut engine, tiles) = room();
    let sid = *engine.resources.settlements.keys().min().unwrap();
    let rule = engine.resources.rules.society.firewood.as_mut().unwrap();
    rule.burn_start_minute = 0;
    rule.burn_end_minute = 1439;
    rule.burn_per_hour = [3; 4];
    rule.winter_severity_permille = None;
    let owner = engine.resources.settlements.get_mut(&sid).unwrap();
    owner.formation_tile = tiles[4];
    owner.withdraw(ResourceKind::Wood, u32::MAX);
    owner.store(ResourceKind::Wood, 1);
    let ledger = engine.resources.colony.item_ledger.clone();
    sim_systems::runtime::settlement::FireSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    let receipt = engine.resources.thermal.as_ref().unwrap().paid_hearths[&sid];
    assert_eq!(receipt.requested_wood, 3);
    assert_eq!(receipt.debited_wood, 1);
    assert!(!engine.resources.fires[&sid].lit);
    assert_eq!(engine.resources.colony.item_ledger, ledger);
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.resources.colony.item_ledger, ledger);
}

#[test]
fn personal_refresh_does_not_charge_an_extra_cold_hour() {
    let (mut engine, tiles) = room();
    let (_, person) = adults(&engine)[0];
    {
        let mut pos = engine.world.get::<&mut Position>(person).unwrap();
        pos.x = tiles[4].0;
        pos.y = tiles[4].1;
    }
    engine.resources.current_tick = 60;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    let hours = engine.resources.tallies.cold_member_hours;
    engine.resources.current_tick = 61;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.resources.tallies.cold_member_hours, hours);
}

#[test]
fn negative_temperature_rounds_down_and_roof_loss_is_immediate() {
    let (mut engine, tiles) = room();
    let state = engine.resources.thermal.as_mut().unwrap();
    state.rules.adult_body_heat = 0;
    state.rules.child_body_heat = 0;
    state.rules.season_outdoor_d10 = [-50; 4];
    state.rules.hour_offset_d10 = [0; 24];
    engine.resources.current_tick = 1;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    let id = engine.resources.colony.room_of_tile[&tiles[4]];
    engine
        .resources
        .thermal
        .as_mut()
        .unwrap()
        .rooms
        .get_mut(&id)
        .unwrap()
        .temperature_d10 = -49;
    engine.resources.current_tick = 60;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert_eq!(
        thermal::place_temperature(&engine.resources, tiles[4]),
        Some(-50)
    );
    engine
        .resources
        .thermal
        .as_mut()
        .unwrap()
        .rooms
        .get_mut(&id)
        .unwrap()
        .temperature_d10 = 100;
    engine.resources.colony.rooms.get_mut(&id).unwrap().roofed = false;
    assert_eq!(
        thermal::place_temperature(&engine.resources, tiles[4]),
        Some(-50)
    );
}

#[test]
fn hunger_boundary_reads_previous_committed_cold() {
    use sim_core::components::{Cold, Hunger};
    let (mut engine, tiles) = room();
    let (_, person) = adults(&engine)[0];
    let hunger = Hunger::new(0.0, 1.0);
    engine.world.insert_one(person, hunger).unwrap();
    engine.world.insert_one(person, Cold).unwrap();
    {
        let mut pos = engine.world.get::<&mut Position>(person).unwrap();
        pos.x = tiles[4].0;
        pos.y = tiles[4].1;
    }
    engine.resources.current_tick = 60;
    let multiplier = engine
        .resources
        .rules
        .society
        .firewood
        .unwrap()
        .cold_hunger_permille as f32
        / 1000.0;
    sim_systems::runtime::needs::HungerDecaySystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine.world.get::<&Hunger>(person).unwrap().value,
        multiplier
    );
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Cold>(person).is_err());
    engine.resources.current_tick = 61;
    sim_systems::runtime::needs::HungerDecaySystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine.world.get::<&Hunger>(person).unwrap().value,
        multiplier + 1.0
    );
}
