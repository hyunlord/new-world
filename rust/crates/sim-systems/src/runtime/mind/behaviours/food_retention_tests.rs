//! Destination continuity across a detour that reverses geometric nearest ranking.
use super::*;
use crate::runtime::food::items::{self, FoodSource};
use sim_core::colony::{food::FoodOwner, ItemKind, ItemSpec};
use sim_core::rules::WorldRules;

#[test]
fn active_food_owner_survives_nearest_ranking_flip_at_doorway() {
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 1 },
        Position::new(105, 87),
        AgentState::Idle,
        Hunger::new(90.0, 0.05),
        sim_core::components::Thirst::new(90.0, 0.0),
    ));
    let mut resources = SimResources::new(192, 192, MaterialRegistry::new());
    resources.rules = WorldRules::colony(0);
    let mut map = sim_core::nav::CostMap::new(192, 192);
    for x in 99..=105 {
        map.set((x, 85), 0);
        map.set((x, 95), 0);
    }
    for y in 85..=95 {
        map.set((99, y), 0);
        map.set((105, y), 0);
    }
    map.set((105, 87), sim_core::nav::COST_DOOR);
    resources.nav.install(map);
    let inside = (104, 91);
    let outside = (100, 84);
    for tile in [inside, outside] {
        items::create_ground(
            &mut resources,
            tile,
            ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
            2,
            0,
            FoodSource::Harvest,
        )
        .unwrap();
    }
    let distance =
        |from: (u32, u32), to: (u32, u32)| from.0.abs_diff(to.0).max(from.1.abs_diff(to.1));
    // Both stacks have the same item preference; equal distances use stable y/x order.
    let rank = |from, to: (u32, u32)| (distance(from, to), to.1, to.0);
    assert!(rank((105, 87), inside) < rank((105, 87), outside));
    assert_eq!(distance((104, 87), outside), distance((104, 87), inside));
    assert!(rank((104, 87), outside) < rank((104, 87), inside));
    let original = ActionTarget::Food {
        tile: inside,
        owner: FoodOwner::Ground(inside),
    };
    let mut effects = EffectState::default();
    let mut c = context(&mut world, &mut resources, &mut effects, entity, original);
    c.position = Position::new(105, 87);
    assert_eq!(food::start(&mut c), Ok(EffectStatus::Running));
    // The next step toward the original food crosses the doorway's ranking boundary.
    c.position = Position::new(104, 87);
    c.target = ActionTarget::Food {
        tile: outside,
        owner: FoodOwner::Ground(outside),
    };
    assert_eq!(food::advance(&mut c), Ok(EffectStatus::Running));
    assert_eq!(c.effects.seek_set.last(), Some(&(entity, inside)),
        "an active food action must not reverse its destination merely because straight-line ranking changed");
    // Follow actual pathfinder steps while the caller keeps advertising the rival.
    for _ in 0..32 {
        let here = (c.position.x, c.position.y);
        if here == inside {
            break;
        }
        let goal = c.effects.seek_set.last().unwrap().1;
        let path = c.resources.nav.path(here, goal).expect("food path exists");
        let next = *path.first().expect("remaining path");
        assert!(c.resources.nav.can_step(here, next));
        c.position = Position::new(next.0, next.1);
        *c.world.get::<&mut Position>(entity).unwrap() = c.position;
        food::advance(&mut c).unwrap();
    }
    assert_eq!((c.position.x, c.position.y), inside);
    assert_eq!(food::advance(&mut c), Ok(EffectStatus::Finished));
    assert_eq!(c.resources.food.tallies.meals, 1);
    assert!(!c.resources.food.active_targets.contains_key(&1));
    let water = (106, 91);
    c.resources.water_tiles.insert(water, u8::MAX);
    c.target = ActionTarget::Tile(water);
    assert_eq!(needs::drink_start(&mut c), Ok(EffectStatus::Running));
    let path = c
        .resources
        .nav
        .path(inside, water)
        .expect("water through doorway");
    assert!(path.contains(&(105, 87)));
    for &next in path.iter() {
        assert!(c.resources.nav.can_step((c.position.x, c.position.y), next));
        c.position = Position::new(next.0, next.1);
        *c.world.get::<&mut Position>(entity).unwrap() = c.position;
        needs::drink_continue(&mut c).unwrap();
    }
    needs::drink_continue(&mut c).unwrap();
    assert!(
        c.world
            .get::<&sim_core::components::Thirst>(entity)
            .unwrap()
            .value
            < 90.0
    );
}

#[test]
fn active_food_owner_does_not_switch_to_another_owner_on_the_same_tile() {
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 1 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(90.0, 0.05),
    ));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.rules = WorldRules::colony(0);
    resources.nav.install(sim_core::nav::CostMap::new(8, 8));
    let ground = FoodOwner::Ground((2, 2));
    let carried = FoodOwner::Carried(1);
    items::create_ground(
        &mut resources,
        (2, 2),
        ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
        2,
        0,
        FoodSource::Harvest,
    )
    .unwrap();
    let batch = items::take(&mut world, &mut resources, ground, 1, 0).unwrap();
    items::put(&mut world, &mut resources, carried, batch, 0).unwrap();
    let mut effects = EffectState::default();
    let mut c = context(
        &mut world,
        &mut resources,
        &mut effects,
        entity,
        ActionTarget::Food {
            tile: (2, 2),
            owner: ground,
        },
    );
    assert_eq!(food::start(&mut c), Ok(EffectStatus::Running));
    c.target = ActionTarget::Food {
        tile: (2, 2),
        owner: carried,
    };
    assert_eq!(food::advance(&mut c), Ok(EffectStatus::Running));
    assert_eq!(food::advance(&mut c), Ok(EffectStatus::Finished));
    assert!(
        items::stack(c.world, c.resources, ground).is_none(),
        "the admitted owner must supply the completed meal"
    );
    assert_eq!(
        items::stack(c.world, c.resources, carried).unwrap().count,
        1
    );
    assert_eq!(c.resources.food.tallies.meals, 1);
}
