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
        if !super::housing::constructed(res, room) || room.role == role {
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
        // Every physical boundary component must belong to this steward house.
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
#[path = "completion_tests.rs"]
mod tests;
