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
//! `adult_work_share`. `wet_burn_wood_total` and `area_floor_wood_total` are
//! the cumulative Pass 12 tallies (rain burning wet wood, the settled-region
//! winter-fall floor).
//!
//! Per-seed block ([`seed_block`]): every daily fission check that reached the
//! hazard draw, the first day each settlement reached it, members of every
//! settlement at 18:00 of each winter's first day with its exposure deaths in
//! the winter window, and the worldgen fill counts.
use serde_json::{json, Value};
use sim_core::components::{Agent, AgentState, Child, Elder, Founding, Inventory, Position, ResourceKind, SettlementId, TargetKind};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};
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
            Some(TargetKind::Activity) => "activity",
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
    row.insert("wet_burn_wood_total".into(), json!(r.tallies.wet_burn_wood));
    row.insert("area_floor_wood_total".into(), json!(r.tallies.area_floor_wood));
    Value::Object(row)
}

/// Members of every settlement now (for the winter-start samples), founders
/// still walking with a fission group left out (review round 1, L5: they are
/// not at the parent's fire).
pub(super) fn members_now(engine: &SimEngine) -> BTreeMap<SettlementId, u32> {
    let walking: BTreeSet<u64> = engine.world.query::<(&Agent, &Founding)>().iter().map(|(_, (a, _))| a.id).collect();
    engine
        .resources
        .settlements
        .iter()
        .map(|(sid, s)| (*sid, s.member_agents.iter().filter(|id| !walking.contains(id)).count() as u32))
        .collect()
}

/// The per-seed schema-twelve block.
///
/// `fission_checks`: every logged hazard check as [tick, settlement, members,
/// capacity, food, g, q, s, h, draw, split (0/1), site_free (0/1)].
/// `first_draw_day`: per settlement, the zero-based display day
/// ((tick + offset) / ticks_per_day) of its first logged check.
/// `settlement_winters`: for each winter whose window [day D 18:00, day D+7
/// 07:00) lies inside the run, every settlement with members at day D 18:00,
/// founded inside the window or with an exposure death in it:
/// `members_at_winter_start` (founders still walking left out; a settlement
/// founded inside the window counts its members at the first hourly sample
/// after it formed), `exposure_deaths`, `founded_inside`.
pub(super) fn seed_block(
    engine: &SimEngine,
    winter_members: &BTreeMap<u64, BTreeMap<SettlementId, u32>>,
    formed_members: &BTreeMap<SettlementId, u32>,
    exposure: &[super::pass11::ExposureDeath],
    run_days: u64,
) -> Value {
    let r = &engine.resources;
    let tpd = r.ticks_per_day.max(1);
    let offset = r.rules.clock_offset_ticks;
    let checks: Vec<Value> = r
        .fission_checks
        .iter()
        .map(|c| json!([c.tick, c.settlement, c.members, c.capacity, c.food, c.g, c.q, c.s, c.h, c.draw, u8::from(c.split), u8::from(c.site_free)]))
        .collect();
    let mut first: BTreeMap<SettlementId, u64> = BTreeMap::new();
    for c in &r.fission_checks {
        first.entry(c.settlement).or_insert((c.tick + offset) / tpd);
    }
    let founders: BTreeMap<SettlementId, (u64, usize)> = r
        .fission_log
        .iter()
        .filter_map(|f| Some((f.daughter?, (f.founded_tick?, f.members.len()))))
        .collect();
    let mut winters = Vec::new();
    for d in super::pass10::WINTER_FIRST_DAYS {
        let (Some(start), Some(end)) = (
            super::args::clock_tick(d, 1080, tpd, offset),
            super::args::clock_tick(d + 7, 420, tpd, offset),
        ) else {
            continue;
        };
        if end > run_days * tpd {
            continue;
        }
        let at_start = winter_members.get(&d).cloned().unwrap_or_default();
        let deaths: BTreeMap<SettlementId, u64> = exposure
            .iter()
            .filter(|(tick, ..)| (start..end).contains(tick))
            .filter_map(|(_, _, sid)| *sid)
            .fold(BTreeMap::new(), |mut acc, sid| {
                *acc.entry(sid).or_insert(0) += 1;
                acc
            });
        let inside: BTreeSet<SettlementId> =
            founders.iter().filter(|(_, (tick, _))| (start..end).contains(tick)).map(|(sid, _)| *sid).collect();
        let ids: BTreeSet<SettlementId> = at_start
            .iter()
            .filter(|(_, m)| **m > 0)
            .map(|(sid, _)| *sid)
            .chain(inside.iter().copied())
            .chain(deaths.keys().copied())
            .collect();
        let rows: Vec<Value> = ids
            .iter()
            .map(|sid| {
                let members = if inside.contains(sid) {
                    // Review round 1, L5: the founders alive when the daughter
                    // formed (its members at the first hourly sample after),
                    // else the departing group's size.
                    formed_members
                        .get(sid)
                        .map_or_else(|| founders.get(sid).map_or(0, |(_, n)| *n as u64), |m| u64::from(*m))
                } else {
                    u64::from(at_start.get(sid).copied().unwrap_or(0))
                };
                json!({"settlement": sid, "members_at_winter_start": members,
                    "exposure_deaths": deaths.get(sid).copied().unwrap_or(0), "founded_inside": inside.contains(sid)})
            })
            .collect();
        winters.push(json!({"first_day": d, "start_tick": start, "end_tick": end, "settlements": rows}));
    }
    let w = r.worldgen_report;
    let t = r.tallies;
    json!({"fission_checks": checks, "first_draw_day": first.iter().map(|(sid, day)| json!([sid, day])).collect::<Vec<_>>(),
        "settlement_winters": winters, "region_fill_bushes": w.region_fill_bushes, "region_fill_deadwood": w.region_fill_deadwood,
        "wet_burn_wood": t.wet_burn_wood, "area_floor_wood": t.area_floor_wood, "winter_wood_lost_settled": t.winter_wood_lost_settled,
        "winter_wood_lost": t.winter_wood_lost})
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
