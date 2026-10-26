//! Annual metrics: verified durable ancestry, edge depth, and distinct people.
use serde::Serialize;
use sim_core::chronicle::{CauseRef, Chronicle};
use std::collections::{BTreeMap, BTreeSet};

/// Counts within one half-open calendar year, without extrapolating partial years.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct YearYield {
    /// Zero-based calendar year.
    pub year: u64,
    /// Simulated ticks observed in this year.
    pub observed_ticks: u64,
    /// Whether the entire year was simulated.
    pub complete_year: bool,
    /// All entries, including those without a recorded cause.
    pub entries: usize,
    /// Entries with at least three verified edges and no detected cycle.
    pub deep_entries: usize,
    /// Unique observed cause-kind paths among deep entries; IDs are excluded.
    pub distinct_deep_chains: usize,
    /// Distinct people appearing in at least three entries during this year.
    pub people_with_three_entries: usize,
    /// Chains whose explicit cause points to an absent or unresolved record.
    pub missing_chains: usize,
    /// Chains revisiting an entry; excluded from depth counts.
    pub cyclic_chains: usize,
}

enum Chain {
    Complete { depth: usize, kinds: Vec<String> },
    Missing { depth: usize, kinds: Vec<String> },
    Cycle,
}

fn trace(chronicle: &Chronicle, start: u32) -> Chain {
    let mut cursor = start;
    let mut visited = BTreeSet::new();
    let mut kinds = Vec::new();
    let mut depth = 0;
    loop {
        if !visited.insert(cursor) {
            return Chain::Cycle;
        }
        let Some(entry) = chronicle.entry(cursor) else {
            return Chain::Missing { depth, kinds };
        };
        kinds.push(format!("entry:{}", entry.kind.as_str()));
        let (next, edge_kind) = match chronicle.cause_of(cursor) {
            None => return Chain::Complete { depth, kinds },
            Some(CauseRef::Entry(index)) => (Some(index), Some("entry_link".into())),
            Some(CauseRef::Pressure { kind, entry }) => (entry, Some(format!("pressure:{kind}"))),
            Some(CauseRef::Modifier { code, entry }) => (entry, Some(format!("modifier:{code}"))),
            Some(CauseRef::Sample { code, sample }) => {
                let Some(sample) = chronicle.sample(sample) else {
                    return Chain::Missing { depth, kinds };
                };
                kinds.push(format!("sample:{code}"));
                depth += 1;
                match sample.entry {
                    Some(index) => (Some(index), None),
                    None => return Chain::Complete { depth, kinds },
                }
            }
        };
        let Some(next) = next else {
            return Chain::Missing { depth, kinds };
        };
        if chronicle.entry(next).is_none() {
            return Chain::Missing { depth, kinds };
        }
        if visited.contains(&next) {
            return Chain::Cycle;
        }
        if let Some(kind) = edge_kind {
            kinds.push(kind);
        }
        depth += 1;
        cursor = next;
    }
}

/// Read durable causes without touching simulation state. Depth counts edges.
///
/// Windows are `[year * ticks_per_year, min(end_tick, (year+1)*ticks_per_year))`.
/// Depth counts verified record edges, not presentation-only label nodes.
/// Pressure/modifier references add one edge only when their entry exists;
/// samples add one edge when present, plus one if their linked entry exists.
/// Missing ancestry preserves the verified prefix; cycles exclude the path.
/// Signatures contain only observed kinds, without unresolved suffix labels or
/// termination markers. A root and an unresolved path with identical verified
/// kinds deduplicate; a longer observed path is distinct from its prefix.
/// An uncaused entry or a sample without an entry link is a root.
pub fn measure(
    chronicle: &Chronicle,
    end_tick: u64,
    ticks_per_year: u64,
) -> Result<Vec<YearYield>, String> {
    if ticks_per_year == 0 || end_tick == 0 {
        return Err("story-yield requires a positive calendar and observation window".into());
    }
    if chronicle
        .entries()
        .iter()
        .any(|entry| entry.tick >= end_tick)
    {
        return Err(
            "story-yield chronicle contains an entry outside the observation window".into(),
        );
    }
    let mut reports = Vec::new();
    for year in 0..end_tick.div_ceil(ticks_per_year) {
        let start = year * ticks_per_year;
        let observed_ticks = (end_tick - start).min(ticks_per_year);
        let mut report = YearYield {
            year,
            observed_ticks,
            complete_year: observed_ticks == ticks_per_year,
            entries: 0,
            deep_entries: 0,
            distinct_deep_chains: 0,
            people_with_three_entries: 0,
            missing_chains: 0,
            cyclic_chains: 0,
        };
        let mut chains = BTreeSet::new();
        let mut people = BTreeMap::new();
        for (index, entry) in chronicle.entries().iter().enumerate() {
            if entry.tick < start || entry.tick >= start + observed_ticks {
                continue;
            }
            report.entries += 1;
            for agent in entry.agents() {
                *people.entry(agent).or_insert(0usize) += 1;
            }
            let index = u32::try_from(index)
                .map_err(|_| "chronicle index exceeds cause reference range")?;
            let (depth, kinds) = match trace(chronicle, index) {
                Chain::Complete { depth, kinds } => (depth, kinds),
                Chain::Missing { depth, kinds } => {
                    report.missing_chains += 1;
                    (depth, kinds)
                }
                Chain::Cycle => {
                    report.cyclic_chains += 1;
                    continue;
                }
            };
            if depth >= 3 {
                report.deep_entries += 1;
                chains.insert(kinds);
            }
        }
        report.distinct_deep_chains = chains.len();
        report.people_with_three_entries = people.values().filter(|count| **count >= 3).count();
        reports.push(report);
    }
    Ok(reports)
}
