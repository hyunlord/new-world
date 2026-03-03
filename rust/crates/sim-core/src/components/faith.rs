use serde::{Deserialize, Serialize};

/// Oracle memory entry (for future Phase D)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleMemoryEntry {
    pub interpretation: String,
    pub tick: u64,
    pub confidence: f64,
}

/// Faith / spiritual state (used in Phase D oracle system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faith {
    /// Faith strength (0.0..=1.0)
    pub strength: f64,
    /// Cultural tradition ID (e.g., "animism", "shamanism")
    pub tradition: String,
    /// Is this entity a spiritual leader (priest/shaman)
    pub is_priest: bool,
    /// Memories of oracle interpretations
    pub oracle_memory: Vec<OracleMemoryEntry>,
    /// Number of ritual participations
    pub ritual_count: u32,
}

impl Default for Faith {
    fn default() -> Self {
        Self {
            strength: 0.0,
            tradition: String::new(),
            is_priest: false,
            oracle_memory: Vec::new(),
            ritual_count: 0,
        }
    }
}
