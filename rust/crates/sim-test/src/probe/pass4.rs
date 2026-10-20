//! Additive schema-four observations: seasons, store target, foraging and aging.
use serde_json::{json, Value};
use sim_core::components::{
    Agent, AgentState, Child, Elder, Inventory, Life, Position, ResourceKind, TargetKind,
};
use sim_engine::SimEngine;
use sim_systems::runtime::agent::hauls_home;
use sim_systems::runtime::decision::agent_decision::clock_in_window;
use sim_systems::runtime::resource_regen::season_at;
use sim_systems::runtime::settlement::firewood::in_gather_window;
use sim_systems::runtime::settlement::food_economy::{stock_in_hand, stock_target, WINTER};
use std::collections::BTreeMap;

/// Schema-four keys; every value is finite and zero when its rule is off.
pub(super) fn metrics(engine: &SimEngine, fixed_supply: f64) -> Value {
    let r = &engine.resources;
    let (season, day_of_season) = season_at(r).map_or((0, 0), |(s, d)| (s, d));
    let days_per_season = r.rules.seasons.map_or(0, |rule| rule.days_per_season);
    let permille = |index: usize| {
        r.rules
            .seasons
            .map_or(1.0, |rule| f64::from(rule.regen_permille[index]) / 1000.0)
    };
    let mut settlements: Vec<_> = r.settlements.keys().copied().collect();
    settlements.sort_unstable();
    let mut homes = BTreeMap::new();
    let (mut target, mut in_hand) = (0.0, 0u64);
    for sid in &settlements {
        target += stock_target(&engine.world, r, *sid).unwrap_or(0.0);
        in_hand += stock_in_hand(&engine.world, r, *sid);
        if let Some(settlement) = r.settlements.get(sid) {
            for id in &settlement.member_agents {
                homes.entry(*id).or_insert(settlement.formation_tile);
            }
        }
    }
    let forage = r
        .rules
        .agent
        .forage
        .filter(|_| r.rules.society.stock_target.is_some());
    let in_window =
        forage.is_some_and(|rule| clock_in_window(r, rule.start_minute, rule.end_minute));
    let firewood = r.rules.society.firewood;
    let wood_window = in_gather_window(r);
    let mut query = engine.world.query::<(
        &Agent,
        &Position,
        Option<&AgentState>,
        Option<&Inventory>,
        Option<&Child>,
        Option<&Elder>,
        Option<&Life>,
    )>();
    let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
    rows.sort_by_key(|row| row.0.id);
    let (mut elders, mut foraging, mut hauling, mut aged, mut age_days) =
        (0u64, 0u64, 0u64, 0u64, 0.0);
    for (agent, pos, state, inventory, child, elder, life) in rows {
        elders += u64::from(elder.is_some());
        if let Some(life) = life {
            aged += 1;
            age_days +=
                life.age_ticks(engine.current_tick()) as f64 / r.ticks_per_day.max(1) as f64;
        }
        let Some(rule) = forage else { continue };
        foraging += u64::from(matches!(
            state,
            Some(AgentState::Seeking {
                target: TargetKind::GatherFood
            })
        ));
        let carried = inventory.map_or(0, |inv| inv.get(ResourceKind::Food));
        // Pass 5: wood loads count as hauling under the firewood rule.
        let wood = if firewood.is_some() {
            inventory.map_or(0, |inv| inv.get(ResourceKind::Wood))
        } else {
            0
        };
        if child.is_none() && state.is_none_or(|s| *s == AgentState::Idle) {
            if let Some(&(x, y)) = homes.get(&agent.id) {
                let distance = pos.x.abs_diff(x).max(pos.y.abs_diff(y));
                hauling += u64::from(hauls_home(
                    Some(&rule),
                    firewood.as_ref(),
                    carried,
                    wood,
                    in_window,
                    wood_window,
                    distance,
                ));
            }
        }
    }
    let t = r.tallies;
    json!({"season": season, "day_of_season": day_of_season, "days_per_season": days_per_season,
        "elders": elders, "stock_target": target, "stock_in_hand": in_hand,
        "foraging_now": foraging, "hauling_now": hauling, "deaths_old_age": t.deaths_old_age,
        "forage_trips_total": t.forage_trips, "haul_deposits_total": t.haul_deposits,
        "regen_lost_at_cap_total": t.regen_lost_at_cap,
        "mean_age_days": if aged == 0 { 0.0 } else { age_days / aged as f64 },
        "food_supply_season_per_day": fixed_supply * permille(season),
        "food_supply_winter_per_day": fixed_supply * permille(WINTER)})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_world_reports_its_seasons_at_tick_zero() {
        let engine = crate::scenes::baseline_scene(3);
        let seasons = engine.resources.rules.seasons.expect("baseline seasons");
        let row = metrics(&engine, 120.0);
        // Tick 0 is 08:00 of the first spring day; nothing has happened yet.
        assert_eq!(row["season"], 0);
        assert_eq!(row["day_of_season"], 1); // one-based
        assert_eq!(row["days_per_season"], seasons.days_per_season);
        for key in [
            "foraging_now",
            "hauling_now",
            "deaths_old_age",
            "forage_trips_total",
            "haul_deposits_total",
        ] {
            assert_eq!(row[key], 0, "{key}");
        }
        // The fixed supply scaled by the world's spring and winter regrowth.
        let scaled = |season: usize| 120.0 * (f64::from(seasons.regen_permille[season]) / 1000.0);
        assert_eq!(row["food_supply_season_per_day"], scaled(0));
        assert_eq!(row["food_supply_winter_per_day"], scaled(WINTER));
        assert!(row
            .as_object()
            .expect("object")
            .values()
            .all(|v| !v.is_null()));
    }

    #[test]
    fn without_seasons_the_supply_is_unscaled() {
        let mut engine = crate::scenes::baseline_scene(3);
        engine.resources.rules.seasons = None;
        let row = metrics(&engine, 120.0);
        assert_eq!(row["days_per_season"], 0);
        assert_eq!(row["food_supply_season_per_day"], 120.0);
        assert_eq!(row["food_supply_winter_per_day"], 120.0);
    }
}
