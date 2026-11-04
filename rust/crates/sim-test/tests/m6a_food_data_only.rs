//! Fresh selected-content process proves a data-added species across the real food chain.
use std::path::Path;

#[test]
fn apple_addition_runs_in_a_fresh_process_without_simulation_code_changes() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = std::env::temp_dir().join(format!("worldsim-m6a-apple-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fresh external evidence root");
    let output = root.join("observed");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_worldsim_probe"))
        .args(["m6a-data-only", "--out"])
        .arg(&output)
        .env(
            "WORLDSIM_CONTENT_PACK",
            checkout.join("rust/crates/sim-test/data/m6a_apple/pack.ron"),
        )
        .output()
        .expect("selected content process");
    std::fs::write(root.join("stdout.log"), &result.stdout).expect("stdout evidence");
    std::fs::write(root.join("stderr.log"), &result.stderr).expect("stderr evidence");
    assert!(
        result.status.success(),
        "apple fixture failed; evidence at {}",
        root.display()
    );
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("data-only.json")).expect("report"))
            .expect("JSON");
    assert_eq!(value["passed"], true);
    assert_eq!(value["category_fallback"], true);
    assert!(value["intake_evidence"]
        .as_array()
        .is_some_and(|events| !events.is_empty()));
}

#[test]
fn third_crop_runs_real_field_jobs_in_an_isolated_catalog_process() {
    const CHILD: &str = "WORLDSIM_TEST_THIRD_CROP_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let result = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "third_crop_runs_real_field_jobs_in_an_isolated_catalog_process",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .expect("isolated catalog child");
        assert!(
            result.status.success(),
            "child failed: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    use sim_core::colony::content::{bootstrap, BootSelection, NamedSource, RawSources};
    use sim_core::colony::cultivation::FieldPhase;
    use sim_core::colony::{Command, ItemSpec, Job, JobKind, JobStage};
    use sim_core::components::{Agent, AgentState, Position, SeekTarget, Settlement, TargetKind};
    use sim_engine::RuntimeSystem;
    let extension = include_str!("../../sim-core/data/colony/crops.ron")
        .replace("code: 0", "code: 20")
        .replace("code: 1", "code: 21")
        .replace("id: \"emmer\"", "id: \"third_grain\"")
        .replace("id: \"flax\"", "id: \"third_fibre\"")
        .replace("serde_name: \"Emmer\"", "serde_name: \"ThirdGrain\"")
        .replace("serde_name: \"Flax\"", "serde_name: \"ThirdFibre\"");
    let catalog = bootstrap(BootSelection::Explicit {
        request_id: "third-crop-job-test".into(),
        sources: RawSources {
            sources: vec![NamedSource {
                name: "crops.ron".into(),
                bytes: extension.into_bytes(),
            }],
        },
    })
    .expect("selected data-only crop pack");
    let crop = catalog.crop_by_name("third_grain").expect("added crop");
    let def = catalog.crop(crop).expect("definition");
    assert_eq!(crop.code(), 20);
    let mut res = sim_engine::SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.rules.colony.as_mut().expect("colony").knowledge = None;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut home = Settlement::new_with_id(1, 0);
    home.member_agents.insert(7);
    res.settlements.insert(1, home);
    let mut world = hecs::World::new();
    let entity = world.spawn((Agent { id: 7 }, Position::new(2, 2), AgentState::Idle));
    assert!(
        sim_systems::commands::apply_command(
            &mut world,
            &mut res,
            Command::ZoneField {
                min: (2, 2),
                max: (2, 2),
                crop
            }
        )
        .accepted
    );
    let mut executor = sim_systems::runtime::jobs::JobExecuteSystem::new();
    let run = |world: &mut hecs::World,
               res: &mut sim_engine::SimResources,
               executor: &mut sim_systems::runtime::jobs::JobExecuteSystem,
               kind: JobKind,
               duration: u32| {
        let job = Job {
            kind,
            stage: JobStage::ToDest,
            taken_at: res.current_tick,
            amount: def.seeds_per_tile,
            item: def.seed_item,
            material: None,
            fetch: None,
            best_distance: 0,
            progress_tick: res.current_tick,
        };
        for key in job.keys() {
            res.colony.reservations.insert(key, 7);
        }
        world
            .insert(
                entity,
                (
                    job,
                    AgentState::Seeking {
                        target: TargetKind::Job,
                    },
                    SeekTarget::new((2, 2)),
                ),
            )
            .expect("field job");
        for _ in 0..=duration {
            res.current_tick += 1;
            executor.tick(world, res);
        }
        assert!(world.get::<&Job>(entity).is_err(), "actual job must finish");
    };
    run(
        &mut world,
        &mut res,
        &mut executor,
        JobKind::TillField { tile: (2, 2) },
        def.till_ticks,
    );
    assert_eq!(res.cultivation.fields[&(2, 2)].phase, FieldPhase::Tilled);
    // Fixture supplies physical seeds; actual pickup and executor consume them.
    sim_systems::runtime::food::items::create_ground(
        &mut res,
        (1, 1),
        ItemSpec::raw(def.seed_item, None),
        def.seeds_per_tile,
        0,
        sim_systems::runtime::food::items::FoodSource::Debug,
    )
    .expect("physical seed");
    let mut sow = Job {
        kind: JobKind::SowField {
            tile: (2, 2),
            from: (1, 1),
        },
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount: def.seeds_per_tile,
        item: def.seed_item,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: res.current_tick,
    };
    for key in sow.keys() {
        res.colony.reservations.insert(key, 7);
    }
    world
        .insert(
            entity,
            (
                sow,
                Position::new(1, 1),
                AgentState::Seeking {
                    target: TargetKind::Job,
                },
                SeekTarget::new((1, 1)),
            ),
        )
        .expect("seed source arrival");
    res.current_tick += 1;
    executor.tick(&mut world, &mut res);
    assert!(!res.colony.ground_items.contains_key(&(1, 1)));
    sow = *world.get::<&Job>(entity).expect("sow delivery");
    assert_eq!(sow.stage, JobStage::ToDest);
    // Arrival is fixture-controlled; all work and material changes use the real executor.
    world
        .insert_one(entity, Position::new(2, 2))
        .expect("field arrival");
    for _ in 0..=def.sow_ticks {
        res.current_tick += 1;
        executor.tick(&mut world, &mut res);
    }
    assert!(matches!(
        res.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Sown { .. }
    ));
    assert_eq!(
        res.cultivation.counters.seed_units_sown[&crop],
        u64::from(def.seeds_per_tile)
    );
    run(
        &mut world,
        &mut res,
        &mut executor,
        JobKind::TendField { tile: (2, 2) },
        def.tend_ticks,
    );
    sim_systems::runtime::cultivation::update(&mut res);
    for day in 1..=def.growth_days {
        res.current_tick = u64::from(day) * res.ticks_per_day;
        sim_systems::runtime::cultivation::update(&mut res);
    }
    assert!(matches!(
        res.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Ripe { .. }
    ));
    run(
        &mut world,
        &mut res,
        &mut executor,
        JobKind::HarvestField { tile: (2, 2) },
        def.harvest_ticks,
    );
    assert!(matches!(
        res.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Harvested { .. }
    ));
    for output in &def.yields {
        let physical: u32 = res
            .colony
            .ground_items
            .values()
            .filter(|stack| stack.kind == output.item)
            .map(|stack| u32::from(stack.count))
            .sum();
        assert_eq!(physical, u32::from(output.count));
    }
    assert_eq!(
        res.cultivation.fertility[&(2, 2)],
        def.initial_fertility - def.fertility_cost
    );
    assert_eq!(res.cultivation.counters.tiles_tilled, 1);
    assert_eq!(res.cultivation.counters.sowings, 1);
    assert_eq!(res.cultivation.counters.harvests, 1);
    assert_eq!(
        res.colony.item_ledger.consumed[&def.seed_item],
        u64::from(def.seeds_per_tile)
    );
}
