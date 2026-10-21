//! The colony rule (M1): orders, items and jobs. `None` in `WorldRules::baseline`.
use serde::{Deserialize, Serialize};

/// Tunables of the M1 colony: trees, carrying, stockpiles and the hut.
/// Present only in `WorldRules::colony`; every M1 system returns at once
/// without it, so the baseline world never runs any of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyRule {
    /// Fewest logs a tree yields (keyed draw `(seed, "tree_logs", x, y)`).
    pub tree_logs_min: u8,
    /// Most logs a tree yields.
    pub tree_logs_max: u8,
    /// Work ticks at a tree before it falls.
    pub chop_ticks: u32,
    /// Logs one person carries per trip.
    pub carry_logs: u16,
    /// Logs one tile holds.
    pub stack_max_logs: u16,
    /// Hut footprint (width, height) in tiles.
    pub hut_footprint: (u32, u32),
    /// Logs a hut needs before work starts.
    pub hut_logs: u16,
    /// Work ticks once every log is delivered.
    pub hut_progress: u32,
    /// Beds (sleep capacity) at the hut's centre tile.
    pub hut_beds: u32,
    /// Chebyshev radius in which an idle person looks for work.
    pub job_scan_radius: u32,
    /// Ticks without Chebyshev progress after which a walking job is
    /// cancelled as stuck (7.5).
    pub stuck_ticks: u64,
}

impl ColonyRule {
    /// The M1 values (goal 6.1 table; 7.5 for `stuck_ticks`).
    pub const M1: Self = Self {
        tree_logs_min: 3,
        tree_logs_max: 5,
        chop_ticks: 90,
        carry_logs: 5,
        stack_max_logs: 25,
        hut_footprint: (3, 3),
        hut_logs: 12,
        hut_progress: 240,
        hut_beds: 4,
        job_scan_radius: 96,
        stuck_ticks: 240,
    };
}

impl super::WorldRules {
    /// The colony world's rules for world `seed` (M1 goal 8.1, ADR 0014; M1.5
    /// ADR 0017): [`super::WorldRules::baseline`] plus the colony rule, with
    /// automatic shelter planning off (the player builds), fission off (one
    /// colony until neighbours, ADR 0006), a step every 3 ticks instead of 8
    /// (8 is unusable on a 192-tile map: 40 tiles would take 5 sim hours), and
    /// since M1.5 the colony calendar (6-day seasons, a 24-day year), ages in
    /// years ([`COLONY_AGING`], [`COLONY_LIFE_STAGE`]) and births by a
    /// fertility rate ([`COLONY_BIRTH`]) instead of the settlement provision.
    pub const fn colony(seed: u64) -> Self {
        let base = Self::baseline(seed);
        let seasons = match base.seasons {
            Some(rule) => Some(super::SeasonRule {
                days_per_season: COLONY_CALENDAR.days_per_season,
                ..rule
            }),
            None => None,
        };
        let life_stage = match base.society.life_stage {
            Some(rule) => Some(super::LifeStageRule {
                child_ticks: COLONY_LIFE_STAGE.0,
                infant_ticks: Some(COLONY_LIFE_STAGE.1),
                ..rule
            }),
            None => None,
        };
        // M1.5 calibration (DECISIONS D9): autumn stores a whole colony winter
        // of burn (`None` = the calendar's days per season, 6) instead of the
        // baseline's 4 days of its 7-day winter.
        let firewood = match base.society.firewood {
            Some(rule) => Some(super::FirewoodRule {
                winter_prep_days: None,
                ..rule
            }),
            None => None,
        };
        let aging = match base.society.aging {
            Some(rule) => Some(super::AgingRule {
                elder_tether_radius: rule.elder_tether_radius,
                elder_evening_tether: rule.elder_evening_tether,
                ..COLONY_AGING
            }),
            None => None,
        };
        Self {
            agent: super::AgentRules {
                move_interval_ticks: 3,
                ..base.agent
            },
            society: super::SocietyRules {
                shelter_planning: None,
                fission: None,
                birth: COLONY_BIRTH,
                life_stage,
                aging,
                firewood,
                ..base.society
            },
            colony: Some(ColonyRule::M1),
            seasons,
            calendar: COLONY_CALENDAR,
            ..base
        }
    }
}

/// The colony calendar (M1.5 goal 5.1): 6-day seasons, a 24-day year.
pub const COLONY_CALENDAR: super::CalendarRule = super::CalendarRule::COLONY;

const fn years(y: u64) -> u64 {
    COLONY_CALENDAR.years_to_ticks(y)
}

/// The colony's aging in years (M1.5 goal 5.2): elder at 50, lifespan mean 62
/// and spread 8 (the mean of three keyed uniforms, clamped to 40–85),
/// founders 16–40 with at least 10 years left. The tethers are the baseline's.
pub const COLONY_AGING: super::AgingRule = super::AgingRule {
    elder_age_ticks: years(50),
    lifespan_mean_ticks: years(62),
    lifespan_spread_ticks: years(8),
    founder_age_min_ticks: years(16),
    founder_age_max_ticks: years(40),
    founder_min_remaining_ticks: years(10),
    elder_tether_radius: 4,
    elder_evening_tether: true,
    lifespan_bounds_ticks: Some((years(40), years(85))),
};

/// The colony's life stages (M1.5 goal 5.2): `(child until, infant until)`
/// in ticks = 15 and 3 years.
pub const COLONY_LIFE_STAGE: (u64, u64) = (years(15), years(3));

/// The colony's births (M1.5 goal 5.3, ADR 0017; DECISIONS D6 for the rate):
/// 33 permille a day gives about 0.25 births per woman-year once the 18 days
/// of gestation and 48 days after a birth are counted.
pub const COLONY_BIRTH: super::BirthRule = super::BirthRule::Fertility {
    conception_per_day_permille: 33,
    fertile_from_years: 16,
    fertile_to_years: 45,
    gestation_days: 18,
    postpartum_days: 48,
    requires_partner: true,
    max_mean_hunger: 60.0,
    max_mean_thirst: 60.0,
    food_cost: 10,
};
