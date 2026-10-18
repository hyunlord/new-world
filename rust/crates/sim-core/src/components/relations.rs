//! Pass 20 (`game_v25`): how two villages feel about each other, the porter
//! trips in flight, and measurement-only records of trips and daily decisions.
//! Stored in `SimResources::settlement_relations`; empty in every scene
//! without the relations rule.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{AgentId, PorterKind, PorterTrip, SettlementId};

/// Lowest relation value.
const RELATION_MIN: i32 = -1000;
/// Highest relation value.
const RELATION_MAX: i32 = 1000;

/// The relation of one unordered pair of villages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationState {
    /// Current relation in −1,000..=1,000.
    pub value: i32,
    /// No porters, and each avoids bushes nearer the other's hearth.
    pub estranged: bool,
    /// The kin base the value drifts toward (fixed when the pair appears).
    pub base: i32,
}

impl RelationState {
    /// Add `delta`, clamped to −1,000..=1,000.
    pub fn add(&mut self, delta: i32) {
        self.value = self.value.saturating_add(delta).clamp(RELATION_MIN, RELATION_MAX);
    }
}

/// One porter's trip as booked (measurement only): withdrawn = delivered +
/// returned + lost once the trip has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TripRecord {
    /// The porter.
    pub porter: AgentId,
    /// The trip as it departed.
    pub trip: PorterTrip,
    /// Departure cause: 0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood.
    pub cause: u8,
    /// Units stored in the receiver's store.
    pub delivered: u32,
    /// Units stored back in the sender's store.
    pub returned: u32,
    /// Units lost (the porter died, or the sender dissolved).
    pub lost: u32,
    /// Whether the porter turned back before delivering.
    pub failed: bool,
    /// Tick the trip ended; `None` while in flight.
    pub end_tick: Option<u64>,
}

/// One ordered pair (sender, receiver) at a decision minute (measurement only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairDay {
    /// Tick of the decision.
    pub tick: u64,
    /// Possible sender.
    pub from: SettlementId,
    /// Possible receiver.
    pub to: SettlementId,
    /// Relation after the day's drift.
    pub value: i32,
    /// Estranged after the day's drift.
    pub estranged: bool,
    /// Within a porter's reach.
    pub in_reach: bool,
    /// Sender's food days (permille), store food, wood nights and store wood.
    pub from_measure: [u64; 4],
    /// Receiver's food days (permille), store food, wood nights and store wood.
    pub to_measure: [u64; 4],
    /// The kin-aid food condition held (before reach, estrangement and slots).
    pub aid_food: bool,
    /// The kin-aid wood condition held.
    pub aid_wood: bool,
    /// The exchange condition held (sender sends food, receiver wood).
    pub exchange: bool,
    /// Units sent from `from` to `to` today and what (`None` = no trip).
    pub sent: Option<(PorterKind, u32)>,
}

/// Every village pair's relation, the trips in flight and the
/// measurement-only logs (Pass 20).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementRelations {
    /// Every unordered pair of living villages, lower id first.
    pub pairs: BTreeMap<(SettlementId, SettlementId), RelationState>,
    /// Trips in flight by porter (read to book a dead porter's load).
    pub active: BTreeMap<AgentId, TripRecord>,
    /// Trips that delivered and already credited their pair (the first
    /// porter of a trip credits it), keyed (sender, receiver, depart tick).
    pub credited: BTreeSet<(SettlementId, SettlementId, u64)>,
    /// Display day of `disputed`.
    pub dispute_day: u64,
    /// Forager pairs (lower id first) already evaluated on `dispute_day`.
    pub disputed: BTreeSet<(AgentId, AgentId)>,
    /// Pass 21 (daily caps): per village pair (lower id first), the disputes
    /// that cost `dispute_loss` and the quarrels on `dispute_day`; cleared
    /// with `disputed`. Written only under a cap (`dispute_cap_per_pair_day`
    /// or `quarrel_cap_per_pair_day` > 0), so empty in `game_v25`.
    #[serde(default)]
    pub day_counts: BTreeMap<(SettlementId, SettlementId), (u32, u32)>,
    /// Ended trips in end order (measurement only).
    pub trips: Vec<TripRecord>,
    /// Every ordered pair at every decision minute (measurement only).
    pub days: Vec<PairDay>,
}

impl SettlementRelations {
    /// The key of the pair `a`, `b` (lower id first).
    pub const fn key(a: SettlementId, b: SettlementId) -> (SettlementId, SettlementId) {
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// The relation of `a` and `b`, if both are living villages.
    pub fn get(&self, a: SettlementId, b: SettlementId) -> Option<&RelationState> {
        self.pairs.get(&Self::key(a, b))
    }

    /// Whether `a` and `b` are estranged.
    pub fn estranged(&self, a: SettlementId, b: SettlementId) -> bool {
        self.get(a, b).is_some_and(|state| state.estranged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_clamps_to_the_relation_range() {
        let mut state = RelationState { value: 990, estranged: false, base: 0 };
        state.add(40);
        assert_eq!(state.value, RELATION_MAX);
        state.add(-2500);
        assert_eq!(state.value, RELATION_MIN);
        state.add(i32::MIN);
        assert_eq!(state.value, RELATION_MIN);
        state.add(i32::MAX);
        assert_eq!(state.value, RELATION_MAX);
    }

    #[test]
    fn pair_keys_put_the_lower_id_first() {
        let mut relations = SettlementRelations::default();
        assert_eq!(SettlementRelations::key(5, 2), (2, 5));
        assert_eq!(SettlementRelations::key(2, 5), (2, 5));
        relations.pairs.insert((2, 5), RelationState { value: -400, estranged: true, base: 150 });
        assert!(relations.estranged(5, 2) && relations.estranged(2, 5));
        assert!(!relations.estranged(2, 6));
        assert_eq!(relations.get(5, 2).map(|s| s.value), Some(-400));
    }
}
