//! M5.5 (ADR 0031, 0032): the neighbours, their migrant waves and their caravans. `None` outside the M5.5 colony.
use serde::{Deserialize, Serialize};

/// Tunables of the neighbours (`ColonyRule::neighbours`; goal M5.5 §3.2, §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighbourRule {
    /// Days of travel to a neighbour (goal: 2–5).
    pub distance_days: (u8, u8),
    /// A neighbour's people at the start (goal: 20–60).
    pub population: (u16, u16),
    /// A neighbour's attitude at the start (goal: 40–60).
    pub attitude_start: (u8, u8),
    /// Neolithic nodes a neighbour knows (goal: 2–4; every Paleolithic node besides).
    pub neolithic_known: (u8, u8),
    /// Wave 1's day window and size (goal: day 8–12, 2–4 people).
    pub wave1_days: (u32, u32),
    /// See [`Self::wave1_days`].
    pub wave1_size: (u8, u8),
    /// Wave 2's day window and size (goal: day 20–26, 2–6 people).
    pub wave2_days: (u32, u32),
    /// See [`Self::wave2_days`].
    pub wave2_size: (u8, u8),
    /// A seasonal wave's size (goal: 1–4).
    pub roll_size: (u8, u8),
    /// The seasonal roll's chance ‰ = min(`roll_max_permille`, `surplus_permille` · food-surplus days
    /// + stockpile value / `value_divisor` + attitude · `attitude_permille`) (goal: 800, 50, 50, 2).
    pub roll_max_permille: u32,
    /// See [`Self::roll_max_permille`].
    pub surplus_permille: u32,
    /// See [`Self::roll_max_permille`].
    pub value_divisor: u32,
    /// See [`Self::roll_max_permille`].
    pub attitude_permille: u32,
    /// Food units an adult eats in a day (the surplus days' denominator; M5.5 DECISIONS: 3 meals of 1 unit).
    pub food_per_adult_day: u32,
    /// No wave when the colony holds this many people (adults and children; goal: 40).
    pub population_cap: u32,
    /// Days a wave waits at the hearth for an answer (goal: 1).
    pub wait_days: u32,
    /// Attitude lost when a wave is declined or left unanswered (goal: 5; droppable 7.5).
    pub decline_attitude: u8,
    /// A migrant's proficiency in the neighbour's Paleolithic nodes, ‰ (goal: 450–700).
    pub paleo_permille: (u16, u16),
    /// A migrant's chance of each of the neighbour's Neolithic nodes, ‰ (goal: 500), and its proficiency (420–650).
    pub neo_chance_permille: u32,
    /// See [`Self::neo_chance_permille`].
    pub neo_permille: (u16, u16),
    /// A migrant's chance of arriving in a hide wrap, ‰ (goal: 500).
    pub wrap_permille: u32,
    /// Whether caravans come (ADR 0032).
    pub caravans: bool,
    /// A neighbour sends caravans at this attitude or above (goal: 30).
    pub caravan_attitude: u8,
    /// No caravan before this day (goal: 6).
    pub caravan_first_day: u32,
    /// Days a caravan stays at the hearth (goal: 1).
    pub caravan_stay_days: u32,
    /// Stacks of goods a caravan brings and their quality (goal: 6–10 stacks, quality 30–70).
    pub goods_stacks: (u8, u8),
    /// See [`Self::goods_stacks`].
    pub goods_quality: (u8, u8),
    /// Draw weight of an item of a node the neighbour knows and the colony does not, and of a staple (goal: 3, 1).
    pub lack_weight: u32,
    /// See [`Self::lack_weight`].
    pub staple_weight: u32,
    /// Traders buy at this ‰ of value and sell at that ‰ (goal: 800, 1,200).
    pub buy_permille: u32,
    /// See [`Self::buy_permille`].
    pub sell_permille: u32,
    /// Attitude gained per trade (goal: 3).
    pub trade_attitude: u8,
    /// A gift gains one attitude per this much value (goal: 10; droppable 8.4).
    pub gift_value_per_attitude: u32,
    /// A held artifact multiplies the discovery chance of its producing node, ‰ (goal: ×3 = 3,000).
    pub eureka_permille: u32,
}

impl NeighbourRule {
    /// The M5.5 values (goal §3.2, §3.3; M5.5 DECISIONS).
    pub const M5_5: Self = Self {
        distance_days: (2, 5),
        population: (20, 60),
        attitude_start: (40, 60),
        neolithic_known: (2, 4),
        wave1_days: (8, 12),
        wave1_size: (2, 4),
        wave2_days: (20, 26),
        wave2_size: (2, 6),
        roll_size: (1, 4),
        roll_max_permille: 800,
        surplus_permille: 50,
        value_divisor: 50,
        attitude_permille: 2,
        food_per_adult_day: 3,
        population_cap: 40,
        wait_days: 1,
        decline_attitude: 5,
        paleo_permille: (450, 700),
        neo_chance_permille: 500,
        neo_permille: (420, 650),
        wrap_permille: 500,
        caravans: true,
        caravan_attitude: 30,
        caravan_first_day: 6,
        caravan_stay_days: 1,
        goods_stacks: (6, 10),
        goods_quality: (30, 70),
        lack_weight: 3,
        staple_weight: 1,
        buy_permille: 800,
        sell_permille: 1_200,
        trade_attitude: 3,
        gift_value_per_attitude: 10,
        eureka_permille: 3_000,
    };
}
