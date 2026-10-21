//! The calendar (M1.5, ADR 0017): ticks per day, days per season, seasons per year.
//!
//! Every derived length (a year, a number of years or days in ticks) is a
//! method here, never a literal elsewhere. `baseline` keeps its V7 calendar
//! (7-day seasons, a 28-day year); the colony uses 6-day seasons and a 24-day
//! year (ADR 0004 left the calendar to M1.5).
use serde::{Deserialize, Serialize};

/// The world's calendar. Integer only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarRule {
    /// Simulation ticks per day (1 tick = 1 minute).
    pub ticks_per_day: u64,
    /// Days per season.
    pub days_per_season: u32,
    /// Seasons per year (spring, summer, autumn, winter).
    pub seasons_per_year: u32,
}

impl Default for CalendarRule {
    fn default() -> Self {
        Self::BASELINE
    }
}

impl CalendarRule {
    /// The baseline (V7) calendar: 1,440-tick days, 7-day seasons, 4 seasons.
    pub const BASELINE: Self = Self {
        ticks_per_day: 1_440,
        days_per_season: 7,
        seasons_per_year: 4,
    };

    /// The colony calendar (M1.5): 1,440-tick days, 6-day seasons, a 24-day year.
    pub const COLONY: Self = Self {
        ticks_per_day: 1_440,
        days_per_season: 6,
        seasons_per_year: 4,
    };

    /// Days per year.
    pub const fn days_per_year(&self) -> u64 {
        self.days_per_season as u64 * self.seasons_per_year as u64
    }

    /// Ticks per season.
    pub const fn ticks_per_season(&self) -> u64 {
        self.days_per_season as u64 * self.ticks_per_day
    }

    /// Ticks per year.
    pub const fn ticks_per_year(&self) -> u64 {
        self.days_per_year() * self.ticks_per_day
    }

    /// `years` in ticks.
    pub const fn years_to_ticks(&self, years: u64) -> u64 {
        years * self.ticks_per_year()
    }

    /// `days` in ticks.
    pub const fn days_to_ticks(&self, days: u64) -> u64 {
        days * self.ticks_per_day
    }

    /// Whole years in `age_ticks` (integer division; 0 for a negative age or
    /// an empty year).
    pub const fn whole_years(&self, age_ticks: i64) -> u64 {
        let year = self.ticks_per_year();
        if age_ticks <= 0 || year == 0 {
            0
        } else {
            age_ticks as u64 / year
        }
    }

    /// The date of display day `day` (days since the calendar's start):
    /// `(year index, season index, day of season, day of year)`, all zero-based.
    pub const fn date_of_day(&self, day: u64) -> (u64, u32, u32, u32) {
        let per_year = self.days_per_year();
        let per_season = self.days_per_season as u64;
        if per_year == 0 || per_season == 0 {
            return (0, 0, 0, 0);
        }
        let day_of_year = day % per_year;
        (
            day / per_year,
            (day_of_year / per_season) as u32,
            (day_of_year % per_season) as u32,
            day_of_year as u32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_lengths_of_both_calendars() {
        // Colony: 6 × 4 = 24 days, 24 × 1,440 = 34,560 ticks.
        let c = CalendarRule::COLONY;
        assert_eq!(c.days_per_year(), 24);
        assert_eq!(c.ticks_per_year(), 34_560);
        assert_eq!(c.ticks_per_season(), 8_640);
        assert_eq!(c.years_to_ticks(15), 518_400);
        assert_eq!(c.days_to_ticks(18), 25_920);
        // Baseline: 7 × 4 = 28 days, 40,320 ticks.
        let b = CalendarRule::BASELINE;
        assert_eq!(b.days_per_year(), 28);
        assert_eq!(b.ticks_per_year(), 40_320);
        assert_eq!(CalendarRule::default(), b);
    }

    #[test]
    fn whole_years_truncate_and_dates_split_the_year() {
        let c = CalendarRule::COLONY;
        assert_eq!(c.whole_years(34_559), 0);
        assert_eq!(c.whole_years(34_560), 1);
        assert_eq!(c.whole_years(-5), 0);
        // Day 0: year 0, spring, day 0. Day 31 = year 1, day 7 of the year:
        // summer (season 1), its second day (index 1).
        assert_eq!(c.date_of_day(0), (0, 0, 0, 0));
        assert_eq!(c.date_of_day(31), (1, 1, 1, 7));
        assert_eq!(c.date_of_day(23), (0, 3, 5, 23));
        let empty = CalendarRule {
            days_per_season: 0,
            ..c
        };
        assert_eq!(empty.date_of_day(9), (0, 0, 0, 0));
        assert_eq!(empty.whole_years(1_000_000), 0);
    }
}
