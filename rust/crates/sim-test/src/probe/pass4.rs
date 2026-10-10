//! Additive schema-four observations: seasons, store target, foraging and aging.
use serde_json::{json, Value};
use sim_core::components::{
    Agent, AgentState, Child, Elder, Inventory, Life, Position, ResourceKind, TargetKind,
};
use sim_engine::SimEngine;
use sim_systems::runtime::agent::haul_due;
use sim_systems::runtime::decision::agent_decision::clock_in_window;
use sim_systems::runtime::resource_regen::season_at;
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
    let forage = r.rules.agent.forage.filter(|_| r.rules.society.stock_target.is_some());
    let in_window = forage.is_some_and(|rule| clock_in_window(r, rule.start_minute, rule.end_minute));
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
    let (mut elders, mut foraging, mut hauling, mut aged, mut age_days) = (0u64, 0u64, 0u64, 0u64, 0.0);
    for (agent, pos, state, inventory, child, elder, life) in rows {
        elders += u64::from(elder.is_some());
        if let Some(life) = life {
            aged += 1;
            age_days += life.age_ticks(engine.current_tick()) as f64 / r.ticks_per_day.max(1) as f64;
        }
        let Some(rule) = forage else { continue };
        foraging += u64::from(matches!(
            state,
            Some(AgentState::Seeking { target: TargetKind::GatherFood })
        ));
        let carried = inventory.map_or(0, |inv| inv.get(ResourceKind::Food));
        if child.is_none() && state.is_none_or(|s| *s == AgentState::Idle) {
            if let Some(&(x, y)) = homes.get(&agent.id) {
                let distance = pos.x.abs_diff(x).max(pos.y.abs_diff(y));
                hauling += u64::from(haul_due(&rule, carried, in_window, distance));
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
    fn v9_world_reports_neutral_pass4_values() {
        let engine = crate::scenes::game_v9_scene(3);
        let row = metrics(&engine, 120.0);
        for key in ["season", "day_of_season", "days_per_season", "elders", "foraging_now",
            "hauling_now", "deaths_old_age", "forage_trips_total", "haul_deposits_total"] {
            assert_eq!(row[key], 0, "{key}");
        }
        assert_eq!(row["stock_target"], 0.0);
        assert_eq!(row["mean_age_days"], 0.0);
        assert_eq!(row["food_supply_season_per_day"], 120.0);
        assert_eq!(row["food_supply_winter_per_day"], 120.0);
        assert!(row.as_object().expect("object").values().all(|v| !v.is_null()));
    }
}
