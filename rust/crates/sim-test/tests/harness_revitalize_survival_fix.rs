use hecs::Entity;
use sim_core::causal::DeathReason;
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Memory, MemoryArm, MemoryEntry, ResourceKind, SeekTarget,
    Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::survival::despawn_agent;
use sim_test::{probe::fingerprint, scenes::game_scene};

fn fixture() -> (SimEngine, Entity, Entity) {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v8();
    let a = engine.spawn_agent(10, 10);
    let b = engine.spawn_agent(10, 10);
    let partner = engine.world.get::<&Agent>(b).expect("partner").id;
    engine
        .world
        .insert(
            a,
            (
                AgentState::Seeking {
                    target: TargetKind::Agent(partner),
                },
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Social::new(80.0, 0.0),
                MovementRng::new(1),
            ),
        )
        .expect("seeker components");
    engine
        .world
        .insert(b, (AgentState::Idle, Social::new(0.0, 0.0)))
        .expect("partner components");
    (engine, a, b)
}

fn decide(engine: &mut SimEngine, tick: u64) {
    engine.resources.current_tick = tick;
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
}

fn state(engine: &SimEngine, entity: Entity) -> AgentState {
    *engine.world.get::<&AgentState>(entity).expect("state")
}

#[test]
fn harness_social_chase_gives_up_after_limit() {
    let (mut engine, a, b) = fixture();
    let id = engine.world.get::<&Agent>(a).expect("seeker").id;
    let rule = engine.resources.rules.agent.social_seek.expect("game rule");
    for tick in 0..rule.max_chase_ticks {
        decide(&mut engine, tick);
        assert!(matches!(
            state(&engine, a),
            AgentState::Seeking {
                target: TargetKind::Agent(_)
            }
        ));
    }
    decide(&mut engine, rule.max_chase_ticks);
    assert_eq!(state(&engine, a), AgentState::Idle);
    assert!(!engine.resources.social_chase_since.contains_key(&id));
    let retry = engine.resources.social_retry_after[&id];
    assert_eq!(retry, rule.max_chase_ticks + rule.retry_cooldown_ticks);
    engine
        .world
        .get::<&mut Social>(b)
        .expect("social")
        .loneliness = 80.0;
    for tick in rule.max_chase_ticks + 1..retry {
        decide(&mut engine, tick);
        assert_eq!(state(&engine, a), AgentState::Idle);
        assert_eq!(
            state(&engine, b),
            AgentState::Idle,
            "cooldown seeker cannot be selected as a partner"
        );
    }
    decide(&mut engine, retry);
    assert!(!engine.resources.social_retry_after.contains_key(&id));
    assert!(matches!(
        state(&engine, a),
        AgentState::Seeking {
            target: TargetKind::Agent(_)
        }
    ));
}

#[test]
fn harness_social_chase_aborts_when_partner_dies() {
    let (mut engine, a, b) = fixture();
    let aid = engine.world.get::<&Agent>(a).expect("seeker").id;
    let bid = engine.world.get::<&Agent>(b).expect("partner").id;
    engine.register_system(Box::new(AgentDecisionSystem::new()));
    for _ in 0..20 {
        engine.tick();
    }
    assert!(matches!(
        state(&engine, a),
        AgentState::Seeking {
            target: TargetKind::Agent(_)
        }
    ));
    assert_eq!(engine.resources.social_chase_since[&aid], 0);
    engine.resources.social_chase_since.insert(bid, 0);
    engine.resources.social_retry_after.insert(bid, 50);
    despawn_agent(
        &mut engine.world,
        &mut engine.resources,
        b,
        bid,
        (10, 10),
        DeathReason::Combat,
        20,
    );
    assert!(!engine.resources.social_chase_since.contains_key(&bid));
    assert!(!engine.resources.social_retry_after.contains_key(&bid));
    engine.tick();
    assert_eq!(engine.current_tick(), 21);
    assert_eq!(state(&engine, a), AgentState::Idle);
    assert_eq!(engine.resources.social_retry_after[&aid], 500);
    assert!(!engine.resources.social_chase_since.contains_key(&aid));
}

#[test]
fn harness_social_chase_yields_to_thirst() {
    let (mut engine, a, _) = fixture();
    let id = engine.world.get::<&Agent>(a).expect("seeker").id;
    engine.world.get::<&mut Thirst>(a).expect("thirst").value = 60.0;
    engine
        .resources
        .set_water_tile(12, 10, RESOURCE_SOURCE_INFINITE);
    decide(&mut engine, 0);
    assert_eq!(state(&engine, a), AgentState::Idle);
    assert!(!engine.resources.social_retry_after.contains_key(&id));
    decide(&mut engine, 1);
    assert_eq!(
        state(&engine, a),
        AgentState::Seeking {
            target: TargetKind::Water
        }
    );
    for tick in 2..=600 {
        engine.resources.current_tick = tick;
        AgentMovementSystem::new().tick(&mut engine.world, &mut engine.resources);
        decide(&mut engine, tick);
        if engine.world.get::<&Thirst>(a).expect("thirst").value < 60.0 {
            return;
        }
    }
    panic!("social seeker failed to drink reachable water");
}

#[test]
fn harness_need_fallthrough_drinks_when_no_food() {
    for game in [false, true] {
        let (mut engine, a, _) = fixture();
        if !game {
            engine.resources.rules = WorldRules::default();
        }
        *engine.world.get::<&mut AgentState>(a).expect("state") = AgentState::Idle;
        engine.world.get::<&mut Hunger>(a).expect("hunger").value = 80.0;
        engine.world.get::<&mut Thirst>(a).expect("thirst").value = 70.0;
        engine
            .resources
            .set_water_tile(10, 10, RESOURCE_SOURCE_INFINITE);
        decide(&mut engine, 0);
        assert_eq!(
            state(&engine, a),
            AgentState::Seeking {
                target: if game {
                    TargetKind::Water
                } else {
                    TargetKind::Food
                }
            }
        );
    }
}

#[test]
fn harness_economy_clock_initial_values() {
    assert_eq!(game_scene(1).resources.time_of_day, 8.0);
    assert_eq!(
        SimEngine::new(32, 32, MaterialRegistry::new())
            .resources
            .time_of_day,
        0.0
    );
}

#[test]
fn harness_economy_clock_offset_is_display_only() {
    harness_economy_clock_initial_values();
    let mut midnight = game_scene(1);
    let mut morning = game_scene(1);
    midnight.resources.rules.clock_offset_ticks = 0;
    midnight.resources.time_of_day = 0.0;
    for tick in 0..2880 {
        assert_eq!(fingerprint(&midnight), fingerprint(&morning), "tick {tick}");
        midnight.tick();
        morning.tick();
    }
    assert_eq!(fingerprint(&midnight), fingerprint(&morning));
}

#[test]
fn harness_need_fallthrough_excludes_unavailable_memory_bias_candidates() {
    for food_available in [false, true] {
        let (mut engine, a, _) = fixture();
        *engine.world.get::<&mut AgentState>(a).expect("state") = AgentState::Idle;
        engine.world.get::<&mut Hunger>(a).expect("hunger").value = 80.0;
        engine.world.get::<&mut Thirst>(a).expect("thirst").value = 70.0;
        engine
            .resources
            .set_water_tile(10, 10, RESOURCE_SOURCE_INFINITE);
        if food_available {
            engine.resources.set_food_tile(10, 10, 1);
        }
        let mut memory = Memory::new();
        for _ in 0..3 {
            memory.insert(MemoryEntry::new(
                engine.resources.issue_event_id(),
                0,
                1.0,
                1.0,
                if food_available {
                    MemoryArm::Thirst
                } else {
                    MemoryArm::Hunger
                },
            ));
        }
        engine.world.insert_one(a, memory).expect("memory");
        decide(&mut engine, 0);
        // With food present, positive thirst memories override natural Hunger.
        // Without food, even strong hunger memories cannot select unavailable Food.
        assert_eq!(
            state(&engine, a),
            AgentState::Seeking {
                target: TargetKind::Water
            },
            "food available: {food_available}"
        );
    }
}

#[test]
fn harness_critical_preemption_obeys_boundaries_availability_and_legacy() {
    for (legacy, hunger, food_available, interrupted) in [
        (false, 75.0, true, false),
        (false, 76.0, true, true),
        (false, 100.0, false, false),
        (true, 100.0, true, false),
    ] {
        let (mut engine, a, _) = fixture();
        if legacy {
            engine.resources.rules = WorldRules::default();
        }
        *engine.world.get::<&mut AgentState>(a).expect("state") = AgentState::Seeking {
            target: TargetKind::Water,
        };
        engine.world.get::<&mut Hunger>(a).expect("hunger").value = hunger;
        engine.world.get::<&mut Thirst>(a).expect("thirst").value = 60.0;
        engine
            .resources
            .set_water_tile(12, 10, RESOURCE_SOURCE_INFINITE);
        if food_available {
            engine.resources.set_food_tile(13, 10, 1);
        }
        let mut inventory = Inventory::default();
        inventory.add(ResourceKind::Food, 2);
        engine
            .world
            .insert(a, (SeekTarget { tile: (12, 10) }, inventory))
            .expect("intent");
        decide(&mut engine, 0);
        assert_eq!(
            state(&engine, a),
            if interrupted {
                AgentState::Idle
            } else {
                AgentState::Seeking {
                    target: TargetKind::Water,
                }
            }
        );
        assert_eq!(
            engine
                .world
                .get::<&Inventory>(a)
                .expect("inventory")
                .get(ResourceKind::Food),
            2
        );
        if interrupted {
            assert!(engine.world.get::<&SeekTarget>(a).is_err());
            decide(&mut engine, 1);
            assert_eq!(
                state(&engine, a),
                AgentState::Seeking {
                    target: TargetKind::Food
                }
            );
            assert_eq!(
                engine.world.get::<&SeekTarget>(a).expect("food goal").tile,
                (13, 10)
            );
        }
    }
}

#[test]
fn harness_critical_preemption_never_restarts_the_same_actionable_need() {
    for target in [TargetKind::Food, TargetKind::Water] {
        let (mut engine, a, _) = fixture();
        *engine.world.get::<&mut AgentState>(a).expect("state") = AgentState::Seeking { target };
        engine.world.get::<&mut Hunger>(a).expect("hunger").value = if target == TargetKind::Food {
            100.0
        } else {
            0.0
        };
        engine.world.get::<&mut Thirst>(a).expect("thirst").value = 100.0;
        engine
            .resources
            .set_water_tile(12, 10, RESOURCE_SOURCE_INFINITE);
        if target == TargetKind::Food {
            engine.resources.set_food_tile(12, 10, 1);
        }
        engine
            .world
            .insert_one(a, SeekTarget { tile: (12, 10) })
            .expect("intent");
        for tick in 0..3 {
            decide(&mut engine, tick);
            assert_eq!(state(&engine, a), AgentState::Seeking { target });
            assert_eq!(
                engine
                    .world
                    .get::<&SeekTarget>(a)
                    .expect("retained goal")
                    .tile,
                (12, 10)
            );
        }
    }
}

#[test]
fn harness_critical_preemption_food_seek_yields_to_available_critical_water() {
    let (mut engine, a, _) = fixture();
    *engine.world.get::<&mut AgentState>(a).expect("state") = AgentState::Seeking {
        target: TargetKind::Food,
    };
    engine.world.get::<&mut Hunger>(a).expect("hunger").value = 100.0;
    engine.world.get::<&mut Thirst>(a).expect("thirst").value = 76.0;
    engine
        .resources
        .set_water_tile(12, 10, RESOURCE_SOURCE_INFINITE);
    engine
        .world
        .insert_one(a, SeekTarget { tile: (13, 10) })
        .expect("stale food goal");
    decide(&mut engine, 0);
    assert_eq!(state(&engine, a), AgentState::Idle);
    assert!(engine.world.get::<&SeekTarget>(a).is_err());
    decide(&mut engine, 1);
    assert_eq!(
        state(&engine, a),
        AgentState::Seeking {
            target: TargetKind::Water
        }
    );
}

#[test]
fn harness_critical_preemption_protects_an_already_critical_water_trip() {
    for (thirst, interrupted) in [(74.0, true), (75.0, true), (76.0, false), (100.0, false)] {
        let (mut engine, a, _) = fixture();
        *engine.world.get::<&mut AgentState>(a).expect("state") = AgentState::Seeking {
            target: TargetKind::Water,
        };
        engine.world.get::<&mut Hunger>(a).expect("hunger").value = 100.0;
        engine.world.get::<&mut Thirst>(a).expect("thirst").value = thirst;
        engine.resources.set_food_tile(13, 10, 1);
        engine
            .resources
            .set_water_tile(12, 10, RESOURCE_SOURCE_INFINITE);
        engine
            .world
            .insert_one(a, SeekTarget { tile: (12, 10) })
            .expect("water goal");
        decide(&mut engine, 0);
        assert_eq!(
            state(&engine, a),
            if interrupted {
                AgentState::Idle
            } else {
                AgentState::Seeking {
                    target: TargetKind::Water,
                }
            },
            "thirst {thirst}"
        );
        if !interrupted {
            assert_eq!(
                engine
                    .world
                    .get::<&SeekTarget>(a)
                    .expect("retained water goal")
                    .tile,
                (12, 10)
            );
        }
    }
}
