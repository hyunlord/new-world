//! Hardiness of a resident under the Pass 11 exposure rule (`game_v17`).
use serde::{Deserialize, Serialize};

use crate::rng::{keyed_below, STREAM_HARDINESS};

/// Lowest drawn `permille`.
pub const HARDINESS_MIN_PERMILLE: u16 = 700;
/// Highest drawn `permille`.
pub const HARDINESS_MAX_PERMILLE: u16 = 1300;

/// A resident's hidden response to the cold (Pass 11). Despite the name,
/// `permille` is a **frailty factor**: the hourly exposure damage is
/// multiplied by `permille / 1000`, so a higher value means more damage.
/// Drawn once at creation (founders at worldgen, newborns at birth) from the
/// lineage key, never from `AgentId`, and always with replicate 0: an
/// identity trait, not noise. Attached only under
/// `ExposureRule::use_hardiness`; it never updates itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hardiness {
    /// Exposure damage multiplier in thousandths, uniform in 700..=1300.
    pub permille: u16,
}

impl Hardiness {
    /// The hardiness of the resident with lineage key `lineage_key` in world
    /// `seed`: `700 + keyed_below(seed, STREAM_HARDINESS, key, 0, 0, 0, 601)`.
    pub const fn draw(seed: u64, lineage_key: u64) -> Self {
        let span = (HARDINESS_MAX_PERMILLE - HARDINESS_MIN_PERMILLE) as u64 + 1;
        Self {
            permille: HARDINESS_MIN_PERMILLE
                + keyed_below(seed, STREAM_HARDINESS, lineage_key, 0, 0, 0, span) as u16,
        }
    }

    /// The damage multiplier `permille / 1000`.
    pub fn factor(&self) -> f64 {
        f64::from(self.permille) / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_stay_in_range_and_depend_on_the_key() {
        let mut seen = std::collections::BTreeSet::new();
        for key in 0..2_000u64 {
            let h = Hardiness::draw(7, key);
            assert!((HARDINESS_MIN_PERMILLE..=HARDINESS_MAX_PERMILLE).contains(&h.permille));
            assert_eq!(h, Hardiness::draw(7, key));
            seen.insert(h.permille);
        }
        assert!(seen.len() > 400, "uniform over 601 values");
        assert_eq!(Hardiness { permille: 1300 }.factor(), 1.3);
    }
}
