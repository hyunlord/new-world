//! Steward UI reads and mode command forwarding through the ordinary command log.

use godot::prelude::*;
use sim_core::colony::steward::{rules, StewardScope};
use sim_core::colony::{Command, CommandResult, RejectReason};
use sim_engine::{SimEngine, SimResources};

/// Persisted steward mode and successful action provenance, never inferred completion.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StewardView {
    /// Whether automatic planning is enabled.
    pub enabled: bool,
    /// All=0, housing=1, food=2.
    pub scope: u8,
    /// Successful automatic action sequence.
    pub sequence: u64,
    /// Most recent successful rule, absent before an action.
    pub last_rule: Option<u16>,
    /// Tick of that action.
    pub last_tick: u64,
    /// Localization key from the rule data, empty if unavailable.
    pub last_label: String,
    /// Distinct rules with retained accepted-order provenance, in id order.
    pub accepted_rules: Vec<u16>,
    /// Every configured rule id and localization key, in id order, for historical entries.
    pub labels: Vec<(u16, String)>,
}

/// Read persisted provenance without changing mode, cooldowns or command history.
pub fn collect_steward(res: &SimResources) -> StewardView {
    let state = &res.steward;
    let mut labels: Vec<_> = rules().map_or_else(
        |_| Vec::new(),
        |data| data.rules.iter().map(|r| (r.id, r.label.clone())).collect(),
    );
    labels.sort_by_key(|r| r.0);
    let last_label = rules()
        .ok()
        .and_then(|data| data.rules.iter().find(|r| Some(r.id) == state.last_rule))
        .map_or_else(String::new, |r| r.label.clone());
    let accepted_rules = state
        .owned
        .iter()
        .map(|r| r.rule)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    StewardView {
        enabled: state.enabled,
        scope: state.scope.code(),
        sequence: state.sequence,
        last_rule: state.last_rule,
        last_tick: state.last_tick,
        last_label,
        accepted_rules,
        labels,
    }
}

/// Parse the UI scope once and log a valid mode change through the ordinary command path.
pub fn submit_set_steward(engine: &mut SimEngine, on: bool, scope: i64) -> CommandResult {
    let Some(scope) = u8::try_from(scope).ok().and_then(StewardScope::from_code) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    sim_systems::commands::submit(engine, Command::SetSteward { on, scope })
}

pub(super) fn steward_to_dict(v: &StewardView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("enabled", v.enabled);
    d.set("scope", i64::from(v.scope));
    d.set("sequence", i64::try_from(v.sequence).unwrap_or(i64::MAX));
    d.set("last_rule", v.last_rule.map_or(-1, i64::from));
    d.set("last_tick", i64::try_from(v.last_tick).unwrap_or(i64::MAX));
    d.set("last_label", v.last_label.as_str());
    let rules: Vec<i32> = v.accepted_rules.iter().map(|r| i32::from(*r)).collect();
    d.set("accepted_rules", PackedInt32Array::from(rules.as_slice()));
    let mut labels = VarDictionary::new();
    for (id, label) in &v.labels {
        labels.set(i64::from(*id), label.as_str());
    }
    d.set("rules", labels);
    d
}
