//! Isolated selected-species pack spawning, combat, loot and provenance across a save boundary.
use sim_core::causal::CausalEvent;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::content::{
    bootstrap, AnimalKind, BootSelection, NamedSource, PressureKind, RawSources,
};
use sim_core::colony::wildlife::{AnimalState, Fighter};
use sim_core::colony::ItemKind;
use sim_core::components::{Agent, BodyHealth, BuildingKind};
use sim_core::rules::{ColonyRule, WildlifeRule};
use sim_core::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::runtime::combat::CombatSystem;
use sim_systems::runtime::wildlife::{self, AnimalSystem};
use sim_systems::{runtime::story, save};

const CHILD: &str = "WORLDSIM_PREDATOR_REGISTRY_CHILD";
const SPECIES: u32 = u32::MAX;
const PRESSURE: u32 = u32::MAX - 1;

fn extension() -> RawSources {
    RawSources {
        sources: vec![
            NamedSource {
                name: "animals.ron".into(),
                bytes: br#"(schema_version:1, roles:{}, animals:[(
                    identity:(code:4294967295,id:"fixture_jackal",name_key:"FIXTURE_JACKAL",
                        serde_name:"FixtureJackal",serde_aliases:[],resolver_aliases:[]),
                    category:"predator",mechanic:Predator,huntable:false,
                    hp:(value:11,rule:None),grazer:None,
                    predator:Some((sight:(value:12,rule:None),damage:(value:3,rule:None),
                        chase_tiles:(value:2,rule:None),hit_ticks:(value:2,rule:None))),
                    loot:(meat:None,hide:Some((item:"hide",count:3,
                        enabled:(value:true,rule:None)))),
                )])"#.to_vec(),
            },
            NamedSource {
                name: "pressures.ron".into(),
                bytes: br#"(schema_version:1, roles:{}, pressures:[(
                    identity:(code:4294967294,id:"fixture_jackal_pack",name_key:"FIXTURE_JACKAL_PACK",
                        serde_name:"FixtureJackalPack",serde_aliases:[],resolver_aliases:[]),
                    category:"pack",mechanic:Pack,major:true,enabled_rule:None,
                    duration:(value:(2,2),rule:None),duration_draw_slot:None,
                    mercy:(first_day:None,first_winter_days:None,repeat_gap_days:None,winter_end_days:None),
                    due_days:None,due_seasons:[],
                    intensity:PackByWealth(base:(value:1,rule:None),
                        per_items:(value:100,rule:None),max:(value:1,rule:None)),
                    cold:None,drought:None,illness:None,pack:Some((animal:"fixture_jackal")),
                )])"#.to_vec(),
            },
        ],
    }
}

fn register(engine: &mut SimEngine) {
    engine.register_system(Box::new(CombatSystem::new()));
    engine.register_system(Box::new(AnimalSystem));
}

#[test]
fn novel_predator_pack_runs_in_an_isolated_catalogue_process() {
    let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "novel_predator_pack_child", "--nocapture"])
        .env(CHILD, "1")
        .env_remove("WORLDSIM_CONTENT_PACK")
        .status()
        .expect("isolated selected-catalogue child");
    assert!(status.success(), "novel predator runtime regression");
}

#[test]
fn novel_predator_pack_child() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let catalog = bootstrap(BootSelection::Explicit {
        request_id: "predator-runtime-regression".into(),
        sources: extension(),
    })
    .expect("selected predator catalog");
    let species = AnimalKind::checked(SPECIES).expect("full-width species");
    let pressure = PressureKind::checked(PRESSURE).expect("full-width pack pressure");
    assert_eq!(
        catalog.animal_role("default_predator"),
        Some(AnimalKind::Wolf)
    );
    assert_eq!(
        catalog.pressure_role("pack_default"),
        Some(PressureKind::Beasts)
    );
    assert_eq!(
        PressureKind::Beasts
            .definition()
            .pack
            .as_ref()
            .expect("base pack")
            .animal,
        AnimalKind::Wolf
    );

    let mut engine = SimEngine::new(17, 17, MaterialRegistry::new());
    engine.resources.rules.colony = Some(ColonyRule {
        wildlife: Some(WildlifeRule {
            wolf_hp: 99,
            wolf_damage: 19,
            wolf_chase_tiles: 1,
            hit_ticks: 13,
            ..WildlifeRule::M5
        }),
        ..ColonyRule::M5
    });
    engine
        .resources
        .building_kinds
        .insert((8, 8), BuildingKind::Hearth);
    // Disable unrelated ambient seeding; the animal under test must arrive from the actual pressure.
    engine.resources.colony.wildlife.seeded = true;
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let edge = wildlife::wolf_edge(&engine.resources, (8, 8)).expect("reachable edge");
    assert_eq!(edge, (0, 0), "plain symmetric fixture edge");
    let person = engine.spawn_agent(3, 0);
    engine
        .world
        .insert_one(person, BodyHealth::new())
        .expect("fixture health");
    let person_id = engine.world.get::<&Agent>(person).expect("person").id;
    let health = engine.world.get::<&BodyHealth>(person).expect("health").hp;
    register(&mut engine);
    story::try_force(&mut engine.world, &mut engine.resources, pressure, 2)
        .expect("ordinary generic pressure admission");
    let began = engine
        .resources
        .colony
        .story
        .active_of(pressure)
        .expect("actual pack active")
        .began_entry;
    assert!(began.is_some());
    assert!(engine.resources.colony.wildlife.animals.is_empty());
    engine.tick();
    let animal = engine
        .resources
        .colony
        .wildlife
        .wolves()
        .next()
        .expect("pressure spawned predator")
        .clone();
    assert_eq!(
        engine.resources.colony.wildlife.wolves().count(),
        1,
        "declared pack intensity"
    );
    assert_eq!((animal.kind, animal.hp), (species, 11));
    assert_eq!(animal.state, AnimalState::Hunt);
    assert_eq!(
        sim_core::nav::chebyshev(edge, animal.pos),
        2,
        "declared chase speed overrides world's one-tile default"
    );
    assert!(sim_core::nav::chebyshev(animal.pos, (3, 0)) <= 1);
    let animal_id = animal.id;
    for _ in 0..8 {
        if engine.resources.colony.wildlife.tallies.bites > 0 {
            break;
        }
        engine.tick();
    }
    assert_eq!(engine.resources.colony.wildlife.tallies.bites, 1);
    assert_eq!(engine.resources.colony.wildlife.tallies.bite_damage, 3);
    assert_eq!(
        engine
            .world
            .get::<&BodyHealth>(person)
            .expect("survivor")
            .hp,
        health - 3.0
    );
    assert_eq!(
        engine.resources.colony.wildlife.animals[&animal_id].hp, 7,
        "ordinary unarmed retaliation"
    );
    let fight =
        engine.resources.colony.fights[&(Fighter::Animal(animal_id), Fighter::Person(person_id))];
    assert_eq!(
        (fight.ticks, fight.damage),
        (2, 3),
        "declared attack interval and damage"
    );
    assert!(engine.resources.causal_log.iter().any(|(_, log)| log.as_slice().iter().any(|event| {
        matches!(event, CausalEvent::PressureCause { kind, agent, effect:1, .. } if *kind == PRESSURE && *agent == person_id)
    })), "bite retains actual full-width pressure provenance");

    let bytes = save::save(&mut engine).expect("save live novel predator and ongoing fights");
    let mut restored = save::load_with(&bytes, register).expect("restore selected pack combat");
    assert_eq!(
        restored.resources.colony.wildlife.animals[&animal_id].kind,
        species
    );
    assert_eq!(restored.resources.colony.wildlife.animals[&animal_id].hp, 7);
    for _ in 0..20 {
        engine.tick();
        restored.tick();
        if !engine.resources.colony.story.is_active(pressure) {
            break;
        }
    }
    for world in [&engine, &restored] {
        assert!(
            !world
                .resources
                .colony
                .wildlife
                .animals
                .contains_key(&animal_id),
            "real combat killed the predator"
        );
        assert_eq!(world.resources.colony.wildlife.tallies.wolves_killed, 1);
        assert_eq!(
            world
                .resources
                .colony
                .item_ledger
                .created_of(ItemKind::Hide),
            3,
            "declared loot kind and count"
        );
        assert_eq!(world.resources.colony.ground_total(ItemKind::Hide), 3);
        assert_eq!(
            world
                .resources
                .colony
                .item_ledger
                .created_of(ItemKind::WolfHide),
            0,
            "no default-species loot substitution"
        );
        let (index, entry) = world
            .resources
            .chronicle
            .entries()
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.kind == ChronicleKind::WolfKilled)
            .expect("predator death chronicle");
        assert_eq!(entry.cause, SPECIES);
        assert_eq!(entry.subject, Some(person_id));
        assert_eq!(
            world.resources.chronicle.cause_of(index as u32),
            Some(CauseRef::Pressure {
                kind: PRESSURE,
                entry: began
            })
        );
        assert!(
            !world.resources.colony.story.is_active(pressure),
            "empty pack ends its actual pressure"
        );
        assert!(world
            .resources
            .colony
            .story
            .history
            .iter()
            .any(|p| p.kind == pressure));
        assert!(world
            .resources
            .chronicle
            .entries()
            .iter()
            .any(|entry| entry.kind == ChronicleKind::PressureEnded && entry.cause == PRESSURE));
    }
    assert_eq!(
        save::save(&mut engine).expect("continued original"),
        save::save(&mut restored).expect("continued restored"),
        "bounded selected-predator continuation is byte-identical"
    );
}
