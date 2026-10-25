//! Frozen pre-format9 CausalLogStorage; move retained history without replay.
use super::*;

/// Historical wire container.
#[derive(serde::Deserialize)]
pub struct LegacyCausalLogStorage {
    logs: BTreeMap<u32, super::super::ring_buffer::legacy_v7::LegacyTileCausalLog>,
}

impl From<LegacyCausalLogStorage> for CausalLogStorage {
    fn from(old: LegacyCausalLogStorage) -> Self {
        Self {
            logs: old
                .logs
                .into_iter()
                .map(|(tile, log)| (tile, log.into()))
                .collect(),
        }
    }
}
