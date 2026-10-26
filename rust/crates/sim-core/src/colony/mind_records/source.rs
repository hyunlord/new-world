//! Distinct provenance namespaces; source indices are never cast to causal EventIds.
use crate::causal::EventId;
use crate::chronicle::CauseRef;
use serde::{Deserialize, Serialize};

/// Existing durable source behind a factual observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MindFactSource {
    /// Actual existing chronicle entry, pressure, modifier or sample reference.
    Chronicle(CauseRef),
    /// Zero-based index of the actual accepted command in the append-only command log.
    Command(u64),
}
/// Lazily allocated causal bridge header retained even if its tile ring evicts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindFactOriginRecord {
    /// Actual globally allocated causal event ID; never a source-index cast.
    pub id: EventId,
    /// Tick at which this source first became a mind fact's causal parent.
    pub tick: u64,
}
