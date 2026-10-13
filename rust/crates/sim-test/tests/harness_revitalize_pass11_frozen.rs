//! Pass 11 Part 0: the frozen v16 references.
//!
//! The fingerprints and state digests of `game_v16_scene(seed)` for seeds 0–3
//! after 1,440 ticks (day one, spring) and after 40,320 ticks (28 days:
//! winters, births, lifespans and exposure) were recorded on the Pass 11 base
//! `8cca0334`; `game_v16_neutral()` is compared field by field with a literal
//! of its Pass 10 values, and `WorldVariety::draw` for seeds 0–3 with literals.
//! Pass 11 adds fields with values that reproduce v16 and must never change
//! either.
use sim_core::rules::{
    AgentRules, AgingRule, BirthRule, EveningGatherRule, ExposureRule, FirewoodRule, ForageRule, HousingCount,
    LifeStageRule, MealRule, NightRestRule, SeasonRule, SettlementAnchor, ShelterPlanningRule, SocialSeekRule,
    SocietyRules, StockTargetRule, SustainRule, WorldRules,
};
use sim_core::variety::WorldVariety;
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v16_scene;
use sim_test::worldmodel::digest::state_digest;

/// `game_v16_neutral()` as Pass 10 left it (`14372ea2`), written out in full.
fn pass10_v16_neutral_literal() -> WorldRules {
    WorldRules {
        agent: AgentRules {
            move_interval_ticks: 8,
            idle_pause_chance: 0.4,
            idle_tether_radius: Some(8),
            social_seek: Some(SocialSeekRule { max_chase_ticks: 240, retry_cooldown_ticks: 480 }),
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
            evening_gather: Some(EveningGatherRule { start_minute: 1080, end_minute: 1260, tether_radius: 3 }),
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
        },
        society: SocietyRules {
            birth: BirthRule::Provisioned {
                cooldown_ticks: 720,
                food_cost: 10,
                max_mean_hunger: 60.0,
                max_mean_thirst: 60.0,
                sustain: Some(SustainRule { utilization: 0.78, reserve_days: 0.5, forage_radius: 16, capacity_weighted: false }),
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
            stock_target: Some(StockTargetRule { base_days: 2.5, winter_prep: true, hoard_cap_factor: 1.5 }),
            aging: Some(AgingRule {
                elder_age_ticks: 66_240,
                lifespan_mean_ticks: 83_520,
                lifespan_spread_ticks: 11_520,
                founder_age_min_ticks: 8_640,
                founder_age_max_ticks: 57_600,
                founder_min_remaining_ticks: 4_320,
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
                winter_severity_permille: Some([350, 400, 250]),
                winter_severity_burn_per_hour: [2, 3, 4],
                winter_severity_regen_units_per_day: [8, 6, 4],
                claims_bounded_by_amount: true,
                winter_night_burn_permille: Some([250, 500, 250]),
                winter_night_burn_step: 2,
                winter_fall_spread: 2,
                windfall_over_snow: true,
                rain_soak_permille: 0,
                regional_wood: false,
                area_fall_floor_permille: 0,
                rain_wet_burn: false,
            }),
            exposure: Some(ExposureRule { damage_per_cold_hour: [10.0, 4.0, 5.0], heal_block_hours: 2, hunger_weight: 0.0, use_hardiness: false }),
            fission: None,
        },
        clock_offset_ticks: 480,
        seasons: Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] }),
    }
}

#[test]
fn harness_game_v16_neutral_rules_are_the_pass10_literal() {
    let v16n = WorldRules::game_v16_neutral();
    let literal = pass10_v16_neutral_literal();
    assert_eq!(v16n.agent, literal.agent);
    assert_eq!(v16n.society.birth, literal.society.birth);
    assert_eq!(v16n.society.shelter_planning, literal.society.shelter_planning);
    assert_eq!(v16n.society.settlement_anchor, literal.society.settlement_anchor);
    assert_eq!(v16n.society.relief_radius, literal.society.relief_radius);
    assert_eq!(v16n.society.life_stage, literal.society.life_stage);
    assert_eq!(v16n.society.stock_target, literal.society.stock_target);
    assert_eq!(v16n.society.aging, literal.society.aging);
    assert_eq!(v16n.society.meals, literal.society.meals);
    assert_eq!(v16n.society.firewood, literal.society.firewood);
    assert_eq!(v16n.society.exposure, literal.society.exposure);
    assert_eq!(v16n.clock_offset_ticks, literal.clock_offset_ticks);
    assert_eq!(v16n.seasons, literal.seasons);
    assert_eq!(v16n, literal);
}

/// `WorldVariety::draw(seed)` for seeds 0–3 as Pass 10 left it (slots 0..7;
/// Pass 11's wild-region slots 8..10 are new and v16 never reads them, so the
/// pin compares `draw(seed).without_regions()`).
const DRAWS: [(u64, WorldVariety); 4] = [
    (0, WorldVariety { starting_agents: 23, berry_bushes: 14, berry_capacity: 4, deadwood_spots: 20, deadwood_capacity: 5, forest_permille: 241, climate: 2, food_regen_permille: 900, wild_regions: 0, region_bushes: 0, region_deadwood: 0 }),
    (1, WorldVariety { starting_agents: 23, berry_bushes: 11, berry_capacity: 5, deadwood_spots: 18, deadwood_capacity: 5, forest_permille: 239, climate: 1, food_regen_permille: 875, wild_regions: 0, region_bushes: 0, region_deadwood: 0 }),
    (2, WorldVariety { starting_agents: 21, berry_bushes: 14, berry_capacity: 4, deadwood_spots: 14, deadwood_capacity: 7, forest_permille: 259, climate: 1, food_regen_permille: 903, wild_regions: 0, region_bushes: 0, region_deadwood: 0 }),
    (3, WorldVariety { starting_agents: 26, berry_bushes: 9, berry_capacity: 5, deadwood_spots: 23, deadwood_capacity: 7, forest_permille: 299, climate: 2, food_regen_permille: 890, wild_regions: 0, region_bushes: 0, region_deadwood: 0 }),
];

#[test]
fn harness_world_variety_draws_are_the_pass10_literals() {
    for (seed, literal) in DRAWS {
        let v = WorldVariety::draw(seed).without_regions();
        assert_eq!(v.starting_agents, literal.starting_agents, "seed {seed}");
        assert_eq!(v.berry_bushes, literal.berry_bushes, "seed {seed}");
        assert_eq!(v.berry_capacity, literal.berry_capacity, "seed {seed}");
        assert_eq!(v.deadwood_spots, literal.deadwood_spots, "seed {seed}");
        assert_eq!(v.deadwood_capacity, literal.deadwood_capacity, "seed {seed}");
        assert_eq!(v.forest_permille, literal.forest_permille, "seed {seed}");
        assert_eq!(v.climate, literal.climate, "seed {seed}");
        assert_eq!(v.food_regen_permille, literal.food_regen_permille, "seed {seed}");
        assert_eq!(v, literal, "seed {seed}");
    }
}

/// (seed, fingerprint at 1,440, digest at 1,440, fingerprint at 40,320,
/// digest at 40,320, births by 40,320) recorded on `8cca0334`.
const PINS: [(u64, &str, u64, &str, u64, u64); 4] = [
    (0, "f2a59cc0340d4cf5", 0x8e24_1bc0_ba81_fa31, "01429a8504dc720d", 0xd2e9_100f_603f_ad61, 22),
    (1, "1a0f4e457a6c7dd0", 0xf38b_f5c0_1b7e_1fb0, "62c89595e0926242", 0xd13c_186b_b443_8434, 15),
    (2, "fbe0252e1eec90ba", 0x88e0_4e15_61d3_9fd6, "795d4752ea5c793e", 0x7677_5eb9_8f97_2aa2, 30),
    (3, "b9a720aaff4262d4", 0xd2a1_b519_1004_a7f3, "34dd9a1152bb0940", 0x757e_cadc_5d8e_f288, 8),
];

#[test]
fn harness_v16_fingerprints_unchanged() {
    // Each seed runs 40,320 ticks; the four seeds run on scoped threads.
    let rows: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = PINS
            .iter()
            .map(|&(seed, ..)| {
                scope.spawn(move || {
                    let mut engine = game_v16_scene(seed);
                    for _ in 0..1440 {
                        engine.tick();
                    }
                    let day = (fingerprint(&engine), state_digest(&engine));
                    for _ in 1440..40_320 {
                        engine.tick();
                    }
                    (seed, day, fingerprint(&engine), state_digest(&engine), engine.resources.tallies.births)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("seed thread")).collect()
    });
    let mut mismatches = Vec::new();
    for ((seed, (fp_day, dg_day), fp_end, dg_end, births), pin) in rows.iter().zip(PINS) {
        assert_eq!(*seed, pin.0);
        let actual = (fp_day.as_str(), *dg_day, fp_end.as_str(), *dg_end, *births);
        if actual != (pin.1, pin.2, pin.3, pin.4, pin.5) {
            mismatches.push(format!("({seed}, {:?}, {:#018x}, {:?}, {:#018x}, {}),", actual.0, actual.1, actual.2, actual.3, actual.4));
        }
    }
    assert!(mismatches.is_empty(), "v16 pins differ:\n{}", mismatches.join("\n"));
}

