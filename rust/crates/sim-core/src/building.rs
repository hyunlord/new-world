use crate::ids::{BuildingId, SettlementId};
use serde::{Deserialize, Serialize};

/// A constructed building in a settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingId,
    pub building_type: String, // e.g., "stockpile", "shelter", "campfire"
    pub settlement_id: SettlementId,
    /// Tile position (top-left corner of the footprint)
    pub x: i32,
    pub y: i32,
    /// Footprint width in tiles (minimum 1)
    pub width: u32,
    /// Footprint height in tiles (minimum 1)
    pub height: u32,
    /// Construction progress (0.0..=1.0; 1.0 = complete)
    pub construction_progress: f32,
    /// Is fully built and functional
    pub is_complete: bool,
    /// Tick when construction started
    pub construction_started_tick: u64,
    /// Condition / durability (0.0..=1.0)
    pub condition: f32,
}

impl Building {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BuildingId,
        building_type: String,
        settlement_id: SettlementId,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        started_tick: u64,
    ) -> Self {
        Self {
            id,
            building_type,
            settlement_id,
            x,
            y,
            width: width.max(1),
            height: height.max(1),
            construction_progress: 0.0,
            is_complete: false,
            construction_started_tick: started_tick,
            condition: 1.0,
        }
    }

    /// Returns true if this building's footprint contains the given tile.
    pub fn occupies_tile(&self, tile_x: i32, tile_y: i32) -> bool {
        tile_x >= self.x
            && tile_x < self.x + self.width as i32
            && tile_y >= self.y
            && tile_y < self.y + self.height as i32
    }

    /// Returns true if this building's footprint overlaps the given rectangle.
    pub fn overlaps(&self, other_x: i32, other_y: i32, other_w: u32, other_h: u32) -> bool {
        self.x < other_x + other_w as i32
            && self.x + self.width as i32 > other_x
            && self.y < other_y + other_h as i32
            && self.y + self.height as i32 > other_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn campfire() -> Building {
        Building::new(
            BuildingId(1),
            "campfire".to_string(),
            SettlementId(1),
            5,
            5,
            1,
            1,
            0,
        )
    }

    fn shelter() -> Building {
        Building::new(
            BuildingId(2),
            "shelter".to_string(),
            SettlementId(1),
            8,
            8,
            2,
            2,
            0,
        )
    }

    #[test]
    fn building_new_enforces_min_dimensions() {
        let b = Building::new(BuildingId(3), "x".to_string(), SettlementId(1), 0, 0, 0, 0, 0);
        assert_eq!(b.width, 1);
        assert_eq!(b.height, 1);
    }

    #[test]
    fn occupies_tile_single() {
        let b = campfire();
        assert!(b.occupies_tile(5, 5));
        assert!(!b.occupies_tile(6, 5));
        assert!(!b.occupies_tile(4, 5));
    }

    #[test]
    fn occupies_tile_multi() {
        let b = shelter();
        assert!(b.occupies_tile(8, 8));
        assert!(b.occupies_tile(9, 8));
        assert!(b.occupies_tile(8, 9));
        assert!(b.occupies_tile(9, 9));
        assert!(!b.occupies_tile(10, 8));
        assert!(!b.occupies_tile(8, 10));
        assert!(!b.occupies_tile(7, 8));
    }

    #[test]
    fn overlaps_adjacent_does_not_overlap() {
        let b = shelter(); // (8,8) 2x2 → occupies (8..10, 8..10)
        // Building at (10,8) is adjacent — no overlap
        assert!(!b.overlaps(10, 8, 2, 2));
        assert!(!b.overlaps(8, 10, 2, 2));
        assert!(!b.overlaps(6, 8, 2, 2));
    }

    #[test]
    fn overlaps_same_position() {
        let b = shelter();
        assert!(b.overlaps(8, 8, 2, 2));
    }

    #[test]
    fn overlaps_partial() {
        let b = shelter(); // (8,8) 2x2
        assert!(b.overlaps(9, 9, 2, 2));
        assert!(b.overlaps(7, 7, 2, 2));
    }
}
