//! Physical nonfood batches: consumed inputs await qualifying work and output space.
use serde::{Deserialize, Serialize};

use super::{BillId, ItemSpec, WorkshopId};

/// A batch owns its workshop slot until output is placed or the workshop is lost.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassiveBatch {
    /// Physical workshop that owns this batch.
    pub workshop: WorkshopId,
    /// Bill that started the transform.
    pub bill: BillId,
    /// Output placement origin.
    pub tile: (u32, u32),
    /// Exact uncreated output; no physical items exist until placement commits.
    pub output: ItemSpec,
    /// Output count.
    pub count: u16,
    /// Qualifying simulation ticks still required; zero retains output ownership.
    pub remaining_ticks: u64,
    /// Whether a cardinal river neighbour is required for time to advance.
    pub requires_river: bool,
}

/// Ordered batches, persisted separately from historical resource bodies.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingState {
    /// Stable craft-completion order, independent of map iteration.
    pub pending: Vec<PassiveBatch>,
}
