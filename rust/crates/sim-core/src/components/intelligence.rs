use serde::{Deserialize, Serialize};
use crate::enums::IntelligenceType;

pub const INTELLIGENCE_COUNT: usize = 8;

/// Gardner 8 intelligences (g-factor + residual model)
/// All values 0.0..=1.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intelligence {
    /// Base values: [linguistic, logical, spatial, musical, kinesthetic, interpersonal, intrapersonal, naturalistic]
    pub values: [f64; INTELLIGENCE_COUNT],
    /// g-factor (general intelligence, 0.0..=1.0)
    pub g_factor: f64,
    /// ACE (Adverse Childhood Experiences) penalty accumulated
    pub ace_penalty: f64,
    /// Cumulative nutrition damage (applied in critical window)
    pub nutrition_penalty: f64,
}

impl Default for Intelligence {
    fn default() -> Self {
        Self {
            values: [0.5; INTELLIGENCE_COUNT],
            g_factor: 0.5,
            ace_penalty: 0.0,
            nutrition_penalty: 0.0,
        }
    }
}

impl Intelligence {
    #[inline]
    pub fn get(&self, t: IntelligenceType) -> f64 {
        self.values[t as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, t: IntelligenceType) -> &mut f64 {
        &mut self.values[t as usize]
    }
}
