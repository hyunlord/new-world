//! Inert newborn life-stage marker; runtime rules decide when to attach it.
use serde::{Deserialize, Serialize};

/// A resident who has not yet reached the configured adult age.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Child {
    /// Simulation tick at which this resident was born.
    pub born_tick: u64,
}

impl Child {
    /// M1.5: whether the child is an infant at `tick` (younger than
    /// `infant_ticks`; derived from the age, never stored). Infants keep the
    /// child behaviour: no foraging, no jobs, tethered at the hearth.
    pub const fn is_infant(&self, tick: u64, infant_ticks: Option<u64>) -> bool {
        match infant_ticks {
            Some(limit) => tick.saturating_sub(self.born_tick) < limit,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Child;

    #[test]
    fn child_marker_preserves_the_birth_tick() {
        let child = Child { born_tick: 1440 };
        assert_eq!(child.born_tick, 1440);
    }

    #[test]
    fn infancy_is_derived_from_the_age() {
        let child = Child { born_tick: 1_000 };
        assert!(child.is_infant(1_000, Some(500)));
        assert!(child.is_infant(1_499, Some(500)));
        assert!(!child.is_infant(1_500, Some(500)));
        assert!(!child.is_infant(1_000, None));
    }
}
