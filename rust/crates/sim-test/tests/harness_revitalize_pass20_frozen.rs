//! Pass 20 Part 0: `game_v24` pins recorded from unchanged production Rust at
//! 5cb16639b2334800aee9b907a7f3b26f0c657aaf before any implementation change.
//! One `#[test]` per seed so the harness can run them in parallel. Pin tuple:
//! fingerprint/digest after 1,440, 40,320 and 86,400 ticks (day 60, past the
//! first pairings and fissions), then births and completed fissions at day 60.
//! Every `game_v24` rule leaf is asserted separately, without struct literals,
//! so a new `SocietyRules` field does not touch this file.
use sim_core::rules::{BirthRule, HousingCount, SettlementAnchor, ThresholdLaw, WorldRules};
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v24_scene;
use sim_test::worldmodel::digest::state_digest;

const V24_SEVERITY: [[u16; 3]; 5] = [
    [200, 400, 400],
    [350, 400, 250],
    [350, 400, 250],
    [200, 400, 400],
    [500, 350, 150],
];
const V24_REGEN: [[u16; 4]; 5] = [
    [675, 1125, 900, 450],
    [656, 1093, 875, 437],
    [677, 1128, 903, 451],
    [667, 1112, 890, 445],
    [746, 1243, 995, 497],
];

#[allow(
    clippy::bool_assert_comparison,
    reason = "Part 0 requires individual assert_eq! field pins"
)]
fn assert_v24_fields(seed: usize, r: &WorldRules) {
    let a = &r.agent;
    let routine = a.routine.expect("v24 routine");
    assert_eq!(routine.day_start_minute, 420, "seed {seed}");
    assert_eq!(routine.day_end_minute, 1080, "seed {seed}");
    assert_eq!(routine.base_duration_minutes, 90, "seed {seed}");
    assert_eq!(routine.duration_span_minutes, 60, "seed {seed}");
    assert_eq!(routine.min_duration_minutes, 20, "seed {seed}");
    assert_eq!(routine.walk_timeout_minutes, 150, "seed {seed}");
    assert_eq!(routine.rest_minutes_min, 20, "seed {seed}");
    assert_eq!(routine.rest_minutes_max, 60, "seed {seed}");
    assert_eq!(
        routine.adult_weights,
        [180, 180, 140, 200, 120],
        "seed {seed}"
    );
    assert_eq!(routine.elder_weights, [500, 500], "seed {seed}");
    let stagger = routine.end_stagger.expect("v24 end_stagger");
    assert_eq!(stagger.base_minute, 1065, "seed {seed}");
    assert_eq!(stagger.key_span, 30, "seed {seed}");
    assert_eq!(stagger.p_span_minutes, 45, "seed {seed}");
    assert_eq!(a.move_interval_ticks, 8, "seed {seed}");
    assert_eq!(a.idle_pause_chance, 0.4, "seed {seed}");
    assert_eq!(a.idle_tether_radius, Some(8), "seed {seed}");
    let social_seek = a.social_seek.expect("v24 social_seek");
    assert_eq!(social_seek.max_chase_ticks, 240, "seed {seed}");
    assert_eq!(social_seek.retry_cooldown_ticks, 480, "seed {seed}");
    assert_eq!(a.need_fallthrough, true, "seed {seed}");
    assert_eq!(a.critical_preemption, Some(75.0), "seed {seed}");
    assert_eq!(a.strict_consume, true, "seed {seed}");
    let night_rest = a.night_rest.expect("v24 night_rest");
    assert_eq!(night_rest.start_minute, 1260, "seed {seed}");
    assert_eq!(night_rest.end_minute, 360, "seed {seed}");
    assert_eq!(night_rest.fatigue_threshold, 0.0, "seed {seed}");
    assert_eq!(night_rest.recovery_per_tick, 0.12, "seed {seed}");
    assert_eq!(night_rest.wake_preempt, 75.0, "seed {seed}");
    let evening_gather = a.evening_gather.expect("v24 evening_gather");
    assert_eq!(evening_gather.start_minute, 1080, "seed {seed}");
    assert_eq!(evening_gather.end_minute, 1260, "seed {seed}");
    assert_eq!(evening_gather.tether_radius, 3, "seed {seed}");
    let forage = a.forage.expect("v24 forage");
    assert_eq!(forage.start_minute, 360, "seed {seed}");
    assert_eq!(forage.end_minute, 1080, "seed {seed}");
    assert_eq!(forage.forage_radius, 16, "seed {seed}");
    assert_eq!(forage.haul_threshold, 4, "seed {seed}");
    assert_eq!(forage.max_foragers_per_bush, 2, "seed {seed}");
    assert_eq!(a.preempt_gathering, true, "seed {seed}");
    assert_eq!(a.keyed_draws, true, "seed {seed}");
    assert_eq!(a.local_food_availability, true, "seed {seed}");

    let temperament = a.temperament.expect("v24 temperament");
    assert_eq!(temperament.effects, true, "seed {seed}");
    assert_eq!(temperament.ns_tether_span, 4, "seed {seed}");
    assert_eq!(temperament.ha_bedtime_span_minutes, 60, "seed {seed}");
    assert_eq!(temperament.rd_social_rate_span_permille, 500, "seed {seed}");
    assert_eq!(temperament.p_food_haul_span, 3, "seed {seed}");
    assert_eq!(temperament.p_wood_haul_span, 2, "seed {seed}");
    assert_eq!(temperament.ns_departure_weight, 1000, "seed {seed}");

    let s = &r.society;
    let family = s.family.expect("v24 family");
    assert_eq!(family.min_familiarity_permille, 200, "seed {seed}");
    assert_eq!(family.pair_minute, 1200, "seed {seed}");
    assert_eq!(family.elder_partner_follows, true, "seed {seed}");
    let BirthRule::Provisioned {
        cooldown_ticks,
        food_cost,
        max_mean_hunger,
        max_mean_thirst,
        sustain,
    } = s.birth
    else {
        panic!("seed {seed}: v24 provisioned births");
    };
    assert_eq!(cooldown_ticks, 720, "seed {seed}");
    assert_eq!(food_cost, 10, "seed {seed}");
    assert_eq!(max_mean_hunger, 60.0, "seed {seed}");
    assert_eq!(max_mean_thirst, 60.0, "seed {seed}");
    let sustain = sustain.expect("v24 sustained births");
    assert_eq!(sustain.utilization, 0.78, "seed {seed}");
    assert_eq!(sustain.reserve_days, 0.5, "seed {seed}");
    assert_eq!(sustain.forage_radius, 16, "seed {seed}");
    assert_eq!(sustain.capacity_weighted, true, "seed {seed}");
    let shelter_planning = s.shelter_planning.expect("v24 shelter_planning");
    assert_eq!(shelter_planning.residents_per_shelter, 8, "seed {seed}");
    assert_eq!(shelter_planning.max_shelters, 12, "seed {seed}");
    assert_eq!(shelter_planning.plan_interval_ticks, 60, "seed {seed}");
    assert_eq!(shelter_planning.ring_min, 2, "seed {seed}");
    assert_eq!(shelter_planning.ring_max, 5, "seed {seed}");
    assert_eq!(shelter_planning.required_progress, 240, "seed {seed}");
    assert_eq!(shelter_planning.max_builders_per_site, 4, "seed {seed}");
    assert_eq!(shelter_planning.housing, HousingCount::Owned, "seed {seed}");
    assert_eq!(shelter_planning.planning_headroom, 2, "seed {seed}");
    assert_eq!(s.settlement_anchor, SettlementAnchor::Hearth, "seed {seed}");
    assert_eq!(s.relief_radius, Some(8), "seed {seed}");
    let life_stage = s.life_stage.expect("v24 life_stage");
    assert_eq!(life_stage.child_ticks, 5760, "seed {seed}");
    assert_eq!(life_stage.hunger_rate, 0.03, "seed {seed}");
    assert_eq!(life_stage.thirst_rate, 0.048, "seed {seed}");
    assert_eq!(life_stage.sleep_rate, 0.03, "seed {seed}");
    assert_eq!(life_stage.tether_radius, 3, "seed {seed}");
    let stock_target = s.stock_target.expect("v24 stock_target");
    assert_eq!(stock_target.base_days, 2.5, "seed {seed}");
    assert_eq!(stock_target.winter_prep, true, "seed {seed}");
    assert_eq!(stock_target.hoard_cap_factor, 1.5, "seed {seed}");
    let aging = s.aging.expect("v24 ages");
    assert_eq!(aging.elder_age_ticks, 66240, "seed {seed}");
    assert_eq!(aging.lifespan_mean_ticks, 83520, "seed {seed}");
    assert_eq!(aging.lifespan_spread_ticks, 11520, "seed {seed}");
    assert_eq!(aging.founder_age_min_ticks, 8640, "seed {seed}");
    assert_eq!(aging.founder_age_max_ticks, 57600, "seed {seed}");
    assert_eq!(aging.founder_min_remaining_ticks, 4320, "seed {seed}");
    assert_eq!(aging.elder_tether_radius, 4, "seed {seed}");
    assert_eq!(aging.elder_evening_tether, true, "seed {seed}");
    let meals = s.meals.expect("v24 meals");
    assert_eq!(meals.breakfast_start, 360, "seed {seed}");
    assert_eq!(meals.breakfast_end, 480, "seed {seed}");
    assert_eq!(meals.dinner_start, 1080, "seed {seed}");
    assert_eq!(meals.dinner_end, 1200, "seed {seed}");
    assert_eq!(meals.meal_hunger, 30.0, "seed {seed}");
    assert_eq!(meals.camp_radius, 8, "seed {seed}");
    let f = s.firewood.expect("v24 burns firewood");
    assert_eq!(f.gather_start_minute, 600, "seed {seed}");
    assert_eq!(f.gather_end_minute, 1020, "seed {seed}");
    assert_eq!(f.wood_radius, 18, "seed {seed}");
    assert_eq!(f.haul_threshold, 2, "seed {seed}");
    assert_eq!(f.max_gatherers_per_spot, 1, "seed {seed}");
    assert_eq!(f.regen_permille, 500, "seed {seed}");
    assert_eq!(f.base_days, 1.5, "seed {seed}");
    assert_eq!(f.winter_prep, true, "seed {seed}");
    assert_eq!(f.burn_start_minute, 1080, "seed {seed}");
    assert_eq!(f.burn_end_minute, 420, "seed {seed}");
    assert_eq!(f.burn_per_hour, [2, 1, 2, 3], "seed {seed}");
    assert_eq!(f.cold_start_minute, 1320, "seed {seed}");
    assert_eq!(f.cold_end_minute, 360, "seed {seed}");
    assert_eq!(f.cold_hunger_permille, 1500, "seed {seed}");
    assert_eq!(f.winter_prep_days, Some(4.0), "seed {seed}");
    assert_eq!(f.winter_prep_in_summer, false, "seed {seed}");
    assert_eq!(f.winter_base_days, Some(4.0), "seed {seed}");
    assert_eq!(f.winter_regen_units_per_day, Some(6), "seed {seed}");
    assert_eq!(f.winter_deadwood_cap, Some(1), "seed {seed}");
    assert_eq!(f.night_season_from_evening, true, "seed {seed}");
    assert_eq!(f.urgent_start_minute, Some(540), "seed {seed}");
    assert_eq!(
        f.winter_severity_permille,
        Some(V24_SEVERITY[seed]),
        "seed {seed}"
    );
    assert_eq!(f.winter_severity_burn_per_hour, [2, 3, 4], "seed {seed}");
    assert_eq!(
        f.winter_severity_regen_units_per_day,
        [8, 6, 4],
        "seed {seed}"
    );
    assert_eq!(f.claims_bounded_by_amount, true, "seed {seed}");
    assert_eq!(
        f.winter_night_burn_permille,
        Some([250, 500, 250]),
        "seed {seed}"
    );
    assert_eq!(f.winter_night_burn_step, 2, "seed {seed}");
    assert_eq!(f.winter_fall_spread, 2, "seed {seed}");
    assert_eq!(f.windfall_over_snow, true, "seed {seed}");
    assert_eq!(f.rain_soak_permille, 300, "seed {seed}");
    assert_eq!(f.regional_wood, true, "seed {seed}");
    assert_eq!(f.area_fall_floor_permille, 1000, "seed {seed}");
    assert_eq!(f.rain_wet_burn, true, "seed {seed}");
    assert_eq!(f.floor_separate, true, "seed {seed}");
    let exposure = s.exposure.expect("v24 exposure");
    assert_eq!(
        exposure.damage_per_cold_hour,
        [7.5, 5.0, 5.0],
        "seed {seed}"
    );
    assert_eq!(exposure.heal_block_hours, 2, "seed {seed}");
    assert_eq!(exposure.hunger_weight, 0.5, "seed {seed}");
    assert_eq!(exposure.use_hardiness, true, "seed {seed}");

    let fi = s.fission.expect("v24 splits");
    assert_eq!(fi.min_members, 28, "seed {seed}");
    assert_eq!(fi.crowding, 1.0, "seed {seed}");
    assert_eq!(fi.group_share, 0.35, "seed {seed}");
    assert_eq!(fi.group_min, 6, "seed {seed}");
    assert_eq!(fi.group_max, 14, "seed {seed}");
    assert_eq!(fi.min_adults, 4, "seed {seed}");
    assert_eq!(fi.cooldown_days, 28, "seed {seed}");
    assert_eq!(fi.min_age_days, 10, "seed {seed}");
    assert_eq!(fi.site_clearance, 16, "seed {seed}");
    assert_eq!(fi.stock_share, true, "seed {seed}");
    assert_eq!(fi.max_settlements, 4, "seed {seed}");
    assert_eq!(fi.settle_timeout_days, 2, "seed {seed}");
    assert_eq!(fi.remnant_members, 2, "seed {seed}");
    assert_eq!(fi.allowed_seasons, [true, true, true, false], "seed {seed}");
    let hazard = fi.hazard.expect("v24 hazard");
    assert_eq!(hazard.size_lo, 24, "seed {seed}");
    assert_eq!(hazard.size_hi, 44, "seed {seed}");
    assert_eq!(hazard.crowd_lo_permille, 850, "seed {seed}");
    assert_eq!(hazard.crowd_hi_permille, 1050, "seed {seed}");
    assert_eq!(hazard.scarcity_days, 4, "seed {seed}");
    assert_eq!(hazard.weight_crowd_permille, 0, "seed {seed}");
    assert_eq!(hazard.weight_scarcity_permille, 1000, "seed {seed}");
    assert_eq!(hazard.max_daily_permille, 300, "seed {seed}");
    let pressure = fi.pressure.expect("v24 fission pressure");
    let ThresholdLaw::Exponential { mean_permille } = pressure.threshold_table else {
        panic!("seed {seed}: v24 exponential threshold");
    };
    assert_eq!(mean_permille, 1200, "seed {seed}");
    assert_eq!(pressure.decay_permille, 10, "seed {seed}");
    assert_eq!(pressure.reset_on_split, true, "seed {seed}");
    assert_eq!(r.clock_offset_ticks, 480, "seed {seed}");
    let seasons = r.seasons.expect("v24 seasons");
    assert_eq!(seasons.days_per_season, 7, "seed {seed}");
    assert_eq!(seasons.regen_permille, V24_REGEN[seed], "seed {seed}");
}
#[test]
fn harness_game_v24_rules_pinned_field_by_field() {
    for seed in 0..5_usize {
        let world_seed = u64::try_from(seed).expect("five small seeds");
        assert_v24_fields(seed, &WorldRules::game_v24(world_seed));
    }
}

/// (fingerprint, digest) after 1,440 / 40,320 / 86,400 ticks, births, fissions.
type Pin = ([(&'static str, u64); 3], u64, u64);

fn check(name: &str, make: fn(u64) -> sim_engine::SimEngine, seed: u64, pin: Pin) {
    let (points, births, fissions) = pin;
    let mut engine = make(seed);
    let mut done = 0;
    for ((fp, dg), end) in points.into_iter().zip([1_440_u64, 40_320, 86_400]) {
        for _ in done..end {
            engine.tick();
        }
        done = end;
        assert_eq!(
            fingerprint(&engine),
            fp,
            "{name} seed {seed} fingerprint at tick {end}"
        );
        assert_eq!(
            state_digest(&engine),
            dg,
            "{name} seed {seed} digest at tick {end}"
        );
    }
    assert_eq!(
        engine.resources.tallies.births, births,
        "{name} seed {seed} births"
    );
    assert_eq!(
        engine.resources.tallies.fissions, fissions,
        "{name} seed {seed} fissions"
    );
}

#[test]
fn harness_v24_seed0_pinned() {
    check(
        "game_v24",
        game_v24_scene,
        0,
        (
            [
                ("e253a74e0cc213b3", 0x936424005a49f0c6),
                ("3d257126608aab13", 0xe22e267d7864288c),
                ("3fc890d37454f54f", 0x80d5d64ae84b46dc),
            ],
            82,
            1,
        ),
    );
}

#[test]
fn harness_v24_seed1_pinned() {
    check(
        "game_v24",
        game_v24_scene,
        1,
        (
            [
                ("1f1b53d1a0d7f4f1", 0x944a92902bcd7952),
                ("62fde7d48ff8cbdb", 0x9b5c6fedab8eba92),
                ("0fefc585fdb84bfe", 0xad80243da245b79e),
            ],
            37,
            0,
        ),
    );
}

#[test]
fn harness_v24_seed2_pinned() {
    check(
        "game_v24",
        game_v24_scene,
        2,
        (
            [
                ("8081f62d680a37dd", 0x2b146c1475e5c1a5),
                ("7410dbd8d68d0755", 0x28d97307d544f8b9),
                ("5ea9b6499065a7c1", 0xc7482151d99a3f7e),
            ],
            51,
            0,
        ),
    );
}

#[test]
fn harness_v24_seed3_pinned() {
    check(
        "game_v24",
        game_v24_scene,
        3,
        (
            [
                ("e12406199ec2f886", 0x4781add790bc2ee7),
                ("c29ec409274a50c5", 0x1fbff52c06af3e35),
                ("3afc618b78897cda", 0xadfdac0e1aefeef0),
            ],
            29,
            0,
        ),
    );
}

#[test]
fn harness_v24_seed4_pinned() {
    check(
        "game_v24",
        game_v24_scene,
        4,
        (
            [
                ("79d8f78971064dc7", 0x3fa1b2f2bf3bcae2),
                ("5fbf784a9b1cae2e", 0x8de7b84ba844b219),
                ("fb8c906bc767a72a", 0xfd099a6b16710527),
            ],
            52,
            0,
        ),
    );
}
