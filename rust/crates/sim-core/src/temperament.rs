use serde::{Deserialize, Serialize};

use crate::components::Personality;
use crate::config;

/// Four-axis TCI temperament values used by shared runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemperamentAxes {
    /// Novelty seeking.
    pub ns: f64,
    /// Harm avoidance.
    pub ha: f64,
    /// Reward dependence.
    pub rd: f64,
    /// Persistence.
    pub p: f64,
}

impl TemperamentAxes {
    /// Returns a copy with all axis values clamped into `[0.0, 1.0]`.
    pub fn clamped(self) -> Self {
        Self {
            ns: self.ns.clamp(0.0, 1.0),
            ha: self.ha.clamp(0.0, 1.0),
            rd: self.rd.clamp(0.0, 1.0),
            p: self.p.clamp(0.0, 1.0),
        }
    }
}

impl Default for TemperamentAxes {
    fn default() -> Self {
        Self {
            ns: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
            ha: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
            rd: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
            p: config::TEMPERAMENT_DEFAULT_AXIS_VALUE,
        }
    }
}

/// Shared ECS temperament component derived from genes/personality inputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Temperament {
    /// Minimal polygenic core used by the scaffold.
    pub genes: [f64; 4],
    /// Latent temperament axes.
    pub latent: TemperamentAxes,
    /// Expressed temperament axes used by runtime systems.
    pub expressed: TemperamentAxes,
    /// Whether a latent/expressed divergence is currently unlocked.
    pub awakened: bool,
}

impl Temperament {
    /// Derives a temperament scaffold from the current HEXACO personality state.
    pub fn from_personality(personality: &Personality) -> Self {
        let latent = TemperamentAxes {
            ns: ((personality.axes[5] + personality.axes[2]) * 0.5).clamp(0.0, 1.0),
            ha: personality.axes[1].clamp(0.0, 1.0),
            rd: ((personality.axes[2] + personality.axes[3]) * 0.5).clamp(0.0, 1.0),
            p: personality.axes[4].clamp(0.0, 1.0),
        };
        Self {
            genes: [latent.ns, latent.ha, latent.rd, latent.p],
            latent,
            expressed: latent,
            awakened: false,
        }
    }

    /// Applies one axis delta and keeps the component within valid bounds.
    pub fn apply_shift(&mut self, ns: f64, ha: f64, rd: f64, p: f64) {
        self.expressed = TemperamentAxes {
            ns: self.expressed.ns + ns,
            ha: self.expressed.ha + ha,
            rd: self.expressed.rd + rd,
            p: self.expressed.p + p,
        }
        .clamped();
        self.awakened = self.expressed != self.latent;
    }

    /// Returns a locale key for the current high-level temperament label.
    pub fn archetype_label_key(&self) -> &'static str {
        let axes = self.expressed;
        if axes.ns >= 0.6 && axes.ha < 0.5 {
            "TEMPERAMENT_SANGUINE"
        } else if axes.ns >= 0.6 && axes.ha >= 0.5 {
            "TEMPERAMENT_CHOLERIC"
        } else if axes.ns < 0.5 && axes.ha >= 0.6 {
            "TEMPERAMENT_MELANCHOLIC"
        } else {
            "TEMPERAMENT_PHLEGMATIC"
        }
    }
}

impl Default for Temperament {
    fn default() -> Self {
        Self {
            genes: [config::TEMPERAMENT_DEFAULT_AXIS_VALUE; 4],
            latent: TemperamentAxes::default(),
            expressed: TemperamentAxes::default(),
            awakened: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperament_from_personality_maps_hexaco_axes() {
        let personality = Personality {
            axes: [0.4, 0.8, 0.7, 0.6, 0.9, 0.5],
            facets: [0.5; 24],
        };

        let temperament = Temperament::from_personality(&personality);

        assert!(temperament.expressed.ha > 0.7);
        assert!(temperament.expressed.p > 0.8);
    }

    #[test]
    fn temperament_shift_clamps_and_sets_awakened() {
        let mut temperament = Temperament::default();
        temperament.apply_shift(0.6, 0.0, 0.0, 0.0);
        assert_eq!(temperament.expressed.ns, 1.0);
        assert!(temperament.awakened);
    }
}
