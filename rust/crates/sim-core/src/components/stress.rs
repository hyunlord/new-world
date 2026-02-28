use serde::{Deserialize, Serialize};
use crate::enums::{StressState, MentalBreakType};

/// Lazarus stress appraisal model + McEwen allostatic load
/// Stress level 0.0..=1.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stress {
    /// Current stress level (0.0..=1.0)
    pub level: f64,
    /// Stress reserve (capacity for more stress; starts at 1.0)
    pub reserve: f64,
    /// Allostatic load (cumulative wear — hard to recover, McEwen 1998)
    pub allostatic_load: f64,
    /// Current stress phase
    pub state: StressState,
    /// Active mental break (if any)
    pub active_mental_break: Option<MentalBreakType>,
    /// Ticks remaining in current mental break
    pub mental_break_remaining: u32,
    /// Total mental breaks this lifetime
    pub mental_break_count: u32,
}

impl Default for Stress {
    fn default() -> Self {
        Self {
            level: 0.0,
            reserve: 1.0,
            allostatic_load: 0.0,
            state: StressState::Calm,
            active_mental_break: None,
            mental_break_remaining: 0,
            mental_break_count: 0,
        }
    }
}

impl Stress {
    /// Recalculate stress state from current level
    /// Thresholds from GameConfig: alert=0.3, resistance=0.5, exhaustion=0.7, collapse=0.9
    pub fn recalculate_state(&mut self) {
        self.state = if self.level >= 0.9 {
            StressState::Collapse
        } else if self.level >= 0.7 {
            StressState::Exhaustion
        } else if self.level >= 0.5 {
            StressState::Resistance
        } else if self.level >= 0.3 {
            StressState::Alert
        } else {
            StressState::Calm
        };
    }

    pub fn is_in_mental_break(&self) -> bool {
        self.active_mental_break.is_some()
    }
}
