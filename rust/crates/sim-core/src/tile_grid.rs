use serde::{Deserialize, Serialize};

use crate::room::RoomId;

/// Structural tile state used for future building and room foundations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StructuralTile {
    /// Optional wall material id on this tile.
    pub wall_material: Option<String>,
    /// Optional floor material id on this tile.
    pub floor_material: Option<String>,
    /// Optional roof material id on this tile.
    pub roof_material: Option<String>,
    /// Wall hit points for structural damage systems.
    pub wall_hp: f64,
    /// Optional detected room id.
    pub room_id: Option<RoomId>,
    /// True if this tile is a door opening. Doors block room flow for
    /// detection purposes (so BFS treats them as boundaries) but can be
    /// distinguished from walls by downstream pathfinding/influence systems.
    #[serde(default)]
    pub is_door: bool,
    /// Optional furniture id placed on this tile (P2-B3 component building).
    /// Walls and furniture can coexist on the same tile (e.g. an interior
    /// fire pit inside a wall ring), so this is independent of `wall_material`.
    #[serde(default)]
    pub furniture_id: Option<String>,
}

impl StructuralTile {
    /// Returns true when this tile currently blocks room traversal.
    /// Walls AND doors both block BFS so enclosed rooms can form without
    /// leaking through the door opening.
    pub fn blocks_room_flow(&self) -> bool {
        self.wall_material.is_some() || self.is_door
    }

    /// Returns true when this tile can participate in room detection.
    pub fn is_room_floor(&self) -> bool {
        self.floor_material.is_some() && !self.blocks_room_flow()
    }
}

/// Shared structural tile grid for room detection and wall metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileGrid {
    width: u32,
    height: u32,
    tiles: Vec<StructuralTile>,
}

impl TileGrid {
    /// Creates an empty structural grid for the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![StructuralTile::default(); (width * height) as usize],
        }
    }

    /// Returns the grid dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns true when the tile coordinate is in bounds.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    /// Returns an immutable structural tile reference.
    pub fn get(&self, x: u32, y: u32) -> &StructuralTile {
        &self.tiles[self.index(x, y)]
    }

    /// Returns a mutable structural tile reference.
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut StructuralTile {
        let idx = self.index(x, y);
        &mut self.tiles[idx]
    }

    /// Clears all room IDs in preparation for a fresh detection pass.
    pub fn clear_room_ids(&mut self) {
        for tile in &mut self.tiles {
            tile.room_id = None;
        }
    }

    /// Clears all structural layers and room ids in place.
    pub fn clear(&mut self) {
        for tile in &mut self.tiles {
            *tile = StructuralTile::default();
        }
    }

    /// Sets the wall material and hp on a tile.
    pub fn set_wall(&mut self, x: u32, y: u32, material_id: impl Into<String>, wall_hp: f64) {
        let tile = self.get_mut(x, y);
        tile.wall_material = Some(material_id.into());
        tile.wall_hp = wall_hp.max(0.0);
    }

    /// Sets the floor material on a tile.
    pub fn set_floor(&mut self, x: u32, y: u32, material_id: impl Into<String>) {
        self.get_mut(x, y).floor_material = Some(material_id.into());
    }

    /// Sets the roof material on a tile.
    pub fn set_roof(&mut self, x: u32, y: u32, material_id: impl Into<String>) {
        self.get_mut(x, y).roof_material = Some(material_id.into());
    }

    /// Marks a tile as a door opening. Doors block room flow (so BFS treats
    /// them as boundaries) without having a wall_material, letting downstream
    /// systems distinguish doors from solid walls.
    pub fn set_door(&mut self, x: u32, y: u32) {
        self.get_mut(x, y).is_door = true;
    }

    /// Sets a furniture id on a tile (P2-B3 component building).
    pub fn set_furniture(&mut self, x: u32, y: u32, furniture_id: impl Into<String>) {
        self.get_mut(x, y).furniture_id = Some(furniture_id.into());
    }

    /// Removes any furniture from a tile. Returns the previous furniture id
    /// so callers can distinguish "nothing to remove" from "successful
    /// removal" when needed. This is the canonical furniture-removal API
    /// used by demolition paths and test fixtures that need to demote a
    /// room's role by destroying the furniture that was voting on it.
    pub fn remove_furniture(&mut self, x: u32, y: u32) -> Option<String> {
        self.get_mut(x, y).furniture_id.take()
    }

    /// Returns the furniture id on a tile, if any.
    pub fn get_furniture(&self, x: u32, y: u32) -> Option<&str> {
        self.get(x, y).furniture_id.as_deref()
    }

    /// Assigns a room id to one tile.
    pub fn assign_room(&mut self, x: u32, y: u32, room_id: RoomId) {
        self.get_mut(x, y).room_id = Some(room_id);
    }

    /// Returns orthogonal neighbors in bounds.
    pub fn orthogonal_neighbors(&self, x: u32, y: u32) -> Vec<(u32, u32)> {
        let mut neighbors = Vec::with_capacity(4);
        for (dx, dy) in [(0_i32, -1_i32), (1, 0), (0, 1), (-1, 0)] {
            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            if self.in_bounds(next_x, next_y) {
                neighbors.push((next_x as u32, next_y as u32));
            }
        }
        neighbors
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_grid_tracks_material_layers_and_room_ids() {
        let mut grid = TileGrid::new(4, 4);
        grid.set_wall(1, 1, "stone", 12.0);
        grid.set_floor(1, 2, "wood");
        grid.assign_room(1, 2, RoomId(4));

        assert_eq!(grid.get(1, 1).wall_material.as_deref(), Some("stone"));
        assert_eq!(grid.get(1, 2).room_id, Some(RoomId(4)));
        assert!(grid.get(1, 2).is_room_floor());
        assert!(grid.get(1, 1).blocks_room_flow());
    }

    #[test]
    fn tile_grid_neighbors_respect_bounds() {
        let grid = TileGrid::new(2, 2);
        let neighbors = grid.orthogonal_neighbors(0, 0);
        assert_eq!(neighbors, vec![(1, 0), (0, 1)]);
    }

    #[test]
    fn tile_grid_clear_resets_structure_layers() {
        let mut grid = TileGrid::new(3, 3);
        grid.set_wall(1, 1, "stone", 12.0);
        grid.set_floor(1, 2, "wood");
        grid.clear();

        assert!(grid.get(1, 1).wall_material.is_none());
        assert!(grid.get(1, 2).floor_material.is_none());
        assert_eq!(grid.get(1, 1).wall_hp, 0.0);
    }
}
