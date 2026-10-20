mod pass17_family_support;
use hecs::Entity;
use pass17_family_support::*;
use sim_core::components::{
    Agent, BuildingKind, FamilyEvent, FamilyObservations, Kin, Lineage, ResourceKind, Sex,
    Temperament,
};
use sim_core::rules::{BirthRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::settlement::SettlementSystem;

fn birth_world() -> (SimEngine, SettlementSystem, [Entity; 6]) {
    let mut e = fixture();
    let people = [
        person(&mut e, false),
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, true),
        person(&mut e, false),
        person(&mut e, false),
    ];
    pair(&mut e, people[1], people[0]);
    pair(&mut e, people[3], people[2]);
    e.resources.settlements.clear();
    // R1 exact: shelter_planning and life_stage are unchanged from v20 to the baseline.
    let live = WorldRules::baseline(7);
    e.resources.rules.society.shelter_planning = live.society.shelter_planning;
    e.resources.rules.society.life_stage = live.society.life_stage;
    e.resources.rules.society.birth = BirthRule::Provisioned {
        cooldown_ticks: 10,
        food_cost: 5,
        max_mean_hunger: 50.0,
        max_mean_thirst: 50.0,
        sustain: None,
    };
    for (i, kind) in [
        BuildingKind::Hearth,
        BuildingKind::Shelter,
        BuildingKind::Shelter,
        BuildingKind::Shelter,
    ]
    .into_iter()
    .enumerate()
    {
        let tile = (20 + i as u32, 20);
        e.resources.building_registry.insert(100 + i as u64, tile);
        e.resources.building_kinds.insert(tile, kind);
    }
    let mut system = SettlementSystem::new();
    system.tick(&mut e.world, &mut e.resources);
    assert_eq!(e.resources.settlements.len(), 1);
    e.resources
        .settlements
        .values_mut()
        .next()
        .expect("formed")
        .store(ResourceKind::Food, 100);
    (e, system, people)
}

fn born(e: &SimEngine) -> (u64, Option<u64>, Option<u64>, bool, u64) {
    e.world
        .query::<&FamilyObservations>()
        .iter()
        .flat_map(|(_, log)| log.events.iter())
        .find_map(|event| match event {
            FamilyEvent::Born {
                child,
                mother,
                father,
                fallback,
                lineage_key,
                ..
            } => Some((*child, *mother, *father, *fallback, *lineage_key)),
            _ => None,
        })
        .expect("birth observation")
}

#[test]
fn births_choose_oldest_mother_use_her_lineage_and_rotate_fairly() {
    // Given two couples, with the lower-id mother having already given birth.
    let (mut e, mut system, people) = birth_world();
    e.world
        .get::<&mut Kin>(people[1])
        .expect("mother")
        .last_birth_tick = Some(1);
    let mut lineage = *e.world.get::<&Lineage>(people[3]).expect("lineage");
    let expected_key = lineage.child(e.resources.world_seed).key;
    // When the unchanged cooldown permits a birth.
    e.resources.current_tick = 10;
    system.tick(&mut e.world, &mut e.resources);
    // Then the never-birthed mother provides the key and the real ordered parents.
    let event = born(&e);
    assert_eq!(
        (event.1, event.2, event.3, event.4),
        (
            Some(id(&e, people[3])),
            Some(id(&e, people[2])),
            false,
            expected_key
        )
    );
    assert_eq!(
        e.world
            .get::<&Kin>(people[3])
            .expect("mother")
            .last_birth_tick,
        Some(10)
    );
    assert_eq!(
        e.world.get::<&Lineage>(people[3]).expect("mother").births,
        1
    );
    assert_eq!(
        e.world.get::<&Lineage>(people[2]).expect("father").births,
        0
    );
    let newborn = e
        .world
        .query::<(&Agent, &Kin, &Sex, &Temperament)>()
        .iter()
        .find_map(|(_, (agent, kin, sex, _))| (agent.id == event.0).then_some((*kin, *sex)))
        .expect("newborn family");
    assert_eq!((newborn.0.mother, newborn.0.father), (event.1, event.2));
    assert_eq!(newborn.1, Sex::from_lineage(7, expected_key));
    e.resources.current_tick = 20;
    system.tick(&mut e.world, &mut e.resources);
    assert_eq!(born(&e).1, Some(id(&e, people[1])));
}

#[test]
fn dead_partner_cannot_parent_before_evening_cleanup_and_fallback_is_exact() {
    // Given two couples whose fathers have died before the next pairing time.
    let (mut e, mut system, people) = birth_world();
    for entity in [people[0], people[2]] {
        let id = id(&e, entity);
        e.world.despawn(entity).expect("death");
        for settlement in e.resources.settlements.values_mut() {
            settlement.member_agents.remove(&id);
        }
    }
    let mut expected: Vec<_> = e
        .resources
        .settlements
        .values()
        .next()
        .expect("settlement")
        .member_agents
        .iter()
        .copied()
        .collect();
    expected.sort();
    // When birth gates pass before the dead partners are cleared at 20:00.
    e.resources.current_tick = 10;
    system.tick(&mut e.world, &mut e.resources);
    // Then the two lowest remaining ids are recorded explicitly as fallback parents.
    assert_eq!(
        (born(&e).1, born(&e).2, born(&e).3),
        (Some(expected[0]), Some(expected[1]), true)
    );
    assert!(e
        .world
        .get::<&Kin>(people[1])
        .expect("mother")
        .last_birth_tick
        .is_none());
}

#[test]
fn family_does_not_relax_birth_timing_food_or_capacity_gates() {
    // Given the same provisioned settlement with family disabled and enabled.
    for enabled in [false, true] {
        let (mut e, mut system, _) = birth_world();
        if !enabled {
            e.resources.rules.society.family = None;
        }
        // When cooldown, food, and capacity each independently block birth.
        e.resources.current_tick = 9;
        system.tick(&mut e.world, &mut e.resources);
        assert_eq!(e.resources.tallies.births, 0);
        e.resources
            .settlements
            .values_mut()
            .next()
            .expect("settlement")
            .stockpile
            .insert(ResourceKind::Food, 0);
        e.resources.current_tick = 10;
        system.tick(&mut e.world, &mut e.resources);
        assert_eq!(e.resources.tallies.births, 0);
        e.resources
            .settlements
            .values_mut()
            .next()
            .expect("settlement")
            .stockpile
            .insert(ResourceKind::Food, 100);
        e.resources
            .rules
            .society
            .shelter_planning
            .as_mut()
            .expect("planning")
            .residents_per_shelter = 1;
        e.resources.current_tick = 11;
        system.tick(&mut e.world, &mut e.resources);
        // Then all gates retain the same result in both modes.
        assert_eq!(e.resources.tallies.births, 0);
        assert_eq!(e.world.query::<&FamilyObservations>().iter().count(), 0);
    }
}
