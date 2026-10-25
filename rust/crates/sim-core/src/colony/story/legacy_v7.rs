//! Frozen pre-format9 storyteller state; preserve schedule and touched history.
use super::*;

/// Historical storyteller wire layout.
#[derive(serde::Deserialize, Default)]
pub struct LegacyStoryteller {
    /// Pressures planned for this season and not begun, in day order.
    pub planned: Vec<Pressure>,
    /// Pressures in force, in begin order.
    pub active: Vec<Pressure>,
    /// Every pressure that ended, in end order.
    pub history: Vec<Pressure>,
    /// The day the last major began (the 6-day gap).
    pub last_major_day: Option<u32>,
    /// The first display day the storyteller has not run yet.
    pub next_check_day: u32,
    /// Seasons planned so far (the next season index to plan).
    pub seasons_planned: u32,
    /// Majors a season's draw chose but mercy or the gap removed (measurement).
    pub skipped: u32,
    /// Bumped when a pressure begins or ends (renderers poll it).
    pub version: u64,
    /// `(pressure kind code, person, effect)` already given a `PressureCause`
    /// event while that pressure is active (cleared when it ends).
    pub touched: std::collections::BTreeSet<(u8, crate::components::AgentId, u8)>,
}
impl From<LegacyStoryteller> for Storyteller {
    fn from(old: LegacyStoryteller) -> Self {
        Self {
            planned: old.planned,
            active: old.active,
            history: old.history,
            last_major_day: old.last_major_day,
            next_check_day: old.next_check_day,
            seasons_planned: old.seasons_planned,
            skipped: old.skipped,
            version: old.version,
            touched: old
                .touched
                .into_iter()
                .map(|(kind, agent, effect)| (u32::from(kind), agent, effect))
                .collect(),
        }
    }
}
