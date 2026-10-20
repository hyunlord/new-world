//! Merged in V8-0 (8.4) from `harness_revitalize_pass17_family.rs`,
//! `harness_revitalize_pass17_family_beds.rs`,
//! `harness_revitalize_pass17_family_births.rs`,
//! `harness_revitalize_pass17_family_fission.rs`; each source is one module
//! below, unchanged.

mod family_support;

// From `harness_revitalize_pass17_family.rs`.
mod pairing {
    use super::family_support::*;
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
}

// From `harness_revitalize_pass17_family_beds.rs`.
mod beds {
    use super::family_support::*;
    use sim_core::components::{
        AgentState, BuildingKind, Child, Kin, SeekTarget, Sleep, TargetKind,
    };
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
}

// From `harness_revitalize_pass17_family_births.rs`.
mod births {
    use super::family_support::*;
    use hecs::Entity;
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
}

// From `harness_revitalize_pass17_family_fission.rs`.
mod fission {
    use super::family_support::*;
    use hecs::Entity;
    use sim_core::components::{
        Child, FamilyEvent, FamilyObservations, Kin, Lineage, ResourceKind,
    };
    use sim_core::rng::{keyed_u64, STREAM_FISSION};
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine, WildRegion};
    use sim_systems::runtime::settlement::FissionSystem;

    fn departure_world(count: usize, min: u32, max: u32, adults: u32) -> (SimEngine, Vec<Entity>) {
        let mut e = fixture();
        // R1: the live fission and family rules (`baseline`: pressure trigger,
        // hazard on, elder partners follow). The gates below are numeric test
        // values (R2) so a handful of residents may split; the split itself is
        // driven through the live pressure law (see the pressure preset below).
        e.resources.rules = WorldRules::baseline(7);
        // 1320 + the 480-tick clock offset = 06:00 of day 1, the daily check (spring).
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
        // A settlement this small has hazard h = 0 (members <= size_lo 24), so
        // the day's check only decays its pressure by `decay_permille`; a preset
        // pressure far above any drawn threshold (exponential, mean 1.2) still
        // meets it after the decay, which is the live split condition P >= theta.
        e.resources
            .settlements
            .get_mut(&1)
            .expect("settlement")
            .fission_pressure = 1_000_000;
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
        // The split came through the live pressure law: a drawn threshold (> 0)
        // met by the pressure after the day's decay (1_000_000 - 10).
        let record = &e.resources.fission_log[0];
        assert!(record.threshold_at_split > 0);
        assert_eq!(record.pressure_at_split, 999_990);
        assert!(record.pressure_at_split >= record.threshold_at_split);
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
}
