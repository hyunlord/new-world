//! Incremental index updates preserve full-rebuild observations and cache invalidation.
use super::*;

fn fixture() -> (TickContext, SimResources) {
    let mut world = World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let mut resources = SimResources::new(32, 32, sim_core::MaterialRegistry::new());
    resources.food_tiles.insert((1, 1), 3);
    resources.food_tiles.insert((2, 2), 1);
    resources.food_tiles.insert((3, 3), 0);
    resources
        .food_tiles
        .insert((17, 17), sim_engine::RESOURCE_SOURCE_INFINITE);
    (TickContext::capture(&world, &resources).unwrap(), resources)
}

fn indexed(context: &TickContext) -> Vec<((u32, u32), u8)> {
    context.food_index.borrow().tiles.iter().copied().collect()
}

#[test]
fn incremental_tiles_match_full_rebuild_for_all_amount_transitions() {
    let (mut context, mut resources) = fixture();
    for (tile, amount) in [
        ((1, 1), Some(2)),
        ((2, 2), None),
        ((4, 4), Some(5)),
        ((3, 3), Some(2)),
        ((3, 3), Some(0)),
        ((3, 3), None),
        ((17, 17), Some(sim_engine::RESOURCE_SOURCE_INFINITE)),
        ((9, 9), None),
    ] {
        match amount {
            Some(value) => {
                resources.food_tiles.insert(tile, value);
            }
            None => {
                resources.food_tiles.remove(&tile);
            }
        }
        let old = context.versions[Domain::FoodSources as usize];
        context
            .invalidate_food_tiles(&resources, [tile, tile])
            .unwrap();
        assert_eq!(context.versions[Domain::FoodSources as usize], old + 1);
        assert_eq!(context.food_index.borrow().version, old + 1);
        let rebuilt = FoodIndex::capture(&resources, old + 1);
        assert_eq!(
            indexed(&context),
            rebuilt.tiles.iter().copied().collect::<Vec<_>>()
        );
    }
}

#[test]
fn multi_tile_update_and_noop_still_invalidate_targets_and_gather_cache() {
    let (mut context, mut resources) = fixture();
    context
        .set_derived(7, "food", 1, &[Domain::FoodSources])
        .unwrap();
    context
        .set_target(7, "food", None, &[Domain::FoodSources])
        .unwrap();
    let old = context.versions[Domain::FoodSources as usize];
    context
        .invalidate_food_tiles(&resources, [(17, 17), (9, 9)])
        .unwrap();
    assert_eq!(context.versions[Domain::FoodSources as usize], old + 1);
    assert!(matches!(
        context.derived(7, "food"),
        Err(ContextError::StaleDerived { .. })
    ));
    assert!(matches!(
        context.target(7, "food"),
        Err(ContextError::StaleDerived { .. })
    ));
    context.set_target_scope(7, TargetScope::default()).unwrap();
    assert!(context.gather_available(7, &resources).unwrap());
    resources.food_tiles.clear();
    context
        .invalidate_food_tiles(&resources, [(1, 1), (2, 2), (3, 3), (17, 17)])
        .unwrap();
    assert!(!context.gather_available(7, &resources).unwrap());
    assert!(indexed(&context).is_empty());
}

#[test]
fn prior_generic_invalidation_cannot_be_hidden_by_known_tile_updates() {
    let (mut context, mut resources) = fixture();
    let original = indexed(&context);
    resources.food_tiles.insert((20, 20), 8);
    context.invalidate(Domain::FoodSources).unwrap();
    resources.food_tiles.remove(&(1, 1));
    context.invalidate_food_tiles(&resources, [(1, 1)]).unwrap();
    assert_eq!(context.food_index.borrow().version, 0);
    assert_eq!(indexed(&context), original);
    let mut visited = Vec::new();
    context.visit_food_in_square(&resources, (16, 16), 32, |tile, amount| {
        visited.push((tile, amount))
    });
    assert!(visited.contains(&((20, 20), 8)));
    assert!(!visited.iter().any(|(tile, _)| *tile == (1, 1)));
    assert_eq!(context.food_index.borrow().version, 2);
}

#[test]
fn version_overflow_does_not_mutate_index_or_advance_version() {
    let (mut context, mut resources) = fixture();
    context.versions[Domain::FoodSources as usize] = u64::MAX;
    context.food_index.get_mut().version = u64::MAX;
    let original = indexed(&context);
    resources.food_tiles.remove(&(1, 1));
    assert_eq!(
        context.invalidate_food_tiles(&resources, [(1, 1)]),
        Err(ContextError::VersionOverflow(Domain::FoodSources))
    );
    assert_eq!(context.versions[Domain::FoodSources as usize], u64::MAX);
    assert_eq!(context.food_index.borrow().version, u64::MAX);
    assert_eq!(indexed(&context), original);
}
