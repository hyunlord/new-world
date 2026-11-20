//! HOUSE1: persistent house identity, households and house ownership (ADR 0073).
//!
//! A house reuses its root [`BuildingSiteId`]; the grid and furniture stay the physical truth and
//! [`HouseView`] is a derived read over them. Households are persistent groups with a monotonic
//! [`HouseholdId`]; [`HousingState::ownership`] maps each owned house to exactly one household.
use super::building_site::BuildingSiteId;
use crate::components::{AgentId, SettlementId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A house is identified by the root site id of its first Dwelling template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HouseId(pub BuildingSiteId);

/// Persistent per-house history (H2's extension fields are out of scope).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseHistory {
    /// Tick of the first physical completion, when known.
    pub first_completed_tick: Option<u64>,
}

/// Monotonic, persistent household identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HouseholdId(pub u32);

/// One co-residential household.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Household {
    /// Persistent identity.
    pub id: HouseholdId,
    /// Oldest adult member (ties lowest id); a minor when no adult is left.
    pub head: AgentId,
    /// Living members.
    pub members: BTreeSet<AgentId>,
    /// Tick the household was formed.
    pub formed_tick: u64,
    /// Household it split from (a sub-household while it owns no house).
    pub parent: Option<HouseholdId>,
}

/// Post-marital residence rule (Murdock 1949), read from the dwelling data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResidenceRule {
    /// A couple forms a new household.
    #[default]
    Neolocal,
    /// The wife joins the husband's household.
    Patrilocal,
    /// The husband joins the wife's household.
    Matrilocal,
}

/// Why a house changed owner (chronicle `HouseOwnershipChanged` value byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipReason {
    /// A new Dwelling completed and went to a homeless household.
    Completed,
    /// A homeless household moved into a vacant house.
    Claimed,
    /// An heir household took over an orphaned house.
    Inherited,
    /// The owning household ended or its last adult died.
    Vacated,
    /// The owning household left the house's settlement.
    Departed,
    /// The house is physically lost.
    Lost,
}

impl OwnershipReason {
    /// Stable chronicle code.
    pub const fn code(self) -> u8 {
        match self {
            Self::Completed => 0,
            Self::Claimed => 1,
            Self::Inherited => 2,
            Self::Vacated => 3,
            Self::Departed => 4,
            Self::Lost => 5,
        }
    }
}

/// Largest allocatable household id: `id + 1` must fit the 27-bit fields of
/// [`pack_ownership_change`] so the packed value stays non-negative.
pub const MAX_HOUSEHOLD_ID: u32 = (1 << 27) - 2;

/// Pack a `HouseOwnershipChanged` chronicle value: reason in bits 0..8, `from + 1` in 8..35,
/// `to + 1` in 36..63 (0 = none; bit 63 stays clear).
pub fn pack_ownership_change(
    from: Option<HouseholdId>,
    to: Option<HouseholdId>,
    reason: OwnershipReason,
) -> i64 {
    let id = |h: Option<HouseholdId>| {
        h.filter(|h| h.0 <= MAX_HOUSEHOLD_ID)
            .map_or(0, |h| i64::from(h.0) + 1)
    };
    i64::from(reason.code()) | (id(from) << 8) | (id(to) << 36)
}

/// Inverse of [`pack_ownership_change`]: `(from, to, reason code)`.
pub fn unpack_ownership_change(value: i64) -> (Option<HouseholdId>, Option<HouseholdId>, u8) {
    let id = |bits: i64| {
        u32::try_from(bits & 0x07FF_FFFF)
            .ok()
            .and_then(|v| v.checked_sub(1))
            .map(HouseholdId)
    };
    (id(value >> 8), id(value >> 36), (value & 0xFF) as u8)
}

/// Saved house/household state (format26 tail); empty in every baseline world.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HousingState {
    /// Houses that physically completed at least once.
    pub houses: BTreeMap<HouseId, HouseHistory>,
    /// Living households.
    pub households: BTreeMap<HouseholdId, Household>,
    /// Owned houses; a house has at most one owner.
    pub ownership: BTreeMap<HouseId, HouseholdId>,
    /// Next household identity.
    pub next_household: u32,
    /// Whether founding (rule 1) has run for this world.
    pub founded: bool,
    /// Change counter: households, ownership or bed assignments moved (bridge cache key).
    pub revision: u64,
}

impl HousingState {
    /// The household holding `agent`, if any (lowest id if the state were inconsistent).
    pub fn household_of(&self, agent: AgentId) -> Option<HouseholdId> {
        self.households
            .values()
            .find(|household| household.members.contains(&agent))
            .map(|household| household.id)
    }

    /// Houses owned by `household`, in id order.
    pub fn owned_by(&self, household: HouseholdId) -> Vec<HouseId> {
        self.ownership
            .iter()
            .filter(|(_, owner)| **owner == household)
            .map(|(house, _)| *house)
            .collect()
    }

    /// Allocate the next household id; `None` on exhaustion.
    pub fn allocate(&mut self) -> Option<HouseholdId> {
        let id = self.next_household;
        if id > MAX_HOUSEHOLD_ID {
            return None;
        }
        self.next_household = id + 1;
        Some(HouseholdId(id))
    }

    /// Structural checks for a decoded save: unique membership, owners exist, known houses,
    /// ids below the allocator.
    pub fn validate(&self) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for (id, household) in &self.households {
            if *id != household.id || id.0 >= self.next_household || id.0 > MAX_HOUSEHOLD_ID {
                return Err(format!("household {} identity", id.0));
            }
            if household.members.is_empty() || !household.members.contains(&household.head) {
                return Err(format!(
                    "household {} has no members or a head outside them",
                    id.0
                ));
            }
            if household
                .parent
                .is_some_and(|parent| parent.0 >= self.next_household)
            {
                return Err(format!("household {} parent out of range", id.0));
            }
            for member in &household.members {
                if !seen.insert(*member) {
                    return Err(format!("agent {member} in two households"));
                }
            }
        }
        for (house, owner) in &self.ownership {
            if !self.households.contains_key(owner) {
                return Err(format!("house {} owned by a missing household", house.0));
            }
            if !self.houses.contains_key(house) {
                return Err(format!("owned house {} has no history", house.0));
            }
        }
        if !self.founded && (!self.households.is_empty() || !self.ownership.is_empty()) {
            return Err("unfounded housing state holds households".into());
        }
        Ok(())
    }
}

/// Current physical state of a house.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HouseStatus {
    /// Every piece of the current generation stands.
    Complete,
    /// A same-id generation (upgrade) is being built.
    Upgrading,
    /// The site lost physical pieces; the id and history stay.
    Damaged,
}

impl HouseStatus {
    /// Stable bridge name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Upgrading => "upgrading",
            Self::Damaged => "damaged",
        }
    }
}

/// One bed slot source inside a house: the tile the bed assignment uses and its slot count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedRef {
    /// Bed tile (bedroll tile or hut centre).
    pub tile: (u32, u32),
    /// Sleeping slots at this tile.
    pub slots: u32,
    /// Whether the house's settlement can walk to it.
    pub reachable: bool,
}

/// Derived read model of one house (never saved).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseView {
    /// House identity (root site id).
    pub id: HouseId,
    /// Settlement whose formation tile is nearest (ties lowest id).
    pub settlement: Option<SettlementId>,
    /// Current template id.
    pub template: String,
    /// Current template era id.
    pub era: String,
    /// Current template tier.
    pub tier: u8,
    /// Dwelling yard stage.
    pub yard_stage: u8,
    /// Occupied days counted by house growth.
    pub occupied_days: u32,
    /// Physical status.
    pub status: HouseStatus,
    /// First physical completion, when known.
    pub first_completed_tick: Option<u64>,
    /// Footprint cells of completed pieces, row-major.
    pub completed_cells: Vec<(u32, u32)>,
    /// Roofed interior room cells, row-major (empty unless intact).
    pub roof_cells: Vec<(u32, u32)>,
    /// Rooms strictly inside the footprint.
    pub room_ids: Vec<u32>,
    /// Beds inside the footprint.
    pub bed_refs: Vec<BedRef>,
    /// Hearth tiles inside the footprint.
    pub hearth_refs: Vec<(u32, u32)>,
    /// Living agents whose assigned bed is in this house.
    pub resident_ids: Vec<AgentId>,
    /// Parallel to `resident_ids`: each resident's current family group (`households()`, keyed
    /// by the group's lowest AgentId) — a display key, not the persistent household.
    pub household_groups: Vec<AgentId>,
    /// The group with most residents here, ties lowest key; `None` when nobody lives here.
    pub primary_household: Option<AgentId>,
    /// Reachable bed slots.
    pub capacity: u32,
    /// Owning household.
    pub owner_household: Option<HouseholdId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ownership_values_round_trip() {
        for (from, to, reason) in [
            (None, Some(HouseholdId(0)), OwnershipReason::Completed),
            (Some(HouseholdId(7)), None, OwnershipReason::Lost),
            (
                Some(HouseholdId(3)),
                Some(HouseholdId(9)),
                OwnershipReason::Inherited,
            ),
            (
                Some(HouseholdId(MAX_HOUSEHOLD_ID)),
                Some(HouseholdId(MAX_HOUSEHOLD_ID)),
                OwnershipReason::Claimed,
            ),
        ] {
            let value = pack_ownership_change(from, to, reason);
            assert!(value >= 0);
            assert_eq!(unpack_ownership_change(value), (from, to, reason.code()));
        }
    }

    #[test]
    fn validate_rejects_double_membership_and_dangling_owner() {
        let mut state = HousingState {
            founded: true,
            next_household: 2,
            ..Default::default()
        };
        for id in [0, 1] {
            state.households.insert(
                HouseholdId(id),
                Household {
                    id: HouseholdId(id),
                    head: 5,
                    members: BTreeSet::from([5]),
                    formed_tick: 0,
                    parent: None,
                },
            );
        }
        assert!(state.validate().is_err());
        state.households.remove(&HouseholdId(1));
        assert!(state.validate().is_ok());
        state.houses.insert(HouseId(4), HouseHistory::default());
        state.ownership.insert(HouseId(4), HouseholdId(1));
        assert!(state.validate().is_err());
        state.ownership.clear();
        if let Some(household) = state.households.get_mut(&HouseholdId(0)) {
            household.head = 6;
        }
        assert!(state.validate().is_err(), "head outside members");
        if let Some(household) = state.households.get_mut(&HouseholdId(0)) {
            household.members.clear();
        }
        assert!(state.validate().is_err(), "empty household");
    }

    #[test]
    fn allocation_stops_before_the_packed_id_range_ends() {
        let mut state = HousingState {
            next_household: MAX_HOUSEHOLD_ID,
            ..Default::default()
        };
        assert_eq!(state.allocate(), Some(HouseholdId(MAX_HOUSEHOLD_ID)));
        assert_eq!(state.allocate(), None);
    }
}
