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
