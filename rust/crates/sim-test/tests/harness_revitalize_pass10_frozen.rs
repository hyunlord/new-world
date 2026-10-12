//! Pass 10 Part 0: the frozen v15 references.
//!
//! The fingerprints and state digests of `game_v15_scene(seed)` for seeds 0–3
//! after 1,440 ticks (day one, spring) and after 40,320 ticks (28 days:
//! winters, births and lifespans) were recorded on the Pass 10 base
//! `6aa4dba0`; `game_v15()` is compared field by field with a literal of its
//! Pass 9 values. Pass 10 adds fields with values that reproduce v15 and must
//! never change either.
use sim_core::components::Cold;
use sim_core::rules::{
    AgentRules, AgingRule, BirthRule, EveningGatherRule, FirewoodRule, ForageRule, HousingCount, LifeStageRule,
    MealRule, NightRestRule, SeasonRule, SettlementAnchor, ShelterPlanningRule, SocialSeekRule, SocietyRules,
    StockTargetRule, SustainRule, WorldRules,
};
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v15_scene;
use sim_test::worldmodel::digest::state_digest;

/// `game_v15()` as Pass 9 left it (`6aa4dba0`), written out in full.
fn pass9_v15_literal() -> WorldRules {
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
                winter_severity_permille: Some([350, 400, 250]),
                winter_severity_burn_per_hour: [2, 3, 4],
                winter_severity_regen_units_per_day: [8, 6, 4],
                claims_bounded_by_amount: true,
                winter_night_burn_permille: Some([250, 500, 250]),
                winter_night_burn_step: 2,
                winter_fall_spread: 2,
                windfall_over_snow: true,
            }),
            exposure: None,
        },
        clock_offset_ticks: 480,
        seasons: Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] }),
    }
}

#[test]
fn harness_game_v15_rules_are_the_pass9_literal() {
    let v15 = WorldRules::game_v15();
    let literal = pass9_v15_literal();
    assert_eq!(v15.agent, literal.agent);
    assert_eq!(v15.society.birth, literal.society.birth);
    assert_eq!(v15.society.shelter_planning, literal.society.shelter_planning);
    assert_eq!(v15.society.settlement_anchor, literal.society.settlement_anchor);
    assert_eq!(v15.society.relief_radius, literal.society.relief_radius);
    assert_eq!(v15.society.life_stage, literal.society.life_stage);
    assert_eq!(v15.society.stock_target, literal.society.stock_target);
    assert_eq!(v15.society.aging, literal.society.aging);
    assert_eq!(v15.society.meals, literal.society.meals);
    assert_eq!(v15.society.firewood, literal.society.firewood);
    assert_eq!(v15.clock_offset_ticks, literal.clock_offset_ticks);
    assert_eq!(v15.seasons, literal.seasons);
    assert_eq!(v15, literal);
}

/// (seed, fingerprint at 1,440, digest at 1,440, fingerprint at 40,320,
/// digest at 40,320, births by 40,320) recorded on `6aa4dba0`.
const PINS: [(u64, &str, u64, &str, u64, u64); 4] = [
    (0, "77bee3d5679db007", 0xcc11_abc6_7d2c_6049, "8f1794218ad25438", 0xb370_bddd_b2af_c8bc, 22),
    (1, "c5525d8391ac8219", 0xedb6_ef95_9a15_9755, "00a294390654d3c1", 0xf706_ed0c_84bb_27ae, 25),
    (2, "8c2856e0216d0854", 0xb15c_d381_ed43_3054, "3bfb64b71126273f", 0x3bc7_bdbe_4c9c_502e, 26),
    (3, "75ca217f9dab71aa", 0x5321_98b1_237b_f0bb, "8ff8fdaee9c07b4e", 0x08c4_caeb_a800_4258, 24),
];

#[test]
fn harness_v15_fingerprints_unchanged() {
    // Each seed runs 40,320 ticks; the four seeds run on scoped threads.
    let rows: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = PINS
            .iter()
            .map(|&(seed, ..)| {
                scope.spawn(move || {
                    let mut engine = game_v15_scene(seed);
                    for _ in 0..1440 {
                        engine.tick();
                    }
                    let day = (fingerprint(&engine), state_digest(&engine));
                    // Day one is spring: nobody is cold.
                    let cold = engine.world.query::<&Cold>().iter().count();
                    for _ in 1440..40_320 {
                        engine.tick();
                    }
                    (seed, day, cold, fingerprint(&engine), state_digest(&engine), engine.resources.tallies.births)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("seed thread")).collect()
    });
    let mut mismatches = Vec::new();
    for ((seed, (fp_day, dg_day), cold, fp_end, dg_end, births), pin) in rows.iter().zip(PINS) {
        assert_eq!(*seed, pin.0);
        assert_eq!(*cold, 0, "seed {seed}: cold on day one");
        let actual = (fp_day.as_str(), *dg_day, fp_end.as_str(), *dg_end, *births);
        if actual != (pin.1, pin.2, pin.3, pin.4, pin.5) {
            mismatches.push(format!("({seed}, {:?}, {:#018x}, {:?}, {:#018x}, {}),", actual.0, actual.1, actual.2, actual.3, actual.4));
        }
    }
    assert!(mismatches.is_empty(), "v15 pins differ:\n{}", mismatches.join("\n"));
}
