//! Revitalize Pass 7 — the god's hand: interventions applied between ticks.
//!
//! The types live here so [`crate::SimResources`] can hold the log; the code
//! that applies them lives in `sim_systems::runtime::intervention`, which can
//! read seasons and the firewood helpers. Interventions draw no random numbers
//! and run only when a schedule or the FFI command asks for one.

use sim_core::components::SettlementId;

/// One act of the god's hand. Amounts are units, except `Rain` (hours) and
/// `ColdSnap` (nights).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intervention {
    /// Add food to a settlement's stockpile.
    GiftFood {
        /// Receiving settlement.
        settlement: SettlementId,
        /// Food units to add.
        amount: u32,
    },
    /// Remove food from a settlement's stockpile (never below zero).
    SpoilFood {
        /// Affected settlement.
        settlement: SettlementId,
        /// Food units to remove.
        amount: u32,
    },
    /// Add wood to a settlement's stockpile.
    GiftWood {
        /// Receiving settlement.
        settlement: SettlementId,
        /// Wood units to add.
        amount: u32,
    },
    /// Remove wood from a settlement's stockpile (never below zero).
    RotWood {
        /// Affected settlement.
        settlement: SettlementId,
        /// Wood units to remove.
        amount: u32,
    },
    /// Rain puts a settlement's fire out for a number of hours.
    Rain {
        /// Affected settlement.
        settlement: SettlementId,
        /// Hours of rain from now.
        hours: u32,
    },
    /// The next nights are colder: every fire burns one more unit an hour and
    /// an unlit hearth leaves its members cold in any season.
    ColdSnap {
        /// Nights of cold snap to add.
        nights: u32,
    },
    /// Deadwood falls on the registered spots, one unit a spot per pass.
    Windfall {
        /// Units to place.
        units: u32,
    },
}

/// The record of one applied intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterventionRecord {
    /// Completed ticks when it was applied (the next `tick()` runs at this value).
    pub tick: u64,
    /// What was asked.
    pub intervention: Intervention,
    /// Units changed, hours or nights set, or units placed; 0 when nothing changed.
    pub applied: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_compare_by_value() {
        let a = InterventionRecord { tick: 5, intervention: Intervention::ColdSnap { nights: 1 }, applied: 1 };
        assert_eq!(a, a);
        assert_ne!(a, InterventionRecord { applied: 0, ..a });
    }
}
