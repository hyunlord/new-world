//! Lazy construction preserves eager observations at every invalidation boundary.
use super::*;
use std::cell::Cell;

fn fixture() -> (TickContext, TickContext, SimResources) {
    let mut world = World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let mut resources = SimResources::new(32, 32, sim_core::MaterialRegistry::new());
    for (tile, amount) in [((1, 1), 3), ((2, 2), 0), ((17, 17), 255)] {
        resources.food_tiles.insert(tile, amount);
    }
    let lazy = TickContext::capture(&world, &resources).unwrap();
    let mut eager = TickContext::capture(&world, &resources).unwrap();
    *eager.food_index.get_mut() = FoodIndex::capture(&resources, 0);
    (lazy, eager, resources)
}

fn observations(context: &TickContext, resources: &SimResources) -> Vec<((u32, u32), u8)> {
    let mut result = Vec::new();
    context.visit_food_in_square(resources, (16, 16), 32, |tile, amount| {
        result.push((tile, amount));
    });
    result
}

#[test]
fn capture_and_partial_notifications_defer_materialization_until_query() {
    let (mut lazy, mut eager, mut resources) = fixture();
    assert!(!lazy.food_index.borrow().materialized);
    for (tile, amount) in [((1, 1), Some(0)), ((2, 2), None), ((4, 4), Some(9))] {
        match amount {
            Some(amount) => {
                resources.food_tiles.insert(tile, amount);
            }
            None => {
                resources.food_tiles.remove(&tile);
            }
        }
        for context in [&mut lazy, &mut eager] {
            context
                .invalidate_food_tiles(&resources, [tile, tile])
                .unwrap();
        }
        assert!(!lazy.food_index.borrow().materialized);
        assert!(lazy.food_index.borrow().tiles.iter().next().is_none());
        assert_eq!(
            lazy.food_index.borrow().version,
            eager.food_index.borrow().version
        );
    }
    assert_eq!(
        observations(&lazy, &resources),
        observations(&eager, &resources)
    );
    assert!(lazy.food_index.borrow().materialized);
    // Once materialized, existing incremental mutation and iteration order apply.
    resources.food_tiles.insert((4, 4), 0);
    resources.food_tiles.insert((3, 3), 8);
    for context in [&mut lazy, &mut eager] {
        context
            .invalidate_food_tiles(&resources, [(4, 4), (3, 3)])
            .unwrap();
    }
    assert_eq!(
        observations(&lazy, &resources),
        observations(&eager, &resources)
    );
}

#[test]
fn stale_partial_notification_cannot_materialize_or_hide_unknown_changes() {
    let (mut lazy, mut eager, mut resources) = fixture();
    resources.food_tiles.insert((20, 20), 7);
    for context in [&mut lazy, &mut eager] {
        context.invalidate(Domain::FoodSources).unwrap();
    }
    resources.food_tiles.remove(&(1, 1));
    for context in [&mut lazy, &mut eager] {
        let notifications = Cell::new(0);
        context
            .invalidate_food_tiles(
                &resources,
                [(1, 1)].into_iter().inspect(|_| {
                    notifications.set(notifications.get() + 1);
                }),
            )
            .unwrap();
        assert_eq!(notifications.get(), 0);
        assert_eq!(context.food_index.borrow().version, 0);
    }
    assert!(!lazy.food_index.borrow().materialized);
    assert_eq!(
        observations(&lazy, &resources),
        observations(&eager, &resources)
    );
    assert_eq!(lazy.food_index.borrow().version, 2);
}

#[test]
fn fresh_notifications_preserve_iterator_consumption_without_materializing() {
    let (mut lazy, mut eager, resources) = fixture();
    for context in [&mut lazy, &mut eager] {
        let notifications = Cell::new(0);
        context
            .invalidate_food_tiles(
                &resources,
                [(1, 1), (1, 1)].into_iter().inspect(|_| {
                    notifications.set(notifications.get() + 1);
                }),
            )
            .unwrap();
        assert_eq!(notifications.get(), 2);
    }
    assert!(!lazy.food_index.borrow().materialized);
}

#[test]
fn overflow_preserves_unmaterialized_index_and_does_not_consume_notifications() {
    let (mut lazy, mut eager, resources) = fixture();
    for context in [&mut lazy, &mut eager] {
        context.versions[Domain::FoodSources as usize] = u64::MAX;
        context.food_index.get_mut().version = u64::MAX;
        let before = context
            .food_index
            .borrow()
            .tiles
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let notifications = Cell::new(0);
        assert_eq!(
            context.invalidate_food_tiles(
                &resources,
                [(1, 1)].into_iter().inspect(|_| {
                    notifications.set(notifications.get() + 1);
                })
            ),
            Err(ContextError::VersionOverflow(Domain::FoodSources))
        );
        assert_eq!(notifications.get(), 0);
        assert_eq!(context.versions[Domain::FoodSources as usize], u64::MAX);
        assert_eq!(context.food_index.borrow().version, u64::MAX);
        assert_eq!(
            context
                .food_index
                .borrow()
                .tiles
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            before
        );
    }
    assert!(!lazy.food_index.borrow().materialized);
    assert_eq!(
        observations(&lazy, &resources),
        observations(&eager, &resources)
    );
}

#[test]
fn rejected_queries_preserve_error_order_without_materializing() {
    let (mut lazy, mut eager, resources) = fixture();
    lazy.scopes[0] = None;
    eager.scopes[0] = None;
    assert_eq!(
        lazy.gather_candidate(99, &resources),
        eager.gather_candidate(99, &resources)
    );
    assert_eq!(
        lazy.gather_candidate(7, &resources),
        eager.gather_candidate(7, &resources)
    );
    assert!(!lazy.food_index.borrow().materialized);
    for context in [&mut lazy, &mut eager] {
        context.set_target_scope(7, TargetScope::default()).unwrap();
        context
            .set_derived(7, "food", 1, &[Domain::FoodSources])
            .unwrap();
        context
            .set_target(7, "food", None, &[Domain::FoodSources])
            .unwrap();
        context.invalidate_food_tiles(&resources, []).unwrap();
    }
    assert_eq!(lazy.derived(7, "food"), eager.derived(7, "food"));
    assert_eq!(lazy.target(7, "food"), eager.target(7, "food"));
    assert_eq!(
        lazy.gather_candidate(7, &resources),
        eager.gather_candidate(7, &resources)
    );
}
