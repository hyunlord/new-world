//! M5 (ADR 0027): the storyteller's rule — when pressure comes and what it does. `None` outside the M5 colony.
use serde::{Deserialize, Serialize};

/// Tunables of the storyteller and of the four pressure kinds (`ColonyRule::story`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryRule {
    /// Shortest and longest cold snap in days (goal 3.1: 2–4).
    pub cold_snap_days: (u32, u32),
    /// Shortest and longest drought in days (goal 3.1: 4–6).
    pub drought_days: (u32, u32),
    /// Days a person stays sick (goal 3.1: 3).
    pub sick_days: u32,
    /// Most days a wolf pack stays before it leaves (goal 3.2: 2).
    pub beasts_max_days: u32,
    /// No drought or beasts before this day (goal 3.1: 12).
    pub mercy_day: u32,
    /// No illness before this day (goal 3.1: 18).
    pub illness_day: u32,
    /// No cold snap in the first winter's first days (goal 3.1: 2).
    pub first_winter_mercy_days: u32,
    /// No beasts within this many days of the last beasts (goal 3.1: 6).
    pub beasts_gap_days: u32,
    /// No beasts in winter's last days (goal 3.1: "any but winter's last days"; DECISIONS: 2).
    pub winter_end_mercy_days: u32,
    /// Fewest days between two majors (goal 3.1: 6).
    pub major_gap_days: u32,
    /// A cold snap or drought lasts a day more with this many adults (goal 3.1: 20).
    pub big_colony_adults: u32,
    /// Burn per hour in a cold snap, ‰ of the night's rate (goal 3.1: ×2).
    pub snap_burn_permille: u32,
    /// Exposure damage of a cold person outside a roofed room in a snap, ‰ (goal 3.1: ×2).
    pub snap_exposure_permille: u32,
    /// Work ticks outdoors in a cold snap, ‰ (goal 3.1: 1,300).
    pub snap_outdoor_work_permille: u32,
    /// Forage regrowth in a drought, ‰ (goal 3.1: 400).
    pub drought_regen_permille: u32,
    /// Thirst growth in a drought, ‰ (goal 3.1: 1,200).
    pub drought_thirst_permille: u32,
    /// Work ticks of a sick person, ‰ (goal 3.1: 1,500).
    pub sick_work_permille: u32,
    /// Sleep need growth of a sick person, ‰ (goal 3.1: 1,500).
    pub sick_sleep_permille: u32,
    /// Chance a night that a sick person's illness passes to one who sleeps in the same bedroom, ‰ (goal 3.1: 200).
    pub sick_spread_permille: u32,
    /// Wolves in a pack before wealth (goal 3.1: 2).
    pub wolves_base: u32,
    /// One wolf more per this many items on stockpile cells (goal 3.1: 150).
    pub wolves_per_items: u32,
    /// Most wolves in a pack (goal 3.1: 4).
    pub wolves_max: u32,
    /// A cold snap is due when none came in the last this many days (DECISIONS D7: a year).
    pub cold_due_days: u32,
    /// Beasts are due when none came in the last this many days (DECISIONS D7: three seasons).
    pub beasts_due_days: u32,
    /// Whether illness is part of the pass (droppable 6.4; on in M5).
    pub illness: bool,
    /// Whether beasts come at all (on in M5; the soak turns them off).
    #[serde(default = "yes")]
    pub beasts: bool,
}

const fn yes() -> bool {
    true
}

impl StoryRule {
    /// The M5 values (goal 3.1; DECISIONS D7).
    pub const M5: Self = Self {
        cold_snap_days: (2, 4),
        drought_days: (4, 6),
        sick_days: 3,
        beasts_max_days: 2,
        mercy_day: 12,
        illness_day: 18,
        first_winter_mercy_days: 2,
        beasts_gap_days: 6,
        winter_end_mercy_days: 2,
        major_gap_days: 6,
        big_colony_adults: 20,
        snap_burn_permille: 2_000,
        snap_exposure_permille: 2_000,
        snap_outdoor_work_permille: 1_300,
        drought_regen_permille: 400,
        drought_thirst_permille: 1_200,
        sick_work_permille: 1_500,
        sick_sleep_permille: 1_500,
        sick_spread_permille: 200,
        wolves_base: 2,
        wolves_per_items: 150,
        wolves_max: 4,
        cold_due_days: 24,
        beasts_due_days: 18,
        illness: true,
        beasts: true,
    };
}
