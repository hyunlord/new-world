//! Inert cold-night marker; `SocietyRules::firewood` decides when it is attached.
use serde::{Deserialize, Serialize};

/// Zero-sized marker for a settlement member spending a winter night in the
/// cold window while its settlement's fire is out (Pass 5). Attached and
/// removed only by `FireSystem`; `HungerDecaySystem` scales hunger growth by
/// `FirewoodRule::cold_hunger_permille` for marked agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Cold;

#[cfg(test)]
mod tests {
    use super::Cold;

    #[test]
    fn cold_is_a_zero_sized_marker() {
        assert_eq!(std::mem::size_of::<Cold>(), 0);
        assert_eq!(<Cold as Default>::default(), Cold);
    }
}
