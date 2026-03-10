use crate::enums::{ResourceType, TerrainType};
use serde::{Deserialize, Serialize};

/// Resource deposit on a tile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileResource {
    pub resource_type: ResourceType,
    pub amount: f64,
    pub max_amount: f64,
    pub regen_rate: f64,
}

/// A single world tile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: TerrainType,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
    pub resources: Vec<TileResource>,
    pub passable: bool,
    /// Movement cost multiplier (0.0 = impassable, 1.0 = normal)
    pub move_cost: f32,
    /// Settlement ID on this tile (if any)
    pub settlement_id: Option<u64>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: TerrainType::Grassland,
            elevation: 0.5,
            moisture: 0.5,
            temperature: 0.5,
            resources: Vec::new(),
            passable: true,
            move_cost: 1.0,
            settlement_id: None,
        }
    }
}
