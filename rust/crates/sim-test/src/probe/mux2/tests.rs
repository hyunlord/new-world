//! M-UX2 observer counterexamples and measurement-purity checks.
use super::*;
use sim_core::material::MaterialId;
use sim_core::rooms::RoomRole;

fn house() -> (SimEngine, Room) {
    let mut engine = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    let room = Room {
        id: 1,
        tiles: BTreeSet::from([(2, 2), (3, 2)]),
        doors: BTreeSet::from([(2, 1)]),
        roofed: true,
        role: RoomRole::Bedroom,
        dug: false,
    };
    for (x, y) in [(1, 2), (4, 2), (2, 1), (3, 1), (2, 3), (3, 3)] {
        let index = engine.resources.tile_grid.idx(x, y);
        engine.resources.tile_grid.wall_material[index] = Some(MaterialId::from_str_hash("oak"));
    }
    let index = engine.resources.tile_grid.idx(2, 1);
    engine.resources.tile_grid.set_door(index, true);
    (engine, room)
}

#[test]
fn walled_house_rejects_natural_perimeter_open_gap_and_fake_door() {
    let (mut engine, room) = house();
    assert!(walled_house(&engine.resources, &room));
    let index = engine.resources.tile_grid.idx(1, 2);
    engine.resources.tile_grid.wall_material[index] = None;
    engine.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Hill;
    assert!(
        !walled_house(&engine.resources, &room),
        "roofed hill pocket is not a walled house"
    );
    engine.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Plain;
    assert!(!walled_house(&engine.resources, &room), "open gap");
    engine.resources.tile_grid.wall_material[index] = Some(MaterialId::from_str_hash("oak"));
    let door = engine.resources.tile_grid.idx(2, 1);
    engine.resources.tile_grid.set_door(door, false);
    assert!(
        !walled_house(&engine.resources, &room),
        "cached room doors do not create a physical door"
    );
}

#[test]
fn observer_is_read_only_and_never_counts_generic_combat_as_wolf_death() {
    let mut engine = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    engine.tick();
    engine.resources.tallies.deaths_combat = 2;
    engine
        .resources
        .fatal_animal_attacks
        .push(sim_engine::FatalAnimalAttack {
            tick: 0,
            victim: 17,
            attacker: 4,
            kind: sim_core::colony::wildlife::AnimalKind::Wolf,
        });
    let encoded = sim_systems::save::encode(&engine.resources).expect("resources");
    let before = crate::probe::fingerprint(&engine);
    let mut watch = Mux2Watch::default();
    watch.after_tick(&engine);
    let report = watch.report(&engine);
    assert_eq!(report["wolf_deaths"], 1);
    assert_eq!(report["combat_deaths"], 2);
    assert_eq!(report["observer_integrity_errors"], json!([]));
    assert_eq!(before, crate::probe::fingerprint(&engine));
    assert_eq!(
        encoded,
        sim_systems::save::encode(&engine.resources).expect("unchanged")
    );
    engine.resources.fatal_animal_attacks.clear();
    assert_eq!(
        encoded,
        sim_systems::save::encode(&engine.resources).expect("transient excluded from save")
    );
    engine.tick();
    watch.after_tick(&engine);
    assert_eq!(watch.report(&engine)["wolf_deaths"], 1);
}

#[test]
fn transient_events_clear_on_tick_and_duplicate_observation_is_invalid() {
    let mut engine = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    engine
        .resources
        .fatal_animal_attacks
        .push(sim_engine::FatalAnimalAttack {
            tick: 0,
            victim: 17,
            attacker: 4,
            kind: sim_core::colony::wildlife::AnimalKind::Wolf,
        });
    engine.tick();
    assert!(engine.resources.fatal_animal_attacks.is_empty());
    let mut watch = Mux2Watch::default();
    watch.after_tick(&engine);
    watch.after_tick(&engine);
    assert!(!watch.errors.is_empty());
}

#[test]
fn zero_population_never_vacuously_has_beds_for_all() {
    let (mut engine, room) = house();
    engine.resources.colony.rooms.insert(room.id, room);
    engine
        .resources
        .colony
        .beds
        .insert((2, 2), MaterialId::from_str_hash("oak"));
    engine.tick();
    let mut watch = Mux2Watch::default();
    watch.after_tick(&engine);
    assert_eq!(watch.walled_beds_tick, None);
}
