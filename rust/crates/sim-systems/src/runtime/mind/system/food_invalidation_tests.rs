//! Coordinator coalesces known keys and lets unknown food mutations win.
use super::*;

#[test]
fn generic_food_dirty_wins_over_partial_tile_notifications() {
    let mut world = World::new();
    world.spawn((
        sim_core::components::Agent { id: 7 },
        sim_core::components::Position::new(1, 1),
        AgentState::Idle,
    ));
    let mut resources = SimResources::new(16, 16, sim_core::MaterialRegistry::new());
    resources.food_tiles.insert((1, 1), 1);
    let mut context = TickContext::capture(&world, &resources).unwrap();
    context
        .set_derived(7, "food", 1, &[Domain::FoodSources])
        .unwrap();
    resources.food_tiles.remove(&(1, 1));
    resources.food_tiles.insert((8, 8), 4);
    let mut effects = EffectState::default();
    effects.mark_dirty(EffectDomain::FoodTile((1, 1)));
    effects.mark_dirty(EffectDomain::FoodSources);
    invalidate(&mut context, &mut effects, &resources).unwrap();
    let mut tiles = Vec::new();
    context.visit_food_in_square(&resources, (8, 8), 16, |tile, amount| {
        tiles.push((tile, amount))
    });
    assert_eq!(tiles, vec![((8, 8), 4)]);
    assert!(context.derived(7, "food").is_err());
    assert!(effects.dirty.is_empty());
}

#[test]
fn dirty_marks_match_sorted_set_and_reuse_storage_after_invalidation() {
    let world = World::new();
    let resources = SimResources::new(16, 16, sim_core::MaterialRegistry::new());
    let mut context = TickContext::capture(&world, &resources).unwrap();
    let mut effects = EffectState::default();
    let marks = [
        EffectDomain::Targets,
        EffectDomain::FoodTile((8, 2)),
        EffectDomain::Needs,
        EffectDomain::Claims,
        EffectDomain::FoodTile((1, 9)),
        EffectDomain::Relations,
        EffectDomain::Invitations,
        EffectDomain::RoutineOccupancy,
        EffectDomain::Jobs,
        EffectDomain::FoodSources,
        EffectDomain::FoodTile((8, 2)),
        EffectDomain::Needs,
    ];
    let mut oracle = std::collections::BTreeSet::new();
    for domain in marks {
        effects.mark_dirty(domain);
        oracle.insert(domain);
        assert_eq!(effects.dirty, oracle.iter().copied().collect::<Vec<_>>());
    }
    let allocation = effects.dirty.as_ptr();
    let capacity = effects.dirty.capacity();
    for _ in 0..3 {
        invalidate(&mut context, &mut effects, &resources).unwrap();
        assert!(effects.dirty.is_empty());
        assert_eq!(effects.dirty.as_ptr(), allocation);
        assert_eq!(effects.dirty.capacity(), capacity);
        for domain in marks {
            effects.mark_dirty(domain);
        }
        assert_eq!(effects.dirty, oracle.iter().copied().collect::<Vec<_>>());
    }
}
