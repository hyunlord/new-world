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
