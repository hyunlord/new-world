//! Performance tracker for per-system tick timing.
//!
//! Tracks how long each system takes per tick and maintains a rolling history
//! of total tick durations. Only active when `debug_mode = true` on `SimEngine`.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const TICK_HISTORY_CAP: usize = 300;

/// Per-system cumulative statistics.
#[derive(Default, Clone, Debug)]
pub struct SystemStats {
    /// Total execution time across all calls (microseconds).
    pub total_us: u64,
    /// Number of times this system was called.
    pub call_count: u64,
    /// Maximum single-call execution time (microseconds).
    pub max_us: u64,
}

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
    /// Per-system cumulative stats (never cleared automatically, accumulates across all ticks).
    pub cumulative_stats: HashMap<String, SystemStats>,

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
            cumulative_stats: HashMap::new(),
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

            // Accumulate cumulative stats
            let stats = self.cumulative_stats
                .entry(name.to_string())
                .or_default();
            stats.total_us += us;
            stats.call_count += 1;
            if us > stats.max_us {
                stats.max_us = us;
            }
        }
    }

    /// Generate a performance report string sorted by total time (descending).
    pub fn report(&self) -> String {
        let mut entries: Vec<(&str, &SystemStats)> = self.cumulative_stats
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        entries.sort_by(|a, b| b.1.total_us.cmp(&a.1.total_us));

        let mut lines = Vec::new();
        lines.push(format!("{:<40} {:>10} {:>10} {:>10} {:>10}",
            "System", "Total(ms)", "Avg(µs)", "Max(µs)", "Calls"));
        lines.push("-".repeat(84));

        let mut grand_total_us: u64 = 0;
        for (name, stats) in &entries {
            let total_ms = stats.total_us as f64 / 1000.0;
            let avg_us = if stats.call_count > 0 {
                stats.total_us as f64 / stats.call_count as f64
            } else {
                0.0
            };
            grand_total_us += stats.total_us;
            lines.push(format!("{:<40} {:>10.1} {:>10.1} {:>10} {:>10}",
                name, total_ms, avg_us, stats.max_us, stats.call_count));
        }
        lines.push("-".repeat(84));
        lines.push(format!("TOTAL: {:.1}ms across {} systems",
            grand_total_us as f64 / 1000.0, entries.len()));

        // Tick statistics
        if !self.tick_history.is_empty() {
            let sum: u64 = self.tick_history.iter().sum();
            let count = self.tick_history.len() as u64;
            let avg_tick_us = sum / count.max(1);
            let max_tick_us = self.tick_history.iter().max().copied().unwrap_or(0);
            let min_tick_us = self.tick_history.iter().min().copied().unwrap_or(0);
            lines.push(format!("\nTick stats (last {} ticks):", count));
            lines.push(format!("  Avg: {:.2}ms  Min: {:.2}ms  Max: {:.2}ms  TPS: {:.1}",
                avg_tick_us as f64 / 1000.0,
                min_tick_us as f64 / 1000.0,
                max_tick_us as f64 / 1000.0,
                1_000_000.0 / avg_tick_us as f64));
        }

        lines.join("\n")
    }

    /// Average tick time in milliseconds.
    pub fn avg_tick_ms(&self) -> f64 {
        if self.tick_history.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.tick_history.iter().sum();
        (sum as f64) / (self.tick_history.len() as f64 * 1000.0)
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
        tracker
            .section_times
            .insert("A".to_string(), ("Survival".to_string(), 1000, 3));
        tracker.begin_tick();
        assert!(tracker.section_times.is_empty());
    }
}
