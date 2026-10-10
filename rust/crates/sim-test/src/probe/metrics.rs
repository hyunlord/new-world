//! Snapshot-only metrics, accumulated in AgentId order for stable floating sums.
use super::fingerprint::{fingerprint, state_tag};
use serde_json::{json, Value};
use sim_core::components::{
    Agent, AgentState, ConstructionSite, Hunger, Inventory, Position, ResourceKind, Sleep, Social,
    Thirst,
};
use sim_engine::{SimEngine, RESOURCE_SOURCE_INFINITE};
use std::collections::BTreeMap;

/// Linearly interpolate a sorted quantile, returning zero for no observations.
pub(super) fn percentile(values: &[f64], fraction: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let position = fraction * (sorted.len() - 1) as f64;
    let lo = position.floor() as usize;
    let hi = position.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (position - lo as f64)
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// Collect the stable schema without changing simulation state.
pub(super) fn sample(engine: &SimEngine, scene: &str, seed: u64, initial_shelters: usize) -> Value {
    let r = &engine.resources;
    let mut homes = BTreeMap::new();
    let mut settlements: Vec<_> = r.settlements.values().collect();
    settlements.sort_by_key(|s| s.settlement_id);
    for settlement in &settlements {
        for id in &settlement.member_agents {
            homes.entry(*id).or_insert(settlement.formation_tile);
        }
    }
    let mut query = engine.world.query::<(
        &Agent,
        &Position,
        Option<&AgentState>,
        Option<&Hunger>,
        Option<&Thirst>,
        Option<&Sleep>,
        Option<&Social>,
        Option<&Inventory>,
    )>();
    let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
    rows.sort_by_key(|row| row.0.id);
    let alive = rows.len();
    let (mut hunger, mut thirst, mut fatigue, mut loneliness, mut distances) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut idle, mut seeking, mut consuming, mut breach, mut carried) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    for (agent, pos, state, h, t, sl, so, inv) in rows {
        let h = h.map_or(0.0, |v| f64::from(v.value));
        let t = t.map_or(0.0, |v| v.value);
        hunger.push(h);
        thirst.push(t);
        fatigue.push(sl.map_or(0.0, |v| v.fatigue));
        loneliness.push(so.map_or(0.0, |v| v.loneliness));
        match state_tag(state) {
            0 => {
                idle += 1;
                if h > 50.0 || t > 50.0 {
                    breach += 1;
                }
            }
            1 => seeking += 1,
            _ => consuming += 1,
        }
        carried += u64::from(inv.map_or(0, |v| v.get(ResourceKind::Food)));
        if let Some(&(x, y)) = homes.get(&agent.id) {
            distances.push(f64::from(pos.x.abs_diff(x).max(pos.y.abs_diff(y))));
        }
    }
    // Part 1 has no building kind taxonomy; this is intentionally zero until Part 2.
    let shelters = 0usize;
    json!({
        "schema_version": 1, "scene": scene, "seed": seed,
        "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "tick": engine.current_tick(), "day": engine.current_tick() as f64 / r.ticks_per_day as f64,
        "alive": alive, "births": r.tallies.births,
        "deaths_starvation": r.tallies.deaths_starvation, "deaths_dehydration": r.tallies.deaths_dehydration, "deaths_combat": r.tallies.deaths_combat,
        "hunger_mean": mean(&hunger), "hunger_p90": percentile(&hunger, 0.9),
        "thirst_mean": mean(&thirst), "thirst_p90": percentile(&thirst, 0.9),
        "fatigue_mean": mean(&fatigue), "loneliness_mean": mean(&loneliness),
        "state_idle": idle, "state_seeking": seeking, "state_consuming": consuming,
        "idle_with_breach": breach, "settlements": settlements.len(), "settled_agents": distances.len(),
        "home_distance_mean": mean(&distances), "buildings": r.building_registry.len(),
        "shelters": shelters, "construction_sites": engine.world.query::<&ConstructionSite>().iter().count(),
        "shelters_completed": shelters.saturating_sub(initial_shelters),
        "stockpile_food": settlements.iter().map(|s| u64::from(s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))).sum::<u64>(),
        "carried_food": carried,
        "food_tiles_total": r.food_tiles.values().map(|v| u64::from(*v)).sum::<u64>(),
        "food_tiles_finite_total": r.food_tiles.values().filter(|v| **v != RESOURCE_SOURCE_INFINITE).map(|v| u64::from(*v)).sum::<u64>(),
        "water_tiles_count": r.water_tiles.values().filter(|v| **v > 0).count(),
        "fingerprint": fingerprint(engine)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quantiles_interpolate_and_empty_is_finite() {
        assert_eq!(percentile(&[], 0.9), 0.0);
        assert_eq!(percentile(&[10.0, 0.0], 0.5), 5.0);
        assert_eq!(percentile(&[0.0, 10.0], 0.9), 9.0);
    }
    #[test]
    fn states_partition_population_and_snapshot_does_not_mutate() {
        let engine = crate::scenes::legacy_scene();
        let before = fingerprint(&engine);
        let row = sample(&engine, "legacy", 0, 0);
        let count = |key: &str| row[key].as_u64().expect("integer metric");
        assert_eq!(
            count("alive"),
            count("state_idle") + count("state_seeking") + count("state_consuming")
        );
        assert_eq!(before, fingerprint(&engine));
    }
}
