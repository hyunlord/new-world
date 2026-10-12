//! Last cold hour of a resident under `SocietyRules::exposure` (Pass 10).
use serde::{Deserialize, Serialize};

/// The tick of the last `FireSystem` run that marked this resident `Cold`.
/// Attached and updated only by `FireSystem` under the exposure rule;
/// `StarvationSystem` withholds healing while the resident is `Cold` and for
/// `heal_block_hours` after the marked hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureClock {
    /// Simulation tick of the last `FireSystem` run that marked the resident `Cold`.
    pub last_cold_tick: u64,
}

impl ExposureClock {
    /// Whether healing is still blocked at `tick`: the marked hour itself
    /// (60 ticks) plus `heal_block_hours` whole hours after it.
    pub const fn blocks_healing(&self, tick: u64, heal_block_hours: u32) -> bool {
        tick < self.last_cold_tick + 60 + 60 * heal_block_hours as u64
    }
}

#[cfg(test)]
mod tests {
    use super::ExposureClock;

    #[test]
    fn healing_is_blocked_for_the_marked_hour_and_the_block() {
        let clock = ExposureClock { last_cold_tick: 600 };
        // Marked at 600: blocked through 659 (the hour) plus two hours (to 779).
        assert!(clock.blocks_healing(600, 2));
        assert!(clock.blocks_healing(779, 2));
        assert!(!clock.blocks_healing(780, 2));
        // No block hours: only the marked hour itself.
        assert!(clock.blocks_healing(659, 0));
        assert!(!clock.blocks_healing(660, 0));
    }
}
