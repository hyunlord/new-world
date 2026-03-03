use serde::{Deserialize, Serialize};

/// Economic tendencies and wealth (Layer 4.7)
/// Based on Kahneman & Tversky (1979), Modigliani (1966), Piff (2010)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Economic {
    /// Wealth score (0.0..=1.0, derived from resources)
    pub wealth: f64,
    /// Settlement-relative wealth normalization (0.0..=1.0)
    #[serde(default)]
    pub wealth_norm: f64,
    /// Saving tendency (0.0..=1.0)
    pub saving_tendency: f64,
    /// Risk appetite (0.0..=1.0)
    pub risk_appetite: f64,
    /// Generosity (0.0..=1.0)
    pub generosity: f64,
    /// Materialism (0.0..=1.0)
    pub materialism: f64,
}

impl Default for Economic {
    fn default() -> Self {
        Self {
            wealth: 0.0,
            wealth_norm: 0.0,
            saving_tendency: 0.5,
            risk_appetite: 0.5,
            generosity: 0.5,
            materialism: 0.3,
        }
    }
}
