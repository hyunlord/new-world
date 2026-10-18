//! Inert aging substrate; `SocietyRules::aging` decides when it is attached.
use serde::{Deserialize, Serialize};

/// Birth tick and drawn lifespan of a resident under the aging rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Life {
    /// Simulation tick of birth; founders are born before tick zero.
    pub born_tick: i64,
    /// Ticks this resident lives before dying of old age.
    pub lifespan_ticks: u64,
}

impl Life {
    /// Age in ticks at simulation tick `tick`.
    pub fn age_ticks(&self, tick: u64) -> i64 {
        (tick as i64).saturating_sub(self.born_tick)
    }
}

/// Zero-sized marker for a resident past the configured elder age.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Elder;

#[cfg(test)]
mod tests {
    use super::Life;

    #[test]
    fn founder_age_counts_ticks_before_world_start() {
        let life = Life { born_tick: -8_640, lifespan_ticks: 83_520 };
        assert_eq!(life.age_ticks(0), 8_640);
        assert_eq!(life.age_ticks(1_440), 10_080);
    }
}
