//! Pass 16 Part 0: the frozen `game_v19` references.
//!
//! The fingerprints and state digests of `game_v19_scene(seed)` and
//! `game_v19_nofission_scene(seed)` for seeds 0–4 after 1,440 ticks (day one)
//! and after 40,320 ticks (28 days) were recorded on the Pass 16 base
//! `92688a95` (simulation code as `da8e45c7`) before any other Rust change of
//! the pass. `WorldRules::game_v19(s)` and `WorldRules::game_v19_nofission(s)`
//! for s in 0–4 are compared **field by field** with their recorded values —
//! never through a whole `WorldRules` / `AgentRules` literal, so a field added
//! to `AgentRules` later (Pass 16's `temperament`) cannot silently change v19
//! and this file never needs that field. `game_v19` is the research world's
//! scene (the world-model dataset is recorded on it): these must never change.
use sim_core::rules::{
    BirthRule, EveningGatherRule, ExposureRule, FissionHazard, FissionPressure, ForageRule,
    HousingCount, LifeStageRule, MealRule, NightRestRule, SeasonRule, SettlementAnchor,
    ShelterPlanningRule, SocialSeekRule, StockTargetRule, SustainRule, ThresholdLaw, WorldRules,
};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v19_nofission_scene, game_v19_scene};
use sim_test::worldmodel::digest::state_digest;

/// Per seed 0–4: `FirewoodRule::winter_severity_permille` and
/// `SeasonRule::regen_permille` (the only rule fields `WorldVariety` moves).
const V19_SEVERITY: [[u16; 3]; 5] = [
    [200, 400, 400],
    [350, 400, 250],
    [350, 400, 250],
    [200, 400, 400],
    [500, 350, 150],
];
const V19_REGEN: [[u16; 4]; 5] = [
    [675, 1125, 900, 450],
    [656, 1093, 875, 437],
    [677, 1128, 903, 451],
    [667, 1112, 890, 445],
    [746, 1243, 995, 497],
];

/// Asserts every field of `r` (game_v19 or game_v19_nofission of `seed`) one
/// by one; `fission` is checked only when `with_fission`.
fn assert_v19_fields(seed: usize, r: &WorldRules, with_fission: bool) {
    let a = &r.agent;
    assert_eq!(a.move_interval_ticks, 8, "seed {seed}");
    assert_eq!(a.idle_pause_chance, 0.4, "seed {seed}");
    assert_eq!(a.idle_tether_radius, Some(8), "seed {seed}");
    assert_eq!(
        a.social_seek,
        Some(SocialSeekRule {
            max_chase_ticks: 240,
            retry_cooldown_ticks: 480,
        }),
        "seed {seed}"
    );
    assert!(a.need_fallthrough, "seed {seed}");
    assert_eq!(a.critical_preemption, Some(75.0), "seed {seed}");
    assert!(a.strict_consume, "seed {seed}");
    assert_eq!(
        a.night_rest,
        Some(NightRestRule {
            start_minute: 1260,
            end_minute: 360,
            fatigue_threshold: 0.0,
            recovery_per_tick: 0.12,
            wake_preempt: 75.0,
        }),
        "seed {seed}"
    );
    assert_eq!(
        a.evening_gather,
        Some(EveningGatherRule {
            start_minute: 1080,
            end_minute: 1260,
            tether_radius: 3,
        }),
        "seed {seed}"
    );
    assert_eq!(
        a.forage,
        Some(ForageRule {
            start_minute: 360,
            end_minute: 1080,
            forage_radius: 16,
            haul_threshold: 4,
            max_foragers_per_bush: 2,
        }),
        "seed {seed}"
    );
    assert!(a.preempt_gathering, "seed {seed}");
    assert!(a.keyed_draws, "seed {seed}");
    assert!(a.local_food_availability, "seed {seed}");

    let s = &r.society;
    assert_eq!(
        s.birth,
        BirthRule::Provisioned {
            cooldown_ticks: 720,
            food_cost: 10,
            max_mean_hunger: 60.0,
            max_mean_thirst: 60.0,
            sustain: Some(SustainRule {
                utilization: 0.78,
                reserve_days: 0.5,
                forage_radius: 16,
                capacity_weighted: true,
            }),
        },
        "seed {seed}"
    );
    assert_eq!(
        s.shelter_planning,
        Some(ShelterPlanningRule {
            residents_per_shelter: 8,
            max_shelters: 12,
            plan_interval_ticks: 60,
            ring_min: 2,
            ring_max: 5,
            required_progress: 240,
            max_builders_per_site: 4,
            housing: HousingCount::Owned,
            planning_headroom: 2,
        }),
        "seed {seed}"
    );
    assert_eq!(s.settlement_anchor, SettlementAnchor::Hearth, "seed {seed}");
    assert_eq!(s.relief_radius, Some(8), "seed {seed}");
    assert_eq!(
        s.life_stage,
        Some(LifeStageRule {
            child_ticks: 5760,
            hunger_rate: 0.03,
            thirst_rate: 0.048,
            sleep_rate: 0.03,
            tether_radius: 3,
        }),
        "seed {seed}"
    );
    assert_eq!(
        s.stock_target,
        Some(StockTargetRule {
            base_days: 2.5,
            winter_prep: true,
            hoard_cap_factor: 1.5,
        }),
        "seed {seed}"
    );
    let aging = s.aging.expect("v19 ages");
    assert_eq!(aging.elder_age_ticks, 66240, "seed {seed}");
    assert_eq!(aging.lifespan_mean_ticks, 83520, "seed {seed}");
    assert_eq!(aging.lifespan_spread_ticks, 11520, "seed {seed}");
    assert_eq!(aging.founder_age_min_ticks, 8640, "seed {seed}");
    assert_eq!(aging.founder_age_max_ticks, 57600, "seed {seed}");
    assert_eq!(aging.founder_min_remaining_ticks, 4320, "seed {seed}");
    assert_eq!(aging.elder_tether_radius, 4, "seed {seed}");
    assert!(aging.elder_evening_tether, "seed {seed}");
    assert_eq!(
        s.meals,
        Some(MealRule {
            breakfast_start: 360,
            breakfast_end: 480,
            dinner_start: 1080,
            dinner_end: 1200,
            meal_hunger: 30.0,
            camp_radius: 8,
        }),
        "seed {seed}"
    );
    let f = s.firewood.expect("v19 burns firewood");
    assert_eq!(f.gather_start_minute, 600, "seed {seed}");
    assert_eq!(f.gather_end_minute, 1020, "seed {seed}");
    assert_eq!(f.wood_radius, 18, "seed {seed}");
    assert_eq!(f.haul_threshold, 2, "seed {seed}");
    assert_eq!(f.max_gatherers_per_spot, 1, "seed {seed}");
    assert_eq!(f.regen_permille, 500, "seed {seed}");
    assert_eq!(f.base_days, 1.5, "seed {seed}");
    assert!(f.winter_prep, "seed {seed}");
    assert_eq!(f.burn_start_minute, 1080, "seed {seed}");
    assert_eq!(f.burn_end_minute, 420, "seed {seed}");
    assert_eq!(f.burn_per_hour, [2, 1, 2, 3], "seed {seed}");
    assert_eq!(f.cold_start_minute, 1320, "seed {seed}");
    assert_eq!(f.cold_end_minute, 360, "seed {seed}");
    assert_eq!(f.cold_hunger_permille, 1500, "seed {seed}");
    assert_eq!(f.winter_prep_days, Some(4.0), "seed {seed}");
    assert!(!f.winter_prep_in_summer, "seed {seed}");
    assert_eq!(f.winter_base_days, Some(4.0), "seed {seed}");
    assert_eq!(f.winter_regen_units_per_day, Some(6), "seed {seed}");
    assert_eq!(f.winter_deadwood_cap, Some(1), "seed {seed}");
    assert!(f.night_season_from_evening, "seed {seed}");
    assert_eq!(f.urgent_start_minute, Some(540), "seed {seed}");
    assert_eq!(
        f.winter_severity_permille,
        Some(V19_SEVERITY[seed]),
        "seed {seed}"
    );
    assert_eq!(f.winter_severity_burn_per_hour, [2, 3, 4], "seed {seed}");
    assert_eq!(
        f.winter_severity_regen_units_per_day,
        [8, 6, 4],
        "seed {seed}"
    );
    assert!(f.claims_bounded_by_amount, "seed {seed}");
    assert_eq!(
        f.winter_night_burn_permille,
        Some([250, 500, 250]),
        "seed {seed}"
    );
    assert_eq!(f.winter_night_burn_step, 2, "seed {seed}");
    assert_eq!(f.winter_fall_spread, 2, "seed {seed}");
    assert!(f.windfall_over_snow, "seed {seed}");
    assert_eq!(f.rain_soak_permille, 300, "seed {seed}");
    assert!(f.regional_wood, "seed {seed}");
    assert_eq!(f.area_fall_floor_permille, 1000, "seed {seed}");
    assert!(f.rain_wet_burn, "seed {seed}");
    assert!(f.floor_separate, "seed {seed}");
    assert_eq!(
        s.exposure,
        Some(ExposureRule {
            damage_per_cold_hour: [7.5, 5.0, 5.0],
            heal_block_hours: 2,
            hunger_weight: 0.5,
            use_hardiness: true,
        }),
        "seed {seed}"
    );
    if with_fission {
        let fi = s.fission.expect("v19 splits");
        assert_eq!(fi.min_members, 28, "seed {seed}");
        assert_eq!(fi.crowding, 1.0, "seed {seed}");
        assert_eq!(fi.group_share, 0.35, "seed {seed}");
        assert_eq!(fi.group_min, 6, "seed {seed}");
        assert_eq!(fi.group_max, 14, "seed {seed}");
        assert_eq!(fi.min_adults, 4, "seed {seed}");
        assert_eq!(fi.cooldown_days, 28, "seed {seed}");
        assert_eq!(fi.min_age_days, 10, "seed {seed}");
        assert_eq!(fi.site_clearance, 16, "seed {seed}");
        assert!(fi.stock_share, "seed {seed}");
        assert_eq!(fi.max_settlements, 4, "seed {seed}");
        assert_eq!(fi.settle_timeout_days, 2, "seed {seed}");
        assert_eq!(fi.remnant_members, 2, "seed {seed}");
        assert_eq!(fi.allowed_seasons, [true, true, true, false], "seed {seed}");
        assert_eq!(
            fi.hazard,
            Some(FissionHazard {
                size_lo: 24,
                size_hi: 44,
                crowd_lo_permille: 850,
                crowd_hi_permille: 1050,
                scarcity_days: 4,
                weight_crowd_permille: 0,
                weight_scarcity_permille: 1000,
                max_daily_permille: 300,
            }),
            "seed {seed}"
        );
        assert_eq!(
            fi.pressure,
            Some(FissionPressure {
                threshold_table: ThresholdLaw::Exponential {
                    mean_permille: 1200
                },
                decay_permille: 10,
                reset_on_split: true,
            }),
            "seed {seed}"
        );
    } else {
        assert_eq!(s.fission, None, "seed {seed}");
    }
    assert_eq!(r.clock_offset_ticks, 480, "seed {seed}");
    assert_eq!(
        r.seasons,
        Some(SeasonRule {
            days_per_season: 7,
            regen_permille: V19_REGEN[seed],
        }),
        "seed {seed}"
    );
}

#[test]
fn harness_game_v19_rules_pinned_field_by_field() {
    for seed in 0..5usize {
        assert_v19_fields(seed, &WorldRules::game_v19(seed as u64), true);
        assert_v19_fields(seed, &WorldRules::game_v19_nofission(seed as u64), false);
    }
}

type Pin = (u64, &'static str, u64, &'static str, u64, u64, u64);

/// (seed, fingerprint at 1,440, digest at 1,440, fingerprint at 40,320,
/// digest at 40,320, births by 40,320, fissions founded by 40,320) recorded on
/// the Pass 16 base before any Rust change. No seed of 0–4 has split by day
/// 28 under v19 (no v19 seed 0–31 splits before day 28), so the two scenes
/// coincide here.
const V19_PINS: [Pin; 5] = [
    (
        0,
        "8c6d54f83a031558",
        0xa1bd_fb48_6a7e_ee07,
        "b4bc616cf7dd0318",
        0x443d_3362_d6fa_0389,
        22,
        0,
    ),
    (
        1,
        "0c67aeb7cb8116b9",
        0xc520_c4bc_cc02_8afb,
        "383610517e8ef2b4",
        0x915d_d8a6_2d5f_68ec,
        15,
        0,
    ),
    (
        2,
        "9584c89d85e4e080",
        0xdabd_4327_e784_f7c6,
        "1577d90157eb2aab",
        0x9e0d_ed92_2037_f1de,
        30,
        0,
    ),
    (
        3,
        "5b79f7f040b91299",
        0x374a_45ee_b7ba_f6e1,
        "2506716fd9da65fe",
        0x15d4_7e4f_a6fc_c0a1,
        8,
        0,
    ),
    (
        4,
        "03431ad9cffafa8d",
        0xa666_ca2e_0c4f_832c,
        "959cb2651c1dbe0c",
        0xda47_f5ed_93ee_70af,
        22,
        0,
    ),
];

/// As [`V19_PINS`] for `game_v19_nofission_scene` (fissions always 0).
const V19N_PINS: [Pin; 5] = V19_PINS;

fn check(name: &str, make: fn(u64) -> sim_engine::SimEngine, pins: [Pin; 5]) {
    // Each seed runs 40,320 ticks; the five seeds run on scoped threads.
    let rows: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = pins
            .iter()
            .map(|&(seed, ..)| {
                scope.spawn(move || {
                    let mut engine = make(seed);
                    for _ in 0..1440 {
                        engine.tick();
                    }
                    let day = (fingerprint(&engine), state_digest(&engine));
                    for _ in 1440..40_320 {
                        engine.tick();
                    }
                    let t = &engine.resources.tallies;
                    (
                        seed,
                        day,
                        fingerprint(&engine),
                        state_digest(&engine),
                        t.births,
                        t.fissions,
                    )
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("seed thread"))
            .collect()
    });
    let mut mismatches = Vec::new();
    for ((seed, (fp_day, dg_day), fp_end, dg_end, births, fissions), pin) in rows.iter().zip(pins) {
        assert_eq!(*seed, pin.0);
        let actual = (
            fp_day.as_str(),
            *dg_day,
            fp_end.as_str(),
            *dg_end,
            *births,
            *fissions,
        );
        if actual != (pin.1, pin.2, pin.3, pin.4, pin.5, pin.6) {
            mismatches.push(format!(
                "({seed}, {:?}, {:#018x}, {:?}, {:#018x}, {}, {}),",
                actual.0, actual.1, actual.2, actual.3, actual.4, actual.5
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{name} pins differ:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn harness_v19_fingerprints_unchanged() {
    check("game_v19", game_v19_scene, V19_PINS);
}

#[test]
fn harness_v19_nofission_fingerprints_unchanged() {
    check("game_v19_nofission", game_v19_nofission_scene, V19N_PINS);
}
