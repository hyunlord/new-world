/// EngineSnapshot — read-only diagnostic view of engine state.
///
/// Captures the key counters and calendar position at a given tick.
/// Used for:
/// - Save file metadata (what tick was this saved at?)
/// - Diagnostics (GDScript overlay, debug HUD)
/// - FFI export to Godot (Phase R-3)
/// - Test assertions without needing engine internals
///
/// Snapshots are cheap to create (just copies of scalar values) and
/// fully serializable via serde.
use serde::{Deserialize, Serialize};

/// A lightweight, serializable read-only snapshot of `SimEngine` state.
///
/// Created via `SimEngine::snapshot()`. Does NOT borrow the engine —
/// all fields are owned copies of scalar values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    /// Absolute tick at snapshot time (0-indexed).
    pub tick: u64,
    /// In-game year at snapshot time (1-indexed).
    pub year: u32,
    /// Day of in-game year at snapshot time (1-365).
    pub day_of_year: u32,
    /// Number of live entities in the ECS world.
    pub entity_count: usize,
    /// Number of settlements in SimResources.
    pub settlement_count: usize,
    /// Number of registered simulation systems.
    pub system_count: usize,
    /// Cumulative events dispatched since engine creation.
    pub events_dispatched: u64,
}

impl EngineSnapshot {
    /// Human-readable date string: "Year N, Day D".
    pub fn date_string(&self) -> String {
        format!("Year {}, Day {}", self.year, self.day_of_year)
    }

    /// Total in-game ticks per year based on snapshot year/day.
    ///
    /// Uses the constant 4380 (12 ticks/day × 365 days/year).
    pub fn tick_of_year(&self) -> u64 {
        self.tick % 4380
    }
}

impl std::fmt::Display for EngineSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[tick={} {} entities={} settlements={} events={}]",
            self.tick,
            self.date_string(),
            self.entity_count,
            self.settlement_count,
            self.events_dispatched,
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snap(tick: u64, year: u32, day: u32) -> EngineSnapshot {
        EngineSnapshot {
            tick,
            year,
            day_of_year: day,
            entity_count: 10,
            settlement_count: 2,
            system_count: 5,
            events_dispatched: 100,
        }
    }

    #[test]
    fn date_string_format() {
        let snap = make_snap(100, 1, 9);
        assert_eq!(snap.date_string(), "Year 1, Day 9");
    }

    #[test]
    fn tick_of_year_wraps() {
        // 4380 ticks per year; tick 4380 = start of year 2, tick_of_year = 0
        let snap = make_snap(4380, 2, 1);
        assert_eq!(snap.tick_of_year(), 0);

        let snap2 = make_snap(4381, 2, 1);
        assert_eq!(snap2.tick_of_year(), 1);
    }

    #[test]
    fn display_format_contains_key_fields() {
        let snap = make_snap(42, 1, 4);
        let s = snap.to_string();
        assert!(s.contains("tick=42"));
        assert!(s.contains("entities=10"));
        assert!(s.contains("settlements=2"));
    }

    #[test]
    fn round_trips_serde() {
        let snap = make_snap(999, 3, 100);
        let json = serde_json::to_string(&snap).unwrap();
        let back: EngineSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tick, 999);
        assert_eq!(back.year, 3);
        assert_eq!(back.day_of_year, 100);
        assert_eq!(back.entity_count, 10);
    }
}
