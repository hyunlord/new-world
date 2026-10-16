//! Additive schema-thirteen observations (Pass 13).
//!
//! Row keys: `area_floor_placed_total` and `area_floor_lost_at_max_total`, the
//! cumulative tallies of the settled-area floor in its own accumulator
//! (`FirewoodRule::floor_separate`, `game_v19`).
//!
//! Per-seed block ([`seed_block`]): under fission by accumulated pressure
//! (`FissionRule::pressure`), every eligible daily check as `[tick,
//! settlement, P, θ, h, g, q, s]` (P after the day's accumulation); the
//! display day each settlement's P first exceeded half of its θ; and every
//! departed fission as `[fission_id, parent, depart_tick, P at the split, θ at
//! the split, daughter or -1, founded tick or -1, cancelled 0/1]`.
use serde_json::{json, Value};
use sim_core::components::SettlementId;
use sim_engine::SimEngine;
use std::collections::BTreeMap;

/// Schema-thirteen row keys (see the module docs).
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let t = &engine.resources.tallies;
    json!({"area_floor_placed_total": t.area_floor_placed, "area_floor_lost_at_max_total": t.area_floor_lost_at_max})
}

/// The per-seed schema-thirteen block (see the module docs).
pub(super) fn seed_block(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let tpd = r.ticks_per_day.max(1);
    let offset = r.rules.clock_offset_ticks;
    let pressured = r.rules.society.fission.is_some_and(|f| f.pressure.is_some());
    let checks: Vec<Value> = r
        .fission_checks
        .iter()
        .filter(|_| pressured)
        .map(|c| json!([c.tick, c.settlement, c.pressure, c.threshold, c.h, c.g, c.q, c.s]))
        .collect();
    let mut half: BTreeMap<SettlementId, u64> = BTreeMap::new();
    for c in r.fission_checks.iter().filter(|_| pressured) {
        if c.threshold > 0 && 2 * c.pressure > c.threshold {
            half.entry(c.settlement).or_insert((c.tick + offset) / tpd);
        }
    }
    let splits: Vec<Value> = r
        .fission_log
        .iter()
        .map(|f| {
            json!([f.fission_id, f.parent, f.depart_tick, f.pressure_at_split, f.threshold_at_split,
                f.daughter.map_or(-1, i64::from), f.founded_tick.map_or(-1, |t| t as i64), u8::from(f.cancelled_tick.is_some())])
        })
        .collect();
    json!({"pressure_checks": checks, "half_threshold_day": half.iter().map(|(sid, day)| json!([sid, day])).collect::<Vec<_>>(),
        "splits": splits, "area_floor_placed": r.tallies.area_floor_placed, "area_floor_lost_at_max": r.tallies.area_floor_lost_at_max})
}
