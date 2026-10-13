//! Additive schema-twelve observations (Pass 12).
//!
//! Row keys: what the settlement adults are doing (the J6 bisection of Part 1
//! asks which activity took the afternoon). Every adult member (neither child
//! nor elder) is counted once, in the first class that applies:
//! `adult_act_haul` (Idle and hauling home, as `adult_work_share` counts it),
//! `adult_act_idle` (Idle otherwise), then by the target of its Seeking or
//! Consuming state: `forage` (GatherFood), `wood` (GatherWood), `build`
//! (ConstructionSite), `eat` (Food), `drink` (Water), `rest` (Sleep),
//! `founding` (an Agent target while walking with a fission group), `social`
//! (any other Agent target). `adult_act_walking` counts the adults in a
//! Seeking state (on their way to any target) and `adult_act_total` all
//! adult members; `adult_act_forage + wood + build + haul` is the numerator of
//! `adult_work_share`.
use serde_json::{json, Value};
use sim_core::components::{Agent, AgentState, Child, Elder, Founding, Inventory, Position, ResourceKind, TargetKind};
use sim_engine::SimEngine;
use sim_systems::runtime::agent::hauls_home;
use sim_systems::runtime::decision::agent_decision::clock_in_window;
use sim_systems::runtime::settlement::firewood::in_gather_window;

/// The activity classes, in report order.
pub(super) const ACTIVITIES: [&str; 10] = ["forage", "wood", "build", "haul", "idle", "eat", "drink", "rest", "social", "founding"];

/// Schema-twelve row keys (see the module docs).
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let homes = super::pass5::member_homes(engine);
    let forage = r.rules.agent.forage.filter(|_| r.rules.society.stock_target.is_some());
    let firewood = r.rules.society.firewood;
    let forage_window = forage.is_some_and(|rule| clock_in_window(r, rule.start_minute, rule.end_minute));
    let wood_window = in_gather_window(r);
    let mut counts = [0u64; ACTIVITIES.len()];
    let (mut walking, mut total) = (0u64, 0u64);
    let mut query = engine.world.query::<(
        &Agent,
        &Position,
        Option<&AgentState>,
        Option<&Inventory>,
        Option<&Child>,
        Option<&Elder>,
        Option<&Founding>,
    )>();
    let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
    rows.sort_by_key(|row| row.0.id);
    for (agent, pos, state, inventory, child, elder, founding) in rows {
        let Some(&(hx, hy)) = homes.get(&agent.id) else { continue };
        if child.is_some() || elder.is_some() {
            continue;
        }
        total += 1;
        let state = state.copied().unwrap_or_default();
        walking += u64::from(matches!(state, AgentState::Seeking { .. }));
        let class = match state.target() {
            None => {
                let carried = |kind| inventory.map_or(0, |inv| inv.get(kind));
                let wood = if firewood.is_some() { carried(ResourceKind::Wood) } else { 0 };
                let hauling = hauls_home(forage.as_ref(), firewood.as_ref(), carried(ResourceKind::Food), wood,
                    forage_window, wood_window, pos.x.abs_diff(hx).max(pos.y.abs_diff(hy)));
                if hauling { "haul" } else { "idle" }
            }
            Some(TargetKind::GatherFood) => "forage",
            Some(TargetKind::GatherWood) => "wood",
            Some(TargetKind::ConstructionSite) => "build",
            Some(TargetKind::Food) => "eat",
            Some(TargetKind::Water) => "drink",
            Some(TargetKind::Sleep) => "rest",
            Some(TargetKind::Agent(_)) if founding.is_some() => "founding",
            Some(TargetKind::Agent(_)) => "social",
        };
        if let Some(i) = ACTIVITIES.iter().position(|a| *a == class) {
            counts[i] += 1;
        }
    }
    let mut row = serde_json::Map::new();
    for (name, count) in ACTIVITIES.iter().zip(counts) {
        row.insert(format!("adult_act_{name}"), json!(count));
    }
    row.insert("adult_act_walking".into(), json!(walking));
    row.insert("adult_act_total".into(), json!(total));
    Value::Object(row)
}

#[cfg(test)]
mod tests {
    #[test]
    fn work_classes_add_up_to_the_work_share_numerator() {
        let mut engine = crate::scenes::game_v16_scene(3);
        for _ in 0..(1440 + 420) {
            engine.tick();
        }
        let row = super::metrics(&engine);
        let all = crate::probe::metrics::sample(&engine, "game_v16", 3, 0);
        let get = |k: &str| row[k].as_u64().expect(k);
        let total = get("adult_act_total");
        let classes: u64 = super::ACTIVITIES.iter().map(|a| get(&format!("adult_act_{a}"))).sum();
        assert_eq!(classes, total);
        let work = get("adult_act_forage") + get("adult_act_wood") + get("adult_act_build") + get("adult_act_haul");
        let share = all["adult_work_share"].as_f64().expect("share");
        assert!(total > 0);
        assert!((share - work as f64 / total as f64).abs() < 1e-12, "{share} vs {work}/{total}");
    }
}
