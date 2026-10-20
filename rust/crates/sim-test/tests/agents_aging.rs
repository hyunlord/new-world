//! Merged in V8-0 (8.4) from `harness_revitalize_pass4_aging.rs`,
//! `harness_revitalize_pass6_elders.rs`; each source is one module below,
//! unchanged.

// From `harness_revitalize_pass4_aging.rs`.
mod lifespans {
    //! Pass 4 Part 3: deterministic lifespans, elders and old-age replacement.
    use sim_core::components::{
        Agent, AgentId, AgentState, Elder, Life, Lineage, Position, TargetKind,
    };
    use sim_core::rules::WorldRules;
    use sim_systems::runtime::agent::aging::{
        keyed_founder_life, keyed_newborn_life, newborn_life,
    };
    use sim_systems::worldgen::WorldGenConfig;
    use sim_test::baseline_engine;
    use std::collections::BTreeSet;

    const DAY: u64 = 1440;

    fn rule() -> sim_core::rules::AgingRule {
        // The live aging rule (R1, exact: the lifespan fields are unchanged since v10).
        WorldRules::baseline(7)
            .society
            .aging
            .expect("baseline aging rule")
    }

    #[test]
    fn harness_aging_lifespans_bounded_and_deterministic() {
        let rule = rule();
        let (low, high) = (
            rule.lifespan_mean_ticks - rule.lifespan_spread_ticks,
            rule.lifespan_mean_ticks + rule.lifespan_spread_ticks,
        );
        let mut total = 0u64;
        let mut keyed_total = 0u64;
        for id in 0..1000u64 {
            let life = newborn_life(20_261_010, id, 5_000, &rule);
            assert!(
                (low..=high).contains(&life.lifespan_ticks),
                "id {id}: {}",
                life.lifespan_ticks
            );
            assert_eq!(
                life,
                newborn_life(20_261_010, id, 5_000, &rule),
                "twin draw differs for {id}"
            );
            assert_eq!(life.born_tick, 5_000);
            total += life.lifespan_ticks;
            // The live (keyed, `keyed_draws` = true) draw obeys the same band and
            // does not depend on the birth tick.
            let keyed = keyed_newborn_life(20_261_010, id, 5_000, 0, &rule);
            assert!(
                (low..=high).contains(&keyed.lifespan_ticks),
                "keyed id {id}"
            );
            assert_eq!(
                keyed.lifespan_ticks,
                keyed_newborn_life(20_261_010, id, 5_001, 0, &rule).lifespan_ticks
            );
            keyed_total += keyed.lifespan_ticks;
        }
        let keyed_mean_days = keyed_total as f64 / 1000.0 / DAY as f64;
        assert!(
            (57.0..=59.0).contains(&keyed_mean_days),
            "keyed mean lifespan {keyed_mean_days} days"
        );
        let mean_days = total as f64 / 1000.0 / DAY as f64;
        assert!(
            (57.0..=59.0).contains(&mean_days),
            "mean lifespan {mean_days} days"
        );
    }

    #[test]
    fn harness_aging_founders() {
        let rule = rule();
        for seed in 0..8u64 {
            let mut engine = baseline_engine(seed);
            let lives: Vec<(Life, u64)> = engine
                .world
                .query::<(&Agent, &Life, &Lineage)>()
                .iter()
                .map(|(_, (_, l, lineage))| (*l, lineage.key))
                .collect();
            // Baseline founder counts are the world's draw (`WorldVariety::draw`),
            // no longer a fixed 24.
            let founders = WorldGenConfig::baseline(seed).starting_agents as usize;
            let (min, max) = sim_core::variety::RANGES.starting_agents;
            assert!(
                (min as usize..=max as usize).contains(&founders),
                "seed {seed}: {founders} founders"
            );
            assert_eq!(
                lives.len(),
                founders,
                "every founder carries Life (seed {seed})"
            );
            for (life, _) in &lives {
                let age = (-life.born_tick) as u64;
                assert!((6 * DAY..=40 * DAY).contains(&age), "founder age {age}");
                assert!(
                    life.lifespan_ticks >= age + 3 * DAY,
                    "founder would die before day 3"
                );
            }
            // Baseline founders draw their life from their lineage key (keyed draws).
            for (life, key) in &lives {
                assert_eq!(*life, keyed_founder_life(seed, *key, 0, &rule));
            }
            if seed == 0 {
                for _ in 0..3 * DAY {
                    engine.tick();
                }
                assert_eq!(engine.resources.tallies.deaths_old_age, 0);
            }
        }
    }

    #[test]
    fn harness_aging_old_age_death_and_replacement() {
        // Baseline seed 1 (R3; the only AgingSystem-tick test). Measured: the first
        // old-age death at tick 23,401 (day 16), a birth on the next tick, and over the
        // full 60 days 28 old-age deaths and 37 births with no starvation or
        // dehydration. The run stops once a birth follows the first old-age
        // death to stay short.
        let mut engine = baseline_engine(1);
        let mut alive: BTreeSet<AgentId> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
        let mut first_old_age = None;
        let mut birth_after = false;
        let mut cleaned = 0;
        for _ in 0..60 * DAY {
            if birth_after {
                break;
            }
            let births = engine.resources.tallies.births;
            let old_age = engine.resources.tallies.deaths_old_age;
            engine.tick();
            let now: BTreeSet<AgentId> = engine
                .world
                .query::<&Agent>()
                .iter()
                .map(|(_, a)| a.id)
                .collect();
            if engine.resources.tallies.deaths_old_age > old_age {
                first_old_age.get_or_insert(engine.current_tick());
                for gone in alive.difference(&now) {
                    assert!(
                        !engine.resources.bed_of.contains_key(gone),
                        "bed kept for dead {gone}"
                    );
                    assert!(
                        engine
                            .resources
                            .settlements
                            .values()
                            .all(|s| !s.member_agents.contains(gone)),
                        "roster kept dead {gone}"
                    );
                    cleaned += 1;
                }
            }
            if first_old_age.is_some() && engine.resources.tallies.births > births {
                birth_after = true;
            }
            alive = now;
        }
        assert!(first_old_age.is_some(), "no old-age death in 60 days");
        assert!(cleaned >= 1);
        assert!(birth_after, "no birth after the first old-age death");
        assert_eq!(engine.resources.tallies.deaths_starvation, 0);
        assert_eq!(engine.resources.tallies.deaths_dehydration, 0);
    }

    #[test]
    fn harness_aging_elders_stay_home() {
        // Baseline seed 5 (R3): routine elders (elder weights) still never take
        // building or gathering work and never drift out of their tether while
        // idle. Measured: 6,509 idle elder observations in 3 days.
        let radius = rule().elder_tether_radius;
        let mut engine = baseline_engine(5);
        while engine.resources.settlements.is_empty() {
            engine.tick();
        }
        let (home, members) = {
            let settlement = engine
                .resources
                .settlements
                .values()
                .next()
                .expect("village");
            let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
            members.sort_unstable();
            (settlement.formation_tile, members)
        };
        // Make the first six members elders, standing at the hearth.
        let elders: Vec<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .filter(|(_, a)| members[..6].contains(&a.id))
            .map(|(e, _)| e)
            .collect();
        for entity in &elders {
            engine
                .world
                .insert(
                    *entity,
                    (Elder, Position::new(home.0, home.1), AgentState::Idle),
                )
                .expect("elder");
            let _ = engine
                .world
                .remove_one::<sim_core::components::SeekTarget>(*entity);
        }
        let mut last_idle_distance = vec![None; elders.len()];
        let mut idle_observations = 0;
        for _ in 0..3 * DAY {
            engine.tick();
            for (index, entity) in elders.iter().enumerate() {
                let Ok(state) = engine.world.get::<&AgentState>(*entity).map(|s| *s) else {
                    continue;
                };
                assert!(
                    !matches!(
                        state.target(),
                        Some(TargetKind::ConstructionSite | TargetKind::GatherFood)
                    ),
                    "elder entered {state:?}"
                );
                let pos = *engine.world.get::<&Position>(*entity).expect("pos");
                let distance = pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1));
                if state == AgentState::Idle {
                    idle_observations += 1;
                    // Inside the tether it stays inside; outside (back from a need trip) it never drifts out.
                    if let Some(previous) = last_idle_distance[index] {
                        assert!(
                            distance <= radius.max(previous),
                            "idle elder drifted to {distance} from {previous}"
                        );
                    }
                    last_idle_distance[index] = Some(distance);
                } else {
                    last_idle_distance[index] = None;
                }
            }
        }
        assert!(idle_observations > 1000);
        assert_eq!(engine.resources.tallies.deaths_combat, 0);
    }
}

// From `harness_revitalize_pass6_elders.rs`.
mod elders {
    //! Pass 6 Part 4: elders at the evening fire.
    //!
    //! Under `AgingRule::elder_evening_tether` an idle elder's tether inside the
    //! evening-gather window is `min(elder_tether_radius, evening tether)`;
    //! outside the window it stays `elder_tether_radius`. Numbers come from the
    //! live rules (`WorldRules::baseline`; aging and evening gather are unchanged
    //! since v12, so the thresholds are exact).
    use sim_core::components::{Agent, AgentState, Elder, Position, Settlement};
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};

    const HOME: (u32, u32) = (20, 20);

    /// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// Idle elders of one settlement at `HOME`, each starting `start` tiles east of it.
    fn elders(rules: WorldRules, count: u64, start: u32) -> (SimEngine, Vec<hecs::Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = rules;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        let mut entities = Vec::new();
        for seed in 0..count {
            let entity = engine.spawn_agent(HOME.0 + start, HOME.1);
            engine
                .world
                .insert(
                    entity,
                    (AgentState::Idle, Elder, MovementRng::new(seed + 11)),
                )
                .expect("elder");
            settlement.add_member_agent(engine.world.get::<&Agent>(entity).expect("agent").id);
            entities.push(entity);
        }
        engine.resources.settlements.insert(1, settlement);
        (engine, entities)
    }

    /// Run the movement system from `from` for `ticks` ticks; the largest Chebyshev
    /// distance from `HOME` observed after each tick.
    fn widest(engine: &mut SimEngine, entities: &[hecs::Entity], from: u64, ticks: u64) -> u32 {
        let mut movement = AgentMovementSystem::new();
        let mut widest = 0;
        for tick in from..from + ticks {
            engine.resources.current_tick = tick;
            movement.tick(&mut engine.world, &mut engine.resources);
            for entity in entities {
                let pos = *engine.world.get::<&Position>(*entity).expect("position");
                widest = widest.max(pos.x.abs_diff(HOME.0).max(pos.y.abs_diff(HOME.1)));
            }
        }
        widest
    }

    #[test]
    fn harness_elder_evening_tether() {
        let rules = WorldRules::baseline(7);
        let aging = rules.society.aging.expect("baseline aging");
        let evening = rules.agent.evening_gather.expect("baseline evening gather");
        assert!(aging.elder_evening_tether);
        let tight = aging.elder_tether_radius.min(evening.tether_radius);
        assert!(
            tight < aging.elder_tether_radius,
            "the evening tether is the tighter one"
        );
        // 19:00: idle elders inside the evening tether stay there for 120 ticks.
        let (mut engine, entities) = elders(rules, 6, tight);
        assert_eq!(widest(&mut engine, &entities, tick_at(3, 1140), 120), tight);
        // One standing at the elder tether at 19:00 is drawn in to the evening tether.
        let (mut engine, entities) = elders(rules, 6, aging.elder_tether_radius);
        widest(&mut engine, &entities, tick_at(3, 1140), 60);
        assert!(widest(&mut engine, &entities, tick_at(3, 1200), 60) <= tight);
        // 12:00: the elder tether alone; elders may stand at it, never beyond.
        let (mut engine, entities) = elders(rules, 6, tight);
        assert_eq!(
            widest(&mut engine, &entities, tick_at(3, 720), 300),
            aging.elder_tether_radius
        );
    }

    #[test]
    fn harness_elder_tether_returns_after_the_evening() {
        // At 21:00 the evening window closes and the elder tether alone holds again.
        let rules = WorldRules::baseline(7);
        let aging = rules.society.aging.expect("baseline aging");
        let evening = rules.agent.evening_gather.expect("evening");
        let (mut engine, entities) = elders(
            rules,
            6,
            aging.elder_tether_radius.min(evening.tether_radius),
        );
        let after = tick_at(3, u64::from(evening.end_minute));
        assert_eq!(
            widest(&mut engine, &entities, after, 300),
            aging.elder_tether_radius
        );
    }
}
