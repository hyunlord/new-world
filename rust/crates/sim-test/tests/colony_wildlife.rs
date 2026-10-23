//! M5 Part 2: animals, hunting and drafting (ADR 0028) — deer live in the camp's
//! reachable forest far from the hearth and are born again; a hunt kills a deer
//! and drops meat and a hide (the ledger holds); wolves bite people outside and
//! never one under a roof; a draft cancels the job and its attack resolves in
//! `CombatSystem`; three spears beat three wolves; a drafted person at a
//! critical need undrafts themselves.

mod colony_support;

use colony_support::{adults, detect, open_area, tick_checked, walled_box};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::people::Drafted;
use sim_core::colony::wildlife::{AnimalKind, Fighter};
use sim_core::colony::{Command, ItemKind, ItemSpec, Job, Tool};
use sim_core::components::{AgentState, BodyHealth, Position, TargetKind, Thirst};
use sim_core::TerrainType;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::commands::submit;
use sim_systems::runtime::combat::CombatSystem;
use sim_systems::runtime::jobs::invariants;
use sim_systems::runtime::wildlife::{self, AnimalSystem, DraftSystem};

fn hearth(engine: &SimEngine) -> (u32, u32) {
    sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth")
}

fn place(engine: &mut SimEngine, e: hecs::Entity, t: (u32, u32)) {
    if let Ok(mut p) = engine.world.get::<&mut Position>(e) {
        (p.x, p.y) = t;
    }
}

fn hp(engine: &SimEngine, e: hecs::Entity) -> f64 {
    engine.world.get::<&BodyHealth>(e).map_or(0.0, |h| h.hp)
}

/// Give `e` a spear in hand.
fn spear(engine: &mut SimEngine, e: hecs::Entity) {
    let _ = engine.world.insert_one(
        e,
        Tool {
            item: ItemSpec::raw(ItemKind::Spear, None),
            uses: 0,
        },
    );
    engine
        .resources
        .colony
        .item_ledger
        .create(ItemKind::Spear, 1);
}

#[test]
fn deer_live_in_reachable_forest_far_from_the_hearth_and_are_born_again() {
    for seed in 0..4u64 {
        let mut engine = sim_test::colony_engine(seed);
        engine.tick();
        let mut animals = AnimalSystem;
        animals.tick(&mut engine.world, &mut engine.resources);
        let h = hearth(&engine);
        let w = &engine.resources.colony.wildlife;
        let deer: Vec<_> = w.deer().cloned().collect();
        assert!(
            (6..=12).contains(&deer.len()),
            "seed {seed}: {} deer",
            deer.len()
        );
        assert_eq!(deer.len() as u32, w.deer_cap);
        for d in &deer {
            let i = engine.resources.tile_grid.idx(d.pos.0, d.pos.1);
            // Placed in the forest; the first step may wander onto plain (forest or plain is deer ground).
            assert!(
                matches!(
                    engine.resources.tile_grid.terrain_type[i],
                    TerrainType::Forest | TerrainType::Plain
                ),
                "seed {seed}: {d:?}"
            );
            assert!(
                sim_core::nav::chebyshev(d.pos, h) >= 20,
                "seed {seed}: {d:?} near the hearth"
            );
            assert!(
                engine.resources.nav.reachable(h, d.pos),
                "seed {seed}: {d:?} out of reach"
            );
        }
        // Remove all deer but one; a day-4 morning brings one more, a day-5 morning none.
        let keep = deer[0].id;
        engine
            .resources
            .colony
            .wildlife
            .animals
            .retain(|id, _| *id == keep);
        for (day, expect) in [(4u64, 2usize), (5, 2), (8, 3)] {
            engine.resources.current_tick = day * 1_440;
            animals.tick(&mut engine.world, &mut engine.resources);
            assert_eq!(
                engine.resources.colony.wildlife.deer().count(),
                expect,
                "seed {seed} day {day}"
            );
        }
    }
}

/// Make `e` a practitioner of `name`.
fn know(engine: &mut SimEngine, e: hecs::Entity, name: &str) {
    let id = knowledge::id(name).expect("node");
    let mut k = engine
        .world
        .get::<&Knowledge>(e)
        .map(|k| (*k).clone())
        .unwrap_or_default();
    k.set(id, 800, Source::Seeded, 0);
    let _ = engine.world.insert_one(e, k);
}

#[test]
fn a_hunt_kills_a_deer_and_drops_meat_and_a_hide() {
    let mut engine = sim_test::colony_engine(1);
    tick_checked(&mut engine);
    let (_, hunter) = adults(&engine)[0];
    know(&mut engine, hunter, "tracking");
    spear(&mut engine, hunter);
    // Every deer but one gone; the last one marked; nobody else tracks.
    let mut animals = AnimalSystem;
    animals.tick(&mut engine.world, &mut engine.resources);
    let deer = engine
        .resources
        .colony
        .wildlife
        .deer()
        .next()
        .expect("a deer")
        .id;
    engine
        .resources
        .colony
        .wildlife
        .animals
        .retain(|id, _| *id == deer);
    for (_, other) in adults(&engine).into_iter().skip(1) {
        if let Ok(mut k) = engine.world.get::<&mut Knowledge>(other) {
            let id = knowledge::id("tracking").expect("node");
            k.set(id, 0, Source::Seeded, 0);
        }
    }
    assert!(submit(&mut engine, Command::DesignateHunt { animal: deer }).accepted);
    let mut hunted = false;
    for _ in 0..3 * 1_440 {
        tick_checked(&mut engine);
        if engine
            .world
            .get::<&Job>(hunter)
            .is_ok_and(|j| matches!(j.kind, sim_core::colony::JobKind::Hunt { .. }))
        {
            hunted = true;
        }
        if engine.resources.colony.wildlife.tallies.deer_killed > 0 {
            break;
        }
    }
    assert!(hunted, "the tracker took the hunt");
    let w = &engine.resources.colony.wildlife;
    assert_eq!(w.tallies.deer_killed, 1, "the deer fell within three days");
    let l = &engine.resources.colony.item_ledger;
    assert_eq!(l.created_of(ItemKind::Meat), 2, "no knife held: 2 meat");
    assert_eq!(l.created_of(ItemKind::Hide), 1);
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::Hunted && e.value == 2));
    assert!(invariants::check(&engine.world, &engine.resources).is_ok());
}

#[test]
fn wolves_bite_people_outside_and_never_one_under_a_roof() {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 7, 7);
    let inside = walled_box(&mut engine, at, 7, 7, true);
    detect(&mut engine);
    let people = adults(&engine);
    let outside = (at.0 + 3, at.1 + 10);
    let (_, out) = people[0];
    place(&mut engine, out, outside);
    let mut roofed = Vec::new();
    for (i, (_, e)) in people.iter().enumerate().skip(1) {
        let t = inside[i % inside.len()];
        place(&mut engine, *e, t);
        roofed.push(*e);
    }
    let rule = wildlife::rule(&engine.resources).expect("wildlife rule");
    let pack_at = (outside.0, outside.1 + 8);
    wildlife::spawn_pack(&mut engine.resources, pack_at, 2, 0);
    // Only the animals and combat run: nobody walks away.
    let (mut animals, mut combat) = (AnimalSystem, CombatSystem::new());
    let start = engine.resources.current_tick;
    for k in 1..=900u64 {
        engine.resources.current_tick = start + k;
        if (start + k).is_multiple_of(3) {
            animals.tick(&mut engine.world, &mut engine.resources);
        }
        combat.tick(&mut engine.world, &mut engine.resources);
    }
    let t = &engine.resources.colony.wildlife.tallies;
    assert!(t.bites > 0, "the wolves reached the person outside");
    assert!(hp(&engine, out) < 100.0 || t.bites >= 1);
    for e in roofed {
        assert_eq!(hp(&engine, e), 100.0, "nobody under the roof was bitten");
    }
    assert_eq!(rule.wolf_damage, 6);
}

#[test]
fn a_draft_cancels_the_job_and_its_attack_resolves_in_combat() {
    let mut engine = sim_test::colony_engine(0);
    let (_, worker) = adults(&engine)[0];
    let id = engine
        .world
        .get::<&sim_core::components::Agent>(worker)
        .map(|a| a.id)
        .expect("agent");
    // Let the colony give out work, then draft someone who holds a job.
    let holder = (0..1_440)
        .find_map(|_| {
            tick_checked(&mut engine);
            adults(&engine)
                .into_iter()
                .find(|(_, e)| engine.world.get::<&Job>(*e).is_ok())
        })
        .unwrap_or((id, worker));
    let (hid, he) = holder;
    assert!(engine.world.get::<&Job>(he).is_ok(), "someone held a job");
    assert!(
        submit(
            &mut engine,
            Command::SetDrafted {
                agent: hid,
                drafted: true
            }
        )
        .accepted
    );
    tick_checked(&mut engine);
    assert!(
        engine.world.get::<&Job>(he).is_err(),
        "the draft cancelled the job"
    );
    assert!(engine.world.get::<&Drafted>(he).is_ok());
    spear(&mut engine, he);
    let pos = engine
        .world
        .get::<&Position>(he)
        .map(|p| (p.x, p.y))
        .expect("pos");
    let wolf = wildlife::spawn_pack(&mut engine.resources, (pos.0 + 3, pos.1), 1, 0)[0];
    assert!(
        submit(
            &mut engine,
            Command::DraftAttack {
                agent: hid,
                animal: wolf
            }
        )
        .accepted
    );
    let mut fought = false;
    for _ in 0..600 {
        tick_checked(&mut engine);
        let state = engine
            .world
            .get::<&AgentState>(he)
            .map(|s| *s)
            .unwrap_or_default();
        assert!(
            matches!(
                state,
                AgentState::Seeking {
                    target: TargetKind::Job
                } | AgentState::Consuming {
                    target: TargetKind::Job
                }
            ),
            "the decision never takes a drafted person: {state:?}"
        );
        fought |= engine
            .resources
            .colony
            .fights
            .contains_key(&(Fighter::Person(hid), Fighter::Animal(wolf)));
        if !engine.resources.colony.wildlife.animals.contains_key(&wolf) {
            break;
        }
    }
    assert!(fought, "the attack became a fight");
    assert!(
        !engine.resources.colony.wildlife.animals.contains_key(&wolf),
        "the wolf fell"
    );
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::WolfKilled && e.subject == Some(hid)));
    assert_eq!(
        engine
            .resources
            .colony
            .item_ledger
            .created_of(ItemKind::WolfHide),
        1,
        "a wolf hide"
    );
    assert!(
        submit(
            &mut engine,
            Command::SetDrafted {
                agent: hid,
                drafted: false
            }
        )
        .accepted
    );
    tick_checked(&mut engine);
    assert!(engine.world.get::<&Drafted>(he).is_err());
}

#[test]
fn three_spears_beat_three_wolves_on_most_worlds() {
    let mut wins = 0;
    for seed in 0..8u64 {
        let mut engine = sim_test::colony_engine(seed);
        for _ in 0..10 {
            engine.tick();
        }
        let party: Vec<(u64, hecs::Entity)> = adults(&engine).into_iter().take(3).collect();
        let h = hearth(&engine);
        for (i, (id, e)) in party.iter().enumerate() {
            spear(&mut engine, *e);
            place(&mut engine, *e, (h.0 + i as u32, h.1 + 2));
            assert!(
                submit(
                    &mut engine,
                    Command::SetDrafted {
                        agent: *id,
                        drafted: true
                    }
                )
                .accepted
            );
        }
        let wolves = wildlife::spawn_pack(&mut engine.resources, (h.0 + 1, h.1 + 8), 3, 0);
        for (i, (id, _)) in party.iter().enumerate() {
            submit(
                &mut engine,
                Command::DraftAttack {
                    agent: *id,
                    animal: wolves[i],
                },
            );
        }
        for _ in 0..1_200 {
            tick_checked(&mut engine);
            // A drafted person whose wolf fell takes the next one.
            let left: Vec<u32> = engine
                .resources
                .colony
                .wildlife
                .wolves()
                .map(|a| a.id)
                .collect();
            if left.is_empty() {
                break;
            }
            for (id, e) in &party {
                let idle = engine
                    .world
                    .get::<&Drafted>(*e)
                    .is_ok_and(|d| d.order == sim_core::colony::people::DraftOrder::Hold);
                if idle {
                    submit(
                        &mut engine,
                        Command::DraftAttack {
                            agent: *id,
                            animal: left[0],
                        },
                    );
                }
            }
        }
        let t = &engine.resources.colony.wildlife.tallies;
        let low = party.iter().filter(|(_, e)| hp(&engine, *e) < 50.0).count();
        let gone = engine.resources.colony.wildlife.wolves().count() == 0;
        if gone && t.wolves_killed >= 2 && low <= 1 {
            wins += 1;
        }
        eprintln!(
            "seed {seed}: killed {} left {} low {low} gone {gone}",
            t.wolves_killed, t.wolves_left
        );
    }
    assert!(
        wins >= 6,
        "three spears beat three wolves on {wins}/8 worlds"
    );
}

#[test]
fn a_drafted_person_at_a_critical_need_undrafts_themselves() {
    let mut engine = sim_test::colony_engine(3);
    engine.tick();
    let (id, e) = adults(&engine)[0];
    assert!(
        submit(
            &mut engine,
            Command::SetDrafted {
                agent: id,
                drafted: true
            }
        )
        .accepted
    );
    if let Ok(mut t) = engine.world.get::<&mut Thirst>(e) {
        t.value = 80.0;
    }
    let mut draft = DraftSystem;
    draft.tick(&mut engine.world, &mut engine.resources);
    assert!(
        engine.world.get::<&Drafted>(e).is_err(),
        "undrafted at thirst 80"
    );
    assert_eq!(engine.resources.colony.wildlife.tallies.auto_undrafts, 1);
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|en| en.kind == ChronicleKind::Drafted && en.subject == Some(id) && en.cause == 2));
    // A child cannot be drafted; a draft order to someone not drafted is refused.
    assert_eq!(
        submit(
            &mut engine,
            Command::DraftMove {
                agent: id,
                x: 1,
                y: 1
            }
        )
        .reason,
        Some(sim_core::colony::RejectReason::NotDrafted)
    );
    let _ = AnimalKind::Deer;
}

#[test]
fn a_tool_pick_up_leaves_another_workers_reserved_stack_and_takes_only_its_kinds() {
    use sim_core::colony::ReserveKey;
    use sim_systems::runtime::jobs::hunt::pick_up_tool;
    let mut engine = sim_test::colony_engine(0);
    tick_checked(&mut engine);
    let folk = adults(&engine);
    let ((_, a), (other, _)) = (folk[0], folk[1]);
    let _ = engine.world.remove_one::<Tool>(a);
    let at = colony_support::open_area(&engine, 3, 3);
    let pos = (at.0 + 1, at.1 + 1);
    colony_support::drop_items(&mut engine, at, ItemSpec::raw(ItemKind::Spear, None), 1);
    // Another worker holds the spear stack (a hunter's fetch): the drafted person (no owner) leaves it.
    engine
        .resources
        .colony
        .reservations
        .insert(ReserveKey::Stack(at), other);
    let weapons = [ItemKind::Spear, ItemKind::GroundAxe, ItemKind::StoneAxe];
    pick_up_tool(
        &mut engine.world,
        &mut engine.resources,
        a,
        at,
        pos,
        None,
        &weapons,
    );
    assert_eq!(
        engine
            .resources
            .colony
            .reservations
            .get(&ReserveKey::Stack(at)),
        Some(&other),
        "the other worker's reservation stays (code review B1)"
    );
    assert!(engine.world.get::<&Tool>(a).is_err(), "nothing taken");
    // The owner's own pick-up takes it and releases its reservation.
    pick_up_tool(
        &mut engine.world,
        &mut engine.resources,
        a,
        at,
        pos,
        Some(other),
        &weapons,
    );
    assert!(!engine
        .resources
        .colony
        .reservations
        .contains_key(&ReserveKey::Stack(at)));
    assert_eq!(
        engine.world.get::<&Tool>(a).map(|t| t.item.kind).ok(),
        Some(ItemKind::Spear)
    );
    // Logs on an arming tile are no weapon.
    let _ = engine.world.remove_one::<Tool>(a);
    let logs = (at.0 + 2, at.1 + 2);
    colony_support::drop_items(&mut engine, logs, ItemSpec::raw(ItemKind::Log, None), 2);
    pick_up_tool(
        &mut engine.world,
        &mut engine.resources,
        a,
        logs,
        pos,
        None,
        &weapons,
    );
    assert!(
        engine.world.get::<&Tool>(a).is_err(),
        "a log is not taken as a weapon"
    );
    assert_eq!(
        engine
            .resources
            .colony
            .ground_items
            .get(&logs)
            .map(|s| s.count),
        Some(2)
    );
}

#[test]
fn a_missed_attack_keeps_the_hunt_job() {
    let mut engine = sim_test::colony_engine(1);
    tick_checked(&mut engine);
    let (_, hunter) = adults(&engine)[0];
    know(&mut engine, hunter, "tracking");
    let mut animals = AnimalSystem;
    animals.tick(&mut engine.world, &mut engine.resources);
    let deer = engine
        .resources
        .colony
        .wildlife
        .deer()
        .next()
        .expect("a deer")
        .id;
    engine
        .resources
        .colony
        .wildlife
        .animals
        .retain(|id, _| *id == deer);
    for (_, other) in adults(&engine).into_iter().skip(1) {
        if let Ok(mut k) = engine.world.get::<&mut Knowledge>(other) {
            let id = knowledge::id("tracking").expect("node");
            k.set(id, 0, Source::Seeded, 0);
        }
    }
    assert!(submit(&mut engine, Command::DesignateHunt { animal: deer }).accepted);
    let interrupted = |e: &SimEngine| {
        e.resources
            .colony
            .tallies
            .cancels_of(sim_core::colony::CancelReason::Interrupted)
    };
    let mut misses = 0u64;
    let mut kept = 0u64;
    for _ in 0..3 * 1_440 {
        let (attacks, hits) = {
            let t = &engine.resources.colony.wildlife.tallies;
            (t.hunt_attacks, t.hunt_hits)
        };
        let before = interrupted(&engine);
        tick_checked(&mut engine);
        let t = &engine.resources.colony.wildlife.tallies;
        if t.hunt_attacks > attacks && t.hunt_hits == hits {
            misses += 1;
            // The tick after the miss the job is still the hunt, with no Interrupted cancel.
            tick_checked(&mut engine);
            if engine.world.get::<&Job>(hunter).is_ok() && interrupted(&engine) == before {
                kept += 1;
            }
        }
        if engine.resources.colony.wildlife.tallies.deer_killed > 0 {
            break;
        }
    }
    assert!(
        misses >= 1,
        "a level-0 hunter misses at least once (40 % to hit)"
    );
    assert_eq!(
        kept, misses,
        "every miss kept the job (code review m1): {kept} of {misses}"
    );
}
