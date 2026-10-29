//! Baseline isolation and real food-daily continuation across the save boundary.
use sim_core::colony::food::FoodState;
use sim_systems::save;

#[test]
fn baseline_continuation_keeps_food_empty_and_registration_unchanged() {
    let mut uninterrupted = sim_test::baseline_engine(7);
    for _ in 0..12 {
        uninterrupted.tick();
    }
    let original_names: Vec<String> = uninterrupted
        .system_names()
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert!(!original_names.iter().any(|name| name == "FoodDailySystem"));
    let bytes = save::save(&mut uninterrupted).expect("baseline save");
    let mut loaded = save::load(&bytes).expect("baseline reload");
    for _ in 0..12 {
        uninterrupted.tick();
        loaded.tick();
    }
    assert_eq!(loaded.system_names(), original_names);
    assert_eq!(loaded.resources.food, FoodState::default());
    assert_eq!(uninterrupted.resources.food, FoodState::default());
    assert_eq!(
        save::save(&mut loaded).expect("loaded save"),
        save::save(&mut uninterrupted).expect("continuous save")
    );
}

#[test]
fn colony_food_boundary_continues_exactly_after_reload() {
    let mut uninterrupted = sim_test::colony_engine(0);
    let daily = uninterrupted.resources.rules.calendar.ticks_per_day;
    // Cross the first daily update, then compare a second boundary from a save.
    for _ in 0..daily {
        uninterrupted.tick();
    }
    assert!(uninterrupted.resources.food.last_daily_day.is_some());
    assert!(!uninterrupted.resources.food.plants.is_empty());
    let bytes = save::save(&mut uninterrupted).expect("colony food save");
    let mut loaded = save::load(&bytes).expect("colony food reload");
    assert_eq!(uninterrupted.resources.food, loaded.resources.food);
    for _ in 0..daily {
        uninterrupted.tick();
        loaded.tick();
    }
    assert_eq!(
        save::save(&mut loaded).expect("loaded save"),
        save::save(&mut uninterrupted).expect("continuous save")
    );
}

#[test]
fn retained_food_owner_roundtrips_and_continues_including_stale_source() {
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::{ItemKind, ItemSpec};
    use sim_core::components::{Agent, AgentState, Position, TargetKind};
    use sim_systems::runtime::food::items::{self, FoodSource};

    for stale in [false, true] {
        let mut engine = sim_test::colony_engine(0);
        let (entity, id, tile) = engine
            .world
            .query::<(&Agent, &Position)>()
            .iter()
            .map(|(entity, (agent, pos))| (entity, agent.id, (pos.x, pos.y)))
            .min_by_key(|(_, id, _)| *id)
            .unwrap();
        let owner = FoodOwner::Ground(tile);
        if !stale {
            items::create_ground(
                &mut engine.resources,
                tile,
                ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
                1,
                0,
                FoodSource::Harvest,
            )
            .unwrap();
        }
        *engine.world.get::<&mut AgentState>(entity).unwrap() = AgentState::Seeking {
            target: TargetKind::Food,
        };
        engine.resources.food.active_targets.insert(id, owner);
        let bytes =
            save::save(&mut engine).expect("live or normally depleted admitted source saves");
        let mut loaded = save::load(&bytes).unwrap();
        assert_eq!(loaded.resources.food.active_targets.get(&id), Some(&owner));
        assert_eq!(save::save(&mut loaded).unwrap(), bytes);
        for _ in 0..32 {
            engine.tick();
            loaded.tick();
        }
        assert_eq!(
            save::save(&mut loaded).unwrap(),
            save::save(&mut engine).unwrap()
        );
    }
}

#[test]
fn retained_food_owner_rejects_missing_agent_and_out_of_bounds_tile() {
    use sim_core::colony::food::FoodOwner;
    use sim_core::components::Agent;
    let mut engine = sim_test::colony_engine(0);
    engine
        .resources
        .food
        .active_targets
        .insert(u64::MAX, FoodOwner::Ground((0, 0)));
    assert!(save::save(&mut engine).is_err());
    engine.resources.food.active_targets.clear();
    let id = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, agent)| agent.id)
        .min()
        .unwrap();
    let tile = (engine.resources.tile_grid.width, 0);
    engine
        .resources
        .food
        .active_targets
        .insert(id, FoodOwner::Ground(tile));
    assert!(save::save(&mut engine).is_err());
}
