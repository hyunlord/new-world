//! M5 (ADR 0027): the storyteller's state — planned, active and past pressures. Logic lives in
//! `sim_systems::runtime::story`.
use serde::{Deserialize, Serialize};

/// Frozen historical storyteller decoding.
pub mod legacy_v7;

/// Installed pressure identity.
pub use super::content::ids::PressureKind;

impl PressureKind {
    /// Historical bundled identities, retained for compatibility fixtures only.
    pub const ALL: [Self; 4] = [Self::ColdSnap, Self::Drought, Self::Beasts, Self::Illness];

    /// Resolve an explicit registered code without narrowing its width.
    pub fn from_code(code: u32) -> Option<Self> {
        super::content::installed_or_bundled()
            .pressure_ids()
            .find(|kind| kind.code() == code)
    }

    /// Every installed pressure in stable code order.
    pub fn all() -> impl ExactSizeIterator<Item = Self> {
        super::content::installed_or_bundled().pressure_ids()
    }

    /// The installed definition for this validated identity.
    pub fn definition(self) -> &'static super::content::PressureDef {
        super::content::installed_or_bundled()
            .pressure(self)
            .unwrap_or_else(|| {
                panic!(
                    "installed pressure definition missing for code {}",
                    self.code()
                )
            })
    }

    /// Resolve a required validated role.
    pub fn role(role: &str) -> Self {
        super::content::installed_or_bundled()
            .pressure_role(role)
            .unwrap_or_else(|| panic!("required installed pressure role missing: {role}"))
    }

    /// Canonical name used by logs and presentation.
    pub fn name(self) -> &'static str {
        &self.definition().identity.id
    }

    /// Resolve a registered name or alias.
    pub fn from_name(name: &str) -> Option<Self> {
        super::content::installed_or_bundled().pressure_by_name(name)
    }

    /// Whether this definition participates in major pressure spacing.
    pub fn is_major(self) -> bool {
        self.definition().major
    }
}

/// What a pressure cost, counted while it was active.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PressureCost {
    /// Deaths of any cause while it was active.
    pub deaths: u32,
    /// Items it took: the extra firewood a cold snap burned, the forage a drought did not regrow.
    pub items_lost: u32,
    /// Person-ticks of work lost: sick or drafted people while it was active.
    pub work_ticks_lost: u64,
}

impl PressureCost {
    /// Work lost in whole person-days (`ticks_per_day` ticks a day; rounded down).
    pub const fn work_days_lost(&self, ticks_per_day: u64) -> u64 {
        if ticks_per_day == 0 {
            0
        } else {
            self.work_ticks_lost / ticks_per_day
        }
    }
}

/// One pressure, planned, active or past.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pressure {
    /// What it is.
    pub kind: PressureKind,
    /// The display day it begins (at the storyteller's daily run).
    pub start_day: u32,
    /// The display day it ends (exclusive; beasts end earlier when the pack is gone).
    pub end_day: u32,
    /// The chronicle index of its `PressureBegan` entry, once begun.
    pub began_entry: Option<u32>,
    /// The deaths counted when it began (the cost's base).
    pub deaths_at_start: u64,
    /// Its intensity: wolves in the pack, extra days of a snap or drought (0 otherwise).
    pub intensity: u32,
    /// The person it struck (illness: the first sick person).
    pub subject: Option<crate::components::AgentId>,
    /// What it cost so far.
    pub cost: PressureCost,
}

/// The storyteller (`ColonyState::story`; ADR 0027).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Storyteller {
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
    pub touched: std::collections::BTreeSet<(u32, crate::components::AgentId, u8)>,
}

impl Storyteller {
    /// The active pressure of `kind`, if any.
    pub fn active_of(&self, kind: PressureKind) -> Option<&Pressure> {
        self.active.iter().find(|p| p.kind == kind)
    }

    /// Whether a pressure of `kind` is in force.
    pub fn is_active(&self, kind: PressureKind) -> bool {
        self.active_of(kind).is_some()
    }

    /// The last day a pressure of `kind` began (active or past), if any.
    pub fn last_began(&self, kind: PressureKind) -> Option<u32> {
        self.history
            .iter()
            .chain(self.active.iter())
            .filter(|p| p.kind == kind)
            .map(|p| p.start_day)
            .max()
    }

    /// Every begun pressure (past then active), in begin order of each list.
    pub fn begun(&self) -> impl Iterator<Item = &Pressure> + '_ {
        self.history.iter().chain(self.active.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_and_round_trip() {
        let codes: Vec<u32> = PressureKind::ALL.iter().map(|k| k.code()).collect();
        assert_eq!(codes, vec![0, 1, 2, 3]);
        for kind in PressureKind::ALL {
            assert_eq!(PressureKind::from_code(kind.code()), Some(kind));
        }
        assert!(!PressureKind::Illness.is_major());
        assert_eq!(
            PressureCost {
                work_ticks_lost: 2_879,
                ..PressureCost::default()
            }
            .work_days_lost(1_440),
            1
        );
    }
}
