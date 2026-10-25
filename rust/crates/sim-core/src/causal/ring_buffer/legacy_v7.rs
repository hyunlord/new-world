//! Frozen pre-format9 TileCausalLog; move retained history without replay.
use super::*;

/// Historical wire container.
#[derive(serde::Deserialize)]
pub struct LegacyTileCausalLog {
    events: ArrayVec<super::super::event::legacy_v7::LegacyCausalEvent, TILE_CAUSAL_RING_SIZE>,
}

impl From<LegacyTileCausalLog> for TileCausalLog {
    fn from(old: LegacyTileCausalLog) -> Self {
        Self {
            events: old.events.into_iter().map(Into::into).collect(),
        }
    }
}
