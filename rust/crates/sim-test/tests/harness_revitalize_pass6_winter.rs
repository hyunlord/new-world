//! Pass 6 Part 2: autumn for wood, a lean winter.
//!
//! Expected numbers come from `WorldRules::game_v12()` (Part 5 may retune it);
//! `harness_wood_target_formula_table` pins the formula with explicit literals.
use hecs::Entity;
use sim_core::components::{Agent, AgentState, Cold, Hunger, Inventory, ResourceKind, Settlement, Sleep, Social, Thirst};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{FirewoodRule, SeasonRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
use sim_systems::runtime::settlement::firewood::{burn_per_day, wood_target};
use sim_systems::runtime::settlement::FireSystem;

const HOME: (u32, u32) = (20, 20);
const SPRING: u64 = 0;
const SUMMER: u64 = 7;
const AUTUMN: u64 = 14;
const WINTER: u64 = 21;
const NEXT_SPRING: u64 = 28;

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

fn v12_rule() -> FirewoodRule {
    WorldRules::game_v12().society.firewood.expect("v12 keeps the firewood rule")
}

fn resources_at(rules: WorldRules, tick: u64) -> SimResources {
    let mut resources = SimResources::new(64, 64, MaterialRegistry::new());
    resources.rules = rules;
    resources.current_tick = tick;
    resources
}

/// One settlement at `HOME` holding `wood`, with `members` idle residents.
fn village(rules: WorldRules, tick: u64, wood: u32, members: usize) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
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
    engine.resources.settlements[&1].stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0)
}

/// Run one regrowth pulse at `tick`.
fn pulse(resources: &mut SimResources, tick: u64) {
    resources.current_tick = tick;
    let mut world = hecs::World::new();
    ResourceRegenSystem::new().tick(&mut world, resources);
}

/// The formula of section 7.2 for the current season of `resources`.
fn expected_target(rule: &FirewoodRule, season: usize, days_per_season: u32) -> f64 {
    let base = if season == 3 { rule.winter_base_days.unwrap_or(rule.base_days) } else { rule.base_days };
    let prep_season = season == 2 || (season == 1 && rule.winter_prep_in_summer);
    let prep_days = rule.winter_prep_days.unwrap_or(f64::from(days_per_season));
    base * burn_per_day(rule, season) + if rule.winter_prep && prep_season { prep_days * burn_per_day(rule, 3) } else { 0.0 }
}

#[test]
fn harness_wood_target_formula_table() {
    // The Pass 6 start values with explicit literals: spring 1.5 x 26, summer
    // 1.5 x 13, autumn 1.5 x 26 + 3 x 39, winter 4 x 39.
    let mut rules = WorldRules::game_v11();
    let rule = rules.society.firewood.as_mut().expect("firewood");
    rule.winter_prep_days = Some(3.0);
    rule.winter_prep_in_summer = false;
    rule.winter_base_days = Some(4.0);
    for (day, want) in [(SPRING, 39.0), (SUMMER, 19.5), (AUTUMN, 156.0), (WINTER, 156.0), (NEXT_SPRING, 39.0)] {
        let resources = resources_at(rules, tick_at(day, 720));
        assert_eq!(wood_target(&resources), Some(want), "display day {day}");
    }
    // Without seasons: base days of the spring burn only.
    rules.seasons = None;
    assert_eq!(wood_target(&resources_at(rules, tick_at(SUMMER, 720))), Some(39.0));
}

#[test]
fn harness_wood_target_by_season() {
    let rules = WorldRules::game_v12();
    let rule = v12_rule();
    let dps = rules.seasons.expect("v12 seasons").days_per_season;
    for (season, day) in [(0usize, SPRING), (1, SUMMER), (2, AUTUMN), (3, WINTER)] {
        let resources = resources_at(rules, tick_at(day, 720));
        assert_eq!(wood_target(&resources), Some(expected_target(&rule, season, dps)), "season {season}");
    }
    // Winter keeps `winter_base_days` of the winter burn; it exceeds one night.
    let winter = wood_target(&resources_at(rules, tick_at(WINTER, 720))).expect("target");
    assert!(winter > burn_per_day(&rule, 3));
}

#[test]
fn harness_autumn_only_prep() {
    let v12 = v12_rule();
    assert!(!v12.winter_prep_in_summer);
    let summer = wood_target(&resources_at(WorldRules::game_v12(), tick_at(SUMMER, 720))).expect("summer");
    let autumn = wood_target(&resources_at(WorldRules::game_v12(), tick_at(AUTUMN, 720))).expect("autumn");
    assert_eq!(summer, v12.base_days * burn_per_day(&v12, 1), "summer gathers for summer nights only");
    let prep = v12.winter_prep_days.expect("v12 prep days") * burn_per_day(&v12, 3);
    assert_eq!(autumn, v12.base_days * burn_per_day(&v12, 2) + prep);
    // v11 still prepares in summer with a full season of winter burn.
    let v11 = WorldRules::game_v11().society.firewood.expect("v11");
    let v11_summer = wood_target(&resources_at(WorldRules::game_v11(), tick_at(SUMMER, 720))).expect("v11 summer");
    assert_eq!(v11_summer, v11.base_days * burn_per_day(&v11, 1) + 7.0 * burn_per_day(&v11, 3));
}

#[test]
fn harness_winter_cap_buries_deadwood() {
    let cap = v12_rule().winter_deadwood_cap.expect("v12 winter cap");
    let spots = [((10, 10), 6u8), ((12, 10), 1), ((14, 10), 3)];
    let mut resources = resources_at(WorldRules::game_v12(), tick_at(WINTER, 480));
    for (spot, amount) in spots {
        resources.wood_source_max.insert(spot, 6);
        resources.wood_tiles.insert(spot, amount);
    }
    // Autumn's last pulse leaves the ground as it was.
    pulse(&mut resources, tick_at(WINTER - 1, 1320));
    assert_eq!(resources.wood_tiles[&(10, 10)], 6);
    // The first winter pulse lowers every spot above the cap to the cap.
    let mut winter = resources_at(WorldRules::game_v12(), 0);
    for (spot, amount) in spots {
        winter.wood_source_max.insert(spot, 6);
        winter.wood_tiles.insert(spot, amount);
    }
    pulse(&mut winter, tick_at(WINTER, 480));
    for (spot, amount) in spots {
        assert_eq!(winter.wood_tiles.get(&spot).copied().unwrap_or(0), amount.min(cap), "{spot:?}");
    }
    // A cap of 0 removes every spot from the ground; the registrations stay.
    let mut rules = WorldRules::game_v12();
    rules.society.firewood.as_mut().expect("firewood").winter_deadwood_cap = Some(0);
    let mut bare = resources_at(rules, 0);
    for (spot, amount) in spots {
        bare.wood_source_max.insert(spot, 6);
        bare.wood_tiles.insert(spot, amount);
    }
    pulse(&mut bare, tick_at(WINTER, 480));
    assert!(bare.wood_tiles.is_empty(), "no zero-valued tile is stored");
    assert_eq!(bare.wood_source_max.len(), 3);
    // With a cap of 0 every winter unit is lost.
    for step in 1..=12 {
        pulse(&mut bare, tick_at(WINTER, 480) + step * REGEN_INTERVAL);
    }
    assert!(bare.wood_tiles.is_empty());
    let per_day = u64::from(v12_rule().winter_regen_units_per_day.expect("regen"));
    assert_eq!(bare.tallies.winter_wood_lost, 13 * per_day / (1440 / REGEN_INTERVAL));
}

#[test]
fn harness_winter_spread_regrowth() {
    let rule = v12_rule();
    let per_day = u64::from(rule.winter_regen_units_per_day.expect("v12 winter regrowth"));
    let cap = rule.winter_deadwood_cap.expect("v12 winter cap");
    assert!(per_day > 0 && cap > 0, "the scenario needs regrowth and a positive cap");
    let spots = [(5u32, 9u32), (5, 12), (8, 3), (9, 9)];
    let mut resources = resources_at(WorldRules::game_v12(), 0);
    for spot in spots {
        resources.wood_source_max.insert(spot, 6);
    }
    let start = tick_at(WINTER, 480);
    let pulses_per_day = 1440 / REGEN_INTERVAL;
    let mut arrivals = Vec::new();
    let mut lost_at = Vec::new();
    let capacity = spots.len() as u64 * u64::from(cap);
    let total_pulses = (capacity + 2) * pulses_per_day / per_day + 1;
    for step in 0..total_pulses {
        let before = resources.wood_tiles.clone();
        let lost_before = resources.tallies.winter_wood_lost;
        pulse(&mut resources, start + step * REGEN_INTERVAL);
        // Exactly floor((step + 1) x per_day / 12) units have arrived or been lost.
        let landed: u64 = resources.wood_tiles.values().map(|v| u64::from(*v)).sum();
        assert_eq!(landed + resources.tallies.winter_wood_lost, (step + 1) * per_day / pulses_per_day, "pulse {step}");
        let changed: Vec<_> = resources.wood_tiles.iter().filter(|(k, v)| before.get(k) != Some(v)).map(|(k, _)| *k).collect();
        assert!(changed.len() <= 1, "one spot at a time");
        arrivals.extend(changed);
        if resources.tallies.winter_wood_lost > lost_before {
            lost_at.push(step);
        }
        assert!(resources.wood_tiles.values().all(|v| *v <= cap && *v > 0));
    }
    // Units arrive in key order, wrapping, one per spot until every spot holds the cap.
    let mut order: Vec<(u32, u32)> = spots.to_vec();
    order.sort();
    let expected: Vec<(u32, u32)> = (0..capacity as usize).map(|i| order[i % order.len()]).collect();
    assert_eq!(arrivals, expected);
    // Afterwards every unit finds no spot below the cap and is lost.
    assert_eq!(lost_at.len(), 2);
    assert_eq!(resources.tallies.winter_wood_lost, 2);
    // Picking up the second spot's wood: the next unit skips the full spots
    // after the cursor (the last spot placed), wraps and lands there.
    resources.wood_tiles.remove(&order[1]);
    let mut step = total_pulses;
    while !resources.wood_tiles.contains_key(&order[1]) {
        pulse(&mut resources, start + step * REGEN_INTERVAL);
        step += 1;
        assert!(step < total_pulses + pulses_per_day, "a unit arrives within a day");
    }
    assert_eq!(resources.wood_tiles[&order[1]], 1);
    assert_eq!(resources.tallies.winter_wood_lost, 2);
}

#[test]
fn harness_spring_restores_deadwood() {
    let rule = v12_rule();
    let spot = (10u32, 10u32);
    let mut resources = resources_at(WorldRules::game_v12(), 0);
    resources.wood_source_max.insert(spot, 6);
    resources.wood_tiles.insert(spot, 6);
    resources.wood_regen_acc = 500;
    // A whole winter of pulses: the Pass 5 accumulator is untouched.
    let first = tick_at(WINTER, 480);
    let last = tick_at(NEXT_SPRING, 0);
    let mut tick = first;
    while tick < last {
        pulse(&mut resources, tick);
        tick += REGEN_INTERVAL;
    }
    assert_eq!(resources.wood_regen_acc, 500);
    assert!(resources.wood_tiles.get(&spot).copied().unwrap_or(0) <= rule.winter_deadwood_cap.expect("cap"));
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
    let rule = v12_rule();
    assert!(rule.night_season_from_evening);
    // 02:00 after the last winter evening (display day 27): display day 28 is spring.
    let night = tick_at(NEXT_SPRING, 120);
    let winter_hour = u32::from(rule.burn_per_hour[3]);
    let (mut engine, members) = village(WorldRules::game_v12(), night, 10, 2);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_store(&engine), 10 - winter_hour, "the winter rate burns to the end of the night");
    assert!(engine.resources.fires[&1].lit);
    assert!(engine.world.get::<&Cold>(members[0]).is_err());
    // Out of wood that night: the members are cold.
    let (mut engine, members) = village(WorldRules::game_v12(), night, 0, 2);
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
    // v11 books the same night to spring (Pass 5): spring rate, no cold.
    let v11 = WorldRules::game_v11().society.firewood.expect("v11");
    let (mut engine, _) = village(WorldRules::game_v11(), night, 10, 1);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood_store(&engine), 10 - u32::from(v11.burn_per_hour[0]));
    let (mut engine, _) = village(WorldRules::game_v11(), night, 0, 1);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.world.query::<&Cold>().iter().count(), 0);
}

#[test]
fn harness_firewood_v11_unchanged() {
    let v11 = WorldRules::game_v11();
    // Pass 5 targets: 1.5 x burn plus a full season of winter burn in summer and autumn.
    for (day, want) in [(SPRING, 39.0), (SUMMER, 292.5), (AUTUMN, 312.0), (WINTER, 58.5)] {
        assert_eq!(wood_target(&resources_at(v11, tick_at(day, 720))), Some(want), "display day {day}");
    }
    // Pass 5 regrowth all year: no cap, no spread accumulator, nothing lost.
    let mut resources = resources_at(v11, 0);
    resources.wood_source_max.insert((10, 10), 6);
    resources.wood_tiles.insert((10, 10), 6);
    resources.wood_source_max.insert((12, 10), 6);
    for step in 0..24 {
        pulse(&mut resources, tick_at(WINTER, 480) + step * REGEN_INTERVAL);
    }
    assert_eq!(resources.wood_tiles[&(10, 10)], 6);
    assert_eq!(resources.wood_tiles[&(12, 10)], 6, "24 pulses x 0.5 refill an empty spot");
    assert_eq!((resources.wood_winter_acc, resources.wood_winter_cursor), (0, None));
    assert_eq!(resources.tallies.winter_wood_lost, 0);
    // The season rule of v11 is unchanged.
    assert_eq!(v11.seasons, Some(SeasonRule { days_per_season: 7, regen_permille: [750, 1250, 1000, 500] }));
}
