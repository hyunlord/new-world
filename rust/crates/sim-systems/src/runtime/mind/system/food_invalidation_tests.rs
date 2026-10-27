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
    effects
        .dirty
        .extend([EffectDomain::FoodTile((1, 1)), EffectDomain::FoodSources]);
    invalidate(&mut context, &mut effects, &resources).unwrap();
    let mut tiles = Vec::new();
    context.visit_food_in_square(&resources, (8, 8), 16, |tile, amount| {
        tiles.push((tile, amount))
    });
    assert_eq!(tiles, vec![((8, 8), 4)]);
    assert!(context.derived(7, "food").is_err());
    assert!(effects.dirty.is_empty());
}
