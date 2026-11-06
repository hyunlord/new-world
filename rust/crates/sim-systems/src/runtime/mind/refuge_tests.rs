//! Physical target and authored recovery regressions; no warm/health side effects.
use super::{context::TickContext, refuge, ColonyMindSystem, MindProgram};
use crate::worldgen::{build_world_without_runtime_for_tests, WorldGenConfig};
use sim_core::components::{Agent, BodyHealth, Cold, Hunger, Position, Thirst};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};

fn fixture() -> (SimEngine, hecs::Entity, u64, (u32, u32)) {
    let mut engine =
        build_world_without_runtime_for_tests(&WorldGenConfig::colony(0), WorldRules::colony(0));
    crate::runtime::influence::BuildingStampSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    crate::runtime::settlement::SettlementSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    crate::runtime::thermal::initialize(&mut engine.resources);
    let (entity, id, tile) = engine
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(e, (a, p))| (e, a.id, (p.x, p.y)))
        .min_by_key(|(_, id, _)| *id)
        .expect("person");
    let room = sim_core::rooms::Room {
        id: 900,
        tiles: [tile].into(),
        doors: Default::default(),
        roofed: true,
        role: sim_core::rooms::RoomRole::Unassigned,
        dug: false,
    };
    engine.resources.colony.room_of_tile.insert(tile, 900);
    engine.resources.colony.rooms.insert(900, room);
    engine
        .resources
        .thermal
        .as_mut()
        .expect("thermal")
        .rooms
        .insert(
            900,
            sim_core::colony::thermal::RoomThermal {
                temperature_d10: 200,
                tiles: [tile].into(),
                last_integrated_tick: None,
            },
        );
    engine.resources.water_tiles.insert(tile, 2);
    crate::runtime::food::items::create_ground(
        &mut engine.resources,
        tile,
        sim_core::colony::ItemSpec::raw(
            sim_core::colony::ItemKind::from_name("berries").expect("food"),
            None,
        ),
        2,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("actionable need sources");
    (engine, entity, id, tile)
}

#[test]
fn refuge_requires_real_warmth_and_has_no_effect_on_health() {
    let (mut engine, entity, id, tile) = fixture();
    let context = TickContext::capture(&engine.world, &engine.resources).expect("context");
    let before = *engine.world.get::<&BodyHealth>(entity).expect("hp");
    assert_eq!(
        refuge::target(context.row(id).expect("row"), &context, &engine.resources)
            .expect("warm target")
            .tile,
        tile
    );
    engine
        .resources
        .thermal
        .as_mut()
        .expect("thermal")
        .rooms
        .get_mut(&900)
        .expect("room")
        .temperature_d10 = -200;
    assert!(refuge::target(context.row(id).expect("row"), &context, &engine.resources).is_none());
    assert_eq!(
        *engine.world.get::<&BodyHealth>(entity).expect("hp"),
        before
    );
}

#[test]
fn data_only_refuge_recovers_after_cold_clears_and_yields_to_critical_needs() {
    for thirsty in [true, false] {
        let (mut engine, entity, id, _) = fixture();
        let source = include_str!("../../../../sim-core/data/colony/mind.ron")
            .replace("warm_refuge_enter", "alternate_refuge_enter")
            .replace("warm_refuge_recover", "alternate_refuge_recover");
        let mut mind = ColonyMindSystem::new(MindProgram::load(&source).expect("data policy"));
        engine
            .world
            .insert_one(entity, Cold)
            .expect("initial derived-condition fixture");
        engine
            .world
            .get::<&mut BodyHealth>(entity)
            .expect("hp")
            .apply_damage(1.0);
        engine
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 0.0;
        engine
            .world
            .get::<&mut Thirst>(entity)
            .expect("thirst")
            .value = 0.0;
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("entry");
        assert!(engine
            .world
            .get::<&sim_core::components::Activity>(entity)
            .is_ok());
        engine
            .world
            .remove_one::<Cold>(entity)
            .expect("warm condition fixture");
        engine.resources.current_tick += 1;
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("recovery");
        let context = TickContext::capture(&engine.world, &engine.resources).expect("context");
        assert!(refuge::active(context.row(id).expect("row")));
        if thirsty {
            engine
                .world
                .get::<&mut Thirst>(entity)
                .expect("thirst")
                .value = 90.0;
        } else {
            engine
                .world
                .get::<&mut Hunger>(entity)
                .expect("hunger")
                .value = 90.0;
        }
        engine.resources.current_tick += 1;
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("critical thirst");
        assert!(engine
            .world
            .get::<&sim_core::components::Activity>(entity)
            .is_err());
    }
}

#[test]
fn refuge_releases_after_existing_survival_owner_restores_full_health() {
    let (mut engine, entity, _, _) = fixture();
    let source = include_str!("../../../../sim-core/data/colony/mind.ron");
    let mut mind = ColonyMindSystem::new(MindProgram::load(source).expect("policy"));
    engine.world.insert_one(entity, Cold).expect("cold fixture");
    engine
        .world
        .get::<&mut BodyHealth>(entity)
        .expect("hp")
        .apply_damage(0.01);
    engine
        .world
        .get::<&mut Hunger>(entity)
        .expect("hunger")
        .value = 0.0;
    engine
        .world
        .get::<&mut Thirst>(entity)
        .expect("thirst")
        .value = 0.0;
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .expect("entry");
    assert!(engine
        .world
        .get::<&sim_core::components::Activity>(entity)
        .is_ok());
    engine
        .world
        .remove_one::<Cold>(entity)
        .expect("warm fixture");
    crate::runtime::survival::StarvationSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    let health = *engine.world.get::<&BodyHealth>(entity).expect("health");
    assert_eq!(health.hp, health.max_hp);
    engine.resources.current_tick += 1;
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .expect("release");
    assert!(engine
        .world
        .get::<&sim_core::components::Activity>(entity)
        .is_err());
}

#[path = "refuge_boundary_tests.rs"]
mod boundary;

#[test]
fn refuge_recovery_yields_when_a_need_prevents_existing_healing() {
    for thirsty in [false, true] {
        let (mut engine, entity, _, _) = fixture();
        let mut mind = ColonyMindSystem::new(
            MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
                .expect("policy"),
        );
        engine.world.insert_one(entity, Cold).expect("cold");
        engine
            .world
            .get::<&mut BodyHealth>(entity)
            .expect("health")
            .apply_damage(10.0);
        engine
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 0.0;
        engine
            .world
            .get::<&mut Thirst>(entity)
            .expect("thirst")
            .value = 0.0;
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("enter");
        assert!(engine
            .world
            .get::<&sim_core::components::Activity>(entity)
            .is_ok());
        engine
            .world
            .remove_one::<Cold>(entity)
            .expect("actual warm fixture");
        let blocked_need = crate::runtime::survival::SAFE_NEED_CEILING + 10.0;
        if thirsty {
            engine
                .world
                .get::<&mut Thirst>(entity)
                .expect("thirst")
                .value = blocked_need;
        } else {
            engine
                .world
                .get::<&mut Hunger>(entity)
                .expect("hunger")
                .value = blocked_need as f32;
        }
        let before = *engine.world.get::<&BodyHealth>(entity).expect("health");
        crate::runtime::survival::StarvationSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&BodyHealth>(entity).expect("health"),
            before
        );
        engine.resources.current_tick += 1;
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("meet healing prerequisite");
        assert!(
            engine
                .world
                .get::<&sim_core::components::Activity>(entity)
                .is_err(),
            "recovery cannot monopolize while its own healing prerequisite is unmet"
        );
    }
}

#[test]
fn refuge_recovery_resumes_while_warm_after_need_completion() {
    let (mut engine, entity, id, _) = fixture();
    let mut mind = ColonyMindSystem::new(
        MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
            .expect("policy"),
    );
    engine
        .world
        .get::<&mut BodyHealth>(entity)
        .expect("health")
        .apply_damage(10.0);
    engine
        .world
        .get::<&mut Hunger>(entity)
        .expect("hunger")
        .value = 0.0;
    engine
        .world
        .get::<&mut Thirst>(entity)
        .expect("thirst")
        .value = 0.0;
    engine
        .world
        .insert_one(entity, sim_core::components::AgentState::Idle)
        .expect("completed need");
    let _ = engine.world.remove_one::<Cold>(entity);
    assert!(engine
        .world
        .get::<&sim_core::components::Activity>(entity)
        .is_err());
    let before = *engine.world.get::<&BodyHealth>(entity).expect("health");
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .expect("resume in real warm room");
    assert!(
        engine
            .world
            .get::<&sim_core::components::Activity>(entity)
            .is_ok(),
        "an interrupted recovery must not require becoming cold again"
    );
    assert_eq!(
        *engine.world.get::<&BodyHealth>(entity).expect("health"),
        before
    );
    let policy = MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
        .expect("policy");
    let node = policy
        .policy
        .node(engine.resources.mind.running[&id].node)
        .expect("selected node");
    assert!(
        node.name.starts_with("warm_refuge"),
        "ordinary leisure is not recovery: {}",
        node.name
    );
}

#[test]
fn refuge_recovery_completes_one_real_need_trip_without_cold_recapture() {
    use sim_core::colony::{ItemKind, ItemSpec};
    use sim_core::components::Activity;
    for thirsty in [false, true] {
        let (mut engine, entity, _, home) = fixture();
        let others: Vec<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .filter(|(e, _)| *e != entity)
            .map(|(e, _)| e)
            .collect();
        for other in others {
            engine.world.despawn(other).expect("isolated resident");
        }
        engine.resources.rules.clock_offset_ticks = 18 * engine.resources.ticks_per_day + 480;
        engine.resources.current_tick = 1;
        let source = (home.0 + 1, home.1);
        assert!(engine.resources.nav.reachable(home, source));
        assert!(
            crate::runtime::thermal::felt_temperature(&engine.resources, None, source)
                .expect("outdoor")
                < engine
                    .resources
                    .thermal
                    .as_ref()
                    .expect("thermal")
                    .rules
                    .cold_threshold_d10
        );
        engine.resources.water_tiles.clear();
        engine.resources.colony.ground_items.clear();
        if thirsty {
            engine.resources.water_tiles.insert(source, 2);
        } else {
            let born = crate::runtime::food::day(&engine.resources);
            crate::runtime::food::items::create_ground(
                &mut engine.resources,
                source,
                ItemSpec::raw(ItemKind::from_name("berries").expect("food"), None),
                2,
                born,
                crate::runtime::food::items::FoodSource::Harvest,
            )
            .expect("physical meal");
        }
        engine
            .world
            .get::<&mut BodyHealth>(entity)
            .expect("health")
            .apply_damage(10.0);
        engine
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 0.0;
        engine
            .world
            .get::<&mut Thirst>(entity)
            .expect("thirst")
            .value = 0.0;
        engine
            .world
            .insert_one(entity, Cold)
            .expect("admit recovery");
        let mut mind = ColonyMindSystem::new(
            MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
                .expect("policy"),
        );
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("enter");
        assert!(engine.world.get::<&Activity>(entity).is_ok());
        if thirsty {
            engine
                .world
                .get::<&mut Thirst>(entity)
                .expect("thirst")
                .value = 60.0;
        } else {
            engine
                .world
                .get::<&mut Hunger>(entity)
                .expect("hunger")
                .value = 60.0;
        }
        let mut movement = crate::runtime::agent::AgentMovementSystem::new();
        let mut visited_source = false;
        let mut returned = false;
        for _ in 0..120 {
            engine.resources.current_tick += 1;
            crate::runtime::thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
            mind.try_tick(&mut engine.world, &mut engine.resources)
                .unwrap_or_else(|error| panic!("{error}; thirsty={thirsty} tick={} source={source:?} passable={} state={:?} hunger={:?} thirst={:?}", engine.resources.current_tick, engine.resources.nav.passable(source), engine.world.get::<&sim_core::components::AgentState>(entity).ok(), engine.world.get::<&Hunger>(entity).ok(), engine.world.get::<&Thirst>(entity).ok()));
            movement.tick(&mut engine.world, &mut engine.resources);
            let position = *engine.world.get::<&Position>(entity).expect("position");
            visited_source |= (position.x, position.y) == source;
            let remaining = if thirsty {
                u32::from(
                    engine
                        .resources
                        .water_tiles
                        .get(&source)
                        .copied()
                        .unwrap_or(0),
                )
            } else {
                engine
                    .resources
                    .colony
                    .ground_items
                    .get(&source)
                    .map_or(0, |item| u32::from(item.count))
            };
            if visited_source
                && remaining == 1
                && (position.x, position.y) == home
                && engine.world.get::<&Activity>(entity).is_ok()
            {
                returned = true;
                break;
            }
        }
        assert!(
            visited_source,
            "recovery must release the resident to the real need source"
        );
        assert!(
            returned,
            "one real consumption must complete and the resident must resume recovery"
        );
        if !thirsty {
            assert_eq!(engine.resources.tallies.meals_eaten, 1);
        }
    }
}

#[test]
fn refuge_recovery_missing_need_source_does_not_transfer_to_leisure() {
    let (mut engine, entity, _, _) = fixture();
    engine.resources.water_tiles.clear();
    engine.resources.colony.ground_items.clear();
    engine
        .world
        .get::<&mut BodyHealth>(entity)
        .expect("health")
        .apply_damage(10.0);
    engine
        .world
        .get::<&mut Hunger>(entity)
        .expect("hunger")
        .value = 0.0;
    engine
        .world
        .get::<&mut Thirst>(entity)
        .expect("thirst")
        .value = 0.0;
    engine
        .world
        .insert_one(entity, Cold)
        .expect("admit recovery");
    let mut mind = ColonyMindSystem::new(
        MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
            .expect("policy"),
    );
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .expect("enter");
    engine.world.remove_one::<Cold>(entity).expect("warm room");
    engine
        .world
        .get::<&mut Hunger>(entity)
        .expect("hunger")
        .value = 60.0;
    let before = *engine.world.get::<&BodyHealth>(entity).expect("health");
    for _ in 0..3 {
        engine.resources.current_tick += 1;
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("no foreign leisure lifecycle");
        assert!(
            engine
                .world
                .get::<&sim_core::components::Activity>(entity)
                .is_ok(),
            "wait in the real refuge when the needed food is unavailable"
        );
    }
    assert_eq!(
        *engine.world.get::<&BodyHealth>(entity).expect("health"),
        before
    );
}
