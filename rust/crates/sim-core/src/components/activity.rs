//! Economy-neutral daily activities, enabled only by the routine rule.
use serde::{Deserialize, Serialize};

use super::SettlementId;

/// The stable activity order used by bridge codes and observer arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActivityKind {
    /// Attend the settlement hearth.
    TendFire,
    /// Work beside a stockpile without creating a product.
    Craft,
    /// Work beside a shelter without changing it.
    Repair,
    /// Spend time in the circle around the hearth.
    HearthCircle,
    /// Visit the outer ring around the settlement.
    Explore,
    /// Move within one tile of a child's chosen place.
    Play,
    /// Rest near the hearth as an elder.
    SitByFire,
}

impl ActivityKind {
    /// Stable bridge code in the range zero through six.
    pub const fn code(self) -> u8 {
        match self {
            Self::TendFire => 0,
            Self::Craft => 1,
            Self::Repair => 2,
            Self::HearthCircle => 3,
            Self::Explore => 4,
            Self::Play => 5,
            Self::SitByFire => 6,
        }
    }

    /// Index into the seven activity weights or observer counters.
    pub const fn index(self) -> usize {
        self.code() as usize
    }
}

/// A place reserved while walking to or performing one activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    /// The chosen activity.
    pub kind: ActivityKind,
    /// Home settlement at the time of choice.
    pub settlement: SettlementId,
    /// Reserved world tile, in x/y order.
    pub place: (u32, u32),
    /// Simulation tick of the choice, used for the walk timeout.
    pub chosen_tick: u64,
    /// First tick at the place; absent while walking.
    pub arrived_tick: Option<u64>,
    /// Exclusive duration deadline; absent while walking.
    pub until_tick: Option<u64>,
}

/// Pause between activities, independent of higher-priority needs or work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityRest {
    /// Earliest simulation tick at which another activity may start.
    pub until_tick: u64,
}

/// One decision event, without cumulative simulation-side counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutineOutcome {
    /// A place was reserved, possibly after the one allowed redraw.
    Started {
        /// Whether the original kind lacked a place.
        redrawn: bool,
    },
    /// No eligible kind had a place.
    NoPlace {
        /// Whether the fallback choice was attempted.
        redrawn: bool,
    },
    /// An activity ended or was interrupted.
    Ended {
        /// True only for a duration/day-end exit after arrival.
        completed: bool,
    },
}

/// Latest routine event; the probe consumes it only on its exact tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineObservation {
    /// Simulation tick of the event.
    pub tick: u64,
    /// Activity kind, absent if no kind could be selected.
    pub kind: Option<ActivityKind>,
    /// Lifecycle result for the probe's counters and raw event trace.
    pub outcome: RoutineOutcome,
}

/// Ordered events when an activity starts and is interrupted in the same tick.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineObservationBatch {
    /// Simulation tick shared by all events; older batches are ignored.
    pub tick: u64,
    /// Decision order, preserving starts and redraws before an interruption.
    pub events: Vec<RoutineObservation>,
}
