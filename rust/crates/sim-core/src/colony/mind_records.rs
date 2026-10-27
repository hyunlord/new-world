//! Lossless shared mind-choice payloads; immutable policy-bound decoding and last-16 history.
//! The causal tile log and person history share one allocation per emitted choice.

mod codec;
mod dictionary;
mod display;
mod history;
mod reader;
mod source;
#[cfg(test)]
mod tests;

use super::mind::{BandId, CompiledMind, FactId, MindRevision, NodeId};
use crate::causal::EventId;
use crate::components::AgentId;
pub use dictionary::{MindRecordDictionary, NodeLabel};
pub use display::{DisplayCandidate, DisplayFact, MindChoiceDisplay};
pub use history::{MindHistory, MIND_HISTORY_LIMIT};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use source::{MindFactOriginRecord, MindFactSource};
use std::sync::Arc;
use thiserror::Error;

/// One scored candidate, in the raw ranking order used by the selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceCandidate {
    /// Node identity under this record's immutable policy dictionary.
    pub node: NodeId,
    /// Exact signed integer score, without narrowing.
    pub score: i64,
}
/// One deciding fact, deduplicated by identity; provenance is never fabricated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceFact {
    /// Policy-local fact identity.
    pub fact: FactId,
    /// Exact value observed at selection, not a later refreshed value.
    pub value: i64,
    /// Actual causal event for this fact when available.
    pub cause: Option<EventId>,
}
/// Complete logical record returned by the checked UI/save boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindChoiceRecord {
    /// Policy identity under which all recorded slots were selected.
    pub revision: MindRevision,
    /// Globally assigned causal event identity.
    pub id: EventId,
    /// Actual cause chosen from the deciding facts, when one exists.
    pub parent: Option<EventId>,
    /// Person whose choice changed.
    pub agent: AgentId,
    /// Simulation tick at emission.
    pub tick: u64,
    /// Selected node; hysteresis may retain it outside raw top three.
    pub node: NodeId,
    /// Selected priority band.
    pub band: BandId,
    /// Selected score, recalculated at this decision.
    pub score: i64,
    /// Up to three raw candidates, in score/tie order.
    pub top3: Vec<ChoiceCandidate>,
    /// Every deciding fact, sorted uniquely by FactId.
    pub facts: Vec<ChoiceFact>,
}

#[derive(Debug, PartialEq, Eq)]
struct Payload {
    id: EventId,
    tick: u64,
    parent: Option<EventId>,
    bytes: Box<[u8]>,
    revision: u64,
    agent: AgentId,
    node: usize,
}
/// Validated compact record. Clones share immutable bytes, including across tile/person logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedMindChoice(Arc<Payload>);
impl PackedMindChoice {
    /// Encode a complete record after checking its IDs against the compiled dictionary.
    pub fn new(policy: &CompiledMind, record: &MindChoiceRecord) -> Result<Self, MindRecordError> {
        let bytes = codec::encode(policy, record)?;
        Ok(Self(Arc::new(Payload {
            id: record.id,
            tick: record.tick,
            parent: record.parent,
            revision: record.revision.code(),
            agent: record.agent,
            node: record.node.index(),
            bytes: bytes.into_boxed_slice(),
        })))
    }
    /// Parse a standalone wire record; IDs remain unresolved until `decode` checks a policy.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MindRecordError> {
        let raw = codec::parse(bytes)?;
        Ok(Self(Arc::new(Payload {
            id: raw.id,
            tick: raw.tick,
            parent: raw.parent,
            revision: raw.revision,
            agent: raw.agent,
            node: raw.node,
            bytes: bytes.into(),
        })))
    }
    /// Decode every field under the exact revision and saved dictionary.
    pub fn decode(
        &self,
        policy: &CompiledMind,
        dictionary: &MindRecordDictionary,
    ) -> Result<MindChoiceRecord, MindRecordError> {
        dictionary.check(policy)?;
        codec::resolve(codec::parse(self.bytes())?, policy)
    }
    /// Compare immutable identity cached by the checked constructors.
    /// History must still pass its ordinary policy validation before runtime use.
    pub fn matches_running(
        &self,
        agent: crate::components::AgentId,
        running: crate::colony::mind::RunningChoice,
    ) -> Result<bool, MindRecordError> {
        Ok(self.0.revision == running.revision.code()
            && self.0.agent == agent
            && self.0.node == running.node.index())
    }
    /// Causal identity, cached after checked parsing.
    pub fn id(&self) -> EventId {
        self.0.id
    }
    /// Actual causal parent, cached after checked parsing.
    pub fn parent(&self) -> Option<EventId> {
        self.0.parent
    }
    /// Simulation tick, cached after checked parsing.
    pub fn tick(&self) -> u64 {
        self.0.tick
    }
    /// Complete canonical wire representation, including absolute header IDs.
    pub fn bytes(&self) -> &[u8] {
        &self.0.bytes
    }
    /// Shared ownership check for allocation-sensitive verification.
    pub fn shares_payload(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Serialize for PackedMindChoice {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bytes().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for PackedMindChoice {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
/// A malformed payload or incompatible dictionary; callers must surface the failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MindRecordError {
    /// Malformed, truncated, noncanonical or overflowing wire data.
    #[error("invalid mind record: {0}")]
    Invalid(&'static str),
    /// The record or saved dictionary belongs to another policy.
    #[error("mind record policy dictionary/revision mismatch")]
    PolicyMismatch,
    /// A node/fact/band slot does not exist in the bound policy.
    #[error("mind record contains unknown {0} slot")]
    UnknownSlot(&'static str),
}
