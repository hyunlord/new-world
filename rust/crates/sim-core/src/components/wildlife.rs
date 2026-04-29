use serde::{Deserialize, Serialize};

/// Ecological kind of a wildlife entity (Phase A1 — wander only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildlifeKind {
    Wolf,
    Bear,
    Boar,
}

impl WildlifeKind {
    /// Base Danger influence intensity emitted per tick.
    /// Bear most threatening, Boar least.
    /// Used by `WildlifeRuntimeSystem` to stamp Danger into the influence grid.
    pub fn danger_intensity(self) -> f64 {
        match self {
            Self::Wolf => 0.7,
            Self::Bear => 0.9,
            Self::Boar => 0.5,
        }
    }

    /// HP damage dealt per successful attack (Phase A3).
    /// Magnitudes chosen so that `bear > wolf > boar` and each falls within
    /// the design bands `wolf ∈ [3,9]`, `bear ∈ [9,24]`, `boar ∈ [0,2]`.
    pub fn attack_damage(self) -> u8 {
        match self {
            Self::Wolf => 5,
            Self::Bear => 15,
            Self::Boar => 1,
        }
    }

    /// Stable lowercase name used in causal log source kind (e.g. "wolf_attack").
    pub fn kind_name(self) -> &'static str {
        match self {
            Self::Wolf => "wolf",
            Self::Bear => "bear",
            Self::Boar => "boar",
        }
    }
}

/// ECS component for non-human fauna.
///
/// Spawned by `WildlifeRuntimeSystem` at tick 0 and persists for the
/// lifetime of the simulation.  Phase A1 implements spawn + wander only;
/// threat detection is A2; combat is A3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wildlife {
    pub kind: WildlifeKind,
    pub max_hp: f64,
    pub current_hp: f64,
    pub move_speed: f64,
    /// Tile where this entity spawned — used as wander anchor.
    pub home_tile: (i32, i32),
    /// Maximum Chebyshev distance from `home_tile` before wander is rejected.
    pub wander_radius: i32,
    /// Tick of the last successful attack; used for cooldown gating (Phase A3).
    pub last_attack_tick: u32,
}

impl Wildlife {
    /// Wolf: fast pack predator, moderate HP.
    pub fn wolf(home: (i32, i32)) -> Self {
        Self {
            kind: WildlifeKind::Wolf,
            max_hp: 60.0,
            current_hp: 60.0,
            move_speed: 1.4,
            home_tile: home,
            wander_radius: 15,
            last_attack_tick: 0,
        }
    }

    /// Bear: slow solitary predator, high HP.
    pub fn bear(home: (i32, i32)) -> Self {
        Self {
            kind: WildlifeKind::Bear,
            max_hp: 120.0,
            current_hp: 120.0,
            move_speed: 0.9,
            home_tile: home,
            wander_radius: 10,
            last_attack_tick: 0,
        }
    }

    /// Boar: medium speed, medium HP.
    pub fn boar(home: (i32, i32)) -> Self {
        Self {
            kind: WildlifeKind::Boar,
            max_hp: 80.0,
            current_hp: 80.0,
            move_speed: 1.1,
            home_tile: home,
            wander_radius: 12,
            last_attack_tick: 0,
        }
    }

    /// Returns true when the entity has any remaining HP.
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wolf_full_hp_at_construction() {
        let w = Wildlife::wolf((10, 10));
        assert_eq!(w.current_hp, w.max_hp);
        assert_eq!(w.kind, WildlifeKind::Wolf);
    }

    #[test]
    fn bear_full_hp_at_construction() {
        let b = Wildlife::bear((5, 5));
        assert_eq!(b.current_hp, b.max_hp);
        assert!(b.max_hp > 60.0, "bear should have more HP than wolf");
    }

    #[test]
    fn boar_full_hp_at_construction() {
        let b = Wildlife::boar((0, 0));
        assert_eq!(b.current_hp, b.max_hp);
        assert_eq!(b.home_tile, (0, 0));
    }

    #[test]
    fn home_tile_stored_correctly() {
        let w = Wildlife::wolf((42, 99));
        assert_eq!(w.home_tile, (42, 99));
    }
}
