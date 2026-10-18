//! Pass 20 (`game_v25`): a porter carrying food or wood to another village
//! and back. Attached by `RelationsSystem` at departure and removed when the
//! porter is home again (or its load is lost); no `PorterTrip` exists in any
//! earlier scene.
//!
//! While it carries the marker the porter walks (`Seeking { Agent(own id) }`,
//! the founding leader's self-target pattern) to the receiver's hearth and
//! back; it follows no tether, never hauls home, is never marked `Cold` at
//! its own hearth, is skipped by the stockpile deposit and never joins a
//! fission group. It stays a member of its own village throughout.
use serde::{Deserialize, Serialize};

use super::{ResourceKind, SettlementId};

/// What a porter carries.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PorterKind {
    /// Food from the sender's store.
    Food = 0,
    /// Firewood from the sender's store.
    Wood = 1,
}

impl PorterKind {
    /// The stockpile resource of this load.
    pub const fn resource(self) -> ResourceKind {
        match self {
            Self::Food => ResourceKind::Food,
            Self::Wood => ResourceKind::Wood,
        }
    }

    /// Stable numeric discriminant (0 food, 1 wood).
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Which way a porter is walking.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PorterStage {
    /// Walking to the receiver's hearth with the load.
    Outbound = 0,
    /// Walking home (after delivering, or turned back with the load).
    Return = 1,
}

impl PorterStage {
    /// Stable numeric discriminant (0 out, 1 back).
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// One porter's trip (Pass 20).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PorterTrip {
    /// The sending village (the porter's own).
    pub from: SettlementId,
    /// The receiving village.
    pub to: SettlementId,
    /// Food or wood.
    pub kind: PorterKind,
    /// Units taken from the sender's store at departure.
    pub amount: u32,
    /// Outbound or on the way back.
    pub stage: PorterStage,
    /// Tick of departure.
    pub depart_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_are_stable() {
        assert_eq!(PorterKind::Food.as_u8(), 0);
        assert_eq!(PorterKind::Wood.as_u8(), 1);
        assert_eq!(PorterStage::Outbound.as_u8(), 0);
        assert_eq!(PorterStage::Return.as_u8(), 1);
        assert_eq!(PorterKind::Food.resource(), ResourceKind::Food);
        assert_eq!(PorterKind::Wood.resource(), ResourceKind::Wood);
    }
}
