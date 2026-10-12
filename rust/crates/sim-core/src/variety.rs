//! Hidden world parameters drawn from a world's seed (Pass 10).
use serde::{Deserialize, Serialize};

/// Pass 10: the hidden parameters of one world, drawn from its seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    };
}
