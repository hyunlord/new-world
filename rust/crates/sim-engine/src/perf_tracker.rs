//! Performance tracker for per-system tick timing.
//!
//! Tracks how long each system takes per tick and maintains a rolling history
//! of total tick durations. Only active when `debug_mode = true` on `SimEngine`.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const TICK_HISTORY_CAP: usize = 300;

/// Tracks per-system and per-tick execution times.
///
/// Only updated when `SimEngine::debug_mode` is true, incurring no overhead
/// in production builds.
pub struct PerfTracker {
    /// system_name → last execution time in microseconds
    pub system_times: HashMap<String, u64>,
    /// section_id → (label, total_us_this_tick, system_count)
    pub section_times: HashMap<String, (String, u64, u32)>,
    /// Ring buffer of total tick durations (capacity 300, microseconds)
    pub tick_history: VecDeque<u64>,
    /// Current tick total in microseconds (accumulates during a tick)
    pub current_tick_us: u64,

    // Internal timing state — not exposed publicly
    tick_start: Option<Instant>,
    system_starts: HashMap<String, Instant>,
}

impl PerfTracker {
    /// Create a new tracker with 300-entry tick history.
    pub fn new() -> Self {
        Self {
            system_times: HashMap::new(),
            section_times: HashMap::new(),
            tick_history: VecDeque::with_capacity(TICK_HISTORY_CAP),
            current_tick_us: 0,
            tick_start: None,
            system_starts: HashMap::new(),
        }
    }

    /// Start timing a new tick. Resets per-tick accumulators.
    pub fn begin_tick(&mut self) {
        self.current_tick_us = 0;
        self.section_times.clear();
        self.tick_start = Some(Instant::now());
    }

    /// End timing the current tick and push duration to history.
    pub fn end_tick(&mut self) {
        if let Some(start) = self.tick_start.take() {
            let us = start.elapsed().as_micros() as u64;
            self.current_tick_us = us;
            if self.tick_history.len() >= TICK_HISTORY_CAP {
                self.tick_history.pop_front();
            }
            self.tick_history.push_back(us);
        }
    }

    /// Start timing a specific system.
    pub fn begin_system(&mut self, name: &str) {
        self.system_starts.insert(name.to_string(), Instant::now());
    }

    /// End timing a specific system and record elapsed microseconds.
    pub fn end_system(&mut self, name: &str) {
        if let Some(start) = self.system_starts.remove(name) {
            let us = start.elapsed().as_micros() as u64;
            self.system_times.insert(name.to_string(), us);
        }
    }
}

impl Default for PerfTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_tracker_is_empty() {
        let tracker = PerfTracker::new();
        assert!(tracker.system_times.is_empty());
        assert!(tracker.tick_history.is_empty());
        assert_eq!(tracker.current_tick_us, 0);
    }

    #[test]
    fn tick_history_bounded_at_300() {
        let mut tracker = PerfTracker::new();
        for _ in 0..350 {
            tracker.begin_tick();
            tracker.end_tick();
        }
        assert_eq!(tracker.tick_history.len(), 300);
    }

    #[test]
    fn system_timing_records_elapsed() {
        let mut tracker = PerfTracker::new();
        tracker.begin_system("test_system");
        thread::sleep(Duration::from_micros(100));
        tracker.end_system("test_system");
        // should record at least some microseconds
        assert!(*tracker.system_times.get("test_system").unwrap() > 0);
    }

    #[test]
    fn end_system_without_begin_is_noop() {
        let mut tracker = PerfTracker::new();
        tracker.end_system("nonexistent"); // should not panic
        assert!(tracker.system_times.is_empty());
    }

    #[test]
    fn begin_tick_clears_section_times() {
        let mut tracker = PerfTracker::new();
        tracker.section_times.insert("A".to_string(), ("Survival".to_string(), 1000, 3));
        tracker.begin_tick();
        assert!(tracker.section_times.is_empty());
    }
}
