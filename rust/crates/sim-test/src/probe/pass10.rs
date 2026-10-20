//! Additive schema-ten observations: exposure deaths, health and the hidden
//! world parameters (Pass 10).
use serde_json::{json, Value};
use sim_core::components::{Agent, BodyHealth};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

use super::args::clock_tick;

/// The first display day of each winter of a 120-day run (days per season 7).
pub(super) const WINTER_FIRST_DAYS: [u64; 4] = [22, 50, 78, 106];

/// Schema-ten row keys; every value is finite.
///
/// `deaths_exposure`: the cumulative exposure tally. `min_hp`: the lowest hp
/// among living agents (0 without agents). `injured_members`: settlement
/// members whose hp is below their maximum.
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let (min_hp, injured) = health(engine);
    json!({"deaths_exposure": engine.resources.tallies.deaths_exposure, "min_hp": min_hp,
        "injured_members": injured})
}

/// `(min_hp, injured_members)` as [`metrics`] reports them; the recorder reads
/// the same function.
pub(crate) fn health(engine: &SimEngine) -> (f64, u64) {
    let members: BTreeSet<_> = engine
        .resources
        .settlements
        .values()
        .flat_map(|s| s.member_agents.iter().copied())
        .collect();
    let mut rows: Vec<_> = engine
        .world
        .query::<(&Agent, &BodyHealth)>()
        .iter()
        .map(|(_, (a, h))| (a.id, *h))
        .collect();
    rows.sort_by_key(|row| row.0);
    let min_hp = rows
        .iter()
        .map(|row| row.1.hp)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let injured = rows
        .iter()
        .filter(|(id, h)| members.contains(id) && h.hp < h.max_hp)
        .count() as u64;
    (min_hp, injured)
}

/// Per-seed facts of the world at tick 0: the hidden world parameters (null
/// outside `game_v16`), the seed's own winter severity weights, the starting
/// agents and the worldgen counts (deadwood spots requested and placed,
/// berry-repair terrain swaps).
pub(super) fn seed_info(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let mut info = json!({
        "world_variety": r.world_variety.map(|v| variety_json(&v)),
        "winter_severity_permille": r.rules.society.firewood.and_then(|rule| rule.winter_severity_permille),
        "food_regen_permille": r.rules.seasons.map(|s| s.regen_permille),
        "starting_agents": engine.world.query::<&Agent>().iter().count(),
        "deadwood_spots_requested": r.worldgen_report.deadwood_spots_requested,
        "deadwood_spots_placed": r.worldgen_report.deadwood_spots_placed,
        "berry_repair_swaps": r.worldgen_report.berry_repair_swaps,
        "exposure": r.rules.society.exposure.map(|e| json!({
            "damage_per_cold_hour": e.damage_per_cold_hour, "heal_block_hours": e.heal_block_hours})),
    });
    // Pass 11: per-seed v17 facts, present only in worlds that have them so
    // earlier outputs are unchanged.
    if let Some(object) = info.as_object_mut() {
        if let Some(regions) = super::pass11::regions_json(engine) {
            object.insert("wild_regions".into(), regions);
        }
        if let Some(e) = r
            .rules
            .society
            .exposure
            .filter(|e| e.use_hardiness || e.hunger_weight != 0.0)
        {
            object.insert(
                "exposure_v2".into(),
                json!({"hunger_weight": e.hunger_weight, "use_hardiness": e.use_hardiness}),
            );
        }
        if let Some(rain) = r
            .rules
            .society
            .firewood
            .filter(|f| f.rain_soak_permille > 0 || f.regional_wood)
        {
            object.insert(
                "firewood_v17".into(),
                json!({"rain_soak_permille": rain.rain_soak_permille,
                "regional_wood": rain.regional_wood}),
            );
        }
        if let Some(f) = r.rules.society.fission {
            object.insert("fission".into(), json!({"min_members": f.min_members, "crowding": f.crowding,
                "group_share": f.group_share, "group_min": f.group_min, "group_max": f.group_max,
                "min_adults": f.min_adults, "cooldown_days": f.cooldown_days, "min_age_days": f.min_age_days,
                "site_clearance": f.site_clearance, "stock_share": f.stock_share,
                "max_settlements": f.max_settlements, "settle_timeout_days": f.settle_timeout_days,
                "remnant_members": f.remnant_members}));
        }
        if let Some(utilization) = match r.rules.society.birth {
            sim_core::rules::BirthRule::Provisioned {
                sustain: Some(s), ..
            } if r.rules.society.fission.is_some()
                || r.rules.society.exposure.is_some_and(|e| e.use_hardiness) =>
            {
                Some(s.utilization)
            }
            _ => None,
        } {
            object.insert("sustain_utilization".into(), json!(utilization));
        }
    }
    info
}

/// The eight hidden parameters of a `game_v16` world as a JSON object (Pass
/// 11: plus the three wild-region parameters when the world has regions).
pub(crate) fn variety_json(v: &sim_core::variety::WorldVariety) -> Value {
    let mut out = json!({"starting_agents": v.starting_agents, "berry_bushes": v.berry_bushes,
        "berry_capacity": v.berry_capacity, "deadwood_spots": v.deadwood_spots,
        "deadwood_capacity": v.deadwood_capacity, "forest_permille": v.forest_permille,
        "climate": v.climate, "food_regen_permille": v.food_regen_permille});
    // Pass 11: the wild-region parameters, only for worlds that have regions.
    if v.wild_regions > 0 {
        if let Some(object) = out.as_object_mut() {
            object.insert("wild_regions".into(), json!(v.wild_regions));
            object.insert("region_bushes".into(), json!(v.region_bushes));
            object.insert("region_deadwood".into(), json!(v.region_deadwood));
        }
    }
    out
}

/// Seed-winter rows of one run: for each winter whose window
/// [day D 18:00, day D+7 07:00) lies inside the run, the members and alive
/// count at day D 18:00, the dark nights (evenings D..D+6 whose 02:00 sample
/// has the fire out), the severity (read at day D+1 02:00) and the exposure
/// deaths whose internal tick lies in the window.
pub(super) fn seed_winters(
    clock_rows: &BTreeMap<(u64, u32), Value>,
    exposure_ticks: &[u64],
    days: u64,
    ticks_per_day: u64,
    offset: u64,
) -> Vec<Value> {
    let mut out = Vec::new();
    for d in WINTER_FIRST_DAYS {
        let (Some(start), Some(end)) = (
            clock_tick(d, 1080, ticks_per_day, offset),
            clock_tick(d + 7, 420, ticks_per_day, offset),
        ) else {
            continue;
        };
        if end > days * ticks_per_day {
            continue;
        }
        let at_start = clock_rows.get(&(d, 1080));
        let dark = (d..d + 7)
            .filter(|e| {
                clock_rows
                    .get(&(e + 1, 120))
                    .is_some_and(|row| row["fire_lit"].as_u64() == Some(0))
            })
            .count();
        let severity = clock_rows
            .get(&(d + 1, 120))
            .and_then(|row| row["winter_severity"].as_i64())
            .unwrap_or(-1);
        let deaths = exposure_ticks
            .iter()
            .filter(|t| (start..end).contains(*t))
            .count();
        out.push(json!({"first_day": d, "start_tick": start, "end_tick": end,
            "members_at_start": at_start.and_then(|row| row["settled_agents"].as_u64()),
            "alive_at_start": at_start.and_then(|row| row["alive"].as_u64()),
            "dark_nights": dark, "severity": severity, "exposure_deaths": deaths}));
    }
    out
}

/// Every 06:00 sample of a run: `[day, min_hp, injured_members, alive]`.
pub(super) fn dawn_health(clock_rows: &BTreeMap<(u64, u32), Value>) -> Vec<Value> {
    clock_rows
        .iter()
        .filter(|((_, minute), _)| *minute == 360)
        .map(|((day, _), row)| json!([day, row["min_hp"], row["injured_members"], row["alive"]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_world_reports_full_health_and_its_variety() {
        let engine = crate::scenes::baseline_scene(3);
        let variety = sim_core::variety::WorldVariety::draw_v18(3);
        let row = metrics(&engine);
        assert_eq!(row.as_object().expect("object").len(), 3);
        // Tick 0: nobody has been exposed yet.
        assert_eq!(row["deaths_exposure"], 0);
        assert_eq!(row["min_hp"], 100.0);
        assert_eq!(row["injured_members"], 0);
        let info = seed_info(&engine);
        assert_eq!(info["world_variety"], variety_json(&variety));
        let exposure = engine.resources.rules.society.exposure.expect("exposure");
        assert_eq!(
            info["exposure"],
            json!({"damage_per_cold_hour": exposure.damage_per_cold_hour,
                "heal_block_hours": exposure.heal_block_hours})
        );
        assert_eq!(info["starting_agents"], variety.starting_agents);
        let firewood = engine.resources.rules.society.firewood.expect("firewood");
        assert_eq!(
            info["winter_severity_permille"],
            json!(firewood.winter_severity_permille)
        );
        assert_eq!(info["deadwood_spots_requested"], variety.deadwood_spots);
    }

    #[test]
    fn seed_winter_windows_count_deaths_and_dark_nights() {
        let mut rows = BTreeMap::new();
        rows.insert((22, 1080), json!({"settled_agents": 40, "alive": 41}));
        for day in 23..=29 {
            rows.insert(
                (day, 120),
                json!({"fire_lit": u64::from(day % 2 == 0), "winter_severity": 2}),
            );
        }
        // Window [day 22 18:00, day 29 07:00) = [30_840, 40_260) with offset 480.
        let start = clock_tick(22, 1080, 1440, 480).expect("start");
        let end = clock_tick(29, 420, 1440, 480).expect("end");
        assert_eq!((start, end), (30_840, 40_260));
        let ticks = [start - 1, start, end - 1, end];
        let winters = seed_winters(&rows, &ticks, 120, 1440, 480);
        assert_eq!(winters.len(), 4);
        assert_eq!(winters[0]["exposure_deaths"], 2);
        assert_eq!(winters[0]["dark_nights"], 4);
        assert_eq!(winters[0]["severity"], 2);
        assert_eq!(winters[0]["members_at_start"], 40);
        assert!(seed_winters(&rows, &ticks, 10, 1440, 480).is_empty());
    }
}
