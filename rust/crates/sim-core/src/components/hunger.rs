//! `Hunger` need component (V7 Phase 5-α / P5α-3).
//!
//! First of the daily-routine need components. Holds a scalar `value` in
//! `[0.0, SATURATION]` and a per-tick `growth_rate`. The
//! `HungerDecaySystem` in `sim-systems` calls [`Hunger::tick`] on every
//! entity carrying this component, every tick.
//!
//! `SATURATION` is the hard upper bound; once `value` reaches it,
//! further ticks are no-ops until something external (a "eat" action in
//! Phase 5-β) reduces it.

use serde::{Deserialize, Serialize};

/// Per-agent hunger state.
///
/// `value` is the current hunger level (0 = sated, [`Hunger::SATURATION`]
/// = fully starved). `growth_rate` is the per-tick increment applied by
/// `HungerDecaySystem`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hunger {
    /// Current hunger level. Always within `[0.0, SATURATION]`.
    pub value: f32,
    /// Per-tick growth, added to `value` by `HungerDecaySystem::tick`.
    pub growth_rate: f32,
}

impl Hunger {
    /// Hard upper bound on `value`. Phase 5-α / P5α-3 contract.
    pub const SATURATION: f32 = 100.0;

    /// Construct a new `Hunger` with `value` clamped to
    /// `[0.0, SATURATION]`. Out-of-range inputs are silently clamped
    /// rather than panicking — needs components are spawned from data
    /// files in Phase 5-β onward and a bad value should not crash.
    pub fn new(initial: f32, growth_rate: f32) -> Self {
        Self {
            value: initial.clamp(0.0, Self::SATURATION),
            growth_rate,
        }
    }

    /// Advance one tick: `value = min(value + growth_rate, SATURATION)`.
    ///
    /// `growth_rate` is taken as authoritative (no clamp). Negative
    /// growth rates are allowed in principle (a future "eating" effect
    /// could be modelled this way) but currently expected to be ≥ 0.
    pub fn tick(&mut self) {
        self.value = (self.value + self.growth_rate).min(Self::SATURATION);
        // Clamp lower bound too, in case growth_rate is negative.
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
        assert_eq!(Hunger::SATURATION, 100.0_f32);
    }

    #[test]
    fn new_clamps_negative_initial() {
        let h = Hunger::new(-10.0, 1.0);
        assert_eq!(h.value, 0.0);
        assert_eq!(h.growth_rate, 1.0);
    }

    #[test]
    fn new_clamps_over_saturation_initial() {
        let h = Hunger::new(250.0, 1.0);
        assert_eq!(h.value, Hunger::SATURATION);
    }

    #[test]
    fn tick_adds_growth_rate() {
        let mut h = Hunger::new(0.0, 2.5);
        h.tick();
        assert_eq!(h.value, 2.5);
        h.tick();
        assert_eq!(h.value, 5.0);
    }

    #[test]
    fn tick_clamps_at_saturation() {
        let mut h = Hunger::new(99.0, 5.0);
        h.tick();
        assert_eq!(h.value, Hunger::SATURATION);
        for _ in 0..10 {
            h.tick();
        }
        assert_eq!(h.value, Hunger::SATURATION);
    }

    #[test]
    fn serde_round_trip() {
        let h = Hunger::new(42.5, 1.5);
        let encoded = ron::to_string(&h).unwrap();
        let decoded: Hunger = ron::from_str(&encoded).unwrap();
        assert_eq!(h, decoded);
    }
}
