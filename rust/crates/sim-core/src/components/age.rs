use serde::{Deserialize, Serialize};
use crate::enums::GrowthStage;

/// Current age tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Age {
    /// Age in ticks (authoritative)
    pub ticks: u64,
    /// Age in fractional years (derived, cached)
    pub years: f64,
    /// Growth stage (derived from age ticks)
    pub stage: GrowthStage,
    /// True if entity is alive
    pub alive: bool,
}

impl Default for Age {
    fn default() -> Self {
        Self {
            ticks: 0,
            years: 0.0,
            stage: GrowthStage::Adult,
            alive: true,
        }
    }
}

impl Age {
    /// Update years and stage from ticks (call after advancing ticks)
    /// ticks_per_year = 4380 (12 ticks/day * 365 days/year)
    pub fn update_derived(&mut self, ticks_per_year: u64) {
        self.years = self.ticks as f64 / ticks_per_year as f64;
        self.stage = GrowthStage::from_age_ticks(self.ticks);
    }
}
