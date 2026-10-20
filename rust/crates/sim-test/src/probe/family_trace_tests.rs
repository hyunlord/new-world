//! Unit tests of the family tracer (compiled as the `tests` module of `family_trace.rs`).
use super::super::agent_trace::AgentTracer;
use super::*;
use sim_core::components::{Child, Elder, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::FissionRecord;

fn engine() -> SimEngine {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    engine.resources.world_seed = 42;
    engine.resources.rules = WorldRules::baseline(42);
    engine
}

fn founder(engine: &mut SimEngine, key: u64, female: bool) -> (hecs::Entity, AgentId) {
    let entity = engine.spawn_agent(4, 4);
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    engine
        .world
        .insert(
            entity,
            (
                Lineage { key, births: 0 },
                Sex { female },
                Kin::default(),
                Temperament::founder(engine.resources.world_seed, key),
            ),
        )
        .expect("identity");
    (entity, id)
}

fn emit(engine: &mut SimEngine, events: Vec<FamilyEvent>) -> hecs::Entity {
    engine.world.spawn((FamilyObservations {
        tick: engine.resources.current_tick,
        events,
    },))
}

fn file<'a>(output: &'a FamilyOutput, suffix: &str) -> &'a [Value] {
    &output
        .files
        .iter()
        .find(|(name, _)| *name == suffix)
        .expect("family file")
        .1
}

#[test]
fn durable_birth_and_pairing_survive_subject_removal_and_reconcile_base_people() {
    let mut engine = engine();
    let (mother_entity, mother) = founder(&mut engine, 10, true);
    let (_, father) = founder(&mut engine, 11, false);
    let mut trace = AgentTracer::start(&engine);
    let temperament = Temperament::child(
        42,
        12,
        [
            Some(Temperament::founder(42, 10)),
            Some(Temperament::founder(42, 11)),
        ],
    );
    engine.tick();
    emit(
        &mut engine,
        vec![
            FamilyEvent::Paired { mother, father },
            FamilyEvent::Born {
                child: 900,
                mother: Some(mother),
                father: Some(father),
                fallback: false,
                lineage_key: 12,
                female: true,
                temperament,
            },
        ],
    );
    engine.resources.tallies.births += 1;
    engine.world.despawn(mother_entity).expect("mother removed");
    trace.after_tick(&engine);
    let output = trace.family_output(&engine).expect("enabled");
    let people = file(&output, "family_people");
    assert_eq!(people.len(), 3);
    let child = people
        .iter()
        .find(|person| person["id"] == 900)
        .expect("newborn retained");
    assert_eq!(child["alive_end"], false);
    assert_eq!(child["born_tick"], 0);
    assert_eq!(child["temperament"], json!(temperament));
    let mother = people
        .iter()
        .find(|person| person["id"] == mother)
        .expect("mother retained");
    assert_eq!(mother["pairing_tick"], 0);
    assert_eq!(mother["alive_end"], false);
    assert_eq!(file(&output, "family_events").len(), 2);
    assert_eq!(output.meta["births_total"], 1);
    assert_eq!(output.meta["observation_misses"], 0);
    let (_, ordinary, _, meta) = trace.finish(&engine);
    assert_eq!(ordinary.len(), 3);
    let newborn = ordinary
        .iter()
        .find(|person| person["id"] == 900)
        .expect("base newborn retained");
    assert_eq!(newborn["alive_end"], false);
    assert_eq!(newborn["ever_adult"], false);
    assert_eq!(newborn["born_event_found"], true);
    assert_eq!(meta["born_event_misses"], 0);
    assert_eq!(meta["component_mismatches"], 0);
}

#[test]
fn current_tick_events_are_observed_once_and_stale_batches_are_ignored() {
    let mut engine = engine();
    let (_, mother) = founder(&mut engine, 10, true);
    let (_, father) = founder(&mut engine, 11, false);
    let mut trace = FamilyTracer::start(&engine).expect("enabled");
    engine.tick();
    emit(
        &mut engine,
        vec![FamilyEvent::Born {
            child: 900,
            mother: Some(mother),
            father: Some(father),
            fallback: true,
            lineage_key: 12,
            female: false,
            temperament: Temperament::founder(42, 12),
        }],
    );
    engine.resources.tallies.births += 1;
    assert_eq!(trace.after_tick(&engine).len(), 1);
    assert!(trace.after_tick(&engine).is_empty());
    engine.tick();
    assert!(trace.after_tick(&engine).is_empty());
    let output = trace.output(&engine);
    assert_eq!(file(&output, "family_events").len(), 1);
    assert_eq!(output.meta["observation_misses"], 0);
    assert_eq!(
        output.meta["births_by_day"],
        json!([{"day":1,"births":1,"couple_births":0,"fallback_births":1}])
    );
}

#[test]
fn daily_births_use_elapsed_day_twenty_and_include_zero_days() {
    let mut engine = engine();
    let mut trace = FamilyTracer::start(&engine).expect("enabled");
    for _ in 0..=19 * 1440 {
        engine.tick();
    }
    emit(
        &mut engine,
        vec![FamilyEvent::Born {
            child: 900,
            mother: None,
            father: None,
            fallback: true,
            lineage_key: 12,
            female: false,
            temperament: Temperament::founder(42, 12),
        }],
    );
    engine.resources.tallies.births += 1;
    trace.after_tick(&engine);
    let output = trace.output(&engine);
    assert_eq!(file(&output, "family_events")[0]["simulation_day"], 20);
    assert_eq!(file(&output, "family_events")[0]["tick"], 27360);
    let days = output.meta["births_by_day"].as_array().expect("days");
    assert_eq!(days.len(), 20);
    assert!(days[..19].iter().all(|row| row["births"] == 0));
    assert_eq!(
        days[19],
        json!({"day":20,"births":1,"couple_births":0,"fallback_births":1})
    );
}

#[test]
fn exact_day_sixty_snapshot_uses_completed_elapsed_ticks_not_display_day() {
    let mut engine = engine();
    founder(&mut engine, 10, true);
    let mut trace = FamilyTracer::start(&engine).expect("enabled");
    for _ in 0..86399 {
        engine.tick();
    }
    trace.after_tick(&engine);
    assert!(trace.output(&engine).meta["day60"].is_null());
    engine.tick();
    trace.after_tick(&engine);
    let output = trace.output(&engine);
    assert_eq!(output.meta["day60"]["tick"], 86399);
    assert_eq!(output.meta["day60"]["elapsed_ticks"], 86400);
    assert_eq!(output.meta["day60"]["simulation_day"], 60);
    assert_eq!(output.meta["day60"]["display_day"], 61);
    assert_eq!(output.meta["day60"]["minute"], 479);
    assert_eq!(output.meta["observation_misses"], 0);
}

#[test]
fn hourly_households_and_exact_twenty_hour_beds_preserve_all_life_stages() {
    let mut engine = engine();
    engine.resources.rules.clock_offset_ticks = 1200;
    let (mother_entity, mother) = founder(&mut engine, 10, true);
    let (father_entity, father) = founder(&mut engine, 11, false);
    let (child_entity, child) = founder(&mut engine, 12, true);
    let (elder_entity, elder) = founder(&mut engine, 13, false);
    engine
        .world
        .get::<&mut Kin>(mother_entity)
        .expect("mother")
        .partner = Some(father);
    engine
        .world
        .get::<&mut Kin>(father_entity)
        .expect("father")
        .partner = Some(mother);
    *engine.world.get::<&mut Kin>(child_entity).expect("child") = Kin {
        mother: Some(mother),
        father: Some(father),
        ..Kin::default()
    };
    engine
        .world
        .insert_one(child_entity, Child { born_tick: 0 })
        .expect("child");
    engine.world.insert_one(elder_entity, Elder).expect("elder");
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement
        .member_agents
        .extend([mother, father, child, elder]);
    engine.resources.settlements.insert(1, settlement);
    engine.resources.bed_of.insert(mother, (5, 5));
    engine.resources.bed_of.insert(father, (5, 5));
    let mut trace = FamilyTracer::start(&engine).expect("enabled");
    engine.tick();
    trace.after_tick(&engine);
    let output = trace.output(&engine);
    let hour = &file(&output, "family_hours")[0];
    assert_eq!(hour["tick"], 0);
    assert_eq!(hour["minute"], 1200);
    assert_eq!(hour["people_with_bed"], 2);
    assert_eq!(hour["bedless"], 2);
    let members = hour["people"].as_array().expect("members");
    for id in [mother, father, child] {
        assert_eq!(
            members.iter().find(|row| row["id"] == id).expect("member")["household_key"],
            mother.min(father).min(child)
        );
    }
    assert_eq!(
        members
            .iter()
            .find(|row| row["id"] == child)
            .expect("child")["life_stage"],
        "child"
    );
    assert_eq!(
        members
            .iter()
            .find(|row| row["id"] == elder)
            .expect("elder")["life_stage"],
        "elder"
    );
    assert!(file(&output, "family_people")
        .iter()
        .all(|row| row["shelter_nights"].as_array().expect("nights").len() == 1));
}

fn fission(id: u32, founded_tick: Option<u64>, cancelled_tick: Option<u64>) -> FissionRecord {
    FissionRecord {
        fission_id: id,
        parent: 1,
        site: (8, 8),
        depart_tick: 0,
        members: vec![1, 2],
        adults: 2,
        daughter: founded_tick.map(|_| 2),
        founded_tick,
        cancelled_tick,
        food_taken: 0,
        wood_taken: 0,
        settled_at_leader: false,
        pressure_at_split: 0,
        threshold_at_split: 0,
    }
}

#[test]
fn departure_rows_join_all_fission_statuses_and_retain_source_partitions() {
    let mut engine = engine();
    let mut trace = FamilyTracer::start(&engine).expect("enabled");
    engine.tick();
    for id in 1..=3 {
        family_events::observe(
            &mut trace,
            &engine,
            FamilyEvent::Departed {
                fission_id: id,
                parent: 1,
                leader: 1,
                members: vec![1, 2],
                households: vec![vec![1, 2], vec![3, 4]],
                couples: vec![[1, 2], [3, 4]],
                couples_total: 2,
                couples_split: 0,
                households_split: 0,
            },
        );
    }
    engine.resources.fission_log.extend([
        fission(1, Some(10), None),
        fission(2, None, Some(10)),
        fission(3, None, None),
    ]);
    let output = trace.output(&engine);
    let rows = file(&output, "family_fissions");
    assert_eq!(
        rows.iter()
            .map(|row| row["status"].as_str().expect("status"))
            .collect::<Vec<_>>(),
        ["completed", "cancelled", "pending"]
    );
    assert_eq!(rows[0]["households_departed_whole"], 1);
    assert_eq!(rows[0]["households_total"], 2);
    assert_eq!(rows[0]["couples_total"], 2);
    assert_eq!(output.meta["observation_misses"], 0);
    trace.departures.remove(&2);
    assert_eq!(trace.output(&engine).meta["observation_misses"], 1);
}

#[test]
fn family_none_has_no_trace_and_twins_compare_sex_kin_and_durable_events() {
    let mut first = engine();
    let (entity, _) = founder(&mut first, 10, true);
    let mut second = engine();
    let (other, _) = founder(&mut second, 10, true);
    assert!(twins_match(&first, &second));
    second.world.get::<&mut Sex>(other).expect("sex").female = false;
    assert!(!twins_match(&first, &second));
    second.world.get::<&mut Sex>(other).expect("sex").female = true;
    second
        .world
        .get::<&mut Kin>(other)
        .expect("kin")
        .last_birth_tick = Some(20);
    assert!(!twins_match(&first, &second));
    first
        .world
        .get::<&mut Kin>(entity)
        .expect("kin")
        .last_birth_tick = Some(20);
    assert!(twins_match(&first, &second));
    emit(&mut first, Vec::new());
    assert!(!twins_match(&first, &second));
    emit(&mut second, Vec::new());
    assert!(twins_match(&first, &second));
    first.resources.rules.society.family = None;
    let before = super::super::fingerprint::fingerprint(&first);
    assert!(FamilyTracer::start(&first).is_none());
    assert_eq!(before, super::super::fingerprint::fingerprint(&first));
}
