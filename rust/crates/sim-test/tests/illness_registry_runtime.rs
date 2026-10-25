//! Isolated selected-catalog illness provenance through real runtime and save boundaries.
use std::collections::BTreeSet;

use hecs::Entity;
use sim_core::causal::{CausalEvent, DeathReason};
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::content::{bootstrap, BootSelection, NamedSource, RawSources};
use sim_core::colony::people::{Modifier, Sick};
use sim_core::colony::story::PressureKind;
use sim_core::components::{Agent, AgentId, Sleep};
use sim_core::rooms::{Room, RoomRole};
use sim_core::rules::ColonyRule;
use sim_core::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::runtime::{mood, needs::SleepDecaySystem, story, survival::despawn_agent};
use sim_systems::save;

const NOVEL: u32 = u32::MAX;

fn register(engine: &mut SimEngine) {
    engine.register_system(Box::new(SleepDecaySystem::new()));
    engine.register_system(Box::new(story::StorytellerSystem));
}

fn resident(engine: &mut SimEngine, x: u32) -> (Entity, AgentId) {
    let entity = engine.spawn_agent(x, 2);
    engine
        .world
        .insert_one(entity, Sleep::new(0.0, 1.0))
        .expect("sleep");
    let id = engine.world.get::<&Agent>(entity).expect("resident").id;
    engine.resources.bed_of.insert(id, (x, 2));
    engine.resources.colony.room_of_tile.insert((x, 2), 1);
    (entity, id)
}

fn advance_to(engine: &mut SimEngine, tick: u64) {
    assert!(tick <= 5_000, "bounded lifecycle fixture");
    while engine.current_tick() < tick {
        engine.tick();
    }
}

fn round_trip(engine: &mut SimEngine) -> SimEngine {
    let encoded = save::save(engine).expect("save actual runtime state");
    let restored = save::load_with(&encoded, register).expect("load actual runtime state");
    assert_eq!(
        restored.resources.content_state,
        engine.resources.content_state
    );
    assert_eq!(
        restored.resources.colony.story,
        engine.resources.colony.story
    );
    restored
}

#[test]
fn novel_illness_retains_origin_across_spread_end_save_recovery_and_death() {
    // Given one full-width illness definition and three occupants of a bedroom.
    bootstrap(BootSelection::Explicit {
        request_id: "illness-runtime-provenance".into(),
        sources: RawSources { sources: vec![NamedSource {
            name: "pressures.ron".into(),
            bytes: br#"(schema_version:1,roles:{},schedule:None,pressures:[(
                identity:(code:4294967295,id:"fixture_fever",name_key:"FIXTURE_FEVER",
                    serde_name:"FixtureFever",serde_aliases:[],resolver_aliases:[]),
                category:"illness",mechanic:Illness,major:false,enabled_rule:None,
                duration:(value:(2,2),rule:None),duration_draw_slot:None,
                mercy:(first_day:None,first_winter_days:None,repeat_gap_days:None,winter_end_days:None),
                due_days:None,due_seasons:[],intensity:None,cold:None,drought:None,pack:None,
                illness:Some((work:(value:1750,rule:None),sleep:(value:2000,rule:None),
                    spread:(value:1000,rule:None))),
            )])"#.to_vec(),
        }] },
    }).expect("install selected illness before constructing a world");
    let kind = PressureKind::checked(NOVEL).expect("full-width illness");
    let mut engine = SimEngine::new(12, 12, MaterialRegistry::new());
    engine.resources.rules.colony = Some(ColonyRule::M5);
    engine.resources.rules.seasons = None;
    engine.resources.rules.clock_offset_ticks = 0;
    register(&mut engine);
    engine.resources.colony.rooms.insert(
        1,
        Room {
            id: 1,
            tiles: BTreeSet::from([(2, 2), (3, 2), (4, 2)]),
            doors: BTreeSet::new(),
            roofed: true,
            role: RoomRole::Bedroom,
            dug: false,
        },
    );
    let first = resident(&mut engine, 2);
    let second = resident(&mut engine, 3);

    // When generic admission begins the novel pressure and rejects the bundled same mechanic.
    story::try_force(&mut engine.world, &mut engine.resources, kind, 2).expect("novel force");
    let before_conflict = engine.resources.colony.story.clone();
    assert_eq!(
        story::try_force(
            &mut engine.world,
            &mut engine.resources,
            PressureKind::Illness,
            2
        ),
        Err(story::ForceError::MechanicConflict {
            active: kind,
            requested: PressureKind::Illness
        })
    );
    assert_eq!(engine.resources.colony.story, before_conflict);
    let initial_id = *engine
        .resources
        .content_state
        .illness_origins
        .keys()
        .next()
        .expect("initial onset");
    let initial = if first.1 == initial_id { first } else { second };
    let other = if first.1 == initial_id { second } else { first };
    assert_eq!(
        story::work_permille(&engine.world, &engine.resources, initial.0, (2, 2)),
        1750
    );
    engine.tick(); // Sleep runs before the daily storyteller's guaranteed same-bedroom spread.

    // Then actual IDs are retained in spread, cost, causal events and sleep effects.
    assert_eq!(engine.resources.content_state.illness_origins.len(), 2);
    assert_eq!(
        engine
            .world
            .get::<&Sleep>(initial.0)
            .expect("sleep")
            .fatigue,
        2.0
    );
    assert_eq!(
        engine.world.get::<&Sleep>(other.0).expect("sleep").fatigue,
        1.0
    );
    assert_eq!(
        engine
            .resources
            .colony
            .story
            .active_of(kind)
            .expect("active")
            .cost
            .work_ticks_lost,
        2 * engine.resources.ticks_per_day
    );
    let onset_events = engine
        .resources
        .causal_log
        .iter()
        .flat_map(|(_, log)| log.iter())
        .filter(|event| {
            matches!(
                event,
                CausalEvent::PressureCause {
                    kind: NOVEL,
                    effect: 2,
                    ..
                }
            )
        })
        .count();
    assert_eq!(onset_events, 2);
    let began = engine
        .resources
        .colony
        .story
        .active_of(kind)
        .expect("active")
        .began_entry;
    let sickness: Vec<_> = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.kind == ChronicleKind::Sick)
        .collect();
    assert_eq!(sickness.len(), 2);
    for (index, _) in sickness {
        assert_eq!(
            engine.resources.chronicle.cause_of(index as u32),
            Some(CauseRef::Pressure {
                kind: NOVEL,
                entry: began
            })
        );
    }

    // When the pressure ends early, a save/load retains the per-person origin and effects.
    story::end_pressure(&mut engine.resources, kind);
    engine = round_trip(&mut engine);
    assert_eq!(
        engine
            .resources
            .colony
            .story
            .history
            .last()
            .expect("history")
            .kind,
        kind
    );
    assert_eq!(
        story::work_permille(&engine.world, &engine.resources, other.0, (3, 2)),
        1750
    );
    assert_eq!(
        mood::cause_for_person(&engine.resources, other.1, Modifier::Sick, None),
        CauseRef::Modifier {
            code: Modifier::Sick.code(),
            entry: began
        }
    );
    engine.tick();
    assert_eq!(
        engine
            .world
            .get::<&Sleep>(other.0)
            .expect("lingering sleep")
            .fatigue,
        3.0
    );
    let third = resident(&mut engine, 4);
    advance_to(&mut engine, 1_441);
    assert_eq!(
        engine.resources.content_state.illness_origins.get(&third.1),
        Some(&kind)
    );
    assert!(engine.world.get::<&Sick>(third.0).is_ok());
    let last_sick = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .enumerate()
        .rev()
        .find(|(_, entry)| entry.kind == ChronicleKind::Sick)
        .expect("post-end spread");
    assert_eq!(last_sick.1.subject, Some(third.1));
    assert_eq!(
        engine.resources.chronicle.cause_of(last_sick.0 as u32),
        Some(CauseRef::Pressure {
            kind: NOVEL,
            entry: began
        })
    );

    // When one carrier dies, the shared lifecycle seam cleans it before an immediate save.
    let tick = engine.current_tick();
    despawn_agent(
        &mut engine.world,
        &mut engine.resources,
        initial.0,
        initial.1,
        (2, 2),
        DeathReason::OldAge,
        tick,
    );
    assert!(!engine
        .resources
        .content_state
        .illness_origins
        .contains_key(&initial.1));
    assert_eq!(
        engine.resources.content_state.illness_origins.get(&other.1),
        Some(&kind)
    );
    engine = round_trip(&mut engine);

    // Then remaining carriers recover on their actual duration and save without stale origins.
    advance_to(&mut engine, 4_321);
    assert!(engine.world.get::<&Sick>(other.0).is_err());
    assert!(engine.world.get::<&Sick>(third.0).is_err());
    assert!(engine.resources.content_state.illness_origins.is_empty());
    assert_eq!(
        story::work_permille(&engine.world, &engine.resources, third.0, (4, 2)),
        1000
    );
    let _ = round_trip(&mut engine);
}
