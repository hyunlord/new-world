//! Frozen format7–10 steward wire layout; never append current state fields here.
use serde::{Deserialize, Serialize};
use sim_core::colony::steward::{OwnedOrder, StewardScope, StewardState, Target};
use sim_core::colony::Command;
use std::collections::BTreeMap;
/// All gameplay steward state. OFF is the default for probes and migrated saves.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyStewardState {
    /// Player's toggle.
    pub enabled: bool,
    /// Allowed action families.
    pub scope: StewardScope,
    /// Earliest tick each rule may fire again.
    pub cooldowns: BTreeMap<u16, u64>,
    /// Targets the player canceled, with absolute retry tick.
    pub vetoes: BTreeMap<Target, u64>,
    /// Accepted auto orders, for cancellation attribution.
    pub owned: Vec<OwnedOrder>,
    /// Monotonic successful action sequence, for renderer toasts.
    pub sequence: u64,
    /// Last successful rule.
    pub last_rule: Option<u16>,
    /// Last successful action tick.
    pub last_tick: u64,
    /// Recorded playback suppresses planning; source actions are authoritative.
    pub replay: bool,
    /// Source commands waiting for the after-assignment phase, in log order.
    pub pending: Vec<(u64, Command)>,
}
impl From<LegacyStewardState> for StewardState {
    fn from(old: LegacyStewardState) -> Self {
        Self {
            enabled: old.enabled,
            scope: old.scope,
            cooldowns: old.cooldowns,
            vetoes: old.vetoes,
            owned: old.owned,
            sequence: old.sequence,
            last_rule: old.last_rule,
            last_tick: old.last_tick,
            replay: old.replay,
            pending: old.pending,
            defence: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_layout_ends_after_pending_and_defaults_independent_defence() {
        let old = LegacyStewardState {
            enabled: true,
            sequence: 12,
            last_rule: Some(4),
            last_tick: 360,
            pending: vec![(
                361,
                Command::SetDrafted {
                    agent: 7,
                    drafted: true,
                },
            )],
            ..Default::default()
        };
        // A tuple fixes the actual old field sequence independently of the struct.
        let wire = super::super::encode(&(
            old.enabled,
            old.scope,
            &old.cooldowns,
            &old.vetoes,
            &old.owned,
            old.sequence,
            old.last_rule,
            old.last_tick,
            old.replay,
            &old.pending,
        ))
        .expect("legacy tuple");
        assert_eq!(wire, super::super::encode(&old).expect("legacy struct"));
        let migrated: StewardState = super::super::decode::<LegacyStewardState>(&wire)
            .expect("old wire")
            .into();
        assert!(migrated.enabled && migrated.defence.enabled);
        assert!(migrated.defence.owned_ids.is_empty());
        assert!(migrated.defence.manual_until.is_empty());
        assert_eq!(
            (migrated.sequence, migrated.last_rule, migrated.last_tick),
            (12, Some(4), 360)
        );
        assert_eq!(migrated.pending, old.pending);
        assert!(
            super::super::decode::<StewardState>(&wire).is_err(),
            "new fields cannot be read from an old body without migration"
        );
    }
}
