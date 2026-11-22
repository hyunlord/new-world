//! SIMFIX1 (SIMAUDIT1 N2 tickets SIMFIX1, SIMFIX2, WORK1 and the starving search of PURPOSE1): the day-life
//! tunables, read from `data/colony/daylife.ron` (owner directive 2026-10-24: tunables live in data).
//!
//! The file is embedded and parsed once. It is not a `WorldRules` field, so it is not part of a saved game and the
//! save format is unchanged; colony systems read it through [`DaylifeRule::get`] and keep their pre-SIMFIX1
//! behaviour when the file is malformed (`None`). Baseline worlds never reach the readers (they gate on
//! `rules.colony`).
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// When the job layer may admit a new job (SIMFIX1 F4, WORK1 provisioning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobAdmission {
    /// A job's estimate must end this many minutes before the person's bedtime.
    pub bedtime_slack_minutes: u32,
    /// Ticks of work added to a job's walk legs in its estimate (all kinds but hunts).
    pub work_estimate_ticks: u32,
    /// A job whose estimate exceeds this many ticks is admitted only when hunger and thirst stay below their seek
    /// thresholds to its end (a hungry or thirsty person eats or drinks first).
    pub provision_over_ticks: u32,
}

/// Hunts: give-up rules (WORK1 F5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HuntRule {
    /// A hunt that has not closed in on its prey for this many ticks ends (reason `Stuck`).
    pub no_gain_ticks: u64,
    /// A hunt ends (reason `Stuck`) after this many ticks; also a hunt's duration estimate at admission.
    pub max_ticks: u64,
    /// After giving up, the hunter is not offered another hunt for this many ticks.
    pub retry_after_ticks: u64,
}

/// Talk (SIMFIX2 F1): invitations, a paced talk and giving up a chase nobody takes up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SocialRule {
    /// Loneliness a whole talk relieves for each partner (the former one-shot `SOCIAL_CONSUME_AMOUNT`).
    pub relief_total: u32,
    /// Progress ticks a talk lasts; the relief is paid out evenly over them (0 keeps the 3-tick talk).
    pub talk_ticks: u32,
    /// A chase whose partner has not walked or talked back after this many ticks ends with the retry cooldown.
    pub stall_ticks: u64,
    /// Free people look for a talk partner within their block of `peer_cell` × `peer_cell` tiles (was 1: the same tile).
    pub peer_cell: u32,
}

/// Steward pacing (D-SIMFIX1-22): the planning pass is spread over ticks so no frame carries it whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StewardPacing {
    /// The pass's rules run in this many contiguous slices (data order), one slice per tick from the
    /// period's first tick; 1 keeps the whole pass on one tick.
    pub pass_slices: u32,
}

/// The day-life tunables (`data/colony/daylife.ron`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaylifeRule {
    /// Job admission horizon.
    pub job_admission: JobAdmission,
    /// Hunts.
    pub hunt: HuntRule,
    /// Talk.
    pub social: SocialRule,
    /// Steward pacing.
    pub steward: StewardPacing,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DataFile {
    schema_version: u32,
    rule: DaylifeRule,
}

const DATA_RON: &str = include_str!("../../data/colony/daylife.ron");

impl DaylifeRule {
    /// The embedded day-life rule; `None` when the file is malformed (the features stay off).
    pub fn get() -> Option<&'static Self> {
        static DATA: OnceLock<Option<DaylifeRule>> = OnceLock::new();
        DATA.get_or_init(|| parse(DATA_RON)).as_ref()
    }
}

/// Parse a `daylife.ron`-shaped file; `None` when malformed, of another schema or naming an unknown field.
pub fn parse(text: &str) -> Option<DaylifeRule> {
    let f = ron::from_str::<DataFile>(text).ok()?;
    (f.schema_version == 1).then_some(f.rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_data_file_parses_strictly() {
        let r = DaylifeRule::get().expect("daylife.ron parses");
        assert_eq!(r.job_admission.bedtime_slack_minutes, 20);
        assert_eq!(r.hunt.max_ticks, 360);
        let typo = DATA_RON.replacen("bedtime_slack_minutes:", "bedtime_slack_minutesx:", 1);
        assert!(parse(&typo).is_none(), "a misspelt field is refused");
        let other = DATA_RON.replace("schema_version: 1", "schema_version: 99");
        assert!(parse(&other).is_none(), "another schema is refused");
    }
}
