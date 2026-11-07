//! Steward completion ownership and physical housing regression fixtures.
use super::*;
use sim_core::{
    colony::{
        steward::{rules, OwnedOrder},
        CommandRecord, CommandResult,
    },
    rooms::{Room, RoomRole},
};
fn fixture() -> (sim_engine::SimEngine, Rule) {
    let mut e = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(29),
        sim_core::rules::WorldRules::colony(29),
    );
    let rule = rules().expect("rules").rules[2].clone();
    let Action::House {
        bed, door, wall, ..
    } = rule.action
    else {
        panic!("house")
    };
    e.resources
        .colony
        .beds
        .insert((10, 10), sim_core::colony::materials::OAK);
    e.resources.colony.rooms.insert(
        1,
        Room {
            dwelling_kind: None,
            id: 1,
            tiles: BTreeSet::from([(10, 10)]),
            doors: BTreeSet::from([(10, 9)]),
            roofed: true,
            role: RoomRole::Workshop,
            dug: false,
        },
    );
    for (x, y) in [(10, 9), (9, 10), (11, 10), (10, 11)] {
        let index = e.resources.tile_grid.idx(x, y);
        e.resources.tile_grid.wall_material[index] = Some(sim_core::colony::materials::OAK);
    }
    let index = e.resources.tile_grid.idx(10, 10);
    e.resources.tile_grid.wall_material[index] = None;
    e.resources.tile_grid.set_dug(index, false);
    let index = e.resources.tile_grid.idx(10, 9);
    e.resources.tile_grid.set_door(index, true);
    for (kind, x, y) in [
        (wall, 9, 10),
        (wall, 11, 10),
        (wall, 10, 11),
        (bed, 10, 10),
        (door, 10, 9),
    ] {
        e.resources.steward.owned.push(OwnedOrder {
            rule: rule.id,
            command: Command::PlaceBlueprint { kind, x, y },
            targets: Vec::new(),
        });
    }
    (e, rule)
}
#[test]
fn completion_requires_owned_beds_boundary_and_no_foreign_override() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (mut e, rule) = fixture();
    assert!(plan(&e.resources, &rule).is_some());
    e.resources
        .colony
        .room_overrides
        .insert(1, RoomRole::Workshop);
    assert!(plan(&e.resources, &rule).is_none());
    e.resources.colony.room_overrides.clear();
    e.resources.steward.owned.pop();
    assert!(plan(&e.resources, &rule).is_none());
    let (mut e, rule) = fixture();
    e.resources.steward.owned.remove(3);
    assert!(plan(&e.resources, &rule).is_none());
    let (mut e, rule) = fixture();
    e.resources
        .colony
        .beds
        .insert((11, 10), sim_core::colony::materials::OAK);
    e.resources
        .colony
        .rooms
        .get_mut(&1)
        .expect("room")
        .tiles
        .insert((11, 10));
    assert!(plan(&e.resources, &rule).is_none());
}
#[test]
fn mixed_player_boundary_is_not_an_exclusively_owned_enclosure() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (mut e, rule) = fixture();
    e.resources.steward.owned.remove(1);
    assert!(
        plan(&e.resources, &rule).is_none(),
        "one owned door does not own a foreign wall"
    );
}
#[test]
fn later_player_bed_replacement_does_not_inherit_historical_ownership() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (mut e, rule) = fixture();
    let Action::House { bed, .. } = rule.action else {
        panic!("house")
    };
    e.resources.colony.command_log.push(CommandRecord {
        tick: 1,
        command: Command::PlaceBlueprint {
            kind: bed,
            x: 10,
            y: 10,
        },
        result: CommandResult::ok(1),
    });
    assert!(plan(&e.resources, &rule).is_none());
}
#[test]
fn data_role_and_absent_optional_field_are_respected() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (e, _) = fixture();
    let text = include_str!("../../../../sim-core/data/colony/steward.ron");
    let changed =
        sim_core::colony::steward::parse_rules(&text.replace("Some(Bedroom)", "Some(Hall)"))
            .expect("data");
    assert_eq!(
        plan(&e.resources, &changed.rules[2]),
        Some(Command::SetRoomRole {
            room: 1,
            role: Some(RoomRole::Hall)
        })
    );
    let old = sim_core::colony::steward::parse_rules(
        &text.replace(", completion_role: Some(Bedroom)", ""),
    )
    .expect("old data");
    assert!(plan(&e.resources, &old.rules[2]).is_none());
}
#[test]
fn valid_manual_auto_survives_room_id_change_but_unknown_does_not_block() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (mut e, rule) = fixture();
    e.resources.colony.command_log.push(CommandRecord {
        tick: 0,
        command: Command::SetRoomRole {
            room: 999,
            role: None,
        },
        result: CommandResult::rejected(RejectReason::Unknown),
    });
    assert!(plan(&e.resources, &rule).is_some());
    e.resources.colony.command_log.push(CommandRecord {
        tick: 0,
        command: Command::SetRoomRole {
            room: 1,
            role: None,
        },
        result: CommandResult::rejected(RejectReason::Nothing),
    });
    let mut room = e.resources.colony.rooms.remove(&1).expect("room");
    room.id = 2;
    e.resources.colony.rooms.insert(2, room);
    assert!(plan(&e.resources, &rule).is_none());
}

#[test]
fn bedless_store_completion_requires_real_owned_enclosure() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (mut e, mut rule) = fixture();
    let Action::House {
        completion_role, ..
    } = &mut rule.action
    else {
        panic!("house");
    };
    *completion_role = Some(RoomRole::Storehouse);
    e.resources.colony.rooms.get_mut(&1).expect("room").role = RoomRole::Unassigned;
    e.resources.colony.beds.clear();
    e.resources.steward.owned.remove(3);
    assert_eq!(
        plan(&e.resources, &rule),
        Some(Command::SetRoomRole {
            room: 1,
            role: Some(RoomRole::Storehouse),
        })
    );
    e.resources.colony.rooms.get_mut(&1).expect("room").roofed = false;
    assert!(plan(&e.resources, &rule).is_none());
    e.resources.colony.rooms.get_mut(&1).expect("room").roofed = true;
    e.resources.steward.owned.remove(0);
    assert!(plan(&e.resources, &rule).is_none());
}

#[test]
fn store_planner_places_real_boundaries_without_beds() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let (mut e, mut rule) = fixture();
    e.resources.colony.rooms.clear();
    e.resources.colony.beds.clear();
    e.resources.colony.trees.clear();
    e.resources.content_state.tree_materials.clear();
    e.resources.colony.nodes.clear();
    let Action::House {
        completion_role,
        bed,
        ..
    } = &mut rule.action
    else {
        panic!("house");
    };
    *completion_role = Some(RoomRole::Storehouse);
    let bed = *bed;
    let commands = super::super::planning::plan(&e.world, &e.resources, &rule);
    assert!(!commands.is_empty(), "Store must plan real construction");
    assert!(commands.iter().all(|command| !matches!(command,
        Command::PlaceBlueprint { kind, .. } if *kind == bed)));
    for command in commands {
        assert!(crate::commands::apply_command(&mut e.world, &mut e.resources, command).accepted);
    }
}

#[test]
fn inferred_matching_role_still_commits_owned_house_completion_once() {
    let (mut engine, rule) = fixture();
    engine
        .resources
        .colony
        .rooms
        .get_mut(&1)
        .expect("room")
        .role = RoomRole::Bedroom;
    assert!(engine.resources.colony.room_overrides.is_empty());
    let command = plan(&engine.resources, &rule).expect("inferred role is not committed intent");
    assert_eq!(
        command,
        Command::SetRoomRole {
            room: 1,
            role: Some(RoomRole::Bedroom)
        }
    );
    assert!(
        crate::commands::apply_command(&mut engine.world, &mut engine.resources, command).accepted
    );
    assert_eq!(
        engine.resources.colony.room_overrides.get(&1),
        Some(&RoomRole::Bedroom)
    );
    assert!(
        plan(&engine.resources, &rule).is_none(),
        "do not issue repeated no-op completion"
    );
}

fn shared_wall_store() -> (sim_engine::SimEngine, Rule) {
    let (mut e, sleeping) = fixture();
    let store = rules()
        .expect("rules")
        .rules
        .iter()
        .find(|rule| {
            matches!(
                rule.action,
                Action::House {
                    completion_role: Some(RoomRole::Storehouse),
                    ..
                }
            )
        })
        .expect("store rule")
        .clone();
    let Action::House { wall, door, .. } = store.action else {
        panic!("house")
    };
    e.resources.colony.beds.clear();
    e.resources.steward.owned.clear();
    e.resources.colony.command_log.clear();
    e.resources.colony.rooms.get_mut(&1).expect("room").role = RoomRole::Unassigned;
    for (source, kind, tile) in [
        (sleeping.id, wall, (9, 10)),
        (store.id, wall, (11, 10)),
        (store.id, wall, (10, 11)),
        (store.id, door, (10, 9)),
    ] {
        let index = e.resources.tile_grid.idx(tile.0, tile.1);
        e.resources.tile_grid.wall_material[index] = None;
        e.resources.tile_grid.set_door(index, false);
        e.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Plain;
        e.resources.colony.trees.remove(&tile);
        e.resources.colony.nodes.remove(&tile);
        let command = Command::Steward {
            rule: source,
            command: Box::new(Command::PlaceBlueprint {
                kind,
                x: tile.0,
                y: tile.1,
            }),
        };
        assert!(
            crate::commands::apply_and_log(&mut e.world, &mut e.resources, command, 1).accepted
        );
        let entity = e
            .world
            .query::<&sim_core::colony::ColonySite>()
            .iter()
            .find(|(_, site)| site.top_left == tile)
            .map(|(entity, _)| entity)
            .expect("accepted site");
        // Fixture supplies construction materials; actual completion owns physical wall writes.
        {
            let mut needs = e
                .world
                .get::<&mut sim_core::colony::MaterialNeeds>(entity)
                .expect("needs");
            for line in &mut needs.lines {
                line.delivered = line.needed;
            }
        }
        crate::runtime::jobs::space::complete_site(&mut e.world, &mut e.resources, entity);
    }
    assert!(super::super::housing::constructed(
        &e.resources,
        &e.resources.colony.rooms[&1]
    ));
    assert_eq!(
        e.resources
            .colony
            .command_log
            .iter()
            .filter(|record| record.result.accepted)
            .count(),
        4
    );
    (e, store)
}

#[test]
fn store_completes_with_previously_built_steward_house_wall() {
    let (e, store) = shared_wall_store();
    assert_eq!(
        plan(&e.resources, &store),
        Some(Command::SetRoomRole {
            room: 1,
            role: Some(RoomRole::Storehouse),
        }),
        "a completed store may share a still-steward-owned sleeping-house wall"
    );
}

#[test]
fn shared_wall_store_preserves_active_roles_and_manual_intent() {
    for role in [RoomRole::Bedroom, RoomRole::Workshop, RoomRole::Hall] {
        let (mut e, store) = shared_wall_store();
        e.resources.colony.rooms.get_mut(&1).expect("room").role = role;
        assert!(
            plan(&e.resources, &store).is_none(),
            "must not steal {role:?}"
        );
    }
    let (mut e, store) = shared_wall_store();
    e.resources.colony.command_log.push(CommandRecord {
        tick: 2,
        command: Command::SetRoomRole {
            room: 1,
            role: None,
        },
        result: CommandResult::rejected(RejectReason::Nothing),
    });
    assert!(plan(&e.resources, &store).is_none());
}

#[test]
fn shared_wall_store_requires_current_physical_steward_ownership() {
    let (mut e, store) = shared_wall_store();
    let old = e.resources.colony.command_log[0].clone();
    let Command::Steward { command, .. } = old.command else {
        panic!("steward wall")
    };
    e.resources.colony.command_log.push(CommandRecord {
        tick: 2,
        command: *command,
        result: CommandResult::ok(1),
    });
    assert!(
        plan(&e.resources, &store).is_none(),
        "manual replacement revokes shared provenance"
    );

    let (mut e, store) = shared_wall_store();
    e.resources
        .steward
        .owned
        .retain(|order| order.rule != store.id);
    assert!(
        plan(&e.resources, &store).is_none(),
        "store needs its own accepted boundary"
    );

    let (mut e, store) = shared_wall_store();
    e.resources.steward.owned.remove(0);
    assert!(
        plan(&e.resources, &store).is_none(),
        "physical shared wall alone is not ownership"
    );

    let (mut e, store) = shared_wall_store();
    let index = e.resources.tile_grid.idx(9, 10);
    e.resources.tile_grid.wall_material[index] = None;
    assert!(
        plan(&e.resources, &store).is_none(),
        "ownership alone is not a physical enclosure"
    );
}
