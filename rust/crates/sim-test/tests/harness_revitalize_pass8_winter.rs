//! Pass 8 Part 2: winters that vary night by night and day by day, and
//! windfall that lands on the snow.
//!
//! Expected numbers are read from `WorldRules::game_v14()` (Part 4 may change
//! it); `harness_night_and_fall_draw_table` pins the draws with explicit
//! literals computed by an independent Python implementation of section 7.2.
use sim_core::components::{ResourceKind, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FirewoodRule, WorldRules};
use sim_engine::{Intervention, RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::intervention::apply_intervention;
use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
use sim_systems::runtime::settlement::firewood::{
    burn_hours, night_burn_delta, night_rate, night_tick_at, severity_level_at, tonight_burn, urgent_with,
    winter_burn_per_hour, winter_fall_at, winter_fall_delta, wood_target,
};
use sim_systems::runtime::settlement::FireSystem;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use sim_test::worldmodel::digest::state_digest;

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

fn v14_rule() -> FirewoodRule {
    WorldRules::game_v14().society.firewood.expect("v14 keeps the firewood rule")
}

fn resources(rules: WorldRules, seed: u64) -> SimResources {
    let mut r = SimResources::new(8, 8, MaterialRegistry::new());
    r.rules = rules;
    r.world_seed = seed;
    r
}

/// The `n` first winter display days (seven-day seasons: days 28y + 21 .. 28y + 27).
fn winter_days(n: usize) -> Vec<u64> {
    (0..).flat_map(|year: u64| (21..28).map(move |d| 28 * year + d)).take(n).collect()
}

/// v14 with every winter forced to `level` and the night weights `night`.
fn v14_with(level: usize, night: [u16; 3], step: u16) -> WorldRules {
    let mut rules = WorldRules::game_v14();
    if let Some(rule) = rules.society.firewood.as_mut() {
        let mut weights = [0u16; 3];
        weights[level] = 1000;
        rule.winter_severity_permille = Some(weights);
        rule.winter_night_burn_permille = Some(night);
        rule.winter_night_burn_step = step;
    }
    rules
}

/// Wood burned by one hourly fire update at system tick `tick` from a store of 100.
fn burned_at(rules: WorldRules, seed: u64, tick: u64, cold_snap: bool) -> u32 {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = seed;
    engine.resources.current_tick = tick;
    if cold_snap {
        engine.resources.cold_snap_nights = 1;
    }
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.store(ResourceKind::Wood, 100);
    engine.resources.settlements.insert(1, settlement);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    100 - engine.resources.settlements[&1].stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0)
}

#[test]
fn harness_night_burn_is_a_pure_draw() {
    let rule = v14_rule();
    let weights = rule.winter_night_burn_permille.expect("v14 draws night burns");
    let step = i16::try_from(rule.winter_night_burn_step).expect("small step");
    let mut r = resources(WorldRules::game_v14(), 11);
    // Same seed and evening, same delta, whatever the current tick.
    let first = night_burn_delta(&r, &rule, 23);
    r.current_tick = 777_777;
    assert_eq!(night_burn_delta(&r, &rule, 23), first);
    // Over 10,000 winter evenings each delta's share is within 0.02 of its weight.
    let mut counts = [0u32; 3];
    for day in winter_days(10_000) {
        let delta = night_burn_delta(&r, &rule, day);
        let index = [-step, 0, step].iter().position(|d| *d == delta).expect("delta is -step, 0 or +step");
        counts[index] += 1;
    }
    for (count, weight) in counts.iter().zip(weights) {
        assert!((f64::from(*count) / 10_000.0 - f64::from(weight) / 1000.0).abs() <= 0.02, "{counts:?} against {weights:?}");
    }
    // Outside a winter there is no draw.
    for day in [0u64, 7, 14, 20, 28, 48] {
        assert_eq!(night_burn_delta(&r, &rule, day), 0, "day {day}");
    }
    // v13 rules: no delta anywhere in four years.
    let v13 = WorldRules::game_v13().society.firewood.expect("v13 firewood");
    let r13 = resources(WorldRules::game_v13(), 11);
    assert!((0..112).all(|day| night_burn_delta(&r13, &v13, day) == 0));
}

#[test]
fn harness_fall_spread_is_a_pure_draw() {
    let rule = v14_rule();
    let spread = i32::try_from(rule.winter_fall_spread).expect("small spread");
    let width = usize::try_from(2 * spread + 1).expect("width");
    let mut r = resources(WorldRules::game_v14(), 5);
    let first = winter_fall_delta(&r, &rule, 22);
    r.current_tick = 123_456;
    assert_eq!(winter_fall_delta(&r, &rule, 22), first);
    let mut counts = vec![0u32; width];
    for day in winter_days(10_000) {
        let delta = winter_fall_delta(&r, &rule, day);
        assert!((-spread..=spread).contains(&delta));
        counts[usize::try_from(delta + spread).expect("index")] += 1;
    }
    for count in &counts {
        assert!((f64::from(*count) / 10_000.0 - 1.0 / width as f64).abs() <= 0.02, "{counts:?}");
    }
    assert_eq!(winter_fall_delta(&r, &rule, 14), 0, "autumn has no draw");
    let v13 = WorldRules::game_v13().society.firewood.expect("v13 firewood");
    let r13 = resources(WorldRules::game_v13(), 5);
    assert!((0..112).all(|day| winter_fall_delta(&r13, &v13, day) == 0));
}

#[test]
fn harness_night_and_fall_draw_table() {
    // (seed, day, u % 1000 of the night draw, night delta under 250/500/250 step 2,
    //  u % 5 of the fall draw, fall delta under spread 2), from the Python
    // re-implementation of section 7.2 (not from this crate).
    const TABLE: [(u64, u64, u64, i16, u64, i32); 16] = [
        (0, 21, 750, 2, 2, 0),
        (0, 24, 282, 0, 1, -1),
        (0, 27, 106, -2, 0, -2),
        (0, 49, 78, -2, 3, 1),
        (1, 21, 558, 0, 4, 2),
        (1, 24, 990, 2, 4, 2),
        (1, 27, 258, 0, 1, -1),
        (1, 49, 280, 0, 2, 0),
        (2, 21, 174, -2, 4, 2),
        (2, 24, 577, 0, 4, 2),
        (2, 27, 315, 0, 4, 2),
        (2, 49, 451, 0, 2, 0),
        (3, 21, 307, 0, 4, 2),
        (3, 24, 707, 0, 4, 2),
        (3, 27, 870, 2, 1, -1),
        (3, 49, 78, -2, 1, -1),
    ];
    let mut rule = v14_rule();
    rule.winter_night_burn_permille = Some([250, 500, 250]);
    rule.winter_night_burn_step = 2;
    rule.winter_fall_spread = 2;
    let mut rules = WorldRules::game_v14();
    rules.society.firewood = Some(rule);
    for (seed, day, _u_night, night, _u_fall, fall) in TABLE {
        let r = resources(rules, seed);
        assert_eq!(night_burn_delta(&r, &rule, day), night, "seed {seed} day {day}");
        assert_eq!(winter_fall_delta(&r, &rule, day), fall, "seed {seed} day {day}");
    }
}

#[test]
fn harness_villagers_do_not_see_tonights_draw() {
    // `wood_target`, `tonight_burn` and urgency under game_v14() equal those of
    // game_v14() without the draws at every hour of the first two winters.
    let mut blind = WorldRules::game_v14();
    if let Some(rule) = blind.society.firewood.as_mut() {
        rule.winter_night_burn_permille = None;
        rule.winter_fall_spread = 0;
    }
    let rule = v14_rule();
    let blind_rule = blind.society.firewood.expect("firewood");
    for seed in 0..4 {
        let mut with = resources(WorldRules::game_v14(), seed);
        let mut without = resources(blind, seed);
        let mut changed = 0;
        for day in [21u64, 22, 23, 24, 25, 26, 27, 28, 49, 50, 51, 52, 53, 54, 55, 56] {
            for hour in 0..24 {
                let tick = tick_at(day, hour * 60);
                with.current_tick = tick;
                without.current_tick = tick;
                assert_eq!(wood_target(&with), wood_target(&without), "seed {seed} day {day} hour {hour}");
                assert_eq!(tonight_burn(&with, &rule), tonight_burn(&without, &blind_rule));
                for in_hand in [0u64, 20, 39, 52, 80] {
                    assert_eq!(urgent_with(&with, in_hand), urgent_with(&without, in_hand));
                }
                changed += u32::from(night_rate(&with, &rule, tick) != night_rate(&without, &blind_rule, tick));
            }
        }
        assert!(changed > 0, "the fire itself does see the draw (seed {seed})");
    }
}

#[test]
fn harness_night_rate_floor_is_one() {
    // Mild (2 an hour) with every night drawn -3: the floor keeps 1 an hour.
    let rules = v14_with(0, [1000, 0, 0], 3);
    let rule = rules.society.firewood.expect("firewood");
    let r = resources(rules, 4);
    let night = tick_at(22, 20 * 60);
    assert_eq!(winter_burn_per_hour(&r, &rule, night), 2);
    assert_eq!(night_burn_delta(&r, &rule, 22), -3);
    assert_eq!(night_rate(&r, &rule, night), 1);
    assert_eq!(burned_at(rules, 4, night, false), 1);
    // A cold snap adds its unit above the floor.
    assert_eq!(burned_at(rules, 4, night, true), 2);
    // Outside the burn window the rate is 0 and nothing burns.
    assert_eq!(night_rate(&r, &rule, tick_at(22, 12 * 60)), 0);
    assert_eq!(burned_at(rules, 4, tick_at(22, 12 * 60), false), 0);
    // Harsh (4) drawn +3 burns 7.
    let harsh = v14_with(2, [0, 0, 1000], 3);
    assert_eq!(burned_at(harsh, 4, night, false), 7);
}

#[test]
fn harness_last_night_keeps_its_evening_draw() {
    // 02:00 of display day 28 (the first spring day of year one) belongs to the
    // last winter evening (day 27): it burns that evening's draw.
    let rules = WorldRules::game_v14();
    let rule = v14_rule();
    let seed = (0..1000)
        .find(|seed| night_burn_delta(&resources(rules, *seed), &rule, 27) != 0)
        .expect("a seed whose last winter evening draws a change");
    let r = resources(rules, seed);
    let delta = i64::from(night_burn_delta(&r, &rule, 27));
    for (day, minute) in [(27u64, 18 * 60), (27, 23 * 60), (28, 2 * 60), (28, 6 * 60)] {
        let tick = tick_at(day, minute);
        let booked = night_tick_at(&r, &rule, tick);
        let base = i64::from(winter_burn_per_hour(&r, &rule, booked));
        assert!(severity_level_at(&r, &rule, booked).is_some(), "day {day} {minute}: a winter night");
        let want = u32::try_from((base + delta).max(1)).expect("rate");
        assert_eq!(night_rate(&r, &rule, tick), want, "day {day} minute {minute}");
        assert_eq!(burned_at(rules, seed, tick, false), want);
    }
    // 18:00 of day 28 is a spring evening: its own rate, no draw.
    let spring = tick_at(28, 18 * 60);
    assert_eq!(night_rate(&r, &rule, spring), u32::from(rule.burn_per_hour[0]));
}

/// Engine with three registered deadwood spots (maxima 6, 2, 6) and empty tiles,
/// completed tick `tick`.
fn deadwood_engine(rules: WorldRules, tick: u64) -> SimEngine {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    for _ in 0..tick {
        // `apply_intervention` reads the completed tick; advance the bare engine.
        engine.tick();
    }
    for (spot, max) in [((30, 30), 6u8), ((31, 30), 2), ((32, 30), 6)] {
        engine.resources.wood_source_max.insert(spot, max);
    }
    engine
}

#[test]
fn harness_windfall_lands_on_snow() {
    let day22 = tick_at(22, 12 * 60);
    // v14: a winter windfall fills every spot to its max, and later winter
    // pulses do not bury it.
    let mut engine = deadwood_engine(WorldRules::game_v14(), day22);
    let record = apply_intervention(&mut engine, Intervention::Windfall { units: 20 });
    assert_eq!(record.applied, 6 + 2 + 6);
    let mut regen = ResourceRegenSystem;
    for pulse in 1..=12 {
        engine.resources.current_tick = day22 + pulse * REGEN_INTERVAL;
        regen.tick(&mut engine.world, &mut engine.resources);
    }
    let cap = v14_rule().winter_deadwood_cap.expect("winter cap");
    assert!(engine.resources.wood_tiles.values().any(|v| *v > cap), "{:?}", engine.resources.wood_tiles);
    assert_eq!(engine.resources.wood_tiles.get(&(30, 30)), Some(&6));
    // The first pulse of the next winter (the snowfall) buries it.
    engine.resources.current_tick = tick_at(49, 0);
    regen.tick(&mut engine.world, &mut engine.resources);
    assert!(engine.resources.wood_tiles.values().all(|v| *v <= cap));
    // v13: the snow cap holds the windfall.
    let mut engine = deadwood_engine(WorldRules::game_v13(), day22);
    assert_eq!(apply_intervention(&mut engine, Intervention::Windfall { units: 20 }).applied, 3 * u64::from(cap));
}

/// `state_digest` every 60 ticks over 35 days of `seed` under `rules`.
fn digests(seed: u64, rules: WorldRules) -> Vec<u64> {
    let mut engine = build_world_with(&WorldGenConfig::game_v13(seed), rules);
    let mut out = Vec::new();
    while engine.current_tick() <= 35 * 1440 {
        if engine.current_tick().is_multiple_of(60) {
            out.push(state_digest(&engine));
        }
        engine.tick();
    }
    out
}

#[test]
fn harness_snow_burial_once_is_inert_without_windfall() {
    // v13 with windfall_over_snow and no interventions equals plain v13: the
    // burial at the first winter pulse leaves every spot at most the cap, and
    // nothing else raises a spot above it during winter.
    let mut once = WorldRules::game_v13();
    if let Some(rule) = once.society.firewood.as_mut() {
        rule.windfall_over_snow = true;
    }
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4u64)
            .map(|seed| scope.spawn(move || (seed, digests(seed, WorldRules::game_v13()), digests(seed, once))))
            .collect();
        for handle in handles {
            let (seed, plain, buried_once) = handle.join().expect("worker");
            assert_eq!(plain.len(), 35 * 24 + 1);
            assert_eq!(plain, buried_once, "seed {seed}");
        }
    });
}

#[test]
fn harness_firewood_v13_unchanged() {
    // Under v13 the fire burns the severity's rate (+1 on a cold snap) at
    // every hour of a year and the winter fall is the severity's: Pass 7.
    let rules = WorldRules::game_v13();
    let rule = rules.society.firewood.expect("v13 firewood");
    for seed in 0..4 {
        let mut r = resources(rules, seed);
        for hour in 0..28 * 24 {
            let tick = tick_at(0, 480) + hour * 60;
            r.current_tick = tick;
            let night = night_tick_at(&r, &rule, tick);
            let minute = (tick + 480) % 1440;
            let burning = !(420..1080).contains(&minute);
            let want = if burning { u32::from(winter_burn_per_hour(&r, &rule, night)) } else { 0 };
            assert_eq!(night_rate(&r, &rule, tick), want, "seed {seed} hour {hour}");
            r.cold_snap_tonight = true;
            assert_eq!(night_rate(&r, &rule, tick), if burning { want + 1 } else { 0 });
            r.cold_snap_tonight = false;
            let fall = severity_level_at(&r, &rule, tick).map(|level| rule.winter_severity_regen_units_per_day[usize::from(level)]);
            assert_eq!(winter_fall_at(&r, &rule, tick), fall, "seed {seed} hour {hour}");
        }
        assert_eq!(tonight_burn(&r, &rule), f64::from(winter_burn_per_hour(&r, &rule, night_tick_at(&r, &rule, r.current_tick))) * f64::from(burn_hours(&rule)));
    }
}
