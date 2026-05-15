//! `Thirst` need component (V7 Phase 5-β / P5β-1).
//!
//! Second of the daily-routine need components, mirroring the
//! [`crate::components::Hunger`] contract introduced in Phase 5-α:
//! scalar `value` in `[0.0, SATURATION]` + per-tick `growth_rate`,
//! advanced by `ThirstDecaySystem` (priority 131) in `sim-systems`.
//!
//! # Numeric type
//!
//! `value` / `growth_rate` are `f64` to comply with the project-wide
//! "ALL f64 for simulation math (determinism)" rule defined in the root
//! `CLAUDE.md`. The α-shipped [`crate::components::Hunger`] still uses
//! `f32` because changing it is out of scope for β; the f64/f32 split
//! between the two needs is intentional and tracked for a future
//! α-cleanup pass.
//!
//! Kept structurally identical to `Hunger` so the future "consume" effect
//! (Phase 5-β `Consuming { Water }` FSM transition) can use a single
//! decrement helper for both channels without bespoke per-need logic.

use serde::{Deserialize, Serialize};

/// Per-agent thirst state.
///
/// `value` is the current thirst level (0 = quenched,
/// [`Thirst::SATURATION`] = fully parched). `growth_rate` is the per-tick
/// increment applied by `ThirstDecaySystem`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Thirst {
    /// Current thirst level. Always within `[0.0, SATURATION]`.
    pub value: f64,
    /// Per-tick growth, added to `value` by `ThirstDecaySystem::tick`.
    pub growth_rate: f64,
}

impl Thirst {
    /// Hard upper bound on `value`. Mirrors [`crate::components::Hunger::SATURATION`]
    /// but in `f64` per the project's simulation-math rule.
    pub const SATURATION: f64 = 100.0;

    /// Construct a new `Thirst` with `value` clamped to
    /// `[0.0, SATURATION]`. Out-of-range inputs are silently clamped.
    pub fn new(initial: f64, growth_rate: f64) -> Self {
        Self {
            value: initial.clamp(0.0, Self::SATURATION),
            growth_rate,
        }
    }

    /// Advance one tick: `value = min(value + growth_rate, SATURATION)`,
    /// then clamp negative results to `0.0` so a future "drink" action
    /// modelled via negative growth_rate cannot underflow.
    pub fn tick(&mut self) {
        self.value = (self.value + self.growth_rate).min(Self::SATURATION);
        if self.value < 0.0 {
            self.value = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_constant_is_100() {
        assert_eq!(Thirst::SATURATION, 100.0_f64);
    }

    #[test]
    fn new_clamps_negative_initial() {
        let t = Thirst::new(-5.0, 1.0);
        assert_eq!(t.value, 0.0);
        assert_eq!(t.growth_rate, 1.0);
    }

    #[test]
    fn new_clamps_over_saturation_initial() {
        let t = Thirst::new(250.0, 1.0);
        assert_eq!(t.value, Thirst::SATURATION);
    }

    #[test]
    fn tick_adds_growth_rate() {
        let mut t = Thirst::new(0.0, 3.0);
        t.tick();
        assert_eq!(t.value, 3.0);
        t.tick();
        assert_eq!(t.value, 6.0);
    }

    #[test]
    fn tick_clamps_at_saturation() {
        let mut t = Thirst::new(99.0, 5.0);
        t.tick();
        assert_eq!(t.value, Thirst::SATURATION);
        for _ in 0..10 {
            t.tick();
        }
        assert_eq!(t.value, Thirst::SATURATION);
    }

    #[test]
    fn serde_round_trip() {
        let t = Thirst::new(42.5, 1.5);
        let encoded = ron::to_string(&t).unwrap();
        let decoded: Thirst = ron::from_str(&encoded).unwrap();
        assert_eq!(t, decoded);
    }
}
