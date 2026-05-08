//! Tile Grid module — Phase 2 (Tile Grid + Influence System) base.
//!
//! Phase 0 design final reference: `PHASE_2_PHASE0_DESIGN_FINAL.md`.
//!
//! - Section 2.2: SoA layout (256×256, ~1.7 MB hot data).
//! - Section 4: Algorithm precise spec.
//!
//! `TerrainType` is owned by the `material` module and accessible via
//! `crate::material::TerrainType`; this module does not re-export it.

pub mod grid;

pub use grid::TileGrid;
