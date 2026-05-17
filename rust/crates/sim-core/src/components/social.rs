//! `Social` need component (V7 Phase 7-α / P7α-1).
//!
//! First multi-agent need component. Tracks per-agent `loneliness` as a
//! direct structural mirror of [`crate::components::Sleep`] (Phase 5-γ /
//! P5γ-1): scalar `loneliness` in `[0.0, SATURATION]` + per-tick
//! `growth_rate`. Consumed by the Phase 7-β `SocialInteractionSystem`
//! (priority 134) — α ships only the data substrate.
//!
//! # Numeric type
//!
//! `loneliness` / `growth_rate` are `f64` per the project-wide "ALL f64
//! for simulation math (determinism)" rule. Constructor mirrors
//! `Sleep::new` exactly: `initial` is clamped to `[0.0, SATURATION]`,
//! `growth_rate` is passed through unchanged (callers are responsible
//! for keeping it finite). `tick` floors `loneliness` at `0.0` so a
//! future "socialise" effect modelled via negative `growth_rate` cannot
//! underflow.
//!
//! See `.harness/plans/phase7.md §3` and Phase 7 anchor in
//! `.harness/audit/section_8_plus_design.md`.

use serde::{Deserialize, Serialize};

/// Per-agent social need state.
///
/// `loneliness` is the current loneliness level (0 = fully social,
/// [`Social::SATURATION`] = lonely). `growth_rate` is the per-tick
/// increment applied by a future `SocialInteractionSystem`
/// (Phase 7-β scope).
///
/// `Default` is intentionally NOT derived — callers must use
/// [`Social::new`] so that `initial` is clamped at the boundary
/// (locked fact P7α-1).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Social {
    /// Current loneliness level. Always within `[0.0, SATURATION]` after
    /// construction; subsequent direct field mutation is the caller's
    /// responsibility (mirrors `Sleep::fatigue`).
    pub loneliness: f64,
    /// Per-tick growth, added to `loneliness` by future
    /// `SocialInteractionSystem`. Pass-through — not sanitised at
    /// construction. Mirrors `Sleep::growth_rate`.
    pub growth_rate: f64,
}

impl Social {
    /// Hard upper bound on `loneliness`. Direct mirror of
    /// [`crate::components::Sleep::SATURATION`].
    pub const SATURATION: f64 = 100.0;

    /// Construct a new `Social` with `loneliness` clamped to
    /// `[0.0, SATURATION]`. `growth_rate` is preserved exactly.
    /// Direct structural mirror of [`crate::components::Sleep::new`].
    pub fn new(initial: f64, growth_rate: f64) -> Self {
        Self {
            loneliness: initial.clamp(0.0, Self::SATURATION),
            growth_rate,
        }
    }

    /// Advance one tick: `loneliness = min(loneliness + growth_rate,
    /// SATURATION)`, then clamp negative results to `0.0` so a
    /// "socialise" effect modelled via negative `growth_rate` cannot
    /// underflow. Mirrors [`crate::components::Sleep::tick`].
    pub fn tick(&mut self) {
        self.loneliness = (self.loneliness + self.growth_rate).min(Self::SATURATION);
        if self.loneliness < 0.0 {
            self.loneliness = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_constant_is_100() {
        assert_eq!(Social::SATURATION, 100.0_f64);
    }

    #[test]
    fn new_clamps_negative_initial() {
        let s = Social::new(-5.0, 1.0);
        assert_eq!(s.loneliness, 0.0);
        assert_eq!(s.growth_rate, 1.0);
    }

    #[test]
    fn new_clamps_over_saturation_initial() {
        let s = Social::new(250.0, 1.0);
        assert_eq!(s.loneliness, Social::SATURATION);
    }

    #[test]
    fn new_preserves_negative_growth_rate() {
        let s = Social::new(50.0, -10.0);
        assert_eq!(s.loneliness, 50.0);
        assert_eq!(s.growth_rate, -10.0);
    }

    #[test]
    fn tick_adds_growth_rate() {
        let mut s = Social::new(0.0, 3.0);
        s.tick();
        assert_eq!(s.loneliness, 3.0);
        s.tick();
        assert_eq!(s.loneliness, 6.0);
    }

    #[test]
    fn tick_clamps_at_saturation() {
        let mut s = Social::new(99.0, 5.0);
        s.tick();
        assert_eq!(s.loneliness, Social::SATURATION);
        for _ in 0..10 {
            s.tick();
        }
        assert_eq!(s.loneliness, Social::SATURATION);
    }

    #[test]
    fn tick_floors_at_zero_on_negative_growth_rate() {
        let mut s = Social::new(5.0, -10.0);
        s.tick();
        assert_eq!(s.loneliness, 0.0);
        s.tick();
        assert_eq!(s.loneliness, 0.0);
    }

    #[test]
    fn serde_round_trip() {
        let s = Social::new(42.5, 0.7);
        let encoded = ron::to_string(&s).unwrap();
        let decoded: Social = ron::from_str(&encoded).unwrap();
        assert_eq!(s, decoded);
    }
}
