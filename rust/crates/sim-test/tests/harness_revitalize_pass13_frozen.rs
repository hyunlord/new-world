//! Pass 13 Part 0: the frozen v18 references.
//!
//! The fingerprints and state digests of `game_v18_scene(seed)` and
//! `game_v18_nofission_scene(seed)` for seeds 0–3 after 1,440 ticks (day one)
//! and after 40,320 ticks (28 days: the first winter, births, exposure and,
//! in v18, the hazard fission path) were recorded on the Pass 13 base
//! `2aeb22c9` (code `1127455b`); `WorldRules::game_v18(0)` and
//! `WorldRules::V18_HAZARD` are compared field by field with literals of their
//! Pass 12 values. Pass 13 adds fields with values that reproduce v18 and must
//! never change any of these.
use sim_core::rules::{
    AgentRules, AgingRule, BirthRule, EveningGatherRule, ExposureRule, FirewoodRule, FissionHazard,
    FissionRule, ForageRule, HousingCount, LifeStageRule, MealRule, NightRestRule, SeasonRule,
    SettlementAnchor, ShelterPlanningRule, SocialSeekRule, SocietyRules, StockTargetRule,
    SustainRule, WorldRules,
};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v18_nofission_scene, game_v18_scene};
use sim_test::worldmodel::digest::state_digest;

/// `WorldRules::V18_HAZARD` as Pass 12 left it (`1127455b`).
const PASS12_V18_HAZARD: FissionHazard = FissionHazard {
    size_lo: 28,
    size_hi: 48,
    crowd_lo_permille: 850,
    crowd_hi_permille: 1050,
    scarcity_days: 2,
    weight_crowd_permille: 500,
    weight_scarcity_permille: 500,
    max_daily_permille: 120,
};

/// `WorldRules::game_v18(0)` as Pass 12 left it (`1127455b`), written out in full.
fn pass12_v18_seed0_literal() -> WorldRules {
    WorldRules {
        agent: AgentRules {
            move_interval_ticks: 8,
            idle_pause_chance: 0.4,
            idle_tether_radius: Some(8),
            social_seek: Some(SocialSeekRule {
                max_chase_ticks: 240,
                retry_cooldown_ticks: 480,
            }),
            need_fallthrough: true,
            critical_preemption: Some(75.0),
            strict_consume: true,
            night_rest: Some(NightRestRule {
                start_minute: 1260,
                end_minute: 360,
                fatigue_threshold: 0.0,
                recovery_per_tick: 0.12,
                wake_preempt: 75.0,
            }),
            evening_gather: Some(EveningGatherRule {
                start_minute: 1080,
                end_minute: 1260,
                tether_radius: 3,
            }),
            forage: Some(ForageRule {
                start_minute: 360,
                end_minute: 1080,
                forage_radius: 16,
                haul_threshold: 4,
                max_foragers_per_bush: 2,
            }),
            preempt_gathering: true,
            keyed_draws: true,
            local_food_availability: true,
        },
        society: SocietyRules {
            birth: BirthRule::Provisioned {
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
            shelter_planning: Some(ShelterPlanningRule {
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
            settlement_anchor: SettlementAnchor::Hearth,
            relief_radius: Some(8),
            life_stage: Some(LifeStageRule {
                child_ticks: 5760,
                hunger_rate: 0.03,
                thirst_rate: 0.048,
                sleep_rate: 0.03,
                tether_radius: 3,
            }),
            stock_target: Some(StockTargetRule {
                base_days: 2.5,
                winter_prep: true,
                hoard_cap_factor: 1.5,
            }),
            aging: Some(AgingRule {
                elder_age_ticks: 66240,
                lifespan_mean_ticks: 83520,
                lifespan_spread_ticks: 11520,
                founder_age_min_ticks: 8640,
                founder_age_max_ticks: 57600,
                founder_min_remaining_ticks: 4320,
                elder_tether_radius: 4,
                elder_evening_tether: true,
            }),
            meals: Some(MealRule {
                breakfast_start: 360,
                breakfast_end: 480,
                dinner_start: 1080,
                dinner_end: 1200,
                meal_hunger: 30.0,
                camp_radius: 8,
            }),
            firewood: Some(FirewoodRule {
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
                winter_prep_days: Some(4.0),
                winter_prep_in_summer: false,
                winter_base_days: Some(4.0),
                winter_regen_units_per_day: Some(6),
                winter_deadwood_cap: Some(1),
                night_season_from_evening: true,
                urgent_start_minute: Some(540),
                winter_severity_permille: Some([200, 400, 400]),
                winter_severity_burn_per_hour: [2, 3, 4],
                winter_severity_regen_units_per_day: [8, 6, 4],
                claims_bounded_by_amount: true,
                winter_night_burn_permille: Some([250, 500, 250]),
                winter_night_burn_step: 2,
                winter_fall_spread: 2,
                windfall_over_snow: true,
                rain_soak_permille: 300,
                regional_wood: true,
                area_fall_floor_permille: 1000,
                rain_wet_burn: true,
            }),
            exposure: Some(ExposureRule {
                damage_per_cold_hour: [8.0, 5.0, 5.0],
                heal_block_hours: 2,
                hunger_weight: 0.5,
                use_hardiness: true,
            }),
            fission: Some(FissionRule {
                min_members: 32,
                crowding: 1.0,
                group_share: 0.35,
                group_min: 6,
                group_max: 14,
                min_adults: 4,
                cooldown_days: 28,
                min_age_days: 10,
                site_clearance: 16,
                stock_share: true,
                max_settlements: 4,
                settle_timeout_days: 2,
                remnant_members: 2,
                allowed_seasons: [true, true, true, false],
                hazard: Some(PASS12_V18_HAZARD),
            }),
        },
        clock_offset_ticks: 480,
        seasons: Some(SeasonRule {
            days_per_season: 7,
            regen_permille: [675, 1125, 900, 450],
        }),
    }
}

#[test]
fn harness_v18_hazard_is_the_pass12_literal() {
    assert_eq!(WorldRules::V18_HAZARD, PASS12_V18_HAZARD);
    assert!(PASS12_V18_HAZARD.is_valid());
}

#[test]
fn harness_game_v18_seed0_rules_are_the_pass12_literal() {
    let v18 = WorldRules::game_v18(0);
    let literal = pass12_v18_seed0_literal();
    assert_eq!(v18.agent, literal.agent);
    assert_eq!(v18.society.birth, literal.society.birth);
    assert_eq!(
        v18.society.shelter_planning,
        literal.society.shelter_planning
    );
    assert_eq!(v18.society.life_stage, literal.society.life_stage);
    assert_eq!(v18.society.stock_target, literal.society.stock_target);
    assert_eq!(v18.society.aging, literal.society.aging);
    assert_eq!(v18.society.meals, literal.society.meals);
    assert_eq!(v18.society.firewood, literal.society.firewood);
    assert_eq!(v18.society.exposure, literal.society.exposure);
    assert_eq!(v18.society.fission, literal.society.fission);
    assert_eq!(v18.seasons, literal.seasons);
    assert_eq!(v18, literal);
    // v18_nofission is v18 without the fission rule.
    let nofission = WorldRules::game_v18_nofission(0);
    assert_eq!(
        WorldRules {
            society: SocietyRules {
                fission: None,
                ..literal.society
            },
            ..literal
        },
        nofission
    );
}

type Pin = (u64, &'static str, u64, &'static str, u64, u64, u64);

/// (seed, fingerprint at 1,440, digest at 1,440, fingerprint at 40,320,
/// digest at 40,320, births by 40,320, fissions founded by 40,320) recorded on
/// `2aeb22c9`. No seed of 0–3 has split by day 28 under v18 (the hazard is
/// checked and drawn every eligible morning but never fires), so the two
/// scenes coincide here; the hazard path is pinned by the probe guards (4.6).
const V18_PINS: [Pin; 4] = [
    (
        0,
        "8c6d54f83a031558",
        0xa1bd_fb48_6a7e_ee07,
        "1110a81bc6637a00",
        0x5c34_121c_def3_64a6,
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
        "82ca575c3d6d5187",
        0x95dd_d425_44e0_6441,
        30,
        0,
    ),
    (
        3,
        "5b79f7f040b91299",
        0x374a_45ee_b7ba_f6e1,
        "aec51e5226701a97",
        0x375f_0f71_e62c_48d0,
        8,
        0,
    ),
];

/// As [`V18_PINS`] for `game_v18_nofission_scene` (fissions always 0).
const V18N_PINS: [Pin; 4] = [
    (
        0,
        "8c6d54f83a031558",
        0xa1bd_fb48_6a7e_ee07,
        "1110a81bc6637a00",
        0x5c34_121c_def3_64a6,
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
        "82ca575c3d6d5187",
        0x95dd_d425_44e0_6441,
        30,
        0,
    ),
    (
        3,
        "5b79f7f040b91299",
        0x374a_45ee_b7ba_f6e1,
        "aec51e5226701a97",
        0x375f_0f71_e62c_48d0,
        8,
        0,
    ),
];

fn check(name: &str, make: fn(u64) -> sim_engine::SimEngine, pins: [Pin; 4]) {
    // Each seed runs 40,320 ticks; the four seeds run on scoped threads.
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
fn harness_v18_fingerprints_unchanged() {
    check("game_v18", game_v18_scene, V18_PINS);
}

#[test]
fn harness_v18_nofission_fingerprints_unchanged() {
    check("game_v18_nofission", game_v18_nofission_scene, V18N_PINS);
}
