//! Pure selector inputs and outcomes, separate from authored policy metadata.

use super::{BandId, BehaviourId, MindRevision, NodeId};
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

/// Observer/executor state supplied to the pure selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunningChoice {
    /// Dictionary/revision owning the node ID.
    pub revision: MindRevision,
    /// Previously selected node.
    pub node: NodeId,
    /// Lifecycle ended; do not apply hysteresis to this incumbent.
    pub finished: bool,
}
/// Stateless evaluation identity; AgentId use follows the explicit CORE contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalKey {
    /// World seed.
    pub world_seed: u64,
    /// Current sorted person identity.
    pub agent: u64,
    /// Current simulation tick.
    pub tick: u64,
    /// Noise replicate for separately keyed behaviour draws.
    pub replicate: u32,
}
/// One eligible scored node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    /// Authored node under the current dictionary.
    pub node: NodeId,
    /// Node's priority band.
    pub band: BandId,
    /// Registered behaviour implementation.
    pub behaviour: BehaviourId,
    /// Checked final integer score.
    pub score: i64,
}
/// Semantic node transition, not an AgentState progress phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// No prior running choice exists.
    Start,
    /// Same unfinished node remains selected.
    Continue,
    /// A different semantic node replaces the previous choice.
    Switch,
    /// Same node is selected after its previous lifecycle completed.
    RestartAfterFinish,
}
/// Selection result; exact witnesses remain in the caller's reusable scratch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// Actual choice, possibly outside raw top3 due to hysteresis.
    pub selected: Candidate,
    /// Highest three raw scores in the winning band, with authored-order ties.
    pub top3: ArrayVec<Candidate, 3>,
    /// Required lifecycle dispatch.
    pub transition: Transition,
    /// An otherwise better raw candidate was held off by the running margin.
    pub retained_by_hysteresis: bool,
}
