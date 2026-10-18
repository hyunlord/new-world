//! Current-tick family witnesses survive the death of their subjects.

use serde::{Deserialize, Serialize};

use super::{AgentId, SettlementId, Temperament};

/// A single non-Agent entity carries the current tick's ordered events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyObservations {
    /// System tick to which every event belongs.
    pub tick: u64,
    /// Runtime insertion order; readers ignore this batch on other ticks.
    pub events: Vec<FamilyEvent>,
}

/// Exact family transitions, without cumulative production tallies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyEvent {
    /// A mutual partnership formed at this tick.
    Paired {
        /// Female member's stable AgentId.
        mother: AgentId,
        /// Male member's stable AgentId.
        father: AgentId,
    },
    /// Parent selection and identity, preserved even if the newborn dies now.
    Born {
        /// Newborn's stable AgentId.
        child: AgentId,
        /// Selected first parent, including the old-rule fallback.
        mother: Option<AgentId>,
        /// Selected second parent, including the old-rule fallback.
        father: Option<AgentId>,
        /// Whether no living qualifying couple existed at the birth tick.
        fallback: bool,
        /// Identity key derived from the selected first parent.
        lineage_key: u64,
        /// Newborn's lineage-keyed sex.
        female: bool,
        /// Inherited four-axis values before any possible same-tick death.
        temperament: Temperament,
    },
    /// Departure snapshot; observers join its id to a completed fission.
    Departed {
        /// Existing FissionRecord id; departure alone is not completion.
        fission_id: u32,
        /// Parent settlement before membership transfer.
        parent: SettlementId,
        /// Departure leader's stable AgentId.
        leader: AgentId,
        /// Selected departing members, in stable AgentId order.
        members: Vec<AgentId>,
        /// All living parent-settlement households at departure.
        households: Vec<Vec<AgentId>>,
        /// All mutual living parent-settlement couples at departure.
        couples: Vec<[AgentId; 2]>,
        /// Denominator witnessed by `couples`.
        couples_total: u32,
        /// Couples with exactly one partner in `members`.
        couples_split: u32,
        /// Households with both departing and remaining members.
        households_split: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_witnesses_round_trip_with_birth_and_departure_identities() {
        let batch = FamilyObservations {
            tick: u64::MAX,
            events: vec![
                FamilyEvent::Paired {
                    mother: 0,
                    father: 1,
                },
                FamilyEvent::Born {
                    child: u64::MAX,
                    mother: Some(0),
                    father: Some(1),
                    fallback: false,
                    lineage_key: u64::MAX,
                    female: true,
                    temperament: Temperament::from_axes([0, 500, 1000, 300]),
                },
                FamilyEvent::Departed {
                    fission_id: u32::MAX,
                    parent: 3,
                    leader: 0,
                    members: vec![0, 2],
                    households: vec![vec![0, 1, 2]],
                    couples: vec![[0, 1]],
                    couples_total: 1,
                    couples_split: 1,
                    households_split: 1,
                },
            ],
        };
        let encoded = ron::to_string(&batch).expect("family witnesses serialize");
        let decoded: FamilyObservations =
            ron::from_str(&encoded).expect("family witnesses deserialize");
        assert_eq!(decoded, batch);
    }
}
