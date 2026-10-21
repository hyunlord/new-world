//! M1.5 pregnancy (ADR 0017): attached by `FertilitySystem` at conception,
//! removed at the birth. Closed-form: only the conception and due ticks are stored.
use serde::{Deserialize, Serialize};

use super::{AgentId, SettlementId};

/// A woman expecting a child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pregnant {
    /// Simulation tick of the conception.
    pub conceived_tick: u64,
    /// Simulation tick at which the child is due.
    pub due_tick: u64,
    /// The father (her partner at the conception).
    pub father: AgentId,
}

impl Pregnant {
    /// Whole days left until the birth at simulation tick `tick` (rounded up; 0 when due).
    pub const fn days_left(&self, tick: u64, ticks_per_day: u64) -> u64 {
        if tick >= self.due_tick || ticks_per_day == 0 {
            0
        } else {
            (self.due_tick - tick).div_ceil(ticks_per_day)
        }
    }
}

/// A birth that is due: `FertilitySystem` queues it in
/// `SimResources::due_births`; `SettlementSystem` spawns the child through
/// its birth path on the next tick (sorted by mother).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DueBirth {
    /// The mother.
    pub mother: AgentId,
    /// The father.
    pub father: AgentId,
    /// The settlement whose store and hearth receive the child.
    pub settlement: SettlementId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_left_round_up_and_end_at_zero() {
        let p = Pregnant {
            conceived_tick: 0,
            due_tick: 25_920,
            father: 2,
        };
        assert_eq!(p.days_left(0, 1_440), 18);
        assert_eq!(p.days_left(1, 1_440), 18);
        assert_eq!(p.days_left(1_440, 1_440), 17);
        assert_eq!(p.days_left(25_919, 1_440), 1);
        assert_eq!(p.days_left(25_920, 1_440), 0);
        assert_eq!(p.days_left(30_000, 1_440), 0);
        assert_eq!(p.days_left(0, 0), 0);
    }
}
