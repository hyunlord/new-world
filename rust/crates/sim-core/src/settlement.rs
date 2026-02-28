use serde::{Deserialize, Serialize};
use crate::ids::{SettlementId, EntityId, BuildingId, TechId};
use crate::enums::TechState;
use std::collections::HashMap;

/// Settlement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: SettlementId,
    pub name: String,
    /// Tile position (center)
    pub x: i32,
    pub y: i32,
    /// Entity members
    pub members: Vec<EntityId>,
    /// Buildings in this settlement
    pub buildings: Vec<BuildingId>,
    /// Technology knowledge state: tech_id → TechState
    pub tech_states: HashMap<TechId, TechState>,
    /// Current leader (if any)
    pub leader_id: Option<EntityId>,
    /// Ticks until next leader election
    pub leader_reelection_countdown: u32,
    /// Stockpile resources: resource_type → amount
    pub stockpile_food: f64,
    pub stockpile_wood: f64,
    pub stockpile_stone: f64,
    /// Era label (e.g., "stone_age", "tribal", "bronze_age")
    pub current_era: String,
    /// Founding tick
    pub founded_tick: u64,
    /// Migration cooldown ticks remaining
    pub migration_cooldown: u32,
}

impl Settlement {
    pub fn new(id: SettlementId, name: String, x: i32, y: i32, founded_tick: u64) -> Self {
        Self {
            id,
            name,
            x,
            y,
            members: Vec::new(),
            buildings: Vec::new(),
            tech_states: HashMap::new(),
            leader_id: None,
            leader_reelection_countdown: 0,
            stockpile_food: 0.0,
            stockpile_wood: 0.0,
            stockpile_stone: 0.0,
            current_era: "stone_age".to_string(),
            founded_tick,
            migration_cooldown: 0,
        }
    }

    pub fn population(&self) -> usize {
        self.members.len()
    }

    pub fn has_tech(&self, tech_id: &str) -> bool {
        matches!(
            self.tech_states.get(tech_id),
            Some(TechState::KnownLow) | Some(TechState::KnownStable)
        )
    }
}
