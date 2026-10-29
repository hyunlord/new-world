//! Historical winter-day displays use recorded demand, never today's population.
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::food::WinterPlanEvidence;
use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
use sim_core::components::{Agent, Position};
use sim_core::rules::WorldRules;
use sim_engine::SimEngine;
use sim_systems::{
    runtime::{food, steward},
    save,
};
use std::collections::BTreeSet;

fn scene() -> SimEngine {
    let mut engine = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    engine.resources.rules = WorldRules::colony(0);
    engine.resources.rules.clock_offset_ticks = 0;
    engine.resources.ticks_per_day = engine.resources.rules.calendar.ticks_per_day;
    engine.world.spawn((Agent { id: 1 }, Position::new(2, 2)));
    for _ in 0..18 * engine.resources.ticks_per_day {
        engine.tick();
    }
    engine.resources.current_tick = engine.current_tick();
    engine.resources.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([(2, 2)]),
        },
    );
    food::items::create_ground(
        &mut engine.resources,
        (2, 2),
        ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
        16,
        18,
        food::items::FoodSource::Harvest,
    )
    .unwrap();
    steward::food::observe(&engine.world, &mut engine.resources);
    engine
}

#[test]
fn plan_denominator_survives_population_change_and_format14_roundtrip() {
    let mut engine = scene();
    let (&index, &original) = engine.resources.food.winter_plans.iter().next().unwrap();
    assert_eq!(original.daily_demand_milli, 2400);
    assert_eq!(original.stored_units, 16);
    assert_eq!(original.required_units, 15);
    let days_milli = original.stored_units * 1_000_000 / original.daily_demand_milli;
    engine.world.spawn((Agent { id: 2 }, Position::new(3, 2)));
    let bytes = save::save(&mut engine).unwrap();
    assert_eq!(&bytes[8..12], &14_u32.to_le_bytes());
    let loaded = save::load_with(&bytes, |_| {}).unwrap();
    let historical = loaded.resources.food.winter_plans[&index];
    assert_eq!(historical, original);
    assert_eq!(
        historical.stored_units * 1_000_000 / historical.daily_demand_milli,
        days_milli
    );
    let policy = sim_core::colony::steward::rules()
        .unwrap()
        .rules
        .iter()
        .find_map(|r| match &r.action {
            sim_core::colony::steward::Action::FoodPlan(p) => Some(p),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        steward::food::summary(&loaded.world, &loaded.resources, policy).daily_milli,
        4800
    );
    food::validate(&loaded.world, &loaded.resources).unwrap();
}

#[test]
fn invalid_plan_index_kind_tick_and_quantities_are_rejected() {
    let mut engine = scene();
    let (&index, &original) = engine.resources.food.winter_plans.iter().next().unwrap();
    for bad in [
        WinterPlanEvidence {
            tick: original.tick + 1,
            ..original
        },
        WinterPlanEvidence {
            stored_units: original.stored_units + 1,
            ..original
        },
        WinterPlanEvidence {
            required_units: original.required_units + 1,
            ..original
        },
    ] {
        engine.resources.food.winter_plans.insert(index, bad);
        assert!(food::validate(&engine.world, &engine.resources).is_err());
    }
    engine.resources.food.winter_plans.insert(index, original);
    let wrong = engine.resources.chronicle.push_caused(
        ChronicleEntry::new(original.tick, ChronicleKind::FirstCookedMeal),
        None,
    );
    engine.resources.food.winter_plans.insert(wrong, original);
    assert!(food::validate(&engine.world, &engine.resources).is_err());
    engine.resources.food.winter_plans.remove(&wrong);
    engine
        .resources
        .food
        .winter_plans
        .insert(u32::MAX, original);
    assert!(food::validate(&engine.world, &engine.resources).is_err());
    engine.resources.food.winter_plans.remove(&u32::MAX);
    food::validate(&engine.world, &engine.resources).unwrap();
}
