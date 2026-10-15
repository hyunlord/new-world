//! Opt-in rule shapes for Pass 17; older scenes keep both policies absent.
use serde::{Deserialize, Serialize};

/// Daily activities that do not consume or produce economy resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineRule {
    /// Earliest display-clock minute, before the lineage-key stagger.
    pub day_start_minute: u32,
    /// Exclusive display-clock end of activities.
    pub day_end_minute: u32,
    /// Duration for persistence 500, counted from arrival.
    pub base_duration_minutes: u32,
    /// Duration adjustment at persistence 0 or 1000.
    pub duration_span_minutes: u32,
    /// Minimum duration after adjustment.
    pub min_duration_minutes: u32,
    /// Maximum time walking toward the chosen place.
    pub walk_timeout_minutes: u32,
    /// Inclusive lower bound of the pause after an activity.
    pub rest_minutes_min: u32,
    /// Inclusive upper bound of the pause after an activity.
    pub rest_minutes_max: u32,
    /// TendFire, Craft, Repair, HearthCircle and Explore base weights.
    pub adult_weights: [u16; 5],
    /// HearthCircle and SitByFire base weights for elders.
    pub elder_weights: [u16; 2],
}

/// Familiarity threshold and daily clock time for stable adult pairing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyRule {
    /// Minimum familiarity multiplied by one thousand.
    pub min_familiarity_permille: u16,
    /// Display-clock minute for pairing and clearing deceased partners.
    pub pair_minute: u32,
}
