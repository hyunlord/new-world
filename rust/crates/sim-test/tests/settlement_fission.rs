//! Merged in V8-0 (8.4) from `harness_revitalize_pass11_fission.rs`,
//! `harness_revitalize_pass12_fission.rs`,
//! `harness_revitalize_pass13_fission.rs`; each source is one module below,
//! unchanged.

// From `harness_revitalize_pass11_fission.rs`.
mod regions {
    //! Pass 11 Part 2: wild regions and village fission, under the live rules
    //! (`WorldRules::baseline`; V8-0 port of the `game_v17` fixtures).
    use std::collections::BTreeSet;

    use hecs::Entity;
    use sim_core::components::{
        Agent, AgentId, AgentState, BuildingKind, Child, Elder, Founding, Lineage, Position,
        ResourceKind, Settlement,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rng::{keyed_u64, NoiseFork, STREAM_FISSION};
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine, WildRegion};
    use sim_systems::runtime::settlement::fission::sustain_capacity;
    use sim_systems::runtime::settlement::FissionSystem;
    use sim_test::baseline_engine;

    const HOME: (u32, u32) = (20, 20);
    const SITE: (u32, u32) = (46, 20);

    /// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// One resident: id, stage (0 child, 1 adult, 2 elder), lineage key.
    type Resident = (AgentId, u8, u64);

    /// A bare `WorldRules::baseline(seed)` world: one settlement (id 1) at
    /// `HOME` founded at tick 0 holding 100 food and 30 wood, `bushes`
    /// capacity-4 bushes around it, one wild region centred on `SITE`, and the
    /// given residents spawned in order.
    ///
    /// The live fission rule splits by accumulated pressure against a hidden
    /// threshold (Pass 13), not by the crowding gate `game_v17` used. The
    /// settlement's threshold is set to 1, so the first eligible 06:00 check with
    /// a positive hazard splits: with 32 or more members (above `size_lo` 24) and
    /// 100 food (below four days of their need) h > 0 at once.
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

    /// `n` residents: ids 1..=n, every fifth a child, every seventh an elder.
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

    /// Run `system` at every tick of [from, to).
    fn run(engine: &mut SimEngine, system: &mut FissionSystem, from: u64, to: u64) {
        for tick in from..to {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
    }

    /// Bushes giving a capacity of about `target` members (capacity-4 bushes).
    fn bushes_for(seed: u64, target: u32) -> u32 {
        (1..400)
            .find(|&b| {
                let (engine, _) = world(seed, b, &[]);
                sustain_capacity(&engine.resources, 1).is_some_and(|c| c >= target)
            })
            .expect("enough bushes")
    }

    fn group_of(engine: &SimEngine) -> BTreeSet<AgentId> {
        engine
            .world
            .query::<(&Agent, &Founding)>()
            .iter()
            .map(|(_, (a, _))| a.id)
            .collect()
    }

    #[test]
    fn harness_fission_group_rules() {
        for seed in [3u64, 11, 900_123] {
            let bushes = bushes_for(seed, 30);
            let people = village(32);
            let (mut engine, _) = world(seed, bushes, &people);
            let mut system = FissionSystem::new();
            let check = tick_at(28, 360);
            run(&mut engine, &mut system, check, check + 1);
            let record = engine
                .resources
                .fission_log
                .first()
                .expect("fission")
                .clone();
            let group: BTreeSet<AgentId> = record.members.iter().copied().collect();
            assert_eq!(group, group_of(&engine));
            // Size: clamp(round(32 x 0.35), 6, 14) = 11.
            assert_eq!(group.len(), 11);
            let stage = |id: &AgentId| {
                people
                    .iter()
                    .find(|p| p.0 == *id)
                    .map(|p| p.1)
                    .expect("resident")
            };
            assert!(group.iter().all(|id| stage(id) != 2), "no elder leaves");
            let adults = group.iter().filter(|id| stage(id) == 1).count();
            assert!(adults >= 4 && adults as u32 == record.adults);
            // The keyed ranking: first four adults by rank, then the best-ranked rest.
            let mut ranked: Vec<(u64, AgentId, u8)> = people
                .iter()
                .filter(|p| p.1 != 2)
                .map(|p| (keyed_u64(seed, STREAM_FISSION, p.2, check, 0, 0), p.0, p.1))
                .collect();
            ranked.sort_unstable();
            let mut expected: BTreeSet<AgentId> = ranked
                .iter()
                .filter(|r| r.2 == 1)
                .take(4)
                .map(|r| r.1)
                .collect();
            for r in &ranked {
                if expected.len() >= 11 {
                    break;
                }
                expected.insert(r.1);
            }
            assert_eq!(group, expected, "seed {seed}");
            // The leader is the group's lowest-AgentId adult; every founder walks to the site.
            let leader = *group.iter().find(|id| stage(id) == 1).expect("adult");
            for (_, (agent, founding)) in engine.world.query::<(&Agent, &Founding)>().iter() {
                assert_eq!(
                    (founding.leader, founding.site, founding.since_tick),
                    (leader, SITE, check),
                    "agent {}",
                    agent.id
                );
            }
        }
    }

    #[test]
    fn harness_fission_group_resampled_by_replicate() {
        let mut differs = 0;
        for seed in 900_200..900_208u64 {
            let bushes = bushes_for(seed, 30);
            let check = tick_at(28, 360);
            let pick = |fork: Option<NoiseFork>| {
                let (mut engine, _) = world(seed, bushes, &village(36));
                engine.resources.noise_fork = fork;
                let mut system = FissionSystem::new();
                run(&mut engine, &mut system, check, check + 1);
                engine
                    .resources
                    .fission_log
                    .first()
                    .expect("fission")
                    .members
                    .clone()
            };
            let plain = pick(None);
            // Replicate 0 is the unforked run by definition.
            assert_eq!(
                pick(Some(NoiseFork {
                    tick: check - 100,
                    replicate: 0
                })),
                plain
            );
            if pick(Some(NoiseFork {
                tick: check - 100,
                replicate: 3,
            })) != plain
            {
                differs += 1;
            }
            // A fork after the check changes nothing.
            assert_eq!(
                pick(Some(NoiseFork {
                    tick: check + 1,
                    replicate: 3
                })),
                plain
            );
        }
        assert!(
            differs >= 1,
            "replicate 3 resamples the group in at least one of 8 seeds"
        );
    }

    #[test]
    fn harness_fission_founds_daughter() {
        let seed = 11;
        let bushes = bushes_for(seed, 30);
        let (mut engine, entities) = world(seed, bushes, &village(32));
        let mut system = FissionSystem::new();
        let check = tick_at(28, 360);
        let before = |e: &SimEngine, kind| {
            e.resources.settlements[&1]
                .stockpile
                .get(&kind)
                .copied()
                .unwrap_or(0)
        };
        let (food0, wood0) = (
            before(&engine, ResourceKind::Food),
            before(&engine, ResourceKind::Wood),
        );
        run(&mut engine, &mut system, check, check + 1);
        let record = engine.resources.fission_log[0].clone();
        // Food and wood are conserved at departure: parent + pending = before.
        assert_eq!(record.food_taken, 100 * 11 / 32);
        assert_eq!(record.wood_taken, 30 * 11 / 32);
        assert_eq!(
            before(&engine, ResourceKind::Food) + record.food_taken,
            food0
        );
        assert_eq!(
            before(&engine, ResourceKind::Wood) + record.wood_taken,
            wood0
        );
        // The group walks: three founders arrive at the site.
        assert_eq!(entities.len(), 32);
        let mut moved = 0;
        for (_, (agent, pos)) in engine.world.query_mut::<(&Agent, &mut Position)>() {
            if record.members.contains(&agent.id) && moved < 3 {
                *pos = Position::new(SITE.0 + 1, SITE.1);
                moved += 1;
            }
        }
        run(&mut engine, &mut system, check + 1, check + 2);
        // Hearth and two shelters are stamped next to the site.
        let hearth = *engine
            .resources
            .building_kinds
            .iter()
            .find(|(tile, kind)| {
                **kind == BuildingKind::Hearth
                    && tile.0.abs_diff(SITE.0).max(tile.1.abs_diff(SITE.1)) <= 2
            })
            .map(|(tile, _)| tile)
            .expect("daughter hearth");
        let shelters = engine
            .resources
            .building_kinds
            .iter()
            .filter(|(tile, kind)| {
                **kind == BuildingKind::Shelter
                    && tile.0.abs_diff(hearth.0).max(tile.1.abs_diff(hearth.1)) <= 3
            })
            .count();
        assert_eq!(shelters, 2);
        assert_eq!(engine.resources.building_event_queue.len(), 3);
        // The formation scan forms the daughter (as SettlementSystem would, with
        // the founders standing in radius as members of both rosters).
        let mut daughter = Settlement::new_with_id(2, check + 2);
        daughter.formation_tile = hearth;
        engine.resources.settlements.insert(2, daughter);
        run(&mut engine, &mut system, check + 2, check + 3);
        let parent = &engine.resources.settlements[&1];
        let daughter = &engine.resources.settlements[&2];
        for id in &record.members {
            assert!(
                daughter.member_agents.contains(id) && !parent.member_agents.contains(id),
                "agent {id} moved"
            );
        }
        assert_eq!(daughter.member_agents.len(), 11);
        assert_eq!(parent.member_agents.len(), 21);
        assert_eq!(daughter.parent, Some(1));
        // The pending store is delivered: totals conserved.
        assert_eq!(
            daughter
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0),
            record.food_taken
        );
        assert_eq!(
            daughter
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0),
            record.wood_taken
        );
        assert_eq!(
            parent
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0)
                + daughter
                    .stockpile
                    .get(&ResourceKind::Food)
                    .copied()
                    .unwrap_or(0),
            food0
        );
        assert_eq!(engine.world.query::<&Founding>().iter().count(), 0);
        assert_eq!(engine.resources.tallies.fissions, 1);
        assert_eq!(engine.resources.fission_log[0].daughter, Some(2));
    }

    #[test]
    fn harness_fission_timeout_and_cancel() {
        let seed = 11;
        let bushes = bushes_for(seed, 30);
        let check = tick_at(28, 360);
        // Nobody walks (only FissionSystem runs): after two days the leader stands
        // at home, inside the parent's clearance, so the fission is cancelled.
        let (mut engine, _) = world(seed, bushes, &village(32));
        let food0 = engine.resources.settlements[&1].stockpile[&ResourceKind::Food];
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, check, check + 2 * 1440 + 2);
        assert_eq!(engine.resources.tallies.fissions_cancelled, 1);
        assert_eq!(engine.resources.tallies.fissions, 0);
        assert!(engine.resources.fission_log[0].cancelled_tick.is_some());
        assert_eq!(
            engine.world.query::<&Founding>().iter().count(),
            0,
            "markers removed"
        );
        assert_eq!(
            engine.resources.settlements[&1].stockpile[&ResourceKind::Food],
            food0,
            "store returned"
        );
        assert_eq!(engine.resources.settlements[&1].member_agents.len(), 32);
        // Timeout far from every hearth: the group settles where its leader stands.
        let (mut engine, _) = world(seed, bushes, &village(32));
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, check, check + 1);
        let leader = engine
            .world
            .query::<&Founding>()
            .iter()
            .next()
            .map(|(_, f)| f.leader)
            .expect("leader");
        let spot = (30, 50);
        for (_, (agent, pos)) in engine.world.query_mut::<(&Agent, &mut Position)>() {
            if agent.id == leader {
                *pos = Position::new(spot.0, spot.1);
            }
        }
        run(&mut engine, &mut system, check + 1, check + 2 * 1440 + 1);
        assert!(engine.resources.fission_log[0].settled_at_leader);
        assert_eq!(
            engine.resources.building_kinds.get(&spot),
            Some(&BuildingKind::Hearth)
        );
        // Review round 1, M1: every founder now walks to the placed hearth.
        for (_, founding) in engine.world.query::<&Founding>().iter() {
            assert_eq!(founding.site, spot);
        }
        // No daughter forms here (no SettlementSystem), so one day later it is
        // cancelled and the hearth retired.
        run(
            &mut engine,
            &mut system,
            check + 2 * 1440 + 1,
            check + 3 * 1440 + 2,
        );
        assert_eq!(engine.resources.tallies.fissions_cancelled, 1);
        assert!(engine.resources.retired_hearths.contains(&spot));
    }

    #[test]
    fn harness_remnant_merges() {
        let seed = 3;
        let (mut engine, _) = world(seed, 10, &village(30));
        // A remnant of two members (settlement 2) and an empty settlement (3).
        let mut remnant = Settlement::new_with_id(2, 0);
        remnant.formation_tile = (50, 50);
        let mut empty = Settlement::new_with_id(3, 0);
        empty.formation_tile = (10, 50);
        for id in [101u64, 102] {
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
        assert!(
            !engine.resources.settlements.contains_key(&3),
            "an empty settlement dissolves at once"
        );
        assert!(engine.resources.retired_hearths.contains(&(10, 50)));
        assert!(
            engine.resources.settlements.contains_key(&2),
            "a remnant waits 24 hours"
        );
        run(&mut engine, &mut system, start + 61, start + 1440 + 61);
        assert!(
            !engine.resources.settlements.contains_key(&2),
            "merged away"
        );
        assert!(engine.resources.retired_hearths.contains(&(50, 50)));
        assert_eq!(engine.resources.tallies.remnant_merges, 1);
        for id in [101u64, 102] {
            assert!(
                engine
                    .resources
                    .settlements
                    .values()
                    .all(|s| !s.member_agents.contains(&id)),
                "agent {id} is a non-member"
            );
        }
        assert!(engine
            .resources
            .dissolutions
            .iter()
            .any(|(sid, _)| *sid == 2));
        assert_eq!(
            engine.resources.settlements[&1].member_agents.len(),
            30,
            "the village is untouched"
        );
    }

    #[test]
    fn harness_v17_regions_placed() {
        let distance = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
        let mut placed = 0;
        for seed in [0u64, 7, 31, 900_001, 900_017] {
            let engine = baseline_engine(seed);
            let r = &engine.resources;
            let v = r.world_variety.expect("variety");
            let hearth = *r
                .building_kinds
                .iter()
                .find(|(_, kind)| **kind == BuildingKind::Hearth)
                .map(|(tile, _)| tile)
                .expect("camp hearth");
            assert!(r.wild_regions.len() as u32 <= u32::from(v.wild_regions));
            assert_eq!(
                r.worldgen_report.regions_requested,
                u32::from(v.wild_regions)
            );
            assert_eq!(
                r.worldgen_report.regions_placed as usize,
                r.wild_regions.len()
            );
            let reach = r
                .rules
                .agent
                .forage
                .map_or(0, |f| f.forage_radius)
                .max(r.rules.society.firewood.map_or(0, |f| f.wood_radius));
            for (i, region) in r.wild_regions.iter().enumerate() {
                let c = region.centre;
                assert!(
                    (22..=28).contains(&distance(c, hearth)),
                    "seed {seed}: centre {c:?}"
                );
                assert!(!r.water_tiles.contains_key(&c));
                for other in &r.wild_regions[..i] {
                    assert!(distance(c, other.centre) >= 20);
                }
                assert!(region.bushes <= v.region_bushes && region.deadwood <= v.region_deadwood);
            }
            // Every region deadwood spot lies in some region's ring (2–6, or the
            // fill ring 7–9 of a short region, `region_fill`) and beyond the
            // camp's reach; every centre has water within 12 (`region_water_max`).
            for region in &r.wild_regions {
                assert!(region.water_distance <= 12, "seed {seed}: {region:?}");
            }
            for (tile, area) in &r.wood_spot_area {
                if *area > 0 {
                    let region = r.wild_regions[usize::from(*area) - 1];
                    assert!((2..=9).contains(&distance(*tile, region.centre)));
                    assert!(distance(*tile, hearth) > reach);
                }
            }
            // Deterministic and independent of the replicate.
            let mut forked = baseline_engine(seed);
            forked.apply_noise_fork(3);
            assert_eq!(forked.resources.wild_regions, r.wild_regions);
            assert_eq!(forked.resources.wood_spot_area, r.wood_spot_area);
            assert_eq!(baseline_engine(seed).resources.wild_regions, r.wild_regions);
            placed += r.wild_regions.len();
        }
        assert!(placed >= 5);
    }

    /// Review round 1, M4: the real founding path in a whole world — the
    /// group walks, the stamped hearth and shelters are registered, the formation
    /// scan forms the daughter, and on that tick the group is moved, the store
    /// delivered, the parent linked and the shelters owned by the daughter.
    #[test]
    fn harness_fission_founds_daughter_in_a_whole_world() {
        // V8-0: baseline world 8 splits at 06:00 of display day 28 and founds its
        // daughter on day 29 (re-measured; 30-day scan of seeds 0..48, where the
        // earliest split is day 28 — no seed splits before the first winter).
        let mut engine = baseline_engine(8);
        let mut departed = None;
        for _ in 0..34 * 1440 {
            let attempts = engine.resources.fission_log.len();
            engine.tick();
            if departed.is_none() && engine.resources.fission_log.len() > attempts {
                departed = Some(engine.current_tick());
            }
            if engine.resources.tallies.fissions == 1 {
                break;
            }
        }
        assert!(departed.is_some(), "a group departed");
        assert_eq!(
            engine.resources.tallies.fissions, 1,
            "a daughter was founded"
        );
        let record = engine.resources.fission_log[0].clone();
        let daughter_id = record.daughter.expect("daughter");
        let daughter = &engine.resources.settlements[&daughter_id];
        let parent = &engine.resources.settlements[&record.parent];
        assert_eq!(daughter.parent, Some(record.parent));
        // The hearth is registered through the building queue and is the daughter's anchor.
        assert_eq!(
            engine
                .resources
                .building_kinds
                .get(&daughter.formation_tile),
            Some(&BuildingKind::Hearth)
        );
        assert!(engine
            .resources
            .building_registry
            .values()
            .any(|p| *p == daughter.formation_tile));
        // Every living founder is a member of the daughter only, and no longer walks.
        let living: BTreeSet<AgentId> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
        for id in record.members.iter().filter(|id| living.contains(id)) {
            assert!(
                daughter.member_agents.contains(id) && !parent.member_agents.contains(id),
                "agent {id}"
            );
        }
        assert_eq!(engine.world.query::<&Founding>().iter().count(), 0);
        // The store is delivered (nothing could be spent on the founding tick:
        // FissionSystem runs last) and the two shelters are the daughter's.
        assert_eq!(
            daughter
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0),
            record.food_taken
        );
        assert_eq!(
            daughter
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0),
            record.wood_taken
        );
        let shelters = engine
            .resources
            .building_kinds
            .iter()
            .filter(|(tile, kind)| {
                **kind == BuildingKind::Shelter
                    && tile
                        .0
                        .abs_diff(daughter.formation_tile.0)
                        .max(tile.1.abs_diff(daughter.formation_tile.1))
                        <= 3
            })
            .map(|(tile, _)| *tile)
            .collect::<Vec<_>>();
        assert_eq!(shelters.len(), 2);
        for tile in shelters {
            assert_eq!(
                engine.resources.building_owner.get(&tile),
                Some(&daughter_id)
            );
        }
        // Sticky membership: the only moves between villages outside the
        // fission are residents the formation scan found standing at the new
        // hearth, and the tally counts exactly those. V8-0 re-measure, world 8:
        // one such resident (an adult of the camp, not in the group) — the
        // `game_v17` world 900 000 of Pass 11 had none.
        let extra = daughter
            .member_agents
            .iter()
            .filter(|id| !record.members.contains(id))
            .count() as u64;
        assert_eq!(engine.resources.tallies.exclusivity_moves, extra);
        assert!(
            extra <= 1,
            "{extra} residents joined the daughter by proximity"
        );
    }

    /// Review round 1, M5: under `FissionRule` a bush inside two settlements'
    /// forage radii counts one half in each settlement's capacity.
    #[test]
    fn harness_shared_bushes_count_once() {
        use sim_systems::runtime::settlement::food_economy::supply_per_day;
        let (mut engine, _) = world(5, 0, &village(4));
        for x in 30..40 {
            engine.resources.set_food_source_max(x, 20, 4);
        }
        let alone = supply_per_day(&engine.resources, 1, 16);
        let mut other = Settlement::new_with_id(2, 0);
        other.formation_tile = (44, 20);
        other.add_member_agent(99);
        engine.resources.settlements.insert(2, other);
        let shared = supply_per_day(&engine.resources, 1, 16);
        // Bushes x = 30..=36 lie within 16 of both (20, 20) and (44, 20); x = 37..=39 lie
        // beyond 16 of the first settlement, so they never enter its supply.
        assert!(
            (shared - alone / 2.0).abs() < 1e-9,
            "alone {alone}, shared {shared}"
        );
    }
}

// From `harness_revitalize_pass12_fission.rs`.
mod hazard {
    //! Pass 12 Part 2: fission as a daily hazard (PASS12_GOAL.md 7.1).
    //!
    //! V8-0: rebased from `game_v18` with the goal's start hazard on
    //! `WorldRules::baseline(seed)`, whose hazard is Pass 13's (`V19_HAZARD`: size
    //! 24..44, crowding weight 0, scarcity weight 1000 over 4 days, ceiling 300)
    //! and whose fission accumulates the hazard as pressure against a hidden
    //! threshold (Pass 13) instead of a daily draw: a logged check carries draw 0
    //! and splits when its pressure reaches its threshold.
    use sim_core::components::{
        Agent, AgentId, AgentState, Child, Elder, Lineage, Position, ResourceKind, Settlement,
    };
    use sim_core::fission_threshold::fission_threshold;
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::{FissionHazard, WorldRules};
    use sim_engine::{FissionCheck, RuntimeSystem, SimEngine, WildRegion};
    use sim_systems::runtime::settlement::fission::{hazard_terms, sustain_capacity};
    use sim_systems::runtime::settlement::food_economy::adult_need_per_day;
    use sim_systems::runtime::settlement::FissionSystem;

    const HOME: (u32, u32) = (20, 20);
    const SITE: (u32, u32) = (46, 20);

    /// The baseline fission hazard.
    fn baseline_hazard() -> FissionHazard {
        WorldRules::baseline(0)
            .society
            .fission
            .and_then(|f| f.hazard)
            .expect("baseline hazard")
    }

    /// The baseline rules of world `seed`.
    fn baseline_rules(seed: u64) -> WorldRules {
        WorldRules::baseline(seed)
    }

    /// Tick at which the display clock (480-tick offset) reads `minute` on the
    /// zero-based display day `day` (season = (day / 7) % 4).
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// A bare world under `rules`: one settlement (id 1) at `HOME` founded at tick
    /// 0 holding `food`, `bushes` capacity-4 bushes, one wild region at `SITE`,
    /// and `members` residents (ids 1..=n; every fifth a child, every seventh an
    /// elder) spawned in order, or in reverse order with `reverse`.
    fn world(
        rules: WorldRules,
        seed: u64,
        bushes: u32,
        food: u32,
        members: u64,
        reverse: bool,
    ) -> SimEngine {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = rules;
        engine.resources.world_seed = seed;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        settlement.store(ResourceKind::Food, food);
        for i in 0..bushes {
            engine
                .resources
                .set_food_source_max(10 + i % 20, 10 + i / 20, 4);
        }
        let mut ids: Vec<u64> = (1..=members).collect();
        if reverse {
            ids.reverse();
        }
        for id in ids {
            let entity = engine.world.spawn((
                Position::new(HOME.0, HOME.1),
                Agent { id: id as AgentId },
                AgentState::Idle,
                Lineage {
                    key: id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x55,
                    births: 0,
                },
            ));
            if id % 7 == 0 {
                engine.world.insert_one(entity, Elder).expect("elder");
            } else if id % 5 == 0 {
                engine
                    .world
                    .insert_one(entity, Child { born_tick: 0 })
                    .expect("child");
            }
            settlement.add_member_agent(id as AgentId);
        }
        engine.resources.settlements.insert(1, settlement);
        engine.resources.wild_regions.push(WildRegion {
            centre: SITE,
            bushes: 8,
            deadwood: 12,
            water_distance: 3,
        });
        engine
    }

    /// Run a fresh `FissionSystem` at every tick of [from, to).
    fn run(engine: &mut SimEngine, from: u64, to: u64) {
        let mut system = FissionSystem::new();
        for tick in from..to {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
    }

    /// Recompute a logged check's terms from its inputs (7.1's formula).
    fn recompute(engine: &SimEngine, c: &FissionCheck) -> (i64, i64, i64, i64) {
        let hazard = engine
            .resources
            .rules
            .society
            .fission
            .and_then(|f| f.hazard)
            .expect("hazard");
        hazard_terms(
            &hazard,
            c.members,
            c.capacity,
            c.food,
            adult_need_per_day(engine.resources.ticks_per_day),
        )
    }

    #[test]
    fn harness_hazard_formula() {
        let hz = baseline_hazard();
        assert_eq!(
            (
                hz.size_lo,
                hz.size_hi,
                hz.scarcity_days,
                hz.max_daily_permille
            ),
            (24, 44, 4, 300)
        );
        assert_eq!(
            (hz.weight_crowd_permille, hz.weight_scarcity_permille),
            (0, 1000)
        );
        // Pure terms (need per member 1.0), by hand: an ordinary case (g = 6/20,
        // c = 1000 so q = 150/200, s = 1 - 30/120, h = 300 x 0.3 x 0.75 = 67),
        // capacity 0 (counts as 1: q = 1000; crowding weighs 0 so h is unchanged),
        // an empty store (s = 1000, h = 90) and size at size_lo (h = 0).
        assert_eq!(hazard_terms(&hz, 30, 30, 30, 1.0), (300, 750, 750, 67));
        assert_eq!(hazard_terms(&hz, 30, 0, 30, 1.0), (300, 1000, 750, 67));
        assert_eq!(hazard_terms(&hz, 30, 30, 0, 1.0), (300, 750, 1000, 90));
        assert_eq!(hazard_terms(&hz, 24, 0, 0, 1.0), (0, 1000, 1000, 0));
        // Constructed settlements: the logged check is the formula on the
        // settlement's members, sustain capacity and own store; with pressure the
        // first check's pressure is its hazard, against the settlement's drawn
        // threshold, and no draw is made. Four cases: no bushes (capacity 0), an
        // empty store, a roomy full store, the smallest eligible village (28).
        let seed = 900_412;
        let check = tick_at(28, 360);
        let law = baseline_rules(seed)
            .society
            .fission
            .and_then(|f| f.pressure)
            .expect("baseline pressure")
            .threshold_table;
        for (bushes, food, members) in [
            (0u32, 100u32, 30u64),
            (40, 0, 30),
            (200, 2000, 30),
            (40, 100, 28),
        ] {
            let mut engine = world(baseline_rules(seed), seed, bushes, food, members, false);
            let capacity = sustain_capacity(&engine.resources, 1).expect("capacity");
            let resident_members = engine.resources.settlements[&1].member_agents.len() as u32;
            run(&mut engine, check, check + 1);
            let logged = engine.resources.fission_checks.clone();
            assert_eq!(logged.len(), 1, "case {bushes}/{food}/{members}");
            let c = logged[0];
            assert_eq!(
                (c.tick, c.settlement, c.members, c.capacity, c.food),
                (check, 1, resident_members, capacity, food)
            );
            assert_eq!((c.g, c.q, c.s, c.h), recompute(&engine, &c));
            assert_eq!(c.draw, 0, "pressure makes no draw");
            assert_eq!(c.pressure, c.h.max(0) as u64, "first day: P = h");
            assert_eq!(c.threshold, fission_threshold(law, seed, 1, 0));
            assert!(c.threshold > 0);
            assert_eq!(c.split, c.pressure > 0 && c.pressure >= c.threshold);
            assert!(c.site_free);
            assert_eq!(engine.resources.fission_log.len(), usize::from(c.split));
            if bushes == 0 {
                assert_eq!((capacity, c.q), (0, 1000));
            }
            if food == 0 {
                assert_eq!(c.s, 1000);
            }
        }
    }

    #[test]
    fn harness_hazard_rises_with_shortfall() {
        let hz = baseline_hazard();
        let mut last = i64::MAX;
        for food in (0..=200).step_by(10) {
            let h = hazard_terms(&hz, 30, 30, food, 1.0).3;
            assert!(h <= last, "h never rises as the store grows");
            last = h;
        }
        assert!(hazard_terms(&hz, 30, 30, 0, 1.0).3 > hazard_terms(&hz, 30, 30, 200, 1.0).3);
        // In a constructed settlement: the same village with an empty store has
        // s = 1000 and a higher hazard than with a full one.
        let seed = 900_413;
        let check = tick_at(28, 360);
        let hazard_with = |food: u32| {
            let mut engine = world(baseline_rules(seed), seed, 40, food, 30, false);
            run(&mut engine, check, check + 1);
            engine.resources.fission_checks[0]
        };
        let (empty, full) = (hazard_with(0), hazard_with(5000));
        assert_eq!((empty.s, full.s), (1000, 0));
        assert!(empty.h > full.h);
    }

    #[test]
    fn harness_allowed_seasons() {
        let seed = 900_414;
        // Baseline allowed seasons [spring, summer, autumn]: spring (day 28),
        // summer (day 35) and autumn (day 14) checks are logged; winter (day 21)
        // never is.
        assert_eq!(
            baseline_rules(seed)
                .society
                .fission
                .expect("fission")
                .allowed_seasons,
            [true, true, true, false]
        );
        for (day, logged) in [(14u64, true), (21, false), (28, true), (35, true)] {
            let mut engine = world(baseline_rules(seed), seed, 0, 0, 30, false);
            run(&mut engine, tick_at(day, 360), tick_at(day, 360) + 1);
            assert_eq!(
                !engine.resources.fission_checks.is_empty(),
                logged,
                "day {day}"
            );
        }
        // No allowed season: no check at all.
        let mut rules = baseline_rules(seed);
        if let Some(fission) = rules.society.fission.as_mut() {
            fission.allowed_seasons = [false; 4];
        }
        let mut engine = world(rules, seed, 0, 0, 30, false);
        run(&mut engine, tick_at(10, 0), tick_at(40, 0));
        assert!(
            engine.resources.fission_checks.is_empty() && engine.resources.fission_log.is_empty()
        );
    }
}

// From `harness_revitalize_pass13_fission.rs`.
mod pressure {
    //! Pass 13 Part 2: fission by accumulated pressure against a hidden threshold
    //! (PASS13_GOAL.md 7.1), under the live rules (`WorldRules::baseline`; the
    //! fission rule of `game_v19` is the baseline's, unchanged by v20–v26).
    use sim_core::components::{
        Agent, AgentId, AgentState, Child, Elder, Lineage, Position, ResourceKind, Settlement,
    };
    use sim_core::fission_threshold::{
        exp_quantile_permille, fission_threshold, threshold_draw, EXP_TABLE,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rng::NoiseFork;
    use sim_core::rules::{FissionHazard, FissionPressure, ThresholdLaw, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine, WildRegion};
    use sim_systems::runtime::settlement::fission::{hazard_terms, sustain_capacity};
    use sim_systems::runtime::settlement::food_economy::adult_need_per_day;
    use sim_systems::runtime::settlement::FissionSystem;
    use sim_test::baseline_engine;
    use sim_test::probe::fingerprint;

    const HOME: (u32, u32) = (20, 20);
    const SITE: (u32, u32) = (46, 20);

    /// The goal's start values (7.1): the mechanism tests use them explicitly so
    /// their numbers do not move with Part 3's tuning.
    const START_HAZARD: FissionHazard = FissionHazard {
        size_lo: 28,
        size_hi: 48,
        crowd_lo_permille: 850,
        crowd_hi_permille: 1050,
        scarcity_days: 2,
        weight_crowd_permille: 350,
        weight_scarcity_permille: 650,
        max_daily_permille: 200,
    };

    const START_PRESSURE: FissionPressure = FissionPressure {
        threshold_table: ThresholdLaw::Exponential {
            mean_permille: 1200,
        },
        decay_permille: 10,
    };

    /// `WorldRules::baseline(seed)` with the start hazard and pressure (7.1's
    /// start values; numeric parameters only) and the start `min_members` 32.
    fn start_rules(seed: u64) -> WorldRules {
        let mut rules = WorldRules::baseline(seed);
        if let Some(fission) = rules.society.fission.as_mut() {
            fission.hazard = Some(START_HAZARD);
            fission.pressure = Some(START_PRESSURE);
            fission.min_members = 32;
        }
        rules
    }

    /// Tick at which the display clock (480-tick offset) reads `minute` on the
    /// zero-based display day `day` (season = (day / 7) % 4).
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// A bare world under `rules`: settlement 1 at `HOME` founded at tick 0
    /// holding `food` with threshold `theta`, `bushes` capacity-4 bushes, one wild
    /// region at `SITE`, and `members` residents (ids 1..=n; every fifth a child,
    /// every seventh an elder) spawned in order, or in reverse order with `reverse`.
    fn world(
        rules: WorldRules,
        seed: u64,
        bushes: u32,
        food: u32,
        members: u64,
        theta: u64,
        reverse: bool,
    ) -> SimEngine {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = rules;
        engine.resources.world_seed = seed;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        settlement.fission_threshold = theta;
        settlement.store(ResourceKind::Food, food);
        for i in 0..bushes {
            engine
                .resources
                .set_food_source_max(10 + i % 20, 10 + i / 20, 4);
        }
        let mut ids: Vec<u64> = (1..=members).collect();
        if reverse {
            ids.reverse();
        }
        for id in ids {
            let entity = engine.world.spawn((
                Position::new(HOME.0, HOME.1),
                Agent { id: id as AgentId },
                AgentState::Idle,
                Lineage {
                    key: id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x55,
                    births: 0,
                },
            ));
            if id % 7 == 0 {
                engine.world.insert_one(entity, Elder).expect("elder");
            } else if id % 5 == 0 {
                engine
                    .world
                    .insert_one(entity, Child { born_tick: 0 })
                    .expect("child");
            }
            settlement.add_member_agent(id as AgentId);
        }
        engine.resources.settlements.insert(1, settlement);
        engine.resources.wild_regions.push(WildRegion {
            centre: SITE,
            bushes: 8,
            deadwood: 12,
            water_distance: 3,
        });
        engine
    }

    /// Run one `FissionSystem` at every tick of [from, to).
    fn run(engine: &mut SimEngine, system: &mut FissionSystem, from: u64, to: u64) {
        for tick in from..to {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
        }
    }

    #[test]
    fn harness_threshold_is_world_draw() {
        // The law the live rules use (whatever Part 3's tuning set).
        let law = WorldRules::baseline(7)
            .society
            .fission
            .and_then(|f| f.pressure)
            .expect("baseline pressure")
            .threshold_table;
        // The key never contains the replicate: replicate 0 and replicate 3 of a
        // far) both reach. V8-0: 14 days instead of 120 — baseline world 7 passes
        // every gate from display day 11 (10 days old, 28+ members), so each
        // replicate logs four checks of the camp before the first winter.
        let seed = 7;
        let (mut a, mut b) = (baseline_engine(seed), baseline_engine(seed));
        a.resources.noise_fork = Some(NoiseFork {
            tick: 1,
            replicate: 0,
        });
        b.resources.noise_fork = Some(NoiseFork {
            tick: 1,
            replicate: 3,
        });
        for _ in 0..14 * 1440 {
            a.tick();
            b.tick();
        }
        assert!(
            a.resources.fission_checks.len() >= 3 && b.resources.fission_checks.len() >= 3,
            "{} and {} checks",
            a.resources.fission_checks.len(),
            b.resources.fission_checks.len()
        );
        let thetas = |e: &SimEngine| -> std::collections::BTreeMap<(u32, u32), u64> {
            let mut k: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
            let mut seen = std::collections::BTreeMap::new();
            let mut splits = e
                .resources
                .fission_log
                .iter()
                .map(|f| (f.depart_tick, f.parent))
                .peekable();
            for c in &e.resources.fission_checks {
                while splits.peek().is_some_and(|(t, _)| *t < c.tick) {
                    let (_, parent) = splits.next().expect("peeked");
                    *k.entry(parent).or_insert(0) += 1;
                }
                let kk = k.get(&c.settlement).copied().unwrap_or(0);
                seen.insert((c.settlement, kk), c.threshold);
            }
            seen
        };
        let (ta, tb) = (thetas(&a), thetas(&b));
        assert!(
            !ta.is_empty() && !tb.is_empty(),
            "both replicates reach the check"
        );
        let mut shared = 0;
        for (key, theta) in &ta {
            // Every logged θ is the keyed world draw of (seed, sid, k).
            assert_eq!(
                *theta,
                fission_threshold(law, seed, u64::from(key.0), key.1),
                "{key:?}"
            );
            if let Some(other) = tb.get(key) {
                assert_eq!(theta, other, "replicates share θ of {key:?}");
                shared += 1;
            }
        }
        assert!(shared >= 1);
        for (key, theta) in &tb {
            assert_eq!(
                *theta,
                fission_threshold(law, seed, u64::from(key.0), key.1),
                "{key:?}"
            );
        }
        // θ of the camp differs across eight seeds.
        let camp: std::collections::BTreeSet<u64> =
            (0..8).map(|s| fission_threshold(law, s, 1, 0)).collect();
        assert!(camp.len() >= 7, "{camp:?}");
        // The camp of a whole world draws θ when it forms (k = 0).
        let mut engine = baseline_engine(3);
        for _ in 0..1440 {
            engine.tick();
        }
        let camp = engine.resources.settlements.get(&1).expect("camp");
        assert_eq!(camp.fission_threshold, fission_threshold(law, 3, 1, 0));
        assert!(camp.fission_threshold >= 1);
    }

    #[test]
    fn harness_threshold_table() {
        // The table is the generator's output (tools/revitalize/exp_table_p13.py at tag v7-final-sim):
        // E(i/1024) in thousandths, the top entry ln 2048.
        assert_eq!(EXP_TABLE.len(), 1025);
        for (i, want) in [
            (0usize, 0u32),
            (1, 1),
            (256, 288),
            (512, 693),
            (768, 1386),
            (1000, 3753),
            (1023, 6931),
            (1024, 7625),
        ] {
            assert_eq!(EXP_TABLE[i], want, "entry {i}");
        }
        // Mean of the interpolated quantile over a uniform u grid: 1000 ± 5.
        let n = 100_000u64;
        let mean = (0..n)
            .map(|j| exp_quantile_permille(j * 1_000_000 / n + 5))
            .sum::<u64>() as f64
            / n as f64;
        assert!((mean - 1000.0).abs() <= 5.0, "table mean {mean}");
        // 10,000 drawn θ (seeds × settlements × splits): mean within ± 3 % of the
        // law's mean, median within ± 3 % of 0.693 × mean.
        let law = ThresholdLaw::Exponential {
            mean_permille: 1200,
        };
        let mut drawn: Vec<u64> = (0..10_000u64)
            .map(|i| fission_threshold(law, 100 + i / 50, 1 + (i / 5) % 10, (i % 5) as u32))
            .collect();
        let mean = drawn.iter().sum::<u64>() as f64 / drawn.len() as f64;
        assert!((mean - 1200.0).abs() <= 36.0, "θ mean {mean}");
        drawn.sort_unstable();
        let median = (drawn[4999] + drawn[5000]) as f64 / 2.0;
        assert!(
            (median - 0.693 * 1200.0).abs() <= 0.03 * 0.693 * 1200.0,
            "θ median {median}"
        );
        // The uniform draw behind θ is replicate-free and uses the split count as its slot.
        assert_ne!(threshold_draw(5, 1, 0), threshold_draw(5, 1, 1));
        assert_ne!(threshold_draw(5, 1, 0), threshold_draw(5, 2, 0));
    }

    /// The h of the bare village (constant: nobody eats or ages in a bare world).
    fn constant_h(engine: &SimEngine) -> u64 {
        let s = &engine.resources.settlements[&1];
        let members = s.member_agents.len() as u32;
        let food = s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0);
        let capacity = sustain_capacity(&engine.resources, 1).expect("capacity");
        hazard_terms(
            &START_HAZARD,
            members,
            capacity,
            food,
            adult_need_per_day(engine.resources.ticks_per_day),
        )
        .3 as u64
    }

    #[test]
    fn harness_pressure_accumulates_and_splits() {
        let seed = 412;
        // 40 members, no bushes (capacity 0: q = 1000), a small store: h > 0 and constant.
        let probe = world(start_rules(seed), seed, 0, 20, 40, 1, false);
        let h = constant_h(&probe);
        assert!(h > 0);
        // θ = 4h - 1: P = h, 2h, 3h stay below it on days 28, 29, 30; day 31 (P = 4h) splits.
        let theta = 4 * h - 1;
        let mut engine = world(start_rules(seed), seed, 0, 20, 40, theta, false);
        let mut system = FissionSystem::new();
        let mut seen = Vec::new();
        for day in 28..32u64 {
            run(
                &mut engine,
                &mut system,
                tick_at(day, 0),
                tick_at(day + 1, 0),
            );
            let c = *engine
                .resources
                .fission_checks
                .last()
                .expect("a check every eligible morning");
            assert_eq!(c.tick, tick_at(day, 360));
            assert_eq!((c.h as u64, c.threshold, c.draw), (h, theta, 0));
            seen.push((day, c.pressure, c.split, engine.resources.fission_log.len()));
        }
        assert_eq!(
            seen,
            vec![
                (28, h, false, 0),
                (29, 2 * h, false, 0),
                (30, 3 * h, false, 0),
                (31, 4 * h, true, 1)
            ]
        );
        // The split resets P and redraws θ with k = 1.
        let record = engine.resources.fission_log[0].clone();
        assert_eq!(
            (record.pressure_at_split, record.threshold_at_split),
            (4 * h, theta)
        );
        let parent = &engine.resources.settlements[&1];
        assert_eq!(parent.fission_pressure, 0);
        assert_eq!(
            parent.fission_threshold,
            fission_threshold(START_PRESSURE.threshold_table, seed, 1, 1)
        );

        // Winter (a failing gate): P neither grows nor decays. Days 47–48 are
        // autumn, 49–55 winter, 56 spring again.
        let mut engine = world(start_rules(seed), seed, 0, 20, 40, u64::MAX, false);
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, tick_at(47, 0), tick_at(57, 0));
        let checks = engine.resources.fission_checks.clone();
        let days: Vec<u64> = checks.iter().map(|c| (c.tick + 480) / 1440).collect();
        assert_eq!(days, vec![47, 48, 56]);
        assert_eq!(
            checks.iter().map(|c| c.pressure).collect::<Vec<_>>(),
            vec![h, 2 * h, 3 * h]
        );

        // h = 0 (a village at size_lo): P decays by `decay_permille` a day, floor 0.
        let mut engine = world(start_rules(seed), seed, 0, 20, 28, u64::MAX, false);
        if let Some(s) = engine.resources.settlements.get_mut(&1) {
            s.fission_pressure = 25;
        }
        let mut rules = start_rules(seed);
        if let Some(f) = rules.society.fission.as_mut() {
            f.min_members = 20;
        }
        engine.resources.rules = rules;
        assert_eq!(constant_h(&engine), 0);
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, tick_at(28, 0), tick_at(32, 0));
        let ps: Vec<u64> = engine
            .resources
            .fission_checks
            .iter()
            .map(|c| c.pressure)
            .collect();
        assert_eq!(ps, vec![15, 5, 0, 0]);
        assert!(
            engine.resources.fission_log.is_empty(),
            "P = 0 never splits"
        );

        // A split blocked by no free site keeps P (and θ) and counts the block.
        let mut engine = world(start_rules(seed), seed, 0, 20, 40, h, false);
        engine.resources.wild_regions.clear();
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, tick_at(28, 0), tick_at(30, 0));
        assert!(engine.resources.fission_log.is_empty());
        assert_eq!(engine.resources.tallies.fission_blocked_no_site, 2);
        assert_eq!(engine.resources.settlements[&1].fission_pressure, 2 * h);
        assert_eq!(engine.resources.settlements[&1].fission_threshold, h);
    }

    #[test]
    fn harness_pressure_moves_with_food() {
        // A bare village of 40 whose store is four times what keeps s = 0 (food ≥
        // members × adult need × scarcity_days, 2 × 40 × need) against the same village with the store
        // removed at display day 20 (06:00, before the check): the removal raises
        // s from 0 to 1000, so h, and with it P, grows faster from that morning.
        // Eight seeds (thus eight θ): the split comes no later in all eight and
        // earlier in every seed whose split day differs.
        let need_full = (40.0 * adult_need_per_day(1440) * 2.0).floor() as u32;
        let mut earlier = 0;
        let mut no_later = 0;
        for seed in 0..8u64 {
            let theta = fission_threshold(START_PRESSURE.threshold_table, seed, 1, 0);
            let split_day = |remove: bool| {
                let mut engine =
                    world(start_rules(seed), seed, 40, 4 * need_full, 40, theta, false);
                let mut system = FissionSystem::new();
                for day in 14..48u64 {
                    if remove && day == 20 {
                        if let Some(s) = engine.resources.settlements.get_mut(&1) {
                            let held = s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0);
                            s.withdraw(ResourceKind::Food, held);
                        }
                    }
                    run(
                        &mut engine,
                        &mut system,
                        tick_at(day, 0),
                        tick_at(day + 1, 0),
                    );
                    if !engine.resources.fission_log.is_empty() {
                        return day;
                    }
                }
                u64::MAX
            };
            let (kept, removed) = (split_day(false), split_day(true));
            no_later += usize::from(removed <= kept);
            earlier += usize::from(removed < kept);
        }
        assert!(no_later >= 6, "no later in {no_later} of 8");
        assert!(earlier >= 3, "earlier in {earlier} of 8");
    }

    #[test]
    fn harness_pressure_order_independent() {
        let seed = 415;
        let theta = fission_threshold(START_PRESSURE.threshold_table, seed, 1, 0);
        let outcome = |reverse: bool| {
            let mut engine = world(start_rules(seed), seed, 0, 20, 40, theta, reverse);
            let mut system = FissionSystem::new();
            run(&mut engine, &mut system, tick_at(28, 0), tick_at(45, 0));
            (
                engine.resources.fission_checks.clone(),
                engine.resources.fission_log.clone(),
            )
        };
        let (checks, log) = outcome(false);
        assert!(!log.is_empty(), "the village splits");
        assert_eq!((checks.clone(), log.clone()), outcome(true));
        // Two villages inserted in either order: each one's checks and split day
        // are its own.
        let two = |reverse: bool| {
            let mut engine = world(start_rules(seed), seed, 0, 20, 40, theta, false);
            let mut second = Settlement::new_with_id(2, 0);
            second.formation_tile = (20, 50);
            second.fission_threshold =
                fission_threshold(START_PRESSURE.threshold_table, seed, 2, 0);
            for id in 101..=140u64 {
                engine.world.spawn((
                    Position::new(20, 50),
                    Agent { id: id as AgentId },
                    AgentState::Idle,
                    Lineage {
                        key: id ^ 0x77,
                        births: 0,
                    },
                ));
                second.add_member_agent(id as AgentId);
            }
            let first = engine.resources.settlements.remove(&1).expect("first");
            if reverse {
                engine.resources.settlements.insert(2, second);
                engine.resources.settlements.insert(1, first);
            } else {
                engine.resources.settlements.insert(1, first);
                engine.resources.settlements.insert(2, second);
            }
            engine.resources.wild_regions.push(WildRegion {
                centre: (50, 50),
                bushes: 8,
                deadwood: 12,
                water_distance: 3,
            });
            let mut system = FissionSystem::new();
            run(&mut engine, &mut system, tick_at(28, 0), tick_at(45, 0));
            (
                engine.resources.fission_checks.clone(),
                engine.resources.fission_log.clone(),
            )
        };
        assert_eq!(two(false), two(true));
        // A whole world: a twin run gives the same checks, splits and
        // fingerprint. V8-0: 14 days instead of 120 (world 9 logs its first
        // check on display day 11).
        let run_world = || {
            let mut engine = baseline_engine(9);
            for _ in 0..14 * 1440 {
                engine.tick();
            }
            engine
        };
        let (a, b) = (run_world(), run_world());
        assert!(a.resources.fission_checks.len() >= 3);
        assert_eq!(a.resources.fission_checks, b.resources.fission_checks);
        assert_eq!(a.resources.fission_log, b.resources.fission_log);
        assert_eq!(fingerprint(&a), fingerprint(&b));
    }
}
