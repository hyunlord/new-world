//! M-SPINE settlement-era coverage and actual completed-job discovery origins.
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::knowledge::{self, Cond, Knowledge, Source, TechAggregate};
use sim_core::colony::{ItemKind, Job, JobKind, JobStage, Skills};
use sim_core::components::{Agent, Child};
use sim_systems::runtime::knowledge::{aggregate::compute, discover::holds};

#[test]
fn spine_eras_from_data() {
    // The first observation establishes the founder era without inventing a change.
    let mut engine = sim_test::colony_engine(0);
    engine.tick();
    assert!(!engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::EraChanged));
    let (initial, initial_events) =
        compute(&engine.world, &engine.resources, &TechAggregate::default());
    assert!(!initial.era_progress.is_empty());
    assert!(!initial_events
        .iter()
        .any(|e| e.kind == ChronicleKind::EraChanged));
    // Given the actual authored map and a living colony whose adults know all markers.
    let adults: Vec<_> = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    assert!(!adults.is_empty());
    for entity in &adults {
        let mut knowledge = Knowledge::default();
        for node in &knowledge::defs().nodes {
            if node.marker_weight > 0 {
                knowledge.set(node.id, 1000, Source::Discovered, 0);
            }
        }
        engine
            .world
            .insert_one(*entity, knowledge)
            .expect("adult knowledge");
    }
    // When the aggregate reads practitioners, the authored neolithic denominator has no absent cap.
    let (full, events) = compute(&engine.world, &engine.resources, &initial);
    assert_eq!(full.era_progress, [0, 1000, 1000, 0]);
    assert_eq!(full.current_era, (2, 2));
    assert!(events
        .iter()
        .any(|e| e.kind == ChronicleKind::EraChanged && e.value == 2 && e.cause == 2));
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == ChronicleKind::EraChanged)
            .count(),
        1
    );
    // A saved, measured world cannot announce the same era again after loading.
    engine.resources.colony.tech = full.clone();
    let bytes = sim_systems::save::save(&mut engine).expect("measured era save");
    let loaded = sim_systems::save::load(&bytes).expect("measured era load");
    let (resumed, events) = compute(
        &loaded.world,
        &loaded.resources,
        &loaded.resources.colony.tech,
    );
    assert_eq!(resumed.current_era, full.current_era);
    assert!(!events.iter().any(|e| e.kind == ChronicleKind::EraChanged));
    // Given a sole remaining practitioner, losing that person must allow era regression.
    for entity in adults.iter().skip(1) {
        engine.world.despawn(*entity).expect("other adults leave");
    }
    let (last, _) = compute(&engine.world, &engine.resources, &full);
    assert_eq!(last.current_era, (2, 2));
    engine
        .world
        .despawn(adults[0])
        .expect("last practitioner dies");
    // Then the live aggregate returns to the founder era and records the regression once.
    let (lost, events) = compute(&engine.world, &engine.resources, &last);
    assert_eq!(lost.era_progress, [0, 0, 0, 0]);
    assert_eq!(lost.current_era, (0, 2));
    assert!(events
        .iter()
        .any(|e| e.kind == ChronicleKind::KnowledgeLost));
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == ChronicleKind::EraChanged)
            .count(),
        1
    );
    let (_, repeated) = compute(&engine.world, &engine.resources, &lost);
    assert!(!repeated.iter().any(|e| e.kind == ChronicleKind::EraChanged));
}

#[test]
fn spine_practice_counts_finished_jobs_once_and_never_cancellation() {
    // Given one colonist with no completed builds and a real job completion seam.
    let mut engine = sim_test::colony_engine(1);
    let entity = engine
        .world
        .query::<&Agent>()
        .iter()
        .next()
        .map(|(e, _)| e)
        .expect("colonist");
    let condition = Cond::Practice(JobKind::Build { site: (0, 0) }.code(), 1);
    let job = Job {
        kind: JobKind::Build { site: (0, 0) },
        stage: JobStage::Working { ticks_left: 0 },
        taken_at: 0,
        amount: 0,
        item: ItemKind::Log,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    engine.world.insert_one(entity, job).expect("job");
    assert!(!holds(&engine.world, &engine.resources, condition));
    // When the existing completion seam consumes the job, exactly one completion is recorded.
    sim_systems::runtime::jobs::cancel::finish(&mut engine.world, &mut engine.resources, entity);
    sim_systems::runtime::jobs::cancel::finish(&mut engine.world, &mut engine.resources, entity);
    assert!(holds(&engine.world, &engine.resources, condition));
    assert!(!holds(
        &engine.world,
        &engine.resources,
        Cond::Practice(4, 2)
    ));
    assert_eq!(
        engine
            .world
            .get::<&Skills>(entity)
            .expect("skills")
            .completed[4],
        1
    );
    // Then cancellation cannot count a second completed build.
    engine.world.insert_one(entity, job).expect("second job");
    sim_systems::runtime::jobs::cancel::cancel(
        &mut engine.world,
        &mut engine.resources,
        entity,
        sim_core::colony::CancelReason::Unreachable,
    );
    assert!(!holds(
        &engine.world,
        &engine.resources,
        Cond::Practice(4, 2)
    ));
}
