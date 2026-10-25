//! Merged in V8-0 (8.4) from `harness_revitalize_pass19_chronicle.rs`,
//! `harness_revitalize_pass19_chronicle_events.rs`; each source is one module
//! below, unchanged.

// From `harness_revitalize_pass19_chronicle.rs`.
mod living_people {
    //! Pass 19 Part 1: the chronicle is complete enough to rebuild every living
    //! person's roster from its membership entries (formation, joins, leaves,
    //! births, settling, deaths), names only people with a record, and closes
    //! every death; through a fission of the baseline world (V8-0: the
    //! no-fission and legacy scenes are gone).
    use sim_core::chronicle::{Chronicle, ChronicleKind};
    use sim_core::components::{Agent, AgentId, SettlementId};
    use sim_engine::SimEngine;
    use std::collections::{BTreeMap, BTreeSet};

    /// Each agent's rosters as the chronicle's membership entries describe them.
    fn replay(chronicle: &Chronicle) -> BTreeMap<AgentId, BTreeSet<SettlementId>> {
        let mut rosters: BTreeMap<AgentId, BTreeSet<SettlementId>> = BTreeMap::new();
        for entry in chronicle.entries() {
            let (Some(subject), settlement) = (entry.subject, entry.settlement) else {
                continue;
            };
            match entry.kind {
                ChronicleKind::FoundingMember | ChronicleKind::Joined | ChronicleKind::Born => {
                    rosters.entry(subject).or_default().extend(settlement);
                }
                ChronicleKind::LeftSettlement => {
                    if let (Some(roster), Some(sid)) = (rosters.get_mut(&subject), settlement) {
                        roster.remove(&sid);
                    }
                }
                ChronicleKind::Settled => {
                    rosters.insert(subject, settlement.into_iter().collect());
                }
                ChronicleKind::Died => {
                    rosters.remove(&subject);
                }
                _ => {}
            }
        }
        rosters
    }

    fn actual(engine: &SimEngine) -> BTreeMap<AgentId, BTreeSet<SettlementId>> {
        let mut rosters: BTreeMap<AgentId, BTreeSet<SettlementId>> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, agent)| (agent.id, BTreeSet::new()))
            .collect();
        for (sid, settlement) in &engine.resources.settlements {
            for id in &settlement.member_agents {
                rosters.entry(*id).or_default().insert(*sid);
            }
        }
        rosters
    }

    fn check(engine: &SimEngine, name: &str) {
        let chronicle = &engine.resources.chronicle;
        let replayed = replay(chronicle);
        for (id, roster) in actual(engine) {
            let expected = replayed.get(&id).cloned().unwrap_or_default();
            assert_eq!(
                roster,
                expected,
                "{name} tick {}: agent {id}",
                engine.current_tick()
            );
            assert!(
                chronicle.person(id).is_some(),
                "{name}: living agent {id} has a record"
            );
        }
        for entry in chronicle.entries() {
            for id in entry.agents() {
                assert!(
                    chronicle.person(id).is_some(),
                    "{name}: {entry:?} names {id} without a record"
                );
            }
            if entry.kind == ChronicleKind::Died {
                let person = chronicle
                    .person(entry.subject.expect("died subject"))
                    .expect("record");
                assert_eq!(person.died_tick, Some(entry.tick), "{name}: {entry:?}");
                assert_eq!(
                    person.death_reason.map(u32::from),
                    Some(entry.cause),
                    "{name}: {entry:?}"
                );
            }
        }
    }

    /// Run `days` days, checking at every day boundary; returns the engine.
    fn run(mut engine: SimEngine, name: &str, days: u64) -> SimEngine {
        let ticks_per_day = engine.resources.ticks_per_day;
        check(&engine, name);
        for _ in 0..days {
            for _ in 0..ticks_per_day {
                engine.tick();
            }
            check(&engine, name);
        }
        engine
    }

    fn count(chronicle: &Chronicle, kind: ChronicleKind) -> usize {
        chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == kind)
            .count()
    }

    #[test]
    fn harness_chronicle_rebuilds_rosters_through_a_fission() {
        // Baseline world 8 splits at 06:00 of display day 28 and founds its
        // daughter on day 29 (V8-0 re-measure; the earliest split in a scan of
        // seeds 0..48), so 32 days cover the whole fission.
        let start = sim_test::baseline_engine(8);
        let founders = start.world.query::<&Agent>().iter().count();
        let engine = run(start, "baseline", 32);
        let chronicle = &engine.resources.chronicle;
        assert_eq!(engine.resources.tallies.fissions, 1);
        assert_eq!(count(chronicle, ChronicleKind::DaughterFounded), 1);
        assert_eq!(
            count(chronicle, ChronicleKind::Born) as u64,
            engine.resources.tallies.births
        );
        // Measured on world 8 at day 32: 20 pairings, a group of 14 departing,
        // 23 founding members (every founder), 39 births, 7 deaths.
        assert!(count(chronicle, ChronicleKind::Paired) >= 10);
        assert_eq!(
            count(chronicle, ChronicleKind::Departed),
            engine.resources.fission_log[0].members.len()
        );
        assert!(count(chronicle, ChronicleKind::Departed) >= 6, "group_min");
        assert_eq!(count(chronicle, ChronicleKind::FoundingMember), founders);
        let deaths = engine.resources.tallies;
        let died = deaths.deaths_starvation
            + deaths.deaths_dehydration
            + deaths.deaths_combat
            + deaths.deaths_old_age
            + deaths.deaths_exposure;
        assert_eq!(count(chronicle, ChronicleKind::Died) as u64, died);
        assert!(died >= 3, "the run closes deaths too ({died})");
    }
}

// From `harness_revitalize_pass19_chronicle_events.rs`.
mod write_paths {
    //! Pass 19 review round 1, finding 1: the chronicle's write paths that the
    //! 32 evaluation worlds never reached — a cancelled fission (with a member who
    //! died on the way), a remnant merge, and the dissolution of an empty
    //! settlement by `FissionSystem` and by `SettlementSystem`. The setups are
    //! Pass 11's (`harness_revitalize_pass11_fission.rs`).
    use hecs::Entity;
    use sim_core::causal::event::DeathReason;
    use sim_core::chronicle::{ChronicleEntry, ChronicleKind, DepartCause};
    use sim_core::components::{
        Agent, AgentId, AgentState, Child, Elder, Founding, Lineage, Position, ResourceKind,
        Settlement,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine, WildRegion};
    use sim_systems::runtime::settlement::fission::sustain_capacity;
    use sim_systems::runtime::settlement::{FissionSystem, SettlementSystem};
    use sim_systems::runtime::survival::despawn_agent;

    const HOME: (u32, u32) = (20, 20);
    const SITE: (u32, u32) = (46, 20);

    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    type Resident = (AgentId, u8, u64);

    /// Pass 11's bare world under `WorldRules::baseline(seed)` (one settlement
    /// at `HOME`). The live fission rule splits by accumulated pressure against
    /// a hidden threshold; the settlement's threshold is set to 1 so the first
    /// eligible 06:00 check with a positive hazard (32 members > `size_lo` 24,
    /// 100 food < 4 days of their need) splits, as the crowding gate did in
    /// Pass 11's `game_v17` fixture.
    fn world(seed: u64, bushes: u32, residents: &[Resident]) -> (SimEngine, Vec<Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = WorldRules::baseline(seed);
        engine.resources.world_seed = seed;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        settlement.fission_threshold = 1;
        settlement.store(ResourceKind::Food, 100);
        settlement.store(ResourceKind::Wood, 30);
        for i in 0..bushes {
            engine
                .resources
                .set_food_source_max(10 + i % 20, 10 + i / 20, 4);
        }
        let mut entities = Vec::new();
        for &(id, stage, key) in residents {
            let entity = engine.world.spawn((
                Position::new(HOME.0, HOME.1),
                Agent { id },
                AgentState::Idle,
                Lineage { key, births: 0 },
            ));
            match stage {
                0 => engine
                    .world
                    .insert_one(entity, Child { born_tick: 0 })
                    .expect("child"),
                2 => engine.world.insert_one(entity, Elder).expect("elder"),
                _ => {}
            }
            settlement.add_member_agent(id);
            entities.push(entity);
        }
        engine.resources.settlements.insert(1, settlement);
        engine.resources.wild_regions.push(WildRegion {
            centre: SITE,
            bushes: 6,
            deadwood: 8,
            water_distance: 3,
        });
        (engine, entities)
    }

    fn village(n: u64) -> Vec<Resident> {
        (1..=n)
            .map(|id| {
                let stage = if id % 7 == 0 {
                    2
                } else if id % 5 == 0 {
                    0
                } else {
                    1
                };
                (id, stage, id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x55)
            })
            .collect()
    }

    fn run(engine: &mut SimEngine, system: &mut FissionSystem, from: u64, to: u64) {
        for tick in from..to {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
    }

    fn bushes_for(seed: u64, target: u32) -> u32 {
        (1..400)
            .find(|&b| {
                let (engine, _) = world(seed, b, &[]);
                sustain_capacity(&engine.resources, 1).is_some_and(|c| c >= target)
            })
            .expect("enough bushes")
    }

    fn of_kind(engine: &SimEngine, kind: ChronicleKind) -> Vec<ChronicleEntry> {
        engine
            .resources
            .chronicle
            .entries()
            .iter()
            .copied()
            .filter(|e| e.kind == kind)
            .collect()
    }

    #[test]
    fn harness_chronicle_records_a_cancelled_fission_and_skips_the_dead() {
        let seed = 11;
        let bushes = bushes_for(seed, 30);
        let check = tick_at(28, 360);
        let (mut engine, entities) = world(seed, bushes, &village(32));
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, check, check + 1);
        let group: Vec<AgentId> = {
            let mut ids: Vec<AgentId> = engine
                .world
                .query::<(&Agent, &Founding)>()
                .iter()
                .map(|(_, (a, _))| a.id)
                .collect();
            ids.sort_unstable();
            ids
        };
        assert!(group.len() >= 6, "{group:?}");
        // The split: FissionStarted, then every member in AgentId order.
        let fission_id = engine.resources.fission_log[0].fission_id;
        let started = of_kind(&engine, ChronicleKind::FissionStarted);
        assert_eq!(started.len(), 1);
        assert_eq!(
            (started[0].tick, started[0].settlement, started[0].value),
            (check, Some(1), i64::from(fission_id))
        );
        let departed = of_kind(&engine, ChronicleKind::Departed);
        assert_eq!(
            departed
                .iter()
                .filter_map(|e| e.subject)
                .collect::<Vec<_>>(),
            group
        );
        // The live family rule labels each member with why the household
        // selection took them (V8-0 re-measure, seed 11: 4 `Seed` adults — the
        // rule's `min_adults` — and 7 `Household`); never `Ranked`, which only the
        // no-family path writes.
        let cause_count = |cause: DepartCause| {
            departed
                .iter()
                .filter(|e| e.cause == u32::from(cause.as_u8()))
                .count()
        };
        assert_eq!(cause_count(DepartCause::Ranked), 0);
        assert_eq!(cause_count(DepartCause::Seed), 4);
        assert_eq!(
            cause_count(DepartCause::Seed) + cause_count(DepartCause::Household),
            group.len()
        );
        assert!(departed.iter().all(|e| e.other == started[0].other));
        // One member dies on the way (the highest id; not the leader).
        let dead = *group.last().expect("group");
        let entity = entities[usize::try_from(dead - 1).expect("id")];
        despawn_agent(
            &mut engine.world,
            &mut engine.resources,
            entity,
            dead,
            HOME,
            DeathReason::Starvation,
            check + 10,
        );
        // Two days later the leader still stands at home: the fission is cancelled.
        run(&mut engine, &mut system, check + 1, check + 2 * 1440 + 2);
        assert_eq!(engine.resources.tallies.fissions_cancelled, 1);
        let cancelled_tick = engine.resources.fission_log[0]
            .cancelled_tick
            .expect("cancelled");
        let cancelled = of_kind(&engine, ChronicleKind::FissionCancelled);
        assert_eq!(cancelled.len(), 1);
        assert_eq!(
            (
                cancelled[0].tick,
                cancelled[0].settlement,
                cancelled[0].value
            ),
            (cancelled_tick, Some(1), i64::from(fission_id))
        );
        let back = of_kind(&engine, ChronicleKind::DepartureCancelled);
        let living: Vec<AgentId> = group.iter().copied().filter(|id| *id != dead).collect();
        assert_eq!(
            back.iter().filter_map(|e| e.subject).collect::<Vec<_>>(),
            living,
            "living members in AgentId order"
        );
        assert!(back.iter().all(|e| e.tick == cancelled_tick
            && e.settlement == Some(1)
            && e.value == i64::from(fission_id)));
        // Order: the cancellation directly precedes its members' entries.
        let entries = engine.resources.chronicle.entries();
        let at = entries
            .iter()
            .position(|e| e.kind == ChronicleKind::FissionCancelled)
            .expect("cancel");
        assert!(entries[at + 1..at + 1 + living.len()]
            .iter()
            .all(|e| e.kind == ChronicleKind::DepartureCancelled));
        // The dead member: Died, record closed, no cancellation entry.
        let died = of_kind(&engine, ChronicleKind::Died);
        assert_eq!(died.len(), 1);
        assert_eq!(
            (died[0].subject, died[0].settlement, died[0].cause),
            (
                Some(dead),
                Some(1),
                u32::from(DeathReason::Starvation.as_u8())
            )
        );
        let record = engine
            .resources
            .chronicle
            .person(dead)
            .expect("record created at death");
        assert_eq!(
            (record.died_tick, record.death_reason),
            (Some(check + 10), Some(DeathReason::Starvation.as_u8()))
        );
    }

    /// One remnant-merge scenario (Pass 11's): a two-member remnant (settlement 2)
    /// and an empty settlement (3) beside the 30-member village; returns the engine
    /// after the merge.
    fn remnant_scenario(seed: u64) -> SimEngine {
        let (mut engine, _) = world(seed, 10, &village(30));
        let mut remnant = Settlement::new_with_id(2, 0);
        remnant.formation_tile = (50, 50);
        let mut empty = Settlement::new_with_id(3, 0);
        empty.formation_tile = (10, 50);
        for id in [102u64, 101] {
            engine.world.spawn((
                Position::new(50, 50),
                Agent { id },
                AgentState::Idle,
                Lineage { key: id, births: 0 },
            ));
            remnant.add_member_agent(id);
        }
        engine.resources.settlements.insert(2, remnant);
        engine.resources.settlements.insert(3, empty);
        let mut system = FissionSystem::new();
        let start = tick_at(3, 600);
        run(&mut engine, &mut system, start, start + 61);
        let dissolved = of_kind(&engine, ChronicleKind::SettlementDissolved);
        assert_eq!(dissolved.len(), 1, "only the empty settlement so far");
        assert_eq!((dissolved[0].settlement, dissolved[0].cause), (Some(3), 0));
        run(&mut engine, &mut system, start + 61, start + 1440 + 61);
        assert_eq!(engine.resources.tallies.remnant_merges, 1);
        engine
    }

    #[test]
    fn harness_chronicle_records_remnant_merges_and_empty_dissolutions() {
        // `Settlement::member_agents` is a std `HashSet` with a random hasher per
        // set, so a two-member roster iterates in either order about half the
        // time: 32 fresh engines make a missing sort fail with probability
        // ~1 - 2^-32 (review round 2, finding 1).
        for repeat in 0..32 {
            let engine = remnant_scenario(3);
            let tail: Vec<(ChronicleKind, Option<AgentId>, Option<u32>, u32)> = engine
                .resources
                .chronicle
                .entries()
                .iter()
                .skip(1)
                .map(|e| (e.kind, e.subject, e.settlement, e.cause))
                .collect();
            assert_eq!(
                tail,
                vec![
                    (ChronicleKind::LeftSettlement, Some(101), Some(2), 0),
                    (ChronicleKind::LeftSettlement, Some(102), Some(2), 0),
                    (ChronicleKind::SettlementDissolved, None, Some(2), 1),
                ],
                "repeat {repeat}"
            );
            let merge_tick = engine
                .resources
                .dissolutions
                .iter()
                .find(|(sid, _)| *sid == 2)
                .expect("merged")
                .1;
            assert!(engine.resources.chronicle.entries()[1..]
                .iter()
                .all(|e| e.tick == merge_tick));
            // The merged members got records from their live components on first mention.
            assert!(
                engine.resources.chronicle.person(101).is_some()
                    && engine.resources.chronicle.person(102).is_some()
            );
        }
    }

    #[test]
    fn harness_chronicle_records_a_settlement_system_dissolution() {
        // `SettlementSystem::run_dissolutions`: no live member and no building.
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let mut settlement = Settlement::new_with_id(7, 0);
        settlement.formation_tile = (5, 5);
        engine.resources.settlements.insert(7, settlement);
        engine.resources.current_tick = 40;
        let mut system = SettlementSystem::new();
        system.tick(&mut engine.world, &mut engine.resources);
        assert!(!engine.resources.settlements.contains_key(&7));
        let entries = engine.resources.chronicle.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            (
                entries[0].kind,
                entries[0].tick,
                entries[0].settlement,
                entries[0].cause
            ),
            (ChronicleKind::SettlementDissolved, 40, Some(7), 0)
        );
    }
}
