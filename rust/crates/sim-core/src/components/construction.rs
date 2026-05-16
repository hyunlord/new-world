//! V7 Phase 6-α / P6α-1 — Building-construction data substrate.
//!
//! First sub-stage of V7 Phase 6 (Building System Deep). Lands the
//! component-layer prerequisites for agent-driven construction without
//! introducing any runtime system, causal event, or agent-decision
//! change — those follow in β and γ.
//!
//! Two structs live here:
//!
//! - [`BuildingBlueprint`] — immutable design spec (footprint + required
//!   construction-tick count). Identified by a stable [`BlueprintId`]
//!   alias over `u64`, mirroring the [`AgentId`](crate::components::AgentId)
//!   precedent from Phase 5-α.
//! - [`ConstructionSite`] — in-progress construction at a specific tile.
//!   Composes a [`BuildingBlueprint`] with mutable `progress: u32` and
//!   a [`Position`]. Saturating `advance()` returns a one-shot completion
//!   edge so the Phase 6-β `ConstructionSystem` can fire downstream
//!   effects exactly once.
//!
//! See `.harness/plans/phase6.md` §2.1 for the full sub-stage
//! decomposition. Path (b) symmetry note: `TargetKind::ConstructionSite`
//! is added as a sibling of `Sleep` (see [`crate::components::TargetKind`]),
//! NOT as a new `AgentState` variant — preserving the Phase 5-γ
//! decision exactly.

use serde::{Deserialize, Serialize};

use crate::components::position::Position;

/// Stable per-blueprint identifier. Mirrors
/// [`AgentId`](crate::components::AgentId) (also `u64`).
///
/// Phase 6-α scope: the alias exists and is wired to
/// [`BuildingBlueprint::id`]. Allocation strategy (monotonic mint vs.
/// content-hash) is intentionally NOT pinned here — that decision lands
/// alongside the Phase 6-β `ConstructionSystem` that produces sites.
pub type BlueprintId = u64;

/// Immutable design specification of a building, decoupled from any
/// individual construction in progress.
///
/// `footprint_width` × `footprint_height` define the rectangular tile
/// span the finished building occupies. `required_progress` is the
/// number of `ConstructionSystem` ticks needed to complete construction
/// (β scope — α only defines the field).
///
/// All fields are `pub`: Phase 6-α treats `BuildingBlueprint` as pure
/// owned data with no internal invariants the struct must enforce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingBlueprint {
    /// Stable per-blueprint identity.
    pub id: BlueprintId,
    /// Footprint width in tiles.
    pub footprint_width: u32,
    /// Footprint height in tiles.
    pub footprint_height: u32,
    /// Construction-system ticks required to complete this build.
    pub required_progress: u32,
}

impl BuildingBlueprint {
    /// Pure assignment constructor — no normalization, no clamping.
    ///
    /// Field order matches the struct declaration:
    /// `(id, footprint_width, footprint_height, required_progress)`.
    pub fn new(
        id: BlueprintId,
        footprint_width: u32,
        footprint_height: u32,
        required_progress: u32,
    ) -> Self {
        Self {
            id,
            footprint_width,
            footprint_height,
            required_progress,
        }
    }

    /// Convenience accessor: returns `(footprint_width, footprint_height)`
    /// in that order. Tuple order is locked by P6α-2 — the width-first
    /// convention matches existing dirty-region / influence-grid usage.
    pub fn footprint(&self) -> (u32, u32) {
        (self.footprint_width, self.footprint_height)
    }
}

/// An in-progress construction at a specific tile. Composed of the
/// immutable [`BuildingBlueprint`] plus mutable progress state.
///
/// `progress` is `pub` so the Phase 6-β `ConstructionSystem` (and tests
/// exercising the `is_complete()` `>=` semantics beyond what [`advance`]
/// can reach via saturation) can write directly. There are no invariants
/// the struct must enforce internally — saturation lives in [`advance`]
/// and the completion check lives in [`is_complete`].
///
/// [`advance`]: ConstructionSite::advance
/// [`is_complete`]: ConstructionSite::is_complete
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructionSite {
    /// Immutable design spec.
    pub blueprint: BuildingBlueprint,
    /// Current tick count toward `blueprint.required_progress`.
    /// Phase 6-α scope: producers write either via [`advance`] (saturating
    /// +1 per call) or via direct field write.
    ///
    /// [`advance`]: ConstructionSite::advance
    pub progress: u32,
    /// Tile this site occupies (top-left of the footprint).
    pub position: Position,
}

impl ConstructionSite {
    /// Construct a new site with `progress = 0`. `blueprint` and `position`
    /// are copied verbatim.
    pub fn new(blueprint: BuildingBlueprint, position: Position) -> Self {
        Self {
            blueprint,
            progress: 0,
            position,
        }
    }

    /// Saturating tick. Returns `true` exactly when this call transitioned
    /// `progress` from `< required_progress` to `== required_progress`
    /// (one-shot completion edge). After the edge fires, repeated calls
    /// return `false` and leave `progress` pinned at `required_progress`.
    ///
    /// Edge cases:
    /// - `required_progress == 0`: site is born complete; the first
    ///   `advance()` is a no-op returning `false`, since the call cannot
    ///   *transition* progress to a value it already holds.
    /// - `progress > required_progress` (only reachable via direct field
    ///   write — see P6α-8 / Assertion 8): `advance()` does not move
    ///   progress backward and returns `false`.
    pub fn advance(&mut self) -> bool {
        let req = self.blueprint.required_progress;
        if self.progress >= req {
            // Already complete (or past via direct write). Saturate, no edge.
            return false;
        }
        // Saturating add of 1 — explicit form avoids relying on
        // `u32::saturating_add` semantics being preserved if `req` is
        // ever changed away from `u32`. P6α-12 locks the integer-tick
        // type so a simple `+ 1` cannot overflow before the `>= req`
        // check above pins us out of this branch.
        self.progress += 1;
        self.progress == req
    }

    /// Inclusive completion check: `progress >= required_progress`.
    ///
    /// `>=` (not `==`, not `>`) means a `required_progress == 0`
    /// blueprint is complete at construction, and a `progress` value
    /// pushed past `required_progress` via direct field write still
    /// reports complete (Phase 6-β `ConstructionSystem` invariant).
    pub fn is_complete(&self) -> bool {
        self.progress >= self.blueprint.required_progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_id_is_u64() {
        let _: BlueprintId = 0u64;
    }

    #[test]
    fn new_sets_all_fields_verbatim() {
        let bp = BuildingBlueprint::new(7, 3, 4, 42);
        assert_eq!(bp.id, 7);
        assert_eq!(bp.footprint_width, 3);
        assert_eq!(bp.footprint_height, 4);
        assert_eq!(bp.required_progress, 42);
    }

    #[test]
    fn footprint_returns_width_then_height() {
        let bp = BuildingBlueprint::new(1, 5, 9, 0);
        assert_eq!(bp.footprint(), (5, 9));
    }

    #[test]
    fn site_constructor_zeroes_progress() {
        let bp = BuildingBlueprint::new(1, 2, 2, 5);
        let site = ConstructionSite::new(bp, Position { x: 11, y: 22 });
        assert_eq!(site.progress, 0);
        assert_eq!(site.blueprint, bp);
        assert_eq!(site.position, Position { x: 11, y: 22 });
    }

    #[test]
    fn advance_increments_one_per_call() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 5), Position { x: 0, y: 0 });
        assert!(!site.advance());
        assert_eq!(site.progress, 1);
    }

    #[test]
    fn advance_fires_completion_edge_exactly_once() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 3), Position { x: 0, y: 0 });
        let returns: Vec<bool> = (0..4).map(|_| site.advance()).collect();
        assert_eq!(returns, vec![false, false, true, false]);
    }

    #[test]
    fn advance_required_one_fires_immediately() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 1), Position { x: 0, y: 0 });
        let returns: Vec<bool> = (0..3).map(|_| site.advance()).collect();
        assert_eq!(returns, vec![true, false, false]);
        assert_eq!(site.progress, 1);
    }

    #[test]
    fn advance_saturates_at_required() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 3), Position { x: 0, y: 0 });
        for _ in 0..10 {
            site.advance();
        }
        assert_eq!(site.progress, 3);
    }

    #[test]
    fn is_complete_uses_ge_semantics() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 5), Position { x: 0, y: 0 });
        for _ in 0..4 {
            site.advance();
        }
        assert!(!site.is_complete());
        site.advance();
        assert!(site.is_complete());
        site.progress = 6;
        assert!(site.is_complete());
    }

    #[test]
    fn required_zero_is_trivially_complete() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 0), Position { x: 0, y: 0 });
        assert!(site.is_complete());
        assert!(!site.advance());
        assert_eq!(site.progress, 0);
    }

    #[test]
    fn blueprint_serde_round_trip() {
        let bp = BuildingBlueprint::new(99, 4, 6, 25);
        let encoded = ron::to_string(&bp).unwrap();
        let decoded: BuildingBlueprint = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded, bp);
    }

    #[test]
    fn site_serde_round_trip_mid_construction() {
        let mut site =
            ConstructionSite::new(BuildingBlueprint::new(1, 2, 2, 5), Position { x: 7, y: 8 });
        site.advance();
        let encoded = ron::to_string(&site).unwrap();
        let decoded: ConstructionSite = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded, site);
        assert_eq!(decoded.progress, 1);
    }
}
