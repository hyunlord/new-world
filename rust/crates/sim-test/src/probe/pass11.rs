//! Additive schema-eleven observations: fission, settlements alive, exposure
//! deaths by stage and by settlement-hour (Pass 11).
use serde_json::{json, Value};
use sim_core::components::{Agent, Founding, SettlementId};
use sim_engine::{FissionRecord, SimEngine};
use std::collections::BTreeMap;

use super::pass10::WINTER_FIRST_DAYS;

/// Display days at which each settlement's members are reported.
pub(super) const MEMBER_DAYS: [u64; 4] = [30, 60, 90, 118];

/// Schema-eleven row keys; every value is finite.
///
/// `settlements_alive`: settlements with at least one member. `fire_lit_home`:
/// whether the lowest-id settlement's fire is lit (0 without settlements).
/// `founding_agents`: residents walking with a fission group. The five
/// `*_total` keys are the cumulative fission tallies.
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let alive = r
        .settlements
        .values()
        .filter(|s| !s.member_agents.is_empty())
        .count();
    let home = r.settlements.keys().copied().min();
    let fire_lit_home = home.is_some_and(|sid| r.fires.get(&sid).is_some_and(|fire| fire.lit));
    let founding = engine.world.query::<(&Agent, &Founding)>().iter().count();
    let t = r.tallies;
    json!({"settlements_alive": alive, "fissions_total": t.fissions, "fissions_cancelled_total": t.fissions_cancelled,
        "fission_blocked_no_site_total": t.fission_blocked_no_site, "remnant_merges_total": t.remnant_merges,
        "exclusivity_moves_total": t.exclusivity_moves, "fire_lit_home": u64::from(fire_lit_home),
        "founding_agents": founding})
}

/// One settlement seen at a day checkpoint: (members, founded tick, parent).
pub(super) type SettlementDay = BTreeMap<SettlementId, (u32, u64, Option<SettlementId>)>;

/// The settlements of `engine` now, for [`seed_block`].
pub(super) fn settlements_now(engine: &SimEngine) -> SettlementDay {
    engine
        .resources
        .settlements
        .iter()
        .map(|(sid, s)| (*sid, (s.member_agents.len() as u32, s.founded_at, s.parent)))
        .collect()
}

/// An exposure death in the census: internal tick, stage, settlement.
pub(super) type ExposureDeath = (u64, String, Option<SettlementId>);

/// The per-seed schema-eleven block.
///
/// `fission_log`: every departed group. `settlements`: every settlement seen
/// at a day checkpoint (founding tick, parent, founders = its fission group's
/// size or null, members at days 30/60/90/118 or null, dissolution tick or
/// null). `exposure_by_winter`: exposure deaths by stage in each seed-winter
/// window (as `pass10::seed_winters`) with the elders alive at the day-D
/// checkpoint (`elders`). `exposure_settlement_nights`: for every
/// (settlement, night) with an exposure death, the most deaths in one hour.
/// The night of a death is the display day of its evening (its 12:00-to-12:00
/// day); the hour is its display hour.
pub(super) fn seed_block(
    engine: &SimEngine,
    days: &BTreeMap<u64, SettlementDay>,
    exposure: &[ExposureDeath],
    elders: &BTreeMap<u64, u64>,
    run_days: u64,
) -> Value {
    let r = &engine.resources;
    let tpd = r.ticks_per_day.max(1);
    let offset = r.rules.clock_offset_ticks;
    let display_day = |tick: u64| (tick + offset) / tpd;
    let log: Vec<Value> = r.fission_log.iter().map(record_json).collect();
    let dissolved: BTreeMap<SettlementId, u64> = r
        .dissolutions
        .iter()
        .map(|(sid, tick)| (*sid, *tick))
        .collect();
    let mut seen: BTreeMap<SettlementId, (u64, Option<SettlementId>)> = BTreeMap::new();
    for row in days.values() {
        for (sid, (_, founded, parent)) in row {
            seen.entry(*sid).or_insert((*founded, *parent));
        }
    }
    let settlements: Vec<Value> = seen
        .iter()
        .map(|(sid, (founded, parent))| {
            let founders = r.fission_log.iter().find(|f| f.daughter == Some(*sid)).map(|f| f.members.len());
            let members: Vec<Value> = MEMBER_DAYS
                .iter()
                .map(|d| days.get(d).and_then(|row| row.get(sid)).map_or(Value::Null, |row| json!(row.0)))
                .collect();
            // Alive (>= 1 member) at the day checkpoint 28 days after founding;
            // null when that day lies beyond the run.
            let check_day = display_day(*founded) + 28;
            let alive_28 = if check_day > run_days {
                Value::Null
            } else {
                json!(days.get(&check_day).and_then(|row| row.get(sid)).is_some_and(|row| row.0 >= 1))
            };
            json!({"settlement_id": sid, "founded_tick": founded, "founded_day": display_day(*founded), "parent": parent,
                "founders": founders, "members_at": members, "dissolved_tick": dissolved.get(sid),
                "alive_28_days_after_founding": alive_28})
        })
        .collect();
    // Exposure deaths by stage per seed-winter window [day D 18:00, D+7 07:00).
    let mut winters = Vec::new();
    for d in WINTER_FIRST_DAYS {
        let (Some(start), Some(end)) = (
            super::args::clock_tick(d, 1080, tpd, offset),
            super::args::clock_tick(d + 7, 420, tpd, offset),
        ) else {
            continue;
        };
        if end > run_days * tpd {
            continue;
        }
        let mut stages = BTreeMap::from([("child", 0u64), ("adult", 0), ("elder", 0)]);
        for (_, stage, _) in exposure.iter().filter(|(t, ..)| (start..end).contains(t)) {
            if let Some(count) = stages.get_mut(stage.as_str()) {
                *count += 1;
            }
        }
        winters.push(
            json!({"first_day": d, "child": stages["child"], "adult": stages["adult"],
            "elder": stages["elder"], "elders_at_start": elders.get(&d)}),
        );
    }
    // Most exposure deaths in one hour per (settlement, night).
    let mut nights: BTreeMap<(Option<SettlementId>, u64), BTreeMap<u64, u64>> = BTreeMap::new();
    for (tick, _, sid) in exposure {
        let clock = tick + offset;
        let night = clock.saturating_sub(tpd / 2) / tpd;
        let hour = clock / (tpd / 24).max(1);
        *nights
            .entry((*sid, night))
            .or_default()
            .entry(hour)
            .or_default() += 1;
    }
    let settlement_nights: Vec<Value> = nights
        .iter()
        .map(|((sid, night), hours)| {
            json!({"settlement": sid, "night_day": night,
            "deaths": hours.values().sum::<u64>(), "max_in_one_hour": hours.values().max()})
        })
        .collect();
    let max_hour = nights
        .values()
        .flat_map(|h| h.values())
        .max()
        .copied()
        .unwrap_or(0);
    let first_fission = r.fission_log.iter().filter_map(|f| f.founded_tick).min();
    json!({
        "fissions": r.tallies.fissions, "fissions_cancelled": r.tallies.fissions_cancelled,
        "fission_attempts": r.fission_log.len(), "fission_blocked_no_site": r.tallies.fission_blocked_no_site,
        "remnant_merges": r.tallies.remnant_merges, "exclusivity_moves": r.tallies.exclusivity_moves,
        "first_fission_tick": first_fission, "first_fission_day": first_fission.map(display_day),
        "fission_log": log, "settlements": settlements,
        "exposure_by_winter": winters, "exposure_settlement_nights": settlement_nights,
        "max_exposure_deaths_one_settlement_one_hour": max_hour,
    })
}

fn record_json(f: &FissionRecord) -> Value {
    json!({"fission_id": f.fission_id, "parent": f.parent, "site": [f.site.0, f.site.1],
        "depart_tick": f.depart_tick, "group_size": f.members.len(), "adults": f.adults,
        "daughter": f.daughter, "founded_tick": f.founded_tick, "cancelled_tick": f.cancelled_tick,
        "food_taken": f.food_taken, "wood_taken": f.wood_taken, "settled_at_leader": f.settled_at_leader})
}

/// Per-seed wild-region facts (only when the world has regions).
pub(crate) fn regions_json(engine: &SimEngine) -> Option<Value> {
    let r = &engine.resources;
    if r.worldgen_report.regions_requested == 0 {
        return None;
    }
    let w = r.worldgen_report;
    Some(json!({
        "regions_requested": w.regions_requested, "regions_placed": w.regions_placed,
        "region_bushes_requested": w.region_bushes_requested, "region_bushes_placed": w.region_bushes_placed,
        "region_deadwood_requested": w.region_deadwood_requested, "region_deadwood_placed": w.region_deadwood_placed,
        "centres": r.wild_regions.iter().map(|g| json!({"centre": [g.centre.0, g.centre.1], "bushes": g.bushes,
            "deadwood": g.deadwood, "water_distance": g.water_distance})).collect::<Vec<_>>(),
    }))
}
