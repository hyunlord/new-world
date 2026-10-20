//! Additive schema-five observations: meals, firewood, the fire, cold and work.
use serde_json::{json, Value};
use sim_core::components::{
    Agent, AgentState, Child, Cold, Elder, Hunger, Inventory, Position, ResourceKind, TargetKind,
    Thirst,
};
use sim_engine::SimEngine;
use sim_systems::runtime::agent::hauls_home;
use sim_systems::runtime::decision::agent_decision::clock_in_window;
use sim_systems::runtime::settlement::firewood::{in_gather_window, wood_in_hand, wood_target};
use std::collections::BTreeMap;

/// Hunger or thirst at or above which a gatherer counts as critical (X11).
const CRITICAL_GATHERER: f64 = 90.0;

fn share(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// Schema-five keys; every value is finite and zero when its rule is off.
///
/// `adult_work_share`: settlement members that are neither children nor
/// elders and are seeking or consuming `GatherFood`, `GatherWood` or a
/// construction site, or hauling home, divided by those members.
/// `idle_share`: residents that are Idle and not hauling, divided by all
/// residents.
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let mut settlements: Vec<_> = r.settlements.keys().copied().collect();
    settlements.sort_unstable();
    let mut homes = BTreeMap::new();
    let (mut wood_stock, mut target, mut in_hand) = (0u64, 0.0, 0u64);
    for sid in &settlements {
        target += wood_target(r).unwrap_or(0.0);
        in_hand += wood_in_hand(&engine.world, r, *sid);
        if let Some(settlement) = r.settlements.get(sid) {
            wood_stock += u64::from(
                settlement
                    .stockpile
                    .get(&ResourceKind::Wood)
                    .copied()
                    .unwrap_or(0),
            );
            for id in &settlement.member_agents {
                homes.entry(*id).or_insert(settlement.formation_tile);
            }
        }
    }
    let fire_lit = !settlements.is_empty()
        && settlements
            .iter()
            .all(|sid| r.fires.get(sid).is_some_and(|fire| fire.lit));
    let WorkCounts {
        cold,
        critical,
        idle,
        adults,
        working,
        alive,
    } = work_counts(engine, &homes);
    let t = r.tallies;
    json!({"meals_from_store_total": t.meals_from_store, "ground_meals_total": t.ground_meals,
        "wood_stock": wood_stock, "wood_target": target, "wood_in_hand": in_hand,
        "fire_lit": u64::from(fire_lit), "cold_members": cold,
        "cold_member_hours_total": t.cold_member_hours, "wood_trips_total": t.wood_trips,
        "wood_gathered_total": t.wood_gathered,
        "deadwood_total": r.wood_tiles.values().map(|v| u64::from(*v)).sum::<u64>(),
        "gather_preempted_total": t.gather_preempted, "gather_seekers_critical": critical,
        "adult_work_share": share(working, adults), "idle_share": share(idle, alive)})
}

/// Counts behind the schema-five work and cold keys, from residents in
/// AgentId order: `Cold` markers, critical gatherers, idle residents not
/// hauling, settlement adults (neither child nor elder), those adults at work
/// (the numerator of `adult_work_share`) and all residents.
pub(crate) struct WorkCounts {
    pub(crate) cold: u64,
    pub(crate) critical: u64,
    pub(crate) idle: u64,
    pub(crate) adults: u64,
    pub(crate) working: u64,
    pub(crate) alive: u64,
}

/// [`WorkCounts`] of `engine`; `homes` maps each member to its settlement's
/// formation tile (first settlement in id order).
pub(crate) fn work_counts(engine: &SimEngine, homes: &BTreeMap<u64, (u32, u32)>) -> WorkCounts {
    let r = &engine.resources;
    let forage = r
        .rules
        .agent
        .forage
        .filter(|_| r.rules.society.stock_target.is_some());
    let firewood = r.rules.society.firewood;
    let forage_window =
        forage.is_some_and(|rule| clock_in_window(r, rule.start_minute, rule.end_minute));
    let wood_window = in_gather_window(r);
    let mut query = engine.world.query::<(
        &Agent,
        &Position,
        Option<&AgentState>,
        Option<&Inventory>,
        Option<&Child>,
        Option<&Elder>,
        Option<&Cold>,
        Option<&Hunger>,
        Option<&Thirst>,
    )>();
    let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
    rows.sort_by_key(|row| row.0.id);
    let alive = rows.len() as u64;
    let (mut cold, mut critical, mut idle, mut adults, mut working) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    for (agent, pos, state, inventory, child, elder, marker, hunger, thirst) in rows {
        cold += u64::from(marker.is_some());
        let state = state.copied().unwrap_or_default();
        let gathering = matches!(
            state,
            AgentState::Seeking {
                target: TargetKind::GatherFood | TargetKind::GatherWood
            }
        );
        let needy = hunger.is_some_and(|h| f64::from(h.value) >= CRITICAL_GATHERER)
            || thirst.is_some_and(|t| t.value >= CRITICAL_GATHERER);
        critical += u64::from(gathering && needy);
        let home = homes.get(&agent.id).copied();
        let hauling = state == AgentState::Idle
            && child.is_none()
            && home.is_some_and(|(x, y)| {
                let carried = |kind| inventory.map_or(0, |inv| inv.get(kind));
                let wood = if firewood.is_some() {
                    carried(ResourceKind::Wood)
                } else {
                    0
                };
                hauls_home(
                    forage.as_ref(),
                    firewood.as_ref(),
                    carried(ResourceKind::Food),
                    wood,
                    forage_window,
                    wood_window,
                    pos.x.abs_diff(x).max(pos.y.abs_diff(y)),
                )
            });
        idle += u64::from(state == AgentState::Idle && !hauling);
        if home.is_some() && child.is_none() && elder.is_none() {
            adults += 1;
            let at_work = matches!(
                state.target(),
                Some(
                    TargetKind::GatherFood | TargetKind::GatherWood | TargetKind::ConstructionSite
                )
            );
            working += u64::from(at_work || hauling);
        }
    }
    WorkCounts {
        cold,
        critical,
        idle,
        adults,
        working,
        alive,
    }
}

/// Settlement members mapped to their formation tile (first settlement in id order).
pub(crate) fn member_homes(engine: &SimEngine) -> BTreeMap<u64, (u32, u32)> {
    let r = &engine.resources;
    let mut settlements: Vec<_> = r.settlements.keys().copied().collect();
    settlements.sort_unstable();
    let mut homes = BTreeMap::new();
    for sid in &settlements {
        if let Some(settlement) = r.settlements.get(sid) {
            for id in &settlement.member_agents {
                homes.entry(*id).or_insert(settlement.formation_tile);
            }
        }
    }
    homes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_world_reports_zero_meal_and_firewood_tallies_at_tick_zero() {
        let engine = crate::scenes::baseline_scene(3);
        let row = metrics(&engine);
        for key in [
            "meals_from_store_total",
            "ground_meals_total",
            "cold_member_hours_total",
            "wood_trips_total",
            "wood_gathered_total",
            "gather_preempted_total",
        ] {
            assert_eq!(row[key], 0, "{key}");
        }
        // Tick zero: nobody belongs to a settlement yet and everybody is idle.
        assert_eq!(row["adult_work_share"], 0.0);
        assert_eq!(row["idle_share"], 1.0);
        assert!(row
            .as_object()
            .expect("object")
            .values()
            .all(|v| !v.is_null()));
    }

    #[test]
    fn baseline_world_reports_its_deadwood() {
        let engine = crate::scenes::baseline_scene(3);
        let row = metrics(&engine);
        // Tick 0: every placed deadwood spot (camp ring and wild regions) holds
        // the world's drawn `deadwood_capacity`.
        let report = &engine.resources.worldgen_report;
        let capacity = sim_core::variety::WorldVariety::draw(3).deadwood_capacity;
        let spots = report.deadwood_spots_placed + report.region_deadwood_placed;
        assert!(
            spots >= report.deadwood_spots_requested / 2,
            "{spots} spots"
        );
        assert_eq!(
            row["deadwood_total"],
            u64::from(spots) * u64::from(capacity)
        );
    }
}
