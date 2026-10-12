//! Hidden world parameters drawn from a world's seed (Pass 10).
//!
//! Each parameter is one keyed draw on `STREAM_WORLD_VARIETY` with slot =
//! its index in the struct (0..7; Pass 11's wild regions 8..10) and subject,
//! tick and replicate 0, so a noise fork never changes the world. An integer range `lo..=hi` is
//! `lo + below(draw, hi - lo + 1)`; the climate is the index of
//! `below(draw, 1000)` in the cumulative `CLIMATE_PERMILLE`.
use serde::{Deserialize, Serialize};

use crate::rng::{keyed_below, STREAM_WORLD_VARIETY};

/// Inclusive ranges of the drawn parameters (Pass 10 section 7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarietyRanges {
    /// Founders.
    pub starting_agents: (u32, u32),
    /// Renewable food sources.
    pub berry_bushes: (u32, u32),
    /// Maximum food at a bush.
    pub berry_capacity: (u32, u32),
    /// Deadwood spots requested.
    pub deadwood_spots: (u32, u32),
    /// Maximum wood at a spot.
    pub deadwood_capacity: (u32, u32),
    /// Forest share of non-river land, thousandths.
    pub forest_permille: (u32, u32),
    /// Scale of the seasonal food regrowth, thousandths.
    pub food_regen_permille: (u32, u32),
    /// Wild regions away from the camp (Pass 11, slot 8).
    pub wild_regions: (u32, u32),
    /// Bushes per wild region (Pass 11, slot 9).
    pub region_bushes: (u32, u32),
    /// Deadwood spots per wild region (Pass 11, slot 10).
    pub region_deadwood: (u32, u32),
}

/// The ranges `game_v16` draws from.
pub const RANGES: VarietyRanges = VarietyRanges {
    starting_agents: (18, 30),
    berry_bushes: (9, 15),
    berry_capacity: (3, 5),
    deadwood_spots: (14, 26),
    deadwood_capacity: (5, 7),
    forest_permille: (220, 380),
    food_regen_permille: (850, 1150),
    wild_regions: (1, 3),
    // Part 3 round 2: 5..=9 -> 7..=11 (+2 at both ends).
    region_bushes: (7, 11),
    // Part 3 round 3: 6..=12 -> 8..=14 (+2 at both ends).
    region_deadwood: (8, 14),
};

/// Climate weights in thousandths: mild-leaning, neutral, harsh-leaning.
pub const CLIMATE_PERMILLE: [u16; 3] = [333, 334, 333];

/// Winter severity weights (mild, normal, harsh) of each climate; the neutral
/// climate is v15's.
pub const CLIMATE_SEVERITY_PERMILLE: [[u16; 3]; 3] = [[500, 350, 150], [350, 400, 250], [200, 400, 400]];

const fn uniform(seed: u64, slot: u32, range: (u32, u32)) -> u32 {
    range.0 + keyed_below(seed, STREAM_WORLD_VARIETY, 0, 0, slot, 0, (range.1 - range.0 + 1) as u64) as u32
}

/// Pass 10: the hidden parameters of one world, drawn from its seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldVariety {
    /// Founders at tick 0 (v15: 24).
    pub starting_agents: u32,
    /// Renewable food sources (v15: 12).
    pub berry_bushes: u32,
    /// Maximum food at each bush (v15: 4).
    pub berry_capacity: u8,
    /// Deadwood spots requested from worldgen (v15: 20).
    pub deadwood_spots: u32,
    /// Maximum wood at each deadwood spot (v15: 6).
    pub deadwood_capacity: u8,
    /// Forest share of non-river land in thousandths (v15: 300).
    pub forest_permille: u32,
    /// 0 mild-leaning, 1 neutral, 2 harsh-leaning winters (v15: 1).
    pub climate: u8,
    /// Scale of `SeasonRule::regen_permille` in thousandths (v15: 1000).
    pub food_regen_permille: u32,
    /// Pass 11: wild regions placed away from the camp (`game_v17`; v16: 0).
    pub wild_regions: u8,
    /// Pass 11: bushes in each wild region (v16: 0).
    pub region_bushes: u32,
    /// Pass 11: deadwood spots in each wild region (v16: 0).
    pub region_deadwood: u32,
}

impl WorldVariety {
    /// The v15 world: every parameter at its Pass 9 value.
    pub const NEUTRAL: Self = Self {
        starting_agents: 24,
        berry_bushes: 12,
        berry_capacity: 4,
        deadwood_spots: 20,
        deadwood_capacity: 6,
        forest_permille: 300,
        climate: 1,
        food_regen_permille: 1000,
        wild_regions: 0,
        region_bushes: 0,
        region_deadwood: 0,
    };

    /// The hidden parameters of world `seed` (never reads a replicate).
    pub const fn draw(seed: u64) -> Self {
        Self::draw_with(seed, &RANGES, &CLIMATE_PERMILLE)
    }

    /// [`Self::draw`] with explicit ranges and climate weights.
    pub const fn draw_with(seed: u64, ranges: &VarietyRanges, climate_permille: &[u16; 3]) -> Self {
        let roll = keyed_below(seed, STREAM_WORLD_VARIETY, 0, 0, 6, 0, 1000) as u16;
        let climate = if roll < climate_permille[0] {
            0
        } else if roll < climate_permille[0] + climate_permille[1] {
            1
        } else {
            2
        };
        Self {
            starting_agents: uniform(seed, 0, ranges.starting_agents),
            berry_bushes: uniform(seed, 1, ranges.berry_bushes),
            berry_capacity: uniform(seed, 2, ranges.berry_capacity) as u8,
            deadwood_spots: uniform(seed, 3, ranges.deadwood_spots),
            deadwood_capacity: uniform(seed, 4, ranges.deadwood_capacity) as u8,
            forest_permille: uniform(seed, 5, ranges.forest_permille),
            climate,
            food_regen_permille: uniform(seed, 7, ranges.food_regen_permille),
            wild_regions: uniform(seed, 8, ranges.wild_regions) as u8,
            region_bushes: uniform(seed, 9, ranges.region_bushes),
            region_deadwood: uniform(seed, 10, ranges.region_deadwood),
        }
    }

    /// This world's parameters without wild regions: what `game_v16` uses
    /// and records (slots 0..7 only), so v16 outputs stay unchanged.
    pub const fn without_regions(&self) -> Self {
        Self { wild_regions: 0, region_bushes: 0, region_deadwood: 0, ..*self }
    }

    /// The winter severity weights of this world's climate.
    pub const fn winter_severity_permille(&self) -> [u16; 3] {
        CLIMATE_SEVERITY_PERMILLE[self.climate as usize]
    }

    /// `regen_permille` scaled by `food_regen_permille`: each entry
    /// `x * p / 1000`, rounded down (integer arithmetic; exact at p = 1000).
    pub const fn scale_regen(&self, regen_permille: [u16; 4]) -> [u16; 4] {
        let p = self.food_regen_permille;
        [
            (regen_permille[0] as u32 * p / 1000) as u16,
            (regen_permille[1] as u32 * p / 1000) as u16,
            (regen_permille[2] as u32 * p / 1000) as u16,
            (regen_permille[3] as u32 * p / 1000) as u16,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_is_the_v15_world_and_scales_nothing() {
        let n = WorldVariety::NEUTRAL;
        assert_eq!(n.winter_severity_permille(), [350, 400, 250]);
        assert_eq!(n.scale_regen([750, 1250, 1000, 500]), [750, 1250, 1000, 500]);
        let low = WorldVariety { food_regen_permille: 850, ..n };
        // 1250 x 850 / 1000 = 1062.5, rounded down.
        assert_eq!(low.scale_regen([750, 1250, 1000, 500]), [637, 1062, 850, 425]);
    }

    #[test]
    fn draws_stay_in_their_ranges() {
        for seed in 0..2_000 {
            let v = WorldVariety::draw(seed);
            assert!((18..=30).contains(&v.starting_agents));
            assert!((3..=5).contains(&v.berry_capacity));
            assert!(v.climate < 3);
            assert!((1..=3).contains(&v.wild_regions));
            assert!((7..=11).contains(&v.region_bushes));
            assert!((8..=14).contains(&v.region_deadwood));
            assert_eq!(v, WorldVariety::draw(seed));
        }
    }
}
