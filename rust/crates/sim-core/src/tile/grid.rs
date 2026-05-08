//! `TileGrid` — Struct of Arrays (SoA) tile grid.
//!
//! Phase 0 Section 2.2 base. Default 256×256 yields ~65k tiles and ~1.7 MB
//! of hot data, which fits comfortably in modern L3 caches.

use arrayvec::ArrayVec;

use crate::material::{MaterialId, TerrainType};

/// 2D tile grid stored as Struct of Arrays for cache-friendly traversal.
///
/// Each parallel array has length `width * height`. Walls, floors and
/// ground resources are sparse (`Option<MaterialId>`); terrain, elevation
/// and outdoor flags are dense.
pub struct TileGrid {
    /// Width in tiles.
    pub width: u32,
    /// Height in tiles.
    pub height: u32,
    /// Wall material per tile (`None` = no wall).
    pub wall_material: Vec<Option<MaterialId>>,
    /// Floor material per tile (`None` = bare ground).
    pub floor_material: Vec<Option<MaterialId>>,
    /// Resource material exposed at ground level (`None` = no resource).
    pub ground_resource: Vec<Option<MaterialId>>,
    /// Terrain classification per tile (10 variants from `TerrainType`).
    pub terrain_type: Vec<TerrainType>,
    /// Elevation per tile (`u8`, 0..=255). 128 represents nominal sea level.
    pub elevation: Vec<u8>,
    /// Outdoor flag — `true` means the tile is exposed to sky (sun base for
    /// the Light channel, weather exposure base for Warmth, etc.).
    pub outdoor: Vec<bool>,
}

impl TileGrid {
    /// Build a fresh grid of the given dimensions. All tiles default to
    /// `Plain` terrain, mid-elevation, outdoor and no walls/floors/resources.
    pub fn new(width: u32, height: u32) -> Self {
        let total = (width as usize) * (height as usize);
        Self {
            width,
            height,
            wall_material: vec![None; total],
            floor_material: vec![None; total],
            ground_resource: vec![None; total],
            terrain_type: vec![TerrainType::Plain; total],
            elevation: vec![128; total],
            outdoor: vec![true; total],
        }
    }

    /// Linear index into the SoA arrays for the given `(x, y)` coordinate.
    ///
    /// Phase 0 Section 2.2.3 — row-major (`y * width + x`).
    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width, "x {} out of bounds {}", x, self.width);
        debug_assert!(y < self.height, "y {} out of bounds {}", y, self.height);
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Total number of tiles (`width * height`).
    #[inline]
    pub fn len(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// `true` if the grid has zero tiles (degenerate).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// 4-directional neighbours (N, S, E, W) clipped to grid bounds.
    pub fn neighbors_4(&self, x: u32, y: u32) -> ArrayVec<(u32, u32), 4> {
        let mut neighbours = ArrayVec::new();
        if x > 0 {
            neighbours.push((x - 1, y));
        }
        if x + 1 < self.width {
            neighbours.push((x + 1, y));
        }
        if y > 0 {
            neighbours.push((x, y - 1));
        }
        if y + 1 < self.height {
            neighbours.push((x, y + 1));
        }
        neighbours
    }

    /// 8-directional neighbours including diagonals, clipped to grid bounds.
    pub fn neighbors_8(&self, x: u32, y: u32) -> ArrayVec<(u32, u32), 8> {
        let mut neighbours = ArrayVec::new();
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0
                    && (nx as u32) < self.width
                    && ny >= 0
                    && (ny as u32) < self.height
                {
                    neighbours.push((nx as u32, ny as u32));
                }
            }
        }
        neighbours
    }

    /// `true` if the tile at the given linear index has a wall.
    #[inline]
    pub fn is_wall(&self, idx: usize) -> bool {
        self.wall_material[idx].is_some()
    }

    /// `true` if the tile at the given linear index is exposed to the sky.
    #[inline]
    pub fn is_outdoor(&self, idx: usize) -> bool {
        self.outdoor[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idx_correct() {
        let g = TileGrid::new(8, 8);
        assert_eq!(g.idx(0, 0), 0);
        assert_eq!(g.idx(7, 0), 7);
        assert_eq!(g.idx(0, 1), 8);
        assert_eq!(g.idx(7, 7), 63);
    }

    #[test]
    fn test_len_and_is_empty() {
        let g = TileGrid::new(4, 5);
        assert_eq!(g.len(), 20);
        assert!(!g.is_empty());

        let z = TileGrid::new(0, 5);
        assert!(z.is_empty());
    }

    #[test]
    fn test_neighbors_4_corner() {
        let g = TileGrid::new(8, 8);
        let n = g.neighbors_4(0, 0);
        assert_eq!(n.len(), 2);
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(0, 1)));
    }

    #[test]
    fn test_neighbors_4_edge() {
        let g = TileGrid::new(8, 8);
        let n = g.neighbors_4(3, 0);
        assert_eq!(n.len(), 3);
        assert!(n.contains(&(2, 0)));
        assert!(n.contains(&(4, 0)));
        assert!(n.contains(&(3, 1)));
    }

    #[test]
    fn test_neighbors_4_interior() {
        let g = TileGrid::new(8, 8);
        let n = g.neighbors_4(4, 4);
        assert_eq!(n.len(), 4);
        assert!(n.contains(&(3, 4)));
        assert!(n.contains(&(5, 4)));
        assert!(n.contains(&(4, 3)));
        assert!(n.contains(&(4, 5)));
    }

    #[test]
    fn test_neighbors_8_interior() {
        let g = TileGrid::new(8, 8);
        let n = g.neighbors_8(4, 4);
        assert_eq!(n.len(), 8);
    }

    #[test]
    fn test_neighbors_8_corner() {
        let g = TileGrid::new(8, 8);
        let n = g.neighbors_8(0, 0);
        assert_eq!(n.len(), 3);
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(1, 1)));
    }

    #[test]
    fn test_is_wall_default_false() {
        let g = TileGrid::new(4, 4);
        for i in 0..g.len() {
            assert!(!g.is_wall(i));
        }
    }

    #[test]
    fn test_is_wall_after_set() {
        let mut g = TileGrid::new(4, 4);
        let i = g.idx(2, 2);
        g.wall_material[i] = Some(MaterialId::from_str_hash("granite"));
        assert!(g.is_wall(i));
        assert!(!g.is_wall(0));
    }

    #[test]
    fn test_is_outdoor_default_true() {
        let g = TileGrid::new(4, 4);
        for i in 0..g.len() {
            assert!(g.is_outdoor(i));
        }
    }

    #[test]
    fn test_terrain_default_plain() {
        let g = TileGrid::new(3, 3);
        for t in &g.terrain_type {
            assert_eq!(*t, TerrainType::Plain);
        }
    }
}
