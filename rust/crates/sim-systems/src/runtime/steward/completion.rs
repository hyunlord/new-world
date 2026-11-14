//! Pure House role completion from accepted ownership (blueprint orders, steward template sites); manual role intent always wins.
use sim_core::colony::{
    building_site::{BuildCause, PiecePhase},
    content::{installed_or_bundled, CompletionCategory},
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
    let latest_placement = |tile: (u32, u32)| {
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
                    Command::PlaceBlueprint { kind, x, y } if (*x, *y) == tile => {
                        Some((source_rule, *kind))
                    }
                    _ => None,
                }
            })
    };
    let replaced_by_other =
        |tile| latest_placement(tile).is_some_and(|(owner, _)| owner != Some(rule.id));
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
    let store = role == sim_core::rooms::RoomRole::Storehouse;
    if beds.is_empty() && !store {
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
    let mut compatible_boundary = boundary.clone();
    if store {
        if let Ok(data) = sim_core::colony::steward::rules() {
            for order in &res.steward.owned {
                let Command::PlaceBlueprint { kind, x, y } = order.command else {
                    continue;
                };
                if latest_placement((x, y)) != Some((Some(order.rule), kind)) {
                    continue;
                }
                if data.rules.iter().any(|source| {
                    source.id == order.rule
                        && matches!(source.action, Action::House { wall, door, .. }
                        if kind == wall || kind == door)
                }) {
                    compatible_boundary.insert((x, y));
                }
            }
            // ADR 0061 houses are template sites: their completed wall/door pieces count too,
            // unless a later placement replaced the tile.
            for site in res.colony.buildings.sites.values() {
                let BuildCause::Steward(source) = site.cause else {
                    continue;
                };
                if !data.rules.iter().any(|rule| {
                    u32::from(rule.id) == source && matches!(rule.action, Action::House { .. })
                }) {
                    continue;
                }
                for piece in &site.pieces {
                    let tile = piece.tile;
                    if piece.phase == PiecePhase::Complete
                        && installed_or_bundled()
                            .blueprint(piece.kind)
                            .is_some_and(|def| {
                                matches!(
                                    def.completion,
                                    CompletionCategory::Wall | CompletionCategory::Door
                                )
                            })
                        && !res
                            .colony
                            .command_log
                            .iter()
                            .skip(site.source_record as usize)
                            .any(|record| {
                                let command = match &record.command {
                                    Command::Steward { command, .. } => &**command,
                                    command => command,
                                };
                                record.result.accepted
                                    && matches!(command, Command::PlaceBlueprint { x, y, .. } if (*x, *y) == tile)
                            })
                    {
                        compatible_boundary.insert(tile);
                    }
                }
            }
        }
    }
    for room in res.colony.rooms.values() {
        if !super::housing::constructed(res, room)
            || (store
                && !matches!(
                    room.role,
                    sim_core::rooms::RoomRole::Unassigned | sim_core::rooms::RoomRole::Storehouse
                ))
            || res.colony.room_overrides.get(&room.id) == Some(&role)
        {
            continue;
        }
        let finished: Vec<_> = room
            .tiles
            .iter()
            .filter(|tile| res.colony.beds.contains_key(tile))
            .collect();
        if (!store && finished.is_empty())
            || (store && !finished.is_empty())
            || finished.iter().any(|tile| !beds.contains(tile))
        {
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
        // A store may reuse steward house walls (blueprint or template), but must contribute its own real boundary.
        if built_boundary.iter().any(|tile| boundary.contains(tile))
            && built_boundary
                .iter()
                .all(|tile| compatible_boundary.contains(tile))
        {
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
