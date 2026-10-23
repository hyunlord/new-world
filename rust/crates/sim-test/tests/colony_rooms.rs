//! M3 Part 3: rooms and roles (ADR 0023) — an enclosure is a room with or
//! without its door, a dug pocket is a room, each role is recognized, the
//! player's override sticks, and the roles change behaviour: beds in a room no
//! one can reach are not given out, roofed sleepers keep warm, hauls prefer
//! the storehouse.

mod colony_support;

use colony_support::{
    adults, bedroll, detect, drop_items, job_world_m2, open_area, run_until, wall, walled_box,
    worker,
};
use sim_core::colony::materials::OAK;
use sim_core::colony::{Command, ItemKind, ItemSpec, Occupant, RejectReason};
use sim_core::components::{AgentState, BuildingKind, Cold, Position, ResourceKind, TargetKind};
use sim_core::rooms::RoomRole;
use sim_core::rules::ColonyRule;
use sim_core::TerrainType;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::commands::submit;
use sim_systems::runtime::nav;
use sim_systems::runtime::rooms::room_at;

#[test]
fn an_enclosure_is_a_room_with_or_without_its_door_but_its_beds_need_the_door() {
    for door in [true, false] {
        let mut engine = sim_test::colony_engine(4);
        for _ in 0..10 {
            engine.tick();
        }
        let at = open_area(&engine, 7, 5);
        let inside = walled_box(&mut engine, at, 7, 5, door);
        for t in inside.iter().take(4) {
            bedroll(&mut engine, *t);
        }
        detect(&mut engine);
        let room = room_at(&engine.resources, inside[0])
            .expect("a room")
            .clone();
        assert_eq!(room.tiles.len(), 15);
        assert_eq!(room.doors.len(), usize::from(door));
        assert!(room.roofed && !room.dug);
        assert_eq!(room.role, RoomRole::Bedroom);
        assert_eq!(
            sim_systems::runtime::rooms::beds_in(&engine.resources, &room),
            4
        );
        // Beds are given out at 20:00 or when the beds change: a day covers both.
        for _ in 0..1_440 {
            engine.tick();
        }
        let given = engine
            .resources
            .bed_of
            .values()
            .filter(|bed| room.tiles.contains(bed))
            .count();
        if door {
            assert_eq!(given, 4, "every bed of the reachable bedroom is given out");
        } else {
            assert_eq!(
                given, 0,
                "no one can reach the beds of a room without a door"
            );
        }
        let formed = engine
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == sim_core::chronicle::ChronicleKind::RoomFormed)
            .count();
        assert_eq!(formed, 1, "RoomFormed once");
    }
}

#[test]
fn a_dug_pocket_is_a_room_with_dug_set() {
    let mut engine = job_world_m2(24, 24);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    for y in 6..=18u32 {
        for x in 6..=18u32 {
            let i = engine.resources.tile_grid.idx(x, y);
            engine.resources.tile_grid.terrain_type[i] = TerrainType::Hill;
        }
    }
    nav::ensure(&mut engine.resources);
    engine.resources.nav.install(nav::build(&engine.resources));
    // A 3 × 3 pocket with a one-tile mouth on its west side.
    for (x, y) in (11..=13)
        .flat_map(|y| (11..=13).map(move |x| (x, y)))
        .chain([(10, 12), (9, 12), (8, 12), (7, 12), (6, 12)])
    {
        assert!(sim_systems::runtime::jobs::space::dig(
            &mut engine.resources,
            (x, y)
        ));
    }
    detect(&mut engine);
    // The corridor joins the pocket to open ground at (5, 12): no room yet...
    assert!(room_at(&engine.resources, (12, 12)).is_none());
    // ... a door across the corridor closes it.
    wall(&mut engine, (8, 12), true);
    detect(&mut engine);
    let room = room_at(&engine.resources, (12, 12))
        .expect("the pocket")
        .clone();
    assert!(room.dug && room.roofed);
    assert_eq!(
        room.tiles.len(),
        9 + 2,
        "the pocket and the corridor inside the door"
    );
}

#[test]
fn each_role_is_recognized_and_an_override_sticks() {
    let mut engine = job_world_m2(40, 30);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    nav::ensure(&mut engine.resources);
    let bedroom = walled_box(&mut engine, (2, 2), 5, 5, true);
    let store = walled_box(&mut engine, (10, 2), 5, 5, true);
    let shop = walled_box(&mut engine, (18, 2), 5, 5, true);
    let hall = walled_box(&mut engine, (26, 2), 5, 5, true);
    let empty = walled_box(&mut engine, (2, 12), 5, 5, true);
    bedroll(&mut engine, bedroom[0]);
    let (x0, y0) = store[0];
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0,
                y0,
                x1: x0 + 2,
                y1: y0 + 1
            }
        )
        .accepted
    );
    engine
        .resources
        .colony
        .occupied
        .insert(shop[4], Occupant::Workshop(1));
    engine
        .resources
        .building_kinds
        .insert(hall[4], BuildingKind::Hearth);
    detect(&mut engine);
    let role = |e: &SimEngine, t: (u32, u32)| room_at(&e.resources, t).map(|r| r.role);
    assert_eq!(role(&engine, bedroom[0]), Some(RoomRole::Bedroom));
    assert_eq!(
        role(&engine, store[0]),
        Some(RoomRole::Storehouse),
        "6 of 9 cells"
    );
    assert_eq!(role(&engine, shop[0]), Some(RoomRole::Workshop));
    assert_eq!(role(&engine, hall[0]), Some(RoomRole::Hall));
    assert_eq!(role(&engine, empty[0]), Some(RoomRole::Unassigned));
    // A bed in the storehouse does not make it a bedroom (it has stockpile cells).
    bedroll(&mut engine, store[8]);
    sim_systems::runtime::rooms::assign_roles(&engine.world, &mut engine.resources);
    assert_eq!(role(&engine, store[0]), Some(RoomRole::Storehouse));
    // Override: the empty room becomes a bedroom by command and stays one
    // when its wall moves (75 % of the old tiles shared: the same id).
    let id = engine.resources.colony.room_of_tile[&empty[0]];
    let set =
        |engine: &mut SimEngine, role| submit(engine, Command::SetRoomRole { room: id, role });
    assert!(set(&mut engine, Some(RoomRole::Bedroom)).accepted);
    assert_eq!(
        set(&mut engine, Some(RoomRole::Bedroom)).reason,
        Some(RejectReason::Nothing)
    );
    assert_eq!(role(&engine, empty[0]), Some(RoomRole::Bedroom));
    // Move the east wall (x = 6) one tile east: the room grows 9 → 12 tiles.
    for y in 12..=16u32 {
        let g = &mut engine.resources.tile_grid;
        let i = g.idx(6, y);
        g.wall_material[i] = None;
        nav::refresh(&mut engine.resources, (6, y));
        wall(&mut engine, (7, y), false);
    }
    wall(&mut engine, (6, 12), false);
    wall(&mut engine, (6, 16), false);
    let g = &mut engine.resources.tile_grid;
    let (a, b) = (g.idx(6, 12), g.idx(6, 16));
    g.wall_material[a] = Some(OAK);
    g.wall_material[b] = Some(OAK);
    detect(&mut engine);
    assert_eq!(
        engine.resources.colony.room_of_tile.get(&empty[0]),
        Some(&id)
    );
    assert_eq!(
        role(&engine, empty[0]),
        Some(RoomRole::Bedroom),
        "the override followed the id"
    );
    assert!(set(&mut engine, None).accepted, "back to automatic");
    assert_eq!(role(&engine, empty[0]), Some(RoomRole::Unassigned));
    assert_eq!(
        submit(
            &mut engine,
            Command::SetRoomRole {
                room: 999,
                role: None
            }
        )
        .reason,
        Some(RejectReason::Unknown)
    );
}

#[test]
fn sleepers_under_a_roof_take_no_night_cold() {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 5, 5);
    let inside = walled_box(&mut engine, at, 5, 5, true);
    detect(&mut engine);
    let people = adults(&engine);
    let ((_, a), (_, b)) = (people[0], people[1]);
    let asleep = AgentState::Consuming {
        target: TargetKind::Sleep,
    };
    let outside = (at.0 + 2, at.1 + 6);
    for (e, t) in [(a, inside[4]), (b, outside)] {
        let _ = engine.world.insert_one(e, asleep);
        if let Ok(mut p) = engine.world.get::<&mut Position>(e) {
            p.x = t.0;
            p.y = t.1;
        }
    }
    // The fire is out: no wood in any settlement.
    let ids: Vec<_> = engine.resources.settlements.keys().copied().collect();
    for sid in ids {
        if let Some(s) = engine.resources.settlements.get_mut(&sid) {
            s.withdraw(ResourceKind::Wood, u32::MAX);
        }
    }
    // A winter midnight (minute 0 = tick 960 + k·1440 with the 480 offset).
    let rule = engine
        .resources
        .rules
        .society
        .firewood
        .expect("firewood rule");
    let night = (0..64u64)
        .map(|k| 960 + k * 1_440)
        .find(|t| {
            engine.resources.current_tick = *t;
            sim_systems::runtime::settlement::firewood::night_season(&engine.resources, &rule) == 3
        })
        .expect("a winter night within 64 days");
    engine.resources.current_tick = night;
    let mut fire = sim_systems::runtime::settlement::FireSystem::new();
    fire.tick(&mut engine.world, &mut engine.resources);
    assert!(
        engine.world.get::<&Cold>(b).is_ok(),
        "the sleeper outside is cold"
    );
    assert!(
        engine.world.get::<&Cold>(a).is_err(),
        "the sleeper under the roof is not"
    );
}

#[test]
fn a_hauled_log_goes_into_the_storehouse_at_equal_distance() {
    let mut engine = job_world_m2(40, 30);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    nav::ensure(&mut engine.resources);
    // A 3 × 3 storehouse (walls 13..=17 × 13..=17, door south) with its 9 cells zoned,
    // and one open-air cell at the same distance from the log, nearer in (y, x).
    let inside = walled_box(&mut engine, (13, 13), 5, 5, true);
    for t in &inside {
        let r = submit(
            &mut engine,
            Command::AddStockpile {
                x0: t.0,
                y0: t.1,
                x1: t.0,
                y1: t.1,
            },
        );
        assert!(r.accepted);
    }
    detect(&mut engine);
    assert_eq!(
        room_at(&engine.resources, inside[0]).map(|r| r.role),
        Some(RoomRole::Storehouse)
    );
    // The log at (15, 22); the inner cell (15, 16) is 6 away, the open cell (21, 22) too.
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 21,
                y0: 22,
                x1: 21,
                y1: 22
            }
        )
        .accepted
    );
    drop_items(&mut engine, (15, 22), ItemSpec::raw(ItemKind::Log, None), 3);
    worker(&mut engine, 15, 25);
    let used = run_until(&mut engine, 600, |e| {
        !e.resources.colony.ground_items.contains_key(&(15, 22))
    });
    assert!(used < 600, "not hauled");
    run_until(&mut engine, 600, |e| {
        e.world
            .query::<&sim_core::colony::Carry>()
            .iter()
            .next()
            .is_none()
    });
    let c = &engine.resources.colony;
    assert!(
        !c.ground_items.contains_key(&(21, 22)),
        "not to the open-air cell"
    );
    let indoors: u16 = inside
        .iter()
        .filter_map(|t| c.ground_items.get(t))
        .map(|s| s.count)
        .sum();
    assert_eq!(indoors, 3, "the logs went under the roof");
}
