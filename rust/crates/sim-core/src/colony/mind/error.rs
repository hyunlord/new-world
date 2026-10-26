//! Contextual content and evaluation errors; no silent fallback.

use super::{FactId, NodeId};
use thiserror::Error;

/// A contextual policy loading failure; explicit invalid data never falls back.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("mind.ron {context}: {message}")]
pub struct MindLoadError {
    /// Node, registry or field path.
    pub context: String,
    /// Concrete failure, including unknown names or RON source location.
    pub message: String,
}
impl MindLoadError {
    pub(super) fn new(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            message: message.into(),
        }
    }
}
/// Runtime input or arithmetic error; callers must surface rather than silently idle.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MindEvalError {
    /// Rows must be unique and sorted by AgentId.
    #[error("mind facts require strictly increasing AgentIds")]
    AgentOrder,
    /// Input row, column or running node is absent.
    #[error("mind invalid {0} index")]
    InvalidIndex(&'static str),
    /// A provider failed to write a referenced factual cell.
    #[error("mind fact {fact:?} not filled for row {row}")]
    UnwrittenFact {
        /// Required column.
        fact: FactId,
        /// Missing row index.
        row: usize,
    },
    /// Structural life stage is mandatory.
    #[error("mind life stage not filled for row {0}")]
    MissingLifeStage(usize),
    /// Boolean providers must not emit arbitrary integer values.
    #[error("mind Bool fact {0:?} requires 0 or 1")]
    InvalidBoolean(FactId),
    /// A saved choice or table belongs to another policy.
    #[error("mind policy revision mismatch")]
    PolicyMismatch,
    /// The row does not match the agent/tick evaluation key.
    #[error("mind agent/tick key mismatch")]
    KeyMismatch,
    /// Integer scoring exceeded the required i64 output range.
    #[error("mind score overflow at node {node:?}, fact {fact:?}")]
    ScoreOverflow {
        /// Node whose term or accumulator overflowed.
        node: NodeId,
        /// Fact read by the failing term.
        fact: FactId,
    },
    /// Authored conditions failed to provide an eligible fallback.
    #[error("mind has no eligible node")]
    NoEligibleNode,
}
