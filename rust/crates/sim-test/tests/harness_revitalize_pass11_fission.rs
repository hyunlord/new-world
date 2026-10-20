//! Pass 11 Part 2: wild regions and village fission (`game_v17`).
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
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v16_scene, game_v17_nofission_scene, game_v17_scene};

const HOME: (u32, u32) = (20, 20);
const SITE: (u32, u32) = (46, 20);

/// Tick at which the display clock (480-tick offset) reads `minute` on display day `day`.
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// One resident: id, stage (0 child, 1 adult, 2 elder), lineage key.
type Resident = (AgentId, u8, u64);

/// A bare `game_v17(seed)` world: one settlement (id 1) at `HOME` founded at
/// tick 0 holding 100 food and 30 wood, `bushes` capacity-4 bushes around it,
/// one wild region centred on `SITE`, and the given residents spawned in order.
fn world(seed: u64, bushes: u32, residents: &[Resident]) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v17(seed);
    engine.resources.world_seed = seed;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
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
fn harness_fission_inert_before_v17() {
    for engine in [game_v16_scene(3), game_v17_nofission_scene(3)] {
        let mut engine = engine;
        for _ in 0..40 * 1440 {
            engine.tick();
        }
        let t = engine.resources.tallies;
        assert_eq!(
            (
                t.fissions,
                t.fissions_cancelled,
                t.fission_blocked_no_site,
                t.remnant_merges
            ),
            (0, 0, 0, 0)
        );
        assert!(engine.resources.fission_log.is_empty());
        assert_eq!(engine.world.query::<&Founding>().iter().count(), 0);
        assert!(engine.resources.retired_hearths.is_empty());
        assert_eq!(engine.resources.settlements.len(), 1);
    }
    for rules in [
        WorldRules::game_v16(3),
        WorldRules::game_v16_neutral(),
        WorldRules::game_v17_nofission(3),
    ] {
        assert_eq!(rules.society.fission, None);
    }
    // The v16 world has no regions; v17's worlds share their camp with v16.
    assert!(game_v16_scene(3).resources.wild_regions.is_empty());
    let (v16, v17) = (game_v16_scene(5), game_v17_scene(5));
    assert!(!v17.resources.wild_regions.is_empty());
    for (tile, kind) in &v16.resources.building_kinds {
        assert_eq!(v17.resources.building_kinds.get(tile), Some(kind));
    }
}

#[test]
fn harness_fission_only_when_crowded() {
    let seed = 7;
    let bushes = bushes_for(seed, 30);
    let cap = {
        let (engine, _) = world(seed, bushes, &[]);
        sustain_capacity(&engine.resources, 1).expect("capacity")
    };
    let rule = WorldRules::game_v17(seed).society.fission.expect("rule");
    // Below crowding (and below min_members): never splits.
    let below =
        ((rule.crowding * f64::from(cap)).ceil() as u64 - 1).min(u64::from(rule.min_members) - 1);
    let (mut engine, _) = world(seed, bushes, &village(below));
    let mut system = FissionSystem::new();
    run(&mut engine, &mut system, tick_at(14, 0), tick_at(45, 0));
    assert!(
        engine.resources.fission_log.is_empty(),
        "{below} members of capacity {cap} split"
    );
    // Above crowding: one split, at 06:00 of the first spring or summer day
    // after the minimum age (day 14 is autumn, 21 winter, 28 spring).
    let above = u64::from(cap.max(rule.min_members)) + 2;
    let (mut engine, _) = world(seed, bushes, &village(above));
    let mut system = FissionSystem::new();
    run(&mut engine, &mut system, tick_at(14, 0), tick_at(45, 0));
    let log = &engine.resources.fission_log;
    assert_eq!(log.len(), 1, "one fission (cooldown 28 days)");
    assert_eq!(log[0].depart_tick, tick_at(28, 360));
    assert_eq!(log[0].site, SITE);
    assert_eq!(log[0].parent, 1);
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
fn harness_fission_order_independent() {
    let seed = 900_321;
    let bushes = bushes_for(seed, 30);
    let check = tick_at(28, 360);
    let people = village(33);
    let mut reversed = people.clone();
    reversed.reverse();
    let outcome = |residents: &[Resident]| {
        let (mut engine, _) = world(seed, bushes, residents);
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, check, check + 1);
        (engine.resources.fission_log.clone(), group_of(&engine))
    };
    assert_eq!(outcome(&people), outcome(&reversed));
    // A whole v17 world: two runs of the same seed agree tick for tick.
    let (mut a, mut b) = (game_v17_scene(900_000), game_v17_scene(900_000));
    for _ in 0..30 * 1440 {
        a.tick();
        b.tick();
    }
    assert_eq!(fingerprint(&a), fingerprint(&b));
    assert_eq!(a.resources.fission_log, b.resources.fission_log);
}

#[test]
fn harness_v17_regions_placed() {
    let distance = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
    let mut placed = 0;
    for seed in [0u64, 7, 31, 900_001, 900_017] {
        let engine = game_v17_scene(seed);
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
        // Every region bush and deadwood spot lies in some region's ring and beyond the camp's reach.
        for (tile, area) in &r.wood_spot_area {
            if *area > 0 {
                let region = r.wild_regions[usize::from(*area) - 1];
                assert!((2..=6).contains(&distance(*tile, region.centre)));
                assert!(distance(*tile, hearth) > reach);
            }
        }
        // Deterministic and independent of the replicate.
        let mut forked = game_v17_scene(seed);
        forked.apply_noise_fork(3);
        assert_eq!(forked.resources.wild_regions, r.wild_regions);
        assert_eq!(forked.resources.wood_spot_area, r.wood_spot_area);
        assert_eq!(game_v17_scene(seed).resources.wild_regions, r.wild_regions);
        // The camp of the v17 world is the v16 world's (same bushes and deadwood near it).
        let v16 = game_v16_scene(seed);
        for (tile, max) in &v16.resources.wood_source_max {
            assert_eq!(r.wood_source_max.get(tile), Some(max));
            assert_eq!(r.wood_spot_area.get(tile), Some(&0));
        }
        for (tile, max) in &v16.resources.food_source_max {
            assert_eq!(r.food_source_max.get(tile), Some(max));
        }
        placed += r.wild_regions.len();
    }
    assert!(placed >= 5);
}

/// Review round 1, M4: the real founding path in a whole v17 world — the
/// group walks, the stamped hearth and shelters are registered, the formation
/// scan forms the daughter, and on that tick the group is moved, the store
/// delivered, the parent linked and the shelters owned by the daughter.
#[test]
fn harness_fission_founds_daughter_in_a_whole_world() {
    let mut engine = game_v17_scene(900_000);
    let mut departed = None;
    for _ in 0..40 * 1440 {
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
    // Sticky membership: nobody else moved between the villages.
    assert_eq!(engine.resources.tallies.exclusivity_moves, 0);
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
    // Without the fission rule nothing is shared (v16 accounting).
    engine.resources.rules = WorldRules::game_v16(5);
    assert!((supply_per_day(&engine.resources, 1, 16) - alone).abs() < 1e-9);
}
