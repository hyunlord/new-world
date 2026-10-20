//! Merged in V8-0 (8.4) from `harness_revitalize_pass4_seasons.rs`,
//! `harness_revitalize_pass6_winter.rs`, `harness_revitalize_pass7_winter.rs`,
//! `harness_revitalize_pass8_winter.rs`; each source is one module below,
//! unchanged.

// From `harness_revitalize_pass4_seasons.rs`.
mod regrowth {
    //! Pass 4 Part 4: seasonal regrowth and a store that matters in winter.
    //!
    //! V8-0: rebased from `game_v10` (fixed regrowth [750, 1250, 1000, 500]) on
    //! the baseline, whose regrowth is that table scaled by the world's food-regen
    //! draw (`WorldVariety::draw`), so the expected values come from the rule.
    use sim_core::components::{Agent, Hunger, Settlement};
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::resource_regen::{season_at, ResourceRegenSystem};
    use sim_systems::runtime::settlement::food_economy::{
        members_need_per_day, stock_target, winter_supply_per_day,
    };

    const DAY: u64 = 1440;
    const OFFSET: u64 = 480;
    /// Regrowth pulses per day (`REGEN_INTERVAL` = 120 ticks).
    const PULSES: u64 = 12;

    /// Display day `d` (zero-based) starts at simulation tick `d * DAY - OFFSET`.
    fn day_start(d: u64) -> u64 {
        d * DAY - OFFSET
    }

    #[test]
    fn harness_season_regen_schedule() {
        // Seeds 0..8 draw food regen 875..1131 permille, so the four seasons'
        // rates differ from world to world (seed 1: [656, 1093, 875, 437]).
        for seed in 0..8u64 {
            let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
            let baseline = WorldRules::baseline(seed);
            engine.resources.rules.clock_offset_ticks = baseline.clock_offset_ticks;
            engine.resources.rules.seasons = baseline.seasons;
            let rule = baseline.seasons.expect("baseline seasons");
            assert_eq!(rule.days_per_season, 7);
            assert_eq!(OFFSET, baseline.clock_offset_ticks);
            engine.resources.set_food_source_max(5, 5, 4);
            let mut regen = ResourceRegenSystem::new();
            // Display days 1..=28 are full days (day 0 starts at 08:00); they cover a
            // whole year shifted by one day, so every season appears.
            let mut per_day = vec![0u64; 29];
            for tick in (day_start(1)..day_start(29)).step_by(120) {
                engine.resources.current_tick = tick;
                regen.tick(&mut engine.world, &mut engine.resources);
                assert!(
                    engine.resources.food_tiles.values().all(|v| *v > 0),
                    "seed {seed}: zero-valued food tile at {tick}"
                );
                let d = (tick + OFFSET) / DAY;
                per_day[d as usize] +=
                    u64::from(engine.resources.food_tiles.remove(&(5, 5)).unwrap_or(0));
            }
            // A day of a season grows 12 pulses x its permille, carried exactly:
            // the floor or the ceiling of 12 p / 1000 units.
            for d in 1..=28u64 {
                let season = ((d / 7) % 4) as usize;
                let p = u64::from(rule.regen_permille[season]);
                let lo = PULSES * p / 1000;
                let hi = (PULSES * p).div_ceil(1000);
                assert!(
                    (lo..=hi).contains(&per_day[d as usize]),
                    "seed {seed}: display day {d} (season {season}) grew {}, want {lo}..={hi}",
                    per_day[d as usize]
                );
            }
            // A whole season (display days 7..=13 summer, 14..=20 autumn, 21..=27
            // winter) grows 84 pulses x p, within one unit of the carried remainder.
            let mut totals = [0u64; 4];
            for (season, first) in [(1usize, 7u64), (2, 14), (3, 21)] {
                totals[season] = per_day[first as usize..first as usize + 7].iter().sum();
                let exact = 7 * PULSES * u64::from(rule.regen_permille[season]);
                assert!(
                    (exact / 1000..=exact / 1000 + 1).contains(&totals[season]),
                    "seed {seed}: season {season} grew {}, want {} or one more",
                    totals[season],
                    exact / 1000
                );
            }
            assert!(
                totals[1] > totals[2] && totals[2] > totals[3],
                "seed {seed}: summer > autumn > winter, got {totals:?}"
            );
            assert_eq!(
                engine.resources.tallies.regen_lost_at_cap, 0,
                "a drained bush never reaches its cap"
            );
        }
    }

    #[test]
    fn harness_season_target_includes_winter() {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = WorldRules::baseline(1);
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (20, 20);
        for _ in 0..42 {
            let entity = engine.spawn_agent(20, 20);
            let id = engine.world.get::<&Agent>(entity).expect("agent").id;
            engine
                .world
                .insert_one(entity, Hunger::new(0.0, 0.05))
                .expect("hunger");
            settlement.add_member_agent(id);
        }
        engine.resources.settlements.insert(1, settlement);
        for x in 21..33 {
            engine.resources.set_food_source_max(x, 22, 4);
        }
        let rule = engine
            .resources
            .rules
            .society
            .stock_target
            .expect("target rule");
        for d in 0..28u64 {
            engine.resources.current_tick = day_start(d.max(1)) + 600;
            if d == 0 {
                engine.resources.current_tick = 0;
            }
            let (season, _) = season_at(&engine.resources).expect("seasons on");
            let need = members_need_per_day(&engine.world, &engine.resources, 1);
            let winter = winter_supply_per_day(&engine.resources, 1, 16);
            let base = rule.base_days * need;
            let want = if season == 1 || season == 2 {
                base + 7.0 * (need - winter).max(0.0)
            } else {
                base
            };
            let got = stock_target(&engine.world, &engine.resources, 1).expect("target");
            assert_eq!(got, want, "display day {d}, season {season}");
            // Independent numbers (seed 1, winter regrowth 437 permille): 42 x 2.4
            // = 100.8 need, 2.5 days = 252, winter supply 12 x 12 x 0.437 = 62.928,
            // so summer/autumn add 7 x 37.872 = 265.104.
            let fixed = if season == 1 || season == 2 {
                252.0 + 265.104
            } else {
                252.0
            };
            assert!(
                (got - fixed).abs() < 1e-3,
                "display day {d}: target {got}, expected {fixed}"
            );
            if season == 1 || season == 2 {
                assert!(
                    got > base,
                    "summer/autumn target must include the winter shortfall"
                );
            }
        }
        assert!((winter_supply_per_day(&engine.resources, 1, 16) - 62.928).abs() < 1e-9);
    }
}

// From `harness_revitalize_pass6_winter.rs`.
mod lean_winter {
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
    fn village(
        rules: WorldRules,
        tick: u64,
        wood: u32,
        members: usize,
    ) -> (SimEngine, Vec<Entity>) {
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
            + if prep_season {
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
}

// From `harness_revitalize_pass7_winter.rs`.
mod severity {
    //! Pass 7 Part 2: winters that differ, trips that find wood.
    //!
    //! Each winter draws a severity (mild, normal, harsh) from a pure hash of the
    //! world seed and the year; it sets the night burn and the winter deadwood
    //! fall. A deadwood spot takes no more claims than the units it holds, and a
    //! gatherer who has arrived keeps its claim until the pickup. Expected numbers
    //! come from `WorldRules::baseline(SEED)` (V8-0; was `game_v13`);
    //! `harness_winter_severity_table` pins the draw with explicit literals. The
    //! baseline also changes each winter night's burn by a drawn -2/0/+2 and each
    //! winter day's fall by a drawn -2..=+2 (Pass 8), so the burn and fall
    //! assertions add the night's or day's draw to the severity's value.
    use std::collections::BTreeSet;

    use hecs::Entity;
    use sim_core::components::{
        Agent, AgentState, Hunger, Inventory, Position, ResourceKind, SeekTarget, Settlement,
        Sleep, Social, TargetKind, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{FirewoodRule, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine, SimResources};
    use sim_systems::runtime::agent::MovementRng;
    use sim_systems::runtime::decision::AgentDecisionSystem;
    use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
    use sim_systems::runtime::settlement::firewood::{
        burn_hours, night_burn_delta, severity_hash, urgent_with, winter_burn_per_hour,
        winter_fall_delta, winter_severity, winter_severity_at, wood_target,
    };
    use sim_systems::runtime::settlement::FireSystem;

    const HOME: (u32, u32) = (20, 20);
    /// World seed of the fixture rules (climate 1: severity weights 350 / 400 /
    /// 250) and of the fixtures' night and fall draws.
    const SEED: u64 = 1;
    const AUTUMN_DAY: u64 = 20;
    /// Display day 22: a winter day of year 0 (seven-day seasons; winter is days 21–27).
    const WINTER_DAY: u64 = 22;

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

    /// The baseline with every winter forced to `level` (a one-hot weight vector).
    fn rules_at(level: usize) -> WorldRules {
        let mut rules = rules();
        if let Some(rule) = rules.society.firewood.as_mut() {
            let mut weights = [0u16; 3];
            weights[level] = 1000;
            rule.winter_severity_permille = Some(weights);
        }
        rules
    }

    /// A seed whose year-0 and year-1 winters draw `levels` under the fixture's weights.
    fn seed_with(levels: [u8; 2]) -> u64 {
        let rule = baseline_rule();
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        (0..10_000)
            .find(|seed| {
                resources.world_seed = *seed;
                winter_severity(&resources, &rule, 0) == levels[0]
                    && winter_severity(&resources, &rule, 1) == levels[1]
            })
            .expect("a seed with these winters")
    }

    /// One settlement at `HOME` holding `wood`, `adults` idle satisfied members on
    /// the formation tile, and deadwood `spots` holding the given amounts.
    fn village(
        rules: WorldRules,
        tick: u64,
        wood: u32,
        adults: usize,
        spots: &[((u32, u32), u8)],
    ) -> (SimEngine, Vec<Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = rules;
        engine.resources.world_seed = SEED;
        engine.resources.current_tick = tick;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        settlement.store(ResourceKind::Wood, wood);
        let mut entities = Vec::new();
        for index in 0..adults {
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
        for &(spot, amount) in spots {
            engine.resources.wood_tiles.insert(spot, amount);
            engine.resources.wood_source_max.insert(spot, 6);
        }
        (engine, entities)
    }

    fn wood_store(engine: &SimEngine) -> u32 {
        engine.resources.settlements[&1]
            .stockpile
            .get(&ResourceKind::Wood)
            .copied()
            .unwrap_or(0)
    }

    /// Wood burned by one hourly fire update at `tick`.
    fn burned_at(rules: WorldRules, tick: u64) -> u32 {
        let (mut engine, _) = village(rules, tick, 100, 0, &[]);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        100 - wood_store(&engine)
    }

    fn targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
        entities
            .iter()
            .map(|e| {
                let state = *engine.world.get::<&AgentState>(*e).expect("state");
                matches!(
                    state,
                    AgentState::Seeking {
                        target: TargetKind::GatherWood
                    }
                )
                .then(|| engine.world.get::<&SeekTarget>(*e).ok().map(|s| s.tile))
                .flatten()
            })
            .collect()
    }

    #[test]
    fn harness_winter_severity_is_a_pure_draw() {
        let rule = baseline_rule();
        let weights = rule
            .winter_severity_permille
            .expect("the baseline draws winter severity");
        assert_eq!(weights, [350, 400, 250]);
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules = rules();
        resources.world_seed = 7;
        // Same seed and year, same level; the draw reads no state but the seed.
        let first = winter_severity(&resources, &rule, 3);
        resources.current_tick = 99_999;
        assert_eq!(winter_severity(&resources, &rule, 3), first);
        // Over 10,000 years of one seed each level is within 2 % of its weight.
        let mut counts = [0u32; 3];
        for year in 0..10_000 {
            counts[usize::from(winter_severity(&resources, &rule, year))] += 1;
        }
        for (count, weight) in counts.iter().zip(weights) {
            let share = f64::from(*count) / 10_000.0;
            assert!(
                (share - f64::from(weight) / 1000.0).abs() <= 0.02,
                "{counts:?} against {weights:?}"
            );
        }
        // The 128 probe seed-winters (seeds 0–31, years 0–3) are 128 distinct draws.
        let draws: BTreeSet<u64> = (0..32)
            .flat_map(|seed| (0..4).map(move |year| severity_hash(seed, year)))
            .collect();
        assert_eq!(draws.len(), 128);
        // Only winter hours carry a severity: from tick 0 to 07:00 of display
        // day 21 (the last autumn night ends then) every hour reads -1; from there
        // to 07:00 of display day 28 (the last winter night) every hour a level.
        for tick in (0..tick_at(21, 420)).step_by(60) {
            assert_eq!(winter_severity_at(&resources, tick), -1, "tick {tick}");
        }
        for tick in (tick_at(21, 420)..tick_at(28, 420)).step_by(60) {
            assert!(winter_severity_at(&resources, tick) >= 0, "tick {tick}");
        }
        assert_eq!(winter_severity_at(&resources, tick_at(28, 420)), -1);
    }

    #[test]
    fn harness_winter_severity_table() {
        // u = severity_hash(seed, year) % 1000 against weights 300 / 400 / 300.
        let rule = FirewoodRule {
            winter_severity_permille: Some([300, 400, 300]),
            ..baseline_rule()
        };
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        for (seed, year, u, level) in TABLE {
            assert_eq!(
                severity_hash(seed, year) % 1000,
                u,
                "seed {seed} year {year}"
            );
            resources.world_seed = seed;
            assert_eq!(
                winter_severity(&resources, &rule, year),
                level,
                "seed {seed} year {year}"
            );
        }
    }

    /// (seed, year, u, level) written out by hand from the formula of section 7.2.
    const TABLE: [(u64, u64, u64, u8); 8] = [
        (0, 0, 456, 1),
        (0, 1, 182, 0),
        (1, 0, 134, 0),
        (1, 1, 710, 2),
        (2, 0, 556, 1),
        (2, 1, 605, 1),
        (3, 0, 211, 0),
        (3, 1, 691, 1),
    ];

    #[test]
    fn harness_winter_burn_by_severity() {
        let rule = baseline_rule();
        let hours = f64::from(burn_hours(&rule));
        for level in 0..3 {
            let burn = rule.winter_severity_burn_per_hour[level];
            let (engine, _) = village(rules_at(level), tick_at(WINTER_DAY, 1320), 0, 0, &[]);
            let r = &engine.resources;
            // The fire at 22:00 of a winter evening burns the severity's rate
            // changed by that evening's draw (seed 1, display day 22: +0).
            let delta = night_burn_delta(r, &rule_of(r), WINTER_DAY);
            assert_eq!(delta, 0);
            assert_eq!(
                burned_at(rules_at(level), tick_at(WINTER_DAY, 1320)),
                u32::from(burn)
                    .saturating_add_signed(i32::from(delta))
                    .max(1),
                "level {level}"
            );
            assert_eq!(winter_burn_per_hour(r, &rule_of(r), r.current_tick), burn);
            // The winter target keeps `winter_base_days` of the severity's burn.
            let base = rule.winter_base_days.expect("baseline winter base");
            assert_eq!(
                wood_target(r),
                Some(base * f64::from(burn) * hours),
                "level {level}"
            );
            // Urgency compares wood in hand with tonight's severity burn.
            let tonight = (f64::from(burn) * hours) as u64;
            assert!(urgent_with(r, tonight - 1));
            assert!(!urgent_with(r, tonight));
        }
        // Outside winter the season's burn, whatever the weights say.
        assert_eq!(
            burned_at(rules_at(2), tick_at(AUTUMN_DAY, 1320)),
            u32::from(rule.burn_per_hour[2])
        );
    }

    fn rule_of(resources: &SimResources) -> FirewoodRule {
        resources.rules.society.firewood.expect("firewood")
    }

    #[test]
    fn harness_winter_fall_by_severity() {
        let rule = baseline_rule();
        for level in 0..3 {
            let spots: Vec<((u32, u32), u8)> = (0..20).map(|i| ((30 + i, 30), 0)).collect();
            let (mut engine, _) = village(rules_at(level), 0, 0, 0, &[]);
            for (spot, _) in &spots {
                engine.resources.wood_source_max.insert(*spot, 6);
            }
            // Twelve pulses: the whole of display day 22 (00:00 to 22:00), whose
            // fall is the severity's changed by the day's draw (floor 0).
            let start = tick_at(WINTER_DAY, 0);
            let mut world = hecs::World::new();
            for pulse in 0..12 {
                engine.resources.current_tick = start + pulse * REGEN_INTERVAL;
                ResourceRegenSystem::new().tick(&mut world, &mut engine.resources);
            }
            let fallen: u32 = engine
                .resources
                .wood_tiles
                .values()
                .map(|v| u32::from(*v))
                .sum();
            let delta = winter_fall_delta(&engine.resources, &rule, WINTER_DAY);
            assert!((-2..=2).contains(&delta), "delta {delta}");
            let want =
                i64::from(rule.winter_severity_regen_units_per_day[level]) + i64::from(delta);
            assert_eq!(i64::from(fallen), want.max(0), "level {level}");
            // The severities' falls stay ordered: mild 8, normal 6, harsh 4 (+-2).
            assert!(fallen >= 2, "level {level}: {fallen}");
        }
    }

    #[test]
    fn harness_last_night_keeps_its_winter_severity() {
        let rule = baseline_rule();
        // Year 0 harsh, year 1 mild: each winter's last night keeps its own winter.
        let seed = seed_with([2, 0]);
        let rules = rules();
        for (year, level) in [(0u64, 2usize), (1, 0)] {
            // 02:00 of the first spring day closes the last winter evening.
            let tick = tick_at(28 * (year + 1), 120);
            let (mut engine, _) = village(rules, tick, 100, 0, &[]);
            engine.resources.world_seed = seed;
            assert_eq!(winter_severity_at(&engine.resources, tick), level as i64);
            let delta = night_burn_delta(&engine.resources, &rule, 28 * (year + 1) - 1);
            FireSystem::new().tick(&mut engine.world, &mut engine.resources);
            assert_eq!(
                100 - wood_store(&engine),
                u32::from(rule.winter_severity_burn_per_hour[level])
                    .saturating_add_signed(i32::from(delta))
                    .max(1),
                "year {year}"
            );
            // 07:00 that morning: spring.
            assert_eq!(
                winter_severity_at(&engine.resources, tick_at(28 * (year + 1), 420)),
                -1
            );
        }
    }

    #[test]
    fn harness_autumn_prep_ignores_severity() {
        let rule = baseline_rule();
        let hours = f64::from(burn_hours(&rule));
        let prep = rule.winter_prep_days.expect("baseline autumn prep");
        let autumn = rule.base_days * f64::from(rule.burn_per_hour[2]) * hours
            + prep * f64::from(rule.burn_per_hour[3]) * hours;
        for level in 0..3 {
            let (engine, _) = village(rules_at(level), tick_at(AUTUMN_DAY, 480), 0, 0, &[]);
            assert_eq!(
                wood_target(&engine.resources),
                Some(autumn),
                "level {level}"
            );
        }
    }

    #[test]
    fn harness_claims_bounded_by_amount() {
        let rule = baseline_rule();
        assert!(rule.claims_bounded_by_amount);
        let start = u64::from(rule.urgent_start_minute.expect("urgent mode"));
        let cap = 2 * rule.max_gatherers_per_spot as usize;
        let urgent = tick_at(WINTER_DAY, start);
        // A one-unit spot takes one claim even under the urgent cap.
        let (mut engine, entities) = village(rules(), urgent, 0, cap + 1, &[((26, 20), 1)]);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            targets(&engine, &entities)
                .iter()
                .filter(|t| t.is_some())
                .count(),
            1
        );
        // A three-unit spot under the urgent cap takes the cap.
        let (mut engine, entities) = village(rules(), urgent, 0, cap + 1, &[((26, 20), 3)]);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            targets(&engine, &entities)
                .iter()
                .filter(|t| t.is_some())
                .count(),
            cap.min(3)
        );
        // A gatherer who has arrived (`Consuming{GatherWood}`) holds its claim: the
        // other member walks to the farther spot.
        let near = (22, 20);
        let far = (30, 20);
        let (mut engine, entities) = village(rules(), urgent, 0, 2, &[(near, 1), (far, 1)]);
        {
            let mut pos = engine
                .world
                .get::<&mut Position>(entities[0])
                .expect("position");
            (pos.x, pos.y) = near;
        }
        engine
            .world
            .insert_one(
                entities[0],
                AgentState::Consuming {
                    target: TargetKind::GatherWood,
                },
            )
            .expect("state");
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(targets(&engine, &entities)[1], Some(far));
        // Re-targeting obeys the same bound: a gatherer whose spot is gone finds
        // the only other one-unit spot claimed and stops.
        let (mut engine, entities) = village(rules(), urgent, 0, 2, &[(far, 1)]);
        for (entity, tile) in [(entities[0], near), (entities[1], far)] {
            engine
                .world
                .insert_one(
                    entity,
                    AgentState::Seeking {
                        target: TargetKind::GatherWood,
                    },
                )
                .expect("state");
            engine
                .world
                .insert_one(entity, SeekTarget { tile })
                .expect("target");
        }
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(targets(&engine, &entities), vec![None, Some(far)]);
        assert_eq!(engine.resources.tallies.wood_retargets, 1);
    }
}

// From `harness_revitalize_pass8_winter.rs`.
mod night_draws {
    //! Pass 8 Part 2: winters that vary night by night and day by day, and
    //! windfall that lands on the snow.
    //!
    //! Expected numbers are read from `WorldRules::baseline(SEED)` (V8-0; was
    //! `game_v14`, whose firewood draws the baseline keeps: night weights 250 /
    //! 500 / 250 step 2, fall spread 2; seed 1's climate has v14's severity
    //! weights 350 / 400 / 250); `harness_night_and_fall_draw_table` pins the
    //! draws with explicit literals computed by an independent Python
    //! implementation of section 7.2.
    use sim_core::components::{ResourceKind, Settlement};
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{FirewoodRule, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine, SimResources};
    use sim_systems::runtime::settlement::firewood::{
        burn_hours, night_burn_delta, night_rate, night_tick_at, severity_level_at, tonight_burn,
        urgent_with, winter_burn_per_hour, winter_fall_delta, wood_target,
    };

    /// World seed of the fixture rules (`WorldRules::baseline`).
    const SEED: u64 = 1;
    use sim_systems::runtime::settlement::FireSystem;

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

    fn resources(rules: WorldRules, seed: u64) -> SimResources {
        let mut r = SimResources::new(8, 8, MaterialRegistry::new());
        r.rules = rules;
        r.world_seed = seed;
        r
    }

    /// The `n` first winter display days (seven-day seasons: days 28y + 21 .. 28y + 27).
    fn winter_days(n: usize) -> Vec<u64> {
        (0..)
            .flat_map(|year: u64| (21..28).map(move |d| 28 * year + d))
            .take(n)
            .collect()
    }

    /// The baseline with every winter forced to `level` and the night weights `night`.
    fn rules_with(level: usize, night: [u16; 3], step: u16) -> WorldRules {
        let mut rules = rules();
        if let Some(rule) = rules.society.firewood.as_mut() {
            let mut weights = [0u16; 3];
            weights[level] = 1000;
            rule.winter_severity_permille = Some(weights);
            rule.winter_night_burn_permille = Some(night);
            rule.winter_night_burn_step = step;
        }
        rules
    }

    /// Wood burned by one hourly fire update at system tick `tick` from a store of 100.
    fn burned_at(rules: WorldRules, seed: u64, tick: u64) -> u32 {
        let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
        engine.resources.rules = rules;
        engine.resources.world_seed = seed;
        engine.resources.current_tick = tick;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.store(ResourceKind::Wood, 100);
        engine.resources.settlements.insert(1, settlement);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        100 - engine.resources.settlements[&1]
            .stockpile
            .get(&ResourceKind::Wood)
            .copied()
            .unwrap_or(0)
    }

    #[test]
    fn harness_night_burn_is_a_pure_draw() {
        let rule = baseline_rule();
        let weights = rule
            .winter_night_burn_permille
            .expect("the baseline draws night burns");
        let step = i16::try_from(rule.winter_night_burn_step).expect("small step");
        let mut r = resources(rules(), 11);
        // Same seed and evening, same delta, whatever the current tick.
        let first = night_burn_delta(&r, &rule, 23);
        r.current_tick = 777_777;
        assert_eq!(night_burn_delta(&r, &rule, 23), first);
        // Over 10,000 winter evenings each delta's share is within 0.02 of its weight.
        let mut counts = [0u32; 3];
        for day in winter_days(10_000) {
            let delta = night_burn_delta(&r, &rule, day);
            let index = [-step, 0, step]
                .iter()
                .position(|d| *d == delta)
                .expect("delta is -step, 0 or +step");
            counts[index] += 1;
        }
        for (count, weight) in counts.iter().zip(weights) {
            assert!(
                (f64::from(*count) / 10_000.0 - f64::from(weight) / 1000.0).abs() <= 0.02,
                "{counts:?} against {weights:?}"
            );
        }
        // Outside a winter there is no draw.
        for day in [0u64, 7, 14, 20, 28, 48] {
            assert_eq!(night_burn_delta(&r, &rule, day), 0, "day {day}");
        }
        assert_eq!(weights, [250, 500, 250]);
        assert_eq!(step, 2);
    }

    #[test]
    fn harness_fall_spread_is_a_pure_draw() {
        let rule = baseline_rule();
        let spread = i32::try_from(rule.winter_fall_spread).expect("small spread");
        let width = usize::try_from(2 * spread + 1).expect("width");
        let mut r = resources(rules(), 5);
        let first = winter_fall_delta(&r, &rule, 22);
        r.current_tick = 123_456;
        assert_eq!(winter_fall_delta(&r, &rule, 22), first);
        let mut counts = vec![0u32; width];
        for day in winter_days(10_000) {
            let delta = winter_fall_delta(&r, &rule, day);
            assert!((-spread..=spread).contains(&delta));
            counts[usize::try_from(delta + spread).expect("index")] += 1;
        }
        for count in &counts {
            assert!(
                (f64::from(*count) / 10_000.0 - 1.0 / width as f64).abs() <= 0.02,
                "{counts:?}"
            );
        }
        assert_eq!(winter_fall_delta(&r, &rule, 14), 0, "autumn has no draw");
        assert_eq!(spread, 2);
    }

    #[test]
    fn harness_night_and_fall_draw_table() {
        // (seed, day, u % 1000 of the night draw, night delta under 250/500/250 step 2,
        //  u % 5 of the fall draw, fall delta under spread 2), from the Python
        // re-implementation of section 7.2 (not from this crate).
        const TABLE: [(u64, u64, u64, i16, u64, i32); 16] = [
            (0, 21, 750, 2, 2, 0),
            (0, 24, 282, 0, 1, -1),
            (0, 27, 106, -2, 0, -2),
            (0, 49, 78, -2, 3, 1),
            (1, 21, 558, 0, 4, 2),
            (1, 24, 990, 2, 4, 2),
            (1, 27, 258, 0, 1, -1),
            (1, 49, 280, 0, 2, 0),
            (2, 21, 174, -2, 4, 2),
            (2, 24, 577, 0, 4, 2),
            (2, 27, 315, 0, 4, 2),
            (2, 49, 451, 0, 2, 0),
            (3, 21, 307, 0, 4, 2),
            (3, 24, 707, 0, 4, 2),
            (3, 27, 870, 2, 1, -1),
            (3, 49, 78, -2, 1, -1),
        ];
        // The baseline's draw parameters are the table's.
        let rule = baseline_rule();
        assert_eq!(rule.winter_night_burn_permille, Some([250, 500, 250]));
        assert_eq!(rule.winter_night_burn_step, 2);
        assert_eq!(rule.winter_fall_spread, 2);
        let rules = rules();
        for (seed, day, _u_night, night, _u_fall, fall) in TABLE {
            let r = resources(rules, seed);
            assert_eq!(
                night_burn_delta(&r, &rule, day),
                night,
                "seed {seed} day {day}"
            );
            assert_eq!(
                winter_fall_delta(&r, &rule, day),
                fall,
                "seed {seed} day {day}"
            );
        }
    }

    #[test]
    fn harness_villagers_do_not_see_tonights_draw() {
        // `wood_target`, `tonight_burn` and urgency read the severity's rate, never
        // the night's draw, at every hour of the first two winters; the fire's
        // `night_rate` does read it. Expected values come from the severity's rate
        // alone (`winter_burn_per_hour` of the booked tick).
        let rule = baseline_rule();
        let hours = f64::from(burn_hours(&rule));
        let base_days = rule.winter_base_days.expect("baseline winter base");
        for seed in 0..4 {
            let mut with = resources(rules(), seed);
            let mut changed = 0;
            for day in [
                21u64, 22, 23, 24, 25, 26, 27, 28, 49, 50, 51, 52, 53, 54, 55, 56,
            ] {
                for hour in 0..24 {
                    let tick = tick_at(day, hour * 60);
                    with.current_tick = tick;
                    let severity_rate = f64::from(winter_burn_per_hour(
                        &with,
                        &rule,
                        night_tick_at(&with, &rule, tick),
                    ));
                    let tonight = severity_rate * hours;
                    assert_eq!(
                        tonight_burn(&with, &rule),
                        tonight,
                        "seed {seed} day {day} hour {hour}"
                    );
                    if severity_level_at(&with, &rule, tick).is_some() {
                        let day_rate = f64::from(winter_burn_per_hour(&with, &rule, tick));
                        assert_eq!(
                            wood_target(&with),
                            Some(base_days * day_rate * hours),
                            "seed {seed} day {day} hour {hour}"
                        );
                    }
                    for in_hand in [0u64, 20, 39, 52, 80] {
                        assert_eq!(urgent_with(&with, in_hand), (in_hand as f64) < tonight);
                    }
                    let burning = night_rate(&with, &rule, tick) > 0;
                    changed += u32::from(
                        burning && f64::from(night_rate(&with, &rule, tick)) != severity_rate,
                    );
                }
            }
            // Half the winter evenings draw a change (weights 250 / 500 / 250):
            // measured 117 / 65 / 91 / 91 off-rate burn hours on seeds 0..4 of the
            // ~200 sampled; the bar is a quarter of the ~100 expected.
            assert!(
                changed >= 26,
                "the fire itself does see the draw (seed {seed}: {changed} hours)"
            );
        }
    }

    #[test]
    fn harness_night_rate_floor_is_one() {
        // Mild (2 an hour) with every night drawn -3: the floor keeps 1 an hour.
        let rules = rules_with(0, [1000, 0, 0], 3);
        let rule = rules.society.firewood.expect("firewood");
        let r = resources(rules, 4);
        let night = tick_at(22, 20 * 60);
        assert_eq!(winter_burn_per_hour(&r, &rule, night), 2);
        assert_eq!(night_burn_delta(&r, &rule, 22), -3);
        assert_eq!(night_rate(&r, &rule, night), 1);
        assert_eq!(burned_at(rules, 4, night), 1);
        // Outside the burn window the rate is 0 and nothing burns.
        assert_eq!(night_rate(&r, &rule, tick_at(22, 12 * 60)), 0);
        assert_eq!(burned_at(rules, 4, tick_at(22, 12 * 60)), 0);
        // Harsh (4) drawn +3 burns 7.
        let harsh = rules_with(2, [0, 0, 1000], 3);
        assert_eq!(burned_at(harsh, 4, night), 7);
    }

    #[test]
    fn harness_last_night_keeps_its_evening_draw() {
        // 02:00 of display day 28 (the first spring day of year one) belongs to the
        // last winter evening (day 27): it burns that evening's draw.
        let rules = rules();
        let rule = baseline_rule();
        let seed = (0..1000)
            .find(|seed| night_burn_delta(&resources(rules, *seed), &rule, 27) != 0)
            .expect("a seed whose last winter evening draws a change");
        let r = resources(rules, seed);
        let delta = i64::from(night_burn_delta(&r, &rule, 27));
        for (day, minute) in [(27u64, 18 * 60), (27, 23 * 60), (28, 2 * 60), (28, 6 * 60)] {
            let tick = tick_at(day, minute);
            let booked = night_tick_at(&r, &rule, tick);
            let base = i64::from(winter_burn_per_hour(&r, &rule, booked));
            assert!(
                severity_level_at(&r, &rule, booked).is_some(),
                "day {day} {minute}: a winter night"
            );
            let want = u32::try_from((base + delta).max(1)).expect("rate");
            assert_eq!(
                night_rate(&r, &rule, tick),
                want,
                "day {day} minute {minute}"
            );
            assert_eq!(burned_at(rules, seed, tick), want);
        }
        // 18:00 of day 28 is a spring evening: its own rate, no draw.
        let spring = tick_at(28, 18 * 60);
        assert_eq!(
            night_rate(&r, &rule, spring),
            u32::from(rule.burn_per_hour[0])
        );
    }
}
