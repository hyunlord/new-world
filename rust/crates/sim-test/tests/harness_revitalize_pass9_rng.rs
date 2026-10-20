//! Pass 9: keyed draws, lineage keys and noise forks (sections 6.3 and 7.4),
//! on the baseline world (keyed draws on).
use std::collections::BTreeSet;

use sim_core::components::{Agent, Lineage};
use sim_core::variety::WorldVariety;
use sim_test::baseline_engine;

#[test]
fn harness_lineage_keys_stable_under_birth_shift() {
    // Two parents P and Q (two settlements, or a dead founder: the first
    // parents differ). Removing Q's child leaves every child of P unchanged.
    let seed = 12_000;
    let births = |with_q: bool| {
        let (mut p, mut q) = (Lineage::founder(seed, 0), Lineage::founder(seed, 1));
        let mut keys = Vec::new();
        for step in 0..12 {
            if step == 3 && with_q {
                keys.push(("q", q.child(seed).key));
            }
            keys.push(("p", p.child(seed).key));
        }
        keys
    };
    let with: Vec<u64> = births(true)
        .into_iter()
        .filter(|(who, _)| *who == "p")
        .map(|(_, k)| k)
        .collect();
    let without: Vec<u64> = births(false).into_iter().map(|(_, k)| k).collect();
    assert_eq!(with, without);
    // A birth of the same parent shifts its later children (the known limitation).
    let mut p = Lineage::founder(seed, 0);
    let first = p.child(seed);
    assert_ne!(p.child(seed).key, first.key);
}

#[test]
fn harness_lineage_founder_keys_follow_seed_and_spawn_index() {
    for seed in [0u64, 7, 12_031] {
        let engine = baseline_engine(seed);
        let mut founders: Vec<(u64, Lineage)> = engine
            .world
            .query::<(&Agent, &Lineage)>()
            .iter()
            .map(|(_, (a, l))| (a.id, *l))
            .collect();
        founders.sort_by_key(|row| row.0);
        assert_eq!(
            founders.len(),
            WorldVariety::draw_v18(seed).starting_agents as usize
        );
        for (index, (_, lineage)) in founders.iter().enumerate() {
            assert_eq!(
                *lineage,
                Lineage::founder(seed, index as u32),
                "seed {seed} founder {index}"
            );
        }
    }
}

#[test]
fn harness_lineage_keys_do_not_collide() {
    let seed = 3;
    let mut keys = BTreeSet::new();
    for index in 0..10_000u32 {
        assert!(keys.insert(Lineage::founder(seed, index).key));
    }
    for parent in 0..10u32 {
        let mut p = Lineage::founder(seed, parent);
        for _ in 0..1_000 {
            assert!(keys.insert(p.child(seed).key));
        }
    }
    assert_eq!(keys.len(), 20_000);
}

#[test]
fn harness_newborns_carry_their_first_parents_key() {
    // Every resident of a 28-day baseline run carries a lineage, keys are distinct,
    // and every newborn's key lies in the child closure of the founders' keys
    // (founder -> child -> grandchild), i.e. it came from a first parent and its
    // birth count, never from the settlement/tick fallback (review round 1, M1).
    let seed = 1;
    let mut engine = baseline_engine(seed);
    let founders = WorldVariety::draw_v18(seed).starting_agents;
    for _ in 0..40_320 {
        engine.tick();
    }
    let rows: Vec<Lineage> = engine
        .world
        .query::<&Lineage>()
        .iter()
        .map(|(_, l)| *l)
        .collect();
    let keys: BTreeSet<u64> = rows.iter().map(|l| l.key).collect();
    assert_eq!(keys.len(), rows.len());
    let alive = engine.world.query::<&Agent>().iter().count();
    assert_eq!(rows.len(), alive, "every resident carries a lineage");
    let births = engine.resources.tallies.births;
    // V8-0 re-measure (baseline seed 1): 23 founders, 15 births, 32 alive.
    assert!(births >= 8, "seed {seed}: {births} births in 28 days");
    let mut closure = BTreeSet::new();
    let mut generation: Vec<Lineage> = (0..founders).map(|i| Lineage::founder(seed, i)).collect();
    // Three levels (founders, 400 children each, 40 grandchildren per child):
    // a 28-day village has ~15-26 births, at most a few per mother (review round 2).
    for depth in 0..2 {
        let mut next = Vec::new();
        for parent in &generation {
            closure.insert(parent.key);
            let mut p = *parent;
            for _ in 0..if depth == 0 { 400 } else { 40 } {
                next.push(p.child(seed));
            }
        }
        generation = next;
    }
    closure.extend(generation.iter().map(|l| l.key));
    let newborns: Vec<u64> = keys
        .iter()
        .copied()
        .filter(|k| !(0..founders).any(|i| Lineage::founder(seed, i).key == *k))
        .collect();
    assert!(!newborns.is_empty());
    for key in newborns {
        assert!(
            closure.contains(&key),
            "newborn key {key:#x} is not a first parent's child"
        );
    }
}

// ---------------------------------------------------------------- section 7.4 ----

use sim_core::components::{AgentState, Position, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_core::rng::{
    keyed_step, keyed_u64, keyed_unit_f64, STREAM_MOVE_DX, STREAM_MOVE_DY, STREAM_MOVE_PAUSE,
};
use sim_core::rules::WorldRules;
use sim_engine::{apply_noise_fork, NoiseFork, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_test::probe::fingerprint;

#[test]
fn harness_keyed_draw_is_pure() {
    let key = (
        12_000u64,
        STREAM_MOVE_PAUSE,
        0xABCD_u64,
        28_800u64,
        0u32,
        3u32,
    );
    let value = keyed_u64(key.0, key.1, key.2, key.3, key.4, key.5);
    assert_eq!(value, keyed_u64(key.0, key.1, key.2, key.3, key.4, key.5));
    for other in [
        keyed_u64(key.0 + 1, key.1, key.2, key.3, key.4, key.5),
        keyed_u64(key.0, STREAM_MOVE_DX, key.2, key.3, key.4, key.5),
        keyed_u64(key.0, key.1, key.2 + 1, key.3, key.4, key.5),
        keyed_u64(key.0, key.1, key.2, key.3 + 1, key.4, key.5),
        keyed_u64(key.0, key.1, key.2, key.3, key.4 + 1, key.5),
        keyed_u64(key.0, key.1, key.2, key.3, key.4, key.5 + 1),
    ] {
        assert_ne!(value, other);
    }
    // ~uniform: each of 16 buckets of 10^6 draws within +-0.01 of 1/16.
    let mut buckets = [0u32; 16];
    for tick in 0..1_000_000u64 {
        let u = keyed_unit_f64(7, STREAM_MOVE_PAUSE, 99, tick, 0, 0);
        buckets[(u * 16.0) as usize] += 1;
    }
    for (i, count) in buckets.iter().enumerate() {
        let share = f64::from(*count) / 1e6;
        assert!((share - 1.0 / 16.0).abs() <= 0.01, "bucket {i}: {share}");
    }
}

#[test]
fn harness_keyed_step_balanced() {
    // Each step value within +-0.01 of 1/3, and the nine (dx, dy) cells within
    // +-0.01 of 1/9 (the two streams do not move together).
    let mut cells = [[0u32; 3]; 3];
    for tick in 0..1_000_000u64 {
        let dx = keyed_step(5, STREAM_MOVE_DX, 41, tick, 0, 0);
        let dy = keyed_step(5, STREAM_MOVE_DY, 41, tick, 0, 0);
        cells[(dx + 1) as usize][(dy + 1) as usize] += 1;
    }
    let share = |count: u32| f64::from(count) / 1e6;
    for (x, row) in cells.iter().enumerate() {
        assert!(
            (share(row.iter().sum()) - 1.0 / 3.0).abs() <= 0.01,
            "dx {x}"
        );
        for (y, cell) in row.iter().enumerate() {
            assert!((share(*cell) - 1.0 / 9.0).abs() <= 0.01, "cell {x},{y}");
        }
    }
    for y in 0..3 {
        let column: u32 = cells.iter().map(|row| row[y]).sum();
        assert!((share(column) - 1.0 / 3.0).abs() <= 0.01, "dy {y}");
    }
}

/// One agent on an open 64 x 64 grid under default rules plus the pause chance
/// and keyed draws; it is busy (Consuming, frozen) for `busy` ticks,
/// then idle. Returns its per-tick displacement from tick 100 to 400.
fn steps(busy: u64) -> Vec<(i64, i64)> {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.world_seed = 12_000;
    engine.resources.rules = WorldRules::default();
    engine.resources.rules.agent.idle_pause_chance = 0.4;
    engine.resources.rules.agent.keyed_draws = true;
    engine.register_system(Box::new(AgentMovementSystem::new()));
    let state = if busy > 0 {
        AgentState::Consuming {
            target: TargetKind::Food,
        }
    } else {
        AgentState::Idle
    };
    let agent = engine.world.spawn((
        Position::new(32, 32),
        MovementRng::new(77),
        state,
        Agent { id: 0 },
        Lineage::founder(12_000, 0),
    ));
    let mut out = Vec::new();
    for tick in 0..400u64 {
        if tick == busy {
            let _ = engine.world.insert_one(agent, AgentState::Idle);
        }
        // Keep the agent away from the border so no step is clamped.
        let _ = engine.world.insert_one(agent, Position::new(32, 32));
        engine.tick();
        if tick >= 100 {
            let pos = *engine.world.get::<&Position>(agent).expect("agent");
            out.push((i64::from(pos.x) - 32, i64::from(pos.y) - 32));
        }
    }
    out
}

#[test]
fn harness_movement_independent_of_history() {
    // Keyed: the step at every tick from 100 on is the same whether the agent
    // was idle all along or busy for its first 50 ticks.
    let idle = steps(0);
    assert_eq!(idle, steps(50));
    // ... and the steps are real draws, not a frozen agent: with pause chance
    // 0.4 and (0, 0) one ninth of the cells, about 0.6 x 8/9 = 0.53 of the 300
    // ticks move (V8-0 measure: 147 of 300).
    let moved = idle.iter().filter(|&&d| d != (0, 0)).count();
    assert!((120..=200).contains(&moved), "{moved} of 300 ticks moved");
    // In a baseline world the movement component is attached but never advanced.
    let mut engine = baseline_engine(2);
    let before: Vec<(u64, MovementRng)> = {
        let mut rows: Vec<_> = engine
            .world
            .query::<(&Agent, &MovementRng)>()
            .iter()
            .map(|(_, (a, r))| (a.id, *r))
            .collect();
        rows.sort_by_key(|r| r.0);
        rows
    };
    for _ in 0..1440 {
        engine.tick();
    }
    for (id, rng) in before {
        let now = engine
            .world
            .query::<(&Agent, &MovementRng)>()
            .iter()
            .find(|(_, (a, _))| a.id == id)
            .map(|(_, (_, r))| *r);
        if let Some(now) = now {
            assert_eq!(now, rng, "agent {id}'s sequential stream advanced");
        }
    }
}

/// The probe `fingerprint` after every completed tick in `0..=ticks`, sampled every 60.
fn digests(mut engine: SimEngine, ticks: u64, fork: Option<(u64, u32, bool)>) -> Vec<String> {
    let mut out = Vec::new();
    for t in 0..=ticks {
        if t % 60 == 0 {
            out.push(fingerprint(&engine));
        }
        if t == ticks {
            break;
        }
        if let Some((tick, replicate, direct)) = fork {
            if t == tick {
                if direct {
                    engine.resources.noise_fork = Some(NoiseFork { tick, replicate });
                } else {
                    apply_noise_fork(&mut engine, replicate);
                }
            }
        }
        engine.tick();
    }
    out
}

#[test]
fn harness_noise_fork_replicate0_is_identity() {
    // Replicate 0 set directly (not through apply_noise_fork, which returns early).
    let base = digests(baseline_engine(12_000), 4 * 1440, None);
    let fork = digests(baseline_engine(12_000), 4 * 1440, Some((1440, 0, true)));
    assert_eq!(base, fork);
}

#[test]
fn harness_noise_fork_prefix_identity() {
    let fork_tick = 1440;
    let base = digests(baseline_engine(12_001), 4 * 1440, None);
    let fork = digests(
        baseline_engine(12_001),
        4 * 1440,
        Some((fork_tick, 3, false)),
    );
    let prefix = (fork_tick / 60) as usize;
    // Rows at every 60th tick <= T are equal (row T is still pre-fork) ...
    assert_eq!(base[..=prefix], fork[..=prefix]);
    // ... and the runs differ at some later row.
    assert!(base[prefix + 1..]
        .iter()
        .zip(&fork[prefix + 1..])
        .any(|(a, b)| a != b));
}
