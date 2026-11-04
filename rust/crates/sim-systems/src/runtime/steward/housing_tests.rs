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

#[test]
fn cultivation_avoids_future_house_and_accepted_site_work_tiles() {
    let (mut engine, _, hearth) = fixture();
    engine.resources.food.plants.clear();
    let reserved = super::reserved_house_tiles(&engine.world, &engine.resources, hearth);
    assert!(!reserved.is_empty());
    let rules = sim_core::colony::steward::rules().expect("rules");
    let policy = rules
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy),
            _ => None,
        })
        .expect("food policy");
    let fields = |world: &hecs::World, res: &sim_engine::SimResources| {
        crate::runtime::steward::food::plan(world, res, policy)
            .into_iter()
            .filter_map(|command| {
                if let Command::ZoneField { min, .. } = command {
                    Some(min)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    let initial = fields(&engine.world, &engine.resources);
    assert!(
        !initial.is_empty(),
        "unrelated outdoor ground remains available"
    );
    assert!(initial.iter().all(|tile| !reserved.contains(tile)));
    let occupied = initial[0];
    engine.world.spawn((ColonySite {
        kind: sim_core::colony::BlueprintKind::WoodBench,
        top_left: occupied,
        size: (1, 1),
    },));
    let work = initial[1];
    engine
        .resources
        .colony
        .site_work_tiles
        .insert(occupied, work);
    let remaining = fields(&engine.world, &engine.resources);
    assert!(!remaining.is_empty());
    assert!(remaining
        .iter()
        .all(|tile| !reserved.contains(tile) && *tile != occupied && *tile != work));
}

#[test]
fn later_house_envelope_preserves_active_and_erased_established_fields() {
    use sim_core::colony::cultivation::{FieldPhase, FieldTile, FieldWork};
    for (active, phase) in [
        (true, FieldPhase::Untilled),
        (false, FieldPhase::Sown { planted_day: 0 }),
    ] {
        let (mut engine, action, hearth) = fixture();
        let before = house(&engine.world, &engine.resources, hearth, &action, true);
        let tile = *before
            .reserved
            .iter()
            .next()
            .expect("initial house footprint");
        let crop = sim_core::colony::content::installed_or_bundled()
            .crop_by_name("emmer")
            .expect("crop");
        engine.resources.cultivation.fields.insert(
            tile,
            FieldTile {
                owner: 1,
                crop,
                phase,
                active,
                work: FieldWork::Auto,
                tended: false,
            },
        );
        let after = house(&engine.world, &engine.resources, hearth, &action, true);
        assert!(
            !after.reserved.is_empty(),
            "another house site remains possible"
        );
        assert!(
            !after.reserved.contains(&tile),
            "actual next planned envelope must avoid retained field land"
        );
        assert_eq!(engine.resources.cultivation.fields[&tile].phase, phase);
    }
}
