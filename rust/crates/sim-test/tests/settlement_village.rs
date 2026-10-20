//! Merged in V8-0 (8.4) from `harness_revitalize_pass3_village.rs`,
//! `harness_revitalize_loop.rs`; each source is one module below, unchanged.

// From `harness_revitalize_pass3_village.rs`.
mod shelters {
    use sim_core::components::{
        building_kind::shelter_blueprint, AgentState, BuildingKind, ConstructionSite, Position,
        Settlement, TargetKind,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{HousingCount, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::construction::ConstructionSystem;
    use sim_systems::runtime::settlement::{
        housing_capacity, SettlementSystem, ShelterPlanningSystem,
    };
    use sim_test::baseline_engine;

    /// The live game's village rules (`WorldRules::baseline`) copied into
    /// `WorldRules::default()` (R1, exact: `shelter_planning` and the hearth
    /// anchor are unchanged since v9).
    fn village_rules() -> WorldRules {
        let live = WorldRules::baseline(7);
        let mut rules = WorldRules::default();
        rules.society.shelter_planning = live.society.shelter_planning;
        rules.society.settlement_anchor = live.society.settlement_anchor;
        rules
    }

    #[test]
    fn harness_village_one_settlement_per_hearth() {
        // Baseline scene (R3). Two days stay well before the first possible
        // fission (`min_age_days` 10), so one hearth means one village owning
        // every building (8 seeds × 2 days: under 1 s in release).
        for seed in 0..8 {
            let mut engine = baseline_engine(seed);
            let mut formed = false;
            for _ in 0..2880 {
                engine.tick();
                if !formed && !engine.resources.settlements.is_empty() {
                    assert!(
                        engine.resources.building_owner.len() >= 5,
                        "ownership at formation"
                    );
                }
                formed |= !engine.resources.settlements.is_empty();
                if formed && engine.current_tick().is_multiple_of(60) {
                    assert_eq!(
                        engine.resources.settlements.len(),
                        1,
                        "seed {seed} tick {}",
                        engine.current_tick()
                    );
                    let sid = *engine.resources.settlements.keys().next().expect("village");
                    assert!(engine
                        .resources
                        .building_owner
                        .values()
                        .all(|owner| *owner == sid));
                    assert!(engine.resources.building_owner.len() >= 5);
                }
            }
            assert!(formed, "seed {seed}");
        }
    }

    #[test]
    fn harness_village_owned_capacity_counts_each_shelter_once() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        for sid in [1, 2] {
            let mut settlement = Settlement::new_with_id(sid, 0);
            settlement.formation_tile = (10, 10);
            engine.resources.settlements.insert(sid, settlement);
        }
        for x in 10..14 {
            engine
                .resources
                .building_kinds
                .insert((x, 10), BuildingKind::Shelter);
            engine.resources.building_owner.insert((x, 10), 1);
        }
        let rule = village_rules().society.shelter_planning.expect("planning");
        // Baseline counts owned housing; the v8 `NearFormation` arm is gone.
        assert_eq!(rule.housing, HousingCount::Owned);
        assert_eq!(housing_capacity(&engine.resources, 1, &rule), 32);
        assert_eq!(housing_capacity(&engine.resources, 2, &rule), 0);
    }

    #[test]
    fn harness_village_plans_with_headroom() {
        // One owned shelter (8 places) and baseline headroom 2: six residents
        // (6 + 2 ≥ 8) already plan a second shelter, five (5 + 2 < 8) do not.
        for (members_below_capacity, expected_sites) in [(2, 1), (3, 0)] {
            let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
            engine.resources.rules = village_rules();
            let rule = engine
                .resources
                .rules
                .society
                .shelter_planning
                .expect("planning");
            assert_eq!(rule.planning_headroom, 2);
            let mut settlement = Settlement::new_with_id(1, 0);
            settlement.formation_tile = (10, 10);
            for id in 0..rule.residents_per_shelter - members_below_capacity {
                settlement.add_member_agent(u64::from(id));
            }
            engine.resources.settlements.insert(1, settlement);
            engine
                .resources
                .building_kinds
                .insert((10, 10), BuildingKind::Shelter);
            engine.resources.building_owner.insert((10, 10), 1);
            let mut planner = ShelterPlanningSystem::default();
            for tick in 1..=rule.plan_interval_ticks {
                engine.resources.current_tick = tick;
                planner.tick(&mut engine.world, &mut engine.resources);
            }
            assert_eq!(
                engine.world.query::<&ConstructionSite>().iter().count(),
                expected_sites,
                "{} residents",
                rule.residents_per_shelter - members_below_capacity
            );
        }
    }

    #[test]
    fn harness_village_completed_site_is_owned() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = village_rules();
        let rule = engine
            .resources
            .rules
            .society
            .shelter_planning
            .expect("planning");
        engine
            .resources
            .settlements
            .insert(1, Settlement::new_with_id(1, 0));
        let tile = (10, 10);
        engine.world.spawn((ConstructionSite::new(
            shelter_blueprint(1),
            Position::new(tile.0, tile.1),
        ),));
        engine.world.spawn((
            Position::new(tile.0, tile.1),
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        ));
        engine.resources.site_owner.insert(tile, 1);
        let before = housing_capacity(&engine.resources, 1, &rule);
        ConstructionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.resources.building_owner.get(&tile), Some(&1));
        assert_eq!(
            housing_capacity(&engine.resources, 1, &rule),
            before + rule.residents_per_shelter
        );
        assert!(!engine.resources.site_owner.contains_key(&tile));
    }

    #[test]
    fn harness_village_dissolution_releases_only_its_owned_buildings() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = village_rules();
        engine
            .resources
            .settlements
            .insert(1, Settlement::new_with_id(1, 0));
        engine.resources.building_owner.insert((10, 10), 1);
        engine.resources.building_owner.insert((20, 20), 2);
        SettlementSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(!engine.resources.settlements.contains_key(&1));
        assert!(!engine.resources.building_owner.contains_key(&(10, 10)));
        assert_eq!(engine.resources.building_owner.get(&(20, 20)), Some(&2));
    }
}

// From `harness_revitalize_loop.rs`.
mod living_loop {
    use sim_core::components::{
        Agent, AgentState, BuildingKind, Hunger, Position, ResourceKind, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{BirthRule, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
    use sim_systems::runtime::settlement::SettlementSystem;
    use sim_test::baseline_engine;

    #[test]
    fn harness_population_game_world_survives_3_days() {
        // Baseline scene (R3; was the v8 `game_scene`).
        for seed in 1..=3 {
            let mut engine = baseline_engine(seed);
            let (mut samples, mut breach) = (0_u64, 0_u64);
            for _ in 0..4320 {
                engine.tick();
                for (_, (state, hunger, thirst)) in engine
                    .world
                    .query::<(&AgentState, &Hunger, &Thirst)>()
                    .iter()
                {
                    samples += 1;
                    if matches!(state, AgentState::Idle)
                        && (hunger.value > 50.0 || thirst.value > 50.0)
                    {
                        breach += 1;
                    }
                }
            }
            assert_eq!(
                engine.resources.tallies.deaths_dehydration, 0,
                "seed {seed}"
            );
            // Founders are the world's draw now (was a fixed 24 with a floor of
            // 20). Measured on baseline seeds 1..=3: 23 -> 28, 21 -> 26 and
            // 26 -> 27 residents after 3 days, idle need breaches 322/109,996,
            // 334/101,520 and 293/115,920 (about 0.3 %, bar lowered from 10 % to 2 %).
            let founders = sim_systems::worldgen::WorldGenConfig::baseline(seed).starting_agents;
            assert!(
                engine.world.query::<&Agent>().iter().count() >= founders as usize,
                "seed {seed}"
            );
            assert!(
                breach as f64 / samples as f64 <= 0.02,
                "seed {seed}: {breach}/{samples}"
            );
        }
    }

    #[test]
    fn harness_building_shelters_planned_and_completed() {
        let mut engine = baseline_engine(1);
        let initial = engine.resources.building_kinds.clone();
        let mut planned = std::collections::BTreeSet::new();
        for _ in 0..4320 {
            engine.tick();
            planned.extend(engine.resources.site_owner.keys().copied());
        }
        let completed: Vec<_> = engine
            .resources
            .building_kinds
            .iter()
            .filter(|(tile, kind)| **kind == BuildingKind::Shelter && !initial.contains_key(tile))
            .collect();
        // Baseline seed 1 (R3; was the v8 `game_scene`): measured one site planned
        // by ShelterPlanningSystem and completed within 3 days.
        assert!(!completed.is_empty());
        let sites_now: std::collections::BTreeSet<_> = engine
            .world
            .query::<&sim_core::components::ConstructionSite>()
            .iter()
            .map(|(_, s)| (s.position.x, s.position.y))
            .collect();
        // No planned site is lost: each became a shelter or is still being built.
        for tile in &planned {
            assert!(
                engine.resources.building_kinds.get(tile) == Some(&BuildingKind::Shelter)
                    || sites_now.contains(tile),
                "planned site {tile:?} vanished"
            );
        }
        for (tile, _) in completed {
            assert!(planned.contains(tile));
            assert_eq!(
                engine.resources.sleep_tiles.get(tile),
                Some(&RESOURCE_SOURCE_INFINITE)
            );
            assert!(!engine.resources.site_owner.contains_key(tile));
        }
        let sites: std::collections::BTreeSet<_> = engine
            .world
            .query::<&sim_core::components::ConstructionSite>()
            .iter()
            .map(|(_, s)| (s.position.x, s.position.y))
            .collect();
        assert!(engine
            .resources
            .site_owner
            .keys()
            .all(|tile| sites.contains(tile)));
    }

    #[test]
    fn harness_population_births_gated_by_food_and_housing() {
        // R1: baseline shelter planning (owned housing), hearth anchor and the
        // baseline sustain gate; cooldown 10, food cost 6 and 4 residents a
        // shelter are test-local numeric values (R2). Three residents and one owned
        // shelter: food admits one birth, then housing (4 of 4) blocks the next.
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let live = WorldRules::baseline(7);
        let BirthRule::Provisioned { sustain, .. } = live.society.birth else {
            panic!("baseline birth is provisioned");
        };
        engine.resources.rules.society.shelter_planning = live.society.shelter_planning;
        engine.resources.rules.society.settlement_anchor = live.society.settlement_anchor;
        engine.resources.rules.society.birth = BirthRule::Provisioned {
            cooldown_ticks: 10,
            food_cost: 6,
            max_mean_hunger: 60.0,
            max_mean_thirst: 60.0,
            sustain,
        };
        engine
            .resources
            .rules
            .society
            .shelter_planning
            .as_mut()
            .expect("planning")
            .residents_per_shelter = 4;
        for _ in 0..3 {
            let agent = engine.spawn_agent(10, 10);
            engine
                .world
                .insert(agent, (Hunger::new(0.0, 0.0), Thirst::new(0.0, 0.0)))
                .expect("components");
        }
        engine.resources.building_registry.insert(100, (10, 10));
        engine
            .resources
            .building_kinds
            .insert((10, 10), BuildingKind::Hearth);
        engine.resources.building_registry.insert(101, (11, 10));
        for x in 8..12 {
            engine.resources.food_source_max.insert((x, 14), 4);
        }
        engine
            .resources
            .building_kinds
            .insert((11, 10), BuildingKind::Shelter);
        let mut system = SettlementSystem::new();
        for tick in 0..=20 {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
        assert_eq!(engine.resources.tallies.births, 0);
        assert_eq!(engine.resources.settlements.len(), 1);
        let id = *engine
            .resources
            .settlements
            .keys()
            .next()
            .expect("settlement");
        engine
            .resources
            .settlements
            .get_mut(&id)
            .expect("settlement")
            .store(ResourceKind::Food, 12);
        engine.resources.current_tick = 21;
        system.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.resources.tallies.births, 1);
        assert_eq!(
            engine.resources.settlements[&id].stockpile[&ResourceKind::Food],
            6
        );
        for tick in 22..=42 {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
        assert_eq!(engine.resources.tallies.births, 1, "housing is now full");
        assert_eq!(
            engine.resources.settlements[&id].stockpile[&ResourceKind::Food],
            6
        );
    }

    #[test]
    fn harness_population_rules_default_is_legacy() {
        let rules = WorldRules::default();
        assert_eq!(rules.agent.move_interval_ticks, 1);
        assert_eq!(rules.agent.idle_pause_chance, 0.0);
        assert_eq!(rules.agent.idle_tether_radius, None);
        assert_eq!(rules.society.birth, BirthRule::Legacy);
        assert_eq!(rules.society.shelter_planning, None);
    }

    #[test]
    fn harness_building_assigned_builder_walks_and_clears_missing_site() {
        use sim_core::components::{
            building_kind::shelter_blueprint, ConstructionSite, SeekTarget, Settlement, TargetKind,
        };
        use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
        use sim_systems::runtime::decision::AgentDecisionSystem;
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = WorldRules::baseline(7);
        engine.resources.rules.agent.move_interval_ticks = 1;
        engine.resources.rules.agent.idle_pause_chance = 1.0;
        let entity = engine.spawn_agent(10, 10);
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    MovementRng::new(1),
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                ),
            )
            .expect("components");
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        let mut settlement = Settlement::new_with_id(0, 0);
        settlement.formation_tile = (10, 10);
        settlement.add_member_agent(id);
        engine.resources.settlements.insert(0, settlement);
        let site = engine.world.spawn((ConstructionSite::new(
            shelter_blueprint(240),
            Position::new(14, 10),
        ),));
        engine.resources.site_owner.insert((14, 10), 0);
        let mut decisions = AgentDecisionSystem::new();
        let mut movement = AgentMovementSystem::new();
        decisions.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            }
        );
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("target")
                .tile,
            (14, 10)
        );
        for _ in 0..3 {
            movement.tick(&mut engine.world, &mut engine.resources);
            decisions.tick(&mut engine.world, &mut engine.resources);
            assert_eq!(
                *engine.world.get::<&AgentState>(entity).expect("state"),
                AgentState::Seeking {
                    target: TargetKind::ConstructionSite
                }
            );
        }
        assert_eq!(
            engine.world.get::<&Position>(entity).expect("position").x,
            13
        );
        engine.world.despawn(site).expect("site");
        decisions.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Idle
        );
        assert!(engine.world.get::<&SeekTarget>(entity).is_err());
    }

    #[test]
    fn harness_population_movement_only_steps_on_configured_tick_multiples() {
        use sim_core::components::{SeekTarget, TargetKind};
        use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules.agent.move_interval_ticks = 6;
        let rng = MovementRng::new(123);
        // Identity remains optional: the legacy movement surface also handles
        // entities carrying Position/MovementRng without the Agent component.
        let entity = engine.world.spawn((
            Position::new(3, 4),
            rng,
            AgentState::Seeking {
                target: TargetKind::Water,
            },
            SeekTarget::new((15, 4)),
        ));
        let mut movement = AgentMovementSystem::new();
        for tick in 0..=12 {
            engine.resources.current_tick = tick;
            movement.tick(&mut engine.world, &mut engine.resources);
            let expected_x = match tick {
                0..=5 => 4,
                6..=11 => 5,
                _ => 6,
            };
            assert_eq!(
                *engine.world.get::<&Position>(entity).expect("position"),
                Position::new(expected_x, 4),
                "tick {tick}"
            );
            assert_eq!(
                *engine.world.get::<&MovementRng>(entity).expect("rng"),
                rng,
                "directed movement and skipped ticks must consume no random samples"
            );
        }
    }
}
