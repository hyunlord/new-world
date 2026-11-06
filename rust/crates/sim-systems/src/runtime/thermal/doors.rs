//! Physical door passage interval union; no navigation mutation.
use sim_core::colony::thermal::{DoorThermal, ThermalState};
use sim_core::rooms::Tile;
use sim_engine::SimResources;

pub(super) fn account(door: &mut DoorThermal, tick: u64) {
    let end = if door.held_open {
        tick
    } else {
        tick.min(door.open_until_tick)
    };
    let elapsed = end.saturating_sub(door.accounted_until_tick);
    door.open_ticks_in_hour += u32::try_from(elapsed).unwrap_or(0);
    door.accounted_until_tick = tick;
}

/// Record actual passage, merging overlapping open intervals.
pub fn note_door_passage(res: &mut SimResources, tile: Tile, tick: u64) {
    if !res.tile_grid.is_door(res.tile_grid.idx(tile.0, tile.1)) {
        return;
    }
    super::initialize(res);
    let Some(state) = res.thermal.as_mut() else {
        return;
    };
    let door = state.doors.entry(tile).or_insert(DoorThermal {
        accounted_until_tick: tick,
        ..DoorThermal::default()
    });
    account(door, tick);
    door.open_until_tick = door
        .open_until_tick
        .max(tick.saturating_add(u64::from(state.rules.door_passage_ticks)));
    state.view_version = state.view_version.wrapping_add(1);
}

pub(super) fn advance(state: &mut ThermalState, tick: u64) {
    for door in state.doors.values_mut() {
        account(door, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_passages_count_union_and_close_without_extra_ticks() {
        let mut door = DoorThermal {
            open_until_tick: 12,
            accounted_until_tick: 10,
            ..DoorThermal::default()
        };
        account(&mut door, 11);
        door.open_until_tick = 13;
        account(&mut door, 11);
        account(&mut door, 20);
        assert_eq!(door.open_ticks_in_hour, 3);
        account(&mut door, 21);
        assert_eq!(door.open_ticks_in_hour, 3);
    }
}
