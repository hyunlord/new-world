//! Pass 6 Part 2: autumn for wood, a lean winter.
//!
//! V8-0: rebased from `game_v12` on `WorldRules::baseline(SEED)`. The baseline
//! winter has a drawn severity (Pass 7) and night-by-night burns (Pass 8):
//! world seed 3's first winter is normal (3 an hour, as v12's single winter
//! rate), its last winter evening (display day 27) burns +2 an hour, and its
//! first winter day's deadwood fall is 8. Expected numbers come from the rule;
//! `harness_wood_target_formula_table` pins the formula with explicit literals.
use hecs::Entity;
use sim_core::components::{
    Agent, AgentState, Cold, Hunger, Inventory, ResourceKind, Settlement, Sleep, Social, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FirewoodRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
use sim_systems::runtime::settlement::firewood::{
    burn_hours, burn_per_day, night_burn_delta, winter_fall_at, winter_severity, wood_target,
};
use sim_systems::runtime::settlement::FireSystem;

const HOME: (u32, u32) = (20, 20);
/// World seed of the fixture rules and of the severity and night draws.
const SEED: u64 = 3;
const SPRING: u64 = 0;
const SUMMER: u64 = 7;
const AUTUMN: u64 = 14;
const WINTER: u64 = 21;
const NEXT_SPRING: u64 = 28;

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

fn resources_at(rules: WorldRules, tick: u64) -> SimResources {
    let mut resources = SimResources::new(64, 64, MaterialRegistry::new());
    resources.rules = rules;
    resources.world_seed = SEED;
    resources.current_tick = tick;
    resources
}

/// One settlement at `HOME` holding `wood`, with `members` idle residents.
fn village(rules: WorldRules, tick: u64, wood: u32, members: usize) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = SEED;
    engine.resources.current_tick = tick;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.store(ResourceKind::Wood, wood);
    let mut entities = Vec::new();
    for index in 0..members {
        let entity = engine.spawn_agent(HOME.0, HOME.1);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.05),
                    Thirst::new(0.0, 0.08),
                    Sleep::new(0.0, 0.03),
                    Social::new(0.0, 0.04),
                    Inventory::default(),
                    MovementRng::new(index as u64 + 1),
                ),
            )
            .expect("components");
        settlement.add_member_agent(id);
        entities.push(entity);
    }
    engine.resources.settlements.insert(1, settlement);
    (engine, entities)
}

fn wood_store(engine: &SimEngine) -> u32 {
    engine.resources.settlements[&1]
        .stockpile
        .get(&ResourceKind::Wood)
        .copied()
        .unwrap_or(0)
}

/// Run one regrowth pulse at `tick`.
fn pulse(resources: &mut SimResources, tick: u64) {
    resources.current_tick = tick;
    let mut world = hecs::World::new();
    ResourceRegenSystem::new().tick(&mut world, resources);
}

/// The formula of section 7.2 for `season` of the first year; a winter burns
/// at its drawn severity's rate (Pass 7), the autumn preparation at
/// `burn_per_hour[3]` (nobody knows the coming winter).
fn expected_target(rule: &FirewoodRule, season: usize, days_per_season: u32) -> f64 {
    let base = if season == 3 {
        rule.winter_base_days.unwrap_or(rule.base_days)
    } else {
        rule.base_days
    };
    let per_night = if season == 3 {
        let level = winter_severity(&resources_at(rules(), 0), rule, 0);
        f64::from(rule.winter_severity_burn_per_hour[usize::from(level)])
            * f64::from(burn_hours(rule))
    } else {
        burn_per_day(rule, season)
    };
    let prep_season = season == 2 || (season == 1 && rule.winter_prep_in_summer);
    let prep_days = rule.winter_prep_days.unwrap_or(f64::from(days_per_season));
    base * per_night
        + if rule.winter_prep && prep_season {
            prep_days * burn_per_day(rule, 3)
        } else {
            0.0
        }
}

#[test]
fn harness_wood_target_formula_table() {
    // The baseline values with explicit literals: spring 1.5 x 26, summer
    // 1.5 x 13, autumn 1.5 x 26 + 4 x 39 (prep at the normal winter rate),
    // winter 4 x 39 (seed 3's first winter is normal: 3 an hour x 13 hours).
    let rules = rules();
    let rule = baseline_rule();
    assert_eq!(rule.winter_prep_days, Some(4.0));
    assert!(!rule.winter_prep_in_summer);
    assert_eq!(rule.winter_base_days, Some(4.0));
    assert_eq!(winter_severity(&resources_at(rules, 0), &rule, 0), 1);
    for (day, want) in [
        (SPRING, 39.0),
        (SUMMER, 19.5),
        (AUTUMN, 195.0),
        (WINTER, 156.0),
        (NEXT_SPRING, 39.0),
    ] {
        let resources = resources_at(rules, tick_at(day, 720));
        assert_eq!(wood_target(&resources), Some(want), "display day {day}");
    }
}

#[test]
fn harness_wood_target_by_season() {
    let rules = rules();
    let rule = baseline_rule();
    let dps = rules.seasons.expect("baseline seasons").days_per_season;
    for (season, day) in [(0usize, SPRING), (1, SUMMER), (2, AUTUMN), (3, WINTER)] {
        let resources = resources_at(rules, tick_at(day, 720));
        assert_eq!(
            wood_target(&resources),
            Some(expected_target(&rule, season, dps)),
            "season {season}"
        );
    }
    // Winter keeps `winter_base_days` of the winter burn; it exceeds one night.
    let winter = wood_target(&resources_at(rules, tick_at(WINTER, 720))).expect("target");
    assert!(winter > burn_per_day(&rule, 3));
}

#[test]
fn harness_autumn_only_prep() {
    let rule = baseline_rule();
    assert!(!rule.winter_prep_in_summer);
    let summer = wood_target(&resources_at(rules(), tick_at(SUMMER, 720))).expect("summer");
    let autumn = wood_target(&resources_at(rules(), tick_at(AUTUMN, 720))).expect("autumn");
    assert_eq!(
        summer,
        rule.base_days * burn_per_day(&rule, 1),
        "summer gathers for summer nights only"
    );
    let prep = rule.winter_prep_days.expect("baseline prep days") * burn_per_day(&rule, 3);
    assert_eq!(autumn, rule.base_days * burn_per_day(&rule, 2) + prep);
    // Autumn prepares four nights of winter burn on top of its own 1.5 days.
    assert!(autumn >= summer + 150.0, "summer {summer}, autumn {autumn}");
}

#[test]
fn harness_winter_cap_buries_deadwood() {
    let rule = baseline_rule();
    let cap = rule.winter_deadwood_cap.expect("baseline winter cap");
    // Baseline `windfall_over_snow`: the snow falls at the first winter pulse
    // (00:00 of display day 21), not at every winter pulse (Pass 8).
    assert!(rule.windfall_over_snow);
    let first_winter_pulse = tick_at(WINTER, 0);
    assert_eq!(first_winter_pulse % REGEN_INTERVAL, 0);
    let spots = [((10, 10), 6u8), ((12, 10), 1), ((14, 10), 3)];
    let mut resources = resources_at(rules(), tick_at(WINTER, 480));
    for (spot, amount) in spots {
        resources.wood_source_max.insert(spot, 6);
        resources.wood_tiles.insert(spot, amount);
    }
    // Autumn's last pulse leaves the ground as it was.
    pulse(&mut resources, first_winter_pulse - REGEN_INTERVAL);
    assert_eq!(resources.wood_tiles[&(10, 10)], 6);
    // The first winter pulse lowers every spot above the cap to the cap; the
    // day's fall (8 on seed 3, 8/12 of a unit a pulse) lands nothing yet.
    let mut winter = resources_at(rules(), 0);
    for (spot, amount) in spots {
        winter.wood_source_max.insert(spot, 6);
        winter.wood_tiles.insert(spot, amount);
    }
    pulse(&mut winter, first_winter_pulse);
    for (spot, amount) in spots {
        assert_eq!(
            winter.wood_tiles.get(&spot).copied().unwrap_or(0),
            amount.min(cap),
            "{spot:?}"
        );
    }
    // A cap of 0 removes every spot from the ground; the registrations stay.
    let mut rules = rules();
    rules
        .society
        .firewood
        .as_mut()
        .expect("firewood")
        .winter_deadwood_cap = Some(0);
    let mut bare = resources_at(rules, 0);
    for (spot, amount) in spots {
        bare.wood_source_max.insert(spot, 6);
        bare.wood_tiles.insert(spot, amount);
    }
    pulse(&mut bare, first_winter_pulse);
    assert!(bare.wood_tiles.is_empty(), "no zero-valued tile is stored");
    assert_eq!(bare.wood_source_max.len(), 3);
    // With a cap of 0 every winter unit is lost: the twelve pulses of display
    // day 21 carry exactly that day's fall (seed 3: 8 units).
    for step in 1..12 {
        pulse(&mut bare, first_winter_pulse + step * REGEN_INTERVAL);
    }
    assert!(bare.wood_tiles.is_empty());
    let per_day = winter_fall_at(&bare, &rule, first_winter_pulse).expect("winter fall");
    assert_eq!(per_day, 8);
    assert_eq!(bare.tallies.winter_wood_lost, u64::from(per_day));
}

#[test]
fn harness_spring_restores_deadwood() {
    let rule = baseline_rule();
    let spot = (10u32, 10u32);
    let mut resources = resources_at(rules(), 0);
    resources.wood_source_max.insert(spot, 6);
    resources.wood_tiles.insert(spot, 6);
    resources.wood_regen_acc = 500;
    // A whole winter of pulses: the Pass 5 accumulator is untouched.
    let first = tick_at(WINTER, 0);
    let last = tick_at(NEXT_SPRING, 0);
    let mut tick = first;
    while tick < last {
        pulse(&mut resources, tick);
        tick += REGEN_INTERVAL;
    }
    assert_eq!(resources.wood_regen_acc, 500);
    assert!(
        resources.wood_tiles.get(&spot).copied().unwrap_or(0)
            <= rule.winter_deadwood_cap.expect("cap")
    );
    // Spring: the Pass 5 regrowth returns, regen_permille per pulse per spot, up to capacity.
    resources.wood_tiles.insert(spot, 1);
    let spring = tick_at(NEXT_SPRING, 480);
    pulse(&mut resources, spring);
    let gained = (500 + rule.regen_permille) / 1000;
    assert_eq!(resources.wood_tiles[&spot], 1 + gained as u8);
    let mut step = 1;
    while resources.wood_tiles[&spot] < 6 {
        pulse(&mut resources, spring + step * REGEN_INTERVAL);
        step += 1;
        assert!(step < 40, "the spot refills within a few days");
    }
    assert_eq!(resources.wood_tiles[&spot], 6);
}

#[test]
fn harness_last_winter_night_is_winter() {
    let rule = baseline_rule();
    assert!(rule.night_season_from_evening);
    // 02:00 after the last winter evening (display day 27): display day 28 is
    // spring. The night burns the winter evening's rate: seed 3's normal
    // winter (3 an hour) with day 27's draw of +2, i.e. 5, not spring's 2.
    let night = tick_at(NEXT_SPRING, 120);
    let level = winter_severity(&resources_at(rules(), 0), &rule, 0);
    let delta = night_burn_delta(&resources_at(rules(), 0), &rule, NEXT_SPRING - 1);
    let winter_hour = u32::from(rule.winter_severity_burn_per_hour[usize::from(level)])
        .saturating_add_signed(i32::from(delta));
    assert_eq!(winter_hour, 5);
    assert_ne!(winter_hour, u32::from(rule.burn_per_hour[0]));
    let (mut engine, members) = village(rules(), night, 10, 2);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        wood_store(&engine),
        10 - winter_hour,
        "the winter rate burns to the end of the night"
    );
    assert!(engine.resources.fires[&1].lit);
    assert!(engine.world.get::<&Cold>(members[0]).is_err());
    // Out of wood that night: the members are cold.
    let (mut engine, members) = village(rules(), night, 0, 2);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.fires[&1].lit);
    for member in &members {
        assert!(engine.world.get::<&Cold>(*member).is_ok());
    }
    assert_eq!(engine.resources.tallies.cold_member_hours, 2);
    // 07:00 closes the night: the spring day follows, nobody is cold.
    engine.resources.current_tick = tick_at(NEXT_SPRING, 420);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Cold>(members[0]).is_err());
}

#[test]
fn harness_first_winter_night_is_autumn() {
    // 02:00 of the first winter day (display day 21) closes the last autumn
    // evening's night: the autumn rate burns and nobody is cold (review round 1).
    let rule = baseline_rule();
    let night = tick_at(WINTER, 120);
    let (mut engine, _) = village(rules(), night, 10, 2);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_store(&engine), 10 - u32::from(rule.burn_per_hour[2]));
    let (mut engine, _) = village(rules(), night, 0, 2);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.fires[&1].lit);
    assert_eq!(engine.world.query::<&Cold>().iter().count(), 0);
    // 22:00 of that day is a winter night: out of wood, the members are cold.
    engine.resources.current_tick = tick_at(WINTER, 1320);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.world.query::<&Cold>().iter().count(), 2);
}
