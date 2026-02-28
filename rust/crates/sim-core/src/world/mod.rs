pub mod tile;
pub mod resource_map;

pub use tile::{Tile, TileResource};
pub use resource_map::ResourceMap;

use serde::{Deserialize, Serialize};
use crate::world::tile::Tile;

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
}
