//! Merged in V8-0 (8.4) from `harness_revitalize_pass5_firewood.rs`,
//! `harness_revitalize_pass6_urgent.rs`; each source is one module below,
//! unchanged.

// From `harness_revitalize_pass5_firewood.rs`.
mod trips {
    //! Pass 5 Part 3: deadwood, firewood trips, the hearth fire and cold nights.
    use hecs::Entity;
    use sim_core::components::{
        Agent, AgentState, Cold, Hunger, Inventory, Position, ResourceKind, SeekTarget, Settlement,
        Sleep, Social, TargetKind, Thirst, SETTLEMENT_PROXIMITY_RADIUS,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_core::variety::WorldVariety;
    use sim_core::TerrainType;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
    use sim_systems::runtime::baseline::decision::AgentDecisionSystem;
    use sim_systems::runtime::needs::HungerDecaySystem;
    use sim_systems::runtime::resource_regen::ResourceRegenSystem;
    use sim_systems::runtime::settlement::firewood::{tonight_burn, wood_in_hand, wood_target};
    use sim_systems::runtime::settlement::{FireSystem, StockpileDepositSystem};
    use sim_test::baseline_engine;

    const HOME: (u32, u32) = (20, 20);
    /// World seed of the fixture rules (`WorldRules::baseline`).
    const SEED: u64 = 1;

    /// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// Bare baseline world: one settlement at `HOME` holding `wood` in store, `adults`
    /// idle satisfied members on the formation tile, and full deadwood spots.
    fn wood_fixture(
        adults: usize,
        spots: &[(u32, u32)],
        per_spot: u32,
        tick: u64,
        wood: u32,
    ) -> (SimEngine, Vec<Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        let mut rules = WorldRules::baseline(SEED);
        if let Some(firewood) = rules.society.firewood.as_mut() {
            firewood.max_gatherers_per_spot = per_spot;
        }
        engine.resources.rules = rules;
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
        for &spot in spots {
            engine.resources.wood_tiles.insert(spot, 6);
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

    fn wood_targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
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

    fn hearth(engine: &SimEngine) -> (u32, u32) {
        *engine
            .resources
            .building_kinds
            .iter()
            .find(|(_, kind)| **kind == sim_core::components::BuildingKind::Hearth)
            .expect("hearth")
            .0
    }

    #[test]
    fn harness_deadwood_placement() {
        let chebyshev = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
        for seed in 0..4u64 {
            let engine = baseline_engine(seed);
            let r = &engine.resources;
            let variety = WorldVariety::draw(seed);
            let center = hearth(&engine);
            assert_eq!(r.wood_tiles, r.wood_source_max, "every spot starts full");
            // The camp's spots are area 0 (`wood_spot_area`); the wild regions'
            // spots lie 22-28 tiles out. Seeds 0..4 draw 14..23 camp spots of
            // capacity 5..7 and place every one of them.
            let camp: Vec<_> = r
                .wood_spot_area
                .iter()
                .filter(|(_, area)| **area == 0)
                .map(|(spot, _)| *spot)
                .collect();
            assert_eq!(camp.len(), variety.deadwood_spots as usize, "seed {seed}");
            assert!(camp.len() >= 14, "seed {seed}: {} camp spots", camp.len());
            for &spot in &camp {
                assert!(
                    (5..=18).contains(&chebyshev(spot, center)),
                    "seed {seed} spot {spot:?}"
                );
                assert_eq!(r.wood_source_max[&spot], variety.deadwood_capacity);
            }
            let spots: Vec<_> = r.wood_source_max.keys().copied().collect();
            assert_eq!(
                spots.len(),
                r.wood_spot_area.len(),
                "every spot has an area"
            );
            for (i, &spot) in spots.iter().enumerate() {
                let terrain = r.tile_grid.terrain_type[r.tile_grid.idx(spot.0, spot.1)];
                assert!(
                    matches!(terrain, TerrainType::Forest | TerrainType::Plain),
                    "{terrain:?}"
                );
                assert!(!r.water_tiles.contains_key(&spot));
                assert!(!r.building_kinds.contains_key(&spot));
                assert!(!r.food_source_max.contains_key(&spot));
                for &other in &spots[i + 1..] {
                    assert!(chebyshev(spot, other) >= 2, "{spot:?} {other:?}");
                }
            }
            // Deterministic per seed; a different seed places differently.
            assert_eq!(
                baseline_engine(seed).resources.wood_source_max,
                r.wood_source_max
            );
        }
        assert_ne!(
            baseline_engine(0).resources.wood_source_max,
            baseline_engine(1).resources.wood_source_max
        );
    }

    #[test]
    fn harness_wood_regen_schedule() {
        let (mut engine, _) = wood_fixture(0, &[(30, 20), (31, 25)], 1, 0, 0);
        engine.resources.wood_tiles.remove(&(30, 20));
        let mut regen = ResourceRegenSystem::new();
        let mut seen = Vec::new();
        for _ in 0..12 {
            regen.tick(&mut engine.world, &mut engine.resources);
            assert!(
                engine.resources.wood_tiles.values().all(|v| *v > 0),
                "no zero-valued tile"
            );
            seen.push(
                engine
                    .resources
                    .wood_tiles
                    .get(&(30, 20))
                    .copied()
                    .unwrap_or(0),
            );
        }
        // 500 permille per 120-tick pulse: one unit every second pulse, six a day.
        assert_eq!(seen, vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
        assert_eq!(
            engine.resources.wood_tiles[&(31, 25)],
            6,
            "a full spot stays at its ceiling"
        );
    }

    #[test]
    fn harness_firewood_trip_and_deposit() {
        let (mut engine, entities) = wood_fixture(1, &[(26, 20)], 1, tick_at(1, 600), 0);
        let gatherer = entities[0];
        let mut movement = AgentMovementSystem::new();
        let mut decision = AgentDecisionSystem::new();
        let mut deposit = StockpileDepositSystem::new();
        let start = engine.resources.current_tick;
        decision.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&AgentState>(gatherer).expect("state"),
            AgentState::Seeking {
                target: TargetKind::GatherWood
            }
        );
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(gatherer)
                .expect("target")
                .tile,
            (26, 20)
        );
        assert_eq!(engine.resources.tallies.wood_trips, 1);
        let mut picked = false;
        for tick in start + 1..start + 400 {
            engine.resources.current_tick = tick;
            movement.tick(&mut engine.world, &mut engine.resources);
            decision.tick(&mut engine.world, &mut engine.resources);
            if tick % 10 == 0 {
                deposit.tick(&mut engine.world, &mut engine.resources);
            }
            let carried = engine
                .world
                .get::<&Inventory>(gatherer)
                .expect("inv")
                .get(ResourceKind::Wood);
            picked |= carried == 2;
            if picked && wood_store(&engine) == 2 {
                break;
            }
        }
        assert!(picked, "the gatherer never picked up a wood load");
        assert_eq!(wood_store(&engine), 2, "the load was not deposited");
        assert_eq!(engine.resources.tallies.wood_gathered, 2);
        assert_eq!(engine.resources.wood_tiles[&(26, 20)], 4);
        assert_eq!(
            engine
                .world
                .get::<&Inventory>(gatherer)
                .expect("inv")
                .get(ResourceKind::Wood),
            0
        );
        assert_eq!(
            engine.resources.tallies.haul_deposits, 1,
            "a walked-home load counts as a haul"
        );
        let pos = *engine.world.get::<&Position>(gatherer).expect("pos");
        assert!(pos.x.abs_diff(HOME.0).max(pos.y.abs_diff(HOME.1)) <= SETTLEMENT_PROXIMITY_RADIUS);
    }

    #[test]
    fn harness_firewood_stops_at_target() {
        let mut engine = baseline_engine(2);
        while engine.resources.settlements.is_empty() {
            engine.tick();
        }
        // Move to 10:00 of display day 1 (inside the firewood window).
        while engine.current_tick() < tick_at(1, 600) {
            engine.tick();
        }
        let sid = *engine.resources.settlements.keys().next().expect("village");
        let target = wood_target(&engine.resources).expect("baseline wood target");
        assert!(target > 0.0);
        let surplus = target.ceil() as u32 + 30;
        engine
            .resources
            .settlements
            .get_mut(&sid)
            .expect("village")
            .store(ResourceKind::Wood, surplus);
        let gathering: Vec<_> = engine
            .world
            .query::<&AgentState>()
            .iter()
            .filter(|(_, s)| {
                matches!(
                    s,
                    AgentState::Seeking {
                        target: TargetKind::GatherWood
                    } | AgentState::Consuming {
                        target: TargetKind::GatherWood
                    }
                )
            })
            .map(|(e, _)| e)
            .collect();
        for entity in gathering {
            engine
                .world
                .insert_one(entity, AgentState::Idle)
                .expect("idle");
            let _ = engine.world.remove_one::<SeekTarget>(entity);
        }
        let trips = engine.resources.tallies.wood_trips;
        while engine.current_tick() < tick_at(1, 1020) {
            engine.tick();
            let target = wood_target(&engine.resources).unwrap_or(0.0);
            assert!(wood_in_hand(&engine.world, &engine.resources, sid) as f64 >= target);
        }
        assert_eq!(
            engine.resources.tallies.wood_trips, trips,
            "firewood arm fired above the target"
        );
        // Below the target the arm fires again.
        let (mut engine, entities) = wood_fixture(1, &[(26, 20)], 1, tick_at(1, 600), 0);
        let target = wood_target(&engine.resources).expect("target");
        engine
            .resources
            .settlements
            .get_mut(&1)
            .expect("village")
            .store(ResourceKind::Wood, target.ceil() as u32);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![None]);
        engine
            .resources
            .settlements
            .get_mut(&1)
            .expect("village")
            .withdraw(ResourceKind::Wood, 1);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![Some((26, 20))]);
        // Outside the window nobody goes, even with no wood at all: 08:59 is
        // before the baseline's urgent start (09:00) and the normal start (10:00).
        let urgent = engine
            .resources
            .rules
            .society
            .firewood
            .and_then(|rule| rule.urgent_start_minute)
            .expect("baseline urgent start");
        assert_eq!(urgent, 540);
        let (mut engine, entities) = wood_fixture(1, &[(26, 20)], 1, tick_at(1, 539), 0);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![None]);
    }

    /// Wood for tonight's burn (spring: 2 an hour x 13 hours = 26): enough to keep
    /// the settlement out of urgent mode (which doubles the cap), still below the
    /// 1.5-day target, so the arm fires at the rule's own cap.
    fn tonight(tick: u64) -> u32 {
        let (engine, _) = wood_fixture(0, &[], 1, tick, 0);
        let rule = engine.resources.rules.society.firewood.expect("firewood");
        tonight_burn(&engine.resources, &rule).ceil() as u32
    }

    #[test]
    fn harness_firewood_claims_cap() {
        let spots = [(30, 20), (20, 31)];
        let wood = tonight(tick_at(1, 600));
        assert_eq!(wood, 26);
        let (mut engine, entities) = wood_fixture(3, &spots, 1, tick_at(1, 600), wood);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = wood_targets(&engine, &entities);
        assert_eq!(
            chosen.iter().filter(|t| t.is_some()).count(),
            2,
            "{chosen:?}"
        );
        for spot in spots {
            assert_eq!(
                chosen.iter().filter(|t| **t == Some(spot)).count(),
                1,
                "{chosen:?}"
            );
        }
        assert_eq!(engine.resources.tallies.wood_trips, 2);

        let (mut engine, entities) = wood_fixture(4, &spots, 2, tick_at(1, 600), wood);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = wood_targets(&engine, &entities);
        for spot in spots {
            assert_eq!(
                chosen.iter().filter(|t| **t == Some(spot)).count(),
                2,
                "{chosen:?}"
            );
        }
    }

    #[test]
    fn harness_firewood_retargets_empty_spot() {
        let spots = [(32, 20), (20, 33)];
        let (mut engine, entities) = wood_fixture(1, &spots, 1, tick_at(1, 600), 0);
        let gatherer = entities[0];
        let mut movement = AgentMovementSystem::new();
        let mut decision = AgentDecisionSystem::new();
        let start = engine.resources.current_tick;
        let mut phase = 0;
        for tick in start..start + 400 {
            engine.resources.current_tick = tick;
            movement.tick(&mut engine.world, &mut engine.resources);
            decision.tick(&mut engine.world, &mut engine.resources);
            let state = *engine.world.get::<&AgentState>(gatherer).expect("state");
            let seek = engine
                .world
                .get::<&SeekTarget>(gatherer)
                .ok()
                .map(|s| s.tile);
            if matches!(
                state,
                AgentState::Seeking {
                    target: TargetKind::GatherWood
                }
            ) {
                let target =
                    seek.unwrap_or_else(|| panic!("wood gatherer without a target at tick {tick}"));
                assert!(
                    engine.resources.wood_tiles.contains_key(&target),
                    "seeking empty spot {target:?} at tick {tick}"
                );
            }
            match phase {
                0 if seek == Some(spots[0]) && tick > start + 16 => {
                    engine.resources.wood_tiles.remove(&spots[0]);
                    phase = 1;
                }
                1 => {
                    assert_eq!(
                        engine
                            .world
                            .get::<&SeekTarget>(gatherer)
                            .ok()
                            .map(|s| s.tile),
                        Some(spots[1])
                    );
                    engine.resources.wood_tiles.remove(&spots[1]);
                    phase = 2;
                }
                2 => {
                    assert_eq!(
                        state,
                        AgentState::Idle,
                        "gatherer must idle when no spot qualifies"
                    );
                    assert!(engine.world.get::<&SeekTarget>(gatherer).is_err());
                    return;
                }
                _ => {}
            }
        }
        panic!("scenario did not complete (phase {phase})");
    }

    #[test]
    fn harness_fire_burns_and_goes_out() {
        // 18:00 on spring day 1: the fire burns two units and is lit.
        let (mut engine, _) = wood_fixture(1, &[], 1, tick_at(1, 1080), 5);
        let mut fire = FireSystem::new();
        fire.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_store(&engine), 3);
        assert!(engine.resources.fires[&1].lit);
        // 02:00 with one unit left: the hour cannot be paid, the fire is out.
        engine.resources.current_tick = tick_at(2, 120);
        engine
            .resources
            .settlements
            .get_mut(&1)
            .expect("village")
            .withdraw(ResourceKind::Wood, 2);
        fire.tick(&mut engine.world, &mut engine.resources);
        assert!(!engine.resources.fires[&1].lit);
        // An empty store at 02:00 stays out.
        fire.tick(&mut engine.world, &mut engine.resources);
        assert!(!engine.resources.fires[&1].lit);
        assert_eq!(wood_store(&engine), 0);
        // Outside the burn window nothing burns and the fire is out.
        let (mut engine, _) = wood_fixture(1, &[], 1, tick_at(1, 720), 5);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_store(&engine), 5);
        assert!(!engine.resources.fires[&1].lit);
        // Over a full spring night the fire burns 2 x 13 = 26 units.
        let (mut engine, _) = wood_fixture(0, &[], 1, tick_at(1, 1080), 100);
        for tick in tick_at(1, 1080)..tick_at(2, 1080) {
            engine.resources.current_tick = tick;
            if tick % 60 == 0 {
                FireSystem::new().tick(&mut engine.world, &mut engine.resources);
            }
        }
        assert_eq!(wood_store(&engine), 74);
    }

    #[test]
    fn harness_fire_cold_raises_hunger() {
        // Display day 21 is the first day of winter (seven-day seasons).
        let winter_night = tick_at(21, 1380);
        let growth = |engine: &mut SimEngine, entity: Entity| {
            let before = engine.world.get::<&Hunger>(entity).expect("hunger").value;
            HungerDecaySystem::new().tick(&mut engine.world, &mut engine.resources);
            engine.world.get::<&Hunger>(entity).expect("hunger").value - before
        };
        // Fire out (no wood) on a winter night: members are cold, hunger x1.5.
        let (mut engine, entities) = wood_fixture(2, &[], 1, winter_night, 0);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(!engine.resources.fires[&1].lit);
        for &entity in &entities {
            assert!(engine.world.get::<&Cold>(entity).is_ok());
        }
        assert_eq!(engine.resources.tallies.cold_member_hours, 2);
        let cold = growth(&mut engine, entities[0]);
        assert!((cold - 0.075).abs() < 1e-6, "cold growth {cold}");
        // Fire lit: no marker, growth x1.0.
        let (mut engine, entities) = wood_fixture(2, &[], 1, winter_night, 10);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(engine.resources.fires[&1].lit);
        assert!(engine.world.get::<&Cold>(entities[0]).is_err());
        let warm = growth(&mut engine, entities[0]);
        assert!((warm - 0.05).abs() < 1e-6, "warm growth {warm}");
        // The marker leaves once the fire is lit again or the cold window ends.
        let (mut engine, entities) = wood_fixture(1, &[], 1, winter_night, 0);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(engine.world.get::<&Cold>(entities[0]).is_ok());
        engine.resources.current_tick = tick_at(22, 420);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(engine.world.get::<&Cold>(entities[0]).is_err());
        // A spring night with the fire out is not cold.
        let (mut engine, entities) = wood_fixture(1, &[], 1, tick_at(1, 1380), 0);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(engine.world.get::<&Cold>(entities[0]).is_err());
    }
}

// From `harness_revitalize_pass6_urgent.rs`.
mod urgent_mornings {
    //! Pass 6 Part 3: urgent mornings.
    //!
    //! When tonight's burn is not in hand a settlement is in urgent mode: its
    //! members' firewood window opens at `urgent_start_minute` and two gatherers
    //! may share a spot. Expected numbers come from `WorldRules::baseline(SEED)`
    //! (V8-0; was `game_v12`). The baseline winter burns at its drawn severity
    //! (Pass 7): world seed 3's first winter is normal, 3 an hour x 13 hours = 39
    //! tonight, as v12's single winter rate.
    use hecs::Entity;
    use sim_core::components::{
        Agent, AgentState, Hunger, Inventory, ResourceKind, SeekTarget, Settlement, Sleep, Social,
        TargetKind, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{FirewoodRule, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::MovementRng;
    use sim_systems::runtime::baseline::decision::AgentDecisionSystem;
    use sim_systems::runtime::settlement::firewood::{
        is_urgent, night_season, tonight_burn, wood_target,
    };

    const HOME: (u32, u32) = (20, 20);
    /// World seed of the fixture rules and of the winter severity draw.
    const SEED: u64 = 3;
    /// Display day 22: a winter day (seven-day seasons; winter is days 21–27).
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

    /// Bare world under `rules`: one settlement at `HOME` holding `wood`,
    /// `adults` idle satisfied members on the formation tile, full deadwood spots.
    fn fixture(
        rules: WorldRules,
        adults: usize,
        spots: &[(u32, u32)],
        tick: u64,
        wood: u32,
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
        for &spot in spots {
            engine.resources.wood_tiles.insert(spot, 6);
            engine.resources.wood_source_max.insert(spot, 6);
        }
        (engine, entities)
    }

    fn wood_targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
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

    /// Tonight's burn on `day` at `minute` (the severity's rate x 13 hours).
    fn tonight(day: u64, minute: u64) -> u32 {
        let (engine, _) = fixture(rules(), 0, &[], tick_at(day, minute), 0);
        let rule = baseline_rule();
        tonight_burn(&engine.resources, &rule).ceil() as u32
    }

    #[test]
    fn harness_urgent_window_opens_at_dawn() {
        let rule = baseline_rule();
        let start = u64::from(rule.urgent_start_minute.expect("baseline urgent mode"));
        let gather = u64::from(rule.gather_start_minute);
        assert!(
            start < gather,
            "urgent mornings start before the normal window"
        );
        let burn = tonight(WINTER_DAY, start);
        assert_eq!(burn, 39, "seed 3's normal first winter");
        // Tonight's burn is not in hand: an adult leaves for wood at the urgent start.
        let (mut engine, entities) = fixture(
            rules(),
            1,
            &[(26, 20)],
            tick_at(WINTER_DAY, start),
            burn - 1,
        );
        assert!(is_urgent(&engine.world, &engine.resources, 1));
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![Some((26, 20))]);
        assert_eq!(
            (
                engine.resources.tallies.wood_trips,
                engine.resources.tallies.urgent_wood_trips
            ),
            (1, 1)
        );
        // One minute before the urgent start nobody goes.
        let (mut engine, entities) =
            fixture(rules(), 1, &[(26, 20)], tick_at(WINTER_DAY, start - 1), 0);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![None]);
        // Tonight's burn in hand (still below the target): not before the normal window.
        let target = wood_target(&engine.resources).expect("target");
        assert!(f64::from(burn) < target);
        for minute in [start, gather - 1] {
            let (mut engine, entities) =
                fixture(rules(), 1, &[(26, 20)], tick_at(WINTER_DAY, minute), burn);
            assert!(!is_urgent(&engine.world, &engine.resources, 1));
            AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
            assert_eq!(
                wood_targets(&engine, &entities),
                vec![None],
                "minute {minute}"
            );
        }
        let (mut engine, entities) =
            fixture(rules(), 1, &[(26, 20)], tick_at(WINTER_DAY, gather), burn);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![Some((26, 20))]);
        assert_eq!(
            (
                engine.resources.tallies.wood_trips,
                engine.resources.tallies.urgent_wood_trips
            ),
            (1, 0)
        );
        // The urgent window closes with the normal one.
        let end = u64::from(rule.gather_end_minute);
        let (mut engine, entities) = fixture(rules(), 1, &[(26, 20)], tick_at(WINTER_DAY, end), 0);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![None]);
    }

    #[test]
    fn harness_urgent_two_per_spot() {
        let rule = baseline_rule();
        let start = u64::from(rule.urgent_start_minute.expect("baseline urgent mode"));
        let per_spot = rule.max_gatherers_per_spot as usize;
        let adults = 2 * per_spot + 1;
        // Urgent: twice the claim cap on the only spot.
        let (mut engine, entities) =
            fixture(rules(), adults, &[(26, 20)], tick_at(WINTER_DAY, start), 0);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = wood_targets(&engine, &entities);
        assert_eq!(
            chosen.iter().filter(|t| **t == Some((26, 20))).count(),
            2 * per_spot,
            "{chosen:?}"
        );
        assert_eq!(
            engine.resources.tallies.urgent_wood_trips,
            2 * per_spot as u64
        );
        // Not urgent (10:00, tonight's burn in hand): the Pass 5 cap.
        let burn = tonight(WINTER_DAY, u64::from(rule.gather_start_minute));
        let (mut engine, entities) = fixture(
            rules(),
            adults,
            &[(26, 20)],
            tick_at(WINTER_DAY, u64::from(rule.gather_start_minute)),
            burn,
        );
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = wood_targets(&engine, &entities);
        assert_eq!(
            chosen.iter().filter(|t| **t == Some((26, 20))).count(),
            per_spot,
            "{chosen:?}"
        );
        // Re-targeting: gatherers whose spot emptied share the remaining spot up to the doubled cap.
        for (wood, minute, want) in [
            (0, start, 2 * per_spot),
            (burn, u64::from(rule.gather_start_minute), per_spot),
        ] {
            let (mut engine, entities) = fixture(
                rules(),
                adults,
                &[(30, 20)],
                tick_at(WINTER_DAY, minute),
                wood,
            );
            for entity in &entities {
                engine
                    .world
                    .insert_one(
                        *entity,
                        AgentState::Seeking {
                            target: TargetKind::GatherWood,
                        },
                    )
                    .expect("seek");
                engine
                    .world
                    .insert_one(*entity, SeekTarget::new((40, 40)))
                    .expect("empty spot");
            }
            AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
            let chosen = wood_targets(&engine, &entities);
            assert_eq!(
                chosen.iter().filter(|t| **t == Some((30, 20))).count(),
                want,
                "wood {wood}: {chosen:?}"
            );
            assert_eq!(
                chosen.iter().filter(|t| t.is_none()).count(),
                adults - want,
                "the rest idle: {chosen:?}"
            );
            assert_eq!(
                engine.resources.tallies.wood_trips, 0,
                "re-targeting starts no trip"
            );
        }
    }

    #[test]
    fn harness_urgent_ends_when_wood_is_in_hand() {
        let rule = baseline_rule();
        let start = u64::from(rule.urgent_start_minute.expect("baseline urgent mode"));
        let burn = tonight(WINTER_DAY, start);
        let (mut engine, entities) = fixture(rules(), 1, &[], tick_at(WINTER_DAY, start), burn - 2);
        assert!(is_urgent(&engine.world, &engine.resources, 1));
        // A gatherer walking home with one unit is wood in hand too.
        engine
            .world
            .get::<&mut Inventory>(entities[0])
            .expect("inv")
            .add(ResourceKind::Wood, 1);
        assert!(is_urgent(&engine.world, &engine.resources, 1));
        engine
            .resources
            .settlements
            .get_mut(&1)
            .expect("village")
            .store(ResourceKind::Wood, 1);
        assert!(
            !is_urgent(&engine.world, &engine.resources, 1),
            "tonight's burn is in hand"
        );
        // With the flag cleared, a new trip waits for the normal window.
        engine.resources.wood_tiles.insert((26, 20), 6);
        engine.resources.wood_source_max.insert((26, 20), 6);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(wood_targets(&engine, &entities), vec![None]);
        assert_eq!(engine.resources.tallies.urgent_wood_trips, 0);
        // The night ahead decides: after midnight the evening's season still counts.
        let (engine, _) = fixture(rules(), 0, &[], tick_at(WINTER_DAY + 6, 120), 0);
        assert_eq!(
            night_season(&engine.resources, &rule),
            3,
            "the last winter night is a winter night"
        );
    }

    #[test]
    fn harness_urgent_is_per_settlement() {
        // Two settlements at the urgent start: only the one short of tonight's burn sends
        // its members, and only it may put two gatherers on a spot (review round 1).
        let rule = baseline_rule();
        let start = u64::from(rule.urgent_start_minute.expect("baseline urgent mode"));
        let burn = tonight(WINTER_DAY, start);
        let per_spot = rule.max_gatherers_per_spot as usize;
        let (mut engine, short) = fixture(
            rules(),
            2 * per_spot,
            &[(26, 20)],
            tick_at(WINTER_DAY, start),
            0,
        );
        let mut stocked = Settlement::new_with_id(2, 0);
        stocked.formation_tile = (40, 20);
        stocked.store(ResourceKind::Wood, burn);
        let mut others = Vec::new();
        for index in 0..2 {
            let entity = engine.spawn_agent(40, 20);
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
                        MovementRng::new(index + 50),
                    ),
                )
                .expect("components");
            stocked.add_member_agent(id);
            others.push(entity);
        }
        engine.resources.settlements.insert(2, stocked);
        engine.resources.wood_tiles.insert((44, 20), 6);
        engine.resources.wood_source_max.insert((44, 20), 6);
        assert!(is_urgent(&engine.world, &engine.resources, 1));
        assert!(!is_urgent(&engine.world, &engine.resources, 2));
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            wood_targets(&engine, &short),
            vec![Some((26, 20)); 2 * per_spot]
        );
        assert_eq!(
            wood_targets(&engine, &others),
            vec![None, None],
            "the stocked village waits for its normal window"
        );
        assert_eq!(
            engine.resources.tallies.urgent_wood_trips,
            2 * per_spot as u64
        );
    }
}
