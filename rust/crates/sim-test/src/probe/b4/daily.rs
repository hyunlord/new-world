//! Daily and hourly raw counters, using the existing colony death and cold definitions.
use super::{craft, types::*};
use sim_core::colony::{people::Apparel, CancelReason};
use sim_core::components::{Agent, Child, Cold};
use sim_engine::SimEngine;
use std::collections::BTreeMap;

pub fn population(engine: &SimEngine) -> u64 {
    engine.world.query::<&Agent>().iter().fold(0, |n, _| n + 1)
}

pub fn day_end(tick: u64) -> Option<u32> {
    let elapsed = tick.checked_add(1)?;
    elapsed
        .is_multiple_of(1440)
        .then(|| u32::try_from(elapsed / 1440).ok())
        .flatten()
}

pub fn sample_cold(engine: &SimEngine, counts: &mut ColdCounts) {
    let r = &engine.resources;
    if !r.current_tick.is_multiple_of(60) {
        return;
    }
    let minute = (r.current_tick + r.rules.clock_offset_ticks) % 1440;
    if (360..1200).contains(&minute)
        || sim_systems::runtime::resource_regen::season_at(r).is_none_or(|(s, _)| s != 3)
    {
        return;
    }
    for (_, (_, apparel, cold)) in engine
        .world
        .query::<(&Agent, Option<&Apparel>, Option<&Cold>)>()
        .without::<&Child>()
        .iter()
    {
        if apparel.is_some_and(|a| a.top.cold_permille() < 1000) {
            counts.clothed_eligible += 1;
            counts.clothed_cold += u64::from(cold.is_some());
        } else {
            counts.unclothed_eligible += 1;
            counts.unclothed_cold += u64::from(cold.is_some());
        }
    }
}

pub fn row(engine: &SimEngine, day: u32, cold: &ColdCounts, conservation: &Conservation) -> DayRow {
    let r = &engine.resources;
    let mut garments: BTreeMap<u32, (u64, u32)> = BTreeMap::new();
    for (_, (_, a)) in engine.world.query::<(&Agent, &Apparel)>().iter() {
        let entry = garments
            .entry(a.top.item().code())
            .or_insert((0, a.top.cold_permille()));
        entry.0 += 1;
    }
    let insulating_wearers = garments
        .values()
        .filter(|(_, p)| *p < 1000)
        .map(|(n, _)| n)
        .sum();
    let t = &r.tallies;
    let w = &r.colony.wildlife.tallies;
    DayRow {
        day,
        executed_tick: r.current_tick,
        population: population(engine),
        insulating_wearers,
        garment_counts: garments
            .into_iter()
            .map(|(item_code, (count, cold_permille))| GarmentCount {
                item_code,
                count,
                cold_permille,
            })
            .collect(),
        craft_tallies: craft::rows(&r.colony.tallies.craft_quality),
        cold: cold.clone(),
        deaths_total: t.deaths_starvation
            + t.deaths_dehydration
            + t.deaths_exposure
            + t.deaths_combat
            + t.deaths_old_age,
        jobs_done_total: r.colony.tallies.jobs_done,
        cancel_totals: r
            .colony
            .tallies
            .cancels
            .iter()
            .map(|(reason, &count)| CancelCount {
                reason: cancel_name(*reason),
                count,
            })
            .collect(),
        meat: MeatCounts {
            dropped: w.meat_dropped,
            eaten: w.meat_eaten,
            spoiled: w.meat_spoiled,
            hauls: w.meat_hauls,
        },
        ledger_violation_ticks: conservation.ledger_violation_ticks,
        reservation_violation_ticks: conservation.reservation_violation_ticks,
    }
}

fn cancel_name(reason: CancelReason) -> &'static str {
    match reason {
        CancelReason::Needs => "needs",
        CancelReason::TargetGone => "target_gone",
        CancelReason::Command => "command",
        CancelReason::Stuck => "stuck",
        CancelReason::Interrupted => "interrupted",
        CancelReason::Died => "died",
        CancelReason::Unreachable => "unreachable",
    }
}
