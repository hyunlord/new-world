use crate::enums::EmotionType;
use serde::{Deserialize, Serialize};

pub const EMOTION_COUNT: usize = 8;

/// Plutchik 8-emotion model
/// All values 0.0..=1.0 (intensity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emotion {
    /// Current emotion intensities
    pub primary: [f64; EMOTION_COUNT],
    /// Baseline emotion levels (personality-derived)
    pub baseline: [f64; EMOTION_COUNT],
}

impl Default for Emotion {
    fn default() -> Self {
        Self {
            primary: [0.0; EMOTION_COUNT],
            baseline: [0.0; EMOTION_COUNT],
        }
    }
}

impl Emotion {
    #[inline]
    pub fn get(&self, e: EmotionType) -> f64 {
        self.primary[e as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, e: EmotionType) -> &mut f64 {
        &mut self.primary[e as usize]
    }

    #[inline]
    pub fn baseline(&self, e: EmotionType) -> f64 {
        self.baseline[e as usize]
    }

    pub fn add(&mut self, e: EmotionType, delta: f64) {
        self.primary[e as usize] = (self.primary[e as usize] + delta).clamp(0.0, 1.0);
    }
}
