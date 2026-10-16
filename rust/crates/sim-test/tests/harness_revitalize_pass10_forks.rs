//! Pass 10 (H9): Pass 9's fork properties hold for `game_v16` — replicate 0 is
//! the unforked run, a fork leaves the prefix untouched and changes the run
//! after it, newborn lineage keys come from first parents, and the sequential
//! movement streams never advance.
use std::collections::BTreeSet;

use sim_core::components::{Agent, Lineage};
use sim_core::variety::WorldVariety;
use sim_engine::{apply_noise_fork, NoiseFork, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_test::scenes::{game_v16_scene, game_v18_nofission_scene, game_v18_scene};
use sim_test::worldmodel::digest::state_digest;

/// `state_digest` every 60 ticks over `0..=ticks`, with an optional fork
/// `(tick, replicate, set directly)` applied before that tick.
fn digests(mut engine: SimEngine, ticks: u64, fork: Option<(u64, u32, bool)>) -> Vec<u64> {
    let mut out = Vec::new();
    for t in 0..=ticks {
        if t % 60 == 0 {
            out.push(state_digest(&engine));
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
fn harness_v16_noise_fork_replicate0_is_identity() {
    let base = digests(game_v16_scene(12_000), 4 * 1440, None);
    let fork = digests(game_v16_scene(12_000), 4 * 1440, Some((1440, 0, true)));
    assert_eq!(base, fork);
}

#[test]
fn harness_v16_noise_fork_prefix_identity() {
    let fork_tick = 1440;
    let base = digests(game_v16_scene(12_001), 4 * 1440, None);
    let fork = digests(game_v16_scene(12_001), 4 * 1440, Some((fork_tick, 3, false)));
    let prefix = (fork_tick / 60) as usize;
    assert_eq!(base[..=prefix], fork[..=prefix]);
    assert!(base[prefix + 1..].iter().zip(&fork[prefix + 1..]).any(|(a, b)| a != b));
}

/// Pass 12 J10: the same two fork properties for both v18 scenes.
#[test]
fn harness_v18_noise_fork_replicate0_and_prefix_identity() {
    for make in [game_v18_scene as fn(u64) -> SimEngine, game_v18_nofission_scene] {
        let base = digests(make(12_002), 4 * 1440, None);
        assert_eq!(base, digests(make(12_002), 4 * 1440, Some((1440, 0, true))));
        let fork = digests(make(12_002), 4 * 1440, Some((1440, 3, false)));
        let prefix = 1440 / 60;
        assert_eq!(base[..=prefix], fork[..=prefix]);
        assert!(base[prefix + 1..].iter().zip(&fork[prefix + 1..]).any(|(a, b)| a != b));
    }
}

#[test]
fn harness_v16_newborns_carry_their_first_parents_key() {
    let seed = 1;
    let founders = WorldVariety::draw(seed).starting_agents;
    let mut engine = game_v16_scene(seed);
    let before: Vec<(u64, MovementRng)> = {
        let mut rows: Vec<_> = engine.world.query::<(&Agent, &MovementRng)>().iter().map(|(_, (a, r))| (a.id, *r)).collect();
        rows.sort_by_key(|r| r.0);
        rows
    };
    for _ in 0..40_320 {
        engine.tick();
    }
    let rows: Vec<Lineage> = engine.world.query::<&Lineage>().iter().map(|(_, l)| *l).collect();
    let keys: BTreeSet<u64> = rows.iter().map(|l| l.key).collect();
    assert_eq!(keys.len(), rows.len());
    assert_eq!(rows.len(), engine.world.query::<&Agent>().iter().count(), "every resident carries a lineage");
    assert!(engine.resources.tallies.births > 0);
    let mut closure = BTreeSet::new();
    let mut generation: Vec<Lineage> = (0..founders).map(|i| Lineage::founder(seed, i)).collect();
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
    for key in &keys {
        assert!(closure.contains(key), "key {key:#x} is neither a founder's nor a first parent's child");
    }
    // The sequential movement streams stay attached and never advance (keyed draws).
    for (id, rng) in before {
        let now = engine.world.query::<(&Agent, &MovementRng)>().iter().find(|(_, (a, _))| a.id == id).map(|(_, (_, r))| *r);
        if let Some(now) = now {
            assert_eq!(now, rng, "agent {id}'s sequential stream advanced");
        }
    }
}
