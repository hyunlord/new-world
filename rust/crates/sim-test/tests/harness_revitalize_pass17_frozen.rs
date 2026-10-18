//! Pass 17 Part 0: v20 pins recorded from unchanged production Rust at
//! adfdecfece79ab019964729319389fd602685fab before any implementation change.
//! Every pre-existing rule field is pinned separately without struct literals.
//! Pin tuples: seed, fingerprint/digest at 1,440 ticks, fingerprint/digest at
//! 40,320 ticks, births, fissions. The long file is the required historical
//! field/value table, kept together in the one pin file specified by Part 0.
use sim_core::rules::{BirthRule, HousingCount, SettlementAnchor, ThresholdLaw, WorldRules};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v20_inert_scene, game_v20_scene};
use sim_test::worldmodel::digest::state_digest;

const V20_SEVERITY: [[u16; 3]; 5] = [
    [200, 400, 400],
    [350, 400, 250],
    [350, 400, 250],
    [200, 400, 400],
    [500, 350, 150],
];
const V20_REGEN: [[u16; 4]; 5] = [
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
fn assert_v20_fields(seed: usize, r: &WorldRules, effects: bool) {
    let a = &r.agent;
    assert_eq!(a.routine, None, "seed {seed}");
    assert_eq!(a.move_interval_ticks, 8, "seed {seed}");
    assert_eq!(a.idle_pause_chance, 0.4, "seed {seed}");
    assert_eq!(a.idle_tether_radius, Some(8), "seed {seed}");
    let social_seek = a.social_seek.expect("v20 social_seek");
    assert_eq!(social_seek.max_chase_ticks, 240, "seed {seed}");
    assert_eq!(social_seek.retry_cooldown_ticks, 480, "seed {seed}");
    assert_eq!(a.need_fallthrough, true, "seed {seed}");
    assert_eq!(a.critical_preemption, Some(75.0), "seed {seed}");
    assert_eq!(a.strict_consume, true, "seed {seed}");
    let night_rest = a.night_rest.expect("v20 night_rest");
    assert_eq!(night_rest.start_minute, 1260, "seed {seed}");
    assert_eq!(night_rest.end_minute, 360, "seed {seed}");
    assert_eq!(night_rest.fatigue_threshold, 0.0, "seed {seed}");
    assert_eq!(night_rest.recovery_per_tick, 0.12, "seed {seed}");
    assert_eq!(night_rest.wake_preempt, 75.0, "seed {seed}");
    let evening_gather = a.evening_gather.expect("v20 evening_gather");
    assert_eq!(evening_gather.start_minute, 1080, "seed {seed}");
    assert_eq!(evening_gather.end_minute, 1260, "seed {seed}");
    assert_eq!(evening_gather.tether_radius, 3, "seed {seed}");
    let forage = a.forage.expect("v20 forage");
    assert_eq!(forage.start_minute, 360, "seed {seed}");
    assert_eq!(forage.end_minute, 1080, "seed {seed}");
    assert_eq!(forage.forage_radius, 16, "seed {seed}");
    assert_eq!(forage.haul_threshold, 4, "seed {seed}");
    assert_eq!(forage.max_foragers_per_bush, 2, "seed {seed}");
    assert_eq!(a.preempt_gathering, true, "seed {seed}");
    assert_eq!(a.keyed_draws, true, "seed {seed}");
    assert_eq!(a.local_food_availability, true, "seed {seed}");

    let temperament = a.temperament.expect("v20 temperament");
    assert_eq!(temperament.effects, effects, "seed {seed}");
    assert_eq!(temperament.ns_tether_span, 4, "seed {seed}");
    assert_eq!(temperament.ha_bedtime_span_minutes, 60, "seed {seed}");
    assert_eq!(temperament.rd_social_rate_span_permille, 500, "seed {seed}");
    assert_eq!(temperament.p_food_haul_span, 3, "seed {seed}");
    assert_eq!(temperament.p_wood_haul_span, 2, "seed {seed}");
    assert_eq!(temperament.ns_departure_weight, 1000, "seed {seed}");

    let s = &r.society;
    assert_eq!(s.family, None, "seed {seed}");
    let BirthRule::Provisioned {
        cooldown_ticks,
        food_cost,
        max_mean_hunger,
        max_mean_thirst,
        sustain,
    } = s.birth
    else {
        panic!("seed {seed}: v20 provisioned births");
    };
    assert_eq!(cooldown_ticks, 720, "seed {seed}");
    assert_eq!(food_cost, 10, "seed {seed}");
    assert_eq!(max_mean_hunger, 60.0, "seed {seed}");
    assert_eq!(max_mean_thirst, 60.0, "seed {seed}");
    let sustain = sustain.expect("v20 sustained births");
    assert_eq!(sustain.utilization, 0.78, "seed {seed}");
    assert_eq!(sustain.reserve_days, 0.5, "seed {seed}");
    assert_eq!(sustain.forage_radius, 16, "seed {seed}");
    assert_eq!(sustain.capacity_weighted, true, "seed {seed}");
    let shelter_planning = s.shelter_planning.expect("v20 shelter_planning");
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
    let life_stage = s.life_stage.expect("v20 life_stage");
    assert_eq!(life_stage.child_ticks, 5760, "seed {seed}");
    assert_eq!(life_stage.hunger_rate, 0.03, "seed {seed}");
    assert_eq!(life_stage.thirst_rate, 0.048, "seed {seed}");
    assert_eq!(life_stage.sleep_rate, 0.03, "seed {seed}");
    assert_eq!(life_stage.tether_radius, 3, "seed {seed}");
    let stock_target = s.stock_target.expect("v20 stock_target");
    assert_eq!(stock_target.base_days, 2.5, "seed {seed}");
    assert_eq!(stock_target.winter_prep, true, "seed {seed}");
    assert_eq!(stock_target.hoard_cap_factor, 1.5, "seed {seed}");
    let aging = s.aging.expect("v20 ages");
    assert_eq!(aging.elder_age_ticks, 66240, "seed {seed}");
    assert_eq!(aging.lifespan_mean_ticks, 83520, "seed {seed}");
    assert_eq!(aging.lifespan_spread_ticks, 11520, "seed {seed}");
    assert_eq!(aging.founder_age_min_ticks, 8640, "seed {seed}");
    assert_eq!(aging.founder_age_max_ticks, 57600, "seed {seed}");
    assert_eq!(aging.founder_min_remaining_ticks, 4320, "seed {seed}");
    assert_eq!(aging.elder_tether_radius, 4, "seed {seed}");
    assert_eq!(aging.elder_evening_tether, true, "seed {seed}");
    let meals = s.meals.expect("v20 meals");
    assert_eq!(meals.breakfast_start, 360, "seed {seed}");
    assert_eq!(meals.breakfast_end, 480, "seed {seed}");
    assert_eq!(meals.dinner_start, 1080, "seed {seed}");
    assert_eq!(meals.dinner_end, 1200, "seed {seed}");
    assert_eq!(meals.meal_hunger, 30.0, "seed {seed}");
    assert_eq!(meals.camp_radius, 8, "seed {seed}");
    let f = s.firewood.expect("v20 burns firewood");
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
        Some(V20_SEVERITY[seed]),
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
    let exposure = s.exposure.expect("v20 exposure");
    assert_eq!(
        exposure.damage_per_cold_hour,
        [7.5, 5.0, 5.0],
        "seed {seed}"
    );
    assert_eq!(exposure.heal_block_hours, 2, "seed {seed}");
    assert_eq!(exposure.hunger_weight, 0.5, "seed {seed}");
    assert_eq!(exposure.use_hardiness, true, "seed {seed}");

    let fi = s.fission.expect("v20 splits");
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
    let hazard = fi.hazard.expect("v20 hazard");
    assert_eq!(hazard.size_lo, 24, "seed {seed}");
    assert_eq!(hazard.size_hi, 44, "seed {seed}");
    assert_eq!(hazard.crowd_lo_permille, 850, "seed {seed}");
    assert_eq!(hazard.crowd_hi_permille, 1050, "seed {seed}");
    assert_eq!(hazard.scarcity_days, 4, "seed {seed}");
    assert_eq!(hazard.weight_crowd_permille, 0, "seed {seed}");
    assert_eq!(hazard.weight_scarcity_permille, 1000, "seed {seed}");
    assert_eq!(hazard.max_daily_permille, 300, "seed {seed}");
    let pressure = fi.pressure.expect("v20 fission pressure");
    let ThresholdLaw::Exponential { mean_permille } = pressure.threshold_table else {
        panic!("seed {seed}: v20 exponential threshold");
    };
    assert_eq!(mean_permille, 1200, "seed {seed}");
    assert_eq!(pressure.decay_permille, 10, "seed {seed}");
    assert_eq!(pressure.reset_on_split, true, "seed {seed}");
    assert_eq!(r.clock_offset_ticks, 480, "seed {seed}");
    let seasons = r.seasons.expect("v20 seasons");
    assert_eq!(seasons.days_per_season, 7, "seed {seed}");
    assert_eq!(seasons.regen_permille, V20_REGEN[seed], "seed {seed}");
}
#[test]
fn harness_game_v20_rules_pinned_field_by_field() {
    for seed in 0..5_usize {
        let world_seed = u64::try_from(seed).expect("five small seeds");
        assert_v20_fields(seed, &WorldRules::game_v20(world_seed), true);
        assert_v20_fields(seed, &WorldRules::game_v20_inert(world_seed), false);
    }
}

type Pin = (u64, &'static str, u64, &'static str, u64, u64, u64);

const V20_PINS: [Pin; 5] = [
    (
        0,
        "31c527e7a75a46c4",
        0x63531cea2c24f4d8,
        "3c5b3986d9d0fd4d",
        0x7d36cfff77dc2ffe,
        22,
        0,
    ),
    (
        1,
        "305c505dda043970",
        0xab349a3106f6ebbf,
        "f22173ff6482cb31",
        0xb4fa3047105fc245,
        15,
        0,
    ),
    (
        2,
        "855c9a0e640101e7",
        0xf0927311bceeddb3,
        "018e5aee7c0a5f3a",
        0xacc550f929e8f635,
        30,
        0,
    ),
    (
        3,
        "5500cb2b686b3864",
        0xd87df6564bd4a168,
        "c756d9d926426aaf",
        0x2afc8dd44586b6f9,
        8,
        0,
    ),
    (
        4,
        "16643e67b3ff69b4",
        0xb084065c0ffb396b,
        "334dbfb0997d87a6",
        0xdf76fd83abb0b1d0,
        22,
        0,
    ),
];

const V20_INERT_PINS: [Pin; 5] = [
    (
        0,
        "8c6d54f83a031558",
        0xa1bdfb486a7eee07,
        "b4bc616cf7dd0318",
        0x443d3362d6fa0389,
        22,
        0,
    ),
    (
        1,
        "0c67aeb7cb8116b9",
        0xc520c4bccc028afb,
        "383610517e8ef2b4",
        0x915dd8a62d5f68ec,
        15,
        0,
    ),
    (
        2,
        "9584c89d85e4e080",
        0xdabd4327e784f7c6,
        "1577d90157eb2aab",
        0x9e0ded922037f1de,
        30,
        0,
    ),
    (
        3,
        "5b79f7f040b91299",
        0x374a45eeb7baf6e1,
        "2506716fd9da65fe",
        0x15d47e4fa6fcc0a1,
        8,
        0,
    ),
    (
        4,
        "03431ad9cffafa8d",
        0xa666ca2e0c4f832c,
        "959cb2651c1dbe0c",
        0xda47f5ed93ee70af,
        22,
        0,
    ),
];

fn check(name: &str, make: fn(u64) -> sim_engine::SimEngine, pins: &[Pin]) {
    for &(seed, fp_day, dg_day, fp_end, dg_end, births, fissions) in pins {
        let mut engine = make(seed);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(
            fingerprint(&engine),
            fp_day,
            "{name} seed {seed} day-one fingerprint"
        );
        assert_eq!(
            state_digest(&engine),
            dg_day,
            "{name} seed {seed} day-one digest"
        );
        for _ in 1440..40_320 {
            engine.tick();
        }
        assert_eq!(
            fingerprint(&engine),
            fp_end,
            "{name} seed {seed} day-28 fingerprint"
        );
        assert_eq!(
            state_digest(&engine),
            dg_end,
            "{name} seed {seed} day-28 digest"
        );
        assert_eq!(
            engine.resources.tallies.births, births,
            "{name} seed {seed} births"
        );
        assert_eq!(
            engine.resources.tallies.fissions, fissions,
            "{name} seed {seed} fissions"
        );
    }
}

#[test]
fn harness_v20_fingerprints_unchanged() {
    check("game_v20", game_v20_scene, &V20_PINS);
}

#[test]
fn harness_v20_inert_fingerprints_unchanged() {
    check("game_v20_inert", game_v20_inert_scene, &V20_INERT_PINS);
}
