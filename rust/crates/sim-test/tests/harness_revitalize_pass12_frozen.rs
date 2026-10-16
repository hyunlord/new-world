//! Pass 12 Part 0: the frozen v17 references.
//!
//! The fingerprints and state digests of `game_v17_scene(seed)` and
//! `game_v17_nofission_scene(seed)` for seeds 0–3 after 1,440 ticks (day one)
//! and after 40,320 ticks (28 days: the first winter, births, exposure and,
//! in v17, the first fissions) were recorded on the Pass 12 base `31559232`;
//! `WorldRules::game_v17(0)` and `RANGES` are compared field by field with
//! literals of their Pass 11 values. Pass 12 adds fields with values that
//! reproduce v17 and must never change any of these.
use sim_core::rules::{
    AgentRules, AgingRule, BirthRule, EveningGatherRule, ExposureRule, FirewoodRule, FissionRule,
    ForageRule, HousingCount, LifeStageRule, MealRule, NightRestRule, SeasonRule, SettlementAnchor,
    ShelterPlanningRule, SocialSeekRule, SocietyRules, StockTargetRule, SustainRule, WorldRules,
};
use sim_core::variety::{VarietyRanges, RANGES};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v17_nofission_scene, game_v17_scene};
use sim_test::worldmodel::digest::state_digest;

/// `WorldRules::game_v17(0)` as Pass 11 left it (`31559232`), written out in full.
fn pass11_v17_seed0_literal() -> WorldRules {
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
            local_food_availability: false,
            temperament: None,
            routine: None,
        },
        society: SocietyRules {
            family: None,
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
                area_fall_floor_permille: 0,
                rain_wet_burn: false,
                floor_separate: false,
            }),
            exposure: Some(ExposureRule {
                damage_per_cold_hour: [9.0, 5.0, 6.0],
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
                cooldown_days: 42,
                min_age_days: 14,
                site_clearance: 16,
                stock_share: true,
                max_settlements: 4,
                settle_timeout_days: 2,
                remnant_members: 2,
                allowed_seasons: [true, true, false, false],
                hazard: None,
                pressure: None,
            }),
        },
        clock_offset_ticks: 480,
        seasons: Some(SeasonRule {
            days_per_season: 7,
            regen_permille: [675, 1125, 900, 450],
        }),
    }
}

/// `RANGES` as Pass 11 left it (shared by v16 and v17; Pass 12's v18 ranges
/// are a separate constant).
const PASS11_RANGES: VarietyRanges = VarietyRanges {
    starting_agents: (18, 30),
    berry_bushes: (9, 15),
    berry_capacity: (3, 5),
    deadwood_spots: (14, 26),
    deadwood_capacity: (5, 7),
    forest_permille: (220, 380),
    food_regen_permille: (850, 1150),
    wild_regions: (1, 3),
    region_bushes: (7, 11),
    region_deadwood: (8, 14),
};

#[test]
fn harness_game_v17_seed0_rules_are_the_pass11_literal() {
    let v17 = WorldRules::game_v17(0);
    let literal = pass11_v17_seed0_literal();
    assert_eq!(v17.agent, literal.agent);
    assert_eq!(v17.society.birth, literal.society.birth);
    assert_eq!(
        v17.society.shelter_planning,
        literal.society.shelter_planning
    );
    assert_eq!(v17.society.life_stage, literal.society.life_stage);
    assert_eq!(v17.society.stock_target, literal.society.stock_target);
    assert_eq!(v17.society.aging, literal.society.aging);
    assert_eq!(v17.society.meals, literal.society.meals);
    assert_eq!(v17.society.firewood, literal.society.firewood);
    assert_eq!(v17.society.exposure, literal.society.exposure);
    assert_eq!(v17.society.fission, literal.society.fission);
    assert_eq!(v17.seasons, literal.seasons);
    assert_eq!(v17, literal);
    // v17_nofission is v17 without the fission rule.
    let nofission = WorldRules::game_v17_nofission(0);
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

#[test]
fn harness_ranges_are_the_pass11_literal() {
    assert_eq!(RANGES.starting_agents, PASS11_RANGES.starting_agents);
    assert_eq!(RANGES.berry_bushes, PASS11_RANGES.berry_bushes);
    assert_eq!(RANGES.berry_capacity, PASS11_RANGES.berry_capacity);
    assert_eq!(RANGES.deadwood_spots, PASS11_RANGES.deadwood_spots);
    assert_eq!(RANGES.deadwood_capacity, PASS11_RANGES.deadwood_capacity);
    assert_eq!(RANGES.forest_permille, PASS11_RANGES.forest_permille);
    assert_eq!(
        RANGES.food_regen_permille,
        PASS11_RANGES.food_regen_permille
    );
    assert_eq!(RANGES.wild_regions, PASS11_RANGES.wild_regions);
    assert_eq!(RANGES.region_bushes, PASS11_RANGES.region_bushes);
    assert_eq!(RANGES.region_deadwood, PASS11_RANGES.region_deadwood);
    assert_eq!(RANGES, PASS11_RANGES);
}

type Pin = (u64, &'static str, u64, &'static str, u64, u64, u64);

/// (seed, fingerprint at 1,440, digest at 1,440, fingerprint at 40,320,
/// digest at 40,320, births by 40,320, fissions founded by 40,320) recorded on
/// `31559232`. Seeds 1 and 2 differ from v17_nofission at 40,320: their first
/// group left at 06:00 of display day 29 and is still walking (no founding yet).
const V17_PINS: [Pin; 4] = [
    (0, "5c744f9647246196", 0x710e_1ecb_f78d_0a3f, "1b8c15ee9a359b88", 0xabf9_9be8_5c22_6262, 22, 0),
    (1, "a21992a9805ef61a", 0xff44_03d6_6081_b52b, "d155c8143b2c29ef", 0xb482_26f8_dfe8_0fc0, 15, 0),
    (2, "dc9d81f1a9fa9099", 0x3fcf_bae2_00cf_bf23, "e2cd560145aa69be", 0x88fc_d721_733f_b468, 30, 0),
    (3, "60edafb5e9b68ec6", 0xff52_1d6a_e390_5302, "cf04f4f982581a40", 0x42cc_3ce2_47c1_3fbe, 8, 0),
];

/// As [`V17_PINS`] for `game_v17_nofission_scene` (fissions always 0).
const V17N_PINS: [Pin; 4] = [
    (0, "5c744f9647246196", 0x710e_1ecb_f78d_0a3f, "1b8c15ee9a359b88", 0xabf9_9be8_5c22_6262, 22, 0),
    (1, "a21992a9805ef61a", 0xff44_03d6_6081_b52b, "9c5b3dc2e3584f5e", 0xaabe_d199_f280_cb80, 15, 0),
    (2, "dc9d81f1a9fa9099", 0x3fcf_bae2_00cf_bf23, "cf979d598222efcb", 0x01a5_1cfb_c960_fa2d, 30, 0),
    (3, "60edafb5e9b68ec6", 0xff52_1d6a_e390_5302, "cf04f4f982581a40", 0x42cc_3ce2_47c1_3fbe, 8, 0),
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
fn harness_v17_fingerprints_unchanged() {
    check("game_v17", game_v17_scene, V17_PINS);
}

#[test]
fn harness_v17_nofission_fingerprints_unchanged() {
    check("game_v17_nofission", game_v17_nofission_scene, V17N_PINS);
}
