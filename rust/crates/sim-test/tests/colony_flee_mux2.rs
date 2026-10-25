//! M-UX2 flight regressions: actual route travel times, refreshed refuge selection and roof-entry immunity.
mod colony_support;

use sim_core::colony::people::Fleeing;
use sim_core::colony::wildlife::{Fight, Fighter};
use sim_core::components::{Agent, AgentState, BodyHealth, Position};
use sim_core::nav::{CostMap, COST_PLAIN, IMPASSABLE};
use sim_core::rooms::{Room, RoomRole};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::wildlife::{self, FleeSystem};
use std::collections::BTreeSet;

fn fixture() -> SimEngine {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = WorldRules::colony(0);
    engine.resources.rules.agent.move_interval_ticks = 3;
    let mut map = CostMap::new(32, 32);
    for y in 0..32 {
        for x in 0..32 {
            map.set((x, y), COST_PLAIN);
        }
    }
    engine.resources.nav.install(map);
    engine
}

fn roof(engine: &mut SimEngine, id: u32, tile: (u32, u32)) {
    engine.resources.colony.rooms.insert(
        id,
        Room {
            id,
            tiles: BTreeSet::from([tile]),
            doors: BTreeSet::new(),
            roofed: true,
            role: RoomRole::Unassigned,
            dug: false,
        },
    );
    engine.resources.colony.room_of_tile.insert(tile, id);
}

#[test]
fn a_geometrically_nearer_roof_is_rejected_when_the_chasing_wolf_arrives_first() {
    // Given: ten person steps versus eighteen wolf steps at twice the pace.
    let mut engine = fixture();
    let room = (2, 10);
    roof(&mut engine, 1, room);
    wildlife::spawn_pack(&mut engine.resources, (20, 10), 1, 0);
    // When: choosing a refuge with a 3-tick person and a 2-step/3-tick wolf.
    let to = wildlife::flee::refuge(&engine.world, &engine.resources, (12, 10), (20, 10), 8);
    // Then: straight-line proximity must not select a room the wolf reaches first.
    assert_ne!(to, room);
}

#[test]
fn a_wall_detour_cannot_make_the_geometrically_nearest_room_win() {
    // Given: the nearby room requires walking around a wall; the eastern room is closer by route.
    let mut engine = fixture();
    let nearby = (8, 10);
    let reachable = (16, 10);
    roof(&mut engine, 1, nearby);
    roof(&mut engine, 2, reachable);
    for y in 4..=16 {
        engine.resources.nav.set_cost((9, y), IMPASSABLE);
    }
    let wolf = (10, 25);
    wildlife::spawn_pack(&mut engine.resources, wolf, 1, 0);
    assert!(
        engine
            .resources
            .nav
            .path((10, 10), nearby)
            .expect("detour")
            .len()
            > 6
    );
    // When: choosing among roofs by actual traversable path.
    let to = wildlife::flee::refuge(&engine.world, &engine.resources, (10, 10), wolf, 8);
    // Then: the direct six-step room wins, not the two-tile straight-line decoy.
    assert_eq!(to, reachable);
}

#[test]
fn a_flight_reconsiders_its_destination_at_the_thirty_tick_boundary() {
    // Given: a flight still geometrically ahead of a wolf, with a newly available nearby roof.
    let mut engine = fixture();
    let person = engine.spawn_agent(10, 10);
    engine
        .world
        .insert_one(person, AgentState::Idle)
        .expect("state");
    let wolf = (16, 10);
    let wolf_id = wildlife::spawn_pack(&mut engine.resources, wolf, 1, 0)[0];
    let old = (2, 2);
    engine
        .world
        .insert_one(
            person,
            Fleeing {
                until: 60,
                from: wolf_id,
                to: old,
                since: 0,
            },
        )
        .expect("flight");
    roof(&mut engine, 1, (9, 10));
    engine.resources.current_tick = 30;
    // When: the required re-evaluation cadence elapses.
    FleeSystem.tick(&mut engine.world, &mut engine.resources);
    // Then: a new safe room replaces a stale away destination without rewriting the flight origin.
    let flight = engine
        .world
        .get::<&Fleeing>(person)
        .expect("flight continues");
    assert_eq!((flight.to, flight.since), ((9, 10), 0));
}

#[test]
fn a_pending_wolf_bite_is_cancelled_when_the_person_has_entered_a_roofed_room() {
    // Given: a bite started outside but resolves after the target moved under the roof.
    let mut engine = fixture();
    let person = engine.spawn_agent(10, 10);
    engine
        .world
        .insert_one(person, BodyHealth::new())
        .expect("health fixture");
    let id = engine.world.get::<&Agent>(person).expect("agent").id;
    roof(&mut engine, 1, (10, 10));
    let wolf = wildlife::spawn_pack(&mut engine.resources, (11, 10), 1, 0)[0];
    let key = (Fighter::Animal(wolf), Fighter::Person(id));
    engine.resources.colony.fights.insert(
        key,
        Fight {
            progress: 0,
            ticks: 1,
            damage: 6,
            hits_left: 0,
            quarrel: None,
        },
    );
    let before = engine.world.get::<&BodyHealth>(person).expect("health").hp;
    // When: the live fight resolver processes the pending adjacent bite.
    wildlife::fight::resolve_fights(&mut engine.world, &mut engine.resources);
    // Then: roof immunity covers pending attacks as well as new target selection.
    assert_eq!(
        engine.world.get::<&BodyHealth>(person).expect("health").hp,
        before
    );
    assert!(!engine.resources.colony.fights.contains_key(&key));
    assert_eq!(engine.resources.colony.wildlife.tallies.bites, 0);
    assert!(engine.resources.fatal_animal_attacks.is_empty());
}

#[test]
fn a_roofed_person_does_not_resume_a_stale_outdoor_flight() {
    // Given: an existing flight has entered another safe room before its old target.
    let mut engine = fixture();
    let person = engine.spawn_agent(10, 10);
    let wolf = wildlife::spawn_pack(&mut engine.resources, (16, 10), 1, 0)[0];
    engine
        .world
        .insert_one(person, AgentState::Idle)
        .expect("state");
    engine
        .world
        .insert_one(
            person,
            Fleeing {
                until: 60,
                from: wolf,
                to: (2, 2),
                since: 0,
            },
        )
        .expect("flight");
    roof(&mut engine, 1, (10, 10));
    engine.resources.current_tick = 30;
    // When: the threat remains nearby at a re-evaluation tick.
    FleeSystem.tick(&mut engine.world, &mut engine.resources);
    // Then: the destination stays at the roof, rather than leading its occupant outside.
    assert_eq!(
        engine.world.get::<&Fleeing>(person).expect("flight").to,
        (10, 10)
    );
    assert_eq!(
        engine.world.get::<&Position>(person).expect("position").x,
        10
    );
}
#[test]
fn an_unreachable_roof_is_not_a_refuge_even_when_it_is_close() {
    let mut engine = fixture();
    let room = (8, 10);
    roof(&mut engine, 1, room);
    for x in 7..=9 {
        for y in 9..=11 {
            if (x, y) != room {
                engine.resources.nav.set_cost((x, y), IMPASSABLE);
            }
        }
    }
    wildlife::spawn_pack(&mut engine.resources, (16, 10), 1, 0);
    assert!(engine.resources.nav.path((10, 10), room).is_none());
    assert_ne!(
        wildlife::flee::refuge(&engine.world, &engine.resources, (10, 10), (16, 10), 8),
        room
    );
}

#[test]
fn a_flight_keeps_its_destination_between_thirty_tick_rechecks() {
    let mut engine = fixture();
    let person = engine.spawn_agent(10, 10);
    engine
        .world
        .insert_one(person, AgentState::Idle)
        .expect("state");
    let wolf = wildlife::spawn_pack(&mut engine.resources, (16, 10), 1, 0)[0];
    let old = (2, 2);
    engine
        .world
        .insert_one(
            person,
            Fleeing {
                until: 60,
                from: wolf,
                to: old,
                since: 0,
            },
        )
        .expect("flight");
    roof(&mut engine, 1, (9, 10));
    engine.resources.current_tick = 27;
    FleeSystem.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine.world.get::<&Fleeing>(person).expect("flight").to,
        old
    );
}

#[test]
fn a_faster_person_can_take_the_same_roof_a_slower_person_cannot() {
    let mut engine = fixture();
    let room = (2, 10);
    roof(&mut engine, 1, room);
    wildlife::spawn_pack(&mut engine.resources, (20, 10), 1, 0);
    engine.resources.rules.agent.move_interval_ticks = 1;
    assert_eq!(
        wildlife::flee::refuge(&engine.world, &engine.resources, (12, 10), (20, 10), 8),
        room
    );
    engine.resources.rules.agent.move_interval_ticks = 3;
    assert_ne!(
        wildlife::flee::refuge(&engine.world, &engine.resources, (12, 10), (20, 10), 8),
        room
    );
}

#[test]
fn an_active_flight_rechecks_a_distant_threat_without_extending_its_expiry() {
    let mut engine = fixture();
    let person = engine.spawn_agent(10, 10);
    engine
        .world
        .insert_one(person, AgentState::Idle)
        .expect("state");
    let wolf = wildlife::spawn_pack(&mut engine.resources, (30, 10), 1, 0)[0];
    engine
        .world
        .insert_one(
            person,
            Fleeing {
                until: 45,
                from: wolf,
                to: (2, 2),
                since: 0,
            },
        )
        .expect("flight");
    roof(&mut engine, 1, (9, 10));
    engine.resources.current_tick = 30;
    FleeSystem.tick(&mut engine.world, &mut engine.resources);
    let flight = engine.world.get::<&Fleeing>(person).expect("flight");
    assert_eq!((flight.to, flight.until), ((9, 10), 45));
    drop(flight);
    engine.resources.current_tick = 45;
    FleeSystem.tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Fleeing>(person).is_err());
}

#[test]
fn an_arrival_time_tie_is_not_a_safe_roof() {
    let mut engine = fixture();
    let room = (6, 10);
    roof(&mut engine, 1, room);
    wildlife::spawn_pack(&mut engine.resources, (20, 10), 1, 0);
    // Six person steps take 18 ticks; seven wolf moves starting now also take 18.
    assert_ne!(
        wildlife::flee::refuge(&engine.world, &engine.resources, (12, 10), (20, 10), 8),
        room
    );
}

#[test]
fn a_wolf_in_a_disconnected_region_cannot_win_the_roof_race() {
    let mut engine = fixture();
    let room = (2, 10);
    roof(&mut engine, 1, room);
    for y in 0..32 {
        engine.resources.nav.set_cost((15, y), IMPASSABLE);
    }
    wildlife::spawn_pack(&mut engine.resources, (20, 10), 1, 0);
    assert!(engine.resources.nav.path((20, 10), room).is_none());
    assert_eq!(
        wildlife::flee::refuge(&engine.world, &engine.resources, (12, 10), (20, 10), 8),
        room
    );
}

#[test]
fn fatal_animal_events_exclude_nonfatal_bites_and_simultaneous_human_kills() {
    let mut engine = fixture();
    engine.resources.current_tick = 123;
    let wolf = wildlife::spawn_pack(&mut engine.resources, (11, 10), 1, 0)[0];
    let wolf_kind = engine.resources.colony.wildlife.animals[&wolf].kind;
    let bitten = engine.spawn_agent(10, 10);
    let killed = engine.spawn_agent(11, 11);
    let human = engine.spawn_agent(5, 5);
    let human_victim = engine.spawn_agent(6, 5);
    for entity in [bitten, killed, human, human_victim] {
        engine
            .world
            .insert_one(entity, BodyHealth::new())
            .expect("health");
    }
    engine
        .world
        .get::<&mut BodyHealth>(killed)
        .expect("health")
        .hp = 1.0;
    engine
        .world
        .get::<&mut BodyHealth>(human_victim)
        .expect("health")
        .hp = 1.0;
    let id = |entity| engine.world.get::<&Agent>(entity).expect("agent").id;
    let (bitten_id, killed_id, human_id, human_victim_id) =
        (id(bitten), id(killed), id(human), id(human_victim));
    for target in [bitten_id, killed_id] {
        engine.resources.colony.fights.insert(
            (Fighter::Animal(wolf), Fighter::Person(target)),
            Fight {
                progress: 0,
                ticks: 1,
                damage: 6,
                hits_left: 1,
                quarrel: None,
            },
        );
    }
    engine.resources.colony.fights.insert(
        (Fighter::Person(human_id), Fighter::Person(human_victim_id)),
        Fight {
            progress: 0,
            ticks: 1,
            damage: 6,
            hits_left: 1,
            quarrel: Some(0),
        },
    );
    wildlife::fight::resolve_fights(&mut engine.world, &mut engine.resources);
    assert!(engine.world.contains(bitten));
    assert!(!engine.world.contains(killed));
    assert!(!engine.world.contains(human_victim));
    assert_eq!(engine.resources.tallies.deaths_combat, 2);
    assert_eq!(engine.resources.fatal_animal_attacks.len(), 1);
    let event = &engine.resources.fatal_animal_attacks[0];
    assert_eq!(
        (event.tick, event.victim, event.attacker, event.kind),
        (123, killed_id, wolf, wolf_kind)
    );
}
