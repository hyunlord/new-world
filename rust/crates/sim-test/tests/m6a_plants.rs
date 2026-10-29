//! Frozen placement, seasonal transitions and atomic harvest ownership.
use sim_core::colony::content::{installed_or_bundled, PlantPhase};
use sim_core::colony::food::PlantState;
use sim_core::colony::{Designation, ReserveKey};
use sim_core::rules::CalendarRule;
use sim_systems::runtime::food::plants;
use sim_systems::runtime::jobs::harvest::{self, HarvestIndex, HarvestQuery};

#[test]
fn plant_windows_follow_authored_phases_and_late_summer_grain() {
    let catalog = installed_or_bundled();
    let def = |name| {
        catalog
            .plant(catalog.plant_by_name(name).expect("species"))
            .expect("definition")
    };
    let calendar = CalendarRule::COLONY;
    for day in 0..48 {
        let year_day = day % 24;
        assert_eq!(
            plants::phase(def("wild_root"), &calendar, day) == PlantPhase::Ripe,
            year_day < 18
        );
        assert_eq!(
            plants::phase(def("berry_bush"), &calendar, day) == PlantPhase::Ripe,
            (6..18).contains(&year_day)
        );
        assert_eq!(
            plants::phase(def("wild_grain"), &calendar, day) == PlantPhase::Ripe,
            (9..12).contains(&year_day)
        );
        assert_eq!(
            plants::phase(def("hazel"), &calendar, day) == PlantPhase::Ripe,
            (12..18).contains(&year_day)
        );
    }
}

#[test]
fn depleted_plants_regrow_once_without_banking_missed_opportunities() {
    let catalog = installed_or_bundled();
    let species = catalog.plant_by_name("wild_root").expect("roots");
    let def = catalog.plant(species).expect("root definition");
    let calendar = CalendarRule::COLONY;
    let mut plant = PlantState {
        species,
        units_left: 0,
        next_regrow_day: 2,
    };
    plants::advance_plant(&mut plant, def, (&calendar, 1, Some(0)));
    assert_eq!(plant.units_left, 0);
    plants::advance_plant(&mut plant, def, (&calendar, 2, Some(1)));
    assert_eq!(plant.units_left, 5);
    plants::advance_plant(&mut plant, def, (&calendar, 16, Some(2)));
    assert_eq!(
        plant.units_left, 5,
        "missed work never accumulates harvests"
    );
    plants::advance_plant(&mut plant, def, (&calendar, 18, Some(16)));
    assert_eq!(plant.units_left, 0);
    plants::advance_plant(&mut plant, def, (&calendar, 24, Some(18)));
    assert_eq!(plant.units_left, 5);

    let hazel = catalog.plant_by_name("hazel").expect("hazel");
    let def = catalog.plant(hazel).expect("hazel definition");
    let mut plant = PlantState {
        species: hazel,
        units_left: 0,
        next_regrow_day: u32::MAX,
    };
    plants::advance_plant(&mut plant, def, (&calendar, 13, Some(12)));
    assert_eq!(plant.units_left, 0, "once per continuous ripe window");
    plants::advance_plant(&mut plant, def, (&calendar, 36, Some(13)));
    assert_eq!(
        plant.units_left, 8,
        "a later year's ripe window resets depletion"
    );
}

#[test]
fn placement_matches_frozen_reachable_radius16_census() {
    let expected = [
        [30, 40, 27, 50, 16],
        [14, 7, 24, 31, 4],
        [38, 19, 26, 28, 10],
        [34, 22, 31, 41, 14],
        [27, 13, 21, 29, 7],
        [13, 3, 8, 13, 2],
        [30, 12, 24, 31, 12],
        [51, 13, 30, 34, 8],
    ];
    let catalog = installed_or_bundled();
    let species = [
        "berry_bush",
        "hazel",
        "wild_grain",
        "wild_root",
        "wild_flax",
    ]
    .map(|name| catalog.plant_by_name(name).expect("species"));
    for (seed, counts) in expected.into_iter().enumerate() {
        let engine = sim_test::colony_engine(seed as u64);
        let res = &engine.resources;
        let camp = sim_systems::worldgen::colony::hearth(res).expect("hearth");
        let region = res.nav.region_of(camp);
        let actual = species.map(|kind| {
            res.food
                .plants
                .iter()
                .filter(|(tile, plant)| {
                    plant.species == kind
                        && sim_core::spatial::chebyshev(**tile, camp) <= 16
                        && res.nav.region_of(**tile) == region
                })
                .count()
        });
        assert_eq!(actual, counts, "seed {seed}");
    }
    assert!(sim_test::baseline_engine(0)
        .resources
        .food
        .plants
        .is_empty());
}

#[test]
fn harvest_respects_reservation_and_creates_each_yield_exactly_once() {
    let mut engine = sim_test::colony_engine(0);
    let res = &mut engine.resources;
    let camp = sim_systems::worldgen::colony::hearth(res).expect("hearth");
    let region = res.nav.region_of(camp);
    let tile = res
        .food
        .plants
        .keys()
        .copied()
        .find(|tile| plants::harvestable(res, *tile) && res.nav.reaches(region, *tile))
        .expect("reachable spring harvest");
    res.colony.designations.insert(tile, Designation::Harvest);
    res.colony.touch(false);
    let mut index = HarvestIndex::default();
    let query = HarvestQuery {
        practice: u32::MAX,
        region,
        pos: tile,
        radius: 0,
    };
    let job = harvest::pick(res, &mut index, query).expect("ripe marked plant");
    assert_eq!(harvest::work_ticks(res, tile, u32::MAX), 40);
    assert_eq!(harvest::work_ticks(res, tile, 0), 60);
    res.colony.reservations.insert(ReserveKey::Plant(tile), 999);
    assert!(harvest::pick(res, &mut index, query).is_none());
    res.colony.reservations.remove(&ReserveKey::Plant(tile));
    let before = res.colony.item_ledger.created_of(job.item);
    assert!(harvest::complete(res, tile));
    assert_eq!(
        res.colony.item_ledger.created_of(job.item) - before,
        u64::from(job.amount)
    );
    assert_eq!(res.food.plants[&tile].units_left, 0);
    assert!(!harvest::complete(res, tile));
    assert!(!res.colony.designations.contains_key(&tile));
    assert_eq!(res.food.last_daily_day, Some(0));
    let previous = res.food.last_daily_day;
    plants::advance_day(res, 1, previous);
    assert_eq!(
        res.food.plants[&tile].units_left, 0,
        "first midnight does not refill early"
    );
    plants::advance_day(res, 2, Some(1));
    assert_eq!(res.food.plants[&tile].units_left, job.amount);
}

#[test]
fn drought_only_delays_depleted_recurring_food_inside_its_dates() {
    use sim_core::colony::story::PressureKind;
    let mut engine = sim_test::colony_engine(0);
    let catalog = installed_or_bundled();
    let root = catalog.plant_by_name("wild_root").expect("roots");
    let berry = catalog.plant_by_name("berry_bush").expect("berries");
    let hazel = catalog.plant_by_name("hazel").expect("hazel");
    engine.resources.current_tick = 6 * 1440;
    sim_systems::runtime::story::try_force(
        &mut engine.world,
        &mut engine.resources,
        PressureKind::Drought,
        5,
    )
    .expect("drought");
    engine.resources.food.plants.clear();
    for (tile, species, units_left, next_regrow_day) in [
        ((0, 0), root, 0, 7),
        ((1, 0), root, 5, u32::MAX),
        ((2, 0), berry, 0, 0),
        ((3, 0), hazel, 0, u32::MAX),
    ] {
        engine.resources.food.plants.insert(
            tile,
            PlantState {
                species,
                units_left,
                next_regrow_day,
            },
        );
    }
    plants::advance_day(&mut engine.resources, 5, Some(4));
    assert_eq!(
        engine.resources.food.plants[&(0, 0)].next_regrow_day,
        7,
        "future drought has no effect"
    );
    plants::advance_day(&mut engine.resources, 6, Some(5));
    assert_eq!(
        engine.resources.food.plants[&(0, 0)].next_regrow_day,
        7,
        "no completed drought interval on start day"
    );
    assert_eq!(
        engine.resources.food.plants[&(1, 0)].units_left,
        5,
        "standing yield is not removed"
    );
    assert_eq!(
        engine.resources.food.plants[&(2, 0)].units_left,
        catalog.plant(berry).unwrap().yield_count,
        "first seasonal yield remains authored"
    );
    plants::advance_day(&mut engine.resources, 11, Some(10));
    assert_eq!(
        engine.resources.food.plants[&(0, 0)].units_left,
        5,
        "the fifth completed interval earns growth"
    );
    let no_regrow = engine.resources.food.plants.get_mut(&(3, 0)).unwrap();
    no_regrow.next_regrow_day = u32::MAX;
    plants::advance_day(&mut engine.resources, 13, Some(12));
    assert_eq!(
        engine.resources.food.plants[&(3, 0)].units_left,
        0,
        "no-regrow sentinel never becomes a deadline"
    );
    assert!(engine.resources.colony.ground_items.is_empty());
    assert_eq!(engine.resources.food.tallies.harvested_units, 0);
}

#[test]
fn drought_delayed_growth_continues_identically_after_save_load() {
    use sim_core::colony::story::PressureKind;
    use sim_systems::save;
    let mut original = sim_test::colony_engine(0);
    let root = installed_or_bundled()
        .plant_by_name("wild_root")
        .expect("roots");
    let tile = original
        .resources
        .food
        .plants
        .iter()
        .find(|(_, p)| p.species == root)
        .map(|(tile, _)| *tile)
        .unwrap();
    let plant = original.resources.food.plants.get_mut(&tile).unwrap();
    plant.units_left = 0;
    plant.next_regrow_day = 2;
    sim_systems::runtime::story::try_force(
        &mut original.world,
        &mut original.resources,
        PressureKind::Drought,
        5,
    )
    .expect("drought");
    for _ in 0..1440 {
        original.tick();
    }
    assert!(original.resources.food.plants[&tile].next_regrow_day > 2);
    let mut loaded = save::load(&save::save(&mut original).expect("save delayed growth"))
        .expect("reload delayed growth");
    for _ in 0..5 * 1440 {
        original.tick();
        loaded.tick();
    }
    assert_eq!(original.resources.food, loaded.resources.food);
    assert_eq!(
        save::save(&mut original).unwrap(),
        save::save(&mut loaded).unwrap()
    );
}

#[test]
fn drought_deadline_saturation_never_creates_the_no_regrowth_sentinel() {
    use sim_core::colony::story::PressureKind;
    let mut engine = sim_test::colony_engine(0);
    let root = installed_or_bundled().plant_by_name("wild_root").unwrap();
    engine
        .resources
        .rules
        .colony
        .as_mut()
        .unwrap()
        .story
        .as_mut()
        .unwrap()
        .drought_regen_permille = 0;
    sim_systems::runtime::story::try_force(
        &mut engine.world,
        &mut engine.resources,
        PressureKind::Drought,
        5,
    )
    .unwrap();
    let pressure = engine
        .resources
        .colony
        .story
        .active
        .iter_mut()
        .find(|p| p.kind == PressureKind::Drought)
        .unwrap();
    pressure.start_day = u32::MAX - 4;
    pressure.end_day = u32::MAX;
    engine.resources.food.plants.clear();
    engine.resources.food.plants.insert(
        (0, 0),
        PlantState {
            species: root,
            units_left: 0,
            next_regrow_day: u32::MAX - 1,
        },
    );
    plants::advance_day(&mut engine.resources, u32::MAX - 2, Some(u32::MAX - 3));
    assert_eq!(
        engine.resources.food.plants[&(0, 0)].next_regrow_day,
        u32::MAX - 1
    );
    plants::advance_day(&mut engine.resources, u32::MAX - 1, Some(u32::MAX - 2));
    assert_eq!(
        engine.resources.food.plants[&(0, 0)].units_left,
        5,
        "finite time cannot turn into permanent depletion"
    );
}

#[test]
fn scheduled_drought_counts_five_completed_days_at_both_clock_offsets() {
    use sim_core::colony::story::{Pressure, PressureKind};
    use sim_core::rules::WorldRules;
    use sim_engine::SimEngine;
    use sim_systems::runtime::{food::FoodDailySystem, story::StorytellerSystem};

    fn through(engine: &mut SimEngine, tick: u64) {
        while engine.current_tick() <= tick {
            engine.tick();
        }
    }
    let start = 30u32;
    let end = start + 5;
    let species = installed_or_bundled().plant_by_name("wild_root").unwrap();
    for offset in [0, 480] {
        let mut engine = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
        engine.resources.rules = WorldRules::colony(0);
        engine.resources.rules.clock_offset_ticks = offset;
        engine.resources.ticks_per_day = engine.resources.rules.calendar.ticks_per_day;
        let ticks = engine.resources.ticks_per_day;
        engine.resources.food.last_daily_day = Some(0);
        engine.resources.colony.story.seasons_planned = u32::MAX;
        engine.resources.colony.story.next_check_day = start;
        engine.resources.colony.story.planned = vec![Pressure {
            kind: PressureKind::Drought,
            start_day: start,
            end_day: end,
            began_entry: None,
            deaths_at_start: 0,
            intensity: 0,
            subject: None,
            cost: Default::default(),
        }];
        engine.register_system(Box::new(FoodDailySystem::new(&engine.resources)));
        engine.register_system(Box::new(StorytellerSystem));
        let boundary = |day: u32| u64::from(day) * ticks - offset;
        // Advance the real scheduler to just before the planned start midnight.
        through(&mut engine, boundary(start) - 1);
        engine.resources.food.plants.insert(
            (0, 0),
            PlantState {
                species,
                units_left: 0,
                next_regrow_day: start + 2,
            },
        );
        assert!(!engine
            .resources
            .colony
            .story
            .is_active(PressureKind::Drought));
        through(&mut engine, boundary(start));
        assert_eq!(
            engine.resources.food.plants[&(0, 0)].next_regrow_day,
            start + 2,
            "planned pressure must not charge an uncompleted interval, offset {offset}"
        );
        through(&mut engine, u64::from(start) * ticks);
        assert!(engine
            .resources
            .colony
            .story
            .is_active(PressureKind::Drought));
        let mut observed = Vec::new();
        for day in start + 1..=end {
            through(&mut engine, boundary(day));
            let plant = engine.resources.food.plants[&(0, 0)];
            observed.push((plant.next_regrow_day, plant.units_left));
        }
        assert_eq!(
            observed,
            vec![(33, 0), (34, 0), (34, 0), (35, 0), (u32::MAX, 5)],
            "400 permille earns exactly two growth days and withholds three, offset {offset}"
        );
        assert_eq!(
            engine
                .resources
                .colony
                .story
                .is_active(PressureKind::Drought),
            offset != 0,
            "midnight precedes Story's ending tick only with the display offset"
        );
        through(&mut engine, u64::from(end) * ticks);
        assert!(!engine
            .resources
            .colony
            .story
            .is_active(PressureKind::Drought));
        assert!(engine
            .resources
            .colony
            .story
            .history
            .iter()
            .any(|p| p.kind == PressureKind::Drought && p.start_day == start && p.end_day == end));
        let plant = engine.resources.food.plants.get_mut(&(0, 0)).unwrap();
        plant.units_left = 0;
        plant.next_regrow_day = end + 2;
        through(&mut engine, boundary(end + 1));
        assert_eq!(
            engine.resources.food.plants[&(0, 0)].next_regrow_day,
            end + 2
        );
        through(&mut engine, boundary(end + 2));
        assert_eq!(
            engine.resources.food.plants[&(0, 0)].units_left,
            5,
            "ended pressure does not leave a growth penalty"
        );
        assert!(engine.resources.colony.ground_items.is_empty());
        assert_eq!(engine.resources.food.tallies.harvested_units, 0);
    }
}
