//! Typed recorder inputs and errors; causes preserve their original namespace.
use sim_core::causal::EventId;
use sim_core::colony::mind::FactId;
use sim_core::colony::mind_records::{MindFactSource, MindRecordError};
use sim_core::components::{AgentId, SettlementId};
use std::fmt;
/// Verified actor identity supplied by the AgentId-sorted coordinator.
#[derive(Debug, Clone, Copy)]
pub struct RecordPerson {
    /// Actual living person from the coordinator's sorted snapshot.
    pub agent: AgentId,
    /// Actual current tile, validated against world bounds before recording.
    pub position: (u32, u32),
    /// Actual home membership when known.
    pub settlement: Option<SettlementId>,
}
/// Recorded causes are distinct from descriptive fact values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FactCause {
    /// A real already-issued causal event.
    Event(EventId),
    /// An actual external record requiring a typed causal bridge.
    Source(MindFactSource),
}
/// Stable parent preference among actual deciding-fact sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CausePriority {
    /// Actual animal/pressure evidence for a threat.
    Threat,
    /// Actual assigned order evidence.
    Job,
    /// Actual completed meal or other need evidence.
    Need,
    /// Other registered factual provenance.
    Other,
}
/// One actual source for a deciding fact; absence is never replaced with a guessed cause.
#[derive(Debug, Clone, Copy)]
pub struct FactOrigin {
    /// Fact read by this choice.
    pub fact: FactId,
    /// Genuine cause in its own namespace.
    pub source: FactCause,
    /// Deterministic primary-parent preference.
    pub priority: CausePriority,
}
/// Recorder failures are surfaced by the coordinator, never replaced with a silent idle.
#[derive(Debug)]
pub enum RecordError {
    /// A caller, source or world invariant failed.
    Invalid(&'static str),
    /// Compact record validation failed.
    Payload(MindRecordError),
}
impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::Payload(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for RecordError {}
impl From<MindRecordError> for RecordError {
    fn from(value: MindRecordError) -> Self {
        Self::Payload(value)
    }
}
