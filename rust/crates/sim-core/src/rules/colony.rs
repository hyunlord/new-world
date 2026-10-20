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
    /// The M1 colony world's rules for world `seed` (goal 8.1, ADR 0014):
    /// [`super::WorldRules::baseline`] plus the colony rule, with automatic
    /// shelter planning off (the player builds), fission off (one colony
    /// until neighbours, ADR 0006) and a step every 3 ticks instead of 8
    /// (8 is unusable on a 192-tile map: 40 tiles would take 5 sim hours).
    pub const fn colony(seed: u64) -> Self {
        let base = Self::baseline(seed);
        Self {
            agent: super::AgentRules {
                move_interval_ticks: 3,
                ..base.agent
            },
            society: super::SocietyRules {
                shelter_planning: None,
                fission: None,
                ..base.society
            },
            colony: Some(ColonyRule::M1),
            ..base
        }
    }
}
