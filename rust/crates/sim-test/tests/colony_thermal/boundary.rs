//! Focused thermal boundary regression cases.
use super::*;

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
                "boundary::data_only_material_profile_changes_real_room_heat",
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
fn actual_first_tick_passage_opens_door_but_stationary_presence_does_not() {
    use sim_core::components::{AgentState, SeekTarget, TargetKind};
    for moving in [false, true] {
        let (mut engine, tiles) = room();
        let room_id = engine.resources.colony.room_of_tile[&tiles[0]];
        let door = *engine.resources.colony.rooms[&room_id]
            .doors
            .first()
            .unwrap();
        let people = adults(&engine);
        for &(_, entity) in &people {
            engine
                .world
                .insert_one(
                    entity,
                    AgentState::Consuming {
                        target: TargetKind::Sleep,
                    },
                )
                .unwrap();
        }
        let (_, person) = people[0];
        {
            let mut pos = engine.world.get::<&mut Position>(person).unwrap();
            pos.x = door.0;
            pos.y = if moving { door.1 - 1 } else { door.1 };
        }
        if moving {
            engine
                .world
                .insert_one(
                    person,
                    AgentState::Seeking {
                        target: TargetKind::Job,
                    },
                )
                .unwrap();
            engine
                .world
                .insert_one(person, SeekTarget::new(door))
                .unwrap();
        }
        engine.resources.current_tick = 0;
        engine.resources.thermal = None;
        sim_systems::runtime::agent::AgentMovementSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        let opened = engine
            .resources
            .thermal
            .as_ref()
            .and_then(|state| state.doors.get(&door))
            .is_some_and(|d| d.open_until_tick > 0);
        assert_eq!(opened, moving, "only accepted actual movement opens a door");
    }
}
