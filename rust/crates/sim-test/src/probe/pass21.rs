//! Additive keys of the Pass 20 per-seed block (Pass 21, `game_v26`; present
//! only when the relations rule caps disputes, so `game_v25` summaries gain
//! nothing).
//!
//! `tallies`: the Pass 21 counters (disputes past the daily dispute cap,
//! disputes whose quarrel draw met the daily quarrel cap, quarrels between
//! estranged villages). `pairs`: every village pair that existed, as `[a, b,
//! days_estranged, [estrangement ticks], [reconciliation ticks]]` in key
//! order — days estranged counts the decision minutes at which the pair was
//! estranged after the day's drift (from the pair-day log), the ticks come
//! from the chronicle.
use std::collections::BTreeMap;

use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_engine::SimEngine;

/// One pair's estrangement history: days estranged, estrangement ticks,
/// reconciliation ticks.
type History = (u64, Vec<u64>, Vec<u64>);

/// The Pass 21 keys, `None` without a dispute cap.
pub(super) fn block(engine: &SimEngine) -> Option<Value> {
    let r = &engine.resources;
    let rule = r.rules.society.relations?;
    if rule.dispute_cap_per_pair_day == 0 {
        return None;
    }
    let t = &r.tallies;
    let mut history: BTreeMap<(u32, u32), History> = BTreeMap::new();
    for day in r.settlement_relations.days.iter().filter(|d| d.from < d.to) {
        let entry = history.entry((day.from, day.to)).or_default();
        entry.0 += u64::from(day.estranged);
    }
    for entry in r.chronicle.entries() {
        let (Some(a), Some(b)) = (entry.settlement, entry.settlement2) else {
            continue;
        };
        let row = match entry.kind {
            ChronicleKind::VillagesEstranged | ChronicleKind::VillagesReconciled => {
                history.entry((a.min(b), a.max(b))).or_default()
            }
            _ => continue,
        };
        if entry.kind == ChronicleKind::VillagesEstranged {
            row.1.push(entry.tick);
        } else {
            row.2.push(entry.tick);
        }
    }
    let pairs: Vec<Value> = history
        .into_iter()
        .map(|((a, b), (days, estranged, reconciled))| json!([a, b, days, estranged, reconciled]))
        .collect();
    Some(json!({
        "tallies": {
            "disputes_uncosted": t.disputes_uncosted,
            "quarrels_capped": t.quarrels_capped,
            "quarrels_while_estranged": t.quarrels_while_estranged,
        },
        "pairs": pairs,
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_capped_rules_have_the_keys() {
        let mut engine = crate::scenes::baseline_scene(3);
        let block = super::block(&engine).expect("baseline keys");
        assert_eq!(block["tallies"]["disputes_uncosted"], 0);
        assert!(block["pairs"].as_array().is_some_and(Vec::is_empty));
        // Relations without a dispute cap (the removed `game_v25`'s rule) and
        // no relations rule at all have no keys.
        let mut uncapped = engine.resources.rules.society.relations.expect("relations");
        uncapped.dispute_cap_per_pair_day = 0;
        engine.resources.rules.society.relations = Some(uncapped);
        assert!(super::block(&engine).is_none());
        engine.resources.rules.society.relations = None;
        assert!(super::block(&engine).is_none());
    }
}
