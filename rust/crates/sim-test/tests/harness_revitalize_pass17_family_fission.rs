mod pass17_family_support;
use hecs::Entity;
use pass17_family_support::*;
use sim_core::components::{Child, FamilyEvent, FamilyObservations, Kin, Lineage, ResourceKind};
use sim_core::rng::{keyed_u64, STREAM_FISSION};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine, WildRegion};
use sim_systems::runtime::settlement::FissionSystem;

fn departure_world(count: usize, min: u32, max: u32, adults: u32) -> (SimEngine, Vec<Entity>) {
    let mut e = fixture();
    e.resources.rules = WorldRules::game_v17(7);
    e.resources.rules.society.family = Some(WorldRules::V22_FAMILY);
    e.resources.current_tick = 1320;
    let rule = e.resources.rules.society.fission.as_mut().expect("fission");
    rule.min_age_days = 0;
    rule.cooldown_days = 0;
    rule.min_members = 0;
    rule.crowding = 0.0;
    rule.group_share = 0.0;
    rule.group_min = min;
    rule.group_max = max;
    rule.min_adults = adults;
    let mut people: Vec<_> = (0..count).map(|_| person(&mut e, false)).collect();
    people.sort_by_key(|entity| {
        let key = e.world.get::<&Lineage>(*entity).expect("lineage").key;
        (
            keyed_u64(7, STREAM_FISSION, key, 1320, 0, 0),
            id(&e, *entity),
        )
    });
    e.resources.wild_regions.push(WildRegion {
        centre: (46, 20),
        bushes: 6,
        deadwood: 8,
        water_distance: 3,
    });
    e.resources
        .settlements
        .get_mut(&1)
        .expect("settlement")
        .store(ResourceKind::Food, 100);
    (e, people)
}

fn children(e: &mut SimEngine, parents: [Entity; 2], children: &[Entity]) {
    pair(e, parents[0], parents[1]);
    let (mother, father) = (id(e, parents[0]), id(e, parents[1]));
    for entity in children {
        e.world
            .insert(
                *entity,
                (
                    Child { born_tick: 0 },
                    Kin {
                        mother: Some(mother),
                        father: Some(father),
                        ..Kin::default()
                    },
                ),
            )
            .expect("child");
    }
}

fn depart(e: &mut SimEngine) -> FamilyEvent {
    FissionSystem::new().tick(&mut e.world, &mut e.resources);
    assert_eq!(e.resources.fission_log.len(), 1);
    e.world
        .query::<&FamilyObservations>()
        .iter()
        .find_map(|(_, observations)| {
            observations
                .events
                .iter()
                .find(|event| matches!(event, FamilyEvent::Departed { .. }))
                .cloned()
        })
        .expect("departure observation")
}

#[test]
fn household_seed_can_exceed_target_k_while_original_stock_share_is_preserved() {
    let (mut e, people) = departure_world(6, 3, 6, 2);
    children(&mut e, [people[0], people[1]], &people[2..4]);
    let event = depart(&mut e);
    let FamilyEvent::Departed {
        members,
        households,
        couples,
        couples_total,
        couples_split,
        households_split,
        fission_id,
        ..
    } = event
    else {
        unreachable!()
    };
    let mut expected: Vec<_> = people[..4].iter().map(|entity| id(&e, *entity)).collect();
    expected.sort();
    assert_eq!(members, expected);
    assert_eq!(e.resources.fission_log[0].members, members);
    assert_eq!(e.resources.fission_log[0].fission_id, fission_id);
    assert_eq!(e.resources.fission_log[0].food_taken, 50);
    assert_eq!((couples_total, couples_split, households_split), (1, 0, 0));
    assert_eq!(couples.len(), 1);
    assert_eq!(households.iter().map(Vec::len).sum::<usize>(), 6);
}

#[test]
fn seed_skips_household_above_group_max_and_keeps_later_households_whole() {
    let (mut e, people) = departure_world(7, 3, 3, 2);
    children(&mut e, [people[0], people[1]], &people[2..4]);
    let FamilyEvent::Departed {
        members,
        couples_split,
        households_split,
        ..
    } = depart(&mut e)
    else {
        unreachable!()
    };
    let mut expected: Vec<_> = people[4..].iter().map(|entity| id(&e, *entity)).collect();
    expected.sort();
    assert_eq!(members, expected);
    assert_eq!((couples_split, households_split), (0, 0));
}

#[test]
fn group_min_top_up_splits_only_after_whole_households_cannot_fit() {
    let (mut e, people) = departure_world(5, 4, 4, 1);
    let mother = id(&e, people[0]);
    for entity in &people[1..3] {
        e.world
            .insert(
                *entity,
                (
                    Child { born_tick: 0 },
                    Kin {
                        mother: Some(mother),
                        father: Some(999),
                        ..Kin::default()
                    },
                ),
            )
            .expect("widow child");
    }
    pair(&mut e, people[3], people[4]);
    let FamilyEvent::Departed {
        members,
        households,
        couples,
        couples_total,
        couples_split,
        households_split,
        ..
    } = depart(&mut e)
    else {
        unreachable!()
    };
    let mut expected: Vec<_> = people[..4].iter().map(|entity| id(&e, *entity)).collect();
    expected.sort();
    assert_eq!(members, expected);
    assert_eq!((couples_total, couples_split, households_split), (1, 1, 1));
    assert_eq!(
        couples
            .iter()
            .filter(|pair| members.contains(&pair[0]) != members.contains(&pair[1]))
            .count(),
        couples_split as usize
    );
    assert_eq!(
        households
            .iter()
            .filter(|household| household.iter().any(|id| members.contains(id))
                && household.iter().any(|id| !members.contains(id)))
            .count(),
        households_split as usize
    );
}
