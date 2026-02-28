use serde::{Deserialize, Serialize};
use crate::enums::ValueType;

pub const VALUE_COUNT: usize = 33;

/// 33 personal values (-1.0..=+1.0)
/// Positive = value embraced, negative = value rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Values {
    pub values: [f64; VALUE_COUNT],
}

impl Default for Values {
    fn default() -> Self {
        Self { values: [0.0; VALUE_COUNT] }
    }
}

impl Values {
    #[inline]
    pub fn get(&self, v: ValueType) -> f64 {
        self.values[v as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, v: ValueType) -> &mut f64 {
        &mut self.values[v as usize]
    }

    #[inline]
    pub fn set(&mut self, v: ValueType, val: f64) {
        self.values[v as usize] = val.clamp(-1.0, 1.0);
    }

    /// Value alignment score with another Values component (0.0..=1.0)
    pub fn alignment_with(&self, other: &Values) -> f64 {
        let dot: f64 = self.values.iter().zip(other.values.iter())
            .map(|(a, b)| a * b)
            .sum();
        (dot / VALUE_COUNT as f64 + 1.0) / 2.0
    }
}
