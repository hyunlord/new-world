//! Full registered engine preserves physical food seeking across legacy stale-target cleanup.
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{ItemKind, ItemSpec, WorkPriorities, WorkType};
use sim_core::components::{
    Agent, AgentState, Hunger, Position, SeekTarget, Sleep, TargetKind, Thirst,
};
use sim_engine::SimEngine;
use sim_systems::runtime::food::{self, items};

fn scene() -> (SimEngine, hecs::Entity, u64, (u32, u32)) {
    let mut engine = sim_test::colony_engine(0);
    let mut agents: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, a)| (a.id, e))
        .collect();
    agents.sort_by_key(|row| row.0);
    let (id, person) = agents[0];
    for (_, entity) in agents {
        engine
            .world
            .insert_one(
                entity,
                WorkPriorities {
                    levels: [0; WorkType::ALL.len()],
                },
            )
            .unwrap();
        if let Ok(mut hunger) = engine.world.get::<&mut Hunger>(entity) {
            hunger.value = 0.0;
        }
    }
    engine.world.get::<&mut Hunger>(person).unwrap().value = 90.0;
    engine.world.get::<&mut Thirst>(person).unwrap().value = 0.0;
    engine.world.get::<&mut Sleep>(person).unwrap().fatigue = 0.0;
    let pos = *engine.world.get::<&Position>(person).unwrap();
    let source = (0..engine.resources.tile_grid.height)
        .flat_map(|y| (0..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .filter(|tile| pos.x.abs_diff(tile.0).max(pos.y.abs_diff(tile.1)) == 4)
        .filter(|tile| engine.resources.nav.reachable((pos.x, pos.y), *tile))
        .find(|tile| {
            sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *tile)
                && !engine.resources.colony.ground_items.contains_key(tile)
        })
        .unwrap();
    let kind = ItemKind::from_name("porridge").unwrap();
    items::create_ground(
        &mut engine.resources,
        source,
        ItemSpec::raw(kind, None),
        3,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    // Supported custom finite-water scene activates legacy stale cleanup.
    // Canonical colony water and beds do not register finite ceilings.
    let water = *engine
        .resources
        .water_tiles
        .keys()
        .min()
        .expect("water source");
    engine.resources.set_water_tile(water.0, water.1, 10);
    engine.resources.set_water_source_max(water.0, water.1, 10);
    assert!(engine.resources.food_tiles.is_empty());
    assert!(
        !engine.resources.water_source_max.is_empty()
            || !engine.resources.sleep_source_max.is_empty()
    );
    (engine, person, id, source)
}

#[test]
fn full_engine_food_seek_survives_cleanup_and_walks_to_real_meal() {
    let (mut engine, person, id, source) = scene();
    engine.tick();
    assert_eq!(
        *engine.world.get::<&AgentState>(person).unwrap(),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );
    assert_eq!(
        engine.world.get::<&SeekTarget>(person).unwrap().tile,
        source
    );
    assert!(!engine.resources.food.meals.contains_key(&id));
    for _ in 0..180 {
        engine.tick();
        if engine
            .resources
            .food
            .meals
            .get(&id)
            .is_some_and(|h| !h.last_meals.is_empty())
        {
            break;
        }
    }
    let meal = &engine.resources.food.meals[&id].last_meals[0];
    assert_eq!(meal.source_owner, FoodOwner::Ground(source));
    assert_eq!(meal.item, ItemKind::from_name("porridge").unwrap());
    let pos = engine.world.get::<&Position>(person).unwrap();
    assert_eq!((pos.x, pos.y), source);
    food::validate(&engine.world, &engine.resources).unwrap();
}

#[test]
fn depleted_physical_target_is_reconsidered_by_mind_after_a_real_transfer() {
    let (mut engine, person, id, source) = scene();
    engine.tick();
    assert_eq!(
        engine.world.get::<&SeekTarget>(person).unwrap().tile,
        source
    );
    let target = (0..engine.resources.tile_grid.height)
        .flat_map(|y| (0..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .find(|tile| {
            *tile != source
                && engine.resources.nav.reachable(source, *tile)
                && sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *tile)
                && !engine.resources.colony.ground_items.contains_key(tile)
        })
        .unwrap();
    let day = food::day(&engine.resources);
    let batch = items::take(
        &mut engine.world,
        &mut engine.resources,
        FoodOwner::Ground(source),
        3,
        day,
    )
    .unwrap();
    items::put(
        &mut engine.world,
        &mut engine.resources,
        FoodOwner::Ground(target),
        batch,
        day,
    )
    .unwrap();
    assert!(!engine.resources.colony.ground_items.contains_key(&source));
    for _ in 0..4 {
        engine.tick();
    }
    assert!(engine
        .world
        .get::<&SeekTarget>(person)
        .ok()
        .is_none_or(|seek| seek.tile != source));
    assert!(engine.resources.food.meals.get(&id).is_none_or(|h| h
        .last_meals
        .iter()
        .all(|m| m.source_owner != FoodOwner::Ground(source))));
    food::validate(&engine.world, &engine.resources).unwrap();
}

#[test]
fn legacy_food_and_colony_water_still_use_their_live_counter_cleanup() {
    use sim_engine::RuntimeSystem;
    for colony in [false, true] {
        let mut engine = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
        if colony {
            engine.resources.rules = sim_core::rules::WorldRules::colony(0);
        }
        let target = if colony {
            TargetKind::Water
        } else {
            TargetKind::Food
        };
        let entity = engine.world.spawn((
            Position::new(2, 2),
            AgentState::Seeking { target },
            SeekTarget::new((3, 3)),
        ));
        engine.resources.water_source_max.insert((7, 7), 1);
        sim_systems::runtime::resource_regen::StaleSeekTargetSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Idle
        );
        assert!(engine.world.get::<&SeekTarget>(entity).is_err());
    }
}
