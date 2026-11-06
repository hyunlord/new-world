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
