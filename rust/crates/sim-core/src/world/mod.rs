pub mod resource_map;
pub mod tile;

pub use resource_map::ResourceMap;
pub use tile::{Tile, TileResource};

use crate::enums::{ResourceType, TerrainType};
use serde::{Deserialize, Serialize};

/// The full world map (flat array, 256×256 by default)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMap {
    pub width: u32,
    pub height: u32,
    /// Flat tile array. Index = y * width + x
    tiles: Vec<Tile>,
    /// World generation seed
    pub seed: u64,
}

impl WorldMap {
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::default(); (width * height) as usize],
            seed,
        }
    }

    #[inline]
    pub fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> &Tile {
        &self.tiles[self.index(x, y)]
    }

    #[inline]
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        let idx = self.index(x, y);
        &mut self.tiles[idx]
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    /// Returns `true` if there is stone access within `radius` tiles of (cx, cy).
    ///
    /// Stone access means: a Hill or Mountain terrain tile, or a tile with a
    /// `TileResource::Stone` entry with `amount > 0`.
    pub fn has_stone_access(&self, cx: i32, cy: i32, radius: i32) -> bool {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = cx + dx;
                let y = cy + dy;
                if !self.in_bounds(x, y) {
                    continue;
                }
                let tile = self.get(x as u32, y as u32);
                if matches!(tile.terrain, TerrainType::Hill | TerrainType::Mountain) {
                    return true;
                }
                if tile
                    .resources
                    .iter()
                    .any(|r| r.resource_type == ResourceType::Stone && r.amount > 0.0)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Returns a passable settlement spawn point with stone access within `stone_radius`.
    ///
    /// If `(preferred_x, preferred_y)` already has stone access, it is returned unchanged.
    /// Otherwise performs a perimeter-expanding spiral search up to radius 100.
    /// Falls back to the original point if no valid location is found.
    pub fn find_settlement_location_with_stone(
        &self,
        preferred_x: i32,
        preferred_y: i32,
        stone_radius: i32,
    ) -> (i32, i32) {
        if self.has_stone_access(preferred_x, preferred_y, stone_radius) {
            return (preferred_x, preferred_y);
        }
        for r in 1..=100_i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let x = preferred_x + dx;
                    let y = preferred_y + dy;
                    if !self.in_bounds(x, y) {
                        continue;
                    }
                    let tile = self.get(x as u32, y as u32);
                    if !tile.passable {
                        continue;
                    }
                    if self.has_stone_access(x, y, stone_radius) {
                        return (x, y);
                    }
                }
            }
        }
        (preferred_x, preferred_y)
    }

    /// Returns all walkable tile positions within `radius` tiles of (cx, cy).
    ///
    /// Searches a square of side (2*radius+1) centered on (cx, cy).
    /// Only includes tiles that are in-bounds and have `passable == true`.
    pub fn walkable_tiles_near(&self, cx: i32, cy: i32, radius: i32) -> Vec<(i32, i32)> {
        let mut result = Vec::new();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = cx + dx;
                let y = cy + dy;
                if self.in_bounds(x, y) {
                    let tile = self.get(x as u32, y as u32);
                    if tile.passable {
                        result.push((x, y));
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ResourceType, TerrainType};

    #[test]
    fn test_world_map_creation() {
        let world = WorldMap::new(256, 256, 42);
        assert_eq!(world.tile_count(), 65_536);
        assert!(world.in_bounds(0, 0));
        assert!(world.in_bounds(255, 255));
        assert!(!world.in_bounds(256, 0));
    }

    #[test]
    fn test_tile_access() {
        let mut world = WorldMap::new(10, 10, 0);
        world.get_mut(5, 5).elevation = 0.8;
        assert!((world.get(5, 5).elevation - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_has_stone_access_hill_terrain() {
        let mut map = WorldMap::new(10, 10, 0);
        map.get_mut(5, 5).terrain = TerrainType::Hill;
        assert!(map.has_stone_access(5, 5, 1));
        assert!(map.has_stone_access(4, 4, 2));
        assert!(!map.has_stone_access(0, 0, 1));
    }

    #[test]
    fn test_has_stone_access_tile_resource() {
        let mut map = WorldMap::new(10, 10, 0);
        map.get_mut(3, 3).resources.push(TileResource {
            resource_type: ResourceType::Stone,
            amount: 50.0,
            max_amount: 50.0,
            regen_rate: 0.0,
        });
        assert!(map.has_stone_access(3, 3, 1));
        assert!(!map.has_stone_access(0, 0, 1));
    }

    #[test]
    fn test_find_settlement_location_preferred_valid() {
        let mut map = WorldMap::new(20, 20, 0);
        map.get_mut(10, 10).terrain = TerrainType::Hill;
        let (x, y) = map.find_settlement_location_with_stone(10, 10, 5);
        assert_eq!((x, y), (10, 10));
    }

    #[test]
    fn test_find_settlement_location_shifts_when_no_local_stone() {
        let mut map = WorldMap::new(50, 50, 0);
        // Place Hill at (40, 25) — far from preferred (25, 25) but within search range
        map.get_mut(40, 25).terrain = TerrainType::Hill;
        let (x, y) = map.find_settlement_location_with_stone(25, 25, 3);
        // Must NOT return (25, 25) — no stone there within radius 3
        assert!((x, y) != (25, 25) || map.has_stone_access(25, 25, 3));
    }

    #[test]
    fn test_walkable_tiles_near() {
        let map = WorldMap::new(10, 10, 0);
        // Default tiles have passable=true
        let tiles = map.walkable_tiles_near(5, 5, 1);
        // Within bounds at (5,5) with radius 1: should be 3x3 = 9 tiles (all in bounds)
        assert!(!tiles.is_empty());
        assert_eq!(tiles.len(), 9);
        assert!(tiles
            .iter()
            .all(|(x, y)| *x >= 4 && *x <= 6 && *y >= 4 && *y <= 6));
    }
}
