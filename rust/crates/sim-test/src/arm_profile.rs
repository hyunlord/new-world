//! Feature-only external arm measurements; never part of simulation summaries or saves.

use std::collections::BTreeMap;

use serde::Serialize;
use sim_systems::perf_arms::{self, ScopeTotal};

/// A complete named-scope ledger for one primary world, excluding determinism twins.
#[derive(Debug, Default)]
pub struct ArmProfile {
    ticks: u64,
    totals: BTreeMap<&'static str, ScopeTotal>,
}

#[derive(Serialize)]
struct Row {
    name: &'static str,
    calls: u64,
    inclusive_ns: u64,
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    ticks: u64,
    timing: &'static str,
    scopes: Vec<Row>,
}

impl ArmProfile {
    /// Reset thread-local instrumentation before starting an independent world.
    pub fn start() -> Self {
        Self::discard();
        Self::default()
    }

    /// Discard setup or twin-world instrumentation without adding it to the ledger.
    pub fn discard() {
        let _ = perf_arms::take();
    }

    /// Drain exactly one primary-world tick.
    pub fn after_tick(&mut self) {
        self.add(perf_arms::take());
        self.ticks += 1;
    }

    fn add(&mut self, values: BTreeMap<&'static str, ScopeTotal>) {
        for (name, value) in values {
            let entry = self.totals.entry(name).or_default();
            entry.calls = entry.calls.saturating_add(value.calls);
            entry.ns = entry.ns.saturating_add(value.ns);
        }
    }

    /// Serialize all registered scopes, including unhit scopes, as a separate artifact.
    pub fn report(&self) -> Result<serde_json::Value, serde_json::Error> {
        let mut totals = self.totals.clone();
        for name in perf_arms::ALL_ARMS {
            totals.entry(name).or_default();
        }
        serde_json::to_value(Report {
            schema_version: 1,
            ticks: self.ticks,
            timing: "inclusive scopes overlap; do not sum nested timings",
            scopes: totals
                .into_iter()
                .map(|(name, total)| Row {
                    name,
                    calls: total.calls,
                    inclusive_ns: total.ns,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_twin_and_previous_world_scopes_from_primary_counts() {
        // Given stale setup observations before a fresh primary world.
        drop(perf_arms::Scope::start("fixture.setup"));
        let mut profile = ArmProfile::start();
        // When one primary tick is collected and the following twin is discarded.
        drop(perf_arms::Scope::start("fixture.primary"));
        profile.after_tick();
        drop(perf_arms::Scope::start("fixture.twin"));
        ArmProfile::discard();
        // Then only the primary tick appears, with no state carried to the next world.
        assert_eq!(profile.ticks, 1);
        assert_eq!(profile.totals["fixture.primary"].calls, 1);
        assert!(!profile.totals.contains_key("fixture.setup"));
        assert!(!profile.totals.contains_key("fixture.twin"));
        assert!(ArmProfile::start().totals.is_empty());
    }

    #[test]
    fn merges_counts_and_preserves_explicit_zero_scopes() {
        let mut profile = ArmProfile::default();
        profile.add(BTreeMap::from([(
            "fixture",
            ScopeTotal { ns: 7, calls: 2 },
        )]));
        profile.add(BTreeMap::from([(
            "fixture",
            ScopeTotal { ns: 11, calls: 3 },
        )]));
        let value = profile.report().expect("report");
        let scopes = value["scopes"].as_array().expect("scopes");
        let fixture = scopes
            .iter()
            .find(|r| r["name"] == "fixture")
            .expect("fixture");
        assert_eq!(fixture["calls"], 5);
        assert_eq!(fixture["inclusive_ns"], 18);
        for name in perf_arms::ALL_ARMS {
            let row = scopes
                .iter()
                .find(|r| r["name"] == *name)
                .expect("registered scope");
            assert_eq!(row["calls"], 0);
        }
        assert_eq!(value["ticks"], 0);
    }
}
