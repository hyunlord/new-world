//! Physical field tail and exact pre-cultivation body regression coverage.
use super::*;
use sim_core::colony::cultivation::*;

#[test]
fn format15_migrates_empty_fields_and_format16_retains_erased_soil() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("catalog");
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    let old = legacy_cultivation_v15::LegacyBodyV15 {
        current_tick: engine.current_tick(),
        world: world::capture(&engine.world).expect("world"),
        systems: engine.system_states().expect("systems"),
        steward: std::mem::take(&mut engine.resources.steward),
        content_state: std::mem::take(&mut engine.resources.content_state),
        mind: std::mem::take(&mut engine.resources.mind),
        food: std::mem::take(&mut engine.resources.food),
        resources: engine.resources,
    };
    let envelope = Envelope {
        revision: "catalog-v1-fnv1a64:242eb9f976d38fe8:63794".into(),
        payload: encode(&old).expect("exact format15"),
    };
    let mut bytes = MAGIC.to_vec();
    bytes.extend(15u32.to_le_bytes());
    bytes.extend(encode(&envelope).expect("envelope"));
    let mut loaded = load_with(&bytes, |_| {}).expect("format15 migration");
    assert_eq!(loaded.resources.cultivation, CultivationState::default());
    loaded.resources.cultivation.fertility.insert((2, 2), 600);
    loaded.resources.cultivation.counters.seeded_food_units = 8;
    loaded.resources.cultivation.eaten_by_year.insert(
        2,
        EatenTotals {
            units: 4,
            field_lower: 1_000_000,
            field_upper: 2_000_000,
        },
    );
    let saved = save(&mut loaded).expect("format16");
    let mut again = load_with(&saved, |_| {}).expect("reload");
    assert_eq!(again.resources.cultivation, loaded.resources.cultivation);
    assert_eq!(save(&mut again).expect("stable"), saved);
}

#[test]
fn origin_tail_wire_preserves_all_nonzero_maps_and_pending_alignment() {
    use sim_core::colony::food::FoodOwner;
    let mut tail = CultivationState::default();
    tail.food_origins
        .insert(FoodOwner::Ground((1, 2)), OriginShare::FIELD);
    tail.craft_origins.insert(
        7,
        OriginShare {
            lower: 123,
            upper: 124,
        },
    );
    tail.drying_origins.push(OriginShare::FIELD);
    tail.eaten_by_year.insert(
        2,
        EatenTotals {
            units: 3,
            field_lower: 123,
            field_upper: 456,
        },
    );
    tail.counters.seeded_food_units = 8;
    tail.counters.seed_lost = 2;
    let bytes = encode(&tail).expect("tail");
    let loaded: CultivationState = decode(&bytes).expect("tail reload");
    assert_eq!(loaded, tail);
    let mut resources = SimEngine::new(4, 4, MaterialRegistry::new()).resources;
    resources
        .cultivation
        .drying_origins
        .push(OriginShare::FIELD);
    assert!(cultivation_validation::validate(&hecs::World::new(), &resources).is_err());
    resources.cultivation.drying_origins.clear();
    resources.cultivation.craft_origins.insert(
        7,
        OriginShare {
            lower: 100,
            upper: 99,
        },
    );
    assert!(cultivation_validation::validate(&hecs::World::new(), &resources).is_err());
}

#[test]
fn full_save_roundtrip_preserves_physical_origin_and_work_in_progress() {
    use sim_core::colony::content::installed_or_bundled;
    use sim_core::colony::food::{DryingBatch, FoodOwner};
    use sim_core::colony::{ItemSpec, Job, JobKind, JobStage};
    use sim_core::components::{Agent, Position};
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    engine.resources.rules = sim_core::rules::WorldRules::colony(0);
    engine.resources.rules.clock_offset_ticks = 0;
    let catalog = installed_or_bundled();
    let grain = catalog.item_by_name("wild_grain").expect("grain");
    let dried = catalog
        .item_by_name("dried_berries")
        .expect("dried berries");
    crate::runtime::food::items::create_ground(
        &mut engine.resources,
        (1, 1),
        ItemSpec::raw(grain, None),
        3,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("physical grain");
    engine
        .resources
        .cultivation
        .food_origins
        .insert(FoodOwner::Ground((1, 1)), OriginShare::FIELD);
    engine.world.spawn((
        Agent { id: 7 },
        Position::new(1, 2),
        Job {
            kind: JobKind::Craft {
                workshop: 1,
                bill: 1,
                at: (1, 2),
            },
            stage: JobStage::Working { ticks_left: 12 },
            taken_at: 0,
            item: grain,
            amount: 2,
            material: None,
            fetch: None,
            best_distance: 0,
            progress_tick: 0,
        },
    ));
    engine.resources.cultivation.craft_origins.insert(
        7,
        OriginShare {
            lower: 250_000,
            upper: 250_001,
        },
    );
    engine.resources.food.pending_drying.push(DryingBatch {
        workshop: 1,
        bill: 2,
        tile: (2, 2),
        output: ItemSpec::raw(dried, None),
        count: 2,
        due_tick: 100,
    });
    engine
        .resources
        .cultivation
        .drying_origins
        .push(OriginShare::FIELD);
    engine.resources.cultivation.eaten_by_year.insert(
        2,
        EatenTotals {
            units: 3,
            field_lower: 1_000_000,
            field_upper: 2_000_000,
        },
    );
    let expected = engine.resources.cultivation.clone();
    let bytes = save(&mut engine).expect("physical origin save");
    let loaded = load_with(&bytes, |_| {}).expect("physical origin load");
    assert_eq!(loaded.resources.cultivation, expected);
    assert_eq!(
        loaded.resources.food.pending_drying,
        engine.resources.food.pending_drying
    );
    assert!(loaded
        .world
        .query::<(&Agent, &Job)>()
        .iter()
        .any(|(_, (agent, job))| agent.id == 7
            && matches!(job.stage, JobStage::Working { ticks_left: 12 })));
}

#[test]
fn inflight_sowing_roundtrip_continues_once_with_nonempty_fields_and_soil() {
    use sim_core::colony::content::installed_or_bundled;
    use sim_core::colony::{Carry, ItemSpec, Job, JobKind, JobStage};
    use sim_core::components::{Agent, AgentState, Position, SeekTarget, Settlement, TargetKind};
    use sim_engine::RuntimeSystem;
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules = sim_core::rules::WorldRules::colony(0);
    engine.resources.rules.clock_offset_ticks = 0;
    engine
        .resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .knowledge = None;
    engine
        .resources
        .nav
        .install(sim_core::nav::CostMap::new(8, 8));
    let crop = installed_or_bundled().crop_by_name("emmer").expect("crop");
    let def = installed_or_bundled().crop(crop).expect("definition");
    let mut home = Settlement::new_with_id(1, 0);
    home.member_agents.insert(7);
    engine.resources.settlements.insert(1, home);
    for (tile, phase, active, work) in [
        ((2, 2), FieldPhase::Tilled, true, FieldWork::Now),
        (
            (3, 3),
            FieldPhase::Sown { planted_day: 0 },
            false,
            FieldWork::Paused,
        ),
    ] {
        engine.resources.cultivation.fields.insert(
            tile,
            FieldTile {
                owner: 1,
                crop,
                phase,
                active,
                work,
                tended: true,
            },
        );
        engine.resources.cultivation.fertility.insert(tile, 650);
    }
    crate::runtime::food::items::create_ground(
        &mut engine.resources,
        (1, 1),
        ItemSpec::raw(def.seed_item, None),
        def.seeds_per_tile,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("physical seed");
    let job = Job {
        kind: JobKind::SowField {
            tile: (2, 2),
            from: (1, 1),
        },
        stage: JobStage::ToSource,
        taken_at: 0,
        amount: def.seeds_per_tile,
        item: def.seed_item,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    for key in job.keys() {
        engine.resources.colony.reservations.insert(key, 7);
    }
    let entity = engine.world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        job,
        AgentState::Seeking {
            target: TargetKind::Job,
        },
        SeekTarget::new((1, 1)),
    ));
    let mut executor = crate::runtime::jobs::JobExecuteSystem::new();
    executor.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine
            .world
            .get::<&Carry>(entity)
            .expect("seed cargo")
            .count,
        def.seeds_per_tile
    );
    engine
        .world
        .insert_one(entity, Position::new(2, 2))
        .expect("fixture arrival");
    engine.resources.current_tick += 1;
    executor.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine.world.get::<&Job>(entity).expect("sow work").stage,
        JobStage::Working {
            ticks_left: def.sow_ticks
        }
    );
    assert_eq!(engine.resources.cultivation.counters.sowings, 0);
    let before = engine.resources.cultivation.clone();
    let before_ledger = engine.resources.colony.item_ledger.clone();
    let bytes = save(&mut engine).expect("nonempty fields and in-flight sow save");
    let mut loaded = load_with(&bytes, |_| {}).expect("load in-flight sow");
    assert_eq!(loaded.resources.cultivation, before);
    assert_eq!(loaded.resources.colony.item_ledger, before_ledger);
    assert!(loaded.world.query::<(&Agent, &Carry, &Job)>().iter().any(
        |(_, (agent, carry, job))| agent.id == 7
            && carry.count == def.seeds_per_tile
            && job.stage
                == JobStage::Working {
                    ticks_left: def.sow_ticks
                }
    ));
    for candidate in [&mut engine, &mut loaded] {
        let mut executor = crate::runtime::jobs::JobExecuteSystem::new();
        for _ in 0..def.sow_ticks {
            candidate.resources.current_tick += 1;
            executor.tick(&mut candidate.world, &mut candidate.resources);
        }
        assert_eq!(
            candidate.resources.cultivation.fields[&(2, 2)].phase,
            FieldPhase::Sown { planted_day: 0 }
        );
        assert_eq!(candidate.resources.cultivation.counters.sowings, 1);
        assert_eq!(
            candidate.resources.cultivation.counters.seed_units_sown[&crop],
            u64::from(def.seeds_per_tile)
        );
        assert_eq!(candidate.resources.cultivation.fertility[&(2, 2)], 650);
        assert_eq!(
            candidate.resources.cultivation.fields[&(3, 3)],
            before.fields[&(3, 3)]
        );
        assert!(candidate.world.query::<&Carry>().iter().next().is_none());
        assert!(candidate.world.query::<&Job>().iter().next().is_none());
        assert_eq!(
            candidate
                .resources
                .colony
                .item_ledger
                .consumed
                .get(&def.seed_item)
                .copied()
                .unwrap_or(0),
            before_ledger
                .consumed
                .get(&def.seed_item)
                .copied()
                .unwrap_or(0)
                + u64::from(def.seeds_per_tile)
        );
        let ledger = candidate.resources.colony.item_ledger.clone();
        candidate.resources.current_tick += 1;
        executor.tick(&mut candidate.world, &mut candidate.resources);
        assert_eq!(
            candidate.resources.colony.item_ledger, ledger,
            "no second seed debit"
        );
        assert_eq!(candidate.resources.cultivation.counters.sowings, 1);
    }
    assert_eq!(engine.resources.cultivation, loaded.resources.cultivation);
    assert_eq!(
        engine.resources.colony.item_ledger,
        loaded.resources.colony.item_ledger
    );
}
