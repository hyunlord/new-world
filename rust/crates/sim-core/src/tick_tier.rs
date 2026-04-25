//! System frequency tier classification for runtime scheduling.
//!
//! Provides Hot/Warm/Cold categorization for runtime systems based on
//! `tick_interval`. Metadata-only — execution logic unchanged.
//!
//! Boundaries (chosen empirically from the current 62-system distribution):
//! - Hot:  tick_interval ≤ 2  (≥30 Hz at 60 FPS — every-frame or near-every-frame)
//! - Warm: 3..=30              (~2~30 Hz — periodic background work)
//! - Cold: ≥ 31                (<2 Hz — rare, annual, or maintenance)

use serde::{Deserialize, Serialize};

/// Tier classification for a runtime system based on its tick interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TickTier {
    /// Runs every tick or every other tick (≥30 Hz at 60 FPS).
    Hot,
    /// Runs periodically (~2~30 Hz).
    Warm,
    /// Runs rarely (annual / maintenance / once-per-day, <2 Hz).
    Cold,
}

impl TickTier {
    /// Auto-classify a tier from a tick interval (in ticks).
    ///
    /// Boundary semantics:
    /// - 1, 2          → Hot
    /// - 3..=30        → Warm
    /// - 31, 365, …    → Cold
    #[inline]
    pub fn from_interval(tick_interval: i32) -> TickTier {
        match tick_interval {
            i if i <= 2 => TickTier::Hot,
            i if i <= 30 => TickTier::Warm,
            _ => TickTier::Cold,
        }
    }

    /// Stable lowercase identifier for serialization / FFI / logs.
    pub fn name(&self) -> &'static str {
        match self {
            TickTier::Hot => "hot",
            TickTier::Warm => "warm",
            TickTier::Cold => "cold",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_interval_classifies_correctly() {
        assert_eq!(TickTier::from_interval(1), TickTier::Hot);
        assert_eq!(TickTier::from_interval(2), TickTier::Hot);
        assert_eq!(TickTier::from_interval(3), TickTier::Warm);
        assert_eq!(TickTier::from_interval(10), TickTier::Warm);
        assert_eq!(TickTier::from_interval(30), TickTier::Warm);
        assert_eq!(TickTier::from_interval(31), TickTier::Cold);
        assert_eq!(TickTier::from_interval(365), TickTier::Cold);
    }

    #[test]
    fn name_is_lowercase_stable() {
        assert_eq!(TickTier::Hot.name(), "hot");
        assert_eq!(TickTier::Warm.name(), "warm");
        assert_eq!(TickTier::Cold.name(), "cold");
    }
}
