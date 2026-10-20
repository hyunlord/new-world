//! Pass 18 (feature `subprofile` only): per-tick sub-system scopes collected
//! from `sim_systems::subprofile` and summarised into `worldsim_perf`'s JSON
//! as a `subprofile` block. Never compiled into the default build.
use serde_json::{json, Map, Value};
use sim_systems::subprofile::{take, ScopeTotal};
use std::collections::{BTreeMap, BTreeSet};

use crate::perf::{display_clock, SLOWEST_KEPT, WARMUP_TICKS};

/// Every tick's scope totals, in tick order.
#[derive(Debug, Default)]
pub struct SubprofileLog {
    ticks: Vec<(u64, u32, BTreeMap<&'static str, ScopeTotal>)>,
}

impl SubprofileLog {
    /// Drop anything accumulated before the run (world generation).
    pub fn new() -> Self {
        let _ = take();
        Self::default()
    }

    /// Collect the totals of tick `tick`, whose wall time was `us`.
    pub fn record(&mut self, tick: u64, us: u32) {
        self.ticks.push((tick, us, take()));
    }

    /// The `subprofile` block: steady-state totals per scope, the slowest
    /// ticks' breakdown, and every tick's per-scope nanoseconds.
    pub fn summary(&self, offset: u64, ticks_per_day: u64) -> Value {
        let steady: Vec<_> = self
            .ticks
            .iter()
            .filter(|(tick, ..)| *tick >= WARMUP_TICKS)
            .collect();
        let names: BTreeSet<&'static str> = self
            .ticks
            .iter()
            .flat_map(|(.., totals)| totals.keys().copied())
            .collect();
        let mut scopes = Map::new();
        for name in &names {
            let rows: Vec<ScopeTotal> = steady
                .iter()
                .map(|(.., totals)| totals.get(name).copied().unwrap_or_default())
                .collect();
            let ns: u64 = rows.iter().map(|row| row.ns).sum();
            let calls: u64 = rows.iter().map(|row| row.calls).sum();
            let max_ns = rows.iter().map(|row| row.ns).max().unwrap_or(0);
            let max_calls = rows.iter().map(|row| row.calls).max().unwrap_or(0);
            let n = steady.len().max(1) as f64;
            scopes.insert(
                (*name).to_owned(),
                json!({"total_ms": ns as f64 / 1e6, "calls": calls,
                       "mean_us_per_tick": ns as f64 / n / 1000.0,
                       "mean_calls_per_tick": calls as f64 / n,
                       "mean_us_per_call": if calls == 0 { 0.0 } else { ns as f64 / calls as f64 / 1000.0 },
                       "max_us_per_tick": max_ns as f64 / 1000.0, "max_calls_per_tick": max_calls}),
            );
        }
        let mut order: Vec<_> = steady.clone();
        order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let slowest: Vec<Value> = order
            .iter()
            .take(SLOWEST_KEPT)
            .map(|(tick, us, totals)| {
                let (day, clock) = display_clock(*tick, offset, ticks_per_day);
                let parts: Map<String, Value> = totals
                    .iter()
                    .map(|(name, row)| ((*name).to_owned(), json!({"us": row.ns as f64 / 1000.0, "calls": row.calls})))
                    .collect();
                json!({"tick": tick, "display_day": day, "clock": clock, "total_ms": f64::from(*us) / 1000.0, "scopes": parts})
            })
            .collect();
        let mut per_tick = Map::new();
        per_tick.insert(
            "tick".into(),
            json!(self
                .ticks
                .iter()
                .map(|(tick, ..)| *tick)
                .collect::<Vec<_>>()),
        );
        per_tick.insert(
            "tick_us".into(),
            json!(self.ticks.iter().map(|(_, us, _)| *us).collect::<Vec<_>>()),
        );
        for name in &names {
            let ns: Vec<u64> = self
                .ticks
                .iter()
                .map(|(.., totals)| totals.get(name).map_or(0, |row| row.ns))
                .collect();
            let calls: Vec<u64> = self
                .ticks
                .iter()
                .map(|(.., totals)| totals.get(name).map_or(0, |row| row.calls))
                .collect();
            per_tick.insert(format!("{name}.ns"), json!(ns));
            per_tick.insert(format!("{name}.calls"), json!(calls));
        }
        json!({"steady_ticks": steady.len(), "scopes": scopes, "slowest_ticks": slowest, "per_tick": per_tick})
    }
}
