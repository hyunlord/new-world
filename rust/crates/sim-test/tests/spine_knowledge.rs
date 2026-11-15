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
    // C3 iron markers make the iron era the full-marker era.
    assert_eq!(full.era_progress, [0, 1000, 1000, 1000]);
    assert_eq!(full.current_era, (3, 2));
    assert!(events
        .iter()
        .any(|e| e.kind == ChronicleKind::EraChanged && e.value == 3 && e.cause == 2));
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
    assert_eq!(last.current_era, (3, 2));
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
fn spine_era_steps_down_only_after_a_full_practice_window_below() {
    // Given a colony whose steady practitioners keep the neolithic just under its
    // transition, and one seasonal worker whose practice lifts it over.
    let mut engine = sim_test::colony_engine(0);
    let window = sim_systems::runtime::knowledge::rule(&engine.resources)
        .expect("colony knowledge rule")
        .practice_days;
    let transition = u64::from(knowledge::defs().eras.eras[1].transition_permille);
    let adults: Vec<_> = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    let count = adults.len() as u64;
    // Smallest practitioner count at or over the threshold; one fewer is under it.
    let over = (transition * count).div_ceil(1000);
    assert!(over >= 1 && over < count, "{over} of {count}");
    let neolithic: Vec<_> = knowledge::defs()
        .nodes
        .iter()
        .filter(|n| n.marker_weight > 0 && n.era.code() == 1)
        .map(|n| n.id)
        .collect();
    // The founders' own measurement establishes the founder era.
    let (mut tech, _) = compute(&engine.world, &engine.resources, &TechAggregate::default());
    assert_eq!(tech.current_era, (0, 2));
    for (i, entity) in adults.iter().enumerate() {
        let mut k = Knowledge::default();
        if (i as u64) < over {
            for id in &neolithic {
                k.set(*id, 1000, Source::Discovered, 0);
            }
        }
        engine
            .world
            .insert_one(*entity, k)
            .expect("adult knowledge");
    }
    let seasonal = adults[usize::try_from(over - 1).expect("index")];
    // The seasonal worker returns every 2 × window days (below for window − 1 days each
    // cycle, as winter work-mix noise does), then stops on `last_day`.
    let period = 2 * window;
    let last_day = 4 * period;
    let mut changes = Vec::new();
    let mut below_since = None;
    for day in 0..last_day + 3 * window {
        for (i, entity) in adults.iter().enumerate() {
            let works = if *entity == seasonal {
                day % period == 0 && day <= last_day
            } else {
                (i as u64) < over - 1
            };
            if works {
                let mut k = engine.world.get::<&mut Knowledge>(*entity).expect("k");
                for id in &neolithic {
                    k.touch(*id, day * 1440);
                }
            }
        }
        for slot in 0..6 {
            engine.resources.current_tick = day * 1440 + slot * 240;
            let (next, events) = compute(&engine.world, &engine.resources, &tech);
            let progress = u64::from(next.era_progress[1]);
            if progress < transition {
                below_since.get_or_insert(engine.resources.current_tick);
            } else {
                below_since = None;
            }
            for e in events
                .iter()
                .filter(|e| e.kind == ChronicleKind::EraChanged)
            {
                changes.push((e.tick, e.value, e.cause, below_since));
            }
            tech = next;
        }
    }
    // Then the noise announces one transition up and no regression while it lasts,
    // and the era falls back once, only after a full practice window below.
    assert_eq!(changes.len(), 2, "{changes:?}");
    assert_eq!((changes[0].1, changes[0].2), (1, 1), "{changes:?}");
    let (tick, value, cause, since) = changes[1];
    assert_eq!((value, cause), (0, 2), "{changes:?}");
    let since = since.expect("below when it falls");
    assert_eq!(since, (last_day + window + 1) * 1440, "{changes:?}");
    assert_eq!(tick, since + window * 1440, "{changes:?}");
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
