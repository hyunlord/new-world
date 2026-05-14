//! Cause-effect tracking — V7 Phase 3-α (Week 5-6 entry).
//!
//! Tile-level adaptation of the A-4 V6 32-event-per-entity ring buffer.
//! V7 chooses a smaller per-tile budget (8 events) because the relevant
//! consumer is the "왜?" UI, which only ever needs the *recent* state of
//! a single tile, not full lineage. The three event variants
//! ([`CausalEvent::BuildingPlaced`] / [`CausalEvent::StampDirty`] /
//! [`CausalEvent::InfluenceChanged`]) cover the Phase 0~2 surface;
//! later phases extend the enum without changing the ring size.
//!
//! Pipeline (P3α-1..4 decisions):
//!
//! ```text
//! FFI → building_event_queue
//!   ↓ BSS drains queue
//!     • BuildingPlaced  at centre tile
//!     • StampDirty × 6 channels at centre tile
//!   ↓ IUS drains dirty_regions per channel
//!     • InfluenceChanged at region centre per channel per region
//! ```

pub mod event;
pub mod ring_buffer;
pub mod storage;

pub use event::CausalEvent;
pub use ring_buffer::{TileCausalLog, TILE_CAUSAL_RING_SIZE};
pub use storage::CausalLogStorage;
