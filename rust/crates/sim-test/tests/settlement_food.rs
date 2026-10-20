//! Merged in V8-0 (8.4) from `harness_revitalize_pass3_food.rs`,
//! `harness_revitalize_pass4_forage.rs`, `harness_revitalize_pass5_meals.rs`;
//! each source is one module below, unchanged.

// From `harness_revitalize_pass3_food.rs`.
mod store {
    use sim_core::components::{
        Agent, AgentState, BuildingKind, Hunger, ResourceKind, Settlement, TargetKind, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::decision::AgentDecisionSystem;
    use sim_systems::runtime::settlement::{
        food_economy::{members_need_per_day, supply_per_day, sustainable_adults},
        SettlementSystem, StockpileConsumeSystem,
    };

    /// The village food rules of the live game (`WorldRules::baseline`) copied
    /// into `WorldRules::default()` (R1): the provisioned birth gate (utilization
    /// 0.78, capacity-weighted supply), owned housing, hearth anchors, the relief
    /// radius and strict consumption. Nothing else ticks in these fixtures.
    fn food_rules() -> WorldRules {
        let live = WorldRules::baseline(7);
        let mut rules = WorldRules::default();
        rules.society.birth = live.society.birth;
        rules.society.shelter_planning = live.society.shelter_planning;
        rules.society.settlement_anchor = live.society.settlement_anchor;
        rules.society.relief_radius = live.society.relief_radius;
        rules.agent.strict_consume = live.agent.strict_consume;
        rules.agent.idle_tether_radius = live.agent.idle_tether_radius;
        rules
    }

    fn birth_fixture(members: u64, stock: u32) -> (SimEngine, SettlementSystem, u32) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = food_rules();
        for _ in 0..members {
            let entity = engine.spawn_agent(20, 20);
            engine
                .world
                .insert(entity, (Hunger::new(0.0, 0.05), Thirst::new(0.0, 0.08)))
                .expect("needs");
        }
        engine.resources.building_registry.insert(100, (20, 20));
        engine
            .resources
            .building_kinds
            .insert((20, 20), BuildingKind::Hearth);
        for x in 21..27 {
            engine
                .resources
                .building_registry
                .insert(u64::from(100 + x), (x, 20));
            engine
                .resources
                .building_kinds
                .insert((x, 20), BuildingKind::Shelter);
        }
        for x in 20..30 {
            engine.resources.food_source_max.insert((x, 22), 4);
        }
        let mut settlements = SettlementSystem::new();
        settlements.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.resources.settlements.len(), 1);
        let sid = *engine.resources.settlements.keys().next().expect("village");
        engine
            .resources
            .settlements
            .get_mut(&sid)
            .expect("village")
            .store(ResourceKind::Food, stock);
        (engine, settlements, sid)
    }

    fn tick_settlements(engine: &mut SimEngine, system: &mut SettlementSystem, ticks: u64) {
        for tick in 1..=ticks {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
    }

    #[test]
    fn harness_food_supply_and_need_formula() {
        let (engine, _, sid) = birth_fixture(24, 500);
        assert_eq!(supply_per_day(&engine.resources, sid, 16), 120.0);
        assert!((members_need_per_day(&engine.world, &engine.resources, sid) - 57.6).abs() < 1e-4);
        // Baseline utilization 0.78 (v9 had 0.85 → 42): 0.78 × 120 / 2.4 = 39.
        let utilization = match engine.resources.rules.society.birth {
            sim_core::rules::BirthRule::Provisioned {
                sustain: Some(sustain),
                ..
            } => sustain.utilization,
            _ => panic!("baseline birth is provisioned with a sustain gate"),
        };
        assert_eq!(utilization, 0.78);
        assert_eq!(sustainable_adults(120.0, utilization, 1440), 39);
    }

    #[test]
    fn harness_food_no_free_meals() {
        // Baseline `strict_consume`: a second eater on an emptied tile gets
        // nothing (the old v8 arm, where it ate for free, is gone with v8).
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = food_rules();
        assert!(engine.resources.rules.agent.strict_consume);
        let a = engine.spawn_agent(10, 10);
        let b = engine.spawn_agent(10, 10);
        engine
            .world
            .insert(
                a,
                (
                    Hunger::new(80.0, 0.0),
                    AgentState::Consuming {
                        target: TargetKind::Food,
                    },
                ),
            )
            .expect("first eater");
        engine
            .world
            .insert_one(b, Hunger::new(80.0, 0.0))
            .expect("second eater");
        engine.resources.set_food_tile(10, 10, 1);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.world.get::<&Hunger>(a).expect("first").value, 50.0);
        engine
            .world
            .remove_one::<AgentState>(a)
            .expect("first finished");
        engine
            .world
            .insert_one(
                b,
                AgentState::Consuming {
                    target: TargetKind::Food,
                },
            )
            .expect("second commit");
        engine.resources.current_tick = 1;
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.world.get::<&Hunger>(b).expect("second").value, 80.0);
        assert_eq!(engine.resources.tallies.meals_without_food, 0);
    }

    #[test]
    fn harness_food_birth_blocked_at_supply() {
        // 39 members: a 40th adult needs 96.0 > 0.78 × 120 = 93.6 (the first
        // refused size under the baseline gate; v9's was 42).
        let (mut engine, mut system, _) = birth_fixture(39, 500);
        tick_settlements(&mut engine, &mut system, 2880);
        assert_eq!(engine.resources.tallies.births, 0);
    }

    #[test]
    fn harness_food_birth_blocked_without_reserve() {
        let (mut engine, mut system, _) = birth_fixture(24, 38);
        tick_settlements(&mut engine, &mut system, 2880);
        assert_eq!(engine.resources.tallies.births, 0);
    }

    #[test]
    fn harness_food_birth_allowed_when_sustainable() {
        let (mut engine, mut system, _) = birth_fixture(24, 500);
        tick_settlements(&mut engine, &mut system, 721);
        assert_eq!(engine.resources.tallies.births, 1);
        // 38 members: the 39th adult needs 93.6 ≤ 93.6, the largest admitted size.
        let (mut engine, mut system, _) = birth_fixture(38, 500);
        tick_settlements(&mut engine, &mut system, 721);
        assert_eq!(engine.resources.tallies.births, 1);
    }

    #[test]
    fn harness_food_relief_reaches_tether() {
        let live = WorldRules::baseline(7);
        assert_eq!(live.society.relief_radius, live.agent.idle_tether_radius);
        // A hungry member 7 tiles from the store (inside relief radius 8) eats
        // from it without walking home.
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = food_rules();
        let entity = engine.spawn_agent(17, 10);
        engine
            .world
            .insert_one(entity, Hunger::new(80.0, 0.0))
            .expect("hunger");
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (10, 10);
        settlement.add_member_agent(id);
        settlement.store(ResourceKind::Food, 1);
        engine.resources.settlements.insert(1, settlement);
        StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            engine.world.get::<&Hunger>(entity).expect("hunger").value,
            50.0
        );
    }
}

// From `harness_revitalize_pass4_forage.rs`.
mod forage {
    //! Pass 4 Part 2: purposeful foraging, hauling home and the store target.
    use hecs::Entity;
    use sim_core::components::{
        Agent, AgentState, Hunger, Inventory, Position, ResourceKind, SeekTarget, Settlement,
        Sleep, Social, TargetKind, Thirst, SETTLEMENT_PROXIMITY_RADIUS,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
    use sim_systems::runtime::decision::AgentDecisionSystem;
    use sim_systems::runtime::settlement::food_economy::{stock_in_hand, stock_target};
    use sim_test::baseline_engine;

    fn food_stock(engine: &SimEngine) -> u32 {
        engine
            .resources
            .settlements
            .values()
            .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))
            .sum()
    }

    fn is_gathering(state: AgentState) -> bool {
        matches!(
            state,
            AgentState::Seeking {
                target: TargetKind::GatherFood
            } | AgentState::Consuming {
                target: TargetKind::GatherFood
            }
        )
    }

    /// Bare world: one settlement at (20, 20) whose idle, satisfied members stand on
    /// the formation tile, and stocked bushes; clock at 08:00 inside the forage window.
    /// Rules: `WorldRules::baseline` (R1) with `max_foragers_per_bush` set to
    /// `per_bush` (R2, numeric: baseline is 2, v10 was 1).
    fn forage_fixture(
        adults: usize,
        bushes: &[(u32, u32)],
        per_bush: u32,
    ) -> (SimEngine, Vec<Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        let mut rules = WorldRules::baseline(7);
        if let Some(forage) = rules.agent.forage.as_mut() {
            forage.max_foragers_per_bush = per_bush;
        }
        engine.resources.rules = rules;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (20, 20);
        let mut entities = Vec::new();
        for index in 0..adults {
            let entity = engine.spawn_agent(20, 20);
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
        for &(x, y) in bushes {
            engine.resources.set_food_tile(x, y, 4);
            engine.resources.set_food_source_max(x, y, 4);
        }
        (engine, entities)
    }

    fn targets(engine: &SimEngine, entities: &[Entity]) -> Vec<Option<(u32, u32)>> {
        entities
            .iter()
            .map(|e| {
                let state = *engine.world.get::<&AgentState>(*e).expect("state");
                matches!(
                    state,
                    AgentState::Seeking {
                        target: TargetKind::GatherFood
                    }
                )
                .then(|| engine.world.get::<&SeekTarget>(*e).ok().map(|s| s.tile))
                .flatten()
            })
            .collect()
    }

    #[test]
    fn harness_forage_stops_at_target() {
        let mut engine = baseline_engine(2);
        while engine.resources.settlements.is_empty() {
            engine.tick();
        }
        let sid = *engine.resources.settlements.keys().next().expect("village");
        let target = stock_target(&engine.world, &engine.resources, sid).expect("v10 target");
        assert!(target > 0.0);
        let surplus = (3.0 * target).ceil() as u32 + 200;
        engine
            .resources
            .settlements
            .get_mut(&sid)
            .expect("village")
            .store(ResourceKind::Food, surplus);
        // Drop trips already underway so only new decisions are observed.
        let gathering: Vec<_> = engine
            .world
            .query::<&AgentState>()
            .iter()
            .filter(|(_, s)| is_gathering(**s))
            .map(|(e, _)| e)
            .collect();
        for entity in gathering {
            engine
                .world
                .insert_one(entity, AgentState::Idle)
                .expect("idle");
            let _ = engine.world.remove_one::<SeekTarget>(entity);
        }
        let trips = engine.resources.tallies.forage_trips;
        for _ in 0..1440 {
            engine.tick();
            let target = stock_target(&engine.world, &engine.resources, sid).unwrap_or(0.0);
            let in_hand = stock_in_hand(&engine.world, &engine.resources, sid) as f64;
            if in_hand >= 2.0 * target {
                let gatherers = engine
                    .world
                    .query::<&AgentState>()
                    .iter()
                    .filter(|(_, s)| is_gathering(**s))
                    .count();
                assert_eq!(
                    gatherers,
                    0,
                    "gathering at tick {} with {in_hand} >= 2 x {target}",
                    engine.current_tick()
                );
            }
        }
        assert_eq!(
            engine.resources.tallies.forage_trips, trips,
            "forage arm fired above the target"
        );
        assert!(stock_in_hand(&engine.world, &engine.resources, sid) as f64 >= 2.0 * target);
    }

    #[test]
    fn harness_haul_home_deposits() {
        let mut engine = baseline_engine(4);
        while engine.resources.settlements.is_empty() {
            engine.tick();
        }
        let (sid, home, member) = {
            let (sid, settlement) = engine.resources.settlements.iter().next().expect("village");
            let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
            members.sort_unstable();
            (*sid, settlement.formation_tile, members[0])
        };
        let entity = engine
            .world
            .query::<&Agent>()
            .iter()
            .find(|(_, a)| a.id == member)
            .map(|(e, _)| e)
            .expect("member entity");
        let start = (home.0 + 12, home.1);
        let mut inventory = Inventory::default();
        inventory.add(ResourceKind::Food, 6);
        engine
            .world
            .insert(
                entity,
                (
                    Position::new(start.0, start.1),
                    AgentState::Idle,
                    inventory,
                    Hunger::new(0.0, 0.05),
                    Thirst::new(0.0, 0.08),
                    Sleep::new(0.0, 0.03),
                    Social::new(0.0, 0.04),
                ),
            )
            .expect("hauler");
        let _ = engine.world.remove_one::<SeekTarget>(entity);
        let before = food_stock(&engine);
        let deposits = engine.resources.tallies.haul_deposits;
        let limit = 16 * engine.resources.rules.agent.move_interval_ticks + 20;
        let mut arrived = None;
        for elapsed in 1..=limit {
            engine.tick();
            let pos = *engine.world.get::<&Position>(entity).expect("pos");
            if arrived.is_none()
                && pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1)) <= SETTLEMENT_PROXIMITY_RADIUS
            {
                arrived = Some(elapsed);
            }
            if engine
                .world
                .get::<&Inventory>(entity)
                .expect("inv")
                .get(ResourceKind::Food)
                == 0
            {
                break;
            }
        }
        assert!(arrived.is_some(), "hauler never reached the deposit radius");
        assert_eq!(
            engine
                .world
                .get::<&Inventory>(entity)
                .expect("inv")
                .get(ResourceKind::Food),
            0
        );
        assert!(
            food_stock(&engine) >= before + 6,
            "stock {} did not gain the load over {before}",
            food_stock(&engine)
        );
        // Baseline: other villagers haul in the same window (measured 5 deposits
        // on seed 4 instead of v10's 1), so the tally only bounds from below; the
        // hauler's own deposit is its load emptying inside the deposit radius.
        assert!(engine.resources.tallies.haul_deposits > deposits);
        let pos = *engine.world.get::<&Position>(entity).expect("pos");
        assert!(
            pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1)) <= SETTLEMENT_PROXIMITY_RADIUS,
            "load emptied away from home at {pos:?}"
        );
        assert!(engine.resources.settlements.contains_key(&sid));
    }

    #[test]
    fn harness_forage_claims_cap() {
        let bushes = [(30, 20), (20, 31)];
        let (mut engine, entities) = forage_fixture(3, &bushes, 1);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = targets(&engine, &entities);
        assert_eq!(
            chosen.iter().filter(|t| t.is_some()).count(),
            2,
            "{chosen:?}"
        );
        for bush in bushes {
            assert_eq!(
                chosen.iter().filter(|t| **t == Some(bush)).count(),
                1,
                "{chosen:?}"
            );
        }
        assert_eq!(engine.resources.tallies.forage_trips, 2);

        let (mut engine, entities) = forage_fixture(4, &bushes, 2);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let chosen = targets(&engine, &entities);
        for bush in bushes {
            assert_eq!(
                chosen.iter().filter(|t| **t == Some(bush)).count(),
                2,
                "{chosen:?}"
            );
        }
    }

    #[test]
    fn harness_forage_retargets_when_bush_empties() {
        let bushes = [(32, 20), (20, 33)];
        let (mut engine, entities) = forage_fixture(1, &bushes, 1);
        let forager = entities[0];
        let mut movement = AgentMovementSystem::new();
        let mut decision = AgentDecisionSystem::new();
        let mut phase = 0;
        for tick in 0..400u64 {
            engine.resources.current_tick = tick;
            movement.tick(&mut engine.world, &mut engine.resources);
            decision.tick(&mut engine.world, &mut engine.resources);
            let state = *engine.world.get::<&AgentState>(forager).expect("state");
            let pos = *engine.world.get::<&Position>(forager).expect("pos");
            let seek = engine
                .world
                .get::<&SeekTarget>(forager)
                .ok()
                .map(|s| s.tile);
            // After every decision pass a gatherer heads to a tile that holds food:
            // a bush emptied since the last tick is replaced within that one tick.
            if matches!(
                state,
                AgentState::Seeking {
                    target: TargetKind::GatherFood
                }
            ) {
                let target = seek.unwrap_or_else(|| {
                    panic!("forager without a target at tick {tick} at {pos:?}")
                });
                assert!(
                    engine.resources.food_tiles.contains_key(&target),
                    "seeking empty tile {target:?} at tick {tick}"
                );
            }
            match phase {
                0 if seek == Some(bushes[0]) && tick > 16 => {
                    // Empty the claimed bush before the forager arrives.
                    engine.resources.set_food_tile(32, 20, 0);
                    phase = 1;
                }
                1 => {
                    let retargeted = engine
                        .world
                        .get::<&SeekTarget>(forager)
                        .ok()
                        .map(|s| s.tile);
                    assert_eq!(
                        retargeted,
                        Some(bushes[1]),
                        "no re-target after the bush emptied"
                    );
                    engine.resources.set_food_tile(20, 33, 0);
                    phase = 2;
                }
                2 => {
                    assert_eq!(
                        state,
                        AgentState::Idle,
                        "forager must idle when no bush qualifies"
                    );
                    assert!(engine.world.get::<&SeekTarget>(forager).is_err());
                    return;
                }
                _ => {}
            }
        }
        panic!("scenario did not complete (phase {phase})");
    }

    #[test]
    fn harness_forage_gatherer_whose_tile_vanished_idles_instead_of_freezing() {
        // A gatherer without a target and no food within reach (its tile was taken
        // earlier in the same tick) must not stay frozen in Seeking{GatherFood}.
        let (mut engine, entities) = forage_fixture(1, &[], 1);
        let gatherer = entities[0];
        engine
            .world
            .insert_one(
                gatherer,
                AgentState::Seeking {
                    target: TargetKind::GatherFood,
                },
            )
            .expect("state");
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&AgentState>(gatherer).expect("state"),
            AgentState::Idle
        );
        assert!(engine.world.get::<&SeekTarget>(gatherer).is_err());
    }

    #[test]
    fn harness_haul_home_after_the_forage_window() {
        // Outside the window even one carried unit is hauled home and deposited.
        let mut engine = baseline_engine(4);
        let end = u64::from(
            engine
                .resources
                .rules
                .agent
                .forage
                .expect("forage")
                .end_minute,
        );
        while engine.current_tick() + 480 < end + 10 {
            engine.tick();
        }
        let (home, member) = {
            let settlement = engine
                .resources
                .settlements
                .values()
                .next()
                .expect("village");
            let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
            members.sort_unstable();
            (settlement.formation_tile, members[members.len() - 1])
        };
        let entity = engine
            .world
            .query::<&Agent>()
            .iter()
            .find(|(_, a)| a.id == member)
            .map(|(e, _)| e)
            .expect("member");
        let mut inventory = Inventory::default();
        inventory.add(ResourceKind::Food, 1);
        engine
            .world
            .insert(
                entity,
                (
                    Position::new(home.0, home.1 + 10),
                    AgentState::Idle,
                    inventory,
                    Hunger::new(0.0, 0.05),
                    Thirst::new(0.0, 0.08),
                    Sleep::new(0.0, 0.03),
                    Social::new(0.0, 0.04),
                ),
            )
            .expect("hauler");
        let _ = engine.world.remove_one::<SeekTarget>(entity);
        let (before, deposits) = (food_stock(&engine), engine.resources.tallies.haul_deposits);
        for _ in 0..(10 * engine.resources.rules.agent.move_interval_ticks + 20) {
            engine.tick();
            if engine
                .world
                .get::<&Inventory>(entity)
                .expect("inv")
                .get(ResourceKind::Food)
                == 0
            {
                break;
            }
        }
        assert_eq!(
            engine
                .world
                .get::<&Inventory>(entity)
                .expect("inv")
                .get(ResourceKind::Food),
            0
        );
        assert!(food_stock(&engine) > before);
        // Other villagers also deposit (measured 36 vs 35 on baseline seed 4), so
        // the hauler's own deposit is its load emptying inside the deposit radius.
        assert!(engine.resources.tallies.haul_deposits > deposits);
        let pos = *engine.world.get::<&Position>(entity).expect("pos");
        assert!(
            pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1)) <= SETTLEMENT_PROXIMITY_RADIUS,
            "load emptied away from home at {pos:?}"
        );
    }
}

// From `harness_revitalize_pass5_meals.rs`.
mod meals {
    //! Pass 5 Part 2: meals at the hearth served from the settlement store.
    use hecs::Entity;
    use sim_core::components::{
        Agent, AgentState, Hunger, Inventory, ResourceKind, Settlement, Sleep, Social, TargetKind,
        Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::MovementRng;
    use sim_systems::runtime::decision::AgentDecisionSystem;
    use sim_systems::runtime::settlement::StockpileConsumeSystem;

    const HOME: (u32, u32) = (20, 20);
    /// World seed of the fixture rules; the meal rule does not depend on it.
    const SEED: u64 = 1;

    /// Tick at which the display clock (480-tick offset) reads `minute`.
    fn tick_at(minute: u64) -> u64 {
        (minute + 1440 - 480) % 1440
    }

    /// Bare world: one baseline settlement at `HOME` with `store` food and one member
    /// per `(position, hunger, thirst)`; a stocked bush four tiles from camp and
    /// one far away, water on the river bank. Clock at `minute`.
    fn meal_fixture(
        members: &[((u32, u32), f32, f64)],
        store: u32,
        minute: u64,
    ) -> (SimEngine, Vec<Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = WorldRules::baseline(SEED);
        engine.resources.current_tick = tick_at(minute);
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        settlement.store(ResourceKind::Food, store);
        let mut entities = Vec::new();
        for (index, &((x, y), hunger, thirst)) in members.iter().enumerate() {
            let entity = engine.spawn_agent(x, y);
            let id = engine.world.get::<&Agent>(entity).expect("agent").id;
            engine
                .world
                .insert(
                    entity,
                    (
                        AgentState::Idle,
                        Hunger::new(hunger, 0.05),
                        Thirst::new(thirst, 0.08),
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
        for (x, y) in [(24, 20), (50, 50)] {
            engine.resources.set_food_tile(x, y, 4);
            engine.resources.set_food_source_max(x, y, 4);
        }
        engine.resources.set_water_tile(20, 30, 255);
        (engine, entities)
    }

    fn store(engine: &SimEngine) -> u32 {
        engine.resources.settlements[&1]
            .stockpile
            .get(&ResourceKind::Food)
            .copied()
            .unwrap_or(0)
    }

    fn state(engine: &SimEngine, entity: Entity) -> AgentState {
        *engine.world.get::<&AgentState>(entity).expect("state")
    }

    fn hunger(engine: &SimEngine, entity: Entity) -> f32 {
        engine.world.get::<&Hunger>(entity).expect("hunger").value
    }

    #[test]
    fn harness_meal_breakfast_from_store() {
        let at_camp = [
            ((20, 20), 30.0, 0.0),
            ((21, 19), 30.0, 0.0),
            ((26, 24), 30.0, 0.0),
        ];
        let (mut engine, entities) = meal_fixture(&at_camp, 50, 390);
        let meals = engine.resources.tallies.meals_from_store;
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(store(&engine), 50 - 3, "one unit per member at breakfast");
        assert_eq!(engine.resources.tallies.meals_from_store, meals + 3);
        for &entity in &entities {
            assert_eq!(hunger(&engine, entity), 0.0);
        }
        for _ in 0..5 {
            engine.resources.current_tick += 1;
            AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
            for &entity in &entities {
                assert_ne!(
                    state(&engine, entity),
                    AgentState::Seeking {
                        target: TargetKind::Food
                    }
                );
            }
        }
        // Outside the meal windows hunger 30 is not served.
        let (mut engine, _) = meal_fixture(&at_camp, 50, 720);
        StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(store(&engine), 50);
        // Dinner serves like breakfast.
        let (mut engine, _) = meal_fixture(&at_camp, 50, 1140);
        StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(store(&engine), 47);
    }

    #[test]
    fn harness_meal_hungry_at_camp_waits_for_store() {
        // Noon (outside the meal windows, inside the forage window); a bush four
        // tiles away. Thirst 60 is breached but not critical, so it must not send
        // the waiting member to the river either.
        // Start one tick after a serving pass (the system runs every 5 ticks), so
        // the member waits through four decision ticks before it is served.
        let (mut engine, entities) = meal_fixture(&[((21, 21), 55.0, 60.0)], 10, 721);
        let member = entities[0];
        let mut decision = AgentDecisionSystem::new();
        let mut consume = StockpileConsumeSystem::new();
        let start = engine.resources.current_tick;
        assert_eq!(start % 5, 1);
        let mut fed_at = None;
        for tick in start..start + 5 {
            engine.resources.current_tick = tick;
            decision.tick(&mut engine.world, &mut engine.resources);
            if fed_at.is_none() {
                assert_eq!(
                    state(&engine, member),
                    AgentState::Idle,
                    "waiting member left at tick {tick}"
                );
            }
            if tick % 5 == 0 {
                consume.tick(&mut engine.world, &mut engine.resources);
            }
            if fed_at.is_none() && hunger(&engine, member) < 50.0 {
                fed_at = Some(tick);
            }
        }
        let fed_at = fed_at.expect("member was not fed within five ticks");
        assert_eq!(fed_at - start, 4);
        assert_eq!(store(&engine), 9);
        assert_eq!(engine.resources.tallies.forage_trips, 0);

        // A critical water need is the one exception.
        let (mut engine, entities) = meal_fixture(&[((21, 21), 55.0, 80.0)], 10, 720);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            state(&engine, entities[0]),
            AgentState::Seeking {
                target: TargetKind::Water
            }
        );
    }

    #[test]
    fn harness_meal_away_eats_at_bush() {
        let (mut engine, entities) = meal_fixture(&[((32, 20), 55.0, 0.0)], 10, 720);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            state(&engine, entities[0]),
            AgentState::Seeking {
                target: TargetKind::Food
            }
        );
        StockpileConsumeSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            store(&engine),
            10,
            "a member twelve tiles out is not served"
        );
    }

    #[test]
    fn harness_meal_empty_store_falls_back() {
        let (mut engine, entities) = meal_fixture(&[((21, 21), 55.0, 0.0)], 0, 720);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            state(&engine, entities[0]),
            AgentState::Seeking {
                target: TargetKind::Food
            }
        );
    }

    #[test]
    fn harness_meal_waits_when_every_bush_is_empty() {
        // Review fix: with need fallthrough an empty map hides the Food need, but a
        // hungry member at a stocked camp still waits for its meal; no firewood or
        // other arm sends it out (11:00, inside the firewood window, wood below target).
        let (mut engine, entities) = meal_fixture(&[((21, 21), 60.0, 0.0)], 10, 661);
        engine.resources.food_tiles.clear();
        engine.resources.wood_tiles.insert((26, 20), 6);
        engine.resources.wood_source_max.insert((26, 20), 6);
        let start = engine.resources.current_tick;
        let mut consume = StockpileConsumeSystem::new();
        for tick in start..start + 5 {
            engine.resources.current_tick = tick;
            AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
            if hunger(&engine, entities[0]) > 50.0 {
                assert_eq!(state(&engine, entities[0]), AgentState::Idle, "tick {tick}");
            }
            if tick % 5 == 0 {
                consume.tick(&mut engine.world, &mut engine.resources);
            }
        }
        assert!(
            hunger(&engine, entities[0]) < 50.0,
            "the member was not fed"
        );
        assert_eq!(
            engine.resources.tallies.wood_trips, 0,
            "no trip while it waited"
        );
    }
}
