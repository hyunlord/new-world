//! Persistent policy state, stored separately from the legacy resource wire layout.

use super::mind::{FactId, RunningChoice};
use super::mind_job_origins::{MindDraftOrigin, MindJobOrigin, MindOrderKey};
use super::mind_records::{MindFactOriginRecord, MindFactSource, MindHistory};
use crate::causal::EventId;
use crate::components::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Empty state preserves the baseline decision system. Compiled callbacks are rebuilt on load.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MindState {
    /// Exact authored policy source; absent before the mind switch.
    pub policy_source: Option<String>,
    /// Current per-person choices, bound to the compiled policy revision.
    pub running: BTreeMap<AgentId, RunningChoice>,
    /// Independent last-sixteen decision history for each person.
    pub history: Option<MindHistory>,
    /// Sparse descriptive provenance; never consulted by score evaluation.
    pub fact_origins: BTreeMap<(AgentId, FactId), EventId>,
    /// Accepted command source attached to each concrete live order.
    pub order_origins: BTreeMap<MindOrderKey, MindFactSource>,
    /// Source copied at actual job assignment; exact kind and timestamp guard reuse.
    pub job_origins: BTreeMap<AgentId, MindJobOrigin>,
    /// Source of the actual direct draft order, matched by since/order.
    pub draft_origins: BTreeMap<AgentId, MindDraftOrigin>,
    /// Genuine external causes materialized once as typed causal bridge events.
    pub fact_source_events: BTreeMap<MindFactSource, MindFactOriginRecord>,
}
