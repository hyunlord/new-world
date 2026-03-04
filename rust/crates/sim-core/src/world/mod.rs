pub mod tile;
pub mod resource_map;

pub use tile::{Tile, TileResource};
pub use resource_map::ResourceMap;

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
    fn test_walkable_tiles_near() {
        let mut map = WorldMap::new(10, 10, 0);
        // Default tiles have passable=true
        let tiles = map.walkable_tiles_near(5, 5, 1);
        // Within bounds at (5,5) with radius 1: should be 3x3 = 9 tiles (all in bounds)
        assert!(!tiles.is_empty());
        assert_eq!(tiles.len(), 9);
        assert!(tiles.iter().all(|(x, y)| *x >= 4 && *x <= 6 && *y >= 4 && *y <= 6));
    }
}
