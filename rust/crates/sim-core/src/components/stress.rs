use serde::{Deserialize, Serialize};
use crate::enums::{StressState, MentalBreakType};

/// A single per-tick stress contribution that decays over time.
/// Corresponds to GDScript `ed.stress_traces` entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTrace {
    /// Source identifier (e.g. stressor event id)
    pub source_id: String,
    /// Per-tick stress contribution
    pub per_tick: f32,
    /// Decay rate per tick (fraction removed each tick)
    pub decay_rate: f32,
}

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
    /// Resilience score (personality-derived stress resistance)
    pub resilience: f64,
    /// Last tick's stress delta (for tracking direction)
    pub stress_delta_last: f32,
    /// General Adaptation Syndrome stage (Selye 1956): 0=alarm, 1=resistance, 2=exhaustion
    pub gas_stage: i32,
    /// Active per-tick stress traces with decay
    pub stress_traces: Vec<StressTrace>,
    /// Hidden threat accumulator (from C05 Denial coping — Gross 1998)
    pub hidden_threat_accumulator: f32,
    /// Shaken state countdown (ticks remaining)
    pub shaken_remaining: i32,
    /// Work efficiency penalty from being shaken (Yerkes-Dodson)
    pub shaken_work_penalty: f32,
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
            resilience: 0.5,
            stress_delta_last: 0.0,
            gas_stage: 0,
            stress_traces: Vec::new(),
            hidden_threat_accumulator: 0.0,
            shaken_remaining: 0,
            shaken_work_penalty: 0.0,
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
