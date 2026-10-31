//! Persistent policy state, stored separately from the legacy resource wire layout.
//! Why-not records (M-READ §2.2) are presentation-only: `#[serde(skip)]`,
//! not saved, not hashed, excluded from gameplay state comparison.

use super::mind::{BandId, BlockedCandidate, FactId, NodeId, RunningChoice};
use super::mind_job_origins::{MindDraftOrigin, MindJobOrigin, MindOrderKey};
use super::mind_records::{MindFactOriginRecord, MindFactSource, MindHistory};
use crate::causal::EventId;
use crate::components::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Maximum why-not entries kept per person (bounded ring).
pub const WHY_NOT_RING_SIZE: usize = 4;

/// One why-not entry: a blocked candidate from a single MindChoice with its
/// first failing predicate. Presentation only, never affects simulation.
#[derive(Debug, Clone)]
pub struct WhyNotEntry {
    /// Simulation tick when this was recorded.
    pub tick: u64,
    /// Blocked node id.
    pub node: NodeId,
    /// Blocked node's band.
    pub band: BandId,
    /// Fact name of the first failing predicate.
    pub fact_name: String,
    /// Operator of the first failing predicate.
    pub op: String,
    /// Authored threshold value.
    pub threshold: i64,
    /// Actual observed value.
    pub actual: i64,
    /// Cause EventId of the failing fact, when available.
    pub cause: Option<EventId>,
}

/// Per-person why-not state: a bounded ring of recent blocked candidates
/// plus an hourly history bitmask for request aggregation.
#[derive(Debug, Clone, Default)]
pub struct WhyNotState {
    /// Bounded ring of recent entries, newest first.
    pub entries: Vec<WhyNotEntry>,
    /// Hour index of the last update (tick / 60).
    pub last_hour: u64,
    /// Bitmask: bit i set = person was blocked in (last_hour - i)th hour.
    /// Bit 0 = current hour, bit 1 = previous hour, etc.
    pub hour_mask: u8,
}

impl WhyNotState {
    /// Record new blocked candidates for this person at the given tick.
    pub fn push(&mut self, tick: u64, blocked: &[BlockedCandidate], fact_names: &[String]) {
        if blocked.is_empty() {
            return;
        }
        let hour = tick / 60;
        // Shift the hour mask if time advanced.
        if hour > self.last_hour {
            let shift = (hour - self.last_hour).min(8) as u32;
            self.hour_mask = self.hour_mask.checked_shl(shift).unwrap_or(0);
            self.last_hour = hour;
        }
        // Mark current hour as blocked.
        self.hour_mask |= 1;
        // Add entries, keeping bounded ring.
        for bc in blocked.iter().take(2) {
            let fact_name = fact_names.get(bc.fact.index()).cloned().unwrap_or_default();
            let entry = WhyNotEntry {
                tick,
                node: bc.node,
                band: bc.band,
                fact_name,
                op: bc.op.to_owned(),
                threshold: bc.threshold,
                actual: bc.actual,
                cause: None,
            };
            if self.entries.len() >= WHY_NOT_RING_SIZE {
                self.entries.pop();
            }
            self.entries.insert(0, entry);
        }
    }

    /// Whether this person was blocked for >= threshold of the last window_hours.
    pub fn is_active(&self, current_hour: u64, window_hours: u32, threshold: u32) -> bool {
        let shift = current_hour.saturating_sub(self.last_hour).min(8) as u32;
        let mask = self.hour_mask.checked_shl(shift).unwrap_or(0);
        let count = (0..window_hours).filter(|i| mask & (1 << i) != 0).count() as u32;
        count >= threshold
    }

    /// Latest blocked entries (newest first).
    pub fn latest(&self) -> &[WhyNotEntry] {
        &self.entries
    }

    /// Approximate bytes used by this state.
    pub fn byte_size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entries.capacity()
                * (std::mem::size_of::<WhyNotEntry>() + 32/* est. string alloc */)
    }
}

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
    /// M-READ §2.2: per-person why-not records. Presentation only: not saved,
    /// not hashed, excluded from gameplay state comparison by the normalising
    /// script. The map is rebuilt from scratch after load.
    #[serde(skip)]
    pub why_not: BTreeMap<AgentId, WhyNotState>,
}
