//! Physical housing regressions: natural enclosure does not replace constructed walls.
use super::*;
use sim_core::rooms::{Room, RoomRole};

fn fixture() -> (sim_engine::SimEngine, Action, Tile) {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let mut engine = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(4),
        sim_core::rules::WorldRules::colony(4),
    );
    let rules = sim_core::colony::steward::rules().expect("rules");
    for command in plan(&engine.world, &engine.resources, &rules.rules[0]) {
        assert!(
            crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                .accepted
        );
    }
    engine.resources.colony.trees.clear();
    engine.resources.content_state.tree_materials.clear();
    let hearth = hearth_tiles(&engine.resources)[0];
    for (_, position) in engine.world.query::<&mut Position>().iter() {
        position.x = hearth.0;
        position.y = hearth.1;
    }
    (engine, rules.rules[2].action.clone(), hearth)
}

#[test]
fn natural_enclosure_beds_do_not_clear_physical_housing_deficit() {
    let (mut engine, action, hearth) = fixture();
    assert!(
        !house(&engine.world, &engine.resources, hearth, &action, false)
            .reserved
            .is_empty()
    );
    let count = engine.world.query::<&Agent>().iter().count() as u32;
    let tiles: BTreeSet<_> = (2..2 + count).map(|x| (x, 2)).collect();
    for &tile in &tiles {
        engine
            .resources
            .colony
            .beds
            .insert(tile, sim_core::colony::materials::OAK);
        engine.resources.colony.room_of_tile.insert(tile, 999);
    }
    engine.resources.colony.rooms.insert(
        999,
        Room {
            id: 999,
            tiles,
            doors: BTreeSet::from([(2, 1)]),
            roofed: true,
            role: RoomRole::Bedroom,
            dug: false,
        },
    );
    assert!(
        !house(&engine.world, &engine.resources, hearth, &action, false)
            .reserved
            .is_empty(),
        "roofed beds without a physical perimeter must leave housing demand"
    );
}

#[test]
fn planned_house_has_complete_constructed_cardinal_boundary() {
    let (engine, action, hearth) = fixture();
    let ready = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(!ready.commands.is_empty(), "fixture must produce a house");
    let Action::House {
        wall, door, bed, ..
    } = action
    else {
        panic!("house")
    };
    let mut boundary = BTreeSet::new();
    let mut doors = BTreeSet::new();
    let mut beds = BTreeSet::new();
    for command in ready.commands {
        if let Command::PlaceBlueprint { kind, x, y } = command {
            if kind == wall || kind == door {
                boundary.insert((x, y));
            }
            if kind == door {
                doors.insert((x, y));
            }
            if kind == bed {
                beds.insert((x, y));
            }
        }
    }
    let grid = &engine.resources.tile_grid;
    let rooms = sim_core::rooms::detect(
        grid.width,
        grid.height,
        1,
        256,
        |tile| boundary.contains(&tile) || crate::runtime::rooms::bounds(&engine.resources, tile),
        |tile| doors.contains(&tile),
        |tile| boundary.contains(&tile),
    );
    let room = rooms
        .iter()
        .find(|room| beds.is_subset(&room.tiles))
        .expect("planned bedroom");
    assert!(!beds.is_empty());
    for &(x, y) in &room.tiles {
        for tile in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if !room.tiles.contains(&tile) {
                assert!(
                    boundary.contains(&tile)
                        || grid.wall_material[grid.idx(tile.0, tile.1)].is_some(),
                    "natural boundary {tile:?} cannot substitute for a built wall"
                );
            }
        }
    }
}
