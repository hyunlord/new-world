//! Food milestones retain checkable boundary observations and never invent past causes.
use hecs::World;
use sim_core::causal::event::DeathReason;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::food::PlantState;
use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
use sim_core::components::{Agent, Hunger, Position, Thirst};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;
use sim_systems::runtime::{food, steward, survival};
use std::collections::BTreeSet;

fn scene() -> (World, SimResources, hecs::Entity) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        Hunger::new(100.0, 0.05),
        Thirst::new(12.0, 0.05),
    ));
    (world, res, entity)
}

#[test]
fn hunger_death_links_sample_stores_and_observed_harvest_reason() {
    let (mut world, mut res, entity) = scene();
    survival::despawn_agent(
        &mut world,
        &mut res,
        entity,
        7,
        (2, 2),
        DeathReason::Starvation,
        0,
    );
    let (death_index, death) = res
        .chronicle
        .entries()
        .iter()
        .enumerate()
        .find(|(_, e)| e.kind == ChronicleKind::Died)
        .unwrap();
    assert_eq!(death.cause, 0); // Published DeathReason::Starvation code.
    let CauseRef::Entry(chain) = res.chronicle.cause_of(death_index as u32).unwrap() else {
        panic!("death chain");
    };
    let CauseRef::Sample { sample, .. } = res.chronicle.cause_of(chain).unwrap() else {
        panic!("actual needs");
    };
    let sample = res.chronicle.sample(sample).unwrap();
    assert_eq!(
        (sample.agent, sample.tick, sample.hunger, sample.thirst),
        (7, 0, 1000, 120)
    );
    let stores = sample.entry.unwrap();
    assert_eq!(res.chronicle.entries()[stores as usize].value, 0);
    let CauseRef::Entry(harvest) = res.chronicle.cause_of(stores).unwrap() else {
        panic!("harvest observation");
    };
    let CauseRef::Entry(reason) = res.chronicle.cause_of(harvest).unwrap() else {
        panic!("reason observation");
    };
    assert_eq!(res.chronicle.entries()[reason as usize].cause, 3);
}

#[test]
fn ripe_opportunity_does_not_invent_a_worker_or_historical_failure() {
    let (world, mut res, entity) = scene();
    let species = sim_core::colony::content::installed_or_bundled()
        .plant_by_name("wild_root")
        .unwrap();
    res.food.plants.insert(
        (3, 2),
        PlantState {
            species,
            units_left: 2,
            next_regrow_day: u32::MAX,
        },
    );
    food::events::hunger_death(&world, &mut res, entity, 7, (2, 2), 0).unwrap();
    assert_eq!(res.chronicle.entries()[0].cause, 6);
    assert_eq!(res.chronicle.entries()[1].value, 2);
    world.get::<&mut Hunger>(entity).unwrap().value = 30.0;
    let count = res.chronicle.len();
    assert!(food::events::hunger_death(&world, &mut res, entity, 7, (2, 2), 0).is_none());
    assert_eq!(res.chronicle.len(), count);
}

#[test]
fn non_hunger_death_does_not_add_food_explanation() {
    let (mut world, mut res, entity) = scene();
    survival::despawn_agent(
        &mut world,
        &mut res,
        entity,
        7,
        (2, 2),
        DeathReason::Combat,
        0,
    );
    assert!(res
        .chronicle
        .entries()
        .iter()
        .all(|e| e.kind != ChronicleKind::HungerFoodChain));
}

#[test]
fn first_survived_winter_links_real_start_stores_even_with_steward_disabled() {
    let (world, mut res, _) = scene();
    res.steward.enabled = false;
    res.current_tick = 18 * res.ticks_per_day;
    let tile = (2, 2);
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([tile]),
        },
    );
    food::items::create_ground(
        &mut res,
        tile,
        ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
        16,
        18,
        food::items::FoodSource::Harvest,
    )
    .unwrap();
    steward::food::observe(&world, &mut res);
    let start = res
        .chronicle
        .entries()
        .iter()
        .position(|e| e.kind == ChronicleKind::WinterFoodPlan)
        .unwrap();
    assert_eq!(res.chronicle.entries()[start].value, 16);
    res.current_tick = 24 * res.ticks_per_day;
    steward::food::observe(&world, &mut res);
    let end = res
        .chronicle
        .entries()
        .iter()
        .position(|e| e.kind == ChronicleKind::WinterStoredSurvived)
        .unwrap();
    assert_eq!(
        res.chronicle.cause_of(end as u32),
        Some(CauseRef::Entry(start as u32))
    );
    steward::food::observe(&world, &mut res);
    assert_eq!(
        res.chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == ChronicleKind::WinterStoredSurvived)
            .count(),
        1
    );
}

#[test]
fn spring_food_without_a_qualified_winter_start_is_not_a_survival_milestone() {
    let (world, mut res, _) = scene();
    res.current_tick = 24 * res.ticks_per_day;
    res.food.firsts.winter_started_with_stores.insert(0);
    res.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([(2, 2)]),
        },
    );
    food::items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None),
        16,
        24,
        food::items::FoodSource::Harvest,
    )
    .unwrap();
    steward::food::observe(&world, &mut res);
    assert!(res
        .chronicle
        .entries()
        .iter()
        .all(|e| e.kind != ChronicleKind::WinterStoredSurvived));
}
