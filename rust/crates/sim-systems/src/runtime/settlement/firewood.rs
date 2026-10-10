//! Revitalize Pass 5 — firewood accounting shared by decisions, the fire and probes.
//!
//! Pure reads: none of these helpers mutates simulation state or draws RNG.
//! Without `SocietyRules::firewood` every helper reports nothing (`None` / 0).

use crate::runtime::decision::agent_decision::clock_in_window;
use crate::runtime::resource_regen::{season_at, season_at_tick};
use hecs::World;
use sim_core::components::{Agent, Inventory, ResourceKind, SettlementId};
use sim_core::rules::FirewoodRule;
use sim_engine::SimResources;

/// Minutes in a day; windows are minute-of-day.
const MINUTES_PER_DAY: u32 = 1440;

/// Winter index of `SeasonRule` seasons and of `burn_per_hour`.
const WINTER: usize = 3;

/// Whole hours in the half-open, possibly midnight-wrapping burn window
/// (13 for 18:00–07:00).
pub fn burn_hours(rule: &FirewoodRule) -> u32 {
    let start = rule.burn_start_minute % MINUTES_PER_DAY;
    let end = rule.burn_end_minute % MINUTES_PER_DAY;
    (end + MINUTES_PER_DAY - start) % MINUTES_PER_DAY / 60
}

/// Index into `burn_per_hour`: the current season, or 0 without seasons.
pub fn burn_season(resources: &SimResources) -> usize {
    season_at(resources).map_or(0, |(season, _)| season.min(3))
}

/// Index into `burn_per_hour` for the night in progress (Pass 6). Under
/// `night_season_from_evening`, before `burn_end_minute` the night belongs to
/// the evening it began, so the season is read at `current_tick - ticks_per_day`;
/// otherwise (and always in Pass 5) it is [`burn_season`].
pub fn night_season(resources: &SimResources, rule: &FirewoodRule) -> usize {
    if rule.night_season_from_evening && resources.ticks_per_day != 0 {
        let minute = resources.current_tick.saturating_add(resources.rules.clock_offset_ticks) % resources.ticks_per_day;
        if minute < u64::from(rule.burn_end_minute % MINUTES_PER_DAY) {
            let evening = resources.current_tick.saturating_sub(resources.ticks_per_day);
            return season_at_tick(resources, evening).map_or(0, |(season, _)| season.min(3));
        }
    }
    burn_season(resources)
}

/// Urgent mode (Pass 6): `urgent_start_minute` is set and `in_hand` wood is
/// below tonight's burn, `burn_per_day(night_season)`. False without the rule.
pub fn urgent_with(resources: &SimResources, in_hand: u64) -> bool {
    resources.rules.society.firewood.is_some_and(|rule| {
        rule.urgent_start_minute.is_some()
            && (in_hand as f64) < burn_per_day(&rule, night_season(resources, &rule))
    })
}

/// Whether settlement `sid` is in urgent mode now: [`urgent_with`] of its
/// [`wood_in_hand`].
pub fn is_urgent(world: &World, resources: &SimResources, sid: SettlementId) -> bool {
    resources.rules.society.firewood.is_some_and(|rule| rule.urgent_start_minute.is_some())
        && resources.settlements.contains_key(&sid)
        && urgent_with(resources, wood_in_hand(world, resources, sid))
}

/// Wood burned per night in season `season`: `burn_per_hour[season] x burn hours`.
pub fn burn_per_day(rule: &FirewoodRule, season: usize) -> f64 {
    f64::from(rule.burn_per_hour[season.min(3)]) * f64::from(burn_hours(rule))
}

/// Wood target of every settlement: `base(s) x burn_per_day(s) + prep(s)` for
/// the current season `s` (Pass 6 section 7.2). `base(s)` is
/// `winter_base_days` in winter when set, else `base_days`. With seasons on and
/// winter preparation enabled, `prep(s) = prep_days x burn_per_day(winter)` in
/// autumn, and in summer too under `winter_prep_in_summer`; `prep_days` is
/// `winter_prep_days`, else `days_per_season` (Pass 5). `None` without the
/// firewood rule.
pub fn wood_target(resources: &SimResources) -> Option<f64> {
    let rule = resources.rules.society.firewood?;
    let season = burn_season(resources);
    let base = if season == WINTER { rule.winter_base_days.unwrap_or(rule.base_days) } else { rule.base_days };
    let mut target = base * burn_per_day(&rule, season);
    if let (true, Some(seasons), Some((current, _))) =
        (rule.winter_prep, resources.rules.seasons, season_at(resources))
    {
        if current == 2 || (current == 1 && rule.winter_prep_in_summer) {
            let prep_days = rule.winter_prep_days.unwrap_or(f64::from(seasons.days_per_season));
            target += prep_days * burn_per_day(&rule, WINTER);
        }
    }
    Some(target)
}

/// Stockpile wood plus wood carried by live members, summed in AgentId order.
pub fn wood_in_hand(world: &World, resources: &SimResources, sid: SettlementId) -> u64 {
    let Some(settlement) = resources.settlements.get(&sid) else {
        return 0;
    };
    let mut carried: Vec<_> = world
        .query::<(&Agent, &Inventory)>()
        .iter()
        .filter(|(_, (agent, _))| settlement.member_agents.contains(&agent.id))
        .map(|(_, (agent, inventory))| (agent.id, inventory.get(ResourceKind::Wood)))
        .collect();
    carried.sort_by_key(|(id, _)| *id);
    u64::from(settlement.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0))
        + carried.iter().map(|(_, wood)| u64::from(*wood)).sum::<u64>()
}

/// Whether the clock lies inside the firewood gathering window.
pub fn in_gather_window(resources: &SimResources) -> bool {
    resources.rules.society.firewood.is_some_and(|rule| {
        clock_in_window(resources, rule.gather_start_minute, rule.gather_end_minute)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::Settlement;
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{SeasonRule, WorldRules};

    fn rule() -> FirewoodRule {
        FirewoodRule {
            gather_start_minute: 600,
            gather_end_minute: 1020,
            wood_radius: 18,
            haul_threshold: 2,
            max_gatherers_per_spot: 1,
            regen_permille: 500,
            base_days: 1.5,
            winter_prep: true,
            burn_start_minute: 1080,
            burn_end_minute: 420,
            burn_per_hour: [2, 1, 2, 3],
            cold_start_minute: 1320,
            cold_end_minute: 360,
            cold_hunger_permille: 1500,
            winter_prep_days: None,
            winter_prep_in_summer: true,
            winter_base_days: None,
            winter_regen_units_per_day: None,
            winter_deadwood_cap: None,
            night_season_from_evening: false,
            urgent_start_minute: None,
        }
    }

    #[test]
    fn burn_window_is_thirteen_hours_and_burn_per_day_scales() {
        let r = rule();
        assert_eq!(burn_hours(&r), 13);
        assert_eq!(burn_per_day(&r, 0), 26.0);
        assert_eq!(burn_per_day(&r, 3), 39.0);
    }

    #[test]
    fn target_adds_a_winter_of_burn_in_summer_and_autumn_only() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        assert_eq!(wood_target(&resources), None);
        resources.rules = WorldRules::game_v10();
        resources.rules.society.firewood = Some(rule());
        resources.rules.seasons = Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] });
        // Display day d at 08:00: tick = d x 1440 (480-tick clock offset).
        let at_day = |r: &mut SimResources, d: u64| r.current_tick = d * 1440;
        at_day(&mut resources, 0);
        assert_eq!(wood_target(&resources), Some(1.5 * 26.0));
        at_day(&mut resources, 7);
        assert_eq!(wood_target(&resources), Some(1.5 * 13.0 + 7.0 * 39.0));
        at_day(&mut resources, 14);
        assert_eq!(wood_target(&resources), Some(1.5 * 26.0 + 7.0 * 39.0));
        at_day(&mut resources, 21);
        assert_eq!(wood_target(&resources), Some(1.5 * 39.0));
        resources.rules.seasons = None;
        assert_eq!(wood_target(&resources), Some(1.5 * 26.0));
    }

    #[test]
    fn night_season_belongs_to_the_evening_only_when_enabled() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules = WorldRules::game_v10();
        resources.rules.seasons = Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] });
        let mut r = rule();
        // 02:00 of display day 28 (the first spring day of year two) after the
        // last winter evening (display day 27): tick = 28 x 1440 + 120 - 480.
        resources.current_tick = 28 * 1440 + 120 - 480;
        assert_eq!(night_season(&resources, &r), 0);
        r.night_season_from_evening = true;
        assert_eq!(night_season(&resources, &r), 3);
        // 07:00 ends the burn window: the current season again.
        resources.current_tick = 28 * 1440 + 420 - 480;
        assert_eq!(night_season(&resources, &r), 0);
        // 22:00 on the last winter evening: winter either way.
        resources.current_tick = 27 * 1440 + 1320 - 480;
        assert_eq!(night_season(&resources, &r), 3);
        r.night_season_from_evening = false;
        assert_eq!(night_season(&resources, &r), 3);
    }

    #[test]
    fn urgent_needs_the_rule_and_less_than_tonights_burn() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules = WorldRules::game_v10();
        assert!(!urgent_with(&resources, 0));
        let mut r = rule();
        resources.rules.society.firewood = Some(r);
        assert!(!urgent_with(&resources, 0), "urgent mode is off without a start minute");
        r.urgent_start_minute = Some(420);
        resources.rules.society.firewood = Some(r);
        resources.rules.seasons = None;
        // Without seasons tonight burns 2 x 13 = 26.
        assert!(urgent_with(&resources, 25));
        assert!(!urgent_with(&resources, 26));
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.store(ResourceKind::Wood, 25);
        resources.settlements.insert(1, settlement);
        let world = World::new();
        assert!(is_urgent(&world, &resources, 1));
        assert!(!is_urgent(&world, &resources, 2));
    }

    #[test]
    fn wood_in_hand_counts_stockpile_and_members_only() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        let mut world = World::new();
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.add_member_agent(1);
        settlement.store(ResourceKind::Wood, 5);
        resources.settlements.insert(1, settlement);
        for (id, wood) in [(1u64, 2u32), (2, 7)] {
            let mut inventory = Inventory::default();
            inventory.add(ResourceKind::Wood, wood);
            world.spawn((Agent { id }, inventory));
        }
        assert_eq!(wood_in_hand(&world, &resources, 1), 7);
        assert_eq!(wood_in_hand(&world, &resources, 9), 0);
    }
}
