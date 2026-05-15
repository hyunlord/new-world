//! `Sleep` need component (V7 Phase 5-γ / P5γ-1).
//!
//! Third of the daily-routine need components, mirroring the
//! [`crate::components::Thirst`] `f64` contract introduced in Phase 5-β
//! exactly: scalar `fatigue` in `[0.0, SATURATION]` + per-tick
//! `growth_rate`, advanced by `SleepDecaySystem` (priority 132) in
//! `sim-systems`.
//!
//! # Numeric type
//!
//! `fatigue` / `growth_rate` are `f64` to comply with the project-wide
//! "ALL f64 for simulation math (determinism)" rule. The structurally
//! identical `Thirst` component already follows this convention; γ keeps
//! the symmetry so the consume effect in `AgentDecisionSystem` can treat
//! all three needs uniformly (modulo the per-need field name).

use serde::{Deserialize, Serialize};

/// Per-agent sleep state.
///
/// `fatigue` is the current fatigue level (0 = fully rested,
/// [`Sleep::SATURATION`] = exhausted). `growth_rate` is the per-tick
/// increment applied by `SleepDecaySystem`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sleep {
    /// Current fatigue level. Always within `[0.0, SATURATION]`.
    pub fatigue: f64,
    /// Per-tick growth, added to `fatigue` by `SleepDecaySystem::tick`.
    pub growth_rate: f64,
}

impl Sleep {
    /// Hard upper bound on `fatigue`. Mirrors
    /// [`crate::components::Thirst::SATURATION`].
    pub const SATURATION: f64 = 100.0;

    /// Construct a new `Sleep` with `fatigue` clamped to
    /// `[0.0, SATURATION]`. Out-of-range inputs are silently clamped.
    pub fn new(initial: f64, growth_rate: f64) -> Self {
        Self {
            fatigue: initial.clamp(0.0, Self::SATURATION),
            growth_rate,
        }
    }

    /// Advance one tick: `fatigue = min(fatigue + growth_rate, SATURATION)`,
    /// then clamp negative results to `0.0` so a "sleep" action modelled
    /// via negative growth_rate cannot underflow.
    pub fn tick(&mut self) {
        self.fatigue = (self.fatigue + self.growth_rate).min(Self::SATURATION);
        if self.fatigue < 0.0 {
            self.fatigue = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_constant_is_100() {
        assert_eq!(Sleep::SATURATION, 100.0_f64);
    }

    #[test]
    fn new_clamps_negative_initial() {
        let s = Sleep::new(-5.0, 1.0);
        assert_eq!(s.fatigue, 0.0);
        assert_eq!(s.growth_rate, 1.0);
    }

    #[test]
    fn new_clamps_over_saturation_initial() {
        let s = Sleep::new(250.0, 1.0);
        assert_eq!(s.fatigue, Sleep::SATURATION);
    }

    #[test]
    fn tick_adds_growth_rate() {
        let mut s = Sleep::new(0.0, 3.0);
        s.tick();
        assert_eq!(s.fatigue, 3.0);
        s.tick();
        assert_eq!(s.fatigue, 6.0);
    }

    #[test]
    fn tick_clamps_at_saturation() {
        let mut s = Sleep::new(99.0, 5.0);
        s.tick();
        assert_eq!(s.fatigue, Sleep::SATURATION);
        for _ in 0..10 {
            s.tick();
        }
        assert_eq!(s.fatigue, Sleep::SATURATION);
    }

    #[test]
    fn serde_round_trip() {
        let s = Sleep::new(42.5, 0.7);
        let encoded = ron::to_string(&s).unwrap();
        let decoded: Sleep = ron::from_str(&encoded).unwrap();
        assert_eq!(s, decoded);
    }
}
