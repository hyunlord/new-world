//! M5 (ADR 0030): mood and quarrels. `None` outside the M5 colony.
use serde::{Deserialize, Serialize};

/// Tunables of mood and quarrels (`ColonyRule::mood`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodRule {
    /// The mood value people start at and the target's base (goal 3.4: 500).
    pub base: i16,
    /// The value moves this fraction (1/n) of the gap to its target each interval (goal 3.4: 8).
    pub step_divisor: i16,
    /// Below this, tantrum risk: quarrel chance ×3 and work ×`low_work_permille` (goal 3.4: 250).
    pub low: i16,
    /// Below this, a breakdown (goal 3.4: 100).
    pub breakdown: i16,
    /// At or above this, content: work ×`high_work_permille` (goal 3.4: 750).
    pub high: i16,
    /// Work ticks under tantrum risk, ‰ (goal 3.4: 1,200).
    pub low_work_permille: u32,
    /// Work ticks when content, ‰ (goal 3.4: 900).
    pub high_work_permille: u32,
    /// Children have a mood from this age, in years (goal 3.4: 10).
    pub child_years: u64,
    /// Days a discovery cheers (goal 3.4: 3).
    pub joy_days: u16,
    /// Days a kin's death grieves (goal 3.4: 6).
    pub grief_days: u16,
    /// Days a breakdown lasts (goal 3.4: 1).
    pub breakdown_days: u16,
    /// A quarrel's base chance per check, ‰ (goal 3.4: 2).
    pub quarrel_base_permille: u32,
    /// Most quarrel chance per check, ‰ (goal 3.4: 40).
    pub quarrel_cap_permille: u32,
    /// Quarrels happen within this distance (goal 3.4: 2).
    pub quarrel_tiles: u32,
    /// Kin closer than this base never quarrel (goal 3.4: 300).
    pub kin_base_min: i16,
    /// The kin base of a parent, child, sibling or partner (DECISIONS D11: 500).
    pub kin_base: i16,
    /// A quarrel's relation loss before novelty seeking (goal 3.4: 60).
    pub quarrel_relation: i16,
    /// A quarrel's mood loss for both (goal 3.4: 100).
    pub quarrel_mood: i16,
    /// A quarrel's hostility gain, ‰ of the 0–1 hostility scale (goal 3.4: 0.1 = 100).
    pub quarrel_hostility_permille: u32,
    /// A quarrel turns into a fight at this hostility ‰ with both moods below `low` (goal 3.4: 0.6 = 600).
    pub fight_hostility_permille: u32,
    /// A quarrel's fight's damage (goal 3.4: 4).
    pub fight_damage: u16,
    /// A relation drifts back to its base by this much a day (DECISIONS D11: 10).
    pub relation_drift: i16,
    /// Non-kin pairs' base relation lies in −chemistry..=chemistry, keyed per pair (DECISIONS D11: 300).
    pub chemistry: i16,
    /// Most partners a person checks a run: the nearest within `quarrel_tiles`, by (distance, id)
    /// (DECISIONS D18: 8 — a crowd of 279 on one tile made the check O(N²), goal §4 and C3).
    #[serde(default = "partners_max")]
    pub quarrel_partners_max: u32,
    /// M5.5 calibration knobs (goal M5.5 §3.1.6, ADR 0030 amendment; M5 values by default): the weights of the
    /// pressure-related modifiers — cold last night, hungry, sick, a pressure active, a kin died (at most ×2 the M5
    /// weight).
    #[serde(default = "cold_weight")]
    pub cold_weight: i16,
    /// See [`Self::cold_weight`].
    #[serde(default = "hungry_weight")]
    pub hungry_weight: i16,
    /// See [`Self::cold_weight`].
    #[serde(default = "sick_weight")]
    pub sick_weight: i16,
    /// See [`Self::cold_weight`].
    #[serde(default = "pressure_weight")]
    pub pressure_weight: i16,
    /// See [`Self::cold_weight`].
    #[serde(default = "grief_weight")]
    pub grief_weight: i16,
    /// The quarrel formula's constants (goal 3.4: chance = base + (low − mood) / `quarrel_mood_divisor` + (−relation)
    /// / `quarrel_relation_divisor`, × `quarrel_low_factor` under tantrum risk).
    #[serde(default = "mood_divisor")]
    pub quarrel_mood_divisor: i32,
    /// See [`Self::quarrel_mood_divisor`].
    #[serde(default = "relation_divisor")]
    pub quarrel_relation_divisor: i32,
    /// See [`Self::quarrel_mood_divisor`].
    #[serde(default = "low_factor")]
    pub quarrel_low_factor: i32,
}

const fn partners_max() -> u32 {
    8
}

const fn cold_weight() -> i16 {
    -120
}

const fn hungry_weight() -> i16 {
    -150
}

const fn sick_weight() -> i16 {
    -80
}

const fn pressure_weight() -> i16 {
    -40
}

const fn grief_weight() -> i16 {
    -250
}

const fn mood_divisor() -> i32 {
    25
}

const fn relation_divisor() -> i32 {
    100
}

const fn low_factor() -> i32 {
    3
}

impl MoodRule {
    /// The M5 values (goal 3.4; DECISIONS D11).
    pub const M5: Self = Self {
        base: 500,
        step_divisor: 8,
        low: 250,
        breakdown: 100,
        high: 750,
        low_work_permille: 1_200,
        high_work_permille: 900,
        child_years: 10,
        joy_days: 3,
        grief_days: 6,
        breakdown_days: 1,
        quarrel_base_permille: 2,
        quarrel_cap_permille: 40,
        quarrel_tiles: 2,
        kin_base_min: 300,
        kin_base: 500,
        quarrel_relation: 60,
        quarrel_mood: 100,
        quarrel_hostility_permille: 100,
        fight_hostility_permille: 600,
        fight_damage: 4,
        relation_drift: 10,
        chemistry: 300,
        quarrel_partners_max: 8,
        cold_weight: -120,
        hungry_weight: -150,
        sick_weight: -80,
        pressure_weight: -40,
        grief_weight: -250,
        quarrel_mood_divisor: 25,
        quarrel_relation_divisor: 100,
        quarrel_low_factor: 3,
    };

    /// The M5.5 values: [`Self::M5`] as calibrated on seeds 0–7 (goal M5.5 §3.1.6; M5.5 DECISIONS, calibration rounds).
    pub const M5_5: Self = Self {
        // Calibration (M5.5 DECISIONS D15): r1 fewer quarrels between strangers, the pressure felt more; r2 a
        // breakdown a little rarer, a low mood less quarrelsome.
        quarrel_base_permille: 1,
        quarrel_relation_divisor: 200,
        quarrel_mood_divisor: 40,
        pressure_weight: -80,
        sick_weight: -160,
        breakdown: 105,
        ..Self::M5
    };
}
