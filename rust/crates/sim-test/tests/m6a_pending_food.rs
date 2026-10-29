//! Pending output promises retain due-day freshness across retries and format14 saves.
use sim_core::colony::food::{DryingBatch, FoodOwner};
use sim_core::colony::{BlueprintKind, ItemKind, ItemSpec, Workshop};
use sim_core::rules::WorldRules;
use sim_core::{MaterialRegistry, TerrainType};
use sim_engine::SimEngine;
use sim_systems::{runtime::food, save};

fn scene(item: &str, due_day: u64) -> SimEngine {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    let res = &mut engine.resources;
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (2, 2),
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    res.food.pending_drying.push(DryingBatch {
        workshop: 1,
        bill: 1,
        tile: (2, 3),
        output: ItemSpec::raw(ItemKind::from_name(item).unwrap(), None),
        count: 3,
        due_tick: due_day * res.ticks_per_day,
    });
    engine
}

#[test]
fn blocked_cooked_output_keeps_due_age_after_format14_roundtrip() {
    let mut engine = scene("roasted_roots", 0);
    engine
        .resources
        .tile_grid
        .terrain_type
        .fill(TerrainType::Mountain);
    food::drying::advance(&mut engine.resources).unwrap();
    assert_eq!(engine.resources.food.pending_drying.len(), 1);
    let bytes = save::save(&mut engine).unwrap();
    assert_eq!(&bytes[8..12], &14_u32.to_le_bytes());
    let mut loaded = save::load_with(&bytes, |_| {}).unwrap();
    assert_eq!(engine.resources.food, loaded.resources.food);
    for state in [&mut engine, &mut loaded] {
        state.resources.current_tick = state.resources.ticks_per_day;
        state
            .resources
            .tile_grid
            .terrain_type
            .fill(TerrainType::Plain);
        food::drying::advance(&mut state.resources).unwrap();
        assert!(state.resources.food.pending_drying.is_empty());
        assert_eq!(
            state.resources.food.ages[&FoodOwner::Ground((2, 3))].born_day,
            0
        );
        assert_eq!(
            food::storage::remaining_days(
                &state.world,
                &state.resources,
                FoodOwner::Ground((2, 3)),
                1
            )
            .unwrap(),
            1
        );
        food::validate(&state.world, &state.resources).unwrap();
    }
    assert_eq!(engine.resources.food, loaded.resources.food);
}

#[test]
fn blocked_completed_food_expires_once_instead_of_emerging_fresh() {
    let mut engine = scene("roasted_roots", 0);
    let kind = engine.resources.food.pending_drying[0].output.kind;
    engine
        .resources
        .tile_grid
        .terrain_type
        .fill(TerrainType::Mountain);
    food::drying::advance(&mut engine.resources).unwrap();
    engine.resources.current_tick = 2 * engine.resources.ticks_per_day;
    food::drying::advance(&mut engine.resources).unwrap();
    food::drying::advance(&mut engine.resources).unwrap();
    assert!(engine.resources.food.pending_drying.is_empty());
    assert!(engine.resources.food.ages.is_empty());
    assert_eq!(engine.resources.food.tallies.craft_output_units, 3);
    assert_eq!(engine.resources.food.tallies.spoiled_units, 3);
    assert_eq!(engine.resources.colony.item_ledger.created_of(kind), 3);
    assert_eq!(engine.resources.colony.item_ledger.consumed_of(kind), 3);
    food::validate(&engine.world, &engine.resources).unwrap();
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources).unwrap();
}

#[test]
fn passive_drying_age_starts_at_due_day_not_start_or_late_retry_day() {
    let mut engine = scene("dried_meat", 2);
    engine.resources.current_tick = engine.resources.ticks_per_day;
    food::drying::advance(&mut engine.resources).unwrap();
    assert_eq!(engine.resources.food.pending_drying.len(), 1);
    engine.resources.current_tick = 3 * engine.resources.ticks_per_day;
    food::drying::advance(&mut engine.resources).unwrap();
    assert_eq!(
        engine.resources.food.ages[&FoodOwner::Ground((2, 3))].born_day,
        2
    );
    food::validate(&engine.world, &engine.resources).unwrap();
}

#[test]
fn removed_workshop_cancels_promise_without_fabricated_output() {
    let mut engine = scene("dried_meat", 2);
    engine.resources.colony.workshops.remove(&1);
    engine.resources.current_tick = 2 * engine.resources.ticks_per_day;
    food::drying::advance(&mut engine.resources).unwrap();
    assert!(engine.resources.food.pending_drying.is_empty());
    assert_eq!(engine.resources.colony.tallies.crafts_lost, 1);
    assert_eq!(engine.resources.food.tallies.craft_output_units, 0);
    assert_eq!(engine.resources.food.tallies.spoiled_units, 0);
    food::validate(&engine.world, &engine.resources).unwrap();
}
