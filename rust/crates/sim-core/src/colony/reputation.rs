//! Settlement reputation (M-GROW 3.6): a monthly value with its cause list.
//!
//! Data from `data/colony/reputation.ron`; the computation lives in sim-systems
//! (`runtime::reputation`), the state in [`super::ColonyState::reputation`] (saved by the format25 tail).

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const REPUTATION_RON: &str = include_str!("../../data/colony/reputation.ron");

/// The fact a cause reads (fixed set; weights are data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauseFact {
    /// Stored edible days against `food_good_days`.
    Food,
    /// Covered beds against people.
    Shelter,
    /// People bitten by wolves since the last period.
    Safety,
    /// Trades since the last period.
    Trade,
}

impl CauseFact {
    fn parse(id: &str) -> Option<Self> {
        Some(match id {
            "food" => Self::Food,
            "shelter" => Self::Shelter,
            "safety" => Self::Safety,
            "trade" => Self::Trade,
            _ => return None,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCause {
    id: String,
    weight: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    schema_version: u32,
    periods_per_season: u32,
    base: u8,
    causes: Vec<RawCause>,
    food_good_days: u32,
    bite_penalty_permille: u32,
    trade_bonus_permille: u32,
    roll_permille_per_point: u32,
    refuse_below: u8,
    card_below: u8,
    chronicle_step: u8,
}

/// One weighted cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cause {
    /// Data id (locale keys `REPUTATION_CAUSE_<ID>_UP/_DOWN`).
    pub id: String,
    /// What it reads.
    pub fact: CauseFact,
    /// Points at a full score (±1000 ‰).
    pub weight: u16,
}

/// Parsed reputation data (field docs in the data file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReputationData {
    /// See the data file.
    pub periods_per_season: u32,
    /// See the data file.
    pub base: u8,
    /// See the data file.
    pub causes: Vec<Cause>,
    /// See the data file.
    pub food_good_days: u32,
    /// See the data file.
    pub bite_penalty_permille: u32,
    /// See the data file.
    pub trade_bonus_permille: u32,
    /// See the data file.
    pub roll_permille_per_point: u32,
    /// See the data file.
    pub refuse_below: u8,
    /// See the data file.
    pub card_below: u8,
    /// See the data file.
    pub chronicle_step: u8,
}

/// Parse and validate a reputation file.
pub fn parse(text: &str) -> Result<ReputationData, String> {
    let raw: RawFile = ron::from_str(text).map_err(|e| e.to_string())?;
    if raw.schema_version != 1 {
        return Err(format!("reputation schema {}", raw.schema_version));
    }
    let mut causes = Vec::with_capacity(raw.causes.len());
    for c in raw.causes {
        let fact = CauseFact::parse(&c.id).ok_or_else(|| format!("unknown cause {}", c.id))?;
        if causes.iter().any(|known: &Cause| known.id == c.id) {
            return Err(format!("duplicate cause {}", c.id));
        }
        causes.push(Cause {
            id: c.id,
            fact,
            weight: c.weight,
        });
    }
    let total: u32 = causes.iter().map(|c| u32::from(c.weight)).sum();
    if u32::from(raw.base) < total || u32::from(raw.base) + total > 100 {
        return Err("base ± the cause weights must stay within 0..=100".into());
    }
    if raw.periods_per_season == 0 || raw.food_good_days == 0 {
        return Err("periods_per_season and food_good_days must be positive".into());
    }
    Ok(ReputationData {
        periods_per_season: raw.periods_per_season,
        base: raw.base,
        causes,
        food_good_days: raw.food_good_days,
        bite_penalty_permille: raw.bite_penalty_permille,
        trade_bonus_permille: raw.trade_bonus_permille,
        roll_permille_per_point: raw.roll_permille_per_point,
        refuse_below: raw.refuse_below,
        card_below: raw.card_below,
        chronicle_step: raw.chronicle_step,
    })
}

/// The bundled data, parsed once; `None` when the file is invalid (reputation stays off).
pub fn data() -> Option<&'static ReputationData> {
    static DATA: OnceLock<Option<ReputationData>> = OnceLock::new();
    DATA.get_or_init(|| parse(REPUTATION_RON).ok()).as_ref()
}

/// Saved reputation state (format25 tail).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationState {
    /// The current value, `None` before the first period ended.
    pub value: Option<u8>,
    /// Each cause's signed contribution, in data order (empty while `value` is `None`).
    pub contributions: Vec<i32>,
    /// The last period index computed.
    pub period: u32,
    /// Wildlife `bitten` tally at the last period (safety delta).
    pub last_bitten: u64,
    /// Neighbour `trades` tally at the last period (trade delta).
    pub last_trades: u64,
    /// Value of the last chronicle line.
    pub chronicled: Option<u8>,
    /// Migrant waves refused because the value was below `refuse_below`.
    pub refused: u32,
}

impl ReputationState {
    /// Load-boundary check: the value is exactly base + the contributions.
    pub fn validate(&self, data: &ReputationData) -> Result<(), String> {
        match self.value {
            None if self.contributions.is_empty() => Ok(()),
            Some(value) if self.contributions.len() == data.causes.len() => {
                let sum = i64::from(data.base)
                    + self
                        .contributions
                        .iter()
                        .map(|c| i64::from(*c))
                        .sum::<i64>();
                if sum == i64::from(value) {
                    Ok(())
                } else {
                    Err(format!("reputation {value} is not base + causes ({sum})"))
                }
            }
            _ => Err("reputation contributions do not match the causes".into()),
        }
    }

    /// Cause indices by descending |contribution| (ties: data order).
    pub fn ranked(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.contributions.len()).collect();
        order.sort_by_key(|&i| (std::cmp::Reverse(self.contributions[i].unsigned_abs()), i));
        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reputation_ron_parses_and_rejects_bad_files() {
        let d = data().expect("reputation.ron");
        assert_eq!(d.causes.len(), 4);
        assert!(parse(&REPUTATION_RON.replace("\"trade\"", "\"festival\"")).is_err());
        assert!(parse(&REPUTATION_RON.replace("base: 50", "base: 20")).is_err());
        assert!(parse(&REPUTATION_RON.replace("schema_version: 1", "schema_version: 2")).is_err());
    }
}
