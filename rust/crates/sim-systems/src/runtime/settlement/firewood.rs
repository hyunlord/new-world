//! Revitalize Pass 5 — firewood accounting shared by decisions, the fire and probes.
//!
//! Pure reads: none of these helpers mutates simulation state or draws RNG.
//! Without `SocietyRules::firewood` every helper reports nothing (`None` / 0).

use crate::runtime::agent::MovementRng;
use crate::runtime::decision::agent_decision::clock_in_window;
use crate::runtime::resource_regen::{day_index_at, season_at, season_at_tick};
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
        let minute = resources
            .current_tick
            .saturating_add(resources.rules.clock_offset_ticks)
            % resources.ticks_per_day;
        if minute < u64::from(rule.burn_end_minute % MINUTES_PER_DAY) {
            let evening = resources
                .current_tick
                .saturating_sub(resources.ticks_per_day);
            return season_at_tick(resources, evening).map_or(0, |(season, _)| season.min(3));
        }
    }
    burn_season(resources)
}

/// Salt of the winter severity draw (Pass 7): "WINTERSE" in ASCII.
const SEVERITY_SALT: u64 = 0x5749_4E54_4552_5345;

/// One splitmix64 step of `x` (the first output of a stream seeded with `x`).
fn mix(x: u64) -> u64 {
    MovementRng::new(x).next_u64()
}

/// The severity draw of (world seed, year) before reduction (Pass 7):
/// `mix(mix(world_seed ^ SALT) ^ year)`. Pure.
pub fn severity_hash(world_seed: u64, year: u64) -> u64 {
    mix(mix(world_seed ^ SEVERITY_SALT) ^ year)
}

/// Severity of the winter of `year` (Pass 7): 0 mild, 1 normal, 2 harsh.
///
/// `u = severity_hash(world_seed, year) % 1000` against the cumulative
/// `winter_severity_permille` weights. Pure: no state, no stream. The double
/// mix keeps seed `s`, year `y` apart from seed `s ^ y`, year 0. Without
/// weights every winter is normal (1).
pub fn winter_severity(resources: &SimResources, rule: &FirewoodRule, year: u64) -> u8 {
    let Some([w0, w1, _]) = rule.winter_severity_permille else {
        return 1;
    };
    let u = severity_hash(resources.world_seed, year) % 1000;
    if u < u64::from(w0) {
        0
    } else if u < u64::from(w0) + u64::from(w1) {
        1
    } else {
        2
    }
}

/// Year of simulation tick `tick`: display day / (4 x days_per_season); 0
/// without seasons.
pub fn year_at_tick(resources: &SimResources, tick: u64) -> u64 {
    resources.rules.seasons.map_or(0, |seasons| {
        day_index_at(resources, tick)
            .checked_div(4 * u64::from(seasons.days_per_season))
            .unwrap_or(0)
    })
}

/// The tick whose season and year a night belongs to (Pass 7): under
/// `night_season_from_evening`, before `burn_end_minute` the same minute of the
/// previous day (the evening the night began); otherwise `tick` itself. The
/// booking of [`night_season`].
pub fn night_tick_at(resources: &SimResources, rule: &FirewoodRule, tick: u64) -> u64 {
    if rule.night_season_from_evening && resources.ticks_per_day != 0 {
        let minute =
            tick.saturating_add(resources.rules.clock_offset_ticks) % resources.ticks_per_day;
        if minute < u64::from(rule.burn_end_minute % MINUTES_PER_DAY) {
            return tick.saturating_sub(resources.ticks_per_day);
        }
    }
    tick
}

/// Severity level of the winter holding `tick` (Pass 7), `None` outside a
/// winter, without seasons or without weights. No night booking: callers
/// pass the booked tick.
pub fn severity_level_at(resources: &SimResources, rule: &FirewoodRule, tick: u64) -> Option<u8> {
    rule.winter_severity_permille?;
    let (season, _) = season_at_tick(resources, tick)?;
    (season == WINTER).then(|| winter_severity(resources, rule, year_at_tick(resources, tick)))
}

/// Display value of the winter severity at completed-or-system tick `tick`
/// (Pass 7 probe schema 7, FFI and recorder): 0 mild, 1 normal, 2 harsh, -1
/// outside a winter or without the rule or weights. Samples from 00:00 to the
/// burn end follow the night booking ([`night_tick_at`]).
pub fn winter_severity_at(resources: &SimResources, tick: u64) -> i64 {
    resources.rules.society.firewood.map_or(-1, |rule| {
        severity_level_at(resources, &rule, night_tick_at(resources, &rule, tick))
            .map_or(-1, i64::from)
    })
}

/// Wood burned per hour in the season holding `tick` (Pass 7): in a winter of
/// drawn severity `winter_severity_burn_per_hour[level]`, otherwise
/// `burn_per_hour[season]` (index 0 without seasons). The fire, the winter
/// target and urgency all read it, so they never disagree; callers pass the
/// booked tick (the evening's for a night, the current one for the target).
pub fn winter_burn_per_hour(resources: &SimResources, rule: &FirewoodRule, tick: u64) -> u16 {
    match severity_level_at(resources, rule, tick) {
        Some(level) => rule.winter_severity_burn_per_hour[usize::from(level).min(2)],
        None => {
            rule.burn_per_hour
                [season_at_tick(resources, tick).map_or(0, |(season, _)| season.min(3))]
        }
    }
}

/// Salt of the night burn draw (Pass 8): "NIGHTBUR" in ASCII.
const NIGHT_BURN_SALT: u64 = 0x4E49_4748_5442_5552;

/// Salt of the winter fall draw (Pass 8): "FALLSPRD" in ASCII.
const FALL_SPREAD_SALT: u64 = 0x4641_4C4C_5350_5244;

/// Night-by-night change of the winter burn per hour (Pass 8): with
/// `u = mix(mix(world_seed ^ SALT) ^ evening_day) % 1000` against the
/// cumulative `winter_night_burn_permille` weights, `-step`, 0 or `+step`.
/// `evening_day` is the display day index of the evening the night began.
/// Pure: no state, no stream. 0 unless that evening lies in a winter of drawn
/// severity and night weights are set.
pub fn night_burn_delta(resources: &SimResources, rule: &FirewoodRule, evening_day: u64) -> i16 {
    let Some([w0, w1, _]) = rule.winter_night_burn_permille else {
        return 0;
    };
    if !in_severity_winter(resources, rule, evening_day) {
        return 0;
    }
    let step = i16::try_from(rule.winter_night_burn_step).unwrap_or(i16::MAX);
    let u = mix(mix(resources.world_seed ^ NIGHT_BURN_SALT) ^ evening_day) % 1000;
    if u < u64::from(w0) {
        -step
    } else if u < u64::from(w0) + u64::from(w1) {
        0
    } else {
        step
    }
}

/// Day-by-day change of the winter deadwood fall (Pass 8): with
/// `u = mix(mix(world_seed ^ SALT) ^ day)`, `(u % (2 x spread + 1)) - spread`.
/// Pure. 0 unless display day `day` lies in a winter of drawn severity and
/// `winter_fall_spread > 0`.
pub fn winter_fall_delta(resources: &SimResources, rule: &FirewoodRule, day: u64) -> i32 {
    let spread = rule.winter_fall_spread;
    if spread == 0 || !in_severity_winter(resources, rule, day) {
        return 0;
    }
    let width = 2 * u64::from(spread) + 1;
    let u = mix(mix(resources.world_seed ^ FALL_SPREAD_SALT) ^ day);
    i32::try_from(u % width).unwrap_or(0) - i32::try_from(spread).unwrap_or(0)
}

/// Whether display day `day` lies in a winter of drawn severity (seasons on,
/// severity weights set).
fn in_severity_winter(resources: &SimResources, rule: &FirewoodRule, day: u64) -> bool {
    // Any tick of display day `day` gives its season and year; take its last.
    let tick = day
        .saturating_add(1)
        .saturating_mul(resources.ticks_per_day)
        .saturating_sub(resources.rules.clock_offset_ticks.saturating_add(1));
    severity_level_at(resources, rule, tick).is_some()
}

/// The rate `FireSystem` withdraws per settlement-hour at system tick `tick`
/// (Pass 8): 0 outside the burn window; otherwise the night's rate
/// ([`winter_burn_per_hour`] of the evening's tick), changed by the evening's
/// [`night_burn_delta`] with a floor of 1 when night weights are set. The
/// fire and the probe key both read it, so they never disagree. Villagers never read it: `wood_target`,
/// [`tonight_burn`] and urgency keep the severity's rate.
pub fn night_rate(resources: &SimResources, rule: &FirewoodRule, tick: u64) -> u32 {
    if resources.ticks_per_day == 0 {
        return 0;
    }
    let minute = tick.saturating_add(resources.rules.clock_offset_ticks) % resources.ticks_per_day;
    if !crate::runtime::decision::agent_decision::minute_in_window(
        minute,
        rule.burn_start_minute,
        rule.burn_end_minute,
    ) {
        return 0;
    }
    let night = night_tick_at(resources, rule, tick);
    let base = u32::from(winter_burn_per_hour(resources, rule, night));
    if rule.winter_night_burn_permille.is_some()
        && severity_level_at(resources, rule, night).is_some()
    {
        let delta = i64::from(night_burn_delta(
            resources,
            rule,
            day_index_at(resources, night),
        ));
        u32::try_from((i64::from(base) + delta).max(1)).unwrap_or(1)
    } else {
        base
    }
}

/// Winter deadwood fall per day in force at tick `tick` (Pass 8): the
/// severity's fall (or `winter_regen_units_per_day` without weights) plus the
/// day's [`winter_fall_delta`], floor 0. `None` outside a winter or without
/// the winter spread regrowth. `ResourceRegenSystem`, the probe key and the
/// recorder all read it.
pub fn winter_fall_at(resources: &SimResources, rule: &FirewoodRule, tick: u64) -> Option<u32> {
    let per_day = rule.winter_regen_units_per_day?;
    let (season, _) = season_at_tick(resources, tick)?;
    if season != WINTER {
        return None;
    }
    let fall = severity_level_at(resources, rule, tick).map_or(per_day, |level| {
        rule.winter_severity_regen_units_per_day[usize::from(level).min(2)]
    });
    let delta = winter_fall_delta(resources, rule, day_index_at(resources, tick));
    Some(u32::try_from((i64::from(fall) + i64::from(delta)).max(0)).unwrap_or(0))
}

/// Wood burned tonight (Pass 7): [`winter_burn_per_hour`] of the night's
/// booked tick times the burn hours. Equals `burn_per_day(night_season)`
/// without severity weights.
pub fn tonight_burn(resources: &SimResources, rule: &FirewoodRule) -> f64 {
    let tick = night_tick_at(resources, rule, resources.current_tick);
    f64::from(winter_burn_per_hour(resources, rule, tick)) * f64::from(burn_hours(rule))
}

/// Urgent mode (Pass 6): `urgent_start_minute` is set and `in_hand` wood is
/// below tonight's burn ([`tonight_burn`]; Pass 7 severity included). False
/// without the rule.
pub fn urgent_with(resources: &SimResources, in_hand: u64) -> bool {
    resources.rules.society.firewood.is_some_and(|rule| {
        rule.urgent_start_minute.is_some() && (in_hand as f64) < tonight_burn(resources, &rule)
    })
}

/// Whether settlement `sid` is in urgent mode now: [`urgent_with`] of its
/// [`wood_in_hand`].
pub fn is_urgent(world: &World, resources: &SimResources, sid: SettlementId) -> bool {
    resources
        .rules
        .society
        .firewood
        .is_some_and(|rule| rule.urgent_start_minute.is_some())
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
    let base = if season == WINTER {
        rule.winter_base_days.unwrap_or(rule.base_days)
    } else {
        rule.base_days
    };
    // Pass 7: the current day's burn, the winter's severity included; the
    // autumn preparation below keeps `burn_per_hour[3]` (nobody knows the
    // coming winter).
    let per_night = f64::from(winter_burn_per_hour(
        resources,
        &rule,
        resources.current_tick,
    )) * f64::from(burn_hours(&rule));
    let mut target = base * per_night;
    if let (Some(seasons), Some((current, _))) = (resources.rules.seasons, season_at(resources)) {
        if current == 2 || (current == 1 && rule.winter_prep_in_summer) {
            let prep_days = rule
                .winter_prep_days
                .unwrap_or(f64::from(seasons.days_per_season));
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
    u64::from(
        settlement
            .stockpile
            .get(&ResourceKind::Wood)
            .copied()
            .unwrap_or(0),
    ) + carried
        .iter()
        .map(|(_, wood)| u64::from(*wood))
        .sum::<u64>()
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
            winter_severity_permille: None,
            winter_severity_burn_per_hour: [3, 3, 3],
            winter_severity_regen_units_per_day: [6, 6, 6],
            claims_bounded_by_amount: false,
            winter_night_burn_permille: None,
            winter_night_burn_step: 0,
            winter_fall_spread: 0,
            windfall_over_snow: false,
            rain_soak_permille: 0,
            regional_wood: false,
            area_fall_floor_permille: 0,
            floor_separate: false,
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
        resources.rules = WorldRules::baseline(0);
        resources.rules.society.firewood = Some(rule());
        resources.rules.seasons = Some(SeasonRule {
            days_per_season: 7,
            regen_permille: [750, 1250, 1000, 500],
        });
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
        resources.rules = WorldRules::baseline(0);
        resources.rules.seasons = Some(SeasonRule {
            days_per_season: 7,
            regen_permille: [750, 1250, 1000, 500],
        });
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
        resources.rules = WorldRules::baseline(0);
        resources.rules.society.firewood = None;
        assert!(!urgent_with(&resources, 0));
        let mut r = rule();
        resources.rules.society.firewood = Some(r);
        assert!(
            !urgent_with(&resources, 0),
            "urgent mode is off without a start minute"
        );
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
    fn severity_is_minus_one_without_weights_and_booked_by_evening() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        assert_eq!(winter_severity_at(&resources, 0), -1);
        resources.rules = WorldRules::baseline(0);
        // The baseline without severity weights: no severity anywhere.
        if let Some(mut fw) = resources.rules.society.firewood {
            fw.winter_severity_permille = None;
            resources.rules.society.firewood = Some(fw);
        }
        for day in [0u64, 21, 27, 28] {
            assert_eq!(winter_severity_at(&resources, day * 1440), -1);
        }
        let mut rule = resources.rules.society.firewood.expect("baseline firewood");
        rule.winter_severity_permille = Some([0, 0, 1000]);
        resources.rules.society.firewood = Some(rule);
        // Display day 21 is the first winter day; 08:00 = day x 1440.
        assert_eq!(winter_severity_at(&resources, 20 * 1440), -1);
        assert_eq!(winter_severity_at(&resources, 21 * 1440), 2);
        // 02:00 of display day 21 still belongs to the autumn evening of day 20.
        assert_eq!(winter_severity_at(&resources, 21 * 1440 + 120 - 480), -1);
        // 02:00 of display day 28 (spring) belongs to the last winter evening.
        assert_eq!(winter_severity_at(&resources, 28 * 1440 + 120 - 480), 2);
        assert_eq!(winter_severity_at(&resources, 28 * 1440 + 420 - 480), -1);
        assert_eq!(year_at_tick(&resources, 28 * 1440), 1);
        assert_eq!(year_at_tick(&resources, 27 * 1440), 0);
    }

    #[test]
    fn winter_burn_reads_the_severity_and_matches_the_unweighted_burn() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules = WorldRules::baseline(0);
        let mut r = resources.rules.society.firewood.expect("baseline firewood");
        // Without severity weights the night burns the season's plain rate.
        r.winter_severity_permille = None;
        resources.rules.society.firewood = Some(r);
        // 22:00 of winter day 22 and 02:00 after it: Pass 6 rates without weights.
        let evening = 22 * 1440 + 1320 - 480;
        for tick in [evening, evening + 240, 14 * 1440] {
            resources.current_tick = tick;
            let season = night_season(&resources, &r);
            assert_eq!(tonight_burn(&resources, &r), burn_per_day(&r, season));
        }
        r.winter_severity_permille = Some([0, 0, 1000]);
        r.winter_severity_burn_per_hour = [2, 3, 4];
        resources.current_tick = evening + 240;
        assert_eq!(
            winter_burn_per_hour(&resources, &r, night_tick_at(&resources, &r, evening + 240)),
            4
        );
        assert_eq!(tonight_burn(&resources, &r), 4.0 * 13.0);
        // Autumn keeps its own rate.
        assert_eq!(winter_burn_per_hour(&resources, &r, 14 * 1440), 2);
    }

    #[test]
    fn severity_draw_follows_the_weights() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        let mut r = rule();
        assert_eq!(winter_severity(&resources, &r, 3), 1, "no weights: normal");
        for (weights, level) in [([1000, 0, 0], 0), ([0, 1000, 0], 1), ([0, 0, 1000], 2)] {
            r.winter_severity_permille = Some(weights);
            for seed in 0..20 {
                resources.world_seed = seed;
                assert_eq!(winter_severity(&resources, &r, seed + 1), level);
            }
        }
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
