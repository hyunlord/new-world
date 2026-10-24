//! Pure House role completion from accepted ownership; manual role intent always wins.
use sim_core::colony::{
    steward::{Action, Rule},
    Command, RejectReason,
};
use sim_engine::SimResources;
use std::collections::BTreeSet;

/// Complete only an owned sleeping enclosure, without mutating or manufacturing a room.
pub(super) fn plan(res: &SimResources, rule: &Rule) -> Option<Command> {
    let Action::House {
        bed,
        wall,
        door,
        completion_role: Some(role),
        ..
    } = rule.action
    else {
        return None;
    };
    // Auto can be an idempotent Nothing. The saved raw log preserves that valid intent
    // even after OFF periods and room ID changes; invalid unknown-room requests do not.
    if res.colony.command_log.iter().any(|record| {
        matches!(record.command, Command::SetRoomRole { .. })
            && (record.result.accepted || record.result.reason == Some(RejectReason::Nothing))
    }) {
        return None;
    }
    let replaced_by_other = |tile: (u32, u32)| {
        res.colony
            .command_log
            .iter()
            .rev()
            .filter(|record| record.result.accepted)
            .find_map(|record| {
                let (command, source_rule) = match &record.command {
                    Command::Steward { rule, command } => (&**command, Some(*rule)),
                    command => (command, None),
                };
                match command {
                    Command::PlaceBlueprint { x, y, .. } if (*x, *y) == tile => {
                        Some(source_rule != Some(rule.id))
                    }
                    _ => None,
                }
            })
            .unwrap_or(false)
    };
    let owned = res
        .steward
        .owned
        .iter()
        .filter(|order| order.rule == rule.id);
    let beds: BTreeSet<_> = owned
        .clone()
        .filter_map(|order| match order.command {
            Command::PlaceBlueprint { kind, x, y } if kind == bed && !replaced_by_other((x, y)) => {
                Some((x, y))
            }
            _ => None,
        })
        .collect();
    if beds.is_empty() {
        return None;
    }
    let boundary: BTreeSet<_> = owned
        .clone()
        .filter_map(|order| match order.command {
            Command::PlaceBlueprint { kind, x, y }
                if (kind == wall || kind == door) && !replaced_by_other((x, y)) =>
            {
                Some((x, y))
            }
            _ => None,
        })
        .collect();
    for room in res.colony.rooms.values() {
        if !room.roofed || room.dug || room.doors.is_empty() || room.role == role {
            continue;
        }
        let finished: Vec<_> = room
            .tiles
            .iter()
            .filter(|tile| res.colony.beds.contains_key(tile))
            .collect();
        if finished.is_empty() || finished.iter().any(|tile| !beds.contains(tile)) {
            continue;
        }
        if res.colony.room_overrides.get(&room.id).is_some_and(|&current| {
            !owned.clone().any(|order| matches!(order.command,
                Command::SetRoomRole { room: id, role: Some(value) } if id == room.id && value == current))
        }) { continue; }
        let built_boundary: BTreeSet<_> = room
            .tiles
            .iter()
            .flat_map(|&(x, y)| {
                [
                    (x.saturating_sub(1), y),
                    (x + 1, y),
                    (x, y.saturating_sub(1)),
                    (x, y + 1),
                ]
            })
            .filter(|&(x, y)| {
                x < res.tile_grid.width
                    && y < res.tile_grid.height
                    && res.tile_grid.wall_material[res.tile_grid.idx(x, y)].is_some()
            })
            .chain(room.doors.iter().copied())
            .collect();
        // Natural terrain may enclose the room, but every built boundary component must be ours.
        if !built_boundary.is_empty() && built_boundary.iter().all(|tile| boundary.contains(tile)) {
            return Some(Command::SetRoomRole {
                room: room.id,
                role: Some(role),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
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
        let Action::House { bed, door, .. } = rule.action else {
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
        for (kind, x, y) in [(bed, 10, 10), (door, 10, 9)] {
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
        e.resources.steward.owned.remove(0);
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
        let (mut e, rule) = fixture();
        let i = e.resources.tile_grid.idx(11, 10);
        e.resources.tile_grid.wall_material[i] = Some(sim_core::colony::materials::OAK);
        assert!(
            plan(&e.resources, &rule).is_none(),
            "one owned door does not own a foreign wall"
        );
    }
    #[test]
    fn later_player_bed_replacement_does_not_inherit_historical_ownership() {
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
}
