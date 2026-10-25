//! Persisted steward mode, provenance and cancellation vetoes (save-body extension v7).
use crate::colony::Command;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Which automatic action families the player permits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StewardScope {
    /// All basic colony orders.
    #[default]
    All,
    /// Shelter and its material supply only.
    Housing,
    /// Food and hearth supply only.
    Food,
}
impl StewardScope {
    /// Stable bridge code.
    pub const fn code(self) -> u8 {
        match self {
            Self::All => 0,
            Self::Housing => 1,
            Self::Food => 2,
        }
    }
    /// Parse a bridge code without silently substituting a different scope.
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::All),
            1 => Some(Self::Housing),
            2 => Some(Self::Food),
            _ => None,
        }
    }
    /// Whether an action family is allowed; shared actions use All.
    pub const fn permits(self, family: Self) -> bool {
        matches!(self, Self::All) || matches!(family, Self::All) || self.code() == family.code()
    }
}

/// A target retained after cancellation so an automatic order cannot immediately return.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    /// A tile occupied or designated by an order.
    Tile(u32, u32),
    /// A workshop recipe (survives the removal of its bill id).
    Bill(
        u32,
        #[serde(deserialize_with = "crate::colony::content_state::deserialize_recipe_code")] u32,
    ),
    /// An apprenticeship pair and knowledge node.
    Teach(u64, u8),
}
/// Provenance for an accepted automatic ordinary command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedOrder {
    /// Rule id.
    pub rule: u16,
    /// Its ordinary command, without the source envelope.
    pub command: Command,
    /// Exact affected targets at issue time.
    pub targets: Vec<Target>,
}
/// All gameplay steward state. OFF is the default for probes and migrated saves.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StewardState {
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
    /// Independent defence policy and per-person ownership.
    pub defence: DefenceState,
}
impl StewardState {
    /// Whether a target is still protected by the player's cancellation.
    pub fn vetoed(&self, target: &Target, tick: u64) -> bool {
        self.vetoes.get(target).is_some_and(|until| tick < *until)
    }
}

/// Saved defence ownership; manual orders override automation for one day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefenceState {
    /// Enabled by default; the overall steward switch still gates automation.
    pub enabled: bool,
    /// Only these people may be automatically released or redirected.
    pub owned_ids: BTreeSet<u64>,
    /// Explicit player orders remain protected until this exclusive tick.
    pub manual_until: BTreeMap<u64, u64>,
    /// First tick without a nearby threat after deployment.
    pub quiet_since: Option<u64>,
    /// Pack identity whose quiet timeout has already released defenders.
    pub retired_pack: Option<u32>,
    /// Pack for which the current approach response already recruited once.
    pub responded_pack: Option<u32>,
    /// Actual count in the most recent recruitment batch.
    pub last_count: u16,
}
impl Default for DefenceState {
    fn default() -> Self {
        Self {
            enabled: true,
            owned_ids: BTreeSet::new(),
            manual_until: BTreeMap::new(),
            quiet_since: None,
            retired_pack: None,
            responded_pack: None,
            last_count: 0,
        }
    }
}
