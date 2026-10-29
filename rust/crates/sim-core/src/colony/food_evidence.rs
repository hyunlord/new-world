//! Bounded acquisition observations; receipts do not assert counterfactual causes.
use super::{food::FoodOwner, food::FoodTallies, CancelReason, ItemKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// Scalar colony flows: harvest, craft-in, craft-out, eaten, spoiled, lost, import, export.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowCounters(
    /// Nominal units in the documented flow order, including imports and exports.
    pub [u64; 8],
);
impl FlowCounters {
    /// Snapshot cumulative accounting without copying per-item maps.
    pub fn capture(t: &FoodTallies) -> Self {
        Self([
            t.harvested_units,
            t.craft_input_units,
            t.craft_output_units,
            t.eaten_units,
            t.spoiled_units,
            t.lost_units,
            t.traded_in_units,
            t.traded_out_units,
        ])
    }
    /// Subtract an earlier snapshot; reject any counter regression.
    pub fn since(self, before: Self) -> Option<Self> {
        let mut delta = [0; 8];
        for (i, value) in delta.iter_mut().enumerate() {
            *value = self.0[i].checked_sub(before.0[i])?;
        }
        Some(Self(delta))
    }
}

/// A colony-wide interval, never attributed exclusively to one person's attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowInterval {
    /// Tick at which the opening counters were observed.
    pub from_tick: u64,
    /// Tick at which the closing counters were observed.
    pub to_tick: u64,
    /// Differences of the cumulative colony counters across this interval.
    pub units: FlowCounters,
}

/// An observed harvest boundary, not an inference from death-time ripe plants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionReason {
    /// The target or its current ripe yield failed completion admission.
    TargetUnavailable,
    /// Actual output placement found insufficient legal ground capacity.
    OutputPlacementFull,
    /// Output creation rejected a non-capacity transaction precondition.
    OutputRejected,
    /// A held harvest job ended with this actual cancellation reason.
    Cancelled(CancelReason),
    /// The named harvest job was still physically held when the person died.
    Unfinished,
    /// Only use after a real admission check, never inferred from current settings.
    WorkDisabled,
    /// Only use when the actual indexed query had no ripe candidate in this region.
    NoRipeInRegion,
}

/// One timestamped observed boundary, not a reconstructed historical explanation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionReceipt {
    /// Tick of the observed admission, completion, cancellation or query.
    pub tick: u64,
    /// Actual target when one was selected; absent for untargeted queries.
    pub target: Option<(u32, u32)>,
    /// Assignment tick of the observed harvest job, if a job existed.
    pub job_started: Option<u64>,
    /// Factual outcome observed at this boundary.
    pub reason: AcquisitionReason,
}

/// Starts at a real failed attempt and ends at intake or successful harvest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionEpisode {
    /// First failure observation since the last successful intake or harvest.
    pub since_tick: u64,
    /// Colony counters captured when the episode began.
    pub counters: FlowCounters,
    /// Most recent actual observation; intervening conditions are not inferred.
    pub last: AcquisitionReceipt,
}

/// Immutable payload of one actual operation parent, keyed by Chronicle index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationEvidence {
    /// A personal acquisition observation beside explicitly colony-wide flows.
    Acquisition {
        /// Exact last boundary observation copied before the person was removed.
        receipt: AcquisitionReceipt,
        /// Colony flow since the episode began, not solely this person's output.
        flow: FlowInterval,
    },
    /// A completed intake that caused the first cooked-meal milestone.
    Intake {
        /// Physical owner observed at the operation boundary; it may later disappear.
        owner: FoodOwner,
        /// Actual registry identity of the consumed or placed item.
        item: ItemKind,
        /// Observed stack birth day; stored stacks use the conservative merged age.
        born_day: u32,
        /// Lifetime of the actual source storage class at intake.
        lifetime_days: u32,
    },
    /// Successful placement in this owner; source ancestry is not claimed.
    Stored {
        /// Physical owner observed at the operation boundary; it may later disappear.
        owner: FoodOwner,
        /// Actual registry identity of the consumed or placed item.
        item: ItemKind,
        /// Whole items successfully placed by the operation.
        pieces: u16,
        /// Observed stack birth day; stored stacks use the conservative merged age.
        born_day: u32,
    },
}

/// Saved observational state: bounded live history and immutable emitted causes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodEvidence {
    /// At most one live acquisition episode per affected person.
    pub active: BTreeMap<u64, AcquisitionEpisode>,
    /// At most four completed daily intervals; no per-item map copies.
    pub recent: VecDeque<FlowInterval>,
    /// Last daily observation tick and cumulative counters.
    pub cursor: Option<(u64, FlowCounters)>,
    /// Immutable terminal payloads keyed by their real Chronicle entry indexes.
    pub operations: BTreeMap<u32, OperationEvidence>,
}
