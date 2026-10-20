mod pass17_family_support;
use pass17_family_support::*;
use sim_core::components::{AgentState, BuildingKind, Child, Kin, SeekTarget, Sleep, TargetKind};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::agent::BedAssignmentSystem;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::housing_capacity;

fn shelters(e: &mut SimEngine, count: u32, capacity: u32) {
    // R1 exact: night_rest and shelter_planning are unchanged from v20 to the baseline.
    let rules = WorldRules::baseline(7);
    e.resources.rules.agent.night_rest = rules.agent.night_rest;
    e.resources.rules.society.shelter_planning = rules.society.shelter_planning;
    e.resources
        .rules
        .society
        .shelter_planning
        .as_mut()
        .expect("planning")
        .residents_per_shelter = capacity;
    for x in 21..21 + count {
        e.resources.building_owner.insert((x, 20), 1);
        e.resources
            .building_kinds
            .insert((x, 20), BuildingKind::Shelter);
        e.resources
            .sleep_tiles
            .insert((x, 20), RESOURCE_SOURCE_INFINITE);
    }
}

#[test]
fn household_packing_leaves_unused_slots_without_changing_capacity_or_fallback() {
    // Given a singleton, a three-person household, and a final singleton.
    let mut e = fixture();
    let people = [
        person(&mut e, false),
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, true),
        person(&mut e, false),
    ];
    let ids = people.map(|entity| id(&e, entity));
    pair(&mut e, people[1], people[2]);
    e.world
        .insert(
            people[3],
            (
                Child { born_tick: 0 },
                Kin {
                    mother: Some(ids[1]),
                    father: Some(ids[2]),
                    ..Kin::default()
                },
            ),
        )
        .expect("child");
    shelters(&mut e, 2, 3);
    // When households first-fit, then the first shelter's two remaining slots stay empty.
    BedAssignmentSystem::default().tick(&mut e.world, &mut e.resources);
    assert_eq!(e.resources.bed_of.get(&ids[0]), Some(&(21, 20)));
    for id in &ids[1..4] {
        assert_eq!(e.resources.bed_of.get(id), Some(&(22, 20)));
    }
    assert!(!e.resources.bed_of.contains_key(&ids[4]));
    let rule = e
        .resources
        .rules
        .society
        .shelter_planning
        .expect("planning");
    assert_eq!(housing_capacity(&e.resources, 1, &rule), 6);
    // When the bedless person becomes tired, then the existing nearest-shelter fallback still applies.
    e.world
        .insert_one(people[4], Sleep::new(90.0, 0.0))
        .expect("sleep");
    AgentDecisionSystem::new().tick(&mut e.world, &mut e.resources);
    assert_eq!(
        *e.world.get::<&AgentState>(people[4]).expect("state"),
        AgentState::Seeking {
            target: TargetKind::Sleep
        }
    );
    let target = *e
        .world
        .get::<&SeekTarget>(people[4])
        .expect("fallback target");
    assert_eq!(target.tile, (21, 20));
}

#[test]
fn oversized_household_spans_shelters_and_zero_capacity_assigns_nobody() {
    // Given one five-person household and three two-place shelters.
    let mut e = fixture();
    let people = [
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, true),
    ];
    pair(&mut e, people[0], people[1]);
    let ids = people.map(|entity| id(&e, entity));
    for entity in &people[2..] {
        e.world
            .insert(
                *entity,
                (
                    Child { born_tick: 0 },
                    Kin {
                        mother: Some(ids[0]),
                        father: Some(ids[1]),
                        ..Kin::default()
                    },
                ),
            )
            .expect("child");
    }
    shelters(&mut e, 3, 2);
    // When assigned, then the oversized household spans shelters in member-id order.
    BedAssignmentSystem::default().tick(&mut e.world, &mut e.resources);
    for (index, id) in ids.iter().enumerate() {
        assert_eq!(
            e.resources.bed_of.get(id),
            Some(&(21 + index as u32 / 2, 20))
        );
    }
    // When capacity is zero, then every old assignment is removed without division by zero.
    shelters(&mut e, 3, 0);
    BedAssignmentSystem::default().tick(&mut e.world, &mut e.resources);
    assert!(e.resources.bed_of.is_empty());
}
