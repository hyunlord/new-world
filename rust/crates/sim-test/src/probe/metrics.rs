//! Snapshot-only metrics, accumulated in AgentId order for stable floating sums.
use super::fingerprint::{fingerprint, state_tag};
use serde_json::{json, Value};
use sim_core::components::{
    Agent, AgentState, BuildingKind, ConstructionSite, Hunger, Inventory, Position, ResourceKind,
    SettlementMigrant, Sleep, Social, TargetKind, Thirst,
};
use sim_engine::{SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::decision::agent_decision::{
    actionable_need_with_fatigue_threshold, fatigue_threshold_at, NeedAvailability,
};
use std::collections::{BTreeMap, BTreeSet};

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

/// Count typed shelters, including boot placements not stamped until tick one.
pub(super) fn shelter_count(engine: &SimEngine) -> usize {
    engine
        .resources
        .building_kinds
        .values()
        .filter(|kind| **kind == BuildingKind::Shelter)
        .count()
}

/// Current completed-shelter capacity; absent for legacy housing policy.
pub(super) fn housing_capacity(engine: &SimEngine) -> Option<u64> {
    engine
        .resources
        .rules
        .society
        .shelter_planning
        .map(|rule| shelter_count(engine) as u64 * u64::from(rule.residents_per_shelter))
}

/// Collect the stable schema without changing simulation state.
pub(crate) fn sample(engine: &SimEngine, scene: &str, seed: u64, initial_shelters: usize) -> Value {
    let r = &engine.resources;
    let mut homes = BTreeMap::new();
    let mut settlements: Vec<_> = r.settlements.values().collect();
    settlements.sort_by_key(|s| s.settlement_id);
    for settlement in &settlements {
        for id in &settlement.member_agents {
            homes.entry(*id).or_insert(settlement.formation_tile);
        }
    }
    let availability = NeedAvailability::from_resources(r);
    let fatigue_threshold = fatigue_threshold_at(r);
    let physical_sources = [
        r.food_tiles.values().any(|q| *q > 0),
        r.water_tiles.values().any(|q| *q > 0),
        r.sleep_tiles.values().any(|q| *q > 0),
    ];
    let mut actionable = 0u64;
    let mut unavailable = 0u64;
    let social_chasing = engine
        .world
        .query::<(&AgentState, Option<&SettlementMigrant>)>()
        .iter()
        .filter(|(_, (state, migrant))| {
            migrant.is_none()
                && matches!(
                    state,
                    AgentState::Seeking {
                        target: TargetKind::Agent(_)
                    }
                )
        })
        .count();
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
        let has_actionable_need =
            actionable_need_with_fatigue_threshold(h, t, sl, availability, fatigue_threshold)
                .is_some();
        let h = h.map_or(0.0, |v| f64::from(v.value));
        let t = t.map_or(0.0, |v| v.value);
        hunger.push(h);
        thirst.push(t);
        fatigue.push(sl.map_or(0.0, |v| v.fatigue));
        loneliness.push(so.map_or(0.0, |v| v.loneliness));
        match state_tag(state) {
            0 => {
                idle += 1;
                let needs = [
                    h > 50.0,
                    t > 50.0,
                    sl.is_some_and(|sleep| sleep.fatigue > 50.0),
                ];
                actionable += u64::from(has_actionable_need);
                // Physical unavailability remains observable even when legacy rules
                // force the decision helper to consider absent resources available.
                if needs.iter().any(|breached| *breached)
                    && !needs
                        .iter()
                        .zip(physical_sources)
                        .any(|(breached, source)| *breached && source)
                {
                    unavailable += 1;
                }
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
    let shelters = shelter_count(engine);
    let stamped_tiles: BTreeSet<_> = r.building_registry.values().copied().collect();
    let buildings = r.building_registry.len()
        + r.building_kinds
            .keys()
            .filter(|tile| !stamped_tiles.contains(*tile))
            .count();
    let mut row = json!({
        "schema_version": 13, "scene": scene, "seed": seed,
        "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "tick": engine.current_tick(), "day": engine.current_tick() as f64 / r.ticks_per_day as f64,
        "alive": alive, "births": r.tallies.births,
        "deaths_starvation": r.tallies.deaths_starvation, "deaths_dehydration": r.tallies.deaths_dehydration, "deaths_combat": r.tallies.deaths_combat,
        "hunger_mean": mean(&hunger), "hunger_p90": percentile(&hunger, 0.9),
        "thirst_mean": mean(&thirst), "thirst_p90": percentile(&thirst, 0.9),
        "fatigue_mean": mean(&fatigue), "loneliness_mean": mean(&loneliness),
        "state_idle": idle, "state_seeking": seeking, "state_consuming": consuming,
        "idle_with_breach": breach, "settlements": settlements.len(), "settled_agents": distances.len(),
        "home_distance_mean": mean(&distances), "buildings": buildings,
        "shelters": shelters, "construction_sites": engine.world.query::<&ConstructionSite>().iter().count(),
        "shelters_completed": shelters.saturating_sub(initial_shelters),
        "stockpile_food": settlements.iter().map(|s| u64::from(s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))).sum::<u64>(),
        "carried_food": carried,
        "food_tiles_total": r.food_tiles.values().map(|v| u64::from(*v)).sum::<u64>(),
        "food_tiles_finite_total": r.food_tiles.values().filter(|v| **v != RESOURCE_SOURCE_INFINITE).map(|v| u64::from(*v)).sum::<u64>(),
        "water_tiles_count": r.water_tiles.values().filter(|v| **v > 0).count(),
        "idle_breach_actionable": actionable, "idle_breach_unavailable": unavailable,
        "social_chasing": social_chasing, "social_completed": r.tallies.social_completed,
        "fingerprint": fingerprint(engine)
    });
    let economy = super::long_run::metrics(engine);
    let fixed_supply = economy["food_supply_per_day"].as_f64().unwrap_or(0.0);
    for extra in [economy, super::pass4::metrics(engine, fixed_supply), super::pass5::metrics(engine),
        super::pass6::metrics(engine), super::pass7::metrics(engine), super::pass8::metrics(engine),
        super::pass10::metrics(engine), super::pass11::metrics(engine), super::pass12::metrics(engine), super::pass13::metrics(engine)] {
        if let (Some(base), Some(extra)) = (row.as_object_mut(), extra.as_object()) {
            base.extend(extra.iter().map(|(key, value)| (key.clone(), value.clone())));
        }
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn idle_actionable_uses_night_threshold_and_preserves_v8() {
        use sim_core::rules::{NightRestRule, WorldRules};
        for (night_enabled, offset, expected) in [(false, 1260, 0), (true, 1260, 1), (true, 720, 0)]
        {
            let mut engine = SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
            engine.resources.rules = WorldRules::game_v8();
            engine.resources.rules.clock_offset_ticks = offset;
            if night_enabled {
                engine.resources.rules.agent.night_rest = Some(NightRestRule {
                    start_minute: 1260,
                    end_minute: 360,
                    fatigue_threshold: 0.0,
                    recovery_per_tick: 0.12,
                    wake_preempt: 75.0,
                });
            }
            let agent = engine.spawn_agent(1, 1);
            engine
                .world
                .insert(agent, (AgentState::Idle, Sleep::new(10.0, 0.0)))
                .expect("sleep");
            engine.resources.sleep_tiles.insert((1, 1), 1);
            let row = sample(&engine, "fixture", 0, 0);
            assert_eq!(row["idle_breach_actionable"], expected);
            assert_eq!(row["idle_with_breach"], 0);
            assert_eq!(row["idle_breach_unavailable"], 0);
        }
    }

    #[test]
    fn idle_actionable_falls_through_unavailable_food_and_counts_sleep() {
        let mut engine = SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
        engine.resources.rules.agent.need_fallthrough = true;
        let e = engine.spawn_agent(1, 1);
        engine
            .world
            .insert(
                e,
                (
                    AgentState::Idle,
                    Hunger::new(80.0, 0.0),
                    Thirst::new(70.0, 0.0),
                    Sleep::new(0.0, 0.0),
                ),
            )
            .expect("components");
        engine.resources.water_tiles.insert((2, 2), 1);
        let row = sample(&engine, "test", 0, 0);
        assert_eq!(row["idle_breach_actionable"], 1);
        assert_eq!(row["idle_breach_unavailable"], 0);
        engine.resources.water_tiles.clear();
        let row = sample(&engine, "test", 0, 0);
        assert_eq!(row["idle_breach_actionable"], 0);
        assert_eq!(row["idle_breach_unavailable"], 1);
        engine
            .world
            .insert(
                e,
                (
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(80.0, 0.0),
                ),
            )
            .expect("sleep");
        engine.resources.sleep_tiles.insert((1, 1), 1);
        let row = sample(&engine, "test", 0, 0);
        assert_eq!(row["idle_breach_actionable"], 1);
        assert_eq!(row["idle_with_breach"], 0); // v1 meaning stays hunger/thirst only.
    }

    #[test]
    fn legacy_helper_actionability_does_not_hide_physical_unavailability() {
        let mut engine = SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
        let entity = engine.spawn_agent(1, 1);
        engine
            .world
            .insert(entity, (AgentState::Idle, Hunger::new(80.0, 0.0)))
            .expect("components");
        let legacy = sample(&engine, "legacy", 0, 0);
        assert_eq!(legacy["idle_breach_actionable"], 1);
        assert_eq!(legacy["idle_breach_unavailable"], 1);
        engine.resources.rules.agent.need_fallthrough = true;
        let game = sample(&engine, "game", 0, 0);
        assert_eq!(game["idle_breach_actionable"], 0);
        assert_eq!(game["idle_breach_unavailable"], 1);
    }

    #[test]
    fn game_boot_shelters_are_initial_not_new_completions() {
        let mut engine = crate::scenes::game_scene(1);
        let initial = shelter_count(&engine);
        assert_eq!(initial, 3);
        for _ in 0..2 {
            let row = sample(&engine, "game", 1, initial);
            assert_eq!(row["buildings"], 5);
            assert_eq!(row["shelters"], 3);
            assert_eq!(row["shelters_completed"], 0);
            engine.tick();
        }
    }

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
