//! Job churn diagnosis (M1.5 goal 8.2.1), measurement only: which need took
//! a worker off its job, at which job stage, and how high that need was.
//!
//! The decision seam counts every `CancelReason::Needs` release in
//! `colony.release_counts` by `(need, kind, stage, value bin)`
//! (`sim_core::colony::JobReleaseKey`, written by
//! `jobs::seam::settle_after_decision` with the decision's choice in hand).
//! This module turns the counts into the report's histograms. The goal's
//! five needs are hunger, thirst, sleep, social and other; forage and
//! firewood (the survival work of `ForageRule` and `FirewoodRule`) are kept
//! apart inside "other".
use serde_json::{json, Value};
use sim_core::colony::CancelReason;
use sim_engine::SimEngine;
use std::collections::BTreeMap;

/// The need name of a key's need code.
pub const NEEDS: [&str; 7] = [
    "hunger", "thirst", "sleep", "social", "forage", "firewood", "other",
];
/// Job kind names by code.
pub const KINDS: [&str; 4] = ["chop", "haul", "deliver", "build"];
/// Job stage names by code.
pub const STAGES: [&str; 3] = ["to_source", "working", "to_dest"];

fn name(list: &[&'static str], code: u8) -> &'static str {
    list.get(usize::from(code)).copied().unwrap_or("?")
}

/// The diagnosis of one run (see the module header).
pub fn report(engine: &SimEngine) -> Value {
    let counts = &engine.resources.colony.release_counts;
    let mut by_need: BTreeMap<&str, u64> = BTreeMap::new();
    let mut by_need_stage: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_need_kind: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_stage: BTreeMap<&str, u64> = BTreeMap::new();
    let mut value_bins: BTreeMap<String, u64> = BTreeMap::new();
    let mut total = 0u64;
    for (&(need, kind, stage, bin), &n) in counts {
        let need = name(&NEEDS, need);
        total += n;
        *by_need.entry(need).or_insert(0) += n;
        *by_stage.entry(name(&STAGES, stage)).or_insert(0) += n;
        *by_need_stage
            .entry(format!("{need}/{}", name(&STAGES, stage)))
            .or_insert(0) += n;
        *by_need_kind
            .entry(format!("{need}/{}", name(&KINDS, kind)))
            .or_insert(0) += n;
        if bin != 255 {
            *value_bins
                .entry(format!("{need}/{:03}", u32::from(bin) * 10))
                .or_insert(0) += n;
        }
    }
    let tally = engine
        .resources
        .colony
        .tallies
        .cancels_of(CancelReason::Needs);
    json!({
        "needs_tally": tally,
        "counted": total,
        "by_need": by_need,
        "by_stage": by_stage,
        "by_need_stage": by_need_stage,
        "by_need_kind": by_need_kind,
        "need_value_bins": value_bins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_groups_the_seam_counts() {
        let mut engine = crate::scenes::colony_scene(0);
        let counts = &mut engine.resources.colony.release_counts;
        counts.insert((0, 0, 1, 7), 3);
        counts.insert((1, 1, 2, 5), 2);
        counts.insert((4, 0, 0, 255), 4);
        let r = report(&engine);
        assert_eq!(r["counted"], json!(9));
        assert_eq!(r["by_need"]["hunger"], json!(3));
        assert_eq!(r["by_need_stage"]["thirst/to_dest"], json!(2));
        assert_eq!(r["by_need_kind"]["forage/chop"], json!(4));
        assert_eq!(r["need_value_bins"]["hunger/070"], json!(3));
        assert_eq!(r["by_stage"]["to_source"], json!(4));
    }
}
