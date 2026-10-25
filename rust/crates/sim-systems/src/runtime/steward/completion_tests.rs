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
