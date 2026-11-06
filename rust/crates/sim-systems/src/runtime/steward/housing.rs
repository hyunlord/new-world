//! Physical housing eligibility for steward demand and completion; natural enclosure is insufficient.
use sim_core::rooms::Room;
use sim_engine::SimResources;

pub(super) fn constructed(res: &SimResources, room: &Room) -> bool {
    if !room.roofed || room.dug || room.tiles.is_empty() || room.doors.is_empty() {
        return false;
    }
    let grid = &res.tile_grid;
    let mut door = false;
    for &(x, y) in &room.tiles {
        if x == 0 || y == 0 || x + 1 >= grid.width || y + 1 >= grid.height {
            return false;
        }
        let index = grid.idx(x, y);
        if grid.wall_material[index].is_some() || grid.is_dug(index) {
            return false;
        }
        for tile in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if room.tiles.contains(&tile) {
                continue;
            }
            let index = grid.idx(tile.0, tile.1);
            if grid.wall_material[index].is_none() {
                return false;
            }
            door |= grid.is_door(index);
        }
    }
    door
}

/// An existing primary-hearth refuge has a real roof, sufficient interior and a door.
/// Ordinary bed-house demand still uses the stricter constructed-wall predicate above.
pub(super) fn hearth_is_sheltered(
    res: &SimResources,
    hearth: (u32, u32),
    minimum_area: u32,
) -> bool {
    res.colony
        .room_of_tile
        .get(&hearth)
        .and_then(|id| res.colony.rooms.get(id))
        .is_some_and(|room| {
            room.roofed
                && room.tiles.contains(&hearth)
                && room.tiles.len() >= minimum_area as usize
                && !room.doors.is_empty()
        })
}
