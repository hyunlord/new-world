//! Inert newborn life-stage marker; runtime rules decide when to attach it.
use serde::{Deserialize, Serialize};

/// A resident who has not yet reached the configured adult age.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Child {
    /// Simulation tick at which this resident was born.
    pub born_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::Child;

    #[test]
    fn child_marker_preserves_the_birth_tick() {
        let child = Child { born_tick: 1440 };
        assert_eq!(child.born_tick, 1440);
    }
}
