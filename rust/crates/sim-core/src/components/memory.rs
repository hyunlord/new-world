use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub event_type: String,
    pub target_id: Option<u64>,
    pub tick: u64,
    /// Initial encoding intensity (0.0..=1.0)
    pub intensity: f64,
    /// Decayed current intensity
    pub current_intensity: f64,
    pub is_permanent: bool,
}

/// A trauma scar (persistent psychological wound)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaScar {
    pub scar_id: String,
    pub acquired_tick: u64,
    /// Severity (0.0..=1.0)
    pub severity: f64,
    /// How many times this scar has been reactivated
    pub reactivation_count: u32,
}

/// Memory component (Baddeley & Hitch 1974, Ebbinghaus 1885)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Short-term working memory (max MEMORY_WORKING_MAX=100 entries)
    pub short_term: VecDeque<MemoryEntry>,
    /// Permanent history (intensity > MEMORY_PERMANENT_THRESHOLD=0.5)
    pub permanent: Vec<MemoryEntry>,
    /// Trauma scars
    pub trauma_scars: Vec<TraumaScar>,
    /// Tick of last memory compression
    pub last_compression_tick: u64,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            short_term: VecDeque::with_capacity(100),
            permanent: Vec::new(),
            trauma_scars: Vec::new(),
            last_compression_tick: 0,
        }
    }
}

impl Memory {
    pub fn scar_count(&self) -> usize {
        self.trauma_scars.len()
    }
}
