use serde::{Deserialize, Serialize};
use crate::config::GameConfig;

/// Game calendar — tracks simulation time.
///
/// Time structure (from GameConfig):
///   1 tick = 2 game hours (TICK_HOURS = 2)
///   12 ticks = 1 day (TICKS_PER_DAY = 12)
///   365 days = 1 year (DAYS_PER_YEAR = 365)
///   4380 ticks = 1 year (TICKS_PER_YEAR = 4380)
///
/// No "months" concept in the simulation — uses day_of_year (1-365) directly.
/// Month/day-of-month are display utilities only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCalendar {
    /// Absolute simulation tick (starts at 0)
    pub tick: u64,
    /// Current year (starts at 1)
    pub year: u32,
    /// Day of year (1-365)
    pub day_of_year: u32,
    /// Ticks per day (from config: 12)
    pub ticks_per_day: u32,
    /// Days per year (from config: 365)
    pub days_per_year: u32,
}

impl GameCalendar {
    pub fn new(config: &GameConfig) -> Self {
        Self {
            tick: 0,
            year: 1,
            day_of_year: 1,
            ticks_per_day: config.ticks_per_day,
            days_per_year: config.days_per_year,
        }
    }

    /// Advance the calendar by 1 tick
    pub fn advance_tick(&mut self) {
        self.tick += 1;
        if self.tick.is_multiple_of(self.ticks_per_day as u64) {
            self.day_of_year += 1;
            if self.day_of_year > self.days_per_year {
                self.day_of_year = 1;
                self.year += 1;
            }
        }
    }

    /// Total ticks per year
    #[inline]
    pub fn ticks_per_year(&self) -> u64 {
        self.ticks_per_day as u64 * self.days_per_year as u64
    }

    /// Convert tick count to fractional years
    pub fn ticks_to_years(&self, ticks: u64) -> f64 {
        ticks as f64 / self.ticks_per_year() as f64
    }

    /// Convert fractional years to ticks (rounded down)
    pub fn years_to_ticks(&self, years: f64) -> u64 {
        (years * self.ticks_per_year() as f64) as u64
    }

    /// Get current age in fractional years given birth_tick
    pub fn age_years(&self, birth_tick: u64) -> f64 {
        if self.tick < birth_tick {
            return 0.0;
        }
        self.ticks_to_years(self.tick - birth_tick)
    }

    /// Get current age in ticks given birth_tick
    pub fn age_ticks(&self, birth_tick: u64) -> u64 {
        self.tick.saturating_sub(birth_tick)
    }

    /// Approximate month (1-12) from day_of_year — display utility only
    pub fn month(&self) -> u32 {
        ((self.day_of_year - 1) / 30).min(11) + 1
    }

    /// Approximate day of month (1-30) from day_of_year — display utility only
    pub fn day_of_month(&self) -> u32 {
        ((self.day_of_year - 1) % 30) + 1
    }

    /// Elapsed complete years since start
    pub fn elapsed_years(&self) -> u32 {
        self.year.saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn test_advance_one_day() {
        let config = GameConfig::default();
        let mut cal = GameCalendar::new(&config);
        assert_eq!(cal.tick, 0);
        assert_eq!(cal.day_of_year, 1);

        for _ in 0..12 {
            cal.advance_tick();
        }
        assert_eq!(cal.tick, 12);
        assert_eq!(cal.day_of_year, 2);
        assert_eq!(cal.year, 1);
    }

    #[test]
    fn test_year_rollover() {
        let config = GameConfig::default();
        let mut cal = GameCalendar::new(&config);
        // Advance one full year (4380 ticks)
        for _ in 0..4380 {
            cal.advance_tick();
        }
        assert_eq!(cal.tick, 4380);
        assert_eq!(cal.year, 2);
        assert_eq!(cal.day_of_year, 1);
    }

    #[test]
    fn test_1000_tick_calendar() {
        let config = GameConfig::default();
        let mut cal = GameCalendar::new(&config);
        for _ in 0..1000 {
            cal.advance_tick();
        }
        // 1000 ticks / 4380 ticks_per_year ≈ 0.228 years — still year 1
        assert_eq!(cal.tick, 1000);
        assert_eq!(cal.year, 1);
        // 1000 / 12 = 83.3 days → day 84
        assert_eq!(cal.day_of_year, 84);
    }

    #[test]
    fn test_ticks_to_years() {
        let config = GameConfig::default();
        let cal = GameCalendar::new(&config);
        let years = cal.ticks_to_years(4380);
        assert!((years - 1.0).abs() < 1e-9);
    }
}
