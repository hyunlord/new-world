use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::tile_grid::TileGrid;

/// ECS/shared identifier for one detected room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub u32);

/// Lightweight room role scaffold for future higher-level semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomRole {
    /// No higher-level role has been assigned yet.
    Unknown,
    /// General shelter-like room.
    Shelter,
    /// Hearth or heat-centric room.
    Hearth,
    /// Storage-oriented room.
    Storage,
    /// Crafting-oriented room.
    Crafting,
    /// Ritual/spiritual room: assigned when a totem is the majority furniture.
    Ritual,
}

/// Result of one room-detection pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Room {
    /// Stable room id for this detection pass.
    pub id: RoomId,
    /// Tiles belonging to the room.
    pub tiles: Vec<(u32, u32)>,
    /// Whether the region is enclosed by walls/bounds.
    pub enclosed: bool,
    /// Current role hint for downstream systems.
    pub role: RoomRole,
}

impl Room {
    /// Returns the number of structural floor tiles in the room.
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}

/// Detects orthogonally connected floor regions from a structural tile grid.
pub fn detect_rooms(grid: &TileGrid) -> Vec<Room> {
    let (width, height) = grid.dimensions();
    let mut visited = vec![false; (width * height) as usize];
    let mut rooms = Vec::new();
    let mut next_room_id: u32 = 1;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if visited[idx] || !grid.get(x, y).is_room_floor() {
                continue;
            }

            let mut queue = VecDeque::from([(x, y)]);
            let mut room_tiles = Vec::new();
            let mut enclosed = true;
            visited[idx] = true;

            while let Some((tile_x, tile_y)) = queue.pop_front() {
                room_tiles.push((tile_x, tile_y));
                for (next_x, next_y) in grid.orthogonal_neighbors(tile_x, tile_y) {
                    let next_idx = (next_y * width + next_x) as usize;
                    let next_tile = grid.get(next_x, next_y);
                    if next_tile.is_room_floor() && !visited[next_idx] {
                        visited[next_idx] = true;
                        queue.push_back((next_x, next_y));
                        continue;
                    }
                    if !next_tile.blocks_room_flow() && !next_tile.is_room_floor() {
                        enclosed = false;
                    }
                }

                if tile_x == 0 || tile_y == 0 || tile_x + 1 == width || tile_y + 1 == height {
                    enclosed = false;
                }
            }

            rooms.push(Room {
                id: RoomId(next_room_id),
                tiles: room_tiles,
                enclosed,
                role: if enclosed {
                    RoomRole::Shelter
                } else {
                    RoomRole::Unknown
                },
            });
            next_room_id += 1;
        }
    }

    rooms
}

/// Applies detected room ids back onto the structural grid.
pub fn assign_room_ids(grid: &mut TileGrid, rooms: &[Room]) {
    grid.clear_room_ids();
    for room in rooms {
        for (x, y) in &room.tiles {
            grid.assign_room(*x, *y, room.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_grid::TileGrid;

    fn enclosed_test_grid() -> TileGrid {
        let mut grid = TileGrid::new(5, 5);
        for x in 1..=3 {
            for y in 1..=3 {
                grid.set_floor(x, y, "wood");
            }
        }
        for x in 0..5 {
            grid.set_wall(x, 0, "stone", 10.0);
            grid.set_wall(x, 4, "stone", 10.0);
        }
        for y in 0..5 {
            grid.set_wall(0, y, "stone", 10.0);
            grid.set_wall(4, y, "stone", 10.0);
        }
        grid
    }

    #[test]
    fn room_detection_finds_enclosed_floor_region() {
        let grid = enclosed_test_grid();
        let rooms = detect_rooms(&grid);
        assert_eq!(rooms.len(), 1);
        assert!(rooms[0].enclosed);
        assert_eq!(rooms[0].tile_count(), 9);
    }

    #[test]
    fn room_assignment_writes_room_ids_back_to_grid() {
        let mut grid = enclosed_test_grid();
        let rooms = detect_rooms(&grid);
        assign_room_ids(&mut grid, &rooms);
        assert_eq!(grid.get(2, 2).room_id, Some(RoomId(1)));
    }
}
