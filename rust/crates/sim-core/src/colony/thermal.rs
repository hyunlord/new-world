//! Colony-only physical heat, door intervals and dwelling history; explicit save sidecar.
use crate::components::SettlementId;
use crate::rooms::{RoomId, Tile};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
mod rules;
pub use rules::{DwellingStageDef, ThermalRules};

/// Authoritative colony heat and dwelling state, serialized as a separate tail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThermalState {
    /// Saved resolved profile; reload never substitutes current tuning.
    pub rules: ThermalRules,
    /// Last integrated temperatures by authoritative room identifier.
    pub rooms: BTreeMap<RoomId, RoomThermal>,
    /// Door timing by actual constructed tile.
    pub doors: BTreeMap<Tile, DoorThermal>,
    /// Last debit receipt by primary hearth settlement owner.
    pub paid_hearths: BTreeMap<SettlementId, PaidHeat>,
    /// Stage and deficiency start anchored to a real bed tile.
    pub dwellings: BTreeMap<Tile, DwellingThermal>,
    /// Last committed thermal boundary; None means no prior integration.
    pub last_integrated_tick: Option<u64>,
    /// Last charged cold exposure boundary.
    pub last_exposure_tick: Option<u64>,
    /// Derived presentation invalidation; omitted from save bytes.
    #[serde(skip)]
    pub view_version: u64,
}
impl ThermalState {
    /// Construct empty history from already validated rules.
    pub fn new(rules: ThermalRules) -> Self {
        Self {
            rules,
            rooms: BTreeMap::new(),
            doors: BTreeMap::new(),
            paid_hearths: BTreeMap::new(),
            dwellings: BTreeMap::new(),
            last_integrated_tick: None,
            last_exposure_tick: None,
            view_version: 0,
        }
    }
}
/// Uniform room temperature and footprint used for topology remapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomThermal {
    /// Last actual integration for this footprint; absent until its first paid interval.
    pub last_integrated_tick: Option<u64>,
    /// Uniform signed room temperature in deci-degrees.
    pub temperature_d10: i32,
    /// Previous integrated interior footprint for overlap remapping.
    pub tiles: BTreeSet<Tile>,
}
/// One physical door and its union of open simulation intervals.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoorThermal {
    /// Explicit persistent open order.
    pub held_open: bool,
    /// Automatic passage deadline in simulation ticks.
    pub open_until_tick: u64,
    /// Exclusive endpoint already added to the hourly open-time union.
    pub accounted_until_tick: u64,
    /// Union duration in the current sixty-tick interval.
    pub open_ticks_in_hour: u32,
}
/// Receipt from the existing physical fuel owner; never another fuel balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaidHeat {
    /// Simulation boundary at which the debit occurred.
    pub tick: u64,
    /// Physical hearth origin; heat applies only to its containing room.
    pub tile: Tile,
    /// Existing hourly fuel request.
    pub requested_wood: u32,
    /// Actual resource debit, including unsuccessful partial payment.
    pub debited_wood: u32,
}
/// Bed-anchored stage history independent of transient room identifiers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DwellingThermal {
    /// Earned authored stage identity; None means no dwelling benefit.
    pub stage: Option<u16>,
    /// Continuous deficiency start, cleared by recovery.
    pub deficient_since_tick: Option<u64>,
}
