//! C1 natural habitats, physical seed causality and data-only wild crop extension.
use sim_core::colony::content::{installed_or_bundled, Catalog, NamedSource, RawSources};
use sim_core::variety::WorldVariety;

#[test]
fn c1_origin_gating() {
    let catalog = installed_or_bundled();
    let mild_seed = (0..64)
        .find(|seed| WorldVariety::draw(*seed).climate == 0)
        .expect("mild seed");
    let harsh_seed = (0..64)
        .find(|seed| WorldVariety::draw(*seed).climate == 2)
        .expect("harsh seed");
    let mut mild = sim_test::colony_engine(mild_seed);
    let mut harsh = sim_test::colony_engine(harsh_seed);
    assert_eq!(WorldVariety::draw(mild.resources.world_seed).climate, 0);
    assert_eq!(WorldVariety::draw(harsh.resources.world_seed).climate, 2);
    for (source, crop_name, recipe_name) in [
        ("wild_lentil", "lentil", "save_lentil_seed"),
        ("wild_pea", "pea", "save_pea_seed"),
    ] {
        let plant = catalog.plant_by_name(source).expect("wild species");
        assert!(
            mild.resources
                .food
                .plants
                .values()
                .any(|p| p.species == plant),
            "{source} missing in Mild seed {mild_seed}"
        );
        assert!(
            !harsh
                .resources
                .food
                .plants
                .values()
                .any(|p| p.species == plant),
            "{source} appeared in Harsh seed {harsh_seed}"
        );
        let def = catalog
            .crop(catalog.crop_by_name(crop_name).expect("crop"))
            .expect("definition");
        assert_eq!(def.wild_source, Some(plant));
        assert!(!harsh
            .resources
            .colony
            .ground_items
            .values()
            .any(|stack| stack.kind == def.seed_item));
        let recipe = catalog
            .recipe(catalog.recipe_by_name(recipe_name).expect("seed saving"))
            .expect("recipe");
        assert_eq!(recipe.output, (def.seed_item, 2));
        assert_eq!(
            recipe.inputs,
            vec![(catalog.plant(plant).expect("plant").yield_item, 2)]
        );
        for owner in harsh.resources.settlements.keys() {
            assert_eq!(
                sim_systems::runtime::cultivation::ground_seed_count(
                    &harsh.resources,
                    *owner,
                    catalog.crop_by_name(crop_name).expect("crop")
                ),
                0
            );
        }
    }
    for engine in [&mut mild, &mut harsh] {
        for _ in 0..12 {
            engine.tick();
        }
        assert!(!engine.resources.colony.wildlife.animals.is_empty());
        for animal in engine.resources.colony.wildlife.animals.values() {
            let index = engine.resources.tile_grid.idx(animal.pos.0, animal.pos.1);
            assert!(animal.kind.definition().origin.admits(
                WorldVariety::draw(engine.resources.world_seed).climate,
                engine.resources.tile_grid.terrain_type[index],
                engine.resources.tile_grid.elevation[index]
            ));
        }
    }
    let boar = catalog.animal_by_name("wild_boar").expect("boar");
    let reindeer = catalog.animal_by_name("reindeer").expect("reindeer");
    assert!(!mild
        .resources
        .colony
        .wildlife
        .animals
        .values()
        .any(|a| a.kind == boar));
    assert!(!mild
        .resources
        .colony
        .wildlife
        .animals
        .values()
        .any(|a| a.kind == reindeer));
    assert!(harsh
        .resources
        .colony
        .wildlife
        .animals
        .values()
        .any(|a| a.kind == reindeer));
    assert!(!harsh
        .resources
        .colony
        .wildlife
        .animals
        .values()
        .any(|a| a.kind == boar));
    for engine in [&mild, &harsh] {
        for (&tile, plant) in &engine.resources.food.plants {
            let i = engine.resources.tile_grid.idx(tile.0, tile.1);
            assert!(catalog.plant(plant.species).expect("plant").origin.admits(
                WorldVariety::draw(engine.resources.world_seed).climate,
                engine.resources.tile_grid.terrain_type[i],
                engine.resources.tile_grid.elevation[i]
            ));
        }
    }
}

#[test]
fn c1_data_only_crop_with_wild_source() {
    let extension = RawSources {
        sources: [
            ("plants.ron", include_str!("../data/ext_pack/plants.ron")),
            ("crops.ron", include_str!("../data/ext_pack/crops.ron")),
        ]
        .into_iter()
        .map(|(name, data)| NamedSource {
            name: name.into(),
            bytes: data.as_bytes().to_vec(),
        })
        .collect(),
    };
    let catalog = Catalog::from_sources(RawSources::bundled(), Some(extension.clone()))
        .expect("data-only source and crop");
    let wild = catalog
        .plant_by_name("test_wild_pulse")
        .expect("new wild identity");
    let crop = catalog
        .crop(
            catalog
                .crop_by_name("test_pulse")
                .expect("new crop identity"),
        )
        .expect("crop");
    assert_eq!(crop.wild_source, Some(wild));
    let mut engine = sim_test::colony_engine(0);
    engine.resources.food.plants.clear();
    engine.resources.world_variety = Some(WorldVariety {
        climate: 0,
        ..WorldVariety::draw(0)
    });
    sim_systems::worldgen::plants::populate_with_catalog(&mut engine.resources, &catalog)
        .expect("selected population");
    assert!(engine
        .resources
        .food
        .plants
        .values()
        .any(|plant| plant.species == wild));
    let mut broken = extension;
    broken.sources[1].bytes = String::from_utf8(broken.sources[1].bytes.clone())
        .expect("utf8")
        .replace("Some(\"test_wild_pulse\")", "Some(\"missing_source\")")
        .into_bytes();
    assert!(Catalog::from_sources(RawSources::bundled(), Some(broken)).is_err());
}

/// Arrival and worker travel are fixtures; trade, till, seed pickup and sow use
/// production commands/execution. Knowledge alone must never create physical seed.
#[test]
fn c1_origin_gating_trade_is_a_physical_seed_cause() {
    use sim_core::colony::cultivation::FieldPhase;
    use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
    use sim_core::colony::neighbours::{Caravan, Goods, VisitState};
    use sim_core::colony::{Command, ItemKind, ItemSpec, Job, JobKind, JobStage};
    use sim_core::components::{AgentState, Position, SeekTarget, TargetKind};
    use sim_engine::RuntimeSystem;
    let seed = (0..64)
        .find(|s| WorldVariety::draw(*s).climate == 2)
        .expect("Harsh");
    let mut engine = sim_test::colony_engine(seed);
    for _ in 0..12 {
        engine.tick();
    }
    let catalog = installed_or_bundled();
    let crop = catalog.crop_by_name("lentil").expect("lentil");
    let def = catalog.crop(crop).expect("crop");
    let home = engine
        .resources
        .settlements
        .values()
        .next()
        .expect("home")
        .clone();
    let worker = sim_systems::runtime::knowledge::people(&engine.world, &engine.resources)
        .into_iter()
        .filter(|p| p.adult && home.member_agents.contains(&p.id))
        .min_by_key(|p| (std::cmp::Reverse(p.ns), p.id))
        .expect("curious adult");
    let (id, entity) = (worker.id, worker.entity);
    let lentil = knowledge::id("lentil").expect("lentil knowledge");
    let entries = vec![KnowledgeEntry::new(
        knowledge::id("cultivation").expect("cultivation prerequisite"),
        Source::Seeded,
        1000,
        0,
    )];
    engine
        .world
        .insert_one(entity, Knowledge::from_entries(entries))
        .expect("fixture knowledge");
    assert!(!engine
        .world
        .get::<&Knowledge>(entity)
        .expect("knowledge")
        .practises(lentil));
    assert!(
        !sim_systems::runtime::neighbours::trade::clues(&engine.world, &engine.resources)
            .contains_key(&lentil)
    );
    let mut tiles = Vec::new();
    for y in 0..engine.resources.tile_grid.height {
        for x in 0..engine.resources.tile_grid.width {
            if sim_systems::runtime::cultivation::commands::zoneable(&engine.resources, (x, y))
                && !engine.resources.colony.ground_items.contains_key(&(x, y))
            {
                tiles.push((x, y));
            }
        }
    }
    tiles.sort_by_key(|t| {
        (
            t.0.abs_diff(home.formation_tile.0)
                .max(t.1.abs_diff(home.formation_tile.1)),
            t.1,
            t.0,
        )
    });
    let tile = tiles[0];
    let source = tiles[1];
    let pay = tiles[2];
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::ZoneField {
                min: tile,
                max: tile,
                crop: catalog
                    .crop_by_name("emmer")
                    .expect("existing cultivation crop")
            }
        )
        .accepted
    );
    // Stay within the authored spring sow window while stepping only job execution.
    engine.resources.rules.clock_offset_ticks = 0;
    let mut executor = sim_systems::runtime::jobs::JobExecuteSystem::new();
    let run = |engine: &mut sim_engine::SimEngine,
               executor: &mut sim_systems::runtime::jobs::JobExecuteSystem,
               kind: JobKind,
               stage: JobStage,
               at: (u32, u32),
               ticks: u32| {
        sim_systems::runtime::jobs::cancel::cancel(
            &mut engine.world,
            &mut engine.resources,
            entity,
            sim_core::colony::CancelReason::TargetGone,
        );
        let job = Job {
            kind,
            stage,
            taken_at: engine.resources.current_tick,
            amount: def.seeds_per_tile,
            item: def.seed_item,
            material: None,
            fetch: None,
            best_distance: 0,
            progress_tick: engine.resources.current_tick,
        };
        for key in job.keys() {
            engine.resources.colony.reservations.insert(key, id);
        }
        engine
            .world
            .insert(
                entity,
                (
                    job,
                    Position::new(at.0, at.1),
                    AgentState::Seeking {
                        target: TargetKind::Job,
                    },
                    SeekTarget::new(at),
                ),
            )
            .expect("job fixture arrival");
        for _ in 0..ticks {
            engine.resources.current_tick += 1;
            executor.tick(&mut engine.world, &mut engine.resources);
        }
    };
    run(
        &mut engine,
        &mut executor,
        JobKind::TillField { tile },
        JobStage::ToDest,
        tile,
        def.till_ticks + 2,
    );
    assert_eq!(
        engine.resources.cultivation.fields[&tile].phase,
        FieldPhase::Tilled
    );
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::ZoneField {
                min: tile,
                max: tile,
                crop
            }
        )
        .accepted
    );
    run(
        &mut engine,
        &mut executor,
        JobKind::SowField { tile, from: source },
        JobStage::ToSource,
        source,
        def.sow_ticks + 2,
    );
    assert_eq!(
        engine.resources.cultivation.fields[&tile].phase,
        FieldPhase::Tilled
    );
    assert!(
        engine.world.get::<&Job>(entity).is_err(),
        "seedless job cancelled"
    );
    assert_eq!(
        engine
            .resources
            .cultivation
            .counters
            .seed_units_sown
            .get(&crop)
            .copied()
            .unwrap_or(0),
        0
    );
    // The external caravan is a fixture. Incoming seeds are created only by Trade.
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::AddStockpile {
                x0: pay.0,
                y0: pay.1,
                x1: pay.0,
                y1: pay.1
            }
        )
        .accepted
    );
    sim_systems::runtime::food::items::create_ground(
        &mut engine.resources,
        pay,
        ItemSpec::raw(ItemKind::FlintBlade, None),
        20,
        0,
        sim_systems::runtime::food::items::FoodSource::Debug,
    )
    .expect("barter stock");
    engine.resources.colony.neighbours.caravans.push(Caravan {
        id: 999,
        neighbour: 0,
        goods: vec![Goods {
            kind: def.seed_item,
            count: def.seeds_per_tile,
            quality: 50,
        }],
        pos: source,
        path: vec![],
        state: VisitState::Waiting { since: 0 },
        day: 0,
        spot: Some(source),
        owed: vec![],
        trades: 0,
    });
    let trade = sim_systems::commands::apply_command(
        &mut engine.world,
        &mut engine.resources,
        Command::Trade {
            caravan: 999,
            give: vec![(pay.0, pay.1, 20)],
            take: vec![(0, def.seeds_per_tile)],
        },
    );
    assert!(trade.accepted, "{trade:?}");
    assert_eq!(
        engine
            .resources
            .colony
            .item_ledger
            .traded_in_of(def.seed_item),
        u64::from(def.seeds_per_tile)
    );
    assert_eq!(
        sim_systems::runtime::neighbours::trade::clues(&engine.world, &engine.resources)
            .get(&lentil),
        Some(&def.seed_item)
    );
    let mut discovery = sim_systems::runtime::knowledge::discover::DiscoverySystem;
    let deadline = engine.resources.current_tick + 48 * engine.resources.ticks_per_day;
    while !engine
        .world
        .get::<&Knowledge>(entity)
        .expect("knowledge")
        .practises(lentil)
        && engine.resources.current_tick < deadline
    {
        engine.resources.current_tick += discovery.tick_interval();
        discovery.tick(&mut engine.world, &mut engine.resources);
    }
    let learned = engine.world.get::<&Knowledge>(entity).expect("knowledge");
    assert!(
        learned.practises(lentil),
        "imported seed must permit bounded discovery"
    );
    assert_eq!(
        learned
            .entries()
            .iter()
            .find(|e| e.id() == lentil)
            .expect("learned entry")
            .source(),
        Source::Discovered
    );
    drop(learned);
    assert!(engine.resources.chronicle.entries().iter().any(|e| e.kind
        == sim_core::chronicle::ChronicleKind::Eureka
        && e.subject == Some(id)
        && e.value == i64::from(lentil)
        && e.cause == def.seed_item.code()));
    // Durable traded seed can wait for the next authored spring; no odds changed.
    for _ in 0..96 {
        if sim_systems::runtime::cultivation::sowing_window(&engine.resources, crop) {
            break;
        }
        engine.resources.current_tick += engine.resources.ticks_per_day;
    }
    assert!(sim_systems::runtime::cultivation::sowing_window(
        &engine.resources,
        crop
    ));
    // Learned knowledge still cannot substitute for a physical pickup: an
    // empty source cancels even though the real traded seed lies elsewhere.
    assert!(!engine.resources.colony.ground_items.contains_key(&tile));
    run(
        &mut engine,
        &mut executor,
        JobKind::SowField { tile, from: tile },
        JobStage::ToSource,
        tile,
        1,
    );
    assert_eq!(
        engine.resources.cultivation.fields[&tile].phase,
        FieldPhase::Tilled
    );
    assert!(engine.world.get::<&Job>(entity).is_err());
    run(
        &mut engine,
        &mut executor,
        JobKind::SowField { tile, from: source },
        JobStage::ToSource,
        source,
        1,
    );
    engine
        .world
        .insert_one(entity, Position::new(tile.0, tile.1))
        .expect("field arrival");
    for _ in 0..def.sow_ticks + 2 {
        engine.resources.current_tick += 1;
        executor.tick(&mut engine.world, &mut engine.resources);
    }
    assert!(matches!(
        engine.resources.cultivation.fields[&tile].phase,
        FieldPhase::Sown { .. }
    ));
    assert_eq!(
        engine.resources.cultivation.counters.seed_units_sown[&crop],
        u64::from(def.seeds_per_tile)
    );
}

#[test]
fn c1_highland_origin_has_natural_goats_and_sheep() {
    use sim_core::material::TerrainType;
    let catalog = installed_or_bundled();
    let goat = catalog.animal_by_name("wild_goat").expect("goat");
    let sheep = catalog.animal_by_name("wild_sheep").expect("sheep");
    let mut selected = None;
    for seed in 0..64 {
        let engine = sim_test::colony_engine(seed);
        let res = &engine.resources;
        let camp = sim_systems::worldgen::colony::hearth(res).expect("camp");
        let region = res.nav.region_of(camp);
        let climate = WorldVariety::draw(seed).climate;
        let mut count = 0;
        for y in 0..res.tile_grid.height {
            for x in 0..res.tile_grid.width {
                let tile = (x, y);
                let index = res.tile_grid.idx(x, y);
                if x.abs_diff(camp.0).max(y.abs_diff(camp.1)) <= 40
                    && res.nav.passable(tile)
                    && res.nav.reaches(region, tile)
                    && !res.colony.trees.contains_key(&tile)
                    && res.tile_grid.terrain_type[index] == TerrainType::Plain
                    && res.tile_grid.elevation[index] >= 160
                    && [goat, sheep].iter().all(|kind| {
                        kind.definition().origin.admits(
                            climate,
                            res.tile_grid.terrain_type[index],
                            res.tile_grid.elevation[index],
                        )
                    })
                {
                    count += 1;
                }
            }
        }
        if count >= 20 {
            selected = Some((seed, count, engine));
            break;
        }
    }
    let (seed, count, mut engine) = selected.expect(
        "no seed0..63 has20 reachable open highland tiles within40camp; report, do not tune",
    );
    println!("C1 natural highland seed={seed} admitted_open_tiles_within40={count}");
    for _ in 0..12 {
        engine.tick();
    }
    for kind in [goat, sheep] {
        let animals: Vec<_> = engine
            .resources
            .colony
            .wildlife
            .animals
            .values()
            .filter(|animal| animal.kind == kind)
            .collect();
        assert!(
            !animals.is_empty(),
            "{} absent in natural seed{seed}",
            kind.definition().identity.id
        );
        for animal in animals {
            let index = engine.resources.tile_grid.idx(animal.pos.0, animal.pos.1);
            assert!(kind.definition().origin.admits(
                WorldVariety::draw(seed).climate,
                engine.resources.tile_grid.terrain_type[index],
                engine.resources.tile_grid.elevation[index]
            ));
        }
    }
}

#[test]
fn c1_inert_boar_keeps_habitat_but_never_spawns_naturally() {
    use sim_core::colony::knowledge::ContentStatus;
    use sim_engine::RuntimeSystem;
    let catalog = installed_or_bundled();
    let boar = catalog.animal_by_name("wild_boar").expect("boar retained");
    let def = boar.definition();
    assert_eq!(def.status, ContentStatus::Inert);
    assert!(def.debt.as_deref().is_some_and(|d| d.contains("M-BAL")));
    assert_eq!(
        def.population
            .as_ref()
            .expect("population retained")
            .initial,
        (2, 4)
    );
    let seed = (0..64)
        .find(|seed| WorldVariety::draw(*seed).climate == 0)
        .expect("Mild");
    let mut engine = sim_test::colony_engine(seed);
    assert!(
        (0..engine.resources.tile_grid.height).any(|y| (0..engine.resources.tile_grid.width).any(
            |x| {
                let i = engine.resources.tile_grid.idx(x, y);
                engine.resources.nav.passable((x, y))
                    && def.origin.admits(
                        0,
                        engine.resources.tile_grid.terrain_type[i],
                        engine.resources.tile_grid.elevation[i],
                    )
            }
        )),
        "habitat exists; Inert is what suppresses admission"
    );
    let mut system = sim_systems::runtime::wildlife::AnimalSystem;
    for day in [0, 8, 16] {
        engine.resources.current_tick = day * engine.resources.ticks_per_day;
        system.tick(&mut engine.world, &mut engine.resources);
        assert!(!engine
            .resources
            .colony
            .wildlife
            .animals
            .values()
            .any(|a| a.kind == boar));
    }
    let mut sources = RawSources::bundled();
    let animals = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "animals.ron")
        .expect("animals");
    let text = String::from_utf8(animals.bytes.clone()).expect("utf8");
    animals.bytes=text.replace("debt: Some(\"M-BAL: crash detector seed5 starvation after resident-boar labour disruption; disabled by the one allowed fallback.\")", "debt: None").into_bytes();
    assert!(
        Catalog::from_sources(sources, None).is_err(),
        "Inert requires explicit debt"
    );
}
