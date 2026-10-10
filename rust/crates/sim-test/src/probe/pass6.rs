//! Additive schema-six observations: wood work, urgent mode, winter regrowth and elders.
use serde_json::{json, Value};
use sim_core::components::{Agent, AgentState, BuildingKind, Elder, Position, TargetKind};
use sim_engine::SimEngine;
use sim_systems::runtime::settlement::firewood::is_urgent;
use std::collections::BTreeMap;

/// Schema-six keys; every value is finite and zero when its rule is off.
///
/// `wood_gatherers_now`: settlement members seeking or consuming `GatherWood`.
/// `urgent_wood`: 1 when any settlement is in urgent mode (a state flag, read
/// at any hour). `idle_elders`: elders that are Idle (or have no state);
/// `idle_elders_near_hearth`: those within Chebyshev 3 of their settlement's
/// formation tile when that tile holds the hearth (as `idle_adults_near_hearth`).
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let mut settlements: Vec<_> = r.settlements.keys().copied().collect();
    settlements.sort_unstable();
    let mut homes = BTreeMap::new();
    let mut urgent = false;
    for sid in &settlements {
        urgent |= is_urgent(&engine.world, r, *sid);
        if let Some(settlement) = r.settlements.get(sid) {
            for id in &settlement.member_agents {
                homes.entry(*id).or_insert(settlement.formation_tile);
            }
        }
    }
    let mut query = engine.world.query::<(&Agent, &Position, Option<&AgentState>, Option<&Elder>)>();
    let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
    rows.sort_by_key(|row| row.0.id);
    let (mut gatherers, mut idle_elders, mut near) = (0u64, 0u64, 0u64);
    for (agent, pos, state, elder) in rows {
        let home = homes.get(&agent.id).copied();
        gatherers += u64::from(
            home.is_some()
                && matches!(
                    state,
                    Some(AgentState::Seeking { target: TargetKind::GatherWood }
                        | AgentState::Consuming { target: TargetKind::GatherWood })
                ),
        );
        if elder.is_some() && state.is_none_or(|state| *state == AgentState::Idle) {
            idle_elders += 1;
            near += u64::from(home.is_some_and(|(x, y)| {
                r.building_kinds.get(&(x, y)) == Some(&BuildingKind::Hearth)
                    && pos.x.abs_diff(x).max(pos.y.abs_diff(y)) <= 3
            }));
        }
    }
    let t = r.tallies;
    json!({"wood_gatherers_now": gatherers, "urgent_wood": u64::from(urgent),
        "urgent_wood_trips_total": t.urgent_wood_trips, "winter_wood_lost_total": t.winter_wood_lost,
        "idle_elders": idle_elders, "idle_elders_near_hearth": near})
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{ResourceKind, Settlement};
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;

    #[test]
    fn v11_world_reports_neutral_pass6_values() {
        let engine = crate::scenes::game_v11_scene(3);
        let row = metrics(&engine);
        assert_eq!(row.as_object().expect("object").len(), 6);
        for key in ["wood_gatherers_now", "urgent_wood", "urgent_wood_trips_total", "winter_wood_lost_total",
            "idle_elders", "idle_elders_near_hearth"] {
            assert_eq!(row[key], 0, "{key}");
        }
    }

    #[test]
    fn gatherers_elders_and_urgent_have_exact_values() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = WorldRules::game_v11();
        if let Some(rule) = engine.resources.rules.society.firewood.as_mut() {
            rule.urgent_start_minute = Some(420);
        }
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (10, 10);
        settlement.store(ResourceKind::Wood, 3);
        let states = [
            (AgentState::Seeking { target: TargetKind::GatherWood }, false, (14, 14), true),
            (AgentState::Consuming { target: TargetKind::GatherWood }, false, (15, 15), true),
            (AgentState::Seeking { target: TargetKind::GatherWood }, false, (16, 16), false),
            (AgentState::Idle, true, (12, 13), true),
            (AgentState::Idle, true, (14, 10), true),
            (AgentState::Seeking { target: TargetKind::Water }, true, (10, 10), true),
        ];
        for (state, elder, (x, y), member) in states {
            let entity = engine.spawn_agent(x, y);
            engine.world.insert_one(entity, state).expect("state");
            if elder {
                engine.world.insert_one(entity, Elder).expect("elder");
            }
            let id = engine.world.get::<&Agent>(entity).expect("agent").id;
            if member {
                settlement.add_member_agent(id);
            }
        }
        engine.resources.settlements.insert(1, settlement);
        engine.resources.building_kinds.insert((10, 10), BuildingKind::Hearth);
        engine.resources.tallies.urgent_wood_trips = 4;
        engine.resources.tallies.winter_wood_lost = 2;
        let row = metrics(&engine);
        for (key, want) in [
            ("wood_gatherers_now", 2),
            ("urgent_wood", 1),
            ("urgent_wood_trips_total", 4),
            ("winter_wood_lost_total", 2),
            ("idle_elders", 2),
            ("idle_elders_near_hearth", 1),
        ] {
            assert_eq!(row[key], want, "{key}");
        }
        // Tonight's burn (2 x 13 = 26 at tick 0, spring) in hand ends urgent mode.
        engine.resources.settlements.get_mut(&1).expect("village").store(ResourceKind::Wood, 23);
        assert_eq!(metrics(&engine)["urgent_wood"], 0);
    }
}
