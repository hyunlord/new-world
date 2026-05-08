//! Influence System (Phase 0 Section 2 — V7 Tile Grid + Influence).
//!
//! Eight base channels (Warmth, Light, Noise, FoodAroma, Danger, Social,
//! Spiritual, Beauty) propagate over the tile grid each tick. Each channel
//! has its own decay function, aggregation policy, update tier and wall
//! blocking rule (see [`channel`]).
//!
//! Module layout (built incrementally across T7.2 → T7.5):
//! - [`channel`] — `InfluenceChannel`, `AggKind`, `UpdateTier`, `DecayKind`,
//!   `BlockingDerive`, `ChannelDef`.
//! - [`grid`] — `InfluenceGrid` (8-channel double-buffered SoA) +
//!   `DirtyRegion`.
//!
//! v0.1.1 ISSUE fixes wired in this module:
//! - 1: Warmth source = STATIC heat sources only
//! - 2: linear decay distinct from wall blocking
//! - 3: Danger linear `alpha = 5` with sight-radius cap (15)
//! - 5: no global `source_templates` HashMap on `InfluenceGrid`
//! - 6: per-channel aggregation policy (`AggKind`)

pub mod channel;
pub mod grid;

pub use channel::{
    AggKind, BlockingDerive, ChannelDef, DecayKind, InfluenceChannel, UpdateTier,
};
pub use grid::{DirtyRegion, InfluenceGrid};
