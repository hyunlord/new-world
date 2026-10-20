//! Pass 8 Part 2: winters that vary night by night and day by day, and
//! windfall that lands on the snow.
//!
//! Expected numbers are read from `WorldRules::baseline(SEED)` (V8-0; was
//! `game_v14`, whose firewood draws the baseline keeps: night weights 250 /
//! 500 / 250 step 2, fall spread 2; seed 1's climate has v14's severity
//! weights 350 / 400 / 250); `harness_night_and_fall_draw_table` pins the
//! draws with explicit literals computed by an independent Python
//! implementation of section 7.2.
use sim_core::components::{ResourceKind, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FirewoodRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::settlement::firewood::{
    burn_hours, night_burn_delta, night_rate, night_tick_at, severity_level_at, tonight_burn,
    urgent_with, winter_burn_per_hour, winter_fall_delta, wood_target,
};

/// World seed of the fixture rules (`WorldRules::baseline`).
const SEED: u64 = 1;
use sim_systems::runtime::settlement::FireSystem;

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

fn rules() -> WorldRules {
    WorldRules::baseline(SEED)
}

fn baseline_rule() -> FirewoodRule {
    rules().society.firewood.expect("baseline firewood rule")
}

fn resources(rules: WorldRules, seed: u64) -> SimResources {
    let mut r = SimResources::new(8, 8, MaterialRegistry::new());
    r.rules = rules;
    r.world_seed = seed;
    r
}

/// The `n` first winter display days (seven-day seasons: days 28y + 21 .. 28y + 27).
fn winter_days(n: usize) -> Vec<u64> {
    (0..)
        .flat_map(|year: u64| (21..28).map(move |d| 28 * year + d))
        .take(n)
        .collect()
}

/// The baseline with every winter forced to `level` and the night weights `night`.
fn rules_with(level: usize, night: [u16; 3], step: u16) -> WorldRules {
    let mut rules = rules();
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
fn burned_at(rules: WorldRules, seed: u64, tick: u64) -> u32 {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = seed;
    engine.resources.current_tick = tick;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.store(ResourceKind::Wood, 100);
    engine.resources.settlements.insert(1, settlement);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    100 - engine.resources.settlements[&1]
        .stockpile
        .get(&ResourceKind::Wood)
        .copied()
        .unwrap_or(0)
}

#[test]
fn harness_night_burn_is_a_pure_draw() {
    let rule = baseline_rule();
    let weights = rule
        .winter_night_burn_permille
        .expect("the baseline draws night burns");
    let step = i16::try_from(rule.winter_night_burn_step).expect("small step");
    let mut r = resources(rules(), 11);
    // Same seed and evening, same delta, whatever the current tick.
    let first = night_burn_delta(&r, &rule, 23);
    r.current_tick = 777_777;
    assert_eq!(night_burn_delta(&r, &rule, 23), first);
    // Over 10,000 winter evenings each delta's share is within 0.02 of its weight.
    let mut counts = [0u32; 3];
    for day in winter_days(10_000) {
        let delta = night_burn_delta(&r, &rule, day);
        let index = [-step, 0, step]
            .iter()
            .position(|d| *d == delta)
            .expect("delta is -step, 0 or +step");
        counts[index] += 1;
    }
    for (count, weight) in counts.iter().zip(weights) {
        assert!(
            (f64::from(*count) / 10_000.0 - f64::from(weight) / 1000.0).abs() <= 0.02,
            "{counts:?} against {weights:?}"
        );
    }
    // Outside a winter there is no draw.
    for day in [0u64, 7, 14, 20, 28, 48] {
        assert_eq!(night_burn_delta(&r, &rule, day), 0, "day {day}");
    }
    assert_eq!(weights, [250, 500, 250]);
    assert_eq!(step, 2);
}

#[test]
fn harness_fall_spread_is_a_pure_draw() {
    let rule = baseline_rule();
    let spread = i32::try_from(rule.winter_fall_spread).expect("small spread");
    let width = usize::try_from(2 * spread + 1).expect("width");
    let mut r = resources(rules(), 5);
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
        assert!(
            (f64::from(*count) / 10_000.0 - 1.0 / width as f64).abs() <= 0.02,
            "{counts:?}"
        );
    }
    assert_eq!(winter_fall_delta(&r, &rule, 14), 0, "autumn has no draw");
    assert_eq!(spread, 2);
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
    // The baseline's draw parameters are the table's.
    let rule = baseline_rule();
    assert_eq!(rule.winter_night_burn_permille, Some([250, 500, 250]));
    assert_eq!(rule.winter_night_burn_step, 2);
    assert_eq!(rule.winter_fall_spread, 2);
    let rules = rules();
    for (seed, day, _u_night, night, _u_fall, fall) in TABLE {
        let r = resources(rules, seed);
        assert_eq!(
            night_burn_delta(&r, &rule, day),
            night,
            "seed {seed} day {day}"
        );
        assert_eq!(
            winter_fall_delta(&r, &rule, day),
            fall,
            "seed {seed} day {day}"
        );
    }
}

#[test]
fn harness_villagers_do_not_see_tonights_draw() {
    // `wood_target`, `tonight_burn` and urgency read the severity's rate, never
    // the night's draw, at every hour of the first two winters; the fire's
    // `night_rate` does read it. Expected values come from the severity's rate
    // alone (`winter_burn_per_hour` of the booked tick).
    let rule = baseline_rule();
    let hours = f64::from(burn_hours(&rule));
    let base_days = rule.winter_base_days.expect("baseline winter base");
    for seed in 0..4 {
        let mut with = resources(rules(), seed);
        let mut changed = 0;
        for day in [
            21u64, 22, 23, 24, 25, 26, 27, 28, 49, 50, 51, 52, 53, 54, 55, 56,
        ] {
            for hour in 0..24 {
                let tick = tick_at(day, hour * 60);
                with.current_tick = tick;
                let severity_rate = f64::from(winter_burn_per_hour(
                    &with,
                    &rule,
                    night_tick_at(&with, &rule, tick),
                ));
                let tonight = severity_rate * hours;
                assert_eq!(
                    tonight_burn(&with, &rule),
                    tonight,
                    "seed {seed} day {day} hour {hour}"
                );
                if severity_level_at(&with, &rule, tick).is_some() {
                    let day_rate = f64::from(winter_burn_per_hour(&with, &rule, tick));
                    assert_eq!(
                        wood_target(&with),
                        Some(base_days * day_rate * hours),
                        "seed {seed} day {day} hour {hour}"
                    );
                }
                for in_hand in [0u64, 20, 39, 52, 80] {
                    assert_eq!(urgent_with(&with, in_hand), (in_hand as f64) < tonight);
                }
                let burning = night_rate(&with, &rule, tick) > 0;
                changed += u32::from(
                    burning && f64::from(night_rate(&with, &rule, tick)) != severity_rate,
                );
            }
        }
        // Half the winter evenings draw a change (weights 250 / 500 / 250):
        // measured 117 / 65 / 91 / 91 off-rate burn hours on seeds 0..4 of the
        // ~200 sampled; the bar is a quarter of the ~100 expected.
        assert!(
            changed >= 26,
            "the fire itself does see the draw (seed {seed}: {changed} hours)"
        );
    }
}

#[test]
fn harness_night_rate_floor_is_one() {
    // Mild (2 an hour) with every night drawn -3: the floor keeps 1 an hour.
    let rules = rules_with(0, [1000, 0, 0], 3);
    let rule = rules.society.firewood.expect("firewood");
    let r = resources(rules, 4);
    let night = tick_at(22, 20 * 60);
    assert_eq!(winter_burn_per_hour(&r, &rule, night), 2);
    assert_eq!(night_burn_delta(&r, &rule, 22), -3);
    assert_eq!(night_rate(&r, &rule, night), 1);
    assert_eq!(burned_at(rules, 4, night), 1);
    // Outside the burn window the rate is 0 and nothing burns.
    assert_eq!(night_rate(&r, &rule, tick_at(22, 12 * 60)), 0);
    assert_eq!(burned_at(rules, 4, tick_at(22, 12 * 60)), 0);
    // Harsh (4) drawn +3 burns 7.
    let harsh = rules_with(2, [0, 0, 1000], 3);
    assert_eq!(burned_at(harsh, 4, night), 7);
}

#[test]
fn harness_last_night_keeps_its_evening_draw() {
    // 02:00 of display day 28 (the first spring day of year one) belongs to the
    // last winter evening (day 27): it burns that evening's draw.
    let rules = rules();
    let rule = baseline_rule();
    let seed = (0..1000)
        .find(|seed| night_burn_delta(&resources(rules, *seed), &rule, 27) != 0)
        .expect("a seed whose last winter evening draws a change");
    let r = resources(rules, seed);
    let delta = i64::from(night_burn_delta(&r, &rule, 27));
    for (day, minute) in [(27u64, 18 * 60), (27, 23 * 60), (28, 2 * 60), (28, 6 * 60)] {
        let tick = tick_at(day, minute);
        let booked = night_tick_at(&r, &rule, tick);
        let base = i64::from(winter_burn_per_hour(&r, &rule, booked));
        assert!(
            severity_level_at(&r, &rule, booked).is_some(),
            "day {day} {minute}: a winter night"
        );
        let want = u32::try_from((base + delta).max(1)).expect("rate");
        assert_eq!(
            night_rate(&r, &rule, tick),
            want,
            "day {day} minute {minute}"
        );
        assert_eq!(burned_at(rules, seed, tick), want);
    }
    // 18:00 of day 28 is a spring evening: its own rate, no draw.
    let spring = tick_at(28, 18 * 60);
    assert_eq!(
        night_rate(&r, &rule, spring),
        u32::from(rule.burn_per_hour[0])
    );
}
