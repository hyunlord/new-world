//! Influence System (Phase 0 Section 2 — V7 Tile Grid + Influence).
//!
//! Eight base channels (Warmth, Light, Noise, FoodAroma, Danger, Social,
//! Spiritual, Beauty) propagate over the tile grid each tick. Each channel
//! has its own decay function, aggregation policy, update tier and wall
//! blocking rule (see [`channel`]).
//!
//! Module layout (T7.2 → T7.5):
//! - [`channel`] — `InfluenceChannel`, `AggKind`, `UpdateTier`, `DecayKind`,
//!   `BlockingDerive`, `ChannelDef`.
//! - [`grid`] — `InfluenceGrid` (8-channel double-buffered SoA) +
//!   `DirtyRegion`.
//! - [`blocking`] — `MaterialBlockingCache` (Material × Channel → block
//!   coefficient lookup table).
//! - [`propagate`] — BFS / shadowcast / linear / social-stamp primitives
//!   plus the `LodTier` classifier.
//!
//! v0.1.1 ISSUE fixes wired in this module:
//! - 1: Warmth source = STATIC heat sources only
//! - 2: linear decay distinct from wall blocking
//! - 3: Danger linear `alpha = 5` with sight-radius cap (15)
//! - 4: Light = recursive symmetric shadowcasting (not BFS)
//! - 5: no global `source_templates` HashMap on `InfluenceGrid`
//! - 6: per-channel aggregation policy (`AggKind`)
//! - 7: blocking cache passed by reference (no global static)
//! - 9: Social stamping skips Simplified/Dormant LOD tiers
//! - EC-2: source tile exempt from self-blocking in BFS

pub mod blocking;
pub mod channel;
pub mod grid;
pub mod propagate;

pub use blocking::MaterialBlockingCache;
pub use channel::{
    AggKind, BlockingDerive, ChannelDef, DecayKind, InfluenceChannel, UpdateTier,
};
pub use grid::{DirtyRegion, InfluenceGrid};
pub use propagate::{
    LodTier, propagate_bfs, propagate_danger, propagate_noise, propagate_shadowcast,
    stamp_social_aggregate,
};
