use serde::{Deserialize, Serialize};
use crate::ids::{BuildingId, SettlementId};

/// A constructed building in a settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingId,
    pub building_type: String,  // e.g., "stockpile", "shelter", "campfire"
    pub settlement_id: SettlementId,
    /// Tile position
    pub x: i32,
    pub y: i32,
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
    pub fn new(
        id: BuildingId,
        building_type: String,
        settlement_id: SettlementId,
        x: i32,
        y: i32,
        started_tick: u64,
    ) -> Self {
        Self {
            id,
            building_type,
            settlement_id,
            x,
            y,
            construction_progress: 0.0,
            is_complete: false,
            construction_started_tick: started_tick,
            condition: 1.0,
        }
    }
}
