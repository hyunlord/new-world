//! `BodyHealth` component (V7 Phase 9-α / P9Plan-1).
//!
//! Per-agent HP substrate. Simple `{ hp: f64, max_hp: f64 }` — no per-body-part
//! damage model (Phase 10+ concern). Death condition: `hp <= 0.0`.
//! Phase 9-β `CombatSystem` (priority 137) applies `DAMAGE_PER_COMBAT_TICK`
//! via `apply_damage()` and calls `is_dead()` to determine despawn.
//!
//! See `.harness/plans/phase9.md §3` for the locked P9Plan-1 facts.

use serde::{Deserialize, Serialize};

/// Default maximum HP for a newly spawned agent (P9Plan-1).
pub const DEFAULT_MAX_HP: f64 = 100.0;

/// Per-agent health state.
///
/// `hp` is the current hit points (0.0 = dead, `max_hp` = fully healthy).
/// `max_hp` is set at spawn time and does not change during a combat encounter.
/// Both fields are `f64` per the project-wide "ALL f64 for simulation math
/// (determinism)" rule.
///
/// `Copy` is derived — both fields are primitive `f64`, same as `Hunger`,
/// `Thirst`, and `Sleep`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodyHealth {
    /// Current HP. Always within `[0.0, max_hp]` after any method call.
    pub hp: f64,
    /// Maximum HP. Set at construction; not modified by damage or healing.
    pub max_hp: f64,
}

impl BodyHealth {
    /// Construct with `hp = max_hp = DEFAULT_MAX_HP`.
    pub fn new() -> Self {
        Self {
            hp: DEFAULT_MAX_HP,
            max_hp: DEFAULT_MAX_HP,
        }
    }

    /// Construct with a custom `max_hp`. `hp` starts at `max_hp`.
    /// `max_hp` is clamped to a minimum of `f64::EPSILON` to avoid
    /// divide-by-zero in future percentage calculations.
    pub fn new_with_max(max_hp: f64) -> Self {
        let max = max_hp.max(f64::EPSILON);
        Self { hp: max, max_hp: max }
    }

    /// Apply damage: `hp = (hp - amount).max(0.0)`. Saturates at 0.
    pub fn apply_damage(&mut self, amount: f64) {
        self.hp = (self.hp - amount).max(0.0).min(self.max_hp);
    }

    /// Heal: `hp = (hp + amount).min(max_hp)`. Saturates at `max_hp`.
    pub fn heal(&mut self, amount: f64) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Returns `true` when `hp <= 0.0` (agent should be despawned by
    /// `CombatSystem`).
    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}

impl Default for BodyHealth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initialises_full_hp() {
        let bh = BodyHealth::new();
        assert_eq!(bh.hp, DEFAULT_MAX_HP);
        assert_eq!(bh.max_hp, DEFAULT_MAX_HP);
    }

    #[test]
    fn new_with_max_custom() {
        let bh = BodyHealth::new_with_max(50.0);
        assert_eq!(bh.hp, 50.0);
        assert_eq!(bh.max_hp, 50.0);
    }

    #[test]
    fn new_with_max_clamps_to_epsilon() {
        let bh = BodyHealth::new_with_max(0.0);
        assert_eq!(bh.max_hp, f64::EPSILON);
        assert_eq!(bh.hp, f64::EPSILON);

        let bh2 = BodyHealth::new_with_max(-5.0);
        assert_eq!(bh2.max_hp, f64::EPSILON);
        assert_eq!(bh2.hp, f64::EPSILON);
    }

    #[test]
    fn apply_damage_reduces_hp() {
        let mut bh = BodyHealth::new();
        bh.apply_damage(30.0);
        assert_eq!(bh.hp, 70.0);
    }

    #[test]
    fn apply_damage_saturates_at_zero() {
        let mut bh = BodyHealth::new();
        bh.apply_damage(200.0);
        assert_eq!(bh.hp, 0.0);
    }

    #[test]
    fn heal_increases_hp() {
        let mut bh = BodyHealth::new_with_max(100.0);
        bh.apply_damage(50.0);
        bh.heal(20.0);
        assert_eq!(bh.hp, 70.0);
    }

    #[test]
    fn heal_saturates_at_max_hp() {
        let mut bh = BodyHealth::new_with_max(100.0);
        bh.apply_damage(10.0);
        bh.heal(999.0);
        assert_eq!(bh.hp, bh.max_hp);
    }

    #[test]
    fn is_dead_at_zero() {
        let mut bh = BodyHealth::new();
        bh.apply_damage(DEFAULT_MAX_HP);
        assert!(bh.is_dead());
    }

    #[test]
    fn is_dead_false_above_zero() {
        let bh = BodyHealth::new();
        assert!(!bh.is_dead());
    }

    #[test]
    fn serde_round_trip() {
        let bh = BodyHealth::new_with_max(75.0);
        let s = ron::to_string(&bh).expect("serialize");
        let r: BodyHealth = ron::from_str(&s).expect("deserialize");
        assert_eq!(bh, r);
    }
}
