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
/// first failing predicate. Allocation-free: stores FactId and op code, not
/// String copies. Names are resolved at bridge-read time from the dictionary.
#[derive(Debug, Clone, Copy)]
pub struct WhyNotEntry {
    /// Simulation tick when this was recorded.
    pub tick: u64,
    /// Blocked node id.
    pub node: NodeId,
    /// Blocked node's band.
    pub band: BandId,
    /// Fact column of the first failing predicate (resolved to name at read time).
    pub fact: super::mind::FactId,
    /// Operator code: 0=Lt 1=Le 2=Gt 3=Ge 4=Eq 5=Has 6=Not.
    pub op_code: u8,
    /// Authored threshold value.
    pub threshold: i64,
    /// Actual observed value.
    pub actual: i64,
}

impl WhyNotEntry {
    /// Operator string from the code.
    pub fn op_str(&self) -> &'static str {
        match self.op_code {
            0 => "<",
            1 => "<=",
            2 => ">",
            3 => ">=",
            4 => "==",
            5 => "has",
            6 => "not",
            _ => "?",
        }
    }
    /// Encode an operator string to a code.
    pub fn op_to_code(op: &str) -> u8 {
        match op {
            "<" => 0,
            "<=" => 1,
            ">" => 2,
            ">=" => 3,
            "==" => 4,
            "has" => 5,
            "not" => 6,
            _ => 7,
        }
    }
}

/// Per-person why-not state: a fixed-size ring of recent blocked candidates
/// plus an hourly history bitmask for request aggregation. Allocation-free.
#[derive(Debug, Clone, Default)]
pub struct WhyNotState {
    /// Fixed-size ring; `count` entries valid starting at index 0 (newest first).
    pub entries: [Option<WhyNotEntry>; WHY_NOT_RING_SIZE],
    /// Number of valid entries.
    pub count: u8,
    /// Hour index of the last update (tick / 60).
    pub last_hour: u64,
    /// Bitmask: bit i set = person was blocked in (last_hour - i)th hour.
    pub hour_mask: u8,
}

impl WhyNotState {
    /// Record blocked candidates without any heap allocation.
    /// Called only on node change (not continuations).
    pub fn push_compact(&mut self, tick: u64, blocked: &[BlockedCandidate]) {
        if blocked.is_empty() {
            return;
        }
        let hour = tick / 60;
        if hour > self.last_hour {
            let shift = (hour - self.last_hour).min(8) as u32;
            self.hour_mask = self.hour_mask.checked_shl(shift).unwrap_or(0);
            self.last_hour = hour;
        }
        self.hour_mask |= 1;
        for bc in blocked.iter().take(2) {
            // Shift entries right to make room at index 0.
            let cap = WHY_NOT_RING_SIZE;
            for i in (1..cap).rev() {
                self.entries[i] = self.entries[i - 1];
            }
            self.entries[0] = Some(WhyNotEntry {
                tick,
                node: bc.node,
                band: bc.band,
                fact: bc.fact,
                op_code: WhyNotEntry::op_to_code(bc.op),
                threshold: bc.threshold,
                actual: bc.actual,
            });
            if (self.count as usize) < cap {
                self.count += 1;
            }
        }
    }

    /// Whether this person was blocked for >= threshold of the last window_hours.
    pub fn is_active(&self, current_hour: u64, window_hours: u32, threshold: u32) -> bool {
        let shift = current_hour.saturating_sub(self.last_hour).min(8) as u32;
        let mask = self.hour_mask.checked_shl(shift).unwrap_or(0);
        let count = (0..window_hours).filter(|i| mask & (1 << i) != 0).count() as u32;
        count >= threshold
    }

    /// Valid entries (newest first).
    pub fn latest(&self) -> &[Option<WhyNotEntry>] {
        &self.entries[..self.count as usize]
    }

    /// Approximate bytes used by this state (fixed size, no heap).
    pub fn byte_size(&self) -> usize {
        std::mem::size_of::<Self>()
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
