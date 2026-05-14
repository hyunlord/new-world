//! Canonical `Position` component (V7 Phase 4-α first deliverable).
//!
//! Tile-coordinate position (not pixels). Pixel conversion happens in
//! the GDScript renderer per CLAUDE.md architecture invariant
//! (`phase4.md` §3 — "Position = (u32, u32) tile coordinates, not pixels").
//!
//! This replaces the Phase 2 placeholder in
//! `sim_systems::runtime::influence::agent_sample::Position` per the
//! self-documenting landmark at `agent_sample.rs:9-15`. The "single-line
//! rewire" landmark contract is honoured by preserving the exact field
//! layout (`x: u32, y: u32`).
//!
//! `Serialize + Deserialize` are derived so the component participates
//! in the eventual save-load round-trip without further plumbing.

use serde::{Deserialize, Serialize};

/// Tile-coordinate position of an entity in the world.
///
/// Coordinates index into the `width × height` grid owned by
/// `SimResources::tile_grid` / `SimResources::influence_grid`. Systems
/// reading `Position` should bounds-check against
/// `resources.influence_grid.width`/`height` (both `u32`) before sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Tile X coordinate (column).
    pub x: u32,
    /// Tile Y coordinate (row).
    pub y: u32,
}

impl Position {
    /// Construct a new position at `(x, y)`.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_coordinates() {
        let p = Position::new(7, 11);
        assert_eq!(p.x, 7);
        assert_eq!(p.y, 11);
    }

    #[test]
    fn equality_holds_for_same_coords() {
        assert_eq!(Position::new(3, 4), Position::new(3, 4));
        assert_ne!(Position::new(3, 4), Position::new(4, 3));
    }

    #[test]
    fn copy_semantics_preserve_value() {
        let p = Position::new(1, 2);
        let q = p;
        assert_eq!(p, q);
    }

    /// Serde guard — round-trip via RON exercises both `Serialize` and
    /// `Deserialize`. If serde derives are removed this fails to compile
    /// (missing trait impls) before it even runs, which is the desired
    /// build-time guard.
    #[test]
    fn serde_round_trip_preserves_coordinates() {
        let original = Position::new(42, 17);
        let encoded = ron::to_string(&original).expect("Position must Serialize");
        let decoded: Position =
            ron::from_str(&encoded).expect("Position must Deserialize");
        assert_eq!(original, decoded);
    }
}
