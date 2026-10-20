mod pass17_family_support;
use pass17_family_support::*;
use sim_core::components::{
    Agent, Child, Elder, FamilyEvent, FamilyObservations, Kin, Lineage, RelationshipKey,
    RelationshipState, Sex,
};
use sim_engine::RuntimeSystem;
use sim_systems::runtime::agent::{households::households, PairingSystem};

#[test]
fn pairing_uses_clock_offset_familiarity_ties_and_female_order() {
    // Given two women and two men with a familiarity tie inserted in reverse order.
    let mut e = fixture();
    let people = [
        person(&mut e, true),
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, false),
    ];
    let ids = people.map(|entity| id(&e, entity));
    for female in [ids[1], ids[0]] {
        for male in [ids[3], ids[2]] {
            e.resources.relationships.insert(
                RelationshipKey::new(female, male),
                RelationshipState {
                    familiarity: 0.3,
                    hostility: 0.0,
                },
            );
        }
    }
    e.resources.rules.clock_offset_ticks = 480;
    e.resources.current_tick = 719;
    PairingSystem.tick(&mut e.world, &mut e.resources);
    assert!(e
        .world
        .get::<&Kin>(people[0])
        .expect("kin")
        .partner
        .is_none());
    // When the offset clock reaches exactly 20:00.
    e.resources.current_tick = 720;
    PairingSystem.tick(&mut e.world, &mut e.resources);
    // Then lower female and male ids win ties and both partnerships are mutual.
    for (a, b) in [(0, 2), (1, 3), (2, 0), (3, 1)] {
        assert_eq!(
            e.world.get::<&Kin>(people[a]).expect("kin").partner,
            Some(ids[b])
        );
    }
    let events = e
        .world
        .query::<&FamilyObservations>()
        .iter()
        .next()
        .expect("events")
        .1
        .events
        .clone();
    assert_eq!(
        events,
        vec![
            FamilyEvent::Paired {
                mother: ids[0],
                father: ids[2]
            },
            FamilyEvent::Paired {
                mother: ids[1],
                father: ids[3]
            }
        ]
    );
}

#[test]
fn family_none_is_inert_and_dead_partner_is_cleared_only_at_pair_minute() {
    // Given a widow whose missing partner remains recorded.
    let mut e = fixture();
    let mother = person(&mut e, true);
    let father = person(&mut e, false);
    pair(&mut e, mother, father);
    e.world.despawn(father).expect("death");
    e.resources.current_tick = 1200;
    let rule = e.resources.rules.society.family.take();
    // When pairing is disabled, then no writes or observer entity occur.
    PairingSystem.tick(&mut e.world, &mut e.resources);
    assert!(e.world.get::<&Kin>(mother).expect("kin").partner.is_some());
    assert_eq!(e.world.query::<&FamilyObservations>().iter().count(), 0);
    e.resources.rules.society.family = rule;
    e.resources.current_tick = 1199;
    PairingSystem.tick(&mut e.world, &mut e.resources);
    assert!(e.world.get::<&Kin>(mother).expect("kin").partner.is_some());
    e.resources.current_tick = 1200;
    PairingSystem.tick(&mut e.world, &mut e.resources);
    assert!(e.world.get::<&Kin>(mother).expect("kin").partner.is_none());
}

#[test]
fn pairing_excludes_close_kin_children_elders_and_below_threshold() {
    // Given a woman whose possible matches fail one eligibility condition each.
    let mut e = fixture();
    let mother = person(&mut e, true);
    let candidates: Vec<_> = (0..6).map(|_| person(&mut e, false)).collect();
    let mother_id = id(&e, mother);
    let ids: Vec<_> = candidates.iter().map(|entity| id(&e, *entity)).collect();
    e.world.get::<&mut Kin>(mother).expect("mother").father = Some(ids[0]);
    e.world
        .get::<&mut Kin>(candidates[1])
        .expect("child")
        .mother = Some(mother_id);
    e.world
        .get::<&mut Kin>(candidates[2])
        .expect("sibling")
        .mother = Some(ids[0]);
    e.world
        .insert_one(candidates[3], Child { born_tick: 0 })
        .expect("child");
    e.world.insert_one(candidates[4], Elder).expect("elder");
    // The live rule pairs at familiarity >= 0.200 (baseline `V23` family);
    // the last candidate sits just below it.
    let threshold = e
        .resources
        .rules
        .society
        .family
        .expect("live family rule")
        .familiarity_threshold();
    for (index, other) in ids.iter().enumerate() {
        e.resources.relationships.insert(
            RelationshipKey::new(mother_id, *other),
            RelationshipState {
                familiarity: if index == 5 { threshold - 0.001 } else { 1.0 },
                hostility: 0.0,
            },
        );
    }
    // When the daily matcher runs.
    e.resources.current_tick = 1200;
    PairingSystem.tick(&mut e.world, &mut e.resources);
    // Then nobody is paired.
    assert!(e
        .world
        .query::<&Kin>()
        .iter()
        .all(|(_, kin)| kin.partner.is_none()));
    // Control: at exactly the threshold the last candidate does pair, so the
    // exclusions above are not an inert matcher.
    e.resources.relationships.insert(
        RelationshipKey::new(mother_id, ids[5]),
        RelationshipState {
            familiarity: threshold,
            hostility: 0.0,
        },
    );
    e.resources.current_tick = 1200 + e.resources.ticks_per_day;
    PairingSystem.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        e.world.get::<&Kin>(mother).expect("kin").partner,
        Some(ids[5])
    );
}

#[test]
fn household_partition_prefers_mother_and_preserves_widow_children_once() {
    // Given two couples and a child whose recorded parents belong to different couples.
    let mut e = fixture();
    let people = [
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, true),
    ];
    pair(&mut e, people[0], people[1]);
    pair(&mut e, people[2], people[3]);
    let ids = people.map(|entity| id(&e, entity));
    e.world
        .insert(
            people[4],
            (
                Child { born_tick: 0 },
                Kin {
                    mother: Some(ids[0]),
                    father: Some(ids[3]),
                    ..Kin::default()
                },
            ),
        )
        .expect("child");
    // When households are partitioned, then the mother's household claims the child once.
    let members = ids.into_iter().collect();
    assert_eq!(
        households(&e.world, &members),
        vec![vec![ids[0], ids[1], ids[4]], vec![ids[2], ids[3]]]
    );
    e.world.despawn(people[1]).expect("mother widowed");
    e.world.despawn(people[3]).expect("recorded father dead");
    assert_eq!(
        households(&e.world, &members),
        vec![vec![ids[0], ids[4]], vec![ids[2]]]
    );
}

#[test]
fn family_system_registration_follows_maturation_before_beds_once() {
    // Given the production rhythm registration function.
    let mut e = fixture();
    // When systems are registered.
    sim_systems::register_rhythm_systems(&mut e);
    // Then stable equal-priority registration preserves the required order.
    let names = e.system_names();
    assert_eq!(
        names
            .iter()
            .filter(|name| **name == "PairingSystem")
            .count(),
        1
    );
    let at = |name| {
        names
            .iter()
            .position(|candidate| *candidate == name)
            .expect("system")
    };
    assert!(at("MaturationSystem") < at("PairingSystem"));
    assert!(at("PairingSystem") < at("BedAssignmentSystem"));
}

#[test]
fn founders_receive_keyed_sex_and_empty_kin_only_with_family_enabled() {
    // R3: the baseline world (family rule on). Seed 7 draws its founder count
    // from `WorldVariety::draw(7)`; every founder gets a keyed sex and an
    // empty Kin, and no family event exists at tick zero.
    let new = sim_test::baseline_engine(7);
    let people = new.world.query::<&Agent>().iter().count();
    assert_eq!(
        people as u32,
        sim_core::variety::WorldVariety::draw(7).starting_agents
    );
    assert_eq!(new.world.query::<&Sex>().iter().count(), people);
    assert_eq!(new.world.query::<&Kin>().iter().count(), people);
    let mut women = 0;
    for (_, (lineage, sex, kin)) in new.world.query::<(&Lineage, &Sex, &Kin)>().iter() {
        assert_eq!(*sex, Sex::from_lineage(7, lineage.key));
        assert_eq!(*kin, Kin::default());
        women += usize::from(sex.female);
    }
    // Both sexes are present among the founders (the keyed draw is not constant).
    assert!(women > 0 && women < people, "women {women} of {people}");
    assert_eq!(new.world.query::<&FamilyObservations>().iter().count(), 0);
}
