use serde::{Deserialize, Serialize};
use crate::enums::NeedType;

pub const NEED_COUNT: usize = 13;

/// 13 needs (Maslow + Alderfer ERG model)
/// All values 0.0..=1.0 (1.0 = fully satisfied)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    /// Need values indexed by NeedType
    pub values: [f64; NEED_COUNT],
    /// Energy level (0.0..=1.0) — separate from needs
    pub energy: f64,
    /// Starvation grace tick counter (0 = no grace, counts down to death)
    pub starvation_grace_ticks: i32,
    /// ERG frustration state tracking
    pub growth_frustration_ticks: u32,
    pub relatedness_frustration_ticks: u32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            values: [1.0; NEED_COUNT],
            energy: 1.0,
            starvation_grace_ticks: 0,
            growth_frustration_ticks: 0,
            relatedness_frustration_ticks: 0,
        }
    }
}

impl Needs {
    #[inline]
    pub fn get(&self, n: NeedType) -> f64 {
        self.values[n as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, n: NeedType) -> &mut f64 {
        &mut self.values[n as usize]
    }

    #[inline]
    pub fn set(&mut self, n: NeedType, v: f64) {
        self.values[n as usize] = v.clamp(0.0, 1.0);
    }

    /// True if any existence need (hunger/thirst/warmth/safety) is critically low
    pub fn is_existence_critical(&self) -> bool {
        self.values[NeedType::Hunger as usize] < 0.15
            || self.values[NeedType::Thirst as usize] < 0.15
            || self.values[NeedType::Warmth as usize] < 0.10
            || self.values[NeedType::Safety as usize] < 0.15
    }
}
