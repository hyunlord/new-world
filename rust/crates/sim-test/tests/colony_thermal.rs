//! Focused physical heat, numeric Cold and bed-stage regressions; no balance bars.
mod colony_support;
use colony_support::{adults, bedroll, detect, open_area, walled_box};
use sim_core::colony::thermal::PaidHeat;
use sim_core::components::{Position, ResourceKind};
use sim_engine::RuntimeSystem;
use sim_systems::runtime::thermal;

fn room() -> (sim_engine::SimEngine, Vec<(u32, u32)>) {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 5, 5);
    let tiles = walled_box(&mut engine, at, 5, 5, true);
    detect(&mut engine);
    engine
        .resources
        .building_kinds
        .insert(tiles[4], sim_core::components::BuildingKind::Hearth);
    thermal::initialize(&mut engine.resources);
    (engine, tiles)
}

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
    rule.burn_per_hour = [3;4];
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
fn data_only_material_profile_changes_real_room_heat() {
    use sim_core::colony::content::{self, BootSelection, NamedSource, RawSources};
    if let Ok(insulation) = std::env::var("THERMAL_HEAT_TEST_CHILD") {
        let selection = if insulation == "900" {
            BootSelection::Bundled
        } else {
            let source = sim_core::colony::thermal::ThermalRules::SOURCE
                .replace("(\"oak\",900)", "(\"oak\",700)");
            BootSelection::Explicit {
                request_id: "thermal-runtime-test".into(),
                sources: RawSources {
                    sources: vec![NamedSource {
                        name: "thermal".into(),
                        bytes: source.into_bytes(),
                    }],
                },
            }
        };
        content::bootstrap(selection).unwrap();
        let (mut engine, tiles) = room();
        assert_eq!(
            engine
                .resources
                .thermal
                .as_ref()
                .unwrap()
                .rules
                .material_insulation_permille[&sim_core::colony::materials::OAK],
            insulation.parse::<u16>().unwrap()
        );
        let sid = *engine.resources.settlements.keys().min().unwrap();
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
        println!(
            "THERMAL_RESULT={}",
            thermal::place_temperature(&engine.resources, tiles[4]).unwrap()
        );
        return;
    }
    let mut temperatures = Vec::new();
    for insulation in ["900", "700"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "data_only_material_profile_changes_real_room_heat",
                "--nocapture",
            ])
            .env("THERMAL_HEAT_TEST_CHILD", insulation)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let temperature = stdout
            .lines()
            .find_map(|line| line.split_once("THERMAL_RESULT="))
            .map(|(_, v)| v.trim().parse::<i32>().unwrap())
            .unwrap();
        temperatures.push(temperature);
    }
    assert!(
        temperatures[0] > temperatures[1],
        "actual selected boot must affect room heat: {temperatures:?}"
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
    let mut hunger = Hunger::default();
    hunger.value = 0.0;
    hunger.growth_rate = 1.0;
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

#[test]
fn newly_enclosed_room_needs_its_own_hourly_sample_for_warm_stage() {
    let (mut engine,tiles)=room();
    let bed=tiles[0];bedroll(&mut engine,bed);
    let (_,person)=adults(&engine)[0];
    {let mut pos=engine.world.get::<&mut Position>(person).unwrap();pos.x=bed.0;pos.y=bed.1;}
    let sid=*engine.resources.settlements.keys().min().unwrap();
    engine.resources.current_tick=11;
    thermal::record_paid_heat(&mut engine.resources,sid,PaidHeat{tick:0,tile:tiles[4],requested_wood:3,debited_wood:3});
    thermal::update_after_rooms(&mut engine.world,&mut engine.resources);
    assert!(!thermal::dwelling_view(&engine.world,&engine.resources,bed).unwrap().warm_occupants,"a sample before this room existed is not evidence of its warmth");
    engine.resources.current_tick=60;
    thermal::record_paid_heat(&mut engine.resources,sid,PaidHeat{tick:60,tile:tiles[4],requested_wood:3,debited_wood:3});
    thermal::update_after_rooms(&mut engine.world,&mut engine.resources);
    assert!(thermal::dwelling_view(&engine.world,&engine.resources,bed).unwrap().warm_occupants);
}
