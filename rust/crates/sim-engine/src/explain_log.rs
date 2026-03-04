//! Per-entity ring-buffer of recent simulation explanation log entries.
//!
//! ExplainLog gives the player a window into *why* an agent did something:
//! which system acted on them, what happened, and how important it was.
//!
//! This module is a stub — no systems write to it yet. The ring-buffer
//! infrastructure is in place for future per-system integration.

use std::collections::HashMap;
use std::collections::VecDeque;

// ── ExplainEntry ──────────────────────────────────────────────────────────────

/// A single explanation log entry produced by a simulation system.
#[derive(Debug, Clone)]
pub struct ExplainEntry {
    /// Simulation tick when this event occurred.
    pub tick: u64,
    /// Name of the system that generated this entry (e.g. "StressSystem").
    pub system_name: &'static str,
    /// Category label for the event type (e.g. "stress_spike", "need_critical").
    pub event_type: &'static str,
    /// Human-readable description for the UI explain panel.
    pub summary: String,
    /// Importance level: 0=trivial, 1=normal, 2=important, 3=critical.
    pub severity: u8,
}

// ── ExplainLog ────────────────────────────────────────────────────────────────

/// Per-entity ring buffer of recent explain log entries.
///
/// Each entity has at most `max_per_entity` entries (default: 20).
/// When the buffer is full, the oldest entry is discarded.
pub struct ExplainLog {
    entries: HashMap<u64, VecDeque<ExplainEntry>>,
    max_per_entity: usize,
}

impl Default for ExplainLog {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplainLog {
    /// Create a new ExplainLog with a 20-entry per-entity ring buffer.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_per_entity: 20,
        }
    }

    /// Push an entry for `entity_bits`. Drops the oldest if the buffer is full.
    pub fn push(&mut self, entity_bits: u64, entry: ExplainEntry) {
        let deque = self
            .entries
            .entry(entity_bits)
            .or_insert_with(|| VecDeque::with_capacity(self.max_per_entity));
        if deque.len() >= self.max_per_entity {
            deque.pop_front();
        }
        deque.push_back(entry);
    }

    /// Get the most recent `count` entries for `entity_bits`, newest first.
    pub fn get_recent(&self, entity_bits: u64, count: usize) -> Vec<&ExplainEntry> {
        self.entries
            .get(&entity_bits)
            .map(|d| d.iter().rev().take(count).collect())
            .unwrap_or_default()
    }

    /// Returns the number of entities that have at least one log entry.
    pub fn entity_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the total number of log entries across all entities.
    pub fn total_entries(&self) -> usize {
        self.entries.values().map(|d| d.len()).sum()
    }

    /// Clear all entries for a specific entity (called on entity death).
    pub fn clear_entity(&mut self, entity_bits: u64) {
        self.entries.remove(&entity_bits);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(tick: u64) -> ExplainEntry {
        ExplainEntry {
            tick,
            system_name: "TestSystem",
            event_type: "test_event",
            summary: format!("Test entry at tick {}", tick),
            severity: 1,
        }
    }

    #[test]
    fn ring_buffer_retains_last_20() {
        let mut log = ExplainLog::new();
        let entity_bits = 42u64;

        // Push 25 entries
        for i in 0..25u64 {
            log.push(entity_bits, make_entry(i));
        }

        // Should only retain last 20
        let recent = log.get_recent(entity_bits, 100);
        assert_eq!(recent.len(), 20, "Expected 20 entries, got {}", recent.len());

        // Most recent should be tick 24 (newest first)
        assert_eq!(recent[0].tick, 24, "First entry should be tick 24");
        // Oldest retained should be tick 5
        assert_eq!(recent[19].tick, 5, "Last entry should be tick 5");
    }

    #[test]
    fn get_recent_respects_count_limit() {
        let mut log = ExplainLog::new();
        let entity_bits = 1u64;

        for i in 0..10u64 {
            log.push(entity_bits, make_entry(i));
        }

        let recent = log.get_recent(entity_bits, 3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].tick, 9);
        assert_eq!(recent[2].tick, 7);
    }

    #[test]
    fn empty_entity_returns_empty_vec() {
        let log = ExplainLog::new();
        let result = log.get_recent(999, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn total_entries_counts_correctly() {
        let mut log = ExplainLog::new();
        log.push(1, make_entry(0));
        log.push(1, make_entry(1));
        log.push(2, make_entry(0));
        assert_eq!(log.total_entries(), 3);
        assert_eq!(log.entity_count(), 2);
    }

    #[test]
    fn clear_entity_removes_entries() {
        let mut log = ExplainLog::new();
        log.push(1, make_entry(0));
        log.push(1, make_entry(1));
        log.clear_entity(1);
        assert_eq!(log.total_entries(), 0);
        assert!(log.get_recent(1, 10).is_empty());
    }
}
