//! Pass 8 Part 0: the frozen v13 references.
//!
//! The fingerprints of `game_v13_scene(seed)` after one day (1,440 ticks) were
//! recorded on the Pass 8 base `ce7acb25`; `game_v13()` is compared field by
//! field with a literal of its Pass 7 values. Pass 8 adds fields with values
//! that reproduce v13 and must never change either.
use sim_core::components::Cold;
use sim_core::rules::{
    AgentRules, AgingRule, BirthRule, EveningGatherRule, FirewoodRule, ForageRule, HousingCount, LifeStageRule,
    MealRule, NightRestRule, SeasonRule, SettlementAnchor, ShelterPlanningRule, SocialSeekRule, SocietyRules,
    StockTargetRule, SustainRule, WorldRules,
};
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v13_scene;

/// `game_v13()` as Pass 7 left it (`ce7acb25`), written out in full.
fn pass7_v13_literal() -> WorldRules {
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
            keyed_draws: false,
            local_food_availability: false,
            temperament: None,
        },
        society: SocietyRules {
            birth: BirthRule::Provisioned {
                cooldown_ticks: 720,
                food_cost: 10,
                max_mean_hunger: 60.0,
                max_mean_thirst: 60.0,
                sustain: Some(SustainRule { utilization: 0.81, reserve_days: 0.5, forage_radius: 16, capacity_weighted: false }),
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
                winter_severity_permille: Some([520, 280, 200]),
                winter_severity_burn_per_hour: [2, 3, 4],
                winter_severity_regen_units_per_day: [8, 6, 4],
                claims_bounded_by_amount: true,
                winter_night_burn_permille: None,
                winter_night_burn_step: 0,
                winter_fall_spread: 0,
                windfall_over_snow: false,
                rain_soak_permille: 0,
                regional_wood: false,
                area_fall_floor_permille: 0,
                rain_wet_burn: false,
                floor_separate: false,
            }),
            exposure: None,
            fission: None,
        },
        clock_offset_ticks: 480,
        seasons: Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] }),
    }
}

#[test]
fn harness_game_v13_rules_are_the_pass7_literal() {
    let v13 = WorldRules::game_v13();
    let literal = pass7_v13_literal();
    assert_eq!(v13.agent, literal.agent);
    assert_eq!(v13.society.birth, literal.society.birth);
    assert_eq!(v13.society.shelter_planning, literal.society.shelter_planning);
    assert_eq!(v13.society.settlement_anchor, literal.society.settlement_anchor);
    assert_eq!(v13.society.relief_radius, literal.society.relief_radius);
    assert_eq!(v13.society.life_stage, literal.society.life_stage);
    assert_eq!(v13.society.stock_target, literal.society.stock_target);
    assert_eq!(v13.society.aging, literal.society.aging);
    assert_eq!(v13.society.meals, literal.society.meals);
    assert_eq!(v13.society.firewood, literal.society.firewood);
    assert_eq!(v13.clock_offset_ticks, literal.clock_offset_ticks);
    assert_eq!(v13.seasons, literal.seasons);
    assert_eq!(v13, literal);
}

#[test]
fn harness_v13_fingerprints_unchanged() {
    // (seed, fingerprint, forage trips, wood trips) after 1,440 ticks.
    for (seed, expected, forage, wood) in [
        (0u64, "519530c0aaaae844", 83u64, 20u64),
        (1, "24d191164e3c4132", 69, 32),
        (2, "b44a71909b8615cc", 84, 28),
        (3, "c16d5b002bea6cc5", 55, 37),
    ] {
        let mut engine = game_v13_scene(seed);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(fingerprint(&engine), expected, "seed {seed}");
        assert_eq!(engine.resources.tallies.forage_trips, forage, "seed {seed}");
        assert_eq!(engine.resources.tallies.wood_trips, wood, "seed {seed}");
        // Day one is spring: nobody is cold.
        assert_eq!(engine.world.query::<&Cold>().iter().count(), 0);
    }
}
